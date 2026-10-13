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
        self.original = self.docs[0].read_text()

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

    def verdict(self, sentence: str) -> int:
        """Exit code with exactly `sentence` appended to the real corpus."""
        target = self.docs[0]
        target.write_text(self.original + "\n\n" + sentence + "\n")
        return self.run_gate().returncode

    def test_the_qualifier_must_bind_to_the_term(self):
        """A qualifier a few words away in the same sentence binds; one that
        merely sits nearby does not. This replaced a +/-80 character window,
        which any generic word (`error`, `scalar`, `RL`) cleared."""
        self.assertLessEqual(ct.BIND_WORDS, 3)

    def test_a_generic_word_nearby_does_not_clear_it(self):
        """`error` in `error bars` is not the arm `broadcast error`."""
        self.assertEqual(self.verdict(
            "Broadcast credit fails to learn the task, shown by the error "
            "bars."), 1)

    def test_generic_words_within_three_words_do_not_clear_it(self):
        """Each of these passed when `error`, `xor` and `scalar` counted
        anywhere within three words; none names the rule."""
        for bare in ("Broadcast fails with error bars overlapping chance.",
                     "Broadcast fails (standard error 0.02).",
                     "Broadcast fails, as XOR shows nothing.",
                     "`broadcast` fails the gate everywhere.",
                     "Broadcast fails on a scalar summary of the run.",
                     "Broadcast error bars overlap chance."):
            with self.subTest(bare=bare):
                self.assertEqual(self.verdict(bare), 1, bare)

    def test_a_code_span_is_exempt_only_as_a_mention_of_the_word(self):
        self.assertEqual(self.verdict("The word `broadcast` drifted in."), 0)
        self.assertEqual(self.verdict("`broadcast` fails the gate."), 1)

    def test_generic_compounds_and_task_labels_are_accepted(self):
        for ok in ("A scalar broadcast reaches chance.",
                   "Every broadcast scalar fails coincidence.",
                   "The RL broadcast arm reaches chance.",
                   "NumPy depth_locality mid: broadcast 0.816 / DFA 0.9.",
                   "- NumPy xor_thresh: broadcast 0.501 / DFA 0.99"):
            with self.subTest(ok=ok):
                self.assertEqual(self.verdict(ok), 0, ok)

    def test_a_qualifier_in_the_next_sentence_does_not_clear_it(self):
        self.assertEqual(self.verdict(
            "We find that broadcast fails. Graded updates succeed."), 1)

    def test_a_qualifier_just_past_the_old_window_does_not_clear_it(self):
        pad = " ".join(["word"] * 17)  # 85 characters, no sentence break
        self.assertEqual(self.verdict(f"Broadcast fails {pad} three-factor."), 1)

    def test_inflections_are_checked(self):
        for bare in ("Broadcasting fails the gate.",
                     "Broadcasts fail the gate.",
                     "The broadcast-based rule fails the gate."):
            with self.subTest(bare=bare):
                self.assertEqual(self.verdict(bare), 1)

    def test_a_code_span_mention_of_the_word_is_not_a_use(self):
        self.assertEqual(self.verdict("A bare `broadcast` drifted in."), 0)

    def test_the_rule_definition_qualifies(self):
        self.assertEqual(self.verdict(
            "It is ±1 × surrogate eligibility under broadcast."), 0)

    def test_qualified_forms_are_accepted(self):
        for ok in ("Broadcast-graded reaches 0.9975.",
                   "RL ±1 broadcast REINFORCE reaches 0.78.",
                   "Broadcasting ±1 three-factor credit fails.",
                   "The two RL broadcast contrasts were measured.",
                   "On `xor_thresh`, broadcast error stays at chance."):
            with self.subTest(ok=ok):
                self.assertEqual(self.verdict(ok), 0, ok)

    def test_every_recorded_exception_still_appears_somewhere(self):
        """A DELIBERATE entry whose phrase has left the corpus is an
        exemption with nothing behind it, and the next bare use inherits it."""
        corpus = "\n".join(d.read_text() for d in self.docs)
        for phrase in ct.DELIBERATE:
            self.assertIn(phrase, corpus,
                          f"{phrase!r} is exempted but no longer written")


if __name__ == "__main__":
    unittest.main(verbosity=2)
