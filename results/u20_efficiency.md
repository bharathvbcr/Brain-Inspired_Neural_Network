# U18-U20 / Gate G5 — throughput and efficiency

**Exploratory post-G2 override.** C1 Gate G2 FAIL remains unchanged.

- schedule: scientific
- C1 config hash: `c1-118207fbc3eaba53`
- verdict: **FAIL**
- activity sparsity: 0.0156
- local accuracy: 0.4912
- parameter-matched dense accuracy: 0.5000

## U18 partitioned delta engine (F1 adaptive)

| parity with sequential | graph cut edges | sequential seconds | adaptive partitioned seconds | always-rayon seconds |
|---|---:|---:|---:|---:|
| true | 0 | 0.001892 | 0.004334 | 0.004424 |

Parallel threshold: `PARALLEL_CELL_THRESHOLD=8` distinct cells/tick.

## U19 reset-aware scan training (F1 barriers)

| scanned steps | reset-free segments | sequential reset barriers | mean seg len | max seg len | barrier fraction | scan headroom | wall seconds |
|---:|---:|---:|---:|---:|---:|---:|---:|
| 250000 | 251 | 250 | 996.02 | 997 | 0.001000 | 0.999000 | 0.014498 |

## U20 measured work disclosure

| condition | accuracy | modeled work/accuracy | wall seconds | peak RSS bytes |
|---|---:|---:|---:|---:|
| local assembly | 0.4912 | 1886489.3589 | 0.006202 | 15384576 |
| dense local | 0.5000 | 4978492.0000 | 0.011490 | 16367616 |
| dense parameter-matched | 0.5000 | 1796194.0000 | 0.006743 | 16416768 |

## F5 activity≠compute accounting

| condition | activity sparsity | event_work | naive_activity_work (N×a) | work_vs_activity_ratio | source_spikes | synaptic_deliveries |
|---|---:|---:|---:|---:|---:|---:|
| local assembly | 0.0156 | 801758.0 | 2.1 | 388731.15 | 816 | 13085 |
| dense local | 0.0156 | 2489246.0 | 2.1 | 1206907.15 | 937 | 61680 |
| dense parameter-matched | 0.0156 | 898097.0 | 2.1 | 435440.97 | 937 | 40971 |

Modeled work uses disjoint source-spike, delivery, cell-update, and plasticity-update counters. It is a work proxy, **not hardware energy**. Gate G5 requires lower work/accuracy at matched accuracy and disclosed sparsity. F5 ratio ≫ 1 means sparse activity understates event/queue work.
