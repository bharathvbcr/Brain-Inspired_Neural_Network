#!/usr/bin/env python3
"""The terminology gate still fails on a genuinely bare use.

Widening a qualifier list until the corpus passes is how a check becomes
decoration. Each test here injects a sentence that WOULD widen the lead
matched FAIL and asserts the gate catches it.
"""
from __future__ import annotations

import importlib.util
import pathlib
import shutil
import subprocess
import sys
import tempfile
import unittest

ROOT = pathlib.Path(__file__).resolve().parent.parent
GATE = ROOT / "scripts/check_terminology.py"

spec = importlib.util.spec_from_file_location("check_terminology", GATE)
ct = importlib.util.module_from_spec(spec)
spec.loader.exec_module(ct)


class TerminologyTest(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = pathlib.Path(tempfile.mkdtemp())
        self.addCleanup(shutil.rmtree, self.tmp, ignore_errors=True)
        (self.tmp / "results").mkdir()
        (self.tmp / "scripts").mkdir()
        self.docs = []
        for doc in ct.DOCS:
            rel = f"results/{doc.name}"
            shutil.copy(doc, self.tmp / rel)
            self.docs.append(self.tmp / rel)
        shutil.copy(GATE, self.tmp / "scripts/check_terminology.py")

    def run_gate(self) -> subprocess.CompletedProcess:
        return subprocess.run(
            [sys.executable, str(self.tmp / "scripts/check_terminology.py")],
            capture_output=True, text=True)

    def append(self, text: str) -> None:
        target = self.docs[0]
        target.write_text(target.read_text() + "\n\n" + text + "\n")

    def test_the_real_corpus_passes(self):
        proc = self.run_gate()
        self.assertEqual(proc.returncode, 0, proc.stdout)
        self.assertIn("every one qualified", proc.stdout)

    def test_the_widest_misreading_is_caught(self):
        """The sentence this whole requirement exists to prevent."""
        self.append("We conclude that broadcast plasticity cannot recover "
                    "the reference on this task, and that this is a property "
                    "of the topology itself rather than of one update.")
        proc = self.run_gate()
        self.assertEqual(proc.returncode, 1, proc.stdout)
        self.assertIn("no qualifier", proc.stdout)

    def test_a_bare_use_beside_the_lead_fail_is_caught(self):
        self.append("The lead negative is that broadcast fails the gate.")
        proc = self.run_gate()
        self.assertEqual(proc.returncode, 1, proc.stdout)

    def test_a_qualified_use_is_accepted(self):
        self.append("Broadcast ±1 three-factor fails the matched gate.")
        self.assertEqual(self.run_gate().returncode, 0)

    def test_a_missing_document_is_a_failure_not_a_pass(self):
        self.docs[1].unlink()
        proc = self.run_gate()
        self.assertEqual(proc.returncode, 1, proc.stdout)
        self.assertIn("must not read as a passing one", proc.stdout)

    def test_the_gate_reads_every_document_that_records_the_requirement(self):
        names = {d.name for d in ct.DOCS}
        self.assertIn("PAPER_DRAFT.md", names)
        self.assertIn("PAPER_FIGURE_SPEC.md", names)
        self.assertIn("PUBLISHABLE_CLAIMS.md", names)
        self.assertIn("VENUE_FORMATTING.md", names)

    def test_the_qualifier_window_is_not_unbounded(self):
        """A window wide enough to always find a qualifier somewhere is a
        gate that passes everything."""
        self.assertLessEqual(ct.WINDOW, 120)

    def test_every_recorded_exception_still_appears_somewhere(self):
        """A DELIBERATE entry whose phrase has left the corpus is an
        exemption with nothing behind it, and the next bare use inherits it."""
        corpus = "\n".join(d.read_text() for d in self.docs)
        for phrase in ct.DELIBERATE:
            self.assertIn(phrase, corpus,
                          f"{phrase!r} is exempted but no longer written")


if __name__ == "__main__":
    unittest.main(verbosity=2)
