#!/usr/bin/env bash
# GC2: no external ML framework deps.
#
# The check matches **crate names**, not raw `cargo tree` output.
#
# It used to match the raw output, which prints each workspace crate's absolute
# path in parentheses. That made the gate depend on where the repository is
# checked out rather than on what it depends on: a worktree under any directory
# containing one of the banned substrings fails. `scratchpad` contains `tch`,
# so a checkout under `.../scratchpad/...` reported
# "banned ML framework dependency" for every local crate, naming paths rather
# than dependencies. A gate that fails for a reason unrelated to what it
# guards teaches people to ignore it.
#
# The matcher is calibrated below against a fixture of names it must catch and
# names it must not, and **refuses to report a pass if the calibration fails** —
# the same rule `scripts/find_weak_checks.py` applies to itself.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

# Hyphen-delimited component match, so `tch`, `tch-sys`, `torch-sys`,
# `candle-core` and `burn-tensor` are all caught, while `patch`, `match` and
# `scratchpad` are not.
banned='(^|-)(torch|tch|candle|burn|dfdx)(-|$)'

# Crate name from a `cargo tree` line: strip the tree-drawing prefix, then the
# version and everything after it.
crate_names() {
  sed -E 's/^[^A-Za-z0-9_-]*//; s/ v[0-9].*$//' | sed '/^$/d' | sort -u
}

# --- calibration: the matcher must still detect what it exists to detect -----
must_catch=$'tch v0.13.0\ntorch-sys v0.13.0\ncandle-core v0.4.1\nburn v0.12.0\ndfdx v0.13.0'
must_ignore=$'binn-learn v0.1.0 (/tmp/scratchpad/head-check/binn-learn)\npatch v1.0.0\nmatcher v2.0.0\nbatch-run v0.1.0'

caught="$(printf '%s\n' "$must_catch" | crate_names | rg -e "$banned" -c || true)"
if [[ "$caught" != "5" ]]; then
  echo "GC2 SELF-CHECK FAILED: matcher caught ${caught:-0} of 5 known banned crates."
  echo "The gate cannot report a pass it is not able to fail. Fix the pattern."
  exit 1
fi
ignored="$(printf '%s\n' "$must_ignore" | crate_names | rg -e "$banned" -c || true)"
if [[ -n "$ignored" && "$ignored" != "0" ]]; then
  echo "GC2 SELF-CHECK FAILED: matcher flagged $ignored name(s) it must not:"
  printf '%s\n' "$must_ignore" | crate_names | rg -e "$banned"
  exit 1
fi

# --- the check itself --------------------------------------------------------
tree="$(cargo tree --workspace -e normal,build 2>/dev/null || cargo tree --workspace)"
hits="$(printf '%s\n' "$tree" | crate_names | rg -n -e "$banned" || true)"
if [[ -n "$hits" ]]; then
  echo "GC2 FAIL: banned ML framework dependency in cargo tree:"
  echo "$hits"
  exit 1
fi
echo "GC2 PASS: no banned ML framework dependencies (matcher calibrated: 5/5 caught, 0 false)"
