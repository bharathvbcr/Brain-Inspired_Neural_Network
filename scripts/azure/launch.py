#!/usr/bin/env python3
"""Prepare and optionally launch the bounded Azure VMSS BINN campaign.

The default prepares immutable inputs only. ``--launch`` creates the managed
identity and private four-node scale set, grants two narrowly scoped roles, and
scales to the requested capacity.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import shlex
import subprocess
import sys
import tarfile
import tempfile
import time
import urllib.parse
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent
if str(ROOT) not in sys.path:
    sys.path.insert(0, str(ROOT))

from scripts.aws.plan_cells import estimated_seconds
from scripts.azure.plan_cells import NODE_COUNT
from scripts.azure.run_shard import shard, simulate_schedule

SUBSCRIPTION = "63bfe60c-4632-40e9-a2a9-5ddf6a9f28ac"
RG = "rg-binn-paper-cpu-20260820"
STORAGE = "binncpu28ac20260820"
CONTAINER = "azure-d32l4-scope-v1"
VMSS = "vmss-binn-paper-cpu"
IDENTITY = "id-binn-paper-cpu"
VNET = "vnet-binn-paper-cpu"
SUBNET = "snet-workers"
NSG = "nsg-binn-paper-cpu"
NAT_GATEWAY = "nat-binn-paper-cpu"
NAT_PUBLIC_IP = "pip-binn-paper-cpu"
DEFAULT_SKU = "Standard_D64als_v7"
DEFAULT_NODES = NODE_COUNT
DEFAULT_HOURS = 19.0
DEFAULT_HOST_CORES = 64
DEFAULT_NORMAL_THREADS = 4
DEFAULT_WIDE_THREADS = 8
DEFAULT_CONCURRENCY = 16
COMPUTE_BUDGET_USD = 196.0
CAMPAIGN_INFRA_BUDGET_USD = 197.0
NAT_GATEWAY_HOURLY_USD = 0.045
PUBLIC_IP_HOURLY_USD = 0.005
MEASURED_THREAD_SPEEDUP = {4: 3.38, 8: 5.72}
# Exact one-epoch h128/d32/L4 run on the campaign code: 75.731654 seconds
# before batch-scoped weight layouts, 72.525924 seconds after. The scientific
# fields were bit-identical. This scales the older estimator derived from the
# pre-optimisation binary; it is not claimed as an Azure hardware benchmark.
MEASURED_KERNEL_RUNTIME_FACTOR = 72.525924 / 75.731654

EXCLUDE_DIRS = {"target", ".git", "data", "viz", "hybrid-results"}
EXCLUDE_PREFIXES = (
    ".venv", "results/shd_attention_pilot_v1", "results/shd_attention_campaign_v1",
    "results/runs",
)


def az(*argv: str, parse: bool = True, check: bool = True):
    result = subprocess.run(["az", *argv], capture_output=True, text=True)
    if check and result.returncode != 0:
        raise RuntimeError(f"az {' '.join(argv[:4])} failed:\n{result.stderr.strip()}")
    if not parse:
        return result
    return json.loads(result.stdout) if result.stdout.strip() else {}


def build_tarball(path: Path) -> str:
    def keep(name: str) -> bool:
        relative = name[2:] if name.startswith("./") else name
        if relative.split("/", 1)[0] in EXCLUDE_DIRS:
            return False
        return not relative.startswith(EXCLUDE_PREFIXES)

    with tarfile.open(path, "w:gz") as archive:
        for item in sorted(ROOT.iterdir()):
            if keep(item.name):
                archive.add(item, arcname=item.name,
                            filter=lambda info: info if keep(info.name) else None)
    return hashlib.sha256(path.read_bytes()).hexdigest()


def retail_hourly(region: str, sku: str) -> float:
    query = (
        f"serviceName eq 'Virtual Machines' and armRegionName eq '{region}' "
        f"and armSkuName eq '{sku}' and priceType eq 'Consumption'"
    )
    url = "https://prices.azure.com/api/retail/prices?" + urllib.parse.urlencode({"$filter": query})
    with urllib.request.urlopen(url, timeout=30) as response:
        items = json.load(response)["Items"]
    candidates = [item for item in items
                  if "Windows" not in item["productName"]
                  and "Spot" not in item["meterName"]
                  and "Low Priority" not in item["meterName"]
                  and item["type"] == "Consumption"]
    if len(candidates) != 1:
        raise RuntimeError(f"expected one dedicated Linux price, got {len(candidates)}")
    return float(candidates[0]["retailPrice"])


def upload(path: Path, name: str) -> None:
    az("storage", "blob", "upload", "--account-name", STORAGE,
       "--container-name", CONTAINER, "--name", name, "--file", str(path),
       "--auth-mode", "key", "--overwrite", "true", "--output", "none",
       parse=False)


def prepare_inputs(temp: Path, args: argparse.Namespace, hourly: float) -> tuple[Path, str, Path]:
    az("storage", "container", "create", "--account-name", STORAGE,
       "--name", CONTAINER, "--auth-mode", "key", "--public-access", "off",
       "--output", "none", parse=False)
    plan = temp / "cells.json"
    subprocess.run([sys.executable, str(ROOT / "scripts/azure/plan_cells.py"),
                    "--out", str(plan)], cwd=ROOT, check=True)
    source = temp / "source.tar.gz"
    source_sha = build_tarball(source)
    manifest = temp / "manifest.json"
    manifest.write_text(json.dumps({
        "protocol": "azure-d32l4-scope-v1",
        "source_sha256": source_sha,
        "plan_sha256": hashlib.sha256(plan.read_bytes()).hexdigest(),
        "cells": len(json.loads(plan.read_text())),
        "preregistration": "results/PREREG_2026-08-20_AZURE_D32L4_SCOPE.md",
        "infrastructure": {
            "region": args.region,
            "sku": args.sku,
            "nodes": args.nodes,
            "normal_threads_per_cell": args.threads_per_cell,
            "wide_threads_per_cell": args.wide_threads_per_cell,
            "wide_cell_rule": "hidden>=1024 and attn_dim>=64",
            "max_concurrent_cells_per_node": args.concurrent_cells,
            "host_cores_per_node": DEFAULT_HOST_CORES,
            "compile_target_cpu": "x86-64-v4",
            "gate_policy": {
                "cross_platform_expected_status": "FAIL",
                "azure_local_quorum": f"{args.nodes}/{args.nodes}",
                "scientific_fields_must_be_bit_identical": True,
            },
            "hourly_price_per_node_usd": hourly,
            "watchdog_hours": args.hours,
            "outbound_nat_gateway": NAT_GATEWAY if args.enable_outbound_nat else None,
            "outbound_public_ip": NAT_PUBLIC_IP if args.enable_outbound_nat else None,
        },
    }, indent=2, sort_keys=True) + "\n")
    inputs = {
        "input/source.tar.gz": source,
        "input/cells.json": plan,
        "input/manifest.json": manifest,
        "input/train.events": ROOT / "data/shd/events/train.events",
        "input/test.events": ROOT / "data/shd/events/test.events",
    }
    for name, path in inputs.items():
        if not path.is_file():
            raise RuntimeError(f"required input missing: {path}")
        upload(path, name)
    return plan, source_sha, manifest


def ensure_role(principal: str, role: str, scope: str) -> None:
    # Fresh managed identities can take minutes to appear in Microsoft Graph.
    # The object-id form is authoritative here and avoids an unnecessary Graph
    # name lookup while preserving exact-assignee idempotency.
    existing = az(
        "role", "assignment", "list",
        "--assignee-object-id", principal,
        "--fill-principal-name", "false",
        "--role", role, "--scope", scope, "--query", "[].id",
    )
    if existing:
        return
    az("role", "assignment", "create", "--assignee-object-id", principal,
       "--assignee-principal-type", "ServicePrincipal", "--role", role,
       "--scope", scope, "--output", "none", parse=False)


def ensure_outbound_nat(region: str) -> None:
    """Attach explicit outbound-only Internet access to the worker subnet."""
    public_ip = az(
        "network", "public-ip", "show", "--resource-group", RG,
        "--name", NAT_PUBLIC_IP, check=False,
    )
    if not public_ip:
        az(
            "network", "public-ip", "create", "--resource-group", RG,
            "--name", NAT_PUBLIC_IP, "--location", region,
            "--allocation-method", "Static", "--sku", "Standard",
            "--version", "IPv4", "--tags", "project=binn", "purpose=paper-cpu-egress",
            "--output", "none", parse=False,
        )
    nat_gateway = az(
        "network", "nat", "gateway", "show", "--resource-group", RG,
        "--name", NAT_GATEWAY, check=False,
    )
    if not nat_gateway:
        az(
            "network", "nat", "gateway", "create", "--resource-group", RG,
            "--name", NAT_GATEWAY, "--location", region,
            "--public-ip-addresses", NAT_PUBLIC_IP, "--sku", "Standard",
            "--idle-timeout", "4", "--tags", "project=binn", "purpose=paper-cpu-egress",
            "--output", "none", parse=False,
        )
    az(
        "network", "vnet", "subnet", "update", "--resource-group", RG,
        "--vnet-name", VNET, "--name", SUBNET,
        "--nat-gateway", NAT_GATEWAY, "--default-outbound", "false",
        "--output", "none", parse=False,
    )


def launch(args: argparse.Namespace, bootstrap: Path) -> None:
    identity = az("identity", "show", "--name", IDENTITY, "--resource-group", RG,
                  check=False)
    if not identity:
        identity = az("identity", "create", "--name", IDENTITY, "--resource-group", RG,
                      "--location", args.region)
    principal = identity["principalId"]
    identity_id = identity["id"]
    container_scope = (
        f"/subscriptions/{SUBSCRIPTION}/resourceGroups/{RG}/providers/Microsoft.Storage/"
        f"storageAccounts/{STORAGE}/blobServices/default/containers/{CONTAINER}"
    )
    ensure_role(principal, "Storage Blob Data Contributor", container_scope)

    az("network", "nsg", "create", "--resource-group", RG, "--name", NSG,
       "--location", args.region, "--tags", "project=binn", "purpose=paper-cpu",
       "--output", "none", parse=False)
    az("network", "vnet", "create", "--resource-group", RG, "--name", VNET,
       "--location", args.region, "--address-prefix", "10.42.0.0/16",
       "--subnet-name", SUBNET, "--subnet-prefix", "10.42.1.0/24",
       "--network-security-group", NSG, "--tags", "project=binn", "purpose=paper-cpu",
       "--output", "none", parse=False)
    if args.enable_outbound_nat:
        ensure_outbound_nat(args.region)

    vmss = az("vmss", "show", "--resource-group", RG, "--name", VMSS, check=False)
    if not vmss:
        with tempfile.TemporaryDirectory(prefix="binn-azure-key-") as key_dir:
            private_key = Path(key_dir) / "id_ed25519"
            subprocess.run(["ssh-keygen", "-q", "-t", "ed25519", "-N", "",
                            "-f", str(private_key)], check=True)
            az("vmss", "create", "--resource-group", RG, "--name", VMSS,
               "--location", args.region, "--image", "Ubuntu2204",
               "--vm-sku", args.sku, "--instance-count", "0",
               "--orchestration-mode", "Uniform", "--upgrade-policy-mode", "manual",
               "--disable-overprovision", "--admin-username", "azureuser",
               "--authentication-type", "ssh", "--ssh-key-values", str(private_key) + ".pub",
               "--assign-identity", identity_id, "--vnet-name", VNET, "--subnet", SUBNET,
               "--nsg", NSG, "--public-ip-address", "", "--load-balancer", "",
               "--storage-sku", "Standard_LRS", "--os-disk-size-gb", "64",
               "--custom-data", str(bootstrap), "--tags", "project=binn",
               "purpose=paper-cpu", "expires=2026-08-24", "--output", "none", parse=False)
        vmss = az("vmss", "show", "--resource-group", RG, "--name", VMSS)

    ensure_role(principal, "Virtual Machine Contributor", vmss["id"])
    # Allow RBAC propagation before cloud-init tries its first storage read.
    time.sleep(30)
    az("vmss", "scale", "--resource-group", RG, "--name", VMSS,
       "--new-capacity", str(args.nodes), "--output", "none", parse=False)
    print(f"launched {args.nodes} x {args.sku} in {args.region}")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--region", default="eastus")
    parser.add_argument("--sku", default=DEFAULT_SKU)
    parser.add_argument("--nodes", type=int, default=DEFAULT_NODES)
    parser.add_argument("--hours", type=float, default=DEFAULT_HOURS)
    parser.add_argument("--threads-per-cell", type=int, default=DEFAULT_NORMAL_THREADS)
    parser.add_argument("--wide-threads-per-cell", type=int, default=DEFAULT_WIDE_THREADS)
    parser.add_argument("--concurrent-cells", type=int, default=DEFAULT_CONCURRENCY)
    parser.add_argument("--cell-timeout-secs", type=int, default=86_400)
    parser.add_argument("--enable-outbound-nat", action="store_true")
    parser.add_argument("--launch", action="store_true")
    args = parser.parse_args()
    if args.nodes < 1 or args.hours <= 0:
        parser.error("nodes and hours must be positive")
    if args.sku != DEFAULT_SKU or args.nodes != DEFAULT_NODES:
        parser.error(
            f"this frozen campaign supports exactly {DEFAULT_NODES} x {DEFAULT_SKU}"
        )
    if (args.threads_per_cell, args.wide_threads_per_cell, args.concurrent_cells) != (
        DEFAULT_NORMAL_THREADS,
        DEFAULT_WIDE_THREADS,
        DEFAULT_CONCURRENCY,
    ):
        parser.error("this frozen campaign requires the validated 4/8-thread, 16-cell policy")
    hourly = retail_hourly(args.region, args.sku)
    compute = hourly * args.nodes * args.hours
    if compute > COMPUTE_BUDGET_USD:
        parser.error(f"compute estimate ${compute:.2f} exceeds ${COMPUTE_BUDGET_USD:.2f} cap")
    network = args.hours * (NAT_GATEWAY_HOURLY_USD + PUBLIC_IP_HOURLY_USD)
    if args.enable_outbound_nat and compute + network > CAMPAIGN_INFRA_BUDGET_USD:
        parser.error(
            f"compute plus fixed network estimate ${compute + network:.2f} exceeds "
            f"${CAMPAIGN_INFRA_BUDGET_USD:.2f} cap"
        )
    if args.launch and not args.enable_outbound_nat:
        parser.error("launch requires --enable-outbound-nat for the private worker subnet")
    print(f"retail compute: {args.nodes} x ${hourly:.3f}/h x {args.hours:.1f}h = ${compute:.2f}")
    if args.enable_outbound_nat:
        print(f"fixed outbound network ceiling: ${network:.2f} plus data processing")
    if args.launch and az("vmss", "show", "--resource-group", RG, "--name", VMSS,
                          check=False):
        parser.error(
            "the campaign VMSS already exists; refusing to replace frozen inputs or custom data. "
            "Use watch.py, deallocate.py, or an explicit az vmss scale command."
        )

    with tempfile.TemporaryDirectory(prefix="binn-azure-launch-") as directory:
        temp = Path(directory)
        plan, source_sha, manifest = prepare_inputs(temp, args, hourly)
        cells = json.loads(plan.read_text())
        node_estimates = []
        for node_index in range(args.nodes):
            schedule = simulate_schedule(
                shard(cells, node_index, args.nodes),
                host_cores=DEFAULT_HOST_CORES,
                max_concurrency=args.concurrent_cells,
                normal_threads=args.threads_per_cell,
                wide_threads=args.wide_threads_per_cell,
                duration_seconds=lambda cell, threads: (
                    estimated_seconds(cell)
                    * MEASURED_KERNEL_RUNTIME_FACTOR
                    / MEASURED_THREAD_SPEEDUP[threads]
                ),
            )
            node_estimates.append(schedule)
        estimated_wall = max(item["elapsed_secs"] for item in node_estimates) / 3600
        if estimated_wall > args.hours * 0.95:
            parser.error(
                f"measured-cost estimate {estimated_wall:.1f}h leaves less than 5% of "
                f"the {args.hours:.1f}h watchdog; reduce the matrix before spending"
            )
        print(f"prepared {len(cells)} cells; estimated {estimated_wall:.1f}h; "
              f"source sha256 {source_sha[:16]}...; {manifest.name} uploaded")
        if not args.launch:
            print("prepared only: no identity, role assignment, or VM was created")
            return 0
        prefix = {
            "STORAGE_ACCOUNT": STORAGE, "CONTAINER": CONTAINER,
            "NODE_COUNT": str(args.nodes), "VMSS_NAME": VMSS,
            "RESOURCE_GROUP": RG, "SUBSCRIPTION_ID": SUBSCRIPTION,
            "SOURCE_SHA256": source_sha,
            "THREADS_PER_CELL": str(args.threads_per_cell),
            "WIDE_THREADS_PER_CELL": str(args.wide_threads_per_cell),
            "CONCURRENT_CELLS": str(args.concurrent_cells),
            "CELL_TIMEOUT_SECS": str(args.cell_timeout_secs),
            "MAX_RUNTIME_SECONDS": str(int(args.hours * 3600)),
        }
        bootstrap = temp / "bootstrap.sh"
        exports = "".join(f"export {key}={shlex.quote(value)}\n" for key, value in prefix.items())
        bootstrap.write_text("#!/usr/bin/env bash\n" + exports
                             + (ROOT / "scripts/azure/bootstrap.sh").read_text())
        launch(args, bootstrap)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
