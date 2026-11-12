#!/usr/bin/env python3
"""run_python_tests.sh must tell "ran", "could not run" and "failed" apart.

Each case builds a throwaway repository root holding a copy of the runner, the
real requirements-shd-calibration.txt and a few fake `scripts/test_*.py` files,
so the real corpus is never touched and the runner discovering THIS file cannot
recurse: the copy only sees the fakes.

Fake interpreters stand in for the optional-deps venv: `fake_venv(...)` writes
an executable `bin/python` that delegates to the real interpreter, with a stub
module on the path for each dependency it is meant to "have".

`BINN_RUNNER_UNDER_TEST` points the harness at another copy of the runner (e.g.
`git show HEAD:scripts/run_python_tests.sh`) to see which cases it fails.
"""

from __future__ import annotations

import os
import shutil
import stat
import subprocess
import sys
import tempfile
import textwrap
import time
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
RUNNER = Path(os.environ.get("BINN_RUNNER_UNDER_TEST", HERE / "run_python_tests.sh"))
RECORD_CHECKS = Path(os.environ.get("BINN_RECORD_CHECKS_UNDER_TEST", HERE / "record_checks.sh"))
REQUIREMENTS = HERE.parent / "requirements-shd-calibration.txt"

PASSING = (
    "import unittest\n"
    "class T(unittest.TestCase):\n"
    "    def test_a(self): self.assertTrue(True)\n"
    "    def test_b(self): self.assertTrue(True)\n"
    "unittest.main()\n"
)
ONE_PASSING = PASSING.replace("    def test_b(self): self.assertTrue(True)\n", "")
H5PY_THEN_PASS = "import h5py\n" + PASSING
H5PY_THEN_FAIL = H5PY_THEN_PASS.replace("assertTrue(True)\n    def test_b", "assertTrue(False)\n    def test_b", 1)
H5PY_THEN_HANG = "import h5py\nimport time\ntime.sleep(60)\n"
# Exits 0 having run nothing: a "pass" with no evidence behind it.
H5PY_THEN_NOTHING = "import h5py\n"
H5PY_THEN_ALL_SKIPPED = (
    "import h5py, unittest\n"
    "class T(unittest.TestCase):\n"
    "    @unittest.skip('x')\n"
    "    def test_a(self): pass\n"
    "unittest.main()\n"
)
# Two tests, one skipped: ran and passed, but not everything ran.
PARTIAL_SKIP = (
    "import unittest\n"
    "class T(unittest.TestCase):\n"
    "    def test_a(self): self.assertTrue(True)\n"
    "    @unittest.skip('x')\n"
    "    def test_b(self): pass\n"
    "unittest.main()\n"
)
H5PY_THEN_PARTIAL_SKIP = "import h5py\n" + PARTIAL_SKIP
# Passes, but prints to stdout. Redirected to a file, stdout is block-buffered
# and flushes at exit, after unittest's summary on stderr, so the summary is
# not among the last lines of the output (scripts/test_paper_build.py does this).
NOISY_AFTER_SUMMARY = PASSING.replace(
    "    def test_b(self): self.assertTrue(True)\n",
    "    def test_b(self): print('\\n'.join(f'FAIL: fixture text {i}' for i in range(30)))\n")
H5PY_THEN_NOISY_AFTER_SUMMARY = "import h5py\n" + NOISY_AFTER_SUMMARY
# A real failure that still exits 0: `unittest.main(exit=False)`.
FAILED_EXIT_ZERO = PASSING.replace(
    "    def test_b(self): self.assertTrue(True)\n",
    "    def test_b(self): self.assertTrue(False)\n").replace(
    "unittest.main()\n", "unittest.main(exit=False)\n")
ALL_SKIPPED = H5PY_THEN_ALL_SKIPPED.replace("import h5py, unittest", "import unittest")
# Imports cleanly, defines a TestCase with no tests: "Ran 0 tests", exit 5 on
# 3.12+, exit 0 on older interpreters; either way not a pass.
ZERO_TESTS = "import unittest\nclass T(unittest.TestCase): pass\nunittest.main()\n"
# Every test skipped, but stdout carries a line shaped like a summary. Buffered,
# it lands after the real one and becomes the "last summary".
FAKE_SUMMARY_ALL_SKIPPED = ALL_SKIPPED.replace(
    "unittest.main()\n", "print('Ran 3 tests in 0.010s')\nunittest.main()\n")
# A real zero-test run that still exits 0: `unittest.main(exit=False)` prints
# "NO TESTS RAN" and returns. Python 3.12's own exit 5 is what hid this case.
ZERO_TESTS_EXIT_ZERO = ZERO_TESTS.replace("unittest.main()\n", "unittest.main(exit=False)\n")
# Every test skipped, then an atexit hook writes a clean-looking summary AFTER
# unittest's own, so it is the last text in the file whatever the buffering.
ATEXIT_SUMMARY_ALL_SKIPPED = (
    "import atexit\n"
    "atexit.register(lambda: print('Ran 3 tests in 0.010s\\n\\nOK', flush=True))\n" + ALL_SKIPPED)
# A printed summary-shaped line, block-buffered without PYTHONUNBUFFERED: it
# flushes at exit, after unittest's real summary, and would be reported as it.
PRINTED_SUMMARY_PASSING = PASSING.replace(
    "unittest.main()\n", "print('Ran 5 tests in 0.010s\\n\\nOK')\nunittest.main()\n")
# `FAILED` and then a stray `OK` line: only the FAILED branch of the verdict
# scan rejects it; without that branch the `OK` after it would be the verdict.
FAILED_THEN_STRAY_OK = FAILED_EXIT_ZERO + "print('OK')\n"
# A real failure whose output merely CONTAINS the words of a missing import.
H5PY_THEN_FAIL_QUOTING_IMPORT_ERROR = (
    "import h5py\n"
    "print(\"warning: No module named 'h5py' in a subprocess\")\n"
    "raise SystemExit(1)\n"
)
# Ignores SIGALRM and leaves a child holding stdout and the process group.
H5PY_THEN_IGNORE_SIGALRM = (
    "import h5py, os, signal, subprocess, time\n"
    "signal.signal(signal.SIGALRM, signal.SIG_IGN)\n"
    "child = subprocess.Popen(['sleep', '40'])\n"
    "open(os.environ['CHILD_PID_FILE'], 'w').write(str(child.pid))\n"
    "time.sleep(40)\n"
)


def alive(pid: int) -> bool:
    try:
        os.kill(pid, 0)
    except ProcessLookupError:
        return False
    return True


class RunnerBase(unittest.TestCase):
    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp(prefix="rpt-"))
        self.addCleanup(shutil.rmtree, self.tmp, ignore_errors=True)
        self.root = self.make_root(self.tmp / "repo")

    def make_root(self, path: Path) -> Path:
        (path / "scripts").mkdir(parents=True)
        shutil.copy(RUNNER, path / "scripts" / "run_python_tests.sh")
        shutil.copy(REQUIREMENTS, path / "requirements-shd-calibration.txt")
        return path

    def fake_venv(self, name: str, with_h5py: bool = True, mods: tuple[str, ...] | None = None) -> str:
        """An interpreter that can import exactly the modules in `mods`."""
        if mods is None:
            mods = ("h5py",) if with_h5py else ()
        stubs = self.tmp / f"{name}-stubs"
        stubs.mkdir()
        for mod in mods:
            (stubs / f"{mod}.py").write_text("")
        binary = self.tmp / name / "bin" / "python"
        binary.parent.mkdir(parents=True)
        binary.write_text(f'#!/bin/sh\nPYTHONPATH="{stubs}" exec "{sys.executable}" "$@"\n')
        binary.chmod(binary.stat().st_mode | stat.S_IXUSR)
        return str(binary)

    def add(self, name: str, body: str) -> str:
        (self.root / "scripts" / name).write_text(body)
        return f"scripts/{name}"

    def fake_python3_bin(self) -> Path:
        """A `python3` on PATH that is the host interpreter minus site packages."""
        bindir = self.tmp / "sysbin"
        bindir.mkdir(exist_ok=True)
        (bindir / "python3").write_text(f'#!/bin/sh\nexec "{sys.executable}" -I "$@"\n')
        (bindir / "python3").chmod(0o755)
        return bindir

    def run_gate(self, pinned: str, venvs: str, timeout: str = "30", extra_env: dict | None = None):
        env = dict(os.environ)
        env.update(
            BINN_NEEDS_OPTIONAL_DEPS_ENV=pinned,
            BINN_VENV_PYTHONS=venvs,
            BINN_VENV_TIMEOUT=timeout,
        )
        env.pop("BINN_PYTHON", None)
        env.update(extra_env or {})
        # System python3 for the first pass is whatever `python3` resolves to;
        # pin it to an h5py-free one so the host's packages cannot leak in.
        env["PATH"] = f"{self.fake_python3_bin()}:{env['PATH']}"
        return subprocess.run(
            ["bash", str(self.root / "scripts" / "run_python_tests.sh")],
            capture_output=True, text=True, env=env, timeout=120,
        )


class RunnerTest(RunnerBase):
    def test_h5py_test_runs_and_passes_in_the_venv(self):
        t = self.add("test_needs.py", H5PY_THEN_PASS)
        r = self.run_gate(t, self.fake_venv("venv"))
        self.assertEqual(r.returncode, 0, r.stdout)
        self.assertRegex(r.stdout, r"\[ ok \] scripts/test_needs.py  \(Ran 2 tests; in venv\)")
        self.assertIn("passed: 1   failed: 0   could not run: 0", r.stdout)
        self.assertIn("All discovered python tests executed and passed.", r.stdout)

    def test_no_venv_is_could_not_run_never_a_pass(self):
        t = self.add("test_needs.py", H5PY_THEN_PASS)
        r = self.run_gate(t, str(self.tmp / "absent" / "bin" / "python"))
        # A distinct status: not 0 (that says "passed"), not 1 (that fails the
        # gate on a pin that is expected to be unrunnable here).
        self.assertEqual(r.returncode, 3, r.stdout)
        self.assertIn("[SKIP] scripts/test_needs.py", r.stdout)
        self.assertIn("passed: 0   failed: 0   could not run: 1", r.stdout)
        self.assertIn("COULD NOT RUN", r.stdout)
        self.assertNotIn("executed and passed", r.stdout)

    def test_venv_without_the_dependency_stays_could_not_run(self):
        t = self.add("test_needs.py", H5PY_THEN_PASS)
        r = self.run_gate(t, self.fake_venv("bare", with_h5py=False))
        self.assertEqual(r.returncode, 3, r.stdout)
        self.assertIn("[SKIP]", r.stdout)
        self.assertIn("could not run: 1", r.stdout)

    def test_second_candidate_is_used_when_first_is_missing(self):
        t = self.add("test_needs.py", H5PY_THEN_PASS)
        venvs = f"{self.tmp}/nope/bin/python\n{self.fake_venv('second')}"
        r = self.run_gate(t, venvs)
        self.assertEqual(r.returncode, 0, r.stdout)
        self.assertIn("in second)", r.stdout)

    def test_next_candidate_is_tried_when_the_first_lacks_the_dependency(self):
        t = self.add("test_needs.py", H5PY_THEN_PASS)
        venvs = f"{self.fake_venv('bare', with_h5py=False)}\n{self.fake_venv('good')}"
        r = self.run_gate(t, venvs)
        self.assertEqual(r.returncode, 0, r.stdout)
        self.assertIn("in good)", r.stdout)

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

    def test_test_ignoring_sigalrm_and_leaving_a_child_is_killed(self):
        t = self.add("test_needs.py", H5PY_THEN_IGNORE_SIGALRM)
        pid_file = self.tmp / "child.pid"
        started = time.monotonic()
        r = self.run_gate(t, self.fake_venv("venv"), timeout="2",
                          extra_env={"CHILD_PID_FILE": str(pid_file)})
        self.assertLess(time.monotonic() - started, 20, "the deadline was not enforced")
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn("[FAIL] scripts/test_needs.py", r.stdout)
        self.assertIn("timed out", r.stdout)
        child = int(pid_file.read_text())
        for _ in range(20):
            if not alive(child):
                break
            time.sleep(0.1)
        self.assertFalse(alive(child), "the test's child outlived the gate")

    def test_system_python_run_is_bounded_too(self):
        t = self.add("test_slow.py", "import time\ntime.sleep(40)\n")
        started = time.monotonic()
        r = self.run_gate("", "", extra_env={"BINN_TEST_TIMEOUT": "2"})
        self.assertLess(time.monotonic() - started, 20, "the system run has no deadline")
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn(f"[FAIL] {t}", r.stdout)

    def test_pinned_test_that_passes_under_system_python_is_stale(self):
        t = self.add("test_plain.py", PASSING)
        r = self.run_gate(t, self.fake_venv("venv"))
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn(f"STALE: {t}", r.stdout)
        self.assertIn("passes under the system python", r.stdout)

    def test_pinned_test_failing_for_another_reason_is_not_called_stale_for_not_failing(self):
        t = self.add("test_broken.py", "raise SystemExit(2)\n")
        r = self.run_gate(t, self.fake_venv("venv"))
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertNotIn("did not fail on one", r.stdout)
        self.assertIn("for another reason", r.stdout)
        failed_line = next(l for l in r.stdout.splitlines() if l.startswith("FAILED:"))
        self.assertEqual(failed_line.split().count(t), 1, failed_line)

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

    def test_every_requirements_dependency_is_optional_not_just_the_first_few(self):
        # numpy and pillow (imported as PIL) are in the requirements file; the
        # old hand-kept list omitted them, so a test needing numpy was a FAIL.
        for mod in ("numpy", "PIL", "tqdm", "DCLS"):
            with self.subTest(mod=mod):
                t = self.add("test_dep.py", f"import {mod}\n" + PASSING)
                r = self.run_gate(t, self.fake_venv(f"v{mod}", mods=(mod,)))
                self.assertEqual(r.returncode, 0, r.stdout)
                self.assertIn(f"in v{mod})", r.stdout)

    def test_optional_deps_follow_the_requirements_file(self):
        (self.root / "requirements-shd-calibration.txt").write_text("zzdep>=1.0  # a new one\n")
        t = self.add("test_dep.py", "import zzdep\n" + PASSING)
        r = self.run_gate(t, self.fake_venv("v", mods=("zzdep",)))
        self.assertEqual(r.returncode, 0, r.stdout)
        self.assertIn("in v)", r.stdout)

    def test_failure_that_only_quotes_a_missing_module_is_not_a_skip(self):
        t = self.add("test_needs.py", H5PY_THEN_FAIL_QUOTING_IMPORT_ERROR)
        r = self.run_gate(t, self.fake_venv("venv"))
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn("[FAIL] scripts/test_needs.py", r.stdout)
        self.assertNotIn("[SKIP]", r.stdout)

    def test_system_failure_quoting_a_missing_module_is_not_a_skip(self):
        t = self.add("test_x.py", "print(\"No module named 'h5py'\")\nraise SystemExit(1)\n")
        r = self.run_gate(t, self.fake_venv("venv"))
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn(f"[FAIL] {t}", r.stdout)
        self.assertNotIn("[SKIP]", r.stdout)

    def test_venv_exit_zero_with_no_tests_run_is_not_a_pass(self):
        t = self.add("test_needs.py", H5PY_THEN_NOTHING)
        r = self.run_gate(t, self.fake_venv("venv"))
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn("[FAIL] scripts/test_needs.py", r.stdout)
        self.assertIn("passed: 0", r.stdout)

    def test_venv_all_skipped_is_not_a_pass(self):
        t = self.add("test_needs.py", H5PY_THEN_ALL_SKIPPED)
        r = self.run_gate(t, self.fake_venv("venv"))
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn("[FAIL] scripts/test_needs.py", r.stdout)

    def test_venv_partial_skip_is_reported_not_hidden(self):
        t = self.add("test_needs.py", H5PY_THEN_PARTIAL_SKIP)
        r = self.run_gate(t, self.fake_venv("venv"))
        self.assertEqual(r.returncode, 0, r.stdout)
        self.assertIn(f"[ ok ] {t}  (Ran 2 tests, 1 skipped; in venv)", r.stdout)
        self.assertIn("files with skipped tests: 1", r.stdout)

    def test_system_partial_skip_is_reported_not_hidden(self):
        t = self.add("test_part.py", PARTIAL_SKIP)
        r = self.run_gate("", "")
        self.assertEqual(r.returncode, 0, r.stdout)
        self.assertIn(f"[ ok ] {t}  (Ran 2 tests, 1 skipped)", r.stdout)
        self.assertIn("files with skipped tests: 1", r.stdout)

    def test_system_proof_survives_stdout_flushed_after_the_summary(self):
        t = self.add("test_noisy.py", NOISY_AFTER_SUMMARY)
        r = self.run_gate("", "")
        self.assertEqual(r.returncode, 0, r.stdout)
        self.assertIn(f"[ ok ] {t}  (Ran 2 tests)", r.stdout)

    def test_venv_proof_survives_stdout_flushed_after_the_summary(self):
        t = self.add("test_needs.py", H5PY_THEN_NOISY_AFTER_SUMMARY)
        r = self.run_gate(t, self.fake_venv("venv"))
        self.assertEqual(r.returncode, 0, r.stdout)
        self.assertIn(f"[ ok ] {t}  (Ran 2 tests; in venv)", r.stdout)

    def test_a_printed_summary_line_cannot_stand_in_for_the_real_one(self):
        t = self.add("test_fake.py", FAKE_SUMMARY_ALL_SKIPPED)
        r = self.run_gate("", "")
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn(f"[FAIL] {t}", r.stdout)
        self.assertNotIn("[ ok ]", r.stdout)

    def test_a_failed_summary_is_not_a_pass_even_at_exit_zero(self):
        t = self.add("test_failed0.py", FAILED_EXIT_ZERO)
        r = self.run_gate("", "")
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn(f"[FAIL] {t}", r.stdout)
        self.assertNotIn("[ ok ]", r.stdout)

    def test_system_all_skipped_is_not_a_pass(self):
        t = self.add("test_skip.py", ALL_SKIPPED)
        r = self.run_gate("", "")
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn(f"[FAIL] {t}", r.stdout)
        self.assertNotIn("[ ok ]", r.stdout)
        self.assertIn("passed: 0   failed: 1", r.stdout)
        self.assertNotIn("executed and passed", r.stdout)

    def test_system_zero_tests_is_not_a_pass(self):
        """Characterization: Python 3.12 exits 5 on "Ran 0 tests", so this passes
        against any runner that fails a nonzero exit. The discriminating case is
        `test_system_zero_tests_at_exit_zero_is_not_a_pass`."""
        t = self.add("test_zero.py", ZERO_TESTS)
        r = self.run_gate("", "")
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn(f"[FAIL] {t}", r.stdout)
        self.assertNotIn("[ ok ]", r.stdout)
        self.assertIn("passed: 0   failed: 1", r.stdout)

    def test_a_run_with_skips_never_says_executed_and_passed(self):
        t = self.add("test_part.py", PARTIAL_SKIP)
        self.add("test_plain.py", PASSING)
        r = self.run_gate("", "")
        # Exit status stays 0 (record_checks.sh reads only that); the wording
        # must say plainly that something did not run.
        self.assertEqual(r.returncode, 0, r.stdout)
        self.assertNotIn("executed and passed", r.stdout)
        self.assertIn(f"but 1 file(s) had skipped tests: {t}", r.stdout)

    def test_a_run_with_no_skips_still_says_executed_and_passed(self):
        self.add("test_plain.py", PASSING)
        r = self.run_gate("", "")
        self.assertEqual(r.returncode, 0, r.stdout)
        self.assertIn("All discovered python tests executed and passed.", r.stdout)
        self.assertNotIn("had skipped tests", r.stdout)

    def test_the_last_n_of_n_line_must_be_complete(self):
        t = self.add("test_hand.py", "print('2/2 passed')\nprint('1/3 passed')\n")
        r = self.run_gate("", "")
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn(f"[FAIL] {t}", r.stdout)
        self.assertNotIn("[ ok ]", r.stdout)

    def test_a_later_complete_n_of_n_line_after_a_short_one_is_judged_by_the_last(self):
        t = self.add("test_hand.py", "print('1/3 passed')\nprint('3/3 passed')\n")
        r = self.run_gate("", "")
        self.assertEqual(r.returncode, 0, r.stdout)
        self.assertIn(f"[ ok ] {t}  (3/3 passed)", r.stdout)

    def test_system_zero_tests_at_exit_zero_is_not_a_pass(self):
        # Discriminating: exits 0 on every interpreter, so only the runner's own
        # "no proof any test ran" rule can reject it.
        t = self.add("test_zero0.py", ZERO_TESTS_EXIT_ZERO)
        r = self.run_gate("", "")
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn(f"[FAIL] {t}", r.stdout)
        self.assertNotIn("[ ok ]", r.stdout)

    def test_a_summary_written_after_unittests_own_cannot_turn_all_skipped_into_a_pass(self):
        t = self.add("test_atexit.py", ATEXIT_SUMMARY_ALL_SKIPPED)
        r = self.run_gate("", "")
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn(f"[FAIL] {t}", r.stdout)
        self.assertNotIn("[ ok ]", r.stdout)

    def test_the_same_holds_in_the_venv(self):
        t = self.add("test_atexit.py", "import h5py\n" + ATEXIT_SUMMARY_ALL_SKIPPED)
        r = self.run_gate(t, self.fake_venv("venv"))
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertNotIn("[ ok ]", r.stdout)

    def test_the_reported_summary_is_unittests_own_not_a_printed_one(self):
        # Through the fake venv, whose wrapper has no -I, so PYTHONUNBUFFERED set
        # by bounded() reaches the child. Buffered, the printed "Ran 5 tests"
        # would flush last and be reported.
        t = self.add("test_needs.py", "import h5py\n" + PRINTED_SUMMARY_PASSING)
        r = self.run_gate(t, self.fake_venv("venv"))
        self.assertEqual(r.returncode, 0, r.stdout)
        self.assertIn(f"[ ok ] {t}  (Ran 2 tests; in venv)", r.stdout)

    def test_a_failed_verdict_is_not_rescued_by_a_later_ok_line(self):
        t = self.add("test_failedok.py", FAILED_THEN_STRAY_OK)
        r = self.run_gate("", "")
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn(f"[FAIL] {t}", r.stdout)
        self.assertNotIn("[ ok ]", r.stdout)

    def test_system_exit_zero_with_no_output_is_not_a_pass(self):
        t = self.add("test_silent.py", "pass\n")
        r = self.run_gate("", "")
        self.assertEqual(r.returncode, 1, r.stdout)
        self.assertIn(f"[FAIL] {t}", r.stdout)

    def test_system_unskipped_line_is_unchanged_and_summary_counts_zero(self):
        t = self.add("test_plain.py", PASSING)
        r = self.run_gate("", "")
        self.assertEqual(r.returncode, 0, r.stdout)
        self.assertIn(f"  [ ok ] {t}  (Ran 2 tests)\n", r.stdout)
        self.assertIn("files with skipped tests: 0", r.stdout)

    def test_a_single_test_is_counted(self):
        t = self.add("test_needs.py", "import h5py\n" + ONE_PASSING)
        r = self.run_gate(t, self.fake_venv("venv"))
        self.assertEqual(r.returncode, 0, r.stdout)
        self.assertIn("(Ran 1 test; in venv)", r.stdout)

    def test_hand_rolled_runner_summary_counts_as_proof(self):
        body = "import h5py\nprint('3/3 passed')\n"
        t = self.add("test_needs.py", body)
        r = self.run_gate(t, self.fake_venv("venv"))
        self.assertEqual(r.returncode, 0, r.stdout)
        self.assertIn("(3/3 passed; in venv)", r.stdout)

    def test_paths_with_spaces_work(self):
        root = self.make_root(self.tmp / "a repo with spaces")
        (root / "scripts" / "test_my needs.py").write_text(H5PY_THEN_PASS)
        pinned = "scripts/test_my needs.py"
        venv = self.fake_venv("my venv")
        self.root = root
        r = self.run_gate(pinned, venv)
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        self.assertIn("[ ok ] scripts/test_my needs.py  (Ran 2 tests; in my venv)", r.stdout)
        self.assertNotIn("STALE", r.stdout)
        self.assertNotIn("UNEXPECTED", r.stdout)


FAKE_CHECKS = [
    "check_record_links.py", "check_every_number.py", "check_section0_citations.py",
    "build_paper.py", "check_terminology.py", "check_verdicts_transcribed.py",
    "build_results_index.py", "verify_published_numbers.py", "cross_isa_reproduction.py",
    "aws/check_reproduction.py", "mechanism_coverage.py", "find_weak_checks.py",
]


@unittest.skipUnless(sys.version_info >= (3, 11), "needs a 3.11+ host interpreter")
class RecordChecksTest(RunnerBase):
    """record_checks.sh around a fake run_python_tests.sh and trivial checks."""

    def build(self, runner_status: int):
        (self.root / "scripts" / "aws").mkdir(exist_ok=True)
        shutil.copy(RECORD_CHECKS, self.root / "scripts" / "record_checks.sh")
        (self.root / "scripts" / "run_python_tests.sh").write_text(
            f"echo 'COULD NOT RUN (not passed): 1 test(s)'\nexit {runner_status}\n"
        )
        for name in FAKE_CHECKS:
            # tomllib is 3.11+: stands in for the build scripts' requirement.
            (self.root / "scripts" / name).write_text("import tomllib\n")

    def run_checks(self):
        env = dict(os.environ)
        env.pop("BINN_PYTHON", None)
        # A stock-macOS-style python3 (too old) first on PATH.
        old = self.tmp / "old"
        old.mkdir(exist_ok=True)
        (old / "python3").write_text("#!/bin/sh\nexit 1\n")
        (old / "python3").chmod(0o755)
        env["PATH"] = f"{old}:{os.path.dirname(sys.executable)}:{env['PATH']}"
        return subprocess.run(
            ["bash", str(self.root / "scripts" / "record_checks.sh")],
            capture_output=True, text=True, env=env, timeout=120,
        )

    def test_unrunnable_tests_are_reported_separately_and_do_not_fail_the_gate(self):
        self.build(3)
        r = self.run_checks()
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        self.assertIn("COULD NOT RUN", r.stdout.split("Record checks that ran")[-1])
        self.assertNotIn("All record checks executed and passed", r.stdout)

    def test_clean_run_says_passed(self):
        self.build(0)
        r = self.run_checks()
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        self.assertIn("All record checks executed and passed.", r.stdout)

    def test_a_failing_test_run_fails_the_gate(self):
        self.build(1)
        r = self.run_checks()
        self.assertEqual(r.returncode, 1, r.stdout + r.stderr)
        self.assertIn("RECORD CHECKS FAILED", r.stdout)

    def test_a_python_older_than_3_11_is_skipped_for_a_suitable_one(self):
        self.build(0)
        r = self.run_checks()
        # The too-old python3 first on PATH would make every `import tomllib`
        # check fail; a resolved interpreter makes them pass.
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        self.assertIn("record_checks: using ", r.stdout)
        self.assertNotIn(str(self.tmp / "old"), r.stdout.split("\n")[0])


def _is_main_guard(test) -> bool:
    """`__name__ == "__main__"`, in either operand order."""
    import ast

    if not (isinstance(test, ast.Compare) and len(test.ops) == 1
            and isinstance(test.ops[0], ast.Eq) and len(test.comparators) == 1):
        return False
    operands = (test.left, test.comparators[0])
    return (any(isinstance(o, ast.Name) and o.id == "__name__" for o in operands)
            and any(isinstance(o, ast.Constant) and o.value == "__main__" for o in operands))


def _find_main_guard(node):
    """The first main guard in `node` or the statements it runs inline.

    Descends into `if`/`try`/`with`/loop blocks, which execute at import, but not
    into `def` or `class` bodies, which do not.
    """
    import ast

    if isinstance(node, ast.If) and _is_main_guard(node.test):
        return node
    if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)):
        return None
    for field in ("body", "orelse", "finalbody", "handlers"):
        for child in getattr(node, field, None) or []:
            hit = _find_main_guard(child)
            if hit is not None:
                return hit
    return None


def first_dead_statement(source: str, filename: str = "<test>"):
    """The first top-level statement that follows a module-level main guard.

    Returns `(guard_line, dead_line)` or None. The gate runs each file as a
    script, and `unittest.main()` (or a hand-rolled runner's `sys.exit`) ends
    the process, so anything after the guard is collected by `-m unittest
    discover` but never run by the gate. The guard may be written either way
    round and may sit inside a `try:` (or other inline block).
    """
    import ast

    body = ast.parse(source, filename).body
    for index, node in enumerate(body):
        guard = _find_main_guard(node)
        if guard is not None and index + 1 < len(body):
            return guard.lineno, body[index + 1].lineno
    return None


class MainGuardIsLastTests(unittest.TestCase):
    """Nothing may follow `if __name__ == "__main__":` in a test file.

    The gate runs every `scripts/test_*.py` as a script, so a TestCase defined
    after the guard is never run by it. Covers hand-rolled runners too.
    """

    def test_every_test_file_ends_with_its_main_guard(self):
        offenders = []
        for path in sorted(HERE.glob("test_*.py")):
            hit = first_dead_statement(path.read_text(encoding="utf-8"), str(path))
            if hit:
                guard, dead = hit
                offenders.append(
                    f"{path.name}: statement at line {dead} follows the "
                    f"`if __name__ == \"__main__\":` guard at line {guard} and "
                    "never runs under the gate; move the guard to the end of the file"
                )
        self.assertEqual(offenders, [], "\n".join(offenders))

    def test_the_detector_flags_code_after_the_guard_and_only_that(self):
        dead = 'if __name__ == "__main__":\n    pass\n\nclass T: pass\n'
        self.assertEqual(first_dead_statement(dead), (1, 4))
        self.assertIsNone(first_dead_statement('class T: pass\nif __name__ == "__main__":\n    pass\n'))
        self.assertIsNone(first_dead_statement("class T: pass\n"))
        # A guard written the other way round, or an unrelated `if`, is not one.
        self.assertIsNone(first_dead_statement('if x == "__main__":\n    pass\nx = 1\n'))

    def test_the_detector_flags_a_reversed_operand_guard(self):
        self.assertEqual(
            first_dead_statement('if "__main__" == __name__:\n    pass\nclass T: pass\n'), (1, 3))

    def test_the_detector_flags_a_guard_nested_in_try(self):
        nested = (
            "try:\n"
            '    if __name__ == "__main__":\n'
            "        pass\n"
            "except Exception:\n"
            "    raise\n"
            "class T: pass\n"
        )
        self.assertEqual(first_dead_statement(nested), (2, 6))
        in_else = 'if True:\n    pass\nelse:\n    if "__main__" == __name__:\n        pass\nx = 1\n'
        self.assertEqual(first_dead_statement(in_else), (4, 6))

    def test_a_guard_that_ends_its_block_is_not_dead_code(self):
        # Anything after the guard inside the same nested block is still ahead of
        # nothing runnable only if it is at the end of the whole module.
        self.assertIsNone(first_dead_statement(
            'try:\n    if __name__ == "__main__":\n        pass\nexcept Exception:\n    raise\n'))


if __name__ == "__main__":
    unittest.main(verbosity=2)
