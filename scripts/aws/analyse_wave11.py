#!/usr/bin/env python3
"""Wave 11 verdicts, frozen before the first cell landed.

Registered in `results/AMENDMENT_2026-08-22_WAVE4_WITHOUT_CLIPPING.md`. Wave 11
is wave 4 with `clip_grad_norm` removed and nothing else changed.

Written and frozen before any wave-11 cell existed, for the same reason
`analyse_wave10.py` was: an analyser authored after seeing the data is not an
analysis, it is a selection. The completion bar (18 of 24) and every threshold
below come from the amendment, not from the numbers.

The wave-4 cells this replaces did not fail because the arm cannot learn; they
failed because a 1.0 clip threshold taken from the `ff+fixed` scale bound on
essentially every step of an arm whose own gradient norm exceeds 1.0 in 100 of
100 epochs. See `results/FINDING_2026-08-22_WAVE4_KILLED_ITS_OWN_CELLS.md`.
"""

from __future__ import annotations

import json
import pathlib
import statistics
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]
CELLS = ROOT / "results/shd_attention_campaign_v2"
PLANNED = 24
# From the amendment, section 3. Not 24: the numerical marginality is real and
# independent of clipping, and the unclipped record at this operating point is
# 13 of 15.
COMPLETION_BAR = 18
CHANCE = 0.05
# The same above-chance margin `binn_lab::guards::CeilingHealth` uses.
CHANCE_MARGIN = 0.05
EFFECT_BAR = 0.05
SIGN_AGREEMENT = 10


def load() -> list[dict]:
    return [
        json.loads(path.read_text())
        for path in sorted(CELLS.glob("w11rec__*.json"))
    ]


def usable(cell: dict) -> bool:
    """A cell counts only if it finished and produced finite numbers.

    `non_finite_events` is the field the instrument's own guard increments, and
    a cell that aborted never writes a result at all -- so absence is a failure
    too, and the caller compares against PLANNED rather than against len(cells).
    """
    return cell.get("non_finite_events", 1) == 0 and cell.get("accuracy") is not None


def mean(values: list[float]) -> float:
    return statistics.fmean(values) if values else float("nan")


def main() -> int:
    cells = load()
    good = [c for c in cells if usable(c)]
    print("# Wave 11 — registered verdicts\n")
    print(
        f"Amendment: `results/AMENDMENT_2026-08-22_WAVE4_WITHOUT_CLIPPING.md`. "
        f"Wave 4 re-run with `clip_grad_norm` removed, nothing else changed.\n"
    )
    print(f"**Cells: {len(good)} usable of {PLANNED} planned** "
          f"({len(cells)} present on disk).\n")

    if len(good) < COMPLETION_BAR:
        print(f"## §3 completion expectation NOT MET ({len(good)} < {COMPLETION_BAR})\n")
        print(
            "The amendment's registered response applies: the diagnosis in "
            "`FINDING_2026-08-22_WAVE4_KILLED_ITS_OWN_CELLS.md` is incomplete, "
            "clipping was not the whole cause, and **no further lever is added "
            "without its own amendment**. No scientific verdict is issued — "
            "T4-1, T4-2 and T4-3 are all NOT EVALUABLE.\n"
        )
        return 1

    print(f"## §3 completion expectation MET ({len(good)} >= {COMPLETION_BAR})\n")
    print(
        "The clip flag was the cause. `rec+alif` completes at h256/e100, which "
        "`RESULT_2026-08-20_W4_RECURRENT_ARM_IS_UNUSABLE.md` denied.\n"
    )

    by_arm = {
        arm: [c["accuracy"] for c in good if c["arm"] == arm]
        for arm in ("rec+alif", "rec+alif+attn")
    }
    plain = mean(by_arm["rec+alif"])
    attn = mean(by_arm["rec+alif+attn"])

    print("| arm | n | mean accuracy |")
    print("|---|---:|---:|")
    for arm, values in by_arm.items():
        print(f"| `{arm}` | {len(values)} | {mean(values):.4f} |")
    print()

    # T4-1
    t41 = plain > CHANCE + CHANCE_MARGIN
    print(f"**T4-1** the arm produces usable cells above chance: mean "
          f"{plain:.4f} against {CHANCE} + {CHANCE_MARGIN} -> "
          f"**{'SUPPORTED' if t41 else 'NOT SUPPORTED'}**\n")

    # T4-2, paired by seed and surrogate scale.
    paired = []
    index = {(c["seed"], c.get("surrogate_scale"), c["arm"]): c["accuracy"] for c in good}
    for (seed, scale, arm), value in index.items():
        if arm != "rec+alif":
            continue
        other = index.get((seed, scale, "rec+alif+attn"))
        if other is not None:
            paired.append(other - value)
    delta = attn - plain
    agreeing = max(sum(1 for d in paired if d > 0), sum(1 for d in paired if d < 0))
    t42 = abs(delta) >= EFFECT_BAR and agreeing >= SIGN_AGREEMENT
    print(f"**T4-2** *(two-sided)* attention changes recurrent accuracy: "
          f"{delta:+.4f}, bar |{EFFECT_BAR}|; {agreeing}/{len(paired)} paired seeds "
          f"agree in sign, bar {SIGN_AGREEMENT} -> "
          f"**{'SUPPORTED' if t42 else 'NOT SUPPORTED'}**")
    if t42:
        print(f"  - direction: **{'attention helps' if delta > 0 else 'attention hurts'}**")
    print()

    # T4-3
    by_scale = {
        scale: [c["accuracy"] for c in good if c.get("surrogate_scale") == scale]
        for scale in (1.0, 0.4)
    }
    scale_delta = mean(by_scale[1.0]) - mean(by_scale[0.4])
    t43 = abs(scale_delta) >= EFFECT_BAR
    print(f"**T4-3** *(two-sided)* surrogate scale matters: ss1.0 "
          f"{mean(by_scale[1.0]):.4f} − ss0.4 {mean(by_scale[0.4]):.4f} = "
          f"{scale_delta:+.4f}, bar |{EFFECT_BAR}| -> "
          f"**{'SUPPORTED' if t43 else 'NOT SUPPORTED'}**\n")

    aborted = PLANNED - len(good)
    if aborted:
        print(f"**Marginality note:** {aborted} of {PLANNED} cells still did not "
              f"produce a usable result. That is expected and registered — the "
              f"unclipped record at this operating point is 13 of 15, and "
              f"completing cells show gradient peaks to 3.93e33.\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
