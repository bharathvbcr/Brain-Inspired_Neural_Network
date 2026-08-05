# Temporal eligibility mechanism diagnostic

**Protocol:** v147
**Hash:** `temporal-elig-v147-17a7bfb76ca5896a`
**Difficulty:** `(jitter=0, distractors=4)` only
**Schedule:** 3 fresh seeds, 200/100, 20 epochs, hidden=64, lr=0.005
**Verdict:** **FAIL — corrected matched feedback remains at chance; stop this design**

## Mandatory pre-calibration overfit gate

Training accuracy 0.2500; predicted classes 2/4; majority 0.825; hidden gradient RMS 4.195e-4; hidden step RMS 2.097e-6; readout gradient RMS 5.381e-2; readout step RMS 2.691e-4; replay yes.
Gate (> 0.95 with all classes, live gradients/steps, and exact replay): **no**.

## Frozen easiest-candidate diagnostic

| Seed | RFB train | RFB test | RFB classes | Majority | Hidden grad RMS | Hidden step RMS | Readout grad RMS | Readout step RMS | BPTT train | BPTT test | BPTT classes | Replay |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|
| 9100175362440036353 | 0.2500 | 0.2400 | 3/4 | 0.790 | 4.430e-4 | 2.215e-6 | 5.383e-2 | 2.691e-4 | 1.0000 | 1.0000 | 4/4 | yes |
| 9100175362708472038 | 0.2350 | 0.2400 | 2/4 | 0.970 | 4.222e-4 | 2.111e-6 | 5.382e-2 | 2.691e-4 | 1.0000 | 1.0000 | 4/4 | yes |
| 9100175362976907727 | 0.2300 | 0.2400 | 2/4 | 0.620 | 4.128e-4 | 2.064e-6 | 5.382e-2 | 2.691e-4 | 1.0000 | 1.0000 | 4/4 | yes |

Means: RFB train 0.2383, RFB test 0.2400; BPTT train 1.0000, BPTT test 1.0000.
RFB learning requires mean test >= 0.55; chance-like stop is <= 0.30 while BPTT >= 0.90.

This is a mechanism diagnostic, not a replacement calibration. It runs no optimizer sweep and no difficulty sweep. Protocols v145/v146 remain blocked.
