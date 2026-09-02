#!/usr/bin/env python3
"""Launch the kernel-ablation series: six PyTorch runs on one box.

Deliberately separate from `scripts/aws/launch.py`. That script provisions a
fleet against a claimable cell queue and pins a Rust binary; this one runs six
jobs once and pins a pip environment. The two share a bucket and an instance
profile and nothing else, and merging them would put a claim protocol into a
script that needs none.

# It reuses the campaign bucket ON PURPOSE

`launch.py::ensure_role` RE-SCOPES the shared `binn-campaign-worker` policy to
whichever bucket it is given -- least privilege, one bucket at a time. Pointing
this series at its own bucket would therefore re-scope the policy away from the
campaign bucket, and every worker of a running wave would lose the S3 access it
needs to upload the cell it is holding. Mid-campaign, silently.

So this writes under a `kernel-ablation/` prefix of the campaign bucket, and
never calls `ensure_role` at all. The existing policy already grants the whole
bucket.

The instances also carry their own tag, so `teardown.py --bucket <campaign>`
cannot sweep them up with a wave's fleet, and neither can the reverse.

    python3 scripts/aws/kernel_ablation/launch.py --probe    # environment + timing
    python3 scripts/aws/kernel_ablation/launch.py            # the six runs
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
import tarfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
HERE = Path(__file__).resolve().parent
AWS_TIMEOUT_S = 300
TAG = "binn-kernel-ablation"
PREFIX = "kernel-ablation"
AL2023_ARM64_SSM = "/aws/service/ami-amazon-linux-latest/al2023-ami-kernel-default-arm64"
PROFILE = "binn-campaign-worker"

#: The pinned reference. `bootstrap.sh` re-checks this on the instance and
#: aborts, because a tarball is easier to get wrong than a git ref.
REFERENCE_COMMIT = "d169b4e3049a3d5bff56c84a8b2f0c4e835aafda"

CHECKOUT = ROOT / "results/shd_instrument_v4/reference-cache/SNN-delays"
#: The binned cache spikingjelly generates for `time_step = 10`. The raw `.h5`
#: files the local tree points at are BROKEN symlinks into a directory that no
#: longer exists, so this cache is not an optimisation -- it is the dataset.
DATA = ROOT / "data/shd/reference"
CLEAN_MAIN = ROOT / "scripts/shd_calibration/reference_clean_main.py"


def aws(*argv, parse=True):
    try:
        out = subprocess.run(["aws", *argv], capture_output=True, text=True,
                             timeout=AWS_TIMEOUT_S)
    except subprocess.TimeoutExpired:
        raise SystemExit(f"aws {' '.join(argv[:3])} did not answer in "
                         f"{AWS_TIMEOUT_S}s") from None
    if out.returncode != 0:
        raise SystemExit(f"aws {' '.join(argv[:3])} failed:\n{out.stderr.strip()}")
    if not parse:
        return out
    try:
        return json.loads(out.stdout) if out.stdout.strip() else {}
    except json.JSONDecodeError:
        return out.stdout.strip()


def check_inputs() -> None:
    """Every precondition, before anything is uploaded or provisioned."""
    problems = []
    if not CHECKOUT.is_dir():
        problems.append(f"no reference checkout at {CHECKOUT}")
    else:
        head = subprocess.run(["git", "-C", str(CHECKOUT), "rev-parse", "HEAD"],
                              capture_output=True, text=True).stdout.strip()
        if head != REFERENCE_COMMIT:
            problems.append(f"checkout is at {head}, expected {REFERENCE_COMMIT}")
    cache = DATA / "duration_10"
    if not cache.is_dir():
        problems.append(f"no binned cache at {cache}")
    else:
        n = sum(1 for _ in cache.rglob("*") if _.is_file())
        if n < 10_000:
            problems.append(f"{cache} holds {n} files; the SHD cache is 10420")
    if not CLEAN_MAIN.is_file():
        problems.append(f"no clean runner at {CLEAN_MAIN}")
    if problems:
        raise SystemExit("REFUSING TO LAUNCH:\n  " + "\n  ".join(problems))


def tarball(source: Path, destination: Path, arcname: str, keep=None) -> Path:
    """Pack `source` as `arcname`, optionally keeping only some top-level names.

    The checkout is packed WITH its `.git`: `bootstrap.sh` re-derives HEAD with
    `git rev-parse` and aborts on a mismatch, which a tarball needs more than a
    git clone does -- a tarball is the easier of the two to get wrong.
    """
    with tarfile.open(destination, "w:gz") as tar:
        if keep is None:
            tar.add(source, arcname=arcname)
        else:
            for name in keep:
                tar.add(source / name, arcname=f"{arcname}/{name}")
    return destination


def pack_dataset(destination: Path) -> Path:
    """`duration_10` plus a real `extract/`, and nothing else.

    `spikingjelly/datasets/shd.py:163` skips the download path entirely when
    `extract/` EXISTS, and says in its own message that it will not check what
    is in it. On this machine that directory holds symlinks into a tree that no
    longer exists -- dangling, so the local runs never read the raw `.h5` at all
    and worked from the binned cache. Shipping the cache alone reproduces the
    outcome but not the condition: with no `extract/`, spikingjelly tried to
    download SHD and died on all six runs.

    So `extract/` ships with the REAL `.h5` files from `data/shd/`. That is
    331 MB against an empty directory that would also work, and it buys the
    difference between "the cache happened to be enough" and "the dataset is
    complete": if the cache is ever rejected, the box can rebuild it instead of
    reaching for the network.

    `duration_10.incompatible-current-master` is left behind. It is 86 MB of
    cache built by a spikingjelly this series does not pin, and its name is the
    warning.
    """
    with tarfile.open(destination, "w:gz") as tar:
        tar.add(DATA / "duration_10", arcname="shd-reference/duration_10")
        for split in ("train", "test"):
            source = ROOT / f"data/shd/shd_{split}.h5"
            if not source.is_file():
                raise SystemExit(f"REFUSING TO LAUNCH: no {source}")
            tar.add(source, arcname=f"shd-reference/extract/shd_{split}.h5")
    return destination


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--bucket", default="binn-campaign-v2-511192439661-us-east-1")
    parser.add_argument("--region", default="us-east-1")
    parser.add_argument("--instance-type", default="c7g.8xlarge")
    parser.add_argument("--series", default="kernel", choices=("kernel", "membrane"),
                        help="which registered series to run. `kernel` is arms A "
                             "and B (PREREG_2026-09-01_THE_KERNEL_ABLATION); "
                             "`membrane` is arms B2 and E "
                             "(PREREG_2026-09-02_THE_MEMBRANE_ABLATION). They "
                             "share every pinned version and differ only in which "
                             "arms exist and where results land.")
    parser.add_argument("--probe", action="store_true",
                        help="build the environment, run ONE Arm A seed, and keep "
                             "the box up. Use this before spending the series: it "
                             "turns 'the stack installs on aarch64' and 'an epoch "
                             "takes N minutes' from guesses into measurements.")
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()

    check_inputs()
    mode = "probe" if args.probe else "full"
    print(f"bucket         s3://{args.bucket}/{PREFIX}/")
    print(f"instance       {args.instance_type}  (spot)")
    print(f"mode           {mode}")
    print(f"series         {args.series}")
    print(f"reference      {REFERENCE_COMMIT}")
    if args.dry_run:
        print("\ndry run - nothing uploaded, nothing provisioned")
        return 0

    print("\nuploading inputs")
    checkout_tar = tarball(CHECKOUT, Path("/tmp/snn-delays.tar.gz"), "SNN-delays")
    data_tar = pack_dataset(Path("/tmp/shd-duration10.tar.gz"))
    for path, key in ((checkout_tar, "snn-delays.tar.gz"),
                      (data_tar, "shd-duration10.tar.gz"),
                      (HERE / "requirements.lock", "requirements.lock"),
                      (CLEAN_MAIN, "clean_main.py")):
        aws("s3", "cp", str(path), f"s3://{args.bucket}/{PREFIX}/input/{key}", "--quiet")
        print(f"  {key:<24} {Path(path).stat().st_size / 1e6:.0f} MB")

    bootstrap = (HERE / "bootstrap.sh").read_text().replace("__BUCKET__", args.bucket)
    Path("/tmp/ka-bootstrap.sh").write_text(bootstrap)
    aws("s3", "cp", "/tmp/ka-bootstrap.sh",
        f"s3://{args.bucket}/{PREFIX}/input/bootstrap.sh", "--quiet")

    user_data = (
        "#!/usr/bin/env bash\n"
        f"export KA_MODE={mode}\n"
        f"export KA_SERIES={args.series}\n"
        f"aws s3 cp s3://{args.bucket}/{PREFIX}/input/bootstrap.sh /tmp/ka.sh\n"
        "bash /tmp/ka.sh\n"
    )
    Path("/tmp/ka-user-data.sh").write_text(user_data)

    ami = aws("ssm", "get-parameter", "--name", AL2023_ARM64_SSM,
              "--region", args.region)["Parameter"]["Value"]

    print("\nlaunching")
    launched = aws(
        "ec2", "run-instances", "--region", args.region,
        "--image-id", ami, "--instance-type", args.instance_type, "--count", "1",
        "--instance-market-options",
        "MarketType=spot,SpotOptions={SpotInstanceType=one-time,"
        "InstanceInterruptionBehavior=terminate}",
        "--iam-instance-profile", f"Name={PROFILE}",
        "--block-device-mappings",
        "DeviceName=/dev/xvda,Ebs={VolumeSize=60,VolumeType=gp3,DeleteOnTermination=true}",
        "--instance-initiated-shutdown-behavior", "terminate",
        "--metadata-options", "HttpTokens=required,HttpEndpoint=enabled",
        "--user-data", "file:///tmp/ka-user-data.sh",
        "--tag-specifications",
        f"ResourceType=instance,Tags=[{{Key=Project,Value={TAG}}},{{Key=Name,Value={TAG}}}]",
    )
    ids = [i["InstanceId"] for i in launched["Instances"]]
    print(f"  {' '.join(ids)}")
    print(f"\nlive log:  aws s3 cp s3://{args.bucket}/{PREFIX}/logs/{ids[0]}.log -")
    results = "kernel-ablation" if args.series == "kernel" else "membrane-ablation"
    print(f"results:   aws s3 ls s3://{args.bucket}/{results}/results/")
    return 0


if __name__ == "__main__":
    sys.exit(main())
