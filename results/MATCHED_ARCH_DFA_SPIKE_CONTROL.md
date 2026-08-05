# Spiking-path DFA rescue — preregistration (protocol v10)

**Unit:** C1x-DFA-SPIKE (true graded DFA on LatencyEncoder + k-WTA with disclosed rescue knobs)
**Status:** scientific schedule run — **FAIL**
**Primary hash:** `c1x-dfa-spike-true-dfa-a911e793e590b0ed`
**Date:** 2026-07-23
**Does NOT reopen** `c1-118207fbc3eaba53`, dense-LIF `c1-dfa-*`, or frozen credit
`c1x-dfa-exact-forward-*` / `c1x-iso-s-dfa-*`.

---

## 1. What this asks

Dense-LIF matched DFA (`c1-dfa-c8c4fe0899908b84`) **PASS**es. Exact-forward credit
DFA on LatencyEncoder + hard k-WTA **FAIL**s G2. This protocol tests whether
**true graded DFA** (σ′ eligibility, no STDP absorb) plus disclosed substrate
knobs can produce a **spiking-substrate positive** under unchanged G2 bars.

## 2. Arms (shared LatencyEncoder + sparse assembly forward)

| Arm | Update |
|---|---|
| `true-dfa` (primary) | graded output error × fixed-random DFA × σ′(score)·pre; **no** STDP |
| `hybrid-stdp-dfa` | STDP eligibility × DFA credit (frozen credit-DFA mechanism) |
| `surrogate-gradient` | same-forward straight-through ceiling |

## 3. Disclosed rescue knobs

| Knob | Value |
|---|---|
| multi-pass | `matched_epochs = bptt_epochs` (80 scientific) |
| richer encoder | latency burst `3 × 2` ticks |
| calibrated k-WTA | winner-floor (all finite `v`) |
| denser assembly | `p_sparse = 0.70` |
| η / λ (DFA) | `0.05` / `0` |
| surrogate η | `0.35` |
| trial isolation | pairing clear + full membrane reset |

G2 thresholds unchanged (gap LCB > 0.5, accuracy ≥ 0.65, gradient floor ≥ 0.65,
PC ≥ 0.90, sparsity ∈ [0.005, 0.03]).

## 4. Scientific result

See [`credit_dfa_spike.md`](credit_dfa_spike.md):

| Arm | Mean acc |
|---|---:|
| true-dfa (primary) | **0.6513** |
| hybrid-stdp-dfa | 0.5550 |
| surrogate-gradient | **0.7238** |
| `gap_closed_dfa` mean / LCB | 0.2238 / **0.0733** |
| PC / sparsity | 1.0000 / 0.0156 |
| **Verdict** | **FAIL** |

Harness valid. True DFA barely clears the accuracy floor and beats hybrid STDP×DFA,
but **does not** close the gap vs the same-forward surrogate (LCB ≪ 0.5). High
seed variance (many seeds remain at chance).

Quick/PILOT (`credit_dfa_spike_quick.md`): true-dfa 0.6125 / surrogate 0.7625 —
not a scientific verdict.

## 5. Interpretation

One honest spiking-path rescue attempt **does not** yield a substrate positive.
The dense-LIF DFA PASS remains the rule-isolation result; LatencyEncoder + k-WTA
still handicaps graded+DFA credit even after multi-pass / burst encoder /
winner-floor / denser assembly. Do not reopen v2 or massage thresholds.

## 6. Reproduce

```bash
cargo test --locked -p binn-lab --lib dfa_spike
cargo run --locked --release -p binn-lab --bin credit-assignment -- \
  --dfa-spike --quick --out results/credit_dfa_spike_quick.md
cargo run --locked --release -p binn-lab --bin credit-assignment -- \
  --dfa-spike --out results/credit_dfa_spike.md
```

Code: `binn-lab/src/dfa_spike_config.rs`, `binn-lab/src/runner_dfa_spike.rs`.
