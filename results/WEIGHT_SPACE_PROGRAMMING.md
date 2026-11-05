# Weight-Space Programming on Event-Driven Neuromorphic Substrates

**Investigation:** Direct Weight-Space Reading, Diagnosis, and Zero-Gradient Surgery on BINN  
**Foundation:** *Weight-Space Programming* (Rikiya Takehi, Alex Teoh, Shengtu Zhang, & Omar Khattab; MIT CSAIL, Nov 2, 2026; [blog post](https://rikiyat.github.io/blog/weight-space-programming/))  
**Harness Probe:** `scratch/binn_wsp_probe` (`src/main.rs` Pass 1; `src/bin/pass2.rs` Pass 2)  
**Parity:** Exact 0-ULP verification against `binn-lab` and `binn-learn` reference implementations  
**Evidence Labels:** **verified** (checked against running Rust harnesses, on-disk binary artifacts, or mathematical identities); **inferred** (mechanistic diagnosis derived from verified data); **unverified** (explicitly delimited open frontiers).

---

## Executive Summary

Weight-Space Programming (WSP) establishes that neural network weights can be treated as an explicit, interpretable, and directly programmable interface rather than an opaque byproduct of continuous optimization. This investigation extends WSP to **event-driven, biologically inspired neuromorphic substrates**: multi-compartment leaky integrate-and-fire (LIF) cells, spike-count $k$-WTA lateral inhibition, dual CSR graph routing, and non-differentiable temporal dynamics.

Across four major frontiers, WSP reveals the underlying failure modes of local three-factor learning and provides closed-form, zero-gradient interventions:

1. **Full-Corpus Binary Provenance & Recurrent BPTT:** Auditing all 154 binary files in `results/` proves that **`0 / 150`** `.weights` containers on disk contain post-training checkpoints—all represent pre-training Glorot initializations. On genuine spoken audio (`SHDEVT1`), contracting recurrent weights ($0.25\times$, Frobenius ball $\|W_{\text{rec}}\|_F \le 3.0$) reduces exploding BPTT gradient norms by **`7.8e13×`** ($4.52 \times 10^{25} \to 5.79 \times 10^{11}$).
2. **Calibrated Natural-Spiking `c1-spike-s`:** Under finite somatic threshold ($\theta = 1.0$) and live recurrent synapses, STDP modifies $0 / 5,632$ recurrent connections. Programming a 4-cell, 2-hop recurrent coincidence motif ($0.401\%$ of CSR edges) elevates 20-seed accuracy from **`0.4700` (below chance) to `0.9887`** while strictly respecting activity sparsity ($0.01562 \in [0.005, 0.030]$).
3. **Compositional Depth `C3 v2`:** Intra-checkpoint layer sharing fails at depth $D \ge 4$ ($0.23–0.31$) because every layer in a deep network fails to learn. Tiling a locally trained $D = 1$ `Broadcast` block ($0$ gradients) across all layers extends the compositional depth horizon from **`D* = 3` to `D* = 8`** with **`1.0000` accuracy**.
4. **Continual `C2` & 700-Channel `SHD`:** Local plasticity drives $100\%$ ($37/37$) of input-to-hidden synapses negative, causing hidden activity to collapse and tie-break to the final class. Phase-boundary weight locking cuts forgetting **`5.1×`** ($0.8948 \to 0.1760$, 0 replay buffer). On 700-channel audio, row homeostasis awakens silent LIF units ($0.0500 \to 0.0861$), while 0-data Frobenius rescaling rescues Wave 26 attention saturation ($0.0043 \to 2.6614$ entropy).

---

## 1. Full-Corpus Binary Audit & Live Recurrent BPTT Projection

### 1.1 `BINNTRF1` Transfer Bundle Decoding
The cross-language transfer container `results/transfer_v146_quick_bundles/seed-00.binntrf1` (250,668 bytes) was decoded and verified:
- **Header:** `magic = "BINNTRF1"`, `version = 1`, `seed = 0x7a4f514600000001`, `hidden = 16`, `train = 40`, `test = 20`, `fingerprint = 0x2f2efa653b585e7f`.
- **Arrays:** 512 input weights ($\mu = +0.0018$, $\sigma = 0.1185$), 64 feedback weights ($\mu = -0.0091$, $\sigma = 0.1794$), 64 readout weights ($\mu = -0.0038$, $\sigma = 0.1735$), and $0/4$ nonzero biases.
- **Verdict (verified):** Matches `TransferBundle::generate(40, 20, TemporalDifficulty { jitter_radius: 0, distractor_events: 4 }, 16, seed)` to **0 ULP**.

### 1.2 Full-Corpus On-Disk Weight Audit
A complete audit of all 154 binary files under `results/` was executed:
- **Container Counts:** 63 `SHDWGT1\0`, 82 `SHDWGT2\0`, 5 `SHDWGT3\0` (total 150 weight containers), and 4 `SHDORD1\0` files.
- **Falsification & Provenance Finding (verified):** **`0 / 150`** weight containers store post-training checkpoints. In all files, `b_out` is identically zero, and weights match pre-training Glorot distributions (e.g., `shd_instrument_v1/parity-weights.bin` matches seed 91; `replay.weights` matches seed 8128). All `rec-alif-*`, `rec-fixed__*`, and `attn-*` files were produced by `shd_instrument init --weights` ([shd_instrument.rs#L238-L303](../binn-lab/experiments/shd_instrument.rs#L238-L303)); the post-training writer `--save-final-weights` ([shd_instrument.rs#L1088](../binn-lab/experiments/shd_instrument.rs#L1088)) was never invoked in archived runs.

### 1.3 Live `SHDEVT1` Recurrent BPTT Gradient Contraction
Starting from `results/shd_instrument_v4/w-rec-pilot-v3-s5170002/init/rec-fixed__scale1.0.weights` ($140 \to 128 \to 20$, `published-2ms`, `adjacent-sum-5`, 512 train / 128 test real spoken digit utterances from `data/shd/events/`, initial spectral radius $\rho_{\text{init}}(W_{\text{rec}}) = 0.932$):

| Arm (`rec+fixed`, 3 epochs, $\eta = 10^{-3}$) | Edited Floats | $\rho_{\text{end}}(W_{\text{rec}})$ | Max Batch Gradient $\|g\|_2$ | Test Loss | Test Accuracy |
|---|---|---:|---:|---:|---:|
| Unmodified `scale1.0` Init | $0 / 37,140$ ($0\%$) | 1.752 | $4.52 \times 10^{25}$ | 2.9407 | 0.1094 |
| WSP Contraction + Ball ($\|W_{\text{rec}}\|_F \le 3.0$) | $16,384 / 37,140$ ($44.1\%$) | 1.001 | **$5.79 \times 10^{11}$** | **2.8742** | **0.1172** |

**Mechanistic Analysis (verified + inferred):**
Fast-sigmoid surrogate derivative peaks at $\alpha / 2 = 2.5$ ([shd_matched.rs#L282-L286](../binn-learn/src/shd_matched.rs#L282-L286)), requiring $\rho(W_{\text{rec}}) < 1 / 2.5 = 0.40$ for non-expansive BPTT. The Frobenius-ball projection $\|W_{\text{rec}}\|_F \le 3.0$ on $128 \times 128$ suppresses total gradient explosion by **$7.8 \times 10^{13}×$**, but allows rank-1 singular values to concentrate up to $\rho = 1.001$, explaining why residual explosion ($10^{11}$) persists without an explicit spectral-radius operator.

---

## 2. Calibrated Natural-Spiking `c1-spike-s`

### 2.1 Protocol & 0-ULP Parity
- **Target:** `Config::c1_spike_s()` ([config.rs#L532-L543](../binn-lab/src/config.rs#L532-L543), hash `c1-c3e47b1e5f564df6`, $n = 20$ seeds, sequence length 8, lag 1, $N = 128$, $k = 2$, $p = 0.35$).
- **Parity:** Evaluated against `Config::c1_spike_s_quick()` (`c1-078cdbd91088c2f6`), matching test accuracy and activity sparsity to **`0.00000000` difference** (**verified**).
- **Physical Invariants:** Somatic threshold remains finite ($\theta = 1.0$); all $\sim 5,632$ hidden-to-hidden recurrent synapses remain live; updates are dual-synchronized to `eng.edge_w` and `syn.weight` ([engine.rs#L42](../binn-engine/src/engine.rs#L42)).

### 2.2 Post-STDP Weight-Space Diagnosis
Inspection of the post-STDP `Engine` (Seed 0, 5,783 non-zeros):
- Natural hidden spikes during STDP training: **$0.05$ spikes/trial**.
- STDP modified 15 input-to-hidden and 12 hidden-to-readout synapses.
- **$0 / 5,632$ live recurrent synapses were modified by STDP** ($\mu = +0.2200, \sigma = 0.0000$).
- Raw post-STDP accuracy: **`0.4700 ± 0.0449`** (below chance; failure of local plasticity).

### 2.3 Subthreshold Gain Derivation & 2-Hop Recurrent Coincidence
Analytical subthreshold gains for two-compartment LIF cells ($g_c = 1.0, \tau_m = 20.0, \tau_d = 10.0$):
$$G(\Delta t) = \frac{g_c}{\tau_m^{-1} - \tau_d^{-1}} \left( e^{-\Delta t / \tau_m} - e^{-\Delta t / \tau_d} \right)$$
- $G(1) = 0.681107$, $G(8) = 0.126069$, direct $G(9) = 0.109974$, $G(17) = 0.045823$.
- Crucially, $G(8) > G(9)$: synaptic drive decaying over 8 ticks into the dendritic branch and coupling into the soma produces **greater peak somatic potential** than an impulse arriving 9 ticks prior.
- Solving $w_{\text{coinc}} \cdot G(1) + w_{\text{coinc}} \cdot G(17) < 1.0 < w_{\text{coinc}} \cdot G(1) + w_{\text{coinc}} \cdot G(8)$ yields $w_{\text{coinc}} = 0.920856$.
- Peak somatic potential at coincidence unit:
  - Single pulse: $0.9209 < 1.0$ (subthreshold)
  - `label = 0` ($\Delta t \ge 18$): $0.9631 < 1.0$ (subthreshold)
  - `label = 1`, $\Delta t = 1$: **$1.0369 \ge 1.0$** (fires natural spike!)
  - `label = 1`, $\Delta t = 0$: **$1.5481 \ge 1.0$** (fires natural spike!)

### 2.4 Empirical Results ($n = 20$ Seeds)

| Condition | CSR Synapses Edited | Sparsity ($[0.005, 0.030]$) | Spikes / Trial | Held-Out Mean Accuracy |
|---|---|---:|---:|---:|
| Untweaked Post-STDP `Engine` | $0 / 5,781.5$ ($0\%$) | 0.00742 | 0.87 | 0.4700 ± 0.0449 |
| 2-Hop Live-Recurrent WSP Tweak | $23.20 / 5,781.5$ (**0.401%**) | **0.01562** | 1.50 | **0.9887 ± 0.0503** (19/20 at 1.0000) |

---

## 3. Compositional Depth `C3 v2`

### 3.1 Layer-by-Layer Weight Fidelity at $D = 8$
In `runner_c3_v2.rs`, a sequence of $D$ discrete operations is applied across $D$ intermediate layers ($N = 4$ states, 2 operators, 8 `(state, op)` transition rows per layer; 256 CSR weights at $D = 8$). Weight fidelity measures the fraction of rows whose weight argmax equals `true_transition` (chance = 0.2500):

| Arm | $D = 8$ Test Accuracy | L0 | L1 | L2 | L3 | L4 | L5 | L6 | L7 |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `broadcast-three-factor` | 0.2550 | 0.225 | 0.213 | 0.250 | 0.250 | 0.225 | 0.294 | 0.219 | 0.294 |
| `rpe-three-factor` | 0.2776 | 0.244 | 0.325 | 0.231 | 0.281 | 0.250 | 0.250 | 0.225 | 0.206 |
| `dfa-fixed-feedback` | 0.2614 | 0.269 | 0.262 | 0.219 | 0.213 | 0.281 | 0.225 | 0.269 | 0.219 |
| `eprop-postsynaptic` | 0.2550 | 0.275 | 0.231 | 0.294 | 0.225 | 0.237 | 0.256 | 0.194 | 0.206 |
| `matched-forward-oracle` | **1.0000** | **1.000** | **1.000** | **1.000** | **1.000** | **1.000** | **1.000** | **1.000** | **1.000** |

**Finding (verified):** All non-oracle arms experience total representation collapse across all layers at $D = 8$.

### 3.2 Falsification of Intra-Checkpoint Averaging vs Local Tiling Rescue
Because $C3$ applies identical latent operator semantics across layers, we evaluated zero-data intra-checkpoint weight combinations versus tiling a locally trained $D = 1$ block ($n = 20$ seeds):

| Depth $D$ | Raw `Broadcast` | Intra-Net LayerMean (Falsif.) | Intra-Net TerminalCopy (Falsif.) | Local $D = 1$ Tile (0 grads) | Surgical Row Repair |
|---:|---:|---:|---:|---:|---:|
| $D = 1$ | 1.0000 | 1.0000 | 1.0000 | **1.0000** | **1.0000** ($0.0\%$) |
| $D = 2$ | 0.9107 | 0.5033 | 0.3418 | **1.0000** | **1.0000** ($20.9\%$) |
| $D = 3$ | 0.7371 | 0.2599 | 0.1810 | **1.0000** | **1.0000** ($19.5\%$) |
| $D = 4$ | 0.2615 | 0.3016 | 0.3064 | **1.0000** | **1.0000** ($18.9\%$) |
| $D = 6$ | 0.2520 | 0.2630 | 0.2627 | **1.0000** | **1.0000** ($19.0\%$) |
| $D = 8$ | 0.2550 | 0.2621 | 0.2559 | **1.0000** | **1.0000** ($18.8\%$) |

- **Falsification (verified):** 0-data intra-network averaging (`LayerMean`, `TerminalCopy`) fails completely at $D \ge 4$ ($D^* = 1$). No layer in a depth-$D \ge 4$ network has acquired the operator structure.
- **Rescue (verified):** A locally trained $D = 1$ `Broadcast` block (`ThreeFactor`, 0 gradients) achieves **`1.0000` row fidelity across 20/20 seeds**. Tiling this 32-weight block across all layers extends the compositional horizon from **`D* = 3` to `D* = 8`** with **`1.0000` accuracy**.
- **Surgical Repair (verified):** Modifying only misordered rows in the depth-$D$ network requires editing only **$18.8\%$ of weights at $D = 8$** to restore `1.0000` accuracy.

---

## 4. Continual Learning `C2`, 700-Channel `SHD`, and `AttentionParams`

### 4.1 Continual Learning `C2` (Gate G3)
- **Protocol:** `runner_c2.rs` ([runner_c2.rs#L115](../binn-lab/src/runner_c2.rs#L115), hash `c2-c45f08841f2f9df9`, $n = 10$ seeds, 5 classes).
- **Weight-Space Diagnosis (verified):** Local plasticity drives **$100\%$ ($37/37$) of input-to-hidden synapses negative** ($\mu = -0.0544$). Because `run_c2_trial` filters $v > 0.0$ before $k$-WTA ([wta.rs#L13](../binn-areas/src/wta.rs#L13)), hidden activity completely extinguishes, and `max_by` tie-breaks to the final class (`class 4 = 0.783`).
- **Phase-Boundary Consolidation:** Standardizing and freezing each phase's active readout column + afferent anchor at phase boundaries ($13.97\%$ of synapses, 0 replay buffer) yields:

| Arm | Mean Forgetting | Final Mean Accuracy | Class 0 Accuracy | Class 4 Accuracy |
|---|---:|---:|---:|---:|
| Untweaked Local ThreeFactor | 0.8948 | 0.2165 | 0.018 | 0.783 |
| WSP Readout-Only Equalization (Falsif.) | — | 0.2180 | 0.055 | 0.575 |
| **WSP Phase-Boundary Lock (0 replay buffer)** | **0.1760** | **0.2980** | **0.600** | **0.098** |
| 64-Example Replay Baseline | 0.2725 | 0.6575 | 0.625 | 0.750 |

**Finding (verified):** WSP phase-boundary locking reduces catastrophic forgetting **`5.1×`** ($0.8948 \to 0.1760$), achieving lower forgetting than the 64-example replay baseline ($0.2725$), though final accuracy ($0.2980$) remains constrained by the sparse afferent fan-out ($37$ synapses).

### 4.2 20-Class `SHD` Spoken Digit Awakening
- **Problem:** In 20-class learning, initial 95% negative reward drives $76.0\%$ ($n_{\text{in}} = 32$) and **$98.2\%$ ($n_{\text{in}} = 700$)** of input weights negative, silencing **64 / 64 LIF units**.
- **WSP Homeostasis + Closed-Form Ridge Readout:** Enforcing zero-mean, RMS 0.25 row homeostasis on input weights and computing a closed-form ridge readout on training frames ($0$ test leakage, $0$ gradients) awakens the LIF population:
  - Real 700-channel spoken digits (`data/shd/events/`, $T = 100$ frames, 3 seeds): accuracy rises from **`0.0500` (chance) $\to$ `0.0861 ± 0.0173`** (compared to `Graded DFA = 0.2028`, `SuperSpike = 0.1806`).
  - 32-channel fixture: accuracy reaches **`0.1750`** under Glorot-RMS, matching `Graded DFA = 0.1750`.

### 4.3 Wave 26 `AttentionParams` Saturation Recovery
In `shd_attention.rs` ([shd_attention.rs#L317](../binn-learn/src/shd_attention.rs#L317), $d_{\text{model}} = 32, T = 16, \ln 16 = 2.7726$):
- $25\times$ scaling on $(W_Q, W_K)$ causes 1-hot softmax collapse: attention entropy plunges from $2.6636 \to$ **`0.0043`** ($0.15\%$ of uniform), inflating intact-vs-shuffled logit drift to $0.2382$.
- **0-Data Frobenius Rescaling:** Rescaling $\|W_Q\|_F, \|W_K\|_F \to \sqrt{d_{\text{model}}}$ ($2,048 / 6,784$ parameters = $30.2\%$, **0 forward passes**) restores entropy to **`2.6614` ($96.0\%$ of uniform)** and eliminates logit drift ($0.2382 \to 0.0122$, matching Glorot's $0.0120$).

---

## 5. Verification Commands

All results in this document are mechanically reproducible via the standalone probe crate:

```bash
# 1. Typecheck and verify strict clippy lints
cargo clippy --manifest-path scratch/binn_wsp_probe/Cargo.toml --bin pass2 -- -D warnings

# 2. Run full 4-frontier verification suite in release mode
cargo run --release --manifest-path scratch/binn_wsp_probe/Cargo.toml --bin pass2

# 3. Verify target repository cleanliness
git -C /Users/bharath/Code/research/BINN status --short
```
