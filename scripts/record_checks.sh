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

# Was: one named test file of six. Discovery instead, so a wave that adds an
# analyser and a test for it is covered the day it lands rather than whenever
# someone remembers to extend this line.
run "every discovered python test" bash scripts/run_python_tests.sh
# The record is a citation graph, and "read this only through the document that
# retired it" is only a rule if the link goes there.
run "every internal record link resolves" python3 scripts/check_record_links.py
# A sweep rather than a curated list: it asks whether the cells can produce every
# number in a wave result, and so can catch one nobody thought to name. It prints
# its own coincidence rate; verify_published_numbers.py above is the strong check.
run "every wave-result number is derivable" python3 scripts/check_every_number.py
# A stale index reports a state that has moved, which is worse than none.
run "the record index is current" python3 scripts/build_results_index.py --check
run "published numbers reproduce from cells" python3 scripts/verify_published_numbers.py
run "checks that cannot fail" python3 scripts/find_weak_checks.py

if [[ "$fail" -ne 0 ]]; then
  echo "RECORD CHECKS FAILED — do not cite a number until this is green."
  exit 1
fi
echo "All record checks executed and passed."
