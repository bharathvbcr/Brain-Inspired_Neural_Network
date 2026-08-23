"""Gate E / Gate F harness for PREREG_2026-08-02_SHD_BPTT_CEILING_RECALIF.

Gate F (regression, runnable today)
    The instrument, with recurrence and adaptation disabled, must reproduce the
    already-completed ff+fixed cells **bit-identically**. This is what licenses
    comparing new arms against the 216 completed rust cells. It is also the
    standing check on the 2026-08-02 kernel vectorisation.

Gate E (four-arm parity, runnable once the arms land)
    Every arm must clear the registered cross-backend tolerances before any
    matrix cell runs: forward <= 1e-6, gradient <= 1e-4, update <= 1e-5.

Neither gate is a result. Failing either blocks the campaign.

Determinism (replaces Gate F after a from-scratch rerun)
    Once the instrument changes, there is no recorded history to regress
    against and Gate F has nothing to compare to. The property that still has
    to hold is the one `fresh_process_replay` already assumes: the same cell,
    run twice in separate processes, must produce byte-identical output.

Usage
    python scripts/gates_ef.py gate-f --cell <cell-id>
    python scripts/gates_ef.py gate-f --all-python
    python scripts/gates_ef.py gate-e                      # fails until arms exist
    python scripts/gates_ef.py determinism --cell <cell-id>
    python scripts/gates_ef.py determinism --cell <cell-id> --smoke   # capped, NOT a gate result
"""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import time

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parent))

from shd_calibration.model import train_cell  # noqa: E402
from shd_calibration.runner import contract_from_id  # noqa: E402

ROOT = Path(__file__).resolve().parent.parent
RESULT_ROOT = ROOT / "results" / "shd_instrument_v4"
EVENT_ROOT = ROOT / "data" / "shd" / "events"
FIXTURES = RESULT_ROOT / "fixtures"

# Registered tolerances. These are the existing calibration gates and are not
# adjustable here; a change requires a new prereg file.
FORWARD_TOL = 1e-6
GRADIENT_TOL = 1e-4
UPDATE_TOL = 1e-5

# Metrics that must reproduce exactly. Every scientific field in the cell
# payload, minus wall_secs (timing is not part of the result).
EXACT_KEYS = (
    "accuracy",
    "classes_predicted",
    "majority_prediction",
    "mean_firing_rate",
    "mean_gradient_norm",
    "mean_loss",
    "mean_update_rms",
    "n_test",
    "n_train",
    "non_finite_events",
    "saturated_fraction",
    "scientific_status",
    "silent_fraction",
)

ARMS = ("ff+fixed", "ff+alif", "rec+fixed", "rec+alif")


def relative_error(expected: np.ndarray, observed: np.ndarray) -> float:
    """Identical to runner.relative_error - kept in sync deliberately."""
    numerator = np.linalg.norm(expected - observed)
    denominator = max(
        float(np.linalg.norm(expected)), float(np.linalg.norm(observed)), 1e-12
    )
    return float(numerator / denominator)


# Both parsers live in `gate_f_rust`, which is the one that gets extended.
#
# This file carried byte-equivalent copies until 2026-08-23, when `gate_f_rust`
# grew optional arm and `d<dim>l<layers>` components so Gate F could express a
# cell on something other than `ff+fixed`. The copy here did not, and a rust id
# with those components raised an unpacking `ValueError` from `split("__")`
# **before** reaching the "gate-f regresses the python arm" message below - an
# obscure failure standing in for a clear one. Importing removes the drift
# rather than re-synchronising it.
#
# For a python spec the behaviour is unchanged: `initialization_paths` appends
# an arm only when one is present and is not the default, and a python spec
# carries no `arm` key at all.
from gate_f_rust import initialization_paths, parse_cell_id  # noqa: E402,F401


def gate_f_cell(cell_id: str) -> dict[str, object]:
    """Re-run one completed python cell and demand bit-identical metrics."""
    recorded_path = RESULT_ROOT / "cells" / f"{cell_id}.json"
    if not recorded_path.is_file():
        raise FileNotFoundError(f"no completed cell to regress against: {cell_id}")
    recorded = json.loads(recorded_path.read_text())
    spec = parse_cell_id(cell_id)
    if spec["backend"] != "python":
        raise ValueError("gate-f regresses the python arm; the rust arm is checked by gate-e")
    weights, orders = initialization_paths(spec)
    for path in (weights, orders):
        if not path.is_file():
            raise FileNotFoundError(f"missing initialization artifact: {path}")

    output = RESULT_ROOT / "gate-f" / f"{cell_id}.json"
    output.parent.mkdir(parents=True, exist_ok=True)
    started = time.monotonic()
    train_cell(
        EVENT_ROOT / "train.events",
        EVENT_ROOT / "test.events",
        contract_from_id(str(spec["contract"])),
        str(spec["geometry"]),
        weights,
        orders,
        int(spec["epochs"]),
        output,
    )
    observed = json.loads(output.read_text())

    mismatches = {
        key: {"recorded": recorded[key], "observed": observed[key]}
        for key in EXACT_KEYS
        if repr(recorded[key]) != repr(observed[key])
    }
    return {
        "cell": cell_id,
        "gate": "F",
        "bit_identical": not mismatches,
        "mismatches": mismatches,
        "recorded_wall_secs": recorded["wall_secs"],
        "observed_wall_secs": observed["wall_secs"],
        "speedup": recorded["wall_secs"] / max(observed["wall_secs"], 1e-9),
        "elapsed_secs": time.monotonic() - started,
    }


def run_cell_once(
    cell_id: str,
    output: Path,
    max_train: int | None = None,
    max_test: int | None = None,
) -> None:
    """Train one cell into `output`. Capped runs synthesise orders (smoke only)."""
    spec = parse_cell_id(cell_id)
    weights, orders = initialization_paths(spec)
    output.parent.mkdir(parents=True, exist_ok=True)
    if max_train is not None:
        # The registered order file is shaped for the full 8156-sample split, so
        # a capped run cannot use it. A seeded stand-in keeps the run
        # reproducible while making it unmistakably not a confirmatory result.
        import numpy as np

        from shd_calibration import model as model_module

        generator = np.random.default_rng(int(spec["seed"]))
        stand_in = np.stack(
            [generator.permutation(max_train) for _ in range(int(spec["epochs"]))]
        ).astype(np.int64)
        model_module.load_orders = lambda _path: stand_in
    train_cell(
        EVENT_ROOT / "train.events",
        EVENT_ROOT / "test.events",
        contract_from_id(str(spec["contract"])),
        str(spec["geometry"]),
        weights,
        orders,
        int(spec["epochs"]),
        output,
        max_train,
        max_test,
    )


def determinism(
    cell_id: str, repeats: int, smoke: bool, in_process: bool
) -> dict[str, object]:
    """Run one cell `repeats` times and demand byte-identical result files.

    By default each repeat runs in a **fresh process**, which is what catches
    interpreter-level nondeterminism (hash seeding, iteration order, allocator
    effects) rather than only algorithmic nondeterminism.
    """
    max_train, max_test = (400, 150) if smoke else (None, None)
    directory = RESULT_ROOT / ("determinism-smoke" if smoke else "determinism")
    directory.mkdir(parents=True, exist_ok=True)

    digests: list[str] = []
    payloads: list[dict] = []
    for repeat in range(repeats):
        output = directory / f"{cell_id}.run{repeat}.json"
        started = time.monotonic()
        if in_process:
            run_cell_once(cell_id, output, max_train, max_test)
        else:
            command = [
                sys.executable,
                str(Path(__file__).resolve()),
                "_run-once",
                "--cell",
                cell_id,
                "--out",
                str(output),
            ]
            if smoke:
                command += ["--max-train", str(max_train), "--max-test", str(max_test)]
            completed = subprocess.run(command, capture_output=True, text=True)
            if completed.returncode != 0:
                raise RuntimeError(
                    f"repeat {repeat} failed:\n{completed.stdout}\n{completed.stderr}"
                )
        raw = output.read_bytes()
        digests.append(hashlib.sha256(raw).hexdigest())
        payload = json.loads(raw)
        payload.pop("wall_secs", None)
        payloads.append(payload)
        print(
            f"  repeat {repeat}: {time.monotonic() - started:6.1f}s  "
            f"accuracy={payload['accuracy']!r}"
        )

    # wall_secs legitimately varies, so identity is judged on everything else.
    reference = payloads[0]
    mismatches: dict[str, object] = {}
    for index, payload in enumerate(payloads[1:], start=1):
        for key in sorted(set(reference) | set(payload)):
            if repr(reference.get(key)) != repr(payload.get(key)):
                mismatches[f"run0_vs_run{index}:{key}"] = {
                    "run0": reference.get(key),
                    f"run{index}": payload.get(key),
                }
    return {
        "schema": "shd-determinism-v1",
        "cell": cell_id,
        "repeats": repeats,
        "mode": "smoke" if smoke else "full",
        "fresh_process": not in_process,
        "result_sha256": digests,
        "deterministic": not mismatches,
        "mismatches": mismatches,
    }


def gate_e() -> dict[str, object]:
    """Four-arm cross-backend parity. Blocks until the arms are implemented."""
    missing = []
    for arm in ARMS:
        fixture = FIXTURES / f"rust-parity-{arm.replace('+', '-')}.json"
        if not fixture.is_file():
            missing.append(fixture.name)
    if missing:
        raise SystemExit(
            "GATE E BLOCKED - no arm fixtures yet.\n"
            "  missing: " + ", ".join(missing) + "\n"
            "  Each fixture is produced by:\n"
            "    target/release/shd-instrument parity --arm <arm> --events "
            "results/shd_instrument_v4/fixtures/events.events --index 3 \\\n"
            "      --contract published-10ms --geometry channels-700 \\\n"
            "      --weights <arm weights> --out <fixture>\n"
            "  The --arm flag and the recurrent/adaptive terms do not exist yet; "
            "see GATE_EF_WORK.md for the exact surface each backend must grow."
        )
    raise SystemExit(
        "GATE E: fixtures present but the python arm implementations are not wired. "
        "Implement loss_and_gradient(arm=...) before running."
    )


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    subparsers = parser.add_subparsers(dest="command", required=True)
    f_parser = subparsers.add_parser("gate-f")
    group = f_parser.add_mutually_exclusive_group(required=True)
    group.add_argument("--cell", type=str)
    group.add_argument("--all-python", action="store_true")
    subparsers.add_parser("gate-e")
    d_parser = subparsers.add_parser("determinism")
    d_parser.add_argument("--cell", type=str, required=True)
    d_parser.add_argument("--repeats", type=int, default=2)
    d_parser.add_argument("--smoke", action="store_true",
                          help="capped split with synthesised orders; NOT a gate result")
    d_parser.add_argument("--in-process", action="store_true",
                          help="skip fresh-process isolation (weaker check)")
    once = subparsers.add_parser("_run-once")
    once.add_argument("--cell", type=str, required=True)
    once.add_argument("--out", type=Path, required=True)
    once.add_argument("--max-train", type=int, default=None)
    once.add_argument("--max-test", type=int, default=None)
    args = parser.parse_args(argv)

    if args.command == "gate-e":
        gate_e()
        return 0

    if args.command == "_run-once":
        run_cell_once(args.cell, args.out, args.max_train, args.max_test)
        return 0

    if args.command == "determinism":
        report = determinism(args.cell, args.repeats, args.smoke, args.in_process)
        suffix = "-smoke" if args.smoke else ""
        out = RESULT_ROOT / f"determinism{suffix}" / "report.json"
        out.write_text(json.dumps(report, indent=2, sort_keys=True))
        if report["deterministic"]:
            print(f"\nDETERMINISTIC across {args.repeats} runs "
                  f"({'fresh processes' if not args.in_process else 'in-process'}) -> {out}")
            if args.smoke:
                print("SMOKE MODE: capped split and synthesised orders. "
                      "This is a pilot, not a gate result.")
            return 0
        print("\nNONDETERMINISTIC - the instrument does not reproduce itself.")
        for key, values in report["mismatches"].items():
            print(f"    {key}: {values}")
        return 1

    if args.all_python:
        cells = sorted(p.stem for p in (RESULT_ROOT / "cells").glob("python__*.json"))
    else:
        cells = [args.cell]

    reports = []
    failed = 0
    for cell_id in cells:
        report = gate_f_cell(cell_id)
        reports.append(report)
        status = "BIT-IDENTICAL" if report["bit_identical"] else "REGRESSION"
        print(
            f"[{status}] {cell_id}  "
            f"{report['recorded_wall_secs']:.0f}s -> {report['observed_wall_secs']:.0f}s "
            f"({report['speedup']:.1f}x)"
        )
        if not report["bit_identical"]:
            failed += 1
            for key, values in report["mismatches"].items():
                print(f"    {key}: recorded={values['recorded']!r} observed={values['observed']!r}")

    out = RESULT_ROOT / "gate-f" / "report.json"
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps({"schema": "shd-gate-f-v1", "cells": reports}, indent=2, sort_keys=True))
    print(f"\nGate F: {len(cells) - failed}/{len(cells)} bit-identical -> {out}")
    if failed:
        print("GATE F FAILED - the campaign is blocked until this reproduces exactly.")
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
