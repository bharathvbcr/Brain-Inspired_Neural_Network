# R1 / U16 — multi-area composition

**Kill-gate override:** this run is an **exploratory post-G2 branch**. Gate G2 FAIL under `c1-118207fbc3eaba53` still stands. R1 does **not** reopen the v8 kill-gate; it requires `--enable-r1` / `--override-g2-for r1`.

- config hash: `r1-ab69e1b6eb9b98e6`
- protocol version: 1
- quick/PILOT: true
- seeds: 3
- area sweep: 3..= 5
- cells/area × k-WTA: 8 × 1
- train / test per point: 400 / 100
- compound margin: 0.050
- compound fraction: 0.000
- verdict: **PILOT**

> PILOT only: the quick schedule validates the harness and cannot alone license a scientific composition claim.

## Composition vs additive

| n_areas | composed | additive | compounds | mean nnz | locality |
|---:|---:|---:|---|---:|---:|
| 3 | 0.8567 | 0.9467 | false | 72 | 0.667 |
| 4 | 0.8067 | 0.9333 | false | 104 | 0.615 |
| 5 | 0.7467 | 0.9200 | false | 160 | 0.500 |

## Protocol

Task: noisy majority of `n_areas` latent bits (compositional pooling). **Composed** path: hub-routed linear readout (coupling from hub CSR). **Additive** path: identical learner with uniform coupling (no hub structure; matched train/test and learning rates).

Budgets disclosed: cells = `n_areas × cells_per_area`, nnz from hub CSR, train/test counts above.

## Full scientific schedule

```bash
cargo run -p binn-lab --release --bin r1 -- --enable-r1 \
--out results/r1_composition.md
```
