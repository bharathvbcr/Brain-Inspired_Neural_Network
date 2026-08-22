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


#: Fields a real cell carries and that something downstream reads *unconditionally*
#: — i.e. with `cell[...]`, not `cell.get(...)`. `accuracy` is the measurement
#: itself (`analyse_campaign.py:161`, `analyse_wave8.py::accs`); the other five are
#: the preregistration §5 validity gates as read by `validity_problems` in both
#: analysers. A cell missing any of them is not a cell that will be voided later,
#: it is a KeyError at analysis time. Optional-by-design fields (`seed`,
#: `attn_dim`, the epoch traces) are deliberately absent from this list: older
#: cells legitimately lack them, and `analyse_wave8.load` / `gate_f_rust.py`
#: already treat their absence as a missing witness rather than a defect.
REQUIRED_CELL_FIELDS = (
    "accuracy",
    "non_finite_events",
    "classes_predicted",
    "majority_prediction",
    "silent_fraction",
    "saturated_fraction",
)


def cell_files(target):
    """`{name: (size, mtime_ns)}` for every cell json in `target`."""
    return {path.name: (path.stat().st_size, path.stat().st_mtime_ns)
            for path in target.glob("*.json")}


def cell_problem(path):
    """Why `path` is not a usable cell, or None if it is one.

    Deliberately narrow: this says the file parses and carries the fields the
    analysers index into. It is not a validity gate — whether a cell that parses
    is a *measurement* is `validity_problems`' question, and it needs the whole
    arm to answer it.
    """
    try:
        payload = json.loads(path.read_text())
    except (OSError, UnicodeDecodeError) as err:
        return f"unreadable: {err}"
    except json.JSONDecodeError as err:
        return f"not JSON: {err}"
    if not isinstance(payload, dict):
        return f"not a cell object: top level is {type(payload).__name__}"
    missing = [f for f in REQUIRED_CELL_FIELDS if f not in payload]
    if missing:
        return f"missing {len(missing)} required field(s): {', '.join(missing)}"
    if not isinstance(payload["accuracy"], (int, float)) or isinstance(payload["accuracy"], bool):
        return f"accuracy is not a number: {payload['accuracy']!r}"
    return None


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
        # Two numbers, because they answer two different questions and the line
        # this replaced answered neither honestly: it was a glob of the target
        # directory, so a sync that downloaded nothing still reported every
        # stale file already sitting there as though this run had fetched it.
        #
        # `aws s3 sync` writes only the objects it decided to transfer, and a
        # write changes the file's size or its mtime, so a name whose
        # (size, mtime_ns) entry is new or different since the snapshot below is
        # exactly a cell this run downloaded. Measured, not inferred from the
        # CLI's output — which `--quiet` suppresses, and `--quiet` stays because
        # the alternative is parsing human-readable progress lines.
        before = cell_files(target)
        aws("s3", "sync", f"s3://{args.bucket}/results/", str(target), "--quiet")
        after = cell_files(target)
        downloaded = sorted(n for n, stat in after.items() if before.get(n) != stat)
        print(f"\ndownloaded {len(downloaded)} cells -> {target}")
        print(f"cells on disk: {len(after)} "
              f"(this run and every earlier one; not a download count)")

        # Nothing used to parse what was downloaded, so a truncated or
        # half-written cell counted toward the total exactly like a good one.
        #
        # Every cell in the directory is checked, not only the ones this run
        # fetched: sync compares size and mtime, never content, so a cell that
        # arrived truncated on an earlier run is never re-fetched and would
        # otherwise stay invisible here forever.
        #
        # "Cell" means a file whose result object exists in `results/`. An
        # archive directory holds more than cells - `results/shd_attention_
        # campaign_v2` keeps its plans and manifest beside them - and calling
        # those malformed cells would be a false alarm, which is the fastest way
        # to teach an operator to ignore this line.
        collected = [n for n in sorted(after) if n[:-5] in done]
        unrecognised = sorted(set(after) - set(collected))
        broken = {name: cell_problem(target / name) for name in collected}
        broken = {name: why for name, why in broken.items() if why is not None}
        print(f"validated {len(collected)} cells: {len(collected) - len(broken)} usable, "
              f"{len(broken)} INVALID")
        for name, why in broken.items():
            print(f"  INVALID {name}: {why}")
        if unrecognised:
            print(f"  ({len(unrecognised)} other .json file(s) here are not results of "
                  f"this campaign and were not checked: {unrecognised[:3]})")
        if broken:
            # Exit code: a campaign that is only PARTLY collected is the normal
            # mid-flight state of this script, and a short cell count is not an
            # error. A cell that is on disk and unreadable is a different thing
            # — a corrupt record that the analysers will index straight into —
            # and it must not be possible for `collect.py --out X && analyse X`
            # to proceed past one. So: non-zero, while the counts above are
            # still printed in full so a partial collection is never silently
            # reported as a clean one.
            print("REFUSING to report this as a clean collection. Re-run the "
                  "collection; sync re-fetches a cell whose size differs from "
                  "S3, and `analyse_wave8.py` will refuse the arm either way.")
            return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
