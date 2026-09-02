#!/bin/bash
# Kernel ablation worker. One instance runs the whole series.
#
# This is NOT `scripts/aws/bootstrap.sh` and deliberately shares no code with
# it. That script runs a pinned Rust binary over a claimable cell queue of
# hundreds; this runs six PyTorch jobs, once, on one box. Forcing them together
# would put a claim protocol and a binary pin into a script that needs neither
# and a pip environment into one that must never grow one.
#
# What it does share is the rule both exist to enforce: every version is pinned
# and a failed precondition aborts rather than degrading. HK-4 -- Arm A landing
# within 0.010 of the pinned 0.9376 -- is the gate for the whole series, so an
# environment that quietly resolved a different torch would waste the series
# rather than fail it.
set -euo pipefail

BUCKET="__BUCKET__"
PREFIX="kernel-ablation"
MODE="${KA_MODE:-full}"
ROOT=/opt/ka
# cloud-init user-data runs with no HOME; git and pip both want one.
export HOME=/root
LOG=/var/log/ka-bootstrap.log
exec > >(tee -a "$LOG") 2>&1

# Without this, any `set -e` abort -- a missing python3.12, a wheel that will
# not install, a checkout that fails its commit check -- leaves the box running
# and idle with the only diagnosis in a local file that dies with it. An
# instance that failed must cost the same as one that finished: nothing.
cleanup() {
  status=$?
  if [ "$status" -ne 0 ]; then
    echo "ABORTED with status $status at $(date -u +%FT%TZ)"
  fi
  aws s3 cp "$LOG" "s3://$BUCKET/$PREFIX/logs/${INSTANCE_ID:-unknown}.log" --quiet || true
  shutdown -h now
}
trap cleanup EXIT

INSTANCE_ID="$(curl -s -X PUT 'http://169.254.169.254/latest/api/token' \
  -H 'X-aws-ec2-metadata-token-ttl-seconds: 600' \
  | xargs -I{} curl -s -H 'X-aws-ec2-metadata-token: {}' \
    http://169.254.169.254/latest/meta-data/instance-id)"
echo "instance $INSTANCE_ID  mode $MODE  $(date -u +%FT%TZ)"

# Ship the log to S3 every 60s so a run that goes wrong is visible while it is
# going wrong, not after the box is gone. The reference runner writes its result
# only on the final epoch, so tqdm in this log is the ONLY progress signal there
# is for the next several hours.
(
  while true; do
    aws s3 cp "$LOG" "s3://$BUCKET/$PREFIX/logs/$INSTANCE_ID.log" --quiet || true
    for f in "$ROOT"/runs/*/run.log; do
      [ -f "$f" ] || continue
      aws s3 cp "$f" "s3://$BUCKET/$PREFIX/logs/$INSTANCE_ID-$(basename "$(dirname "$f")").log" --quiet || true
    done
    sleep 60
  done
) &
HEARTBEAT=$!

dnf install -y -q python3.12 python3.12-pip git tar gzip gcc >/dev/null
mkdir -p "$ROOT" && cd "$ROOT"

for f in requirements.lock snn-delays.tar.gz shd-duration10.tar.gz clean_main.py; do
  aws s3 cp "s3://$BUCKET/$PREFIX/input/$f" "$f" --quiet
done
tar xzf snn-delays.tar.gz
tar xzf shd-duration10.tar.gz
CHECKOUT="$ROOT/SNN-delays"
DATA="$ROOT/shd-reference"

# The pinned reference commit. A checkout that is not this one is not the
# reference the paper's 0.9376 came from.
#
# `-c safe.directory` first: the tarball is unpacked by root and carries the
# uploading user's ownership, so git refuses to read the repository at all.
# Without it the verification aborts the series before a single epoch runs --
# which it did, and the EXIT trap terminated the box for about four cents.
# Passed with `-c` rather than written by `git config --global`, because
# cloud-init runs user-data with no HOME and `--global` has nowhere to write --
# which is how the SECOND launch died. Adding the exception is not weakening
# the check: the check is on the commit, and it still runs.
HEAD_SHA="$(git -c safe.directory="$CHECKOUT" -C "$CHECKOUT" rev-parse HEAD)"
EXPECTED=d169b4e3049a3d5bff56c84a8b2f0c4e835aafda
if [ "$HEAD_SHA" != "$EXPECTED" ]; then
  echo "FATAL: checkout is $HEAD_SHA, expected $EXPECTED"
  exit 1
fi
echo "checkout $HEAD_SHA verified"

python3.12 -m venv venv
./venv/bin/pip install -q --upgrade pip
./venv/bin/pip install -q -r requirements.lock
./venv/bin/python - <<'PYCHECK'
import torch, sys
print(f"torch {torch.__version__}  python {sys.version.split()[0]}")
assert torch.__version__.startswith("2.13.0"), f"torch is {torch.__version__}, not 2.13.0"
from DCLS.construct.modules import Dcls1d          # noqa: F401
from spikingjelly.activation_based import neuron   # noqa: F401
import h5py, wandb                                 # noqa: F401
print("environment OK")
PYCHECK

# Six runs: two arms x three seeds. The arm patch is applied to
# `best_config_SHD.py` -> `config.py`, which is what the local harness does.
#
#   Arm A  reference as pinned, max_delay = 25
#   Arm B  max_delay = 1, every derived value following it
#
# Arm C and HK-5 were withdrawn before any run
# (`results/DEFECT_2026-09-01_HK_5_CANNOT_FIRE.md`): `time_mask_size` is
# unreachable in the clean protocol, so holding it changed nothing.
# KA_SERIES selects which registered series runs. They share every pinned
# version and differ only in which arms exist and where results land, so one
# script serves both rather than a copy drifting from its original.
#
#   kernel    PREREG_2026-09-01_THE_KERNEL_ABLATION.md      arms A, B
#   membrane  PREREG_2026-09-02_THE_MEMBRANE_ABLATION.md    arms B2, E
SERIES="${KA_SERIES:-kernel}"
SEEDS="5170001 5170002 5170003"
case "$SERIES" in
  kernel)   ARMS="A B";  RESULT_PREFIX="kernel-ablation/results" ;;
  membrane) ARMS="B2 E"; RESULT_PREFIX="membrane-ablation/results" ;;
  *) echo "FATAL: unknown KA_SERIES $SERIES"; exit 1 ;;
esac
if [ "$MODE" = "probe" ]; then
  ARMS="$(echo "$ARMS" | cut -d" " -f1)"; SEEDS="5170001"
fi
echo "series $SERIES  arms $ARMS"

THREADS=$(( $(nproc) / 6 ))
[ "$THREADS" -lt 1 ] && THREADS=1
echo "nproc $(nproc), $THREADS torch threads per run"

mkdir -p "$ROOT/runs" "$ROOT/out"
RUN_PIDS=()
for ARM in $ARMS; do
  for SEED in $SEEDS; do
    case "$ARM" in
      A)  LABEL=armA-reference ;;
      B)  LABEL=armB-nokernel ;;
      B2) LABEL=armB2-nokernel ;;
      E)  LABEL=armE-nokernel-slowmembrane ;;
    esac
    DIR="$ROOT/runs/$LABEL-$SEED"
    rm -rf "$DIR" && cp -r "$CHECKOUT" "$DIR" && rm -rf "$DIR/.git"
    cp clean_main.py "$DIR/clean_main.py"
    ARM="$ARM" SEED="$SEED" DATA="$DATA" DIR="$DIR" ./venv/bin/python - <<'PYPATCH'
import os, pathlib
arm, seed, data, d = os.environ["ARM"], os.environ["SEED"], os.environ["DATA"], os.environ["DIR"]
src = pathlib.Path(d, "best_config_SHD.py").read_text()

def once(text, old, new):
    assert text.count(old) == 1, f"expected exactly one {old!r}, found {text.count(old)}"
    return text.replace(old, new, 1)

src = once(src, "seed = 0", f"seed = {seed}")
src = once(src, "datasets_path = 'Datasets/SHD'", f"datasets_path = {data!r}")
src = once(src, "run_name = 'Wandb Run Name'", f"run_name = 'BINN-kernel-ablation-{arm}-{seed}'")
if arm in ("B", "B2", "E"):
    # The whole manipulation. `sigInit`, the paddings and the delay positions
    # are derived from this line and follow it; that following is what "the
    # kernel is gone" means. Nothing else is touched.
    src = once(src, "    max_delay = 250//time_step\n", "    max_delay = 1\n")
if arm == "E":
    # The membrane ablation's ONLY additional value. 55.5556 ms normalises to
    # tau 5.5556, whose measured per-step retention is 0.8200 -- the
    # instrument's, matched rather than approximated. The pinned 10.05 ms
    # retains 0.0050, which is what makes the kernel-free reference have no
    # temporal integration at all.
    src = once(src, "    init_tau = 10.05", "    init_tau = 55.5556")
pathlib.Path(d, "config.py").write_text(src)

# macOS spawn semantics forced num_workers=0 locally. Held here so the loader
# concurrency is the same on both platforms and cannot be a reason Arm A moves.
#
# ALL of them, not one: `datasets.py` sets `num_workers=4` at eight call sites
# and the local harness replaced every one. Patching a single site would leave
# six of eight loaders on a different concurrency from the run this must
# reproduce, and would look like it had worked.
ds = pathlib.Path(d, "datasets.py")
text = ds.read_text()
n = text.count("num_workers=4")
assert n == 8, f"expected 8 num_workers=4 sites, found {n}"
text = text.replace("num_workers=4", "num_workers=0")
assert "num_workers=4" not in text
ds.write_text(text)
print(f"{arm} seed {seed}: config written")
PYPATCH
    (
      cd "$DIR"
      export BINN_SHD_REFERENCE_RESULT="$ROOT/out/${LABEL}__s${SEED}.json"
      export OMP_NUM_THREADS="$THREADS" MKL_NUM_THREADS="$THREADS"
      export WANDB_MODE=disabled
      "$ROOT/venv/bin/python" -c "import torch;torch.set_num_threads($THREADS)" >/dev/null
      "$ROOT/venv/bin/python" clean_main.py > run.log 2>&1
      aws s3 cp "$BINN_SHD_REFERENCE_RESULT" \
        "s3://$BUCKET/$RESULT_PREFIX/$(basename "$BINN_SHD_REFERENCE_RESULT")" --quiet
      echo "DONE $LABEL seed $SEED"
    ) &
    RUN_PIDS+=($!)
  done
done

# The RUN pids, not a bare `wait`. A bare `wait` waits for every background job
# including the log heartbeat, which loops forever -- so the script never
# reaches its shutdown and the box runs until someone notices. It did: the
# fourth launch failed all six runs in ninety seconds and was still billing
# twenty minutes later.
for pid in "${RUN_PIDS[@]}"; do
  wait "$pid" || echo "run pid $pid exited non-zero"
done
kill "$HEARTBEAT" 2>/dev/null || true
echo "series complete $(date -u +%FT%TZ)"
# `cleanup` uploads the final log and shuts down.
