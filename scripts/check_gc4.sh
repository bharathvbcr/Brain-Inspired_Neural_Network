#!/usr/bin/env bash
# GC4: fixed encoders through crux — no train/fit on Encoder/Decoder until P4+.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

# The files this gate exists to watch. Named as explicit path arguments, not as
# a --glob with no path: ripgrep given no path reads *stdin*, so the globbed
# form searched whatever the caller's stdin happened to be. Under a pipeline it
# blocked on a read that never returned and the gate hung forever at 0% CPU
# instead of answering.
TARGETS=(binn-data/src/encoder.rs binn-data/src/decoder.rs)

# A gate that cannot run must not report what a gate that ran and passed
# reports. If either file is renamed or moved, "no matches" would otherwise be
# indistinguishable from "no violations" — and GC4 would pass forever without
# reading anything.
for f in "${TARGETS[@]}"; do
  if [[ ! -f "$f" ]]; then
    echo "GC4 CANNOT RUN: $f does not exist; GC4 is not watching anything."
    echo "Point TARGETS in scripts/check_gc4.sh at the file that replaced it."
    exit 1
  fi
done

# Look for method definitions named train/fit on encoder/decoder modules.
hits="$(rg -n -e '^\s*(pub\s+)?(fn|async\s+fn)\s+(train|fit)\b' "${TARGETS[@]}" </dev/null || true)"
if [[ -n "$hits" ]]; then
  echo "GC4 FAIL: Encoder/Decoder exposes train/fit before P4:"
  echo "$hits"
  exit 1
fi
echo "GC4 PASS: no Encoder/Decoder train/fit methods (${#TARGETS[@]} files read)"
