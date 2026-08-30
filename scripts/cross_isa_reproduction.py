#!/usr/bin/env python3
"""Every Azure cell that has an AWS twin, compared field by field.

`FINDING_2026-08-22_REPRODUCIBLE_ACROSS_ISA_UNDER_GLIBC.md` reported 36 cell
pairs and 57,960 values. That was the overlap someone went looking for — the
three expensive width configurations — and it was **hand-selected**. The
archive holds more than that: the truncated Azure campaign and the AWS waves
overlap on **eight** configurations, not three, and the four that were missed
include the paper's own headline width.

So the overlap is now *derived* rather than chosen. Every Azure cell is keyed by
its scientific configuration, every AWS cell likewise, and each Azure cell with
a same-configuration same-seed twin is compared through the canonical
comparator in `scripts/compare_cells.py`. Nothing is enumerated by hand, so a
configuration cannot be left out by not being thought of.

    python3 scripts/cross_isa_reproduction.py            # table + exit status
    python3 scripts/cross_isa_reproduction.py --markdown # table only

Exit 1 on any differing value, on a pair count below the pinned floor, or if a
corpus goes missing — a comparison that could not run must not report the same
thing as one that ran and agreed.

**On the value count.** This counts *leaf* values: every scalar field plus every
element of every per-epoch trajectory, `wall_secs` excluded as a timing rather
than a measurement. The finding's 57,960 counted ten scalars plus four
400-element trajectories per cell; this counts every serialised field, so the
per-cell figure is slightly larger for the same cells. The two numbers are the
same agreement measured with a different denominator, not a disagreement.
"""

from __future__ import annotations

import argparse
import collections
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "scripts"))
from compare_cells import IGNORED  # noqa: E402

#: The x86-64 corpus, and the aarch64 corpora it is compared against.
AZURE = ROOT / "results/azure-d32l4-scope-v1/results"
AWS = [
    ROOT / "results/shd_attention_campaign_v1/cells",
    ROOT / "results/shd_attention_campaign_v2",
]

#: Raised 2026-08-27 from 79/8 when wave 16 regenerated the h256/d32l4 rung:
#: its four Azure cells gained AWS twins and are byte-identical to them, so the
#: overlap grew by one configuration and four pairs and the excluded set fell
#: from 16 cells to 12. The floor moves with the corpus or it stops being a floor.
#: Pinned so that a narrowed glob or a moved directory fails loudly instead of
#: comparing fewer cells and still reporting agreement. Raise it when a wave
#: adds overlap; never lower it to make a run pass.
MIN_PAIRS = 83
MIN_CONFIGURATIONS = 9


def configuration(path: Path, cell: dict) -> tuple:
    """The scientific identity of a cell: everything but seed, host and timing."""
    dim = re.search(r"__(d\d+l\d+)", path.name)
    return (
        cell.get("arm"), cell.get("contract"), cell.get("geometry"),
        cell.get("hidden"), cell.get("epochs"), dim.group(1) if dim else "rate",
        cell.get("surrogate_scale"), cell.get("temporal_condition"),
        cell.get("clip_grad_norm"),
    )


def load(roots: list[Path]) -> dict[tuple, dict[int, tuple[Path, dict]]]:
    out: dict[tuple, dict[int, tuple[Path, dict]]] = {}
    for root in roots:
        if not root.is_dir():
            raise SystemExit(f"corpus missing: {root}")
        for path in sorted(root.glob("*.json")):
            seed = re.search(r"__s(\d+)\.json$", path.name)
            if not seed:
                continue
            try:
                cell = json.loads(path.read_text())
            except json.JSONDecodeError:
                continue
            if not (isinstance(cell, dict) and "accuracy" in cell):
                continue
            out.setdefault(configuration(path, cell), {})[int(seed.group(1))] = (path, cell)
    return out


def leaves(cell: dict) -> dict[str, object]:
    """Every comparable leaf, trajectories flattened, `wall_secs` dropped."""
    flat: dict[str, object] = {}
    for key, value in cell.items():
        if key in IGNORED:
            continue
        if isinstance(value, list):
            for index, item in enumerate(value):
                flat[f"{key}[{index}]"] = item
        elif isinstance(value, dict):
            for sub, item in value.items():
                flat[f"{key}.{sub}"] = item
        else:
            flat[key] = value
    return flat


#: Fields the cell schema GAINED after some cells were already written.
#:
#: An absent field is not agreement, and that guard must survive: a truncated
#: or thin cell has to fail rather than pass by having less to disagree about.
#: But a field that simply did not exist when the older cell was written is a
#: different thing, and until 2026-08-30 the two were indistinguishable here —
#: every schema addition made this gate print "REPRODUCTION FAILED", which
#: reads as a scientific disagreement and was not one. Worse, a real mismatch
#: would have been one line among hundreds of false ones.
#:
#: So each addition is DECLARED, with when and why. An undeclared missing field
#: still fails, which is the property `test_a_missing_field_is_a_disagreement`
#: pins. This list may only grow forward, and every entry is a field that a
#: newer cell has and an older one cannot.
SCHEMA_ADDITIONS = {
    "emitted_unix_s": "2026-08-27, provenance: when the cell was produced",
    "emitted_utc": "2026-08-27, the same timestamp in readable form",
    "seed": "2026-08-27, provenance for attentive arms",
    "clip_sample_grad_norm": "per-sample clipping, absent before it existed",
    "clipped_samples": "the counter for the above",
    "non_finite_forward": "2026-08-29, the forward-finiteness guard "
                          "(DEFECT_2026-08-29_THE_EVALUATION_FORWARD_WAS_NEVER_CHECKED.md). "
                          "No cell written before that date can carry it.",
}


def compare_pair(left: dict, right: dict) -> tuple[int, list[str], list[str]]:
    """Leaves compared, the ones that differ, and declared schema gaps.

    `repr` so 1.0 != 1.
    """
    a, b = leaves(left), leaves(right)
    shared = sorted(set(a) & set(b))
    differing = [k for k in shared if repr(a[k]) != repr(b[k])]
    schema_only = []
    for key in sorted(set(a) ^ set(b)):
        # `epoch_mean_loss[3]` and friends flatten to an indexed leaf; the
        # declaration is on the field, not on every index of it.
        field = key.split("[", 1)[0].split(".", 1)[0]
        if field in SCHEMA_ADDITIONS:
            schema_only.append(key)
        else:
            differing.append(f"{key} (present in one cell only)")
    return len(shared), differing, schema_only


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--markdown", action="store_true",
                        help="print the table only, no verdict line")
    args = parser.parse_args()

    azure, aws = load([AZURE]), load(AWS)
    rows: dict[tuple, list] = collections.defaultdict(lambda: [0, 0, []])
    unmatched: collections.Counter = collections.Counter()
    schema_gaps: set = set()

    for config, seeds in sorted(azure.items()):
        for seed, (_, cell) in sorted(seeds.items()):
            twin = aws.get(config, {}).get(seed)
            if twin is None:
                unmatched[config] += 1
                continue
            count, differing, schema_only = compare_pair(cell, twin[1])
            schema_gaps.update(f.split("[", 1)[0].split(".", 1)[0] for f in schema_only)
            row = rows[config]
            row[0] += 1
            row[1] += count
            row[2].extend(f"{config} s{seed}: {d}" for d in differing)

    print("| configuration | cell pairs | values | differing |")
    print("|---|---:|---:|---:|")
    pairs = values = differ = 0
    for config in sorted(rows, key=lambda c: (c[5] == "rate", c[3], c[1], c[4])):
        count, leaf, bad = rows[config]
        pairs, values, differ = pairs + count, values + leaf, differ + len(bad)
        arm = "rate-only" if config[5] == "rate" else config[5]
        print(f"| h{config[3]} / {arm} / `{config[1]}` / e{config[4]} "
              f"| {count} | {leaf:,} | {len(bad)} |")
    print(f"| **total** | **{pairs}** | **{values:,}** | **{differ}** |")

    if unmatched:
        print(f"\n{sum(unmatched.values())} Azure cell(s) with no AWS twin, "
              "excluded from the table and from the claim:")
        for config, count in sorted(unmatched.items()):
            arm = "rate-only" if config[5] == "rate" else config[5]
            print(f"  h{config[3]} / {arm} / {config[1]} / e{config[4]}: {count}")

    if args.markdown:
        return 0

    problems = []
    if pairs < MIN_PAIRS:
        problems.append(f"{pairs} pairs, floor {MIN_PAIRS} — the overlap shrank; "
                        "a smaller comparison must not report the same result")
    if len(rows) < MIN_CONFIGURATIONS:
        problems.append(f"{len(rows)} configurations, floor {MIN_CONFIGURATIONS}")
    if differ:
        problems.append(f"{differ} differing value(s)")
        lines = sorted(x for row in rows.values() for x in row[2])
        for line in lines[:10]:
            problems.append(f"  {line}")
        # The cap used to be silent, and a silent cap reads as the whole list.
        if len(lines) > 10:
            problems.append(f"  ... and {len(lines) - 10} more not shown")
    # Declared schema additions are NOT failures and are NOT hidden either. A
    # field one side cannot carry is a fact about provenance, and the reader is
    # told which so "compared fewer fields" can never quietly become "agreed".
    if schema_gaps:
        print(f"\n{len(schema_gaps)} declared schema addition(s) skipped, present "
              f"on one side only and not counted as disagreement:")
        for field in sorted(schema_gaps):
            print(f"  {field}  — {SCHEMA_ADDITIONS[field]}")
    if problems:
        print("\nCROSS-ISA REPRODUCTION FAILED")
        for line in problems:
            print(f"  {line}")
        return 1
    print(f"\n{values:,}/{values:,} values identical across {pairs} cell pairs "
          f"and {len(rows)} configurations, aarch64 vs x86-64.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
