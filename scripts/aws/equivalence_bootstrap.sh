#!/usr/bin/env bash
# Instance bootstrap for the source-versus-pinned-binary equivalence test.
#
# Registered in `results/PREREG_2026-08-22_SOURCE_VERSUS_PINNED_BINARY.md`.
#
# Runs both binaries on one host in one boot, so the OS, the libm and the CPU
# are held fixed and the only variable is the binary. Writes only under
# `equivalence/` — `input/` holds the pinned binary, the corpus and the campaign
# plan, and none of it is touched.
set -uo pipefail

export HOME="${HOME:-/root}"
export PATH="$HOME/.cargo/bin:$PATH"
BUCKET="${BUCKET:?BUCKET must be set}"
THREADS="${THREADS:-4}"
WORK=/opt/binn-equiv
PREFIX="equivalence"

exec > >(tee -a /var/log/binn-equivalence.log) 2>&1

TOKEN="$(curl -s -X PUT 'http://169.254.169.254/latest/api/token' -H 'X-aws-ec2-metadata-token-ttl-seconds: 600')"
INSTANCE_ID="$(curl -s -H "X-aws-ec2-metadata-token: $TOKEN" http://169.254.169.254/latest/meta-data/instance-id)"
echo "=== equivalence bootstrap on ${INSTANCE_ID:-unknown} at $(date -u +%FT%TZ): $(nproc) vCPU ==="

# A watchdog, because an instance that hangs costs money silently and an
# instance that dies without saying so is indistinguishable from one that never
# started. 3 h is ~4x the expected 45 min.
( sleep 10800; echo "WATCHDOG: 3 h elapsed, shutting down"; shutdown -h now ) &

fatal() { echo "FATAL: $*"; aws s3 cp /var/log/binn-equivalence.log "s3://$BUCKET/$PREFIX/bootstrap-${INSTANCE_ID}.log" --quiet || true; shutdown -h now; }

dnf install -y -q gcc git tar gzip python3 || dnf install -y gcc git tar gzip python3
command -v aws >/dev/null || dnf install -y -q awscli-2
command -v aws >/dev/null || fatal "no aws cli"

mkdir -p "$WORK" && cd "$WORK"

# --- today's source ---------------------------------------------------------
aws s3 cp "s3://$BUCKET/$PREFIX/source.tar.gz" . --quiet || fatal "no source tarball"
tar xzf source.tar.gz || fatal "source tarball did not extract"
aws s3 cp "s3://$BUCKET/$PREFIX/plan.json" cells.json --quiet || fatal "no plan"

# --- corpus (static, shared with the campaign, read only) -------------------
mkdir -p data/shd/events
aws s3 cp "s3://$BUCKET/input/train.events" data/shd/events/train.events --quiet || fatal "no train corpus"
aws s3 cp "s3://$BUCKET/input/test.events" data/shd/events/test.events --quiet || fatal "no test corpus"

# --- the pinned binary, and its hash is not advisory -------------------------
mkdir -p "$WORK/pinned"
PINNED_BIN="$WORK/pinned/shd-instrument"
aws s3 cp "s3://$BUCKET/input/binary.sha256" /tmp/pinned.sha256 --quiet || fatal "no pinned hash"
aws s3 cp "s3://$BUCKET/input/shd-instrument" "$PINNED_BIN" --quiet || fatal "no pinned binary"
chmod +x "$PINNED_BIN"
PINNED_SHA="$(tr -d ' \n' < /tmp/pinned.sha256)"
GOT_SHA="$(sha256sum "$PINNED_BIN" | cut -d' ' -f1)"
[ "$GOT_SHA" = "$PINNED_SHA" ] || fatal "pinned binary is $PINNED_SHA but the download hashes $GOT_SHA"
echo "pinned binary verified: $PINNED_SHA"

# --- today's binary ----------------------------------------------------------
curl -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain stable
# shellcheck disable=SC1091
[ -f "$HOME/.cargo/env" ] && source "$HOME/.cargo/env"
command -v cargo >/dev/null || fatal "no cargo on PATH"
cargo build --locked --release -p binn-lab --bin shd-instrument || fatal "build failed"
TODAY_BIN="$WORK/target/release/shd-instrument"
TODAY_SHA="$(sha256sum "$TODAY_BIN" | cut -d' ' -f1)"
echo "today's binary: $TODAY_SHA"
# The two hashes are EXPECTED to differ - different toolchain, different build
# path. Byte equality is not the question; behaviour is.
[ "$TODAY_SHA" = "$PINNED_SHA" ] && echo "note: the two binaries are byte-identical, which makes E-1 trivially true"

# --- run every cell with both binaries, concurrently -------------------------
CELLS="$(python3 -c 'import json;print(" ".join(c["id"] for c in json.load(open("cells.json"))))')"
echo "cells: $CELLS"
mkdir -p "$WORK/out"
pids=()
for cell in $CELLS; do
  for which in pinned today; do
    bin="$PINNED_BIN"; [ "$which" = "today" ] && bin="$TODAY_BIN"
    out="$WORK/out/$which/$cell"
    mkdir -p "$out"
    ( python3 scripts/aws/run_cell.py "$cell" --plan cells.json --work "$out" \
        --binary "$bin" --threads "$THREADS" > "$out/run.log" 2>&1; \
      echo "$?" > "$out/exit" ) &
    pids+=($!)
  done
done

# E-4: the same build, the same cell, one thread instead of four.
T1_CELL="$(python3 -c 'import json;print([c["id"] for c in json.load(open("cells.json")) if c["attn_dim"] and c["epochs"]==5][0])')"
T1_OUT="$WORK/out/today-t1/$T1_CELL"
mkdir -p "$T1_OUT"
( python3 scripts/aws/run_cell.py "$T1_CELL" --plan cells.json --work "$T1_OUT" \
    --binary "$TODAY_BIN" --threads 1 > "$T1_OUT/run.log" 2>&1; \
  echo "$?" > "$T1_OUT/exit" ) &
pids+=($!)

for p in "${pids[@]}"; do wait "$p"; done
echo "=== all runs finished at $(date -u +%FT%TZ) ==="

# --- provenance, so a disagreement can be attributed -------------------------
python3 - <<PY > "$WORK/out/environment.json"
import json, platform, subprocess
def sh(*a):
    try: return subprocess.run(a, capture_output=True, text=True).stdout.strip()
    except Exception as e: return f"<{e}>"
print(json.dumps({
  "instance": "${INSTANCE_ID}",
  "pinned_sha256": "${PINNED_SHA}",
  "today_sha256": "${TODAY_SHA}",
  "threads": "${THREADS}",
  "uname": sh("uname", "-srm"),
  "glibc": sh("ldd", "--version").splitlines()[0] if sh("ldd", "--version") else "",
  "rustc": sh("rustc", "--version"),
  "cpu": sh("bash", "-lc", "grep -m1 'BogoMIPS\\|Features' /proc/cpuinfo || true"),
  "nproc": sh("nproc"),
  "python": platform.python_version(),
}, indent=1))
PY

aws s3 cp "$WORK/out" "s3://$BUCKET/$PREFIX/out/" --recursive --quiet
aws s3 cp /var/log/binn-equivalence.log "s3://$BUCKET/$PREFIX/bootstrap-${INSTANCE_ID}.log" --quiet
echo "=== uploaded; shutting down ==="
shutdown -h now
