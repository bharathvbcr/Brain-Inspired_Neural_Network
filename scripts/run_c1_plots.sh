#!/usr/bin/env bash
# Run C1 with the optional Rust plotters feature (no Python).
#
#   ./scripts/run_c1_plots.sh --quick
#   ./scripts/run_c1_plots.sh --config-hash c1-118207fbc3eaba53 --out results/c1_g2_plots.md
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
exec cargo run --locked --release -p binn-lab --features plots --bin c1 -- "$@"
