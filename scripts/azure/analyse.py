#!/usr/bin/env python3
"""Apply the frozen AZ8 verdicts to a downloaded Azure campaign."""

from __future__ import annotations

import argparse
import json
import statistics
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent
if str(ROOT) not in sys.path:
    sys.path.insert(0, str(ROOT))

from scripts.azure.plan_cells import NODE_COUNT


def validity_problems(cell: dict) -> list[str]:
    problems = []
    if cell.get("mechanical_status") != "COMPLETE":
        problems.append(f"mechanical_status={cell.get('mechanical_status')}")
    for field, expected in (("non_finite_events", 0), ("classes_predicted", 20)):
        if cell.get(field) != expected:
            problems.append(f"{field}={cell.get(field)}")
    for field, boundary, relation in (
        ("majority_prediction", 0.30, "lt"),
        ("silent_fraction", 0.95, "le"),
        ("saturated_fraction", 0.05, "le"),
    ):
        value = cell.get(field)
        if not isinstance(value, (int, float)) or (
            relation == "lt" and value >= boundary
        ) or (relation == "le" and value > boundary):
            problems.append(f"{field}={value}")
    return problems


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--plan", required=True)
    parser.add_argument("--results", required=True)
    parser.add_argument("--gates", required=True)
    parser.add_argument("--failures", required=True)
    parser.add_argument("--out")
    args = parser.parse_args()

    plan_entries = json.loads(Path(args.plan).read_text())
    plan = {entry["id"]: entry for entry in plan_entries}
    results_dir = Path(args.results)
    valid: dict[str, dict] = {}
    voided: dict[str, list[str]] = {}
    missing = []
    for cell_id in plan:
        path = results_dir / f"{cell_id}.json"
        if not path.is_file():
            missing.append(cell_id)
            continue
        cell = json.loads(path.read_text())
        problems = validity_problems(cell)
        if problems:
            voided[cell_id] = problems
        else:
            valid[cell_id] = cell

    failure_files = sorted(Path(args.failures).glob("*.json"))
    gate_files = sorted(Path(args.gates).glob("*.json"))
    gates = [json.loads(path.read_text()) for path in gate_files]
    binary_hashes = {gate.get("binary_sha256") for gate in gates}
    gate_ready = len(gates) == NODE_COUNT and len(binary_hashes) == 1

    def selection(*, hidden: int, epochs: int, contract: str = "published-2ms",
                  geometry: str = "adjacent-sum-5", dim: int | None) -> dict[int, float]:
        arm = "ff+fixed" if dim is None else "ff+fixed+attn"
        return {
            spec["seed"]: valid[cell_id]["accuracy"]
            for cell_id, spec in plan.items()
            if cell_id in valid and spec["hidden"] == hidden and spec["epochs"] == epochs
            and spec["contract"] == contract and spec["geometry"] == geometry
            and spec["arm"] == arm and spec["attn_dim"] == dim
        }

    def gain(**setting) -> tuple[float | None, int, int]:
        treatment = selection(dim=setting.pop("dim", 32), **setting)
        control = selection(dim=None, **setting)
        seeds = sorted(set(treatment) & set(control))
        if not seeds:
            return None, 0, 0
        deltas = [treatment[seed] - control[seed] for seed in seeds]
        return statistics.mean(deltas), sum(delta > 0 for delta in deltas), len(seeds)

    lines = ["# Azure d32/L4 scope verdicts", ""]
    lines.append(
        f"Coverage: **{len(valid)} valid / {len(plan)} planned**, "
        f"{len(voided)} voided, {len(failure_files)} failures, {len(missing)} missing."
    )
    lines.append(
        f"Binary/gate provenance: **{'READY' if gate_ready else 'INCOMPLETE'}** "
        f"({len(gates)}/{NODE_COUNT} reports, {len(binary_hashes)} binary hashes)."
    )
    lines.extend(["", "| Hypothesis | Measurement | Verdict |", "|---|---|---|"])

    def paired_row(label: str, setting: dict, criterion: str) -> tuple[float | None, int, int]:
        mean, positive, count = gain(**setting)
        supported = mean is not None and count == 12 and mean >= 0.05 and positive >= 9
        measurement = "incomplete" if mean is None else f"gain {mean:+.4f}; positive {positive}/{count}"
        verdict = "SUPPORTED" if supported else ("NOT SUPPORTED" if count == 12 else "INCOMPLETE")
        lines.append(f"| {label} | {measurement}; {criterion} | **{verdict}** |")
        return mean, positive, count

    h128_e400 = paired_row("AZ8-1 x86 replication", {"hidden": 128, "epochs": 400}, ">=+0.05 and >=9/12")
    h1024_d32 = paired_row("AZ8-2 width scope", {"hidden": 1024, "epochs": 400}, ">=+0.05 and >=9/12")
    paired_row("AZ8-3 geometry scope", {"hidden": 128, "epochs": 400,
               "geometry": "channels-700"}, ">=+0.05 and >=9/12")

    h128_e200 = gain(hidden=128, epochs=200)
    stable_ready = h128_e200[2] == 12 and h128_e400[2] == 12
    change = None if not stable_ready else abs(h128_e400[0] - h128_e200[0])
    stable = change is not None and change < 0.02
    lines.append(
        f"| AZ8-4 budget stability | "
        f"{'incomplete' if change is None else f'abs gain change {change:.4f}'}; <0.02 | "
        f"**{'SUPPORTED' if stable else ('NOT SUPPORTED' if change is not None else 'INCOMPLETE')}** |"
    )

    contract_rows = []
    for contract in ("published-2ms", "published-10ms", "fixed-t100", "fixed-t250", "fixed-t500"):
        contract_rows.append((contract, gain(hidden=128, epochs=400, contract=contract)))
    contract_supported = all(mean is not None and count == 12 and mean >= 0.05 and positive >= 9
                             for _, (mean, positive, count) in contract_rows)
    contract_complete = all(count == 12 for _, (_, _, count) in contract_rows)
    contract_text = "; ".join(
        f"{name}={'?' if mean is None else f'{mean:+.4f} ({positive}/{count})'}"
        for name, (mean, positive, count) in contract_rows
    )
    lines.append(f"| AZ8-5 timing scope | {contract_text} | "
                 f"**{'SUPPORTED' if contract_supported else ('NOT SUPPORTED' if contract_complete else 'INCOMPLETE')}** |")

    h1024_d64 = gain(hidden=1024, epochs=400, dim=64)
    dim_ready = h1024_d32[2] == 12 and h1024_d64[2] == 12
    dim_step = None if not dim_ready else h1024_d64[0] - h1024_d32[0]
    dim_supported = dim_step is not None and dim_step >= 0.02 and h1024_d64[0] >= 0.05
    dim_text = "incomplete" if dim_step is None else f"d64-d32 gain {dim_step:+.4f}; d64 gain {h1024_d64[0]:+.4f}"
    lines.append(f"| AZ8-6 d32 bottleneck | {dim_text} | "
                 f"**{'SUPPORTED' if dim_supported else ('NOT SUPPORTED' if dim_step is not None else 'INCOMPLETE')}** |")

    lines.extend(["", "Gate F licenses absolute comparison with prior machines only when it passes; "
                  "all registered verdicts above are same-binary, same-machine paired contrasts."])
    report = "\n".join(lines) + "\n"
    if args.out:
        Path(args.out).write_text(report)
    print(report, end="")
    complete = len(valid) == len(plan) and not voided and not failure_files and gate_ready
    return 0 if complete else 2


if __name__ == "__main__":
    raise SystemExit(main())
