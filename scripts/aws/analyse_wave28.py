#!/usr/bin/env python3
"""Wave 28 — the dropout rate at which the instrument can feel its own hand.

Frozen with `PREREG_2026-09-05_W28_THE_RATE_AT_WHICH_DROPOUT_BITES.md` and
committed before the first cell exists. Its output is the authority on every
verdict in the result document.

The two clauses are deliberately **conditional, in that order**. Section 2 of
the instrument registration says a null under a count-preserving operator is
ambiguous between "order does not matter" and "the measurement cannot see it".
H28-1 asks whether the instrument can see anything at all; only if it can does
H28-2's null mean something. So H28-2 is NOT EVALUABLE whenever H28-1 fails —
never NOT MET, and never quietly reported as a null.

Every arm may name its own wave, which is the shape wave 27's amendment settled
after the wave-26 amendment fixed only its first instance.
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

THIS_WAVES = {"w28drp"}
#: p30 is wave 27's rung and is the bottom of this ladder, not a repeat of it.
#: `w26pos` holds the `intact` arms both clauses are measured against.
REUSE_WAVES = {"w27drp", "w26pos"}

SEED = re.compile(r"__s(\d+)\.json$")
DEPTH = re.compile(r"__(d\d+l\d+)__")

#: The campaign's bar, unchanged since wave 21, and section 2's bar for H28-1.
BAR = 0.03
MIN_PAIRS = 9
#: Section 2's bar, restated: the rate arm must lose more than this, or the
#: manipulation is not one and nothing measured under it is interpretable.
DROPOUT_MIN_COST = 0.03
ANCHOR_HIDDEN = 128
#: p30 comes from wave 27; the rest are this wave's.
LADDER = (30, 50, 60, 70, 80, 90)
ATTN_PERCENTS = (30, 70, 90)

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


def wave_of(percent):
    """p30 is wave 27's rung. Reusing it is why it is not re-run here."""
    return "w27drp" if percent == 30 else "w28drp"


def index(results):
    out = collections.defaultdict(dict)
    voided = collections.Counter()
    offwidth = collections.Counter()
    collisions = []
    for path in sorted(Path(results).glob("*.json")):
        wave = path.name.split("__", 1)[0]
        if wave not in THIS_WAVES and wave not in REUSE_WAVES:
            continue
        seed = SEED.search(path.name)
        if not seed:
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
        # Never `setdefault`. If two cells land on one (key, seed) the key is
        # missing a dimension the corpus actually varies, and first-wins
        # silently answers with whichever file sorts first. `w26pos` carries a
        # `no-position` read-out beside the default one; without `attn_readout`
        # in the key it sorts first and becomes the baseline, which is a wrong
        # number with no symptom. Collisions are fatal here, not resolved.
        s_id = int(seed.group(1))
        if s_id in out[key]:
            collisions.append((key, s_id, path.name))
            continue
        out[key][s_id] = cell["accuracy"]
    if offwidth:
        print(f"  dropped, not at h{ANCHOR_HIDDEN}: {dict(offwidth)}")
    if collisions:
        raise SystemExit(
            f"INDEX COLLISION: {len(collisions)} cells share a (key, seed) with "
            f"another cell, so the index key is missing a dimension the corpus "
            f"varies. First three: {collisions[:3]}. Refusing to analyse — a "
            f"collision resolved by sort order is a wrong number with no symptom.")
    return out, voided


def paired(*arms):
    if not arms or any(not a for a in arms):
        return []
    shared = set(arms[0])
    for a in arms[1:]:
        shared &= set(a)
    return sorted(shared)


def arm_key(wave, attn, temporal="intact", readout="default"):
    if attn:
        return (wave, "ff+fixed+attn", "d32l4", readout, temporal)
    return (wave, "ff+fixed", None, "default", temporal)


def rate_cost(cells, percent, intact_wave="w26pos"):
    """(mean drop from intact, count above the bar, n) on the RATE arm.

    Section 2's bar is on this arm and only this arm: it asks whether the
    manipulation costs the substrate anything, not whether the read-out cares.
    """
    a = cells.get(arm_key(intact_wave, False), {})
    b = cells.get(arm_key(wave_of(percent), False, f"spike-dropout-p{percent}"), {})
    shared = paired(a, b)
    if not shared:
        return None, 0, 0
    drops = [a[s] - b[s] for s in shared]
    return statistics.fmean(drops), sum(d > DROPOUT_MIN_COST for d in drops), len(drops)


def did(cells, percent, intact_wave="w26pos"):
    """(mean, positive, n) for (attn_intact - attn_X) - (rate_intact - rate_X)."""
    wave = wave_of(percent)
    condition = f"spike-dropout-p{percent}"
    ai = cells.get(arm_key(intact_wave, True), {})
    ax = cells.get(arm_key(wave, True, condition), {})
    bi = cells.get(arm_key(intact_wave, False), {})
    bx = cells.get(arm_key(wave, False, condition), {})
    shared = paired(ai, ax, bi, bx)
    if not shared:
        return None, 0, 0
    deltas = [(ai[s] - ax[s]) - (bi[s] - bx[s]) for s in shared]
    return statistics.fmean(deltas), sum(abs(d) > BAR for d in deltas), len(deltas)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--results",
                        default=str(ROOT / "results/shd_attention_campaign_v3"))
    args = parser.parse_args()

    cells, voided = index(args.results)
    mine = sum(len(v) for k, v in cells.items() if k[0] in THIS_WAVES)
    print("wave 28 — the rate at which dropout bites")
    print(f"  wave-28 cells : {mine}")
    print(f"  reused        : "
          f"{sum(len(v) for k, v in cells.items() if k[0] in REUSE_WAVES)}"
          f"  (w27drp p30, w26pos intact; same binary)")
    print(f"  cells voided  : {sum(voided.values())}")
    for key, count in sorted(voided.items(), key=str):
        print(f"    {count:3d}  {key}")

    if mine == 0:
        print("\nNOTHING TO ANALYSE: no wave-28 cell was found under "
              f"{args.results}. This is not a verdict.")
        return 2

    # ---- H28-1: can the instrument feel its own manipulation? -------------
    print("\n=== H28-1  the rate at which the substrate actually loses accuracy ===")
    print(f"  bar: paired rate-arm drop from intact > {DROPOUT_MIN_COST} "
          f"in >= {MIN_PAIRS} of 12")
    sensitive = []
    for percent in LADDER:
        cost, above, n = rate_cost(cells, percent)
        voids = voided.get(
            arm_key(wave_of(percent), False, f"spike-dropout-p{percent}"), 0)
        ok = cost is not None and cost > DROPOUT_MIN_COST and above >= MIN_PAIRS
        if ok:
            sensitive.append(percent)
        tag = "  SENSITIVE" if ok else ""
        note = f"  voided {voids}" if voids else ""
        print(f"  p{percent:<3} drop "
              f"{NOT_EVALUABLE if cost is None else format(cost, '.4f')}"
              f"  above bar {above}/{n}"
              f"  [{wave_of(percent)}]{tag}{note}")
    smallest = min(sensitive) if sensitive else None
    evaluated = [p for p in LADDER if rate_cost(cells, p)[2] >= MIN_PAIRS]
    blocked = (None if evaluated
               else f"{NOT_EVALUABLE}: no rung has {MIN_PAIRS} seed pairs")
    h1 = status_of(bool(sensitive), blocked)
    verdict(h1, "H28-1  dropout becomes a manipulation the substrate can feel",
            f"the instrument is sensitive from p{smallest} upward. A null "
            "measured at or above that rate means the read-out does not care, "
            "not that the measurement could not see.",
            "no rung on the registered ladder costs the substrate more than the "
            f"bar. Dropout is unusable as an instrument at this anchor, and "
            "section 2's ambiguity stays open — which is a finding about the "
            "instrument, not about order.")
    if smallest is not None:
        print(f"  smallest sensitive rate: p{smallest}")

    # ---- H28-2: and does the read-out care? -------------------------------
    print("\n=== H28-2  does destroying counts cost the read-out its advantage? ===")
    print("  Dropout deletes spikes; it does not move them. Order survives it.")
    usable = []
    short = []
    for percent in ATTN_PERCENTS:
        value, outside, n = did(cells, percent)
        mark = ""
        if percent in sensitive:
            if value is not None and n >= MIN_PAIRS:
                mark = "  (sensitive)"
                usable.append((percent, value))
            else:
                # The campaign's seed-paired floor, applied here as it is to
                # H28-1. A DiD over 6 or 7 quadruples is not a verdict, and a
                # rung short of the floor makes the clause unevaluable rather
                # than contributing a number to it.
                mark = f"  (sensitive, but {n} pairs < floor {MIN_PAIRS})"
                short.append(percent)
        elif value is not None:
            mark = "  (NOT sensitive - no reading taken)"
        print(f"  p{percent:<3} DiD "
              f"{NOT_EVALUABLE if value is None else format(value, '.4f')}"
              f"  |DiD|>{BAR} in {outside}/{n}{mark}")

    if not sensitive:
        blocked2 = (f"{NOT_EVALUABLE}: no rung is sensitive, so no null "
                    "measured here is interpretable")
    elif short:
        blocked2 = (f"{NOT_EVALUABLE}: sensitive rung(s) "
                    f"{', '.join('p%d' % p for p in short)} have fewer than "
                    f"{MIN_PAIRS} seed pairs")
    elif not usable:
        blocked2 = (f"{NOT_EVALUABLE}: no sensitive rung has an attention arm")
    else:
        blocked2 = None
    h2 = status_of(bool(usable) and all(abs(v) < BAR for _, v in usable), blocked2)
    verdict(h2, "H28-2  the read-out's advantage is specific to order",
            "at every sensitive rate, destroying spike counts while leaving "
            "order intact costs the read-out less than the bar — against 0.1208 "
            "for destroying order. Section 2's ambiguity is CLOSED: the null "
            "under the count-preserving operators means order, not blindness.",
            "destroying counts DOES cost the read-out its advantage. What the "
            "read-out consumes is not order alone, and the paper's mechanism "
            "claim is broader than it currently states.")

    print("\n" + "=" * 70)
    print(f"  H28-1: {h1 if h1 in ('MET', 'NOT MET') else NOT_EVALUABLE}")
    print(f"  H28-2: {h2 if h2 in ('MET', 'NOT MET') else NOT_EVALUABLE}")
    unrun = [n for n, s in (("H28-1", h1), ("H28-2", h2))
             if s.startswith(NOT_EVALUABLE)]
    if unrun:
        print(f"\nINCOMPLETE: {', '.join(unrun)} did not run. The result "
              "document records them as unevaluated, never as refuted.")
        return 3
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
