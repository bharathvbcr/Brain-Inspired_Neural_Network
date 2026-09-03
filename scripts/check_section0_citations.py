#!/usr/bin/env python3
"""Every literature number in §0 is covered by a primary-source check.

§0 is the one part of the manuscript whose numbers do not come from cells on
disk, so `check_every_number.py` cannot reach them: its NUMBER pattern is
four-decimal and §0 quotes two-decimal percentages out of published papers.
Until 2026-09-03 that meant §0 had no gate at all, and it carried a standing
note saying so. Reading the sources closed the note. This closes the gap the
note was standing in for, because a one-off read is not a check -- it stops
being true the moment someone adds a number.

What this asserts, and what it deliberately does not:

  IT DOES     require that every literature percentage in §0 appears in the
              verification record's table of checks, so a NEW number cannot
              enter §0 without someone reading a source for it.
  IT DOES     require that each correction the record describes actually
              landed in §0 and stays landed -- the wrong values are named
              here and must be absent.
  IT DOES     require every source the record hashes to have a bibliography
              entry, so the .bib and the record cannot drift apart.
  IT DOES NOT re-verify anything against a PDF. There is no network here and
              a cached PDF would only prove the cache. The record names the
              sha256 of every file that was read, which is what makes the
              2026-09-03 pass repeatable by hand; this script is the guard
              against silent drift after it, not a repeat of it.

A number that this script cannot attribute is an ERROR, not a warning. The
failure mode it exists to prevent is a percentage arriving in §0 from a search
result and reading, to a reader, exactly like one that was checked.
"""
from __future__ import annotations

import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
DRAFT = ROOT / "results/PAPER_DRAFT.md"
RECORD = ROOT / "results/CITATIONS_2026-09-03_SECTION_0_AGAINST_PRIMARIES.md"
BIB = ROOT / "results/references.bib"

#: The manuscript's literature region: everything from the abstract to the end
#: of §1. §0 is where the positioning lives, but the abstract and introduction
#: quote the same numbers, and on 2026-09-03 the abstract was still carrying a
#: frontier band that §0 had already had corrected -- a gate scoped to §0 alone
#: would have reported a pass over it.
SECTION_START = "## Abstract"
SECTION_END = "## 2. Methods"

#: The figure generator draws the frontier band, so a correction that lands in
#: prose and not in the artwork ships a wrong figure at camera-ready quality.
#: That has happened in this repository before. These constants are checked
#: against the record too.
FIGURES_RS = ROOT / "binn-lab/src/paper_figures.rs"
DRAWN_CONSTANTS = {
    "FIELD_FRONTIER_LO": "95.07",
    "FIELD_FRONTIER_HI": "96.3",
    "FIELD_STSC_SNN": "92.36",
    "FIELD_TA_SNN": "91.08",
}

#: A literature value in this manuscript is a PERCENTAGE: two or three digits
#: before the point, one or two after (91.08, 96.3, 68.51). The instrument's own
#: values are four-decimal fractions below 1 (0.8332, +0.1275) and are swept by
#: `check_every_number.py` against cells. Separating them by shape rather than
#: by section is what lets this gate cover the abstract and §1 as well as §0
#: without duplicating that sweep -- and the abstract is exactly where a
#: corrected frontier band survived uncorrected until 2026-09-03.
LITERATURE_SHAPED = re.compile(r"(?<![\d.])(\d{2,3}\.\d{1,2})(?![\d])")

#: Percentage-shaped values in the region that are NOT quoted from a paper.
#: Each says why; an unexplained exemption is how a gate stops gating.
NOT_A_CITATION = {
    "109.2": "bin-shuffling's mean displacement in bins, from this "
             "instrument's own per-cell manipulation audit",
    "145.2": "time reversal's mean displacement in bins, same audit",
    "16.043": "part of the arXiv identifier 2507.16043",
    "10.03464": "part of the arXiv identifier 2410.03464",
}

#: Uncertainties quoted alongside a mean. Each is covered by the record beside
#: the mean it belongs to, which the mean's own coverage already establishes.
DISPERSIONS = {"0.24", "0.56", "0.76", "1.3"}

#: The three corrections of 2026-09-03. Each is (wrong text, what it was).
#: If any reappears in §0 the correction has been undone, and a reader would
#: have no way to tell -- the record would still describe a fix that is gone.
MUST_NOT_REAPPEAR = [
    ("95–96.4", "the frontier band's unsourced upper bound"),
    ("96.4", "the frontier band's unsourced upper bound"),
    ("spiking transformers", "an attribution with no citation behind it"),
    ("Three\nindependent destruction operators", "Yu et al. counted as a third "
     "vote for order-dependence, which is not what they report"),
]

#: Every key the record hashes must be citable. The record is the owner of
#: this list; duplicating it here would just be a second thing to update.
BIB_KEY = re.compile(r"^\| `([a-z0-9]+)` \|", re.M)


def section_zero(text: str) -> str:
    if SECTION_START not in text:
        raise SystemExit(f"FAIL: {DRAFT.name} has no section 0 heading. "
                         "The gate cannot report a pass on a section it "
                         "could not find.")
    body = text.split(SECTION_START, 1)[1]
    if SECTION_END not in body:
        raise SystemExit(f"FAIL: {DRAFT.name} section 0 has no end marker "
                         f"({SECTION_END!r}); refusing to sweep the rest of "
                         "the manuscript as if it were section 0.")
    return body.split(SECTION_END, 1)[0]


def main() -> int:
    for path in (DRAFT, RECORD, BIB):
        if not path.exists():
            print(f"FAIL: {path} is missing. A citation gate with no record "
                  "is not a gate.")
            return 1

    draft, record, bib = (p.read_text() for p in (DRAFT, RECORD, BIB))
    body = section_zero(draft)
    failures: list[str] = []

    # 1. Every literature number in the region is one the record checked.
    for value in sorted(set(LITERATURE_SHAPED.findall(body))):
        if value in NOT_A_CITATION or value in DISPERSIONS:
            continue
        if value not in record:
            failures.append(
                f"the abstract/§0/§1 region quotes {value} and {RECORD.name} does not mention it. "
                "Read the source and record the check, or remove the number. "
                "An unchecked percentage here is indistinguishable to a "
                "reader from a checked one, which is the whole reason this "
                "gate exists.")

    # 2. The corrections stayed corrected. Compared case-insensitively and
    #    with runs of whitespace flattened: a reverted sentence that came back
    #    capitalised differently, or rewrapped by an editor, is the same
    #    revert, and a gate that misses it because of a line break is worse
    #    than no gate because it reports a pass.
    flat = " ".join(body.split()).lower()
    for wrong, what in MUST_NOT_REAPPEAR:
        if " ".join(wrong.split()).lower() in flat:
            failures.append(
                f"the abstract/§0/§1 region contains {wrong!r} again — {what}. The record says this "
                "was corrected on 2026-09-03; either the correction was "
                "reverted or the record is now wrong about the manuscript.")

    # 3. Record and bibliography agree on the sources.
    keys = BIB_KEY.findall(record)
    if len(keys) < 8:
        failures.append(
            f"{RECORD.name} lists only {len(keys)} hashed sources. The "
            "2026-09-03 pass read ten; a shrunken table would let this gate "
            "pass by checking less.")
    for key in keys:
        if f"{{{key}," not in bib:
            failures.append(
                f"{RECORD.name} hashes source `{key}` but {BIB.name} has no "
                f"entry `{key}`. The record and the bibliography have drifted, "
                "and the citation in the built PDF would resolve to nothing.")

    # 4. The disclosure in §0 still points at the record.
    if RECORD.name not in body:
        failures.append(
            f"§0 no longer links {RECORD.name}. The provenance note is what "
            "tells a reader these numbers were read rather than searched.")

    # 5. The artwork draws the same band the prose states.
    if not FIGURES_RS.exists():
        failures.append(f"{FIGURES_RS} is missing; the frontier band the "
                        "figures draw cannot be checked against the record.")
    else:
        rust = FIGURES_RS.read_text()
        for const, value in DRAWN_CONSTANTS.items():
            match = re.search(rf"{const}: f64 = ([0-9.]+);", rust)
            if match is None:
                failures.append(
                    f"{FIGURES_RS.name} no longer defines {const}. The figure "
                    "spec and this gate both expect it; a renamed constant "
                    "silently stops being checked.")
                continue
            drawn = f"{float(match.group(1)) * 100:g}"
            if drawn != value:
                failures.append(
                    f"{FIGURES_RS.name} draws {const} = {drawn}% and the "
                    f"verified record says {value}%. Prose and artwork have "
                    "diverged; the figure is what a reader looks at.")

    for line in failures:
        print(f"  FAIL: {line}")
    checked = len({v for v in LITERATURE_SHAPED.findall(body)
                   if v not in NOT_A_CITATION and v not in DISPERSIONS})
    if failures:
        print(f"\n{len(failures)} problem(s). §0 carries {checked} literature "
              "value(s).")
        return 1
    print(f"abstract+§0+§1: {checked} literature value(s), every one covered by "
          f"{RECORD.name}; {len(keys)} sources hashed and all present in "
          f"{BIB.name}; {len(MUST_NOT_REAPPEAR)} correction(s) still in place.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
