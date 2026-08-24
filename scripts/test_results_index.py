"""Tests for the record index, on the strings that actually broke it.

The retirement classifier was wrong in **both** directions on its first run
against the real corpus, which is why it has a test at all:

  * It listed `SUMMARY_2026-08-22_CAMPAIGN_AND_RECORD_REPAIR.md` — a live
    document — as retired, because line 5 of that file happens to begin
    `withdrawn, what was hardened, and what is still open.` A sentence that
    starts with the word is not a banner.
  * It missed `SUMMARY_2026-08-20_ATTENTION_CAMPAIGN.md`, which *is* superseded
    and says so as `> **SUPERSEDED 2026-08-22 by ...**`. The pattern allowed a
    blockquote and a heading but not bold.

Both directions matter and they fail differently. A false positive buries a live
document under a warning nobody should heed; a false negative leaves a withdrawn
claim looking current in the one place built to show otherwise.

Run: python3 scripts/test_results_index.py
"""

from __future__ import annotations

import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "scripts"))

import build_results_index as index  # noqa: E402


def describe(text: str) -> dict:
    with tempfile.TemporaryDirectory() as tmp:
        path = Path(tmp) / "RESULT_2026-01-01_PROBE.md"
        path.write_text(text)
        return index.describe(path)


class RetirementBannerTest(unittest.TestCase):
    #: Verbatim from the documents these were taken from.
    BANNERS = [
        ("blockquote heading",
         "# T\n\n> ## WITHDRAWN 2026-08-22 — this result measured a protocol parameter.\n"),
        ("blockquote bold",
         "# T\n\n> **SUPERSEDED 2026-08-22 by\n> [`OTHER.md`](OTHER.md).**\n"),
        ("bold with IN PART",
         "# T\n\n> ## SUPERSEDED IN PART, 2026-08-03 / 2026-08-04 — read this first\n"),
        ("bold with a dash",
         "# T\n\n**WITHDRAWN — the arm was never measured.**\n"),
    ]

    NOT_BANNERS = [
        ("a sentence that begins with the word",
         "# T\n\nSupersedes [`OTHER.md`](OTHER.md), which covered waves 1-7 only.\n"
         "This is the whole of it: what was measured, what was\n"
         "withdrawn, what was hardened, and what is still open.\n"),
        ("a document that withdraws something else",
         "# T\n\n**Withdraws:** `OTHER.md`\n\nIts title claim is false.\n"),
        ("the word inside a sentence",
         "# T\n\nThe claim was withdrawn on 2026-08-22 by a later measurement.\n"),
        ("a banner far below the header block",
         "# T\n\n" + "filler\n" * 20 + "> ## WITHDRAWN 2026-08-22 — too late to count\n"),
    ]

    def test_every_real_banner_is_recognised(self):
        for label, text in self.BANNERS:
            with self.subTest(label=label):
                self.assertTrue(describe(text)["retired"], f"missed a banner: {label}")

    def test_nothing_else_is(self):
        for label, text in self.NOT_BANNERS:
            with self.subTest(label=label):
                self.assertFalse(describe(text)["retired"], f"false positive: {label}")

    def test_the_reason_is_carried_not_just_the_flag(self):
        """An index that says 'retired' without saying why sends the reader hunting."""
        doc = describe("# T\n\n> ## WITHDRAWN 2026-08-22 — it measured a protocol parameter.\n")
        self.assertIn("protocol parameter", doc["why"])


class CorpusTest(unittest.TestCase):
    """The two documents that produced the original errors, in the real corpus."""

    def doc(self, name: str) -> dict:
        return index.describe(ROOT / "results" / name)

    def test_the_live_summary_is_not_marked_retired(self):
        self.assertFalse(
            self.doc("SUMMARY_2026-08-22_CAMPAIGN_AND_RECORD_REPAIR.md")["retired"],
            "the current campaign summary is being reported as retired",
        )

    def test_the_superseded_summary_is(self):
        doc = self.doc("SUMMARY_2026-08-20_ATTENTION_CAMPAIGN.md")
        self.assertTrue(doc["retired"], "a superseded summary is being reported as live")

    def test_the_withdrawn_wave_4_result_is(self):
        self.assertTrue(self.doc("RESULT_2026-08-20_W4_RECURRENT_ARM_IS_UNUSABLE.md")["retired"])

    def test_generated_reports_are_separated_from_authored_records(self):
        """A generated report carries no registration and is not a claim on its own."""
        self.assertEqual(index.kind_of("deep_snn_results_v134.md"), index.GENERATED)
        self.assertEqual(index.kind_of("RESULT_2026-08-23_W14_X.md"), "Results")
        self.assertEqual(index.kind_of("PREREG_2026-08-23_X.md"), "Preregistrations")

    def test_the_index_on_disk_is_current(self):
        """A stale index is worse than none: it reports a state that has moved."""
        self.assertTrue(index.INDEX.is_file(), "results/INDEX.md has not been generated")
        self.assertEqual(
            index.INDEX.read_text(), index.build(),
            "results/INDEX.md is out of date; run scripts/build_results_index.py",
        )


if __name__ == "__main__":
    unittest.main(verbosity=2)
