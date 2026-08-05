#!/usr/bin/env bash
# Deprecated: Python/matplotlib plots venv is removed.
# Use plotters via Cargo instead:
#   cargo run --locked --release -p binn-lab --features plots --bin c1 -- …
#   cargo run --locked --release -p binn-lab --features plots --bin paper-figures
set -euo pipefail
echo "setup_plots_venv.sh is retired (no Python plots stack)." >&2
echo "Enable Rust plotters with: cargo … -p binn-lab --features plots" >&2
exit 2
