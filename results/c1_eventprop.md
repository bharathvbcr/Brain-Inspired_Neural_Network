# BINN matched-architecture EventProp H2H (C1-EVENTPROP)

**Does not reopen protocol-v2:** hash `c1-118207fbc3eaba53` and every G2 threshold remain unchanged. Does **not** mutate frozen `c1-match-5dc6822e71229e9e`, `c1-dfa-*`, or `c1-rl-*`. This is protocol **v28** with a fresh `c1-eventprop-*` hash. Mechanism: **discrete EventProp-style spike-triggered adjoint** vs SuperSpike BPTT on the recurrent dense-LIF matched coincidence forward (rule-only; same architecture).

- schedule: **SCIENTIFIC**
- config hash: `c1-eventprop-5bb083d5e88d0ad2`
- protocol version: 28
- seeds: 20
- train/test: 80/40
- hidden / epochs / β: 128 / 80 / 5.0
- gradient lr (shared): 0.0500
- chance baseline: 0.5

## Results

| arm | mean accuracy | variance |
|---|---:|---:|
| `MATCHED_ARCH_EVENTPROP` (EventProp-style spike adjoint) | 0.5000 | 0.000000 |
| `MATCHED_ARCH_GRADIENT_CEILING` (SuperSpike BPTT ceiling) | 0.9150 | 0.021737 |

- `gap_closed_eventprop` mean: **0.0000**  (var 0.000000)
- lower 95% CB (z=1.96): **0.0000**  (needs > 0.50)
- accuracy floor (matched-eventprop ≥ 0.65): not met
- harness validity (matched-gradient ≥ 0.65): met
- **verdict: FAIL**

## Per-seed

| seed | eventprop | gradient | gap_closed_eventprop |
|---:|---:|---:|---:|
| 11400783698133154836 | 0.5000 | 0.9500 | 0.0000 |
| 4354473505674557483 | 0.5000 | 1.0000 | 0.0000 |
| 15755470336878933054 | 0.5000 | 1.0000 | 0.0000 |
| 8709160144386781269 | 0.5000 | 0.7250 | 0.0000 |
| 1663413176759512168 | 0.5000 | 1.0000 | 0.0000 |
| 13063847126729943167 | 0.5000 | 1.0000 | 0.0000 |
| 6018099884191212690 | 0.5000 | 0.5000 | 0.0000 |
| 17418529367395655849 | 0.5000 | 1.0000 | 0.0000 |
| 10372782124890479804 | 0.5000 | 1.0000 | 0.0000 |
| 3326472207309789395 | 0.5000 | 1.0000 | 0.0000 |
| 14727609844688442598 | 0.5000 | 1.0000 | 0.0000 |
| 7681299651961409789 | 0.5000 | 1.0000 | 0.0000 |
| 635552409456233744 | 0.5000 | 0.7250 | 0.0000 |
| 12035986290673633575 | 0.5000 | 1.0000 | 0.0000 |
| 4990234924999853370 | 0.5000 | 0.7250 | 0.0000 |
| 16390668806250807633 | 0.5000 | 1.0000 | 0.0000 |
| 9344921632465108324 | 0.5000 | 0.9500 | 0.0000 |
| 2298611439972956539 | 0.5000 | 1.0000 | 0.0000 |
| 13699608271177332110 | 0.5000 | 1.0000 | 0.0000 |
| 6653298353596641701 | 0.5000 | 0.7250 | 0.0000 |

## Gate (unchanged thresholds)

Primary arm = EventProp. `gap_closed_eventprop = (matched_eventprop − 0.5) / (matched_gradient − 0.5)`, clamped to [0,1]; seeds with `(matched_gradient − 0.5) < g2_min_reference_gap` contribute `closed = 0`. PASS requires gap LCB > 0.5 and mean matched-eventprop ≥ 0.65; mean matched-gradient < 0.65 ⇒ INVALID_HARNESS.

## Honesty / approximations

- **Not neuromorphic HW.** CPU discrete-time simulation only.
- **Not textbook continuous EventProp.** Wunderlich & Pehle (2021) adjoint is for hybrid continuous/discrete dynamics with exact jump conditions. Here the forward is the same discrete hard-threshold LIF as SuperSpike matched-arch; the reverse uses a **hard spike gate** (adjoint jump only when `s_i[t]=1`, jump scale `min(1/max(|I_eff|,ε), JUMP_MAX)` as a discrete stand-in for continuous `1/|du/dt|`) — **not** SuperSpike’s soft `σ'(u)` at every timestep.
- **Cold-start disclosure:** hard spike gating provides **no** hidden-weight gradient on silent timesteps. Under the shared matched init, networks that rarely spike cannot bootstrap the way SuperSpike’s soft surrogate does; exact chance (0.5) is the expected symptom when only the readout bias moves.
- **Rule-only H2H:** identical recurrent `MatchedArch` forward, splits, seeds lineage, epochs, and lr as the SuperSpike ceiling; only the backward credit rule differs.
- **Not a production-learner claim** (GC1-exempt `*_baseline.rs`).

## Reproduce

```bash
cargo run --locked --release -p binn-lab --bin c1 -- --eventprop --quick
cargo run --locked --release -p binn-lab --bin c1 -- --eventprop --out results/c1_eventprop.md
```
