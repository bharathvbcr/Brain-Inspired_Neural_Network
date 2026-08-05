# Matched-Architecture DFA Recipe — preregistration (protocol v5)

**Unit:** C1-DFA (directional graded error × DFA on matched dense-LIF)
**Status:** scientific schedule run — **PASS**
**Config hash:** `c1-dfa-c8c4fe0899908b84`
**Date:** 2026-07-23
**Does NOT reopen** `c1-118207fbc3eaba53` or mutate protocol-v4 `c1-match-*`.
**Distinct from** trial-isolation `c1-iso*` (also numbered protocol 5 in that family).

---

## 1. What this closes

Protocol-v4 matched-arch (`c1-match-5dc6822e71229e9e`) showed the production
broadcast ±1 reward rule fails at chance on an identical dense-LIF forward while
SuperSpike BPTT clears the floor — the FAIL is the **rule**, not the path.

NumPy deep probes (`MATCHED_ARCH_DEEP_FINDINGS.md`) identified a still-local fix:
a **directional graded error** delivered through **per-neuron fixed-random
feedback (DFA)**. This protocol ports that recipe to a Rust `*_baseline.rs` arm
and gates it under unchanged G2 thresholds.

## 2. Arms (shared feed-forward matched graph)

Held identical: dense-LIF forward with **`wrec = 0`** (feed-forward, matching the
NumPy preview), width, encoding, epochs, splits, seed lineage, LIF constants.

| Arm | Rule |
|---|---|
| `MatchedGradient` (`new_feedforward`) | SuperSpike BPTT ceiling |
| `MatchedBroadcastErr` | supervised error `-(p−y)` broadcast to all hidden |
| `MatchedDfa` (primary gated) | same graded error × frozen random feedback `B_i ∈ [-1,1]` |

Learning rate η = **0.05** (graded error is denser than ±1 reward; production
η=0.35 destabilizes this rule). Weight decay λ = **0**. Minibatch size 20.

## 3. Gate (unchanged thresholds)

| Gate | Requirement |
|---|---|
| Gap LCB | lower 95% on `gap_closed_dfa = (dfa − 0.5)/(gradient − 0.5)` **> 0.5** |
| Accuracy floor | mean DFA accuracy **≥ 0.65** |
| Harness validity | mean gradient **≥ 0.65**, else `INVALID_HARNESS` |

## 4. Scientific result

See [`c1_dfa.md`](c1_dfa.md):

| Arm | Mean acc |
|---|---:|
| DFA (primary) | **0.9387** |
| Broadcast graded error | **0.9863** |
| Gradient ceiling | **0.8963** |
| `gap_closed_dfa` mean / LCB | **0.8500 / 0.6894** |
| **Verdict** | **PASS** |

On coincidence (linearly separable), broadcast graded error also clears the floor
— locality is not required here. Nonlinearity is where locality bites (P3).

## 5. Spiking-substrate note

On LatencyEncoder + k-WTA + single-pass (credit-assignment `dfa-exact-forward`,
hashes `c1x-dfa-exact-forward-*` / `c1x-iso-dfa-exact-forward-*`), DFA does **not**
clear G2. The k-WTA / single-pass path re-introduces the handicap even with a
graded directional signal. See `credit_assignment_iso_SUMMARY.md`.

**P4 follow-up (2026-07-23):** one honest rescue under `c1x-dfa-spike-*`
(true graded DFA + multi-pass + burst encoder + winner-floor + denser assembly)
still **FAIL**s — true-dfa 0.6513 / gap LCB 0.0733 (surrogate 0.7238). See
[`MATCHED_ARCH_DFA_SPIKE_CONTROL.md`](MATCHED_ARCH_DFA_SPIKE_CONTROL.md) /
[`credit_dfa_spike.md`](credit_dfa_spike.md). Do not reopen v2.

## 6. Reproduce

```bash
cargo test -p binn-learn matched_dfa
cargo run --locked --release -p binn-lab --bin c1 -- --matched-dfa --quick
cargo run --locked --release -p binn-lab --bin c1 -- --matched-dfa --out results/c1_dfa.md
```

Code: `binn-learn/src/matched_dfa_baseline.rs`, `binn-lab/src/dfa_match_config.rs`,
`binn-lab/src/runner_dfa_match.rs`.
