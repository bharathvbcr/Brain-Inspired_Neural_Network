#!/usr/bin/env python3
"""Regress every cell one corpus duplicates *within itself*, wave against wave.

# Why this exists

Wave 18 registers H18-4: twelve h1024/d32/L2 cells must come back byte-identical
to the `w15col` cells of the same id, and if they do not, **every cell in waves
18 and 19 is void**. It is the campaign's only check on the execution
environment rather than on the code, and it is the right check.

Its problem is *when*. Those twelve cells sit at plan indices 140-151 of 192, so
on 2026-08-27 the fleet had 54 unclaimed cells queued ahead of the check that
could void them, plus 26 in flight. A reproduction failure would have been found
after most of the compute it invalidates had already been spent.

Reordering a live queue to fix that is worse than the disease: appending to a
published queue mid-campaign is what killed eighty claims on 2026-08-27. So this
takes the other route. Instead of one duplicate pair scheduled at one point,
find **every** pair the corpus already contains and check them all, whenever
asked. On 2026-08-27 that was 64 cells across six configurations, sixteen of
them wave 18's own — available while the fleet was at 68 of 192.

# Why it is not `cross_isa_reproduction.py`

It is the same idea on a different axis, and it composes over that file rather
than restating it: `configuration()` and `compare_pair()` are imported, so the
definition of "the same experiment" and of "identical" cannot drift between the
two checks.

The axis is the difference. `cross_isa_reproduction` keys Azure cells against
AWS cells, one corpus against another. Its `load()` folds every root into a
single map keyed by configuration and seed, so **two cells of the same
configuration and seed inside one corpus overwrite each other** and the pair is
never seen. That is exactly the pair this file looks for. The two are
complementary: one asks whether two machines agree, this asks whether one fleet
still agrees with its own record.

# What a PASS does and does not mean

It does **not** discharge H18-4, which is registered on named cells against a
named archive and is answered only by those cells. A PASS here removes one
explanation for a future H18-4 failure. A FAIL is the alarm, and a real one:
the fleet is not reproducing the record, and nothing produced since the
divergence can be compared with anything before it.
"""

from __future__ import annotations

import argparse
import collections
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts"))

# The canonical definitions of "the same experiment" and "identical", shared
# with the cross-ISA check so the two cannot drift apart.
from cross_isa_reproduction import compare_pair, configuration  # noqa: E402

#: Searched by default: the campaign corpora this repository carries.
DEFAULT_ROOTS = (
    ROOT / "results/shd_attention_campaign_v2",
    ROOT / "results/shd_attention_campaign_v1/cells",
)

#: Files that live beside the cells and are deliberately not cells. Counted
#: separately from files this genuinely could not read: nine expected sidecars
#: printed as UNREADABLE on every run is how a reader learns to skip the line
#: that will one day carry a real one.
SIDECAR = re.compile(r"^(manifest|plan_w[0-9_]+)$")

SEED = re.compile(r"__s(\d+)$")


def wave_of(stem: str) -> str | None:
    """The wave tag a cell id starts with, or None if it has no separator."""
    head, sep, _ = stem.partition("__")
    return head if sep else None


def collect(roots) -> tuple[dict, list[str], int]:
    """(index, unreadable files, sidecar count).

    The index maps configuration -> seed -> {wave: (source, cell)}. Keeping the
    wave as a third level is the whole point: `cross_isa_reproduction.load()`
    stops at seed, and so silently keeps only one cell per (configuration, seed)
    however many waves produced it.
    """
    index: dict = collections.defaultdict(lambda: collections.defaultdict(dict))
    unreadable: list[str] = []
    sidecars = 0
    for root in roots:
        root = Path(root)
        if not root.is_dir():
            continue
        for path in sorted(root.glob("*.json")):
            if SIDECAR.match(path.stem):
                sidecars += 1
                continue
            seed = SEED.search(path.stem)
            wave = wave_of(path.stem)
            if not seed or not wave:
                unreadable.append(str(path))
                continue
            try:
                cell = json.loads(path.read_text())
            except ValueError:
                unreadable.append(str(path))
                continue
            if not (isinstance(cell, dict) and "accuracy" in cell):
                unreadable.append(str(path))
                continue
            config = configuration(path, cell)
            # First writer wins for a given wave: a cell present in two archive
            # roots is one cell, not a duplicate run.
            index[config][int(seed.group(1))].setdefault(wave, (root.name, cell))
    return index, unreadable, sidecars


def sweep(index):
    """(agreements, disagreements, values compared) over every duplicated cell."""
    agree, differ, values = [], [], 0
    gaps: set = set()
    for config, by_seed in sorted(index.items(), key=lambda kv: repr(kv[0])):
        for seed, by_wave in sorted(by_seed.items()):
            if len(by_wave) < 2:
                continue
            waves = sorted(by_wave)
            base = by_wave[waves[0]][1]
            for other in waves[1:]:
                # `compare_pair` gained a third return on 2026-08-30: fields
                # one side cannot carry because its schema predates them. They
                # are reported, never counted as disagreement -- see
                # `SCHEMA_ADDITIONS` there for why the two must stay distinct.
                count, differing, schema_only = compare_pair(by_wave[other][1], base)
                values += count
                gaps.update(f.split("[", 1)[0].split(".", 1)[0] for f in schema_only)
                record = (config, seed, waves[0], other, differing)
                (differ if differing else agree).append(record)
    return agree, differ, values, gaps


def label(config) -> str:
    arm, contract, geometry, hidden, epochs, dim, scale, temporal, clip = config
    bits = [f"{arm}", f"h{hidden}", f"e{epochs}", f"{contract}", f"{dim}"]
    if scale is not None:
        bits.append(f"ss{scale}")
    if temporal and temporal != "intact":
        bits.append(str(temporal))
    if clip is not None:
        bits.append(f"clip{clip}")
    return " / ".join(bits)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--results", action="append", default=[],
                        help="extra directory of cells; repeatable")
    parser.add_argument("--archive", action="append", default=[],
                        help="replace the default archive roots; repeatable")
    args = parser.parse_args()

    roots = [Path(a) for a in args.archive] or list(DEFAULT_ROOTS)
    roots += [Path(r) for r in args.results]
    index, unreadable, sidecars = collect(roots)
    cells = sum(len(w) for s in index.values() for w in s.values())
    agree, differ, values, schema_gaps = sweep(index)

    print(f"roots: {len(roots)}   cells indexed: {cells}   "
          f"configurations: {len(index)}   sidecars skipped: {sidecars}")
    if unreadable:
        print(f"UNREADABLE: {len(unreadable)} file(s) that are not sidecars and "
              f"could not be read as cells; they were compared against nothing")
        for path in unreadable[:5]:
            print(f"  {path}")

    if not (agree or differ):
        # The distinction the whole file rests on: nothing to check is not the
        # same answer as everything checked out.
        print("\nNO DUPLICATED CELLS — nothing was compared. This is NOT a pass.")
        return 2

    print(f"\nduplicated cells compared: {len(agree) + len(differ)}   "
          f"values: {values:,}")
    for (waves, config), n in sorted(collections.Counter(
            ((a, b), label(c)) for c, _, a, b, _ in agree).items()):
        print(f"  [ ok ] {n:3d} cell(s)  {waves[0]} vs {waves[1]}  {config}")

    if schema_gaps:
        # Reported, never counted as disagreement, and never silent: a field
        # one side cannot carry is a provenance fact, and hiding it would let
        # "compared fewer fields" drift into "agreed".
        print(f"\n{len(schema_gaps)} declared schema addition(s) skipped: "
              f"{', '.join(sorted(schema_gaps))}")

    if differ:
        print(f"\nREPRODUCTION FAILED — {len(differ)} of "
              f"{len(agree) + len(differ)} duplicated cells disagree.")
        for config, seed, a, b, differing in differ[:10]:
            print(f"  {a} vs {b}  seed {seed}  {label(config)}")
            print(f"      {', '.join(str(d) for d in differing[:6])}")
        if len(differ) > 10:
            print(f"  ... and {len(differ) - 10} more")
        return 1

    print(f"\nALL {len(agree)} DUPLICATED CELLS BYTE-IDENTICAL over "
          f"{values:,} compared values.")
    print("This does NOT discharge H18-4, which is answered only by the cells "
          "its preregistration names.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
