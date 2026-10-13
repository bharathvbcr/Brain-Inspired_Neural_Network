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

#: A qualifier that names the variant or marks the sentence as about something
#: that is NOT the lead FAIL: a contrast arm, a denial of the general reading,
#: or one of the NumPy locality tasks whose arm is `err_broadcast` throughout.
#:
#: It must BIND to the term. A STRONG qualifier names the rule itself and may sit
#: within BIND_WORDS words of it, in the same sentence, before or after ("RL ±1
#: broadcast", "broadcast graded error"). A GENERIC word means something else on
#: its own ("error bars", "standard error", "as XOR shows"), so it only counts
#: as a compound: the very next word on either side ("scalar broadcast",
#: "broadcast error", "RL broadcast contrasts"), or the task label a table line
#: opens with ("xor_thresh: broadcast 0.501"). Before this split, generic words
#: cleared a +/-80 character window and then a 3-word window: "Broadcast fails
#: with error bars overlapping chance" still passed on the `error` of `error
#: bars`.
STRONG = re.compile(
    r"±1|three-factor|surrogate eligibility|graded|REINFORCE|"
    r"credit topolog|topological|err_broadcast|low-richness|e-prop", re.I)
GENERIC = re.compile(
    r"(?:error|scalar|RL|xor\w*|contrasts?|depth.locality)", re.I)
#: "broadcast error bars/rate" is the statistic, not the arm.
NOT_ARM_AFTER_ERROR = re.compile(r"(?:bars?|rates?|margins?|estimates?)$", re.I)
#: A task name opening the line's label: "xor_thresh: broadcast", "NumPy
#: depth_locality mid: broadcast".
TASK_LABEL = re.compile(r"(?:xor|depth.locality)\w*(?:\s+\w+)?\s*:\s*$", re.I)
#: A code span is a MENTION of the word only when the sentence says so.
MENTION_CUE = re.compile(r"\b(?:bare|word|term|string|token|phrase|label)\b", re.I)
#: "any broadcast" only counts when quoted, as the phrase being denied.
QUOTED_ANY = re.compile(r"[\"“‘']any\s*$", re.I)
BIND_WORDS = 3
SENTENCE_BREAK = re.compile(r"[.!?;](?:\s|$)|\n\s*\n")
#: DELIBERATE phrases are looked for this many characters around the term.
DELIBERATE_WINDOW = 80

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

#: Inflections too: "Broadcasting", "broadcasts", "broadcast-based".
WORD = re.compile(r"\bbroadcast\w*", re.I)
# Markdown emphasis and code ticks are not words.
MARKUP = re.compile(r"[*`]")


def _words(chunk: str) -> list[str]:
    return MARKUP.sub("", chunk).split()


def sentence_sides(text: str, match: re.Match) -> tuple[str, str]:
    """The text before and after the term, cut at the sentence's edges."""
    before = text[:match.start()]
    breaks = list(SENTENCE_BREAK.finditer(before))
    if breaks:
        before = before[breaks[-1].end():]
    after = text[match.end():]
    brk = SENTENCE_BREAK.search(after)
    if brk:
        after = after[:brk.start()]
    return before, after


def bound_context(text: str, match: re.Match) -> str:
    """The BIND_WORDS words either side of the term, stopping at the sentence."""
    before, after = sentence_sides(text, match)
    left = " ".join(_words(before)[-BIND_WORDS:])
    right = " ".join(_words(after)[:BIND_WORDS])
    return f"{left} {match.group(0)} {right}"


def _bare(token: str) -> str:
    return token.strip("()[]{}\"“”‘’'")


def is_qualified(text: str, match: re.Match) -> bool:
    """Whether the term carries a qualifier that binds to it (see STRONG)."""
    if STRONG.search(bound_context(text, match)):
        return True
    before, after = sentence_sides(text, match)
    # Tokens with no letter or digit ("&gt;", "/") are separators, not words.
    left = [t for t in _words(before) if re.search(r"\w", t)]
    right = [t for t in _words(after) if re.search(r"\w", t)]
    if left and GENERIC.fullmatch(_bare(left[-1])):
        return True
    if right and GENERIC.fullmatch(_bare(right[0])):
        return not (right[0].lower().startswith("error") and len(right) > 1
                    and NOT_ARM_AFTER_ERROR.match(right[1].rstrip(".,;:)")))
    return bool(TASK_LABEL.search(MARKUP.sub("", before)))


def is_mentioned(text: str, match: re.Match) -> bool:
    """A code span ``word`` that the sentence introduces as the word itself."""
    if text[max(0, match.start() - 1):match.start()] != "`" or \
            text[match.end():match.end() + 1] != "`":
        return False
    before, _ = sentence_sides(text, match)
    return bool(MENTION_CUE.search(" ".join(_words(before)[-BIND_WORDS:])))


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
            around = text[max(0, match.start() - DELIBERATE_WINDOW):
                          match.end() + DELIBERATE_WINDOW]
            if is_qualified(text, match):
                continue
            if is_mentioned(text, match):
                continue  # the word is being mentioned (a code span), not used
            if QUOTED_ANY.search(text[max(0, match.start() - 8):match.start()]):
                continue
            if any(phrase in around for phrase in DELIBERATE):
                continue
            line = text[:match.start()].count("\n") + 1
            excerpt = " ".join(around.split())
            failures.append(
                f"{doc.name}:{line} uses `broadcast` with no qualifier within "
                f"{BIND_WORDS} words in its sentence:\n      ...{excerpt}...\n"
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
