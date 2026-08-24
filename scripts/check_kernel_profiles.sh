#!/usr/bin/env bash
# The kernel that produces every cell is the RELEASE build. The suite tests DEBUG.
#
# # The gap this closes
#
# Every place the test suite runs — `.github/workflows/ci.yml`, `run_all.sh`,
# `overnight.sh`, `run_code_transfer_campaign.sh`, and GC3/GC7 — invokes
# `cargo test` with no `--release`, i.e. opt-level 0. Every scientific cell is
# produced by `cargo build --locked --release` (`scripts/aws/bootstrap.sh`) and
# replayed by `scripts/gate_f_rust.py` from `target/release/shd-instrument`.
#
# So the profile the results come from was validated by nothing but Gate F,
# which regresses *recorded* cells and is therefore blind to any arm that has
# none. The profile the suite validates has never produced a result.
#
# # Why that is not hypothetical here
#
# `shd_attention::positional_code` calls `.sin()` and `.cos()` on the same
# argument. At opt-level >= 2 LLVM's libcall simplification merges the pair into
# Darwin's combined `__sincosf_stret`; at 0 and 1 it emits separate `sinf` and
# `cosf`. The two do not agree to the last ulp, the spike threshold is hard, and
# the difference compounds through Adam — so the attention arms hash differently
# between profiles, and both sides are pinned
# (`PIN_*_OPTIMISED` / `PIN_*_UNOPTIMISED` in `shd_matched_arms.rs`).
#
# That split is exactly what makes a single-profile suite dangerous: a kernel
# change that moved only the optimised side would pass `cargo test`, pass
# GC1–GC7, pass CI, and silently alter every cell produced afterwards.
#
# # What this does
#
# Runs the kernel's own tests in **both** profiles and requires both to pass.
# Scoped to the two crates that carry the instrument, so it costs one extra
# optimised build of those rather than of the workspace.
#
#     bash scripts/check_kernel_profiles.sh
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

# The crates that compute or drive a cell. A change anywhere else cannot move a
# recorded number without going through these.
PACKAGES=(binn-learn binn-lab)

fail=0
# Written as two explicit invocations rather than an array of flags: macOS ships
# bash 3.2, where `"${empty[@]}"` under `set -u` is an unbound-variable error.
# The first version of this script died on exactly that, on its first run.
for profile in debug release; do
  for package in "${PACKAGES[@]}"; do
    echo "=== ${package}, ${profile} ==="
    if [[ "$profile" == "release" ]]; then
      cargo test --locked --release -p "$package" --all-targets || fail=1
    else
      cargo test --locked -p "$package" --all-targets || fail=1
    fi
    [[ "$fail" -eq 1 ]] && echo "FAIL: ${package} in the ${profile} profile"
  done
done

if [[ "$fail" -ne 0 ]]; then
  echo
  echo "The kernel does not pass in both profiles."
  echo "If the attention pins are what moved, decide which side changed before"
  echo "re-pinning: run tests/attention_w_in_independent.rs, which derives the"
  echo "gradient from the documented equations in whatever profile it is built"
  echo "under and therefore answers 'is the kernel right' independently of the"
  echo "pin. Re-pin only the side that actually moved, and say which."
  exit 1
fi

echo
echo "kernel passes in both profiles (debug and release), for: ${PACKAGES[*]}"
