#!/usr/bin/env bash
# Every `scripts/test_*.py`, by discovery, with three outcomes rather than two.
#
# # Why this exists
#
# `record_checks.sh` ran **one** of six test files. The other five were not
# skipped for a reason — nothing had ever listed them. Two of them had been
# broken long enough that nobody noticed:
#
#   * `test_shd_calibration.py` raised `No module named 'scripts'` on every
#     invocation, because `from scripts...` needs the repository ROOT on the
#     path and running the file directly puts `scripts/` there instead. It was
#     unreachable, and unreachable because no gate ran it.
#   * `test_provenance_discharge.py` needs `h5py`.
#
# A test nobody runs is not a check, and each new wave adds another analyser and
# another test file. Discovery rather than a list is the point: a file added
# tomorrow is run tomorrow.
#
# # Three outcomes, because two would lie
#
# A test that **could not run** must never report the same result as one that
# ran and passed. This script separates them, and refuses to let the unrunnable
# set grow silently: it is pinned by name below, so a *new* test that cannot run
# fails the gate rather than joining a crowd.
#
# Only a missing dependency from `requirements-shd-calibration.txt` excuses a
# test. Any other import error is a failure — `No module named 'scripts'` was
# one, and treating it as "optional" would have kept it invisible.
#
# # The optional-deps environment
#
# The system `python3` does not carry those dependencies, but a sibling venv
# can. When a test fails ONLY on an OPTIONAL_DEPS import, it is re-run once under
# the first executable interpreter among VENV_PYTHONS (`.venv-shd-cal`, the
# calibration harness's own 3.12 env, then `.venv-shd`, which holds h5py). The
# re-run decides the outcome:
#
#   exit 0                     -> ran and passed, printed as `in <venv>`
#   fails on an optional dep   -> still could-not-run (SKIP)
#   any other failure/timeout  -> FAIL
#   no venv present            -> could-not-run, never a pass
#
# `NEEDS_OPTIONAL_DEPS_ENV` pins which tests are expected to need that
# environment, i.e. fail on an optional dep under the system python3. It is
# checked both ways, against what the system python3 did and not against where
# the test finally ran, so a venv rescuing a pinned test is not "stale", while a
# pinned test that already passes under the system python3 is.
set -uo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

# Declared in requirements-shd-calibration.txt, whose own header records that
# the calibration harness targets Python 3.12 while this workspace runs 3.14.
# Not installed here, and installing it is a provisioning decision rather than
# something a gate should do behind the operator's back.
OPTIONAL_DEPS='h5py|torch|torchvision|torchaudio|wandb|dcls'

#: Tests that need the optional-deps environment: they fail on an OPTIONAL_DEPS
#: import under the system python3. A test that starts doing so and is NOT on
#: this list fails the gate (UNEXPECTED); one on this list that passes under the
#: system python3 fails it too (STALE). BINN_NEEDS_OPTIONAL_DEPS_ENV overrides
#: the list for scripts/test_run_python_tests.py, which drives this script on
#: a temporary tree.
NEEDS_OPTIONAL_DEPS_ENV="${BINN_NEEDS_OPTIONAL_DEPS_ENV-scripts/test_provenance_discharge.py scripts/test_shd_calibration.py}"

#: Interpreters tried, in order, for a test that needs the optional deps.
#: `.venv-shd-cal` is the calibration harness's Python 3.12 env (torch);
#: `.venv-shd` carries h5py. BINN_VENV_PYTHONS overrides (space-separated).
VENV_PYTHONS="${BINN_VENV_PYTHONS-$ROOT/.venv-shd-cal/bin/python $ROOT/.venv-shd/bin/python}"

#: Seconds a single test may run under the venv before it counts as a failure,
#: so a hung venv test cannot hang the gate.
VENV_TIMEOUT="${BINN_VENV_TIMEOUT:-300}"

# Run "$@" under a SIGALRM deadline. perl's alarm survives exec and macOS ships
# no `timeout`; a killed child exits 142, which is reported as a failure.
bounded() {
  perl -e 'alarm shift; exec @ARGV or die "exec: $!"' "$VENV_TIMEOUT" "$@"
}

passed=0
failed=()
unrunnable=()
needs_env=()   # tests that failed on an optional dep under the system python3

venv_python=""
for candidate in $VENV_PYTHONS; do
  if [ -x "$candidate" ]; then venv_python="$candidate"; break; fi
done

for file in scripts/test_*.py; do
  output="$(python3 "$file" 2>&1)"
  code=$?
  if [ "$code" -eq 0 ]; then
    passed=$((passed + 1))
    printf '  [ ok ] %s  (%s)\n' "$file" \
      "$(printf '%s' "$output" | grep -oE 'Ran [0-9]+ tests' | tail -1)"
    continue
  fi
  missing="$(printf '%s' "$output" | grep -oE "No module named '($OPTIONAL_DEPS)'" | head -1)"
  if [ -n "$missing" ]; then
    needs_env+=("$file")
    if [ -n "$venv_python" ]; then
      output="$(bounded "$venv_python" "$file" 2>&1)"
      code=$?
      venv_name="$(basename "$(dirname "$(dirname "$venv_python")")")"
      if [ "$code" -eq 0 ]; then
        passed=$((passed + 1))
        printf '  [ ok ] %s  (%s; in %s)\n' "$file" \
          "$(printf '%s' "$output" | grep -oE 'Ran [0-9]+ tests' | tail -1)" "$venv_name"
        continue
      fi
      missing="$(printf '%s' "$output" | grep -oE "No module named '($OPTIONAL_DEPS)'" | head -1)"
      if [ -z "$missing" ]; then
        failed+=("$file")
        printf '  [FAIL] %s  (in %s, exit %s)\n' "$file" "$venv_name" "$code"
        printf '%s\n' "$output" | tail -25 | sed 's/^/        /'
        continue
      fi
    fi
    unrunnable+=("$file — $missing")
    printf '  [SKIP] %s  %s\n' "$file" "$missing"
    continue
  fi
  failed+=("$file")
  printf '  [FAIL] %s\n' "$file"
  printf '%s\n' "$output" | tail -25 | sed 's/^/        /'
done

echo
echo "passed: $passed   failed: ${#failed[@]}   could not run: ${#unrunnable[@]}"

# Both guards compare against what the SYSTEM python3 did (needs_env), not where
# the test finally ran: the venv rescuing a test is not what makes a pin stale.
for name in ${needs_env[@]+"${needs_env[@]}"}; do
  case " $NEEDS_OPTIONAL_DEPS_ENV " in
    *" $name "*) ;;
    *)
      echo "UNEXPECTED: $name needs an optional dependency and is not one of the"
      echo "tests pinned in NEEDS_OPTIONAL_DEPS_ENV. A check that could not run"
      echo "here must not be quietly tolerated."
      failed+=("$name")
      ;;
  esac
done

# A pinned entry that passes under the system python3 no longer needs the venv;
# leave it on the list and it stops guarding anything.
for name in $NEEDS_OPTIONAL_DEPS_ENV; do
  listed=0
  for entry in ${needs_env[@]+"${needs_env[@]}"}; do
    [ "$entry" = "$name" ] && listed=1
  done
  if [ "$listed" -eq 0 ]; then
    if [ -f "$name" ]; then
      echo "STALE: $name is pinned as needing the optional-deps env but did not fail on one. Remove it."
    else
      echo "STALE: $name is pinned as needing the optional-deps env but does not exist. Remove it."
    fi
    failed+=("$name")
  fi
done

if [ "${#failed[@]}" -ne 0 ]; then
  echo
  echo "FAILED: ${failed[*]}"
  exit 1
fi

if [ "${#unrunnable[@]}" -ne 0 ]; then
  echo
  echo "NOTE: ${#unrunnable[@]} test(s) could not run and are reported as such rather"
  echo "than as passes. They need an optional-deps environment (.venv-shd-cal or"
  echo ".venv-shd, see VENV_PYTHONS) and none had the dependency."
fi
echo "All discovered python tests executed."
