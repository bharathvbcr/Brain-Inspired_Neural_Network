"""Apply the registered H1/H3 thresholds to the temporal-information campaign.

Thresholds come from `PREREG_2026-08-02_SHD_TEMPORAL_INFORMATION.md` §4 and are
**hardcoded below as constants**. They are not parameters. If a threshold needs
to change, that is an amendment in a new timestamped file, written before the
verdict is looked at — not an edit here.

    .venv-shd/bin/python scripts/temporal_campaign_verdict.py

H1 is an *equivalence* test: it passes on a bounded difference, not on a failure
to reject. A wide confidence interval therefore makes H1 harder to pass, not
easier, which is the correct direction for a claim of "no effect".
"""

from __future__ import annotations

import json
import math
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CAMPAIGN = ROOT / "results" / "shd_instrument_v4" / "temporal-campaign"

# --- registered constants (PREREG_2026-08-02 §4) ---------------------------
H1_EQUIVALENCE_BOUND = 0.02      # |intact - bin-shuffled| <= this, for ff+fixed
H3_SYNCHRONY_BOUND = 0.02        # channel-shuffled worse than bin-shuffled by >= this
GATE_52_INTACT_FLOOR = 0.65      # every intact cell must reach this
GATE_51_RELOCATION_FLOOR = 0.05  # manipulation must actually move entries

CONDITIONS = ("intact", "bin-shuffled", "channel-shuffled", "reversed")
# Extended from 3 to 6 by AMENDMENT_2026-08-03_H1_SEED_EXTENSION.md, whose
# stopping rule is binding: exactly these six, verdict recomputed once,
# reported whichever way it falls. Do not add a seventh.
SEEDS = (5170001, 5170002, 5170003, 5170004, 5170005, 5170006)

# Student t, two-sided 95%, df = n-1. Three seeds is df=2.
T_CRITICAL = {1: 12.706, 2: 4.303, 3: 3.182, 4: 2.776, 5: 2.571}


def mean(xs: list[float]) -> float:
    """Arithmetic mean; **raises** on an empty sequence rather than returning a number.

    Every caller here is guaranteed non-empty by `load`, which refuses a partial
    arm. If that guarantee ever breaks, a bare `sum(xs) / len(xs)` raises
    `ZeroDivisionError` from inside a verdict line, which says nothing about
    which arm was empty. `analyse_wave11.mean` deliberately returns NaN instead,
    because its report tolerates an empty condition; the two are pinned as
    different in `test_campaign_tooling.py`. Do not converge them.
    """
    if not xs:
        raise ValueError("mean of no values: an arm reached a verdict line empty")
    return sum(xs) / len(xs)


def stdev(xs: list[float]) -> float:
    if len(xs) < 2:
        return 0.0
    m = mean(xs)
    return math.sqrt(sum((x - m) ** 2 for x in xs) / (len(xs) - 1))


def ci95(xs: list[float]) -> tuple[float, float]:
    """Two-sided 95% CI on the mean. Widens honestly with few seeds."""
    if len(xs) < 2:
        return (xs[0], xs[0])
    half = T_CRITICAL.get(len(xs) - 1, 1.96) * stdev(xs) / math.sqrt(len(xs))
    m = mean(xs)
    return (m - half, m + half)


def overlaps(a: tuple[float, float], b: tuple[float, float]) -> bool:
    return a[0] <= b[1] and b[0] <= a[1]


def load(arm: str = "ff-fixed") -> dict[str, list[dict]]:
    by_condition: dict[str, list[dict]] = {c: [] for c in CONDITIONS}
    missing = []
    for condition in CONDITIONS:
        for seed in SEEDS:
            path = CAMPAIGN / f"{arm}__{condition}__h512__e100__s{seed}.json"
            if not path.is_file():
                missing.append(path.name)
                continue
            by_condition[condition].append(json.loads(path.read_text()))
    if missing:
        print(f"MISSING {len(missing)} cells:")
        for name in missing:
            print(f"  {name}")
    return by_condition


def main() -> int:
    cells = load()
    # Refuse on ANY missing cell, not just an empty condition.
    #
    # This previously bailed only when a condition had zero cells, so with 3 of
    # 6 seeds present it printed a complete-looking verdict computed on half the
    # data. Under a registered stopping rule that is the exact failure the rule
    # exists to prevent: a verdict readable before the committed sample is in,
    # which is optional stopping whether or not anyone acts on it.
    expected = len(CONDITIONS) * len(SEEDS)
    have = sum(len(v) for v in cells.values())
    if have != expected:
        print(f"\n{have}/{expected} cells present - NO VERDICT. The seed count is "
              f"fixed by amendment; the verdict is computed once, on all of them.")
        return 1

    acc = {c: [d["accuracy"] for d in cells[c]] for c in CONDITIONS}

    print("condition          n   mean      sd        95% CI")
    for c in CONDITIONS:
        lo, hi = ci95(acc[c])
        print(f"{c:<18}{len(acc[c])}   {mean(acc[c]):.4f}    {stdev(acc[c]):.4f}    "
              f"[{lo:.4f}, {hi:.4f}]")

    print("\n--- validity gates ---")
    intact_min = min(acc["intact"])
    gate52 = intact_min >= GATE_52_INTACT_FLOOR
    print(f"5.2 trained regime: min intact {intact_min:.4f} >= {GATE_52_INTACT_FLOOR} -> "
          f"{'PASS' if gate52 else 'FAIL'}")

    gate51 = True
    for c in CONDITIONS:
        if c == "intact":
            continue
        for d in cells[c]:
            audit = d.get("temporal_audit", {})
            if not audit.get("counts_preserved", False):
                gate51 = False
            if audit.get("relocated_fraction", 0.0) < GATE_51_RELOCATION_FLOOR:
                gate51 = False
    print(f"5.1 manipulation:   counts preserved and entries relocated -> "
          f"{'PASS' if gate51 else 'FAIL'}")

    nonfinite = sum(d.get("non_finite_events", 0) for v in cells.values() for d in v)
    print(f"numerical:          non_finite_events total {nonfinite} -> "
          f"{'PASS' if nonfinite == 0 else 'FAIL'}")

    if not (gate52 and gate51 and nonfinite == 0):
        print("\nVALIDITY GATE FAILED - hypotheses are not evaluated")
        return 1

    print("\n--- registered hypotheses ---")
    delta_bin = mean(acc["intact"]) - mean(acc["bin-shuffled"])
    ci_intact, ci_bin = ci95(acc["intact"]), ci95(acc["bin-shuffled"])
    h1_bounded = abs(delta_bin) <= H1_EQUIVALENCE_BOUND
    h1_overlap = overlaps(ci_intact, ci_bin)
    h1 = h1_bounded and h1_overlap
    print(f"H1 (ff+fixed is a rate coder): |intact - bin-shuffled| = {abs(delta_bin):.4f} "
          f"<= {H1_EQUIVALENCE_BOUND} -> {h1_bounded}; CIs overlap -> {h1_overlap}")
    print(f"   VERDICT: {'H1 SUPPORTED' if h1 else 'H1 NOT SUPPORTED'}")

    delta_sync = mean(acc["bin-shuffled"]) - mean(acc["channel-shuffled"])
    h3 = delta_sync >= H3_SYNCHRONY_BOUND
    print(f"H3 (synchrony beyond order): bin-shuffled - channel-shuffled = "
          f"{delta_sync:.4f} >= {H3_SYNCHRONY_BOUND} -> {h3}")
    print("   (registered as confirmatory only if H1 fails)")

    delta_rev = mean(acc["intact"]) - mean(acc["reversed"])
    print(f"\ndescriptive: intact - reversed = {delta_rev:.4f}")
    print("   Not a registered hypothesis. The trained-weights probe predicted "
          "reversed ~ intact;\n   that prediction was post-hoc and is reported as such.")

    n = len(acc["intact"])
    t = T_CRITICAL.get(n - 1, 1.96)
    print(f"\nNOTE: {n} seeds gives df={n-1} and t={t}. Narrower CIs make the "
          "overlap criterion\nHARDER to satisfy, so a pass here is stronger than a "
          "pass on fewer seeds.")
    if n >= 6:
        print("Seed count is fixed at 6 by AMENDMENT_2026-08-03_H1_SEED_EXTENSION.md.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
