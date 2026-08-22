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
            # Passed even though the weight file already carries the arm: the
            # instrument cross-checks the two and refuses a disagreement, so
            # naming it here turns a mismatched artifact into an error instead
            # of a silently different cell.
            *(["--arm", str(spec["arm"])] if spec.get("arm", DEFAULT_ARM) != DEFAULT_ARM else []),
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
