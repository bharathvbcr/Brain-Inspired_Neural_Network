#!/usr/bin/env bash
# Run all BINN experiment suites and save generated reports to results/runs/all_experiments/
set -e

cd "$(cd "$(dirname "$0")/.." && pwd)"

STAMP="$(date +%Y-%m-%d_%H%M%S)"
OUT_DIR="results/runs/${STAMP}_all_experiments"
mkdir -p "$OUT_DIR"

echo "========================================================================"
echo " Starting Full BINN Scientific & Scaling Experiment Suite"
echo " Output Directory: ${OUT_DIR}"
echo " Timestamp: ${STAMP}"
echo "========================================================================"
echo

# 1. Deep SNN Depth Scaling (1L, 2L, 3L, 4L)
echo ">>> [1/10] Running Deep SNN Depth Scaling (1L..4L)..."
cargo run --release -p binn-lab --bin deep-snn-scaling -- --out "${OUT_DIR}/deep_snn_scaling.md"

# 2. Multi-Area Structural Scaling (2, 4, 8 areas)
echo ">>> [2/10] Running Multi-Area Structural Scaling..."
cargo run --release -p binn-lab --bin multi-area-scaling -- --eval-scaling --bench-scaling --out "${OUT_DIR}/multi_area_scaling.md"

# 3. Enhanced Engine & Metal GPU Benchmark
echo ">>> [3/10] Running Enhanced Spiking Engine & Metal GPU Benchmark..."
cargo run --release -p binn-lab --bin c1-enhanced -- --eval-enhanced --bench-metal --out "${OUT_DIR}/c1_enhanced.md"

# 4. Multi-Channel Neuromodulation
echo ">>> [4/10] Running Multi-Channel Neuromodulation Suite..."
cargo run --release -p binn-lab --bin multi-channel-neuromod -- --out "${OUT_DIR}/multi_channel_neuromod.md"

# 5. E/I Inhibition Balance Sweep
echo ">>> [5/10] Running E/I Inhibition Balance Sweep..."
cargo run --release -p binn-lab --bin ei-inhibition-sweep -- --out "${OUT_DIR}/ei_inhibition_sweep.md"

# 6. Protocol 25: Online Learned B_i Live Transfer
echo ">>> [6/10] Running Protocol 25 (Online Learned B_i Live Transfer)..."
cargo run --release -p binn-lab --bin c1 -- --rfb-learned --out "${OUT_DIR}/c1_rfb_learned.md"

# 7. Protocol 28: Adaptive k-WTA Schedule (16 -> 2)
echo ">>> [7/10] Running Protocol 28 (Adaptive k-WTA Schedule)..."
cargo run --release -p binn-lab --bin c1 -- --k-anneal --out "${OUT_DIR}/c1_k_anneal.md"

# 8. SHD Audio Classification Suite (h128, h256, h512, full-smoke)
echo ">>> [8/10] Running SHD Audio Calibration & Full Smoke..."
cargo run --release -p binn-lab --bin c1 -- --shd-cal --shd-hidden 128 --out "${OUT_DIR}/c1_shd_h128.md"
cargo run --release -p binn-lab --bin c1 -- --shd-cal --shd-hidden 256 --out "${OUT_DIR}/c1_shd_h256.md"
cargo run --release -p binn-lab --bin c1 -- --shd-cal --shd-hidden 512 --out "${OUT_DIR}/c1_shd_h512.md"
cargo run --release -p binn-lab --bin c1 -- --shd-full --smoke --out "${OUT_DIR}/c1_shd_full_smoke.md"

# 9. Track B Rescue & Live Transfer Rescue
echo ">>> [9/10] Running Track B & Live Transfer Rescue Suites..."
cargo run --release -p binn-lab --bin track-b-rescue -- --out "${OUT_DIR}/track_b_rescue.md"
cargo run --release -p binn-lab --bin live-transfer-rescue -- --out "${OUT_DIR}/live_transfer_rescue.md"

# 10. DFA Live Size & Calibrated Spiking Protocol
echo ">>> [10/10] Running DFA-Live Size (N=2k) & Calibrated Spiking (c1-spike-s)..."
cargo run --release -p binn-lab --bin c1 -- --dfa-live-size --isolate-condition local-assembly --out "${OUT_DIR}/c1_dfa_live_size.md"
cargo run --release -p binn-lab --bin c1 -- --spike-s --out "${OUT_DIR}/c1_spike_s.md"

echo
echo "========================================================================"
echo " All Experiments Successfully Executed!"
echo " All markdown reports saved in: ${OUT_DIR}/"
echo "========================================================================"
