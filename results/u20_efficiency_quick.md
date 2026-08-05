# U18-U20 / Gate G5 — throughput and efficiency

**Exploratory post-G2 override.** C1 Gate G2 FAIL remains unchanged.

- schedule: PILOT
- C1 config hash: `c1-e0dfdbf4e3d2936b`
- verdict: **PILOT**
- activity sparsity: 0.0156
- local accuracy: 0.5625
- parameter-matched dense accuracy: not-run

## U18 partitioned delta engine (F1 adaptive)

| parity with sequential | graph cut edges | sequential seconds | adaptive partitioned seconds | always-rayon seconds |
|---|---:|---:|---:|---:|
| true | 0 | 0.000409 | 0.001068 | 0.000930 |

Parallel threshold: `PARALLEL_CELL_THRESHOLD=8` distinct cells/tick.

## U19 reset-aware scan training (F1 barriers)

| scanned steps | reset-free segments | sequential reset barriers | mean seg len | max seg len | barrier fraction | scan headroom | wall seconds |
|---:|---:|---:|---:|---:|---:|---:|---:|
| 10000 | 11 | 10 | 909.09 | 997 | 0.001000 | 0.999000 | 0.000641 |

## U20 measured work disclosure

| condition | accuracy | modeled work/accuracy | wall seconds | peak RSS bytes |
|---|---:|---:|---:|---:|
| local assembly | 0.5625 | 118530.0000 | 0.003447 | 9306112 |
| dense local | 0.5000 | 391944.0000 | 0.002540 | 9551872 |
| dense parameter-matched | not-run | not-run | not-run | not-run |

## F5 activity≠compute accounting

| condition | activity sparsity | event_work | naive_activity_work (N×a) | work_vs_activity_ratio | source_spikes | synaptic_deliveries |
|---|---:|---:|---:|---:|---:|---:|
| local assembly | 0.0156 | 59265.0 | 1.1 | 55778.82 | 213 | 1319 |
| dense local | 0.0156 | 195972.0 | 1.1 | 184444.24 | 258 | 7720 |
| dense parameter-matched | not-run | not-run | not-run | not-run | not-run | not-run |

Modeled work uses disjoint source-spike, delivery, cell-update, and plasticity-update counters. It is a work proxy, **not hardware energy**. Gate G5 requires lower work/accuracy at matched accuracy and disclosed sparsity. F5 ratio ≫ 1 means sparse activity understates event/queue work.
