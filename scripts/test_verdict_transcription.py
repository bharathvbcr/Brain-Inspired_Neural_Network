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
        results were covered on 2026-08-28, up from four; fifteen on
        2026-09-07, when waves 26-28 stopped being uncovered."""
        self.assertGreaterEqual(len(CV.PAIRS), 15)


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

    def test_the_plain_terminal_form_is_parsed(self):
        """Waves 26-28 print for a terminal. Neither markdown pattern matches a
        character of it, so before `GENERATED_PLAIN` those three analysers were
        unreadable by this check whatever `PAIRS` said."""
        text = ("H28-1  dropout becomes a manipulation the substrate can feel: MET\n"
                "  the instrument is sensitive from p70 upward.\n")
        self.assertEqual(dict(CV.GENERATED_PLAIN.findall(text)), {"H28-1": "MET"})
        self.assertEqual(dict(CV.GENERATED.findall(text)), {})
        self.assertEqual(dict(CV.GENERATED_INLINE.findall(text)), {})

    def test_the_plain_form_ignores_a_line_that_states_no_verdict(self):
        """A real wave-26 line: a colon, a hypothesis id, and no verdict. It
        must not be read as one, and the verdict must run to end of line."""
        for text in (
            "  H26-1c (secondary, reported not required): entropy at e400 = 0.002",
            "  H27-3, H27-5 and H27-6 carry no MET/NOT MET.",
            "H28-2  the bar: the read-out's advantage: NOT MET but see below",
        ):
            with self.subTest(text=text):
                self.assertEqual(dict(CV.GENERATED_PLAIN.findall(text)), {})

    def test_the_plain_form_never_reads_not_met_as_met(self):
        text = "H28-2  the read-out's advantage is specific to order: NOT MET"
        self.assertEqual(dict(CV.GENERATED_PLAIN.findall(text)), {"H28-2": "NOT MET"})

    def test_the_bold_colon_form_is_parsed(self):
        """`**H28-1: MET.**` — waves 26 and 28. The sentence-ending period
        defeats `GENERATED_INLINE`, and the missing dash defeats `HAND_PROSE`."""
        text = "**H26-1a: NOT MET.** The magnitude clause passed."
        self.assertEqual(dict(CV.HAND_COLON.findall(text)), {"H26-1a": "NOT MET"})
        self.assertEqual(dict(CV.GENERATED_INLINE.findall(text)), {})
        self.assertEqual(dict(CV.HAND_PROSE.findall(text)), {})

    def test_a_heading_scoped_verdict_is_read_from_its_own_section(self):
        """Wave 27's shape: the id in the heading, a bare `**NOT MET.**` in the
        body. The second section's verdict must not leak into the first."""
        text = ("## 1. H27-1 — the campaign's only exact zero\n\n"
                "**MET.** All 12 of 12 seed pairs are byte-identical.\n\n"
                "## 3. H27-2 — the dropout instrument does not work\n\n"
                "**NOT MET.** At `p30` the rate arm loses 0.0124.\n")
        self.assertEqual(CV.heading_scoped(text),
                         {"H27-1": "MET", "H27-2": "NOT MET"})

    def test_a_heading_naming_two_hypotheses_is_skipped(self):
        """`## 5. H27-5 and H27-6 — questions and thresholds, carrying no
        verdict`. Attributing one section's verdict to both would invent one."""
        text = ("## 5. H27-5 and H27-6 — carrying no verdict\n\n"
                "**NOT MET.** this belongs to neither of them\n")
        self.assertEqual(CV.heading_scoped(text), {})

    def test_a_heading_with_no_verdict_in_its_section_yields_nothing(self):
        text = "## 5. H27-5 — a threshold\n\nThe ladder ran clean, no rung voided.\n"
        self.assertEqual(CV.heading_scoped(text), {})


class VerdictFilesAreResolvedAcrossCorporaTest(unittest.TestCase):
    """Waves 26-28 landed in a third campaign directory beside the second."""

    def test_every_pair_resolves_to_exactly_one_file(self):
        for generated, hand in CV.PAIRS:
            with self.subTest(generated=generated):
                self.assertIsNotNone(CV.resolve(generated),
                                     f"{generated} is in neither corpus, or in both")

    def test_a_name_in_no_corpus_does_not_resolve(self):
        self.assertIsNone(CV.resolve("VERDICTS_W999.md"))

    def test_both_corpora_are_searched(self):
        """A single hardcoded directory could not see waves 26-28 at all."""
        found = {CV.resolve(g).parent.name for g, _ in CV.PAIRS}
        self.assertEqual(found, {c.name for c in CV.CAMPAIGNS}, found)


class TheNewestVerdictsAreTheAnalysersOwnBytesTest(unittest.TestCase):
    """Waves 8-25 wrote their `VERDICTS_*.md` by redirecting an analyser's
    stdout and nothing re-derives them: the file is trusted because of how it
    was made, once, by a person who is no longer in the room.

    For waves 26-28 the redirect is checked instead of remembered. If a
    committed verdict file ever stops being what its frozen analyser prints —
    edited by hand, or left behind when cells land — this fails, and the
    cross-check above stops resting on an unverified intermediate.
    """

    # `(verdict file, analyser, extra arguments)`. Wave 26's analyser still
    # defaults `--results` to the v2 corpus, which holds none of its cells; it
    # exits 2 with "NOTHING TO ANALYSE" rather than emitting empty verdicts, so
    # the stale default is safe and is left alone rather than edited in a frozen
    # analyser. The correct invocation is pinned here.
    V3 = ROOT / "results/shd_attention_campaign_v3"
    REGENERATED = [
        ("VERDICTS_W26.md", "analyse_wave26.py",
         ["--results", str(V3), "--probes", str(V3 / "probes")]),
        ("VERDICTS_W27.md", "analyse_wave27.py", ["--results", str(V3)]),
        ("VERDICTS_W28.md", "analyse_wave28.py", ["--results", str(V3)]),
    ]

    def test_each_file_is_byte_identical_to_its_analysers_stdout(self):
        for name, analyser, extra in self.REGENERATED:
            with self.subTest(name=name):
                proc = subprocess.run(
                    [sys.executable, str(ROOT / "scripts/aws" / analyser)] + extra,
                    capture_output=True, text=True)
                self.assertEqual(proc.returncode, 0, proc.stderr[-2000:])
                self.assertEqual(proc.stdout, (self.V3 / name).read_text(),
                                 f"{name} is not what {analyser} prints today")

    def test_each_regenerated_file_carries_a_verdict(self):
        """A comparison against an empty file passes vacuously."""
        for name, _, _ in self.REGENERATED:
            with self.subTest(name=name):
                text = (self.V3 / name).read_text()
                self.assertTrue(dict(CV.GENERATED_PLAIN.findall(text)), name)


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

    def test_the_three_newest_waves_are_compared(self):
        """They were "neither cross-checked nor declared" until 2026-09-07 —
        the campaign's three most recent waves, carrying the paper's live
        mechanism claims, checked by nothing."""
        for pattern in (r"W26_SATURATION_IS_REAL_AND_NOT_SPECIFIC\.md\s+3 compared",
                        r"W27_THE_TIMESCALE_IS_261_MS\.md\s+3 compared",
                        r"W28_THE_READ_OUT_SURVIVES[\w_]*\s+2 compared"):
            with self.subTest(pattern=pattern):
                self.assertRegex(self.proc.stdout, pattern)


if __name__ == "__main__":
    unittest.main(verbosity=2)
