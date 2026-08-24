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
import re
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


# Two defects in this file, found 2026-08-22 after the wave closed and fixed
# here. Both are bugs, not threshold changes, and **no verdict was issued before
# or after**: the completion expectation failed first, so neither line had ever
# run. The bars in this file are exactly as registered.
#
#   1. `surrogate_scale` arrives as f32, so the cell records 0.400000006. An
#      `== 0.4` grouping silently produced an empty bucket and a NaN mean.
#   2. Cells carry no `seed` field at all -- `HARDENING_2026-08-22_THE_EVIDENCE_LAYER_HAD_NO_TESTS.md`
#      already recorded that as open work, and this analyser was written against
#      it anyway. Reading `cell["seed"]` would have raised KeyError.
#
# The lesson is not "be more careful". It is that **freezing an analyser before
# the data does not make it correct** -- it has to be exercised against a
# synthetic fixture before the real cells land, which is what
# `scripts/test_campaign_tooling.py::Wave11AnalyserTest` now does.
CELL_NAME = re.compile(r"__ss(?P<scale>[0-9.]+)__s(?P<seed>\d+)\.json$")


def identify(path: pathlib.Path) -> tuple[float, int]:
    """Recover (surrogate scale, seed) from the cell id.

    The id is the only place the seed exists: the emitted cell has no `seed`
    field. Reading the scale from the name too keeps both halves of the key from
    one source rather than mixing a parsed seed with an f32 field that does not
    compare equal to the value that was planned.
    """
    match = CELL_NAME.search(path.name)
    if not match:
        raise ValueError(f"cell id does not carry a scale and seed: {path.name}")
    return float(match["scale"]), int(match["seed"])


def load() -> list[dict]:
    cells = []
    for path in sorted(CELLS.glob("w11rec__*.json")):
        cell = json.loads(path.read_text())
        cell["_scale"], cell["_seed"] = identify(path)
        cells.append(cell)
    return cells


def usable(cell: dict) -> bool:
    """A cell counts only if it finished and produced finite numbers.

    `non_finite_events` is the field the instrument's own guard increments, and
    a cell that aborted never writes a result at all -- so absence is a failure
    too, and the caller compares against PLANNED rather than against len(cells).
    """
    return cell.get("non_finite_events", 1) == 0 and cell.get("accuracy") is not None


def verdict(supported: bool, *buckets: list) -> str:
    """SUPPORTED / NOT SUPPORTED / NOT EVALUABLE.

    `mean` returns NaN on an empty list by design, and every bar here is a
    comparison — so `abs(nan) >= EFFECT_BAR` is False and an empty bucket
    printed **NOT SUPPORTED**: a scientific verdict issued over no data, in the
    same token a refuted hypothesis gets. This file already draws that
    distinction for the completion gate above; the per-hypothesis verdicts did
    not. `analyse_wave14.py` records the same lesson in its own docstring.
    """
    if any(len(b) == 0 for b in buckets):
        return "NOT EVALUABLE"
    return "SUPPORTED" if supported else "NOT SUPPORTED"


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
    plain_cells = [c for c in good if c["arm"] == "rec+alif"]
    t41 = plain > CHANCE + CHANCE_MARGIN
    print(f"**T4-1** the arm produces usable cells above chance: mean "
          f"{plain:.4f} (n={len(plain_cells)}) against {CHANCE} + {CHANCE_MARGIN} "
          f"-> **{verdict(t41, plain_cells)}**\n")

    # T4-2, paired by seed and surrogate scale.
    paired = []
    index = {(c["_seed"], c["_scale"], c["arm"]): c["accuracy"] for c in good}
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
          f"**{verdict(t42, paired)}**")
    if t42:
        print(f"  - direction: **{'attention helps' if delta > 0 else 'attention hurts'}**")
    print()

    # T4-3
    by_scale = {
        scale: [c["accuracy"] for c in good if c["_scale"] == scale]
        for scale in (1.0, 0.4)
    }
    scale_delta = mean(by_scale[1.0]) - mean(by_scale[0.4])
    t43 = abs(scale_delta) >= EFFECT_BAR
    print(f"**T4-3** *(two-sided)* surrogate scale matters: ss1.0 "
          f"{mean(by_scale[1.0]):.4f} (n={len(by_scale[1.0])}) − ss0.4 "
          f"{mean(by_scale[0.4]):.4f} (n={len(by_scale[0.4])}) = "
          f"{scale_delta:+.4f}, bar |{EFFECT_BAR}| -> "
          f"**{verdict(t43, by_scale[1.0], by_scale[0.4])}**\n")

    aborted = PLANNED - len(good)
    if aborted:
        print(f"**Marginality note:** {aborted} of {PLANNED} cells still did not "
              f"produce a usable result. That is expected and registered — the "
              f"unclipped record at this operating point is 13 of 15, and "
              f"completing cells show gradient peaks to 3.93e33.\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
