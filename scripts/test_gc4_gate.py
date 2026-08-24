#!/usr/bin/env python3
"""GC4 must answer, and must answer about the files it claims to watch.

GC4 hung for two days. Its `rg` call carried a `--glob` but no path argument,
and ripgrep with no path reads *stdin*; run from a pipeline it blocked on a read
that never returned, at 0% CPU, forever. A gate that never answers is worse than
one that answers wrong: nothing downstream can tell "still thinking" from
"passed", and a runner without a timeout simply waits.

Every check here runs the gate against a synthetic tree so the real repository is
never mutated, except `passes_on_the_real_repository`, which is the point.

Pre-fix, `answers_with_an_open_stdin` hangs and `refuses_when_a_target_is_gone`
reports PASS.
"""

from __future__ import annotations

import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
GATE = REPO / "scripts" / "check_gc4.sh"

# Long enough that a slow machine is not called a hang; short enough that a real
# hang does not wedge the suite. The gate answers in about a second.
TIMEOUT_S = 30

CLEAN = "pub struct Encoder;\nimpl Encoder {\n    pub fn encode(&self) {}\n}\n"
VIOLATION = "pub struct Decoder;\nimpl Decoder {\n    pub fn fit(&self) {}\n}\n"


def synthetic_tree(tmp: Path, encoder: str, decoder: str) -> Path:
    """A minimal tree shaped the way the gate resolves its own root."""
    (tmp / "scripts").mkdir(parents=True)
    shutil.copy2(GATE, tmp / "scripts" / "check_gc4.sh")
    src = tmp / "binn-data" / "src"
    src.mkdir(parents=True)
    (src / "encoder.rs").write_text(encoder)
    (src / "decoder.rs").write_text(decoder)
    return tmp / "scripts" / "check_gc4.sh"


def run(gate: Path, stdin=subprocess.DEVNULL) -> subprocess.CompletedProcess:
    return subprocess.run(
        ["bash", str(gate)],
        stdin=stdin,
        capture_output=True,
        text=True,
        timeout=TIMEOUT_S,
    )


def passes_on_a_clean_tree() -> None:
    with tempfile.TemporaryDirectory() as d:
        gate = synthetic_tree(Path(d), CLEAN, CLEAN.replace("Encoder", "Decoder"))
        done = run(gate)
    assert done.returncode == 0, f"clean tree rejected: {done.stdout}{done.stderr}"
    assert "PASS" in done.stdout, done.stdout


def catches_a_violation_in_each_file() -> None:
    """Both watched files, separately — a gate reading only one is half a gate."""
    for name, enc, dec in (
        ("encoder", VIOLATION.replace("Decoder", "Encoder"), CLEAN),
        ("decoder", CLEAN, VIOLATION),
    ):
        with tempfile.TemporaryDirectory() as d:
            gate = synthetic_tree(Path(d), enc, dec)
            done = run(gate)
        assert done.returncode == 1, (
            f"a `fn fit` planted in {name}.rs did not fail the gate; "
            f"exit={done.returncode} out={done.stdout}"
        )
        assert "FAIL" in done.stdout, done.stdout


def refuses_when_a_target_is_gone() -> None:
    """Renaming a watched file must not read as "no violations found".

    This is the vacuous-pass shape: zero matches because zero files were read.
    """
    with tempfile.TemporaryDirectory() as d:
        gate = synthetic_tree(Path(d), CLEAN, CLEAN)
        (Path(d) / "binn-data" / "src" / "encoder.rs").unlink()
        done = run(gate)
    assert done.returncode != 0, (
        "the gate passed with a watched file missing — it would report PASS "
        f"forever after a rename: {done.stdout}"
    )
    assert "CANNOT RUN" in done.stdout, (
        f"the gate failed, but not distinguishably from a real violation: {done.stdout}"
    )


def answers_with_an_open_stdin() -> None:
    """The regression. Pre-fix this call raises TimeoutExpired."""
    reader = None
    with tempfile.TemporaryDirectory() as d:
        gate = synthetic_tree(Path(d), CLEAN, CLEAN)
        # A pipe nobody ever writes to and nobody ever closes: exactly the stdin
        # the gate inherited from the shell that hung.
        reader = subprocess.Popen(
            [sys.executable, "-c", "import time; time.sleep(600)"],
            stdout=subprocess.PIPE,
        )
        try:
            done = run(gate, stdin=reader.stdout)
        except subprocess.TimeoutExpired:
            raise AssertionError(
                f"the gate did not answer within {TIMEOUT_S}s with an open stdin; "
                "it is reading stdin instead of the files it watches"
            ) from None
        finally:
            reader.kill()
            reader.wait()
    assert done.returncode == 0, done.stdout + done.stderr
    assert "PASS" in done.stdout, done.stdout


def passes_on_the_real_repository() -> None:
    done = run(GATE)
    assert done.returncode == 0, (
        f"GC4 fails on the checked-out tree: {done.stdout}{done.stderr}"
    )
    assert "files read" in done.stdout, (
        "the gate no longer reports how many files it read, so a run that read "
        f"nothing looks like a run that read everything: {done.stdout}"
    )


def main() -> int:
    checks = [
        passes_on_a_clean_tree,
        catches_a_violation_in_each_file,
        refuses_when_a_target_is_gone,
        answers_with_an_open_stdin,
        passes_on_the_real_repository,
    ]
    failed = 0
    for check in checks:
        try:
            check()
        except AssertionError as exc:
            print(f"FAIL {check.__name__}: {exc}")
            failed += 1
        else:
            print(f"ok   {check.__name__}")
    print(f"{len(checks) - failed}/{len(checks)} passed")
    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(main())
