# R2 / U17 — scaling curve (Gate G4 DECISION)

**Kill-gate override:** this run is an **exploratory post-G2 branch**. Gate G2 FAIL under `c1-118207fbc3eaba53` still stands. R2 does **not** reopen the v8 kill-gate; it requires `--enable-r2` / `--override-g2-for r2`.

**Gate G4 is DECISION, not kill.** A GO (healthy, non-plateauing) justifies exploring the *next order of magnitude* of areas. It is **not** proof the curve continues to 10⁴–10⁶ areas (v7 F6 / v8 U17).

- config hash: `r2-afafa0fa6f43e3fc`
- protocol version: 1
- quick/PILOT: false
- seeds: 8
- disclosed sweep: 3..= 24 step 3
- fit: capability ≈ -0.1924 · ln(n) + 1.1673  (R²=0.985)
- curve shape: **degrade**
- G4 decision: **NO-GO**

## Capability vs #areas

| n_areas | mean capability | mean nnz |
|---:|---:|---:|
| 3 | 0.9608 | 336 |
| 6 | 0.8354 | 896 |
| 9 | 0.7371 | 1648 |
| 12 | 0.6762 | 2752 |
| 15 | 0.6292 | 3936 |
| 18 | 0.5917 | 5568 |
| 21 | 0.5937 | 7184 |
| 24 | 0.5825 | 8976 |

## Go / no-go interpretation

| shape | G4 reading |
|---|---|
| healthy | GO — justify next OOM of areas (still post-G2 exploratory) |
| plateau | NO-GO — redirect toward edge / continual-learning product |
| degrade | NO-GO — composition cost dominates; do not scale further |

## Fuller sweep (still disclosed; not 10⁴)

```bash
cargo run -p binn-lab --release --bin r2 -- --enable-r2 \
--out results/r2_scaling.md
```
