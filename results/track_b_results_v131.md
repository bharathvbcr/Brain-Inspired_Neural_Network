# Track B Rescue Experiment Report

**Protocol Version:** 131  
**Experiment ID:** track-b-rescue (schedule name; not a `c1-*-<hex>` config hash)  
**Schedule:** FULL SCIENTIFIC (n=20)  
**Substrate:** matched dense-LIF — G2-numeric thresholds only (not live Engine G2)  

**Gap-closed:** clamped to `[0, 1]` via `binn_lab::guards::gap_closed_clamped`, identical to the C1 runner. Seeds whose reference is within 0.15 of chance are excluded rather than divided through.  

## Accuracy Summary (Mean ± SE)

| Arm | Mean Accuracy | SE | Gap Closed Mean | Gap Closed LCB (95%) | Floor (≥0.65) | Gap LCB (>0.5) |
|---|---:|---:|---:|---:|---|---|
| Baseline Flat (±1) | 0.5340 | 0.0258 | — | — | INVALID_HARNESS | — |
| Graded Broadcast | 0.7000 | 0.0562 | — | — | INVALID_HARNESS | — |
| Frozen REINFORCE×B_i | 0.9870 | 0.0130 | — | — | INVALID_HARNESS | — |
| **E1.1 Graded RPE Critic** | **0.5120** | 0.0120 | 0.0240 | **-0.0230** | **INVALID_HARNESS** | **INVALID_HARNESS** |
| **E1.3 Online Learned FB** | **1.0000** | 0.0000 | 1.0000 | **1.0000** | **INVALID_HARNESS** | **INVALID_HARNESS** |
| Gradient Ceiling | 0.9930 | 0.0038 | 1.0000 | 1.0000 | reference | reference |

## Harness health

**HARNESS WARNING — ceiling inverted.** 0 of 20 RPE seeds and 3 of 20 learned-FB seeds produced a raw gap-closed above 1.0, i.e. the arm beat the gradient reference it is supposed to be bounded by. This indicates a saturated task or an undertrained ceiling, not a credit-assignment result. Gap-closed is clamped to [0, 1] for reporting; no PASS is permitted while this warning is present.

Seeds excluded from gap-closed for insufficient reference separation (< 0.15): RPE 0 / 20, learned-FB 0 / 20.

## Scientific Verdict

- E1.1 RPE Critic: **INVALID_HARNESS**
- E1.3 Online Learned FB: **INVALID_HARNESS**
- Matched dense-LIF schedule only — **not** live Engine G2.
