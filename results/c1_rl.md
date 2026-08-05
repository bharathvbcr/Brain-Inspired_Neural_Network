# BINN matched-architecture in-family RL recipe (C1-RL)

**Does not reopen protocol-v2:** hash `c1-118207fbc3eaba53` and every G2 threshold remain unchanged. This is matched-arch protocol **v12** with a fresh `c1-rl-*` hash. Mechanism: **REINFORCE × fixed-random feedback** (`rl_reinforce_fb`) as the **primary** gated arm on the dense-LIF matched forward (feed-forward `wrec=0`). Graded / flat remain contrasts. Does **not** retune failed v11 `rl_graded` (`c1-rl-ef504db58916720d`), mutate `c1-dfa-*`, or remassage `c1x-dfa-spike-*`.

- schedule: **SCIENTIFIC**
- config hash: `c1-rl-42eddc9c801308e9`
- protocol version: 12
- primary arm: `rl_reinforce_fb`
- seeds: 20
- train/test: 80/40
- hidden / epochs / β: 128 / 80 / 5.0
- gradient lr / local η / λ: 0.0500 / 0.0500 / 0.0000
- chance baseline: 0.5

## Results

| arm | mean accuracy | variance |
|---|---:|---:|
| `MATCHED_ARCH_RL_REINFORCE_FB` (REINFORCE × DFA fb) **primary** | 0.9200 | 0.033263 |
| `MATCHED_ARCH_RL_GRADED` (graded reward, broadcast) contrast | 0.5250 | 0.012500 |
| `MATCHED_ARCH_RL_FLAT` (±1 reward, broadcast) contrast | 0.5113 | 0.001544 |
| `MATCHED_ARCH_GRADIENT_CEILING` (SuperSpike BPTT ceiling) | 0.8887 | 0.021215 |

- `gap_closed_rl` mean: **0.8444**  (var 0.133073)
- lower 95% CB (z=1.96): **0.6846**  (needs > 0.50)
- accuracy floor (matched-rl-reinforce-fb ≥ 0.65): met
- harness validity (matched-gradient ≥ 0.65): met
- **verdict: PASS**

## Per-seed

| seed | rl_reinforce_fb | rl_graded | rl_flat | gradient | gap_closed_rl |
|---:|---:|---:|---:|---:|---:|
| 11400783395455400967 | 1.0000 | 0.5000 | 0.5250 | 0.7250 | 1.0000 |
| 4354473774193899576 | 1.0000 | 0.5000 | 0.5000 | 0.7250 | 1.0000 |
| 15755470068493808685 | 0.5000 | 0.5000 | 0.6750 | 1.0000 | 0.0000 |
| 8709160430052438086 | 1.0000 | 0.5000 | 0.5000 | 0.9500 | 1.0000 |
| 1663412925520702587 | 1.0000 | 0.5000 | 0.5000 | 0.9500 | 1.0000 |
| 13063847377968752748 | 0.9000 | 0.5000 | 0.5000 | 0.9500 | 0.8889 |
| 6018099598559110273 | 1.0000 | 0.5000 | 0.5000 | 1.0000 | 1.0000 |
| 17418529635780780218 | 1.0000 | 0.5000 | 0.5000 | 0.9000 | 1.0000 |
| 10372781856371137711 | 1.0000 | 0.5000 | 0.5000 | 0.7250 | 1.0000 |
| 3326471960231729344 | 1.0000 | 0.5000 | 0.5000 | 1.0000 | 1.0000 |
| 14727610091531621621 | 1.0000 | 1.0000 | 0.5000 | 1.0000 | 1.0000 |
| 7681299353578623214 | 1.0000 | 0.5000 | 0.5250 | 0.9500 | 1.0000 |
| 635552673680608515 | 1.0000 | 0.5000 | 0.5000 | 1.0000 | 1.0000 |
| 12035986026617030964 | 1.0000 | 0.5000 | 0.5000 | 1.0000 | 1.0000 |
| 4990235223550412073 | 0.5000 | 0.5000 | 0.5000 | 0.7250 | 0.0000 |
| 16390668559306965314 | 1.0000 | 0.5000 | 0.5000 | 0.7250 | 1.0000 |
| 9344921879408950647 | 1.0000 | 0.5000 | 0.5000 | 0.9500 | 1.0000 |
| 2298611158635821416 | 0.5000 | 0.5000 | 0.5000 | 0.5000 | 0.0000 |
| 13699608552447358365 | 1.0000 | 0.5000 | 0.5000 | 1.0000 | 1.0000 |
| 6653298639128080822 | 1.0000 | 0.5000 | 0.5000 | 1.0000 | 1.0000 |

## Gate (unchanged thresholds)

Primary arm = `rl_reinforce_fb`. `gap_closed_rl = (matched_rl_reinforce_fb − 0.5) / (matched_gradient − 0.5)`, clamped to [0,1]; seeds with `(matched_gradient − 0.5) < g2_min_reference_gap` contribute `closed = 0`. PASS requires gap LCB > 0.5 and mean matched-rl-reinforce-fb ≥ 0.65; mean matched-gradient < 0.65 ⇒ INVALID_HARNESS.

## Recipe notes

- Readout always uses REINFORCE `r·(a−p)` (Bernoulli policy).
- Hidden `rl_reinforce_fb` (**primary**): frozen `B_i ∈ [-1,1]` × `r·(a−p)`.
- Hidden `rl_graded` (contrast; v11 primary): broadcast `(p_correct − baseline)` with EMA baseline.
- Hidden `rl_flat` (contrast): broadcast ±1 reward (production impoverishment).
- Supervised DFA (`c1-dfa-*`) remains a separate protocol; this suite asks whether an **RL** modulator can close the gap in-family.
- v11 graded-primary FAIL is archived at `c1-rl-ef504db58916720d` — not retuned here.

## Reproduce

```bash
cargo run --locked --release -p binn-lab --bin c1 -- --matched-rl --quick
cargo run --locked --release -p binn-lab --bin c1 -- --matched-rl --out results/c1_rl.md
```
