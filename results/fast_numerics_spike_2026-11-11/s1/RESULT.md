# S1: FMA / fma-split numerics spike, BINN branch spike/fast-numerics-fma (scratch)

Code: commit 2c5c8a3 on spike/fast-numerics-fma (Rust otherwise identical to 68a32a3). Binary: S/prof/bins/spike-fma.
Baseline: S/prof/bins/r4 (0d8766d). **Verified**: r4 reproduces the oracle sha 9c364bd9...512d3 (r4check, and all 8 baseline ABBA runs) and the h512 base sha 536951df... (all 4 baseline runs).
Machine shared and loaded throughout (load 100-185, a sibling lane s2 running). Cycles/instructions are the metric; wall is noise-dominated and not used for the verdict.
Navigation tools: `devmap build --manifest` was needed (store was unbuilt), then fresh; `gitpulse_insights` returned with `deadline_expired: true` after 25 s, collisions facet `ok:false` (3 worktrees unscanned) - so no collision-clean claim; no sibling edits to this worktree's files were observed.
Implementation note: the mode is a process-global AtomicU8 set once at startup (spike shortcut, as allowed); exact mode takes the original untouched code paths.

## 1. Profile of the baseline (`/usr/bin/sample`, per-function top-of-stack samples)

(a) d32/L4 attention cell, 6 epochs, 60 s sample starting ~20 s in (wall 280 s; verified from sample.txt):

| function | all samples | non-idle |
|---|---|---|
| dot_columns<SKIP_ZERO=false> | 25.0% | 29.5% |
| dot_columns<SKIP_ZERO=true> | 19.0% | 22.4% |
| attention_gradient (self; axpy_rowwise and softmax inlined) | 14.3% | 16.9% |
| transpose_into | 8.8% | 10.4% |
| expf (+ DYLD stub 1.4%) | 6.1% | 7.2% (+1.6%) |
| attention_forward_presented (self) | 5.0% | 5.8% |
| run_loss_and_gradient_with_workspace (base-arm part) | 1.9% | 2.3% |
| idle: __psynch_cvwait + swtch_pri | 15.2% | excluded |

Total 633,676 samples, idle 96,302, non-idle 537,374.
Amdahl ceiling if the matmul-like kernels cost nothing (dot_columns* + transpose_into): 52.8% of all / 62.3% of non-idle, so **2.12x on all samples, 2.65x on non-idle**. dot_columns alone: 51.9% non-idle, ceiling 2.08x. `axpy_rowwise` is inlined into attention_gradient / attention_forward (not separately symbolised), so it is NOT in these figures; including it would raise the ceiling by an unmeasured amount (bounded by the 16.9% + 5.8% self shares).
Reading: even an infinitely fast dot_columns+transposes only just reaches 2.1x end-to-end on a wall-sample basis, so 2.0x from FMA-class changes is not reachable. (inferred from shares)

(b) h512 channels-700 fixed-t500 base cell. First attempt (2 epochs, sample at +8 s) landed entirely in `frame_events` loading (55%), discarded. Second: 4 epochs, sample 45 s starting +75 s (S/spike/s1/prof_base2):
non-idle (idle 56% of samples: swtch_pri 37.7%, cvwait 18.7%): run_loss_and_gradient_with_workspace self 73.3% (axpy_into inlined here, not separable by symbol), transpose_blocked 10.0%, rayon helper 6.5%. Matmul-like kernels with their own symbol: 10.0% (transposes only), ceiling 1.11x non-idle / 1.05x all. The axpy_into share is within the 73.3% and was not isolated.

## 2. Variants (what changed)
- `fma`: `vfmaq_f32` in dot_block / axpy_blocks, `f32::mul_add` in scalar dot_columns tail, axpy tail and `axpy_into`. Binary contains 231 `fmla.4s` instructions (objdump), so fusion did happen.
- `fma-split`: fma plus 2-way split of every k-sum (rows i%2 to two accumulator sets, combined `a0+a1` at the end) when x.len() >= 16 (the d=32 loops qualify; shorter not split). 2-way chosen because V=8 blocks then hold 16 accumulators + loads without spilling (32 regs); 4-way at V=8 would use all 32 registers. Scalar tails never split.

## 3. Exact-mode proof (verified)
- `train_cell_fixtures.py --binary spike-fma ... --jobs 3`: **23/23 pass, 23 compared, 0 could not be judged** (fx.log).
- `scripts/check_kernel_profiles.sh` (own target dir): passes in both debug and release profiles for binn-learn binn-lab (kernel_profiles.log).

## 4. A/A determinism (verified): 2-epoch oracle cell, final-weights sha256

| variant | threads 16 | 3 | 1 | 16 again | identical |
|---|---|---|---|---|---|
| exact (r4, ref) | 9c364bd9441d1406cd97aa15078a83c1042b93180963f6bf159652528df512d3 | | | | |
| fma | 20f17332f0a22e0a00f9e7187245c4f9cd6f1cc5db4bef0a774689b834013a4b | same | same | same | yes (4/4) |
| fma-split | 22583f8fe7d04e1d03311eedcd1e1550ee15d5070085e5598c4c3c51cf9bc244 | same | same | same | yes (4/4) |

All 8 ABBA variant runs per variant also reproduced the same sha (fma 4/4, fma-split 4/4).
Differences from exact in cell.json (46 fields): fma 11 substantive fields differ (accuracy 0.3953 -> 0.4011, majority_prediction, mean_firing_rate, mean_loss 2.17767 -> 2.17746, mean_gradient_norm, mean_update_rms, epoch_mean_loss, epoch_mean_gradient_norm, epoch_max_gradient_norm, tail_loss_improvement, val_accuracy), fma-split 10 (accuracy equal, the rest differ), plus the two emission timestamps. As expected the numerics change every trajectory.

## 5. Timing: ABBA (r4, V, V, r4, r4, V, V, r4), nice 5, `/usr/bin/time -l`

| cell / variant | instructions min | cycles min | cycles median (r4 -> V) | user CPU median | wall median | speedup vs r4, cycles (min / median) | instr |
|---|---|---|---|---|---|---|---|
| attn d32/L4 2ep, fma | 2323.5G (r4 2793.6G) | 565.3G (r4 577.4G) | 594.1G -> 568.7G | 138.0 -> 130.4 s | 53.1 -> 50.0 s | **1.022x / 1.045x** | 1.20x |
| attn d32/L4 2ep, fma-split | 2240.2G (r4 2794.9G) | 529.2G (r4 587.5G) | 598.5G -> 537.1G | 136.9 -> 124.3 s | 37.2 -> 27.8 s | **1.110x / 1.114x** | 1.25x |
| h512 base 2ep, fma | 264.6G (r4 300.0G) | 133.8G (r4 133.5G) | 137.2G -> 140.2G | 30.3 -> 30.4 s | 17.2 -> 12.9 s | **0.998x / 0.979x** | 1.13x |

Wall is dominated by load swings between runs (individual walls ranged 10-66 s for identical work) and is not evidence. The base-cell fma run produced the **same** final-weights sha as exact (536951df...), i.e. in that cell fma changed instruction count but not one bit of the result (inferred cause: axpy_into scales there are spikes/exact products, not verified).
Instructions drop 13-25% but cycles barely move: the kernels are not instruction-issue bound (inferred: load/latency bound; not profiled at the micro-architectural level).

## 6. Verdict against the pre-stated bar (>= 2.0x fewer cycles on d32/L4 attention AND clean A/A)
- fma: 1.02-1.05x cycles. **Fails the bar by a wide margin.** A/A clean.
- fma-split (2-way): 1.11x cycles. **Fails.** A/A clean.
- Even the Amdahl ceiling for infinitely fast dot_columns+transposes is ~2.1x of all samples (2.65x of non-idle), so no reassociation/FMA variant of these kernels can plausibly reach 2.0x; the first 1.6x precedent bar (ft-3966) is also missed.
- Base cell: no gain (0.98-1.00x cycles), ceiling 1.05-1.11x from the measured shares.
Conclusion by the plan's rule: do not proceed to stage 2 on the FMA path; the Accelerate lane (sibling) is the only candidate left. Not done / caveats: n=4 per arm, single seed, loaded machine; no same-binary null pair (r4 vs r4 spread is visible within the baseline rows: cycles 577-598G for the attention cell and 133.5-140.8G for base, i.e. 3-5%, so the fma 4.5% and base results are inside the noise floor; fma-split's 11% is above it); 4-way split and a longer-k split variant not tried; axpy_into/axpy_rowwise not isolated in the profile.
