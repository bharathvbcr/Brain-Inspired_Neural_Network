#!/usr/bin/env bash
# GC1: no dense matmul / autograd on the production path.
# Ban symbols outside *_baseline.rs files.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

pattern='matmul|dense_layer|autograd|backward\('
# Search Rust sources; exclude labeled baseline files (GC1-exempt).
hits="$(rg -n --glob '*.rs' --glob '!*_baseline.rs' -e "$pattern" . || true)"
if [[ -n "$hits" ]]; then
  echo "GC1 FAIL: banned symbols outside *_baseline.rs:"
  echo "$hits"
  exit 1
fi
echo "GC1 PASS: no banned production-path symbols"
