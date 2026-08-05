# C3 v2 — production-engine credit assignment versus depth

**Exploratory post-G2 preregistration.** Canonical C1 protocol-v2 hash `c1-118207fbc3eaba53` remains a FAIL. C3 v1 remains preserved as a tabular terminal-reward proxy and is not evidence about the production `ThreeFactor` learner.

- protocol version: 2
- verdict: **MEASURED**
- seeds: 20
- depth: 1..=8
- train/test per depth×seed: 2000/500
- D* accuracy floor: 0.650
- production forward parity: **PASS**

## Arm hashes and D*

| arm | hash | D* |
|---|---|---:|
| `broadcast-three-factor` | `c3v2-broadcast-three-factor-c97b35589d7791c1` | 3 |
| `rpe-three-factor` | `c3v2-rpe-three-factor-c0469d6f35801d37` | 3 |
| `eprop-postsynaptic` | `c3v2-eprop-postsynaptic-d2b46d8fae0b8981` | none |
| `dfa-fixed-feedback` | `c3v2-dfa-fixed-feedback-9ad49817f517bead` | none |
| `matched-forward-oracle-gradient-reference` | `c3v2-matched-forward-oracle-gradient-reference-5582ce8e7c3d56a9` | 8 |

## Accuracy by depth

| depth | arm | mean | variance |
|---:|---|---:|---:|
| 1 | `broadcast-three-factor` | 1.0000 | 0.000000 |
| 1 | `rpe-three-factor` | 0.8500 | 0.134211 |
| 1 | `eprop-postsynaptic` | 0.1909 | 0.012271 |
| 1 | `dfa-fixed-feedback` | 0.1909 | 0.012271 |
| 1 | `matched-forward-oracle-gradient-reference` | 0.9865 | 0.001732 |
| 2 | `broadcast-three-factor` | 0.9107 | 0.015671 |
| 2 | `rpe-three-factor` | 0.9248 | 0.013905 |
| 2 | `eprop-postsynaptic` | 0.2981 | 0.008549 |
| 2 | `dfa-fixed-feedback` | 0.2605 | 0.008060 |
| 2 | `matched-forward-oracle-gradient-reference` | 1.0000 | 0.000000 |
| 3 | `broadcast-three-factor` | 0.7371 | 0.052049 |
| 3 | `rpe-three-factor` | 0.7742 | 0.026428 |
| 3 | `eprop-postsynaptic` | 0.3342 | 0.010097 |
| 3 | `dfa-fixed-feedback` | 0.3085 | 0.005104 |
| 3 | `matched-forward-oracle-gradient-reference` | 1.0000 | 0.000000 |
| 4 | `broadcast-three-factor` | 0.2615 | 0.003477 |
| 4 | `rpe-three-factor` | 0.6048 | 0.029760 |
| 4 | `eprop-postsynaptic` | 0.2745 | 0.001091 |
| 4 | `dfa-fixed-feedback` | 0.2915 | 0.006486 |
| 4 | `matched-forward-oracle-gradient-reference` | 1.0000 | 0.000000 |
| 5 | `broadcast-three-factor` | 0.2431 | 0.000897 |
| 5 | `rpe-three-factor` | 0.5041 | 0.008036 |
| 5 | `eprop-postsynaptic` | 0.2608 | 0.002544 |
| 5 | `dfa-fixed-feedback` | 0.2811 | 0.004646 |
| 5 | `matched-forward-oracle-gradient-reference` | 1.0000 | 0.000000 |
| 6 | `broadcast-three-factor` | 0.2520 | 0.000350 |
| 6 | `rpe-three-factor` | 0.4158 | 0.013363 |
| 6 | `eprop-postsynaptic` | 0.2572 | 0.000588 |
| 6 | `dfa-fixed-feedback` | 0.2776 | 0.003064 |
| 6 | `matched-forward-oracle-gradient-reference` | 1.0000 | 0.000000 |
| 7 | `broadcast-three-factor` | 0.2505 | 0.000329 |
| 7 | `rpe-three-factor` | 0.3202 | 0.010356 |
| 7 | `eprop-postsynaptic` | 0.2521 | 0.000436 |
| 7 | `dfa-fixed-feedback` | 0.2787 | 0.004522 |
| 7 | `matched-forward-oracle-gradient-reference` | 1.0000 | 0.000000 |
| 8 | `broadcast-three-factor` | 0.2550 | 0.000430 |
| 8 | `rpe-three-factor` | 0.2776 | 0.005282 |
| 8 | `eprop-postsynaptic` | 0.2550 | 0.000499 |
| 8 | `dfa-fixed-feedback` | 0.2614 | 0.000699 |
| 8 | `matched-forward-oracle-gradient-reference` | 1.0000 | 0.000000 |

## Forward and leakage contract

| check | passed |
|---|---|
| topology | true |
| initial weights | true |
| frozen examples | true |
| initial predictions | true |
| initial winners | true |
| initial charges | true |
| forward target independence | true |
| no test updates | true |

## Protocol

Each layer is a real event-engine transition area. A forced `(state, operation)` source spike deposits through CSR synapses; membrane charge selects one hard k-WTA state winner; the winner is force-spiked so production STDP eligibility records the selected transition. Broadcast and RPE arms receive only terminal reward. E-prop transports current downstream-weight signals, DFA uses an immutable random projection, and the matched reference receives oracle per-layer target pulses only after the shared forward rollout.

Every arm shares topology, initialization, frozen examples, forward predictions/winners/charges at initialization, target-independent forward execution, and test non-update checks. Oracle correction pulses are executed after prediction in every training arm; only the matched reference learns from them.
