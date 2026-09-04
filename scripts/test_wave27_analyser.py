#!/usr/bin/env python3
"""Tests for `scripts/aws/analyse_wave27.py`, frozen with its registration.

Wave 27 carries the campaign's only **equality** gate and its only **void
threshold**, and both are new ways to be wrong:

  * a hidden-shuffle gate that ignored provenance fields would pass a wave it
    should void, and one that counted them would void every wave;
  * a tau rung reported from its surviving seeds is a survivorship statistic,
    which is exactly what section 7 forbids.

Run: python3 scripts/test_wave27_analyser.py
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
        "analyse_wave27", ROOT / "scripts/aws/analyse_wave27.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def cell_json(accuracy, arm="ff+fixed+attn", temporal="intact",
              readout=None, saturated=0.0, hidden=128, val=None):
    payload = {
        "schema": "shd-cal-cell-v1", "arm": arm, "accuracy": accuracy,
        "mechanical_status": "COMPLETE", "non_finite_events": 0,
        "non_finite_forward": 0, "classes_predicted": 20,
        "majority_prediction": 0.1, "silent_fraction": 0.2,
        "saturated_fraction": saturated, "temporal_condition": temporal,
        "hidden": hidden, "epochs": 400, "contract": "published-2ms",
        "geometry": "adjacent-sum-5",
    }
    if readout:
        payload["attn_readout"] = readout
    if val is not None:
        payload["val_accuracy"], payload["n_val"] = val, 1169
    if temporal != "intact":
        payload["temporal_audit"] = {
            "samples": 8156, "counts_preserved": True, "count_mismatches": 0,
            "relocated_fraction": 0.99, "mean_bin_displacement": 120.0,
            "max_bin_displacement": 357.0, "mean_steps": 358.0,
            "occupied_bins_before": 300.0, "occupied_bins_after": 300.0,
            "count_before": 4.0e6, "count_after": 4.0e6,
            "hidden_permutation_relocated": 0.0,
        }
        if temporal == "hidden-shuffled":
            payload["temporal_audit"].update(
                relocated_fraction=0.0, mean_bin_displacement=0.0,
                max_bin_displacement=0.0, hidden_permutation_relocated=0.997)
        elif temporal.startswith("spike-dropout-p"):
            payload["temporal_audit"].update(
                counts_preserved=False, relocated_fraction=0.0,
                max_bin_displacement=0.0, mean_bin_displacement=0.0,
                count_after=2.8e6)
        elif temporal.startswith("window-shuffled-w"):
            w = int(temporal[len("window-shuffled-w"):])
            payload["temporal_audit"].update(
                max_bin_displacement=float(w - 1),
                mean_bin_displacement=w / 3.0,
                relocated_fraction=1.0 - 1.0 / w)
    return payload


SEEDS = range(5170001, 5170013)


class Wave27(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.results = Path(self.tmp.name) / "cells"
        self.results.mkdir()
        self.module = load()

    def tearDown(self):
        self.tmp.cleanup()

    def write(self, cid, payload):
        (self.results / f"{cid}.json").write_text(json.dumps(payload))

    A = "h128__e400__published-2ms__adjacent-sum-5"

    def baseline(self, rate=0.74, attn=0.83, rate_s=0.72, attn_s=0.76):
        """Wave 26's reused arms: intact and bin-shuffled, rate and attn."""
        for s in SEEDS:
            self.write(f"w26pos__ff-fixed__{self.A}__s{s}",
                       cell_json(rate, arm="ff+fixed"))
            self.write(f"w26pos__ff-fixed__{self.A}__bin-shuffled__s{s}",
                       cell_json(rate_s, arm="ff+fixed", temporal="bin-shuffled"))
            self.write(f"w26pos__ff-fixed-attn__{self.A}__d32l4__s{s}",
                       cell_json(attn))
            self.write(f"w26pos__ff-fixed-attn__{self.A}__d32l4__bin-shuffled__s{s}",
                       cell_json(attn_s, temporal="bin-shuffled"))

    def hidden(self, rate=0.74, attn=0.80, provenance_only=True):
        for s in SEEDS:
            c = cell_json(rate, arm="ff+fixed", temporal="hidden-shuffled")
            if provenance_only:
                c["emitted_utc"] = "2026-09-05T00:00:00Z"   # provenance differs
            self.write(f"w27hid__ff-fixed__{self.A}__hidden-shuffled__s{s}", c)
            self.write(f"w27hid__ff-fixed-attn__{self.A}__d32l4__hidden-shuffled__s{s}",
                       cell_json(attn, temporal="hidden-shuffled"))

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

    # ---- the equality gate ------------------------------------------------
    def test_nothing_to_analyse_is_not_a_refutation(self):
        self.baseline()
        code, text = self.run_main()
        self.assertEqual(code, 2)
        self.assertIn("NOTHING TO ANALYSE", text)

    def test_a_provenance_difference_is_not_an_equality_violation(self):
        """Timestamps differ on every re-run. Counting them voids every wave."""
        self.baseline()
        self.hidden(rate=0.74, provenance_only=True)
        _, text = self.run_main()
        self.assertIn("MET", self.clause(text, "H27-1"))
        self.assertNotIn("NOT MET", self.clause(text, "H27-1"))

    def test_a_scientific_difference_voids_the_whole_wave(self):
        self.baseline()
        self.hidden(rate=0.7401)          # accuracy moved: impossible
        code, text = self.run_main()
        self.assertIn("NOT MET", self.clause(text, "H27-1"))
        self.assertIn("WAVE VOID", text)
        self.assertEqual(code, 4, "a failed equality gate must not exit 0")

    def test_a_missing_hidden_arm_is_unevaluated_not_refuted(self):
        self.baseline()
        for s in SEEDS:      # attn only; the rate arm the gate needs is absent
            self.write(f"w27hid__ff-fixed-attn__{self.A}__d32l4__hidden-shuffled__s{s}",
                       cell_json(0.80, temporal="hidden-shuffled"))
        code, text = self.run_main()
        line = self.clause(text, "H27-1")
        self.assertIn("NOT EVALUABLE", line)
        self.assertNotIn("NOT MET", line)
        self.assertNotEqual(code, 4, "absent inputs must not void the wave")

    # ---- the dropout bar --------------------------------------------------
    def test_dropout_that_costs_nothing_is_a_finding_about_the_instrument(self):
        self.baseline()
        self.hidden()
        for s in SEEDS:      # identical accuracy: the manipulation did nothing
            self.write(f"w27drp__ff-fixed__{self.A}__spike-dropout-p30__s{s}",
                       cell_json(0.74, arm="ff+fixed", temporal="spike-dropout-p30"))
        _, text = self.run_main()
        self.assertIn("NOT MET", self.clause(text, "H27-2"))

    def test_dropout_that_bites_meets_its_bar(self):
        self.baseline()
        self.hidden()
        for s in SEEDS:
            self.write(f"w27drp__ff-fixed__{self.A}__spike-dropout-p30__s{s}",
                       cell_json(0.60, arm="ff+fixed", temporal="spike-dropout-p30"))
        _, text = self.run_main()
        self.assertIn("MET", self.clause(text, "H27-2"))
        self.assertNotIn("NOT MET", self.clause(text, "H27-2"))

    # ---- the tau void rule ------------------------------------------------
    def test_a_rung_past_the_gate_is_void_as_a_whole(self):
        """Section 7: survivors are the seeds that happened to fire less."""
        self.baseline()
        self.hidden()
        for i, s in enumerate(SEEDS):
            over = 0.9 if i < 8 else 0.0       # 8 of 12 past the 0.05 gate
            self.write(f"w27tau__ff-fixed__{self.A}__tau40.0__s{s}",
                       cell_json(0.5, arm="ff+fixed", saturated=over))
        _, text = self.run_main()
        rung = [l for l in text.splitlines() if "tau  40.0" in l or "tau 40.0" in l]
        self.assertTrue(rung, text)
        self.assertIn("VOID RUNG", rung[0])

    def test_a_clean_rung_is_not_void(self):
        self.baseline()
        self.hidden()
        for s in SEEDS:
            self.write(f"w27tau__ff-fixed__{self.A}__tau2.5__s{s}",
                       cell_json(0.70, arm="ff+fixed", saturated=0.01))
        _, text = self.run_main()
        rung = [l for l in text.splitlines() if "tau   2.5" in l]
        self.assertTrue(rung, text)
        self.assertNotIn("VOID RUNG", rung[0])

    # ---- width safety -----------------------------------------------------
    def test_an_off_width_cell_is_dropped_and_counted_not_averaged(self):
        self.baseline()
        self.hidden()
        for s in SEEDS:
            self.write(f"w27hid__ff-fixed__h1024__e400__published-2ms"
                       f"__adjacent-sum-5__hidden-shuffled__s{s}",
                       cell_json(0.1, arm="ff+fixed", temporal="hidden-shuffled",
                                 hidden=1024))
        _, text = self.run_main()
        self.assertIn("not at h128", text)

    # ---- tau-half, unchanged ---------------------------------------------
    def test_tau_half_interpolates_and_withholds_as_registered(self):
        m = self.module
        ladder = {2: 0.01, 4: 0.02, 8: 0.03, 16: 0.04,
                  32: 0.05, 64: 0.055, 128: 0.0596, 192: 0.09, 256: 0.11}
        value, why = m.tau_half(ladder, 0.1208)
        self.assertIsNotNone(value)
        self.assertGreater(value, 128)
        self.assertLess(value, 192)
        self.assertEqual(m.tau_half(ladder, None)[0], None)
        self.assertIn("could not be computed", m.tau_half(ladder, None)[1])
        self.assertIn("not above the bar", m.tau_half(ladder, 0.01)[1])

    def test_tau_half_is_withheld_when_the_ladder_never_reaches_it(self):
        m = self.module
        short = {w: 0.01 for w in m.WINDOWS}
        value, why = m.tau_half(short, 0.5)
        self.assertIsNone(value)
        self.assertIn("not reached", why)


if __name__ == "__main__":
    unittest.main(verbosity=2)
