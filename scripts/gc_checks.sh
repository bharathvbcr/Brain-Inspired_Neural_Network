#!/usr/bin/env bash
# Run all BINN global-constraint (GC1–GC7) checks.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")" && pwd)"

fail=0
for n in 1 2 3 4 5 6 7; do
  script="$ROOT/check_gc${n}.sh"
  echo "=== GC${n} ==="
  if ! bash "$script"; then
    fail=1
  fi
done

if [[ "$fail" -ne 0 ]]; then
  echo "One or more GC checks failed."
  exit 1
fi
echo "All GC1–GC7 checks executed and passed."
