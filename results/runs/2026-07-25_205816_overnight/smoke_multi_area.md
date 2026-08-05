# Multi-Area Structural Scaling Report

## Learning accuracy across multi-area depths

Dataset seed fixed at `100` across all area counts (n_train=150, n_test=50); area count is the only independent variable.

| Area count (M) | Total cells | Train accuracy | Test accuracy | 95% CI | Constant-predictor baseline | Verdict |
|---:|---:|---:|---:|---|---:|---|
| M = 2 | 64 | 0.9733 | 1.0000 | [0.9286, 1.0000] | 0.5200 | PASS |
| M = 4 | 128 | 0.7600 | 0.9000 | [0.7864, 0.9565] | 0.5200 | PASS |

### Readout audit

| Arm | Accuracy | 95% CI | Constant-predictor baseline | n | Distinct predictions | Distinct states | Defects |
|---|---:|---|---:|---:|---:|---:|---|
| M = 2 | 1.0000 | [0.9286, 1.0000] | 0.5200 | 50 | 2 | 6 | none |
| M = 4 | 0.9000 | [0.7864, 0.9565] | 0.5200 | 50 | 2 | 7 | none |

**Scaling separation (M = 2 vs M = 4):** intervals OVERLAP — no scaling effect is supported by this data. M=2 95% CI [0.9286, 1.0000]; M=4 95% CI [0.7864, 0.9565]. A scaling claim requires non-overlapping intervals.

