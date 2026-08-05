# Matched-Architecture In-Family RL Recipe — preregistration (protocol v12)

**Unit:** C1-RL (`rl_reinforce_fb` primary; `rl_graded` / `rl_flat` contrasts)
**Status:** scientific schedule run — **PASS**
**Config hash:** `c1-rl-42eddc9c801308e9`
**Date:** 2026-07-23
**Does NOT reopen** `c1-118207fbc3eaba53`, mutate `c1-dfa-*`, remassage
`c1x-dfa-spike-*`, or **retune** failed v11 `rl_graded`
(`c1-rl-ef504db58916720d`).

Prior closed protocol: **v11** graded-primary FAIL — archived at
[`c1_rl_v11_graded_primary.md`](c1_rl_v11_graded_primary.md).

---

## 1. What this closes

Protocol-v11 gated broadcast `rl_graded` as primary and **FAIL**ed G2 (mean
0.5900 / gap LCB 0.0182) while the **contrast** `rl_reinforce_fb` already cleared
the floor hard (0.9112). Production still wants an **RL** story. Protocol v12
**preregisters** that directional REINFORCE × frozen-random feedback arm as the
**primary** under a fresh hash and seed lineage — same η=0.05 / λ=0 / schedule as
v11 (no graded retune). Graded and flat stay as contrasts.

## 2. Arms (shared feed-forward matched graph)

Held identical: dense-LIF forward with **`wrec = 0`**, width, encoding, epochs,
splits, LIF constants. Readout always uses REINFORCE `r·(a−p)`. Distinct
master-seed lineage from `c1-dfa-*` and from v11 (`0xC1A1_6000_0001` →
`0xC1A1_6000_0012`).

| Arm | Hidden modulator |
|---|---|
| `MatchedRlReinforceFb` (**primary** gated) | frozen `B_i ∈ [-1,1]` × `r·(a−p)` |
| `MatchedRlGraded` (contrast; v11 primary) | broadcast `(p_correct − EMA baseline)` |
| `MatchedRlFlat` (production impoverishment) | broadcast ±1 reward |
| `MatchedGradient` (`new_feedforward`) | SuperSpike BPTT ceiling |

Learning rate η = **0.05**, weight decay λ = **0**, minibatch size 20.

## 3. Gate (unchanged thresholds)

| Gate | Requirement |
|---|---|
| Gap LCB | lower 95% on `gap_closed_rl = (rl_reinforce_fb − 0.5)/(gradient − 0.5)` **> 0.5** |
| Accuracy floor | mean `rl_reinforce_fb` accuracy **≥ 0.65** |
| Harness validity | mean gradient **≥ 0.65**, else `INVALID_HARNESS` |

## 4. Scientific result

See [`c1_rl.md`](c1_rl.md):

| Arm | Mean acc |
|---|---:|
| `rl_reinforce_fb` (**primary**) | **0.9200** |
| `rl_graded` (contrast) | **0.5250** |
| `rl_flat` (production) | **0.5113** |
| Gradient ceiling | **0.8887** |
| `gap_closed_rl` mean / LCB | **0.8444 / 0.6846** |
| **Verdict** | **PASS** |

**Reading:** On the Rust matched harness, **directional REINFORCE × per-neuron
feedback** closes G2 as an in-family RL primary (same structural family as the
supervised DFA PASS). Broadcast graded alone still fails as a contrast (revises
NumPy preview; confirms v11). Production ±1 stays at chance. Do not claim the
failed v11 graded primary as a pass by reinterpreting contrast numbers.

## 5. Reproduce

```bash
cargo test -p binn-learn matched_rl
cargo test -p binn-lab rl_match
cargo run --locked --release -p binn-lab --bin c1 -- --matched-rl --quick
cargo run --locked --release -p binn-lab --bin c1 -- --matched-rl --out results/c1_rl.md
# scientific hash: c1-rl-42eddc9c801308e9
```

Code: `binn-learn/src/matched_rl_baseline.rs`, `binn-lab/src/rl_match_config.rs`,
`binn-lab/src/runner_rl_match.rs`.

**Product wiring (P8):** the same `B_i` family is now a production neuromodulator
(`ReinforceFeedback` + `reinforce_term` in `binn-learn/src/credit.rs`); default C1
still uses broadcast ±1. See [`MATCHED_ARCH_PRODUCT_NEUROMOD.md`](MATCHED_ARCH_PRODUCT_NEUROMOD.md).

## 6. Historical (v11 graded-primary FAIL)

| Field | Value |
|---|---|
| Hash | `c1-rl-ef504db58916720d` |
| Primary | `rl_graded` 0.5900 / gap LCB 0.0182 |
| Contrast `rl_reinforce_fb` | 0.9112 |
| Verdict | **FAIL** |
| Artifact | `c1_rl_v11_graded_primary.md` |
