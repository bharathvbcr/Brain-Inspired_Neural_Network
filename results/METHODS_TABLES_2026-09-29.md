# Methods tables: SHD attention read-out (2026-09-29)

Scope: the SHD attention read-out experiments in `results/PAPER_DRAFT.md` §2.3 / §3.5 / Appendix F.
Anchor = `ff+fixed+attn` vs `ff+fixed`, h128, `published-2ms`, `adjacent-sum-5`, attention d32/L4, e400.

**How to read the `source` column.** `file:line` points at code read at HEAD `1b10f92`. `cell:<field>` is a field
of the archived `shd-cal-cell-v1` records. Counts over cells were computed by a script that reads all five
corpora the paper cites (§ "Attention-campaign cells are preserved in five corpora"):
`shd_attention_campaign_v1/cells`, `shd_attention_campaign_v2`, `azure-d32l4-scope-v1/results`,
`shd_attention_campaign_v3`, `shd_attention_wave29_local`. **Status**: V = verified from code or cells;
I = inferred (derived arithmetic, or code read at HEAD applied to cells produced by an earlier pinned binary).

**Caveat on every code row.** The cells were produced by pinned binaries (`22d97c51…` on the AWS fleet per
`shd_attention_campaign_v1/manifest.json` and `v2/manifest.json`; `666a7342…` on Azure per
`azure-d32l4-scope-v1/hosts/node-*.json`; `fec6c404…` for wave 29 per
`AMENDMENT_2026-09-07_WAVE_29_RUNS_ON_THE_LOCAL_PLATFORM.md` §3). The code rows below describe HEAD. The
code comments claim that later changes are bit-identical on recorded cells ("Gate F"). I did not re-run
Gate F, so a code row applies to the archived cells only by inference, except where a cell field records
the value directly.

## Table 1: Hyperparameters and procedure

| Item | Value | Source | Status |
|---|---|---|---|
| Dataset | Spiking Heidelberg Digits, 20 classes, official train/test split | `run_cell.py:31` (`--classes 20`); `cell:n_train`, `cell:n_test` | V |
| Train / test sizes | 8156 / 2264 in 3910 of 3934 cells. The other 24 cells hold out speakers 3 and 6 for validation: 6987 train, 1169 val, 2264 test | `cell:n_train`, `cell:n_test`, `cell:n_val` (v3 only) | V |
| Validation | None for any headline or anchor cell: `val_accuracy` is absent from every v1, v2, Azure and wave-29 cell. Only 24 v3 cells carry `val_accuracy`, and they use a speaker hold-out (speakers 3 and 6). Reported accuracy is test accuracy after the **final** epoch, with no early stopping or selection | `cell:val_accuracy`, `cell:val_speakers="3,6"`; `shd_instrument.rs:1098` (evaluate after last epoch), `:1103-1107`; `partition_by_speaker` `:1263` | V |
| Input encoding | Raw events from `SHDEVT1` cache. Integer **event counts** per (bin, channel) are summed, not binarised | `binn-data/src/shd_contract.rs:159-221` (`:201` `+= 1.0`) | V |
| Channels (geometry) | `adjacent-sum-5`: 700 channels → 140 inputs (`channel / 5`). `channels-700`: 700 inputs | `shd_contract.rs:66-82`; `cell:geometry` | V |
| Time binning, `published-Nms` | dt = N ms, origin = first event, T = ⌊(t_last − t_first)/dt⌋ + 1, **variable per sample**. N ∈ {2, 4, 10}; the corpora use 2 and 10 | `shd_contract.rs:24-29, 166-176` | V |
| Time binning, `fixed-tN` | 1400 ms window from t = 0, T = N ∈ {100, 250, 500}, dt = 14 / 5.6 / 2.8 ms; events past the window are clipped (0 clipped in the data) | `shd_contract.rs:14, 31-39, 177-180`; `shd_instrument_v4/data_summary.json` (`clipped_events: 0`) | V |
| T at `published-2ms` (anchor) | train: min 117, median 353, mean 358.3, max 685. test: min 158, median 350, mean 356.5, max 585 | recomputed from local `data/shd/events/{train,test}.events` with the `shd_contract.rs:166-176` formula | I (local cache; its hash was not checked against the fleet's copy) |
| T at `published-10ms` | train 24–137 (mean 72.1). test 32–117 (mean 71.7) | same computation | I |
| Utterance duration | train mean 0.7156 s (0.2334–1.3691 s). test mean 0.7119 s | `shd_instrument_v4/data_summary.json` | V (archived summary) |
| Neuron model | LIF: u_t = α·u_{t−1}·(1 − s_{t−1}) + Σ_c W_in[h,c]·count_t,c. s_t = H(u_t − θ) | `shd_matched_arms.rs:762-813` | V |
| Membrane retention α | exp(−dt/τ_m) with τ_m = 10.05 ms. α = 0.8195 at `published-2ms`. Other contracts: 10 ms → 0.3697; t100 → 0.2483; t250 → 0.5728; t500 → 0.7568 | `shd_matched_arms.rs:674`; `shd_matched.rs:16`; `shd_instrument.rs:700-704`; `cell:tau_m` = 10.05 in every cell that records it (v1, v2 and Azure predate the field) | V (formula, τ); I (numeric α) |
| τ_m ladder | 2.5 / 5 / 20 / 40 ms, 24 cells each, v3 only (an Appendix F probe, not the anchor) | `cell:tau_m` | V |
| Threshold θ | 1.0 (fixed arms) | `shd_matched.rs:14`; `shd_matched_arms.rs:800-805` | V |
| Reset | Multiplicative hard reset to 0 (`(1 − s)`). Reset is **detached** in the backward pass: du carries α(1 − s)·du_next with no gradient through s | `shd_matched_arms.rs:763, 969` | V |
| ALIF (`ff+alif`, `rec+alif` only) | θ_t = 1 + β_a·a_t, a_t = ρ·a_{t−1} + s_{t−1}, ρ = exp(−1/τ_a) **per step** (not per ms), τ_a = 20, β_a = 0.18 | `shd_matched_arms.rs:54-55, 675-676, 757-761, 800-801` | V |
| Surrogate gradient | Arctan-derivative (Lorentzian): σ′(x) = (α_s/2)/(1 + (π/2·α_s·x)²), x = u − θ, α_s = 5.0 × `surrogate_scale`. Peak 2.5 at scale 1.0, unit area | `shd_matched.rs:15, 290-294`; area test `:311-346` | V |
| Surrogate scale | 1.0 in 3708 of 3934 cells, including every anchor cell. 0.4 for 202 v2 cells. 0.5 and 0.25 for 12 v2 cells each (h1024 levers) | `cell:surrogate_scale` | V |
| Input scale | None: counts enter W_in unscaled. `MATCHED_INPUT_SCALE` is used only by the matched-arch configs, not by this instrument | `shd_matched_arms.rs:765-770`; `binn-lab/src/match_config.rs:191` (only use sites) | V |
| Hidden widths | 128 (anchor), 192, 256, 384, 512, 768, 1024 | `cell:hidden` (2481 / 12 / 239 / 96 / 238 / 204 / 664 cells) | V |
| Hidden layers | 1 | `shd_matched.rs:1` ("one-hidden-layer") | V |
| Recurrence | `ff` = none. `rec` = W_rec [h×h] on s_{t−1}, diagonal held at 0. Arms in corpus: ff+fixed 1486, ff+fixed+attn 2261, ff+alif 12, ff+alif+attn 12, rec+fixed 22, rec+alif 72, rec+alif+attn 69 | `shd_matched_arms.rs:771-798, 280-288`; `cell:arm` | V |
| Rate read-out | r_h = (1/T)·Σ_t s_t,h (unweighted mean spike count). logits = b_out + W_out·r | `shd_matched_arms.rs:816-850` | V |
| Attention block: input | Spikes (not membranes): z_0(t) = W_e·s(t) + pos(t), W_e: [hidden → d] | `shd_attention.rs:50, 710-749` | V |
| Attention d, L | Anchor d = 32, L = 4. Corpus (d, L): (32,4) 1362, (32,1) 508, (32,2) 296, (32,3) 68, (64,4) 72, (64,1) 12, (16,1) 12, (128,1) 12 | `cell:attn_dim`, `cell:attn_layers` | V |
| Heads | 1 (one W_q, W_k, W_v, W_o of shape [d×d] per block) | `shd_attention.rs:276-284, 761-771` | V |
| Score | softmax_row(q·kᵀ/√d) over all T×T pairs | `shd_attention.rs:704-708, 782-812` | V |
| Causal | **No**: full T×T row-softmax, no mask | `shd_attention.rs:784-812` | V |
| Residual / norm / MLP | Residual z ← z + W_o(A·v). No LayerNorm, no feed-forward sublayer, no biases in the block | `shd_attention.rs:814-821, 276-326` | V |
| QK-norm variant | L2-normalised q, k, score × √d. 24 v3 cells only (`attn_readout = qk-norm`) | `shd_attention.rs:191-213, 776-780`; `cell:attn_readout` | V |
| Positional code | Fixed (not learned) sinusoidal over normalised position u = t/(T−1). d/2 sin/cos pairs at geometrically spaced 1…64 cycles per utterance. Added to z_0. `no-position` variant (24 v3 cells) zeroes it | `shd_attention.rs:101-108, 531-554, 728-734` | V |
| Pooling → logits | pooled = mean_t z_L(t). logits += W_a·pooled (W_a: [20 × d]). **Additive** on top of the rate read-out | `shd_attention.rs:55-56, 607-616, 834-841`; `shd_matched_arms.rs:851-858` | V |
| Dropout | None in any model component. `spike-dropout-pN` is an input manipulation, not regularisation (see operators) | no dropout in `shd_attention.rs` / `shd_matched_arms.rs` forward | V |
| Attention parameter count (h128, d32, L4) | hidden·d + L·4d² + 20·d = 21,120 | `shd_attention.rs:383-386` | I (arithmetic) |
| Loss | Softmax cross-entropy on the summed logits, p clamped at 1e−30. Per-sample losses averaged over the batch (gradient scaled 1/|batch|) | `shd_matched_arms.rs:859-862`; `shd_instrument.rs:971` | V |
| Training | Full BPTT through T with the surrogate. The same trainer is used for every arm (not the local rule of the paper's other sections) | `shd_matched_arms.rs:886-1005` | V |
| Optimizer | Adam, β1 = 0.9, β2 = 0.999, ε = 1e−8, bias-corrected | `shd_matched.rs:17-19, 464-540`; `shd_matched_arms.rs:1087-1142` | V |
| Weight decay | 1e−5, **coupled L2** (added to the gradient before the Adam moments; not AdamW). Applied to W_in, W_out, W_rec and all attention matrices; **not** to b_out. A code comment says "decoupled", but the arithmetic is coupled | `shd_instrument.rs:1012` (`1e-5`); `shd_matched.rs:506, 530`; `shd_matched_arms.rs:1107, 1118-1125` | V |
| LR schedule | One-cycle, stepped per optimizer step: linear 1e−3 → 5e−3 over the first 30% of steps, then linear 5e−3 → 1e−5 (base/100) at the end | `shd_instrument.rs:1011`; `shd_matched.rs:542-553` | V |
| Batch size | 256 (the last batch per epoch has 220 samples at n_train = 8156). 32 steps/epoch, 12,800 steps at e400, 3,200 at e100 | `shd_instrument.rs:851-852` | V (size); I (step counts) |
| Epochs | e400 anchor. Corpus: 400 (2639 cells), 100 (1107), 200 (68), 800/50/20/10/5 (24 each, v1 sample-efficiency ladder) | `cell:epochs`; `cell:epoch_mean_loss` has length = epochs | V |
| Epoch order | Fresh shuffle each epoch from PortableRng(seed ⊕ 0x0D3E45E551D00001). Fixed per seed, shared across arms at the same seed | `shd_instrument.rs:304-311` | V |
| Init: W_in, W_out | Glorot-uniform, limits √(6/(n_in + h)) and √(6/(h + 20)), from PortableRng(seed) (xorshift64*) | `shd_matched.rs:104-122, 555-583`; `shd_instrument.rs:247` | V |
| Init: b_out | 0 | `shd_matched.rs:120` | V |
| Init: W_rec | Glorot-uniform √(6/2h) from PortableRng(seed ⊕ 0x5245430000000001) × `--w-rec-scale` (the runner never passes it → 1.0), diagonal zeroed | `shd_instrument.rs:267-281`; `run_cell.py:29-43`; `shd_matched_arms.rs:258` | V |
| Init: attention | W_e, W_q, W_k, W_v, W_a Glorot-uniform, each matrix on its own stream keyed by (seed ⊕ 0x4154544E…, layer). **W_o = 0**, so every block starts as an identity residual. Base weights are bit-identical to the non-attention arm at the same seed | `shd_attention.rs:85-92, 328-381`; `shd_instrument.rs:282-298` | V |
| Gradient clipping | Off for every anchor cell (`clip_grad_norm = null`). Batch global-norm 1000.0 on 36 v2 cells only (h1024 lever). Per-sample clipping never used (`clip_sample_grad_norm = null` wherever recorded) | `cell:clip_grad_norm`, `cell:clip_sample_grad_norm`; `shd_instrument.rs:1001-1010` | V |
| Seeds | Model seeds 5170001–5170012 (n = 12) in v1, Azure and v3. v2 extends to 5170001–5170032 (n up to 32). Wave 29 uses a fresh block, 5290001–5290012. One seed drives weight init and epoch order. The `seed` field is present in 2450 cells; the other 1484 (v1, Azure, part of v2) record it only in the filename | cell filenames `__sNNNN`; `cell:seed`; `run_cell.py:31, 61`; `shd_instrument.rs:618-627` | V |
| Manipulation seed | `temporal_seed = 5170001` for **every** manipulated cell in every archived plan. The permutation or mask realisation is therefore the same for all model seeds; train, test and val streams are disjoint (seed ⊕ 0x1111…/0x2222…/0x3333… ⊕ sample index) | `scripts/aws/plan_cells.py:79`; all 14 archived `plan_*.json` + `v1/plan.json`; `shd_instrument.rs:767-800` | V (archived plans); plans for w16–w23 are not archived |
| Operator: `bin-shuffled` | One permutation of the T bins per sample, shared across channels (order destroyed; within-bin synchrony and counts kept). Applied **once before epoch 1**, so it is fixed across epochs | `shd_temporal.rs:694-701, 649-667`; `shd_instrument.rs:767-777` | V |
| Operator: `channel-shuffled` | Independent bin permutation per input channel (order and cross-channel synchrony destroyed; per-channel counts kept). Fixed across epochs | `shd_temporal.rs:725-747` | V |
| Operator: `reversed` | Bin order reversed | `shd_temporal.rs:685-693` | V |
| Operator: `window-shuffled-wN` | Bins permuted within disjoint N-bin windows (last window short). N ∈ {2, 4, 8, 16, 32, 64, 128, 192, 256} in v3 | `shd_temporal.rs:702-724`; `cell:temporal_condition` | V |
| Operator: `spike-dropout-pN` | Each unit of count deleted with probability N%, from a mask frozen before epoch 1. N ∈ {30, 50, 60, 70, 80, 90} | `shd_temporal.rs:762-794, 649-667` | V |
| Operator: `hidden-shuffled` | Per-sample fixed permutation of the **hidden** spike train's time axis, applied only to the attention read-out's input. The substrate and the rate read-out are untouched (rate read-out reads the unpermuted buffer) | `shd_temporal.rs:748-761`; `shd_matched.rs:38-55`; `shd_attention.rs:638-659, 735` | V |
| Operator scope | Train and test are both manipulated (and val when present) | `shd_instrument.rs:767-800` | V |
| Precision | f32 for parameters, forward, backward and Adam. f64 for diagnostics (norm fallback, update RMS, loss sums) | `shd_matched.rs:227-249, 474`; `shd_instrument.rs:933` | V |
| Pass predicate (`CELL_PASS`) | acc ≥ 0.80 ∧ all 20 classes predicted ∧ majority < 0.30 ∧ silent ≤ 0.95 ∧ saturated ≤ 0.05 ∧ no non-finite events | `shd_instrument.rs:1108-1117` | V |

## Table 2: Compute

| Item | Value | Source | Status |
|---|---|---|---|
| Implementation | Rust (`shd-instrument` binary). CPU only: rayon data-parallel per-sample gradients, reduced in fixed order. No GPU or Metal path is imported by the instrument | `shd_instrument.rs:9-27, 922-931`; `binn-lab/Cargo.toml:166-168`; `cell:backend = "rust"` (all 3934) | V |
| Build | `--release`, `lto = "fat"`, `codegen-units = 1` | `Cargo.toml:64-66` | V (HEAD) |
| AWS fleet (v1, v2, v3) | Linux aarch64, Amazon Linux 2023 (`6.18.41-94.142.amzn2023.aarch64`), c7g spot instances, glibc. v1: 6 instances | `shd_attention_campaign_v1/manifest.json:backend`; `v1/gates/i-*.json:uname` (11 records); `v2/manifest.json:instances` (w8–w9 only) | V for v1 and w8–w9; not recorded per cell |
| Azure (`azure-d32l4-scope-v1`) | 4 × Standard_D64als_v7 (AMD EPYC 9V45, 64 vCPU), Linux 6.8 x86_64, glibc 2.35, target-cpu x86-64-v4. 4 threads/cell (8 for h ≥ 1024 with d ≥ 64), ≤ 16 cells/node | `azure-d32l4-scope-v1/input/manifest.json:infrastructure`; `hosts/node-*.json` | V |
| Wave 29 | Apple M5 Pro, 18 cores, `aarch64-apple-darwin`, Apple libm, rustc 1.98.0, 4 threads/cell | `AMENDMENT_2026-09-07_WAVE_29_RUNS_ON_THE_LOCAL_PLATFORM.md` §2–3 | V (amendment document; not a cell field) |
| libm | glibc on the fleet and Azure. Apple libm on wave 29, **not bit-identical** to glibc | same amendment §3; `v2/manifest.json:cross_machine_gate_f` | V (as recorded) |
| Cell counts | v1 528; v2 2651; Azure 95 (of 252 planned); v3 612; wave 29 48. **Total 3934**, no filename appears in two corpora | `cell:schema = shd-cal-cell-v1`, counted per directory | V |
| Cell-hours (Σ `wall_secs`) | v1 585.6; v2 7892.3; Azure 744.6; v3 3470.6; wave 29 223.2. **Total 12,916.3 cell-hours** | `cell:wall_secs` | V |
| Anchor wall time, e400, median [min–max] | attention vs rate, seconds (hours in parentheses): v1 20,746 (5.76 h) [20,322–20,806], n = 12 vs 501 (0.14 h) [296–542], n = 24. v2 11,731 (3.26 h) [11,631–39,368], n = 32 vs 213 (0.06 h) [206–524], n = 32. v3 39,024 (10.84 h) [38,765–39,121], n = 24 vs 505 (0.14 h) [493–513], n = 24. Wave 29 (M5 Pro) 32,336 (8.98 h) [29,969–35,822], n = 12 vs 538 (0.15 h) [443–799], n = 12 | `cell:wall_secs`. Filter: arm, h128, `published-2ms`, `adjacent-sum-5`, e400, intact, scale 1.0, no clip, τ_m 10.05, d32/L4/default, no val split | V |
| Anchor wall time, e100 | attention vs rate: v1 7,474 s vs 79 s (n = 12 each). v2 9,784 s vs 135 s (n = 12 each) | `cell:wall_secs` | V |
| Ratio attention / rate at anchor | ≈ 41× (v1), 55× (v2), 77× (v3), 60× (wave 29) | derived from the row above | I |

**Wall-time caveat.** Wall time depends on threads per cell and on host contention, and cells do not record
either. The spread across corpora (3.3–10.8 h for the same attention anchor on the same fleet
architecture) is therefore a scheduling effect as much as a cost. Quote a range, not one number.

## Not found / not verifiable

- **Per-cell host, platform, libm, thread count.** No cell field records any of these. Platform is known only
  per corpus from manifests, gate files and the wave-29 amendment. For v2 waves w10–w25 and for v3, no
  archived host record in the corpus names the instance type. `scripts/aws/launch.py:231` defaults to
  `c7g.16xlarge` with 4 threads/cell (`:258`), but that is a code default, not a record of what ran.
- **Plans for w16–w23** are not archived, so `temporal_seed = 5170001` for those waves rests on
  `plan_cells.py:79` (current code), not on an archived plan.
- **Gate F bit-identity** between HEAD code and the pinned binaries that produced the cells was not
  re-run. Every code-sourced row applies to archived cells by inference, except where a cell field records
  the value directly (scale, clip, τ_m, d, L, epochs, sizes, arm, contract, geometry).
- **The event cache's identity.** T-step statistics come from the local `data/shd/events`. I did not
  compare its hash against the cache the fleet fetched.
- **The paper's "1,000+ cells" (§3.5)** is not a single number I could reproduce. The five corpora hold
  3934 cells.
- **GPU use: none found**, from reading the instrument's imports. I did not audit the transitive
  dependency tree for GPU code.
