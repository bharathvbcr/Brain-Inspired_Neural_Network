#!/usr/bin/env bash
# Foundation Microcircuit ~1e6 syn scientific isolate loop.
#   ./scripts/run_foundation_micro.sh [--quick]
set -euo pipefail
cd "$(cd "$(dirname "$0")/.." && pwd)"

CAMP="results/runs/2026-07-24-foundation-micro"
mkdir -p "$CAMP"
QUICK=0
for a in "$@"; do
  case "$a" in
    --quick) QUICK=1 ;;
    *) echo "unknown arg: $a"; exit 2 ;;
  esac
done

C1=./target/release/c1
if [[ ! -x "$C1" ]]; then
  cargo build --locked --release -p binn-lab --bin c1
fi

FLAGS=(--foundation-micro --isolate-condition local-assembly)
TAG="scientific"
if [[ "$QUICK" -eq 1 ]]; then
  FLAGS+=(--quick)
  TAG="quick"
  SEEDS=($(seq 1 2))
else
  SEEDS=($(seq 1 20))
fi

echo "=== foundation-micro ${TAG} camp=${CAMP} $(date -u +%Y-%m-%dT%H:%M:%SZ) ==="
"$C1" "${FLAGS[@]}" --seed "${SEEDS[0]}" 2>&1 | head -6 || true

for s in "${SEEDS[@]}"; do
  echo "--- seed ${s} ---"
  "$C1" "${FLAGS[@]}" --seed "$s" \
    > "${CAMP}/${TAG}-seed${s}.json" \
    2>"${CAMP}/${TAG}-seed${s}.err"
done

echo "=== done $(date -u +%Y-%m-%dT%H:%M:%SZ) ==="
