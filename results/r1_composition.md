# R1 / U16 — multi-area composition

**Kill-gate override:** this run is an **exploratory post-G2 branch**. Gate G2 FAIL under `c1-118207fbc3eaba53` still stands. R1 does **not** reopen the v8 kill-gate; it requires `--enable-r1` / `--override-g2-for r1`.

- config hash: `r1-5d30383e334b9cbe`
- protocol version: 1
- quick/PILOT: false
- seeds: 8
- area sweep: 3..= 10
- cells/area × k-WTA: 16 × 2
- train / test per point: 2000 / 400
- compound margin: 0.050
- compound fraction: 0.000
- verdict: **ADDITIVE**

## Composition vs additive

| n_areas | composed | additive | compounds | mean nnz | locality |
|---:|---:|---:|---|---:|---:|
| 3 | 0.9653 | 0.9709 | false | 336 | 0.714 |
| 4 | 0.9094 | 0.9453 | false | 480 | 0.667 |
| 5 | 0.8878 | 0.9391 | false | 672 | 0.595 |
| 6 | 0.8381 | 0.9259 | false | 896 | 0.536 |
| 7 | 0.8250 | 0.9259 | false | 1152 | 0.486 |
| 8 | 0.7916 | 0.9116 | false | 1440 | 0.444 |
| 9 | 0.7578 | 0.9162 | false | 1648 | 0.437 |
| 10 | 0.7231 | 0.9000 | false | 1984 | 0.403 |

## Protocol

Task: noisy majority of `n_areas` latent bits (compositional pooling). **Composed** path: hub-routed linear readout (coupling from hub CSR). **Additive** path: identical learner with uniform coupling (no hub structure; matched train/test and learning rates).

Budgets disclosed: cells = `n_areas × cells_per_area`, nnz from hub CSR, train/test counts above.

## Full scientific schedule

```bash
cargo run -p binn-lab --release --bin r1 -- --enable-r1 \
--out results/r1_composition.md
```
