"""Tests for the wave-14 analyser, against synthetic grids built to trip it.

Wave 11's analyser was frozen before its data and carried two bugs. Wave 13's
avoided those and was mutation-tested. This one has two failure modes neither of
those had, and both change a verdict silently rather than crashing:

  * **Pooling instead of pairing.** If a comparison averaged each arm over
    whatever completed rather than over seeds where *both* completed, it would
    compare two differently filtered subsets — which is the survivorship wave 11
    recorded, reintroduced through the back door.
  * **A gate that reports anyway.** M-0 must block a comparison, not decorate
    it. A NOT EVALUABLE banner printed beside a mean is the "check that cannot
    fail" shape this repository keeps finding.

Both are checked by mutation in `test_the_traps_are_caught_by_mutation`, which
edits a copy of the analyser and requires the suite to go red.

Run: python3 scripts/test_wave14_analyser.py
"""

from __future__ import annotations

import json
import struct
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "scripts" / "aws"))
sys.path.insert(0, str(ROOT / "scripts"))

import analyse_wave14 as a14  # noqa: E402

SEEDS = a14.SEEDS


def as_f32(value: float) -> float:
    return struct.unpack("<f", struct.pack("<f", value))[0]


def spec_for(arm: str, seed: int) -> dict:
    attn = "__d32l4" if arm.endswith("+attn") else ""
    ident = (
        f"w14sub__{arm.replace('+', '-')}__h128__e400__published-2ms__"
        f"adjacent-sum-5{attn}__ss0.4__s{seed}"
    )
    return {
        "id": ident, "wave": "w14sub", "arm": arm, "hidden": 128, "epochs": 400,
        "seed": seed, "contract": "published-2ms", "geometry": "adjacent-sum-5",
        "attn_dim": 32 if attn else None, "attn_layers": 4 if attn else None,
        "temporal": "intact", "temporal_seed": None, "surrogate_scale": 0.4,
        "clip_grad_norm": None, "n_train": 8156, "n_inputs": 140,
    }


def cell_for(arm: str, accuracy: float) -> dict:
    return {
        "schema": "shd-cal-cell-v1", "arm": arm, "mechanical_status": "COMPLETE",
        "accuracy": accuracy, "classes_predicted": 20, "majority_prediction": 0.11,
        "silent_fraction": 0.02, "saturated_fraction": 0.0, "non_finite_events": 0,
        "temporal_condition": "intact", "surrogate_scale": as_f32(0.4),
        "hidden": 128, "epochs": 400, "contract": "published-2ms",
        "geometry": "adjacent-sum-5",
        "epoch_max_gradient_norm": [1.0, 4.0, 12.0],
    }


class Wave14AnalyserTest(unittest.TestCase):
    def build(self, accuracies: dict[str, dict[int, float]]):
        """A grid. `accuracies[arm][seed]` present => that cell completed."""
        self.tmp = tempfile.TemporaryDirectory()
        root = Path(self.tmp.name)
        results = root / "results"
        results.mkdir()
        plan = []
        for arm in ("rec+alif+attn", "ff+fixed", "ff+fixed+attn"):
            for seed in SEEDS:
                spec = spec_for(arm, seed)
                plan.append(spec)
                if seed in accuracies.get(arm, {}):
                    (results / f"{spec['id']}.json").write_text(
                        json.dumps(cell_for(arm, accuracies[arm][seed]))
                    )
        # The reused wave-13 arm, written under ITS ids, not wave 14's.
        for seed in SEEDS:
            if seed in accuracies.get("rec+alif", {}):
                stem = f"{a14.REUSED_STEM}__s{seed}"
                (results / f"{stem}.json").write_text(
                    json.dumps(cell_for("rec+alif", accuracies["rec+alif"][seed]))
                )
        plan_path = root / "plan.json"
        plan_path.write_text(json.dumps(plan))
        return plan, results, plan_path

    def full(self, rec=0.50, rec_attn=0.70, ff=0.70, ff_attn=0.83):
        """Every arm complete on every seed, with a small per-seed spread."""
        return {
            "rec+alif": {s: rec + 0.001 * i for i, s in enumerate(SEEDS)},
            "rec+alif+attn": {s: rec_attn + 0.001 * i for i, s in enumerate(SEEDS)},
            "ff+fixed": {s: ff + 0.001 * i for i, s in enumerate(SEEDS)},
            "ff+fixed+attn": {s: ff_attn + 0.001 * i for i, s in enumerate(SEEDS)},
        }

    def outcomes(self, accuracies):
        plan, results, _ = self.build(accuracies)
        return a14.collect(plan, results, None)

    # --- pairing -------------------------------------------------------------

    def test_a_gain_is_computed_only_over_seeds_where_both_arms_completed(self):
        """The survivorship trap, made concrete.

        The treatment is missing on the six seeds where it scores worst, so a
        POOLED mean would be flattered by their absence while the control keeps
        all twelve. Paired, those six drop from both sides and the gain is the
        honest one.
        """
        acc = self.full()
        for seed in SEEDS[:6]:
            acc["rec+alif+attn"][seed] = 0.30          # the bad ones ...
        for seed in SEEDS[:6]:
            del acc["rec+alif+attn"][seed]             # ... and they never landed
        outcomes = self.outcomes(acc)

        matched = a14.pairs(outcomes, "rec+alif+attn", "rec+alif")
        self.assertEqual(len(matched), 6, "pairs must be the intersection")
        self.assertEqual([s for s, _, _ in matched], SEEDS[6:])

        paired_gain = sum(t - c for _, t, c in matched) / len(matched)
        completed_treatment = [
            o["cell"]["accuracy"] for (a, _), o in outcomes.items()
            if a == "rec+alif+attn" and o["state"] == "completed"
        ]
        completed_control = [
            o["cell"]["accuracy"] for (a, _), o in outcomes.items()
            if a == "rec+alif" and o["state"] == "completed"
        ]
        pooled_gain = (sum(completed_treatment) / len(completed_treatment)
                       - sum(completed_control) / len(completed_control))
        self.assertGreater(
            pooled_gain, paired_gain + 0.002,
            "fixture must make pooling visibly more flattering, or this proves nothing",
        )

    # --- the M-0 gate --------------------------------------------------------

    def test_m0_blocks_a_comparison_whose_arm_is_short(self):
        acc = self.full()
        for seed in SEEDS[:2]:
            del acc["rec+alif+attn"][seed]       # 10/12, under the 11 bar
        outcomes = self.outcomes(acc)
        ok, why = a14.evaluable(outcomes, "rec+alif+attn", "rec+alif")
        self.assertFalse(ok)
        self.assertIn("10/12", why)

    def test_m0_blocks_a_comparison_with_too_few_surviving_pairs(self):
        """Both arms clear 11/12 and still only nine pairs survive.

        The arms fail on *different* seeds, so per-arm completion is not enough
        and the pair count is a separate bar. Without it this comparison would
        report a mean over nine.
        """
        acc = self.full()
        del acc["rec+alif+attn"][SEEDS[0]]
        del acc["rec+alif"][SEEDS[1]]
        outcomes = self.outcomes(acc)
        self.assertEqual(a14.completions(outcomes, "rec+alif+attn"), 11)
        self.assertEqual(a14.completions(outcomes, "rec+alif"), 11)
        self.assertEqual(len(a14.pairs(outcomes, "rec+alif+attn", "rec+alif")), 10)

        del acc["rec+alif"][SEEDS[2]]            # 10/12 for the control now
        outcomes = self.outcomes(acc)
        ok, why = a14.evaluable(outcomes, "rec+alif+attn", "rec+alif")
        self.assertFalse(ok)

    def test_a_blocked_comparison_prints_no_numbers(self):
        """NOT EVALUABLE must block, not decorate."""
        acc = self.full()
        for seed in SEEDS[:4]:
            del acc["rec+alif+attn"][seed]
        report = self.report(acc)
        self.assertIn("NOT EVALUABLE", report)
        m1 = next(line for line in report.splitlines() if line.startswith("**M-1**"))
        self.assertNotIn("gain **", m1)
        self.assertIn("No mean is reported", m1)

    # --- the reused arm ------------------------------------------------------

    def test_the_reused_arm_is_read_from_wave_13_ids(self):
        outcomes = self.outcomes(self.full())
        self.assertEqual(a14.completions(outcomes, "rec+alif"), 12)

    def test_a_reused_cell_that_is_not_the_expected_configuration_is_voided(self):
        plan, results, _ = self.build(self.full())
        stem = f"{a14.REUSED_STEM}__s{SEEDS[0]}"
        cell = json.loads((results / f"{stem}.json").read_text())
        cell["hidden"] = 256                       # a different cell entirely
        (results / f"{stem}.json").write_text(json.dumps(cell))
        outcomes = a14.collect(plan, results, None)
        self.assertEqual(outcomes[("rec+alif", SEEDS[0])]["state"], "voided")
        self.assertIn("expected spec", outcomes[("rec+alif", SEEDS[0])]["why"])

    # --- thresholds ----------------------------------------------------------

    def test_the_registered_bars_are_the_ones_in_the_prereg(self):
        self.assertEqual((a14.M0_COMPLETIONS, a14.M0_OF, a14.M0_PAIRS), (11, 12, 10))
        self.assertEqual(a14.M1_GAIN, 0.05)
        self.assertEqual(a14.M1_PAIRS_POSITIVE, 10)
        self.assertEqual(a14.M2_DELTA, 0.03)
        self.assertEqual((a14.M3_GATE, a14.M3_SEEDS), (0.80, 9))

    def test_a_clean_grid_produces_the_arithmetic_it_should(self):
        report = self.report(self.full(rec=0.50, rec_attn=0.70, ff=0.70, ff_attn=0.83))
        self.assertIn("+0.2000", report)          # gain(rec+alif)
        self.assertIn("+0.1300", report)          # gain(ff+fixed)
        self.assertIn("+0.0700", report)          # the difference
        self.assertIn("**SUPPORTED**", report)

    def report(self, accuracies) -> str:
        plan, results, plan_path = self.build(accuracies)
        out = Path(self.tmp.name) / "report.md"
        subprocess.run(
            [sys.executable, str(ROOT / "scripts" / "aws" / "analyse_wave14.py"),
             "--plan", str(plan_path), "--results", str(results), "--out", str(out)],
            check=True, capture_output=True, text=True,
        )
        return out.read_text()

    def tearDown(self):
        if hasattr(self, "tmp"):
            self.tmp.cleanup()


class MutationTest(unittest.TestCase):
    """The two traps, checked by breaking the analyser rather than by reading it."""

    MUTATIONS = {
        "pool instead of pair": (
            '        if t and c and t["state"] == "completed" and c["state"] == "completed":',
            '        if t and c and (t["state"] == "completed" or c["state"] == "completed"):',
        ),
        "let the gate decorate instead of block": (
            "    return (not reasons), \"; \".join(reasons)",
            "    return True, \"; \".join(reasons)",
        ),
    }

    def test_the_traps_are_caught_by_mutation(self):
        source = (ROOT / "scripts" / "aws" / "analyse_wave14.py").read_text()
        for label, (before, after) in self.MUTATIONS.items():
            self.assertIn(before, source, f"{label}: anchor text has moved")
            with tempfile.TemporaryDirectory() as tmp:
                tree = Path(tmp) / "scripts"
                subprocess.run(["cp", "-R", str(ROOT / "scripts"), str(tree)], check=True)
                target = tree / "aws" / "analyse_wave14.py"
                target.write_text(source.replace(before, after))
                result = subprocess.run(
                    [sys.executable, str(tree / "test_wave14_analyser.py"),
                     "Wave14AnalyserTest"],
                    capture_output=True, text=True,
                )
                self.assertNotEqual(
                    result.returncode, 0,
                    f"mutation '{label}' left the suite green; it is not being checked",
                )


if __name__ == "__main__":
    unittest.main(verbosity=2)
