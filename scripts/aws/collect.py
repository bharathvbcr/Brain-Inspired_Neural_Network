#!/usr/bin/env python3
"""Progress report and result sync for the campaign.

Reconciles what came back against the plan. A cell is only "done" if its result
object exists; a claim without a result is an orphan (its instance was reclaimed
mid-cell) and `--release-orphans` hands it back to the queue.

Also surfaces the cross-machine Gate F verdict from every instance, because a
FAIL changes what the results are allowed to be compared against and must never
be discovered at analysis time.
"""

from __future__ import annotations

import argparse
import json
import subprocess
from collections import Counter
from datetime import datetime, timezone
from pathlib import Path


def aws(*argv, check=True):
    out = subprocess.run(["aws", *argv], capture_output=True, text=True)
    if check and out.returncode != 0:
        raise SystemExit(f"aws {' '.join(argv[:3])} failed:\n{out.stderr.strip()}")
    return json.loads(out.stdout) if out.stdout.strip().startswith(("{", "[")) else out.stdout


def keys(bucket, prefix):
    found, token = set(), None
    while True:
        argv = ["s3api", "list-objects-v2", "--bucket", bucket, "--prefix", prefix]
        if token:
            argv += ["--starting-token", token]
        page = aws(*argv)
        for item in (page or {}).get("Contents", []):
            found.add(item["Key"][len(prefix):])
        token = (page or {}).get("NextToken")
        if not token:
            return found


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--bucket", required=True)
    parser.add_argument("--out", help="download results into this directory")
    parser.add_argument("--release-orphans", action="store_true",
                        help="delete claims with no result so the queue re-issues them")
    parser.add_argument("--orphan-age-mins", type=int, default=960,
                        help="only release claims older than this (default 960 = 16 h, "
                             "longer than the slowest planned cell). A claim younger than "
                             "that is almost certainly a cell in progress, and releasing it "
                             "lets a second worker duplicate the work.")
    args = parser.parse_args()

    # `aws()` already parses anything that looks like JSON, and the plan is a
    # top-level list, so it comes back parsed. Parsing it twice raised TypeError.
    plan = aws("s3", "cp", f"s3://{args.bucket}/input/cells.json", "-")
    if isinstance(plan, str):
        plan = json.loads(plan)
    done = {k[:-5] for k in keys(args.bucket, "results/") if k.endswith(".json")}
    held = keys(args.bucket, "claims/")
    failed = {k[:-4] for k in keys(args.bucket, "failures/") if k.endswith(".log")}
    planned = {c["id"] for c in plan}

    in_flight = held - done - failed
    # An age guard, because "claimed but no result" is the normal state of every
    # cell currently running. Releasing those does not lose work - the holder
    # still finishes and uploads - but it lets a second worker start the same
    # cell, which is pure waste on a fleet that is already the bottleneck.
    cutoff_age = args.orphan_age_mins * 60
    ages = {}
    listing = aws("s3api", "list-objects-v2", "--bucket", args.bucket, "--prefix", "claims/")
    for item in (listing or {}).get("Contents", []):
        stamp = datetime.fromisoformat(item["LastModified"].replace("Z", "+00:00"))
        ages[item["Key"][len("claims/"):]] = (
            datetime.now(timezone.utc) - stamp
        ).total_seconds()
    orphaned = sorted(c for c in in_flight if ages.get(c, 0.0) > cutoff_age)

    per_wave = Counter(c["wave"] for c in plan)
    done_wave = Counter(c["wave"] for c in plan if c["id"] in done)
    print(f"{'wave':<8}{'done':>7}{'planned':>9}")
    for wave in sorted(per_wave):
        print(f"{wave:<8}{done_wave[wave]:>7}{per_wave[wave]:>9}")
    print(f"{'TOTAL':<8}{len(done):>7}{len(plan):>9}   "
          f"({100 * len(done) / max(len(plan), 1):.0f}%)")
    print(f"\nin flight: {len(in_flight)}  "
          f"(of which older than {args.orphan_age_mins} min, i.e. releasable: {len(orphaned)})")
    print(f"failed:                {len(failed)}")
    unknown = done - planned
    if unknown:
        print(f"WARNING: {len(unknown)} results are not in the plan: {sorted(unknown)[:3]}")

    gates = keys(args.bucket, "gates/")
    print("\ncross-machine Gate F")
    if not gates:
        print("  no instance has reported yet")
    for name in sorted(g for g in gates if g.endswith(".json")):
        payload = aws("s3", "cp", f"s3://{args.bucket}/gates/{name}", "-")
        if isinstance(payload, str):
            payload = json.loads(payload)
        print(f"  {payload['instance']}  {payload['cross_machine_gate_f']}  {payload['uname']}")

    # A campaign whose cells came from more than one binary is not a single
    # experiment, and nothing downstream would notice on its own. Two binaries
    # can behave identically - the Gate F logs are the evidence for that, not an
    # assumption - but the check has to be visible either way.
    binaries = set()
    for name in sorted(g for g in gates if g.endswith(".json")):
        payload = aws("s3", "cp", f"s3://{args.bucket}/gates/{name}", "-")
        if isinstance(payload, str):
            payload = json.loads(payload)
        binaries.add(payload["binary_sha256"])
    if len(binaries) > 1:
        print(f"\nWARNING: {len(binaries)} distinct binaries have run cells: "
              f"{sorted(b[:12] for b in binaries)}")
        print("  Attribute every result to its binary before analysing, or re-run "
              "the minority under one binary.")
    elif binaries:
        print(f"\nsingle binary across the campaign: {next(iter(binaries))[:12]}")

    if args.release_orphans and orphaned:
        for cid in orphaned:
            aws("s3", "rm", f"s3://{args.bucket}/claims/{cid}", check=False)
        print(f"\nreleased {len(orphaned)} claims older than "
              f"{args.orphan_age_mins} min back to the queue")

    if args.out:
        target = Path(args.out)
        target.mkdir(parents=True, exist_ok=True)
        aws("s3", "sync", f"s3://{args.bucket}/results/", str(target), "--quiet")
        print(f"\nsynced {len(list(target.glob('*.json')))} cells -> {target}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
