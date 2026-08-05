#!/usr/bin/env bash
# GC7: execute the harness refusal test for missing activity sparsity.
# Hardened like GC3: `--exact` match + assert ≥1 test actually ran
# (cargo test exits 0 on an empty filter).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

package="binn-lab"
test_name="logging::tests::gc7_refuses_emit_without_activity_sparsity"

out="$(cargo test --locked -p "$package" "$test_name" -- --exact 2>&1)"
if ! echo "$out" | grep -Eq 'test result: ok\. [1-9][0-9]* passed'; then
  echo "GC7 FAIL: ${package}::${test_name} did not run (matched 0 tests?)"
  echo "$out" | tail -8
  exit 1
fi
echo "GC7 PASS: result emission rejects missing activity_sparsity (>=1 test executed)"
