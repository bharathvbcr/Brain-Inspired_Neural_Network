"""Tests for the wave-13 analyser, against a synthetic grid built to trip it.

Freezing an analyser before the data does not make it correct — wave 11's was
frozen and carried two bugs, both of which would have fired had its completion
bar not failed first. This file exists so wave 13's does not repeat them, and it
reproduces the traps rather than a tidied version of them:

  * `surrogate_scale` is an f32. The instrument writes `0.4` as **0.400000006**,
    so any grouping that compares it to `0.4` silently matches nothing and every
    condition comes out empty. The fixture writes the f32 value.
  * The emitted cell has no reliable `seed` field to key on, so the fixture omits
    it entirely; an analyser that reaches for `cell["seed"]` raises here.

Both are checked by mutation: the assertions below fail if the analyser is
changed to group on the float or to key on the missing field.

Run: python3 scripts/test_wave13_analyser.py
"""

from __future__ import annotations

import json
import struct
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "scripts" / "aws"))
sys.path.insert(0, str(ROOT / "scripts"))

import analyse_wave13 as a13  # noqa: E402

SEEDS = [5170001 + i for i in range(12)]


def as_f32(value: float) -> float:
    """The value after a round trip through f32, which is what the cell holds."""
    return struct.unpack("<f", struct.pack("<f", value))[0]


def plan_entry(arm: str, scale: float, seed: int) -> dict:
    tag = f"ss{scale}"
    ident = (
        f"w13rec__{arm.replace('+', '-')}__h128__e400__published-2ms__"
        f"adjacent-sum-5__{tag}__s{seed}"
    )
    return {
        "id": ident,
        "wave": "w13rec",
        "arm": arm,
        "hidden": 128,
        "epochs": 400,
        "seed": seed,
        "contract": "published-2ms",
        "geometry": "adjacent-sum-5",
        "attn_dim": None,
        "attn_layers": None,
        "temporal": "intact",
        "temporal_seed": None,
        "surrogate_scale": scale,
        "clip_grad_norm": None,
        "n_train": 8156,
        "n_inputs": 140,
    }


def healthy_cell(scale: float, accuracy: float, peak: float) -> dict:
    """A complete, valid cell — with `surrogate_scale` as the instrument writes it.

    Deliberately carries **no `seed` key**: the emitted schema does not
    guarantee one, and an analyser that keys on it must fail here rather than in
    production.
    """
    return {
        "schema": "shd-cal-cell-v1",
        "arm": "rec+alif",
        "mechanical_status": "COMPLETE",
        "accuracy": accuracy,
        "classes_predicted": 20,
        "majority_prediction": 0.11,
        "silent_fraction": 0.02,
        "saturated_fraction": 0.0,
        "non_finite_events": 0,
        "temporal_condition": "intact",
        "surrogate_scale": as_f32(scale),
        "epoch_max_gradient_norm": [1.0, peak, 12.0],
    }


class Wave13AnalyserTest(unittest.TestCase):
    def build(self, completions: dict[tuple[str, float], int], with_failures=True):
        """A 48-cell grid where each condition completes `completions[cond]` times."""
        self.tmp = tempfile.TemporaryDirectory()
        root = Path(self.tmp.name)
        results, failures = root / "results", root / "failures"
        results.mkdir()
        failures.mkdir()
        plan = []
        for arm in ("rec+fixed", "rec+alif"):
            for scale in (1.0, 0.4):
                want = completions[(arm, scale)]
                for index, seed in enumerate(SEEDS):
                    spec = plan_entry(arm, scale, seed)
                    plan.append(spec)
                    if index < want:
                        cell = healthy_cell(scale, 0.40 + 0.01 * index, 1e10)
                        cell["arm"] = arm
                        (results / f"{spec['id']}.json").write_text(json.dumps(cell))
                    elif with_failures:
                        (failures / f"{spec['id']}.log").write_text(
                            "shd-instrument: non-finite training value at optimizer "
                            f"step {500 + index}\n"
                        )
        plan_path = root / "plan.json"
        plan_path.write_text(json.dumps(plan))
        return plan, results, failures

    def outcomes(self, completions, with_failures=True):
        plan, results, failures = self.build(completions, with_failures)
        return plan, a13.load_outcomes(plan, results, failures if with_failures else None)

    def test_the_float_scale_does_not_silently_empty_a_condition(self):
        """The trap: `0.4` is stored as 0.400000006.

        The analyser keys conditions off the plan, so the grouping must survive
        a cell whose own field is the f32 value. If it ever grouped on the
        cell's float, the 0.4 conditions would come out empty and this fails.
        """
        wanted = {("rec+fixed", 1.0): 12, ("rec+fixed", 0.4): 11,
                  ("rec+alif", 1.0): 3, ("rec+alif", 0.4): 7}
        plan, outcomes = self.outcomes(wanted)
        self.assertNotEqual(as_f32(0.4), 0.4, "fixture must use the real f32 value")

        counted: dict[tuple[str, float], int] = {}
        for spec in plan:
            key = a13.condition(spec)
            if outcomes[spec["id"]]["state"] == "completed":
                counted[key] = counted.get(key, 0) + 1
        self.assertEqual(counted, wanted)

    def test_a_cell_without_a_seed_field_is_still_analysable(self):
        """The other trap: the emitted cell has no `seed`, so nothing may key on it."""
        wanted = {("rec+fixed", 1.0): 2, ("rec+fixed", 0.4): 2,
                  ("rec+alif", 1.0): 2, ("rec+alif", 0.4): 2}
        _, outcomes = self.outcomes(wanted)
        completed = [o for o in outcomes.values() if o["state"] == "completed"]
        self.assertEqual(len(completed), 8)
        for outcome in completed:
            self.assertNotIn("seed", outcome["cell"])

    def test_a_missing_cell_is_a_divergence_with_its_abort_step(self):
        wanted = {("rec+fixed", 1.0): 10, ("rec+fixed", 0.4): 12,
                  ("rec+alif", 1.0): 12, ("rec+alif", 0.4): 12}
        _, outcomes = self.outcomes(wanted)
        diverged = [o for o in outcomes.values() if o["state"] == "diverged"]
        self.assertEqual(len(diverged), 2)
        self.assertEqual(sorted(o["step"] for o in diverged), [510, 511])

    def test_an_emitted_cell_reporting_non_finite_events_has_not_completed(self):
        """The `e20` rec+fixed cell is exactly this: emitted, and not usable."""
        plan, results, failures = self.build(
            {("rec+fixed", 1.0): 12, ("rec+fixed", 0.4): 12,
             ("rec+alif", 1.0): 12, ("rec+alif", 0.4): 12}
        )
        target = plan[0]["id"]
        cell = json.loads((results / f"{target}.json").read_text())
        cell["non_finite_events"] = 2
        (results / f"{target}.json").write_text(json.dumps(cell))

        outcomes = a13.load_outcomes(plan, results, failures)
        self.assertEqual(outcomes[target]["state"], "voided")
        self.assertIn("non_finite_events=2", outcomes[target]["why"])

    def test_the_registered_bar_is_the_one_in_the_prereg(self):
        self.assertEqual((a13.R1_COMPLETIONS, a13.R1_OF), (11, 12))
        self.assertEqual(a13.R2_R3_DELTA, 6)

    def test_the_abort_pattern_matches_the_instrument_s_real_message(self):
        """Taken from a wave-11 failure log, not paraphrased."""
        real = (
            "shd-instrument: non-finite training value at optimizer step 438\n"
            "Error: cell run failed\n"
        )
        match = a13.ABORT_STEP.search(real)
        self.assertIsNotNone(match, "the pattern no longer matches a real log")
        self.assertEqual(int(match.group(1)), 438)

    def tearDown(self):
        if hasattr(self, "tmp"):
            self.tmp.cleanup()


if __name__ == "__main__":
    unittest.main(verbosity=2)
