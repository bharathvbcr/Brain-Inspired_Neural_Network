# S2: Accelerate `cblas_sgemm` scratch probe (`--numerics blas`)

Worktree `/Users/bharath/Code/research/BINN.wt-spike-blas`, branch `spike/fast-numerics-blas`, commit `9dd85d3` (scratch, never merged,
not pushed). Binary: `S/prof/bins/spike-blas`. Baseline: `S/prof/bins/r4`, which reproduced the oracle sha
`9c364bd9...512d3` before anything else (verified). Machine: Apple M5 Pro, macOS 27.2, load average 120-205 throughout (shared).
Labels: **[V]** verified by a command run here, **[I]** inferred, **[U]** unverified.

## 1. Threading knob

- **[V]** `Accelerate.framework/.../vecLib.framework/Headers/thread_api.h` (Xcode MacOSX SDK) declares
  `int BLASSetThreading(const enum BLAS_THREADING)` and `enum BLAS_THREADING BLASGetThreading()`, both `API_AVAILABLE(macos(15.0)...)`.
  Values: `BLAS_THREADING_MULTI_THREADED = 0` ("Accelerate decides how many threads to use"), `BLAS_THREADING_SINGLE_THREADED = 1`.
  Returns 0 on success, -1 if unsupported. The header says: **"This setting is per thread, saved in a thread local variable."**
  Apple docs agree: https://developer.apple.com/documentation/accelerate/blassetthreading(_:)
- Consequence: it must be called on every thread that calls BLAS. The spike does it lazily on first sgemm per thread (thread_local flag),
  so it covers every rayon worker without plumbing. Return value asserted == 0.
- **[V]** `VECLIB_MAXIMUM_THREADS` does not appear in any SDK header (`rg` over the Headers dir). **[U]** whether the runtime still honours it; not used.
- Multithreaded run: env `BINN_BLAS_MULTITHREADED=1` skips the `BLASSetThreading(1)` call (spike-only switch). The `--numerics blas` flag is a
  process-global (`AtomicBool`, set once at start of `train-cell`), acceptable for a spike.
- FFI declared by hand from `cblas.h`: `cblas_sgemm(order, transA, transB, M, N, K, alpha, A, lda, B, ldb, beta, C, ldc)`, int ABI;
  `CblasRowMajor=101, CblasNoTrans=111, CblasTrans=112` (legacy API, deprecated-warning header, linked with
  `#[link(name = "Accelerate", kind = "framework")]`). No crates.io deps.

## 2. Implementation (`binn-learn/src/shd_attention.rs`, `binn-lab/experiments/shd_instrument.rs`)

All sgemms are row-major, alpha=1, beta=0 (so the +0.0 start and "no zero skip" are simply the new numerics). Exact path left intact in an
`else` branch; `use_blas = blas::enabled()` picks per call. Replaced, whole-matrix per sample per layer:

| site | exact | blas |
|---|---|---|
| forward q,k,v | 3 x T `dot_against_columns` over transposed W | 3 sgemm `X * W^T` (NoTrans, Trans; no transposes) |
| forward scores | T x `dot_against_columns` over `transposed(K)` | 1 sgemm `Q * K^T`; softmax row loop unchanged |
| forward context (T²·d) | T x `dot_columns::<false>` | 1 sgemm `A * V` |
| forward w_o | T x `dot_against_columns` | 1 sgemm `C * Wo^T`, then residual add |
| backward d_attention | T x `dot_against_columns` over `transposed(V)` | 1 sgemm `dC * V^T` |
| backward d_v (T²·d) | transpose A + T x `dot_columns::<true>` | 1 sgemm `A^T * dC` (Trans, no transpose copy) |
| backward d_q (T²·d) | T x `dot_columns::<true>` | 1 sgemm `dE * K` |
| backward d_k (T²·d) | transpose dE + T x `dot_columns::<true>` | 1 sgemm `dE^T * Q` |
| backward embedding | T x `dot_against_columns` over `transposed(w_e)` | 1 sgemm `dS * We^T`, rows scattered to `presentation[t]` |

**Not converted** (still the exact `axpy_rowwise` loops, in both modes): the `w_o` / `w_q,k,v` weight-gradient accumulations, the `d_context`
and `d_input` axpys, the `w_e` gradient, the pooling axpy, softmax, elementwise work, and the SNN forward/backward upstream of the read-out.
These are T·d² or smaller; converting them is a possible extra gain (not measured).

## 3. Exact-mode proof

**[V]** With `spike-blas` in exact mode (default): the oracle cell weights sha = `9c364bd9...512d3` (matches). `scripts/train_cell_fixtures.py
--binary S/prof/bins/spike-blas ... --jobs 3` => **23/23 pass; 23 compared, 0 could not be judged** (output in `S/spike/s2/fx`).

## 4. A/A determinism (blas, 2-epoch oracle cell, seed 5170001)

| RAYON_NUM_THREADS | BLAS threading | final-weights sha256 | accuracy | mean_loss |
|---|---|---|---|---|
| 1 | single | b7ab4a5b91e3f59a4f8aa57881a0e3c7107700d78ac44dafd665adec71a88ccb | 0.400176678 | 2.177263844 |
| 3 | single | b7ab4a5b91e3f59a4f8aa57881a0e3c7107700d78ac44dafd665adec71a88ccb | same | same |
| 16 | single (run 1) | b7ab4a5b91e3f59a4f8aa57881a0e3c7107700d78ac44dafd665adec71a88ccb | same | same |
| 16 | single (run 2) | b7ab4a5b91e3f59a4f8aa57881a0e3c7107700d78ac44dafd665adec71a88ccb | same | same |
| 16 | default multithreaded (run 1) | b7ab4a5b91e3f59a4f8aa57881a0e3c7107700d78ac44dafd665adec71a88ccb | same | same |
| 16 | default multithreaded (run 2) | b7ab4a5b91e3f59a4f8aa57881a0e3c7107700d78ac44dafd665adec71a88ccb | same | same |

Also identical: the first blas run (default rayon width) and all 6 blas cells in the timing runs (6/6 = `b7ab4a5b...`). **[V]** 12/12 identical.
**Caveat [U]:** the multithreaded-default run being identical does not show Accelerate used threads at these sizes (358x358x32 may be
below its threading threshold); it shows only that, for these shapes on this machine, enabling it did not change the bits. Not probed further.
The repo's own `parallelism_is_bit_identical_to_serial_for_every_arm` test was not run in blas mode (it has no blas hook).

**Distance from exact** (same cell, exact = oracle): accuracy 0.395318 (exact) vs 0.400177 (blas); mean_loss 2.177672 vs 2.177264;
epoch_mean_loss [2.65527, 1.70008] vs [2.65487, 1.69966]; mean_gradient_norm 2.3016 vs 2.3251. Weights differ (sha differs), as expected.
This is one seed, 2 epochs: a single sample of the shift, not an estimate of its distribution.

## 5. Timing (attention d32/L4, 2 epochs, `nice -n 5`, `/usr/bin/time -l`)

Script `S/spike/s2/abba.sh`; raw: `abba1.out` (ABBA: r4 blas blas r4 r4 blas blas r4) and `abba2.out` (late pair: blas r4 r4 blas). Load average
(1-min) per run: 119-177 in abba1, 163-181 in abba2. The machine never got quiet, so "late quiet-ish" pair is NOT quiet; wall is heavily contended.

ABBA, n=4 per arm:

| metric | r4 (68a32a3 arith.) runs | blas runs | r4 min / median | blas min / median | speedup min / median |
|---|---|---|---|---|---|
| cycles (x1e9) | 580.1, 584.2, 607.9, 607.7 | 259.9, 261.2, 260.9, 265.6 | 580.1 / 596.1 | 259.9 / 261.1 | **2.23x / 2.28x** |
| instructions (x1e12) | 2.794, 2.794, 2.795, 2.798 | 1.052, 1.053, 1.054, 1.054 | 2.794 / 2.795 | 1.052 / 1.054 | 2.66x / 2.65x |
| user CPU (s) | 140.0, 138.8, 143.3, 138.4 | 60.2, 59.8, 59.7, 59.8 | 138.4 / 139.4 | 59.7 / 59.8 | 2.32x / 2.33x |
| wall (s, from cell JSON) | 24.9, 33.9, 45.2, 61.7 | 18.9, 17.2, 25.7, 48.3 | 24.9 / 39.6 | 17.2 / 22.3 | **1.45x / 1.78x** |

Late pair (abba2, load 163-181): wall r4 50.6, 51.8 vs blas 46.2, 25.2; cycles r4 595.5, 596.2 vs blas 259.0, 256.4 (2.3x).
Pooling all 6 runs per arm: cycles min/median 2.26x / 2.29x; wall min 1.45x, median 1.89x.

Reading: cycle and user-CPU ratios are load-independent and agree (~2.2-2.3x), and user CPU fell by the same factor, so work done on the
matrix coprocessor, if any, is not hiding a large uncounted cost in these counters **[I]** (wall, which would show it, is consistent:
blas is never slower than r4 in any paired comparison). Wall is dominated by scheduler contention at load 120-200 (r4 wall ranged 25-62 s for
identical 139 CPU-s of work), so wall ratios are noisy; a single uncontended-looking 16-thread pair earlier gave 62.5 s (r4) vs 18.6 s (blas),
which is not a controlled comparison.

## 6. Verdict against the pre-stated bar

Bar: >= 2.0x fewer cycles OR less wall time on the attention cell vs 68a32a3, AND clean A/A.

- Cycles: **2.23x (min) / 2.28x (median)**, user CPU 2.3x: **bar met [V]**. Wall: less in every pair, 1.45x min / 1.78x median under load.
- A/A: **clean [V]**: 12/12 bit-identical across RAYON 1/3/16, repeats, and with Accelerate's default threading left on.
- Exact mode untouched: 23/23 fixtures and oracle sha [V].
- Caveats that matter for stage 2: numerics differ from exact (accuracy/loss shift above, one seed); Accelerate kernel selection can change with
  a macOS update [I, per plan s2]; only the T²·d and projection dots were converted, so the ~2.3x is a floor on what this path can reach [I];
  the 2.3x is against the r4 arithmetic of 68a32a3, which already includes the NEON register-blocking, and the plan's Amdahl estimate
  (1.5-1.7x) was too low for cycles on this cell.
- Not done: repo unit tests in blas mode; converting the axpy-based gradient loops; the h512 base cell (no attention, no effect expected);
  a truly quiet machine.
