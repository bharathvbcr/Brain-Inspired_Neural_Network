#!/usr/bin/env bash
# GC1: no dense matmul / autograd on the production path.
# Ban symbols outside *_baseline.rs files.
#
# The scan lives in `gc1_scan.py` because it has to strip comments before
# matching, and because it self-calibrates. See that file for why.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
exec python3 scripts/gc1_scan.py
