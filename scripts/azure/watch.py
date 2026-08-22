#!/usr/bin/env python3
"""Read-only VM and cell progress for the Azure campaign."""

from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent
if str(ROOT) not in sys.path:
    sys.path.insert(0, str(ROOT))

from scripts.azure.launch import CONTAINER, DEFAULT_NODES, RG, STORAGE, VMSS
from scripts.azure.plan_cells import planned_cells


def az(*args: str, check: bool = True):
    result = subprocess.run(["az", *args], check=check, capture_output=True, text=True)
    if result.returncode != 0:
        return None
    return json.loads(result.stdout) if result.stdout.strip() else None


def main() -> int:
    exists = az("vmss", "show", "--resource-group", RG, "--name", VMSS,
                "--query", "id", check=False)
    instances = [] if not exists else (
        az("vmss", "list-instances", "--resource-group", RG, "--name", VMSS,
           "--expand", "instanceView",
           "--query", "[].{id:instanceId,state:instanceView.statuses[-1].displayStatus}") or []
    )
    blobs = az("storage", "blob", "list", "--account-name", STORAGE,
               "--container-name", CONTAINER, "--auth-mode", "key",
               "--query", "[].name") or []
    results = [name for name in blobs if name.startswith("results/") and name.endswith(".json")]
    failures = [name for name in blobs if name.startswith("failures/") and name.endswith(".json")]
    gates = [name for name in blobs if name.startswith("gates/") and name.endswith(".json")]
    print("instances:", ", ".join(f"{row['id']}={row['state']}" for row in instances) or "none")
    print(f"cells: {len(results)}/{len(planned_cells())} complete; "
          f"{len(failures)} failures; {len(gates)}/{DEFAULT_NODES} gate reports")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
