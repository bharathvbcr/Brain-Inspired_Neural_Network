# BINN credit mechanism diagnostic (C1-MECH)

**claim_axis:** Mechanism measurement (recording)
**object_under_test:** One-step update usefulness on frozen matched dense-LIF
**may_claim:** Relative loss-drop / eligibility-energy across credit rules
**must_not_claim:** Gate G2 reopen; neuromorphic SOTA; “local learning impossible”

Does **not** mutate frozen hashes `c1-118207fbc3eaba53`, `c1-match-5dc6822e71229e9e`, `c1-dfa-*`, or `c1-rl-*`.

- schedule: **SCIENTIFIC**
- config hash: `c1-mech-adfc5d6fd9e48e02`
- protocol version: 25
- seeds: 20
- probes/seed: 64
- hidden / β: 128 / 5.0
- forward: feed-forward matched dense-LIF (wrec=0)
- warm-start: 30 SuperSpike epochs on probe set, then freeze

## Primary metrics (means)

| arm | loss_drop | loss_drop_rotate | elig_energy_capture |
|---|---:|---:|---:|
| `broadcast_pm1` | -0.380666 | -0.021800 | 1.0000 |
| `graded_broadcast` | 0.003426 | 0.000872 | 1.0000 |
| `dfa` | -0.438718 | -0.046067 | 1.0000 |
| `rl_reinforce_fb` | -0.438718 | -0.045130 | 1.0000 |
| `superspike` | 0.002436 | -0.000835 | 1.0000 |

`loss_drop` = BCE decrease after a **unit-norm** one-step `win` update. `loss_drop_rotate` shuffles the same-norm vector (direction control). `elig_energy_capture` = fraction of SuperSpike ‖∇L‖² on synapses with |E|>ε (shared E; identical across arms).

## Secondary (not headline)

| arm | cosine(Δw, −∇L_SS) | sign_agree |
|---|---:|---:|
| `broadcast_pm1` | 0.1193 | 0.4975 |
| `graded_broadcast` | 0.5342 | 0.4758 |
| `dfa` | -0.0325 | 0.5020 |
| `rl_reinforce_fb` | -0.0325 | 0.5020 |
| `superspike` | 1.0000 | 1.0000 |

## Per-seed loss_drop

| seed | broadcast_pm1 | graded_broadcast | dfa | rl_reinforce_fb | superspike |
|---:|---:|---:|---:|---:|---:|
| 11400783657817504788 | 0.018782 | 0.035503 | 0.035503 | 0.035503 | 0.035503 |
| 4354473546929731627 | -0.522446 | 0.000000 | -1.286154 | -1.286154 | 0.000000 |
| 15755470287973348414 | -0.402464 | 0.000000 | 0.000000 | 0.000000 | 0.000000 |
| 8709160177085575253 | -0.432824 | 0.000241 | -0.119488 | -0.119488 | 0.000000 |
| 1663413153590176872 | -0.333715 | 0.001013 | 0.000000 | 0.000000 | 0.000000 |
| 13063847081985108095 | -0.389238 | 0.005529 | 0.000000 | 0.000000 | 0.000000 |
| 6018099921050756242 | -0.299610 | 0.000000 | 0.000000 | 0.000000 | 0.000000 |
| 17418529348387070121 | -0.195614 | 0.000000 | 0.000000 | 0.000000 | 0.000000 |
| 10372782153160088764 | -0.455066 | 0.000000 | -1.027622 | -1.027622 | 0.000000 |
| 3326472179711269075 | -0.295244 | 0.000000 | 0.000000 | 0.000000 | 0.000000 |
| 14727609795615085798 | -0.474914 | 0.000000 | -1.131490 | -1.131490 | 0.000000 |
| 7681299684928639229 | -0.511486 | 0.000000 | 0.000000 | 0.000000 | 0.000000 |
| 635552386555333904 | -0.438875 | 0.000000 | -1.158909 | -1.158909 | 0.000000 |
| 12035986315017373991 | -0.629589 | 0.000000 | -1.546012 | -1.546012 | 0.000000 |
| 4990234893542573370 | -0.296681 | 0.000000 | 0.000000 | 0.000000 | 0.000000 |
| 16390668856364351825 | -0.545773 | 0.000000 | -1.183314 | -1.183314 | 0.000000 |
| 9344921661003152740 | -0.468520 | 0.004520 | -1.356863 | -1.356863 | 0.000000 |
| 2298611412676426107 | -0.452504 | 0.000000 | 0.000000 | 0.000000 | 0.000000 |
| 13699608291091887502 | -0.495112 | 0.000000 | 0.000662 | 0.000662 | 0.000000 |
| 6653298317710176677 | 0.007574 | 0.021712 | -0.000666 | -0.000666 | 0.013212 |

## Reproduce

```bash
cargo run --locked --release -p binn-lab --bin c1 -- --matched-mech --quick \
--out results/c1_credit_mech.md
cargo run --locked --release -p binn-lab --bin c1 -- --matched-mech \
--out results/c1_credit_mech.md
```
