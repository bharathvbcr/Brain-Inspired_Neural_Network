#!/usr/bin/env bash
#
# Run protocols v143-v146 in preregistered order.
#
# Usage:
#   ./scripts/run_code_transfer_campaign.sh
#   ./scripts/run_code_transfer_campaign.sh --force
#   ./scripts/run_code_transfer_campaign.sh --skip-gates
#
# The runner is resumable when the source fingerprint is unchanged. It stops
# v145/v146 when v144 does not create a valid frozen-difficulty artifact.

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$REPO_ROOT" || exit 1

FORCE=0
SKIP_GATES=0
for arg in "$@"; do
  case "$arg" in
    --force) FORCE=1 ;;
    --skip-gates) SKIP_GATES=1 ;;
    -h|--help) sed -n '2,12p' "$0"; exit 0 ;;
    *) echo "unknown flag: $arg" >&2; exit 2 ;;
  esac
done

TIMEOUT_HOURS="${BINN_TRANSFER_TIMEOUT_HOURS:-48}"
if ! [[ "$TIMEOUT_HOURS" =~ ^[1-9][0-9]*$ ]]; then
  echo "BINN_TRANSFER_TIMEOUT_HOURS must be a positive integer" >&2
  exit 2
fi
TIMEOUT_SECONDS=$((TIMEOUT_HOURS * 60 * 60))

source_fingerprint() {
  find Cargo.toml Cargo.lock binn-core binn-engine binn-areas binn-learn binn-data binn-lab scripts \
    -type f \( -name '*.rs' -o -name 'Cargo.toml' -o -name 'Cargo.lock' -o -name '*.sh' -o -name '*.py' \) \
    -print 2>/dev/null |
    LC_ALL=C sort |
    while IFS= read -r source_file; do shasum -a 256 "$source_file"; done |
    shasum -a 256 |
    awk '{print $1}'
}

SOURCE_FINGERPRINT="$(source_fingerprint)"
STAMP="$(date +%Y-%m-%d_%H%M%S)"
RUN_DIR="results/runs/${STAMP}_code_transfer"
NEW_RUN_DIR="$RUN_DIR"

if [[ $FORCE -eq 0 ]]; then
  LATEST="$(ls -1d results/runs/*_code_transfer 2>/dev/null | tail -1 || true)"
  if [[ -n "${LATEST:-}" && -d "$LATEST/.state" && ! -f "$LATEST/.complete" ]]; then
    PRIOR_FINGERPRINT="$(sed -n '1p' "$LATEST/.state/source_fingerprint" 2>/dev/null || true)"
    if [[ "$PRIOR_FINGERPRINT" == "$SOURCE_FINGERPRINT" ]]; then
      RUN_DIR="$LATEST"
      echo "==> resuming compatible run: $RUN_DIR"
    else
      RUN_DIR="$NEW_RUN_DIR"
      echo "==> prior incomplete run has a different source fingerprint; starting fresh"
    fi
  fi
fi

LOG_DIR="$RUN_DIR/logs"
STATE_DIR="$RUN_DIR/.state"
MASTER_LOG="$RUN_DIR/campaign.log"
mkdir -p "$LOG_DIR" "$STATE_DIR"
printf '%s\n' "$SOURCE_FINGERPRINT" > "$STATE_DIR/source_fingerprint"

log() {
  printf '[%s] %s\n' "$(date +%H:%M:%S)" "$*" | tee -a "$MASTER_LOG"
}

LOCK_DIR="results/runs/.code-transfer.lock"
if ! mkdir "$LOCK_DIR" 2>/dev/null; then
  LOCK_PID="$(sed -n '1p' "$LOCK_DIR/pid" 2>/dev/null || echo unknown)"
  if [[ "$LOCK_PID" =~ ^[0-9]+$ ]] && ! kill -0 "$LOCK_PID" 2>/dev/null; then
    rm -f "$LOCK_DIR/pid"
    rmdir "$LOCK_DIR" 2>/dev/null || true
    mkdir "$LOCK_DIR" || exit 4
  else
    echo "another code-transfer runner is active (pid $LOCK_PID)" >&2
    exit 4
  fi
fi
printf '%s\n' "$$" > "$LOCK_DIR/pid"

CAFFEINATE_PID=""
if command -v caffeinate >/dev/null 2>&1; then
  caffeinate -dimsu -w $$ >/dev/null 2>&1 &
  CAFFEINATE_PID=$!
fi

cleanup() {
  if [[ -n "$CAFFEINATE_PID" ]]; then
    kill "$CAFFEINATE_PID" 2>/dev/null || true
    wait "$CAFFEINATE_PID" 2>/dev/null || true
  fi
  rm -f "$LOCK_DIR/pid" 2>/dev/null || true
  rmdir "$LOCK_DIR" 2>/dev/null || true
}
trap cleanup EXIT

write_summary() {
  local status="$1"
  local summary="$RUN_DIR/SUMMARY.md"
  {
    printf '# Code-first transfer campaign\n\n'
    printf '**Runner status:** %s  \n' "$status"
    printf '**Source fingerprint:** `%s`  \n' "$SOURCE_FINGERPRINT"
    printf '**Run directory:** `%s`\n\n' "$RUN_DIR"
    printf '| Job | Execution |\n|---|---|\n'
    local state_file job execution
    for state_file in "$STATE_DIR"/*.status; do
      [[ -e "$state_file" ]] || continue
      job="$(basename "$state_file" .status)"
      execution="$(sed -n '1p' "$state_file")"
      printf '| %s | %s |\n' "$job" "$execution"
    done
    printf '\n## Scientific reports\n\n'
    local report verdict
    for report in \
      results/shd_0c1_v143/capped-alif-ff-fixed.md \
      results/shd_0c1_v143/full-superspike.md \
      results/temporal_calibration_v144.md \
      results/temporal_depth_v145.md \
      results/transfer_falsifier_v146.md; do
      if [[ -f "$report" ]]; then
        verdict="$(sed -n 's/^\*\*Verdict:\*\* //p' "$report" | head -1)"
        printf -- '- `%s`: %s\n' "$report" "${verdict:-verdict not found}"
      else
        printf -- '- `%s`: pending\n' "$report"
      fi
    done
    printf '\nExecution success is not a scientific PASS. Read each report verdict and validity gates.\n'
  } > "$summary"
  log "summary: $summary"
}

assert_source_unchanged() {
  local current
  current="$(source_fingerprint)"
  if [[ "$current" != "$SOURCE_FINGERPRINT" ]]; then
    log "FATAL: source changed during the campaign; refusing to mix protocol states"
    write_summary "ABORTED — source changed"
    exit 5
  fi
}

run_job() {
  local name="$1"
  shift
  local marker="$STATE_DIR/$name.done"
  local status_file="$STATE_DIR/$name.status"
  local logfile="$LOG_DIR/$name.log"
  if [[ $FORCE -eq 0 && -f "$marker" ]]; then
    log "CACHE $name ($(sed -n '1p' "$status_file" 2>/dev/null || echo ok))"
    return 0
  fi
  assert_source_unchanged
  log "START $name (timeout ${TIMEOUT_HOURS}h)"
  local started ended rc
  started="$(date +%s)"
  python3 "$SCRIPT_DIR/run_with_timeout.py" "$TIMEOUT_SECONDS" "$@" >"$logfile" 2>&1
  rc=$?
  ended="$(date +%s)"
  if [[ $rc -eq 0 ]]; then
    printf 'ok (%ss)\n' "$((ended - started))" > "$status_file"
    date > "$marker"
    log "OK $name ($((ended - started))s)"
    return 0
  fi
  if [[ $rc -eq 124 ]]; then
    printf 'timeout (%ss)\n' "$((ended - started))" > "$status_file"
  else
    printf 'fail:%s (%ss)\n' "$rc" "$((ended - started))" > "$status_file"
  fi
  log "FAIL $name (exit $rc); see $logfile"
  tail -20 "$logfile" 2>/dev/null | sed 's/^/  /' | tee -a "$MASTER_LOG"
  return "$rc"
}

run_gate() {
  if ! run_job "$@"; then
    write_summary "ABORTED — mechanical gate failed"
    exit 1
  fi
}

if ! command -v cargo >/dev/null 2>&1 || ! command -v python3 >/dev/null 2>&1; then
  log "FATAL: cargo and python3 are required"
  exit 1
fi

SHD_DIR="${BINN_SHD_DIR:-data/shd}"
if [[ ! -f "$SHD_DIR/train.bin" || ! -f "$SHD_DIR/test.bin" ]]; then
  log "FATAL: official SHD cache not found at $SHD_DIR"
  log "Set BINN_SHD_DIR or create data/shd/train.bin and data/shd/test.bin"
  exit 1
fi

log "run directory: $RUN_DIR"
log "source fingerprint: $SOURCE_FINGERPRINT"
log "official SHD cache: $SHD_DIR"

if [[ $SKIP_GATES -eq 0 ]]; then
  run_gate gate_tests cargo test --locked --workspace
  run_gate gate_clippy cargo clippy --locked --workspace --all-targets -- -D warnings
fi
run_gate gate_release_build cargo build --locked --release --workspace

run_gate shd_v143 \
  cargo run --locked --release --quiet -p binn-lab --bin shd-input-control -- \
    --comparison both --out-dir results/shd_0c1_v143

for report in \
  results/shd_0c1_v143/capped-alif-ff-fixed.md \
  results/shd_0c1_v143/full-superspike.md; do
  if [[ ! -f "$report" ]]; then
    log "FATAL: v143 exited successfully without writing $report"
    write_summary "ABORTED — missing v143 report"
    exit 1
  fi
done

if rg -q 'INCONCLUSIVE' \
  results/shd_0c1_v143/capped-alif-ff-fixed.md \
  results/shd_0c1_v143/full-superspike.md; then
  log "v143 is inconclusive; extending the unchanged schedule to 20 seeds"
  run_gate shd_v143_extend \
    cargo run --locked --release --quiet -p binn-lab --bin shd-input-control -- \
      --extend --comparison both --out-dir results/shd_0c1_v143
fi

run_gate temporal_v144 \
  cargo run --locked --release --quiet -p binn-lab --bin temporal-deep-campaign -- \
    --out results/temporal_calibration_v144.md

if [[ ! -f results/temporal_task_calibration_v144.txt ]]; then
  log "STOP: v144 produced no valid frozen difficulty; v145/v146 remain blocked"
  write_summary "COMPLETE — stopped at INVALID_TASK gate"
  touch "$RUN_DIR/.complete"
  exit 0
fi

run_gate temporal_depth_v145 \
  cargo run --locked --release --quiet -p binn-lab --bin temporal-deep-campaign -- \
    --depth-run --out results/temporal_depth_v145.md

run_gate transfer_v146 \
  cargo run --locked --release --quiet -p binn-lab --bin transfer-falsifier -- \
    --out results/transfer_falsifier_v146.md \
    --bundle-dir results/transfer_v146_bundles

write_summary "COMPLETE"
touch "$RUN_DIR/.complete"
log "campaign execution complete; inspect report verdicts in $RUN_DIR/SUMMARY.md"
