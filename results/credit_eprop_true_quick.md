# BINN true surrogate e-prop on exact-forward C1

**Canonical C1 is immutable:** protocol-v2 hash `c1-118207fbc3eaba53` and every G2 threshold remain unchanged. This is a **separate** protocol family (`c1x-eprop-true-*`) and does **not** reopen frozen hybrid `c1x-eprop-exact-forward-fcedc76a80ff0f0e`.

## Mechanism disclosure

| arm | label | eligibility construction |
|---|---|---|
| `TRUE_SURROGATE_DERIVATIVE_EPROP_REFERENCE` | true-surrogate | `e ∝ σ'(score − cutoff) · pre_activity`; `Δw = η · δ · e` with δ from output error transported through readout weights; **no** `ThreeFactor::absorb_spikes` / STDP pairing |
| `HYBRID_STDP_ELIGIBILITY_EPROP_CONTRAST` | hybrid-stdp (contrast) | production STDP eligibility × output-weight-transported M (same as frozen `c1x-eprop-exact-forward-*`) |

- protocol version: 8
- schedule: **PILOT**
- seeds: 5
- matched epochs: 4
- surrogate β: 5.0
- positive control mean: 0.8167
- mean activity sparsity: 0.2500

## Arm hashes

| arm | hash | mean accuracy | variance |
|---|---|---:|---:|
| `true-surrogate-eprop` | `c1x-eprop-true-true-surrogate-eprop-83780ad0dfeb3d77` | 0.7375 | 0.082813 |
| `hybrid-stdp-eprop` | `c1x-eprop-true-hybrid-stdp-eprop-f375ea7756ca0fa2` | 0.7125 | 0.042188 |

## Per-seed

| seed | true-surrogate | hybrid-stdp | true updates | hybrid updates |
|---:|---:|---:|---:|---:|
| 11400783698474794004 | 0.3125 | 0.5625 | 482 | 141888 |
| 4354473506008332331 | 0.9375 | 0.5625 | 596 | 142368 |
| 15755470337203139646 | 0.9375 | 0.9375 | 501 | 141312 |
| 8709160144732483669 | 0.9375 | 0.5625 | 597 | 142368 |
| 1663413177097481320 | 0.5625 | 0.9375 | 404 | 143328 |

## Interpretation contract

- True surrogate e-prop tests whether explicit σ′ eligibility (not STDP absorb) can assign credit on the exact-forward graph.
- Hybrid STDP×M is included only as a labeled contrast to frozen `c1x-eprop-exact-forward-*`; outcomes are not comparable across hash families.
- No outcome reopens canonical protocol-v2 G2.
