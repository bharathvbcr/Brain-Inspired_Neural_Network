#!/usr/bin/env python3
"""Where the paper's difference-in-differences can be computed, and where it cannot.

`PAPER_DRAFT.md`'s lead claim is a difference-in-differences on the *gain*:
attention's cost under bin-shuffling against the rate read-out's own cost, on
the same seeds and the same destruction operator. §3.5 reports it as +0.1347
against +0.0142, a 9.5x factor at n=32.

Computing it at one operating point needs **four** arms on shared seeds:

    rate / intact      rate / bin-shuffled
    attention / intact attention / bin-shuffled

The intact pair is what every width and geometry rung already carries. The
shuffled pair is not: on 2026-08-27 the corpus held twenty operating points with
attention cells and **two** where all four arms exist, both of them
h128 / `published-2ms` / `adjacent-sum-5`.

That is the paper's largest scope limit and it was not written down anywhere.
This derives it rather than restating it, for the same reason
`cross_isa_reproduction.py` derives its overlap: a hand-kept list of what is
covered is a list that stops being true and does not say so.

    python3 scripts/mechanism_coverage.py                 # table + verdict
    python3 scripts/mechanism_coverage.py --results DIR   # include live cells
    python3 scripts/mechanism_coverage.py --markdown      # table only

Exit 1 if no operating point supports the contrast at all, or if the corpus
roots are missing -- a coverage report that could not run must not read like a
coverage report that found nothing.
"""

from __future__ import annotations

import argparse
import collections
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

DEFAULT_ROOTS = (
    ROOT / "results/shd_attention_campaign_v2",
    ROOT / "results/shd_attention_campaign_v1/cells",
)

#: The paper's contrast is stated for the feed-forward fixed-threshold
#: substrate. `rec+alif` carries its own shuffle question and its own wave.
SUBSTRATE = ("ff+fixed", "ff+fixed+attn")

#: The destruction operator §3.5 names. `channel-shuffled` destroys a different
#: structure and answers a different question; counting it here would report
#: coverage the claim does not have.
DESTRUCTION = "bin-shuffled"

SEED = re.compile(r"__s(\d+)\.json$")
DEPTH = re.compile(r"__(d\d+l\d+)")


def read(roots) -> list[dict]:
    """Every comparable cell, flattened. Missing roots are fatal, not empty."""
    found, missing = [], []
    for root in roots:
        root = Path(root)
        if not root.is_dir():
            missing.append(str(root))
            continue
        for path in sorted(root.glob("*.json")):
            seed = SEED.search(path.name)
            if not seed:
                continue
            try:
                cell = json.loads(path.read_text())
            except ValueError:
                continue
            if not (isinstance(cell, dict) and "accuracy" in cell):
                continue
            depth = DEPTH.search(path.name)
            found.append({
                "seed": int(seed.group(1)),
                "hidden": cell.get("hidden"),
                "contract": cell.get("contract"),
                "geometry": cell.get("geometry"),
                "epochs": cell.get("epochs"),
                "depth": depth.group(1) if depth else None,
                "arm": cell.get("arm"),
                "surrogate_scale": cell.get("surrogate_scale"),
                "clip_grad_norm": cell.get("clip_grad_norm"),
                "temporal": cell.get("temporal_condition") or "intact",
            })
    if missing:
        raise SystemExit("corpus root(s) missing: " + ", ".join(missing))
    return found


def coverage(cells):
    """One row per operating point: the four arm counts and the paired n.

    The rate read-out has no read-out depth, so its cells belong to every depth
    at the same width, geometry and contract -- pairing them per depth is what
    makes the contrast a difference of differences rather than two unrelated
    drops.
    """
    usable = [c for c in cells
              if c["epochs"] == 400 and c["clip_grad_norm"] is None
              and c["surrogate_scale"] in (1.0, None)
              and c["arm"] in SUBSTRATE]
    rate: dict = collections.defaultdict(set)
    attn: dict = collections.defaultdict(set)
    for c in usable:
        point = (c["hidden"], c["contract"], c["geometry"])
        if c["depth"] is None:
            rate[(point, c["temporal"])].add(c["seed"])
        else:
            attn[(point, c["depth"], c["temporal"])].add(c["seed"])

    rows = []
    for point, depth, _ in sorted(attn, key=lambda k: (k[0], k[1])):
        if any(r[0] == point and r[1] == depth for r in rows):
            continue
        ri = rate.get((point, "intact"), set())
        rs = rate.get((point, DESTRUCTION), set())
        ai = attn.get((point, depth, "intact"), set())
        as_ = attn.get((point, depth, DESTRUCTION), set())
        rows.append((point, depth, len(ri), len(rs), len(ai), len(as_),
                     len(ri & rs & ai & as_)))
    return rows


def planned(path, cells) -> list[str]:
    """What the queued cells would add if every one of them landed.

    Coverage as measured answers "what can the paper say today". This answers
    "will the compute already committed buy what it was registered to buy" —
    and it is answerable the moment a wave is queued rather than after its last
    cell lands. Wave 21 is 168 cells and roughly 300 slot-hours; a wave whose
    geometry token or read-out depth did not line up with the intact arms it
    has to pair against would produce **nothing**, and would say so only at the
    end.

    A plan entry is turned into the same shape `read` produces, so the
    projection runs through `coverage` itself rather than through a second
    implementation of the pairing rule. Two implementations of that rule would
    drift, and the projection would then reassure about a pairing the analyser
    does not do.
    """
    try:
        entries = json.loads(Path(path).read_text())
    except (OSError, ValueError) as exc:
        raise SystemExit(f"could not read the plan at {path}: {exc}") from None

    projected = list(cells)
    for entry in entries:
        depth = (f"d{entry['attn_dim']}l{entry['attn_layers']}"
                 if entry.get("attn_dim") else None)
        projected.append({
            "seed": entry["seed"],
            "hidden": entry["hidden"],
            "contract": entry["contract"],
            "geometry": entry["geometry"],
            "epochs": entry["epochs"],
            "depth": depth,
            "arm": entry["arm"],
            "surrogate_scale": entry.get("surrogate_scale"),
            "clip_grad_norm": entry.get("clip_grad_norm"),
            "temporal": entry.get("temporal") or "intact",
        })

    now = {(point, depth) for point, depth, *_, n in coverage(cells) if n}
    then = {(point, depth) for point, depth, *_, n in coverage(projected) if n}
    gained = sorted(then - now, key=lambda k: (k[0], k[1]))

    out = ["",
           f"IF EVERY QUEUED CELL LANDS: {len(now)} → {len(then)} operating "
           f"point(s) could support the difference-in-differences."]
    if not gained:
        # The alarm this exists for. Queued compute that buys no new operating
        # point is either ground already covered or a wave whose cells cannot
        # pair with the intact arms they were registered against.
        out.append("  NO new operating point. The queued cells add nothing this "
                   "check can pair — verify the wave against the arms it is "
                   "meant to contrast with BEFORE its compute is spent.")
        return out
    for (hidden, contract, geometry), depth in gained:
        out.append(f"  + h{hidden} `{contract}` `{geometry}` {depth}")
    widths = sorted({point[0] for point, _ in then})
    contracts = sorted({point[1] for point, _ in then})
    geometries = sorted({point[2] for point, _ in then})
    out.append(f"  widths would become {widths}, contracts {contracts}, "
               f"geometries {geometries}")
    return out


def render(rows) -> list[str]:
    out = ["| width | contract | geometry | read-out | rate intact | rate shuffled "
           "| attn intact | attn shuffled | **contrast at n** |",
           "|---:|---|---|---|---:|---:|---:|---:|---:|"]
    for (h, contract, geometry), depth, ri, rs, ai, as_, n in rows:
        out.append(f"| {h} | `{contract}` | `{geometry}` | {depth} | "
                   f"{ri} | {rs} | {ai} | {as_} | "
                   f"{'**' + str(n) + '**' if n else '—'} |")
    return out


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--results", action="append", default=[])
    parser.add_argument("--markdown", action="store_true")
    parser.add_argument("--plan", help="a published cells.json; report what the "
                                       "queued cells would add if they all land")
    args = parser.parse_args()

    roots = list(DEFAULT_ROOTS) + [Path(r) for r in args.results]
    cells = read(roots)
    rows = coverage(cells)
    print("\n".join(render(rows)))
    if args.plan:
        print("\n".join(planned(args.plan, cells)))
    if args.markdown:
        return 0

    supported = [r for r in rows if r[6]]
    print(f"\n{len(supported)} of {len(rows)} operating point(s) can support the "
          f"difference-in-differences; {len(rows) - len(supported)} carry the "
          f"intact arms with no `{DESTRUCTION}` control.")
    if not supported:
        print("NO OPERATING POINT SUPPORTS THE CONTRAST — the paper's lead claim "
              "has no computable control. This is not a pass.")
        return 1
    widths = sorted({r[0][0] for r in supported})
    geometries = sorted({r[0][2] for r in supported})
    contracts = sorted({r[0][1] for r in supported})
    print(f"covered widths: {widths}   contracts: {contracts}   "
          f"geometries: {geometries}")
    if len(widths) == 1:
        print(f"SCOPE: the control exists at ONE width ({widths[0]}). Every "
              f"statement about the mechanism generalising beyond it is "
              f"unsupported by this corpus.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
