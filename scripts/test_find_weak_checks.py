#!/usr/bin/env python3
"""`find_weak_checks.py --strict` fails on a NEW weak check, and says so when a
baselined one has gone.

The scanner exited 0 on any number of findings, and `record_checks.sh` files it
under "checks that cannot fail" -- so a new test that passes for a degenerate
result was reported as a clean run. Strict mode compares against a committed
baseline of the findings that exist and have been read. Each test here builds a
throwaway Rust tree and asserts the gate's exit code.
"""
from __future__ import annotations

import pathlib
import shutil
import subprocess
import sys
import tempfile
import unittest

ROOT = pathlib.Path(__file__).resolve().parent.parent
TOOL = ROOT / "scripts/find_weak_checks.py"

WEAK_TEST = """
#[test]
fn {name}() {{
    let r = thing();
    assert!(r.is_finite());
}}
"""
STRONG_TEST = """
#[test]
fn {name}() {{
    assert_eq!(thing(), 3);
}}
"""


class StrictModeTest(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = pathlib.Path(tempfile.mkdtemp())
        self.addCleanup(shutil.rmtree, self.tmp, ignore_errors=True)
        self.src = self.tmp / "crate/src"
        self.src.mkdir(parents=True)
        self.baseline = self.tmp / "baseline.txt"

    def write(self, name: str, body: str, rel: str = "lib.rs") -> None:
        (self.src / rel).parent.mkdir(parents=True, exist_ok=True)
        path = self.src / rel
        path.write_text(path.read_text() + body.format(name=name)
                        if path.exists() else body.format(name=name))

    def run_tool(self, *flags: str) -> subprocess.CompletedProcess:
        return subprocess.run(
            [sys.executable, str(TOOL), "--root", str(self.tmp),
             "--baseline", str(self.baseline), *flags],
            capture_output=True, text=True)

    def test_default_mode_still_exits_zero_on_findings(self):
        self.write("weak_one", WEAK_TEST)
        self.assertEqual(self.run_tool().returncode, 0)

    def test_a_new_weak_check_fails_strict(self):
        self.write("weak_one", WEAK_TEST)
        self.assertEqual(self.run_tool("--write-baseline").returncode, 0)
        self.assertEqual(self.run_tool("--strict").returncode, 0)
        self.write("weak_two", WEAK_TEST)
        proc = self.run_tool("--strict")
        self.assertEqual(proc.returncode, 1, proc.stdout + proc.stderr)
        self.assertIn("weak_two", proc.stdout)
        self.assertNotIn("weak_one", proc.stdout.split("NEW")[-1])

    def test_a_baselined_finding_that_is_gone_is_reported_not_ignored(self):
        self.write("weak_one", WEAK_TEST)
        self.run_tool("--write-baseline")
        (self.src / "lib.rs").write_text(STRONG_TEST.format(name="weak_one"))
        proc = self.run_tool("--strict")
        self.assertEqual(proc.returncode, 1, proc.stdout + proc.stderr)
        self.assertIn("STALE", proc.stdout)
        self.assertIn("weak_one", proc.stdout)

    def test_line_numbers_do_not_destabilise_the_baseline(self):
        self.write("weak_one", WEAK_TEST)
        self.run_tool("--write-baseline")
        lib = self.src / "lib.rs"
        lib.write_text("// moved down\n" * 40 + lib.read_text())
        self.assertEqual(self.run_tool("--strict").returncode, 0)

    def test_the_baseline_is_sorted_and_deterministic(self):
        for n in ("zeta", "alpha", "mid"):
            self.write(n, WEAK_TEST)
        self.run_tool("--write-baseline")
        first = self.baseline.read_text()
        self.run_tool("--write-baseline")
        self.assertEqual(first, self.baseline.read_text())
        rows = [l for l in first.splitlines() if l and not l.startswith("#")]
        self.assertEqual(rows, sorted(rows))
        self.assertEqual(len(rows), 3)

    def test_a_missing_baseline_is_a_failure_not_a_pass(self):
        self.write("weak_one", WEAK_TEST)
        proc = self.run_tool("--strict")
        self.assertEqual(proc.returncode, 1, proc.stdout + proc.stderr)
        self.assertIn("baseline", (proc.stdout + proc.stderr).lower())

    def test_a_new_tautology_in_a_baselined_test_is_caught(self):
        """The test is already baselined for one assertion; a second tautology
        in it must not ride on that entry."""
        body = ("#[test]\nfn t() {\n    let m = thing();\n"
                "    assert_eq!(m.f(1), m.f(2));\n}\n")
        (self.src / "lib.rs").write_text(body)
        self.run_tool("--write-baseline")
        (self.src / "lib.rs").write_text(body.replace(
            "}\n", "    assert_eq!(m.g(3), m.g(4));\n}\n"))
        self.assertEqual(self.run_tool("--strict").returncode, 1)

    def test_generated_copies_are_not_scanned(self):
        """`.tmp/` and `paper/build/` hold copies of the tree; scanning them
        multiplies every finding and ties the baseline to scratch state."""
        for rel in (".tmp/x/src/lib.rs", "paper/build/x/src/lib.rs"):
            self.write("copy", WEAK_TEST, rel=f"../../{rel}")
        proc = self.run_tool()
        self.assertIn("0 test(s) whose assertions", proc.stdout)

    def test_an_empty_tree_with_an_empty_baseline_passes_strict(self):
        """Calibration (exit 2) runs before the baseline comparison, so strict
        cannot swallow it; that path is exercised by the tool's own fixture."""
        self.baseline.write_text("")
        self.assertEqual(self.run_tool("--strict").returncode, 0)


if __name__ == "__main__":
    unittest.main(verbosity=2)
