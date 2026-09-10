"""Tests for the manuscript sweep in `check_every_number.py`.

`PAPER_DRAFT.md` was excluded from that sweep until 2026-08-27 and the exclusion
was announced rather than closed. The announcement was honest and it was still
the largest hole in the record: the one artefact a reader outside this
repository will ever see was the one artefact no mechanical check touched.

The sweep that replaced it reports three NAMED tiers, and most of this file is
about keeping them apart. Tier C — "this value is still present in one named
primary record" — is weaker than tier A — "the cells produce this value" — and
the failure mode worth testing for is not a wrong number but a weak check
wearing a strong check's clothes.

The rest is the usual negative testing: every rule the sweep enforces is broken
here and asserted to fire. A provenance table that cannot fail is a provenance
table that has stopped meaning anything.

Run: python3 scripts/test_paper_number_sweep.py
"""

from __future__ import annotations

import importlib.util
import re
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "scripts"))

import check_every_number as CEN  # noqa: E402


class EveryLiveManuscriptIsSweptTest(unittest.TestCase):
    """A second manuscript may not exist outside the sweep without saying so.

    This file's own opening records the hole this closes for one document:
    "the one artefact a reader outside this repository will ever see was the
    one artefact no mechanical check touched." It was closed for
    `PAPER_DRAFT.md` and stayed open as a *class* -- on 2026-09-07 a second
    743-line manuscript, `PAPER_BINN_INSTRUMENT_2026-08-31.md`, was found
    making publishable claims with **no verification script reading it at
    all**: not this sweep, not the terminology check, not the citation check,
    not the paper build. It had been that way since 2026-08-31.

    Nothing structural had prevented it, so nothing prevented the next one. The
    rule enforced here is the general form: a document that carries an abstract
    is a manuscript, and a manuscript is either the one this sweep reads or one
    that says on its face that it is not live.

    Retirement is judged by `build_results_index.describe`, which already owns
    that question for the results index, rather than by a second banner matcher
    written here that could drift away from it.
    """

    @staticmethod
    def _index():
        spec = importlib.util.spec_from_file_location(
            "build_results_index", ROOT / "scripts/build_results_index.py")
        mod = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(mod)
        return mod

    @staticmethod
    def _manuscripts() -> list[Path]:
        """Documents carrying an abstract, which is what makes one a paper."""
        return sorted(
            path for path in (ROOT / "results").glob("*.md")
            if re.search(r"^#{1,3} Abstract", path.read_text(errors="replace"),
                         re.M))

    def test_the_detector_finds_the_manuscripts_that_exist(self):
        """A criterion matching nothing would pass the next test forever.

        Two are known: the swept draft and the merged-away instrument paper.
        """
        found = {p.name for p in self._manuscripts()}
        self.assertIn(CEN.PAPER.name, found)
        self.assertIn("PAPER_BINN_INSTRUMENT_2026-08-31.md", found)
        self.assertGreaterEqual(len(found), 2)

    def test_every_manuscript_is_swept_or_retired(self):
        index = self._index()
        stray = [p.name for p in self._manuscripts()
                 if p != CEN.PAPER and not index.describe(p)["retired"]]
        self.assertEqual(
            stray, [],
            "these documents carry an abstract, so a reader will read them as "
            "papers, but this sweep does not check their numbers and they do "
            "not open with a WITHDRAWN or SUPERSEDED banner:\n  "
            + "\n  ".join(stray)
            + "\nEither add the document to the sweep or say on its face that "
              "it is not live. An unchecked manuscript is how "
              "PAPER_BINN_INSTRUMENT_2026-08-31.md accrued 743 lines of claims "
              "that nothing verified.")

    def test_a_new_unbannered_manuscript_would_be_caught(self):
        """The rule above, exercised against a document that breaks it.

        Written into `results/` because that is the directory the rule scans;
        removed in the same test whatever happens.
        """
        intruder = ROOT / "results" / "PAPER_TEST_INTRUDER_DELETE_ME.md"
        intruder.write_text("# A second paper\n\n## Abstract\n\nWe claim 0.9999.\n")
        try:
            self.assertIn(intruder, self._manuscripts())
            index = self._index()
            self.assertFalse(index.describe(intruder)["retired"])
            with self.assertRaises(AssertionError) as caught:
                self.test_every_manuscript_is_swept_or_retired()
            self.assertIn(intruder.name, str(caught.exception))
        finally:
            intruder.unlink()

    def test_a_bannered_manuscript_is_accepted(self):
        """The escape hatch has to work, or the rule is unsatisfiable."""
        retired = ROOT / "results" / "PAPER_TEST_RETIRED_DELETE_ME.md"
        retired.write_text(
            "# An old paper\n\n> **SUPERSEDED 2026-09-07 by the merge.**\n\n"
            "## Abstract\n\nWe claimed 0.9999.\n")
        try:
            index = self._index()
            self.assertTrue(index.describe(retired)["retired"])
            self.test_every_manuscript_is_swept_or_retired()
        finally:
            retired.unlink()


class SweepPaperTest(unittest.TestCase):
    """Drive `sweep_paper` against a scripted manuscript and source tree."""

    def sweep(self, paper: str, sources, extra: dict[str, str] | None = None,
              known=(0.1111,), allowed=("0.2222",), floor=1):
        """`(cells, elsewhere, traced, unexplained, complaints)`.

        `cells` is keyed by generator; `known` seeds the `arm` generator, which
        is enough for every rule tested here. `sources` is the tier-C table, and
        `extra` writes additional documents into the temporary tree so an entry
        can name a real file.
        """
        tiers = {name: set() for name in CEN.TIERS}
        tiers["arm"] = set(known)
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "results").mkdir()
            (root / "results/PAPER_DRAFT.md").write_text(paper)
            for relpath, text in (extra or {}).items():
                target = root / relpath
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_text(text)
            with mock.patch.object(CEN, "ROOT", root), \
                 mock.patch.object(CEN, "PAPER", root / "results/PAPER_DRAFT.md"), \
                 mock.patch.object(CEN, "PAPER_SOURCES", sources), \
                 mock.patch.object(CEN, "KNOWN_COINCIDENCE", {}), \
                 mock.patch.object(CEN, "MIN_PAPER_NUMBERS", floor):
                return CEN.sweep_paper(tiers, set(allowed))

    # --- the clean case ----------------------------------------------------

    def test_each_tier_is_counted_separately(self):
        cells, elsewhere, traced, bad, complaints = self.sweep(
            "0.1111 and 0.2222 and 0.3333",
            [("0.3333", "results/record.md", "the arm")],
            {"results/record.md": "the arm scored 0.3333"})
        self.assertEqual((sum(cells.values()), elsewhere, traced), (1, 1, 1))
        self.assertEqual(cells["arm"], 1)
        self.assertEqual(bad, [])
        self.assertEqual(complaints, [])
        # 0.1111 reaches tier A, 0.2222 is in ELSEWHERE and 0.3333 is traced;
        # none of the three collides with another tier, so no judgement is due.

    def test_a_named_source_wins_over_a_derivation(self):
        """This assertion was the other way round until 2026-08-28, and it was
        wrong. Crediting the cells first relabels provenance: `0.9390` is a
        published result from another paper and became reachable by the
        `paired` generator when the corpus grew. See
        `NamedSourceBeatsCoincidenceTest` for the full case."""
        cells, _, traced, _, complaints = self.sweep(
            "0.1111",
            [("0.1111", "results/record.md", "also here")],
            {"results/record.md": "0.1111"},
            floor=1)
        self.assertEqual((sum(cells.values()), traced), (0, 1))
        self.assertTrue(any("no judgement is recorded" in c for c in complaints),
                        complaints)

    # --- the rules, each broken --------------------------------------------

    def test_a_number_with_no_tier_is_reported(self):
        _, _, _, bad, _ = self.sweep("0.9999", [])
        self.assertEqual(bad, ["0.9999"])

    def test_a_source_that_does_not_exist_is_reported(self):
        _, _, _, _, complaints = self.sweep(
            "0.3333", [("0.3333", "results/gone.md", "the arm")])
        self.assertTrue(any("does not exist" in c for c in complaints),
                        complaints)

    def test_a_source_that_no_longer_carries_the_value_is_reported(self):
        """The drift case: the paper says one thing, its record another."""
        _, _, _, _, complaints = self.sweep(
            "0.3333", [("0.3333", "results/record.md", "the arm")],
            {"results/record.md": "the arm scored 0.3334"})
        self.assertTrue(any("drifted apart" in c for c in complaints),
                        complaints)

    def test_a_paper_side_source_is_refused(self):
        """Tracing the paper to its own table is the claim written twice."""
        _, _, _, _, complaints = self.sweep(
            "0.3333",
            [("0.3333", "results/PAPER_RESULTS_TABLE.md", "the arm")],
            {"results/PAPER_RESULTS_TABLE.md": "0.3333"})
        self.assertTrue(any("own artefacts" in c for c in complaints),
                        complaints)

    def test_every_paper_side_name_is_refused(self):
        """Not just the one that happened to be tested."""
        for name in sorted(CEN.PAPER_SIDE):
            with self.subTest(name=name):
                _, _, _, _, complaints = self.sweep(
                    "0.3333", [("0.3333", f"results/{name}", "the arm")],
                    {f"results/{name}": "0.3333"})
                self.assertTrue(any("own artefacts" in c for c in complaints),
                                f"{name} was accepted as a source")

    def test_a_duplicated_entry_is_reported(self):
        _, _, _, _, complaints = self.sweep(
            "0.3333",
            [("0.3333", "results/a.md", "one"), ("0.3333", "results/b.md", "two")],
            {"results/a.md": "0.3333", "results/b.md": "0.3333"})
        self.assertTrue(any("twice" in c for c in complaints), complaints)

    def test_an_entry_the_paper_no_longer_quotes_is_reported(self):
        """A table that keeps entries after their number leaves the manuscript
        rots into a list of things nothing checks."""
        _, _, _, _, complaints = self.sweep(
            "0.1111", [("0.3333", "results/record.md", "the arm")],
            {"results/record.md": "0.3333"})
        self.assertTrue(any("no longer quoted" in c for c in complaints),
                        complaints)

    def test_a_shrunken_manuscript_is_reported(self):
        """The floor exists so a narrowed pattern cannot pass by sweeping less."""
        _, _, _, _, complaints = self.sweep("0.1111", [], floor=50)
        self.assertTrue(any("below the floor" in c for c in complaints),
                        complaints)

    def test_a_missing_manuscript_does_not_pass_silently(self):
        """"The file is gone" and "every number checks out" must not look alike."""
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            with mock.patch.object(CEN, "ROOT", root), \
                 mock.patch.object(CEN, "PAPER", root / "nothing.md"):
                cells, elsewhere, traced, bad, complaints = CEN.sweep_paper(
                    {name: set() for name in CEN.TIERS}, set())
        self.assertEqual((sum(cells.values()), elsewhere, traced, bad),
                         (0, 0, 0, []))
        self.assertTrue(any("did not run" in c for c in complaints), complaints)


class TheRealTableTest(unittest.TestCase):
    """Invariants of the committed table, independent of the sweep's own run."""

    def test_no_value_is_claimed_at_two_tiers(self):
        """A number is explained once. Naming it in both ELSEWHERE and
        PAPER_SOURCES means two different stories about where it came from, and
        the sweep would silently prefer the stronger one."""
        both = {v for v, _ in CEN.ELSEWHERE} & {v for v, _, _ in CEN.PAPER_SOURCES}
        self.assertEqual(both, set(), f"claimed at two tiers: {sorted(both)}")

    def test_no_entry_cites_a_paper_side_artefact(self):
        cited = {Path(rel).name for _, rel, _ in CEN.PAPER_SOURCES}
        self.assertEqual(cited & CEN.PAPER_SIDE, set())

    def test_every_entry_carries_a_reason(self):
        """`(value, document, "")` would pass every other check and tell a
        reader nothing about what the number is."""
        for value, relpath, what in CEN.PAPER_SOURCES:
            self.assertTrue(what.strip(), f"{value} ({relpath}) has no reason")

    def test_the_v3_corpus_is_swept(self):
        """Waves 26-28 landed in a third corpus directory and CORPORA was last
        edited at wave 25, so all 612 of their cells sat outside this sweep
        while it reported thirteen of their own numbers as underivable. An
        empty corpus and a wrong number are not the same finding."""
        names = {c.name for c in CEN.CORPORA}
        self.assertIn("shd_attention_campaign_v3", names, names)

    def test_the_cell_floor_fires_when_a_corpus_goes_missing(self):
        """`len(groups) < 50` cannot do this job: v2 alone carries hundreds of
        configurations, so three of the four corpora can vanish beneath it.
        Losing the smallest corpus must still trip the floor."""
        loaded = CEN.load()
        cells = sum(len(seeds) for seeds in loaded.values())
        self.assertGreaterEqual(cells, CEN.MIN_CELLS)
        smallest = min(sum(1 for _ in root.glob("*.json")) for root in CEN.CORPORA)
        self.assertLess(cells - smallest, CEN.MIN_CELLS,
                        f"{cells} cells and the smallest corpus is {smallest}; "
                        f"the floor of {CEN.MIN_CELLS} would not notice it going")

    def test_the_cell_floor_is_read_and_defined_once(self):
        """A floor that could not fire is what `MIN_DOCUMENTS` was for a week."""
        source = (ROOT / "scripts/check_every_number.py").read_text()
        self.assertEqual(source.count("\nMIN_CELLS = "), 1)
        self.assertIn("cells < MIN_CELLS", source)

    def test_the_document_floor_is_read_and_defined_once(self):
        """`MIN_DOCUMENTS` was declared on 2026-08-24 and never used: a floor
        that could not fire. It was also defined twice for part of 2026-08-27,
        so breaking the first definition changed nothing."""
        source = (ROOT / "scripts/check_every_number.py").read_text()
        self.assertEqual(source.count("\nMIN_DOCUMENTS = "), 1)
        self.assertIn("len(DOCUMENTS) < MIN_DOCUMENTS", source)
        self.assertIn("len(quoted) < MIN_PAPER_NUMBERS", source)


class NamedSourceBeatsCoincidenceTest(unittest.TestCase):
    """An entry a human wrote outranks a numerical match this script found.

    Crediting the cells first silently relabels provenance. After 242 cells were
    collected on 2026-08-28, three paper numbers with explicit sources became
    reachable by a generator and were reported as "derived from the cells" —
    including `0.9390`, a published 25-tap temporal-convolutional SHD result
    from another paper, which no cell of this campaign can produce. At 22%
    density in the `paired` generator, collisions of that kind are expected.
    """

    def sweep(self, paper, sources, allowed, known, tiers):
        table = {name: set() for name in CEN.TIERS}
        table.update(tiers)
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "results").mkdir()
            (root / "results/PAPER_DRAFT.md").write_text(paper)
            for _, relpath, _ in sources:
                target = root / relpath
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_text(" ".join(v for v, _, _ in sources))
            with mock.patch.object(CEN, "ROOT", root), \
                 mock.patch.object(CEN, "PAPER", root / "results/PAPER_DRAFT.md"), \
                 mock.patch.object(CEN, "PAPER_SOURCES", sources), \
                 mock.patch.object(CEN, "KNOWN_COINCIDENCE", known), \
                 mock.patch.object(CEN, "MIN_PAPER_NUMBERS", 1):
                return CEN.sweep_paper(table, set(allowed))

    def test_a_traced_value_is_not_credited_to_a_generator(self):
        cells, _, traced, bad, complaints = self.sweep(
            "0.3333",
            [("0.3333", "results/record.md", "a literature value")],
            allowed=set(), known={"0.3333": "coincidence"},
            tiers={"paired": {0.3333}})
        self.assertEqual(traced, 1)
        self.assertEqual(sum(cells.values()), 0,
                         "a named source must outrank a coincidental match")
        self.assertEqual((bad, complaints), ([], []))

    def test_an_undeclared_overlap_is_reported(self):
        _, _, _, _, complaints = self.sweep(
            "0.3333",
            [("0.3333", "results/record.md", "a literature value")],
            allowed=set(), known={}, tiers={"paired": {0.3333}})
        self.assertTrue(any("no judgement is recorded" in c for c in complaints),
                        complaints)

    def test_a_stale_declaration_is_reported(self):
        _, _, _, _, complaints = self.sweep(
            "0.3333",
            [("0.3333", "results/record.md", "a literature value")],
            allowed=set(), known={"0.9999": "a collision that has gone"},
            tiers={})
        self.assertTrue(any("no generator reaches any more" in c
                            for c in complaints), complaints)

    def test_an_elsewhere_value_also_outranks_a_generator(self):
        cells, elsewhere, _, _, complaints = self.sweep(
            "0.3333", [], allowed={"0.3333"},
            known={"0.3333": "second-order, not the same computation"},
            tiers={"pooled": {0.3333}})
        self.assertEqual((elsewhere, sum(cells.values())), (1, 0))
        self.assertEqual(complaints, [])

    def test_the_real_declarations_are_all_still_colliding(self):
        """Every entry in the committed list describes a live collision. One
        that has stopped colliding is a judgement about nothing."""
        tiers = CEN.derivable(CEN.load())
        for value in CEN.KNOWN_COINCIDENCE:
            with self.subTest(value=value):
                self.assertIsNotNone(CEN.explain(float(value), tiers))

    def test_every_declaration_carries_a_reason(self):
        for value, why in CEN.KNOWN_COINCIDENCE.items():
            self.assertGreater(len(why.strip()), 20, f"{value}: {why!r}")


class GeneratorTiersTest(unittest.TestCase):
    """`derivable` splits by generator, and each value is credited once.

    The single coincidence rate this script used to print reached 31% as the
    corpora grew to 97 configurations. It was honest and it was one number over
    a set whose density is not uniform: `arm` quantities are sparse, `paired`
    quantities are not, and both were called "derivable".
    """

    @classmethod
    def setUpClass(cls):
        cls.tiers = CEN.derivable(CEN.load())

    def test_the_generators_are_disjoint(self):
        """Overlapping tiers would make the per-document counts sum to more
        than the numbers actually checked."""
        for a in CEN.TIERS:
            for b in CEN.TIERS:
                if a < b:
                    self.assertEqual(self.tiers[a] & self.tiers[b], set(),
                                     f"{a} and {b} overlap")

    def test_every_tier_is_populated(self):
        """A tier that is always empty is a tier that explains nothing, and its
        printed coincidence rate would read as reassurance."""
        for name in CEN.TIERS:
            self.assertTrue(self.tiers[name], f"{name} is empty")

    def test_the_arm_tier_is_sparser_than_the_paired_tier(self):
        """The point of splitting them. If this inverts, the ordering in TIERS
        is telling the reader the opposite of the truth."""
        self.assertLess(len(self.tiers["arm"]), len(self.tiers["paired"]))

    def test_explain_returns_the_strongest_generator(self):
        value = min(self.tiers["arm"])
        self.assertEqual(CEN.explain(value, self.tiers), "arm")
        self.assertIsNone(CEN.explain(-1.0, self.tiers))

    def test_a_paper_number_is_credited_to_exactly_one_generator(self):
        cells, elsewhere, traced, bad, _ = CEN.sweep_paper(
            self.tiers, {v for v, _ in CEN.ELSEWHERE})
        quoted = {m.group(1) for m in
                  CEN.NUMBER.finditer(CEN.PAPER.read_text())}
        self.assertEqual(sum(cells.values()) + elsewhere + traced + len(bad),
                         len(quoted))


class ADocumentWithNoNumbersTest(unittest.TestCase):
    """A sweep that finds nothing to check has not checked anything.

    `RESULT_2026-08-20_W4_RECURRENT_ARM_IS_UNUSABLE.md` is 93 lines long and
    quotes no four-decimal number. It printed `[ok  ]` — the same word as a
    document whose forty numbers were each recomputed from cells — for as long
    as this sweep has existed.
    """

    @classmethod
    def setUpClass(cls):
        cls.proc = subprocess.run(
            [sys.executable, str(ROOT / "scripts/check_every_number.py")],
            capture_output=True, text=True)

    def test_the_empty_document_is_marked_apart_from_a_pass(self):
        line = [l for l in self.proc.stdout.splitlines()
                if "W4_RECURRENT_ARM_IS_UNUSABLE" in l and l.startswith("  [")]
        self.assertEqual(len(line), 1, self.proc.stdout)
        self.assertIn("[none]", line[0])
        self.assertNotIn("[ok  ]", line[0])

    def test_it_is_excluded_from_the_closing_claim(self):
        """The closing claim must count exactly the documents it checked.

        This asserted "13 swept wave results" until 2026-08-28, when landing
        wave 20's result document made it 14 — a true change breaking a test
        that had pinned a number rather than the relationship. The invariant is
        that the claimed count equals the number of documents marked `[ok  ]`:
        one that could not be judged and one with nothing in it are both
        excluded, and neither may be counted as swept."""
        self.assertIn("carry no four-decimal number", self.proc.stdout)
        passed = len(re.findall(r"^  \[ok  \] RESULT_", self.proc.stdout, re.M))
        claimed = int(re.search(r"(\d+) swept wave results",
                                self.proc.stdout).group(1))
        self.assertEqual(claimed, passed,
                         f"claims {claimed} swept but marked {passed} ok")
        self.assertGreater(passed, 10)

    def test_the_document_really_has_no_numbers(self):
        """If it acquires one, this class is asserting a state that has moved
        and the [none] branch is no longer exercised by the real record."""
        text = (ROOT / "results/RESULT_2026-08-20_W4_RECURRENT_ARM_IS_UNUSABLE.md"
                ).read_text()
        self.assertEqual(list(CEN.NUMBER.finditer(text)), [])


class TheRunSaysWhichTierTest(unittest.TestCase):
    """The end-to-end run, on the real record."""

    @classmethod
    def setUpClass(cls):
        cls.proc = subprocess.run(
            [sys.executable, str(ROOT / "scripts/check_every_number.py")],
            capture_output=True, text=True)

    def test_the_sweep_passes_on_the_committed_record(self):
        self.assertEqual(self.proc.returncode, 0, self.proc.stdout[-3000:])

    def test_the_three_tiers_are_printed_separately(self):
        for line in ("tier A, derived from the cells",
                     "tier B, named in ELSEWHERE",
                     "tier C, traced to a named record"):
            self.assertIn(line, self.proc.stdout)

    def test_each_generator_prints_its_own_coincidence_rate(self):
        for name in CEN.TIERS:
            self.assertRegex(self.proc.stdout,
                             rf"{name}\s+\d+ quantities\s+a random 4dp value")

    def test_the_weaker_tier_says_it_is_weaker(self):
        """Three counts side by side read as three kinds of the same thing
        unless the output says otherwise."""
        self.assertIn("tier C is NOT derivation", self.proc.stdout)
        self.assertIn("weaker than tier A", self.proc.stdout)

    def test_the_manuscript_is_no_longer_announced_as_unswept(self):
        self.assertNotIn("NOT SWEPT", self.proc.stdout)
        self.assertIn("PAPER_DRAFT.md", self.proc.stdout)


class PlatformIsNotAnAxisArmsMayPairAcrossTest(unittest.TestCase):
    """Wave 29 ran on Apple libm; every other wave ran on the glibc fleet.

    `operating_point` drops the wave label on purpose, so that a wave can be
    compared against a control an earlier wave recorded. Without a platform in
    the key that same latitude lets a wave-29 arm pair with a fleet arm at the
    same width, budget, contract and geometry -- and the difference it yields is
    part gain and part libm. The sweep would then offer that as a value the
    cells can produce, which is how a number gets credited to cells that did not
    produce it.

    `AMENDMENT_2026-09-07_WAVE_29_RUNS_ON_THE_LOCAL_PLATFORM.md` section 8
    registered this hazard before the wave ran.
    """

    #: Real stems: the wave-29 rate arm and the fleet control it would pair
    #: with. Identical on every axis `operating_point` keeps.
    LOCAL = "w29asy__ff-fixed__h128__e400__published-2ms__adjacent-sum-5"
    FLEET = "w26pos__ff-fixed__h128__e400__published-2ms__adjacent-sum-5"

    def test_the_two_stems_agree_on_every_axis_but_the_platform(self):
        """Otherwise the test below would pass for the wrong reason."""
        local = CEN.operating_point(self.LOCAL)
        fleet = CEN.operating_point(self.FLEET)
        self.assertEqual(local[1:], fleet[1:],
                         "the stems must differ ONLY in platform, or this "
                         "class is asserting something weaker than it claims")

    def test_a_local_arm_and_a_fleet_arm_are_not_comparable(self):
        self.assertNotEqual(CEN.operating_point(self.LOCAL),
                            CEN.operating_point(self.FLEET))

    def test_two_fleet_arms_still_pair(self):
        """The fix must not close the reuse design it is narrowing."""
        other = "w28drp__ff-fixed__h128__e400__published-2ms__adjacent-sum-5"
        self.assertEqual(CEN.operating_point(self.FLEET),
                         CEN.operating_point(other))

    def test_every_non_fleet_wave_on_disk_is_declared(self):
        """A second off-fleet wave must not be able to land undeclared.

        The map is keyed by wave label and the corpus directory is the ground
        truth, so this walks the one directory that is known not to be fleet and
        asserts every wave label in it is spelled in `WAVE_PLATFORM`.
        """
        local_corpus = CEN.ROOT / "results/shd_attention_wave29_local"
        labels = {path.name.split("__")[0]
                  for path in local_corpus.glob("*__s*.json")}
        self.assertTrue(labels, "no cells found; this test would pass vacuously")
        for label in labels:
            self.assertIn(label, CEN.WAVE_PLATFORM,
                          f"{label} is in a non-fleet corpus but is not in "
                          f"WAVE_PLATFORM, so it pairs as though it were fleet")


class TheWave29CorpusIsActuallyReadTest(unittest.TestCase):
    """Its numbers must come from its own cells, not from cells like them.

    Before `shd_attention_wave29_local` was added to `CORPORA`, five of the six
    numbers in the wave-29 result matched anyway -- against fleet cells, with
    zero wave-29 configurations loaded. A corpus that is not read does not
    announce itself; it just quietly agrees with you.
    """

    def test_the_corpus_is_in_corpora(self):
        self.assertIn(CEN.ROOT / "results/shd_attention_wave29_local",
                      CEN.CORPORA)

    def test_wave29_configurations_are_loaded(self):
        groups = CEN.load()
        w29 = {stem for stem in groups if stem.startswith("w29")}
        self.assertEqual(len(w29), 4,
                         f"expected the wave's four arms, loaded {sorted(w29)}")
        for stem in w29:
            self.assertEqual(len(groups[stem]), 12,
                             f"{stem} should carry 12 seeds")

    def test_the_attention_intact_mean_comes_from_wave29_cells(self):
        """0.8261 is the one wave-29 number that did NOT collide with a fleet
        value, which is the only reason the missing corpus was noticed."""
        tiers = CEN.derivable(CEN.load())
        self.assertIn(0.8261, tiers["arm"])


if __name__ == "__main__":
    unittest.main(verbosity=2)
