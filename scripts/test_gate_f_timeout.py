#!/usr/bin/env python3
"""Gate F must answer about every cell, and never confuse silence with a pass.

Gate F used to run the instrument with `subprocess.run(...)` and no timeout, so
a hung instrument hung the gate — the shape that left GC4 blocked for two days.
It also read its output file without deleting the previous one first, so an
instrument that exited 0 without writing anything was compared against the last
run's observation and reported BIT_IDENTICAL.

Every check here drives the real gate against a fake instrument that can hang,
crash, lie, write nothing, or leave a grandchild behind. Nothing touches the
recorded corpus: RESULT_ROOT is redirected at a temporary tree.
"""

from __future__ import annotations

import json
import os
import shutil
import signal
import subprocess
import sys
import tempfile
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import gate_f_rust as G  # noqa: E402

CELL = "rust__fixed-t100__adjacent-sum-5__h128__e20__s5170001"

# The values the fake instrument reproduces when told to behave.
TRUTH = {
    "accuracy": 0.609982332,
    "mean_loss": 1.726072573,
    "mean_gradient_norm": 0.277363307,
    "mean_update_rms": 0.000800666,
    "mean_firing_rate": 0.183328549,
    "majority_prediction": 0.101148410,
    "classes_predicted": 20,
    "silent_fraction": 0.0,
    "saturated_fraction": 0.0,
    "non_finite_events": 0,
    "n_train": 8156,
    "n_test": 2264,
    "wall_secs": 10.6,
    "epoch_mean_loss": [2.9, 2.8],
    "epoch_mean_gradient_norm": [0.19, 0.21],
}

FAKE = r'''#!/usr/bin/env python3
import json, os, subprocess, sys, time
mode = os.environ["FAKE_MODE"]
out = sys.argv[sys.argv.index("--out") + 1]
truth = json.loads(os.environ["FAKE_TRUTH"])
if mode == "grandchild":
    # Detached-looking child in the same process group; only a group kill gets it.
    subprocess.Popen([sys.executable, "-c", "import time; time.sleep(600)"])
    time.sleep(600)
if mode == "ignore_term":
    import signal as s
    s.signal(s.SIGTERM, s.SIG_IGN)
    s.signal(s.SIGINT, s.SIG_IGN)
    time.sleep(600)
if mode == "hang":
    time.sleep(600)
if mode == "exit1":
    sys.stderr.write("instrument refused the arm\n"); sys.exit(1)
if mode == "nowrite":
    sys.exit(0)
if mode == "corrupt":
    open(out, "w").write("{not json"); sys.exit(0)
if mode == "mismatch":
    truth["accuracy"] = 0.5
if mode == "trace_drift":
    truth["epoch_mean_loss"] = [2.9, 2.7]
json.dump(truth, open(out, "w"))
'''


class Harness:
    """A temporary Gate F world: one recorded cell, one fake instrument."""

    def __init__(self) -> None:
        self.dir = Path(tempfile.mkdtemp(prefix="gatef-"))
        self.saved = G.RESULT_ROOT
        G.RESULT_ROOT = self.dir
        (self.dir / "cells").mkdir(parents=True)
        (self.dir / "initialization").mkdir(parents=True)
        (self.dir / "cells" / f"{CELL}.json").write_text(json.dumps(TRUTH))
        (self.dir / "initialization" / "n140-h128-s5170001.weights").write_bytes(b"w")
        (self.dir / "initialization" / "n8156-e100-s5170001.orders").write_bytes(b"o")
        self.binary = self.dir / "fake-instrument"
        self.binary.write_text(FAKE)
        self.binary.chmod(0o755)

    def run(self, mode: str, factor: float = 20.0, floor: float = 900.0) -> dict:
        os.environ["FAKE_MODE"] = mode
        os.environ["FAKE_TRUTH"] = json.dumps(TRUTH)
        return G.regress_cell(CELL, self.binary, factor, floor)

    @property
    def observation(self) -> Path:
        return self.dir / "gate-f-rust" / f"{CELL}.json"

    def close(self) -> None:
        G.RESULT_ROOT = self.saved
        shutil.rmtree(self.dir, ignore_errors=True)


def harnessed(fn):
    def wrapper():
        h = Harness()
        try:
            fn(h)
        finally:
            h.close()
    wrapper.__name__ = fn.__name__
    return wrapper


# ---------------------------------------------------------------- comparison


@harnessed
def a_faithful_rerun_is_bit_identical(h):
    r = h.run("match")
    assert r["status"] == "BIT_IDENTICAL", r
    assert r["mismatches"] == {}, r
    assert r["compared_traces"] == ["epoch_mean_loss", "epoch_mean_gradient_norm"], r


@harnessed
def a_changed_field_is_a_regression(h):
    r = h.run("mismatch")
    assert r["status"] == "REGRESSION", r
    assert "accuracy" in r["mismatches"], r


@harnessed
def a_changed_trace_is_a_regression(h):
    """Traces are the only per-epoch evidence; drift there must not pass."""
    r = h.run("trace_drift")
    assert r["status"] == "REGRESSION", r
    assert "epoch_mean_loss" in r["mismatches"], r


# ------------------------------------------------------------------- hanging


@harnessed
def a_hung_instrument_is_a_timeout_not_a_pass(h):
    start = time.monotonic()
    r = h.run("hang", factor=0.1, floor=3.0)
    elapsed = time.monotonic() - start
    assert r["status"] == "TIMEOUT", r
    assert "mismatches" not in r, "a timeout must not carry an empty mismatch set"
    assert elapsed < 60, f"the gate took {elapsed:.0f}s to give up on a 3s budget"


@harnessed
def a_hung_instrument_is_actually_dead_afterwards(h):
    h.run("hang", factor=0.1, floor=3.0)
    time.sleep(0.5)
    survivors = subprocess.run(
        ["pgrep", "-f", "fake-instrument"], capture_output=True, text=True
    ).stdout.split()
    assert not survivors, f"the instrument outlived the gate: pids {survivors}"


@harnessed
def a_grandchild_does_not_outlive_the_kill(h):
    """`subprocess.run(timeout=)` signals only the direct child; the group must go."""
    r = h.run("grandchild", factor=0.1, floor=3.0)
    assert r["status"] == "TIMEOUT", r
    time.sleep(0.5)
    left = subprocess.run(
        ["pgrep", "-f", "time.sleep(600)"], capture_output=True, text=True
    ).stdout.split()
    assert not left, f"a grandchild survived the timeout: pids {left}"


@harnessed
def an_instrument_that_ignores_sigterm_still_dies(h):
    start = time.monotonic()
    r = h.run("ignore_term", factor=0.1, floor=3.0)
    assert r["status"] == "TIMEOUT", r
    assert time.monotonic() - start < 60, "SIGTERM-ignoring child was not killed promptly"


@harnessed
def the_budget_scales_with_the_recorded_wall_clock(h):
    assert G.cell_timeout({"wall_secs": 100.0}, 20.0, 900.0) == 2000.0
    assert G.cell_timeout({"wall_secs": 1.0}, 20.0, 900.0) == 900.0, "floor must apply"
    assert G.cell_timeout({}, 20.0, 900.0) == 900.0, "a cell with no timing gets the floor"
    assert G.cell_timeout({"wall_secs": None}, 20.0, 900.0) == 900.0


# ------------------------------------------------------- could-not-run cases


@harnessed
def an_instrument_that_writes_nothing_is_not_a_pass(h):
    r = h.run("nowrite")
    assert r["status"] == "ERROR", r
    assert "without writing" in r["detail"], r


@harnessed
def a_stale_observation_cannot_be_mistaken_for_a_fresh_one(h):
    """The hole that made silence look like success.

    A good run leaves an observation behind. If the next run writes nothing and
    the gate does not clear the file first, it compares the *previous* run's
    output and reports BIT_IDENTICAL for a run that produced nothing.
    """
    assert h.run("match")["status"] == "BIT_IDENTICAL"
    assert h.observation.is_file(), "the good run should have left an observation"
    r = h.run("nowrite")
    assert r["status"] == "ERROR", (
        f"the gate reported {r['status']} by reading the previous run's file"
    )
    assert not h.observation.is_file(), "the stale observation was left in place"


@harnessed
def a_gutted_recorded_cell_cannot_pass_by_comparing_nothing(h):
    """Schema drift must not read as agreement.

    Every compared field was skipped when absent from the recorded cell, so a
    cell that had lost them left `mismatches` empty and reported BIT_IDENTICAL.
    `compared_traces` was disclosed per cell but the fields were not, so no
    number in the report would have shown it.
    """
    stripped = {k: v for k, v in TRUTH.items()
                if k not in ("accuracy", "mean_loss", "mean_gradient_norm",
                             "mean_update_rms", "n_train")}
    (h.dir / "cells" / f"{CELL}.json").write_text(json.dumps(stripped))
    r = h.run("match")
    assert r["status"] == "ERROR", (
        f"a cell missing five of twelve measurements reported {r['status']}"
    )
    assert "measurements are present" in r["detail"], r


@harnessed
def a_healthy_cell_discloses_which_fields_were_compared(h):
    r = h.run("match")
    assert r["status"] == "BIT_IDENTICAL", r
    assert len(r["compared_fields"]) == 12, r["compared_fields"]


@harnessed
def a_corrupt_observation_is_an_error(h):
    r = h.run("corrupt")
    assert r["status"] == "ERROR", r
    assert "json" in r["detail"], r


@harnessed
def a_failing_instrument_is_an_error_with_its_output(h):
    r = h.run("exit1")
    assert r["status"] == "ERROR", r
    assert "refused the arm" in r["detail"], r


@harnessed
def a_missing_binary_does_not_crash_the_sweep(h):
    r = G.regress_cell(CELL, h.dir / "does-not-exist", 20.0, 900.0)
    assert r["status"] == "ERROR", r


@harnessed
def a_missing_recorded_cell_is_an_error(h):
    r = G.regress_cell("rust__fixed-t100__adjacent-sum-5__h128__e20__s9999999",
                       h.binary, 20.0, 900.0)
    assert r["status"] == "ERROR", r


@harnessed
def a_malformed_cell_id_is_an_error_not_an_exception(h):
    r = G.regress_cell("nonsense", h.binary, 20.0, 900.0)
    assert r["status"] == "ERROR", r


@harnessed
def a_missing_initialization_artifact_is_an_error(h):
    (h.dir / "initialization" / "n140-h128-s5170001.weights").unlink()
    r = h.run("match")
    assert r["status"] == "ERROR", r
    assert "initialization artifact" in r["detail"], r


# ------------------------------------------------------- reporting and codes


@harnessed
def the_report_separates_what_ran_from_what_did_not(h):
    results = [
        {"status": "BIT_IDENTICAL", "cell": "a", "compared_traces": []},
        {"status": "REGRESSION", "cell": "b", "compared_traces": []},
        {"status": "TIMEOUT", "cell": "c", "compared_traces": []},
        {"status": "ERROR", "cell": "d", "compared_traces": []},
    ]
    path, regressions, unrunnable = G.write_report(h.binary, "deadbeef", results)
    payload = json.loads(path.read_text())
    assert regressions == 1 and unrunnable == 2, (regressions, unrunnable)
    assert payload["compared"] == 2, payload
    assert payload["failures"] == 1 and payload["unrunnable"] == 2, payload
    assert payload["status"] == "FAIL", payload


@harnessed
def unrunnable_alone_is_incomplete_never_pass(h):
    results = [
        {"status": "BIT_IDENTICAL", "cell": "a", "compared_traces": []},
        {"status": "TIMEOUT", "cell": "c", "compared_traces": []},
    ]
    path, _, _ = G.write_report(h.binary, "deadbeef", results)
    payload = json.loads(path.read_text())
    assert payload["status"] == "INCOMPLETE", (
        f"a sweep with an unjudged cell reported {payload['status']}"
    )


@harnessed
def every_run_is_appended_to_the_history(h):
    G.write_report(h.binary, "aaa", [{"status": "BIT_IDENTICAL", "cell": "a",
                                      "compared_traces": []}])
    G.write_report(h.binary, "bbb", [{"status": "TIMEOUT", "cell": "b",
                                      "compared_traces": []}])
    lines = (h.dir / "gate-f-rust" / "runs.jsonl").read_text().strip().splitlines()
    assert len(lines) == 2, lines
    assert json.loads(lines[0])["binary_sha256"] == "aaa"
    assert json.loads(lines[1])["status"] == "INCOMPLETE"


@harnessed
def exit_codes_tell_the_three_outcomes_apart(h):
    os.environ["FAKE_MODE"] = "match"
    os.environ["FAKE_TRUTH"] = json.dumps(TRUTH)
    assert G.main(["--cell", CELL, "--binary", str(h.binary)]) == G.EXIT_PASS

    os.environ["FAKE_MODE"] = "mismatch"
    assert G.main(["--cell", CELL, "--binary", str(h.binary)]) == G.EXIT_REGRESSION

    os.environ["FAKE_MODE"] = "exit1"
    assert G.main(["--cell", CELL, "--binary", str(h.binary)]) == G.EXIT_UNRUNNABLE

    os.environ["FAKE_MODE"] = "hang"
    assert G.main(["--cell", CELL, "--binary", str(h.binary),
                   "--timeout-factor", "0.1", "--timeout-floor", "3"]) == G.EXIT_UNRUNNABLE
    assert G.EXIT_REGRESSION != G.EXIT_UNRUNNABLE != G.EXIT_PASS


@harnessed
def a_report_is_written_even_when_every_cell_fails(h):
    os.environ["FAKE_MODE"] = "exit1"
    os.environ["FAKE_TRUTH"] = json.dumps(TRUTH)
    G.main(["--cell", CELL, "--binary", str(h.binary)])
    payload = json.loads((h.dir / "gate-f-rust" / "report.json").read_text())
    assert payload["unrunnable"] == 1 and payload["compared"] == 0, payload


@harnessed
def a_forever_budget_cannot_be_requested(h):
    for bad in (["--timeout-factor", "0"], ["--timeout-floor", "0"],
                ["--timeout-factor", "-1"]):
        try:
            G.main(["--cell", CELL, "--binary", str(h.binary), *bad])
        except SystemExit as exc:
            assert exc.code == 2, exc.code
        else:
            raise AssertionError(f"{bad} was accepted; the gate would wait forever")


CHECKS = [v for k, v in sorted(globals().items())
          if callable(v) and getattr(v, "__module__", None) == "__main__"
          and not k.startswith(("_", "harnessed", "Harness"))
          and k not in {"main"}]


def main() -> int:
    failed = 0
    for check in CHECKS:
        try:
            check()
        except AssertionError as exc:
            print(f"FAIL {check.__name__}: {exc}")
            failed += 1
        except Exception as exc:  # noqa: BLE001 - a crash is a failure too
            print(f"ERROR {check.__name__}: {type(exc).__name__}: {exc}")
            failed += 1
        else:
            print(f"ok   {check.__name__}")
    print(f"{len(CHECKS) - failed}/{len(CHECKS)} passed")
    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(main())
