#!/usr/bin/env python3
"""Emit one line per campaign event: progress, named failures, fleet, key cells.

# Why this replaces a shell loop

The monitor this supersedes hardcoded `TOTAL=192` when it started, which was the
plan at that moment. Wave 21 appended 168 cells and nothing told it, so it went
on counting toward 192 -- and it would have printed `WAVES 18-20 COMPLETE` and
**exited** at exactly the moment wave 21 began, ending the watch on a campaign
47% unfinished.

The first shell rewrite fixed the denominator by re-reading the published plan,
and then got the numerator wrong the same way: it counted every object under
`results/`, which holds cells from all twenty-one waves, and reported `done 525`
against a plan of 360. Both numbers have to come from the same plan.

So: the plan is re-read every poll, and progress is the intersection of that
plan with what has landed. Nothing here is remembered from start-up.

    python3 scripts/aws/watch_campaign.py --bucket BUCKET [--cell ID ...]
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
import time

#: Matches every other helper in this directory.
AWS_TIMEOUT_S = 300


def aws(args: list[str]) -> str:
    """Best effort: a transient API failure must not kill a long watch."""
    try:
        done = subprocess.run(["aws"] + args, capture_output=True, text=True,
                              timeout=AWS_TIMEOUT_S)
        return done.stdout if done.returncode == 0 else ""
    except (subprocess.TimeoutExpired, OSError):
        return ""


def keys(bucket: str, prefix: str, suffix: str) -> set[str]:
    out = aws(["s3", "ls", f"s3://{bucket}/{prefix}", "--recursive"])
    return {line.split()[-1].rsplit("/", 1)[-1][:-len(suffix)]
            for line in out.splitlines()
            if line.strip() and line.split()[-1].endswith(suffix)}


def plan_ids(bucket: str) -> set[str]:
    raw = aws(["s3", "cp", f"s3://{bucket}/input/cells.json", "-"])
    try:
        return {c["id"] for c in json.loads(raw)}
    except (ValueError, KeyError, TypeError):
        return set()


def instances(region: str) -> int | None:
    out = aws(["ec2", "describe-instances", "--region", region,
               "--filters", "Name=instance-state-name,Values=running,pending",
               "--query", "length(Reservations[].Instances[])",
               "--output", "text"]).strip()
    return int(out) if out.isdigit() else None


def say(line: str) -> None:
    print(line, flush=True)


def main() -> int:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--bucket", required=True)
    p.add_argument("--region", default="us-east-1")
    p.add_argument("--interval", type=int, default=300)
    p.add_argument("--step", type=int, default=8,
                   help="report progress every N newly finished cells")
    p.add_argument("--cell", action="append", default=[],
                   help="a cell whose completion or loss is worth its own line")
    p.add_argument("--once", action="store_true", help="one poll, then exit")
    args = p.parse_args()

    seen_fail: set[str] = set()
    seen_waves: set[str] = set()
    settled: set[str] = set()
    last_done = None
    last_inst = None
    last_plan = None
    down_since = None

    while True:
        plan = plan_ids(args.bucket)
        if not plan:
            # "could not read the plan" must not look like "no work left".
            say("WARN: the published plan could not be read this poll")
            if args.once:
                return 1
            time.sleep(args.interval)
            continue

        results = keys(args.bucket, "results/", ".json")
        failures = keys(args.bucket, "failures/", ".log")
        # Both numbers from the same plan. `results/` holds every wave the
        # bucket has ever run, so counting it raw reports more finished cells
        # than the plan contains.
        done = plan & results
        failed = plan & failures
        inst = instances(args.region)

        if last_plan is None:
            say(f"watching {len(plan)} planned cells: {len(done)} done, "
                f"{len(failed)} failed, {len(plan - done - failed)} outstanding, "
                f"{inst if inst is not None else '?'} instances. "
                f"Plan and progress are both re-read every poll.")
            seen_fail = set(failed)
            seen_waves = {c.split("__")[0] for c in done}
            last_done = len(done)

        for cell in sorted(failed - seen_fail):
            say(f"FAILURE: {cell}  ({len(failed)} of {len(plan)} planned, "
                f"at {len(done)} done)")
        seen_fail = set(failed)

        for cell in args.cell:
            if cell in settled:
                continue
            if cell in results:
                settled.add(cell)
                say(f"KEY CELL COMPLETE: {cell}")
            elif cell in failures:
                settled.add(cell)
                say(f"KEY CELL LOST: {cell}")

        waves = {c.split("__")[0] for c in done}
        for wave in sorted(waves - seen_waves):
            say(f"WAVE STARTED LANDING: {wave}")
        seen_waves = waves

        if last_plan is not None and len(plan) != last_plan:
            say(f"PLAN CHANGED: {last_plan} -> {len(plan)} cells published")
        if last_inst is not None and inst is not None and inst != last_inst:
            say(f"FLEET: {last_inst} -> {inst} instance(s) at "
                f"{len(done)}/{len(plan)}")

        # An idle fleet with work left is the one state where silence is the
        # wrong output, and it is what the previous monitor's single
        # "FLEET DOWN" line could say only once. Repeat it every poll until it
        # is untrue: a watch that has gone quiet and a fleet that has gone away
        # must never look the same.
        outstanding = plan - done - failed
        if inst == 0 and outstanding:
            if down_since is None:
                down_since = time.monotonic()
            mins = int((time.monotonic() - down_since) / 60)
            say(f"FLEET DOWN {mins} min with {len(outstanding)} cell(s) "
                f"unfinished — nothing is claiming work")
        else:
            down_since = None

        if last_done is not None and len(done) >= last_done + args.step:
            say(f"progress {len(done)}/{len(plan)}, {len(failed)} failed, "
                f"{len(outstanding)} outstanding, "
                f"{inst if inst is not None else '?'} instances")
            last_done = len(done)

        if not outstanding:
            say(f"PLAN COMPLETE: {len(done)} done, {len(failed)} failed, "
                f"of {len(plan)} planned")
            return 0

        last_plan, last_inst = len(plan), inst if inst is not None else last_inst
        if args.once:
            return 0
        time.sleep(args.interval)


if __name__ == "__main__":
    sys.exit(main())
