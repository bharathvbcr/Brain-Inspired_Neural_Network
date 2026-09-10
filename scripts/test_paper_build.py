#!/usr/bin/env python3
"""The paper build refuses each thing it claims to refuse.

`build_paper.py` is the only route from the manuscript to a submittable PDF,
and four of its five guards protect against failures that are INVISIBLE in the
output: a tweaked style file looks like a paper, a section dropped from the
split manifest looks like a paper, a de-anonymised PDF looks like a paper, and
a ten-page paper looks like a paper. Only the compile failure announces itself.
So each guard is exercised here against a corpus in which exactly one thing is
wrong.
"""
from __future__ import annotations

import importlib.util
import pathlib
import re
from unittest import mock
import shutil
import subprocess
import sys
import tempfile
import unittest

ROOT = pathlib.Path(__file__).resolve().parent.parent
BUILDER = ROOT / "scripts/build_paper.py"

spec = importlib.util.spec_from_file_location("build_paper", BUILDER)
bp = importlib.util.module_from_spec(spec)
spec.loader.exec_module(bp)


class PureGuardTest(unittest.TestCase):
    """Guards that need no LaTeX run."""

    def test_the_style_hash_matches_the_file_on_disk(self):
        """If this fails, either the style file was replaced or the pin is
        stale. Both are things a human decides, and neither may be papered
        over by the build recomputing the hash it is checking."""
        import hashlib
        got = hashlib.sha256((ROOT / "paper/neurips_2026.sty").read_bytes()).hexdigest()
        self.assertEqual(got, bp.STY_SHA256)

    def test_the_pinned_hash_is_the_one_the_provenance_note_records(self):
        note = (ROOT / "paper/STYLE_PROVENANCE.md").read_text()
        self.assertIn(bp.STY_SHA256, note,
                      "the pin and the note that justifies it have drifted")

    def test_an_undeclared_character_is_named_not_left_to_pdflatex(self):
        with self.assertRaises(SystemExit):
            bp.check_unicode("the Hebbian rule ∂w/∂t is fine\n")

    def test_every_declared_character_is_actually_in_the_preamble(self):
        """A character in DECLARED_UNICODE but not in the `\\newunicodechar`
        block would pass the pre-flight scan and then halt pdflatex."""
        for ch in bp.DECLARED_UNICODE:
            self.assertIn(f"\\newunicodechar{{{ch}}}", bp.PREAMBLE,
                          f"{ch!r} (U+{ord(ch):04X}) is declared but unmapped")

    def test_the_draft_needs_no_character_the_build_cannot_typeset(self):
        bp.check_unicode((ROOT / "results/PAPER_DRAFT.md").read_text())

    def test_record_links_become_markers_and_are_registered(self):
        register: dict[str, int] = {}
        out = bp.rewrite_record_links(
            "see [`RESULT_A.md`](RESULT_A.md) and [B](sub/RESULT_B.md) "
            "and [`RESULT_A.md`](RESULT_A.md) again", register)
        self.assertEqual(out, r"see \rec{1} and \rec{2} and \rec{1} again")
        self.assertEqual(register, {"RESULT_A.md": 1, "RESULT_B.md": 2},
                         "a repeated record must reuse its number, and the "
                         "directory prefix must not create a second entry")

    def test_no_record_pointer_is_dropped(self):
        """The markdown's provenance links are the paper's evidence trail.
        Rewriting them compactly is fine; losing one is not."""
        import re
        draft = (ROOT / "results/PAPER_DRAFT.md").read_text()
        register: dict[str, int] = {}
        out = bp.rewrite_record_links(draft, register)
        self.assertEqual(len(re.findall(r"\\rec\{", out)),
                         len(bp.RECORD_LINK.findall(draft)))
        self.assertGreater(len(register), 20)

    def test_every_section_of_the_draft_is_placed(self):
        """The build refuses an unplaced section; this asserts the real
        manifest currently has none, so that refusal is not masking a
        section that has been quietly missing for a while."""
        import tomllib
        manifest = tomllib.loads((ROOT / "paper/split_manifest.toml").read_text())
        placed = set(manifest["main"]["sections"]) | set(manifest["appendix"]["sections"])
        have = {h for h, _ in bp.split_sections((ROOT / "results/PAPER_DRAFT.md").read_text())}
        self.assertEqual(have - placed, set())
        self.assertEqual(placed - have, set())

    def test_the_page_limit_is_the_venue_s(self):
        self.assertEqual(bp.PAGE_LIMIT, 9, "NeurIPS 2026 allows nine content pages")

    def test_every_typeset_caption_is_the_spec_s_required_wording(self):
        """The spec owns caption text. A caption written in the builder would
        be a second copy, and the two would disagree the first time one of
        them was corrected -- which is how this repository shipped a figure
        drawing a superseded value block at camera-ready quality."""
        spec = (ROOT / "results/PAPER_FIGURE_SPEC.md").read_text()
        captions = bp.spec_captions()
        self.assertEqual(len(captions), 7)
        for stem, caption in captions:
            self.assertIn(caption.split(".")[0], " ".join(spec.split()),
                          f"{stem}'s caption is not the spec's")

    def test_every_captioned_figure_is_placed_in_exactly_one_half(self):
        halves = {bp.FIGURE_PLACEMENT[s] for s, _ in bp.spec_captions()}
        self.assertEqual(halves, {"main", "appendix"})
        self.assertEqual(len(bp.FIGURE_PLACEMENT), len(bp.spec_captions()))

    def test_an_unattributed_caption_stops_the_build(self):
        """A caption block with no artwork target would be dropped, and the
        figure would ship uncaptioned -- invisible in a page of figures."""
        original = bp.SPEC
        try:
            tmp = pathlib.Path(tempfile.mkdtemp())
            self.addCleanup(shutil.rmtree, tmp, ignore_errors=True)
            fake = tmp / "spec.md"
            fake.write_text("**Caption (required wording):**\n\u201cA caption.\u201d\n")
            bp.SPEC = fake
            with self.assertRaises(SystemExit):
                bp.spec_captions()
        finally:
            bp.SPEC = original

    def test_the_anonymity_list_covers_the_committer(self):
        """The identity most likely to leak into a build is the person
        running it, via $HOME in a path or a PDF /Author field."""
        needles = {n for n, _ in bp.IDENTIFYING}
        self.assertIn("/Users/", needles)
        self.assertTrue(any("gmail" in n for n in needles))

    def test_the_preamble_blanks_the_pdf_metadata(self):
        for field in ("pdfauthor", "pdftitle", "pdfcreator"):
            self.assertIn(f"{field}={{}}", bp.PREAMBLE)


class BuiltPaperTest(unittest.TestCase):
    """The real artefact on disk."""

    @classmethod
    def setUpClass(cls) -> None:
        cls.pdf = bp.BUILD / "main.pdf"
        if not cls.pdf.exists():
            raise unittest.SkipTest("no build on disk; run scripts/build_paper.py")

    def test_the_committed_build_passes_its_own_check(self):
        proc = subprocess.run([sys.executable, str(BUILDER), "--check"],
                              capture_output=True, text=True)
        self.assertEqual(proc.returncode, 0, proc.stdout + proc.stderr)

    def test_it_is_within_the_page_limit(self):
        content, total = bp.content_pages(self.pdf)
        self.assertLessEqual(content, bp.PAGE_LIMIT)
        self.assertGreater(total, content,
                           "the appendix should follow the content pages")

    def test_it_says_anonymous_author(self):
        first = subprocess.run(
            ["pdftotext", "-f", "1", "-l", "1", str(self.pdf), "-"],
            capture_output=True, text=True).stdout
        self.assertIn("Anonymous Author", first)

    def test_it_carries_no_identity_in_body_or_metadata(self):
        body = subprocess.run(["pdftotext", str(self.pdf), "-"],
                              capture_output=True, text=True).stdout.lower()
        meta = subprocess.run(["pdfinfo", str(self.pdf)],
                              capture_output=True, text=True).stdout.lower()
        for needle, what in bp.IDENTIFYING:
            self.assertNotIn(needle.lower(), body, what)
            self.assertNotIn(needle.lower(), meta, what)

    def test_a_missing_build_is_a_failure_not_a_pass(self):
        tmp = pathlib.Path(tempfile.mkdtemp())
        self.addCleanup(shutil.rmtree, tmp, ignore_errors=True)
        moved = tmp / "main.pdf"
        shutil.move(self.pdf, moved)
        try:
            proc = subprocess.run([sys.executable, str(BUILDER), "--check"],
                                  capture_output=True, text=True)
            self.assertEqual(proc.returncode, 1)
            self.assertIn("must not look alike", proc.stdout + proc.stderr)
        finally:
            shutil.move(moved, self.pdf)

    def test_every_specified_figure_is_typeset_and_captioned(self):
        body = subprocess.run(["pdftotext", str(self.pdf), "-"],
                              capture_output=True, text=True).stdout
        self.assertEqual(body.count("Figure 1:"), 1)
        for n in range(1, len(bp.spec_captions()) + 1):
            self.assertIn(f"Figure {n}:", body,
                          f"figure {n} has no caption in the built PDF")
        self.assertNotIn(f"Figure {len(bp.spec_captions()) + 1}:", body,
                         "the paper typesets more figures than the spec captions")
        for stem, _ in bp.spec_captions():
            self.assertTrue((bp.BUILD / "figures" / f"{stem}.pdf").exists(),
                            f"{stem} was not copied into the build")


class BibliographyIsNotSilentlyEmptyTest(unittest.TestCase):
    r"""A bibliography that could not build must not look like one that did.

    `build_paper.py` copies `references.bib` into the build and emits
    `\bibliography{references}`, so the document promises a reference list. It
    then ran bibtex with `capture_output=True` and discarded both the exit
    status and the log. bibtex had been reporting

        I found no \citation commands---while reading file main.aux
        You've used 0 entries,

    for as long as the draft has cited in prose, and the build printed success
    every time. Eighteen curated entries, zero of them rendered, nothing saying
    so.

    bibtex exits 0 in that case, so the status alone cannot detect it -- which
    is why the check reads the log.
    """

    @staticmethod
    def _builder():
        spec = importlib.util.spec_from_file_location("build_paper", BUILDER)
        mod = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(mod)
        return mod

    def test_the_no_citations_log_is_detected(self):
        bp = self._builder()
        log = bp.BUILD / "main.blg"
        self.assertTrue(log.exists(), "no bibtex log; build the paper first")
        found = bp.NO_CITATIONS in log.read_text(errors="replace")
        problems = bp.bibliography_problems()
        self.assertEqual(
            bool(problems), found,
            "the log says the bibliography is empty but the check is silent, "
            "or the reverse")

    def test_a_missing_log_is_a_problem_not_a_pass(self):
        """An absent log means 'unknown', and unknown must not read as good."""
        bp = self._builder()
        with mock.patch.object(bp, "BUILD", pathlib.Path("/nonexistent-build")):
            self.assertTrue(bp.bibliography_problems(),
                            "a missing bibtex log passed silently")

    def test_the_check_is_actually_wired_into_the_build(self):
        """The function existing is not the function running.

        Deleting `problems.extend(bibliography_problems())` from `check()`
        leaves every other test in this class passing -- they call the helper
        directly -- and the build goes back to reporting success on an empty
        bibliography. That is the defect this class was written for, reproduced
        inside its own tests, so the wiring is asserted through the CLI rather
        than through the module.
        """
        bp = self._builder()
        log = bp.BUILD / "main.blg"
        if not log.exists() or bp.NO_CITATIONS not in log.read_text(errors="replace"):
            self.skipTest("this build's bibliography is not empty")
        proc = subprocess.run([sys.executable, str(BUILDER), "--check"],
                              capture_output=True, text=True)
        self.assertIn("bibliography is EMPTY", proc.stdout + proc.stderr,
                      "the empty bibliography is not reported by --check, so "
                      "the helper is not wired into check()")
        self.assertNotEqual(proc.returncode, 0,
                            "--check exited 0 on an empty bibliography")

    def test_the_message_names_the_entry_count(self):
        """A reader must learn how many references were lost, not just that
        some were: '0 of 18' is actionable, 'empty' is not."""
        bp = self._builder()
        problems = bp.bibliography_problems()
        if not problems:
            self.skipTest("bibliography is not empty in this build")
        entries = len(re.findall(r"^@\w+\{", bp.BIB.read_text(), re.M))
        self.assertIn(str(entries), problems[0])


if __name__ == "__main__":
    unittest.main(verbosity=2)
