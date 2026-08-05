# BINN true surrogate e-prop on exact-forward C1

**Canonical C1 is immutable:** protocol-v2 hash `c1-118207fbc3eaba53` and every G2 threshold remain unchanged. This is a **separate** protocol family (`c1x-eprop-true-*`) and does **not** reopen frozen hybrid `c1x-eprop-exact-forward-fcedc76a80ff0f0e`.

## Mechanism disclosure

| arm | label | eligibility construction |
|---|---|---|
| `TRUE_SURROGATE_DERIVATIVE_EPROP_REFERENCE` | true-surrogate | `e ∝ σ'(score − cutoff) · pre_activity`; `Δw = η · δ · e` with δ from output error transported through readout weights; **no** `ThreeFactor::absorb_spikes` / STDP pairing |
| `HYBRID_STDP_ELIGIBILITY_EPROP_CONTRAST` | hybrid-stdp (contrast) | production STDP eligibility × output-weight-transported M (same as frozen `c1x-eprop-exact-forward-*`) |

- protocol version: 8
- schedule: **SCIENTIFIC**
- seeds: 20
- matched epochs: 80
- surrogate β: 5.0
- positive control mean: 0.9975
- mean activity sparsity: 0.6250

## Arm hashes

| arm | hash | mean accuracy | variance |
|---|---|---:|---:|
| `true-surrogate-eprop` | `c1x-eprop-true-true-surrogate-eprop-0e2aeb90d68ac5f9` | 0.7125 | 0.097533 |
| `hybrid-stdp-eprop` | `c1x-eprop-true-hybrid-stdp-eprop-92333bf4bd223098` | 0.7350 | 0.045882 |

## Per-seed

| seed | true-surrogate | hybrid-stdp | true updates | hybrid updates |
|---:|---:|---:|---:|---:|
| 11400783695599795220 | 0.3500 | 0.5000 | 102115 | 37017600 |
| 4354473507402610731 | 1.0000 | 0.6500 | 107221 | 36934400 |
| 15755470334345573438 | 1.0000 | 0.7750 | 106068 | 36998400 |
| 8709160146148388949 | 1.0000 | 1.0000 | 92525 | 36979200 |
| 1663413174192598120 | 0.6750 | 0.8750 | 104444 | 36979200 |
| 13063847128491550847 | 0.3500 | 0.6500 | 76831 | 37011200 |
| 6018099881657853074 | 1.0000 | 0.3000 | 95920 | 36953600 |
| 17418529369123709097 | 1.0000 | 1.0000 | 101712 | 36998400 |
| 10372782122357120188 | 0.3500 | 0.5000 | 89944 | 36985600 |
| 3326472209037842643 | 0.3500 | 0.5750 | 83221 | 37030400 |
| 14727609842121528550 | 0.9000 | 0.6500 | 117291 | 37030400 |
| 7681299653991452925 | 0.3500 | 0.5750 | 82998 | 37017600 |
| 635552407157755152 | 1.0000 | 0.8250 | 119634 | 37120000 |
| 12035986292670122279 | 1.0000 | 0.8500 | 102083 | 37004800 |
| 4990234922734929210 | 0.3500 | 1.0000 | 70597 | 36960000 |
| 16390668808247296337 | 0.9750 | 1.0000 | 98715 | 36979200 |
| 9344921630200184164 | 1.0000 | 0.5250 | 93015 | 37011200 |
| 2298611442002999675 | 0.9000 | 1.0000 | 86290 | 37088000 |
| 13699608268878853518 | 0.3500 | 0.9000 | 108843 | 37011200 |
| 6653298355626684837 | 0.3500 | 0.5500 | 44802 | 37004800 |

## Interpretation contract

- True surrogate e-prop tests whether explicit σ′ eligibility (not STDP absorb) can assign credit on the exact-forward graph.
- Hybrid STDP×M is included only as a labeled contrast to frozen `c1x-eprop-exact-forward-*`; outcomes are not comparable across hash families.
- No outcome reopens canonical protocol-v2 G2.
