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


def parse_cell_id(cell_id: str) -> dict[str, object]:
    backend, contract, geometry, hidden, epochs, seed = cell_id.split("__")
    return {
        "backend": backend,
        "contract": contract,
        "geometry": geometry,
        "hidden": int(hidden.removeprefix("h")),
        "epochs": int(epochs.removeprefix("e")),
        "seed": int(seed.removeprefix("s")),
    }


def initialization_paths(spec: dict[str, object]) -> tuple[Path, Path]:
    n_inputs = 700 if spec["geometry"] == "channels-700" else 140
    weights = RESULT_ROOT / "initialization" / f"n{n_inputs}-h{spec['hidden']}-s{spec['seed']}.weights"
    orders = RESULT_ROOT / "initialization" / f"n8156-e100-s{spec['seed']}.orders"
    return weights, orders


def regress_cell(cell_id: str, binary: Path) -> dict[str, object]:
    recorded_path = RESULT_ROOT / "cells" / f"{cell_id}.json"
    if not recorded_path.is_file():
        raise FileNotFoundError(f"no completed cell to regress against: {cell_id}")
    spec = parse_cell_id(cell_id)
    if spec["backend"] != "rust":
        raise ValueError("this gate regresses the rust arm; use gates_ef.py for python")
    recorded = json.loads(recorded_path.read_text())
    weights, orders = initialization_paths(spec)
    for path in (weights, orders):
        if not path.is_file():
            raise FileNotFoundError(f"missing initialization artifact: {path}")

    output = RESULT_ROOT / "gate-f-rust" / f"{cell_id}.json"
    output.parent.mkdir(parents=True, exist_ok=True)
    started = time.monotonic()
    completed = subprocess.run(
        [
            str(binary), "train-cell",
            "--train-events", str(EVENT_ROOT / "train.events"),
            "--test-events", str(EVENT_ROOT / "test.events"),
            "--contract", str(spec["contract"]),
            "--geometry", str(spec["geometry"]),
            "--weights", str(weights),
            "--orders", str(orders),
            "--epochs", str(spec["epochs"]),
            "--out", str(output),
        ],
        capture_output=True,
        text=True,
    )
    if completed.returncode != 0:
        raise RuntimeError(f"cell run failed:\n{completed.stdout}\n{completed.stderr}")
    observed = json.loads(output.read_text())

    mismatches: dict[str, object] = {}
    for field in COMPARED_FIELDS:
        if field not in recorded:
            continue
        if repr(recorded[field]) != repr(observed.get(field)):
            mismatches[field] = {"recorded": recorded[field], "observed": observed.get(field)}
    for trace in COMPARED_TRACES:
        if recorded.get(trace) and observed.get(trace):
            if recorded[trace] != observed[trace]:
                mismatches[trace] = {"recorded": "<trace>", "observed": "<trace differs>"}
    return {
        "cell": cell_id,
        "status": "BIT_IDENTICAL" if not mismatches else "REGRESSION",
        "mismatches": mismatches,
        "wall_secs": round(time.monotonic() - started, 3),
        "compared_traces": [t for t in COMPARED_TRACES if recorded.get(t) and observed.get(t)],
    }


def recorded_rust_cells() -> list[tuple[float, str]]:
    rows = []
    for path in (RESULT_ROOT / "cells").glob("rust__*.json"):
        payload = json.loads(path.read_text())
        rows.append((float(payload.get("wall_secs", 0.0)), path.stem))
    rows.sort()
    return rows


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    parser.add_argument("--cell", action="append", default=[])
    parser.add_argument("--cheapest", type=int, help="regress the N fastest recorded cells")
    parser.add_argument("--all", action="store_true")
    parser.add_argument("--binary", type=Path, default=DEFAULT_BINARY)
    args = parser.parse_args(argv)

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
    print(f"cells:  {len(cells)}\n")
    results = []
    for cell_id in cells:
        result = regress_cell(cell_id, args.binary)
        results.append(result)
        marker = "  ok " if result["status"] == "BIT_IDENTICAL" else " FAIL"
        traces = f" +{len(result['compared_traces'])} traces" if result["compared_traces"] else ""
        print(f"[{marker}] {cell_id}  ({result['wall_secs']:.0f}s){traces}")
        for field, delta in result["mismatches"].items():
            print(f"           {field}: recorded={delta['recorded']} observed={delta['observed']}")

    binary_sha_after = sha256_file(args.binary)
    if binary_sha_after != binary_sha_before:
        raise RuntimeError(
            "the binary changed while the gate was running:\n"
            f"  before {binary_sha_before}\n  after  {binary_sha_after}\n"
            "These results span two kernels and cannot be attributed to either. "
            "Re-run the gate without rebuilding."
        )

    failures = sum(r["status"] != "BIT_IDENTICAL" for r in results)
    payload = {
        "binary": str(args.binary),
        "binary_sha256": binary_sha_before,
        "cells": len(results),
        "failures": failures,
        "status": "PASS" if failures == 0 else "FAIL",
        "results": results,
    }
    report = RESULT_ROOT / "gate-f-rust" / "report.json"
    report.parent.mkdir(parents=True, exist_ok=True)
    report.write_text(json.dumps(payload, indent=2) + "\n")
    # `report.json` is the latest invocation only, so a later narrow run would
    # otherwise silently destroy the evidence a wider earlier run produced —
    # which is exactly the kind of quiet record loss this harness exists to
    # prevent. Every run is also appended here, keyed by binary hash.
    with (RESULT_ROOT / "gate-f-rust" / "runs.jsonl").open("a") as history:
        history.write(json.dumps(payload) + "\n")
    print(f"\n{len(results) - failures}/{len(results)} bit-identical -> "
          f"{'PASS' if failures == 0 else 'FAIL'}")
    print(f"report: {report}")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
