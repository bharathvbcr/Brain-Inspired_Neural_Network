# BINN matched-architecture DFA recipe (C1-DFA)

**Does not reopen protocol-v2:** hash `c1-118207fbc3eaba53` and every G2 threshold remain unchanged. This is matched-arch protocol **v5** with a fresh `c1-dfa-*` hash (distinct from trial-isolation `c1-iso*` which also uses integer 5). Mechanism: **directional graded error × fixed-random DFA feedback** on the dense-LIF matched forward (feed-forward `wrec=0`, matching the NumPy DFA preview).

- schedule: **PILOT (development only — not a scientific verdict)**
- config hash: `c1-dfa-c887a1117d28d518`
- protocol version: 5
- seeds: 5
- train/test: 48/24
- hidden / epochs / β: 128 / 60 / 5.0
- gradient lr / local η / λ: 0.0500 / 0.0500 / 0.0000
- chance baseline: 0.5

## Results

| arm | mean accuracy | variance |
|---|---:|---:|
| `MATCHED_ARCH_DFA_GRADED_ERROR` (graded error × DFA) | 0.7000 | 0.075000 |
| `MATCHED_ARCH_BROADCAST_GRADED_ERROR` (graded error, broadcast) | 1.0000 | 0.000000 |
| `MATCHED_ARCH_GRADIENT_CEILING` (SuperSpike BPTT ceiling) | 0.8833 | 0.012500 |

- `gap_closed_dfa` mean: **0.4000**  (var 0.300000)
- lower 95% CB (z=1.96): **-0.0801**  (needs > 0.50)
- accuracy floor (matched-dfa ≥ 0.65): met
- harness validity (matched-gradient ≥ 0.65): met
- **verdict: PILOT**

## Per-seed

| seed | dfa | broadcast-err | gradient | gap_closed_dfa |
|---:|---:|---:|---:|---:|
| 11400783758604336148 | 1.0000 | 1.0000 | 1.0000 | 1.0000 |
| 4354473583317743659 | 0.5000 | 1.0000 | 0.7500 | 0.0000 |
| 15755470534771635262 | 0.5000 | 1.0000 | 1.0000 | 0.0000 |
| 8709160342300979285 | 1.0000 | 1.0000 | 0.8333 | 1.0000 |
| 1663412979528985704 | 0.5000 | 1.0000 | 0.8333 | 0.0000 |

## Gate (unchanged thresholds)

Primary arm = DFA. `gap_closed_dfa = (matched_dfa − 0.5) / (matched_gradient − 0.5)`, clamped to [0,1]; seeds with `(matched_gradient − 0.5) < g2_min_reference_gap` contribute `closed = 0`. PASS requires gap LCB > 0.5 and mean matched-dfa ≥ 0.65; mean matched-gradient < 0.65 ⇒ INVALID_HARNESS.

## Spiking-substrate note

The dense-LIF result above isolates the **rule**. On the real BINN path (LatencyEncoder + k-WTA + single online pass), the exact-forward DFA arm is already preregistered under credit-assignment (`dfa-exact-forward`, hashes `c1x-dfa-exact-forward-*` / `c1x-iso-dfa-exact-forward-*`). Those runs do **not** clear G2 — the k-WTA / single-pass substrate re-introduces the handicap even when the learning signal is graded + directional. See `results/credit_assignment_iso_SUMMARY.md`.

## Reproduce

```bash
cargo run --locked --release -p binn-lab --bin c1 -- --matched-dfa --quick
cargo run --locked --release -p binn-lab --bin c1 -- --matched-dfa --out results/c1_dfa.md
```
