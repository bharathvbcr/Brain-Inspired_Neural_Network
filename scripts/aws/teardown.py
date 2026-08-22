#!/usr/bin/env python3
"""Terminate campaign instances and, optionally, remove the provisioned IAM.

The bucket is never deleted: it holds the results. Instances self-terminate when
the queue drains, so this is for stopping a run early or cleaning up a fleet
whose queue stalled.
"""

from __future__ import annotations

import argparse
import json
import subprocess

TAG = "binn-campaign"
ROLE = "binn-campaign-worker"
PROFILE = "binn-campaign-worker"


def aws(*argv, check=True):
    out = subprocess.run(["aws", *argv], capture_output=True, text=True)
    if check and out.returncode != 0:
        raise SystemExit(f"aws {' '.join(argv[:3])} failed:\n{out.stderr.strip()}")
    return json.loads(out.stdout) if out.stdout.strip().startswith(("{", "[")) else out.stdout


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--region", default="us-east-1")
    parser.add_argument("--bucket", help="printed for reference; never deleted")
    parser.add_argument("--remove-iam", action="store_true")
    args = parser.parse_args()

    described = aws("ec2", "describe-instances", "--region", args.region,
                    "--filters", f"Name=tag:Project,Values={TAG}",
                    "Name=instance-state-name,Values=pending,running,stopping,stopped")
    ids = [i["InstanceId"] for r in described.get("Reservations", []) for i in r["Instances"]]
    if ids:
        aws("ec2", "terminate-instances", "--region", args.region, "--instance-ids", *ids)
        print(f"terminating {len(ids)}: {' '.join(ids)}")
    else:
        print("no campaign instances running")

    if args.remove_iam:
        aws("iam", "remove-role-from-instance-profile", "--instance-profile-name", PROFILE,
            "--role-name", ROLE, check=False)
        aws("iam", "delete-instance-profile", "--instance-profile-name", PROFILE, check=False)
        aws("iam", "delete-role-policy", "--role-name", ROLE,
            "--policy-name", "campaign-bucket", check=False)
        aws("iam", "detach-role-policy", "--role-name", ROLE,
            "--policy-arn", "arn:aws:iam::aws:policy/AmazonSSMManagedInstanceCore", check=False)
        aws("iam", "delete-role", "--role-name", ROLE, check=False)
        print("IAM role and instance profile removed")

    if args.bucket:
        print(f"bucket s3://{args.bucket} left in place - it holds the results")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
