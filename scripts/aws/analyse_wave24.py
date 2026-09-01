#!/usr/bin/env python3
"""Wave 24 — is the shuffle cost about order, and does it survive a second budget.

Frozen with `results/PREREG_2026-09-01_ORDER_SYNCHRONY_AND_BUDGET.md` in the
same commit, before any cell of this wave existed. Every bar below is that
document's; changing one changes what was registered.

# The two things this analyser does that wave 22's does not

**It reuses cells from another wave, and it says so on every line.** At `e400`
the `intact` and `bin-shuffled` arms come from `w22cov`; only `reversed` and
`channel-shuffled` are new. That is legitimate exactly as long as the pinned
binary is unchanged (prereg §5), and it is dangerous in exactly one way: if a
lookup silently fell back to the wrong wave, a contrast would mix two
experiments and still print a number. So the wave label is part of the key,
every arm is fetched from a NAMED wave, and the source wave is printed in the
provenance table. A missing arm returns nothing; it never resolves to the other
wave's cell.

**It reads two budgets.** `epochs` is part of the key for the same reason
`depth` is part of wave 22's: a field that is dropped from the key merges
silently and looks like a cleaner result.

Run: python3 scripts/aws/analyse_wave24.py
"""

from __future__ import annotations

import collections
import json
import re
import statistics
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts"))

from cell_validity import validity_problems  # noqa: E402

ARCHIVE_V2 = ROOT / "results/shd_attention_campaign_v2"

#: The preregistration's bars.
MIN_PAIRS = 9                   # of 12, per contrast
H24_1_MAX_REVERSED_DID = 0.03   # a displacement-matched null must FAIL this bar
H24_2_SYNCHRONY_BAND = 0.03     # registered as a question, not a prediction
H24_3_MIN_DID = 0.03
H24_3_MIN_POSITIVE = 9
H24_4_MAX_REVERSED_DID = 0.03
TOTAL_CELLS = 336

#: The bucket pin every cell of this contrast must have been produced by. This
#: analyser cannot read S3; the value is recorded so a changed pin is a visible
#: contradiction rather than a silent re-baseline. `bootstrap.sh:78` is what
#: actually enforces it, at launch, by aborting on mismatch.
PINNED_BINARY = "3afd4434431a75a26cc9d5fa46831341fc2f1dd0ef08dc308e18ca139b576364"

THIS_WAVE = "w24ord"
REUSED_WAVE = "w22cov"

ANCHOR = ("published-2ms", "adjacent-sum-5")

#: (hidden, contract, geometry, depth) at e400. Every one is a wave-22 point,
#: which is what makes the intact/bin-shuffled reuse available.
POINTS_E400 = (
    (128, ANCHOR[0], ANCHOR[1], "d32l2"),
    (1024, ANCHOR[0], ANCHOR[1], "d32l1"),
    (128, "fixed-t250", ANCHOR[1], "d32l4"),
)

#: The same estimator at the second budget. Nothing exists here, so every arm
#: is this wave's own.
POINTS_E100 = (
    (128, ANCHOR[0], ANCHOR[1], "d32l2"),
    (1024, ANCHOR[0], ANCHOR[1], "d32l1"),
)

#: Which wave supplies each (epochs, temporal) arm. A contrast whose arm is not
#: in here is not computed at all -- there is no default.
SOURCE = {
    (400, "intact"): REUSED_WAVE,
    (400, "bin-shuffled"): REUSED_WAVE,
    (400, "reversed"): THIS_WAVE,
    (400, "channel-shuffled"): THIS_WAVE,
    (100, "intact"): THIS_WAVE,
    (100, "bin-shuffled"): THIS_WAVE,
    (100, "reversed"): THIS_WAVE,
    (100, "channel-shuffled"): THIS_WAVE,
}

SEED = re.compile(r"__s(\d+)\.json$")
DEPTH = re.compile(r"__(d\d+l\d+)")


def index(roots):
    """(wave, epochs, hidden, contract, geometry, arm, depth, temporal) -> {seed: acc}.

    The wave label and the epoch budget are both in the key. Dropping either
    would let two different experiments answer to one lookup.
    """
    out = collections.defaultdict(dict)
    voided = collections.Counter()
    for root in roots:
        root = Path(root)
        if not root.is_dir():
            continue
        for path in sorted(root.glob("*.json")):
            wave = path.name.split("__", 1)[0]
            if wave not in (THIS_WAVE, REUSED_WAVE):
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
            # A rescue-lever cell shares an operating point with a wave cell and
            # is a different experiment.
            if cell.get("clip_grad_norm") is not None:
                continue
            if cell.get("surrogate_scale") not in (1.0, None):
                continue
            if cell.get("epochs") not in (100, 400):
                continue
            arm = cell.get("arm")
            found = DEPTH.search(path.name)
            if arm == "ff+fixed+attn":
                if not found:
                    continue
                depth = found.group(1)
            elif arm == "ff+fixed":
                if found:
                    continue
                depth = None
            else:
                continue
            key = (wave, cell.get("epochs"), cell.get("hidden"), cell.get("contract"),
                   cell.get("geometry"), arm, depth,
                   cell.get("temporal_condition") or "intact")
            if validity_problems(cell):
                voided[key] += 1
                continue
            out[key].setdefault(int(seed.group(1)), cell["accuracy"])
    return out, voided


def arm(cells, epochs, hidden, contract, geometry, kind, depth, temporal):
    """One arm, fetched from the wave `SOURCE` names for it and from no other."""
    wave = SOURCE.get((epochs, temporal))
    if wave is None:
        return {}
    return cells.get((wave, epochs, hidden, contract, geometry, kind, depth, temporal), {})


def did(cells, epochs, hidden, contract, geometry, depth, condition):
    """(mean DiD, positive count, pairs) for one destruction operator.

    The rate arm is looked up at depth `None` -- it has no read-out -- while the
    attention arm is looked up at `depth`. That pairing is the whole contrast.
    """
    ai = arm(cells, epochs, hidden, contract, geometry, "ff+fixed+attn", depth, "intact")
    ax = arm(cells, epochs, hidden, contract, geometry, "ff+fixed+attn", depth, condition)
    ri = arm(cells, epochs, hidden, contract, geometry, "ff+fixed", None, "intact")
    rx = arm(cells, epochs, hidden, contract, geometry, "ff+fixed", None, condition)
    shared = sorted(set(ai) & set(ax) & set(ri) & set(rx))
    if not shared:
        return None, 0, 0
    deltas = [(ai[s] - ax[s]) - (ri[s] - rx[s]) for s in shared]
    return statistics.fmean(deltas), sum(d > 0 for d in deltas), len(deltas)


def label(hidden, contract, geometry, depth):
    return f"h{hidden} / {contract} / {geometry} / {depth}"


def main() -> int:
    cells, voided = index([ARCHIVE_V2])

    print("# Wave 24 — order, synchrony, and the second budget\n")
    print("Registered: `PREREG_2026-09-01_ORDER_SYNCHRONY_AND_BUDGET.md`.")
    print("This analyser is the authority on every verdict below.\n")
    print(f"Pinned binary this contrast requires: `{PINNED_BINARY}`.")
    print(f"`e400` intact and bin-shuffled arms are reused from `{REUSED_WAVE}`; "
          f"every other arm is `{THIS_WAVE}`.\n")

    own = sum(len(v) for k, v in cells.items() if k[0] == THIS_WAVE)
    print(f"`{THIS_WAVE}` cells indexed: **{own}** of {TOTAL_CELLS} planned.\n")

    # ---- H24-1 and H24-2, at e400 -----------------------------------------
    print("## H24-1 — is the cost generic perturbation brittleness?\n")
    print(f"`reversed` is displacement-matched and information-preserving. "
          f"The bar is the wave-21/22 bar used from the other side: a null must "
          f"**fail** to clear +{H24_1_MAX_REVERSED_DID:.2f}.\n")
    print("| point | DiD(bin) | DiD(reversed) | ratio | positive | pairs | verdict |")
    print("|---|---:|---:|---:|---:|---:|---|")
    h24_1 = []
    baseline = {}
    for hidden, contract, geometry, depth in POINTS_E400:
        b, _, bp = did(cells, 400, hidden, contract, geometry, depth, "bin-shuffled")
        r, rpos, rp = did(cells, 400, hidden, contract, geometry, depth, "reversed")
        baseline[(hidden, contract, geometry, depth)] = (b, bp)
        if b is None or r is None or bp < MIN_PAIRS or rp < MIN_PAIRS:
            print(f"| {label(hidden, contract, geometry, depth)} | "
                  f"{'—' if b is None else f'{b:+.4f}'} | — | — | — | {rp} | "
                  f"not evaluable |")
            h24_1.append(None)
            continue
        met = r < H24_1_MAX_REVERSED_DID
        h24_1.append(met)
        ratio = "—" if b == 0 else f"{r / b:+.2f}"
        print(f"| {label(hidden, contract, geometry, depth)} | {b:+.4f} | {r:+.4f} | "
              f"{ratio} | {rpos}/{rp} | {rp} | "
              f"{'**MET**' if met else '**NOT MET**'} |")
    verdict(h24_1, "H24-1",
            "the cost is not generic perturbation brittleness at the points measured",
            "at least one point's displacement-matched null clears the same bar the "
            "shuffle does -- the manuscript's mechanism sentence is withdrawn")

    print("\n## H24-2 — does cross-channel synchrony contribute?\n")
    print("Registered as a QUESTION, not a prediction. `channel-shuffled` destroys "
          f"order **and** synchrony; the band is ±{H24_2_SYNCHRONY_BAND:.2f}.\n")
    print("| point | DiD(bin) | DiD(channel) | difference | pairs | answer |")
    print("|---|---:|---:|---:|---:|---|")
    for hidden, contract, geometry, depth in POINTS_E400:
        b, bp = baseline[(hidden, contract, geometry, depth)]
        c, _, cp = did(cells, 400, hidden, contract, geometry, depth, "channel-shuffled")
        if b is None or c is None or bp < MIN_PAIRS or cp < MIN_PAIRS:
            print(f"| {label(hidden, contract, geometry, depth)} | "
                  f"{'—' if b is None else f'{b:+.4f}'} | — | — | {cp} | not evaluable |")
            continue
        diff = c - b
        if abs(diff) <= H24_2_SYNCHRONY_BAND:
            answer = "order alone"
        elif diff > 0:
            answer = "**synchrony also read**"
        else:
            answer = "**more destruction, less cost — investigate**"
        print(f"| {label(hidden, contract, geometry, depth)} | {b:+.4f} | {c:+.4f} | "
              f"{diff:+.4f} | {cp} | {answer} |")

    # ---- H24-3 and H24-4, at e100 -----------------------------------------
    print("\n## H24-3 — does the contrast survive a second budget?\n")
    print(f"The H22-1 bar unchanged: DiD > +{H24_3_MIN_DID:.2f}, positive in "
          f"≥ {H24_3_MIN_POSITIVE} of 12.\n")
    print("| point | budget | DiD(bin) | positive | pairs | verdict |")
    print("|---|---|---:|---:|---:|---|")
    h24_3 = []
    for hidden, contract, geometry, depth in POINTS_E100:
        v, pos, pairs = did(cells, 100, hidden, contract, geometry, depth, "bin-shuffled")
        if v is None or pairs < MIN_PAIRS:
            print(f"| {label(hidden, contract, geometry, depth)} | e100 | — | — | "
                  f"{pairs} | not evaluable |")
            h24_3.append(None)
            continue
        met = v > H24_3_MIN_DID and pos >= H24_3_MIN_POSITIVE
        h24_3.append(met)
        print(f"| {label(hidden, contract, geometry, depth)} | e100 | {v:+.4f} | "
              f"{pos}/{pairs} | {pairs} | {'**MET**' if met else '**NOT MET**'} |")
    verdict(h24_3, "H24-3",
            "the mechanism is not a property of the e400 budget alone",
            "the DiD belongs to the long-budget regime, and the paper says so")

    print("\n## H24-4 — does the order control hold at the second budget?\n")
    print("| point | budget | DiD(reversed) | pairs | verdict |")
    print("|---|---|---:|---:|---|")
    h24_4 = []
    for hidden, contract, geometry, depth in POINTS_E100:
        v, _, pairs = did(cells, 100, hidden, contract, geometry, depth, "reversed")
        if v is None or pairs < MIN_PAIRS:
            print(f"| {label(hidden, contract, geometry, depth)} | e100 | — | {pairs} | "
                  f"not evaluable |")
            h24_4.append(None)
            continue
        met = v < H24_4_MAX_REVERSED_DID
        h24_4.append(met)
        print(f"| {label(hidden, contract, geometry, depth)} | e100 | {v:+.4f} | "
              f"{pairs} | {'**MET**' if met else '**NOT MET**'} |")
    verdict(h24_4, "H24-4",
            "the e400 null is not a convergence artefact",
            "reversal is cheap only at e400, so H24-1's support is weaker than it looks "
            "and neither result may be quoted alone")

    # ---- provenance --------------------------------------------------------
    print("\n## Provenance — which wave supplied each arm\n")
    print("| budget | condition | source wave | attention arms found | rate arms found |")
    print("|---|---|---|---:|---:|")
    for (epochs, temporal), wave in sorted(SOURCE.items()):
        points = POINTS_E400 if epochs == 400 else POINTS_E100
        a = sum(1 for h, c, g, d in points
                if arm(cells, epochs, h, c, g, "ff+fixed+attn", d, temporal))
        geometries = {(h, c, g) for h, c, g, _ in points}
        r = sum(1 for h, c, g in geometries
                if arm(cells, epochs, h, c, g, "ff+fixed", None, temporal))
        print(f"| e{epochs} | {temporal} | `{wave}` | {a}/{len(points)} | "
              f"{r}/{len(geometries)} |")

    print(f"\n## H24-5 — cells\n")
    print(f"`{THIS_WAVE}` cells that passed validity: **{own}** of {TOTAL_CELLS}.")
    dropped = sum(v for k, v in voided.items() if k[0] == THIS_WAVE)
    print(f"Voided by `cell_validity`: **{dropped}**.")
    print(f"**H24-5: {'MET' if own == TOTAL_CELLS and dropped == 0 else 'NOT MET'}**"
          " — mechanical; it can fail only by cells failing to run.")
    return 0


def verdict(flags, name, met_reading, not_met_reading):
    if not flags or any(f is None for f in flags):
        print(f"\n**{name}: NOT EVALUABLE** — at least one point did not reach "
              f"{MIN_PAIRS} seed-paired quadruples. A hypothesis that could not be "
              f"tested is not a hypothesis that passed.")
    elif all(flags):
        print(f"\n**{name}: MET** — {met_reading}.")
    else:
        print(f"\n**{name}: NOT MET** — {not_met_reading}.")


if __name__ == "__main__":
    raise SystemExit(main())
