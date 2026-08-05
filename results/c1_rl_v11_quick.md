# BINN matched-architecture in-family RL recipe (C1-RL)

**Does not reopen protocol-v2:** hash `c1-118207fbc3eaba53` and every G2 threshold remain unchanged. This is matched-arch protocol **v11** with a fresh `c1-rl-*` hash. Mechanism: **in-family graded / directional RL modulators** on the dense-LIF matched forward (feed-forward `wrec=0`, matching the NumPy deep preview). Does **not** mutate `c1-dfa-*` or remassage `c1x-dfa-spike-*`.

- schedule: **PILOT (development only — not a scientific verdict)**
- config hash: `c1-rl-8f65af11eb7af21d`
- protocol version: 11
- seeds: 5
- train/test: 48/24
- hidden / epochs / β: 128 / 60 / 5.0
- gradient lr / local η / λ: 0.0500 / 0.0500 / 0.0000
- chance baseline: 0.5

## Results

| arm | mean accuracy | variance |
|---|---:|---:|
| `MATCHED_ARCH_RL_GRADED` (graded reward, broadcast) | 0.6000 | 0.050000 |
| `MATCHED_ARCH_RL_REINFORCE_FB` (REINFORCE × DFA fb) | 0.7000 | 0.075000 |
| `MATCHED_ARCH_RL_FLAT` (±1 reward, broadcast) | 0.5000 | 0.000000 |
| `MATCHED_ARCH_GRADIENT_CEILING` (SuperSpike BPTT ceiling) | 0.9167 | 0.013889 |

- `gap_closed_rl` mean: **0.2000**  (var 0.200000)
- lower 95% CB (z=1.96): **-0.1920**  (needs > 0.50)
- accuracy floor (matched-rl-graded ≥ 0.65): not met
- harness validity (matched-gradient ≥ 0.65): met
- **verdict: PILOT**

## Per-seed

| seed | rl_graded | rl_reinforce_fb | rl_flat | gradient | gap_closed_rl |
|---:|---:|---:|---:|---:|---:|
| 11400783397827083284 | 0.5000 | 1.0000 | 0.5000 | 0.7500 | 0.0000 |
| 4354473772296304683 | 0.5000 | 1.0000 | 0.5000 | 0.8333 | 0.0000 |
| 15755470070915167294 | 0.5000 | 0.5000 | 0.5000 | 1.0000 | 0.0000 |
| 8709160428200325205 | 1.0000 | 0.5000 | 0.5000 | 1.0000 | 1.0000 |
| 1663412927989378152 | 0.5000 | 0.5000 | 0.5000 | 1.0000 | 0.0000 |

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
