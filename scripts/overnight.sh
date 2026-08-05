#!/usr/bin/env bash
#
# BINN overnight run — 2026-07-25 hardening campaign.
#
# Design goals, in priority order:
#   1. NEVER waste the night on a broken build. Everything is gated on
#      `cargo check` + `cargo test` passing first.
#   2. NEVER lose completed work. Each job writes a `.done` marker; re-running
#      the script resumes rather than restarting.
#   3. NEVER let one hung job eat the night. Every job has a wall-clock timeout.
#   4. Decision-relevant work runs FIRST. If the machine dies at 3am you still
#      have the experiment that matters.
#
# Usage:
#   ./scripts/overnight.sh                # full run, resumable
#   ./scripts/overnight.sh --smoke        # tier 0+1 only (~15 min), verifies everything works
#   ./scripts/overnight.sh --force        # ignore .done markers, redo everything
#   ./scripts/overnight.sh --skip-gate    # DANGEROUS: skip build/test gate
#
# Resume after a crash: just run it again. Completed jobs are skipped.

set -uo pipefail

# ---------------------------------------------------------------------------
# Setup
# ---------------------------------------------------------------------------

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$REPO_ROOT" || exit 1

STAMP="$(date +%Y-%m-%d_%H%M%S)"
RUN_DIR="results/runs/${STAMP}_overnight"
LOG_DIR="$RUN_DIR/logs"
STATE_DIR="$RUN_DIR/.state"
NEW_RUN_DIR="$RUN_DIR"

SMOKE_ONLY=0
FORCE=0
SKIP_GATE=0
for arg in "$@"; do
  case "$arg" in
    --smoke)     SMOKE_ONLY=1 ;;
    --force)     FORCE=1 ;;
    --skip-gate) SKIP_GATE=1 ;;
    -h|--help)   sed -n '2,25p' "$0"; exit 0 ;;
    *) echo "unknown flag: $arg" >&2; exit 2 ;;
  esac
done

# Hash the executable protocol surface. A resumed directory may never mix
# binaries built from different source states.
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

# Resume into the most recent compatible, unfinished overnight dir unless forced.
if [[ $FORCE -eq 0 ]]; then
  LATEST="$(ls -1d results/runs/*_overnight 2>/dev/null | tail -1 || true)"
  if [[ -n "${LATEST:-}" && -d "$LATEST/.state" && ! -f "$LATEST/.complete" ]]; then
    PRIOR_FINGERPRINT="$(sed -n '1p' "$LATEST/.state/source_fingerprint" 2>/dev/null || true)"
    if [[ -n "$PRIOR_FINGERPRINT" && "$PRIOR_FINGERPRINT" == "$SOURCE_FINGERPRINT" ]]; then
      RUN_DIR="$LATEST"
      LOG_DIR="$RUN_DIR/logs"
      STATE_DIR="$RUN_DIR/.state"
      echo "==> resuming into compatible run dir: $RUN_DIR"
    else
      echo "==> latest run is legacy or source-incompatible; starting a new run"
      RUN_DIR="$NEW_RUN_DIR"
      LOG_DIR="$RUN_DIR/logs"
      STATE_DIR="$RUN_DIR/.state"
    fi
  fi
fi

mkdir -p "$LOG_DIR" "$STATE_DIR"
printf '%s\n' "$SOURCE_FINGERPRINT" > "$STATE_DIR/source_fingerprint"
MASTER_LOG="$RUN_DIR/overnight.log"

log() { printf '[%s] %s\n' "$(date +%H:%M:%S)" "$*" | tee -a "$MASTER_LOG"; }

# One overnight scheduler globally. `mkdir` is the portable atomic lock
# primitive and prevents `--force` or a source-incompatible invocation from
# racing an existing run in a second directory.
LOCK_DIR="results/runs/.overnight.lock"
if ! mkdir "$LOCK_DIR" 2>/dev/null; then
  LOCK_PID="$(sed -n '1p' "$LOCK_DIR/pid" 2>/dev/null || echo unknown)"
  if [[ "$LOCK_PID" =~ ^[0-9]+$ ]] && ! kill -0 "$LOCK_PID" 2>/dev/null; then
    rm -f "$LOCK_DIR/pid"
    rmdir "$LOCK_DIR" 2>/dev/null || true
    mkdir "$LOCK_DIR" || exit 4
  else
    echo "FATAL: an overnight scheduler is already active (pid $LOCK_PID)." >&2
    exit 4
  fi
fi
printf '%s\n' "$$" > "$LOCK_DIR/pid"

# Prefer GNU timeout; fall back to gtimeout, then the checked-in Python
# process-group timeout. Jobs are never allowed to silently become unbounded.
TIMEOUT_BIN=""
TIMEOUT_MODE=""
if command -v timeout >/dev/null 2>&1; then
  TIMEOUT_BIN="timeout"
  TIMEOUT_MODE="gnu"
elif command -v gtimeout >/dev/null 2>&1; then
  TIMEOUT_BIN="gtimeout"
  TIMEOUT_MODE="gnu"
elif command -v python3 >/dev/null 2>&1; then
  TIMEOUT_BIN="python3"
  TIMEOUT_MODE="python"
else
  log "FATAL: neither GNU timeout/gtimeout nor python3 is available."
  rmdir "$LOCK_DIR" 2>/dev/null || true
  exit 1
fi
log "timeout backend: $TIMEOUT_MODE ($TIMEOUT_BIN)"

# Keep the machine awake on macOS for the duration.
CAFFEINATE_PID=""
if command -v caffeinate >/dev/null 2>&1; then
  caffeinate -dimsu -w $$ >/dev/null 2>&1 &
  CAFFEINATE_PID=$!
  log "caffeinate active (pid $CAFFEINATE_PID) — machine will not sleep"
fi
cleanup() {
  if [[ -n "$CAFFEINATE_PID" ]]; then
    kill "$CAFFEINATE_PID" 2>/dev/null
    wait "$CAFFEINATE_PID" 2>/dev/null
  fi
  rm -f "$LOCK_DIR/pid" 2>/dev/null
  rmdir "$LOCK_DIR" 2>/dev/null
  return 0
}
trap cleanup EXIT

# ---------------------------------------------------------------------------
# Rayon thread count (Apple Silicon P/E asymmetry)
# ---------------------------------------------------------------------------
# rayon defaults to hw.ncpu, which on this host counts efficiency cores. The
# parallel kernels here are fork-join (`par_chunks_mut` in `assoc_scan`,
# `par_iter_mut` in the partitioned engine step): every chunk is the same size,
# so the barrier waits on the slowest worker. An E-core running a P-core-sized
# chunk is roughly 3x slower and becomes the straggler for the whole step.
#
# Default to the performance-core count (hw.perflevel0.logicalcpu). Override
# with RAYON_NUM_THREADS in the environment to measure the alternative.
#
# This does not affect results: `assoc_scan` is chunk-count-independent by
# construction (Phase 1 records exact sequential prefixes) and the engine
# sorts jobs by (partition, cell_id) before dispatch. Thread count changes
# wall time only, so GC3 determinism and replay hashes are unaffected.
if [[ -z "${RAYON_NUM_THREADS:-}" ]]; then
  P_CORES="$(sysctl -n hw.perflevel0.logicalcpu 2>/dev/null || true)"
  if [[ -n "$P_CORES" && "$P_CORES" -gt 0 ]]; then
    export RAYON_NUM_THREADS="$P_CORES"
  fi
fi

log "======================================================================"
log "BINN overnight run"
log "run dir : $RUN_DIR"
log "mode    : $([[ $SMOKE_ONLY -eq 1 ]] && echo SMOKE || echo FULL)"
log "host    : $(hostname) / $(uname -sm)"
log "rustc   : $(rustc --version 2>/dev/null || echo 'NOT FOUND')"
log "cores   : $(sysctl -n hw.ncpu 2>/dev/null || nproc 2>/dev/null || echo '?')"
log "  perf  : $(sysctl -n hw.perflevel0.logicalcpu 2>/dev/null || echo 'n/a')"
log "  eff   : $(sysctl -n hw.perflevel1.logicalcpu 2>/dev/null || echo 'n/a')"
log "rayon   : ${RAYON_NUM_THREADS:-<rayon default = hw.ncpu>}"
log "======================================================================"

# ---------------------------------------------------------------------------
# Job runner
# ---------------------------------------------------------------------------

declare -a JOB_NAMES=() JOB_STATUS=() JOB_SECS=()
ALL_JOB_NAMES=(
  gate_check gate_check_gpu gate_test gate_clippy build_release
  smoke_arch_ablation smoke_arch_lr_sweep smoke_c1_enhanced smoke_multi_area
  smoke_deep_snn smoke_ei_sweep smoke_neuromod smoke_shd_cal
  shd_arch_lr_pilot shd_arch_ablation_h128 shd_cal_h128 track_b_rescue
  live_transfer_rescue deep_snn_scaling multi_area_scaling c1_enhanced
  ei_inhibition_sweep multi_channel_neuromod
)

record_status() {
  local name="$1" status="$2" secs="$3"
  printf '%s\t%s\n' "$status" "$secs" > "$STATE_DIR/$name.status"
}

assert_source_unchanged() {
  local current
  current="$(source_fingerprint)"
  if [[ "$current" != "$SOURCE_FINGERPRINT" ]]; then
    log "FATAL: source changed after this run started."
    log "       Refusing to mix protocol states in $RUN_DIR."
    write_summary
    exit 5
  fi
}

# Defined before first use: `run_gate` and the --smoke early exit both call it.
write_summary() {
  local out="$RUN_DIR/SUMMARY.md"
  {
    echo "# Overnight run summary — $STAMP"
    echo
    echo "Run dir: \`$RUN_DIR\`"
    echo
    echo "| Job | Status | Wall (s) |"
    echo "|---|---|---:|"
    local name status secs
    for name in "${ALL_JOB_NAMES[@]}"; do
      status="pending"; secs="—"
      if [[ -f "$STATE_DIR/$name.status" ]]; then
        IFS=$'\t' read -r status secs < "$STATE_DIR/$name.status"
      elif [[ -f "$STATE_DIR/$name.done" ]]; then
        status="ok (legacy cached)"; secs="?"
      fi
      echo "| $name | $status | $secs |"
    done
    echo
    echo "## Read these first"
    echo
    if [[ $SMOKE_ONLY -eq 1 ]]; then
      echo "1. \`smoke_shd_arch_ablation.md\` — execution and validity guards only;"
      echo "   quick H1/H2 verdicts must remain \`UNDERPOWERED\`."
      echo "2. \`smoke_ei_sweep.md\` and \`smoke_neuromod.md\` — repaired property tests."
      echo "3. Tier 2+ is \`pending\`; this smoke run is not the decisive result."
    else
      echo "1. \`shd_arch_ablation_h128.md\` — **the decisive result.** Check the"
      echo "   preregistered H1/H2 verdicts and the shuffled-label control."
      echo "2. Any job with status \`fail\` or \`timeout\` — see \`logs/<job>.log\`."
      echo "3. Grep every report for harness flags:"
    fi
    echo
    echo '   ```bash'
    echo "   grep -rn 'INVALID_HARNESS\\|DEGENERATE\\|INVERTED\\|LEAK DETECTED\\|MISMATCH' $RUN_DIR/*.md"
    echo '   ```'
    echo
    echo "## Interpretation cheat-sheet"
    echo
    echo "- **H1 PASS** → the 0.234 figure was an architecture artifact. Restate the"
    echo "  SHD claim axis; re-run width/depth sweeps on the winning architecture."
    echo "- **H1 FAIL** → architecture is not the constraint. That is a real negative"
    echo "  result only on a confirmatory schedule. Protocol v142 includes the exact"
    echo "  ALIF adaptation term; keep the learning-rate sweep pilot-only."
    echo "- **INVALID_HARNESS anywhere** → no claim from that run, full stop."
    echo "- A \`readout arm ... is degenerate\` panic is the guard working, not a flake."
    echo
    echo "Execution status only records whether a process completed. Scientific"
    echo "PASS/FAIL/INVALID_HARNESS verdicts live in the generated reports."
  } > "$out"
  log ""
  log "summary written: $out"
}

run_job() {
  local name="$1" timeout_min="$2"; shift 2
  local marker="$STATE_DIR/$name.done"
  local logfile="$LOG_DIR/$name.log"

  if [[ $FORCE -eq 0 && -f "$marker" ]]; then
    local cached_status="ok" cached_secs="?"
    if [[ -f "$STATE_DIR/$name.status" ]]; then
      IFS=$'\t' read -r cached_status cached_secs < "$STATE_DIR/$name.status"
    fi
    log "CACHE $name ($cached_status, ${cached_secs}s)"
    JOB_NAMES+=("$name"); JOB_STATUS+=("$cached_status"); JOB_SECS+=("$cached_secs")
    return 0
  fi

  assert_source_unchanged
  log "START $name (timeout ${timeout_min}m)"
  local t0 rc
  t0=$(date +%s)

  if [[ "$TIMEOUT_MODE" == "gnu" ]]; then
    "$TIMEOUT_BIN" --foreground "${timeout_min}m" "$@" >"$logfile" 2>&1
    rc=$?
  else
    "$TIMEOUT_BIN" "$SCRIPT_DIR/run_with_timeout.py" "$(( timeout_min * 60 ))" \
      "$@" >"$logfile" 2>&1
    rc=$?
  fi

  local t1 secs
  t1=$(date +%s); secs=$((t1 - t0))

  if [[ $rc -eq 0 ]]; then
    date > "$marker"
    record_status "$name" "ok" "$secs"
    log "OK    $name (${secs}s)"
    JOB_STATUS+=("ok")
  elif [[ $rc -eq 124 || $rc -eq 137 ]]; then
    record_status "$name" "timeout" "$secs"
    log "TIMEOUT $name after ${timeout_min}m — see $logfile"
    JOB_STATUS+=("timeout")
  else
    record_status "$name" "fail:$rc" "$secs"
    log "FAIL  $name (exit $rc, ${secs}s) — see $logfile"
    log "      last 15 lines:"
    tail -15 "$logfile" 2>/dev/null | sed 's/^/        /' | tee -a "$MASTER_LOG"
    JOB_STATUS+=("fail:$rc")
  fi
  JOB_NAMES+=("$name"); JOB_SECS+=("$secs")
  return 0
}

# A gate job: if it fails, abort the whole run.
run_gate() {
  local name="$1" timeout_min="$2"; shift 2
  run_job "$name" "$timeout_min" "$@"
  local last_idx=$(( ${#JOB_STATUS[@]} - 1 ))
  local st="${JOB_STATUS[$last_idx]}"
  if [[ "$st" != "ok" && "$st" != "skipped" ]]; then
    log ""
    log "!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!"
    log "GATE FAILED: $name"
    log "Aborting before any experiment runs — the night is not wasted on a"
    log "broken build. Fix the errors in $LOG_DIR/$name.log and re-run."
    log "!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!"
    write_summary
    exit 1
  fi
}

# ---------------------------------------------------------------------------
# PREFLIGHT — catch "the whole night was wasted" conditions up front
# ---------------------------------------------------------------------------

log ""
log "--- PREFLIGHT ---"

PREFLIGHT_FATAL=0

if ! command -v cargo >/dev/null 2>&1; then
  log "FATAL: cargo not found on PATH."
  PREFLIGHT_FATAL=1
fi

# SHD cache. Every decision-relevant job needs it; without it they all abort
# with exit 3 and the night produces nothing.
SHD_DIR="${BINN_SHD_DIR:-data/shd}"
if [[ -f "$SHD_DIR/train.bin" && -f "$SHD_DIR/test.bin" ]]; then
  SHD_BYTES=$(( $(wc -c < "$SHD_DIR/train.bin") + $(wc -c < "$SHD_DIR/test.bin") ))
  log "SHD cache: $SHD_DIR ($(( SHD_BYTES / 1024 / 1024 )) MB)"
else
  log "FATAL: no SHD cache at '$SHD_DIR' (need train.bin + test.bin)."
  log ""
  log "  Every decision-relevant job tonight needs real SHD. The ablation refuses"
  log "  to run on the smoke fixture outside --quick, because a fixture run is"
  log "  indistinguishable from a real result in the report."
  log ""
  log "  Convert it first:"
  log "    PKG_CONFIG_PATH=\"\$(brew --prefix hdf5)/lib/pkgconfig:\${PKG_CONFIG_PATH:-}\" \\"
  log "      cargo run --locked --release -p binn-data --features shd-convert \\"
  log "        --bin convert-shd -- --cache-dir data/shd"
  log ""
  log "  Or point BINN_SHD_DIR at an existing cache."
  PREFLIGHT_FATAL=1
fi

# Disk headroom: reports + logs + release artifacts.
AVAIL_MB=$(df -m . 2>/dev/null | awk 'NR==2 {print $4}')
if [[ -n "${AVAIL_MB:-}" ]]; then
  log "disk free: ${AVAIL_MB} MB"
  if [[ "$AVAIL_MB" -lt 2048 ]]; then
    log "WARNING: under 2 GB free — release build plus logs may not fit."
  fi
fi

if [[ $PREFLIGHT_FATAL -eq 1 ]]; then
  log ""
  log "preflight failed — aborting before anything expensive runs."
  exit 1
fi
log "preflight ok"

# ---------------------------------------------------------------------------
# TIER 0 — build & test gate
# ---------------------------------------------------------------------------

if [[ $SKIP_GATE -eq 0 ]]; then
  log ""
  log "--- TIER 0: build & test gate ---"
  run_gate gate_check    20 cargo check --workspace --all-targets
  run_gate gate_check_gpu 10 cargo check -p binn-core --features gpu
  run_gate gate_test     30 cargo test --workspace
  # Clippy is advisory: a lint failure should not block the science.
  run_job  gate_clippy   20 cargo clippy --workspace --all-targets
  log ""
  log "--- gate passed: build is sound, all guards green ---"
fi

# Build release binaries once so job timings measure science, not compilation.
run_gate build_release 30 cargo build --release --workspace

# ---------------------------------------------------------------------------
# TIER 1 — smoke every rewritten experiment (fast; proves guards don't trip)
# ---------------------------------------------------------------------------

log ""
log "--- TIER 1: smoke tests (~15 min) ---"

run_job smoke_arch_ablation  15 cargo run --release --quiet -p binn-lab --bin shd-arch-ablation -- --quick --out "$RUN_DIR/smoke_shd_arch_ablation.md"
run_job smoke_arch_lr_sweep  20 cargo run --release --quiet -p binn-lab --bin shd-arch-ablation -- --quick --lr-sweep --out "$RUN_DIR/smoke_shd_arch_lr.md"
run_job smoke_c1_enhanced    10 cargo run --release --quiet -p binn-lab --bin c1-enhanced       -- --quick --out "$RUN_DIR/smoke_c1_enhanced.md"
run_job smoke_multi_area     10 cargo run --release --quiet -p binn-lab --bin multi-area-scaling -- --quick --out "$RUN_DIR/smoke_multi_area.md"
run_job smoke_deep_snn       20 cargo run --release --quiet -p binn-lab --bin deep-snn-scaling  -- --quick --out "$RUN_DIR/smoke_deep_snn.md"
run_job smoke_ei_sweep        5 cargo run --release --quiet -p binn-lab --bin ei-inhibition-sweep -- --out "$RUN_DIR/smoke_ei_sweep.md"
run_job smoke_neuromod        5 cargo run --release --quiet -p binn-lab --bin multi-channel-neuromod -- --out "$RUN_DIR/smoke_neuromod.md"
run_job smoke_shd_cal        15 cargo run --release --quiet -p binn-lab --bin c1 -- --shd-cal --quick --out "$RUN_DIR/smoke_shd_cal.md"

if [[ $SMOKE_ONLY -eq 1 ]]; then
  log ""
  log "--- smoke mode: stopping here ---"
  write_summary
  exit 0
fi

# ---------------------------------------------------------------------------
# TIER 2 — THE decisive experiment
# ---------------------------------------------------------------------------
#
# Is DFA ~= 0.234 on SHD a limit of local credit assignment, or of a
# feed-forward fixed-threshold forward model? Everything else is downstream of
# this answer, so it runs first and gets the most generous timeout.

log ""
log "--- TIER 2: SHD architecture ablation (the decisive experiment) ---"

# 2a. Learning-rate pilot FIRST — it is cheap and it de-risks the interpretation
# of everything after it. lr = 0.02 was inherited from `c1-shd-cal-*` and was
# never tuned for a recurrent or adaptive forward. If H1 fails at the fixed lr,
# this pilot is the only way to tell a real null from a learning-rate artifact.
# Running it first also means a mid-night crash still leaves this information.
run_job shd_arch_lr_pilot 150 \
  cargo run --release --quiet -p binn-lab --bin shd-arch-ablation -- \
    --lr-sweep --hidden 128 --out "$RUN_DIR/shd_arch_lr_pilot.md"

# 2b. The confirmatory run. Writes its report after every cell, so even a
# timeout leaves usable data, and cells run in H1-critical order.
run_job shd_arch_ablation_h128 300 \
  cargo run --release --quiet -p binn-lab --bin shd-arch-ablation -- \
    --hidden 128 --out "$RUN_DIR/shd_arch_ablation_h128.md"

# ---------------------------------------------------------------------------
# TIER 3 — supporting re-runs on fixed harnesses
# ---------------------------------------------------------------------------

log ""
log "--- TIER 3: supporting re-runs ---"

# SHD calibration with the fixed (scale-matched) ceiling. Directly comparable to
# the ff+fixed cell of the ablation; if they disagree, one harness is wrong.
# This is the cross-check that validates the new ALIF harness against the old one.
run_job shd_cal_h128 150 \
  cargo run --release --quiet -p binn-lab --bin c1 -- --shd-cal --shd-hidden 128 \
    --out "$RUN_DIR/c1_shd_h128.md"

# Gap-closed now clamped; both should report ceiling-inversion warnings.
run_job track_b_rescue 90 \
  cargo run --release --quiet -p binn-lab --bin track-b-rescue -- \
    --out "$RUN_DIR/track_b_rescue.md"

run_job live_transfer_rescue 90 \
  cargo run --release --quiet -p binn-lab --bin live-transfer-rescue -- \
    --out "$RUN_DIR/live_transfer_rescue.md"

# ---------------------------------------------------------------------------
# TIER 4 — low information value; run last, cheap timeouts
# ---------------------------------------------------------------------------
#
# NOT RUN: the h256 / h512 width sweep. 512 vs 128 measured +0.0056 with
# SE 0.0243 (t = 0.23) on the old harness — there is no width effect to find,
# and it costs ~6.6 hours. Re-run the width sweep only on whichever architecture
# the Tier 2 ablation selects, and only if Tier 2 shows a real architecture gain.

log ""
log "--- TIER 4: low-value suites ---"

# Depth scaling now trains 4 learned-FB arms AND 4 depth-matched ceilings per
# seed, so it costs roughly 2x the old suite. It is demoted to Tier 4 because it
# runs on `CoincidenceTask` with N_IN = 2: a 256-wide 4-deep stack on a
# 2-dimensional near-noiseless input has no depth structure to exploit, so the
# result is weak evidence either way. The `--quick` smoke in Tier 1 already
# confirms the new ceilings run and are not inverted; that is most of the value.
run_job deep_snn_scaling 240 \
  cargo run --release --quiet -p binn-lab --bin deep-snn-scaling -- \
    --out "$RUN_DIR/deep_snn_scaling.md"

run_job multi_area_scaling 60 \
  cargo run --release --quiet -p binn-lab --bin multi-area-scaling -- \
    --out "$RUN_DIR/multi_area_scaling.md"

run_job c1_enhanced 60 \
  cargo run --release --quiet -p binn-lab --bin c1-enhanced -- \
    --out "$RUN_DIR/c1_enhanced.md"

run_job ei_inhibition_sweep 15 \
  cargo run --release --quiet -p binn-lab --bin ei-inhibition-sweep -- \
    --out "$RUN_DIR/ei_inhibition_sweep.md"

run_job multi_channel_neuromod 15 \
  cargo run --release --quiet -p binn-lab --bin multi-channel-neuromod -- \
    --out "$RUN_DIR/multi_channel_neuromod.md"

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------

write_summary
touch "$RUN_DIR/.complete"

log ""
log "======================================================================"
log "overnight run complete"
log "reports : $RUN_DIR/*.md"
log "logs    : $LOG_DIR/"
log "summary : $RUN_DIR/SUMMARY.md"
log "======================================================================"
