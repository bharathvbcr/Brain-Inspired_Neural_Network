#!/usr/bin/env bash
# GC7: execute the harness refusal test for missing activity sparsity.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

cargo test --locked -p binn-lab gc7_refuses_emit_without_activity_sparsity --quiet
echo "GC7 PASS: result emission rejects missing activity_sparsity"
