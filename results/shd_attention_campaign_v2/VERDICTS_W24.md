# Wave 24 — order, synchrony, and the second budget

Registered: `PREREG_2026-09-01_ORDER_SYNCHRONY_AND_BUDGET.md`.
This analyser is the authority on every verdict below.

Pinned binary this contrast requires: `3afd4434431a75a26cc9d5fa46831341fc2f1dd0ef08dc308e18ca139b576364`.
`e400` intact and bin-shuffled arms are reused from `w22cov`; every other arm is `w24ord`.

`w24ord` cells indexed: **336** of 336 planned.

## H24-1 — is the cost generic perturbation brittleness?

`reversed` is displacement-matched and information-preserving. The bar is the wave-21/22 bar used from the other side: a null must **fail** to clear +0.03.

| point | DiD(bin) | DiD(reversed) | ratio | positive | pairs | verdict |
|---|---:|---:|---:|---:|---:|---|
| h128 / published-2ms / adjacent-sum-5 / d32l2 | +0.1145 | -0.0222 | -0.19 | 2/12 | 12 | **MET** |
| h1024 / published-2ms / adjacent-sum-5 / d32l1 | +0.0675 | -0.0040 | -0.06 | 4/12 | 12 | **MET** |
| h128 / fixed-t250 / adjacent-sum-5 / d32l4 | +0.1119 | -0.0099 | -0.09 | 3/12 | 12 | **MET** |

**H24-1: MET** — the cost is not generic perturbation brittleness at the points measured.

## H24-2 — does cross-channel synchrony contribute?

Registered as a QUESTION, not a prediction. `channel-shuffled` destroys order **and** synchrony; the band is ±0.03.

| point | DiD(bin) | DiD(channel) | difference | pairs | answer |
|---|---:|---:|---:|---:|---|
| h128 / published-2ms / adjacent-sum-5 / d32l2 | +0.1145 | +0.1028 | -0.0117 | 12 | order alone |
| h1024 / published-2ms / adjacent-sum-5 / d32l1 | +0.0675 | +0.0495 | -0.0181 | 12 | order alone |
| h128 / fixed-t250 / adjacent-sum-5 / d32l4 | +0.1119 | +0.2157 | +0.1038 | 12 | **synchrony also read** |

## H24-3 — does the contrast survive a second budget?

The H22-1 bar unchanged: DiD > +0.03, positive in ≥ 9 of 12.

| point | budget | DiD(bin) | positive | pairs | verdict |
|---|---|---:|---:|---:|---|
| h128 / published-2ms / adjacent-sum-5 / d32l2 | e100 | +0.1183 | 12/12 | 12 | **MET** |
| h1024 / published-2ms / adjacent-sum-5 / d32l1 | e100 | +0.0633 | 12/12 | 12 | **MET** |

**H24-3: MET** — the mechanism is not a property of the e400 budget alone.

## H24-4 — does the order control hold at the second budget?

| point | budget | DiD(reversed) | pairs | verdict |
|---|---|---:|---:|---|
| h128 / published-2ms / adjacent-sum-5 / d32l2 | e100 | -0.0068 | 12 | **MET** |
| h1024 / published-2ms / adjacent-sum-5 / d32l1 | e100 | -0.0307 | 12 | **MET** |

**H24-4: MET** — the e400 null is not a convergence artefact.

## Provenance — which wave supplied each arm

| budget | condition | source wave | attention arms found | rate arms found |
|---|---|---|---:|---:|
| e100 | bin-shuffled | `w24ord` | 2/2 | 2/2 |
| e100 | channel-shuffled | `w24ord` | 2/2 | 2/2 |
| e100 | intact | `w24ord` | 2/2 | 2/2 |
| e100 | reversed | `w24ord` | 2/2 | 2/2 |
| e400 | bin-shuffled | `w22cov` | 3/3 | 3/3 |
| e400 | channel-shuffled | `w24ord` | 3/3 | 3/3 |
| e400 | intact | `w22cov` | 3/3 | 3/3 |
| e400 | reversed | `w24ord` | 3/3 | 3/3 |

## H24-5 — cells

`w24ord` cells that passed validity: **336** of 336.
Voided by `cell_validity`: **0**.
**H24-5: MET** — mechanical; it can fail only by cells failing to run.
