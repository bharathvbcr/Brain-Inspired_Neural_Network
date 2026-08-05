#!/usr/bin/env bash
# H2 dfa-live size protocol — scientific 8 seeds × {pm1, structured-fb, dfa-live}.
#   ./scripts/run_dfa_live_size.sh [--quick]
set -euo pipefail
cd "$(cd "$(dirname "$0")/.." && pwd)"

CAMP="results/runs/2026-07-24-dfa-live-size"
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

TAG="scientific"
N_SEEDS=8
if [[ "$QUICK" -eq 1 ]]; then
  TAG="quick"
  N_SEEDS=2
fi

echo "=== dfa-live-size ${TAG} n_seeds=${N_SEEDS} camp=${CAMP} $(date -u +%Y-%m-%dT%H:%M:%SZ) ==="

for mode in pm1 structured-fb dfa-live; do
  for s in $(seq 1 "$N_SEEDS"); do
    echo "--- ${mode} seed ${s} ---"
    if [[ "$QUICK" -eq 1 ]]; then
      "$C1" --dfa-live-size --mac-mode "$mode" --quick \
        --isolate-condition local-assembly --seed "$s" \
        > "${CAMP}/${TAG}-${mode}-seed${s}.json" \
        2>"${CAMP}/${TAG}-${mode}-seed${s}.err"
    else
      "$C1" --dfa-live-size --mac-mode "$mode" \
        --isolate-condition local-assembly --seed "$s" \
        > "${CAMP}/${TAG}-${mode}-seed${s}.json" \
        2>"${CAMP}/${TAG}-${mode}-seed${s}.err"
    fi
  done
done

echo "=== done $(date -u +%Y-%m-%dT%H:%M:%SZ) ==="
