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
set -uo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

# Declared in requirements-shd-calibration.txt, whose own header records that
# the calibration harness targets Python 3.12 while this workspace runs 3.14.
# Not installed here, and installing it is a provisioning decision rather than
# something a gate should do behind the operator's back.
OPTIONAL_DEPS='h5py|torch|torchvision|torchaudio|wandb|dcls'

#: Tests known to be unrunnable here, with the reason. A test that becomes
#: unrunnable and is NOT on this list fails the gate.
EXPECTED_UNRUNNABLE="scripts/test_provenance_discharge.py scripts/test_shd_calibration.py"

passed=0
failed=()
unrunnable=()

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

for entry in "${unrunnable[@]}"; do
  name="${entry%% —*}"
  case " $EXPECTED_UNRUNNABLE " in
    *" $name "*) ;;
    *)
      echo "UNEXPECTED: $name could not run, and is not one of the known-unrunnable"
      echo "tests. A check that could not run must not be quietly tolerated."
      failed+=("$name")
      ;;
  esac
done

# A pinned entry that now runs is good news and still has to be recorded: leave
# it on the list and it stops guarding anything.
for name in $EXPECTED_UNRUNNABLE; do
  listed=0
  for entry in "${unrunnable[@]}"; do
    [ "${entry%% —*}" = "$name" ] && listed=1
  done
  if [ "$listed" -eq 0 ]; then
    echo "STALE: $name is on the known-unrunnable list but ran. Remove it."
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
  echo "NOTE: ${#unrunnable[@]} test(s) could not run in this interpreter and are"
  echo "reported as such rather than as passes. They need the calibration"
  echo "harness's own Python 3.12 environment."
fi
echo "All discovered python tests executed."
