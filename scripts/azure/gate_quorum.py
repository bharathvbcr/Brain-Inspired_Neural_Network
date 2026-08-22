#!/usr/bin/env python3
"""Build and validate the fail-closed Azure-local Gate F quorum."""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parent.parent.parent
if str(ROOT) not in sys.path:
    sys.path.insert(0, str(ROOT))

from scripts.gate_f_rust import COMPARED_FIELDS, COMPARED_TRACES


SCHEMA = "azure-local-gate-f-v1"


class QuorumError(RuntimeError):
    """Base class for unusable quorum evidence."""


class QuorumPending(QuorumError):
    """The complete current-launch report set has not arrived yet."""


class QuorumMismatch(QuorumError):
    """Current-launch reports disagree or fail structural validation."""


def _canonical_sha(payload: object) -> str:
    encoded = json.dumps(
        payload, sort_keys=True, separators=(",", ":"), allow_nan=False,
    ).encode()
    return hashlib.sha256(encoded).hexdigest()


def _read_object(path: Path) -> dict[str, Any]:
    payload = json.loads(path.read_text())
    if not isinstance(payload, dict):
        raise ValueError(f"expected a JSON object in {path}")
    return payload


def build_attestation(
    *,
    node: int,
    source_sha256: str,
    binary_sha256: str,
    gate_report: Path,
    gate_output_dir: Path,
    utc: str,
) -> dict[str, object]:
    """Bind all observed Gate F fields to this source, binary, and node."""
    report = _read_object(gate_report)
    if report.get("binary_sha256") != binary_sha256:
        raise QuorumMismatch("Gate F report binary hash does not match the pinned binary")
    status = report.get("status")
    if status not in {"PASS", "FAIL"}:
        raise QuorumMismatch(f"invalid cross-platform Gate F status: {status!r}")
    results = report.get("results")
    if not isinstance(results, list) or not results:
        raise QuorumMismatch("Gate F report has no cell results")
    if report.get("cells") != len(results):
        raise QuorumMismatch("Gate F report cell count does not match its result list")
    failures = report.get("failures")
    if not isinstance(failures, int) or not 0 <= failures <= len(results):
        raise QuorumMismatch("Gate F report has an invalid failure count")
    if (status == "PASS") != (failures == 0):
        raise QuorumMismatch("Gate F status and failure count disagree")

    observations: list[dict[str, object]] = []
    seen: set[str] = set()
    for entry in results:
        if not isinstance(entry, dict) or not isinstance(entry.get("cell"), str):
            raise QuorumMismatch("Gate F result has no valid cell identifier")
        cell = entry["cell"]
        if cell in seen:
            raise QuorumMismatch(f"duplicate Gate F cell: {cell}")
        seen.add(cell)
        observed = _read_object(gate_output_dir / f"{cell}.json")
        missing = [field for field in COMPARED_FIELDS if field not in observed]
        if missing:
            raise QuorumMismatch(f"{cell} is missing scientific fields: {missing}")
        fields = {field: observed[field] for field in COMPARED_FIELDS}
        traces = {trace: observed[trace] for trace in COMPARED_TRACES if trace in observed}
        observations.append({"cell": cell, "fields": fields, "traces": traces})
    observations.sort(key=lambda item: str(item["cell"]))

    return {
        "schema": SCHEMA,
        "node": node,
        "source_sha256": source_sha256,
        "binary_sha256": binary_sha256,
        "cross_platform_gate_f": status,
        "gate_cells": len(observations),
        "gate_failures": failures,
        "observation_sha256": _canonical_sha(observations),
        "observed_results": observations,
        "utc": utc,
    }


def validate_quorum(
    reports_dir: Path,
    *,
    node_count: int,
    source_sha256: str,
    binary_sha256: str,
    expected_cross_platform_status: str,
    expected_gate_cells: int,
) -> dict[str, object]:
    """Require complete, current, bit-identical observations from every node."""
    reports: list[dict[str, Any]] = []
    for node in range(node_count):
        path = reports_dir / f"node-{node}.json"
        if not path.is_file():
            raise QuorumPending(f"node {node} report has not arrived")
        try:
            report = _read_object(path)
        except (OSError, ValueError, json.JSONDecodeError) as error:
            raise QuorumPending(f"node {node} report is not readable yet: {error}") from error
        if report.get("schema") != SCHEMA or report.get("source_sha256") != source_sha256:
            raise QuorumPending(f"node {node} report belongs to an earlier launch")
        reports.append(report)

    observation_hashes: set[str] = set()
    for node, report in enumerate(reports):
        if report.get("node") != node:
            raise QuorumMismatch(f"node {node} report identifies node {report.get('node')!r}")
        if report.get("binary_sha256") != binary_sha256:
            raise QuorumMismatch(f"node {node} used a different binary")
        if report.get("cross_platform_gate_f") != expected_cross_platform_status:
            raise QuorumMismatch(
                f"node {node} cross-platform status is "
                f"{report.get('cross_platform_gate_f')!r}, expected "
                f"{expected_cross_platform_status!r}"
            )
        observations = report.get("observed_results")
        if not isinstance(observations, list) or not observations:
            raise QuorumMismatch(f"node {node} has no normalized observations")
        if report.get("gate_cells") != expected_gate_cells or len(observations) != expected_gate_cells:
            raise QuorumMismatch(
                f"node {node} has {len(observations)} Gate F cells, expected {expected_gate_cells}"
            )
        seen_cells: set[str] = set()
        for observation in observations:
            if not isinstance(observation, dict):
                raise QuorumMismatch(f"node {node} has a malformed observation")
            cell = observation.get("cell")
            fields = observation.get("fields")
            traces = observation.get("traces")
            if not isinstance(cell, str) or cell in seen_cells:
                raise QuorumMismatch(f"node {node} has a missing or duplicate cell identity")
            seen_cells.add(cell)
            if not isinstance(fields, dict) or set(fields) != set(COMPARED_FIELDS):
                raise QuorumMismatch(f"node {node} cell {cell} has an incomplete field set")
            if not isinstance(traces, dict) or not set(traces).issubset(COMPARED_TRACES):
                raise QuorumMismatch(f"node {node} cell {cell} has an invalid trace set")
        try:
            actual_hash = _canonical_sha(observations)
        except (TypeError, ValueError) as error:
            raise QuorumMismatch(f"node {node} observations are not canonical JSON: {error}") from error
        if report.get("observation_sha256") != actual_hash:
            raise QuorumMismatch(f"node {node} observation digest is invalid")
        observation_hashes.add(actual_hash)
    if len(observation_hashes) != 1:
        raise QuorumMismatch("Azure Gate F scientific fields differ across nodes")

    return {
        "schema": SCHEMA,
        "status": "PASS",
        "nodes": list(range(node_count)),
        "source_sha256": source_sha256,
        "binary_sha256": binary_sha256,
        "cross_platform_gate_f": expected_cross_platform_status,
        "observation_sha256": observation_hashes.pop(),
    }


def _write_json(path: Path, payload: object) -> None:
    path.write_text(json.dumps(payload, indent=2, sort_keys=True, allow_nan=False) + "\n")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    subparsers = parser.add_subparsers(dest="command", required=True)
    attest = subparsers.add_parser("attest")
    attest.add_argument("--node", type=int, required=True)
    attest.add_argument("--source-sha256", required=True)
    attest.add_argument("--binary-sha256", required=True)
    attest.add_argument("--gate-report", type=Path, required=True)
    attest.add_argument("--gate-output-dir", type=Path, required=True)
    attest.add_argument("--utc", required=True)
    attest.add_argument("--out", type=Path, required=True)
    validate = subparsers.add_parser("validate")
    validate.add_argument("--reports-dir", type=Path, required=True)
    validate.add_argument("--node-count", type=int, required=True)
    validate.add_argument("--source-sha256", required=True)
    validate.add_argument("--binary-sha256", required=True)
    validate.add_argument("--expected-cross-platform-status", choices=("PASS", "FAIL"), required=True)
    validate.add_argument("--expected-gate-cells", type=int, required=True)
    validate.add_argument("--out", type=Path, required=True)
    args = parser.parse_args(argv)
    try:
        if args.command == "attest":
            payload = build_attestation(
                node=args.node,
                source_sha256=args.source_sha256,
                binary_sha256=args.binary_sha256,
                gate_report=args.gate_report,
                gate_output_dir=args.gate_output_dir,
                utc=args.utc,
            )
        else:
            payload = validate_quorum(
                args.reports_dir,
                node_count=args.node_count,
                source_sha256=args.source_sha256,
                binary_sha256=args.binary_sha256,
                expected_cross_platform_status=args.expected_cross_platform_status,
                expected_gate_cells=args.expected_gate_cells,
            )
        _write_json(args.out, payload)
        return 0
    except QuorumPending as error:
        print(f"PENDING: {error}", file=sys.stderr)
        return 3
    except (QuorumMismatch, OSError, ValueError, json.JSONDecodeError) as error:
        print(f"MISMATCH: {error}", file=sys.stderr)
        return 4


if __name__ == "__main__":
    raise SystemExit(main())
