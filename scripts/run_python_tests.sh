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
# test, and OPTIONAL_DEPS is DERIVED from that file (name -> import name), so
# the two cannot drift. A test is classified as missing a dependency only by the
# traceback's final line, `ModuleNotFoundError: No module named '<dep>'`; the
# same words appearing anywhere else in the output do not count. Any other
# import error is a failure — `No module named 'scripts'` was one, and treating
# it as "optional" would have kept it invisible.
#
# # The optional-deps environment
#
# The system python does not carry those dependencies, but a sibling venv can.
# When a test fails ONLY on an OPTIONAL_DEPS import, it is re-run under each
# executable interpreter among VENV_PYTHONS in turn (`.venv-shd-cal`, the
# calibration harness's own 3.12 env, then `.venv-shd`, numpy + h5py), moving on
# to the next only when the one tried lacked a dependency:
#
#   exit 0 with proof that tests ran  -> ran and passed, printed `in <venv>`
#   exit 0 with no proof / all skipped -> FAIL (a pass that ran nothing)
#   fails on an optional dep           -> next candidate; none left = SKIP
#   any other failure/timeout          -> FAIL
#   no venv present                    -> could-not-run, never a pass
#
# Proof is `Ran N test(s)` with N > 0 and not every one skipped, or a
# hand-rolled runner's `N/N passed` line with N > 0.
#
# `NEEDS_OPTIONAL_DEPS_ENV` pins which tests are expected to need that
# environment, i.e. fail on an optional dep under the system python. It is
# checked both ways, against what the system python did and not against where
# the test finally ran, so a venv rescuing a pinned test is not "stale", while a
# pinned test that already passes under the system python is.
#
# # Exit status
#
#   0  every test ran and passed
#   1  a test failed, or a pin is stale/missing
#   3  nothing failed, but some pinned tests COULD NOT RUN (no venv with the
#      dependency). Not a pass, and not a failure: `record_checks.sh` reports it
#      separately and does not fail the gate on it.
set -uo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

#: The interpreter for the first pass. record_checks.sh resolves one that
#: satisfies the build scripts (>= 3.11) and exports it; run on its own, the
#: login `python3` is used.
PYTHON="${BINN_PYTHON:-python3}"

# Import names of the packages in requirements-shd-calibration.txt. Most match
# the distribution name; these do not.
import_name() {
  case "$1" in
    pillow) echo PIL ;;
    dcls) echo DCLS ;;
    *) echo "$1" ;;
  esac
}

# Derived from requirements-shd-calibration.txt, whose own header records that
# the calibration harness targets Python 3.12 while this workspace runs 3.14.
# Not installed here, and installing it is a provisioning decision rather than
# something a gate should do behind the operator's back.
OPTIONAL_DEPS=""
while IFS= read -r line; do
  line="${line%%#*}"
  line="${line%%[<>=!~;\[ ]*}"
  line="$(printf '%s' "$line" | tr -d '[:space:]' | tr '[:upper:]' '[:lower:]')"
  [ -n "$line" ] || continue
  OPTIONAL_DEPS="${OPTIONAL_DEPS:+$OPTIONAL_DEPS|}$(import_name "$line")"
done < "$ROOT/requirements-shd-calibration.txt"
if [ -z "$OPTIONAL_DEPS" ]; then
  echo "cannot derive OPTIONAL_DEPS from requirements-shd-calibration.txt" >&2
  exit 1
fi

#: Tests that need the optional-deps environment: they fail on an OPTIONAL_DEPS
#: import under the system python. A test that starts doing so and is NOT on
#: this list fails the gate (UNEXPECTED); one on this list that passes under the
#: system python fails it too (STALE). Newline-separated, so a path with a space
#: survives. BINN_NEEDS_OPTIONAL_DEPS_ENV overrides the list for
#: scripts/test_run_python_tests.py, which drives this script on a temporary tree.
NEEDS_OPTIONAL_DEPS_ENV="${BINN_NEEDS_OPTIONAL_DEPS_ENV-scripts/test_provenance_discharge.py
scripts/test_shd_calibration.py
scripts/test_gate_f_discharge.py
scripts/test_transfer_numpy.py}"

#: Interpreters tried, in order, for a test that needs the optional deps
#: (newline-separated). `.venv-shd-cal` is the calibration harness's Python 3.12
#: env (torch); `.venv-shd` carries numpy + h5py. BINN_VENV_PYTHONS overrides.
VENV_PYTHONS="${BINN_VENV_PYTHONS-$ROOT/.venv-shd-cal/bin/python
$ROOT/.venv-shd/bin/python}"

#: Seconds a single test may run before it counts as a failure, so a hung test
#: cannot hang the gate. The venv rerun and the system run have separate knobs.
VENV_TIMEOUT="${BINN_VENV_TIMEOUT:-900}"
TEST_TIMEOUT="${BINN_TEST_TIMEOUT:-3600}"

WORK="$(mktemp -d "${TMPDIR:-/tmp}/binn-rpt.XXXXXX")" || exit 1
trap 'rm -rf "$WORK"' EXIT

# bounded SECONDS OUTFILE CMD... — run CMD in its own process group with stdout
# and stderr going to OUTFILE (never a pipe, so a grandchild holding it open
# cannot block us). On expiry the whole group is SIGKILLed, which a test that
# ignores SIGALRM cannot refuse; the exit status is 124. macOS ships no
# `timeout`, perl is on every machine this runs on. A descendant that calls
# setsid itself escapes the group; that is not defended against.
bounded() {
  perl -e '
    use POSIX ":sys_wait_h";
    my ($limit, $out, @cmd) = @ARGV;
    my $pid = fork();
    defined $pid or die "fork: $!";
    if (!$pid) {
      setpgrp(0, 0);
      open(STDIN, "<", "/dev/null");
      open(STDOUT, ">", $out) or die "open $out: $!";
      open(STDERR, ">&", \*STDOUT) or die "dup: $!";
      exec { $cmd[0] } @cmd or do { print STDERR "exec $cmd[0]: $!\n"; exit 127 };
    }
    setpgrp($pid, $pid);
    my $end = time + $limit;
    my $st;
    while (1) {
      my $r = waitpid($pid, WNOHANG);
      if ($r == $pid) { $st = $?; last }
      if (time >= $end) {
        kill "KILL", -$pid;
        waitpid($pid, 0);
        if (open(my $f, ">>", $out)) { print $f "\ntimed out after ${limit}s; process group killed\n"; close $f }
        exit 124;
      }
      select(undef, undef, undef, 0.1);
    }
    kill "KILL", -$pid;
    exit(($st & 127) ? 128 + ($st & 127) : $st >> 8);
  ' "$@"
}

# The dependency named by the traceback's FINAL line, if it is an optional one.
# Anything else, however much it resembles one earlier in the output, is not.
missing_dep() {
  tail -n 40 "$1" | grep -v '^[[:space:]]*$' | tail -n 1 \
    | grep -oE "^ModuleNotFoundError: No module named '($OPTIONAL_DEPS)(\\..*)?'\$" \
    | sed -E "s/^[^']*'([^'.]*).*/\\1/" | head -n 1
}

# Proof that tests ran, from the last lines of $1. Prints "Ran N tests" /
# "N/N passed" and returns 0; returns 1 when nothing ran or all were skipped.
evidence() {
  local tail_text n skipped
  tail_text="$(tail -n 12 "$1")"
  n="$(printf '%s\n' "$tail_text" | sed -nE 's/^Ran ([0-9]+) tests? in .*/\1/p' | tail -n 1)"
  if [ -n "$n" ]; then
    [ "$n" -gt 0 ] || return 1
    skipped="$(printf '%s\n' "$tail_text" | sed -nE 's/^OK \(.*skipped=([0-9]+).*\)$/\1/p' | tail -n 1)"
    [ -z "$skipped" ] || [ "$skipped" -lt "$n" ] || return 1
    echo "Ran $n test$([ "$n" -eq 1 ] || echo s)"
    return 0
  fi
  # A hand-rolled runner's "N/N passed" (BSD sed has no ERE backreference).
  n="$(printf '%s\n' "$tail_text" | awk -F'[/ ]' '$1 ~ /^[0-9]+$/ && $1 == $2 && $3 == "passed" && NF == 3 { n = $1 } END { if (n != "") print n }')"
  if [ -n "$n" ] && [ "$n" -gt 0 ]; then
    echo "$n/$n passed"
    return 0
  fi
  return 1
}

# "name" is in the newline-separated list "$2".
listed() {
  local entry
  while IFS= read -r entry; do
    [ "$entry" = "$1" ] && return 0
  done <<EOF
$2
EOF
  return 1
}

passed=0
failed=()
unrunnable=()
needs_env=()   # tests that failed on an optional dep under the system python
sys_passed=()  # tests that passed under the system python

# in_array NAME ITEM... — NAME equals one of the ITEMs.
in_array() {
  local n="$1" e
  shift
  for e in "$@"; do
    [ "$e" = "$n" ] && return 0
  done
  return 1
}

# Each file is listed once however many guards it trips.
add_failed() {
  in_array "$1" ${failed[@]+"${failed[@]}"} || failed+=("$1")
}

show_tail() { tail -n 25 "$1" | sed 's/^/        /'; }

venv_pythons=()
while IFS= read -r candidate; do
  [ -n "$candidate" ] && [ -x "$candidate" ] && venv_pythons+=("$candidate")
done <<EOF
$VENV_PYTHONS
EOF

out="$WORK/out"
for file in scripts/test_*.py; do
  bounded "$TEST_TIMEOUT" "$out" "$PYTHON" "$file"
  code=$?
  if [ "$code" -eq 0 ]; then
    passed=$((passed + 1))
    sys_passed+=("$file")
    printf '  [ ok ] %s  (%s)\n' "$file" "$(grep -oE 'Ran [0-9]+ tests?' "$out" | tail -n 1)"
    continue
  fi
  missing="$(missing_dep "$out")"
  if [ -z "$missing" ]; then
    add_failed "$file"
    printf '  [FAIL] %s  (exit %s)\n' "$file" "$code"
    show_tail "$out"
    continue
  fi

  needs_env+=("$file")
  rescued=0
  for venv_python in ${venv_pythons[@]+"${venv_pythons[@]}"}; do
    venv_name="$(basename "$(dirname "$(dirname "$venv_python")")")"
    bounded "$VENV_TIMEOUT" "$out" "$venv_python" "$file"
    code=$?
    if [ "$code" -eq 0 ]; then
      if proof="$(evidence "$out")"; then
        passed=$((passed + 1))
        printf '  [ ok ] %s  (%s; in %s)\n' "$file" "$proof" "$venv_name"
      else
        add_failed "$file"
        printf '  [FAIL] %s  (in %s: exit 0 but no proof any test ran, or all skipped)\n' "$file" "$venv_name"
        show_tail "$out"
      fi
      rescued=1
      break
    fi
    venv_missing="$(missing_dep "$out")"
    if [ -n "$venv_missing" ]; then
      missing="$venv_missing"   # this venv lacks it; try the next
      continue
    fi
    add_failed "$file"
    printf '  [FAIL] %s  (in %s, exit %s)\n' "$file" "$venv_name" "$code"
    show_tail "$out"
    rescued=1
    break
  done
  if [ "$rescued" -eq 0 ]; then
    unrunnable+=("$file — No module named '$missing'")
    printf "  [SKIP] %s  No module named '%s'\\n" "$file" "$missing"
  fi
done

echo
echo "passed: $passed   failed: ${#failed[@]}   could not run: ${#unrunnable[@]}"

# Both guards compare against what the SYSTEM python did (needs_env), not where
# the test finally ran: the venv rescuing a test is not what makes a pin stale.
for name in ${needs_env[@]+"${needs_env[@]}"}; do
  if ! listed "$name" "$NEEDS_OPTIONAL_DEPS_ENV"; then
    echo "UNEXPECTED: $name needs an optional dependency and is not one of the"
    echo "tests pinned in NEEDS_OPTIONAL_DEPS_ENV. A check that could not run"
    echo "here must not be quietly tolerated."
    add_failed "$name"
  fi
done

# A pinned entry that passes under the system python no longer needs the venv;
# leave it on the list and it stops guarding anything.
while IFS= read -r name; do
  [ -n "$name" ] || continue
  in_array "$name" ${needs_env[@]+"${needs_env[@]}"} && continue
  if [ ! -f "$name" ]; then
    echo "STALE: $name is pinned as needing the optional-deps env but does not exist. Remove it."
  elif in_array "$name" ${sys_passed[@]+"${sys_passed[@]}"}; then
    echo "STALE: $name is pinned as needing the optional-deps env but passes under the system python. Remove it."
  else
    echo "STALE: $name is pinned as needing the optional-deps env but failed under the system python for another reason (see above)."
  fi
  add_failed "$name"
done <<EOF
$NEEDS_OPTIONAL_DEPS_ENV
EOF

if [ "${#failed[@]}" -ne 0 ]; then
  echo
  echo "FAILED: ${failed[*]}"
  exit 1
fi

if [ "${#unrunnable[@]}" -ne 0 ]; then
  echo
  echo "COULD NOT RUN (not passed): ${#unrunnable[@]} test(s) need an optional-deps"
  echo "environment (.venv-shd-cal or .venv-shd, see VENV_PYTHONS) and none had the dependency:"
  for entry in "${unrunnable[@]}"; do echo "  - $entry"; done
  exit 3
fi
echo "All discovered python tests executed and passed."
