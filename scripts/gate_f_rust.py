"""Gate F for the rust arm — bit-identical regression of completed rust cells.

`AMENDMENT_2026-08-02_INSTRUMENT_KERNEL_AND_FRAMING.md` §5.3 puts Gate F in
force for the rust arm ("where the 216 cells are the scientific result and
bit-reproducibility is still meaningful"), but the registered implementation in
`gates_ef.py` refuses rust cells — `gate_f_cell` raises "gate-f regresses the
python arm; the rust arm is checked by gate-e" — and Gate E is not implemented
(`gates_ef.py::gate_e` raises "GATE E BLOCKED - no arm fixtures yet").

So a change to the rust kernel had no runnable gate. This is that gate.

It re-runs recorded rust cells through the current binary from the pinned
initialization artifacts and demands every scientific field match the recorded
value bit-exactly. Nothing under `initialization/` or `cells/` is written; output
goes to `gate-f-rust/`.

Why bit-exact and not a tolerance: the spike function is a hard threshold, so a
one-ulp difference flips a spike and compounds through Adam over epochs. That is
what disqualified the python kernel change on 2026-08-02. Fixture-level parity
is not evidence — the parity fixture has atypically sparse frames.

    python scripts/gate_f_rust.py --cell rust__fixed-t100__adjacent-sum-5__h128__e20__s5170001
    python scripts/gate_f_rust.py --cheapest 6
    python scripts/gate_f_rust.py --all            # 216 cells, days
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import signal
import subprocess
import sys
import time
from pathlib import Path


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1 << 20), b""):
            digest.update(chunk)
    return digest.hexdigest()

ROOT = Path(__file__).resolve().parent.parent
RESULT_ROOT = ROOT / "results" / "shd_instrument_v4"
EVENT_ROOT = ROOT / "data" / "shd" / "events"
DEFAULT_BINARY = ROOT / "target" / "release" / "shd-instrument"

# How long a cell may run before the gate gives up on it.
#
# Gate F previously ran the instrument with no timeout at all, so a hung child
# hung the gate — the same shape that left GC4 blocked for two days. A budget is
# derived per cell from what that cell actually took when it was recorded,
# rather than fixed, because the recorded cells span 33s to 3646s and one
# constant cannot fit both ends.
#
# The factor is deliberately loose. Two observations of the same cell on this
# machine came in at 10.6s and 60.8s — a 5.7x spread from load alone — so
# anything under about 10x would kill legitimate work, and killing a cell
# halfway through a multi-day `--all` sweep is worse than the hang it prevents.
# 20x leaves headroom over the worst spread seen and still turns "forever" into
# a bounded, reported outcome.
#: A recorded cell must still carry most of its measurements to be worth
#: regressing against. Every archived cell carries all twelve; a cell below this
#: has lost so much that "no mismatches" would mean "almost nothing compared".
MIN_COMPARED_FIELDS = 10

TIMEOUT_FACTOR = 20.0
#: Floor for cells recorded fast enough that the factor alone is too tight to
#: absorb a cold page cache or a first-touch of the event files.
TIMEOUT_FLOOR_S = 900.0


def cell_timeout(recorded: dict[str, object], factor: float, floor: float) -> float:
    """The budget for one cell, from its own recorded wall clock."""
    was = float(recorded.get("wall_secs") or 0.0)
    return max(floor, was * factor)

# Every field the cell schema reports as a measurement. `wall_secs` is excluded
# because it is a timing, not a result; the status fields are derived from these.
COMPARED_FIELDS = (
    "accuracy",
    "mean_loss",
    "mean_gradient_norm",
    "mean_update_rms",
    "mean_firing_rate",
    "majority_prediction",
    "classes_predicted",
    "silent_fraction",
    "saturated_fraction",
    "non_finite_events",
    "n_train",
    "n_test",
)
# `non_finite_forward` is DELIBERATELY NOT HERE, and this note exists so the
# omission is not read as an oversight and quietly corrected.
#
# It was added to the cell schema on 2026-08-29 with the forward-finiteness
# guard, and adding it here was the first instinct. It cannot go here: this
# tuple is coupled to the FROZEN per-wave analysers by
# `test_reproduction_check.py::test_no_analyser_drops_a_field_gate_f_compares`,
# whose invariant is that an analyser may check more than Gate F and never
# less. `analyse_wave15` and `analyse_wave18` were registered with their
# preregistrations before their first cell existed and are not editable after
# the fact, so a field added here would either break that gate or force an edit
# to a frozen authority.
#
# Nothing is lost. No archived cell carries the field, so comparing it would
# compare nothing today; the guard's teeth are the pass predicate in the
# instrument and `cell_validity.py`, which is live rather than frozen. A wave
# analyser written from now on should carry it, and when every analyser Gate F
# is coupled to does, it belongs in the tuple above.
# Traces are compared when both sides carry them. Cells recorded before the
# convergence-telemetry change have no trace, and their absence is not a failure.
COMPARED_TRACES = (
    "epoch_mean_loss",
    "epoch_mean_gradient_norm",
    # Added with the max-gradient telemetry. Comparison is conditional on both
    # sides carrying the key, so the 216 recorded cells — which predate it — are
    # unaffected, and cells recorded from now on get the extra coverage free.
    "epoch_max_gradient_norm",
)


#: Arm assumed by a cell id that names no arm. Every one of the 216 recorded
#: rust cells is this arm, which is exactly the hole the optional suffix below
#: exists to close.
DEFAULT_ARM = "ff+fixed"


def parse_cell_id(cell_id: str) -> dict[str, object]:
    """Split a cell id, with optional arm and attention-shape suffixes.

    The 216 recorded cells are all six-component ids and all `ff+fixed`, so
    Gate F could not express a cell on any other arm — a change to the shared
    kernel could alter `ff+alif`, `rec+*` or any attention arm and every gate in
    the repository would still pass. Two optional components close that:

        rust__<contract>__<geometry>__h<hidden>__e<epochs>__s<seed>
        rust__...__s<seed>__<arm>
        rust__...__s<seed>__<arm>__d<dim>l<layers>

    The arm is written with hyphens, matching the campaign's own cell filenames
    (`ff-fixed-attn`), and converted to the instrument's `+` spelling here.
    """
    parts = cell_id.split("__")
    if len(parts) < 6 or len(parts) > 8:
        raise ValueError(
            f"cell id {cell_id!r} has {len(parts)} components; expected 6 "
            "(legacy), 7 (with arm) or 8 (with arm and attention shape)"
        )
    backend, contract, geometry, hidden, epochs, seed = parts[:6]
    arm = parts[6].replace("-", "+") if len(parts) > 6 else DEFAULT_ARM
    attention = None
    if len(parts) > 7:
        shape = parts[7]
        if not shape.startswith("d") or "l" not in shape:
            raise ValueError(f"attention shape {shape!r} is not d<dim>l<layers>")
        dim, layers = shape[1:].split("l", 1)
        attention = (int(dim), int(layers))
    if arm.endswith("+attn") and attention is None:
        raise ValueError(f"{cell_id!r} names an attention arm with no d<dim>l<layers>")
    if attention is not None and not arm.endswith("+attn"):
        raise ValueError(f"{cell_id!r} carries an attention shape on a plain arm")
    return {
        "backend": backend,
        "contract": contract,
        "geometry": geometry,
        "hidden": int(hidden.removeprefix("h")),
        "epochs": int(epochs.removeprefix("e")),
        "seed": int(seed.removeprefix("s")),
        "arm": arm,
        "attention": attention,
    }


def initialization_paths(spec: dict[str, object]) -> tuple[Path, Path]:
    """Where a cell's pinned starting point lives.

    The arm is part of the filename for every non-default arm, and the
    attention shape too, because two arms at the same width do not share a
    weight file — `ff+alif` carries adaptation parameters and an attention arm
    carries a whole extra parameter block. Legacy `ff+fixed` names are
    unchanged, so the 216 recorded cells keep resolving to the artifacts they
    were recorded from.
    """
    n_inputs = 700 if spec["geometry"] == "channels-700" else 140
    stem = f"n{n_inputs}-h{spec['hidden']}-s{spec['seed']}"
    if spec.get("arm", DEFAULT_ARM) != DEFAULT_ARM:
        stem += f"-{str(spec['arm']).replace('+', '-')}"
    if spec.get("attention"):
        dim, layers = spec["attention"]
        stem += f"-d{dim}l{layers}"
    weights = RESULT_ROOT / "initialization" / f"{stem}.weights"
    orders = RESULT_ROOT / "initialization" / f"n8156-e100-s{spec['seed']}.orders"
    return weights, orders


def _unrunnable(cell_id: str, status: str, detail: str, started: float,
                timeout_s: float | None = None) -> dict[str, object]:
    """A cell the gate could not judge.

    Deliberately not shaped like a comparison result: no `mismatches` key to be
    read as empty, and a status no caller can confuse with BIT_IDENTICAL. A
    check that could not run must never report what a check that ran and passed
    reports.
    """
    return {
        "cell": cell_id,
        "status": status,
        "detail": detail,
        "wall_secs": round(time.monotonic() - started, 3),
        "timeout_s": timeout_s,
        "compared_traces": [],
    }


def run_cell(command: list[str], timeout_s: float) -> subprocess.CompletedProcess:
    """Run the instrument under a hard budget, leaving nothing behind.

    `start_new_session` puts the child in its own process group so the kill on
    timeout reaches anything it spawned; `subprocess.run(timeout=...)` signals
    only the direct child, which would leave a grandchild holding the pipes and
    the wait would block again — reproducing the hang inside the fix for it.
    """
    with subprocess.Popen(
        command,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        stdin=subprocess.DEVNULL,
        text=True,
        start_new_session=True,
    ) as child:
        try:
            out, err = child.communicate(timeout=timeout_s)
        except subprocess.TimeoutExpired:
            try:
                os.killpg(os.getpgid(child.pid), signal.SIGKILL)
            except (ProcessLookupError, PermissionError):
                child.kill()
            # Reap, but do not wait forever for a process that ignored SIGKILL:
            # an unbounded second wait is the same defect one layer down.
            try:
                out, err = child.communicate(timeout=30)
            except subprocess.TimeoutExpired:
                out, err = "", "child survived SIGKILL; pipes abandoned"
            raise subprocess.TimeoutExpired(command, timeout_s, output=out, stderr=err)
        return subprocess.CompletedProcess(command, child.returncode, out, err)


def regress_cell(cell_id: str, binary: Path, factor: float = TIMEOUT_FACTOR,
                 floor: float = TIMEOUT_FLOOR_S) -> dict[str, object]:
    started = time.monotonic()
    recorded_path = RESULT_ROOT / "cells" / f"{cell_id}.json"
    if not recorded_path.is_file():
        return _unrunnable(cell_id, "ERROR",
                           f"no completed cell to regress against: {cell_id}", started)
    try:
        spec = parse_cell_id(cell_id)
    except ValueError as exc:
        return _unrunnable(cell_id, "ERROR", str(exc), started)
    if spec["backend"] != "rust":
        return _unrunnable(cell_id, "ERROR",
                           "this gate regresses the rust arm; use gates_ef.py for python",
                           started)
    try:
        recorded = json.loads(recorded_path.read_text())
    except json.JSONDecodeError as exc:
        return _unrunnable(cell_id, "ERROR", f"recorded cell is not readable json: {exc}", started)
    weights, orders = initialization_paths(spec)
    for path in (weights, orders):
        if not path.is_file():
            return _unrunnable(cell_id, "ERROR", f"missing initialization artifact: {path}", started)

    output = RESULT_ROOT / "gate-f-rust" / f"{cell_id}.json"
    output.parent.mkdir(parents=True, exist_ok=True)
    # Delete the previous observation before running. Otherwise an instrument
    # that exits 0 without writing leaves the last run's file in place, and the
    # gate compares that — reporting BIT_IDENTICAL for a run that produced
    # nothing at all.
    output.unlink(missing_ok=True)

    timeout_s = cell_timeout(recorded, factor, floor)
    command = [
        str(binary), "train-cell",
        "--train-events", str(EVENT_ROOT / "train.events"),
        "--test-events", str(EVENT_ROOT / "test.events"),
        "--contract", str(spec["contract"]),
        "--geometry", str(spec["geometry"]),
        "--weights", str(weights),
        "--orders", str(orders),
        "--epochs", str(spec["epochs"]),
        "--out", str(output),
        # Passed even though the weight file already carries the arm: the
        # instrument cross-checks the two and refuses a disagreement, so
        # naming it here turns a mismatched artifact into an error instead
        # of a silently different cell.
        *(["--arm", str(spec["arm"])] if spec.get("arm", DEFAULT_ARM) != DEFAULT_ARM else []),
    ]
    try:
        completed = run_cell(command, timeout_s)
    except subprocess.TimeoutExpired:
        return _unrunnable(
            cell_id, "TIMEOUT",
            f"exceeded {timeout_s:.0f}s (recorded at "
            f"{float(recorded.get('wall_secs') or 0.0):.0f}s)",
            started, timeout_s)
    except OSError as exc:
        return _unrunnable(cell_id, "ERROR", f"could not start the instrument: {exc}",
                           started, timeout_s)
    if completed.returncode != 0:
        return _unrunnable(cell_id, "ERROR",
                           f"cell run failed (exit {completed.returncode}):\n"
                           f"{completed.stdout}\n{completed.stderr}", started, timeout_s)
    if not output.is_file():
        return _unrunnable(cell_id, "ERROR",
                           "the instrument exited 0 without writing its cell", started, timeout_s)
    try:
        observed = json.loads(output.read_text())
    except json.JSONDecodeError as exc:
        return _unrunnable(cell_id, "ERROR", f"the written cell is not readable json: {exc}",
                           started, timeout_s)

    mismatches: dict[str, object] = {}
    compared_fields: list[str] = []
    for field in COMPARED_FIELDS:
        if field not in recorded:
            continue
        compared_fields.append(field)
        if repr(recorded[field]) != repr(observed.get(field)):
            mismatches[field] = {"recorded": recorded[field], "observed": observed.get(field)}
    # A cell whose fields have all gone — schema drift, a truncated file — would
    # otherwise leave `mismatches` empty and be reported BIT_IDENTICAL: a gate
    # that passed because it compared nothing. `compared_traces` was already
    # disclosed per cell; the fields, which are the actual measurements, were
    # not, so there was no number in the report to notice this by.
    if len(compared_fields) < MIN_COMPARED_FIELDS:
        return _unrunnable(
            cell_id, "ERROR",
            f"only {len(compared_fields)} of {len(COMPARED_FIELDS)} measurements "
            f"are present in the recorded cell (floor {MIN_COMPARED_FIELDS}); "
            "there is not enough of it left to regress against",
            started, timeout_s)
    for trace in COMPARED_TRACES:
        if recorded.get(trace) and observed.get(trace):
            if recorded[trace] != observed[trace]:
                mismatches[trace] = {"recorded": "<trace>", "observed": "<trace differs>"}
    return {
        "cell": cell_id,
        "status": "BIT_IDENTICAL" if not mismatches else "REGRESSION",
        "mismatches": mismatches,
        "wall_secs": round(time.monotonic() - started, 3),
        "timeout_s": timeout_s,
        "compared_fields": compared_fields,
        "compared_traces": [t for t in COMPARED_TRACES if recorded.get(t) and observed.get(t)],
    }


def recorded_rust_cells() -> list[tuple[float, str]]:
    rows = []
    for path in (RESULT_ROOT / "cells").glob("rust__*.json"):
        payload = json.loads(path.read_text())
        rows.append((float(payload.get("wall_secs", 0.0)), path.stem))
    rows.sort()
    return rows


#: Exit codes. A regression and a cell the gate could not run are different
#: facts about the kernel and must not share a code: one says the kernel
#: changed, the other says nobody knows.
EXIT_PASS = 0
EXIT_REGRESSION = 1
EXIT_UNRUNNABLE = 3

RAN = ("BIT_IDENTICAL", "REGRESSION")


def write_report(binary: Path, binary_sha: str, results: list[dict[str, object]],
                 note: str | None = None) -> tuple[Path, int, int]:
    regressions = sum(r["status"] == "REGRESSION" for r in results)
    unrunnable = sum(r["status"] not in RAN for r in results)
    if regressions:
        status = "FAIL"
    elif unrunnable:
        status = "INCOMPLETE"
    else:
        status = "PASS"
    payload = {
        "binary": str(binary),
        "binary_sha256": binary_sha,
        "cells": len(results),
        "compared": sum(r["status"] in RAN for r in results),
        "failures": regressions,
        "unrunnable": unrunnable,
        "status": status,
        "results": results,
    }
    if note:
        payload["note"] = note
    out = RESULT_ROOT / "gate-f-rust"
    out.mkdir(parents=True, exist_ok=True)
    (out / "report.json").write_text(json.dumps(payload, indent=2) + "\n")
    # `report.json` is the latest invocation only, so a later narrow run would
    # otherwise silently destroy the evidence a wider earlier run produced —
    # which is exactly the kind of quiet record loss this harness exists to
    # prevent. Every run is also appended here, keyed by binary hash.
    with (out / "runs.jsonl").open("a") as history:
        history.write(json.dumps(payload) + "\n")
    return out / "report.json", regressions, unrunnable


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    parser.add_argument("--cell", action="append", default=[])
    parser.add_argument("--cheapest", type=int, help="regress the N fastest recorded cells")
    parser.add_argument("--all", action="store_true")
    parser.add_argument("--binary", type=Path, default=DEFAULT_BINARY)
    parser.add_argument(
        "--timeout-factor", type=float, default=TIMEOUT_FACTOR,
        help=f"multiple of each cell's recorded wall clock (default {TIMEOUT_FACTOR:g})")
    parser.add_argument(
        "--timeout-floor", type=float, default=TIMEOUT_FLOOR_S,
        help=f"lower bound on any cell's budget, seconds (default {TIMEOUT_FLOOR_S:g})")
    args = parser.parse_args(argv)

    if args.timeout_factor <= 0 or args.timeout_floor <= 0:
        parser.error("timeout factor and floor must be positive; there is no "
                     "spelling of this gate that waits forever")

    cells = list(args.cell)
    if args.cheapest:
        cells += [name for _, name in recorded_rust_cells()[: args.cheapest]]
    if args.all:
        cells += [name for _, name in recorded_rust_cells()]
    cells = list(dict.fromkeys(cells))
    if not cells:
        parser.error("pass --cell, --cheapest N, or --all")
    if not args.binary.is_file():
        parser.error(f"no rust binary at {args.binary}")

    # Hash before the first cell as well as after the last. A 13-cell sweep runs
    # for minutes, and `cargo build` during it would silently swap the binary
    # between cells — leaving a report that attributes one hash to results
    # produced by two different kernels. That is precisely the provenance
    # confusion this gate exists to detect, so it must not be able to originate
    # here. Compared at the end; a difference is fatal, not a warning.
    binary_sha_before = sha256_file(args.binary)

    print(f"binary: {args.binary}")
    print(f"sha256: {binary_sha_before}")
    print(f"cells:  {len(cells)}")
    print(f"budget: max({args.timeout_floor:.0f}s, {args.timeout_factor:g}x "
          f"each cell's recorded wall clock)\n")
    results: list[dict[str, object]] = []
    note = None
    try:
        for cell_id in cells:
            result = regress_cell(cell_id, args.binary,
                                  args.timeout_factor, args.timeout_floor)
            results.append(result)
            marker = {"BIT_IDENTICAL": "  ok ", "REGRESSION": " FAIL"}.get(
                result["status"], " ????")
            traces = (f" +{len(result['compared_traces'])} traces"
                      if result["compared_traces"] else "")
            print(f"[{marker}] {cell_id}  ({result['wall_secs']:.0f}s){traces}")
            if result["status"] not in RAN:
                print(f"           {result['status']}: {result['detail']}")
            for field, delta in result.get("mismatches", {}).items():
                print(f"           {field}: recorded={delta['recorded']} "
                      f"observed={delta['observed']}")
    except KeyboardInterrupt:
        # Still write what was learned. Losing the verdicts for the cells that
        # did run, because a later one was interrupted, is the record loss the
        # history file exists to prevent.
        note = (f"interrupted after {len(results)} of {len(cells)} cells; "
                "the remainder were never attempted")
        print(f"\ninterrupted — recording {len(results)} of {len(cells)} cells")

    binary_sha_after = sha256_file(args.binary)
    if binary_sha_after != binary_sha_before:
        # Record before raising: these results are unattributable, and a report
        # that says so is worth more than no report.
        write_report(args.binary, binary_sha_before, results,
                     note="binary changed mid-run; results span two kernels")
        raise RuntimeError(
            "the binary changed while the gate was running:\n"
            f"  before {binary_sha_before}\n  after  {binary_sha_after}\n"
            "These results span two kernels and cannot be attributed to either. "
            "Re-run the gate without rebuilding."
        )

    report, regressions, unrunnable = write_report(
        args.binary, binary_sha_before, results, note)
    compared = len(results) - unrunnable
    print(f"\n{compared - regressions}/{compared} bit-identical", end="")
    if unrunnable:
        print(f", {unrunnable} could not run", end="")
    if note:
        print(f", {len(cells) - len(results)} never attempted", end="")
    print(f" -> {'PASS' if not (regressions or unrunnable or note) else 'FAIL'}")
    print(f"report: {report}")
    if regressions:
        return EXIT_REGRESSION
    if unrunnable or note:
        return EXIT_UNRUNNABLE
    return EXIT_PASS


if __name__ == "__main__":
    sys.exit(main())
