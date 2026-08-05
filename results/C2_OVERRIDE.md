# C2 / U14 — kill-gate override (exploratory)

**Status:** harness + unit/integration tests + `--quick` PILOT path + **full
scientific run** (see below).

**Gate G2 FAIL under `c1-118207fbc3eaba53` still stands.** Enabling C2 does
**not** reopen the v8 kill-gate, does **not** schedule C3/R1/R2/Hub, and does
**not** change C1 / U-NEG defaults.

This branch exists only because leadership explicitly overrode the plan’s
“do not build C2” kill-gate **for U14/C2 behind a flag**.

## Flags

| Mechanism | Example |
|---|---|
| CLI | `--enable-c2` |
| CLI | `--override-g2-for c2` |
| Env | `BINN_OVERRIDE_G2_FOR=c2` |
| Cargo feature (docs only) | `--features enable-c2` (still needs a runtime flag) |

Without one of the runtime overrides, `c2` prints instructions and exits `2`.

## How to run

From `binn/`:

```bash
# Refuse (default — kill-gate intact)
cargo run -p binn-lab --bin c2

# PILOT / quick (CI smoke; never a scientific G3 PASS/FAIL alone)
cargo run -p binn-lab --bin c2 -- --enable-c2 --quick \
  --out results/c2_g3_quick.md

# Full scientific schedule (exploratory; still post-kill-gate)
cargo run -p binn-lab --release --bin c2 -- --enable-c2 \
  --out results/c2_g3.md
```

## Protocol

| Preset | Experiment | Protocol | Config hash | Notes |
|---|---|---:|---|---|
| C1 kill-gate (unchanged) | `c1` | 2 | `c1-118207fbc3eaba53` | Do not reuse |
| C2 default | `c2` | 1 | `c2-c45f08841f2f9df9` | New protocol |
| C2 quick (PILOT) | `c2` | 1 | `c2-ddc6176952829d90` | CI / smoke |

## What C2 measures (U14 / G3)

1. **Class-incremental stream** — no task IDs in the learner API; stream stores
   no raw replay buffer (probes regenerated from seed).
2. **Forgetting curve** — relative drop on earlier classes after later phases.
3. **Overlap interventions** (mechanistic): `force-high`, `force-low`,
   `shuffle-overlap` while holding k-WTA activity fixed.
4. **Matched baseline** — labeled `C2_CAPACITY_REPLAY_GRADIENT_BASELINE` in
   `binn-learn/src/c2_replay_baseline.rs` (GC1-exempt; may store raw examples
   under a disclosed capacity).

**Done-when (G3):** local forgetting below the replay-matched baseline **and**
force-high forgetting > force-low forgetting (predicted direction).

`--quick` always reports `PILOT`.

## Full scientific run (2026-07-23)

| Field | Value |
|---|---|
| Result file | [`c2_g3.md`](c2_g3.md) |
| Config hash | `c2-c45f08841f2f9df9` |
| Protocol | 1 · scientific (non-PILOT) |
| Seeds | 10 |
| Wall time | **1.76 s** real |
| G3 verdict | **FAIL** |
| Local forgetting | 0.8948 |
| Replay baseline forgetting | 0.2725 |
| Local below baseline | false |
| Force-high / force-low / shuffle | 0.8360 / 0.1735 / 0.6479 |
| Intervention direction (high > low) | true |

**Scope:** exploratory override only. Does **not** reopen G2
`c1-118207fbc3eaba53`. Directional overlap prediction held; capacity/forgetting
vs replay baseline did not meet G3.
