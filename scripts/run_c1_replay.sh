#!/usr/bin/env bash
# Run C1 with replay export enabled, then point at the offline viewer.
#
# Viz only: no effect on config hashes, accuracies, or the GC7 log.
# The export is written by the local-assembly condition; with multiple seeds
# the last seed wins (same convention as the plot PNGs).
#
# Examples:
#   ./scripts/run_c1_replay.sh --quick
#   ./scripts/run_c1_replay.sh --config-hash c1-118207fbc3eaba53
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

OUT="${BINN_REPLAY_OUT:-results/c1_replay.json}"

cargo run --locked --release -p binn-lab --bin c1 -- --replay "$OUT" "$@"

echo
echo "replay export: $OUT"
echo "open viz/replay_viewer.html in a browser and load it (or drag-drop)."
