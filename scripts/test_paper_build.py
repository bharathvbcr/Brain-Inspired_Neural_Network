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

import contextlib
import importlib.util
import io
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

    def test_every_style_hash_matches_the_file_on_disk(self):
        """If this fails, either a style file was replaced or the pin is
        stale. Both are things a human decides, and neither may be papered
        over by the build recomputing the hash it is checking."""
        import hashlib
        self.assertEqual(set(bp.STYLE_FILES), {"tmlr.sty", "tmlr.bst", "fancyhdr.sty"})
        for name, want in bp.STYLE_FILES.items():
            got = hashlib.sha256((ROOT / "paper" / name).read_bytes()).hexdigest()
            self.assertEqual(got, want, name)

    def test_every_pinned_hash_is_one_the_provenance_note_records(self):
        note = (ROOT / "paper/STYLE_PROVENANCE.md").read_text()
        for name, want in bp.STYLE_FILES.items():
            self.assertIn(want, note,
                          f"the pin for {name} and the note that justifies it have drifted")

    def test_a_tampered_style_file_stops_the_build(self):
        tmp = pathlib.Path(tempfile.mkdtemp())
        self.addCleanup(shutil.rmtree, tmp, ignore_errors=True)
        for name in bp.STYLE_FILES:
            shutil.copy(ROOT / "paper" / name, tmp / name)
        bp.verify_style(tmp)
        (tmp / "tmlr.sty").write_text((tmp / "tmlr.sty").read_text() + "% tweak\n")
        with self.assertRaises(SystemExit):
            bp.verify_style(tmp)

    def test_a_multi_paragraph_abstract_is_refused(self):
        """TMLR's template: 'The abstract must be limited to one paragraph.'
        The draft's abstract was four paragraphs under NeurIPS."""
        self.assertTrue(bp.abstract_problems("One claim.\n\nAnother claim.\n"))
        self.assertTrue(bp.abstract_problems("   \n"))
        self.assertEqual(bp.abstract_problems("One claim,\nwrapped over lines.\n"), [])

    def test_the_draft_abstract_is_one_paragraph(self):
        import tomllib
        manifest = tomllib.loads((ROOT / "paper/split_manifest.toml").read_text())
        abstract, disclosure = bp.front_matter(
            bp.split_sections((ROOT / "results/PAPER_DRAFT.md").read_text()), manifest)
        self.assertEqual(bp.abstract_problems(abstract), [])
        self.assertIn(bp.DISCLOSURE_MARKER, disclosure.lower(),
                      "the disclosure must say what the first-page check looks for")
        self.assertNotIn("\n\n", disclosure, "a footnote is one paragraph")

    def test_a_missing_front_section_stops_the_build(self):
        sections = [("Abstract", "One paragraph.\n")]
        with self.assertRaises(SystemExit):
            bp.front_matter(sections, {"front": {"abstract": "Abstract",
                                                 "disclosure": "LLM-use disclosure"}})
        with self.assertRaises(SystemExit):
            bp.front_matter(sections, {"front": {"abstract": "Abstract"}})

    def test_the_divider_after_a_front_section_is_not_part_of_it(self):
        abstract, disclosure = bp.front_matter(
            [("A", "Body.\n\n---\n\n"), ("D", "Used a language model.\n\n---\n")],
            {"front": {"abstract": "A", "disclosure": "D"}})
        self.assertEqual((abstract, disclosure), ("Body.", "Used a language model."))

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
        body = set(manifest["main"]["sections"]) | set(manifest["appendix"]["sections"])
        front = set(manifest["front"].values())
        omitted = bp.omitted_sections(manifest)
        self.assertEqual(front & body, set(), "a front section would print twice")
        self.assertEqual(omitted & (body | front), set(), "omitted AND placed")
        placed = body | front | omitted
        have = {h for h, _ in bp.split_sections((ROOT / "results/PAPER_DRAFT.md").read_text())}
        self.assertEqual(have - placed, set())
        self.assertEqual(placed - have, set())

    def test_the_page_limit_is_the_venue_s(self):
        self.assertEqual(bp.PAGE_LIMIT, 12,
                         "TMLR drops its review timeline above twelve main pages")

    def test_the_variants_differ_only_in_option_author_and_directory(self):
        sub, pre = bp.VARIANTS["submission"], bp.VARIANTS["preprint"]
        self.assertEqual((sub["option"], sub["author"]), ("", ""))
        self.assertEqual(pre["option"], "[preprint]")
        self.assertNotEqual(sub["build"], pre["build"],
                            "a shared build directory could leave the named "
                            "PDF where the anonymous one is expected")
        self.assertIn(r"\usepackage{tmlr}", bp.document_head("submission"))
        self.assertIn(r"\usepackage[preprint]{tmlr}", bp.document_head("preprint"))

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
        self.assertEqual(halves, {"main", "appendix", "omitted"})
        self.assertEqual(len(bp.FIGURE_PLACEMENT), len(bp.spec_captions()))

    def test_an_omitted_list_without_a_reason_is_refused(self):
        with self.assertRaises(SystemExit):
            bp.omitted_sections({"omitted": {"sections": ["2.1 X"], "reason": " "}})
        self.assertEqual(bp.omitted_sections({}), set())

    def test_a_reference_into_an_omitted_section_stops_the_build(self):
        """Omitted sections get no paper number, so the resolver refuses a
        typeset reference to one: the paper cannot point at what it left out."""
        import tomllib
        manifest = tomllib.loads((ROOT / "paper/split_manifest.toml").read_text())
        _, ids = bp.plan_numbering(
            manifest["main"]["sections"], manifest["appendix"]["sections"],
            set(manifest["numbering"]["promote"]), dict(manifest["titles"]))
        omitted = bp.omitted_sections(manifest)
        self.assertIn("3.6 The reference is undertrained, and the task saturates", omitted)
        with self.assertRaises(SystemExit):
            bp.rewrite_xrefs("bounded by a saturating reference (§3.6)", ids, "test")

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

    def test_repository_identifiers_are_named_when_present(self):
        """Record filenames, results/ paths and config hashes exist verbatim in
        the author's public repository. Until 2026-09-29 the submission printed
        thirty of them in its record table and nothing checked."""
        leaks = ("see RESULT_2026-08-20_D32L4_CLEARS_THE_080_GATE.md here",
                 "corpus at results/shd_attention_campaign_v2 there",
                 "replay --config-hash c1-118207fbc3eaba53",
                 "finding_2026-08-29_the_h1024 in lower case")
        for text in leaks:
            with self.subTest(text=text):
                self.assertEqual(len(bp.repo_identifier_problems(text)), 1)
        for clean in ("the results of binning", "a result dated 2026-08-20",
                      "Figure 1 and r12"):
            with self.subTest(text=clean):
                self.assertEqual(bp.repo_identifier_problems(clean), [])

    def test_a_count_of_authors_is_named_when_present(self):
        """The paper writes "we" throughout, which says nothing about how many
        wrote it. Until 2026-09-29 the LLM footnote said "the author reviewed
        the text", which says one, on page 1 of a double-blind submission."""
        for text in ("and the author reviewed the text",
                     "and the\nauthor's review",
                     "as its sole author", "a single author wrote it"):
            with self.subTest(text=text):
                self.assertEqual(len(bp.author_count_problems(text)), 1)
        for clean in ("the authors reviewed it", "we reviewed the text",
                      "authorship of the code", "Cramer et al. author"):
            with self.subTest(text=clean):
                self.assertEqual(bp.author_count_problems(clean), [])

    def test_the_text_guards_are_wired_into_the_check(self):
        """The draft is clean, so a real build never reaches either guard's
        failure path, and deleting its call from `check()` left every test
        passing. Each helper is made to report, and `check()` must fail."""
        if not (bp.BUILD / "main.pdf").exists():
            self.fail("build the submission first; a missing build must not "
                      "read as a pass")
        for guard in ("repo_identifier_problems", "author_count_problems"):
            with self.subTest(guard=guard), \
                    mock.patch.object(bp, guard, return_value=[f"{guard} fired"]), \
                    contextlib.redirect_stdout(io.StringIO()) as out:
                self.assertEqual(bp.check("submission"), 1)
                self.assertIn(f"{guard} fired", out.getvalue())

    def test_the_submission_lists_markers_not_filenames(self):
        register = {"RESULT_2026-08-20_X.md": 1, "FINDING_2026-08-29_Y.md": 2}
        sub = bp.record_section("submission", register, "rows")
        self.assertEqual(bp.repo_identifier_problems(sub), [])
        self.assertIn("r1--r2", sub)
        self.assertIn("RECORDS.md", sub)
        self.assertIn("rows", bp.record_section("preprint", register, "rows"))

    def test_the_supplement_map_numbers_records_as_the_paper_does(self):
        import tomllib
        manifest = tomllib.loads((ROOT / "paper/split_manifest.toml").read_text())
        register = bp.record_register(
            manifest, bp.split_sections((ROOT / "results/PAPER_DRAFT.md").read_text()))
        mapping = bp.record_map()
        for name, n in register.items():
            self.assertIn(f"| r{n} | `results/{name}` |", mapping)
        self.assertEqual(len(re.findall(r"^\| r\d+ \|", mapping, re.M)), len(register))

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

    def test_it_says_anonymous_on_page_one(self):
        first = subprocess.run(
            ["pdftotext", "-f", "1", "-l", "1", str(self.pdf), "-"],
            capture_output=True, text=True).stdout
        self.assertIn(bp.ANONYMOUS_BANNER, " ".join(first.split()))

    def test_page_one_carries_the_llm_disclosure(self):
        first = subprocess.run(
            ["pdftotext", "-f", "1", "-l", "1", str(self.pdf), "-"],
            capture_output=True, text=True).stdout.lower()
        self.assertIn(bp.DISCLOSURE_MARKER, " ".join(first.split()))

    def test_every_main_section_heading_survives_conversion(self):
        """A section whose body is empty was dropped whole: pandoc with
        --natbib emits nothing for a document that is only a heading, and the
        build once converted one section at a time."""
        import tomllib
        manifest = tomllib.loads((ROOT / "paper/split_manifest.toml").read_text())
        placed, _ = bp.plan_numbering(
            manifest["main"]["sections"], manifest["appendix"]["sections"],
            set(manifest.get("numbering", {}).get("promote", [])),
            dict(manifest.get("titles", {})))
        tex = (bp.BUILD / "main.tex").read_text()
        for heading, number, _ in placed:
            self.assertRegex(tex, rf"section\*\{{{re.escape(number)} ",
                             f"{heading!r} (numbered {number}) is missing")

    def test_no_heading_is_numbered_twice(self):
        """The manuscript numbers its headings and the prose cross-references
        those numbers, so LaTeX must not number them again. Under the NeurIPS
        build it did: the PDF read "0.2 1. Introduction"."""
        tex = (bp.BUILD / "main.tex").read_text()
        numbered = re.findall(r"\\(?:sub)*section\{", tex)
        self.assertEqual(numbered, [], "a heading is LaTeX-numbered")
        body = subprocess.run(["pdftotext", "-layout", str(self.pdf), "-"],
                              capture_output=True, text=True).stdout
        self.assertIsNone(re.search(r"^\s*0\.\d+\s+\d+\. ", body, re.M))

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
        typeset = [s for s, _ in bp.spec_captions() if bp.FIGURE_PLACEMENT[s] != "omitted"]
        for n in range(1, len(typeset) + 1):
            self.assertIn(f"Figure {n}:", body,
                          f"figure {n} has no caption in the built PDF")
        self.assertNotIn(f"Figure {len(typeset) + 1}:", body,
                         "the paper typesets more figures than it places")
        for stem in typeset:
            self.assertTrue((bp.BUILD / "figures" / f"{stem}.pdf").exists(),
                            f"{stem} was not copied into the build")


class NumberingAndCrossReferenceTest(unittest.TestCase):
    """The paper numbers its sections in manifest order and rewrites the
    draft's references to match. Each failure here printed a PDF whose
    references pointed at the wrong section, or at nothing."""

    def plan(self):
        import tomllib
        manifest = tomllib.loads((ROOT / "paper/split_manifest.toml").read_text())
        return bp.plan_numbering(
            manifest["main"]["sections"], manifest["appendix"]["sections"],
            set(manifest.get("numbering", {}).get("promote", [])),
            dict(manifest.get("titles", {})))

    def test_main_text_is_numbered_without_gaps(self):
        placed, _ = self.plan()
        main = [n for _, n, _ in placed if n[0].isdigit()]
        tops = [n for n in main if "." not in n]
        self.assertEqual(tops, [str(i) for i in range(1, len(tops) + 1)])
        for top in tops:
            subs = [int(n.split(".")[1]) for n in main if n.startswith(top + ".")]
            self.assertEqual(subs, list(range(1, len(subs) + 1)), top)

    def test_the_introduction_comes_first(self):
        placed, ids = self.plan()
        self.assertEqual(placed[0][:2], ("1. Introduction", "1"))
        self.assertEqual(ids["0"], "2", "related work (the draft's §0) is section 2")

    def test_every_draft_reference_resolves(self):
        """Rewrites every typeset section and caption; an unresolvable
        reference is SystemExit. Sections in [omitted] are not typeset and
        still cite what left with them, so they are not read here -- until
        2026-09-29 this test rewrote the whole draft."""
        import tomllib
        manifest = tomllib.loads((ROOT / "paper/split_manifest.toml").read_text())
        _, ids = self.plan()
        bodies = dict(bp.split_sections((ROOT / "results/PAPER_DRAFT.md").read_text()))
        placed = (manifest["main"]["sections"] + manifest["appendix"]["sections"]
                  + list(manifest["front"].values()))
        texts = [bodies[h] for h in placed] + [
            c for s, c in bp.spec_captions() if bp.FIGURE_PLACEMENT[s] != "omitted"]
        for text in texts:
            out = bp.rewrite_xrefs(text, ids, "draft")
            self.assertNotRegex(out, r"Figure [0-9SM]\b")

    def test_a_reference_to_a_missing_section_stops_the_build(self):
        with self.assertRaises(SystemExit):
            bp.rewrite_xrefs("see §9.9", {"1": "1"}, "test")
        with self.assertRaises(SystemExit):
            bp.rewrite_xrefs("see Appendix Z", {}, "test")

    def test_a_reference_to_an_untypeset_figure_stops_the_build(self):
        with self.assertRaises(SystemExit):
            bp.rewrite_xrefs("as drawn (Figure 7)", {}, "test")

    def test_section_and_appendix_forms_are_rewritten(self):
        ids = {"3.5": "4.1", "3.1": "H.3", "Appendix E": "B"}
        out = bp.rewrite_xrefs(
            "§3.5, Section 3.5, Section 3.1, §3.1, Appendix E and Figure 1",
            ids, "test")
        self.assertEqual(out, "§4.1, Section 4.1, Appendix H.3, Appendix H.3, Appendix B and "
                              r"Figure \ref{fig:leadfig1_the_conditional}")

    def test_an_appendix_heading_is_not_a_reference(self):
        self.assertEqual(bp.rewrite_xrefs("Appendix E — The width", {}, "t"),
                         "Appendix E — The width")

    def test_spec_figure_ids_cover_exactly_the_typeset_figures(self):
        self.assertEqual(set(bp.SPEC_FIGURE_ID.values()),
                         {s for s, half in bp.FIGURE_PLACEMENT.items() if half != "omitted"})


class PreprintTest(unittest.TestCase):
    """The named arXiv build. It must never be mistaken for the submission."""

    @classmethod
    def setUpClass(cls) -> None:
        cls.pdf = bp.PREPRINT_BUILD / "main.pdf"
        if not cls.pdf.exists():
            raise unittest.SkipTest(
                "no preprint on disk; run scripts/build_paper.py --variant preprint")

    def test_the_preprint_passes_its_own_check(self):
        proc = subprocess.run([sys.executable, str(BUILDER), "--check",
                               "--variant", "preprint"],
                              capture_output=True, text=True)
        self.assertEqual(proc.returncode, 0, proc.stdout + proc.stderr)

    def test_it_names_the_author_and_not_the_venue(self):
        first = subprocess.run(["pdftotext", "-f", "1", "-l", "1", str(self.pdf), "-"],
                               capture_output=True, text=True).stdout
        self.assertIn("Independent Researcher", first)
        self.assertNotIn("TMLR", first)
        self.assertNotIn(bp.ANONYMOUS_BANNER, " ".join(first.split()))

    def test_the_submission_check_rejects_the_preprint(self):
        """If the named PDF were ever copied into the submission directory,
        the submission check must fail on it."""
        tmp = pathlib.Path(tempfile.mkdtemp())
        self.addCleanup(shutil.rmtree, tmp, ignore_errors=True)
        for name in list(bp.STYLE_FILES) + ["main.blg"]:
            shutil.copy(bp.PREPRINT_BUILD / name, tmp / name)
        shutil.copy(self.pdf, tmp / "main.pdf")
        variants = dict(bp.VARIANTS)
        variants["submission"] = dict(variants["submission"], build=tmp)
        with mock.patch.object(bp, "VARIANTS", variants):
            self.assertEqual(bp.check("submission"), 1)


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
        inside its own tests, so the wiring is asserted through `check()`
        rather than through the helper.

        Until 2026-09-29 this ran only when the real build's bibliography was
        empty, so on every good build it skipped and asserted nothing. It now
        runs `check()` against the real build with only its bibtex log swapped
        for one that found no citations.
        """
        bp = self._builder()
        build = self._build_with_an_empty_bibliography(bp)
        out = io.StringIO()
        with mock.patch.dict(bp.VARIANTS["submission"], {"build": build}), \
                contextlib.redirect_stdout(out):
            code = bp.check("submission")
        self.assertIn("bibliography is EMPTY", out.getvalue(),
                      "the empty bibliography is not reported by check(), so "
                      "the helper is not wired into it")
        self.assertNotEqual(code, 0, "check() passed an empty bibliography")

    def test_the_message_names_the_entry_count(self):
        """A reader must learn how many references were lost, not just that
        some were: '0 of 18' is actionable, 'empty' is not. Like the test
        above, this used to skip on every good build."""
        bp = self._builder()
        problems = bp.bibliography_problems(self._build_with_an_empty_bibliography(bp))
        self.assertEqual(len(problems), 1)
        entries = len(re.findall(r"^@\w+\{", bp.BIB.read_text(), re.M))
        self.assertIn(f"0 of the {entries} entries", problems[0])

    def _build_with_an_empty_bibliography(self, bp) -> pathlib.Path:
        """The real build, linked entry by entry (it is hundreds of MB), with
        a bibtex log that found no citations in place of the real one."""
        if not (bp.BUILD / "main.pdf").exists():
            self.fail(f"{bp.BUILD / 'main.pdf'} does not exist; build the paper "
                      "first. A missing build must not read as a pass.")
        tmp = pathlib.Path(tempfile.mkdtemp())
        self.addCleanup(shutil.rmtree, tmp, ignore_errors=True)
        for entry in bp.BUILD.iterdir():
            if entry.name != "main.blg":
                (tmp / entry.name).symlink_to(entry)
        (tmp / "main.blg").write_text(
            f"This is BibTeX\nThe top-level auxiliary file: main.aux\n"
            f"{bp.NO_CITATIONS}---while reading file main.aux\n")
        return tmp


def _linked_build(testcase: unittest.TestCase, variant: str,
                  keep_manifest: bool = False) -> pathlib.Path:
    """The real build for `variant`, linked entry by entry into a temp dir, with
    no inputs manifest unless asked (it is written by the caller, under
    whichever inputs the test has patched in)."""
    src = bp.VARIANTS[variant]["build"]
    if not (src / "main.pdf").exists():
        testcase.fail(f"{src / 'main.pdf'} does not exist; build the paper first. "
                      "A missing build must not read as a pass.")
    tmp = pathlib.Path(tempfile.mkdtemp())
    testcase.addCleanup(shutil.rmtree, tmp, ignore_errors=True)
    for entry in src.iterdir():
        if entry.name != "inputs.sha256.json":
            (tmp / entry.name).symlink_to(entry)
    return tmp


def _run_check(variant: str, build: pathlib.Path, **patched) -> tuple[int, str]:
    patched.setdefault("DRAFT", bp.DRAFT)    # patch.multiple needs one keyword
    variants = {k: dict(v) for k, v in bp.VARIANTS.items()}
    variants[variant]["build"] = build
    out = io.StringIO()
    with mock.patch.object(bp, "VARIANTS", variants), \
            mock.patch.multiple(bp, create=True, **patched), contextlib.redirect_stdout(out):
        return bp.check(variant), out.getvalue()


class StaleBuildTest(unittest.TestCase):
    """`--check` is what record_checks.sh runs. It used to re-read only the
    abstract from the draft and then validate whatever PDF was on disk, so a
    PDF built before the draft last changed was blessed. Appending "The sole
    author of this work is Bharath, see results/foo." to the draft left
    `--check` exiting 0 while a real rebuild failed with four problems."""

    def setUp(self):
        self.tmp = pathlib.Path(tempfile.mkdtemp())
        self.addCleanup(shutil.rmtree, self.tmp, ignore_errors=True)
        self.draft = self.tmp / "PAPER_DRAFT.md"
        shutil.copy(bp.DRAFT, self.draft)
        self.figs = self.tmp / "figures"
        shutil.copytree(bp.FIGDIR, self.figs,
                        ignore=shutil.ignore_patterns("*.png", "*.svg"))
        self.me = self.tmp / "build_paper.py"
        shutil.copy(BUILDER, self.me)
        self.inputs = dict(DRAFT=self.draft, FIGDIR=self.figs, SELF=self.me)

    def _built(self, variant: str = "submission") -> pathlib.Path:
        build = _linked_build(self, variant)
        with mock.patch.multiple(bp, create=True, **self.inputs):
            bp.write_inputs_manifest(variant, build)
        return build

    def test_a_build_matching_its_inputs_passes(self):
        build = self._built()
        code, out = _run_check("submission", build, **self.inputs)
        self.assertEqual(code, 0, out)

    def test_a_missing_manifest_is_stale_not_a_pass(self):
        code, out = _run_check("submission", _linked_build(self, "submission"),
                               **self.inputs)
        self.assertEqual(code, 1, out)
        self.assertIn("stale build", out)

    def test_a_draft_edited_after_the_build_is_stale(self):
        build = self._built()
        self.draft.write_text(self.draft.read_text() +
                              "\nThe sole author of this work is Bharath, see results/foo.\n")
        code, out = _run_check("submission", build, **self.inputs)
        self.assertEqual(code, 1, out)
        self.assertIn("stale build", out)
        self.assertIn("PAPER_DRAFT.md", out)

    def test_a_touched_but_identical_file_is_not_stale(self):
        build = self._built()
        self.draft.write_bytes(self.draft.read_bytes())
        import os
        os.utime(self.draft, (1, 1))
        code, out = _run_check("submission", build, **self.inputs)
        self.assertEqual(code, 0, out)

    def test_a_changed_figure_is_stale(self):
        build = self._built()
        stem = next(s for s, _ in bp.spec_captions()
                    if bp.FIGURE_PLACEMENT[s] != "omitted")
        fig = self.figs / f"{stem}.pdf"
        fig.write_bytes(fig.read_bytes() + b"\n% redrawn\n")
        code, out = _run_check("submission", build, **self.inputs)
        self.assertEqual(code, 1, out)
        self.assertIn(f"{stem}.pdf", out)

    def test_a_changed_builder_is_stale(self):
        build = self._built()
        self.me.write_text(self.me.read_text() + "\n# changed\n")
        code, out = _run_check("submission", build, **self.inputs)
        self.assertEqual(code, 1, out)
        self.assertIn("stale build", out)

    def test_a_changed_supplement_module_is_stale(self):
        """`build_paper.py` loads `build_supplement.py` to rewrite the
        submission's text, so the PDF is a function of it too."""
        supp = self.tmp / "build_supplement.py"
        shutil.copy(bp.SUPPLEMENT, supp)
        self.inputs["SUPPLEMENT"] = supp
        build = self._built()
        code, out = _run_check("submission", build, **self.inputs)
        self.assertEqual(code, 0, out)
        supp.write_text(supp.read_text() + "\n# changed\n")
        code, out = _run_check("submission", build, **self.inputs)
        self.assertEqual(code, 1, out)
        self.assertIn("build_supplement.py", out)

    def _build_with_fake_latex(self, mutate) -> tuple[pathlib.Path, str]:
        """`build()` for real (pandoc included) with pdflatex/bibtex replaced by
        a stub that runs `mutate()` once, as an edit made during compilation."""
        out_dir = self.tmp / "out"
        out_dir.mkdir()
        real = subprocess.run
        fired = []

        def fake(cmd, *a, **kw):
            if cmd[0] in ("pdflatex", "bibtex"):
                if not fired:
                    fired.append(1)
                    mutate()
                return subprocess.CompletedProcess(cmd, 0, "", "")
            return real(cmd, *a, **kw)

        variants = {k: dict(v) for k, v in bp.VARIANTS.items()}
        variants["submission"]["build"] = out_dir
        err = io.StringIO()
        with mock.patch.object(bp, "VARIANTS", variants), \
                mock.patch.multiple(bp, create=True, **self.inputs), \
                mock.patch.object(bp.subprocess, "run", fake), \
                contextlib.redirect_stdout(io.StringIO()), \
                contextlib.redirect_stderr(err):
            try:
                bp.build("submission")
            except SystemExit:
                pass
        return out_dir, err.getvalue()

    def test_a_build_with_stable_inputs_records_them(self):
        out_dir, _ = self._build_with_fake_latex(lambda: None)
        self.assertTrue((out_dir / bp.INPUTS_MANIFEST).exists())

    def test_an_input_edited_mid_build_is_not_blessed(self):
        """Hashing after the pdflatex passes blessed whichever version of the
        draft was on disk by then, while the PDF came from the one read first."""
        def edit():
            self.draft.write_text(self.draft.read_text() + "\nedited mid-build\n")
        out_dir, err = self._build_with_fake_latex(edit)
        self.assertFalse((out_dir / bp.INPUTS_MANIFEST).exists())
        self.assertIn("changed while the paper was building", err)
        self.assertIn("PAPER_DRAFT.md", err)

    def test_a_changed_reference_list_is_stale(self):
        build = self._built()
        bib = self.tmp / "references.bib"
        bib.write_text(bp.BIB.read_text() + "\n% edited\n")
        code, out = _run_check("submission", build, BIB=bib, **self.inputs)
        self.assertEqual(code, 1, out)
        self.assertIn("references.bib", out)

    def test_the_other_variants_manifest_does_not_vouch(self):
        """The preprint's manifest, dropped into the submission build, is for a
        different PDF."""
        pre = self._built("preprint")
        build = _linked_build(self, "submission")
        shutil.copy(pre / "inputs.sha256.json", build / "inputs.sha256.json")
        code, out = _run_check("submission", build, **self.inputs)
        self.assertEqual(code, 1, out)
        self.assertIn("variant", out)

    def test_the_preprint_is_checked_the_same_way(self):
        build = _linked_build(self, "preprint")
        code, out = _run_check("preprint", build, **self.inputs)
        self.assertEqual(code, 1, out)
        self.assertIn("stale build", out)
        self.assertIn("--variant preprint", out)


class UnresolvedReferencesTest(unittest.TestCase):
    """bibtex exits 0 or 1 on a missing database entry, and pdflatex exits 0 on
    an undefined citation, so neither status carries it. Deleting baronig2025
    from references.bib built and checked clean while main.blg said "didn't
    find a database entry" and the PDF printed "(?)"."""

    def _check_with(self, name: str, extra: str):
        build = _linked_build(self, "submission")
        target = build / name
        original = (bp.VARIANTS["submission"]["build"] / name).read_text(errors="replace")
        target.unlink()
        target.write_text(original + extra)
        # Freshness is StaleBuildTest's subject; here the build is current, so
        # a failure can only be the logs.
        if hasattr(bp, "write_inputs_manifest"):
            bp.write_inputs_manifest("submission", build)
        return _run_check("submission", build)

    def test_the_real_logs_and_pdf_are_clean(self):
        build = _linked_build(self, "submission")
        text = subprocess.run(["pdftotext", str(build / "main.pdf"), "-"],
                              capture_output=True, text=True).stdout
        self.assertEqual(bp.resolution_problems(build, text), [])

    def test_a_missing_database_entry_fails_the_check(self):
        code, out = self._check_with(
            "main.blg", "Warning--I didn't find a database entry for \"baronig2025\"\n")
        self.assertEqual(code, 1, out)
        self.assertIn("baronig2025", out)

    def test_a_missing_database_file_fails_the_check(self):
        code, out = self._check_with("main.blg", "I couldn't open database file x.bib\n")
        self.assertEqual(code, 1, out)
        self.assertIn("could not open x.bib", out)

    def test_an_undefined_citation_fails_the_check(self):
        code, out = self._check_with(
            "main.log", "LaTeX Warning: Citation `baronig2025' on page 3 undefined on input line 9.\n")
        self.assertEqual(code, 1, out)
        self.assertIn("undefined citation", out)

    def test_an_undefined_reference_fails_the_check(self):
        code, out = self._check_with(
            "main.log", "LaTeX Warning: Reference `fig:x' on page 3 undefined on input line 9.\n"
                        "LaTeX Warning: There were undefined references.\n")
        self.assertEqual(code, 1, out)
        self.assertIn("undefined reference", out)

    def test_a_question_mark_citation_in_the_pdf_text_fails(self):
        for rendered in ("as shown by Baronig (?) and others", "see Figure ??"):
            with self.subTest(rendered=rendered):
                problems = bp.resolution_problems(
                    bp.VARIANTS["submission"]["build"], rendered)
                self.assertTrue(any("'(?)' or '??'" in p for p in problems))

    def test_prose_that_says_question_marks_is_not_an_unresolved_reference(self):
        build = bp.VARIANTS["submission"]["build"]
        for rendered in ("Is it real?? Yes.", "what (?) means"):
            with self.subTest(rendered=rendered):
                self.assertEqual(
                    bp.resolution_problems(build, rendered, rendered), [])

    def test_an_unresolved_reference_on_top_of_prose_ones_still_fails(self):
        build = bp.VARIANTS["submission"]["build"]
        problems = bp.resolution_problems(
            build, "Is it real?? Yes. See Figure ??.", "Is it real?? Yes.")
        self.assertTrue(any("'(?)' or '??'" in p for p in problems))

    def test_the_forms_this_template_prints_for_undefined_keys_are_detected(self):
        """Measured with tmlr.sty + pdflatex + pdftotext: \\citep -> "(?)",
        \\ref -> "??", two keys in one \\citep -> "(??)"."""
        build = bp.VARIANTS["submission"]["build"]
        for rendered in ("A (?) B", "Figure ??", "G (??)."):
            with self.subTest(rendered=rendered):
                self.assertTrue(any("'(?)' or '??'" in p for p in
                                    bp.resolution_problems(build, rendered, "")))

    def test_a_missing_latex_log_is_unknown_not_clean(self):
        build = _linked_build(self, "submission")
        (build / "main.log").unlink()
        self.assertTrue(bp.resolution_problems(build, ""))


if __name__ == "__main__":
    unittest.main(verbosity=2)
