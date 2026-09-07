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
#: Waves 26-28 landed in a third campaign directory beside the second, and a
#: single hardcoded `CAMPAIGN` could not see their verdicts however they were
#: spelled. Resolution is across all of them and an ambiguous name is an error,
#: so a `VERDICTS_W28.md` in two corpora fails rather than silently picking one.
CAMPAIGNS = [
    ROOT / "results/shd_attention_campaign_v2",
    ROOT / "results/shd_attention_campaign_v3",
]


def resolve(name: str) -> Path | None:
    """The one campaign directory holding `name`, or None if not exactly one."""
    found = [c / name for c in CAMPAIGNS if (c / name).is_file()]
    return found[0] if len(found) == 1 else None

#: `(generated verdicts, the write-up that must agree with them)`.
PAIRS = [
    ("VERDICTS_W8.md", "RESULT_2026-08-21_W8_HEADLINE_SCOPE_IS_MEASURED.md"),
    ("VERDICTS_W12.md", "RESULT_2026-08-23_W12_ATTENTION_DOES_NOT_SUBSTITUTE_FOR_ADAPTATION.md"),
    ("VERDICTS_W13.md", "RESULT_2026-08-23_W13_RECURRENT_STABILITY.md"),
    ("VERDICTS_W14.md", "RESULT_2026-08-23_W14_ATTENTION_AND_RECURRENCE_ARE_COMPLEMENTARY.md"),
    ("VERDICTS_W15.md", "RESULT_2026-08-27_W15_17_THE_COLLAPSE_IS_A_THRESHOLD.md"),
    ("VERDICTS_W18.md",
     "RESULT_2026-08-28_W18_19_THE_DEPTH_OPTIMUM_IS_INTERIOR.md"),
    ("VERDICTS_W20.md",
     "RESULT_2026-08-28_W20_THE_RECURRENT_CLAIM_HOLDS_AT_THIRTY_TWO_SEEDS.md"),
    ("VERDICTS_W21.md",
     "RESULT_2026-08-29_W21_THE_MECHANISM_TRAVELS_BUT_ITS_SIZE_DOES_NOT.md"),
    ("VERDICTS_W23.md",
     "RESULT_2026-08-30_W23_THE_COLLAPSE_IS_LATE.md"),
    ("VERDICTS_W22.md",
     "RESULT_2026-09-01_W22_THE_MECHANISM_AT_EVERY_OPERATING_POINT.md"),
    ("VERDICTS_W24.md",
     "RESULT_2026-09-02_W24_ORDER_SYNCHRONY_AND_BUDGET.md"),
    ("VERDICTS_W25.md",
     "RESULT_2026-09-03_W25_THE_MECHANISM_WHERE_IT_WAS_UNMEASURED.md"),
    ("VERDICTS_W26.md",
     "RESULT_2026-09-04_W26_SATURATION_IS_REAL_AND_NOT_SPECIFIC.md"),
    ("VERDICTS_W27.md",
     "RESULT_2026-09-05_W27_THE_TIMESCALE_IS_261_MS.md"),
    ("VERDICTS_W28.md",
     "RESULT_2026-09-06_W28_THE_READ_OUT_SURVIVES_WHAT_THE_SUBSTRATE_CANNOT.md"),
]

#: Wave results this check CANNOT cross-check, each with the reason.
#:
#: `PAIRS` was a curated list of four and nothing noticed the other ten. That is
#: the same defect as a date window that silently stops matching: the closing
#: line said "every published verdict is the one its analyser computed" while
#: covering four of the campaign's fourteen wave results, and the most recent —
#: carrying H16-1, H16-2, H17-1 and H17-2 — was not among them. Its verdicts had
#: been retyped from an analyser run that was never saved anywhere.
#:
#: Every `RESULT_*_W*.md` must now appear in `PAIRS` or here. An entry naming a
#: document that no longer exists fails, and a document in neither fails, so the
#: list cannot rot into a way of dropping a wave quietly.
NO_VERDICTS = {
    "RESULT_2026-08-19_W1_ATTENTION_AT_CONVERGENCE.md": "no per-wave analyser",
    "RESULT_2026-08-20_W3_SCOPE_LIMITS.md": "no per-wave analyser",
    "RESULT_2026-08-20_W4_RECURRENT_ARM_IS_UNUSABLE.md": "no per-wave analyser",
    "RESULT_2026-08-20_W5_BUDGET_LADDER_INCONCLUSIVE.md": "no per-wave analyser",
    "RESULT_2026-08-20_W6_ATTENTION_IS_SAMPLE_EFFICIENCY.md": "no per-wave analyser",
    "RESULT_2026-08-20_W7_CONVERGENCE_IS_BRACKETED.md": "no per-wave analyser",
    "RESULT_2026-08-21_W9_THE_MECHANISM_HOLDS_AT_THE_HEADLINE.md":
        "no per-wave analyser",
    "RESULT_2026-08-22_W10_RESOLUTION_LADDER.md":
        "analyse_wave10.py writes no VERDICTS file — it has no --out",
    "RESULT_2026-08-22_W11_CLIPPING_WAS_NOT_THE_WHOLE_CAUSE.md":
        "analyse_wave11.py writes no VERDICTS file — it has no --out",
}

#: Written longest-first by convention, and that convention is NOT what makes it
#: safe. Every use below anchors the verdict between `**` on both sides, or
#: against a `**` that follows it, so `MET` cannot match the tail of `NOT MET`:
#: the engine tries `MET` at the position after the separator, fails on `N`, and
#: backtracks to the longer alternative. Reordering this string to put `MET`
#: first was tried on 2026-08-28 and changed no verdict — recorded because a
#: comment claiming the order is load-bearing would be a false reason for a true
#: line, and the next person would preserve the wrong thing.
VERDICT = "NOT SUPPORTED|NOT EVALUABLE|SUPPORTED|NOT MET|MET"
#: The analysers write `**X-1** … -> **VERDICT**`.
# The middle group must not cross another hypothesis label. With a bare `(.*?)`
# under re.S, a hypothesis the analyser reports *without* a verdict — "S-3b
# (reported, no threshold)" — ran on until it found the next hypothesis's
# `-> **VERDICT**` and took it as its own. That both invented a verdict for a
# descriptive hypothesis and made the write-up look like it had failed to
# restate one.
GENERATED = re.compile(
    rf"\*\*([A-Z]-\d+[a-z]?)\*\*((?:(?!\*\*[A-Z]-\d).)*?)(?:->|→)\s*\*\*({VERDICT})\*\*",
    re.S,
)
#: The wave-15 analyser and everything frozen after it write the verdict inside
#: one bold span: `**H15-1: NOT MET** (bar: ...)`. The arrow form above does not
#: match it, and the `if not generated` guard caught that rather than reporting
#: a clean pass over nothing — which is the only reason this second pattern
#: exists rather than a silently empty comparison.
GENERATED_INLINE = re.compile(
    rf"\*\*([A-Z]\d*-\d+[a-z]?):\s*({VERDICT})\*\*")
#: The wave-26, -27 and -28 analysers print for a terminal, not for markdown:
#: `H28-1  dropout becomes a manipulation the substrate can feel: MET`, with no
#: emphasis anywhere. Neither pattern above matches a single character of that,
#: so the `if not generated` guard was the only thing standing between the
#: campaign's three newest waves and a check that read nothing — and it could
#: not fire, because they were never in PAIRS at all.
#:
#: The verdict must run to end of line, and no colon may precede it on that
#: line. That is what keeps `H26-1c (secondary, reported not required): entropy
#: at e400/d32l4 = 0.002..., threshold 0.3 -> below` — a real line, with a real
#: colon, about a hypothesis carrying no verdict — from matching.
GENERATED_PLAIN = re.compile(
    rf"^[ \t]*([A-Z]\d*-\d+[a-z]?)\b[^\n:]*:[ \t]*({VERDICT})[ \t]*$", re.M)
#: A write-up states it either as a heading, `**X-1 — VERDICT`, or in a table.
HAND_PROSE = re.compile(rf"\*\*([A-Z]\d*-\d+[a-z]?)\s*[—–-]+\s*({VERDICT})")
#: A write-up's verdict table. The column count is not fixed: waves 8 and 12-14
#: use four columns and the wave-15/17 table uses three, and a pattern pinned to
#: one shape reported "discussed but unparsable" for every verdict in the other
#: — loudly, which is the only reason it was found rather than passed over.
HAND_TABLE = re.compile(
    rf"\*\*([A-Z]\d*-\d+[a-z]?)\*\*(?:[^\n|]*\|){{1,4}}[^\n]*?\*\*({VERDICT})\*\*")
#: `**H28-1: MET.**` — waves 26 and 28. `HAND_PROSE` demands a dash and
#: `GENERATED_INLINE`'s identical-looking pattern demands `**` immediately after
#: the verdict, so the sentence-ending period defeats both. The trailing `\.?`
#: is the whole difference. `\*\*` still closes the match, so a verdict token
#: cannot be matched as the prefix of a longer word.
HAND_COLON = re.compile(
    rf"\*\*([A-Z]\d*-\d+[a-z]?):\s*({VERDICT})\.?\*\*")
#: Wave 27 puts the hypothesis in the section heading and opens the section
#: body with a bare `**NOT MET.**`. Nothing joins the two but position, so the
#: verdict is read from the heading's own section and no further: the scan stops
#: at the next `##`, and a heading naming two hypotheses (`## 5. H27-5 and
#: H27-6 — questions and thresholds, carrying no verdict`) is skipped rather
#: than having one section's verdict attributed to both of them.
HAND_HEADING = re.compile(r"^##[^\n]*$", re.M)
HAND_HEADING_ID = re.compile(r"\b([A-Z]\d*-\d+[a-z]?)\b")
HAND_HEADING_VERDICT = re.compile(rf"\*\*({VERDICT})\.?\*\*")


def heading_scoped(text: str) -> dict[str, str]:
    """`{hypothesis: verdict}` for headings naming exactly one hypothesis."""
    found: dict[str, str] = {}
    headings = list(HAND_HEADING.finditer(text))
    for i, heading in enumerate(headings):
        ids = HAND_HEADING_ID.findall(heading.group())
        if len(ids) != 1:
            continue
        end = headings[i + 1].start() if i + 1 < len(headings) else len(text)
        verdict = HAND_HEADING_VERDICT.search(text, heading.end(), end)
        if verdict:
            found[ids[0]] = verdict.group(1)
    return found


def main() -> int:
    compared = 0
    problems: list[str] = []

    # Coverage first. A curated list that quietly stops covering the newest
    # wave is worse than no list, because the closing line keeps claiming
    # everything.
    checked = {hand for _, hand in PAIRS}
    for path in sorted((ROOT / "results").glob("RESULT_*_W[0-9]*.md")):
        if path.name not in checked and path.name not in NO_VERDICTS:
            problems.append(
                f"{path.name}: neither cross-checked nor declared in "
                f"NO_VERDICTS. Every wave result is one or the other.")
    for name, why in sorted(NO_VERDICTS.items()):
        if not (ROOT / "results" / name).is_file():
            problems.append(f"NO_VERDICTS names {name}, which does not exist")
        elif name in checked:
            problems.append(f"{name} is both cross-checked and declared "
                            f"uncheckable ({why}); remove the declaration")
    print(f"  {len(checked)} wave result(s) cross-checked, "
          f"{len(NO_VERDICTS)} declared uncheckable")

    for generated_name, hand_name in PAIRS:
        generated_path = resolve(generated_name)
        hand_path = ROOT / "results" / hand_name
        if generated_path is None:
            problems.append(
                f"{generated_name}: not found in exactly one of "
                f"{', '.join(c.name for c in CAMPAIGNS)}")
            continue
        # Scoped to this pair. Testing the global `problems` list meant one
        # problem in the first document silently skipped every document after
        # it, so a single missing file could reduce the check to one comparison.
        pair_problems = [f"missing {p.relative_to(ROOT)}"
                         for p in (generated_path, hand_path) if not p.is_file()]
        if pair_problems:
            problems.extend(pair_problems)
            continue

        generated_text = generated_path.read_text()
        # An analyser that states one hypothesis twice — once in its section and
        # again in a closing summary — must state it the same way both times.
        # `dict()` would keep the last silently, which is how a summary line
        # could contradict the section above it and publish the contradiction.
        generated: dict[str, str] = {}
        for key, verdict in ([(k, v) for k, _, v in GENERATED.findall(generated_text)]
                             + GENERATED_INLINE.findall(generated_text)
                             + GENERATED_PLAIN.findall(generated_text)):
            if generated.setdefault(key, verdict) != verdict:
                problems.append(
                    f"{generated_name}: {key} appears twice with different "
                    f"verdicts ({generated[key]!r} and {verdict!r}). The "
                    f"analyser contradicts itself; nothing downstream can be "
                    f"cross-checked against it.")
        hand_text = hand_path.read_text()
        hand = (dict(HAND_PROSE.findall(hand_text))
                | dict(HAND_TABLE.findall(hand_text))
                | dict(HAND_COLON.findall(hand_text))
                | heading_scoped(hand_text))

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
        # A hypothesis the analyser ruled on, which the write-up *discusses* but
        # whose verdict could not be parsed, is not a hypothesis stated as a
        # trend — it is a verdict this check failed to read. Excusing both alike
        # meant a verdict rewritten in unrecognised punctuation escaped
        # comparison entirely, including one rewritten to its opposite: with
        # `**R-1 — SUPPORTED**` respelled `R-1: NOT SUPPORTED`, this printed
        # "all agree" and exited 0.
        unparsed = [k for k in missing if re.search(rf"\b{re.escape(k)}\b", hand_text)]
        silent = [k for k in missing if k not in unparsed]
        for hypothesis in unparsed:
            problems.append(
                f"{hand_name}: {hypothesis} — the analyser issued "
                f"{generated[hypothesis]!r} and the write-up discusses "
                f"{hypothesis}, but no verdict there could be parsed. Either the "
                "verdict is spelled in a form this check does not recognise, or "
                "it was dropped; both need a human."
            )
        if unparsed:
            print(f"         [FAIL] discussed but unparsable: {', '.join(unparsed)}")
        if silent:
            # Not a failure: a hypothesis registered as descriptive has no
            # verdict, and the write-up does not mention it at all.
            print(f"         not restated as a verdict (descriptive, or stated as "
                  f"a trend): {', '.join(silent)}")

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
