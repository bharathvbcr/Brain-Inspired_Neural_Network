#!/usr/bin/env bash
# GC5: all declared hot paths have real Criterion benches that compile.
# Hardened: every declared bench file (incl. SIMD + timing-wheel) must exist,
# be non-empty, and actually be a criterion bench (contain criterion_main!).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

required=(
  "binn-core/benches/simd_leak_integrate.rs"
  "binn-engine/benches/timing_wheel.rs"
  "binn-engine/benches/engine_step.rs"
  "binn-areas/benches/kwta.rs"
  "binn-learn/benches/plasticity_update.rs"
)
for path in "${required[@]}"; do
  if [[ ! -s "$path" ]]; then
    echo "GC5 FAIL: missing non-empty benchmark $path"
    exit 1
  fi
  if ! grep -q 'criterion_main!' "$path"; then
    echo "GC5 FAIL: $path is not a real criterion bench (no criterion_main!)"
    exit 1
  fi
done
cargo bench --locked --workspace --no-run --quiet
echo "GC5 PASS: simd, timing-wheel, engine-step, k-WTA, and plasticity benches are real and compile"
