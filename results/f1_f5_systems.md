# Engine F1 / F5 systems (P2 engineering)

**Non-claims:** engineering measurement only — not biology, not a G2/G4 reopen, not a neuromorphic energy claim. Modeled work is a software work proxy.

- schedule: scientific
- companion U18–U20 note: `results/u20_efficiency.md`
- C1 config hash: `c1-118207fbc3eaba53` (unchanged kill-gate family)

## F1 — reset-barrier / parallelism headroom

### Same-tick engine delta buckets

| metric | value |
|---|---:|
| sequential wall (s) | 0.001892 |
| adaptive partitioned wall (s) | 0.004334 |
| always-rayon partitioned wall (s) | 0.004424 |
| adaptive / sequential speedup | 0.437 |
| always-rayon / sequential speedup | 0.428 |
| parity with sequential | true |
| ticks with events | 1824 |
| parallel ticks (≥ threshold) | 32 |
| sequential thin ticks | 1792 |
| mean width (distinct cells/tick) | 1.12 |
| max width | 8 |
| width headroom (mean/threshold, cap 1) | 0.140 |
| PARALLEL_CELL_THRESHOLD | 8 |

**Reading:** width headroom near 1 means many buckets meet the parallel threshold; values ≪ 1 mean the stream is thin-tick dominated. Cross-tick spike reset / fan-out remains a sequential barrier. Adaptive path skips rayon on thin ticks (safe; determinism preserved).

### Reset-aware scan (U19)

| metric | value |
|---|---:|
| steps | 250000 |
| reset barriers | 250 |
| segments | 251 |
| mean segment length | 996.02 |
| max segment length | 997 |
| parallelizable steps (len > chunk) | 250000 |
| barrier fraction | 0.001000 |
| scan headroom (1 − barrier fraction) | 0.999000 |
| wall seconds | 0.014498 |

## F5 — activity ≠ compute

| condition | activity | event_work | naive N×a | ratio |
|---|---:|---:|---:|---:|
| local | 0.0156 | 801758.0 | 2.1 | 388731.15 |
| dense-local | 0.0156 | 2489246.0 | 2.1 | 1206907.15 |

Ratio ≫ 1: counting active cells understates queue/delivery/update work.

## How to reproduce

```bash
cargo run --release -p binn-lab --bin efficiency -- --enable-efficiency --out results/u20_efficiency.md
cargo bench -p binn-engine --bench f1_parallelism
cargo test -p binn-engine -p binn-learn --lib
cargo test -p binn-lab --test override_refuse
# optional Polars summary:
cargo test -p binn-lab --features tables harvest -- --nocapture
```

## Remaining limits

- Cross-tick causality (synaptic delay + spike reset) is still sequential.
- Scan headroom is a timeline fraction, not measured wall-clock speedup.
- Adaptive rayon helps thin streams vs always-rayon; delta-bucket grouping still costs vs bare `step_until`, so sequential often remains fastest on CPU.
- Synaptic fan-out can widen ticks and erase thin-tick headroom — F1 timing microbench uses a no-cascade schedule to isolate the barrier.
- G5 FAIL / G2 FAIL stand; this note does not reinterpret kill gates.
