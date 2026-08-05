# v147 — temporal optimiser / budget control

**Schedule:** scientific · difficulty (0, 4) · seeds 3 · hidden 64 · n_test 200  
**Chance:** 0.2500 · **learns threshold:** chance + 0.10

## Why this run exists

v144 compared `train_bptt` (**Adam**) against `train_feedback` (**plain SGD, lr fixed at 0.005**) on 200 examples × 20 epochs, with no learning-rate sweep. That varies the credit pathway, the optimiser and the budget simultaneously. `bptt-sgd` below is the optimiser-matched ceiling — same update rule, same step size, only the gradients differ. It, not `bptt-adam`, is the comparison of record.

## Grid

| Accessibility | Arm | n_train | epochs | lr | Mean acc | SE | Learning curve | Step RMS | Modulator RMS |
|---|---|---:|---:|---:|---:|---:|---|---:|---:|
| accessible | feedback-sgd | 200 | 20 | 0.0010 | 0.2633 | 0.0159 | 0.107 → 0.177 → 0.195 → 0.227 → 0.263 | 2.751e-5 | 1.229e-2 |
| accessible | feedback-sgd | 200 | 20 | 0.0050 | 0.9983 | 0.0017 | 0.307 → 0.687 → 0.832 → 0.918 → 0.998 | 1.373e-4 | 1.215e-2 |
| accessible | feedback-sgd | 200 | 20 | 0.0200 | 1.0000 | 0.0000 | 0.908 → 1.000 → 1.000 → 1.000 → 1.000 | 5.270e-4 | 8.922e-3 |
| accessible | feedback-sgd | 200 | 20 | 0.0800 | 1.0000 | 0.0000 | 1.000 → 1.000 → 1.000 → 1.000 → 1.000 | 9.778e-4 | 7.227e-4 |
| accessible | feedback-sgd | 1000 | 20 | 0.0010 | 1.0000 | 0.0000 | 0.253 → 0.680 → 0.833 → 0.933 → 1.000 | 2.745e-5 | 1.215e-2 |
| accessible | feedback-sgd | 1000 | 20 | 0.0050 | 1.0000 | 0.0000 | 0.998 → 1.000 → 1.000 → 1.000 → 1.000 | 1.254e-4 | 6.956e-3 |
| accessible | feedback-sgd | 1000 | 20 | 0.0200 | 1.0000 | 0.0000 | 1.000 → 1.000 → 1.000 → 1.000 → 1.000 | 2.020e-4 | 5.023e-4 |
| accessible | feedback-sgd | 1000 | 20 | 0.0800 | 1.0000 | 0.0000 | 1.000 → 1.000 → 1.000 → 1.000 → 1.000 | 2.341e-4 | 7.235e-5 |
| accessible | feedback-sgd | 1000 | 100 | 0.0010 | 1.0000 | 0.0000 | 1.000 → 1.000 → 1.000 → 1.000 → 1.000 | 2.506e-5 | 6.961e-3 |
| accessible | feedback-sgd | 1000 | 100 | 0.0050 | 1.0000 | 0.0000 | 1.000 → 1.000 → 1.000 → 1.000 → 1.000 | 4.163e-5 | 3.543e-4 |
| accessible | feedback-sgd | 1000 | 100 | 0.0200 | 1.0000 | 0.0000 | 1.000 → 1.000 → 1.000 → 1.000 → 1.000 | 4.751e-5 | 5.514e-5 |
| accessible | feedback-sgd | 1000 | 100 | 0.0800 | 1.0000 | 0.0000 | 1.000 → 1.000 → 1.000 → 1.000 → 1.000 | 5.200e-5 | 1.093e-5 |
| accessible | bptt-sgd (matched) | 200 | 20 | 0.0010 | 0.9167 | 0.0833 | 0.398 → 0.597 → 0.887 → 0.917 → 0.917 | 3.016e-5 | 1.213e-2 |
| accessible | bptt-sgd (matched) | 200 | 20 | 0.0050 | 1.0000 | 0.0000 | 0.917 → 1.000 → 1.000 → 1.000 → 1.000 | 1.710e-4 | 5.803e-3 |
| accessible | bptt-sgd (matched) | 200 | 20 | 0.0200 | 1.0000 | 0.0000 | 1.000 → 1.000 → 1.000 → 1.000 → 1.000 | 2.765e-4 | 2.415e-4 |
| accessible | bptt-sgd (matched) | 200 | 20 | 0.0800 | 1.0000 | 0.0000 | 1.000 → 1.000 → 1.000 → 1.000 → 1.000 | 3.215e-4 | 3.588e-5 |
| accessible | bptt-sgd (matched) | 1000 | 20 | 0.0010 | 1.0000 | 0.0000 | 0.917 → 1.000 → 1.000 → 1.000 → 1.000 | 3.418e-5 | 5.790e-3 |
| accessible | bptt-sgd (matched) | 1000 | 20 | 0.0050 | 1.0000 | 0.0000 | 1.000 → 1.000 → 1.000 → 1.000 → 1.000 | 5.667e-5 | 1.709e-4 |
| accessible | bptt-sgd (matched) | 1000 | 20 | 0.0200 | 1.0000 | 0.0000 | 1.000 → 1.000 → 1.000 → 1.000 → 1.000 | 6.449e-5 | 2.740e-5 |
| accessible | bptt-sgd (matched) | 1000 | 20 | 0.0800 | 1.0000 | 0.0000 | 1.000 → 1.000 → 1.000 → 1.000 → 1.000 | 7.136e-5 | 5.584e-6 |
| accessible | bptt-sgd (matched) | 1000 | 100 | 0.0010 | 1.0000 | 0.0000 | 1.000 → 1.000 → 1.000 → 1.000 → 1.000 | 1.132e-5 | 1.710e-4 |
| accessible | bptt-sgd (matched) | 1000 | 100 | 0.0050 | 1.0000 | 0.0000 | 1.000 → 1.000 → 1.000 → 1.000 → 1.000 | 1.306e-5 | 2.101e-5 |
| accessible | bptt-sgd (matched) | 1000 | 100 | 0.0200 | 1.0000 | 0.0000 | 1.000 → 1.000 → 1.000 → 1.000 → 1.000 | 1.428e-5 | 4.343e-6 |
| accessible | bptt-sgd (matched) | 1000 | 100 | 0.0800 | 1.0000 | 0.0000 | 1.000 → 1.000 → 1.000 → 1.000 → 1.000 | 1.551e-5 | 9.691e-7 |
| accessible | bptt-adam (reference) | 200 | 20 | 0.0010 | 1.0000 | 0.0000 | 1.000 → 1.000 → 1.000 → 1.000 → 1.000 | 1.549e-4 | 1.198e-10 |
| accessible | bptt-adam (reference) | 1000 | 20 | 0.0010 | 1.0000 | 0.0000 | 1.000 → 1.000 → 1.000 → 1.000 → 1.000 | 1.324e-4 | 1.071e-9 |
| accessible | bptt-adam (reference) | 1000 | 100 | 0.0010 | 1.0000 | 0.0000 | 1.000 → 1.000 → 1.000 → 1.000 → 1.000 | 1.514e-4 | 1.322e-9 |
| immune | feedback-sgd | 200 | 20 | 0.0010 | 0.2517 | 0.0044 | 0.250 → 0.252 → 0.255 → 0.253 → 0.252 | 2.711e-5 | 1.230e-2 |
| immune | feedback-sgd | 200 | 20 | 0.0050 | 0.2400 | 0.0126 | 0.250 → 0.245 → 0.225 → 0.237 → 0.240 | 1.356e-4 | 1.230e-2 |
| immune | feedback-sgd | 200 | 20 | 0.0200 | 0.2500 | 0.0000 | 0.250 → 0.250 → 0.250 → 0.250 → 0.250 | 5.436e-4 | 1.230e-2 |
| immune | feedback-sgd | 200 | 20 | 0.0800 | 0.2500 | 0.0000 | 0.250 → 0.250 → 0.250 → 0.250 → 0.250 | 2.190e-3 | 1.230e-2 |
| immune | feedback-sgd | 1000 | 20 | 0.0010 | 0.2383 | 0.0224 | 0.250 → 0.238 → 0.237 → 0.237 → 0.238 | 2.711e-5 | 1.230e-2 |
| immune | feedback-sgd | 1000 | 20 | 0.0050 | 0.2500 | 0.0132 | 0.242 → 0.245 → 0.245 → 0.248 → 0.250 | 1.356e-4 | 1.230e-2 |
| immune | feedback-sgd | 1000 | 20 | 0.0200 | 0.2500 | 0.0000 | 0.250 → 0.250 → 0.250 → 0.250 → 0.250 | 5.436e-4 | 1.230e-2 |
| immune | feedback-sgd | 1000 | 20 | 0.0800 | 0.2500 | 0.0000 | 0.250 → 0.250 → 0.250 → 0.250 → 0.250 | 2.190e-3 | 1.230e-2 |
| immune | feedback-sgd | 1000 | 100 | 0.0010 | 0.2617 | 0.0233 | 0.238 → 0.237 → 0.247 → 0.248 → 0.262 | 2.711e-5 | 1.230e-2 |
| immune | feedback-sgd | 1000 | 100 | 0.0050 | 0.2800 | 0.0029 | 0.250 → 0.252 → 0.262 → 0.270 → 0.280 | 1.357e-4 | 1.230e-2 |
| immune | feedback-sgd | 1000 | 100 | 0.0200 | 0.2817 | 0.0117 | 0.250 → 0.250 → 0.250 → 0.260 → 0.282 | 5.443e-4 | 1.229e-2 |
| immune | feedback-sgd | 1000 | 100 | 0.0800 | 0.8700 | 0.0651 | 0.250 → 0.250 → 0.283 → 0.495 → 0.870 | 2.185e-3 | 1.196e-2 |
| immune | bptt-sgd (matched) | 200 | 20 | 0.0010 | 0.2533 | 0.0033 | 0.250 → 0.252 → 0.255 → 0.252 → 0.253 | 2.839e-5 | 1.230e-2 |
| immune | bptt-sgd (matched) | 200 | 20 | 0.0050 | 0.2483 | 0.0148 | 0.250 → 0.243 → 0.233 → 0.247 → 0.248 | 1.420e-4 | 1.230e-2 |
| immune | bptt-sgd (matched) | 200 | 20 | 0.0200 | 0.2500 | 0.0000 | 0.250 → 0.250 → 0.250 → 0.250 → 0.250 | 5.665e-4 | 1.230e-2 |
| immune | bptt-sgd (matched) | 200 | 20 | 0.0800 | 0.2500 | 0.0000 | 0.250 → 0.250 → 0.250 → 0.250 → 0.250 | 2.239e-3 | 1.230e-2 |
| immune | bptt-sgd (matched) | 1000 | 20 | 0.0010 | 0.2533 | 0.0192 | 0.250 → 0.245 → 0.250 → 0.245 → 0.253 | 2.839e-5 | 1.230e-2 |
| immune | bptt-sgd (matched) | 1000 | 20 | 0.0050 | 0.2667 | 0.0083 | 0.245 → 0.248 → 0.258 → 0.265 → 0.267 | 1.421e-4 | 1.230e-2 |
| immune | bptt-sgd (matched) | 1000 | 20 | 0.0200 | 0.2500 | 0.0000 | 0.250 → 0.250 → 0.250 → 0.250 → 0.250 | 5.665e-4 | 1.230e-2 |
| immune | bptt-sgd (matched) | 1000 | 20 | 0.0800 | 1.0000 | 0.0000 | 0.250 → 0.250 → 0.250 → 0.508 → 1.000 | 2.362e-3 | 9.690e-3 |
| immune | bptt-sgd (matched) | 1000 | 100 | 0.0010 | 0.3400 | 0.0225 | 0.253 → 0.268 → 0.298 → 0.318 → 0.340 | 2.845e-5 | 1.230e-2 |
| immune | bptt-sgd (matched) | 1000 | 100 | 0.0050 | 0.4517 | 0.0578 | 0.267 → 0.288 → 0.332 → 0.385 → 0.452 | 1.437e-4 | 1.230e-2 |
| immune | bptt-sgd (matched) | 1000 | 100 | 0.0200 | 1.0000 | 0.0000 | 0.250 → 0.390 → 1.000 → 1.000 → 1.000 | 5.680e-4 | 3.133e-3 |
| immune | bptt-sgd (matched) | 1000 | 100 | 0.0800 | 1.0000 | 0.0000 | 1.000 → 1.000 → 1.000 → 1.000 → 1.000 | 8.291e-4 | 1.396e-4 |
| immune | bptt-adam (reference) | 200 | 20 | 0.0010 | 1.0000 | 0.0000 | 0.473 → 0.503 → 0.942 → 1.000 → 1.000 | 1.974e-4 | 8.080e-3 |
| immune | bptt-adam (reference) | 1000 | 20 | 0.0010 | 1.0000 | 0.0000 | 1.000 → 1.000 → 1.000 → 1.000 → 1.000 | 1.462e-4 | 1.951e-6 |
| immune | bptt-adam (reference) | 1000 | 100 | 0.0010 | 1.0000 | 0.0000 | 1.000 → 1.000 → 1.000 → 1.000 → 1.000 | 1.070e-4 | 1.473e-9 |

## Questions

| ID | Question | Measured | Verdict |
|---|---|---|---|
| Q1 | Does `feedback-sgd` learn the **rate-accessible** task at all? | best 1.0000 | learns |
| Q2 | How much of the v144 gap is optimiser? (`bptt-adam` − `bptt-sgd`) | +0.0000 | descriptive |
| Q3 | Shortcut effect at matched optimiser (accessible − immune, feedback arm) | +0.1300 | descriptive |

## Interpretation

**Q1 passes, Q3 supported.** The local arm learns the rate-accessible task (1.0000, 95% CI [0.9812, 1.0000]) and loses +0.1300 going to the rate-immune construction. That is the shortcut effect, measured at matched optimiser. Of the original v144 gap, +0.0000 is attributable to Adam-vs-SGD alone and must be subtracted before any credit-assignment claim.

## Non-claims

- Q2 and Q3 are **descriptive**; neither is a preregistered hypothesis test.
- A `--quick` run is non-citable.
- This run fixes difficulty at the easiest setting. It says nothing about harder ones.
- `bptt-adam` is a best-achievable reference, **not** a matched ceiling. Do not quote a gap against it as a credit-assignment result.
