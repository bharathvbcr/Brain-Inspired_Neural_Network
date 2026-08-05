# BINN matched-architecture control (C1-MATCH)

**Does not reopen protocol-v2:** hash `c1-118207fbc3eaba53` and every G2 threshold remain unchanged. This is protocol **v4** with a fresh `c1-match-*` hash. Mechanism label: **broadcast three-factor on dense-LIF** (not a BINN substrate rescue).

- schedule: **PILOT (development only — not a scientific verdict)**
- config hash: `c1-match-85e9548f0615b85a`
- protocol version: 4
- seeds: 5
- train/test: 24/16
- hidden / epochs / β: 64 / 20 / 5.0
- gradient lr / local η / λ: 0.0500 / 0.3500 / 0.0020
- chance baseline: 0.5

## Results

| arm | mean accuracy | variance |
|---|---:|---:|
| `MATCHED_ARCH_LOCAL_THREE_FACTOR` (broadcast three-factor) | 0.5000 | 0.000000 |
| `MATCHED_ARCH_GRADIENT_CEILING` (SuperSpike BPTT ceiling) | 0.7500 | 0.062500 |

- `gap_closed_matched` mean: **0.0000**  (var 0.000000)
- lower 95% CB (z=1.96): **0.0000**  (needs > 0.50)
- accuracy floor (matched-local ≥ 0.65): not met
- harness validity (matched-gradient ≥ 0.65): met
- **verdict: PILOT**

## Per-seed

| seed | matched-local | matched-gradient | gap_closed_matched |
|---:|---:|---:|---:|
| 11400783419301919764 | 0.5000 | 1.0000 | 0.0000 |
| 4354473785181206571 | 0.5000 | 0.5000 | 0.0000 |
| 15755470049440330814 | 0.5000 | 0.5000 | 0.0000 |
| 8709160415315423317 | 0.5000 | 0.7500 | 0.0000 |
| 1663412915104476264 | 0.5000 | 1.0000 | 0.0000 |

## Gate (unchanged thresholds)

`gap_closed_matched = (matched_local − 0.5) / (matched_gradient − 0.5)`, clamped to [0,1]; seeds with `(matched_gradient − 0.5) < g2_min_reference_gap` contribute `closed = 0`. PASS requires gap LCB > 0.5 and mean matched-local ≥ 0.65; mean matched-gradient < 0.65 ⇒ INVALID_HARNESS.

## Reproduce

```bash
cargo run --locked --release -p binn-lab --bin c1 -- --matched-arch --quick
cargo run --locked --release -p binn-lab --bin c1 -- --matched-arch --out results/c1_match.md
```
