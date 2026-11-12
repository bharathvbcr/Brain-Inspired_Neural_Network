# MEASUREMENT draft: the fast-numerics spike, measured against the pre-stated 2.0× bar

**Status: DRAFT, 2026-11-11.** This is stage 1 of
`PLAN_2026-11-11_FAST_NUMERICS_SERIES.md`. Nothing here is registered, and no
fast-mode code is on main. Both spike branches are scratch and will never be
merged:

- `spike/fast-numerics-fma` at 2c5c8a3;
- `spike/fast-numerics-blas` at 9dd85d3.

It becomes a record when the owner commits it to BINN `results/`.

**Bar, stated in the plan before any measurement.** A variant proceeds only if
both hold:

- **Speed:** at least 2.0× fewer cycles, or at least 2.0× less wall time for
  Accelerate, on the 2-epoch d32/L4 attention cell against 68a32a3.
- **Determinism:** a clean A/A check across thread counts.

**Platform.** Apple M5 Pro, 18 cores, shared with other sessions throughout
(load average 35–205). Cycles, instructions and user CPU come from
`/usr/bin/time -l` and are per-process. Wall time is reported but is noisy.
Every number is n = 4 per arm, interleaved ABBA, on one seed (5170001).

## 1. Where the time is now (68a32a3, 6-epoch attention cell, 60 s of `/usr/bin/sample`)

| function | all samples | non-idle |
|---|---:|---:|
| `dot_columns<SKIP_ZERO=false>` | 25.0% | 29.5% |
| `dot_columns<SKIP_ZERO=true>` | 19.0% | 22.4% |
| `attention_gradient` (self; `axpy_rowwise` and softmax inlined) | 14.3% | 16.9% |
| `transpose_into` | 8.8% | 10.4% |
| `expf`, plus the stub | 6.1% (+1.4%) | 7.2% (+1.6%) |
| `attention_forward_presented` (self) | 5.0% | 5.8% |
| idle (`__psynch_cvwait` + `swtch_pri`) | 15.2% | excluded |

**Amdahl ceilings.**

- Making `dot_columns` and the transposes infinitely fast gives at most 2.12×
  on all samples, or 2.65× on non-idle samples.
- The h512 base cell's matmul-like share is about 10%, so its ceiling is
  1.05–1.11×. Its time is in the inlined `axpy_into` input drive.

## 2. Results

**Exact mode is unchanged in both branches.** Its default path reproduces the
oracle sha 9c364bd9 and passes all 23 fixtures. Lane S1 also passed
`check_kernel_profiles.sh` in both profiles.

| variant | portable to Graviton | A/A, threads {16, 3, 1, 16} | cycles vs 68a32a3 (min / median) | wall vs 68a32a3 | verdict |
|---|---|---|---:|---:|---|
| `fma` (vfmaq in the NEON bodies, `mul_add` in `axpy_into`) | yes | identical, `20f17332…` | 1.022× / 1.045× | noise | **fails** |
| `fma-split` (fma plus a fixed 2-way k split) | yes | identical, `22583f8f…` | 1.110× / 1.114× | noise | **fails** |
| `fma` on the h512 base cell | yes | — | 0.998× / 0.979× | — | fails |
| `blas` (Accelerate `cblas_sgemm` for every read-out matmul, per sample and layer, single-threaded BLAS) | **no, Mac only** | identical, `b7ab4a5b…` (12/12, including Accelerate's default threading) | **2.23× / 2.28×** (lane S2, 16 threads) | 1.45× / 1.78× (16 threads, load 119–205) | **passes the bar on cycles** |

The orchestrator then re-checked `blas` independently.

| threads | cycles min | user CPU min | wall min / median |
|---|---:|---:|---:|
| 3 (the production per-cell layout) | **2.32×** (573.7 → 246.8 G) | 2.36× (133.7 → 56.7 s) | **2.05× / 1.78×** |
| 1 | 2.32× (577.4 → 249.3 G) | 2.34× | 2.47× / 2.20× |

- **Same weights:** every `blas` run gave `b7ab4a5b…`.
- **No hidden coprocessor time:** the wall-to-CPU ratio is about the same for
  both binaries (about 1.2× at 1 thread). So the Accelerate gain is not
  coprocessor time hidden from the CPU counters (**inferred**).

**How far the numbers move from exact** (2 epochs, one seed, not a statistic):

| | exact | fma | blas |
|---|---:|---:|---:|
| accuracy | 0.3953 | 0.4011 | 0.4002 |
| mean_loss | 2.17767 | 2.17746 | 2.17726 |

Lane S1 reports that 10 or 11 cell fields differ from exact under the FMA
variants.

- **Threading setting (verified).** `BLASSetThreading(BLAS_THREADING_SINGLE_THREADED)`
  is declared in the SDK's `thread_api.h`, from macOS 15. It is thread-local,
  so it is set on every rayon worker
  (https://developer.apple.com/documentation/accelerate/blassetthreading(_:)).
  `VECLIB_MAXIMUM_THREADS` appears in no header in this SDK.

## 3. Verdict

- **FMA, with or without the split: the line is closed.** At most 1.11× fewer
  cycles, and the Amdahl ceiling explains why: the kernels are already bound by
  loads and latency, not by the multiply-add count. Instructions fell 13–25%
  while cycles barely moved.
- **Accelerate: passes the pre-stated bar.** 2.2–2.3× fewer cycles and 2.3×
  less CPU, with clean A/A. Wall time at the production 3-thread layout was
  2.05× (min) and 1.78× (median) under load. Stage 2 is therefore *permitted*
  by the plan's rule. Its costs are the ones the plan names:
  - **Mac only.** Graviton has no Accelerate.
  - **OS-dependent.** A macOS update may change Accelerate's kernels, so every
    cell would record `sw_vers`, and replay across an OS update is not
    promised.
  - **A new dependency.** It links a system framework, with no crate.
  - **A 48-cell bridging series** (about 50 CPU-h), plus fast fixtures and a
    fast Gate F.
  - **A caveat a reviewer will see.**

The alternative stays exact. The SSC anchor at e100, about 650–700 CPU-h
(**inferred**), runs on one c8g.16xlarge in about 15 h, with no bridge and no
second numerics regime in the paper. Accelerate's 2.3× on the Mac would bring
the same job to about 300 CPU-h on a machine that is never quiet.

## 4. Owner decisions

1. **Build stage 2 for Accelerate, or not.** If yes: the `--numerics blas`
   contract (plan §3), Accelerate as an approved dependency, and the bridging
   PREREG (plan §4) committed before any bridging cell.
2. **Where SSC runs:** exact on Graviton, needing no bridge and keeping one
   regime, or `blas` on the Mac after the bridge.

## Evidence

The logs are committed with this record in
`results/fast_numerics_spike_2026-11-11/`, copied from the session scratchpad:

- `s1/RESULT.md` (profile, FMA tables, A/A);
- `s2/RESULT.md` (Accelerate, knob citation, A/A);
- `v3.out` and `v1.out` (the orchestrator's 3-thread and 1-thread ABBA).

Left out: every `*.log` (training logs and `s1/kernel_profiles.log`), which
the repository's `.gitignore` excludes and no record in `results/` carries;
and, for size, the `*.weights` and `*.bin` cell outputs and the two raw
`/usr/bin/sample` dumps (`s1/prof_attn/sample.txt`, 21 MB, and
`s1/prof_base2/sample.txt`, 15 MB); the `s1/prof_*.shares` files summarise
the profiles.
The binaries (`prof/bins/{r4,spike-fma,spike-blas}`) were scratch and are not
kept; the spike branches named above rebuild them.
