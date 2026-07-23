#!/usr/bin/env bash
# GC2: no external ML framework deps.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

tree="$(cargo tree --workspace -e normal,build 2>/dev/null || cargo tree --workspace)"
banned='torch|tch|candle|burn|dfdx'
hits="$(printf '%s\n' "$tree" | rg -n -e "$banned" || true)"
if [[ -n "$hits" ]]; then
  echo "GC2 FAIL: banned ML framework dependency in cargo tree:"
  echo "$hits"
  exit 1
fi
echo "GC2 PASS: no banned ML framework dependencies"
