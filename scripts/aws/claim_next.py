#!/usr/bin/env python3
"""Claim the next unrun cell from the S3 work queue, atomically.

Prints the claimed cell id, or nothing when the campaign is done.

The claim is a conditional PUT (`If-None-Match: *`): S3 either creates the key
or rejects the write, atomically, so exactly one worker on any instance wins a
given cell. No coordination service, no up-front sharding — which is the
property that makes a spot reclaim cost one cell instead of a whole shard.

Both prefixes are listed once per claim rather than probed key by key; at ~400
cells the difference is a second against half a minute, per claim, per worker.
"""

from __future__ import annotations

import argparse
import json
import subprocess


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


def keys(bucket: str, prefix: str) -> set[str]:
    found: set[str] = set()
    token = None
    # A paginator that keeps returning a token would otherwise spin forever.
    # 10k pages is far past any real prefix and still terminates.
    for _ in range(10_000):
        argv = ["aws", "s3api", "list-objects-v2", "--bucket", bucket, "--prefix", prefix]
        if token:
            argv += ["--starting-token", token]
        try:
            out = subprocess.run(argv, capture_output=True, text=True,
                                 timeout=AWS_TIMEOUT_S)
        except subprocess.TimeoutExpired:
            raise SystemExit(
                f"list {prefix} did not answer in {AWS_TIMEOUT_S}s"
            ) from None
        if out.returncode != 0:
            # A transient list failure must not look like "nothing is claimed",
            # which would hand the same cell to every worker at once.
            raise SystemExit(f"list {prefix} failed: {out.stderr.strip()[:200]}")
        page = json.loads(out.stdout or "{}")
        for item in page.get("Contents", []):
            found.add(item["Key"][len(prefix):])
        token = page.get("NextToken")
        if not token:
            return found
    raise SystemExit(f"list {prefix} did not terminate after 10000 pages")


# S3's two ways of saying "someone else got there first". Everything else that
# can fail a PUT is an error, not a race.
LOST_RACE = ("PreconditionFailed", "ConditionalRequestConflict")


def claim(bucket: str, cid: str) -> bool:
    """Try to take `cid`. True if this worker won it.

    A failed conditional PUT means one of two very different things, and
    conflating them retires the fleet. `PreconditionFailed` is another worker
    winning the race, so skip the cell and carry on. Anything else -- expired
    instance credentials, a revoked bucket policy, the wrong region -- fails for
    *every* cell in the plan. Treated as a lost race that walks the whole plan,
    prints nothing and exits 0, which `bootstrap.sh:148` reads as `no work left`:
    every worker returns, `wait` completes, and line 169 runs `shutdown -h now`.
    A fleet mid-campaign terminates itself, the campaign comes back short, and
    the only trace is a console log nobody reads.

    So fail loudly. The worker loop already handles a non-zero exit by sleeping
    and retrying, which is the correct response to a credentials blip.
    """
    try:
        out = subprocess.run(
            ["aws", "s3api", "put-object", "--bucket", bucket,
             "--key", f"claims/{cid}", "--if-none-match", "*"],
            capture_output=True, text=True, timeout=AWS_TIMEOUT_S)
    except subprocess.TimeoutExpired:
        # A wedged claim is not a lost race. Exiting sends the worker back
        # through its retry sleep; hanging here would leave it holding nothing
        # and doing nothing, indistinguishable from a machine that is busy.
        raise SystemExit(
            f"claim {cid} did not answer in {AWS_TIMEOUT_S}s"
        ) from None
    if out.returncode == 0:
        return True
    stderr = out.stderr or ""
    if any(marker in stderr for marker in LOST_RACE):
        return False
    raise SystemExit(f"claim {cid} failed, and not because of a race: {stderr.strip()[:200]}")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("bucket")
    parser.add_argument("--plan", default="cells.json")
    args = parser.parse_args()

    done = {k[: -len(".json")] for k in keys(args.bucket, "results/") if k.endswith(".json")}
    held = keys(args.bucket, "claims/")
    with open(args.plan) as handle:
        plan = json.load(handle)
    for cell in plan:
        cid = cell["id"]
        if cid in done or cid in held:
            continue
        if claim(args.bucket, cid):
            print(cid)
            return 0
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
