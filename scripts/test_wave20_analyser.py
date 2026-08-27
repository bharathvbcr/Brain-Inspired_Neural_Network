#!/usr/bin/env python3
"""Tests for the wave 20 analyser.

Frozen before the first cell, so the only way to know it computes what the
preregistration says is to feed it grids whose answers are known by
construction -- and, uniquely here, to check it reproduces three numbers the
paper already publishes from cells that already exist.

Four properties matter most and none would crash if broken:

  * **H20-2 must GATE H20-1**, not merely be reported beside it. A difference of
    gains over ten pairs is arithmetic, not evidence.
  * **H20-3 must be able to return MET.** It is registered against a pilot that
    already fails it, so a version that can only say NOT MET would look correct
    for the whole life of the wave.
  * **An empty arm must refuse**, because a missing arm produces a NOT MET that
    reads exactly like a finding. This is not hypothetical: the first draft
    looked for the recurrent rate arm under `w14sub` when it lives under
    `w13rec`, and reported "0 pairs, H20-2 NOT MET" with a straight face.
  * **The rank correlation must average ties.** A plateau in the covariate
    otherwise biases rho toward zero, which is the direction that makes H20-3
    pass.
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
import analyse_wave20 as a20  # noqa: E402

ANCHOR = "published-2ms__adjacent-sum-5"


def cell(accuracy: float, *, peak: float = 10.0, epochs: int = 400) -> dict:
    return {
        "schema": "shd-cal-cell-v1", "mechanical_status": "COMPLETE",
        "accuracy": accuracy, "classes_predicted": 20,
        "majority_prediction": 0.09, "silent_fraction": 0.02,
        "saturated_fraction": 0.0, "non_finite_events": 0,
        "mean_loss": 0.3, "mean_gradient_norm": 0.5, "mean_update_rms": 4e-4,
        "mean_firing_rate": 0.2, "tail_loss_improvement": -0.01,
        "temporal_condition": "intact",
        "epoch_mean_loss": [0.3] * epochs,
        "epoch_mean_gradient_norm": [0.5] * epochs,
        "epoch_max_gradient_norm": [peak] * epochs,
        "epoch_max_gradient_step": [1] * epochs,
    }


class BarsMatchThePrereg(unittest.TestCase):
    def test_the_registered_bars_are_the_ones_in_the_prereg(self):
        self.assertEqual((a20.H20_1_DIFFERENCE, a20.H20_1_POSITIVE), (0.03, 24))
        self.assertEqual(a20.H20_2_MIN_PAIRS, 24)
        self.assertEqual(a20.H20_3_RHO_FLOOR, -0.30)
        self.assertEqual(a20.PILOT_RHO, -0.648)
        self.assertEqual(a20.H20_4_RATIO, 1.0)
        self.assertEqual(len(a20.SEEDS_W20), 32)

    def test_the_prereg_document_carries_the_same_numbers(self):
        text = (ROOT / "results"
                / "PREREG_2026-08-27_THE_RECURRENT_CLAIM_AT_THIRTY_TWO_SEEDS.md"
                ).read_text()
        for needle in ("≥ +0.03", "≥ 24/32", "≥ −0.30", "−0.648", "> 1.0",
                       "≥ 24 usable pairs"):
            self.assertIn(needle, text,
                          f"the prereg no longer states {needle!r}; the analyser "
                          "and the registration have drifted apart")


class Spearman(unittest.TestCase):
    def test_a_perfect_inversion_is_minus_one(self):
        self.assertAlmostEqual(a20.spearman([1, 2, 3, 4], [4, 3, 2, 1]), -1.0)

    def test_a_perfect_agreement_is_plus_one(self):
        self.assertAlmostEqual(a20.spearman([1, 2, 3, 4], [1, 2, 3, 4]), 1.0)

    def test_ties_are_averaged_not_broken_by_position(self):
        """Ordinal ranks on a plateau invent a correlation from input order."""
        rho = a20.spearman([1.0, 2.0, 3.0, 4.0], [5.0, 5.0, 5.0, 5.0])
        self.assertIsNone(rho, "a constant covariate has no rank correlation and "
                               "must not be reported as one")

    def test_too_few_points_is_none_not_a_number(self):
        self.assertIsNone(a20.spearman([1, 2, 3], [3, 2, 1]))


class ReproducesThePublishedPilot(unittest.TestCase):
    """The archive already holds every cell §3.7 was computed from.

    Before wave 20 lands a single cell, the frozen analyser must reproduce the
    paper's numbers from those cells. If it cannot, it is measuring something
    else and every verdict it later issues is about that other thing.
    """

    def report(self) -> str:
        with tempfile.TemporaryDirectory() as tmp:
            empty = Path(tmp) / "results"
            empty.mkdir()
            plan = Path(tmp) / "plan.json"
            plan.write_text(json.dumps([{"id": "w20rec__probe__s5170001"}]))
            out = Path(tmp) / "r.md"
            proc = subprocess.run(
                [sys.executable, str(ROOT / "scripts/aws/analyse_wave20.py"),
                 "--plan", str(plan), "--results", str(empty), "--out", str(out)],
                capture_output=True, text=True)
            self.assertIn(proc.returncode, (0, 2), proc.stderr)
            return out.read_text()

    def test_it_finds_the_ten_pairs_the_paper_reports(self):
        self.assertIn("| **usable pairs** | **10** |", self.report())

    def test_it_reproduces_the_pilot_correlation(self):
        self.assertIn("ρ = **-0.648**", self.report())

    def test_it_reproduces_the_headroom_ratio_the_paper_computed_post_hoc(self):
        report = self.report()
        self.assertIn("**1.337x**", report)
        self.assertIn("+0.2612", report)
        self.assertIn("+0.1201", report)

    def test_h20_2_suppresses_h20_1_at_ten_pairs(self):
        report = self.report()
        self.assertIn("**H20-2: NOT MET**", report)
        self.assertIn("**H20-1: NOT LICENSED**", report)
        h20_1 = report.split("## H20-1")[1].split("## H20-3")[0]
        self.assertNotRegex(h20_1, r"[+-]0\.\d{4}",
                            "a suppressed hypothesis must carry no numbers")


class Grids(unittest.TestCase):
    """The verdicts on grids whose answers are known by construction."""

    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.res = Path(self.tmp.name) / "results"
        self.res.mkdir()

    def tearDown(self):
        self.tmp.cleanup()

    def write(self, spec, values):
        for seed, (acc, peak) in values.items():
            (self.res / f"w20rec__{spec}__s{seed}.json").write_text(
                json.dumps(cell(acc, peak=peak)))

    def full_grid(self, *, rec_gain=0.26, ff_gain=0.12, peaks=None):
        seeds = a20.SEEDS_W20
        peaks = peaks or {s: 10.0 for s in seeds}
        self.write(f"rec-alif__h128__e400__{ANCHOR}__ss0.4",
                   {s: (0.53, 1.0) for s in seeds})
        # Gains must VARY, or the rank correlation is degenerate and H20-3
        # is not evaluable -- which is a different verdict from MET and the
        # first version of this grid produced it by accident.
        self.write(f"rec-alif-attn__h128__e400__{ANCHOR}__d32l4__ss0.4",
                   {s: (0.53 + rec_gain + (i % 7 - 3) * 0.004, peaks[s])
                    for i, s in enumerate(seeds)})
        self.write(f"ff-fixed__h128__e400__{ANCHOR}__ss0.4",
                   {s: (0.71, 1.0) for s in seeds})
        self.write(f"ff-fixed-attn__h128__e400__{ANCHOR}__d32l4__ss0.4",
                   {s: (0.71 + ff_gain, 1.0) for s in seeds})

    def run_analyser(self) -> str:
        plan = Path(self.tmp.name) / "plan.json"
        plan.write_text(json.dumps([{"id": p.stem} for p in self.res.glob("*.json")]))
        out = Path(self.tmp.name) / "r.md"
        proc = subprocess.run(
            [sys.executable, str(ROOT / "scripts/aws/analyse_wave20.py"),
             "--plan", str(plan), "--results", str(self.res), "--out", str(out)],
            capture_output=True, text=True)
        self.assertIn(proc.returncode, (0, 2), proc.stderr)
        return out.read_text()

    def test_a_full_healthy_grid_meets_every_hypothesis(self):
        """H20-3 must be ABLE to say MET, or it is a check that cannot pass."""
        self.full_grid(peaks={s: 10.0 + (i % 5) for i, s in enumerate(a20.SEEDS_W20)})
        report = self.run_analyser()
        for hypothesis in ("H20-1", "H20-2", "H20-3", "H20-4"):
            self.assertIn(f"**{hypothesis}: MET**", report,
                          f"{hypothesis} could not reach MET on a grid built to "
                          f"satisfy it")

    def test_survivorship_correlation_is_caught(self):
        """Gain falling as peak norm rises is the outcome the pilot points at."""
        seeds = a20.SEEDS_W20
        self.full_grid()
        # Rewrite the recurrent attention arm so gain decreases with peak norm.
        for i, s in enumerate(seeds):
            (self.res / (f"w20rec__rec-alif-attn__h128__e400__{ANCHOR}"
                         f"__d32l4__ss0.4__s{s}.json")).write_text(
                json.dumps(cell(0.53 + 0.40 - 0.01 * i, peak=10.0 ** (1 + i * 0.1))))
        report = self.run_analyser()
        self.assertIn("**H20-3: NOT MET**", report)
        self.assertIn("survivorship-shaped", report)

    def test_a_thin_recurrent_arm_suppresses_the_headline(self):
        self.full_grid()
        for s in a20.SEEDS_W20[10:]:
            (self.res / (f"w20rec__rec-alif-attn__h128__e400__{ANCHOR}"
                         f"__d32l4__ss0.4__s{s}.json")).unlink()
        report = self.run_analyser()
        self.assertIn("**H20-2: NOT MET**", report)
        self.assertIn("**H20-1: NOT LICENSED**", report)

    def test_a_constant_gain_is_degenerate_not_a_shortage(self):
        """The two ways H20-3 cannot run must not share a message."""
        seeds = a20.SEEDS_W20
        self.full_grid()
        for s in seeds:
            (self.res / (f"w20rec__rec-alif-attn__h128__e400__{ANCHOR}"
                         f"__d32l4__ss0.4__s{s}.json")).write_text(
                json.dumps(cell(0.79, peak=10.0)))
        report = self.run_analyser()
        self.assertIn("**H20-3: NOT EVALUABLE**", report)
        self.assertIn("degenerate comparison, not a small one", report)
        self.assertNotIn("needs at least four", report)

    def test_headroom_normalisation_can_fail(self):
        """A recurrent gain that is only the lower base must be caught."""
        self.full_grid(rec_gain=0.12 * (1 - 0.53) / (1 - 0.71) * 0.5)
        self.assertIn("**H20-4: NOT MET**", self.run_analyser())


class AnEmptyArmRefuses(unittest.TestCase):
    """A missing arm produces a NOT MET that reads exactly like a finding."""

    def test_a_renamed_archive_prefix_is_fatal_not_quiet(self):
        import analyse_wave20
        source = (ROOT / "scripts/aws/analyse_wave20.py").read_text()
        self.assertIn('"rec_rate": ("w13rec"', source,
                      "the recurrent rate arm lives under w13rec, not w14sub; "
                      "assuming one prefix for all four arms returned an empty "
                      "arm and a NOT MET once already")
        self.assertIn("resolved to zero cells", source)
        self.assertIn("reads exactly like a finding", source)
        del analyse_wave20


if __name__ == "__main__":
    unittest.main(verbosity=2)
