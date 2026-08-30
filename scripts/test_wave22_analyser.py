#!/usr/bin/env python3
"""Tests for `scripts/aws/analyse_wave22.py`, frozen with its preregistration.

Wave 22 is the first wave to vary the **read-out depth** while the shuffle
control is present, so the failure this file exists to prevent is the H17-2
defect: two read-out depths merged into one contrast, which inflated a published
shuffle cost by 17% the last time it happened. Most tests below are a way of
getting that wrong.

Run: python3 scripts/test_wave22_analyser.py
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
    "analyse_wave22", ROOT / "scripts/aws/analyse_wave22.py")
w22 = importlib.util.module_from_spec(spec)
sys.modules["analyse_wave22"] = w22
spec.loader.exec_module(w22)

SEEDS = [5170001 + i for i in range(12)]


def cell(accuracy, *, arm="ff+fixed+attn", hidden=128, contract="published-2ms",
         geometry="adjacent-sum-5", temporal="intact", epochs=400):
    return {
        "schema": "shd-cal-cell-v1", "arm": arm, "hidden": hidden,
        "epochs": epochs, "accuracy": accuracy, "contract": contract,
        "geometry": geometry, "temporal_condition": temporal,
        "mechanical_status": "COMPLETE", "non_finite_events": 0,
        "classes_predicted": 20, "majority_prediction": 0.1,
        "silent_fraction": 0.1, "saturated_fraction": 0.0,
        "temporal_audit": {"samples": 100, "counts_preserved": True,
                           "relocated_fraction": 0.99, "mean_bin_displacement": 5.0,
                           "occupied_bins_before": 10.0, "occupied_bins_after": 10.0},
        "epoch_mean_loss": [1.0, 0.5], "epoch_mean_gradient_norm": [1.0, 1.0],
        "epoch_max_gradient_norm": [2.0, 2.0],
    }


def write(root, name, payload):
    (Path(root) / name).write_text(json.dumps(payload))


def corpus(entries):
    """entries: list of (filename, cell). Returns the index."""
    tmp = tempfile.mkdtemp()
    for name, payload in entries:
        write(tmp, name, payload)
    return w22.index([tmp])


class DepthKeyingTest(unittest.TestCase):
    """The H17-2 defect, in every form it could return."""

    def test_two_read_out_depths_do_not_share_a_key(self):
        cells, _ = corpus([
            (f"w__ff-fixed-attn__h128__d32l4__s{SEEDS[0]}.json", cell(0.83)),
            (f"w__ff-fixed-attn__h128__d32l2__s{SEEDS[0]}.json", cell(0.70)),
        ])
        keys = {k for k in cells if k[3] == "ff+fixed+attn"}
        depths = {k[4] for k in keys}
        self.assertEqual(depths, {"d32l4", "d32l2"},
                         "the read-out depth is not in the key; two depths would merge")

    def test_a_did_at_one_depth_ignores_another_depths_cells(self):
        """The concrete H17-2 shape: a d32l2 shuffled cell must not be able to
        serve as the d32l4 arm's control."""
        entries = []
        for seed in SEEDS:
            entries += [
                (f"a__ff-fixed-attn__h128__d32l4__s{seed}.json", cell(0.83)),
                (f"b__ff-fixed-attn__h128__d32l2__bin-shuffled__s{seed}.json",
                 cell(0.60, temporal="bin-shuffled")),
                (f"c__ff-fixed__h128__s{seed}.json", cell(0.70, arm="ff+fixed")),
                (f"d__ff-fixed__h128__bin-shuffled__s{seed}.json",
                 cell(0.69, arm="ff+fixed", temporal="bin-shuffled")),
            ]
        cells, _ = corpus(entries)
        value, _, pairs = w22.did(cells, 128, "published-2ms", "adjacent-sum-5", "d32l4")
        self.assertEqual(pairs, 0,
                         "a d32l2 shuffled cell was used as the d32l4 control")
        self.assertIsNone(value)

    def test_the_rate_arm_is_shared_across_depths_deliberately(self):
        """One `ff+fixed` control serves every depth: the rate arm has no
        read-out, so this is correct and is why nine points need only one arm."""
        entries = []
        for seed in SEEDS:
            entries += [
                (f"a__ff-fixed-attn__h128__d32l2__s{seed}.json", cell(0.80)),
                (f"b__ff-fixed-attn__h128__d32l2__bin-shuffled__s{seed}.json",
                 cell(0.60, temporal="bin-shuffled")),
                (f"c__ff-fixed__h128__s{seed}.json", cell(0.70, arm="ff+fixed")),
                (f"d__ff-fixed__h128__bin-shuffled__s{seed}.json",
                 cell(0.69, arm="ff+fixed", temporal="bin-shuffled")),
            ]
        cells, _ = corpus(entries)
        value, positive, pairs = w22.did(cells, 128, "published-2ms",
                                         "adjacent-sum-5", "d32l2")
        self.assertEqual(pairs, 12)
        # (0.80 - 0.60) - (0.70 - 0.69) = 0.19
        self.assertAlmostEqual(value, 0.19, places=6)
        self.assertEqual(positive, 12)


class PairingTest(unittest.TestCase):

    def test_the_did_needs_all_four_arms_on_a_shared_seed(self):
        """A quadruple missing one arm is not a difference of differences."""
        entries = []
        for seed in SEEDS:
            entries += [
                (f"a__ff-fixed-attn__h128__d32l4__s{seed}.json", cell(0.83)),
                (f"c__ff-fixed__h128__s{seed}.json", cell(0.70, arm="ff+fixed")),
                (f"d__ff-fixed__h128__bin-shuffled__s{seed}.json",
                 cell(0.69, arm="ff+fixed", temporal="bin-shuffled")),
            ]
        cells, _ = corpus(entries)
        _, _, pairs = w22.did(cells, 128, "published-2ms", "adjacent-sum-5", "d32l4")
        self.assertEqual(pairs, 0, "a DiD was formed without the shuffled attention arm")

    def test_seeds_present_in_only_some_arms_are_dropped(self):
        entries = []
        for i, seed in enumerate(SEEDS):
            entries += [
                (f"a__ff-fixed-attn__h128__d32l4__s{seed}.json", cell(0.83)),
                (f"c__ff-fixed__h128__s{seed}.json", cell(0.70, arm="ff+fixed")),
                (f"d__ff-fixed__h128__bin-shuffled__s{seed}.json",
                 cell(0.69, arm="ff+fixed", temporal="bin-shuffled")),
            ]
            if i < 5:  # only five seeds get the fourth arm
                entries.append(
                    (f"b__ff-fixed-attn__h128__d32l4__bin-shuffled__s{seed}.json",
                     cell(0.60, temporal="bin-shuffled")))
        cells, _ = corpus(entries)
        _, _, pairs = w22.did(cells, 128, "published-2ms", "adjacent-sum-5", "d32l4")
        self.assertEqual(pairs, 5)
        self.assertLess(pairs, w22.MIN_PAIRS,
                        "five pairs must fall below the floor and be NOT EVALUABLE")


class ExclusionTest(unittest.TestCase):
    """Cells that share an operating point but are different experiments."""

    def test_a_rescue_lever_cell_is_not_admitted(self):
        entries = []
        for seed in SEEDS:
            poisoned = cell(0.55)
            poisoned["clip_grad_norm"] = 1000.0
            entries.append(
                (f"a__ff-fixed-attn__h1024__d32l4__clip1000.0__s{seed}.json", poisoned))
        cells, _ = corpus(entries)
        self.assertEqual(len(cells), 0, "a clipped cell entered the wave's index")

    def test_a_wrong_budget_cell_is_not_admitted(self):
        cells, _ = corpus([
            (f"a__ff-fixed-attn__h128__d32l4__s{SEEDS[0]}.json", cell(0.83, epochs=100)),
        ])
        self.assertEqual(len(cells), 0)

    def test_an_invalid_cell_is_voided_rather_than_scored(self):
        broken = cell(0.83)
        broken["classes_predicted"] = 1
        cells, voided = corpus([
            (f"a__ff-fixed-attn__h128__d32l4__s{SEEDS[0]}.json", broken),
        ])
        self.assertEqual(len(cells), 0)
        self.assertEqual(sum(voided.values()), 1)

    def test_a_cell_with_a_non_finite_forward_is_voided(self):
        """The 2026-08-29 guard, reaching the analyser through
        `cell_validity.py`. A cell whose evaluation forward left f32's range
        must not contribute an accuracy to a published contrast."""
        poisoned = cell(0.83)
        poisoned["non_finite_forward"] = 4
        cells, voided = corpus([
            (f"a__ff-fixed-attn__h128__d32l4__s{SEEDS[0]}.json", poisoned),
        ])
        self.assertEqual(len(cells), 0, "a poisoned forward was scored")
        self.assertEqual(sum(voided.values()), 1)

    def test_a_cell_predating_the_guard_is_still_admitted(self):
        """The corpus is 861 cells that carry no `non_finite_forward`. Voiding
        them would delete the campaign."""
        cells, voided = corpus([
            (f"a__ff-fixed-attn__h128__d32l4__s{SEEDS[0]}.json", cell(0.83)),
        ])
        self.assertEqual(sum(voided.values()), 0)
        self.assertEqual(len(cells), 1)


class RegistrationTest(unittest.TestCase):

    def test_the_bars_are_the_preregistrations(self):
        text = (ROOT / "results/PREREG_2026-08-29_THE_MECHANISM_AT_EVERY_OPERATING_POINT.md").read_text()
        self.assertIn("> +0.03", text)
        self.assertEqual(w22.H22_1_MIN_DID, 0.03)
        self.assertIn("≥ 9 of 12", text)
        self.assertEqual(w22.H22_1_MIN_POSITIVE, 9)
        self.assertIn("within 0.10", text)
        self.assertEqual(w22.H22_3_MAX_DEPTH_RANGE, 0.10)

    def test_all_twelve_points_are_declared(self):
        self.assertEqual(len(w22.POINTS), 12)
        self.assertEqual(len(set(w22.POINTS)), 12, "a point is declared twice")

    def test_the_analyser_checks_at_least_what_gate_f_compares(self):
        """The one-directional invariant: an analyser may check more, never
        less. Asserted here because this analyser is new and the coupling test
        only loads wave 15 and 18."""
        sys.path.insert(0, str(ROOT / "scripts"))
        import gate_f_rust
        measurements = set(gate_f_rust.COMPARED_FIELDS) - {"n_train", "n_test"}
        self.assertEqual(measurements - set(w22.SCIENTIFIC_FIELDS), set())


if __name__ == "__main__":
    unittest.main(verbosity=2)
