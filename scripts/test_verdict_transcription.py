"""Tests for `scripts/check_verdicts_transcribed.py`.

That check compares a hand-written wave result against the verdicts its frozen
analyser computed. It was well built and it was covering **four** of the
campaign's fourteen wave results, because `PAIRS` is a curated list and nothing
noticed the other ten — while its closing line said "every published verdict is
the one its analyser computed".

The most recent result, carrying H16-1, H16-2, H17-1 and H17-2, was among the
uncovered: its verdicts had been retyped from an analyser run that was never
saved anywhere, so nothing could have compared them. They turned out correct.
Nothing had established that.

Run: python3 scripts/test_verdict_transcription.py
"""

from __future__ import annotations

import importlib.util
import re
import subprocess
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
_spec = importlib.util.spec_from_file_location(
    "check_verdicts_transcribed", ROOT / "scripts/check_verdicts_transcribed.py")
CV = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(CV)


class EveryWaveResultIsAccountedForTest(unittest.TestCase):
    """The rule that replaced the curated list: in `PAIRS`, or declared."""

    def setUp(self):
        self.checked = {hand for _, hand in CV.PAIRS}
        self.results = {p.name for p in
                        (ROOT / "results").glob("RESULT_*_W[0-9]*.md")}

    def test_no_wave_result_is_silently_uncovered(self):
        for name in sorted(self.results):
            with self.subTest(name=name):
                self.assertTrue(name in self.checked or name in CV.NO_VERDICTS,
                                f"{name} is neither cross-checked nor declared")

    def test_no_declaration_is_stale(self):
        for name in sorted(CV.NO_VERDICTS):
            with self.subTest(name=name):
                self.assertIn(name, self.results,
                              f"NO_VERDICTS names {name}, which is not a wave "
                              f"result on disk")

    def test_the_two_lists_are_disjoint(self):
        self.assertEqual(self.checked & set(CV.NO_VERDICTS), set())

    def test_every_declaration_carries_a_reason(self):
        for name, why in CV.NO_VERDICTS.items():
            with self.subTest(name=name):
                self.assertTrue(why.strip(), f"{name} is declared with no reason")

    def test_coverage_has_not_shrunk(self):
        """A floor, so a pair removed to make the check pass is caught. Five
        results were covered on 2026-08-28, up from four."""
        self.assertGreaterEqual(len(CV.PAIRS), 5)


class VerdictParsingTest(unittest.TestCase):
    """Both analyser output shapes and both write-up table shapes."""

    def test_the_arrow_form_is_parsed(self):
        text = "**S-1** something -> **SUPPORTED**"
        self.assertEqual(
            {k: v for k, _, v in CV.GENERATED.findall(text)}, {"S-1": "SUPPORTED"})

    def test_the_inline_form_is_parsed(self):
        """`**H15-1: NOT MET**`, which every analyser frozen after wave 15
        writes. The arrow pattern does not match it."""
        text = "**H15-1: NOT MET** (bar: gain >= +0.05)."
        self.assertEqual(dict(CV.GENERATED_INLINE.findall(text)),
                         {"H15-1": "NOT MET"})

    def test_a_three_column_table_is_parsed(self):
        """The wave-15/17 write-up. A pattern pinned to four columns reported
        every one of its verdicts as discussed-but-unparsable."""
        row = "| **H16-2** | The collapse is a threshold | **MET** |"
        self.assertEqual(dict(CV.HAND_TABLE.findall(row)), {"H16-2": "MET"})

    def test_a_four_column_table_is_parsed(self):
        row = "| **S-1** | a claim | 12/12 | **SUPPORTED** |"
        self.assertEqual(dict(CV.HAND_TABLE.findall(row)), {"S-1": "SUPPORTED"})

    def test_not_met_is_never_read_as_met(self):
        """The failure that would turn a refutation into a pass. The `**`
        anchors prevent it, not the alternation order."""
        for text in ("**H15-1: NOT MET**", "| **H15-1** | x | **NOT MET** |",
                     "**H15-1 — NOT MET**"):
            with self.subTest(text=text):
                found = (dict(CV.GENERATED_INLINE.findall(text))
                         | dict(CV.HAND_TABLE.findall(text))
                         | dict(CV.HAND_PROSE.findall(text)))
                self.assertEqual(set(found.values()), {"NOT MET"}, found)

    def test_wave_numbered_ids_are_recognised(self):
        """`H15-1`, not just `S-1`. Before this the newer waves' ids did not
        match and every verdict in them went uncompared."""
        self.assertIn("H15-1", dict(CV.GENERATED_INLINE.findall("**H15-1: MET**")))


class TheCheckRunsCleanTest(unittest.TestCase):

    @classmethod
    def setUpClass(cls):
        cls.proc = subprocess.run(
            [sys.executable, str(ROOT / "scripts/check_verdicts_transcribed.py")],
            capture_output=True, text=True)

    def test_it_passes_on_the_committed_record(self):
        self.assertEqual(self.proc.returncode, 0, self.proc.stdout[-2000:])

    def test_it_reports_how_much_it_covered(self):
        self.assertRegex(self.proc.stdout,
                         r"\d+ wave result\(s\) cross-checked, \d+ declared")

    def test_the_wave_15_result_is_now_compared(self):
        self.assertRegex(self.proc.stdout,
                         r"W15_17_THE_COLLAPSE_IS_A_THRESHOLD\.md\s+7 compared")


if __name__ == "__main__":
    unittest.main(verbosity=2)
