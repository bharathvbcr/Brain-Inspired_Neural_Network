# BINN spiking-path DFA rescue (`c1x-dfa-spike-*`)

**Canonical C1 is immutable:** protocol-v2 hash `c1-118207fbc3eaba53` and every G2 threshold remain unchanged. This is a **separate** protocol family (`c1x-dfa-spike-*`) and does **not** reopen frozen `c1x-dfa-exact-forward-*` / `c1x-iso-s-dfa-*` or dense-LIF `c1-dfa-*`.

## Mechanism disclosure

| arm | label | update |
|---|---|---|
| `TRUE_GRADED_DFA_SPIKE_RESCUE` | true-dfa (primary) | graded output error × fixed-random DFA feedback × σ′(score) · pre; **no** STDP absorb |
| `HYBRID_STDP_DFA_SPIKE_CONTRAST` | hybrid-stdp-dfa | production STDP eligibility × DFA-projected credit (frozen credit-DFA mechanism) |
| `SURROGATE_GRADIENT_SPIKE_CEILING` | surrogate-gradient | same-forward straight-through ceiling for gap / harness |

## Substrate rescue knobs (disclosed)

| knob | value | role |
|---|---|---|
| multi-pass (`matched_epochs`) | 20 | exposure parity with BPTT schedule |
| richer encoder (`burst_count` × `burst_stride`) | 3 × 2 ticks | repeated latency spikes into membrane k-WTA |
| calibrated k-WTA | winner-floor (all finite `v`) | reliable scores after reset |
| denser assembly (`p_sparse`) | 0.70 | more pathways under hard WTA |
| η / λ (DFA arms) | 0.05 / 0.00 | graded-DFA recipe (not production 0.35) |
| surrogate η (ceiling) | 0.35 | production-scale for harness validity |
| trial isolation | pairing clear + full membrane reset | no cross-trial residue |

- protocol version: 10
- schedule: **PILOT**
- seeds: 5
- positive control mean: 0.9583
- mean activity sparsity: 0.0156 (band [0.0050, 0.0300])
- true-dfa mean / gap LCB: 0.6125 / -0.0920
- surrogate-gradient mean: 0.7625
- hybrid-stdp-dfa mean: 0.5625
- **verdict: PILOT**

## Arm hashes

| arm | hash | mean accuracy | variance |
|---|---|---:|---:|
| `true-dfa` | `c1x-dfa-spike-true-dfa-e6e56030ee7ac4ff` | 0.6125 | 0.071094 |
| `hybrid-stdp-dfa` | `c1x-dfa-spike-hybrid-stdp-dfa-03088df177cedf9b` | 0.5625 | 0.076172 |
| `surrogate-gradient` | `c1x-dfa-spike-surrogate-gradient-c97f0c6383a5a206` | 0.7625 | 0.041797 |

## Per-seed

| seed | true-dfa | hybrid-stdp-dfa | surrogate-grad | gap_closed |
|---:|---:|---:|---:|---:|
| 11400783733605366804 | 0.3125 | 0.5000 | 0.9375 | 0.0000 |
| 4354473609856022571 | 1.0000 | 0.3125 | 0.7500 | 1.0000 |
| 15755470509703984190 | 0.5000 | 0.7500 | 0.5625 | 0.0000 |
| 8709160385958834261 | 0.5000 | 0.9375 | 0.5625 | 0.0000 |
| 1663412937284873320 | 0.7500 | 0.3125 | 1.0000 | 0.5000 |

## Interpretation contract

- True graded DFA tests whether the matched dense-LIF recipe can express on LatencyEncoder + k-WTA after disclosed substrate knobs.
- Hybrid STDP×DFA is a labeled contrast to frozen `c1x-dfa-exact-forward-*`.
- Gap uses chance baseline `(dfa − 0.5)/(grad − 0.5)` with unchanged G2 bars.
- No outcome reopens canonical protocol-v2 G2 or mutates `c1-dfa-*`.
