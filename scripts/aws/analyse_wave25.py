#!/usr/bin/env python3
"""Wave 25 — every place the mechanism claim is still asserted without a control.

Frozen with `results/PREREG_2026-09-02_THE_MECHANISM_WHERE_IT_IS_UNMEASURED.md`
in the same commit, before any cell of this wave existed. Every bar below is
that document's.

# What this adds over `analyse_wave24.py`

**`surrogate_scale` is in the key.** H25-1 runs `rec+alif` at ss 0.4, the
registered recurrent operating point, and the corpus holds `rec+alif` cells at
ss 1.0 as well. A key without the scale would let two different experiments
answer one lookup -- the same defect shape as merging two read-out depths, which
inflated a published shuffle cost by 17% (`AMENDMENT_2026-08-27_H17_2_...`).

**Three source waves, and every arm names its own.** H25-2 pairs new
`channel-shuffled` halves against `w22cov`; H25-3 pairs new attention twins
against `w22cov` rate arms; H25-4 borrows two `e100` rate geometries from
`w24ord`. A missing arm returns nothing and never falls back to another wave's
cell, and the provenance table prints what was found where.

Run: python3 scripts/aws/analyse_wave25.py
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

MIN_PAIRS = 9
H25_1_MIN_DID = 0.03            # the shuffle must clear it
H25_1_MAX_REVERSED = 0.03       # the displacement-matched null must not
H25_2_BAND = 0.03               # registered as a question
H25_3_MAX_RANGE = 0.10          # H22-3's bar, re-asked
H25_4_MIN_DID = 0.03
H25_4_MIN_POSITIVE = 9
TOTAL_CELLS = 888

PINNED_BINARY = "3afd4434431a75a26cc9d5fa46831341fc2f1dd0ef08dc308e18ca139b576364"

THIS = "w25mec"
W22 = "w22cov"
W24 = "w24ord"

ANCHOR = ("published-2ms", "adjacent-sum-5")
REC_SCALE = 0.4
#: `None` in a plan and `1.0` in a cell record are the same experiment.
DEFAULT_SCALE = 1.0

#: H25-2: the two rungs whose `channel-shuffled` halves this wave adds. The
#: third, `fixed-t250`, is wave 24's and is reprinted for comparison only.
SYNCHRONY_RUNGS = ("fixed-t100", "fixed-t500")

#: H25-3: wave 22's depth points, each against the `d32l4` twin this wave adds
#: at the same width on the anchor.
DEPTH_POINTS = ((128, "d32l2"), (128, "d64l4"), (256, "d32l1"),
                (512, "d32l1"), (768, "d32l2"))

#: H25-4: the nineteen points still measured only at e400.
POINTS_E100 = (
    (128, "fixed-t100", ANCHOR[1], "d32l4"), (128, "fixed-t250", ANCHOR[1], "d32l4"),
    (128, "fixed-t500", ANCHOR[1], "d32l4"), (128, "published-10ms", ANCHOR[1], "d32l4"),
    (128, ANCHOR[0], ANCHOR[1], "d32l1"), (128, ANCHOR[0], ANCHOR[1], "d32l4"),
    (128, ANCHOR[0], ANCHOR[1], "d64l4"), (128, ANCHOR[0], "channels-700", "d32l1"),
    (128, ANCHOR[0], "channels-700", "d32l4"), (256, ANCHOR[0], ANCHOR[1], "d32l1"),
    (256, ANCHOR[0], ANCHOR[1], "d32l4"), (384, ANCHOR[0], ANCHOR[1], "d32l4"),
    (512, ANCHOR[0], ANCHOR[1], "d32l1"), (512, ANCHOR[0], ANCHOR[1], "d32l4"),
    (768, ANCHOR[0], ANCHOR[1], "d32l2"), (768, ANCHOR[0], ANCHOR[1], "d32l4"),
    (1024, ANCHOR[0], ANCHOR[1], "d32l2"), (1024, ANCHOR[0], ANCHOR[1], "d32l3"),
    (1024, ANCHOR[0], ANCHOR[1], "d32l4"),
)
#: The two geometries whose e100 rate arms came from wave 24.
RATE_FROM_W24 = {(128, ANCHOR[0], ANCHOR[1]), (1024, ANCHOR[0], ANCHOR[1])}

SEED = re.compile(r"__s(\d+)\.json$")
DEPTH = re.compile(r"__(d\d+l\d+)")


def index(roots):
    """(wave, epochs, hidden, contract, geometry, arm, depth, temporal, scale)
    -> {seed: accuracy}."""
    out = collections.defaultdict(dict)
    voided = collections.Counter()
    for root in roots:
        root = Path(root)
        if not root.is_dir():
            continue
        for path in sorted(root.glob("*.json")):
            wave = path.name.split("__", 1)[0]
            if wave not in (THIS, W22, W24):
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
            if cell.get("clip_grad_norm") is not None:
                continue
            if cell.get("epochs") not in (100, 400):
                continue
            arm = cell.get("arm")
            found = DEPTH.search(path.name)
            if arm in ("ff+fixed+attn", "rec+alif+attn"):
                if not found:
                    continue
                depth = found.group(1)
            elif arm in ("ff+fixed", "rec+alif"):
                if found:
                    continue
                depth = None
            else:
                continue
            # The plan writes `surrogate_scale: None` for a default-scale cell
            # and the instrument records `1.0`, so the two spellings mean the
            # same experiment. Normalising them here is what stops a
            # `scale=None` lookup silently matching nothing and printing a
            # clean "not evaluable" over cells that are on disk -- which is
            # exactly what the first run of this analyser did, against wave 22
            # arms it should have found. 0.4 stays distinct, which is the whole
            # reason the field is in the key.
            scale = cell.get("surrogate_scale")
            scale = DEFAULT_SCALE if scale is None else round(float(scale), 4)
            key = (wave, cell.get("epochs"), cell.get("hidden"), cell.get("contract"),
                   cell.get("geometry"), arm, depth,
                   cell.get("temporal_condition") or "intact", scale)
            if validity_problems(cell):
                voided[key] += 1
                continue
            out[key].setdefault(int(seed.group(1)), cell["accuracy"])
    return out, voided


def arm(cells, wave, epochs, hidden, contract, geometry, kind, depth, temporal, scale):
    return cells.get((wave, epochs, hidden, contract, geometry, kind, depth,
                      temporal, scale), {})


def did(cells, condition, *, epochs, hidden, contract, geometry, depth,
        attn_arm="ff+fixed+attn", rate_arm="ff+fixed", scale=DEFAULT_SCALE,
        intact_wave, treated_wave, rate_intact_wave=None, rate_treated_wave=None):
    """(mean DiD, positive count, pairs). Every arm is fetched from a NAMED wave."""
    ri_wave = rate_intact_wave or intact_wave
    rt_wave = rate_treated_wave or treated_wave
    ai = arm(cells, intact_wave, epochs, hidden, contract, geometry, attn_arm, depth, "intact", scale)
    ax = arm(cells, treated_wave, epochs, hidden, contract, geometry, attn_arm, depth, condition, scale)
    ri = arm(cells, ri_wave, epochs, hidden, contract, geometry, rate_arm, None, "intact", scale)
    rx = arm(cells, rt_wave, epochs, hidden, contract, geometry, rate_arm, None, condition, scale)
    shared = sorted(set(ai) & set(ax) & set(ri) & set(rx))
    if not shared:
        return None, 0, 0
    deltas = [(ai[s] - ax[s]) - (ri[s] - rx[s]) for s in shared]
    return statistics.fmean(deltas), sum(d > 0 for d in deltas), len(deltas)


def fmt(value, width=7):
    return "—".rjust(width) if value is None else f"{value:+.4f}".rjust(width)


def verdict(flags, name, met, not_met):
    if not flags or any(f is None for f in flags):
        print(f"\n**{name}: NOT EVALUABLE** — at least one contrast did not reach "
              f"{MIN_PAIRS} seed-paired quadruples. A hypothesis that could not be "
              f"tested is not one that passed.")
    elif all(flags):
        print(f"\n**{name}: MET** — {met}.")
    else:
        print(f"\n**{name}: NOT MET** — {not_met}.")


def main() -> int:
    cells, voided = index([ARCHIVE_V2])
    own = sum(len(v) for k, v in cells.items() if k[0] == THIS)

    print("# Wave 25 — the mechanism where it was unmeasured\n")
    print("Registered: `PREREG_2026-09-02_THE_MECHANISM_WHERE_IT_IS_UNMEASURED.md`.")
    print("This analyser is the authority on every verdict below.\n")
    print(f"Pinned binary this contrast requires: `{PINNED_BINARY}`.")
    print(f"`{THIS}` cells indexed: **{own}** of {TOTAL_CELLS} planned.\n")

    # ---- H25-1 -------------------------------------------------------------
    print("## H25-1 — is the recurrent read-out's advantage order-dependent?\n")
    print("The first manipulated recurrent cells in this campaign. Both halves are "
          f"required: the shuffle must clear +{H25_1_MIN_DID:.2f} in ≥ {MIN_PAIRS} "
          f"of 12, and the displacement-matched null must fail to.\n")
    print("| substrate | condition | DiD | positive | pairs | requirement | verdict |")
    print("|---|---|---:|---:|---:|---|---|")
    common = dict(epochs=400, hidden=128, contract=ANCHOR[0], geometry=ANCHOR[1],
                  depth="d32l4", attn_arm="rec+alif+attn", rate_arm="rec+alif",
                  scale=REC_SCALE, intact_wave=THIS, treated_wave=THIS)
    h25_1 = []
    for condition, want_above in (("bin-shuffled", True), ("reversed", False)):
        value, positive, pairs = did(cells, condition, **common)
        if value is None or pairs < MIN_PAIRS:
            print(f"| `rec+alif` | `{condition}` | — | — | {pairs} | — | not evaluable |")
            h25_1.append(None)
            continue
        if want_above:
            ok = value > H25_1_MIN_DID and positive >= MIN_PAIRS
            need = f"> +{H25_1_MIN_DID:.2f}, ≥ {MIN_PAIRS}/12"
        else:
            ok = value < H25_1_MAX_REVERSED
            need = f"< +{H25_1_MAX_REVERSED:.2f}"
        h25_1.append(ok)
        print(f"| `rec+alif` | `{condition}` | {fmt(value)} | {positive}/{pairs} | "
              f"{pairs} | {need} | {'**MET**' if ok else '**NOT MET**'} |")
    verdict(h25_1, "H25-1",
            "the recurrent read-out consumes temporal order too, and §3.7's "
            "cross-reference to §3.5 is supported rather than assumed",
            "§3.7's interpretive sentence is WITHDRAWN and the paper's mechanism "
            "claim is explicitly feed-forward")

    # ---- H25-2 -------------------------------------------------------------
    print("\n## H25-2 — is the synchrony term a resolution effect?\n")
    print("Registered as a QUESTION. At `fixed-t250` wave 24 measured "
          f"+0.1038; the band is ±{H25_2_BAND:.2f}.\n")
    print("| rung | DiD(bin) | DiD(channel) | difference | pairs | answer |")
    print("|---|---:|---:|---:|---:|---|")
    for contract in SYNCHRONY_RUNGS:
        base = dict(epochs=400, hidden=128, contract=contract, geometry=ANCHOR[1],
                    depth="d32l4")
        b, _, bp = did(cells, "bin-shuffled", **base, intact_wave=W22, treated_wave=W22)
        c, _, cp = did(cells, "channel-shuffled", **base, intact_wave=W22,
                       treated_wave=THIS, rate_intact_wave=W22, rate_treated_wave=THIS)
        if b is None or c is None or bp < MIN_PAIRS or cp < MIN_PAIRS:
            print(f"| `{contract}` | {fmt(b)} | — | — | {cp} | not evaluable |")
            continue
        diff = c - b
        answer = ("order alone" if abs(diff) <= H25_2_BAND else
                  "**synchrony also read**" if diff > 0 else
                  "**more destruction, less cost — investigate**")
        print(f"| `{contract}` | {fmt(b)} | {fmt(c)} | {fmt(diff)} | {cp} | {answer} |")

    # ---- H25-3 -------------------------------------------------------------
    print("\n## H25-3 — does the contrast depend on read-out depth?\n")
    print("H22-3 re-asked with the twins its analyser needed. Registered as a "
          f"QUESTION; range bar {H25_3_MAX_RANGE:.2f}.\n")
    print("| width | depth | DiD | `d32l4` twin | difference | verdict |")
    print("|---:|---|---:|---:|---:|---|")
    h25_3 = []
    for width, depth in DEPTH_POINTS:
        point, _, pp = did(cells, "bin-shuffled", epochs=400, hidden=width,
                           contract=ANCHOR[0], geometry=ANCHOR[1], depth=depth,
                           intact_wave=W22, treated_wave=W22)
        twin, _, tp = did(cells, "bin-shuffled", epochs=400, hidden=width,
                          contract=ANCHOR[0], geometry=ANCHOR[1], depth="d32l4",
                          intact_wave=THIS, treated_wave=THIS,
                          rate_intact_wave=W22, rate_treated_wave=W22)
        if point is None or twin is None or pp < MIN_PAIRS or tp < MIN_PAIRS:
            print(f"| {width} | `{depth}` | {fmt(point)} | {fmt(twin)} | — | not evaluable |")
            h25_3.append(None)
            continue
        diff = abs(point - twin)
        ok = diff <= H25_3_MAX_RANGE
        h25_3.append(ok)
        print(f"| {width} | `{depth}` | {fmt(point)} | {fmt(twin)} | {diff:+.4f} | "
              f"{'within' if ok else '**outside**'} |")
    verdict(h25_3, "H25-3",
            "the contrast is a property of the read-out and not of its depth",
            "read-out depth is a scope axis for the mechanism claim, which nothing "
            "in the paper currently suggests")

    # ---- H25-4 -------------------------------------------------------------
    print("\n## H25-4 — does the contrast survive e100 across the design space?\n")
    print(f"The H22-1 bar unchanged: DiD > +{H25_4_MIN_DID:.2f}, positive in "
          f"≥ {H25_4_MIN_POSITIVE} of 12.\n")
    print("| point | read-out | DiD | positive | pairs | verdict |")
    print("|---|---|---:|---:|---:|---|")
    h25_4 = []
    for hidden, contract, geometry, depth in POINTS_E100:
        rate_wave = W24 if (hidden, contract, geometry) in RATE_FROM_W24 else THIS
        value, positive, pairs = did(cells, "bin-shuffled", epochs=100, hidden=hidden,
                                     contract=contract, geometry=geometry, depth=depth,
                                     intact_wave=THIS, treated_wave=THIS,
                                     rate_intact_wave=rate_wave,
                                     rate_treated_wave=rate_wave)
        label = f"h{hidden} / `{contract}` / `{geometry}`"
        if value is None or pairs < MIN_PAIRS:
            print(f"| {label} | `{depth}` | — | — | {pairs} | not evaluable |")
            h25_4.append(None)
            continue
        ok = value > H25_4_MIN_DID and positive >= H25_4_MIN_POSITIVE
        h25_4.append(ok)
        print(f"| {label} | `{depth}` | {fmt(value)} | {positive}/{pairs} | {pairs} | "
              f"{'**MET**' if ok else '**NOT MET**'} |")
    verdict(h25_4, "H25-4",
            "the budget scope limit is retired rather than narrowed",
            "the mechanism has a budget boundary, and the points where it fails are "
            "named in the abstract")

    # ---- provenance --------------------------------------------------------
    print("\n## Provenance — which wave supplied each arm\n")
    print("| group | arm | source wave | found |")
    print("|---|---|---|---:|")
    rows = [
        ("H25-1", "every arm", THIS,
         sum(1 for c in ("intact", "bin-shuffled", "reversed")
             for a, d in (("rec+alif+attn", "d32l4"), ("rec+alif", None))
             if arm(cells, THIS, 400, 128, ANCHOR[0], ANCHOR[1], a, d, c, REC_SCALE))),
        ("H25-2", "`channel-shuffled`", THIS,
         sum(1 for k in SYNCHRONY_RUNGS
             for a, d in (("ff+fixed+attn", "d32l4"), ("ff+fixed", None))
             if arm(cells, THIS, 400, 128, k, ANCHOR[1], a, d, "channel-shuffled", DEFAULT_SCALE))),
        ("H25-2", "`intact` / `bin-shuffled`", W22,
         sum(1 for k in SYNCHRONY_RUNGS for c in ("intact", "bin-shuffled")
             for a, d in (("ff+fixed+attn", "d32l4"), ("ff+fixed", None))
             if arm(cells, W22, 400, 128, k, ANCHOR[1], a, d, c, DEFAULT_SCALE))),
        ("H25-3", "`d32l4` attention twins", THIS,
         sum(1 for w in (128, 256, 512, 768) for c in ("intact", "bin-shuffled")
             if arm(cells, THIS, 400, w, ANCHOR[0], ANCHOR[1], "ff+fixed+attn", "d32l4", c, DEFAULT_SCALE))),
        ("H25-4", "attention", THIS,
         sum(1 for h, k, g, d in POINTS_E100 for c in ("intact", "bin-shuffled")
             if arm(cells, THIS, 100, h, k, g, "ff+fixed+attn", d, c, DEFAULT_SCALE))),
        ("H25-4", "rate (two geometries)", W24,
         sum(1 for h, k, g in RATE_FROM_W24 for c in ("intact", "bin-shuffled")
             if arm(cells, W24, 100, h, k, g, "ff+fixed", None, c, DEFAULT_SCALE))),
    ]
    for group, what, wave, found in rows:
        print(f"| {group} | {what} | `{wave}` | {found} |")

    print("\n## H25-5 — cells\n")
    dropped = sum(v for k, v in voided.items() if k[0] == THIS)
    print(f"`{THIS}` cells that passed validity: **{own}** of {TOTAL_CELLS}. "
          f"Voided by `cell_validity`: **{dropped}**.")
    print(f"**H25-5: {'MET' if own == TOTAL_CELLS and dropped == 0 else 'NOT MET'}**"
          " — mechanical; it can fail only by cells failing to run.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
