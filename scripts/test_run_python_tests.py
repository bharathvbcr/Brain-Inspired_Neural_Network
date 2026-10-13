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


if __name__ == "__main__":
    unittest.main(verbosity=2)
