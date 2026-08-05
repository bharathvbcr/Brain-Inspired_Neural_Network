# BINN-Hybrid soft-to-hard winner-temperature ladder

- protocol: `binn-hybrid-winner-temp-v1-fa7710de68ad7bfe`
- schedule: **FULL DEVELOPMENT LADDER**
- seeds: 20
- depths: 1 through 8
- budgets: [480, 1920, 7680]
- learning rates: [0.015, 0.035, 0.07]
- temperatures: soft, 2.0000, 1.0000, 0.5000, 0.2500, 0.1000, hard
- frozen test examples per cell: 1000
- all test weights unchanged: **true**
- scientific gate effect: **none**
- transfer collapse temperature: **2.0000**

> Separately preregistered successor diagnostic. Canonical H0 remains `HYBRID_NO_GO`; held-out seeds remain unused; H1-H3 remain stopped. This ladder localizes where soft residual terminal gradients stop transferring across winner discretization. It is not post-hoc tuning of frozen H0 or the production diagnostic.

## Mechanism contract

Training uses only the matched soft residual terminal teacher: linear composition of shared transition weights, terminal softmax cross-entropy, and exact edge gradients. Direct-terminal updates are therefore temperature-independent. Evaluation applies a winner operator to the same residual scores: `soft` keeps the linear state, finite `T` uses `softmax(scores / T)`, and `hard` uses one-hot argmax. Privileged intermediate targets remain an inadmissible ceiling; shuffled labels remain a leakage control.

## Best observed development D* by temperature

| temperature | arm | D* at lower-95 accuracy ≥ 0.65 |
|---|---|---:|
| soft | direct-terminal | 5 |
| soft | privileged-intermediate-target | 7 |
| soft | shuffled-label | none |
| 2.0000 | direct-terminal | 2 |
| 2.0000 | privileged-intermediate-target | 8 |
| 2.0000 | shuffled-label | none |
| 1.0000 | direct-terminal | 2 |
| 1.0000 | privileged-intermediate-target | 8 |
| 1.0000 | shuffled-label | none |
| 0.5000 | direct-terminal | 2 |
| 0.5000 | privileged-intermediate-target | 8 |
| 0.5000 | shuffled-label | none |
| 0.2500 | direct-terminal | 2 |
| 0.2500 | privileged-intermediate-target | 8 |
| 0.2500 | shuffled-label | none |
| 0.1000 | direct-terminal | 2 |
| 0.1000 | privileged-intermediate-target | 8 |
| 0.1000 | shuffled-label | none |
| hard | direct-terminal | 1 |
| hard | privileged-intermediate-target | 8 |
| hard | shuffled-label | none |

## Direct-terminal transfer curve

| temperature | D* | best depth mean | best depth lower 95% | budget | lr |
|---|---:|---:|---:|---:|---:|
| soft | 5 | 0.7124 | 0.6956 | 7680 | 0.0150 |
| 2.0000 | 2 | 1.0000 | 1.0000 | 7680 | 0.0700 |
| 1.0000 | 2 | 1.0000 | 1.0000 | 7680 | 0.0700 |
| 0.5000 | 2 | 1.0000 | 1.0000 | 7680 | 0.0700 |
| 0.2500 | 2 | 1.0000 | 1.0000 | 7680 | 0.0700 |
| 0.1000 | 2 | 0.9567 | 0.9280 | 7680 | 0.0700 |
| hard | 1 | 1.0000 | 1.0000 | 7680 | 0.0700 |

## Collapse rule

Transfer collapse temperature is the softest ladder point at which direct-terminal D* falls strictly below the soft-endpoint D*. If every tempered/hard point preserves the soft D*, collapse is `none` under this development grid.

## Mechanisms by temperature

| temperature | depth | grad norm | soft direct drop | soft rotated drop | tempered direct drop | tempered rotated drop |
|---|---:|---:|---:|---:|---:|---:|
| soft | 1 | 0.9839 | 0.000968 | -0.000282 | 0.000968 | -0.000282 |
| soft | 2 | 1.6168 | 0.002767 | -0.000551 | 0.002767 | -0.000551 |
| soft | 3 | 2.0577 | 0.004628 | -0.001265 | 0.004628 | -0.001265 |
| soft | 4 | 2.7270 | 0.007623 | -0.001972 | 0.007623 | -0.001972 |
| soft | 5 | 3.6104 | 0.013525 | -0.003292 | 0.013525 | -0.003292 |
| soft | 6 | 4.0499 | 0.017241 | -0.004356 | 0.017241 | -0.004356 |
| soft | 7 | 4.7046 | 0.023097 | -0.006089 | 0.023097 | -0.006089 |
| soft | 8 | 5.5297 | 0.031736 | -0.008226 | 0.031736 | -0.008226 |
| 2.0000 | 1 | 0.9839 | 0.000968 | -0.000282 | 0.000099 | -0.000026 |
| 2.0000 | 2 | 1.6168 | 0.002767 | -0.000551 | 0.000071 | -0.000019 |
| 2.0000 | 3 | 2.0577 | 0.004628 | -0.001265 | 0.000058 | -0.000020 |
| 2.0000 | 4 | 2.7270 | 0.007623 | -0.001972 | 0.000066 | -0.000019 |
| 2.0000 | 5 | 3.6104 | 0.013525 | -0.003292 | 0.000084 | -0.000027 |
| 2.0000 | 6 | 4.0499 | 0.017241 | -0.004356 | 0.000093 | -0.000032 |
| 2.0000 | 7 | 4.7046 | 0.023097 | -0.006089 | 0.000107 | -0.000035 |
| 2.0000 | 8 | 5.5297 | 0.031736 | -0.008226 | 0.000123 | -0.000041 |
| 1.0000 | 1 | 0.9839 | 0.000968 | -0.000282 | 0.000182 | -0.000059 |
| 1.0000 | 2 | 1.6168 | 0.002767 | -0.000551 | 0.000206 | -0.000048 |
| 1.0000 | 3 | 2.0577 | 0.004628 | -0.001265 | 0.000178 | -0.000057 |
| 1.0000 | 4 | 2.7270 | 0.007623 | -0.001972 | 0.000176 | -0.000050 |
| 1.0000 | 5 | 3.6104 | 0.013525 | -0.003292 | 0.000205 | -0.000066 |
| 1.0000 | 6 | 4.0499 | 0.017241 | -0.004356 | 0.000221 | -0.000075 |
| 1.0000 | 7 | 4.7046 | 0.023097 | -0.006089 | 0.000250 | -0.000082 |
| 1.0000 | 8 | 5.5297 | 0.031736 | -0.008226 | 0.000289 | -0.000095 |
| 0.5000 | 1 | 0.9839 | 0.000968 | -0.000282 | 0.000261 | -0.000112 |
| 0.5000 | 2 | 1.6168 | 0.002767 | -0.000551 | 0.000547 | -0.000084 |
| 0.5000 | 3 | 2.0577 | 0.004628 | -0.001265 | 0.000787 | -0.000203 |
| 0.5000 | 4 | 2.7270 | 0.007623 | -0.001972 | 0.000922 | -0.000271 |
| 0.5000 | 5 | 3.6104 | 0.013525 | -0.003292 | 0.000937 | -0.000265 |
| 0.5000 | 6 | 4.0499 | 0.017241 | -0.004356 | 0.000952 | -0.000283 |
| 0.5000 | 7 | 4.7046 | 0.023097 | -0.006089 | 0.000963 | -0.000298 |
| 0.5000 | 8 | 5.5297 | 0.031736 | -0.008226 | 0.001012 | -0.000321 |
| 0.2500 | 1 | 0.9839 | 0.000968 | -0.000282 | 0.000124 | -0.000063 |
| 0.2500 | 2 | 1.6168 | 0.002767 | -0.000551 | 0.000268 | -0.000023 |
| 0.2500 | 3 | 2.0577 | 0.004628 | -0.001265 | 0.000399 | -0.000085 |
| 0.2500 | 4 | 2.7270 | 0.007623 | -0.001972 | 0.000538 | -0.000186 |
| 0.2500 | 5 | 3.6104 | 0.013525 | -0.003292 | 0.000671 | -0.000159 |
| 0.2500 | 6 | 4.0499 | 0.017241 | -0.004356 | 0.000794 | -0.000170 |
| 0.2500 | 7 | 4.7046 | 0.023097 | -0.006089 | 0.000912 | -0.000259 |
| 0.2500 | 8 | 5.5297 | 0.031736 | -0.008226 | 0.001038 | -0.000277 |
| 0.1000 | 1 | 0.9839 | 0.000968 | -0.000282 | 0.000001 | -0.000000 |
| 0.1000 | 2 | 1.6168 | 0.002767 | -0.000551 | 0.000001 | -0.000000 |
| 0.1000 | 3 | 2.0577 | 0.004628 | -0.001265 | 0.000002 | -0.000000 |
| 0.1000 | 4 | 2.7270 | 0.007623 | -0.001972 | 0.000002 | -0.000001 |
| 0.1000 | 5 | 3.6104 | 0.013525 | -0.003292 | 0.000003 | -0.000001 |
| 0.1000 | 6 | 4.0499 | 0.017241 | -0.004356 | 0.000003 | -0.000001 |
| 0.1000 | 7 | 4.7046 | 0.023097 | -0.006089 | 0.000003 | -0.000001 |
| 0.1000 | 8 | 5.5297 | 0.031736 | -0.008226 | 0.000004 | -0.000001 |
| hard | 1 | 0.9839 | 0.000968 | -0.000282 | 0.000000 | 0.000000 |
| hard | 2 | 1.6168 | 0.002767 | -0.000551 | 0.000000 | 0.000000 |
| hard | 3 | 2.0577 | 0.004628 | -0.001265 | 0.000000 | 0.000000 |
| hard | 4 | 2.7270 | 0.007623 | -0.001972 | 0.000000 | 0.000000 |
| hard | 5 | 3.6104 | 0.013525 | -0.003292 | 0.000000 | 0.000000 |
| hard | 6 | 4.0499 | 0.017241 | -0.004356 | 0.000000 | 0.000000 |
| hard | 7 | 4.7046 | 0.023097 | -0.006089 | 0.000000 | 0.000000 |
| hard | 8 | 5.5297 | 0.031736 | -0.008226 | 0.000000 | 0.000000 |

## Limits

- Development seeds and hyperparameter selection only.
- Soft teacher remains a disclosed differentiable residual relaxation; hard winners have no ordinary derivative.
- Finite temperatures use softmax winners, which are not identical to the soft linear endpoint; `soft` is an explicit anchor.
- Privileged ceiling is not budget-matched.
- Cannot reopen H0 or authorize H1-H3.
