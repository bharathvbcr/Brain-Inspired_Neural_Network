#!/usr/bin/env python3
"""The supplement's redaction removes every identity it claims to, and the
sweep after it catches what the redaction misses.

The committed record names its author in run manifests (`/Users/<name>/`),
licences, the README's website link and the CI config. None of that shows in
the PDF, which is the only thing `build_paper.py --check` reads -- so a
supplement built straight from the repository would de-anonymise a
double-blind submission while every paper check stayed green.
"""
from __future__ import annotations

import importlib.util
import pathlib
import unittest

ROOT = pathlib.Path(__file__).resolve().parent.parent
spec = importlib.util.spec_from_file_location(
    "build_supplement", ROOT / "scripts/build_supplement.py")
bs = importlib.util.module_from_spec(spec)
spec.loader.exec_module(bs)


class RedactionTest(unittest.TestCase):

    CASES = [
        '"path": "/Users/bharath/Code/parameter_golf/binn/cells/a.json"',
        "Copyright (c) 2026 Bharath Chandra Vaddaram",
        '<a href="https://binn.vbcr.dev/">Explore</a>',
        "contact bharath.vbcr@gmail.com for data",
        "git clone https://github.com/bharathvbcr/Brain-Inspired_Neural_Network.git",
        "/private/tmp/claude-501/-Users-bharath-Code-parameter-golf/3ade",
        "Bharath's stated lean: ignore it",
        "**Owner:** Bharath · **Status:** closed",
    ]

    def test_every_known_form_is_redacted_and_leaves_nothing_the_sweep_finds(self):
        for case in self.CASES:
            out, n = bs.redact(case)
            self.assertGreater(n, 0, case)
            self.assertIsNone(bs.IDENTIFYING.search(out.encode()),
                              f"{case!r} -> {out!r} still identifies")

    def test_paths_stay_paths(self):
        """A manifest path must remain a well-formed path after redaction, or
        a script that reads it would fail in the reviewer's copy for a reason
        that has nothing to do with the science."""
        out, _ = bs.redact('"/Users/bharath/Code/x/cell.json"')
        self.assertEqual(out, '"/home/anon/Code/x/cell.json"')

    def test_text_without_an_identity_is_untouched(self):
        text = "gain +0.1275, 32/32 seeds; see RESULT_2026-09-09_W29.md\n"
        self.assertEqual(bs.redact(text), (text, 0))

    def test_the_sweep_is_case_insensitive_and_reads_bytes(self):
        """Binary files are copied without redaction, so the byte-level sweep
        is the only thing standing between them and the archive."""
        for needle in (b"BHARATH", b"Vbcr", b"vaddaram",
                       b"Brain-Inspired_Neural_Network"):
            self.assertIsNotNone(bs.IDENTIFYING.search(b"\x00\x01" + needle))

    def test_the_cap_is_tmlr_s(self):
        self.assertEqual(bs.SIZE_CAP, 100 * 1024 * 1024)

    def test_generated_output_and_ci_wiring_are_excluded(self):
        for path in ("paper/build/main.pdf", "paper/build-preprint/main.pdf",
                     ".github/workflows/ci.yml", ".devcouncil/repo_map.json"):
            self.assertIsNotNone(bs.EXCLUDE.match(path), path)
        self.assertIsNone(bs.EXCLUDE.match("results/PAPER_DRAFT.md"))


if __name__ == "__main__":
    unittest.main(verbosity=2)
