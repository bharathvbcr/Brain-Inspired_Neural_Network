#!/usr/bin/env bash
# Instance bootstrap for the SHD attention campaign.
#
# Runs as EC2 user-data on Amazon Linux 2023 (aarch64). Everything is idempotent
# and interruption-tolerant: a spot reclaim loses at most the one cell in flight,
# because work is claimed through S3 conditional writes rather than assigned up
# front, and results are uploaded per cell.
#
# S3 layout:
#   input/source.tar.gz          the working tree that produced this run
#   input/{train,test}.events    the corpus
#   input/cells.json             the campaign plan
#   claims/<id>                  atomic claim marker
#   results/<id>.json            the cell
#   logs/<id>.log                its output
#   failures/<id>.log            output of cells that did not complete
#   gates/<instance>.json        the cross-machine reproducibility gate
set -uo pipefail

# cloud-init runs user-data with a near-empty environment - in particular
# with no HOME - and `set -u` turns that into an immediate exit at the
# first `$HOME`. Set it before anything can reference it.
export HOME="${HOME:-/root}"
export PATH="$HOME/.cargo/bin:$PATH"

BUCKET="${BUCKET:?BUCKET must be set}"
THREADS_PER_CELL="${THREADS_PER_CELL:-4}"
# Derived from the host unless explicitly set. The fleet is deliberately
# heterogeneous - the scaler adds whatever size the account ceiling allows at the
# time - so a fixed cell count would oversubscribe the small boxes and starve the
# large ones. One cell per `THREADS_PER_CELL` cores keeps every size saturated
# and none of them thrashing.
if [ -z "${CONCURRENT_CELLS:-}" ]; then
  CONCURRENT_CELLS=$(( $(nproc) / THREADS_PER_CELL ))
  [ "$CONCURRENT_CELLS" -lt 1 ] && CONCURRENT_CELLS=1
fi
WORK=/opt/binn
exec > >(tee -a /var/log/binn-bootstrap.log) 2>&1

TOKEN="$(curl -s -X PUT 'http://169.254.169.254/latest/api/token' -H 'X-aws-ec2-metadata-token-ttl-seconds: 600')"
INSTANCE_ID="$(curl -s -H "X-aws-ec2-metadata-token: $TOKEN" http://169.254.169.254/latest/meta-data/instance-id)"
echo "=== bootstrap on ${INSTANCE_ID:-unknown} at $(date -u +%FT%TZ): $(nproc) vCPU, ${CONCURRENT_CELLS} cells x ${THREADS_PER_CELL} threads ==="

# AL2023 ships python3 and awscli-2, but be explicit: a missing aws binary
# here would look like an empty work queue rather than a broken instance.
dnf install -y -q gcc git tar gzip python3 ripgrep || dnf install -y gcc git tar gzip python3
command -v aws >/dev/null || dnf install -y -q awscli-2
command -v aws >/dev/null || { echo 'FATAL: no aws cli'; shutdown -h now; }
# Toolchain is installed only if we actually have to build - see below.
install_rust() {
  command -v cargo >/dev/null && return 0
  curl -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain stable
  # shellcheck disable=SC1091
  [ -f "$HOME/.cargo/env" ] && source "$HOME/.cargo/env"
  command -v cargo >/dev/null || { echo "FATAL: no cargo on PATH"; shutdown -h now; }
}

mkdir -p "$WORK" && cd "$WORK"
aws s3 cp "s3://$BUCKET/input/source.tar.gz" . --quiet
tar xzf source.tar.gz
mkdir -p data/shd/events
aws s3 cp "s3://$BUCKET/input/train.events" data/shd/events/train.events --quiet
aws s3 cp "s3://$BUCKET/input/test.events" data/shd/events/test.events --quiet
aws s3 cp "s3://$BUCKET/input/cells.json" cells.json --quiet

# --- the campaign binary is PINNED ------------------------------------------
#
# A campaign whose cells came from more than one binary is not one experiment.
# The source tree moves while a fleet runs - a new wave, a new test, a fixed
# script - and any of that changes the compiled hash even when it cannot change
# what `shd-instrument` computes. So the first instance builds, publishes its
# binary and its sha256, and every later instance downloads exactly that.
#
# If a pinned binary is published, using it is mandatory and a hash mismatch is
# fatal. Building "just in case" is precisely how a fleet ends up mixed.
BIN="$WORK/target/release/shd-instrument"
mkdir -p "$WORK/target/release"
if aws s3 cp "s3://$BUCKET/input/binary.sha256" /tmp/pinned.sha256 --quiet 2>/dev/null; then
  PINNED="$(tr -d ' \n' < /tmp/pinned.sha256)"
  aws s3 cp "s3://$BUCKET/input/shd-instrument" "$BIN" --quiet
  chmod +x "$BIN"
  BIN_SHA="$(sha256sum "$BIN" | cut -d' ' -f1)"
  if [ "$BIN_SHA" != "$PINNED" ]; then
    echo "FATAL: pinned binary is $PINNED but the download hashes $BIN_SHA"
    shutdown -h now
  fi
  echo "using pinned binary $BIN_SHA (no build)"
else
  echo "no pinned binary published; building and publishing one"
  install_rust
  cargo build --locked --release -p binn-lab --bin shd-instrument
  BIN_SHA="$(sha256sum "$BIN" | cut -d' ' -f1)"
  aws s3 cp "$BIN" "s3://$BUCKET/input/shd-instrument" --quiet
  printf '%s\n' "$BIN_SHA" > /tmp/binary.sha256
  aws s3 cp /tmp/binary.sha256 "s3://$BUCKET/input/binary.sha256" --quiet
  echo "built and published binary sha256 $BIN_SHA"
fi

# --- cross-machine reproducibility gate -----------------------------------
#
# The recorded cells were produced on macOS/aarch64. `exp`, `sin`, `cos`, `powf`
# and `ln` come from libm, and glibc's are not obliged to agree with Apple's to
# the last ulp. A one-ulp difference flips a spike and compounds through Adam,
# which is why this repository demands bit-exactness rather than a tolerance.
#
# Measure it; do not assume it either way. Gate F re-runs recorded cells from
# their pinned initialisation and demands every scientific field match exactly.
#
# A FAIL does NOT stop the campaign. It means absolute comparisons against the
# macOS record are unlicensed, and every claim must rest on the control arm that
# ran beside its treatment on this same machine — which every wave in
# `plan_cells.py` carries for exactly this reason. The result is recorded either
# way, because "the check could not run" and "the check ran and passed" must
# never look the same downstream.
if BINN_CAMPAIGN_AUTHORIZED=1 RAYON_NUM_THREADS="$THREADS_PER_CELL" \
     python3 scripts/gate_f_rust.py --cheapest 3 > /tmp/gate.log 2>&1; then
  GATE_STATUS=PASS
else
  GATE_STATUS=FAIL
fi
echo "cross-machine Gate F: $GATE_STATUS"
printf '{"instance":"%s","binary_sha256":"%s","cross_machine_gate_f":"%s","uname":"%s","utc":"%s"}\n' \
  "$INSTANCE_ID" "$BIN_SHA" "$GATE_STATUS" "$(uname -srm)" "$(date -u +%FT%TZ)" > /tmp/gate.json
aws s3 cp /tmp/gate.json "s3://$BUCKET/gates/$INSTANCE_ID.json" --quiet
aws s3 cp /tmp/gate.log "s3://$BUCKET/gates/$INSTANCE_ID.log" --quiet

# --- provenance ------------------------------------------------------------
#
# Nothing in a cell's JSON says which host produced it, and the campaign's
# preregistration requires every reported wave to carry its instance's Gate F
# verdict. Instances self-terminate, so the mapping has to leave the box while
# the box is alive. This loop ships the bootstrap log - which names every cell
# each slot claims - to S3 once a minute, for the whole life of the instance.
(
  while true; do
    aws s3 cp /var/log/binn-bootstrap.log \
      "s3://$BUCKET/hostlogs/$INSTANCE_ID.log" --quiet 2>/dev/null || true
    sleep 60
  done
) &

# --- work loop ------------------------------------------------------------
worker() {
  local slot="$1"
  while true; do
    local id
    id="$(python3 scripts/aws/claim_next.py "$BUCKET")" || { echo "slot $slot: claim error"; sleep 15; continue; }
    if [[ -z "$id" ]]; then
      echo "slot $slot: no work left"
      return 0
    fi
    echo "slot $slot: running $id"
    if python3 scripts/aws/run_cell.py "$id" --work "/tmp/$id" --binary "$BIN" \
         --threads "$THREADS_PER_CELL" > "/tmp/$id.log" 2>&1; then
      aws s3 cp "/tmp/$id/cell.json" "s3://$BUCKET/results/$id.json" --quiet
    else
      echo "slot $slot: FAILED $id"
      aws s3 cp "/tmp/$id.log" "s3://$BUCKET/failures/$id.log" --quiet
    fi
    aws s3 cp "/tmp/$id.log" "s3://$BUCKET/logs/$id.log" --quiet
    rm -rf "/tmp/$id" "/tmp/$id.log"
  done
}

for slot in $(seq 1 "$CONCURRENT_CELLS"); do worker "$slot" & done
wait
echo "=== all workers idle at $(date -u +%FT%TZ); shutting down ==="
shutdown -h now
