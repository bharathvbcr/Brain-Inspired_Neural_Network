# BINN matched-architecture in-family RL recipe (C1-RL)

**Does not reopen protocol-v2:** hash `c1-118207fbc3eaba53` and every G2 threshold remain unchanged. This is matched-arch protocol **v12** with a fresh `c1-rl-*` hash. Mechanism: **REINFORCE × fixed-random feedback** (`rl_reinforce_fb`) as the **primary** gated arm on the dense-LIF matched forward (feed-forward `wrec=0`). Graded / flat remain contrasts. Does **not** retune failed v11 `rl_graded` (`c1-rl-ef504db58916720d`), mutate `c1-dfa-*`, or remassage `c1x-dfa-spike-*`.

- schedule: **PILOT (development only — not a scientific verdict)**
- config hash: `c1-rl-9a41bc8f876617a5`
- protocol version: 12
- primary arm: `rl_reinforce_fb`
- seeds: 5
- train/test: 48/24
- hidden / epochs / β: 128 / 60 / 5.0
- gradient lr / local η / λ: 0.0500 / 0.0500 / 0.0000
- chance baseline: 0.5

## Results

| arm | mean accuracy | variance |
|---|---:|---:|
| `MATCHED_ARCH_RL_REINFORCE_FB` (REINFORCE × DFA fb) **primary** | 0.8000 | 0.075000 |
| `MATCHED_ARCH_RL_GRADED` (graded reward, broadcast) contrast | 0.5000 | 0.000000 |
| `MATCHED_ARCH_RL_FLAT` (±1 reward, broadcast) contrast | 0.5000 | 0.000000 |
| `MATCHED_ARCH_GRADIENT_CEILING` (SuperSpike BPTT ceiling) | 0.9167 | 0.006944 |

- `gap_closed_rl` mean: **0.6000**  (var 0.300000)
- lower 95% CB (z=1.96): **0.1199**  (needs > 0.50)
- accuracy floor (matched-rl-reinforce-fb ≥ 0.65): met
- harness validity (matched-gradient ≥ 0.65): met
- **verdict: PILOT**

## Per-seed

| seed | rl_reinforce_fb | rl_graded | rl_flat | gradient | gap_closed_rl |
|---:|---:|---:|---:|---:|---:|
| 11400783397827083271 | 1.0000 | 0.5000 | 0.5000 | 0.8333 | 1.0000 |
| 4354473772296304696 | 1.0000 | 0.5000 | 0.5000 | 1.0000 | 1.0000 |
| 15755470070915167277 | 0.5000 | 0.5000 | 0.5000 | 0.8333 | 0.0000 |
| 8709160428200325190 | 1.0000 | 0.5000 | 0.5000 | 1.0000 | 1.0000 |
| 1663412927989378171 | 0.5000 | 0.5000 | 0.5000 | 0.9167 | 0.0000 |

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
