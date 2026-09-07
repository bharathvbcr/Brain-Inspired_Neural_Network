#!/usr/bin/env python3
"""Wave 29 — the asymmetry, with a bar that can only be failed one way.

Frozen with `PREREG_2026-09-07_W29_THE_ASYMMETRY_HAS_ITS_OWN_BAR.md` and
committed before the first cell exists. Its output is the authority on every
verdict in the result document.

Wave 28 registered `|DiD| < 0.03` for H28-2 and reported NOT MET at p90. The
band was two-sided and the question was not: with

    DiD = (attn_intact - attn_X) - (rate_intact - rate_X)

a POSITIVE DiD is the read-out losing more than the substrate, which is what
H28-2 was written to detect. The measured values were negative -- -0.0132 at
p70, -0.0382 at p90 -- so the clause fired on the outcome nobody had named, and
printed a reading that describes the opposite of its own sign.

This analyser splits that into two one-sided clauses on a NEW seed block. It
refuses wave 28's seeds by value, so the replacement bar cannot be evaluated on
the cells that motivated it.
"""

from __future__ import annotations

import argparse
import collections
import json
import re
import statistics
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent
sys.path.insert(0, str(ROOT / "scripts"))
from cell_validity import validity_problems  # noqa: E402

THIS_WAVES = {"w29asy"}

SEED = re.compile(r"__s(\d+)\.json$")
DEPTH = re.compile(r"__(d\d+l\d+)__")

#: The campaign's bar, unchanged since wave 21.
BAR = 0.03
MIN_PAIRS = 9
#: Wave 28 H28-1's bar, restated: the rate arm must lose more than this or the
#: manipulation is not one and nothing measured under it is interpretable.
DROPOUT_MIN_COST = 0.03
ANCHOR_HIDDEN = 128
#: p90 carries both primary clauses. p70 is descriptive only (H29-3).
PRIMARY_PERCENT = 90
SECONDARY_PERCENT = 70

#: Wave 28's block. Rejected by VALUE, so a re-tagged wave-28 cell cannot enter
#: this analysis and the replacement bar cannot be read on the cells that
#: motivated it. Rule 9.3 is the whole reason this wave exists.
FORBIDDEN_SEEDS = frozenset(range(5170001, 5170013))
REGISTERED_SEEDS = frozenset(range(5290001, 5290013))

NOT_EVALUABLE = "NOT EVALUABLE"


def status_of(met, blocked):
    if blocked:
        return blocked
    return "MET" if met else "NOT MET"


def verdict(status, name, met_reading, not_met_reading):
    print(f"\n{name}: {status}")
    if status == "MET":
        print(f"  {met_reading}")
    elif status == "NOT MET":
        print(f"  {not_met_reading}")
    else:
        print("  The clause did not run. This is not a refutation, and no "
              "reading is taken from it.")
    return status


def index(results):
    out = collections.defaultdict(dict)
    voided = collections.Counter()
    offwidth = collections.Counter()
    borrowed = collections.Counter()
    collisions = []
    for path in sorted(Path(results).glob("*.json")):
        wave = path.name.split("__", 1)[0]
        if wave not in THIS_WAVES:
            continue
        seed = SEED.search(path.name)
        if not seed:
            continue
        s_id = int(seed.group(1))
        # Rule 9.3, enforced rather than asserted.
        if s_id in FORBIDDEN_SEEDS:
            borrowed[wave] += 1
            continue
        if s_id not in REGISTERED_SEEDS:
            borrowed[wave] += 1
            continue
        try:
            cell = json.loads(path.read_text())
        except ValueError:
            continue
        if not (isinstance(cell, dict) and "accuracy" in cell):
            continue
        if cell.get("hidden") != ANCHOR_HIDDEN:
            offwidth[wave] += 1
            continue
        arm = cell.get("arm")
        found = DEPTH.search(path.name)
        depth = found.group(1) if found else None
        if arm == "ff+fixed+attn" and depth is None:
            continue
        if arm == "ff+fixed" and depth is not None:
            continue
        key = (wave, arm, depth, cell.get("attn_readout") or "default",
               cell.get("temporal_condition") or "intact")
        if validity_problems(cell):
            voided[key] += 1
            continue
        # Never `setdefault`. A collision resolved by sort order is a wrong
        # number with no symptom -- wave 28's `no-position` read-out is the
        # worked case.
        if s_id in out[key]:
            collisions.append((key, s_id, path.name))
            continue
        out[key][s_id] = cell["accuracy"]
    if offwidth:
        print(f"  dropped, not at h{ANCHOR_HIDDEN}: {dict(offwidth)}")
    if borrowed:
        print(f"  dropped, seed outside this wave's registered block: "
              f"{dict(borrowed)}")
    if collisions:
        raise SystemExit(
            f"INDEX COLLISION: {len(collisions)} cells share a (key, seed) with "
            f"another cell, so the index key is missing a dimension the corpus "
            f"varies. First three: {collisions[:3]}. Refusing to analyse.")
    return out, voided


def paired(*arms):
    if not arms or any(not a for a in arms):
        return []
    shared = set(arms[0])
    for a in arms[1:]:
        shared &= set(a)
    return sorted(shared)


def arm_key(attn, temporal="intact", readout="default"):
    wave = "w29asy"
    if attn:
        return (wave, "ff+fixed+attn", "d32l4", readout, temporal)
    return (wave, "ff+fixed", None, "default", temporal)


def rate_cost(cells, percent):
    """(mean drop from intact, count above the bar, n) on the RATE arm."""
    a = cells.get(arm_key(False), {})
    b = cells.get(arm_key(False, f"spike-dropout-p{percent}"), {})
    shared = paired(a, b)
    if not shared:
        return None, 0, 0
    drops = [a[s] - b[s] for s in shared]
    return statistics.fmean(drops), sum(d > DROPOUT_MIN_COST for d in drops), len(drops)


def did(cells, percent):
    """(mean, below_plus_bar, below_minus_bar, n) for the per-seed DiD.

    Two counts, because the two primary clauses are one-sided in opposite
    directions and a single `abs()` count is exactly the mistake this wave
    exists to repair.
    """
    condition = f"spike-dropout-p{percent}"
    ai = cells.get(arm_key(True), {})
    ax = cells.get(arm_key(True, condition), {})
    bi = cells.get(arm_key(False), {})
    bx = cells.get(arm_key(False, condition), {})
    shared = paired(ai, ax, bi, bx)
    if not shared:
        return None, 0, 0, 0
    deltas = [(ai[s] - ax[s]) - (bi[s] - bx[s]) for s in shared]
    return (statistics.fmean(deltas),
            sum(d < BAR for d in deltas),
            sum(d < -BAR for d in deltas),
            len(deltas))


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--results",
                        default=str(ROOT / "results/shd_attention_campaign_v3"))
    args = parser.parse_args()

    cells, voided = index(args.results)
    mine = sum(len(v) for k, v in cells.items() if k[0] in THIS_WAVES)
    print("wave 29 — the asymmetry has its own bar")
    print(f"  wave-29 cells : {mine}")
    print(f"  cells voided  : {sum(voided.values())}")
    for key, count in sorted(voided.items(), key=str):
        print(f"    {count:3d}  {key}")

    if mine == 0:
        print("\nNOTHING TO ANALYSE: no wave-29 cell was found under "
              f"{args.results}. This is not a verdict.")
        return 2

    # ---- The instrument must be able to feel its own hand first. ----------
    print(f"\n=== sensitivity gate  (wave 28 H28-1's bar, restated) ===")
    cost, above, n_cost = rate_cost(cells, PRIMARY_PERCENT)
    sensitive = cost is not None and n_cost >= MIN_PAIRS and above >= MIN_PAIRS
    if cost is None:
        print(f"  p{PRIMARY_PERCENT}  no paired rate cells")
    else:
        print(f"  p{PRIMARY_PERCENT}  rate-arm drop {cost:+.4f}  "
              f"above {DROPOUT_MIN_COST} in {above}/{n_cost}"
              f"{'  SENSITIVE' if sensitive else '  NOT sensitive'}")

    blocked = None if sensitive else NOT_EVALUABLE

    # ---- H29-1: does destroying counts COST the read-out? -----------------
    print("\n=== H29-1  does destroying counts cost the read-out? ===")
    print("  one-sided. Refuted only by DiD > +0.03; a negative DiD satisfies "
          "it.")
    mean, below_plus, below_minus, n = did(cells, PRIMARY_PERCENT)
    if mean is None:
        print(f"  p{PRIMARY_PERCENT}  no complete seed quadruple")
        h29_1 = verdict(NOT_EVALUABLE, "H29-1  the read-out is not hurt by "
                        "count destruction", "", "")
    else:
        print(f"  p{PRIMARY_PERCENT}  DiD {mean:+.4f}  "
              f"DiD < +{BAR} in {below_plus}/{n}")
        h29_1 = verdict(
            status_of(n >= MIN_PAIRS and below_plus >= MIN_PAIRS, blocked),
            "H29-1  the read-out is not hurt by count destruction",
            "destroying counts does not cost the read-out its advantage. The "
            "mechanism claim stands as a claim about order.",
            "the read-out loses MORE than the substrate: what it consumes is "
            "not order alone, and wave 28's negative sign was a seed-block "
            "effect.")

    # ---- H29-2: is the growth real, and bigger than the bar? --------------
    print("\n=== H29-2  does the advantage GROW under count destruction? ===")
    print("  one-sided, the direction wave 28 measured, on a new seed block.")
    if mean is None:
        h29_2 = verdict(NOT_EVALUABLE, "H29-2  the advantage grows", "", "")
    else:
        print(f"  p{PRIMARY_PERCENT}  DiD {mean:+.4f}  "
              f"DiD < -{BAR} in {below_minus}/{n}")
        h29_2 = verdict(
            status_of(n >= MIN_PAIRS and below_minus >= MIN_PAIRS, blocked),
            "H29-2  the advantage grows",
            "the read-out's advantage increases under count destruction by more "
            "than the campaign's bar, on a second seed block. The asymmetry is "
            "a result with a bar behind it.",
            "the growth wave 28 saw is not reproduced at this size. The honest "
            "reading is that the advantage is INSENSITIVE to counts, and "
            "-0.0382 was a seed-block effect.")

    # ---- H29-3: descriptive, no verdict. ----------------------------------
    print("\n=== H29-3  monotone in rate?  (descriptive, no bar) ===")
    lower, _, _, n_lower = did(cells, SECONDARY_PERCENT)
    if lower is None or mean is None:
        print(f"  p{SECONDARY_PERCENT} did not land; no direction reported. "
              "This clause carries no verdict either way.")
    else:
        print(f"  p{SECONDARY_PERCENT}  DiD {lower:+.4f}  (n={n_lower})")
        print(f"  p{PRIMARY_PERCENT}  DiD {mean:+.4f}  (n={n})")
        print(f"  DiD(p{PRIMARY_PERCENT}) < DiD(p{SECONDARY_PERCENT}): "
              f"{mean < lower}")
        print("  Two rungs are not a curve, and this may not be cited as one.")

    print("\n" + "=" * 70)
    print(f"  H29-1: {h29_1}")
    print(f"  H29-2: {h29_2}")
    print("  H29-3: descriptive, no verdict")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
