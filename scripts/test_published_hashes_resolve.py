#!/usr/bin/env python3
"""A published reproduction command must name a hash the binary can resolve.

# Why this exists

On 2026-08-25 the matched-architecture configs mixed `MATCHED_INPUT_SCALE` and
the forward graph into their hashes, because the old ones did not: the silent-
initialisation repair moved the input scale 0.5 -> 2.0 while the constant was
outside the hash, so one label named two different experiments either side of
the repair. Retiring them was the correct outcome and
`RESULT_2026-08-25_MATCHED_ARCH_RERUN.md` says so — `from_hash` returns `None`
for all four, and being told "unknown hash" is better than being handed
different numbers under the name you asked for.

What nothing checked is where those four labels still appear. On 2026-08-29
they were still the three `--config-hash` arguments in section A of
`REPRO_ARTIFACT_CHECKLIST.md`, which is the document a reviewer runs, and the
fourth was in section E. **Three of the package's headline reproduction
commands could not run**, and the checklist ticked them.

They were also still the primary values in `PAPER_METRICS_FULL.md` Table A and
in every row of `PAPER_VERIFY.md`, with no banner on either — the superseded
pre-repair block, published as current, in two documents the number sweep
deliberately does not read because they are the paper's own downstream
artefacts (`check_every_number.py`, `PAPER_SIDE`).

# What this asserts

1. The retired/current pairs written in the Rust freeze comments agree with the
   hash each freeze test actually asserts. A comment that drifts from the code
   it documents is how the next one of these starts.
2. No `--config-hash` argument anywhere in the record names a retired hash.
   A command that cannot run must not be published as a reproduction step.
3. Every appearance of a retired hash in a paper-side document is in a
   paragraph that marks it retired. Citing one as a live result is the defect;
   recording that it existed and was withdrawn is the point of keeping it.

   The unit is the paragraph and not the line, because markdown prose wraps: a
   line-scoped version of this test reported `PAPER_SKELETON.md` §9.2, which
   names the three hashes on one line and calls them "deliberately retired" on
   the next. A check that pushes authors to cram a marker word onto whichever
   line a hash happened to land on is measuring the wrapping.

Run: python3 scripts/test_published_hashes_resolve.py
"""

from __future__ import annotations

import re
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT))

from scripts.check_every_number import PAPER_SIDE  # noqa: E402

#: The four suites whose hashes moved on 2026-08-25. Named rather than globbed:
#: there are four, `RESULT_2026-08-25_MATCHED_ARCH_RERUN.md` §8 lists exactly
#: four, and a glob that silently found three would pass every test below.
CONFIGS = (
    "match_config.rs",
    "dfa_match_config.rs",
    "rl_match_config.rs",
    "eventprop_match_config.rs",
)

#: Words that mark a hash as no longer live. Any one of them on the same line
#: as the hash discharges it. `pre-repair` is here because that is the phrase
#: `PUBLISHABLE_CLAIMS.md` already uses for exactly this.
DISCHARGED = (
    "retired", "archived", "superseded", "withdrawn", "pre-repair",
    "not citable", "no longer resolve", "does not resolve", "stale",
)

#: Documents that record the retirement itself, and so quote the hashes as
#: their subject rather than citing them. Each is named with its reason; a new
#: one has to be added here deliberately.
RETIREMENT_RECORDS = {
    # §8 is the table that retired them, and its rows are `retired | current`
    # with the words in the header rather than on every line.
    "RESULT_2026-08-25_MATCHED_ARCH_RERUN.md": "the document that retired them",
    "PREREG_2026-08-25_MATCHED_ARCH_RERUN_ON_BOTH_FORWARDS.md":
        "the preregistration that predicted the break",
}


def freeze_blocks() -> dict[str, tuple[str, str, str]]:
    """`config file -> (retired hash, current hash from the comment, frozen)`."""
    out = {}
    for name in CONFIGS:
        text = (ROOT / "binn-lab/src" / name).read_text()
        retired = re.search(r"//\s+retired:\s+(c1-[\w-]+)", text)
        current = re.search(r"//\s+current:\s+(c1-[\w-]+)", text)
        frozen = re.search(r'assert_eq!\(hash,\s*"(c1-[\w-]+)"\)', text)
        assert retired and current and frozen, name
        out[name] = (retired.group(1), current.group(1), frozen.group(1))
    return out


def retired_hashes() -> set[str]:
    return {r for r, _, _ in freeze_blocks().values()}


def record_documents() -> list[Path]:
    """Every markdown file in `results/`, at any depth."""
    return sorted((ROOT / "results").rglob("*.md"))


class FreezeCommentsTest(unittest.TestCase):
    """The comment and the assertion beside it must say the same thing."""

    def test_all_four_suites_were_parsed(self):
        """A parse that found three would leave one retired hash unbanned."""
        self.assertEqual(len(freeze_blocks()), 4)
        self.assertEqual(len(retired_hashes()), 4)

    def test_the_comment_names_the_hash_the_test_freezes(self):
        for name, (_, current, frozen) in freeze_blocks().items():
            with self.subTest(config=name):
                self.assertEqual(
                    current, frozen,
                    f"{name}: the freeze comment says the current hash is "
                    f"{current} but the test asserts {frozen}")

    def test_no_retired_hash_is_also_a_current_one(self):
        current = {c for _, c, _ in freeze_blocks().values()}
        self.assertEqual(retired_hashes() & current, set())


class ReproductionCommandsTest(unittest.TestCase):
    """`--config-hash` names a hash `from_hash` must still resolve."""

    @classmethod
    def setUpClass(cls):
        cls.retired = retired_hashes()

    def test_the_scan_finds_config_hash_arguments(self):
        """A regex matching nothing would pass the test below forever."""
        found = [h for doc in record_documents()
                 for h in re.findall(r"--config-hash\s+(c1[\w-]+)", doc.read_text())]
        self.assertGreater(len(found), 5, found)

    def test_no_published_command_names_a_retired_hash(self):
        offences = []
        for doc in record_documents():
            # A run record is a log of what ran, under the hash it ran under.
            # Rewriting one would be falsifying the record; they are history.
            if "runs/" in doc.as_posix():
                continue
            for hash_ in re.findall(r"--config-hash\s+(c1[\w-]+)", doc.read_text()):
                if hash_ in self.retired:
                    offences.append(f"{doc.relative_to(ROOT)}: {hash_}")
        self.assertEqual(offences, [], "\n".join(
            ["a published reproduction command names a hash `from_hash` no "
             "longer resolves; the command cannot run:"] + offences))


class PaperSideCitationsTest(unittest.TestCase):
    """A retired hash in a paper document must be marked as retired."""

    @classmethod
    def setUpClass(cls):
        cls.retired = retired_hashes()

    def test_the_paper_side_list_is_the_one_the_sweep_uses(self):
        """One owner. If `check_every_number.py` narrows its list, this narrows
        with it rather than keeping a second copy that quietly disagrees."""
        self.assertIn("PAPER_METRICS_FULL.md", PAPER_SIDE)
        self.assertIn("PAPER_VERIFY.md", PAPER_SIDE)
        self.assertGreaterEqual(len(PAPER_SIDE), 10)

    def test_the_scan_finds_retired_hashes_to_check(self):
        """Negative control: if no paper-side document mentioned one, the test
        below would pass while checking nothing."""
        hits = sum(
            doc.read_text().count(h)
            for doc in record_documents() if doc.name in PAPER_SIDE
            for h in self.retired
        )
        self.assertGreater(hits, 0, "no retired hash appears in any paper-side "
                                    "document; the check below is vacuous")

    def test_every_retired_hash_is_marked_where_a_paper_cites_it(self):
        offences = []
        for doc in record_documents():
            if doc.name not in PAPER_SIDE or doc.name in RETIREMENT_RECORDS:
                continue
            line_no = 1
            for para in doc.read_text().split("\n\n"):
                lowered = para.lower()
                for hash_ in sorted(self.retired):
                    if hash_ in para and not any(w in lowered for w in DISCHARGED):
                        offences.append(f"{doc.relative_to(ROOT)}:{line_no}: {hash_}")
                line_no += para.count("\n") + 2
        self.assertEqual(offences, [], "\n".join(
            ["a paper-side document cites a retired config hash without saying "
             "it is retired:"] + offences))


if __name__ == "__main__":
    unittest.main(verbosity=2)
