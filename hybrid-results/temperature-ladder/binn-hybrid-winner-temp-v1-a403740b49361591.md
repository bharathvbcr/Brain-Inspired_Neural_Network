# BINN-Hybrid soft-to-hard winner-temperature ladder

- protocol: `binn-hybrid-winner-temp-v1-a403740b49361591`
- schedule: **PILOT**
- seeds: 3
- depths: 1 through 8
- budgets: [60, 240]
- learning rates: [0.035]
- temperatures: soft, 1.0000, hard
- frozen test examples per cell: 160
- all test weights unchanged: **true**
- scientific gate effect: **none**
- transfer collapse temperature: **hard**

> Separately preregistered successor diagnostic. Canonical H0 remains `HYBRID_NO_GO`; held-out seeds remain unused; H1-H3 remain stopped. This ladder localizes where soft residual terminal gradients stop transferring across winner discretization. It is not post-hoc tuning of frozen H0 or the production diagnostic.

## Mechanism contract

Training uses only the matched soft residual terminal teacher: linear composition of shared transition weights, terminal softmax cross-entropy, and exact edge gradients. Direct-terminal updates are therefore temperature-independent. Evaluation applies a winner operator to the same residual scores: `soft` keeps the linear state, finite `T` uses `softmax(scores / T)`, and `hard` uses one-hot argmax. Privileged intermediate targets remain an inadmissible ceiling; shuffled labels remain a leakage control.

## Best observed development D* by temperature

| temperature | arm | D* at lower-95 accuracy ≥ 0.65 |
|---|---|---:|
| soft | direct-terminal | 2 |
| soft | privileged-intermediate-target | 8 |
| soft | shuffled-label | none |
| 1.0000 | direct-terminal | 2 |
| 1.0000 | privileged-intermediate-target | 8 |
| 1.0000 | shuffled-label | none |
| hard | direct-terminal | 1 |
| hard | privileged-intermediate-target | 8 |
| hard | shuffled-label | none |

## Direct-terminal transfer curve

| temperature | D* | best depth mean | best depth lower 95% | budget | lr |
|---|---:|---:|---:|---:|---:|
| soft | 2 | 1.0000 | 1.0000 | 240 | 0.0350 |
| 1.0000 | 2 | 0.9771 | 0.9322 | 240 | 0.0350 |
| hard | 1 | 0.8708 | 0.7144 | 240 | 0.0350 |

## Collapse rule

Transfer collapse temperature is the softest ladder point at which direct-terminal D* falls strictly below the soft-endpoint D*. If every tempered/hard point preserves the soft D*, collapse is `none` under this development grid.

## Mechanisms by temperature

| temperature | depth | grad norm | soft direct drop | soft rotated drop | tempered direct drop | tempered rotated drop |
|---|---:|---:|---:|---:|---:|---:|
| soft | 1 | 0.9855 | 0.000971 | -0.000320 | 0.000971 | -0.000320 |
| soft | 2 | 1.7108 | 0.003058 | -0.000600 | 0.003058 | -0.000600 |
| soft | 3 | 1.9468 | 0.004252 | -0.000959 | 0.004252 | -0.000959 |
| soft | 4 | 2.7090 | 0.007550 | -0.001733 | 0.007550 | -0.001733 |
| soft | 5 | 3.7606 | 0.014637 | -0.003109 | 0.014637 | -0.003109 |
| soft | 6 | 3.7637 | 0.014923 | -0.004312 | 0.014923 | -0.004312 |
| soft | 7 | 4.6233 | 0.022448 | -0.005539 | 0.022448 | -0.005539 |
| soft | 8 | 5.6121 | 0.032630 | -0.009130 | 0.032630 | -0.009130 |
| 1.0000 | 1 | 0.9855 | 0.000971 | -0.000320 | 0.000182 | -0.000067 |
| 1.0000 | 2 | 1.7108 | 0.003058 | -0.000600 | 0.000224 | -0.000050 |
| 1.0000 | 3 | 1.9468 | 0.004252 | -0.000959 | 0.000173 | -0.000049 |
| 1.0000 | 4 | 2.7090 | 0.007550 | -0.001733 | 0.000180 | -0.000049 |
| 1.0000 | 5 | 3.7606 | 0.014637 | -0.003109 | 0.000213 | -0.000060 |
| 1.0000 | 6 | 3.7637 | 0.014923 | -0.004312 | 0.000204 | -0.000067 |
| 1.0000 | 7 | 4.6233 | 0.022448 | -0.005539 | 0.000247 | -0.000083 |
| 1.0000 | 8 | 5.6121 | 0.032630 | -0.009130 | 0.000290 | -0.000097 |
| hard | 1 | 0.9855 | 0.000971 | -0.000320 | 0.000000 | 0.000000 |
| hard | 2 | 1.7108 | 0.003058 | -0.000600 | 0.000000 | 0.000000 |
| hard | 3 | 1.9468 | 0.004252 | -0.000959 | 0.000000 | 0.000000 |
| hard | 4 | 2.7090 | 0.007550 | -0.001733 | 0.000000 | 0.000000 |
| hard | 5 | 3.7606 | 0.014637 | -0.003109 | 0.000000 | 0.000000 |
| hard | 6 | 3.7637 | 0.014923 | -0.004312 | 0.000000 | 0.000000 |
| hard | 7 | 4.6233 | 0.022448 | -0.005539 | 0.000000 | 0.000000 |
| hard | 8 | 5.6121 | 0.032630 | -0.009130 | 0.000000 | 0.000000 |

## Limits

- Development seeds and hyperparameter selection only.
- Soft teacher remains a disclosed differentiable residual relaxation; hard winners have no ordinary derivative.
- Finite temperatures use softmax winners, which are not identical to the soft linear endpoint; `soft` is an explicit anchor.
- Privileged ceiling is not budget-matched.
- Cannot reopen H0 or authorize H1-H3.
