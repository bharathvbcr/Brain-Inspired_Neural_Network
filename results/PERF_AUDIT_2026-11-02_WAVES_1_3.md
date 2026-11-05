# Performance Audit & Benchmark Report: Waves 1–4 Optimizations

**Host of record:** Apple M5 Pro (`Mac17,8`, 64 GB unified memory, macOS 27.2), `aarch64-apple-darwin`.  
**Date:** 2026-11-03 (updated through Wave 4)  
**Evaluation protocol:** 5-round interleaved ABBA (`pre -> post -> post -> pre -> pre -> post...`, reporting both `min-of-5` and `median-of-5`).  
**Determinism & Verification:** 100% IEEE-754 bit-identical CPU numerics and PRNG draw order verified across debug and release builds (`check_kernel_profiles.sh`), `c1 --quick` (`c1-e0dfdbf4e3d2936b`), `c1 --micro --n-hidden 10000` (`c1-micro-713322a01c532f01`), `c1 --shd-cal --quick`, `c1 --shd-full --quick`, `shd-instrument temporal-sensitivity`, `shd-instrument train-cell --probe-epochs`, and all workspace + Metal GPU validation tests (`MTL_SHADER_VALIDATION=1 MTL_DEBUG_LAYER=1`). Zero `unsafe` added in BINN library crates (`GC6`), zero new dependencies (`GC2`), zero weakened tests (`find_weak_checks.py --strict`: 39 baseline, 0 new).

---

## 1. Executive Summary: How Fast Is It After Wave 4?

Across four waves of architectural, mega-kernel, GPU command-buffer fusion, and lazy dirty-tracking optimizations in `binn-core`, `binn-engine`, `binn-areas`, `binn-learn`, `binn-lab`, and `sparsl`:

1. **End-to-End Large-Graph Simulation (`c1 --micro`, $N = 10,000$ cells, $2.57\times 10^6$ synapses):**
   - **Pre-Wave-1 baseline:** $\sim 844\text{ ms}$ per run.
   - **Post-Wave-3 / Pre-Wave-4 baseline:** $260.0\text{ ms}$ process wall ($246.3\text{ ms}$ condition wall).
   - **Post-Wave-4 optimized:** **$104.3\text{ ms}$ process wall** (**$92.4\text{ ms}$ condition wall**) — a **$2.49\times$ process / $2.67\times$ condition min-of-5 speedup** ($2.66\times$ / $3.01\times$ median) in Wave 4 alone, and **$\sim 8.1\times$ process / $9.1\times$ condition cumulative speedup** over pre-Wave-1.
   - **Memory footprint:** Peak RSS is **$185.4\text{ MB}$** vs pre-Wave-4 $183.1\text{ MB}$ ($+2.3\text{ MB}$ for the 1-byte `elig_dirty_flag` table across $2.57\text{M}$ synapses, still **$-8.4\text{ MB}$ below pre-Wave-3's $193.9\text{ MB}$** because `Engine::schedule_fan_out` streams `Engine::edge_w` directly without duplicating SoA weights).

2. **SHD Training & Evaluation Mega-Kernels (`binn-learn` & `binn-lab`):**
   - **Feedforward fixed-threshold ALIF (`700×128, T=100`):** **$43.96\times$ min-of-5** ($49.32\times$ median, $178.6\text{ ms} \to 4.06\text{ ms}$ per 16 steps) by factoring per-synapse $(N_{\text{in}}\times H)$ eligibility traces into 1D presynaptic vectors ($N_{\text{in}}$) with outer-product accumulation only at nonzero error steps.
   - **Recurrent adaptive ALIF (`700×128, T=100`):** **$10.22\times$ min-of-5** ($9.92\times$ median, $299.1\text{ ms} \to 29.3\text{ ms}$) via factored 1D voltage eligibility traces (`eps_v_in`, `eps_v_rec`) + sparse active-channel input scans.
   - **e-prop / DFA / RL 700-channel SHD (`c1 --shd-cal` / `c1 --shd-full`):** **$8.72\times$ kernel speedup** ($102.6\text{ ms} \to 11.8\text{ ms}$) and **$1.95\times\text{–}2.03\times$ full-process binary speedup** ($15.4\text{ ms} \to 7.6\text{ ms}$).
   - **Forward-only evaluation & temporal-sensitivity probing (`rec+alif+attn`):** **$4.47\times$ kernel speedup** ($3.69\text{ ms} \to 0.83\text{ ms}$) and **$2.38\times$ end-to-end `shd-instrument temporal-sensitivity` speedup** ($1,205.9\text{ ms} \to 507.0\text{ ms}$) by skipping backward trajectory materialization during eval/probing.

3. **Metal GPU Single-Pass & Multi-Tick Recurrent Mega-Kernels (`sparsl` + `binn-core`):**
   - **Device-resident 64-tick recurrent SpMV+LIF (`N=4,096..16,384, deg=128`):** Encodes up to `MAX_RECURRENT_STEPS_PER_CB = 256` recurrent ticks into **1 `MTLComputeCommandEncoder` + 1 `MTLCommandBuffer`** (`lif_uploads = 1, lif_syncs = 1`), delivering **$18.93\times\text{–}35.25\times$ min-of-5** ($2.12\times\text{–}2.13\times$ median) over per-tick host-packed launches and **$1.70\times\text{–}11.13\times$ min-of-5** ($1.36\times\text{–}1.57\times$ median) over per-tick dense launches.
   - **Single-pass fused packed-spike + LIF (`N=50k`) & skewed CSR hybrid (`N=16k`):** **$3.88\times$** and **$1.68\times$ min-of-5** ($1.21\times$ and $1.74\times$ median) over 2-pass `spmv_spikes` + `lif_integrate` by fusing bitmask reads and membrane integration into one kernel/encoder.

4. **Lazy Dirty-State Tracking (`binn-engine`):**
   - **Lazy `Engine::reset_state` ($N=10,000$, 64 active cells):** **$90.57\times$ min-of-5** ($93.45\times$ median, $10.23\text{ }\mu\text{s} \to 0.113\text{ }\mu\text{s}$), armed lazily on first `reset_state()` so one-shot engines pay zero allocation overhead.
   - **Synaptic delay lookup (`max_synaptic_delay_500k`):** **$0.25\text{ ns}$** ($\sim 606,000\times$ faster than pre-Wave-3 linear scans).

---

## 2. Wave 4: 5-Round Interleaved ABBA Results

### 2.1. End-to-End Binaries & Criterion Harnesses (`pre_wave4` vs `post_wave4`)

| Benchmark Target | Pre-Wave-4 Min | Post-Wave-4 Min | Min-of-5 Speedup | Median-of-5 Speedup | Mechanism & Notes |
|---|---:|---:|---:|---:|---|
| **`c1_micro_10k_cond_wall`** ($N=10\text{k}$, $2.57\text{M}$ syn) | $246.31\text{ ms}$ | **$92.38\text{ ms}$** | **$2.67\times$** | **$3.01\times$** | Lazy `O(D_elig)` `ThreeFactor::apply_weights` + `zero_eligibilities`, `edge_w` SoA fan-out, `+INF` threshold `expf` bypass, indexed readout scan |
| **`c1_micro_10k_proc_wall`** (full binary) | $260.00\text{ ms}$ | **$104.25\text{ ms}$** | **$2.49\times$** | **$2.66\times$** | $\sim 8.10\times$ cumulative vs pre-Wave-1 ($844\text{ ms}$); RSS $183.12 \to 185.43\text{ MB}$ ($+2.31\text{ MB}$) |
| **`shd_instrument_temporal_sensitivity_64`** | $1,205.92\text{ ms}$ | **$506.99\text{ ms}$** | **$2.38\times$** | **$2.29\times$** | Forward-only `shd_matched_forward_arm_tau` + hoisted `tau_cycles` + non-inlined `sincos_pair` |
| **`c1_shd_full_quick`** (700-ch SHD) | $15.40\text{ ms}$ | **$7.58\text{ ms}$** | **$2.03\times$** | **$1.96\times$** | Active-channel sparse scans + channel-major `forward_with_elig` |
| **`c1_shd_cal_quick`** (700-ch SHD) | $15.87\text{ ms}$ | **$8.13\text{ ms}$** | **$1.95\times$** | **$1.78\times$** | Active-channel sparse scans across `ShdEpropCeiling`, `ShdDfa`, `ShdRlReinforceFb` |
| **`multi_area/fused_inter_area_step_2048`** | $7.49\text{ }\mu\text{s}$ | **$6.62\text{ }\mu\text{s}$** | **$1.13\times$** | **$1.17\times$** | In-place scratch `k_wta_in_place` eliminating per-area `Vec` reallocations |
| **`shd_instrument_train_cell_4ep_probed`** | $514.51\text{ ms}$ | **$457.89\text{ ms}$** | **$1.12\times$** | **$1.13\times$** | Forward-only epoch probes + streaming zero-alloc `Adam::update` / `global_norm` + `ShdArmWorkspace` |
| **`kwta/n10000_k100`** | $14.85\text{ }\mu\text{s}$ | **$13.36\text{ }\mu\text{s}$** | **$1.11\times$** | **$1.16\times$** | Pre-sized `k_wta_in_place_finite` without duplicate `.retain()` pass |
| **`timing_wheel/10000`** | $86.63\text{ }\mu\text{s}$ | **$78.86\text{ }\mu\text{s}$** | **$1.10\times$** | **$1.10\times$** | Batched bucket insertion (`TimingWheel::insert_batch_at`) |
| **`plasticity/sparse_three_factor_64k`** | $444.93\text{ }\mu\text{s}$ | **$409.68\text{ }\mu\text{s}$** | **$1.09\times$** | **$1.04\times$** | Clean-synapse STDP timestamping + zero-overhead `elig_all_dirty` slice fast path |
| **`engine_step/1024_external_events`** | $14.81\text{ }\mu\text{s}$ | **$15.00\text{ }\mu\text{s}$** | $0.99\times$ | $0.98\times$ | Lazy-armed `dirty_cells_armed` keeps one-shot engine construction within $0.2\text{ }\mu\text{s}$ noise |
| **`plasticity/three_factor_coincidence_trial`** | $1.74\text{ }\mu\text{s}$ | **$1.80\text{ }\mu\text{s}$** | $0.97\times$ | $0.97\times$ | 3-synapse micro-trial ($60\text{ ns}$ delta within timer jitter) |

### 2.2. CPU & Metal GPU Mega-Kernel Microbenchmarks (5-Round Interleaved ABBA)

| Kernel / Substrate Benchmark | Reference Min | Mega-Kernel / Lazy Min | Min-of-5 Speedup | Median-of-5 Speedup | Architectural Optimization |
|---|---:|---:|---:|---:|---|
| **`engine_reset_10k_cells_64_dirty`** | $10.23\text{ }\mu\text{s}$ | **$0.113\text{ }\mu\text{s}$** | **$90.57\times$** | **$93.45\times$** | Lazy `O(D)` dirty-cell reset instead of $O(N)$ full-population `Cell::reset` |
| **`alif_ff_fixed_16x_700x128_t100`** | $178.56\text{ ms}$ | **$4.06\text{ ms}$** | **$43.96\times$** | **$49.32\times$** | Factored 1D `eps_v_in` ($700$ floats vs $89,600$ synapses) + `AlifWorkspace` |
| **`metal_recurrent_64ticks_n4096_vs_packed`** | $37.96\text{ ms}$ | **$1.08\text{ ms}$** | **$35.25\times$** | **$2.12\times$** | 64 recurrent ticks in 1 `MTLComputeCommandEncoder` + 1 `MTLCommandBuffer` |
| **`metal_recurrent_64ticks_n16384_vs_packed`**| $54.47\text{ ms}$ | **$2.88\text{ ms}$** | **$18.93\times$** | **$2.13\times$** | Device-resident `pack_uchar_spikes_kernel` + `fused_spikes_lif` ping-pong |
| **`metal_recurrent_64ticks_n16384_vs_dense`** | $66.38\text{ ms}$ | **$5.96\text{ ms}$** | **$11.13\times$** | **$1.36\times$** | Eliminates 63 intermediate `waitUntilCompleted` host round-trips |
| **`alif_rec_adaptive_16x_700x128_t100`** | $299.06\text{ ms}$ | **$29.27\text{ ms}$** | **$10.22\times$** | **$9.92\times$** | Factored 1D `eps_v_in`/`eps_v_rec` + active-channel sparse input scans |
| **`eprop_dfa_16x_700x128_t100`** | $102.57\text{ ms}$ | **$11.76\text{ ms}$** | **$8.72\times$** | **$8.89\times$** | Active-channel sparse forward/eligibility + channel-major `w_in` layout |
| **`matched_rec_alif_attn_t250_forward_only`** | $3.69\text{ ms}$ | **$0.83\text{ ms}$** | **$4.47\times$** | **$4.63\times$** | Forward-only `shd_matched_forward_arm_tau` skipping BPTT trajectory storage |
| **`metal_fused_spikes_lif_50k_vs_2pass`** | $3.48\text{ ms}$ | **$0.90\text{ ms}$** | **$3.88\times$** | **$1.21\times$** | Single-pass `fused_spikes_lif_{l1,l8,l32}` eliminating intermediate current DRAM pass |
| **`metal_recurrent_64ticks_n4096_vs_dense`** | $31.03\text{ ms}$ | **$18.22\text{ ms}$** | **$1.70\times$** | **$1.57\times$** | Single command buffer across 64 recurrent ticks |
| **`metal_skewed_hybrid_fused_spikes_lif_16k`**| $0.94\text{ ms}$ | **$0.56\text{ ms}$** | **$1.68\times$** | **$1.74\times$** | Single-encoder indexed hybrid (`FUSED_SPIKES_LIF_INDEXED_KERNEL` l1+l32) |
| **`binn_core_spmv_backend_steps_64ticks`** | $50.18\text{ ms}$ | **$33.93\text{ ms}$** | **$1.48\times$** | **$1.44\times$** | `SpmvBackend::fused_spmv_lif_steps` (`lif_uploads=1, lif_syncs=1` for 64 steps) |
| **`matched_rec_alif_attn_t250_workspace_bptt`**| $4.38\text{ ms}$ | **$4.41\text{ ms}$** | $0.99\times$ | $1.00\times$ | Single-thread parity; eliminates per-sample `Vec` allocator contention across Rayon workers |

---

## 3. Historical Wave 3 Baseline Results (Preserved for Provenance)

Raw measurements from the Wave 3 5-round interleaved ABBA pass on the same M5 Pro host:

| Benchmark Target | Pre-Wave-3 Min | Post-Wave-3 Min | Min-of-5 Speedup | Median-of-5 Speedup | Mechanism & Optimizations Applied |
|---|---:|---:|---:|---:|---|
| **`c1_micro_10k_wall`** ($N=10\text{k}$) | $330.56\text{ ms}$ | **$206.07\text{ ms}$** | **$1.60\times$** | **$1.50\times$** | Atomic delay cache, zero-alloc wiring, clean-line `zero_eligibilities`, resting `expf` fast path |
| **`c1_micro_10k_rss_bytes`** | $193.86\text{ MB}$ | **$183.62\text{ MB}$** | **$-10.24\text{ MB}$** | **$-10.24\text{ MB}$** | Eliminated intermediate CSR row allocations, avoided redundant dirty cache-line sweeps |
| **`engine_step/max_synaptic_delay_500k`** | $153.11\text{ }\mu\text{s}$ | **$0.25\text{ ns}$** | **$605,776\times$** | **$620,745\times$** | Replaced $O(\text{nnz})$ linear traversal with cached `AtomicU64` and `uniform_delay` |
| **`wiring/intra_area_edge_fraction_10k`** | $5.252\text{ ms}$ | **$1.092\text{ ms}$** | **$4.81\times$** | **$4.96\times$** | Replaced linear `prior.areas` scan per edge with $O(1)$ precomputed `cell_area` lookup |
| **`wiring/wire_10k_fan256`** | $48.36\text{ ms}$ | **$27.81\text{ ms}$** | **$1.74\times$** | **$1.55\times$** | Replaced $O(\text{fan}^2)$ `.contains()` with `SeenSet`; built CSR `row_ptr`/`col` directly |
| **`resting_nulls/activity_matched_1024x128`**| $248.74\text{ }\mu\text{s}$ | **$161.26\text{ }\mu\text{s}$** | **$1.54\times$** | **$1.59\times$** | Replaced $O(k)$ array search with flat `u64` bitset membership check |
| **`resting_nulls/rate_matched_1024x128`** | $411.77\text{ }\mu\text{s}$ | **$274.67\text{ }\mu\text{s}$** | **$1.50\times$** | **$1.48\times$** | Replaced $O(k)$ array search with flat `u64` bitset membership check |
| **`sparsl/metal_lif_integrate_65536`** | $280.38\text{ }\mu\text{s}$ | **$226.40\text{ }\mu\text{s}$** | **$1.24\times$** | **$1.32\times$** | Grow-only `LifScratch` pooling; eliminates 4 `MTLBuffer` + 1 `Vec<u8>` allocs per call |
| **`engine_step/1024_external_events`** | $20.03\text{ }\mu\text{s}$ | **$16.23\text{ }\mu\text{s}$** | **$1.23\times$** | **$1.21\times$** | Fast-path resting-state `expf` bypass; hoisted uniform-delay delivery calculation |
| **`fused_inter_area_step_2048`** | $8.26\text{ }\mu\text{s}$ | **$7.60\text{ }\mu\text{s}$** | **$1.09\times$** | **$1.09\times$** | Reusable `dst_charge_scratch` and `scores_scratch` on `MultiAreaNetwork` |
| **`overlap_count_256`** | $802.0\text{ ns}$ | **$759.7\text{ ns}$** | **$1.06\times$** | **$1.08\times$** | Sorted two-pointer merge for balanced sets; fallback binary search for skewed sets |
| **`three_factor_coincidence_trial`** | $1.90\text{ }\mu\text{s}$ | **$1.80\text{ }\mu\text{s}$** | **$1.06\times$** | **$1.04\times$** | Zero-eligibility fast path, delay cache retention, hoisted modulator finiteness check |

---

## 4. Subsystem Breakdown of Waves 1–4 Optimizations

### 4.1. `sparsl` & `binn-core` (Metal GPU Mega-Kernels & Zero-Bounce Unified Memory)
1. **Single-Pass Fused Spike-Bitmask + LIF Kernels:** Added `fused_spikes_lif_{l1,l8,l32}` and `pack_uchar_spikes_kernel` in `sparsl/src/kernels/spmv.metal`, plus safe CPU sequential/parallel equivalents (`cpu_fused_spikes_sequential`, `cpu_fused_spikes_parallel`) in `sparsl/src/spikes.rs`.
2. **Single-Encoder Indexed Hybrid Dispatch:** Added row/column-indexed Metal kernels (`spmv_indexed`, `spmv_spikes_indexed`, `spmv_t_indexed`, `fused_lif_indexed`, `fused_spikes_lif_indexed`, `csr_spmm_indexed_kernel`) so skewed CSR graphs run short-row `l1` and heavy-row `l32` buckets over one full-matrix GPU upload inside a single `MTLComputeCommandEncoder`.
3. **Multi-Tick Device-Resident Recurrent Fusion:** `PreparedCsr::fused_spmv_spikes_lif_recurrent_steps` and `binn_core::SpmvBackend::fused_spmv_lif_steps` keep `(v, theta, spikes, x_spikes)` resident on Unified Memory and encode up to `MAX_RECURRENT_STEPS_PER_CB = 256` ticks per command buffer (`lif_uploads = 1, lif_syncs = 1`, verified by `metal_lif_transfer_stats`).
4. **Zero-Allocation Segment Packing & Spike Readout:** Guarded `x_spikes` buffer, zero-alloc `write_segments_from` / `write_narrow_segments_from`, and direct `read_u8_spikes_as_bool` over `StorageModeShared` memory.

### 4.2. `binn-engine` (Lazy Dirty-State Tracking & Unified `edge_w` SoA Fan-Out)
1. **Lazy Dirty-Eligibility Tracking (`O(D_elig)`):** `Synapses` tracks non-zero eligibility edges in `dirty_elig_edges` with deferred `elig_floor_tick` stamping on idle synapses. `ThreeFactor::apply_weights` and `Synapses::zero_eligibilities` scale with touched active edges rather than $O(\text{nnz})$, while `stamp_clean_elig_time` avoids polluting `dirty_elig_edges` when presynaptic or postsynaptic spikes hit unpaired zero-eligibility edges.
2. **Unified `Engine::edge_w` SoA Fan-Out & Batched Timing Wheel:** `Engine::schedule_fan_out` streams contiguous `f32` weights directly from `Engine::edge_w` (when `Synapses::is_edge_w_synced()` holds) into `TimingWheel::insert_batch_at` (`4` bytes/edge vs `32`-byte `Synapse` stride, zero duplicate SoA weight memory).
3. **Lazy-Armed `O(D)` Cell Reset & Exact `+INF` Threshold Bypass:** `Engine::reset_state` arms `dirty_cells_armed` on first reset so multi-trial simulations reset only touched cells in $O(D)$ while one-shot engines allocate nothing extra; `Cell::advance_to` bypasses `expf` bit-identically when `theta == +INF && dtf <= 2000.0` (muted hidden populations).

### 4.3. `binn-areas` (In-Place Scratch `k-WTA` & Zero-Alloc Wiring)
1. **In-Place Scratch Selection:** `k_wta_in_place`, `k_wta_in_place_finite`, and scratch-reusing `soft_k_wta` eliminate per-step candidate `Vec` allocations across `MultiAreaNetwork::fused_inter_area_step`, `project`, and `project_reference`.
2. **Direct CSR Wiring & Flat Area Tables:** Generation-stamped `SeenSet` sampling and $O(1)$ `cell_area` lookup tables from Wave 3 remain intact.

### 4.4. `binn-learn` & `binn-lab` (Factored Eligibility, Sparse Input Scans & Forward-Only Eval)
1. **Factored 1D ALIF Eligibility (`ShdAlifArm`):** Replaces $O(N_{\text{in}}\cdot H + H^2)$ per-tick voltage trace updates with $O(|\text{active\_in}| + H)$ 1D filtered spike vectors (`eps_v_in`, `eps_v_rec`) plus `AlifWorkspace` reuse.
2. **Active-Channel Sparse Input Scans (`ShdEpropCeiling`, `ShdDfa`, `ShdRlReinforceFb`, `SuperSpikeBaseline`):** Pre-extracts active `(ch, val)` entries per frame and stores `w_in` channel-major so forward currents and eligibility updates touch only firing channels (~1–4% of 700 SHD channels).
3. **Forward-Only Matched-Arm Eval, Hoisted RoPE & Streaming Optimizers:** `shd_matched_forward_arm_tau` evaluates all 8 matched arms (`lif`/`alif`/`rec`/`attn`) without allocating BPTT backward tapes; `shd_attention` hoists `tau_cycles` and isolates `sincos_pair` so vector loops stay register-clean; `shared_bptt` streams `Adam::update`, `apply_sgd_step`, `global_norm`, and `parameter_fingerprint` with zero temporary parameter flattens; `runner.rs` precomputes `readout_edges` once per network.

---

## 5. Invariants & Guardrails Status

- **GC1 (No Production Autograd):** 0 hits in BINN workspace crates.
- **GC2 (No External ML Frameworks):** PASS (`Cargo.toml` and `Cargo.lock` untouched).
- **GC3 (Bit Determinism) & Dual-Profile Kernel Gate (`check_kernel_profiles.sh`):** PASS — 100% bit-exact hashes across debug/release and pre/post binaries on all 6 scientific harnesses.
- **GC4 (Fixed Input Encoders):** PASS.
- **GC5 (Benchmark Coverage):** PASS — all Criterion suites plus `wave4_shd_training` compile and execute.
- **GC6 (Zero Undocumented Unsafe):** PASS — 0 new `unsafe` in BINN crates (`#![deny(unsafe_code)]` preserved); `sparsl` library code outside `backend/metal.rs` remains `#![forbid(unsafe_code)]`.
- **GC7 (Activity Sparsity Logging):** PASS.
- **Metal Validation (`MTL_SHADER_VALIDATION=1 MTL_DEBUG_LAYER=1`):** PASS across both `sparsl` (122 + 23 + 12 + 4 tests) and `binn-core` (46 + 1 shared-event interop tests).
- **Strict Weak Checks (`find_weak_checks.py --strict`):** 39 findings (100% baseline, 0 new weak assertions).
