#!/usr/bin/env python3
"""Tests for the waves 18-19 analyser, against synthetic grids.

The analyser is frozen before the first cell, so the only way to know it
computes what the preregistration says is to feed it grids whose answers are
known by construction. Four properties matter most and none would crash if
broken:

  * a **bar** that drifts from the prereg silently changes a verdict;
  * a **gate** that reports numbers beside a NOT EVALUABLE banner lets a
    blocked comparison be read anyway;
  * **H18-1's argmax** must be able to name an endpoint. An analyser that only
    asked "is L2 best?" would confirm the observation it was built from, so the
    endpoint outcomes are tested explicitly;
  * **H18-4 is destructive**, and a destructive check that cannot fire is worse
    than no check. Both of its directions are tested.

The bars are tested by mutation -- breaking the analyser and requiring the test
to notice -- rather than by reading the source, and the mutation is applied to a
copy so the test cannot corrupt the thing it guards.
"""

from __future__ import annotations

import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "scripts" / "aws"))
import analyse_wave18 as a18  # noqa: E402

ANCHOR = "published-2ms__adjacent-sum-5"
SEEDS = a18.SEEDS
SEEDS_W18 = a18.SEEDS_W18


def cell(accuracy: float, *, norm: float = 0.5, epochs: int = 400) -> dict:
    """A cell that passes every validity gate, so a refusal below is
    attributable to the thing under test and not to the fixture."""
    return {
        "schema": "shd-cal-cell-v1", "mechanical_status": "COMPLETE",
        "accuracy": accuracy, "classes_predicted": 20,
        "majority_prediction": 0.09, "silent_fraction": 0.02,
        "saturated_fraction": 0.0, "non_finite_events": 0,
        "mean_loss": 0.3, "mean_gradient_norm": norm, "mean_update_rms": 4e-4,
        "mean_firing_rate": 0.2, "tail_loss_improvement": -0.01,
        "temporal_condition": "intact",
        "epoch_mean_loss": [0.3] * epochs,
        "epoch_mean_gradient_norm": [norm] * epochs,
        "epoch_max_gradient_norm": [norm * 10] * epochs,
        "epoch_max_gradient_step": [1] * epochs,
    }


class BarsMatchThePrereg(unittest.TestCase):
    """Every threshold, checked against the numbers written in the prereg."""

    def test_the_registered_bars_are_the_ones_in_the_prereg(self):
        self.assertEqual(a18.H18_1_MARGIN, 0.02)
        self.assertEqual(a18.H18_1_INTERIOR, (2, 3))
        self.assertEqual(a18.H18_2_SICK_NORM, 1.0)
        self.assertEqual(a18.H18_2_SICK_GAIN, -0.10)
        self.assertEqual(a18.H18_2_HEALTHY_FLOOR, -0.05)
        self.assertEqual((a18.H18_3_GAIN, a18.H18_3_POSITIVE), (0.02, 15))
        self.assertEqual(a18.MIN_VALID_PER_ARM, 9)
        self.assertEqual(a18.DEPTHS, (1, 2, 3, 4))
        self.assertEqual(len(SEEDS_W18), 20)

    def test_the_prereg_document_carries_the_same_numbers(self):
        """A bar in two places is a bar that can drift in one of them."""
        text = (ROOT / "results"
                / "PREREG_2026-08-27_DEPTH_IS_NOT_MONOTONE_AT_H1024.md").read_text()
        for needle in ("**L2 or L3**", "≥ 0.02", "**≥ 1.0**", "**≤ −0.10**",
                       "**−0.05**", "≥ 15/20", "byte-identical"):
            self.assertIn(needle, text,
                          f"the prereg no longer states {needle!r}; the analyser "
                          "and the registration have drifted apart")


class LadderGrid(unittest.TestCase):
    """H18-1, H18-2 and H18-3 on grids whose answers are known by construction."""

    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.res = Path(self.tmp.name) / "results"
        self.res.mkdir()

    def tearDown(self):
        self.tmp.cleanup()

    def write(self, stem: str, accuracy: float, *, norm: float = 0.5, seeds=None):
        for seed in (seeds or SEEDS_W18):
            (self.res / f"{stem}__s{seed}.json").write_text(
                json.dumps(cell(accuracy, norm=norm)))

    def depth_stem(self, layers: int) -> str:
        return f"w18dep__ff-fixed-attn__h1024__e400__{ANCHOR}__d32l{layers}"

    def control_stem(self) -> str:
        return f"w18dep__ff-fixed__h1024__e400__{ANCHOR}"

    def ladder(self, accuracies: dict[int, float], norms: dict[int, float] | None = None,
               control: float = 0.70, seeds=None):
        self.write(self.control_stem(), control, seeds=seeds)
        for depth, acc in accuracies.items():
            self.write(self.depth_stem(depth), acc,
                       norm=(norms or {}).get(depth, 0.5), seeds=seeds)

    def run_analyser(self, plan_ids: list[str] | None = None) -> str:
        ids = plan_ids or [p.stem for p in self.res.glob("*.json")]
        plan = Path(self.tmp.name) / "plan.json"
        plan.write_text(json.dumps([{"id": i} for i in ids]))
        out = Path(self.tmp.name) / "report.md"
        proc = subprocess.run(
            [sys.executable, str(ROOT / "scripts/aws/analyse_wave18.py"),
             "--plan", str(plan), "--results", str(self.res), "--out", str(out)],
            capture_output=True, text=True)
        self.assertEqual(proc.returncode, 0, proc.stderr)
        return out.read_text()

    def test_an_interior_maximum_clear_of_both_ends_meets_h18_1(self):
        self.ladder({1: 0.69, 2: 0.78, 3: 0.75, 4: 0.60})
        report = self.run_analyser()
        self.assertIn("**H18-1: MET**", report)
        self.assertIn("at **L2**", report)

    def test_a_maximum_at_l1_is_named_as_an_endpoint(self):
        """Depth monotonically hurts: the analyser must say so, not go quiet."""
        self.ladder({1: 0.78, 2: 0.75, 3: 0.72, 4: 0.60})
        report = self.run_analyser()
        self.assertIn("**H18-1: NOT MET**", report)
        self.assertIn("at **L1**", report)
        self.assertIn("twelve-seed accident", report)

    def test_a_maximum_at_l4_voids_the_archive_comparison(self):
        """The archived collapse failing to reproduce is the worst outcome and
        must be reported as such rather than as a passing ladder."""
        self.ladder({1: 0.69, 2: 0.72, 3: 0.75, 4: 0.80})
        report = self.run_analyser()
        self.assertIn("**H18-1: NOT MET**", report)
        self.assertIn("does not reproduce on this", report)

    def test_an_interior_maximum_inside_the_margin_is_unresolved(self):
        """Interior but not clear of the ends: not a win, and not silence."""
        self.ladder({1: 0.770, 2: 0.780, 3: 0.775, 4: 0.765})
        report = self.run_analyser()
        self.assertIn("**H18-1: NOT MET**", report)
        self.assertIn("unresolved", report)

    def test_a_maximum_at_l3_asks_for_the_rung_that_would_bound_it(self):
        self.ladder({1: 0.69, 2: 0.75, 3: 0.80, 4: 0.60})
        report = self.run_analyser()
        self.assertIn("**H18-1: MET**", report)
        self.assertIn("L5", report)

    def test_a_missing_rung_blocks_the_argmax(self):
        self.ladder({1: 0.69, 2: 0.78, 4: 0.60})
        report = self.run_analyser()
        self.assertIn("**H18-1: NOT EVALUABLE**", report)
        self.assertIn("L3", report)

    def test_an_arm_below_the_valid_floor_carries_no_numbers(self):
        """A blocked comparison must not print a gain beside its banner."""
        self.write(self.control_stem(), 0.70)
        self.write(self.depth_stem(2), 0.78, seeds=SEEDS_W18[:5])
        report = self.run_analyser()
        row = next(l for l in report.splitlines() if l.startswith("| L2 "))
        self.assertNotRegex(row, r"[+-]0\.\d{4}",
                            "a blocked row must carry no gain")

    def test_h18_2_fires_when_a_sick_arm_is_not_collapsed(self):
        """The half that catches 'sick but fine' -- the rule's real content."""
        self.ladder({1: 0.69, 2: 0.78, 3: 0.75, 4: 0.74},
                    norms={1: 0.02, 2: 0.7, 3: 0.9, 4: 55.0})
        report = self.run_analyser()
        self.assertIn("**H18-2: NOT MET**", report)
        self.assertIn("L4", report.split("## H18-2")[1].split("## H18-3")[0])

    def test_h18_2_fires_when_a_healthy_arm_collapses(self):
        """The second half. Without it the rule is satisfiable by any grid in
        which the sick arm happens to be the worst one."""
        self.ladder({1: 0.69, 2: 0.78, 3: 0.55, 4: 0.50},
                    norms={1: 0.02, 2: 0.7, 3: 0.02, 4: 55.0})
        report = self.run_analyser()
        self.assertIn("**H18-2: NOT MET**", report)

    def test_h18_2_passes_on_the_shape_the_prereg_predicts(self):
        """And it must be able to say MET, or it says nothing."""
        self.ladder({1: 0.69, 2: 0.78, 3: 0.76, 4: 0.55},
                    norms={1: 0.02, 2: 0.7, 3: 0.9, 4: 55.0})
        report = self.run_analyser()
        self.assertIn("**H18-2: MET**", report)

    def test_h18_3_needs_twenty_pairs_not_twelve(self):
        """The registered bar is 15/20; twelve pairs cannot reach it and must
        not be allowed to look like a verdict."""
        self.ladder({1: 0.69, 2: 0.78, 3: 0.75, 4: 0.60}, seeds=SEEDS)
        report = self.run_analyser()
        self.assertIn("**H18-3: NOT EVALUABLE**", report)
        self.assertIn("cannot be reached", report)

    def test_h18_3_meets_on_a_full_twenty_seed_arm(self):
        self.ladder({1: 0.69, 2: 0.78, 3: 0.75, 4: 0.60})
        report = self.run_analyser()
        self.assertIn("**H18-3: MET**", report)


class HarnessCheckIsDestructive(unittest.TestCase):
    """H18-4, in both directions.

    Against a **constructed** archive rather than the committed corpus, because
    the wave-15 L2 cells are still staged outside the tree pending the corpus
    re-freeze, and a destructive check whose tests skip is indistinguishable
    from one that cannot fire. The committed corpus is used as well when it
    carries those cells, but the constructed archive is what makes both
    directions run today.
    """

    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.res = Path(self.tmp.name) / "results"
        self.res.mkdir()
        self.archive = Path(self.tmp.name) / "archive"
        self.archive.mkdir()
        self.w15 = f"w15col__ff-fixed-attn__h1024__e400__{ANCHOR}__d32l2"
        self.w18 = f"w18dep__ff-fixed-attn__h1024__e400__{ANCHOR}__d32l2"
        for seed in SEEDS:
            body = json.dumps(cell(0.77 + seed % 3 * 1e-3, norm=0.7))
            (self.archive / f"{self.w15}__s{seed}.json").write_text(body)
            (self.res / f"{self.w18}__s{seed}.json").write_text(body)

    def tearDown(self):
        self.tmp.cleanup()

    def run_analyser(self) -> str:
        plan = Path(self.tmp.name) / "plan.json"
        plan.write_text(json.dumps([{"id": p.stem} for p in self.res.glob("*.json")]))
        out = Path(self.tmp.name) / "report.md"
        proc = subprocess.run(
            [sys.executable, str(ROOT / "scripts/aws/analyse_wave18.py"),
             "--plan", str(plan), "--results", str(self.res),
             "--archive", str(self.archive), "--out", str(out)],
            capture_output=True, text=True)
        self.assertEqual(proc.returncode, 0, proc.stderr)
        return out.read_text()

    def test_an_untouched_copy_reproduces(self):
        self.assertIn("**H18-4: MET**", self.run_analyser())

    def test_one_perturbed_cell_voids_every_verdict(self):
        target = self.res / f"{self.w18}__s{SEEDS[0]}.json"
        perturbed = json.loads(target.read_text())
        perturbed["accuracy"] += 1e-9
        target.write_text(json.dumps(perturbed))
        report = self.run_analyser()
        self.assertIn("**H18-4: NOT MET**", report)
        self.assertIn("Every verdict below is VOID", report)
        for hypothesis in ("H18-1", "H18-2", "H18-3", "H19-1"):
            self.assertIn(f"**{hypothesis}: VOID**", report,
                          f"{hypothesis} survived a failed harness check")

    def test_a_field_absent_from_one_side_is_a_difference_not_a_pass(self):
        """A dropped field must not compare equal to a present one."""
        target = self.res / f"{self.w18}__s{SEEDS[0]}.json"
        thinned = json.loads(target.read_text())
        del thinned["mean_update_rms"]
        target.write_text(json.dumps(thinned))
        report = self.run_analyser()
        self.assertIn("**H18-4: NOT MET**", report)
        self.assertIn("absent from one cell", report)

    def test_the_committed_corpus_reproduces_once_the_cells_land(self):
        """The integration form. It skips until the corpus is re-frozen, which
        is why the constructed-archive tests above exist rather than instead of
        them."""
        committed = ROOT / "results/shd_attention_campaign_v2"
        present = [s for s in SEEDS
                   if (committed / f"{self.w15}__s{s}.json").is_file()]
        if len(present) < a18.MIN_VALID_PER_ARM:
            self.skipTest("wave-15 L2 cells not yet landed in the committed corpus")
        for seed in present:
            (self.res / f"{self.w18}__s{seed}.json").write_text(
                (committed / f"{self.w15}__s{seed}.json").read_text())
        plan = Path(self.tmp.name) / "plan.json"
        plan.write_text(json.dumps([{"id": p.stem} for p in self.res.glob("*.json")]))
        out = Path(self.tmp.name) / "report.md"
        subprocess.run(
            [sys.executable, str(ROOT / "scripts/aws/analyse_wave18.py"),
             "--plan", str(plan), "--results", str(self.res), "--out", str(out)],
            capture_output=True, text=True, check=True)
        self.assertIn("**H18-4: MET**", out.read_text())


class MutationTest(unittest.TestCase):
    """The bars, checked by breaking them rather than by reading them.

    The mutation is applied to a **copy** in a temporary directory. A test that
    can corrupt the thing it guards is worse than no test.
    """

    def test_a_drifted_bar_is_caught(self):
        source = ROOT / "scripts/aws/analyse_wave18.py"
        original = source.read_text()
        broken = original.replace("H18_1_MARGIN = 0.02", "H18_1_MARGIN = 0.001", 1)
        self.assertNotEqual(broken, original, "the bar moved or was renamed")

        with tempfile.TemporaryDirectory() as tmp:
            copy = Path(tmp) / "analyse_wave18.py"
            copy.write_text(broken)
            probe = Path(tmp) / "probe.py"
            probe.write_text(
                "import importlib.util, sys\n"
                f"sys.path.insert(0, {str(ROOT / 'scripts')!r})\n"
                f"spec = importlib.util.spec_from_file_location('m', {str(copy)!r})\n"
                "m = importlib.util.module_from_spec(spec)\n"
                "sys.modules['m'] = m\n"
                "spec.loader.exec_module(m)\n"
                "assert m.H18_1_MARGIN == 0.02, 'bar drifted'\n"
            )
            proc = subprocess.run([sys.executable, str(probe)],
                                  capture_output=True, text=True)

        self.assertNotEqual(proc.returncode, 0,
                            "lowering a registered bar was not detected")
        self.assertIn("bar drifted", proc.stderr)
        self.assertEqual(source.read_text(), original)


if __name__ == "__main__":
    unittest.main(verbosity=2)
