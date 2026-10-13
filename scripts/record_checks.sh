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

# The build scripts use `tomllib`, which is 3.11+. A bare `python3` is whatever
# the shell's PATH puts first -- /usr/bin/python3 is 3.9 on a stock macOS and
# some agent shells get it -- so it is resolved ONCE, explicitly, and used for
# every check below and by run_python_tests.sh (via BINN_PYTHON). Order: an
# operator override, the PATH default, then versioned names and the standard
# install prefixes. First one at >= 3.11 wins; none is a loud early failure
# rather than a tomllib ImportError mid-run.
python_ok() {
  command -v "$1" >/dev/null 2>&1 \
    && "$1" -c 'import sys; sys.exit(0 if sys.version_info >= (3, 11) else 1)' 2>/dev/null
}
PY=""
for candidate in "${BINN_PYTHON:-}" python3 python3.14 python3.13 python3.12 python3.11 \
                 /usr/local/bin/python3 /opt/homebrew/bin/python3; do
  [ -n "$candidate" ] || continue
  if python_ok "$candidate"; then PY="$(command -v "$candidate")"; break; fi
done
if [ -z "$PY" ]; then
  echo "record_checks: no Python >= 3.11 found (build scripts need tomllib)." >&2
  echo "  python3 on PATH is: $(command -v python3 || echo none) ($(python3 --version 2>&1 || true))" >&2
  echo "  Put a 3.11+ python3 first on PATH, or set BINN_PYTHON=/path/to/python." >&2
  exit 1
fi
export BINN_PYTHON="$PY"
echo "record_checks: using $PY ($("$PY" --version 2>&1))"
echo

fail=0
partial=0
run() {
  echo "=== $1 ==="
  shift
  local rc=0
  "$@" || rc=$?
  if [ "$rc" -ne 0 ]; then fail=1; fi
  echo
}

# Was: one named test file of six. Discovery instead, so a wave that adds an
# analyser and a test for it is covered the day it lands rather than whenever
# someone remembers to extend this line.
# Exit 3 from it means "nothing failed, but pinned tests could not run": not a
# pass, not a failure. It is kept apart from both so the closing line cannot say
# "passed" for tests that never ran.
echo "=== every discovered python test ==="
rc=0
bash scripts/run_python_tests.sh || rc=$?
case "$rc" in
  0) ;;
  3) partial=1 ;;
  *) fail=1 ;;
esac
echo
# The record is a citation graph, and "read this only through the document that
# retired it" is only a rule if the link goes there.
run "every internal record link resolves" "$PY" scripts/check_record_links.py
# A sweep rather than a curated list: it asks whether the cells can produce every
# number in a wave result, and so can catch one nobody thought to name. It prints
# its own coincidence rate; verify_published_numbers.py above is the strong check.
run "every wave-result number is derivable" "$PY" scripts/check_every_number.py
# The sweep above is four-decimal and reads cells, so it cannot see §0: its
# numbers are two-decimal percentages quoted out of published papers. They were
# read against their primaries on 2026-09-03 and three were wrong. This keeps
# §0 a closed set -- a new percentage there has to arrive with a source read.
run "every §0 citation is covered by a primary" "$PY" scripts/check_section0_citations.py
# The submission itself. Checks the page limit, anonymity in body AND metadata,
# and that the style file is the unmodified one -- three failures that all look
# exactly like a finished paper in the output.
run "the submission builds anonymous and within the page limit" "$PY" scripts/build_paper.py --check
# The lead matched FAIL is broadcast ±1 three-factor, NOT broadcast credit
# topology in general -- broadcast-graded reaches 0.9975 on the same schedule.
# Four documents recorded that requirement and none checked it.
run "the lead matched FAIL is never widened to 'broadcast'" "$PY" scripts/check_terminology.py
# The analyser is frozen so that IT is the authority. That only helps if what
# gets published is what it said, and every verdict in a write-up is retyped.
run "published verdicts match their analyser" "$PY" scripts/check_verdicts_transcribed.py
# A stale index reports a state that has moved, which is worse than none.
run "the record index is current" "$PY" scripts/build_results_index.py --check
run "published numbers reproduce from cells" "$PY" scripts/verify_published_numbers.py
# The reproducibility claim was a hand-picked overlap of three configurations.
# Derived instead, it is eight — and the four that were missed include the
# headline width. A claim this load-bearing should be recomputed, not recalled.
run "the x86 and aarch64 corpora agree" "$PY" scripts/cross_isa_reproduction.py
# The cross-ISA check above asks whether two machines agree. This asks whether
# one fleet still agrees with its own record: every configuration two waves both
# ran, regressed wave against wave. Wave 18 registers that question as H18-4 but
# schedules it at plan index 140 of 192, so a failure would surface after most of
# the compute it voids had been spent. This answers the weaker form of it from
# whatever has landed, on every run, and exits non-zero when nothing matched.
run "the fleet reproduces its own record" "$PY" scripts/aws/check_reproduction.py
# The paper's lead claim is a difference of differences, and computing it needs
# four arms on shared seeds. Derived rather than recalled: on 2026-08-27 two of
# twenty operating points carried all four, both at h128. A scope limit that
# large should be recomputed on every run, not remembered.
# The campaign watcher is `scripts/aws/watch_campaign.py` and there is one of
# them. It replaced `scripts/aws/watch.sh` on 2026-08-28; both watched the same
# bucket, neither referenced the other, and the shell one counted every result
# object the bucket had ever held against the current plan's total, so it
# declared COMPLETE on its first poll. Run it as:
#
#   python3 scripts/aws/watch_campaign.py --bucket $BUCKET
#
# Coverage as MEASURED. The companion question -- will the compute already
# queued buy the operating points the wave was registered to buy -- needs the
# published plan, which lives in S3, and a gate that needs the network is a gate
# that gets skipped. Run it by hand when queueing a mechanism wave:
#
#   aws s3 cp s3://$BUCKET/input/cells.json /tmp/cells.json
#   python3 scripts/mechanism_coverage.py --plan /tmp/cells.json
#
# On 2026-08-28 that reported wave 21 taking coverage from 2 operating points to
# 9, across every width from 128 to 1024 and both contracts and geometries.
run "the mechanism control's coverage" "$PY" scripts/mechanism_coverage.py
run "checks that cannot fail" "$PY" scripts/find_weak_checks.py --strict

if [[ "$fail" -ne 0 ]]; then
  echo "RECORD CHECKS FAILED — do not cite a number until this is green."
  exit 1
fi
if [[ "$partial" -ne 0 ]]; then
  echo "Record checks that ran all passed, BUT some python tests COULD NOT RUN (listed"
  echo "above under COULD NOT RUN) and are not counted as passed. Provision .venv-shd-cal"
  echo "or .venv-shd with requirements-shd-calibration.txt to run them."
  exit 0
fi
echo "All record checks executed and passed."
