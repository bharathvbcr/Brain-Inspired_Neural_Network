# BINN matched-architecture in-family RL recipe (C1-RL)

**Does not reopen protocol-v2:** hash `c1-118207fbc3eaba53` and every G2 threshold remain unchanged. This is matched-arch protocol **v11** with a fresh `c1-rl-*` hash. Mechanism: **in-family graded / directional RL modulators** on the dense-LIF matched forward (feed-forward `wrec=0`, matching the NumPy deep preview). Does **not** mutate `c1-dfa-*` or remassage `c1x-dfa-spike-*`.

- schedule: **SCIENTIFIC**
- config hash: `c1-rl-ef504db58916720d`
- protocol version: 11
- seeds: 20
- train/test: 80/40
- hidden / epochs / β: 128 / 80 / 5.0
- gradient lr / local η / λ: 0.0500 / 0.0500 / 0.0000
- chance baseline: 0.5

## Results

| arm | mean accuracy | variance |
|---|---:|---:|
| `MATCHED_ARCH_RL_GRADED` (graded reward, broadcast) | 0.5900 | 0.034632 |
| `MATCHED_ARCH_RL_REINFORCE_FB` (REINFORCE × DFA fb) | 0.9112 | 0.035163 |
| `MATCHED_ARCH_RL_FLAT` (±1 reward, broadcast) | 0.5312 | 0.010058 |
| `MATCHED_ARCH_GRADIENT_CEILING` (SuperSpike BPTT ceiling) | 0.8762 | 0.025031 |

- `gap_closed_rl` mean: **0.1900**  (var 0.153579)
- lower 95% CB (z=1.96): **0.0182**  (needs > 0.50)
- accuracy floor (matched-rl-graded ≥ 0.65): not met
- harness validity (matched-gradient ≥ 0.65): met
- **verdict: FAIL**

## Per-seed

| seed | rl_graded | rl_reinforce_fb | rl_flat | gradient | gap_closed_rl |
|---:|---:|---:|---:|---:|---:|
| 11400783395455400980 | 0.5000 | 1.0000 | 0.5000 | 1.0000 | 0.0000 |
| 4354473774193899563 | 0.5000 | 1.0000 | 0.5000 | 1.0000 | 0.0000 |
| 15755470068493808702 | 0.5000 | 0.5000 | 0.5000 | 0.7250 | 0.0000 |
| 8709160430052438101 | 1.0000 | 1.0000 | 0.5000 | 0.7250 | 1.0000 |
| 1663412925520702568 | 0.5000 | 0.5000 | 0.5000 | 0.7250 | 0.0000 |
| 13063847377968752767 | 0.5000 | 1.0000 | 0.5000 | 0.9500 | 0.0000 |
| 6018099598559110290 | 0.5000 | 1.0000 | 0.5000 | 0.7250 | 0.0000 |
| 17418529635780780201 | 0.9000 | 1.0000 | 0.5000 | 1.0000 | 0.8000 |
| 10372781856371137724 | 0.9000 | 1.0000 | 0.5000 | 0.7250 | 1.0000 |
| 3326471960231729363 | 0.5000 | 1.0000 | 0.5000 | 1.0000 | 0.0000 |
| 14727610091531621606 | 0.5000 | 1.0000 | 0.5000 | 1.0000 | 0.0000 |
| 7681299353578623229 | 0.5000 | 1.0000 | 0.5000 | 0.7250 | 0.0000 |
| 635552673680608528 | 0.5000 | 0.7250 | 0.9000 | 1.0000 | 0.0000 |
| 12035986026617030951 | 0.5000 | 1.0000 | 0.7250 | 1.0000 | 0.0000 |
| 4990235223550412090 | 0.5000 | 1.0000 | 0.5000 | 1.0000 | 0.0000 |
| 16390668559306965329 | 0.5000 | 1.0000 | 0.5000 | 1.0000 | 0.0000 |
| 9344921879408950628 | 1.0000 | 1.0000 | 0.5000 | 1.0000 | 1.0000 |
| 2298611158635821435 | 0.5000 | 1.0000 | 0.5000 | 0.7250 | 0.0000 |
| 13699608552447358350 | 0.5000 | 0.5000 | 0.5000 | 1.0000 | 0.0000 |
| 6653298639128080805 | 0.5000 | 1.0000 | 0.5000 | 0.5000 | 0.0000 |

## Gate (unchanged thresholds)

Primary arm = `rl_graded`. `gap_closed_rl = (matched_rl_graded − 0.5) / (matched_gradient − 0.5)`, clamped to [0,1]; seeds with `(matched_gradient − 0.5) < g2_min_reference_gap` contribute `closed = 0`. PASS requires gap LCB > 0.5 and mean matched-rl-graded ≥ 0.65; mean matched-gradient < 0.65 ⇒ INVALID_HARNESS.

## Recipe notes

- Readout always uses REINFORCE `r·(a−p)` (Bernoulli policy).
- Hidden `rl_graded`: broadcast `(p_correct − baseline)` with EMA baseline (NumPy `0.9·base + 0.1·mean`).
- Hidden `rl_reinforce_fb`: frozen `B_i ∈ [-1,1]` × `r·(a−p)`.
- Hidden `rl_flat`: broadcast ±1 reward (production impoverishment).
- Supervised DFA (`c1-dfa-*`) remains a separate protocol; this suite asks whether an **RL** modulator can close the gap in-family.

## Reproduce

```bash
cargo run --locked --release -p binn-lab --bin c1 -- --matched-rl --quick
cargo run --locked --release -p binn-lab --bin c1 -- --matched-rl --out results/c1_rl.md
```
