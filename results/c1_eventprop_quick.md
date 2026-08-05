# BINN matched-architecture EventProp H2H (C1-EVENTPROP)

**Does not reopen protocol-v2:** hash `c1-118207fbc3eaba53` and every G2 threshold remain unchanged. Does **not** mutate frozen `c1-match-5dc6822e71229e9e`, `c1-dfa-*`, or `c1-rl-*`. This is protocol **v28** with a fresh `c1-eventprop-*` hash. Mechanism: **discrete EventProp-style spike-triggered adjoint** vs SuperSpike BPTT on the recurrent dense-LIF matched coincidence forward (rule-only; same architecture).

- schedule: **PILOT (development only — not a scientific verdict)**
- config hash: `c1-eventprop-d664f18d1390a416`
- protocol version: 28
- seeds: 5
- train/test: 24/16
- hidden / epochs / β: 64 / 20 / 5.0
- gradient lr (shared): 0.0500
- chance baseline: 0.5

## Results

| arm | mean accuracy | variance |
|---|---:|---:|
| `MATCHED_ARCH_EVENTPROP` (EventProp-style spike adjoint) | 0.5000 | 0.000000 |
| `MATCHED_ARCH_GRADIENT_CEILING` (SuperSpike BPTT ceiling) | 0.7000 | 0.043750 |

- `gap_closed_eventprop` mean: **0.0000**  (var 0.000000)
- lower 95% CB (z=1.96): **0.0000**  (needs > 0.50)
- accuracy floor (matched-eventprop ≥ 0.65): not met
- harness validity (matched-gradient ≥ 0.65): met
- **verdict: PILOT**

## Per-seed

| seed | eventprop | gradient | gap_closed_eventprop |
|---:|---:|---:|---:|
| 11400783698474794004 | 0.5000 | 0.7500 | 0.0000 |
| 4354473506008332331 | 0.5000 | 0.7500 | 0.0000 |
| 15755470337203139646 | 0.5000 | 0.5000 | 0.0000 |
| 8709160144732483669 | 0.5000 | 1.0000 | 0.0000 |
| 1663413177097481320 | 0.5000 | 0.5000 | 0.0000 |

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
