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
| multi-pass (`matched_epochs`) | 80 | exposure parity with BPTT schedule |
| richer encoder (`burst_count` × `burst_stride`) | 3 × 2 ticks | repeated latency spikes into membrane k-WTA |
| calibrated k-WTA | winner-floor (all finite `v`) | reliable scores after reset |
| denser assembly (`p_sparse`) | 0.70 | more pathways under hard WTA |
| η / λ (DFA arms) | 0.05 / 0.00 | graded-DFA recipe (not production 0.35) |
| surrogate η (ceiling) | 0.35 | production-scale for harness validity |
| trial isolation | pairing clear + full membrane reset | no cross-trial residue |

- protocol version: 10
- schedule: **SCIENTIFIC**
- seeds: 20
- positive control mean: 1.0000
- mean activity sparsity: 0.0156 (band [0.0050, 0.0300])
- true-dfa mean / gap LCB: 0.6513 / 0.0733
- surrogate-gradient mean: 0.7238
- hybrid-stdp-dfa mean: 0.5550
- **verdict: FAIL**

## Arm hashes

| arm | hash | mean accuracy | variance |
|---|---|---:|---:|
| `true-dfa` | `c1x-dfa-spike-true-dfa-a911e793e590b0ed` | 0.6513 | 0.026215 |
| `hybrid-stdp-dfa` | `c1x-dfa-spike-hybrid-stdp-dfa-e36521024bfa6e61` | 0.5550 | 0.035368 |
| `surrogate-gradient` | `c1x-dfa-spike-surrogate-gradient-296dc39a4790814c` | 0.7238 | 0.076939 |

## Per-seed

| seed | true-dfa | hybrid-stdp-dfa | surrogate-grad | gap_closed |
|---:|---:|---:|---:|---:|
| 11400783733600254996 | 0.7250 | 0.5000 | 0.3500 | 0.0000 |
| 4354473609861134379 | 0.5000 | 0.5000 | 0.5000 | 0.0000 |
| 15755470509717746750 | 0.5000 | 0.5000 | 0.5000 | 0.0000 |
| 8709160385945071701 | 0.7250 | 0.5000 | 0.3500 | 0.0000 |
| 1663412937281596520 | 0.5000 | 0.3500 | 0.9000 | 0.0000 |
| 13063846887319136383 | 1.0000 | 1.0000 | 0.9000 | 1.0000 |
| 6018099713499751570 | 0.7250 | 1.0000 | 0.9000 | 0.5625 |
| 17418529265423671465 | 0.7250 | 0.5000 | 0.5000 | 0.0000 |
| 10372782091570994364 | 0.5000 | 0.5000 | 1.0000 | 0.0000 |
| 3326472242709780691 | 0.7250 | 0.5000 | 1.0000 | 0.4500 |
| 14727609880088302822 | 0.7250 | 0.7250 | 1.0000 | 0.4500 |
| 7681299756147855613 | 0.7250 | 0.5000 | 0.9000 | 0.5625 |
| 635552582362287376 | 0.7250 | 0.5000 | 1.0000 | 0.4500 |
| 12035986532299163943 | 0.5000 | 0.5000 | 0.3500 | 0.0000 |
| 4990234685521806650 | 1.0000 | 0.3500 | 0.9000 | 1.0000 |
| 16390668635492237649 | 0.7250 | 0.8250 | 0.3500 | 0.0000 |
| 9344921461773778276 | 0.5000 | 0.3250 | 0.3500 | 0.0000 |
| 2298611338001103227 | 0.5000 | 0.5000 | 1.0000 | 0.0000 |
| 13699608237924824462 | 0.5000 | 0.5000 | 1.0000 | 0.0000 |
| 6653298388996501925 | 0.5000 | 0.5250 | 0.7250 | 0.0000 |

## Interpretation contract

- True graded DFA tests whether the matched dense-LIF recipe can express on LatencyEncoder + k-WTA after disclosed substrate knobs.
- Hybrid STDP×DFA is a labeled contrast to frozen `c1x-dfa-exact-forward-*`.
- Gap uses chance baseline `(dfa − 0.5)/(grad − 0.5)` with unchanged G2 bars.
- No outcome reopens canonical protocol-v2 G2 or mutates `c1-dfa-*`.
