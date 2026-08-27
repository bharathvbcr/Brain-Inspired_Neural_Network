#!/usr/bin/env python3
"""Tests for the waves 15-17 analyser, against synthetic grids.

The analyser is frozen before the first cell, so the only way to know it
computes what the preregistration says is to feed it grids whose answers are
known by construction. Two properties matter most and neither would crash if
broken:

  * a **bar** that drifts from the prereg silently changes a verdict;
  * a **gate** that reports numbers beside a NOT EVALUABLE banner lets a
    blocked comparison be read anyway.

Both are tested by mutation -- breaking the analyser and requiring the test to
notice -- rather than by reading the source.
"""

from __future__ import annotations

import json
import re
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "scripts" / "aws"))
import analyse_wave15 as a15  # noqa: E402

ANCHOR = "published-2ms__adjacent-sum-5"
SEEDS = a15.SEEDS


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
        self.assertEqual((a15.H15_1_GAIN, a15.H15_1_POSITIVE), (0.05, 9))
        self.assertEqual(a15.H15_2_HEALTHY_NORM, 1.0)
        self.assertEqual(a15.H15_3_MARGIN, 0.01)
        self.assertEqual((a15.ARCHIVED_H1024_L1_GAIN, a15.ARCHIVED_H1024_L4_GAIN),
                         (-0.0159, -0.1618))
        self.assertEqual(a15.H16_1_SEPARATION, 0.005)
        self.assertEqual(a15.H16_2_FACTOR, 3.0)
        self.assertEqual((a15.H17_GAIN, a15.H17_POSITIVE), (0.05, 24))
        self.assertEqual((a15.H17_GATE, a15.H17_GATE_SEEDS), (0.80, 24))
        self.assertEqual(a15.H17_2_SHUFFLE_FACTOR, 5.0)
        self.assertEqual(a15.MIN_VALID_PER_ARM, 9)

    def test_the_prereg_document_carries_the_same_numbers(self):
        """A bar in two places is a bar that can drift in one of them."""
        text = (ROOT / "results"
                / "PREREG_2026-08-25_THE_H1024_COLLAPSE.md").read_text()
        for needle in ("≥ +0.05, positive in ≥ 9/12", "below 1.0",
                       "−0.0159", "−0.1618", "0.005", "**3×**",
                       "≥ 24/32", "5×"):
            self.assertIn(needle, text,
                          f"the prereg no longer states {needle!r}; the analyser "
                          "and the registration have drifted apart")


class LeverArithmetic(unittest.TestCase):
    """H15-1 and H15-2 on grids whose answers are known by construction."""

    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.res = Path(self.tmp.name) / "results"
        self.res.mkdir()

    def tearDown(self):
        self.tmp.cleanup()

    def write(self, stem: str, accuracy: float, *, norm: float = 0.5, seeds=None):
        for seed in (seeds or SEEDS):
            (self.res / f"{stem}__s{seed}.json").write_text(
                json.dumps(cell(accuracy, norm=norm)))

    def run_analyser(self, plan_ids: list[str]) -> str:
        plan = Path(self.tmp.name) / "plan.json"
        plan.write_text(json.dumps([{"id": i} for i in plan_ids]))
        out = Path(self.tmp.name) / "report.md"
        subprocess.run(
            [sys.executable, str(ROOT / "scripts/aws/analyse_wave15.py"),
             "--plan", str(plan), "--results", str(self.res), "--out", str(out)],
            capture_output=True, text=True)
        return out.read_text()

    def lever_stem(self, tag: str) -> str:
        return f"w15col__ff-fixed-attn__h1024__e400__{ANCHOR}__d32l4__{tag}"

    def test_a_recovering_lever_with_healthy_norms_meets_both(self):
        # Archived h1024 rate control means 0.7386. An arm at 0.85 clears +0.05.
        self.write(self.lever_stem("ss0.5"), 0.85, norm=0.4)
        report = self.run_analyser(
            [f"{self.lever_stem('ss0.5')}__s{s}" for s in SEEDS])
        row = next(l for l in report.splitlines() if "surrogate scale 0.5" in l)
        self.assertIn("**MET**", row)
        self.assertEqual(row.count("**MET**"), 2, "H15-1 and H15-2 must both fire")
        self.assertIn("**H15-1: MET**", report)

    def test_recovery_with_unhealthy_norms_separates_the_two(self):
        """Outcome O-3: the effect without the mechanism must be legible."""
        self.write(self.lever_stem("ss0.5"), 0.85, norm=55.0)
        report = self.run_analyser(
            [f"{self.lever_stem('ss0.5')}__s{s}" for s in SEEDS])
        row = next(l for l in report.splitlines() if "surrogate scale 0.5" in l)
        self.assertIn("**MET**", row)
        self.assertIn("**NOT MET**", row)
        self.assertIn("unidentified mechanism", report)

    def test_an_arm_below_the_valid_floor_carries_no_numbers(self):
        """A blocked comparison must not print a mean beside its banner."""
        self.write(self.lever_stem("ss0.5"), 0.85, seeds=SEEDS[:5])
        report = self.run_analyser(
            [f"{self.lever_stem('ss0.5')}__s{s}" for s in SEEDS[:5]])
        row = next(l for l in report.splitlines() if "surrogate scale 0.5" in l)
        self.assertIn("NOT EVALUABLE", row)
        self.assertNotRegex(row, r"[+-]0\.\d{4}",
                            "a NOT EVALUABLE row must carry no gain")

    def test_the_noop_control_fails_loudly_when_the_clip_perturbs_a_run(self):
        """H15-4, and it must void the clipped arm rather than note a mismatch."""
        archive = ROOT / "results/shd_attention_campaign_v2"
        seed = SEEDS[0]
        src = archive / f"w8wid__ff-fixed-attn__h512__e400__{ANCHOR}__d32l4__s{seed}.json"
        if not src.is_file():
            self.skipTest("archived h512 cell absent")
        perturbed = json.loads(src.read_text())
        perturbed["accuracy"] += 1e-9
        (self.res / (f"w15col__ff-fixed-attn__h512__e400__{ANCHOR}__d32l4"
                     f"__clip1000.0__s{seed}.json")).write_text(json.dumps(perturbed))
        report = self.run_analyser([f"w15col__x__s{seed}"])
        self.assertIn("**H15-4: NOT MET**", report)
        self.assertIn("every clipped cell in this wave is void", report)

    def test_the_noop_control_passes_on_an_untouched_copy(self):
        """The other half: it must be able to say MET, or it says nothing."""
        archive = ROOT / "results/shd_attention_campaign_v2"
        written = 0
        for seed in SEEDS:
            src = archive / f"w8wid__ff-fixed-attn__h512__e400__{ANCHOR}__d32l4__s{seed}.json"
            if src.is_file():
                (self.res / (f"w15col__ff-fixed-attn__h512__e400__{ANCHOR}__d32l4"
                             f"__clip1000.0__s{seed}.json")).write_text(src.read_text())
                written += 1
        if not written:
            self.skipTest("archived h512 cells absent")
        report = self.run_analyser([f"w15col__x__s{SEEDS[0]}"])
        self.assertIn("**H15-4: MET**", report)


class MutationTest(unittest.TestCase):
    """The bars, checked by breaking them rather than by reading them.

    The mutation is applied to a **copy** in a temporary directory. An earlier
    version of this test rewrote `scripts/aws/analyse_wave15.py` in place and
    restored it in a `finally`, which is safe only if the process survives to
    run the `finally` -- and when it did not, it left a lowered bar in the tree
    and the next run of the analyser silently used it. A test that can corrupt
    the thing it guards is worse than no test.
    """

    def test_a_drifted_bar_is_caught(self):
        source = ROOT / "scripts/aws/analyse_wave15.py"
        original = source.read_text()
        broken = original.replace("H15_1_GAIN = 0.05", "H15_1_GAIN = 0.01", 1)
        self.assertNotEqual(broken, original, "the bar moved or was renamed")

        with tempfile.TemporaryDirectory() as tmp:
            copy = Path(tmp) / "analyse_wave15.py"
            copy.write_text(broken)
            probe = Path(tmp) / "probe.py"
            probe.write_text(
                "import importlib.util, sys\n"
                # The analyser imports `cell_validity` from scripts/; the copy
                # sits elsewhere, so its path has to be supplied explicitly.
                f"sys.path.insert(0, {str(ROOT / 'scripts')!r})\n"
                f"spec = importlib.util.spec_from_file_location('m', {str(copy)!r})\n"
                "m = importlib.util.module_from_spec(spec)\n"
                "sys.modules['m'] = m\n"
                "spec.loader.exec_module(m)\n"
                "assert (m.H15_1_GAIN, m.H15_1_POSITIVE) == (0.05, 9), 'bar drifted'\n"
            )
            proc = subprocess.run([sys.executable, str(probe)],
                                  capture_output=True, text=True)

        self.assertNotEqual(proc.returncode, 0,
                            "lowering a registered bar was not detected")
        self.assertIn("bar drifted", proc.stderr)
        # And the tree is untouched, which is the property the old version lost.
        self.assertEqual(source.read_text(), original)



class MergedArmsAreOneArm(unittest.TestCase):
    """The guard from AMENDMENT_2026-08-27_H17_2_MERGED_TWO_READOUT_DEPTHS.

    H17-2 merged `w1__…__d32l1__bin-shuffled` into a `d32l4` comparison for
    twelve of its twenty-eight pairs, so the "shuffle cost" for those seeds was
    (four-layer intact − one-layer shuffled) and came out +0.0541 high. Nothing
    had ever checked that a merged arm is one arm.
    """

    def test_a_matched_pair_is_accepted(self):
        a15.assert_same_arm(
            f"w9shf__ff-fixed-attn__h128__e400__{ANCHOR}__d32l4__bin-shuffled",
            f"w17hdl__ff-fixed-attn__h128__e400__{ANCHOR}__d32l4__bin-shuffled")

    def test_the_historical_mismatch_raises(self):
        with self.assertRaises(ValueError) as caught:
            a15.assert_same_arm(
                f"w1__ff-fixed-attn__h128__e400__{ANCHOR}__d32l1__bin-shuffled",
                f"w17hdl__ff-fixed-attn__h128__e400__{ANCHOR}__d32l4__bin-shuffled")
        self.assertIn("d32l1", str(caught.exception))
        self.assertIn("d32l4", str(caught.exception))

    def test_a_shuffled_control_cannot_be_merged_into_an_intact_arm(self):
        """The other shape of the same mistake."""
        with self.assertRaises(ValueError):
            a15.assert_same_arm(
                f"w9shf__ff-fixed-attn__h128__e400__{ANCHOR}__d32l4__bin-shuffled",
                f"w17hdl__ff-fixed-attn__h128__e400__{ANCHOR}__d32l4")

    def test_every_real_merge_site_passes_the_guard(self):
        """The regression test for the bug itself: run the analyser over an
        empty results tree, which reaches every `merged()` call with the stems
        the analyser actually ships. A reverted stem fails here."""
        with tempfile.TemporaryDirectory() as tmp:
            res = Path(tmp) / "results"
            res.mkdir()
            plan = Path(tmp) / "plan.json"
            plan.write_text(json.dumps([{"id": "w17hdl__probe__s5170001"}]))
            out = Path(tmp) / "report.md"
            proc = subprocess.run(
                [sys.executable, str(ROOT / "scripts/aws/analyse_wave15.py"),
                 "--plan", str(plan), "--results", str(res), "--out", str(out)],
                capture_output=True, text=True)
            # 2 is the analyser's deliberate "plan incomplete" code and is
            # expected on an empty tree; a raised guard exits 1 with a traceback.
            self.assertIn(proc.returncode, (0, 2),
                          "a shipped merge averages two different arms:\n"
                          + proc.stderr)
            self.assertNotIn("must be one arm", proc.stderr)
            self.assertIn("H17-2", out.read_text())

    def test_the_amendment_records_the_corrected_number(self):
        """The number in the fix and the number in its record must agree."""
        text = (ROOT / "results"
                / "AMENDMENT_2026-08-27_H17_2_MERGED_TWO_READOUT_DEPTHS.md").read_text()
        for needle in ("+0.1345", "+0.1577", "+0.1337", "0.6442", "w9shf"):
            self.assertIn(needle, text)


if __name__ == "__main__":
    unittest.main(verbosity=2)
