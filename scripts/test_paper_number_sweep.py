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

        `sources` is the tier-C table; `extra` writes additional documents into
        the temporary tree so an entry can name a real file.
        """
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
                 mock.patch.object(CEN, "MIN_PAPER_NUMBERS", floor):
                return CEN.sweep_paper(set(known), set(allowed))

    # --- the clean case ----------------------------------------------------

    def test_each_tier_is_counted_separately(self):
        cells, elsewhere, traced, bad, complaints = self.sweep(
            "0.1111 and 0.2222 and 0.3333",
            [("0.3333", "results/record.md", "the arm")],
            {"results/record.md": "the arm scored 0.3333"})
        self.assertEqual((cells, elsewhere, traced), (1, 1, 1))
        self.assertEqual(bad, [])
        self.assertEqual(complaints, [])

    def test_a_cell_derived_number_is_not_double_counted_as_traced(self):
        """Tier A wins. A number the cells produce must not be reported at the
        weaker tier merely because the table also names it."""
        cells, _, traced, _, _ = self.sweep(
            "0.1111",
            [("0.1111", "results/record.md", "also here")],
            {"results/record.md": "0.1111"},
            floor=1)
        self.assertEqual((cells, traced), (1, 0))

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
                    set(), set())
        self.assertEqual((cells, elsewhere, traced, bad), (0, 0, 0, []))
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
