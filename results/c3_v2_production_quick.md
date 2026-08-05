# C3 v2 — production-engine credit assignment versus depth

**Exploratory post-G2 preregistration.** Canonical C1 protocol-v2 hash `c1-118207fbc3eaba53` remains a FAIL. C3 v1 remains preserved as a tabular terminal-reward proxy and is not evidence about the production `ThreeFactor` learner.

- protocol version: 2
- verdict: **PILOT**
- seeds: 3
- depth: 1..=4
- train/test per depth×seed: 120/80
- D* accuracy floor: 0.650
- production forward parity: **PASS**

> PILOT only: development seeds validate mechanics and cannot support a scientific D* claim.

## Arm hashes and D*

| arm | hash | D* |
|---|---|---:|
| `broadcast-three-factor` | `c3v2-broadcast-three-factor-fc10827e2776e689` | 1 |
| `rpe-three-factor` | `c3v2-rpe-three-factor-8cabaab122b87343` | 1 |
| `eprop-postsynaptic` | `c3v2-eprop-postsynaptic-653b4bf5754d4449` | none |
| `dfa-fixed-feedback` | `c3v2-dfa-fixed-feedback-839dcf673873f2fd` | none |
| `matched-forward-oracle-gradient-reference` | `c3v2-matched-forward-oracle-gradient-reference-c5a089fda3ae7471` | 4 |

## Accuracy by depth

| depth | arm | mean | variance |
|---:|---|---:|---:|
| 1 | `broadcast-three-factor` | 1.0000 | 0.000000 |
| 1 | `rpe-three-factor` | 1.0000 | 0.000000 |
| 1 | `eprop-postsynaptic` | 0.2500 | 0.007656 |
| 1 | `dfa-fixed-feedback` | 0.2500 | 0.007656 |
| 1 | `matched-forward-oracle-gradient-reference` | 0.9375 | 0.011719 |
| 2 | `broadcast-three-factor` | 0.4917 | 0.002708 |
| 2 | `rpe-three-factor` | 0.4958 | 0.031458 |
| 2 | `eprop-postsynaptic` | 0.2667 | 0.006458 |
| 2 | `dfa-fixed-feedback` | 0.2708 | 0.016302 |
| 2 | `matched-forward-oracle-gradient-reference` | 1.0000 | 0.000000 |
| 3 | `broadcast-three-factor` | 0.3167 | 0.006615 |
| 3 | `rpe-three-factor` | 0.3208 | 0.005833 |
| 3 | `eprop-postsynaptic` | 0.2833 | 0.010208 |
| 3 | `dfa-fixed-feedback` | 0.2375 | 0.008906 |
| 3 | `matched-forward-oracle-gradient-reference` | 1.0000 | 0.000000 |
| 4 | `broadcast-three-factor` | 0.2208 | 0.000052 |
| 4 | `rpe-three-factor` | 0.3083 | 0.012708 |
| 4 | `eprop-postsynaptic` | 0.2625 | 0.001094 |
| 4 | `dfa-fixed-feedback` | 0.2833 | 0.010052 |
| 4 | `matched-forward-oracle-gradient-reference` | 1.0000 | 0.000000 |

## Protocol

Each layer is a real event-engine transition area. A forced `(state, operation)` source spike deposits through CSR synapses; membrane charge selects one hard k-WTA state winner; the winner is force-spiked so production STDP eligibility records the selected transition. Broadcast and RPE arms receive only terminal reward. E-prop transports current downstream-weight signals, DFA uses an immutable random projection, and the matched reference receives oracle per-layer target pulses only after the shared forward rollout.

Every arm shares topology, initialization, frozen examples, forward predictions/winners/charges at initialization, and test non-update checks. Oracle correction pulses are executed after prediction in every training arm; only the matched reference learns from them.
