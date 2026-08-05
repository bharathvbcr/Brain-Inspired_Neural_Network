#!/usr/bin/env bash
# GC3: execute one real same-seed fingerprint test in every stateful crate.
# Hardened: a named test that matches ZERO tests must FAIL (cargo test exits 0
# on an empty filter, so we assert at least one test actually ran).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

checks=(
  # Unit-test paths must include their module prefix when using `--exact`.
  "binn-core determinism_gc3::gc3_same_seed_identical_output_hash"
  "binn-engine engine::tests::seed_identical_spike_train"
  "binn-areas gc3_same_seed_identical_wiring_assembly_hash"
  "binn-learn gc3_same_seed_identical_weight_update_hash"
  "binn-learn surrogate_lif_baseline::tests::deterministic_same_seed"
  "binn-data same_seed_identical_encoding_hash"
  "binn-lab c1_same_seed_identical_seed_accuracies"
)

for check in "${checks[@]}"; do
  read -r package test_name <<<"$check"
  # `-- --exact` forces an exact-name match in the test binary.
  out="$(cargo test --locked -p "$package" "$test_name" -- --exact 2>&1)"
  if ! echo "$out" | grep -Eq 'test result: ok\. [1-9][0-9]* passed'; then
    echo "GC3 FAIL: ${package}::${test_name} did not run (matched 0 tests?)"
    echo "$out" | tail -8
    exit 1
  fi
done
echo "GC3 PASS: determinism fingerprints executed (>=1 test each) for all stateful crates"
