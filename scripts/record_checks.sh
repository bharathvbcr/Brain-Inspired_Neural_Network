#!/usr/bin/env bash
# Record-integrity checks — the counterpart to `gc_checks.sh`.
#
# `gc_checks.sh` proves things about the SOURCE. This proves things about the
# EVIDENCE: that the tooling which produced the cells still holds its invariants,
# that no archived cell has drifted, and that every number printed in a result
# document still follows from the cells it claims to come from.
#
# The last of those is the one with no other owner. The analyser produces
# verdicts and a human transcribes them into markdown; neither step checks the
# other, so a transcription slip would survive re-running the analyser forever.
#
# Run: bash scripts/record_checks.sh
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

fail=0
run() {
  echo "=== $1 ==="
  shift
  if ! "$@"; then fail=1; fi
  echo
}

run "campaign tooling invariants" python3 scripts/test_campaign_tooling.py
run "published numbers reproduce from cells" python3 scripts/verify_published_numbers.py
run "checks that cannot fail" python3 scripts/find_weak_checks.py

if [[ "$fail" -ne 0 ]]; then
  echo "RECORD CHECKS FAILED — do not cite a number until this is green."
  exit 1
fi
echo "All record checks executed and passed."
