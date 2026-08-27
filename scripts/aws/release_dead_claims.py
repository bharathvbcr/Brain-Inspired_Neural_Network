#!/usr/bin/env python3
"""Release claims no worker is actually running.

A claim marker is created when a worker takes a cell and removed only by
finishing it, so a killed worker leaves its claim behind and that cell is never
re-issued. An age guard alone cannot tell an orphan from a cell that is simply
slow - the slowest planned cell here runs 14 hours - so this asks the fleet
instead: every running worker's command line carries `--out .../<cell id>/cell.json`,
which is the definitive list of what is genuinely in progress.

Releases exactly: claimed, not finished, and not running anywhere.
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import time


#: Seconds any single `aws` control-plane call may take. These are describe /
#: list / put calls against the AWS API, not training runs: one that has not
#: answered in five minutes is wedged, not slow, and without a bound a stalled
#: connection hangs the campaign silently — the shape that left GC4 blocked for
#: two days.
#:
#: The constant is repeated in each `scripts/aws` helper rather than shared.
#: A shared module would work (bootstrap.sh ships the whole tree as
#: `source.tar.gz`), but `scripts/test_campaign_tooling.py` fakes these calls by
#: assigning `module.subprocess.run`, which only reaches a helper that calls
#: subprocess itself. `test_campaign_tooling.py` pins that every copy agrees.
AWS_TIMEOUT_S = 300


def aws(*argv, check=True):
    try:
        out = subprocess.run(["aws", *argv], capture_output=True, text=True,
                             timeout=AWS_TIMEOUT_S)
    except subprocess.TimeoutExpired:
        raise SystemExit(
            f"aws {' '.join(argv[:3])} did not answer in {AWS_TIMEOUT_S}s; "
            "treating a wedged call as a failure rather than waiting"
        ) from None
    if check and out.returncode != 0:
        raise SystemExit(f"aws {' '.join(argv[:3])} failed:\n{out.stderr.strip()}")
    text = out.stdout.strip()
    return json.loads(text) if text.startswith(("{", "[")) else text


def keys(bucket: str, prefix: str) -> set[str]:
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


def live_cells(region: str) -> set[str]:
    """Cell ids currently being run, taken from live process command lines."""
    ids: set[str] = set()
    instances = aws("ec2", "describe-instances", "--region", region,
                    "--filters", "Name=tag:Project,Values=binn-campaign",
                    "Name=instance-state-name,Values=running",
                    "--query", "Reservations[].Instances[].InstanceId",
                    "--output", "text").split()
    for instance in instances:
        command = aws("ssm", "send-command", "--region", region,
                      "--instance-ids", instance,
                      "--document-name", "AWS-RunShellScript",
                      "--parameters", 'commands=["ps -eo args | grep [s]hd-instrument | tr \' \' \'\\n\' | grep cell.json || true"]',
                      "--query", "Command.CommandId", "--output", "text", check=False)
        if not command:
            print(f"  {instance}: unreachable - treating all its claims as LIVE (safe)")
            return set()  # refuse to release anything on incomplete information
        time.sleep(7)
        out = aws("ssm", "get-command-invocation", "--region", region,
                  "--command-id", command, "--instance-id", instance,
                  "--query", "StandardOutputContent", "--output", "text", check=False)
        for line in str(out).splitlines():
            match = re.search(r"/tmp/([^/]+)/cell\.json", line.strip())
            if match:
                ids.add(match.group(1))
        print(f"  {instance}: {len(ids)} live cells seen so far")
    return ids


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--bucket", required=True)
    parser.add_argument("--region", default="us-east-1")
    parser.add_argument("--apply", action="store_true", help="without this, only reports")
    args = parser.parse_args()

    done = {k[:-5] for k in keys(args.bucket, "results/") if k.endswith(".json")}
    # A cell with a failure log RAN. It reached the trainer and the trainer
    # reached a definite answer -- on 2026-08-27, five `rec+alif` cells whose
    # loss went non-finite mid-training. Before this set was subtracted, every
    # one of those looked exactly like a cell no worker had ever touched:
    # claimed, no result, not running. A dry run on 2026-08-27 called 22 of
    # them orphaned, 17 from waves 11-14 that finished weeks ago.
    #
    # Releasing them re-queues a deterministic run that already has its answer.
    # It cannot produce a different one -- the seed is pinned and the binary is
    # pinned -- so it burns the slot and lands back in `failures/`. Worse, it
    # makes a diverged cell and an interrupted cell indistinguishable, and only
    # the interrupted one may be re-run.
    failed = {k[:-4] for k in keys(args.bucket, "failures/") if k.endswith(".log")}
    held = keys(args.bucket, "claims/")
    print(f"claims held: {len(held)}, results: {len(done)}, failures: {len(failed)}")
    print("asking the fleet what is actually running")
    live = live_cells(args.region)
    if not live:
        print("no live cell list could be built; releasing nothing")
        return 1

    dead = sorted(held - done - failed - live)
    print(f"\nlive: {len(live)}   finished: {len(held & done)}   "
          f"failed: {len(held & failed)}   orphaned: {len(dead)}")
    for cid in dead[:10]:
        print(f"  {cid[:76]}")
    if len(dead) > 10:
        print(f"  ... and {len(dead) - 10} more")

    if not args.apply:
        print("\ndry run; pass --apply to release")
        return 0
    for cid in dead:
        aws("s3", "rm", f"s3://{args.bucket}/claims/{cid}", check=False)
    print(f"\nreleased {len(dead)} orphaned claims back to the queue")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
