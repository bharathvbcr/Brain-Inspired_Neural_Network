#!/usr/bin/env python3
"""`broadcast` never appears bare where it could mean the lead FAIL.

The manuscript's lead matched negative is **broadcast ±1 three-factor** — a
single ±1 reward times surrogate eligibility — and it is NOT a result about
broadcast credit topology in general: on the same schedule a broadcast-*graded*
contrast reaches 0.9975 and ±1 broadcast REINFORCE reaches 0.7775. Four
documents record the requirement to say which one is meant. None of them
checked it, so the requirement lived as a note rather than a property, and one
sentence in §3.4 had already drifted to a bare `broadcast` for an arm the
figure spec calls `err_broadcast`.

Bare uses that are deliberate are listed below with the reason. Everything else
must carry a qualifier. The failure this prevents is a reader taking the lead
negative to be wider than it is, which is the single most likely way this paper
gets over-cited.
"""
from __future__ import annotations

import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
DOCS = [
    ROOT / "results/PAPER_DRAFT.md",
    ROOT / "results/PAPER_FIGURE_SPEC.md",
    ROOT / "results/PUBLISHABLE_CLAIMS.md",
    ROOT / "results/VENUE_FORMATTING.md",
]

#: A qualifier close enough to the word to disambiguate it. Checked in a window
#: around the occurrence rather than by sentence, because the qualifier is as
#: often before ("RL ±1 broadcast") as after ("broadcast graded error").
QUALIFIER = re.compile(
    # Names the variant outright.
    r"±1 three-factor|three-factor|graded|error|scalar|REINFORCE|±1|"
    r"credit topolog|topological|err_broadcast|"
    # Or marks the sentence as being about something that is NOT the lead
    # FAIL: a contrast arm, a denial of the general reading, or one of the
    # NumPy locality tasks whose arm is `err_broadcast` throughout.
    r"contrasts?\b|any broadcast|\bRL\b|xor|depth.locality|e-prop", re.I)
WINDOW = 80

#: Bare uses that are correct as written. Each is a phrase, matched exactly.
DELIBERATE = {
    '"any broadcast"': "the sentence exists to deny the general reading",
    "“any broadcast.”": "same, in the figure spec's curly quotes",
    "‘any broadcast’": "same, inside a required caption",
    "Low addressability (broadcast)": "an axis label on Figure M's grid, where "
                                      "the whole column is the broadcast half",
    "(broadcast fails; DFA solves)": "the XOR locality flip, where the arm is "
                                     "named err_broadcast two lines above",
    "(broadcast also solves there)": "same passage, same arm",
    "the broadcast bar": "refers to the bar drawn in the panel under discussion",
    "Broadcast (err_broadcast)": "a table row that names the arm in the cell",
    "broadcast 0.5008": "a value that identifies the arm on its own",
}

WORD = re.compile(r"\bbroadcast\b", re.I)


def main() -> int:
    failures: list[str] = []
    checked = 0
    for doc in DOCS:
        if not doc.exists():
            failures.append(f"{doc} is missing; the terminology it records "
                            "cannot be checked, and a skipped check must not "
                            "read as a passing one.")
            continue
        text = doc.read_text()
        for match in WORD.finditer(text):
            checked += 1
            around = text[max(0, match.start() - WINDOW):match.end() + WINDOW]
            if QUALIFIER.search(around):
                continue
            if any(phrase in around for phrase in DELIBERATE):
                continue
            line = text[:match.start()].count("\n") + 1
            excerpt = " ".join(around.split())
            failures.append(
                f"{doc.name}:{line} uses `broadcast` with no qualifier within "
                f"{WINDOW} characters:\n      ...{excerpt}...\n"
                "    The lead matched FAIL is broadcast ±1 three-factor "
                "specifically; broadcast-graded reaches 0.9975 and ±1 "
                "broadcast REINFORCE 0.7775. Name which arm is meant, or add "
                "the phrase to DELIBERATE with the reason it is unambiguous.")
    for line in failures:
        print(f"  FAIL: {line}")
    if failures:
        print(f"\n{len(failures)} unqualified use(s) of {checked} checked.")
        return 1
    print(f"terminology: {checked} use(s) of `broadcast` across {len(DOCS)} "
          f"document(s), every one qualified or deliberately bare "
          f"({len(DELIBERATE)} recorded exceptions).")
    return 0


if __name__ == "__main__":
    sys.exit(main())
