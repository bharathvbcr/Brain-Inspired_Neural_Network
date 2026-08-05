# R2-credit — directed-credit mitigation probe

**Does not reopen Gate G4 NO-GO** under frozen `r2-afafa0fa6f43e3fc`, nor Gate G2 FAIL under `c1-118207fbc3eaba53`. This is a **separate** directed-credit hypothesis (`r2-credit-*`): same disclosed #areas grid as R2, credit arms = graded DFA / REINFORCE×frozen B (optional 1-seed ±1 smoke). Frozen R2 is **not** remassaged.

**Kill-gate override:** exploratory post-G2 branch; requires `--enable-r2 --credit` (or `--override-g2-for r2`).

- config hash: `r2-credit-eaa83da10229dd22`
- protocol version: 1
- quick/PILOT: true
- directed seeds: 2
- disclosed sweep: 3..= 9 step 3 (matches frozen R2 grid)
- mitigation reading: **PILOT — directed credit recovers (healthy) on at least one arm**

> PILOT only: the quick schedule validates the harness and cannot alone support a scientific mitigation claim.

## Arm `graded-dfa` (n_seeds=2)

- fit: capability ≈ -0.0274 · ln(n) + 0.6631  (R²=0.133)
- curve shape: **plateau**

| n_areas | mean capability | mean nnz |
|---:|---:|---:|
| 3 | 0.6167 | 72 |
| 6 | 0.6583 | 200 |
| 9 | 0.5750 | 376 |

## Arm `reinforce-fb` (n_seeds=2)

- fit: capability ≈ 0.0507 · ln(n) + 0.5141  (R²=0.265)
- curve shape: **healthy**

| n_areas | mean capability | mean nnz |
|---:|---:|---:|
| 3 | 0.5500 | 72 |
| 6 | 0.6583 | 200 |
| 9 | 0.5917 | 376 |

## Optional ±1 smoke control (1 seed)

_Harness sanity only — not a remassage of frozen G4. Expected to still degrade under broadcast credit._

## Arm `broadcast-pm1` (n_seeds=1)

- fit: capability ≈ -0.0677 · ln(n) + 0.8537  (R²=0.611)
- curve shape: **degrade**

| n_areas | mean capability | mean nnz |
|---:|---:|---:|
| 3 | 0.7667 | 72 |
| 6 | 0.7667 | 200 |
| 9 | 0.6833 | 376 |

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
