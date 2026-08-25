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
  # binn-hybrid-lab carries four determinism tests that GC3 never confirmed had
  # run, while the message below claimed "all stateful crates".
  "binn-hybrid-lab benchmark::tests::feasibility_replay_is_deterministic"
)

#: Crates with no determinism fingerprint, and why. Named rather than omitted:
#: the difference between "this crate has nothing to fingerprint" and "nobody
#: added it to the list" is exactly what the old wording erased.
declare -a EXEMPT=(
  "binn-hybrid-learn no seeded state; every routine is a pure transform of its inputs"
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
# An exemption that has stopped being true is worse than no exemption: it says a
# crate was considered when it no longer is. Re-derive it rather than trust it.
for entry in "${EXEMPT[@]}"; do
  read -r package _reason <<<"$entry"
  if [[ ! -d "$package" ]]; then
    echo "GC3 CANNOT RUN: exempt crate ${package} no longer exists; fix the list."
    exit 1
  fi
  if rg -q --glob "${package}/src/**/*.rs" -e 'fn .*determinis|determinis.*\(\)' . </dev/null 2>/dev/null; then
    echo "GC3 FAIL: ${package} is on the exempt list but now has a determinism test."
    echo "Add it to the checks above so the fingerprint is actually executed."
    exit 1
  fi
done

echo "GC3 PASS: determinism fingerprints executed (>=1 test each) for ${#checks[@]} \
entries covering every crate except: ${EXEMPT[*]%% *}"
