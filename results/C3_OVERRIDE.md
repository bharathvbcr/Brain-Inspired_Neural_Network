# C3 / U15 — kill-gate override (exploratory)

**Status:** harness + unit/integration tests + `--quick` PILOT path + **full
scientific run** (see below).

**Gate G2 FAIL under `c1-118207fbc3eaba53` still stands.** Enabling C3 does
**not** reopen the v8 kill-gate, does **not** change C1 protocol-v2 or C2
hashes, and does **not** license P3 by default.

This branch exists only as an exploratory post-kill-gate credit-depth **tabular
proxy** (U15), behind the same override pattern as C2. It does not instantiate
the production event engine or `ThreeFactor`; production-faithful depth testing
is C3 v2 in [`CREDIT_ASSIGNMENT_PREREGISTRATION.md`](CREDIT_ASSIGNMENT_PREREGISTRATION.md).

## Flags

| Mechanism | Example |
|---|---|
| CLI | `--enable-c3` |
| CLI | `--override-g2-for c3` |
| Env | `BINN_OVERRIDE_G2_FOR=c3` (comma-list / `all` also accepted) |
| Cargo feature (docs only) | `--features enable-c3` (still needs a runtime flag) |

Without one of the runtime overrides, `c3` prints instructions and exits `2`.

## How to run

From `binn/`:

```bash
# Refuse (default — kill-gate intact)
cargo run -p binn-lab --bin c3

# PILOT / quick (CI smoke; never a scientific D* claim alone)
cargo run -p binn-lab --bin c3 -- --enable-c3 --quick \
  --out results/c3_credit_depth_quick.md

# Full scientific schedule (exploratory; still post-kill-gate)
cargo run -p binn-lab --release --bin c3 -- --enable-c3 \
  --out results/c3_credit_depth.md
```

## Protocol

| Preset | Experiment | Protocol | Config hash | Notes |
|---|---|---:|---|---|
| C1 kill-gate (unchanged) | `c1` | 2 | `c1-118207fbc3eaba53` | Do not reuse |
| C2 default (unchanged) | `c2` | 1 | `c2-c45f08841f2f9df9` | Do not reuse |
| C3 default | `c3` | 1 | `c3-445aa8de7761d4f4` | New protocol |
| C3 quick (PILOT) | `c3` | 1 | `c3-adf27f8ffc4185ca` | CI / smoke |

Hashes are pinned in `binn-lab` unit tests; they must never collide with C1/C2.

## What C3 measures (U15)

1. **Compositional depth sweep** — synthetic transition chains from
   `binn_data::CreditDepthTask` / `TemporalClassification::with_depth`.
2. **Tabular terminal-reward proxy** — eligibility-decayed, three-factor-style
   updates; not the production `ThreeFactor` implementation.
3. **Teacher-forced oracle reference** — labeled
   `C3_V1_ORACLE_TEACHER_FORCED_REFERENCE`; supervised oracle next-state at
   every layer, not a production gradient learner.
4. **`D*`** — max depth where local mean accuracy ≥ the hashed floor.

`--quick` always reports `PILOT`.

## Full scientific run (2026-07-23)

| Field | Value |
|---|---|
| Result file | [`c3_credit_depth.md`](c3_credit_depth.md) |
| Config hash | `c3-445aa8de7761d4f4` |
| Protocol | 1 · scientific (non-PILOT) |
| Seeds | 10 |
| Wall time | **0.25 s** real |
| Verdict | **MEASURED** |
| D* local | **3** |
| D* teacher-forced oracle | **8** |
| Acc floor | 0.650 |
| Depth 3 tabular-local / oracle | 0.9460 / 1.0000 |
| Depth 4 tabular-local / oracle | 0.2877 / 1.0000 |

**Scope:** exploratory override only. Does **not** reopen G2
`c1-118207fbc3eaba53`. The tabular proxy collapses between depth 3 and 4 while
the teacher-forced oracle stays perfect through depth 8. This result is not
evidence that production `ThreeFactor` has D*=3.
