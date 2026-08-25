#!/usr/bin/env python3
"""Provision and launch the SHD attention campaign on EC2 spot.

Idempotent: every resource is created only if absent, so re-running after a
partial failure resumes rather than duplicates. Resources created are printed at
the end and are all tagged `binn-campaign`, so `teardown.py` can find them.

Deliberately no inbound network. Instances are autonomous — they claim work from
S3, upload results, and shut themselves down when the queue drains. Debugging is
via SSM Session Manager, which needs no open port.
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
import tarfile
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent
TAG = "binn-campaign"
ROLE = "binn-campaign-worker"
PROFILE = "binn-campaign-worker"
AL2023_ARM64_SSM = "/aws/service/ami-amazon-linux-latest/al2023-ami-kernel-default-arm64"

# Excluded from the source tarball: build output, virtualenvs, git history, the
# corpus (uploaded separately, it is static), and the large recorded artifacts
# that no cell reads. `results/shd_instrument_v4/{initialization,cells,
# cell-manifests}` ARE included - Gate F reads them, and Gate F is the
# cross-machine gate.
TAR_EXCLUDE_DIRS = {"target", ".git", "data", "viz", "hybrid-results"}
TAR_EXCLUDE_PREFIXES = (".venv", "results/shd_attention_pilot_v1")


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


def aws(*argv, check=True, parse=True):
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
    if not parse:
        return out
    try:
        return json.loads(out.stdout) if out.stdout.strip() else {}
    except json.JSONDecodeError:
        return out.stdout.strip()


def build_tarball(path: Path) -> None:
    def keep(name: str) -> bool:
        rel = name[2:] if name.startswith("./") else name
        head = rel.split("/", 1)[0]
        if head in TAR_EXCLUDE_DIRS:
            return False
        return not rel.startswith(TAR_EXCLUDE_PREFIXES)

    with tarfile.open(path, "w:gz") as tar:
        for item in sorted(ROOT.iterdir()):
            if not keep(item.name):
                continue
            tar.add(item, arcname=item.name,
                    filter=lambda info: info if keep(info.name) else None)


def ensure_bucket(bucket: str, region: str) -> None:
    probe = aws("s3api", "head-bucket", "--bucket", bucket, check=False, parse=False)
    if probe.returncode == 0:
        print(f"  bucket {bucket} exists")
        return
    argv = ["s3api", "create-bucket", "--bucket", bucket, "--region", region]
    if region != "us-east-1":
        argv += ["--create-bucket-configuration", f"LocationConstraint={region}"]
    aws(*argv)
    print(f"  bucket {bucket} created")


def ensure_role(bucket: str) -> str:
    """Least-privilege worker role: this bucket, plus SSM for debugging."""
    probe = aws("iam", "get-instance-profile", "--instance-profile-name", PROFILE,
                check=False, parse=False)
    if probe.returncode == 0:
        print(f"  instance profile {PROFILE} exists")
        return PROFILE

    trust = json.dumps({
        "Version": "2012-10-17",
        "Statement": [{"Effect": "Allow",
                       "Principal": {"Service": "ec2.amazonaws.com"},
                       "Action": "sts:AssumeRole"}],
    })
    aws("iam", "create-role", "--role-name", ROLE,
        "--assume-role-policy-document", trust,
        "--tags", f"Key=Project,Value={TAG}")
    policy = json.dumps({
        "Version": "2012-10-17",
        "Statement": [
            {"Effect": "Allow",
             "Action": ["s3:GetObject", "s3:PutObject", "s3:ListBucket"],
             "Resource": [f"arn:aws:s3:::{bucket}", f"arn:aws:s3:::{bucket}/*"]},
        ],
    })
    aws("iam", "put-role-policy", "--role-name", ROLE,
        "--policy-name", "campaign-bucket", "--policy-document", policy)
    aws("iam", "attach-role-policy", "--role-name", ROLE,
        "--policy-arn", "arn:aws:iam::aws:policy/AmazonSSMManagedInstanceCore")
    aws("iam", "create-instance-profile", "--instance-profile-name", PROFILE)
    aws("iam", "add-role-to-instance-profile", "--instance-profile-name", PROFILE,
        "--role-name", ROLE)
    print(f"  role {ROLE} + instance profile {PROFILE} created")
    # IAM is eventually consistent; RunInstances fails if the profile is not yet
    # visible. This wait is why the launch does not need a retry loop.
    time.sleep(12)
    return PROFILE


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--plan", required=True)
    parser.add_argument("--region", default="us-east-1")
    parser.add_argument("--instance-type", default="c7g.16xlarge")
    parser.add_argument("--count", type=int, default=2)
    # Measured, one attention cell, e10/h128, dev machine:
    #
    #   threads  speedup  efficiency  cells per 64 vCPU  throughput
    #         1     1.00        100%                 64        64.0
    #         2     1.85         92%                 32        59.1
    #         4     3.38         84%                 16        54.1
    #         8     5.72         71%                  8        45.8
    #        16     7.90         49%                  4        31.6
    #
    # Throughput is maximised by FEWER threads per cell: the parallel speedup
    # never keeps up with the cores it consumes, so a box finishes more work by
    # running more cells slowly than fewer cells quickly. Results are
    # bit-identical at every thread count, so this costs nothing scientifically.
    #
    # 2x32 rather than 1x64 because the corpus is loaded per process (~1.4 GB
    # measured on the instance, 12 GB for 8 cells on a 123 GB box). 64 cells
    # would be ~96 GB and leaves no headroom for the h1024 wave; 32 cells is
    # ~48 GB, still saturates all 64 vCPU, and lands within 8% of the ceiling.
    #
    # The cost is tail latency: the slowest single cell takes 5.7/1.85 = 3.1x
    # longer than at 8 threads. That is the right trade while 400+ cells are
    # queued, and the wrong one for the last few.
    # 4x16 rather than 2x32: throughput 54.1 vs 59.1 is a 9% give-up, but the
    # slowest single cell drops from ~26 h to ~14 h. With spot interruption in
    # play, a 26-hour cell is a cell that may never finish.
    parser.add_argument("--threads-per-cell", type=int, default=4)
    parser.add_argument("--concurrent-cells", type=int, default=0,
                        help="0 = derive from the instance's vCPU count (default)")
    parser.add_argument("--bucket")
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("--skip-inputs", action="store_true",
                        help="do not re-upload source + corpus; use when scaling an "
                             "already-provisioned campaign. The corpus is ~660 MB and static, "
                             "and re-uploading it delays the thing you are scaling. The plan "
                             "is always uploaded regardless.")
    args = parser.parse_args()

    account = aws("sts", "get-caller-identity")["Account"]
    bucket = args.bucket or f"binn-campaign-{account}-{args.region}"
    cells = json.load(open(args.plan))
    vcpus = args.count * int(args.instance_type.split(".")[1].rstrip("xlarge") or 1) * 4

    print(f"account        {account}")
    print(f"region         {args.region}")
    print(f"bucket         {bucket}")
    print(f"plan           {args.plan} ({len(cells)} cells)")
    print(f"fleet          {args.count} x {args.instance_type}  (~{vcpus} vCPU)")
    print(f"per instance   {args.concurrent_cells or 'nproc/threads'} cells "
          f"x {args.threads_per_cell} threads")
    if args.dry_run:
        print("\ndry run - nothing provisioned")
        return 0

    print("\nprovisioning")
    ensure_bucket(bucket, args.region)
    profile = ensure_role(bucket)

    if args.skip_inputs:
        print("\nskipping source + corpus upload (--skip-inputs)")
        aws("s3", "cp", str(ROOT / "scripts/aws/bootstrap.sh"),
            f"s3://{bucket}/input/bootstrap.sh", "--quiet")
    else:
        upload_inputs(bucket, args)
    # The plan is uploaded on every launch, including under --skip-inputs. It is
    # a few hundred KB, and the alternative is a fleet that silently runs the
    # previous wave's queue because the operator asked to skip a 660 MB corpus.
    # "Which cells run" is never the thing an upload optimisation may decide.
    upload_plan(bucket, args.plan)

    ami = aws("ssm", "get-parameter", "--name", AL2023_ARM64_SSM,
              "--region", args.region)["Parameter"]["Value"]
    return launch_fleet(args, bucket, profile, ami)


def upload_inputs(bucket, args):
    print("\nuploading inputs")
    tarball = Path("/tmp/binn-source.tar.gz")
    build_tarball(tarball)
    print(f"  source.tar.gz {tarball.stat().st_size / 1e6:.0f} MB")
    aws("s3", "cp", str(tarball), f"s3://{bucket}/input/source.tar.gz", "--quiet")
    for split in ("train", "test"):
        aws("s3", "cp", str(ROOT / f"data/shd/events/{split}.events"),
            f"s3://{bucket}/input/{split}.events", "--quiet")
    print("  corpus uploaded")


def upload_plan(bucket, plan_path):
    """Publish the queue, and say so loudly when it is not the queue on disk."""
    try:
        remote = subprocess.run(
            ["aws", "s3", "cp", f"s3://{bucket}/input/cells.json", "-"],
            capture_output=True, text=True, timeout=AWS_TIMEOUT_S)
    except subprocess.TimeoutExpired:
        raise SystemExit(
            f"reading the published queue did not answer in {AWS_TIMEOUT_S}s; "
            "refusing to replace a queue that could not be read"
        ) from None
    if remote.returncode == 0:
        try:
            previous = {c["id"] for c in json.loads(remote.stdout)}
        except (ValueError, KeyError, TypeError):
            previous = None
        current = {c["id"] for c in json.load(open(plan_path))}
        if previous is not None and previous != current:
            print(f"  REPLACING the published queue: {len(previous)} cells -> "
                  f"{len(current)} ({len(current - previous)} new, "
                  f"{len(previous - current)} withdrawn)")
    aws("s3", "cp", plan_path, f"s3://{bucket}/input/cells.json", "--quiet")
    print("  plan uploaded")


def launch_fleet(args, bucket, profile, ami):
    # An unset CONCURRENT_CELLS means "derive from nproc", which is what a
    # deliberately heterogeneous fleet needs: the scaler adds whatever size the
    # account ceiling allows at the time.
    concurrency = (
        f"export CONCURRENT_CELLS={args.concurrent_cells}\n"
        if args.concurrent_cells
        else ""
    )
    user_data = (
        "#!/usr/bin/env bash\n"
        f"export BUCKET={bucket}\n"
        f"export THREADS_PER_CELL={args.threads_per_cell}\n"
        f"{concurrency}"
        "curl -s -o /tmp/bootstrap.sh "
        f"https://{bucket}.s3.{args.region}.amazonaws.com/input/bootstrap.sh 2>/dev/null || true\n"
        f"aws s3 cp s3://{bucket}/input/bootstrap.sh /tmp/bootstrap.sh\n"
        "bash /tmp/bootstrap.sh\n"
    )
    aws("s3", "cp", str(ROOT / "scripts/aws/bootstrap.sh"),
        f"s3://{bucket}/input/bootstrap.sh", "--quiet")
    Path("/tmp/user-data.sh").write_text(user_data)

    print("\nlaunching")
    launched = aws(
        "ec2", "run-instances", "--region", args.region,
        "--image-id", ami, "--instance-type", args.instance_type,
        "--count", str(args.count),
        "--instance-market-options",
        "MarketType=spot,SpotOptions={SpotInstanceType=one-time,InstanceInterruptionBehavior=terminate}",
        "--iam-instance-profile", f"Name={profile}",
        "--block-device-mappings",
        "DeviceName=/dev/xvda,Ebs={VolumeSize=120,VolumeType=gp3,DeleteOnTermination=true}",
        "--instance-initiated-shutdown-behavior", "terminate",
        "--metadata-options", "HttpTokens=required,HttpEndpoint=enabled",
        "--user-data", "file:///tmp/user-data.sh",
        "--tag-specifications",
        f"ResourceType=instance,Tags=[{{Key=Project,Value={TAG}}},{{Key=Name,Value={TAG}}}]",
    )
    ids = [i["InstanceId"] for i in launched["Instances"]]
    print(f"  {len(ids)} instance(s): {' '.join(ids)}")
    print(f"\nprogress:  python3 scripts/aws/collect.py --bucket {bucket}")
    print(f"teardown:  python3 scripts/aws/teardown.py --bucket {bucket}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
