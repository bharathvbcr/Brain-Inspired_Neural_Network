#!/usr/bin/env python3
"""Run one deterministic shard of a campaign and persist every outcome.

The destination is a two-method seam -- `exists` and `put` -- so the same
scheduler, the same per-cell timeout, the same failure persistence and the same
`scripts/aws/run_cell.py` invocation serve a VMSS node writing to Blob Storage
and a single workstation writing to a directory. Wave 29 runs on the latter
(`AMENDMENT_2026-09-07_WAVE_29_RUNS_ON_THE_LOCAL_PLATFORM.md`); standing up a
second runner beside this one would have meant a second scheduler to keep
correct, and the first-fit core-token loop below is the part that is easy to get
wrong.
"""

from __future__ import annotations

import argparse
import copy
import concurrent.futures
import heapq
import json
import os
import shutil
import subprocess
import time
import urllib.error
import urllib.parse
import urllib.request
from pathlib import Path

STORAGE_RESOURCE = "https://storage.azure.com/"
API_VERSION = "2023-11-03"


class ScheduleError(RuntimeError):
    """The declared per-cell resources cannot be scheduled on the host."""


def shard(cells: list[dict], index: int, count: int) -> list[dict]:
    if count < 1 or index < 0 or index >= count:
        raise ValueError(f"invalid shard {index}/{count}")
    return [entry for position, entry in enumerate(cells) if position % count == index]


def thread_count_for_cell(spec: dict, normal_threads: int, wide_threads: int) -> int:
    """Reserve extra cores only for the registered h1024/d64 tail cells."""
    if normal_threads < 1 or wide_threads < 1:
        raise ScheduleError("normal and wide thread counts must be positive")
    if int(spec.get("hidden", 0)) >= 1024 and int(spec.get("attn_dim") or 0) >= 64:
        return wide_threads
    return normal_threads


def _next_fitting_index(
    pending: list[dict], available_cores: int, normal_threads: int, wide_threads: int
) -> int | None:
    for index, spec in enumerate(pending):
        if thread_count_for_cell(spec, normal_threads, wide_threads) <= available_cores:
            return index
    return None


def simulate_schedule(
    cells: list[dict],
    *,
    host_cores: int,
    max_concurrency: int,
    normal_threads: int,
    wide_threads: int,
    duration_seconds,
) -> dict:
    """Simulate the same first-fit core-token scheduler used by a VMSS node."""
    if host_cores < 1 or max_concurrency < 1:
        raise ScheduleError("host cores and max concurrency must be positive")
    pending = list(cells)
    for spec in pending:
        threads = thread_count_for_cell(spec, normal_threads, wide_threads)
        if threads > host_cores:
            raise ScheduleError(
                f"cell {spec.get('id', '<unknown>')} requests {threads} threads "
                f"on a {host_cores}-core host"
            )

    now = 0.0
    available = host_cores
    sequence = 0
    running: list[tuple[float, int, int]] = []
    peak_reserved = 0
    peak_running = 0
    core_seconds = 0.0
    while pending or running:
        while len(running) < max_concurrency:
            index = _next_fitting_index(pending, available, normal_threads, wide_threads)
            if index is None:
                break
            spec = pending.pop(index)
            threads = thread_count_for_cell(spec, normal_threads, wide_threads)
            duration = float(duration_seconds(spec, threads))
            if not duration >= 0.0:
                raise ScheduleError(f"invalid duration for cell {spec.get('id', '<unknown>')}")
            available -= threads
            core_seconds += duration * threads
            heapq.heappush(running, (now + duration, sequence, threads))
            sequence += 1
            peak_reserved = max(peak_reserved, host_cores - available)
            peak_running = max(peak_running, len(running))

        if not running:
            if pending:
                raise ScheduleError("pending cells cannot fit the available host resources")
            break
        now = running[0][0]
        while running and running[0][0] == now:
            _, _, threads = heapq.heappop(running)
            available += threads

    return {
        "elapsed_secs": now,
        "peak_reserved_cores": peak_reserved,
        "peak_running_cells": peak_running,
        "reserved_core_seconds": core_seconds,
        "scheduling_core_utilization": core_seconds / (now * host_cores) if now else 0.0,
    }


class BlobClient:
    def __init__(self, account: str, container: str) -> None:
        self.base = f"https://{account}.blob.core.windows.net/{container}"
        self.token = ""
        self.expires = 0

    def _auth(self) -> str:
        if time.time() + 300 < self.expires:
            return self.token
        query = urllib.parse.urlencode({"api-version": "2018-02-01", "resource": STORAGE_RESOURCE})
        request = urllib.request.Request(
            f"http://169.254.169.254/metadata/identity/oauth2/token?{query}",
            headers={"Metadata": "true"},
        )
        with urllib.request.urlopen(request, timeout=15) as response:
            payload = json.load(response)
        self.token = payload["access_token"]
        self.expires = int(payload["expires_on"])
        return self.token

    def _url(self, name: str) -> str:
        return f"{self.base}/{urllib.parse.quote(name, safe='/')}"

    def exists(self, name: str) -> bool:
        request = urllib.request.Request(
            self._url(name), method="HEAD",
            headers={"Authorization": f"Bearer {self._auth()}", "x-ms-version": API_VERSION},
        )
        try:
            with urllib.request.urlopen(request, timeout=30):
                return True
        except urllib.error.HTTPError as error:
            if error.code == 404:
                return False
            raise

    def put(self, name: str, data: bytes) -> None:
        request = urllib.request.Request(
            self._url(name), data=data, method="PUT",
            headers={
                "Authorization": f"Bearer {self._auth()}",
                "x-ms-version": API_VERSION,
                "x-ms-blob-type": "BlockBlob",
                "Content-Length": str(len(data)),
            },
        )
        with urllib.request.urlopen(request, timeout=120):
            pass


class LocalStore:
    """`BlobClient`'s contract against a directory, for a run with no cloud.

    The blob layout is preserved verbatim -- `results/`, `logs/`, `failures/`,
    `summaries/` under one root -- so a local run and a fleet run produce the
    same tree and `collect.py`'s notion of "landing" stays a separate, deliberate
    step. A local run that wrote straight into a corpus directory would make
    "the wave ran" and "the wave landed" the same event, and landing is the
    moment a freeze changes.
    """

    def __init__(self, root: str) -> None:
        self.root = Path(root)

    def _path(self, name: str) -> Path:
        # `name` is built from a cell id, which `plan_cells.py` assembles from a
        # fixed vocabulary; refuse anything that could climb out of the root
        # rather than trusting that to stay true.
        candidate = (self.root / name).resolve()
        if not candidate.is_relative_to(self.root.resolve()):
            raise ValueError(f"{name!r} escapes the store root")
        return candidate

    def exists(self, name: str) -> bool:
        return self._path(name).is_file()

    def put(self, name: str, data: bytes) -> None:
        path = self._path(name)
        path.parent.mkdir(parents=True, exist_ok=True)
        # Write-then-rename: an interrupted run leaves no half-written cell for
        # the next pass to skip as "already done".
        temporary = path.with_suffix(path.suffix + ".partial")
        temporary.write_bytes(data)
        temporary.replace(path)


def run_one(spec: dict, args: argparse.Namespace, blobs: BlobClient | LocalStore) -> dict:
    cell_id = spec["id"]
    if blobs.exists(f"results/{cell_id}.json"):
        return {
            "id": cell_id,
            "status": "SKIPPED_EXISTING",
            "wall_secs": 0.0,
            "node": args.node_index,
            "threads": args.threads,
        }

    work = Path(args.work_root) / cell_id
    work.mkdir(parents=True, exist_ok=True)
    log_path = work / "cell.log"
    command = [
        "python3", "scripts/aws/run_cell.py", cell_id,
        "--plan", args.plan, "--work", str(work), "--binary", args.binary,
        "--threads", str(args.threads), "--events", args.events,
    ]
    status = "FAILED"
    started = time.monotonic()
    try:
        with log_path.open("wb") as log:
            completed = subprocess.run(
                command, stdout=log, stderr=subprocess.STDOUT,
                timeout=args.cell_timeout_secs, check=False,
            )
        if completed.returncode == 0 and (work / "cell.json").is_file():
            blobs.put(f"results/{cell_id}.json", (work / "cell.json").read_bytes())
            status = "COMPLETE"
        else:
            status = f"EXIT_{completed.returncode}"
    except subprocess.TimeoutExpired:
        status = "TIMEOUT"
        with log_path.open("ab") as log:
            log.write(f"\nTIMEOUT after {args.cell_timeout_secs} seconds\n".encode())
    except Exception as error:  # recorded and surfaced; never converted to a pass
        status = f"RUNNER_ERROR:{type(error).__name__}:{error}"
        with log_path.open("ab") as log:
            log.write(("\n" + status + "\n").encode())

    if log_path.exists():
        blobs.put(f"logs/{cell_id}.log", log_path.read_bytes())
    outcome = {
        "id": cell_id,
        "status": status,
        "wall_secs": time.monotonic() - started,
        "node": args.node_index,
        "threads": args.threads,
    }
    if status != "COMPLETE":
        blobs.put(f"failures/{cell_id}.json", (json.dumps(outcome, sort_keys=True) + "\n").encode())
    shutil.rmtree(work, ignore_errors=True)
    return outcome


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--node-index", type=int, required=True)
    parser.add_argument("--node-count", type=int, required=True)
    parser.add_argument("--plan", default="cells.json")
    parser.add_argument("--binary", required=True)
    parser.add_argument("--events", default="data/shd/events")
    parser.add_argument("--threads", type=int, default=4)
    parser.add_argument("--wide-threads", type=int, default=8)
    parser.add_argument("--concurrency", type=int, default=16)
    parser.add_argument("--cell-timeout-secs", type=int, default=57_600)
    parser.add_argument("--work-root", default="/tmp/binn-cells")
    parser.add_argument("--storage-account")
    parser.add_argument("--container")
    parser.add_argument("--results-dir",
                        help="write the blob layout to this directory instead of "
                             "Blob Storage, for a run with no cloud behind it")
    args = parser.parse_args()

    # Exactly one destination. Defaulting to either would mean a mistyped
    # `--storage-account` silently writes a campaign to the local disk, or a
    # local run silently tries to reach an instance metadata endpoint that is
    # not there and reports every cell as a RUNNER_ERROR.
    azure = bool(args.storage_account or args.container)
    if azure and args.results_dir:
        parser.error("--results-dir and --storage-account/--container are two "
                     "destinations; give one")
    if azure and not (args.storage_account and args.container):
        parser.error("--storage-account and --container are given together")
    if not azure and not args.results_dir:
        parser.error("give a destination: --results-dir, or "
                     "--storage-account with --container")

    if args.threads < 1 or args.wide_threads < 1 or args.concurrency < 1:
        parser.error("thread counts and concurrency must be positive")
    if args.node_count < 1 or args.node_index < 0 or args.node_index >= args.node_count:
        parser.error(f"invalid shard {args.node_index}/{args.node_count}")
    host_cores = os.cpu_count() or 1
    if max(args.threads, args.wide_threads) > host_cores:
        parser.error(f"a cell requests more than the host's {host_cores} CPUs")

    cells = json.loads(Path(args.plan).read_text())
    selected = shard(cells, args.node_index, args.node_count)
    blobs = (BlobClient(args.storage_account, args.container)
             if azure else LocalStore(args.results_dir))
    print(
        f"node {args.node_index}/{args.node_count}: {len(selected)} cells, "
        f"{host_cores} cores, up to {args.concurrency} cells, "
        f"{args.threads} normal/{args.wide_threads} wide threads",
        flush=True,
    )

    outcomes: list[dict] = []
    pending = list(selected)
    running: dict[concurrent.futures.Future, tuple[dict, int]] = {}
    available_cores = host_cores
    peak_reserved_cores = 0
    peak_running_cells = 0
    node_started = time.monotonic()
    with concurrent.futures.ThreadPoolExecutor(max_workers=args.concurrency) as pool:
        while pending or running:
            while len(running) < args.concurrency:
                index = _next_fitting_index(
                    pending, available_cores, args.threads, args.wide_threads
                )
                if index is None:
                    break
                spec = pending.pop(index)
                threads = thread_count_for_cell(spec, args.threads, args.wide_threads)
                cell_args = copy.copy(args)
                cell_args.threads = threads
                future = pool.submit(run_one, spec, cell_args, blobs)
                running[future] = (spec, threads)
                available_cores -= threads
                peak_reserved_cores = max(peak_reserved_cores, host_cores - available_cores)
                peak_running_cells = max(peak_running_cells, len(running))
                print(f"START {spec['id']}: {threads} threads", flush=True)

            if not running:
                raise ScheduleError("pending cells cannot fit the available host resources")
            completed, _ = concurrent.futures.wait(
                running, return_when=concurrent.futures.FIRST_COMPLETED
            )
            for future in completed:
                _, threads = running.pop(future)
                available_cores += threads
                outcome = future.result()
                outcomes.append(outcome)
                print(f"{outcome['status']}: {outcome['id']}", flush=True)

    elapsed_secs = time.monotonic() - node_started
    reserved_core_seconds = sum(o["wall_secs"] * o["threads"] for o in outcomes)

    summary = {
        "node": args.node_index,
        "planned": len(selected),
        "complete": sum(o["status"] == "COMPLETE" for o in outcomes),
        "skipped_existing": sum(o["status"] == "SKIPPED_EXISTING" for o in outcomes),
        "failed": sum(o["status"] not in {"COMPLETE", "SKIPPED_EXISTING"} for o in outcomes),
        "host_cores": host_cores,
        "normal_threads": args.threads,
        "wide_threads": args.wide_threads,
        "max_concurrency": args.concurrency,
        "peak_reserved_cores": peak_reserved_cores,
        "peak_running_cells": peak_running_cells,
        "elapsed_secs": elapsed_secs,
        "reserved_core_seconds": reserved_core_seconds,
        "scheduling_core_utilization": (
            reserved_core_seconds / (elapsed_secs * host_cores) if elapsed_secs else 0.0
        ),
        "outcomes": sorted(outcomes, key=lambda item: item["id"]),
    }
    blobs.put(
        f"summaries/node-{args.node_index}.json",
        (json.dumps(summary, indent=2, sort_keys=True) + "\n").encode(),
    )
    return 0 if summary["failed"] == 0 else 2


if __name__ == "__main__":
    raise SystemExit(main())
