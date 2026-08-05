# Engine F1 / F5 systems (P2 engineering)

**Non-claims:** engineering measurement only — not biology, not a G2/G4 reopen, not a neuromorphic energy claim. Modeled work is a software work proxy.

- schedule: PILOT
- companion U18–U20 note: `results/u20_efficiency_quick.md`
- C1 config hash: `c1-e0dfdbf4e3d2936b` (unchanged kill-gate family)

## F1 — reset-barrier / parallelism headroom

### Same-tick engine delta buckets

| metric | value |
|---|---:|
| sequential wall (s) | 0.000409 |
| adaptive partitioned wall (s) | 0.001068 |
| always-rayon partitioned wall (s) | 0.000930 |
| adaptive / sequential speedup | 0.383 |
| always-rayon / sequential speedup | 0.440 |
| parity with sequential | true |
| ticks with events | 456 |
| parallel ticks (≥ threshold) | 8 |
| sequential thin ticks | 448 |
| mean width (distinct cells/tick) | 1.12 |
| max width | 8 |
| width headroom (mean/threshold, cap 1) | 0.140 |
| PARALLEL_CELL_THRESHOLD | 8 |

**Reading:** width headroom near 1 means many buckets meet the parallel threshold; values ≪ 1 mean the stream is thin-tick dominated. Cross-tick spike reset / fan-out remains a sequential barrier. Adaptive path skips rayon on thin ticks (safe; determinism preserved).

### Reset-aware scan (U19)

| metric | value |
|---|---:|
| steps | 10000 |
| reset barriers | 10 |
| segments | 11 |
| mean segment length | 909.09 |
| max segment length | 997 |
| parallelizable steps (len > chunk) | 9970 |
| barrier fraction | 0.001000 |
| scan headroom (1 − barrier fraction) | 0.999000 |
| wall seconds | 0.000641 |

## F5 — activity ≠ compute

| condition | activity | event_work | naive N×a | ratio |
|---|---:|---:|---:|---:|
| local | 0.0156 | 59265.0 | 1.1 | 55778.82 |
| dense-local | 0.0156 | 195972.0 | 1.1 | 184444.24 |

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
- Adaptive rayon helps thin streams; it does not invent width when events are sparse.
- G5 FAIL / G2 FAIL stand; this note does not reinterpret kill gates.
