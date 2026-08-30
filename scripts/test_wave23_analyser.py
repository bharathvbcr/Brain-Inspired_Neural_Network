#!/usr/bin/env python3
"""Tests for `scripts/aws/analyse_wave23.py`, frozen with its preregistration.

Wave 23 compares two training budgets, so the failure to prevent is comparing
across them where the design says within: a gain is attention-minus-rate **at
the same budget**, and only one registered hypothesis crosses budgets at all.

Run: python3 scripts/test_wave23_analyser.py
"""

from __future__ import annotations

import importlib.util
import json
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
spec = importlib.util.spec_from_file_location(
    "analyse_wave23", ROOT / "scripts/aws/analyse_wave23.py")
w23 = importlib.util.module_from_spec(spec)
sys.modules["analyse_wave23"] = w23
spec.loader.exec_module(w23)

SEEDS = [5170001 + i for i in range(12)]


def cell(accuracy, *, arm="ff+fixed+attn", epochs=100, trace=None):
    return {
        "schema": "shd-cal-cell-v1", "arm": arm, "hidden": 1024,
        "epochs": epochs, "accuracy": accuracy, "contract": "published-2ms",
        "geometry": "adjacent-sum-5", "temporal_condition": "intact",
        "mechanical_status": "COMPLETE", "non_finite_events": 0,
        "classes_predicted": 20, "majority_prediction": 0.1,
        "silent_fraction": 0.1, "saturated_fraction": 0.0,
        "epoch_mean_loss": trace if trace is not None else [1.0] * 20,
        "epoch_mean_gradient_norm": [1.0], "epoch_max_gradient_norm": [2.0],
    }


def corpus(entries):
    tmp = tempfile.mkdtemp()
    for name, payload in entries:
        (Path(tmp) / name).write_text(json.dumps(payload))
    return w23.index(tmp)


def both_arms(epochs, depth, attn_acc, rate_acc, trace=None):
    out = []
    for seed in SEEDS:
        out.append((f"a__ff-fixed-attn__h1024__e{epochs}__{depth}__s{seed}.json",
                    cell(attn_acc, epochs=epochs, trace=trace)))
        out.append((f"c__ff-fixed__h1024__e{epochs}__s{seed}.json",
                    cell(rate_acc, arm="ff+fixed", epochs=epochs)))
    return out


class WithinBudgetTest(unittest.TestCase):

    def test_the_gain_pairs_arms_at_the_same_budget(self):
        cells, _ = corpus(both_arms(100, "d32l4", 0.78, 0.72))
        value, positive, pairs = w23.gain(cells, 100, "d32l4")
        self.assertEqual(pairs, 12)
        self.assertAlmostEqual(value, 0.06, places=6)
        self.assertEqual(positive, 12)

    def test_a_rate_arm_at_another_budget_cannot_serve_as_the_control(self):
        """The comparison the design forbids: e100 attention against an e400
        rate arm is two different amounts of training, not a gain."""
        entries = [(f"a__ff-fixed-attn__h1024__e100__d32l4__s{s}.json",
                    cell(0.78, epochs=100)) for s in SEEDS]
        entries += [(f"c__ff-fixed__h1024__e200__s{s}.json",
                     cell(0.72, arm="ff+fixed", epochs=200)) for s in SEEDS]
        cells, _ = corpus(entries)
        _, _, pairs = w23.gain(cells, 100, "d32l4")
        self.assertEqual(pairs, 0, "an e200 rate arm was paired against e100 attention")

    def test_budgets_outside_the_design_are_not_admitted(self):
        cells, _ = corpus(both_arms(400, "d32l4", 0.60, 0.74))
        self.assertEqual(len(cells), 0, "e400 cells entered a wave that runs e100/e200")

    def test_the_two_depths_do_not_share_a_key(self):
        cells, _ = corpus(both_arms(100, "d32l4", 0.78, 0.72)
                          + both_arms(100, "d32l2", 0.81, 0.72))
        l4, _, _ = w23.gain(cells, 100, "d32l4")
        l2, _, _ = w23.gain(cells, 100, "d32l2")
        self.assertAlmostEqual(l4, 0.06, places=6)
        self.assertAlmostEqual(l2, 0.09, places=6)

    def test_a_shuffled_cell_is_not_admitted(self):
        """No shuffled arm runs at these budgets, so the mechanism control is
        untouched and its h1024 row is unaffected."""
        entries = [(f"a__ff-fixed-attn__h1024__e100__d32l4__bin-shuffled__s{s}.json",
                    {**cell(0.5, epochs=100), "temporal_condition": "bin-shuffled"})
                   for s in SEEDS]
        cells, _ = corpus(entries)
        self.assertEqual(len(cells), 0)


class RetentionTest(unittest.TestCase):
    """H23-4 — the fit is kept at the shorter budget."""

    def test_a_held_fit_is_not_counted_as_lost(self):
        held = [2.0] * 5 + [0.001] * 15
        cells, _ = corpus(both_arms(100, "d32l4", 0.78, 0.72, trace=held))
        lost, measured = w23.lost_fits(cells, 100, "d32l4")
        self.assertEqual((lost, measured), (0, 12))

    def test_a_lost_fit_is_counted(self):
        lost_trace = [2.0] * 3 + [0.02] * 5 + [0.9] * 12
        cells, _ = corpus(both_arms(100, "d32l4", 0.60, 0.72, trace=lost_trace))
        lost, measured = w23.lost_fits(cells, 100, "d32l4")
        self.assertEqual((lost, measured), (12, 12))

    def test_a_non_finite_trace_is_not_measured_either_way(self):
        bad = [1.0] * 19 + [float("nan")]
        cells, _ = corpus(both_arms(100, "d32l4", 0.60, 0.72, trace=bad))
        lost, measured = w23.lost_fits(cells, 100, "d32l4")
        self.assertEqual(measured, 0, "a diverged trace was counted as a retention result")


class RegistrationTest(unittest.TestCase):

    def test_the_bars_are_the_preregistrations(self):
        text = (ROOT / "results/PREREG_2026-08-29_THE_COLLAPSE_IS_LATE.md").read_text()
        self.assertIn("> +0.03", text)
        self.assertEqual(w23.H23_1_MIN_GAIN, 0.03)
        self.assertIn("> +0.10", text)
        self.assertEqual(w23.H23_2_MIN_IMPROVEMENT, 0.10)
        self.assertIn("within ±0.03", text)
        self.assertEqual(w23.H23_3_MAX_L2_SHIFT, 0.03)
        self.assertIn("≤ 3 of 12", text)
        self.assertEqual(w23.H23_4_MAX_LOST, 3)

    def test_the_archived_reference_is_a_constant_not_a_recomputation(self):
        """Recomputing the e400 gain here would let the reference move under
        the comparison it anchors."""
        self.assertEqual(w23.E400_L4_GAIN, -0.1318)
        self.assertEqual(w23.E400_L2_GAIN, 0.0405)
        source = (ROOT / "scripts/aws/analyse_wave23.py").read_text()
        self.assertIn("CONSTANT of", source)

    def test_the_retention_factor_matches_the_finding_it_came_from(self):
        sys.path.insert(0, str(ROOT / "scripts"))
        import fit_retention
        self.assertEqual(w23.RETENTION_FACTOR, fit_retention.RETENTION_FACTOR)
        self.assertEqual(w23.TAIL_EPOCHS, fit_retention.TAIL_EPOCHS)

    def test_the_analyser_checks_at_least_what_gate_f_compares(self):
        sys.path.insert(0, str(ROOT / "scripts"))
        import gate_f_rust
        measurements = set(gate_f_rust.COMPARED_FIELDS) - {"n_train", "n_test"}
        self.assertEqual(measurements - set(w23.SCIENTIFIC_FIELDS), set())


if __name__ == "__main__":
    unittest.main(verbosity=2)
