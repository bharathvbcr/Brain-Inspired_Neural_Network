# C2 / Gate G3 — class-incremental continual learning

**Kill-gate override:** this run is an **exploratory post-G2 branch**. Gate G2 FAIL under `c1-118207fbc3eaba53` still stands. C2 does **not** reopen the v8 kill-gate; it requires `--enable-c2` / `--override-g2-for c2`.

- config hash: `c2-ddc6176952829d90`
- protocol version: 1
- quick/PILOT: true
- seeds: 3
- stream: 4 classes, 24 train/class, 12 test/class
- baseline: `C2_CAPACITY_REPLAY_GRADIENT_BASELINE` (replay_capacity=24, lr=0.35)
- G3 verdict: **PILOT**

## Summary

| metric | value |
|---|---:|
| mean forgetting (local) | 0.6667 |
| mean forgetting (replay baseline) | 0.0046 |
| local below baseline | false |
| forgetting force-high | 0.7361 |
| forgetting force-low | 0.1667 |
| forgetting shuffle | 0.1667 |
| intervention direction (high > low) | true |

## Per-seed

| seed | forget_local | forget_baseline | high | low | shuffle | overlap | acc_local | acc_base |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 11400787387090631700 | 0.5000 | 0.0000 | 0.5694 | 0.0000 | 0.0000 | 0.3333 | 0.2500 | 1.0000 |
| 4354472259681056811 | 0.5000 | 0.0000 | 0.6389 | 0.0000 | 0.1667 | 0.3333 | 0.2500 | 1.0000 |
| 15755472952027477054 | 1.0000 | 0.0139 | 1.0000 | 0.5000 | 0.3333 | 0.1667 | 0.2500 | 0.9792 |

## Preregistered interventions

- `force-high` / `force-low`: bias k-WTA toward / away from earlier class assemblies while holding activity `k` fixed.
- `shuffle-overlap`: randomly reassign the reserved set at the same cardinality.
Predicted direction: **force-high forgetting > force-low forgetting**.

## Full scientific schedule

```bash
cargo run -p binn-lab --release --bin c2 -- --enable-c2 \
--out results/c2_g3.md
```
