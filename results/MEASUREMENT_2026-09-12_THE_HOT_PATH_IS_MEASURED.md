# Measurement — the hot path, compiled, benchmarked and profiled

**Date:** 2026-09-12
**This is a measurement, not a verdict.** No hypothesis is evaluated, no wave's
status changes, and no number here is a result about BINN. It discharges one
instruction with data.
**Discharges:** [`TODO_2026-08-07_OPEN_WORK.md`](TODO_2026-08-07_OPEN_WORK.md) §7,
"Profile before acting on `PERF_AUDIT_2026-08-02.md`"
**Bears on:** [`PERF_AUDIT_2026-08-02.md`](PERF_AUDIT_2026-08-02.md), whose own
opening says "the first action on this document should be to profile, not to
start at #1 and work down"
**Host:** Apple M5 Pro, `aarch64-apple-darwin`, machine otherwise idle
**Binary:** `--release`, which is `lto = "fat"`, `codegen-units = 1`,
`overflow-checks = true`

---

## 1. Why this was waiting

The audit is a **static read**. Its own §0 says so: "no profiling, no
benchmarking, no compilation. Every cost figure below is an operation count or a
bandwidth estimate derived from the code, not a measurement." It then ranks
eight items by expected effect and calls the ranking a hypothesis.

Nothing had tested that hypothesis. The reason recorded in the register was
contention: profiling a machine that is running five training cells measures the
contention. Wave 29 finished on 2026-09-09 and the box is idle, so the reason
expired.

## 2. Two register items were already closed and the register did not know

**The "largest named candidate" is fixed.** §7 lists "the plasticity step
deep-copying its entire CSR *and* CSC on every update, ~30 MB of memcpy per step
at nnz ≈ 2.5e6". `binn-learn/src/three_factor.rs:206-220` now reads

```rust
let conn = &engine.conn;
let conn_rev = &engine.conn_rev;
```

with a comment beginning "This **previously** read `engine.conn.clone()`". The
audit's own "Applied in this pass" table records the change at
`three_factor.rs:134`. `conn.clone()` / `conn_rev.clone()` appear nowhere in the
workspace on a per-step path — the sixteen surviving `conn.clone()` calls are
one-time `set_connectivity` setup in runners, benches and tests.

**The audit's own #1 does not appear in the profile at all.**
`TimingWheel::scan_earliest` — "O(2048 + N), once per tick", ranked first — is
absent from 18,808 samples, because the occupancy bitmasks that replace it were
applied in the same pass.

So the two loudest items in §7 were descriptions of work already done. That is
the register being stale rather than the code being slow, and it is exactly what
profiling before acting was supposed to catch.

## 3. Benchmarks

Six, all four registered criterion targets, mean with 95% CI:

| benchmark | mean | 95% CI |
|---|---:|---|
| `engine_step/1024_external_events` | 18.357 µs | 18.262 – 18.456 µs |
| `kwta/n10000_k100` | 15.560 µs | 15.476 – 15.645 µs |
| `timing_wheel_insert_pop/1000` | 39.920 µs | 39.721 – 40.165 µs |
| `timing_wheel_insert_pop/10000` | 296.500 µs | 293.844 – 299.367 µs |
| `timing_wheel_insert_pop/50000` | 1.450 ms | 1.438 – 1.461 ms |
| `timing_wheel_insert_pop/100000` | 2.806 ms | 2.776 – 2.837 ms |

**The timing wheel does not degrade with occupancy.** Per-operation cost *falls*
from 39.9 ns at N = 1,000 to 28.1 ns at N = 100,000 and then flattens —
throughput 24.7 → 35.6 Melem/s. Whatever the queue's remaining costs are, a
scaling problem is not among them.

## 4. The profile

`xctrace` Time Profiler, 18,808 samples over 18.81 s of
`engine_step/1024_external_events`.

The bench process does two unrelated things, and separating them is necessary
before any percentage means anything:

| | share | time |
|---|---:|---:|
| engine workload, inside the bench closure | **81.2%** | 15.28 s |
| criterion's own statistical analysis (rayon-parallel bootstrap) | 18.7% | 3.53 s |
| dyld startup, other | 0.0% | 3 ms |

**Self time, engine only, as a share of the engine's 81.2%:**

| cost | % of engine | note |
|---|---:|---|
| `expf` | 15.3% | f32 exponential, the real one |
| `DYLD-STUB$$expf` | 9.0% | dynamic-call trampoline, not arithmetic |
| **exponentials, total** | **24.2%** | audit item **#3** |
| `_platform_memmove` | 4.9% | |
| `Engine` drop glue | 3.7% | the bench rebuilds the engine every iteration |
| `_platform_memset` | 1.2% | |
| malloc/free (`_xzm_*`) | ~3.1% | audit item **#4**, the per-cell `Vec` |

**The audit's #3 is the largest actionable cost, not its third.** Every `expf`
sample is called from the bench closure — the engine's own `cell.rs` path — and
verifying that mattered: the f64 `exp` also visible at 6.9% of the process is
called from `rayon::bridge_producer_consumer`, which is criterion's bootstrap,
not the engine. Attributing it to BINN would have inflated the exponential
finding by a third. There is no f64 `.exp()` in `binn-engine` or `binn-core`.

**Nine percent of engine time is spent in call trampolines rather than in
exponentials.** `lto = "fat"` is already set, so this is not a missing build
flag: `expf` lives in the system libm, across a dynamic boundary LTO cannot
cross. Removing it means not calling system libm — which changes the bits, and
[`FINDING_2026-08-19_LIBM_PORTABILITY_OF_REPLAY.md`](FINDING_2026-08-19_LIBM_PORTABILITY_OF_REPLAY.md)
is the record of how much that matters. **Not a candidate.**

## 5. What this does not establish, and it is the important half

**It does not measure the `dt` distribution, so it does not license the memo.**
The audit's only bit-identical route into the 24.2% is a `dt`-keyed memo for the
three `.exp()` calls, and it names the prerequisite: "under lazy integration
`dt` is the gap since a cell was last touched, and I have no evidence about its
distribution. Measure before building the memo."

This profile cannot supply that evidence. `engine_step/1024_external_events`
injects into all 1,024 cells and steps **one tick**, so every cell is touched
once at the same gap: the `dt` distribution here is degenerate by construction.
The measurement says what a cell update costs; it says nothing about how often a
given `dt` recurs across a real run, which is the only thing that decides
whether a memo hits.

So the memo stays unbuilt and the prerequisite stays open, now with a reason that
is measured rather than assumed: **exponentials are worth 24.2% of engine time,
and whether any of it is recoverable depends on a `dt` histogram from a
production workload, which this microbenchmark is not.**

**It does not rank the audit's items 2, 5, 6 or 7.** `last_step_charge.fill`,
the fan-out stride, the `collect()`+sort and `assoc_scan` are inside the 52.3%
of process time that LTO inlined into one opaque frame; the profile cannot
attribute within it. Their ranks remain hypotheses.

**It is one workload on one host.** A single-tick microbenchmark on 1,024 cells
with no connectivity and no plasticity is not the SHD instrument, and the
plasticity path this document clears of a deep copy is not exercised by it at
all.

## 6. Reproducing it

```
cargo bench -p binn-engine -p binn-areas
xctrace record --template 'Time Profiler' --output prof.trace \
  --launch -- target/release/deps/engine_step-<hash> --bench --measurement-time 12
xctrace export --input prof.trace \
  --xpath '//trace-toc/run[@number="1"]/data/table[@schema="time-profile"]'
```

Criterion persists its estimates under `target/criterion/**/new/estimates.json`,
which is where the table in §3 is read from rather than from scrollback.
