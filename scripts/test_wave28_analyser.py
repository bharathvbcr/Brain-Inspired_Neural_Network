#!/usr/bin/env python3
"""Tests for `scripts/aws/analyse_wave28.py`, frozen with its registration.

Wave 28's two clauses are conditional in one direction and the ordering is the
whole point: H28-2's null is only interpretable if H28-1 established that the
instrument can see anything. An analyser that reported H28-2 as a null under an
insensitive instrument would be publishing exactly the ambiguity section 2
exists to remove.

Run: python3 scripts/test_wave28_analyser.py
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
        "analyse_wave28", ROOT / "scripts/aws/analyse_wave28.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def cell_json(accuracy, arm="ff+fixed", temporal="intact", hidden=128,
              majority=0.1, silent=0.2):
    payload = {
        "schema": "shd-cal-cell-v1", "arm": arm, "accuracy": accuracy,
        "mechanical_status": "COMPLETE", "non_finite_events": 0,
        "non_finite_forward": 0, "classes_predicted": 20,
        "majority_prediction": majority, "silent_fraction": silent,
        "saturated_fraction": 0.0, "temporal_condition": temporal,
        "hidden": hidden, "epochs": 400, "contract": "published-2ms",
        "geometry": "adjacent-sum-5",
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


SEEDS = range(5170001, 5170013)
A = "h128__e400__published-2ms__adjacent-sum-5"


class Wave28(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.results = Path(self.tmp.name) / "cells"
        self.results.mkdir()
        self.module = load()

    def tearDown(self):
        self.tmp.cleanup()

    def write(self, cid, payload):
        (self.results / f"{cid}.json").write_text(json.dumps(payload))

    def baseline(self, rate=0.74, attn=0.83):
        for s in SEEDS:
            self.write(f"w26pos__ff-fixed__{A}__s{s}", cell_json(rate))
            self.write(f"w26pos__ff-fixed-attn__{A}__d32l4__s{s}",
                       cell_json(attn, arm="ff+fixed+attn"))

    def rung(self, percent, rate_acc, attn_acc=None):
        wave = "w27drp" if percent == 30 else "w28drp"
        t = f"spike-dropout-p{percent}"
        for s in SEEDS:
            self.write(f"{wave}__ff-fixed__{A}__{t}__s{s}",
                       cell_json(rate_acc, temporal=t))
            if attn_acc is not None:
                self.write(f"{wave}__ff-fixed-attn__{A}__d32l4__{t}__s{s}",
                           cell_json(attn_acc, arm="ff+fixed+attn", temporal=t))

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

    def test_nothing_to_analyse_is_not_a_refutation(self):
        self.baseline()
        code, text = self.run_main()
        self.assertEqual(code, 2)
        self.assertIn("NOTHING TO ANALYSE", text)

    def test_an_insensitive_ladder_is_a_finding_about_the_instrument(self):
        self.baseline()
        for p in (50, 60, 70, 80, 90):
            self.rung(p, 0.735)          # drop 0.005, far under the bar
        _, text = self.run_main()
        self.assertIn("NOT MET", self.clause(text, "H28-1"))

    def test_a_null_under_an_insensitive_instrument_is_never_reported(self):
        """The whole reason section 2 exists. H28-2 must not read NOT MET
        or MET when H28-1 failed -- either would publish the ambiguity."""
        self.baseline()
        for p in (50, 60, 70, 80, 90):
            self.rung(p, 0.735, attn_acc=0.825 if p in (70, 90) else None)
        code, text = self.run_main()
        self.assertIn("NOT MET", self.clause(text, "H28-1"))
        line = self.clause(text, "H28-2")
        self.assertIn("NOT EVALUABLE", line)
        self.assertNotIn("NOT MET", line)
        self.assertEqual(code, 3)

    def test_the_smallest_sensitive_rate_is_the_one_reported(self):
        self.baseline()
        self.rung(30, 0.7276)            # drop 0.0124, under
        self.rung(50, 0.7175)            # drop 0.0225, under
        self.rung(60, 0.7050)            # drop 0.035, OVER
        self.rung(70, 0.7016, attn_acc=0.7916)
        self.rung(80, 0.6800)
        self.rung(90, 0.6265, attn_acc=0.7165)
        _, text = self.run_main()
        self.assertIn("MET", self.clause(text, "H28-1"))
        self.assertIn("smallest sensitive rate: p60", text)

    def test_a_sensitive_instrument_that_finds_a_null_closes_the_ambiguity(self):
        self.baseline()
        for p in (50, 60, 70, 80, 90):
            # rate loses 0.10; attn loses the same, so DiD is ~0
            self.rung(p, 0.64, attn_acc=0.73 if p in (70, 90) else None)
        code, text = self.run_main()
        self.assertIn("MET", self.clause(text, "H28-1"))
        self.assertIn("MET", self.clause(text, "H28-2"))
        self.assertNotIn("NOT MET", self.clause(text, "H28-2"))
        self.assertEqual(code, 0)

    def test_a_read_out_that_does_depend_on_counts_reads_not_met(self):
        self.baseline()
        for p in (50, 60, 70, 80, 90):
            # rate loses 0.10, attn loses 0.25 -> DiD 0.15, well over the bar
            self.rung(p, 0.64, attn_acc=0.58 if p in (70, 90) else None)
        _, text = self.run_main()
        self.assertIn("MET", self.clause(text, "H28-1"))
        self.assertIn("NOT MET", self.clause(text, "H28-2"))

    def test_p30_is_read_from_wave_27_not_re_run(self):
        self.assertEqual(self.module.wave_of(30), "w27drp")
        self.assertEqual(self.module.wave_of(70), "w28drp")
        self.baseline()
        self.rung(30, 0.7276, attn_acc=0.8176)
        cells, _ = self.module.index(self.results)
        cost, _, n = self.module.rate_cost(cells, 30)
        self.assertEqual(n, 12)
        self.assertAlmostEqual(cost, 0.0124, places=4)

    def test_an_off_width_cell_is_dropped_not_averaged(self):
        self.baseline()
        self.rung(70, 0.66)
        for s in SEEDS:
            self.write(f"w28drp__ff-fixed__h1024__e400__published-2ms"
                       f"__adjacent-sum-5__spike-dropout-p70__s{s}",
                       cell_json(0.2, temporal="spike-dropout-p70", hidden=1024))
        _, text = self.run_main()
        self.assertIn("not at h128", text)


if __name__ == "__main__":
    unittest.main(verbosity=2)
