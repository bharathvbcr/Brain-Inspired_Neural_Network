#!/usr/bin/env python3
"""Verdicts for the source-versus-pinned-binary equivalence test.

Registered in `results/PREREG_2026-08-22_SOURCE_VERSUS_PINNED_BINARY.md`.

Comparison is field by field over the same set Gate F uses, not a whole-file
diff. That is not a convenience: the pinned binary predates `clip_sample_grad_norm`
and `clipped_samples`, so today's cells legitimately carry fields the archive
cannot have. A whole-file diff would report that as a disagreement and bury the
question actually being asked. Fields present on one side only are reported
separately, and a field present on both that differs is a failure.

    python3 scripts/aws/analyse_equivalence.py --out <downloaded equivalence/out>
"""

from __future__ import annotations

import argparse
import json
import os
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent
sys.path.insert(0, str(ROOT / "scripts"))
from gate_f_rust import COMPARED_FIELDS, COMPARED_TRACES  # noqa: E402

ARCHIVE_DIRS = [
    ROOT / "results" / "shd_attention_campaign_v1" / "cells",
    ROOT / "results" / "shd_attention_campaign_v2",
]


def archived(cell_id: str) -> dict | None:
    for directory in ARCHIVE_DIRS:
        path = directory / f"{cell_id}.json"
        if path.is_file():
            return json.loads(path.read_text())
    return None


def compare(left: dict, right: dict) -> tuple[list[str], list[str]]:
    """Differences on shared scientific fields, and fields only one side has."""
    differences: list[str] = []
    for field in COMPARED_FIELDS:
        if field not in left or field not in right:
            continue
        # `repr` rather than `==`: these are serialised decimals, and the
        # question is whether the two runs wrote the same characters, not
        # whether they round to the same float.
        if repr(left[field]) != repr(right[field]):
            differences.append(f"{field}: {left[field]!r} vs {right[field]!r}")
    for trace in COMPARED_TRACES:
        if not left.get(trace) or not right.get(trace):
            continue
        if left[trace] != right[trace]:
            first = next(
                (i for i, (a, b) in enumerate(zip(left[trace], right[trace])) if a != b),
                None,
            )
            if len(left[trace]) != len(right[trace]):
                differences.append(
                    f"{trace}: lengths {len(left[trace])} vs {len(right[trace])}"
                )
            else:
                differences.append(
                    f"{trace}: first differs at epoch {first} "
                    f"({left[trace][first]!r} vs {right[trace][first]!r})"
                )
    only = sorted((set(left) ^ set(right)))
    return differences, only


def load(path: Path) -> dict | None:
    return json.loads(path.read_text()) if path.is_file() else None


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", required=True, help="downloaded equivalence/out directory")
    args = parser.parse_args()
    out = Path(args.out)

    environment = load(out / "environment.json")
    lines: list[str] = []
    def w(text=""):
        lines.append(text)

    w("# Source versus pinned binary — equivalence verdicts\n")
    if environment:
        w(f"Instance `{environment['instance']}`, {environment['uname']}, "
          f"{environment['nproc']} vCPU, {environment['threads']} threads per cell.")
        w(f"- pinned  `{environment['pinned_sha256'][:16]}…`")
        w(f"- today   `{environment['today_sha256'][:16]}…`")
        w(f"- {environment['glibc']}")
        w(f"- {environment['rustc']}\n")
        if environment["pinned_sha256"] == environment["today_sha256"]:
            w("**The two binaries are byte-identical, which makes E-1 trivially "
              "true and uninformative.**\n")

    cells = sorted(p.name for p in (out / "today").iterdir()) if (out / "today").is_dir() else []
    if not cells:
        w("**No cells found — the run produced nothing to compare.**")
        print("\n".join(lines))
        return 1

    failures = 0
    unfinished = 0
    w("## E-1 (primary) — today's build vs the pinned binary, same host\n")
    w("| cell | verdict |")
    w("|---|---|")
    for cell in cells:
        today = load(out / "today" / cell / "cell.json")
        pinned = load(out / "pinned" / cell / "cell.json")
        if today is None or pinned is None:
            missing = "today" if today is None else "pinned"
            w(f"| `{cell[:52]}` | **UNFINISHED** ({missing} produced no cell) |")
            unfinished += 1
            continue
        differences, only = compare(today, pinned)
        if differences:
            failures += 1
            w(f"| `{cell[:52]}` | **DIFFERS** — {'; '.join(differences[:3])} |")
        else:
            note = f" (fields on one side only: {', '.join(only)})" if only else ""
            w(f"| `{cell[:52]}` | identical{note} |")
    w()

    for label, which in (("E-2 — the pinned binary vs the archive (tests the environment)", "pinned"),
                         ("E-3 — today's build vs the archive (follows from E-1 and E-2)", "today")):
        w(f"## {label}\n")
        w("| cell | verdict |")
        w("|---|---|")
        for cell in cells:
            observed = load(out / which / cell / "cell.json")
            record = archived(cell)
            if observed is None:
                w(f"| `{cell[:52]}` | UNFINISHED |")
                continue
            if record is None:
                w(f"| `{cell[:52]}` | no archived cell |")
                continue
            differences, _ = compare(observed, record)
            # The E-1 and E-4 loops count their failures; this one rendered
            # `differences` into the table and discarded it, so a real archive
            # disagreement — exactly what E-2 exists to detect — still ended at
            # "**Every comparison is identical.**" with exit 0.
            if differences:
                failures += 1
            w(f"| `{cell[:52]}` | {'**DIFFERS** — ' + '; '.join(differences[:2]) if differences else 'identical'} |")
        w()

    w("## E-4 — thread-count invariance, end to end\n")
    t1_root = out / "today-t1"
    if not t1_root.is_dir():
        w("No single-thread replicate present.")
    else:
        for cell in sorted(p.name for p in t1_root.iterdir()):
            one = load(t1_root / cell / "cell.json")
            four = load(out / "today" / cell / "cell.json")
            if one is None or four is None:
                w(f"- `{cell[:52]}`: UNFINISHED")
                continue
            differences, _ = compare(one, four)
            if differences:
                failures += 1
                w(f"- `{cell[:52]}`: **DIFFERS** at 1 vs 4 threads — {'; '.join(differences[:3])}")
            else:
                w(f"- `{cell[:52]}`: identical at 1 and 4 threads")
    w()

    w("## Verdict\n")
    if unfinished:
        w(f"**{unfinished} cell(s) unfinished** — reported as unfinished, not replaced.")
    if failures:
        w(f"**{failures} comparison(s) differ.** Today's source does not reproduce the "
          "campaign binary on every path tested; see the tables above for which.")
    elif not unfinished:
        w("**Every comparison is identical.** Today's source is behaviourally the "
          "campaign binary on the paths tested, on aarch64/glibc.")
    print("\n".join(lines))
    return 1 if (failures or unfinished) else 0


if __name__ == "__main__":
    raise SystemExit(main())
