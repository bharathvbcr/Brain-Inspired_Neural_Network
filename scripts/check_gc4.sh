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
#
# `|| true` alone was not enough. ripgrep exits 1 for "searched, found nothing"
# and 2 for "could not search"; a missing binary exits 127. Swallowing all of
# them made "rg: command not found" print PASS — and, worse, print the file
# count of a search that never happened. Only exit 1 is a clean no-match.
set +e
hits="$(rg -n -e '^\s*(pub\s+)?(fn|async\s+fn)\s+(train|fit)\b' "${TARGETS[@]}" </dev/null)"
rc=$?
set -e
if [[ $rc -gt 1 ]]; then
  echo "GC4 CANNOT RUN: the search failed (rg exit $rc). GC4 read nothing."
  echo "Install ripgrep, or GC4 is not watching these files at all."
  exit 1
fi
if [[ -n "$hits" ]]; then
  echo "GC4 FAIL: Encoder/Decoder exposes train/fit before P4:"
  echo "$hits"
  exit 1
fi
echo "GC4 PASS: no Encoder/Decoder train/fit methods (${#TARGETS[@]} files read)"
