#!/usr/bin/env bash
# BINN full matrix: build everything, test everything, run every C1 variant.
#
#   ./scripts/run_all.sh                 # build + lint + tests + GC + C1 quick/full/replay/isolates
#   ./scripts/run_all.sh --with-benches  # also run criterion benches (slow)
#   ./scripts/run_all.sh --with-plots    # also rerun C1 full with the plotters plots feature
#   ./scripts/run_all.sh --with-post-g2  # also run opt-in post-G2 quick harnesses
#
# A scientific FAIL verdict at Gate G2 is a *valid* harness outcome and does not
# fail this script; only broken builds/tests/checks do.
set -uo pipefail
cd "$(cd "$(dirname "$0")/.." && pwd)"

WITH_BENCHES=0
WITH_PLOTS=0
WITH_POST_G2=0
for a in "$@"; do
  case "$a" in
    --with-benches) WITH_BENCHES=1 ;;
    --with-plots)   WITH_PLOTS=1 ;;
    --with-post-g2) WITH_POST_G2=1 ;;
    *) echo "unknown arg: $a (use --with-benches / --with-plots / --with-post-g2)"; exit 2 ;;
  esac
done

STAMP="$(date +%Y-%m-%d_%H%M%S)"
OUT="results/runs/${STAMP}"
mkdir -p "$OUT"

NAMES=()
RESULTS=()
step() {
  local name="$1"; shift
  echo
  echo "=== ${name} ==="
  if "$@" 2>&1 | tee "${OUT}/$(echo "$name" | tr ' /' '__').log"; then
    RESULTS+=("PASS")
  else
    RESULTS+=("FAIL")
  fi
  NAMES+=("$name")
}

# ---- 1. Build everything (debug + release, all targets incl. benches/examples)
step "build debug all-targets"   cargo build --locked --workspace --all-targets
step "build release all-targets" cargo build --locked --workspace --all-targets --release

# ---- 2. Lint + format
step "cargo fmt check" cargo fmt --all -- --check
step "cargo clippy -D warnings" cargo clippy --locked --workspace --all-targets -- -D warnings

# ---- 3. Full test suite (unit + property + determinism + doc tests)
step "cargo test workspace" cargo test --locked --workspace
step "cargo doc tests" cargo test --locked --workspace --doc
# The step above runs at opt-level 0. Every cell comes from a release build, and
# the attention arms hash differently between the two - see the script.
step "kernel in both profiles" ./scripts/check_kernel_profiles.sh

# ---- 4. Global constraints GC1-GC7
step "GC1-GC7 checks" ./scripts/gc_checks.sh

# ---- 5. Criterion benches (GC5 hot paths) — optional, slow
if [[ "$WITH_BENCHES" -eq 1 ]]; then
  step "criterion benches" cargo bench --locked --workspace
fi

# ---- 6. C1 / Gate G2 experiment matrix (protocol v2)
step "C1 quick pilot" cargo run --locked --release -p binn-lab --bin c1 -- --quick --out "${OUT}/c1_quick.md"

step "C1 full scientific run" cargo run --locked --release -p binn-lab --bin c1 -- --out "${OUT}/c1_full.md"

# ---- 7. Config-hash replay: full run must reproduce exactly (GC3)
FULL_LOG="${OUT}/C1_full_scientific_run.log"
HASH="$(grep -o 'c1-[0-9a-f]\{16\}' "$FULL_LOG" | head -1)"
if [[ -n "$HASH" ]]; then
  step "C1 replay ${HASH}" cargo run --locked --release -p binn-lab --bin c1 -- --config-hash "$HASH" --out "${OUT}/c1_full_replay.md"
  echo
  echo "=== replay determinism diff (verdict + summary lines) ==="
  for f in "$FULL_LOG" "${OUT}/C1_replay_${HASH}.log"; do
    grep -E 'G2 verdict|^means:|normalized-gap-closed|positive_control' "$f" > "${f}.summary" || true
  done
  if diff "${FULL_LOG}.summary" "${OUT}/C1_replay_${HASH}.log.summary"; then
    NAMES+=("replay determinism"); RESULTS+=("PASS")
  else
    NAMES+=("replay determinism"); RESULTS+=("FAIL")
  fi
else
  echo "could not extract config hash from full run — skipping replay"
  NAMES+=("replay determinism"); RESULTS+=("SKIP")
fi

# ---- 8. Per-condition isolates (peak-RSS single-condition paths)
for cond in local-assembly dense-local gradient-reference eligibility-reference; do
  step "isolate ${cond}" cargo run --locked --release -p binn-lab --bin c1 -- --quick --isolate-condition "$cond"
done

# ---- 9. Post-G2 extensions — explicit opt-in, quick/PILOT by default
# Refusal behavior is covered by `binn-lab/tests/override_refuse.rs`; do not
# treat the intentional exit-2 default as a failed build.
if [[ "$WITH_POST_G2" -eq 1 ]]; then
  step "C2 quick override" cargo run --locked --release -p binn-lab --bin c2 -- --enable-c2 --quick --out "${OUT}/c2_quick.md"
  step "C3 quick override" cargo run --locked --release -p binn-lab --bin c3 -- --enable-c3 --quick --out "${OUT}/c3_quick.md"
  step "R1 quick override" cargo run --locked --release -p binn-lab --bin r1 -- --enable-r1 --quick --out "${OUT}/r1_quick.md"
  step "R2 quick override" cargo run --locked --release -p binn-lab --bin r2 -- --enable-r2 --quick --out "${OUT}/r2_quick.md"
  step "U21-U23 quick override" cargo run --locked --release -p binn-lab --bin extensions -- --enable-extensions --quick --out-dir "${OUT}"
  step "U18-U20 quick override" cargo run --locked --release -p binn-lab --bin efficiency -- --enable-efficiency --quick --out "${OUT}/u20_efficiency_quick.md"
fi

# ---- 10. Optional: plots feature (Rust plotters; no Python)
if [[ "$WITH_PLOTS" -eq 1 ]]; then
  step "C1 full with plots" cargo run --locked --release -p binn-lab --features plots --bin c1 -- --out "${OUT}/c1_full_plots.md"
  step "paper figures" cargo run --locked --release -p binn-lab --features plots --bin paper-figures -- --out "${OUT}/figures"
fi

# ---- Summary
echo
echo "================ RUN-ALL SUMMARY ================"
FAILED=0
for i in "${!NAMES[@]}"; do
  printf '%-32s %s\n' "${NAMES[$i]}" "${RESULTS[$i]}"
  [[ "${RESULTS[$i]}" == "FAIL" ]] && FAILED=1
done
echo "artifacts: ${OUT}/"
grep -h 'G2 verdict' "${OUT}"/*.log 2>/dev/null | sort -u || true
exit "$FAILED"
