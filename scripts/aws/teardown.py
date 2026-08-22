#!/usr/bin/env python3
"""Terminate campaign instances and, optionally, remove the provisioned IAM.

The bucket is never deleted: it holds the results. Instances self-terminate when
the queue drains, so this is for stopping a run early or cleaning up a fleet
whose queue stalled.

The campaign tag is applied twice: once as a server-side filter on the describe
call, and again client-side against the tags in the reply. An instance that
comes back without the confirming tag is reported and left running - see
`confirms_campaign_tag`. Exit status is 2 if anything was refused.
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


def confirms_campaign_tag(instance) -> bool:
    """Does this instance's own tag set say it belongs to this campaign?

    `--filters Name=tag:Project,Values=...` is applied by EC2, and the reply is
    the only evidence that it was applied *as intended*. A widened filter, a
    filter typed against the wrong key, the wrong region, or a hand-edited call
    all come back looking exactly like a correct one: a list of instance ids
    with nothing on them that says whose they are. So the tags are read back off
    each instance and re-checked here, client-side, before anything is
    terminated.

    Absent or unreadable tags are a refusal, not a pass. `describe-instances`
    returns `Tags` on every instance by default, so a reply with none is either
    an untagged instance — which by definition is not one of ours — or a call
    that has been narrowed (a `--query`, a different API shape) until the
    evidence is gone. This is the script that destroys things; it fails closed.
    """
    if not isinstance(instance, dict):
        return False
    for tag in instance.get("Tags") or []:
        if isinstance(tag, dict) and tag.get("Key") == "Project" and tag.get("Value") == TAG:
            return True
    return False


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--region", default="us-east-1")
    parser.add_argument("--bucket", help="printed for reference; never deleted")
    parser.add_argument("--remove-iam", action="store_true")
    args = parser.parse_args()

    described = aws("ec2", "describe-instances", "--region", args.region,
                    "--filters", f"Name=tag:Project,Values={TAG}",
                    "Name=instance-state-name,Values=pending,running,stopping,stopped")
    returned = [i for r in described.get("Reservations", []) for i in r["Instances"]]
    ids = [i["InstanceId"] for i in returned if confirms_campaign_tag(i)]
    refused = [i.get("InstanceId", "<no InstanceId>") for i in returned
               if not confirms_campaign_tag(i)]
    if refused:
        print(f"REFUSED {len(refused)} instance(s) returned by the tag filter whose own "
              f"tags do not confirm Project={TAG}: {' '.join(refused)}")
        print("  Nothing was terminated for them. Either the filter did not do what it "
              "says, or these are not campaign instances. Check by hand.")
    if ids:
        aws("ec2", "terminate-instances", "--region", args.region, "--instance-ids", *ids)
        print(f"terminating {len(ids)}: {' '.join(ids)}")
    elif not refused:
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
    # A refusal is a disagreement between what EC2 was asked for and what it
    # returned, and the operator has to see it even when this runs from a script
    # that only reads the exit status. The confirmed instances are still
    # terminated first - leaving a burning fleet up to make a point would be the
    # more expensive failure - so this reports "not a clean teardown", not
    # "nothing happened".
    return 2 if refused else 0


if __name__ == "__main__":
    raise SystemExit(main())
