#!/usr/bin/env python3
"""Tests for `scripts/aws/analyse_wave29.py`, frozen with its registration.

Wave 29 exists because wave 28 registered `|DiD| < 0.03` -- a two-sided band --
for a one-sided question, and reported NOT MET at p90 on a DiD of **-0.0382**:
the read-out lost LESS than the substrate, and the clause fired on the outcome
nobody had named.

So the tests that matter here are the ones that would pass under the old,
broken shape. `test_a_growing_advantage_satisfies_h29_1_and_meets_h29_2` and
`test_a_shrinking_advantage_fails_h29_1` drive the same |DiD| from opposite
directions and assert the two clauses answer differently. An analyser that had
kept `abs()` passes neither.

`test_wave_28_seeds_are_refused_by_value` is rule 9.3 as a test: the replacement
bar may not be evaluated on the cells that motivated it.

Run: python3 scripts/test_wave29_analyser.py
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

ROOT = Path(__file__).resolve().parent.parent


def load():
    spec = importlib.util.spec_from_file_location(
        "analyse_wave29", ROOT / "scripts/aws/analyse_wave29.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def cell_json(accuracy, arm="ff+fixed", temporal="intact", hidden=128):
    payload = {
        "schema": "shd-cal-cell-v1", "arm": arm, "accuracy": accuracy,
        "mechanical_status": "COMPLETE", "non_finite_events": 0,
        "non_finite_forward": 0, "classes_predicted": 20,
        "majority_prediction": 0.1, "silent_fraction": 0.2,
        "saturated_fraction": 0.0, "temporal_condition": temporal,
        "hidden": hidden, "epochs": 400, "contract": "published-2ms",
        "geometry": "adjacent-sum-5", "n_train": 8156,
    }
    if temporal != "intact":
        percent = int(temporal[len("spike-dropout-p"):])
        payload["temporal_audit"] = {
            "samples": 8156, "counts_preserved": False, "count_mismatches": 0,
            "relocated_fraction": 0.0, "mean_bin_displacement": 0.0,
            "max_bin_displacement": 0.0, "mean_steps": 358.0,
            "occupied_bins_before": 300.0, "occupied_bins_after": 300.0,
            "count_before": 4.0e6, "count_after": 4.0e6 * (1 - percent / 100),
            "hidden_permutation_relocated": 0.0,
        }
    return payload


REGISTERED = range(5290001, 5290013)
WAVE_28_SEEDS = range(5170001, 5170013)
A = "h128__e400__published-2ms__adjacent-sum-5"


class Wave29(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.results = Path(self.tmp.name) / "cells"
        self.results.mkdir()
        self.module = load()

    def tearDown(self):
        self.tmp.cleanup()

    def write(self, cid, payload):
        (self.results / f"{cid}.json").write_text(json.dumps(payload))

    def corpus(self, rate_intact, attn_intact, rate_drop, attn_drop,
               percent=90, seeds=REGISTERED):
        """Four arms on one seed block, so a DiD is defined for every seed."""
        t = f"spike-dropout-p{percent}"
        for s in seeds:
            self.write(f"w29asy__ff-fixed__{A}__s{s}", cell_json(rate_intact))
            self.write(f"w29asy__ff-fixed-attn__{A}__d32l4__s{s}",
                       cell_json(attn_intact, arm="ff+fixed+attn"))
            self.write(f"w29asy__ff-fixed__{A}__{t}__s{s}",
                       cell_json(rate_drop, temporal=t))
            self.write(f"w29asy__ff-fixed-attn__{A}__d32l4__{t}__s{s}",
                       cell_json(attn_drop, arm="ff+fixed+attn", temporal=t))

    def run_main(self):
        buf = io.StringIO()
        sys.argv = ["x", "--results", str(self.results)]
        with contextlib.redirect_stdout(buf):
            code = self.module.main()
        return code, buf.getvalue()

    def clause(self, text, name):
        for line in text.splitlines():
            if line.startswith(name):
                return line
        self.fail(f"{name} printed no verdict")

    # -- the pre-cell state ------------------------------------------------

    def test_no_cells_is_not_a_refutation(self):
        code, text = self.run_main()
        self.assertEqual(code, 2)
        self.assertIn("NOTHING TO ANALYSE", text)
        self.assertIn("This is not a verdict", text)

    # -- rule 9.3, enforced ------------------------------------------------

    def test_wave_28_seeds_are_refused_by_value(self):
        """The replacement bar may not be read on the cells that motivated it.

        Tagged `w29asy` so only the seed value can reject them. A wave that
        filtered on the tag alone would analyse these.
        """
        self.corpus(0.74, 0.83, 0.63, 0.72, seeds=WAVE_28_SEEDS)
        code, text = self.run_main()
        self.assertEqual(code, 2, text)
        self.assertIn("seed outside this wave's registered block", text)

    def test_an_unregistered_seed_block_is_also_refused(self):
        self.corpus(0.74, 0.83, 0.63, 0.72, seeds=range(9990001, 9990013))
        code, text = self.run_main()
        self.assertEqual(code, 2, text)
        self.assertIn("seed outside this wave's registered block", text)

    # -- the one-sidedness, from both directions ---------------------------

    def test_a_growing_advantage_satisfies_h29_1_and_meets_h29_2(self):
        """Wave 28's own sign: the read-out loses less than the substrate.

        rate 0.74 -> 0.63 loses 0.11; attn 0.83 -> 0.77 loses 0.06.
        DiD = 0.06 - 0.11 = -0.05, below -0.03. H29-1 is satisfied (it is only
        failed above +0.03) and H29-2 is MET.
        """
        self.corpus(0.74, 0.83, 0.63, 0.77)
        code, text = self.run_main()
        self.assertEqual(code, 0, text)
        self.assertIn("MET", self.clause(text, "H29-1"))
        self.assertNotIn("NOT MET", self.clause(text, "H29-1"))
        self.assertIn("MET", self.clause(text, "H29-2"))
        self.assertNotIn("NOT MET", self.clause(text, "H29-2"))

    def test_a_shrinking_advantage_fails_h29_1(self):
        """The same |DiD|, the other way. This is the case the two-sided band
        could not distinguish, and it must now read differently."""
        self.corpus(0.74, 0.83, 0.69, 0.72)   # rate loses .05, attn loses .11
        code, text = self.run_main()
        self.assertEqual(code, 0, text)
        self.assertIn("NOT MET", self.clause(text, "H29-1"))
        self.assertIn("NOT MET", self.clause(text, "H29-2"))

    def test_a_flat_did_satisfies_h29_1_and_fails_h29_2(self):
        """Insensitive to counts: no cost, and no growth either. The two
        clauses must split here, or the pair is one clause with two names."""
        self.corpus(0.74, 0.83, 0.63, 0.72)   # both lose 0.11, DiD = 0
        code, text = self.run_main()
        self.assertEqual(code, 0, text)
        self.assertIn("MET", self.clause(text, "H29-1"))
        self.assertNotIn("NOT MET", self.clause(text, "H29-1"))
        self.assertIn("NOT MET", self.clause(text, "H29-2"))

    # -- the sensitivity gate ---------------------------------------------

    def test_an_insensitive_instrument_blocks_both_clauses(self):
        """The rate arm barely moves, so no reading is taken about the
        read-out. NOT EVALUABLE, never NOT MET."""
        self.corpus(0.74, 0.83, 0.735, 0.825)
        code, text = self.run_main()
        self.assertEqual(code, 0, text)
        self.assertIn("NOT sensitive", text)
        self.assertIn("NOT EVALUABLE", self.clause(text, "H29-1"))
        self.assertIn("NOT EVALUABLE", self.clause(text, "H29-2"))

    # -- the guards -------------------------------------------------------

    def test_off_anchor_cells_are_dropped_aloud(self):
        self.corpus(0.74, 0.83, 0.63, 0.77)
        for s in REGISTERED:
            self.write(f"w29asy__ff-fixed__h1024__e400__published-2ms__"
                       f"adjacent-sum-5__s{s}", cell_json(0.5, hidden=1024))
        _, text = self.run_main()
        self.assertIn("not at h128", text)

    def test_a_seed_missing_one_arm_is_dropped_not_imputed(self):
        self.corpus(0.74, 0.83, 0.63, 0.77)
        (self.results / f"w29asy__ff-fixed-attn__{A}__d32l4__"
                        f"spike-dropout-p90__s5290001.json").unlink()
        _, text = self.run_main()
        self.assertIn("/11", text)

    def test_the_fixture_carries_what_cell_validity_pins(self):
        """The wave-13/14 fixtures drifted from the corpus twice, on attn_dim
        and then on n_train, because each guard named the fields that had just
        broken. This one asks the pinned set itself."""
        sys.path.insert(0, str(ROOT / "scripts"))
        import cell_validity as cv
        fixture = cell_json(0.83, arm="ff+fixed+attn")
        for plan_key, cell_key in sorted(cv.PLAN_PINNED_FIELDS.items()):
            if plan_key in ("attn_dim", "attn_layers", "val_speakers",
                            "attn_readout", "tau_m"):
                continue          # not pinned for this wave's plan entries
            with self.subTest(field=cell_key):
                self.assertIn(
                    cell_key, fixture,
                    f"cell_validity pins {cell_key}; a fixture without it "
                    f"voids every cell here while the corpus is fine")


if __name__ == "__main__":
    unittest.main(verbosity=2)
