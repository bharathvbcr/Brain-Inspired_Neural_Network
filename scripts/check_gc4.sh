#!/usr/bin/env bash
# GC4: fixed encoders through crux — no train/fit on Encoder/Decoder until P4+.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

# Look for method definitions named train/fit on encoder/decoder modules.
hits="$(rg -n --glob 'binn-data/src/{encoder,decoder}.rs' -e '^\s*(pub\s+)?(fn|async\s+fn)\s+(train|fit)\b' || true)"
if [[ -n "$hits" ]]; then
  echo "GC4 FAIL: Encoder/Decoder exposes train/fit before P4:"
  echo "$hits"
  exit 1
fi
echo "GC4 PASS: no Encoder/Decoder train/fit methods"
