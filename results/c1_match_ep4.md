# BINN matched-architecture control (C1-MATCH)

**claim_axis:** Integrity
**object_under_test:** Matched dense-LIF broadcast three-factor under 4× epochs
**may_claim:** Under protocol 22, whether the v4 FAIL survives 4× training exposure
**must_not_claim:** Remassage of `c1-match-5dc6822e71229e9e`; impossibility; biology

**Matched undertrain protocol:** `22` — same matched dense-LIF + broadcast three-factor as v4, but local/gradient arms train for **4×** epochs (`bptt_epochs=320`); does **not** remassage `c1-match-5dc6822e71229e9e` or reopen protocol-v2 `c1-118207fbc3eaba53`.

- schedule: **SCIENTIFIC**
- config hash: `c1-match-b46b23549b37d90a`
- protocol version: 22
- seeds: 20
- train/test: 80/40
- hidden / epochs / β: 128 / 320 / 5.0
- gradient lr / local η / λ: 0.0500 / 0.3500 / 0.0020
- chance baseline: 0.5

## Results

| arm | mean accuracy | variance |
|---|---:|---:|
| `MATCHED_ARCH_LOCAL_THREE_FACTOR` (broadcast three-factor) | 0.5000 | 0.000000 |
| `MATCHED_ARCH_GRADIENT_CEILING` (SuperSpike BPTT ceiling) | 0.9788 | 0.003900 |

- `gap_closed_matched` mean: **0.0000**  (var 0.000000)
- lower 95% CB (z=1.96): **0.0000**  (needs > 0.50)
- accuracy floor (matched-local ≥ 0.65): not met
- harness validity (matched-gradient ≥ 0.65): met
- **verdict: FAIL**

## Per-seed

| seed | matched-local | matched-gradient | gap_closed_matched |
|---:|---:|---:|---:|
| 11400783419547483156 | 0.5000 | 1.0000 | 0.0000 |
| 4354473785401079851 | 0.5000 | 0.9500 | 0.0000 |
| 15755470049636217918 | 0.5000 | 1.0000 | 0.0000 |
| 8709160415489814613 | 0.5000 | 1.0000 | 0.0000 |
| 1663412915253046376 | 0.5000 | 1.0000 | 0.0000 |
| 13063847389310150783 | 0.5000 | 0.7250 | 0.0000 |
| 6018099614195475602 | 0.5000 | 1.0000 | 0.0000 |
| 17418529655712112809 | 0.5000 | 1.0000 | 0.0000 |
| 10372781846237699260 | 0.5000 | 1.0000 | 0.0000 |
| 3326471937213389011 | 0.5000 | 1.0000 | 0.0000 |
| 14727610107167986918 | 0.5000 | 1.0000 | 0.0000 |
| 7681299373375738109 | 0.5000 | 0.9500 | 0.0000 |
| 635552697772690704 | 0.5000 | 1.0000 | 0.0000 |
| 12035986003464472871 | 0.5000 | 1.0000 | 0.0000 |
| 4990235204692821306 | 0.5000 | 1.0000 | 0.0000 |
| 16390668544744341841 | 0.5000 | 1.0000 | 0.0000 |
| 9344921903635250532 | 0.5000 | 1.0000 | 0.0000 |
| 2298611169977219451 | 0.5000 | 1.0000 | 0.0000 |
| 13699608533723985294 | 0.5000 | 0.9500 | 0.0000 |
| 6653298624699675045 | 0.5000 | 1.0000 | 0.0000 |

## Gate (unchanged thresholds)

`gap_closed_matched = (matched_local − 0.5) / (matched_gradient − 0.5)`, clamped to [0,1]; seeds with `(matched_gradient − 0.5) < g2_min_reference_gap` contribute `closed = 0`. PASS requires gap LCB > 0.5 and mean matched-local ≥ 0.65; mean matched-gradient < 0.65 ⇒ INVALID_HARNESS.

## Reproduce

```bash
cargo run --locked --release -p binn-lab --bin c1 -- --matched-arch --quick
cargo run --locked --release -p binn-lab --bin c1 -- --matched-arch --out results/c1_match.md
```
