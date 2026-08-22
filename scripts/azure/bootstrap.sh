#!/usr/bin/env bash
# Azure VMSS bootstrap for the preregistered BINN CPU campaign.
set -euo pipefail

BINN_CARGO_HOME=/root/.cargo
export PATH="$BINN_CARGO_HOME/bin:$PATH"

STORAGE_ACCOUNT="${STORAGE_ACCOUNT:?}"
CONTAINER="${CONTAINER:?}"
NODE_COUNT="${NODE_COUNT:?}"
VMSS_NAME="${VMSS_NAME:?}"
RESOURCE_GROUP="${RESOURCE_GROUP:?}"
SUBSCRIPTION_ID="${SUBSCRIPTION_ID:?}"
SOURCE_SHA256="${SOURCE_SHA256:?}"
THREADS_PER_CELL="${THREADS_PER_CELL:-4}"
WIDE_THREADS_PER_CELL="${WIDE_THREADS_PER_CELL:-8}"
CONCURRENT_CELLS="${CONCURRENT_CELLS:-16}"
CELL_TIMEOUT_SECS="${CELL_TIMEOUT_SECS:-86400}"
MAX_RUNTIME_SECONDS="${MAX_RUNTIME_SECONDS:-68400}"
WORK=/opt/binn
API_VERSION=2023-11-03

mkdir -p /var/log/binn /opt/binn
exec > >(tee -a /var/log/binn/bootstrap.log) 2>&1

metadata() {
  curl -fsS -H Metadata:true "http://169.254.169.254/metadata/$1?api-version=2021-02-01&format=text"
}

storage_token() {
  curl -fsS -H Metadata:true \
    'http://169.254.169.254/metadata/identity/oauth2/token?api-version=2018-02-01&resource=https%3A%2F%2Fstorage.azure.com%2F' \
    | python3 -c 'import json,sys; print(json.load(sys.stdin)["access_token"])'
}

management_token() {
  curl -fsS -H Metadata:true \
    'http://169.254.169.254/metadata/identity/oauth2/token?api-version=2018-02-01&resource=https%3A%2F%2Fmanagement.azure.com%2F' \
    | python3 -c 'import json,sys; print(json.load(sys.stdin)["access_token"])'
}

blob_get() {
  local name="$1" target="$2" token
  token="$(storage_token)" || return 1
  curl -fsS -H "Authorization: Bearer $token" -H "x-ms-version: $API_VERSION" \
    "https://$STORAGE_ACCOUNT.blob.core.windows.net/$CONTAINER/$name" -o "$target"
}

blob_put() {
  local source="$1" name="$2" token
  token="$(storage_token)" || return 1
  curl -fsS -X PUT -H "Authorization: Bearer $token" -H "x-ms-version: $API_VERSION" \
    -H 'x-ms-blob-type: BlockBlob' --data-binary "@$source" \
    "https://$STORAGE_ACCOUNT.blob.core.windows.net/$CONTAINER/$name"
}

deallocate_all() {
  local token
  token="$(management_token)" || return 1
  curl -fsS -X POST -H "Authorization: Bearer $token" -H 'Content-Length: 0' \
    "https://management.azure.com/subscriptions/$SUBSCRIPTION_ID/resourceGroups/$RESOURCE_GROUP/providers/Microsoft.Compute/virtualMachineScaleSets/$VMSS_NAME/deallocate?api-version=2025-04-01"
}

deallocate_self() {
  local token
  token="$(management_token)" || return 1
  curl -fsS -X POST -H "Authorization: Bearer $token" -H 'Content-Length: 0' \
    "https://management.azure.com/subscriptions/$SUBSCRIPTION_ID/resourceGroups/$RESOURCE_GROUP/providers/Microsoft.Compute/virtualMachineScaleSets/$VMSS_NAME/virtualMachines/$NODE_INDEX/deallocate?api-version=2025-04-01"
}

if ! INSTANCE_RESOURCE_ID="$(metadata 'instance/compute/resourceId')"; then
  echo "FATAL: Azure instance resource ID is unavailable"
  deallocate_all || true
  exit 2
fi
NODE_INDEX="${INSTANCE_RESOURCE_ID##*/}"
if ! [[ "$NODE_INDEX" =~ ^[0-9]+$ ]] || (( NODE_INDEX >= NODE_COUNT )); then
  echo "FATAL: invalid VMSS shard index $NODE_INDEX for node count $NODE_COUNT"
  deallocate_all || true
  exit 2
fi
echo "=== node $NODE_INDEX/$NODE_COUNT start $(date -u +%FT%TZ), $(nproc) vCPU ==="

# Hard spend guard starts before package installation or compilation. Omitting
# instance IDs deallocates the whole scale set, which stops compute billing.
(
  sleep "$MAX_RUNTIME_SECONDS"
  echo "hard runtime limit reached; deallocating scale set"
  deallocate_all || true
) &

export DEBIAN_FRONTEND=noninteractive
apt-get update -qq
apt-get install -y -qq build-essential ca-certificates curl gzip python3 tar

mkdir -p "$WORK"
for attempt in $(seq 1 30); do
  if blob_get input/source.tar.gz "$WORK/source.tar.gz"; then break; fi
  [[ "$attempt" == 30 ]] && { echo "FATAL: storage access unavailable"; deallocate_all || true; exit 2; }
  sleep 10
done
blob_get input/cells.json "$WORK/cells.json"
mkdir -p "$WORK/data/shd/events"
blob_get input/train.events "$WORK/data/shd/events/train.events"
blob_get input/test.events "$WORK/data/shd/events/test.events"
cd "$WORK"
tar xzf source.tar.gz

if ! command -v cargo >/dev/null; then
  curl -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain stable
fi

BIN="$WORK/target/release/shd-instrument"
# Build exactly once at the Dalsv7 family's portable x86-64-v4 baseline. EPYC
# 9005 provides a full-width AVX-512 path; every other node downloads that
# binary, eliminating duplicate compilation and native-build drift.
if [[ "$NODE_INDEX" == 0 ]]; then
  RUSTFLAGS='-C target-cpu=x86-64-v4' \
    cargo build --locked --release -p binn-lab --bin shd-instrument
  sha256sum "$BIN" | cut -d' ' -f1 > /tmp/binary.sha256
  blob_put "$BIN" input/shd-instrument
  blob_put /tmp/binary.sha256 input/binary.sha256
else
  mkdir -p "$WORK/target/release"
fi

# No cell runs until the pinned binary is available and verifies locally.
deadline=$(( $(date +%s) + 1800 ))
while true; do
  if blob_get input/binary.sha256 /tmp/binary.sha256 2>/dev/null \
     && blob_get input/shd-instrument "$BIN" 2>/dev/null; then
    chmod +x "$BIN"
    EXPECTED_SHA="$(tr -d ' \n' < /tmp/binary.sha256)"
    OBSERVED_SHA="$(sha256sum "$BIN" | cut -d' ' -f1)"
    [[ "$EXPECTED_SHA" == "$OBSERVED_SHA" ]] && break
  fi
  if [[ $(date +%s) -ge $deadline ]]; then
    echo "FATAL: binary quorum did not form within 30 minutes"
    deallocate_all || true
    exit 2
  fi
  sleep 15
done
BIN_SHA="$OBSERVED_SHA"

python3 - "$NODE_INDEX" "$BIN_SHA" > "/tmp/node-$NODE_INDEX-host.json" <<'PY'
import json, platform, subprocess, sys
print(json.dumps({
    "node": int(sys.argv[1]), "binary_sha256": sys.argv[2],
    "platform": platform.platform(),
    "machine": platform.machine(),
    "processor": subprocess.run(["lscpu"], text=True, capture_output=True).stdout,
}, sort_keys=True))
PY
blob_put "/tmp/node-$NODE_INDEX-host.json" "hosts/node-$NODE_INDEX.json"

GATE_OUTPUT="$WORK/results/shd_instrument_v4/gate-f-rust"
# The source archive carries prior local evidence. It must not be mistaken for
# output from this launch if Gate F crashes before producing a fresh report.
rm -rf "$GATE_OUTPUT"
mkdir -p "$GATE_OUTPUT"
set +e
BINN_CAMPAIGN_AUTHORIZED=1 RAYON_NUM_THREADS="$THREADS_PER_CELL" \
  python3 scripts/gate_f_rust.py --cheapest 3 --binary "$BIN" \
  > "/tmp/node-$NODE_INDEX-gate.log" 2>&1
GATE_EXIT=$?
set -e
if [[ "$GATE_EXIT" == 0 ]]; then GATE_STATUS=PASS; else GATE_STATUS=FAIL; fi
blob_put "/tmp/node-$NODE_INDEX-gate.log" "gates/node-$NODE_INDEX.log"
if ! python3 scripts/azure/gate_quorum.py attest \
  --node "$NODE_INDEX" --source-sha256 "$SOURCE_SHA256" \
  --binary-sha256 "$BIN_SHA" --gate-report "$GATE_OUTPUT/report.json" \
  --gate-output-dir "$GATE_OUTPUT" --utc "$(date -u +%FT%TZ)" \
  --out "/tmp/node-$NODE_INDEX-gate.json"; then
  echo "FATAL: Gate F did not produce a complete current-launch attestation"
  blob_put /var/log/binn/bootstrap.log "hostlogs/node-$NODE_INDEX.log" || true
  deallocate_all || true
  exit 2
fi
blob_put "/tmp/node-$NODE_INDEX-gate.json" "gates/node-$NODE_INDEX.json"

# The recorded macOS reference is expected to fail on Linux. Scientific work is
# licensed only when all four current Azure nodes agree exactly on every Gate F
# scientific field, while retaining that cross-platform FAIL in the evidence.
QUORUM_DIR=/tmp/azure-gate-quorum
mkdir -p "$QUORUM_DIR"
deadline=$(( $(date +%s) + 1800 ))
while true; do
  REPORTS_READY=1
  for quorum_node in $(seq 0 $((NODE_COUNT - 1))); do
    if blob_get "gates/node-$quorum_node.json" "$QUORUM_DIR/node-$quorum_node.download" 2>/dev/null; then
      mv "$QUORUM_DIR/node-$quorum_node.download" "$QUORUM_DIR/node-$quorum_node.json"
    else
      REPORTS_READY=0
    fi
  done
  if [[ "$REPORTS_READY" == 1 ]]; then
    set +e
    python3 scripts/azure/gate_quorum.py validate \
      --reports-dir "$QUORUM_DIR" --node-count "$NODE_COUNT" \
      --source-sha256 "$SOURCE_SHA256" --binary-sha256 "$BIN_SHA" \
      --expected-cross-platform-status FAIL --expected-gate-cells 3 \
      --out /tmp/azure-gate-quorum.json
    QUORUM_STATUS=$?
    set -e
    if [[ "$QUORUM_STATUS" == 0 ]]; then break; fi
    if [[ "$QUORUM_STATUS" != 3 ]]; then
      echo "FATAL: current-launch Azure Gate F quorum mismatched"
      blob_put /var/log/binn/bootstrap.log "hostlogs/node-$NODE_INDEX.log" || true
      deallocate_all || true
      exit 2
    fi
  fi
  if [[ $(date +%s) -ge $deadline ]]; then
    echo "FATAL: 4/$NODE_COUNT current-launch Azure Gate F quorum did not form within 30 minutes"
    blob_put /var/log/binn/bootstrap.log "hostlogs/node-$NODE_INDEX.log" || true
    deallocate_all || true
    exit 2
  fi
  sleep 15
done
blob_put /tmp/azure-gate-quorum.json "gates/quorum-node-$NODE_INDEX.json"
if [[ "$NODE_INDEX" == 0 ]]; then
  blob_put /tmp/azure-gate-quorum.json gates/quorum.json
fi
echo "Azure-local Gate F quorum PASS; recorded macOS-to-Azure Gate F $GATE_STATUS"

set +e
python3 scripts/azure/run_shard.py \
  --node-index "$NODE_INDEX" --node-count "$NODE_COUNT" \
  --plan cells.json --binary "$BIN" --events data/shd/events \
  --threads "$THREADS_PER_CELL" --wide-threads "$WIDE_THREADS_PER_CELL" \
  --concurrency "$CONCURRENT_CELLS" \
  --cell-timeout-secs "$CELL_TIMEOUT_SECS" \
  --storage-account "$STORAGE_ACCOUNT" --container "$CONTAINER"
RUN_STATUS=$?
set -e
blob_put /var/log/binn/bootstrap.log "hostlogs/node-$NODE_INDEX.log" || true
echo "node $NODE_INDEX finished with $RUN_STATUS at $(date -u +%FT%TZ); deallocating itself"
deallocate_self || true
exit "$RUN_STATUS"
