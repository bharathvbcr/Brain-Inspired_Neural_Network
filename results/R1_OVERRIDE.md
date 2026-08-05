# R1 / U16 — kill-gate override (exploratory)

**Status:** harness + unit tests + `--quick` PILOT path + **full scientific
run** (see below). Minimal real [`Hub`](../binn-areas/src/hub.rs) (not a full
P5 engine).

**Gate G2 FAIL under `c1-118207fbc3eaba53` still stands.** Enabling R1 does
**not** reopen the v8 kill-gate and does **not** change C1/C2/C3 hashes.

## Flags

| Mechanism | Example |
|---|---|
| CLI | `--enable-r1` |
| CLI | `--override-g2-for r1` |
| Env | `BINN_OVERRIDE_G2_FOR=r1` (comma-list / `all` also accepted) |
| Cargo feature (docs only) | `--features enable-r1` (still needs a runtime flag) |

Without one of the runtime overrides, `r1` prints instructions and exits `2`.

## How to run

From `binn/`:

```bash
# Refuse (default — kill-gate intact)
cargo run -p binn-lab --bin r1

# PILOT / quick (3→5 areas)
cargo run -p binn-lab --bin r1 -- --enable-r1 --quick \
  --out results/r1_composition_quick.md

# Full scientific schedule (3→10 areas; exploratory)
cargo run -p binn-lab --release --bin r1 -- --enable-r1 \
  --out results/r1_composition.md
```

## Protocol

| Preset | Experiment | Protocol | Config hash | Notes |
|---|---|---:|---|---|
| C1 kill-gate (unchanged) | `c1` | 2 | `c1-118207fbc3eaba53` | Do not reuse |
| R1 default | `r1` | 1 | `r1-5d30383e334b9cbe` | 3→10 areas |
| R1 quick (PILOT) | `r1` | 1 | `r1-ab69e1b6eb9b98e6` | 3→5 areas |

## What R1 measures (U16)

1. **Hub composition** — `n` areas wired via `WiringPrior` + hub role.
2. **Compositional task** — noisy majority of `n` latent bits (pooling).
3. **Composed vs additive** — hub-routed readout vs identical learner with
   uniform coupling / no hub structure (matched train/test and learning rates).
4. **Budgets disclosed** — cells (`n × cells_per_area`), CSR nnz, locality,
   train/test counts.

Composition **compounds** when composed accuracy exceeds additive by the
hashed margin on a majority of sweep points.

`--quick` always reports `PILOT`.

## Full scientific run (2026-07-23)

| Field | Value |
|---|---|
| Result file | [`r1_composition.md`](r1_composition.md) |
| Config hash | `r1-5d30383e334b9cbe` |
| Protocol | 1 · scientific (non-PILOT) |
| Seeds | 8 |
| Wall time | **0.21 s** real |
| Verdict | **ADDITIVE** |
| Compound fraction | 0.000 |
| Area sweep | 3..=10 |
| n=3 composed / additive | 0.9653 / 0.9709 |
| n=10 composed / additive | 0.7231 / 0.9000 |

**Scope:** exploratory override only. Does **not** reopen G2
`c1-118207fbc3eaba53`. Hub-composed path never beat the uniform-coupling
additive control by the preregistered margin.
