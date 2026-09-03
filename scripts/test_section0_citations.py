#!/usr/bin/env python3
"""The §0 citation gate fails on each thing it claims to catch.

Every test here builds a corpus in which exactly one property is broken and
asserts the gate reports it. A gate whose failure branches are never executed
is a gate that has only ever been observed to pass, which is the defect class
this repository keeps finding in its own tooling.
"""
from __future__ import annotations

import pathlib
import shutil
import subprocess
import sys
import tempfile
import unittest

ROOT = pathlib.Path(__file__).resolve().parent.parent
GATE = ROOT / "scripts/check_section0_citations.py"
DRAFT = "results/PAPER_DRAFT.md"
RECORD = "results/CITATIONS_2026-09-03_SECTION_0_AGAINST_PRIMARIES.md"
BIB = "results/references.bib"


class GateTest(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = pathlib.Path(tempfile.mkdtemp())
        self.addCleanup(shutil.rmtree, self.tmp, ignore_errors=True)
        (self.tmp / "results").mkdir()
        (self.tmp / "scripts").mkdir()
        for rel in (DRAFT, RECORD, BIB):
            shutil.copy(ROOT / rel, self.tmp / rel)
        (self.tmp / "binn-lab/src").mkdir(parents=True)
        shutil.copy(ROOT / "binn-lab/src/paper_figures.rs",
                    self.tmp / "binn-lab/src/paper_figures.rs")
        shutil.copy(GATE, self.tmp / "scripts/check_section0_citations.py")

    def run_gate(self) -> subprocess.CompletedProcess:
        return subprocess.run(
            [sys.executable, str(self.tmp / "scripts/check_section0_citations.py")],
            capture_output=True, text=True)

    def edit(self, rel: str, old: str, new: str) -> None:
        path = self.tmp / rel
        text = path.read_text()
        self.assertIn(old, text, f"fixture text not found in {rel}")
        path.write_text(text.replace(old, new, 1))

    # -- the copied corpus passes, or none of the rest means anything --------
    def test_the_real_record_passes(self):
        proc = self.run_gate()
        self.assertEqual(proc.returncode, 0, proc.stdout + proc.stderr)
        self.assertIn("every one covered by", proc.stdout)

    # -- 1. a new, unchecked number in §0 -----------------------------------
    def test_an_uncited_percentage_is_refused(self):
        self.edit(DRAFT, "This instrument's\n0.8332 is not in that band",
                  "A further model reports 97.42% on SHD. This instrument's\n"
                  "0.8332 is not in that band")
        proc = self.run_gate()
        self.assertEqual(proc.returncode, 1, proc.stdout)
        self.assertIn("97.42", proc.stdout)
        self.assertIn("does not mention it", proc.stdout)

    def test_an_uncited_percentage_in_the_abstract_is_refused(self):
        """The abstract was carrying a frontier band §0 had already had
        corrected. A gate scoped to §0 alone reported a pass over it."""
        self.edit(DRAFT, "That much\nis unsurprising.",
                  "The field reaches 97.42%. That much\nis unsurprising.")
        proc = self.run_gate()
        self.assertEqual(proc.returncode, 1, proc.stdout)
        self.assertIn("97.42", proc.stdout)

    def test_the_instruments_own_four_decimal_values_are_not_swept(self):
        """`check_every_number.py` owns those, against cells. Sweeping them
        here would report every one of them as an uncited citation."""
        proc = self.run_gate()
        self.assertEqual(proc.returncode, 0, proc.stdout)
        self.assertNotIn("0.8332", proc.stdout)
        self.assertNotIn("0.1275", proc.stdout)

    def test_artwork_drifting_from_the_record_is_refused(self):
        rust = self.tmp / "binn-lab/src/paper_figures.rs"
        rust.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy(ROOT / "binn-lab/src/paper_figures.rs", rust)
        rust.write_text(rust.read_text().replace(
            "FIELD_FRONTIER_HI: f64 = 0.963;", "FIELD_FRONTIER_HI: f64 = 0.964;"))
        proc = self.run_gate()
        self.assertEqual(proc.returncode, 1, proc.stdout)
        self.assertIn("diverged", proc.stdout)

    # -- 2. the corrections staying corrected -------------------------------
    def test_the_reverted_frontier_bound_is_refused(self):
        self.edit(DRAFT, "The SHD frontier runs", "The SHD frontier is 96.4, and runs")
        proc = self.run_gate()
        self.assertEqual(proc.returncode, 1, proc.stdout)
        self.assertIn("96.4", proc.stdout)

    def test_the_reverted_attribution_is_refused(self):
        self.edit(DRAFT, "Adaptation reaches", "Spiking transformers also appear. Adaptation reaches")
        proc = self.run_gate()
        self.assertEqual(proc.returncode, 1, proc.stdout)
        self.assertIn("no citation behind it", proc.stdout)

    # -- 3. record and bibliography drifting apart --------------------------
    def test_a_source_missing_from_the_bibliography_is_refused(self):
        self.edit(BIB, "@article{baronig2025,", "@article{baronig2025DIFFERENT,")
        proc = self.run_gate()
        self.assertEqual(proc.returncode, 1, proc.stdout)
        self.assertIn("baronig2025", proc.stdout)
        self.assertIn("drifted", proc.stdout)

    def test_a_shrunken_record_is_refused(self):
        """The floor stops the gate passing by checking fewer sources."""
        text = (self.tmp / RECORD).read_text()
        for key in ("okabe2018", "soydan2024s7", "chen2025nsa"):
            text = text.replace(f"| `{key}` |", f"| {key} |")
        (self.tmp / RECORD).write_text(text)
        proc = self.run_gate()
        self.assertEqual(proc.returncode, 1, proc.stdout)
        self.assertIn("hashed sources", proc.stdout)

    # -- 4. the disclosure pointing at the record ---------------------------
    def test_a_section_zero_that_drops_the_record_link_is_refused(self):
        self.edit(DRAFT,
                  "[`CITATIONS_2026-09-03_SECTION_0_AGAINST_PRIMARIES.md`]"
                  "(CITATIONS_2026-09-03_SECTION_0_AGAINST_PRIMARIES.md)",
                  "an internal note")
        proc = self.run_gate()
        self.assertEqual(proc.returncode, 1, proc.stdout)
        self.assertIn("no longer links", proc.stdout)

    # -- 5. "could not run" must not look like "passed" ---------------------
    def test_a_missing_record_is_a_failure_not_a_pass(self):
        (self.tmp / RECORD).unlink()
        proc = self.run_gate()
        self.assertEqual(proc.returncode, 1, proc.stdout)
        self.assertIn("not a gate", proc.stdout)

    def test_a_draft_without_the_region_start_is_a_failure_not_a_pass(self):
        path = self.tmp / DRAFT
        path.write_text(path.read_text().replace("## Abstract", "## Summary"))
        proc = self.run_gate()
        self.assertEqual(proc.returncode, 1, proc.stdout + proc.stderr)
        self.assertIn("could not find", proc.stdout + proc.stderr)

    def test_a_draft_without_the_end_marker_does_not_sweep_the_whole_paper(self):
        """Otherwise §0's gate would silently start policing §1-§5, where
        every number is machine-checked by a different gate, and the
        instrument's own displacement figures (109.2, 145.2 bins) would be
        reported here as uncited citations."""
        self.edit(DRAFT, "## 2. Methods", "## 2. How it was done")
        proc = self.run_gate()
        self.assertEqual(proc.returncode, 1, proc.stdout + proc.stderr)
        self.assertIn("refusing to sweep", proc.stdout + proc.stderr)


if __name__ == "__main__":
    unittest.main(verbosity=2)
