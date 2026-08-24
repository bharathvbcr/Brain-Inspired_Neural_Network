#!/usr/bin/env python3
"""Generate `results/INDEX.md` — every record document, classified and dated.

# Why

`results/` holds 487 markdown files and had no index. Finding a document meant
grepping for a phrase you already had to know, and — the part that matters —
**nothing showed at a glance which documents are retired.** A withdrawn result
sits in the same directory listing as a live one, with the same filename shape,
and only its own first fifteen lines say otherwise.

# What this does NOT do

It does not delete anything, and retired documents are listed rather than
hidden. A withdrawal that leaves no trace is indistinguishable from a claim that
was never made, and several of this campaign's most useful documents exist only
to record that an earlier one was wrong. The index marks them; it does not tidy
them away.

# Status detection

A document is **retired** if it opens with a `WITHDRAWN` or `SUPERSEDED` banner
in its own first lines. That is the repository's existing convention and it is
already applied consistently: an audit on 2026-08-23 found every document that
another document declares withdrawn also carries its own banner, with no
exceptions.

Deliberately *not* inferred from other documents' `**Withdraws:**` lines. Two of
those turned out to mean the opposite of retirement — one reads "Supersedes
nothing" and names a predecessor it *follows*, and one supersedes **the
withdrawal in** another document, which restores that document rather than
retiring it. Reading a claim out of a neighbouring file is how those become
errors.

    python3 scripts/build_results_index.py            # write results/INDEX.md
    python3 scripts/build_results_index.py --check    # fail if it is out of date
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
RESULTS = ROOT / "results"
INDEX = RESULTS / "INDEX.md"

#: Filename prefix -> section, in the order the sections are written.
KINDS = [
    ("PREREG", "Preregistrations", "Registered before the data existed."),
    ("RESULT", "Results", "Verdicts against a registration."),
    ("FINDING", "Findings", "Things discovered while looking for something else."),
    ("MEASUREMENT", "Measurements", "Numbers taken without a registered hypothesis."),
    ("AMENDMENT", "Amendments", "Changes to a registration, with their reasons."),
    ("DEFECT", "Defects", "Something the instrument got wrong."),
    ("HARDENING", "Hardening", "Changes that make a defect class impossible."),
    ("AUDIT", "Audits", "Systematic re-reading of an existing record."),
    ("BLOCKER", "Blockers", "Work that cannot proceed, and why."),
    ("SUMMARY", "Summaries", "Roll-ups across a period."),
    ("RECONCILIATION", "Reconciliations", "Two records that had to be made to agree."),
    ("PAPER", "Paper", "The manuscript and its supporting tables."),
    ("CAMPAIGN", "Campaign control", "Claim freezes and campaign-level decisions."),
    ("BLOG", "Writing", "Narrative pieces, not records."),
    ("GATE", "Gates", "Gate definitions and their working notes."),
    ("TODO", "Open work", "What is not done."),
    ("MATCHED", "Matched-architecture notes", "The matched dense-LIF control and its arms."),
    ("SHD", "SHD instrument notes", "Status and working notes for the SHD instrument."),
    ("BUILD", "Build audits", "Audits of the build itself."),
    ("HARD", "Hard audits", "Adversarial re-reads of the whole record."),
    ("HANDOFF", "Handoffs", "State at a handover point."),
    ("PUBLISHABLE", "Claim registers", "What may and may not be claimed."),
    ("REPRO", "Reproduction", "Artifact and reproduction checklists."),
]

#: A leading lowercase letter means the file was written by an experiment
#: binary rather than by a person. They are the raw reports the authored
#: documents cite, and they are indexed separately so a reader can tell at a
#: glance which is which - a generated report has no registration behind it and
#: is not a claim on its own.
GENERATED = "Experiment reports (generated)"

#: A retirement BANNER, not merely a line that starts with the word.
#:
#: The first version of this pattern was wrong in both directions on the same
#: run. It matched `withdrawn, what was hardened, and what is still open.` -- an
#: ordinary sentence that happened to begin a line -- and it missed
#: `> **SUPERSEDED 2026-08-22 by ...**`, because it allowed a blockquote marker
#: and a heading but not bold. So a live summary was listed as retired while a
#: genuinely superseded one was listed as live.
#:
#: A banner is therefore required to be emphasised (heading or bold) AND to be
#: followed by something that makes it a statement about this document: a date,
#: `by`, `IN PART`, or a dash introducing the reason.
RETIRED = re.compile(
    r"^>?\s*(?:#{1,3}\s*)?(?:\*\*)?(WITHDRAWN|SUPERSEDED)\b"
    r"(?=\s*(?:\d{4}-\d{2}-\d{2}|by\b|IN PART|[—–-]))(.*)$",
    re.M | re.I,
)
DATED = re.compile(r"(\d{4}-\d{2}-\d{2})")


def describe(path: Path) -> dict:
    text = path.read_text(errors="replace")
    lines = text.splitlines()
    title = next((line.lstrip("# ").strip() for line in lines if line.startswith("# ")), path.stem)
    banner = RETIRED.search("\n".join(lines[:15]))
    date = DATED.search(path.name)
    return {
        "name": path.name,
        "title": title,
        "date": date.group(1) if date else "",
        "retired": bool(banner),
        # The reason, trimmed to the sentence that fits a table cell.
        "why": (banner.group(2).strip(" —-–:") if banner else "")[:110],
    }


def kind_of(name: str) -> str:
    for prefix, section, _ in KINDS:
        if name.startswith(prefix):
            return section
    if name[:1].islower():
        return GENERATED
    return "Other"


def build() -> str:
    docs = [describe(p) for p in sorted(RESULTS.glob("*.md")) if p.name != INDEX.name]
    by_kind: dict[str, list[dict]] = {}
    for doc in docs:
        by_kind.setdefault(kind_of(doc["name"]), []).append(doc)

    retired = [d for d in docs if d["retired"]]
    out: list[str] = []
    out.append("<!-- Generated by scripts/build_results_index.py. Do not edit by hand. -->")
    out.append("")
    out.append("# Record index")
    out.append("")
    out.append(f"{len(docs)} documents. **{len(retired)} are retired** — withdrawn or "
               "superseded — and are listed here rather than removed, because a "
               "withdrawal that leaves no trace is indistinguishable from a claim "
               "that was never made.")
    out.append("")
    out.append("| section | live | retired |")
    out.append("|---|---:|---:|")
    ordered = [s for _, s, _ in KINDS if s in by_kind]
    ordered += [s for s in (GENERATED, "Other") if s in by_kind]
    for section in ordered:
        group = by_kind[section]
        out.append(f"| {section} | {sum(1 for d in group if not d['retired'])} | "
                   f"{sum(1 for d in group if d['retired'])} |")
    out.append("")

    if retired:
        out.append("## Retired documents")
        out.append("")
        out.append("Read these only through the document that retired them.")
        out.append("")
        out.append("| document | status |")
        out.append("|---|---|")
        for doc in sorted(retired, key=lambda d: d["name"]):
            why = doc["why"] or "see the document's own banner"
            out.append(f"| [{doc['name']}]({doc['name']}) | {why} |")
        out.append("")

    for section in ordered:
        blurbs = {GENERATED: "Written by an experiment binary, not by a person. "
                             "A generated report carries no registration and is "
                             "not a claim on its own.",
                  "Other": "Not yet classified by filename prefix."}
        _, _, blurb = next(((p, s, b) for p, s, b in KINDS if s == section),
                           ("", section, blurbs.get(section, "Uncategorised.")))
        out.append(f"## {section}")
        out.append("")
        out.append(f"*{blurb}*")
        out.append("")
        out.append("| date | document | title |")
        out.append("|---|---|---|")
        for doc in sorted(by_kind[section], key=lambda d: (d["date"], d["name"]), reverse=True):
            mark = " **[retired]**" if doc["retired"] else ""
            title = doc["title"].replace("|", "\\|")
            out.append(f"| {doc['date'] or '—'} | [{doc['name']}]({doc['name']}){mark} | {title} |")
        out.append("")

    return "\n".join(out) + "\n"


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true",
                        help="fail if INDEX.md is not what this would write")
    args = parser.parse_args()

    generated = build()
    if not args.check:
        INDEX.write_text(generated)
        print(f"wrote {INDEX.relative_to(ROOT)}")
        return 0

    if not INDEX.is_file():
        print("results/INDEX.md is missing; run scripts/build_results_index.py",
              file=sys.stderr)
        return 1
    if INDEX.read_text() != generated:
        print("results/INDEX.md is out of date; run scripts/build_results_index.py",
              file=sys.stderr)
        return 1
    print("results/INDEX.md is current")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
