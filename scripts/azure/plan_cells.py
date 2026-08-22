#!/usr/bin/env python3
"""Materialize the preregistered d32/L4 scope campaign.

Cell construction and cost ordering delegate to the existing attention-campaign
owner in ``scripts.aws.plan_cells``.  Azure changes scheduling, never experiment
semantics.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent
if str(ROOT) not in sys.path:
    sys.path.insert(0, str(ROOT))

from scripts.aws.plan_cells import SEEDS, cell, estimated_seconds

PROTOCOL = "azure-d32l4-scope-v1"
NODE_COUNT = 4
CONTRACTS = ("published-2ms", "published-10ms", "fixed-t100", "fixed-t250", "fixed-t500")
WIDTHS = (128, 256, 512, 1024)
BUDGETS = (200, 400)


def planned_cells() -> list[dict]:
    cells: list[dict] = []

    # Primary replication at both budgets, then width scope at the binding e400
    # budget. Both arms are re-measured on Azure; the cross-machine gate forbids
    # importing an AWS/macOS control.
    for hidden in WIDTHS:
        budgets = BUDGETS if hidden == 128 else (400,)
        for epochs in budgets:
            for seed in SEEDS:
                cells.append(cell("az8wid", "ff+fixed", hidden, epochs, seed))
                cells.append(cell("az8wid", "ff+fixed+attn", hidden, epochs, seed,
                                  attn_dim=32, attn_layers=4))

    # Binding geometry scope at the anchor width.
    for epochs in (400,):
        for seed in SEEDS:
            cells.append(cell("az8geo", "ff+fixed", 128, epochs, seed,
                              geometry="channels-700"))
            cells.append(cell("az8geo", "ff+fixed+attn", 128, epochs, seed,
                              geometry="channels-700", attn_dim=32, attn_layers=4))

    # Timing-contract scope. The anchor cells above are reused, never repeated.
    for contract in CONTRACTS[1:]:
        for epochs in (400,):
            for seed in SEEDS:
                cells.append(cell("az8con", "ff+fixed", 128, epochs, seed,
                                  contract=contract))
                cells.append(cell("az8con", "ff+fixed+attn", 128, epochs, seed,
                                  contract=contract, attn_dim=32, attn_layers=4))

    # Registered bottleneck diagnostic at h1024/e400. The d32 and rate-only
    # controls already exist in az8wid. d128 was removed before launch because
    # its measured-cost extrapolation alone consumed 36% of the Azure credit.
    for seed in SEEDS:
        cells.append(cell("az8dim", "ff+fixed+attn", 1024, 400, seed,
                          attn_dim=64, attn_layers=4))

    ids = [entry["id"] for entry in cells]
    if len(ids) != len(set(ids)):
        raise RuntimeError("campaign cell ids are not injective")

    # Strict longest-processing-time order is load-bearing on four large nodes:
    # starting short anchors first leaves the h1024/d64 cells as a paid tail.
    # The anchor is still present and completes early relative to the full
    # campaign; only dispatch order changes, never experiment semantics.
    cells.sort(key=lambda entry: (-estimated_seconds(entry), entry["id"]))
    return cells


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", default="-")
    args = parser.parse_args()
    cells = planned_cells()
    payload = json.dumps(cells, indent=2) + "\n"
    if args.out == "-":
        sys.stdout.write(payload)
    else:
        with open(args.out, "w", encoding="utf-8") as handle:
            handle.write(payload)
        print(f"{PROTOCOL}: {len(cells)} cells -> {args.out}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
