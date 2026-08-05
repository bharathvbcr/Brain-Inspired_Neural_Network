#!/usr/bin/env bash
# Overnight causal size-science queue (H3 → H1 → H2 → Micro OP).
# SHD scientific is owned by another agent — left as a commented no-op.
#
#   ./scripts/overnight_scale.sh
#   ./scripts/overnight_scale.sh --skip-build
#
# Kill criteria (per plan):
#   H3: nnz not monotone in fan → abort science
#   H1: OOM / RSS>48GB / activity OOB → stop ladder
#   H2: no signal vs pm1 → do not scientific-ize
#   Micro OP: OOM / wall>20min/seed → reject OP
set -euo pipefail
cd "$(cd "$(dirname "$0")/.." && pwd)"

CAMP="results/runs/2026-07-24-overnight-scale"
mkdir -p "$CAMP"
LOG="${CAMP}/overnight_scale.log"
exec > >(tee -a "$LOG") 2>&1

SKIP_BUILD=0
for a in "$@"; do
  case "$a" in
    --skip-build) SKIP_BUILD=1 ;;
    *) echo "unknown arg: $a"; exit 2 ;;
  esac
done

C1=(cargo run --locked --release -p binn-lab --bin c1 --)
RSS_KILL_BYTES=$((48 * 1024 * 1024 * 1024))
MICRO_WALL_SECS=1200

echo "=== overnight_scale camp=${CAMP} $(date -u +%Y-%m-%dT%H:%M:%SZ) ==="

if [[ "$SKIP_BUILD" -eq 0 ]]; then
  echo "=== build c1 release ==="
  cargo build --locked --release -p binn-lab --bin c1
fi

run_isolate() {
  local tag="$1"; shift
  local out_json="${CAMP}/${tag}.json"
  local out_md="${CAMP}/${tag}.md"
  echo
  echo "=== ${tag} ==="
  local start end wall
  start=$(date +%s)
  if ! "${C1[@]}" "$@" --isolate-condition local-assembly --out "${out_md}" \
      | tee "${out_json}"; then
    echo "FAIL: ${tag} exited non-zero"
    return 1
  fi
  end=$(date +%s)
  wall=$((end - start))
  # Isolate prints JSON on stdout; also stash a one-line summary md.
  {
    echo "# ${tag}"
    echo
    echo "- wall_secs_script: ${wall}"
    echo "- cmdline: $*"
    echo
    echo '```json'
    grep -E '^\{' "${out_json}" | tail -1 || true
    echo '```'
  } > "${out_md}"
  # Kill checks from JSON fields when present.
  local rss nnz
  rss=$(grep -oE '"peak_rss_bytes":[0-9]+' "${out_json}" | head -1 | cut -d: -f2 || echo 0)
  nnz=$(grep -oE '"measured_nnz":[0-9]+' "${out_json}" | head -1 | cut -d: -f2 || echo 0)
  echo "  measured_nnz=${nnz} peak_rss_bytes=${rss} wall_secs=${wall}"
  if [[ -n "${rss}" && "${rss}" -gt "${RSS_KILL_BYTES}" ]]; then
    echo "KILL: RSS ${rss} > 48GB"
    return 2
  fi
  if [[ "${tag}" == h1-micro-* && "${wall}" -gt "${MICRO_WALL_SECS}" ]]; then
    echo "KILL: Micro OP wall ${wall}s > 20min/seed"
    return 3
  fi
  return 0
}

# ---- Phase 1: H3 density cross N=2k fan{10,32,64,256} quick ----
echo
echo "### H3 density cross"
H3_NNZ=()
H3_FANS=(10 32 64 256)
for fan in "${H3_FANS[@]}"; do
  tag="h3-n2000-fan${fan}-quick"
  run_isolate "$tag" --mac-probe --n-hidden 2000 --max-fan-out "$fan" --k-wta 8 --quick --seed 1
  nnz=$(grep -oE '"measured_nnz":[0-9]+' "${CAMP}/${tag}.json" | head -1 | cut -d: -f2)
  H3_NNZ+=("$nnz")
done

echo "H3 measured nnz by fan: ${H3_FANS[*]} → ${H3_NNZ[*]}"
prev=0
for nnz in "${H3_NNZ[@]}"; do
  if [[ "$nnz" -le "$prev" ]]; then
    echo "ABORT H3: measured nnz not monotone in fan (prev=${prev} cur=${nnz}) — wiring bug"
    exit 4
  fi
  prev=$nnz
done
echo "H3 nnz monotone: PASS"

# ---- Phase 2: H1 syn-matched smoke N=512,2000 (pm1, quick) ----
echo
echo "### H1 syn-matched width ladder (pm1)"
H1_PASS=1
for n in 512 2000; do
  tag="h1-synmatch-n${n}-quick"
  if ! run_isolate "$tag" --mac-probe --syn-matched --n-hidden "$n" --k-wta 8 --quick --seed 1; then
    echo "H1 ladder stop at N=${n}"
    H1_PASS=0
    break
  fi
  # Activity OOB check
  act=$(grep -oE '"activity_sparsity":[0-9.eE+-]+' "${CAMP}/${tag}.json" | head -1 | cut -d: -f2 || echo "")
  echo "  activity_sparsity=${act}"
done

# ---- Phase 3: H2 mode trio at Pass geometry (prefer N=2k syn-matched) ----
echo
echo "### H2 mode trio (quick)"
if [[ "$H1_PASS" -eq 1 ]]; then
  for mode in pm1 structured-fb dfa-live; do
    tag="h2-n2000-${mode}-quick"
    run_isolate "$tag" --mac-probe --syn-matched --n-hidden 2000 --k-wta 8 \
      --mac-mode "$mode" --quick --seed 1 || true
  done
  # Promote n=8 only if SFB/dfa signal vs pm1 (caller inspects JSON; script logs acc).
  echo "H2 quick accuracies:"
  for mode in pm1 structured-fb dfa-live; do
    f="${CAMP}/h2-n2000-${mode}-quick.json"
    if [[ -f "$f" ]]; then
      acc=$(grep -oE '"accuracy":[0-9.eE+-]+' "$f" | head -1 | cut -d: -f2 || echo "?")
      echo "  ${mode}: acc=${acc}"
    fi
  done
else
  echo "SKIP H2: H1 did not Pass harness"
fi

# ---- Phase 4: H1 Micro OP N=10k fan=10 quick (only if H1 Pass) ----
echo
echo "### H1 Micro OP N=10k fan=10"
if [[ "$H1_PASS" -eq 1 ]]; then
  run_isolate "h1-micro-n10000-fan10-quick" \
    --mac-probe --syn-matched --n-hidden 10000 --k-wta 8 --quick --seed 1 || \
    echo "Micro OP rejected (OOM/wall/fail)"
else
  echo "SKIP Micro OP: H1 did not Pass"
fi

# ---- SHD scientific: owned by another agent (no-op here) ----
echo
echo "### SHD scientific"
echo "# SKIP: SHD p27 scientific owned by sibling agent (see OVERNIGHT_NOTE stub)."
# Example (do not enable here):
# # cargo run --locked --release -p binn-lab --bin c1 -- --shd-cal --out "${CAMP}/shd_p27.md"

echo
echo "=== overnight_scale done $(date -u +%Y-%m-%dT%H:%M:%SZ) ==="
echo "Artifacts under ${CAMP}/"
