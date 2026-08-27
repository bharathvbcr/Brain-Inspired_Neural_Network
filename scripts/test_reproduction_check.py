#!/usr/bin/env python3
"""Tests for `scripts/aws/check_reproduction.py`.

The tool's whole value is that it FAILS when the fleet stops reproducing its own
record. A reproduction check that cannot fail is worse than none: it turns
"nobody looked" into "we checked and it was fine".
"""

from __future__ import annotations

import contextlib
import importlib.util
import io
import json
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))


def load(name: str, relative: str):
    spec = importlib.util.spec_from_file_location(name, ROOT / relative)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


CR = load("check_reproduction", "scripts/aws/check_reproduction.py")
XISA = load("cross_isa_reproduction", "scripts/cross_isa_reproduction.py")


def a_cell(**over):
    cell = {
        "arm": "ff+fixed+attn", "contract": "published-2ms",
        "geometry": "adjacent-sum-5", "hidden": 128, "epochs": 400,
        "surrogate_scale": 1.0, "temporal_condition": "intact",
        "clip_grad_norm": None,
        "accuracy": 0.5, "mean_loss": 1.25, "classes_predicted": 20,
        "epoch_mean_loss": [1.0, 0.5], "wall_secs": 111.0,
    }
    cell.update(over)
    return cell


class SweepTest(unittest.TestCase):
    def setUp(self):
        self.dir = Path(tempfile.mkdtemp())

    def write(self, name, cell):
        (self.dir / f"{name}.json").write_text(json.dumps(cell))

    def run_tool(self, *extra):
        buf = io.StringIO()
        argv = ["check_reproduction.py", "--archive", str(self.dir), *extra]
        old, sys.argv = sys.argv, argv
        try:
            with contextlib.redirect_stdout(buf):
                code = CR.main()
        finally:
            sys.argv = old
        return code, buf.getvalue()

    def pair(self, left=None, right=None, config="d32l4"):
        self.write(f"w1__ff-fixed-attn__h128__{config}__s1", left or a_cell())
        self.write(f"w2__ff-fixed-attn__h128__{config}__s1", right or a_cell())

    def test_matching_duplicates_pass(self):
        self.pair()
        code, out = self.run_tool()
        self.assertEqual(code, 0, out)
        self.assertIn("BYTE-IDENTICAL", out)

    def test_one_perturbed_digit_fails(self):
        """The negative case this whole file exists for."""
        self.pair(right=a_cell(accuracy=0.5000001))
        code, out = self.run_tool()
        self.assertEqual(code, 1, out)
        self.assertIn("REPRODUCTION FAILED", out)
        self.assertIn("accuracy", out)

    def test_a_perturbation_inside_a_trajectory_is_caught(self):
        """Per-epoch arrays are most of the evidence; comparing only scalars
        would pass a cell whose whole training curve moved."""
        self.pair(right=a_cell(epoch_mean_loss=[1.0, 0.5000001]))
        self.assertEqual(self.run_tool()[0], 1)

    def test_an_int_does_not_reproduce_a_float(self):
        self.pair(left=a_cell(classes_predicted=20),
                  right=a_cell(classes_predicted=20.0))
        self.assertEqual(self.run_tool()[0], 1)

    def test_a_missing_field_is_a_disagreement_not_a_skip(self):
        thin = a_cell()
        del thin["mean_loss"]
        self.pair(right=thin)
        code, out = self.run_tool()
        self.assertEqual(code, 1, out)
        self.assertIn("present in one cell only", out)

    def test_wall_secs_alone_is_not_a_reproduction_failure(self):
        """A timing is not a measurement. The 4-thread and 16-thread halves of
        the fleet differ on it by design."""
        self.pair(right=a_cell(wall_secs=999.0))
        self.assertEqual(self.run_tool()[0], 0)

    def test_different_configurations_are_never_compared(self):
        """h128 against h256 is not a reproduction failure. It is not a pair."""
        self.write("w1__ff-fixed-attn__h128__d32l4__s1", a_cell(accuracy=0.1))
        self.write("w2__ff-fixed-attn__h256__d32l4__s1",
                   a_cell(hidden=256, accuracy=0.9))
        code, out = self.run_tool()
        self.assertEqual(code, 2, out)

    def test_two_readout_depths_are_never_compared(self):
        """H17-2 merged a d32l1 control into a d32l4 comparison. The depth comes
        from the filename, so a checker that ignored it would repeat that."""
        self.write("w1__ff-fixed-attn__h128__d32l1__s1", a_cell(accuracy=0.1))
        self.write("w2__ff-fixed-attn__h128__d32l4__s1", a_cell(accuracy=0.9))
        self.assertEqual(self.run_tool()[0], 2)

    def test_different_seeds_are_never_compared(self):
        self.write("w1__ff-fixed-attn__h128__d32l4__s1", a_cell(accuracy=0.1))
        self.write("w2__ff-fixed-attn__h128__d32l4__s2", a_cell(accuracy=0.9))
        self.assertEqual(self.run_tool()[0], 2)

    def test_one_wave_twice_in_two_roots_is_not_a_duplicate_run(self):
        """The same cell mirrored into two archives is one cell."""
        second = Path(tempfile.mkdtemp())
        name = "w1__ff-fixed-attn__h128__d32l4__s1.json"
        (self.dir / name).write_text(json.dumps(a_cell()))
        (second / name).write_text(json.dumps(a_cell(accuracy=0.9)))
        buf = io.StringIO()
        old, sys.argv = sys.argv, [
            "x", "--archive", str(self.dir), "--archive", str(second)]
        try:
            with contextlib.redirect_stdout(buf):
                code = CR.main()
        finally:
            sys.argv = old
        self.assertEqual(code, 2, buf.getvalue())

    def test_nothing_to_compare_is_not_a_pass(self):
        """Exit 2, distinct from both 0 and 1.

        `release_dead_claims` computed `held - done` and so called 22 finished
        failures orphans, because "no result" and "never ran" shared a code
        path. The same conflation here would report a clean sweep over nothing.
        """
        code, out = self.run_tool()
        self.assertEqual(code, 2)
        self.assertIn("NOT a pass", out)

    def test_a_pass_disclaims_h18_4(self):
        self.pair()
        self.assertIn("does NOT discharge H18-4", self.run_tool()[1])

    def test_sidecars_are_skipped_without_being_called_unreadable(self):
        self.write("manifest", {"anything": 1})
        self.write("plan_w15_17", [{"id": "x"}])
        self.pair()
        code, out = self.run_tool()
        self.assertEqual(code, 0, out)
        self.assertIn("sidecars skipped: 2", out)
        self.assertNotIn("UNREADABLE", out)

    def test_a_file_that_is_not_a_cell_is_still_reported(self):
        (self.dir / "w9__nonsense__s1.json").write_text('{"no": "accuracy"}')
        self.pair()
        _, out = self.run_tool()
        self.assertIn("UNREADABLE", out)


class SharedDefinitionsTest(unittest.TestCase):
    """The two reproduction checks must not drift apart.

    `cross_isa_reproduction` asks whether two machines agree;
    `check_reproduction` asks whether one fleet still agrees with its own
    record. They are different questions over one definition of "the same
    experiment" and one of "identical", and this pins that it stays one.
    """

    def test_both_helpers_are_imported_rather_than_redefined(self):
        """Loading the file twice gives two module objects, so identity is not
        the test. That the functions come from the other module is."""
        for name in ("compare_pair", "configuration"):
            with self.subTest(name):
                self.assertEqual(getattr(CR, name).__module__,
                                 "cross_isa_reproduction")

    def test_the_checker_defines_neither_itself(self):
        source = (ROOT / "scripts/aws/check_reproduction.py").read_text()
        for name in ("compare_pair", "configuration"):
            with self.subTest(name):
                self.assertNotIn(f"def {name}(", source)

    def test_the_cross_isa_loader_really_does_lose_within_corpus_duplicates(self):
        """The blind spot this file was written to cover, pinned.

        If `cross_isa_reproduction.load` ever starts keeping both cells, this
        test fails and the docstring's claim about why the two tools differ has
        to be rewritten rather than quietly becoming false.
        """
        d = Path(tempfile.mkdtemp())
        for wave in ("w8wid", "w18dep"):
            (d / f"{wave}__ff-fixed-attn__h128__d32l4__s1.json").write_text(
                json.dumps(a_cell()))
        index = XISA.load([d])
        kept = sum(len(seeds) for seeds in index.values())
        self.assertEqual(kept, 1, "cross_isa now keeps both; see the docstring")


class FrozenAnalyserFieldsTest(unittest.TestCase):
    """`analyse_wave15` and `analyse_wave18` each carry a `SCIENTIFIC_FIELDS`.

    They are frozen analysers, registered with their preregistrations before
    their first cell existed, so neither is edited to import a shared copy and
    no third definition is introduced for them to import. Their agreement is
    pinned here instead -- the same trade `AWS_TIMEOUT_S` makes across the
    `scripts/aws` helpers, for the same reason.
    """

    def test_the_two_frozen_copies_agree_with_each_other(self):
        a = load("analyse_wave15", "scripts/aws/analyse_wave15.py")
        b = load("analyse_wave18", "scripts/aws/analyse_wave18.py")
        self.assertEqual(tuple(a.SCIENTIFIC_FIELDS), tuple(b.SCIENTIFIC_FIELDS))

    def test_no_analyser_drops_a_field_gate_f_compares(self):
        """The analysers are deliberately STRICTER than Gate F -- they add the
        per-epoch trajectories and `tail_loss_improvement`, five fields Gate F
        does not compare. The invariant is therefore one-directional: an
        analyser may check more, never less."""
        import gate_f_rust
        # `n_train` and `n_test` are corpus sizes rather than measurements: a
        # cell that reported a different one would not be the same experiment,
        # which `configuration()` decides upstream of any field comparison.
        measurements = set(gate_f_rust.COMPARED_FIELDS) - {"n_train", "n_test"}
        for name in ("analyse_wave15", "analyse_wave18"):
            with self.subTest(name):
                fields = set(load(name, f"scripts/aws/{name}.py").SCIENTIFIC_FIELDS)
                self.assertEqual(measurements - fields, set())


if __name__ == "__main__":
    unittest.main(verbosity=2)
