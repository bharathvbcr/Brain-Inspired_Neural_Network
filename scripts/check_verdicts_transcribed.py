#!/usr/bin/env python3
"""A hand-written result must state the verdict its frozen analyser computed.

# The gap this closes

The analysers write `VERDICTS_W*.md`. A person then writes the `RESULT_*.md`
that the paper cites, and every verdict in it is retyped. Nothing compared the
two, so a wave's write-up could say SUPPORTED where its analyser said NOT
SUPPORTED and every other gate in the repository would stay green:
`verify_published_numbers.py` checks numbers, not verdicts, and the analyser is
frozen precisely so that **it** is the authority — which only helps if what gets
published is what it said.

This is the same shape as the number check and the opposite failure: there, a
mistyped digit; here, a mistyped conclusion, which is worse and easier to make,
because a verdict is one word and the temptation to soften it is real.

# What it does not check

Only verdicts stated as `SUPPORTED`, `NOT SUPPORTED` or `NOT EVALUABLE` against
a hypothesis id. A hypothesis registered as descriptive — wave 8's S-3b, wave 9's
M-3 — has no verdict by design and is skipped rather than demanded, because
demanding one is how a descriptive result acquires a claim it was registered not
to make.

    python3 scripts/check_verdicts_transcribed.py
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CAMPAIGN = ROOT / "results/shd_attention_campaign_v2"

#: `(generated verdicts, the write-up that must agree with them)`.
PAIRS = [
    ("VERDICTS_W8.md", "RESULT_2026-08-21_W8_HEADLINE_SCOPE_IS_MEASURED.md"),
    ("VERDICTS_W12.md", "RESULT_2026-08-23_W12_ATTENTION_DOES_NOT_SUBSTITUTE_FOR_ADAPTATION.md"),
    ("VERDICTS_W13.md", "RESULT_2026-08-23_W13_RECURRENT_STABILITY.md"),
    ("VERDICTS_W14.md", "RESULT_2026-08-23_W14_ATTENTION_AND_RECURRENCE_ARE_COMPLEMENTARY.md"),
]

VERDICT = "SUPPORTED|NOT SUPPORTED|NOT EVALUABLE"
#: The analysers write `**X-1** … -> **VERDICT**`.
GENERATED = re.compile(rf"\*\*([A-Z]-\d+[a-z]?)\*\*(.*?)(?:->|→)\s*\*\*({VERDICT})\*\*", re.S)
#: A write-up states it either as a heading, `**X-1 — VERDICT`, or in a table.
HAND_PROSE = re.compile(rf"\*\*([A-Z]-\d+[a-z]?)\s*[—–-]+\s*({VERDICT})")
HAND_TABLE = re.compile(
    rf"\*\*([A-Z]-\d+[a-z]?)\*\*[^\n|]*\|[^\n|]*\|[^\n|]*\|\s*\*\*({VERDICT})\*\*")


def main() -> int:
    compared = 0
    problems: list[str] = []

    for generated_name, hand_name in PAIRS:
        generated_path, hand_path = CAMPAIGN / generated_name, ROOT / "results" / hand_name
        for path in (generated_path, hand_path):
            if not path.is_file():
                problems.append(f"missing {path.relative_to(ROOT)}")
        if problems:
            continue

        generated = {k: v for k, _, v in GENERATED.findall(generated_path.read_text())}
        hand_text = hand_path.read_text()
        hand = dict(HAND_PROSE.findall(hand_text)) | dict(HAND_TABLE.findall(hand_text))

        if not generated:
            problems.append(f"{generated_name}: no verdicts parsed; the analyser's "
                            "output format has moved and this check is not looking "
                            "at anything")
            continue

        shared = sorted(generated.keys() & hand.keys())
        missing = sorted(generated.keys() - hand.keys())
        wrong = [(k, generated[k], hand[k]) for k in shared if generated[k] != hand[k]]
        compared += len(shared)

        status = "all agree" if not wrong else "DISAGREE"
        print(f"  [{'ok  ' if not wrong else 'FAIL'}] {hand_name[:60]:<60} "
              f"{len(shared)} compared, {status}")
        for hypothesis, said, wrote in wrong:
            problems.append(f"{hand_name}: {hypothesis} — analyser said "
                            f"{said!r}, the write-up says {wrote!r}")
        if missing:
            # Not a failure: a hypothesis registered as descriptive has no
            # verdict, and the write-up records it as a trend instead.
            print(f"         not restated as a verdict (descriptive, or stated as "
                  f"a trend): {', '.join(missing)}")

    if compared < 10:
        print(f"\nonly {compared} verdicts compared; the patterns have stopped "
              "matching", file=sys.stderr)
        return 1

    print(f"\n{compared} verdicts cross-checked against their frozen analyser")
    if problems:
        print(f"{len(problems)} problem(s):")
        for line in problems:
            print(f"  {line}")
        return 1
    print("every published verdict is the one its analyser computed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
