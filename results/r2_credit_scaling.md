# R2-credit — directed-credit mitigation probe

**Does not reopen Gate G4 NO-GO** under frozen `r2-afafa0fa6f43e3fc`, nor Gate G2 FAIL under `c1-118207fbc3eaba53`. This is a **separate** directed-credit hypothesis (`r2-credit-*`): same disclosed #areas grid as R2, credit arms = graded DFA / REINFORCE×frozen B (optional 1-seed ±1 smoke). Frozen R2 is **not** remassaged.

**Kill-gate override:** exploratory post-G2 branch; requires `--enable-r2 --credit` (or `--override-g2-for r2`).

- config hash: `r2-credit-2f5647981724c62b`
- protocol version: 1
- quick/PILOT: false
- directed seeds: 8
- disclosed sweep: 3..= 24 step 3 (matches frozen R2 grid)
- mitigation reading: **directed credit recovers (healthy) on at least one arm**

## Arm `graded-dfa` (n_seeds=8)

- fit: capability ≈ 0.0681 · ln(n) + 0.3243  (R²=0.471)
- curve shape: **healthy**

| n_areas | mean capability | mean nnz |
|---:|---:|---:|
| 3 | 0.3325 | 336 |
| 6 | 0.5288 | 896 |
| 9 | 0.4958 | 1648 |
| 12 | 0.5296 | 2752 |
| 15 | 0.4625 | 3936 |
| 18 | 0.5429 | 5568 |
| 21 | 0.4821 | 7184 |
| 24 | 0.5412 | 8976 |

## Arm `reinforce-fb` (n_seeds=8)

- fit: capability ≈ -0.0424 · ln(n) + 0.6915  (R²=0.531)
- curve shape: **degrade**

| n_areas | mean capability | mean nnz |
|---:|---:|---:|
| 3 | 0.6275 | 336 |
| 6 | 0.6600 | 896 |
| 9 | 0.5763 | 1648 |
| 12 | 0.6033 | 2752 |
| 15 | 0.5367 | 3936 |
| 18 | 0.5883 | 5568 |
| 21 | 0.5471 | 7184 |
| 24 | 0.5712 | 8976 |

## Optional ±1 smoke control (1 seed)

_Harness sanity only — not a remassage of frozen G4. Expected to still degrade under broadcast credit._

## Arm `broadcast-pm1` (n_seeds=1)

- fit: capability ≈ -0.1935 · ln(n) + 1.1628  (R²=0.903)
- curve shape: **degrade**

| n_areas | mean capability | mean nnz |
|---:|---:|---:|
| 3 | 0.9467 | 336 |
| 6 | 0.8300 | 896 |
| 9 | 0.7567 | 1648 |
| 12 | 0.6967 | 2752 |
| 15 | 0.5833 | 3936 |
| 18 | 0.5733 | 5568 |
| 21 | 0.5300 | 7184 |
| 24 | 0.6333 | 8976 |

## Interpretation

| directed shape | reading |
|---|---|
| healthy | directed credit recovers capability as #areas grow |
| plateau | directed credit flattens the degrade curve |
| degrade | directed credit still degrades (no mitigation) |

Either outcome is informative; neither reopens frozen G4 NO-GO.

## How to run

```bash
cargo run --locked --release -p binn-lab --bin r2 -- \
--enable-r2 --credit --quick --out results/r2_credit_scaling_quick.md
cargo run --locked --release -p binn-lab --bin r2 -- \
--enable-r2 --credit --out results/r2_credit_scaling.md
```
