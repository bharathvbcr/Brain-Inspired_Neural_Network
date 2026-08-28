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
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "scripts"))

import check_every_number as CEN  # noqa: E402


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
        self.assertIn("carry no four-decimal number", self.proc.stdout)
        self.assertIn("13 swept wave results", self.proc.stdout)

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


if __name__ == "__main__":
    unittest.main(verbosity=2)
