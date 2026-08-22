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


def keys(bucket: str, prefix: str) -> set[str]:
    found: set[str] = set()
    token = None
    while True:
        argv = ["aws", "s3api", "list-objects-v2", "--bucket", bucket, "--prefix", prefix]
        if token:
            argv += ["--starting-token", token]
        out = subprocess.run(argv, capture_output=True, text=True)
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


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("bucket")
    parser.add_argument("--plan", default="cells.json")
    args = parser.parse_args()

    done = {k[: -len(".json")] for k in keys(args.bucket, "results/") if k.endswith(".json")}
    held = keys(args.bucket, "claims/")
    for cell in json.load(open(args.plan)):
        cid = cell["id"]
        if cid in done or cid in held:
            continue
        claimed = subprocess.run(
            ["aws", "s3api", "put-object", "--bucket", args.bucket,
             "--key", f"claims/{cid}", "--if-none-match", "*"],
            capture_output=True)
        if claimed.returncode == 0:
            print(cid)
            return 0
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
