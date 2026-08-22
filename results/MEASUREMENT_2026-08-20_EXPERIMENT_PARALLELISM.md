# Is BINN optimised for server-scale cores?

**Measured:** 2026-08-20, local M5 Pro (18 cores, arm64) and the `c7g.16xlarge`
campaign fleet (64 vCPU, Graviton3).
**Short answer:** the engine has parallel and SIMD code paths, and **the paper's
workload runs on none of them.** The first leverage point is experiment-level
fan-out; the second is eliminating redundant work inside `binn-learn` without
changing its arithmetic.

---

## 1. The three leverage points that were proposed, checked against the source

| proposal | status | evidence |
|---|---|---|
| "The hot path is already SIMD-structured; wider vectors would help directly" | **False for this workload** | `simd_leak_integrate` has **zero non-bench call sites**. Its only references outside its own module are `binn-core/src/lib.rs:20` (the re-export) and `binn-core/benches/simd_leak_integrate.rs`. It is benchmarked, not used. |
| "The `CpuParallel` backend uses Rayon; more cores = more parallel cell updates within a tick" | **True, and off the paper's path** | `Backend::CpuParallel` lives in `binn-core/src/metal_backend.rs`; `spmv_cpu` at line 228 does use `y.par_iter_mut()`. `SpmvBackend` is constructed in exactly two experiments — `c1_enhanced.rs:296` and `multi_area_scaling.rs:455`. Neither is on the paper's critical path. |
| "Algorithmic batching would need engine changes" | **True but unnecessary** | The engine is not the bottleneck, because the paper's numbers never enter it. |

**The load-bearing fact:** `binn-learn/Cargo.toml` declares exactly two
dependencies — `binn-core` and `binn-engine`. **It does not depend on `rayon`.**
Every matched-architecture arm, every gradient reference, every SHD arm, and the
attention read-out live in `binn-learn`. All of it is single-threaded scalar
`f32`, and no amount of vector width or SpMV parallelism touches it.

## 2. Where the parallelism actually is

The unit of work in every campaign is a **cell** — one (arm, config, seed)
training run — and cells are independent by construction, because each is seeded
solely by its seed and reads its split immutably. That is the axis worth
parallelising, and it lives in `binn-lab/experiments/`, above `binn-learn`, in a
crate that already has `rayon`.

Before today, **2 of 29** experiment binaries used it. Now **3 of 29**:

| binary | state | measured |
|---|---|---|
| `shd_instrument` | parallel (2026-08-19) | **7.9×** at 16 threads |
| `deep_snn_scaling` | parallel (2026-08-20) | **9.1×** at 18 cores |
| `efficiency` | parallel | not benchmarked |
| the other 26 | serial | — |

### `deep_snn_scaling`, measured

| | wall time | SHA-256 of report |
|---|---:|---|
| serial (pre-change binary), `--quick` | 28.77 s | `e2227447…b854b8e3` |
| parallel, `RAYON_NUM_THREADS=1` | — | `e2227447…b854b8e3` |
| parallel, `RAYON_NUM_THREADS=4` | — | `e2227447…b854b8e3` |
| parallel, all 18 cores | **3.16 s** | `e2227447…b854b8e3` |

**9.1× wall clock, byte-identical output at every thread count.** User time is
28.89 s parallel vs 28.14 s serial — a 2.7% coordination overhead, so the speedup
is real work moving to real cores, not work being skipped.

The full schedule went from an estimated ~51 minutes to a **measured 7 minutes**.

### Why it is bit-identical rather than approximately identical

Two properties, both structural rather than lucky:

1. **No cell observes another.** Each is constructed from `seed` alone and takes
   the split by shared reference. There is no accumulator, no RNG stream, and no
   scratch buffer crossing cells.
2. **The one cross-cell reduction is folded in order.** `ModulatorScale::merge`
   is `f64` addition, which is not associative, so it is applied *after*
   `map().collect()` — and rayon's `collect` preserves input order. The parallel
   fold is the same fold, in the same order.

This is the same discipline `binn-core/src/scan.rs:125-128` documents for its
index-based loop, and the same one the `--config-hash` replay property depends
on. Parallelism here costs nothing scientifically, which is why the report hash
is the acceptance test rather than a tolerance band.

## 3. Scaling shape, measured — fewer threads per cell, more cells

One attention cell, e10/h128, on the campaign fleet:

| threads | speedup | efficiency | cells per 64 vCPU | throughput |
|---:|---:|---:|---:|---:|
| 1 | 1.00 | 100% | 64 | 64.0 |
| 2 | 1.85 | 92% | 32 | 59.1 |
| 4 | 3.38 | 84% | 16 | 54.1 |
| 8 | 5.72 | 71% | 8 | 45.8 |
| 16 | 7.90 | 49% | 4 | 31.6 |

**Throughput is maximised by fewer threads per cell**: intra-cell speedup never
keeps up with the cores it consumes, so a box finishes more total work running
more cells slowly than fewer cells quickly.

Threads-per-cell is therefore a **tail-latency** control, not a throughput
control. It is worth spending efficiency on only when one cell is long enough to
become the campaign's floor — the h1024/L4/e400 cell is ~19 h at 8 threads and
~9.5 h at 16, and no amount of extra capacity divides it further once it has its
threads.

## 4. What this means for "more cores"

- **A bigger instance does not make a cell faster** unless that cell is itself
  parallel. Before `shd_instrument` was parallelised, adding vCPU bought exactly
  nothing per cell.
- **A bigger instance does make a campaign faster**, linearly, up to memory: the
  corpus is loaded per process at ~1.4 GB, so a 128 GB box tops out near 32
  concurrent cells regardless of vCPU count.
- **AVX-512 is not yet a measured claim.** The previous fleet was arm64. The
  Azure Dalsv7 build targets `x86-64-v4`, so LLVM may vectorise independent
  elementwise loops, but exact floating-point reductions forbid reassociation.
  Gate F and Azure wall telemetry decide whether the wider ISA helped.
- **The remaining 26 serial binaries are the actual backlog.** Each is a
  self-contained seed loop over independent cells, and each is the same
  ~15-line change: flatten to a work list, `map().collect()`, fold in order,
  and accept only a byte-identical report.

## 5. Scope

- **Verified:** every call-site count and dependency claim above, by grep over
  the workspace excluding `target/`; the `deep_snn_scaling` timings and hashes,
  on this machine, this session.
- **Verified:** the thread-scaling table, measured on the fleet 2026-08-19.
- **Not verified:** that the other 26 binaries would parallelise bit-identically.
  Each needs its own cross-cell-state audit; the two done so far each had exactly
  one ordered reduction to preserve, but that is a sample of two.
- **Not claimed:** that every `binn-learn` inner loop is efficient. The targeted
  audit below proves one redundant layout conversion and identifies attention's
  quadratic time axis; it is not a whole-program hardware profile.

## 6. 256-core scheduler and batch-layout audit

The old Azure configuration (`8 x D16als_v7`, four processes per node) could
reserve all 128 approved cores. Merely changing its SKU to `D64als_v7` would
still launch four processes and reserve only 16 of each node's 64 vCPUs. The
replacement scheduler treats cores as tokens: up to 16 normal four-thread cells
fit per node, while h1024/d64 tail cells reserve eight and the dispatcher
backfills four-thread cells into the remainder. The pure scheduler test reaches
64 reserved cores and asserts it never exceeds 64 or 16 processes.

Source audit found that `loss_and_gradient_arm_scaled` transposed immutable
`w_in` (and recurrent `w_rec`) once per sample. Weights change only after an
entire 256-sample optimizer batch. `ArmWeightLayout` now performs this data-only
transpose once per batch and once per evaluation, and the public one-sample API
retains its original behavior. The dedicated kernel comparison and the
one-vs-four-thread ordered-gradient test are bit-exact.

Exact local replay, h128/d32/L4, e1, seed 5170001, four Rayon threads:

| binary | instrument wall | process wall | scientific fields |
|---|---:|---:|---|
| pre-change | 75.731654 s | 75.88 s | reference |
| batch-layout | 72.525924 s | 72.73 s | bit-identical |

That is a 4.23% wall reduction and 1.044x speedup on the M5 Pro. Separately,
strict longest-processing-time ordering removes the anchor-first tail. With the
measured thread speedups and 4.23% factor, the four D64 shard simulations are
identical: 17.828 h estimated wall, 94.464% scheduled core utilization, and a
64-core peak reservation. These are model outputs, not Azure measurements.

The remaining algorithmic bottleneck is the attention block: its score matrix
is quadratic in timesteps and is retained for backward. Flash-style attention
would change both the forward/backward implementation and floating-point path,
so it is not introduced into this already registered confirmatory campaign.
