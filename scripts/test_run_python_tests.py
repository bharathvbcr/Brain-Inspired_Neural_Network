#!/usr/bin/env python3
"""run_python_tests.sh must tell "ran", "could not run" and "failed" apart.

Each case builds a throwaway repository root holding a copy of the runner and a
few fake `scripts/test_*.py` files, so the real corpus is never touched and the
runner discovering THIS file cannot recurse: the copy only sees the fakes.

Fake interpreters stand in for the optional-deps venv: `fake_venv(...)` writes
an executable `bin/python` that delegates to the real interpreter, with or
without a stub `h5py` on the path.
"""

from __future__ import annotations

import os
import shutil
import stat
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

RUNNER = Path(__file__).resolve().parent / "run_python_tests.sh"

NEEDS_H5PY = "import h5py\nimport unittest\n"
PASSING = (
    "import unittest\n"
    "class T(unittest.TestCase):\n"
    "    def test_a(self): self.assertTrue(True)\n"
    "    def test_b(self): self.assertTrue(True)\n"
    "unittest.main()\n"
)
H5PY_THEN_PASS = "import h5py\n" + PASSING
H5PY_THEN_FAIL = H5PY_THEN_PASS.replace("assertTrue(True)\n    def test_b", "assertTrue(False)\n    def test_b", 1)
H5PY_THEN_HANG = "import h5py\nimport time\ntime.sleep(60)\n"


class RunnerTest(unittest.TestCase):
    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp(prefix="rpt-"))
        self.addCleanup(shutil.rmtree, self.tmp, ignore_errors=True)
        (self.tmp / "scripts").mkdir()
        shutil.copy(RUNNER, self.tmp / "scripts" / "run_python_tests.sh")

    def fake_venv(self, name: str, with_h5py: bool = True) -> str:
        """An interpreter that can (or cannot) `import h5py`."""
        stubs = self.tmp / f"{name}-stubs"
        stubs.mkdir()
        if with_h5py:
            (stubs / "h5py.py").write_text("")
        binary = self.tmp / name / "bin" / "python"
        binary.parent.mkdir(parents=True)
        binary.write_text(f'#!/bin/sh\nPYTHONPATH="{stubs}" exec "{sys.executable}" "$@"\n')
        binary.chmod(binary.stat().st_mode | stat.S_IXUSR)
        return str(binary)

    def add(self, name: str, body: str) -> str:
        (self.tmp / "scripts" / name).write_text(body)
        return f"scripts/{name}"

    def run_gate(self, pinned: str, venvs: str, timeout: str = "30"):
        env = dict(os.environ)
        env.update(
            BINN_NEEDS_OPTIONAL_DEPS_ENV=pinned,
            BINN_VENV_PYTHONS=venvs,
            BINN_VENV_TIMEOUT=timeout,
        )
        # System python3 for the first pass is whatever `python3` resolves to;
        # pin it to an h5py-free one so the host's packages cannot leak in.
        bindir = self.tmp / "sysbin"
        bindir.mkdir(exist_ok=True)
        (bindir / "python3").write_text(f'#!/bin/sh\nexec "{sys.executable}" -I "$@"\n')
        (bindir / "python3").chmod(0o755)
        env["PATH"] = f"{bindir}:{env['PATH']}"
        return subprocess.run(
            ["bash", str(self.tmp / "scripts" / "run_python_tests.sh")],
            capture_output=True, text=True, env=env, timeout=120,
        )

    def test_h5py_test_runs_and_passes_in_the_venv(self):
        t = self.add("test_needs.py", H5PY_THEN_PASS)
        venv = self.fake_venv("venv")
        r = self.run_gate(t, venv)
        self.assertEqual(r.returncode, 0, r.stdout)
        self.assertRegex(r.stdout, r"\[ ok \] scripts/test_needs.py  \(Ran 2 tests; in venv\)")
        self.assertIn("passed: 1   failed: 0   could not run: 0", r.stdout)

    def test_no_venv_is_could_not_run_never_a_pass(self):
        t = self.add("test_needs.py", H5PY_THEN_PASS)
        r = self.run_gate(t, str(self.tmp / "absent" / "bin" / "python"))
        self.assertEqual(r.returncode, 0, r.stdout)
        self.assertIn("[SKIP] scripts/test_needs.py", r.stdout)
        self.assertIn("passed: 0   failed: 0   could not run: 1", r.stdout)

    def test_venv_without_the_dependency_stays_could_not_run(self):
        t = self.add("test_needs.py", H5PY_THEN_PASS)
        r = self.run_gate(t, self.fake_venv("bare", with_h5py=False))
        self.assertEqual(r.returncode, 0, r.stdout)
        self.assertIn("[SKIP]", r.stdout)
        self.assertIn("could not run: 1", r.stdout)

    def test_second_candidate_is_used_when_first_is_missing(self):
        t = self.add("test_needs.py", H5PY_THEN_PASS)
        venvs = f"{self.tmp}/nope/bin/python {self.fake_venv('second')}"
        r = self.run_gate(t, venvs)
        self.assertEqual(r.returncode, 0, r.stdout)
        self.assertIn("in second)", r.stdout)

    def test_failing_test_inside_the_venv_fails_the_gate(self):
        t = self.add("test_needs.py", H5PY_THEN_FAIL)
        r = self.run_gate(t, self.fake_venv("venv"))
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn("[FAIL] scripts/test_needs.py", r.stdout)
        self.assertIn("FAILED: scripts/test_needs.py", r.stdout)

    def test_hung_venv_test_is_bounded_and_fails(self):
        t = self.add("test_needs.py", H5PY_THEN_HANG)
        r = self.run_gate(t, self.fake_venv("venv"), timeout="2")
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn("[FAIL] scripts/test_needs.py", r.stdout)

    def test_pinned_test_that_passes_under_system_python_is_stale(self):
        t = self.add("test_plain.py", PASSING)
        r = self.run_gate(t, self.fake_venv("venv"))
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn(f"STALE: {t}", r.stdout)

    def test_unpinned_optional_dep_test_is_unexpected_even_if_venv_passes_it(self):
        self.add("test_needs.py", H5PY_THEN_PASS)
        r = self.run_gate("", self.fake_venv("venv"))
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn("UNEXPECTED: scripts/test_needs.py", r.stdout)

    def test_non_optional_import_error_is_a_failure(self):
        t = self.add("test_broken.py", "import not_an_optional_dep\n")
        r = self.run_gate(t, self.fake_venv("venv"))
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn("[FAIL] scripts/test_broken.py", r.stdout)


if __name__ == "__main__":
    unittest.main(verbosity=2)
