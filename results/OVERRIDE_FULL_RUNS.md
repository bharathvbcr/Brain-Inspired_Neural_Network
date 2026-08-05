# Override full scientific runs (C2–R2)

Index of **non-PILOT** kill-gate override schedules run 2026-07-23 from `binn/`.

**G2 kill-gate unchanged:** `c1-118207fbc3eaba53` still stands. These runs do
**not** reopen Gate G2. Interpretation is exploratory only.

| Exp | Result | Hash | Verdict | Wall (real) | Key metrics | Override doc |
|---|---|---|---|---:|---|---|
| C2 | [`c2_g3.md`](c2_g3.md) | `c2-c45f08841f2f9df9` | **FAIL** (G3) | 1.76 s | local forget 0.895 vs baseline 0.273; high>low ✓ | [`C2_OVERRIDE.md`](C2_OVERRIDE.md) |
| C3 v1 tabular proxy | [`c3_credit_depth.md`](c3_credit_depth.md) | `c3-445aa8de7761d4f4` | **MEASURED** | 0.25 s | tabular D\*=3, teacher-forced oracle D\*=8 | [`C3_OVERRIDE.md`](C3_OVERRIDE.md) |
| R1 | [`r1_composition.md`](r1_composition.md) | `r1-5d30383e334b9cbe` | **ADDITIVE** | 0.21 s | compound_fraction=0.000 (3→10) | [`R1_OVERRIDE.md`](R1_OVERRIDE.md) |
| R2 | [`r2_scaling.md`](r2_scaling.md) | `r2-afafa0fa6f43e3fc` | **NO-GO** (G4) | 0.17 s | shape=degrade; R²=0.985 | [`R2_OVERRIDE.md`](R2_OVERRIDE.md) |

## Commands used

```bash
cd /Users/bharath/Code/parameter_golf/binn
cargo build -p binn-lab --release --bins

cargo run -p binn-lab --release --bin c2 -- --enable-c2 --out results/c2_g3.md
cargo run -p binn-lab --release --bin c3 -- --enable-c3 --out results/c3_credit_depth.md
cargo run -p binn-lab --release --bin r1 -- --enable-r1 --out results/r1_composition.md
cargo run -p binn-lab --release --bin r2 -- --enable-r2 --out results/r2_scaling.md
```

## Brief reading (exploratory)

- **C2:** overlap intervention direction holds, but local forgetting stays far above the disclosed replay baseline → G3 FAIL.
- **C3 v1:** the tabular terminal-reward proxy reaches depth 3 then collapses; the teacher-forced oracle remains perfect. This is not a production-learner D* claim.
- **R1:** hub composition does not beat matched additive coupling → ADDITIVE.
- **R2:** capability falls with #areas (healthy log-linear degrade) → G4 NO-GO / no next OOM under this setup.
