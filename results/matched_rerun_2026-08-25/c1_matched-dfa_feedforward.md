# BINN matched-architecture DFA recipe (C1-DFA)

**Does not reopen protocol-v2:** hash `c1-118207fbc3eaba53` and every G2 threshold remain unchanged. This is matched-arch protocol **v5** with a fresh `c1-dfa-*` hash (distinct from trial-isolation `c1-iso*` which also uses integer 5). Mechanism: **directional graded error × fixed-random DFA feedback** on the dense-LIF matched forward (feed-forward `wrec=0`, matching the NumPy DFA preview).

- schedule: **SCIENTIFIC**
- config hash: `c1-dfa-f79c01ea36fe27d7`
- protocol version: 5
- seeds: 20
- train/test: 80/40
- hidden / epochs / β: 128 / 80 / 5.0
- gradient lr / local η / λ: 0.0500 / 0.0500 / 0.0000
- chance baseline: 0.5

- forward graph: **feedforward**

## Results

| arm | mean accuracy | variance |
|---|---:|---:|
| `MATCHED_ARCH_DFA_GRADED_ERROR` (graded error × DFA) | 0.9925 | 0.000336 |
| `MATCHED_ARCH_BROADCAST_GRADED_ERROR` (graded error, broadcast) | 0.9975 | 0.000125 |
| `MATCHED_ARCH_GRADIENT_CEILING` (SuperSpike BPTT ceiling) | 1.0000 | 0.000000 |

- `gap_closed_dfa` mean: **0.9850**  (var 0.001342)
- lower 95% CB (z=1.96): **0.9689**  (needs > 0.50)
- accuracy floor (matched-dfa ≥ 0.65): met
- harness validity (matched-gradient ≥ 0.65): met
- **verdict: PASS**

## Per-seed

| seed | dfa | broadcast-err | gradient | gap_closed_dfa |
|---:|---:|---:|---:|---:|
| 11400783759386770452 | 1.0000 | 1.0000 | 1.0000 | 1.0000 |
| 4354473584074487851 | 1.0000 | 1.0000 | 1.0000 | 1.0000 |
| 15755470535504393278 | 1.0000 | 1.0000 | 1.0000 | 1.0000 |
| 8709160343012241493 | 1.0000 | 1.0000 | 1.0000 | 1.0000 |
| 1663412980214426728 | 0.9500 | 0.9500 | 1.0000 | 0.9000 |
| 13063846913105651839 | 0.9500 | 1.0000 | 1.0000 | 0.9000 |
| 6018099687746790546 | 1.0000 | 1.0000 | 1.0000 | 1.0000 |
| 17418529308356632745 | 1.0000 | 1.0000 | 1.0000 | 1.0000 |
| 10372782048638033084 | 1.0000 | 1.0000 | 1.0000 | 1.0000 |
| 3326472285642610899 | 1.0000 | 1.0000 | 1.0000 | 1.0000 |
| 14727609905841394918 | 1.0000 | 1.0000 | 1.0000 | 1.0000 |
| 7681299713215025405 | 1.0000 | 1.0000 | 1.0000 | 1.0000 |
| 635552625295117584 | 1.0000 | 1.0000 | 1.0000 | 1.0000 |
| 12035986489332648231 | 1.0000 | 1.0000 | 1.0000 | 1.0000 |
| 4990234728488322362 | 1.0000 | 1.0000 | 1.0000 | 1.0000 |
| 16390668609705722193 | 0.9500 | 1.0000 | 1.0000 | 0.9000 |
| 9344921418840816996 | 1.0000 | 1.0000 | 1.0000 | 1.0000 |
| 2298611380967487867 | 1.0000 | 1.0000 | 1.0000 | 1.0000 |
| 13699608194958439822 | 1.0000 | 1.0000 | 1.0000 | 1.0000 |
| 6653298414783148453 | 1.0000 | 1.0000 | 1.0000 | 1.0000 |

## Gate (unchanged thresholds)

Primary arm = DFA. `gap_closed_dfa = (matched_dfa − 0.5) / (matched_gradient − 0.5)`, clamped to [0,1]; seeds with `(matched_gradient − 0.5) < g2_min_reference_gap` contribute `closed = 0`. PASS requires gap LCB > 0.5 and mean matched-dfa ≥ 0.65; mean matched-gradient < 0.65 ⇒ INVALID_HARNESS.

## Spiking-substrate note

The dense-LIF result above isolates the **rule**. On the real BINN path (LatencyEncoder + k-WTA + single online pass), the exact-forward DFA arm is already preregistered under credit-assignment (`dfa-exact-forward`, hashes `c1x-dfa-exact-forward-*` / `c1x-iso-dfa-exact-forward-*`). Those runs do **not** clear G2 — the k-WTA / single-pass substrate re-introduces the handicap even when the learning signal is graded + directional. See `results/credit_assignment_iso_SUMMARY.md`.

## Reproduce

```bash
cargo run --locked --release -p binn-lab --bin c1 -- --matched-dfa --quick
cargo run --locked --release -p binn-lab --bin c1 -- --matched-dfa --out results/c1_dfa.md
```
