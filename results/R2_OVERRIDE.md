# R2 / U17 — kill-gate override (exploratory; Gate G4 DECISION)

**Status:** harness + unit tests + `--quick` PILOT path + **full scientific
run** (see below). Depends on R1 hub / composition helpers.

**Gate G2 FAIL under `c1-118207fbc3eaba53` still stands.** Enabling R2 does
**not** reopen the v8 kill-gate.

**Gate G4 is DECISION, not kill.** A GO (healthy, non-plateauing curve)
justifies exploring the *next order of magnitude* of areas. It is **not**
proof the curve continues to 10⁴–10⁶ areas (v7 F6 / v8 U17).

## Flags

| Mechanism | Example |
|---|---|
| CLI | `--enable-r2` |
| CLI | `--override-g2-for r2` |
| Env | `BINN_OVERRIDE_G2_FOR=r2` (comma-list / `all` also accepted) |
| Cargo feature (docs only) | `--features enable-r2` (still needs a runtime flag) |

Without one of the runtime overrides, `r2` prints instructions and exits `2`.

## How to run

From `binn/`:

```bash
# Refuse (default — kill-gate intact)
cargo run -p binn-lab --bin r2

# PILOT / quick (tiny disclosed sweep: 3,6,9)
cargo run -p binn-lab --bin r2 -- --enable-r2 --quick \
  --out results/r2_scaling_quick.md

# Fuller disclosed sweep (still not 10⁴; exploratory)
cargo run -p binn-lab --release --bin r2 -- --enable-r2 \
  --out results/r2_scaling.md
```

## Protocol

| Preset | Experiment | Protocol | Config hash | Notes |
|---|---|---:|---|---|
| C1 kill-gate (unchanged) | `c1` | 2 | `c1-118207fbc3eaba53` | Do not reuse |
| R2 default | `r2` | 1 | `r2-afafa0fa6f43e3fc` | 3..=24 step 3 |
| R2 quick (PILOT) | `r2` | 1 | `r2-a35e33f9937b57bd` | 3,6,9 |

## What R2 measures (U17 / G4)

1. **Capability vs #areas** — hub-composed noisy-majority accuracy over a
   disclosed sweep (not hundreds of areas on first landing; fuller command above).
2. **Log-linear fit** — `capability ≈ a · ln(n) + b` with R².
3. **Shape** — `healthy` / `plateau` / `degrade` from first→last and peak.
4. **G4 decision** — GO only for healthy non-PILOT curves; plateau/degrade
   → NO-GO (redirect), not a kill of the whole project.

`--quick` always reports `PILOT` (never a scientific GO/NO-GO alone).

## Full scientific run (2026-07-23)

| Field | Value |
|---|---|
| Result file | [`r2_scaling.md`](r2_scaling.md) |
| Config hash | `r2-afafa0fa6f43e3fc` |
| Protocol | 1 · scientific (non-PILOT) |
| Seeds | 8 |
| Wall time | **0.17 s** real |
| Curve shape | **degrade** |
| G4 decision | **NO-GO** |
| Fit | capability ≈ −0.1924 · ln(n) + 1.1673 (R²=0.985) |
| Disclosed sweep | 3..=24 step 3 |
| n=3 / n=24 capability | 0.9608 / 0.5825 |

**Scope:** exploratory override only. Does **not** reopen G2
`c1-118207fbc3eaba53`. Degrading curve → do not scale areas further under
this hub/composition setup (G4 redirect, not a project kill).
