#!/usr/bin/env python3
"""Emit one line per campaign event: progress, failures, gates, stalls, reclaims.

# One watcher, because two is how one of them stays broken

This replaces `scripts/aws/watch.sh`, deleted in the same commit. Both watched
the same campaign and neither knew about the other, and on 2026-08-28 `watch.sh`
was **broken**: it counted every object under `results/` — 616 of them, from all
twenty-one waves the bucket has ever run — against the current plan's 360, so
`settled >= TOTAL` was true on its first poll and it would have printed
`COMPLETE` and exited immediately.

This file had the same defect for exactly one poll, in the opposite direction of
discovery: its first run said `done 525` against a plan of 360, which is what
made the numerator wrong in a way somebody noticed. `watch.sh` expressed the
same arithmetic as a comparison rather than a printed pair, so it failed
silently instead.

Two capabilities came from `watch.sh` and are ported here rather than lost:
Gate F verdicts announced as instances join, and the SSM worker-process count
that distinguishes a long cell from a dead fleet. Its header's hardest-won
lesson — that the plan can grow mid-campaign, so the total is re-read every
cycle and never captured at start-up, which "has now done so twice" — was
rediscovered here a third time before that file was read. It is now written
down in one place.

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
import re
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


def gate_verdicts(bucket: str) -> dict[str, str]:
    """`instance -> its cross-machine Gate F verdict`.

    Every reported wave must carry its instances' Gate F verdict, and instances
    self-terminate, so a gate first seen while watching is worth an event.
    Ported from `watch.sh`, which this file replaces.
    """
    out: dict[str, str] = {}
    listing = aws(["s3", "ls", f"s3://{bucket}/gates/"])
    for line in listing.splitlines():
        key = line.split()[-1] if line.strip() else ""
        if not key.endswith(".json"):
            continue
        try:
            body = json.loads(aws(["s3", "cp", f"s3://{bucket}/gates/{key}", "-"]))
        except ValueError:
            continue
        out[body.get("instance", key[:-5])] = body.get("cross_machine_gate_f", "?")
    return out


def worker_processes(region: str) -> int | None:
    """How many `shd-instrument train-cell` processes the fleet is running.

    Silence is not a stall: a single attention cell at e400 runs for hours, so
    no new result in half an hour is the normal state for most of this campaign.
    Asking whether the workers are ALIVE is what distinguishes a long cell from
    a dead fleet. Ported from `watch.sh`.

    Expensive — one `send-command` and a sleep per instance — so it runs only
    after a run of polls with no progress at all, never on the normal path.
    Returns None if any instance cannot be reached, because a partial count
    would read as a partial stall.
    """
    ids = aws(["ec2", "describe-instances", "--region", region,
               "--filters", "Name=instance-state-name,Values=running",
               "--query", "Reservations[].Instances[].InstanceId",
               "--output", "text"]).split()
    if not ids:
        return 0
    total = 0
    for instance in ids:
        command = aws(["ssm", "send-command", "--region", region,
                       "--instance-ids", instance,
                       "--document-name", "AWS-RunShellScript",
                       "--parameters",
                       'commands=["pgrep -fc \'shd-instrument train-cell\' || echo 0"]',
                       "--query", "Command.CommandId", "--output", "text"]).strip()
        if not command:
            return None
        time.sleep(7)
        seen = aws(["ssm", "get-command-invocation", "--region", region,
                    "--command-id", command, "--instance-id", instance,
                    "--query", "StandardOutputContent",
                    "--output", "text"]).strip()
        if not seen.isdigit():
            return None
        total += int(seen)
    return total


def hostlog_claims(bucket: str) -> dict[str, set[str]]:
    """`instance id -> every cell its log has ever claimed`.

    `bootstrap.sh` ships each host's bootstrap log to `hostlogs/<instance>.log`
    once a minute, and every claim appears there as `slot N: running <cell id>`.
    That is the cheap half of the liveness question: `release_dead_claims.py`
    answers the expensive half over SSM, one `send-command` and a seven-second
    sleep per instance, which is far too heavy to poll on.

    It returns the claims per instance rather than one owner per cell, and that
    is the whole correctness of the thing. **Hostlogs accumulate and outlive
    their instances.** A cell claimed by an instance that was reclaimed, then
    released back to the queue and picked up by a live one, is named in BOTH
    logs — the dead instance's log stays in S3 forever. A `cell -> owner` map
    lets whichever log is processed last win, and S3 listing order is
    alphabetical rather than chronological.

    That is not hypothetical: the first version of this returned such a map and
    reported six of wave 20's cells stranded. `release_dead_claims.py`, asking
    the fleet over SSM, said **zero**. All six were running on a live instance
    and were named in a dead one's log only because they had been requeued
    earlier the same day — by the very release this check exists to prompt.
    """
    claims: dict[str, set[str]] = {}
    listing = aws(["s3", "ls", f"s3://{bucket}/hostlogs/", "--recursive"])
    for line in listing.splitlines():
        if not line.strip() or not line.split()[-1].endswith(".log"):
            continue
        key = line.split()[-1]
        instance = key.rsplit("/", 1)[-1][:-4]
        body = aws(["s3", "cp", f"s3://{bucket}/{key}", "-"])
        claims[instance] = set(re.findall(r"slot \d+: running (\S+)", body))
    return claims


def live_instance_ids(region: str) -> set[str] | None:
    out = aws(["ec2", "describe-instances", "--region", region,
               "--filters", "Name=instance-state-name,Values=running,pending",
               "--query", "Reservations[].Instances[].InstanceId",
               "--output", "text"]).strip()
    return set(out.split()) if out else None


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
    p.add_argument("--stall-after", type=int, default=6,
                   help="polls with no progress before asking the fleet, over "
                        "SSM, whether any worker process is alive")
    args = p.parse_args()

    seen_fail: set[str] = set()
    seen_waves: set[str] = set()
    settled: set[str] = set()
    stranded: set[str] = set()
    seen_gates: dict[str, str] = {}
    quiet = 0
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

        # Gates first seen while watching. On the first poll they are summarised
        # in one line rather than announced individually: a restart that emits
        # six notifications saying nothing new is a restart the reader learns
        # to skip.
        gates = gate_verdicts(args.bucket)
        if last_plan is None and gates:
            say("gates: " + "  ".join(f"{i[:11]}={v}" for i, v in sorted(gates.items())))
        else:
            for instance, verdict in sorted(gates.items()):
                if instance not in seen_gates:
                    say(f"GATE {instance}: cross-machine Gate F {verdict}")
        seen_gates = gates

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

        # Key cells are checked for a THIRD outcome besides done and failed.
        # A diverged cell writes a failure log; a cell whose spot instance is
        # reclaimed writes nothing at all -- its claim is simply orphaned and it
        # never finishes. Without this the watcher stays silent forever on the
        # one loss that is actually recoverable, which is the exact shape of
        # "a check that could not run reporting the same result as one that
        # ran and passed".
        # Checked for EVERY outstanding cell, not only the named ones. The first
        # version looked at `--cell` alone, which meant a reclaim anywhere else
        # in the plan went unreported until someone ran release_dead_claims.py
        # by hand — which on 2026-08-27 is exactly how two orphaned cells were
        # found, by luck rather than by the watch.
        #
        # The hostlog read is the same one either way, so widening it costs a
        # larger set comparison and nothing else.
        stranded_now: set[str] = set()
        outstanding_now = plan - done - failed
        alive = live_instance_ids(args.region) if outstanding_now else None
        if alive:
            claims = hostlog_claims(args.bucket)
            # A cell any LIVE instance's log names is running, whatever a dead
            # instance's log also says. This is the same refusal
            # `release_dead_claims.py` makes: never call a cell dead on the
            # word of a host that is gone when a host that is here claims it.
            running = set().union(*(c for i, c in claims.items() if i in alive)) \
                if any(i in alive for i in claims) else set()
            abandoned = set().union(*(c for i, c in claims.items()
                                      if i not in alive)) \
                if any(i not in alive for i in claims) else set()
            stranded_now = (abandoned - running) & outstanding_now
        fresh = sorted(stranded_now - stranded)
        if fresh:
            # A reclaimed instance strands every cell it held at once, so name a
            # few and count the rest rather than emitting one line per slot.
            shown = ", ".join(c.split("__", 1)[-1][:56] for c in fresh[:3])
            more = f" and {len(fresh) - 3} more" if len(fresh) > 3 else ""
            say(f"STRANDED: {len(fresh)} cell(s) claimed by an instance that is "
                f"gone — {shown}{more}. No result and no failure log, so this is "
                f"a spot reclaim and `python3 scripts/aws/release_dead_claims.py "
                f"--bucket {args.bucket} --apply` can requeue them. A divergence "
                f"writes a failure log and cannot be requeued; this can.")
        for cell in args.cell:
            if cell in stranded_now and cell not in stranded:
                say(f"  ...one of them is a KEY CELL: {cell}")
        stranded = stranded_now

        for cell in args.cell:
            if cell in settled:
                continue
            if cell in results:
                settled.add(cell)
                say(f"KEY CELL COMPLETE: {cell}")
            elif cell in failures:
                settled.add(cell)
                say(f"KEY CELL LOST: {cell} — a failure log exists, so it ran to "
                    f"a definite answer. This is not recoverable by requeueing.")

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

        # Silence is not a stall, and it is not health either. After a run of
        # polls with nothing settling, ask the fleet directly rather than infer
        # from silence and cry wolf every half hour.
        if last_done is not None and len(done) == last_done and outstanding:
            quiet += 1
            if quiet >= args.stall_after:
                quiet = 0
                busy = worker_processes(args.region)
                if busy is None:
                    say(f"could not count worker processes at "
                        f"{len(done)}/{len(plan)} — an instance did not answer, "
                        f"so this is neither a stall nor a clean bill")
                elif busy == 0:
                    say(f"STALLED {len(done)}/{len(plan)}: NO WORKER PROCESSES on "
                        f"the running instance(s), {len(outstanding)} cell(s) "
                        f"outstanding. This needs a hand.")
                else:
                    say(f"quiet {len(done)}/{len(plan)} but healthy: {busy} worker "
                        f"process(es) busy on long cells")
        else:
            quiet = 0

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
