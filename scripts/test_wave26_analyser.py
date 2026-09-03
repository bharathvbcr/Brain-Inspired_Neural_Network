#!/usr/bin/env python3
"""Tests for `scripts/aws/analyse_wave26.py`, frozen with its preregistration.

Wave 26 is the first wave whose primary hypothesis is read from a **probe file**
rather than from cell accuracies, and the first to carry three arms that differ
only in a read-out variant. Both are new ways to be wrong:

  * a probe file that did not arrive reads as "no entropy collapse", which is
    the refutation of H26-1 rather than its absence;
  * a `no-position` cell indexed as a `default` cell would put six arms into
    three keys and quietly average the null into the thing it is a null for.

Run: python3 scripts/test_wave26_analyser.py
"""

from __future__ import annotations

import importlib.util
import json
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def load():
    spec = importlib.util.spec_from_file_location(
        "analyse_wave26", ROOT / "scripts/aws/analyse_wave26.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def cell_json(accuracy, arm="ff+fixed+attn", temporal="intact", readout=None):
    payload = {
        "schema": "shd-cal-cell-v1", "arm": arm, "accuracy": accuracy,
        "mechanical_status": "COMPLETE", "non_finite_events": 0,
        "non_finite_forward": 0, "classes_predicted": 20,
        "majority_prediction": 0.1, "silent_fraction": 0.2,
        "saturated_fraction": 0.0, "temporal_condition": temporal,
        "hidden": 128, "epochs": 400, "contract": "published-2ms",
        "geometry": "adjacent-sum-5",
    }
    if readout:
        payload["attn_readout"] = readout
    if temporal != "intact":
        payload["temporal_audit"] = {
            "samples": 8156, "counts_preserved": True, "count_mismatches": 0,
            "relocated_fraction": 0.99, "mean_bin_displacement": 120.0,
            "max_bin_displacement": 357.0, "mean_steps": 358.0,
            "occupied_bins_before": 300.0, "occupied_bins_after": 300.0,
            "count_before": 4.0e6, "count_after": 4.0e6,
            "hidden_permutation_relocated": 0.0,
        }
        if temporal.startswith("window-shuffled-w"):
            window = int(temporal[len("window-shuffled-w"):])
            payload["temporal_audit"]["max_bin_displacement"] = float(window - 1)
            payload["temporal_audit"]["mean_bin_displacement"] = window / 3.0
            payload["temporal_audit"]["relocated_fraction"] = 1.0 - 1.0 / window
    return payload


class Wave26Analyser(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.results = Path(self.tmp.name) / "cells"
        self.probes = Path(self.tmp.name) / "probes"
        self.results.mkdir()
        self.probes.mkdir()
        self.module = load()

    def tearDown(self):
        self.tmp.cleanup()

    def write(self, cell_id, payload):
        (self.results / f"{cell_id}.json").write_text(json.dumps(payload))

    def anchor(self, seed, wave="w26pos", temporal="intact", suffix=""):
        base = f"{wave}__%s__h128__e400__published-2ms__adjacent-sum-5"
        tail = f"__s{seed}.json"
        return base, tail

    def populate_pos(self, rate=0.74, attn=0.83, nopos=0.75,
                     rate_s=0.72, attn_s=0.76, nopos_s=0.74):
        """Six arms x twelve seeds, the H26-2 design."""
        for seed in range(5170001, 5170013):
            self.write(f"w26pos__ff-fixed__h128__e400__published-2ms__adjacent-sum-5__s{seed}",
                       cell_json(rate, arm="ff+fixed"))
            self.write(f"w26pos__ff-fixed__h128__e400__published-2ms__adjacent-sum-5__bin-shuffled__s{seed}",
                       cell_json(rate_s, arm="ff+fixed", temporal="bin-shuffled"))
            self.write(f"w26pos__ff-fixed-attn__h128__e400__published-2ms__adjacent-sum-5__d32l4__s{seed}",
                       cell_json(attn))
            self.write(f"w26pos__ff-fixed-attn__h128__e400__published-2ms__adjacent-sum-5__d32l4__bin-shuffled__s{seed}",
                       cell_json(attn_s, temporal="bin-shuffled"))
            self.write(f"w26pos__ff-fixed-attn__h128__e400__published-2ms__adjacent-sum-5__d32l4__no-position__s{seed}",
                       cell_json(nopos, readout="no-position"))
            self.write(f"w26pos__ff-fixed-attn__h128__e400__published-2ms__adjacent-sum-5__d32l4__no-position__bin-shuffled__s{seed}",
                       cell_json(nopos_s, temporal="bin-shuffled", readout="no-position"))

    def test_nothing_to_analyse_is_not_a_refutation(self):
        """An empty corpus must refuse, not print NOT MET three times."""
        code = self.module.main.__wrapped__ if hasattr(self.module.main, "__wrapped__") else None
        sys.argv = ["x", "--results", str(self.results), "--probes", str(self.probes)]
        self.assertEqual(self.module.main(), 2)

    def test_a_no_position_cell_is_not_indexed_as_a_default_cell(self):
        """Six arms must land in six keys, not three."""
        self.populate_pos()
        cells, _ = self.module.index(self.results)
        readouts = {key[3] for key in cells}
        self.assertEqual(readouts, {"default", "no-position"})
        self.assertEqual(len(cells), 6, sorted(cells))
        for key, seeds in cells.items():
            self.assertEqual(len(seeds), 12, key)

    def test_the_structural_null_is_detected_when_it_is_there(self):
        self.populate_pos(attn=0.83, nopos=0.75)
        cells, _ = self.module.index(self.results)
        default, _, _, by_seed_d = self.module.gain(cells, "w26pos", "default", "intact")
        nopos, _, _, by_seed_n = self.module.gain(cells, "w26pos", "no-position", "intact")
        self.assertAlmostEqual(default, 0.09, places=6)
        self.assertAlmostEqual(nopos, 0.01, places=6)
        self.assertGreater(default - nopos, self.module.BAR)

    def test_a_null_that_is_not_there_does_not_read_as_one(self):
        """If no-position keeps the advantage, H26-2a must fail."""
        self.populate_pos(attn=0.83, nopos=0.828)
        cells, _ = self.module.index(self.results)
        default, _, _, _ = self.module.gain(cells, "w26pos", "default", "intact")
        nopos, _, _, _ = self.module.gain(cells, "w26pos", "no-position", "intact")
        self.assertLess(default - nopos, self.module.BAR)

    def test_a_voided_cell_lowers_n_rather_than_being_replaced(self):
        self.populate_pos()
        bad = self.results / ("w26pos__ff-fixed-attn__h128__e400__published-2ms"
                              "__adjacent-sum-5__d32l4__s5170001.json")
        payload = json.loads(bad.read_text())
        payload["classes_predicted"] = 1
        bad.write_text(json.dumps(payload))
        cells, voided = self.module.index(self.results)
        self.assertEqual(sum(voided.values()), 1)
        _, _, n, _ = self.module.gain(cells, "w26pos", "default", "intact")
        self.assertEqual(n, 11)

    # ---- tau-half ------------------------------------------------------
    def test_tau_half_interpolates_between_the_bracketing_rungs(self):
        ladder = {2: 0.01, 4: 0.02, 8: 0.04, 16: 0.06, 32: 0.08, 64: 0.09, 128: 0.10}
        value, why = self.module.tau_half(ladder, 0.10)
        # half-maximum is 0.05, bracketed by w8 (0.04) and w16 (0.06)
        self.assertEqual(why, "interpolated")
        self.assertAlmostEqual(value, 12.0, places=6)

    def test_tau_half_is_withheld_on_a_non_monotone_ladder(self):
        ladder = {2: 0.01, 4: 0.09, 8: 0.02, 16: 0.06, 32: 0.08, 64: 0.09, 128: 0.10}
        value, why = self.module.tau_half(ladder, 0.10)
        self.assertIsNone(value)
        self.assertIn("WITHHELD", why)

    def test_tau_half_is_undefined_when_there_is_no_order_effect(self):
        """Not large -- undefined. A half-maximum of nothing is nothing."""
        ladder = {w: 0.001 for w in self.module.WINDOWS}
        value, why = self.module.tau_half(ladder, 0.02)
        self.assertIsNone(value)
        self.assertIn("not above the bar", why)

    def test_tau_half_reports_when_the_ladder_never_reaches_it(self):
        ladder = {w: 0.01 for w in self.module.WINDOWS}
        value, why = self.module.tau_half(ladder, 0.10)
        self.assertIsNone(value)
        self.assertIn("not reached", why)

    # ---- probes --------------------------------------------------------
    def probe_file(self, cell_id, series):
        lines = []
        for epoch, entropy, q, k in series:
            lines.append(json.dumps({
                "schema": "shd-readout-probe-v1", "epoch": epoch, "samples": 256,
                "mean_t_steps": 358.0, "residual_norm": 4.0,
                "normalised_entropy": [entropy, entropy],
                "min_normalised_entropy": [entropy, entropy],
                "max_weight": [0.1, 0.1], "saturated_rows": [0.0, 0.0],
                "score_range": [2.0, 2.0], "q_norm": [q, q], "k_norm": [k, k],
                "score_bound": [q * k, q * k],
            }))
        (self.probes / f"{cell_id}.jsonl").write_text("\n".join(lines) + "\n")

    def test_a_probe_that_did_not_arrive_is_not_an_entropy_reading(self):
        probes = self.module.read_probes(self.probes)
        value, n = self.module.probe_stat(probes, "d32l4", 400, "normalised_entropy")
        self.assertIsNone(value)
        self.assertEqual(n, 0)

    def test_entropy_and_the_named_term_are_read_per_depth(self):
        for seed in range(5170001, 5170013):
            self.probe_file(
                f"w26sat__ff-fixed-attn__h1024__e400__published-2ms"
                f"__adjacent-sum-5__d32l4__s{seed}",
                [(1, 0.95, 2.0, 2.0), (100, 0.90, 3.0, 3.0), (400, 0.20, 20.0, 20.0)])
            self.probe_file(
                f"w26sat__ff-fixed-attn__h1024__e400__published-2ms"
                f"__adjacent-sum-5__d32l2__s{seed}",
                [(1, 0.95, 2.0, 2.0), (100, 0.92, 2.2, 2.2), (400, 0.88, 2.4, 2.4)])
        probes = self.module.read_probes(self.probes)
        deep, n_deep = self.module.probe_stat(probes, "d32l4", 400, "normalised_entropy")
        shallow, _ = self.module.probe_stat(probes, "d32l2", 400, "normalised_entropy")
        self.assertEqual(n_deep, 12)
        self.assertAlmostEqual(deep, 0.20, places=6)
        self.assertAlmostEqual(shallow, 0.88, places=6)
        first, _ = self.module.qk_product(probes, "d32l4", 1)
        last, _ = self.module.qk_product(probes, "d32l4", 400)
        self.assertAlmostEqual(last / first, 100.0, places=6)

    def test_an_unparseable_probe_line_is_surfaced_not_skipped(self):
        (self.probes / ("w26sat__ff-fixed-attn__h1024__e400__published-2ms"
                        "__adjacent-sum-5__d32l4__s5170001.jsonl")).write_text(
            '{"schema":"shd-readout-probe-v1","epoch":1,"normalised_entropy":[inf]}\n')
        probes = self.module.read_probes(self.probes)
        record = next(iter(probes.values()))
        self.assertIn("unparseable", record)


if __name__ == "__main__":
    unittest.main(verbosity=2)
