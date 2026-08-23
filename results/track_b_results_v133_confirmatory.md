# Track B Rescue Experiment Report

**Protocol Version:** 132  
**Experiment ID:** track-b-rescue (schedule name; not a `c1-*-<hex>` config hash)  
**Schedule:** FULL SCIENTIFIC (n=20, seed block s_idx 20..40)  
**Substrate:** matched dense-LIF — G2-numeric thresholds only (not live Engine G2)  

**Gap-closed:** clamped to `[0, 1]` via `binn_lab::guards::gap_closed_clamped`, identical to the C1 runner. Seeds whose reference is within 0.15 of chance are excluded rather than divided through.  

## Accuracy Summary (Mean ± SE)

| Arm | Mean Accuracy | SE | Gap Closed Mean | Gap Closed LCB (95%) | Floor (≥0.65) | Gap LCB (>0.5) |
|---|---:|---:|---:|---:|---|---|
| Baseline Flat (±1) | 0.9110 | 0.0104 | — | — | PASS | — |
| Graded Broadcast | 0.9120 | 0.0107 | — | — | PASS | — |
| Frozen REINFORCE×B_i | 1.0000 | 0.0000 | — | — | PASS | — |
| **E1.1 Graded RPE Critic** | **0.5715** | 0.0352 | 0.1430 | **0.0051** | **FAIL** | **FAIL** |
| **E1.3 Online Learned FB** | **1.0000** | 0.0000 | 1.0000 | **1.0000** | **PASS** | **PASS** |
| Gradient Ceiling | 1.0000 | 0.0000 | 1.0000 | 1.0000 | reference | reference |

## Harness health

Ceiling health: no seed exceeded the gradient reference; gap-closed is identifiable.

Seeds excluded from gap-closed for insufficient reference separation (< 0.15): RPE 0 / 20, learned-FB 0 / 20.

## Scientific Verdict

- E1.1 RPE Critic: **FAIL**
- E1.3 Online Learned FB: **PASS**
- Matched dense-LIF schedule only — **not** live Engine G2.
