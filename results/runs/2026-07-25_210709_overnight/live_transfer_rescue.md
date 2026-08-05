# Matched Online-FB Schedule Contrast Report

**Protocol Version:** 132  
**Experiment ID:** live-transfer-rescue (historical name; **matched-only**)  
**Schedule:** FULL SCIENTIFIC (n=20)  
**Substrate:** matched dense-LIF — **not** live Engine / muted-θ / k-WTA  
**claim_axis:** exploratory matched schedule (not MUST live-transfer)  
**must_not_claim:** live k-WTA PASS; Gate G2 cleared on live C1; breaks transfer barrier  

**Gap-closed:** clamped to `[0, 1]` via `binn_lab::guards::gap_closed_clamped`, identical to the C1 runner.  

## Matched Accuracy Summary (Mean ± SE)

| Arm | Mean Accuracy | SE | Gap Closed Mean | Gap Closed LCB (95%) | Floor (≥0.65) | Verdict |
|---|---:|---:|---:|---:|---|---|
| Baseline Flat (±1 Broadcast) | 0.5410 | 0.0190 | — | — | INVALID_HARNESS | INVALID_HARNESS |
| Frozen REINFORCE×B_i (v12) | 1.0000 | 0.0000 | — | — | INVALID_HARNESS | INVALID_HARNESS |
| **Online Learned B_i Alignment** | **1.0000** | **0.0000** | **1.0000** | **1.0000** | **INVALID_HARNESS** | **INVALID_HARNESS** |
| Gradient Reference Ceiling | 0.9895 | 0.0057 | 1.0000 | 1.0000 | reference | reference |

## Harness health

**HARNESS WARNING — ceiling inverted.** 3 of 20 seeds produced a raw gap-closed above 1.0, i.e. the learned-FB arm beat the gradient reference it is supposed to be bounded by. On this matched task (`CoincidenceTask`, N_IN = 2, difficulty 0.05) that indicates task saturation rather than a credit-assignment result: an arm scoring 1.0000 ± 0.0000 across every seed while the BPTT reference scores below it means the task can no longer separate the arms. Gap-closed is clamped to [0, 1]; no PASS is permitted while this warning is present. Seeds excluded for insufficient reference separation (< 0.15): 0 / 20.

## Scientific Verdict

- Accuracy floor (≥0.65): **INVALID_HARNESS**
- Gap-closed LCB (>0.5): **INVALID_HARNESS**
- Online Learned B_i Alignment (matched-only): INVALID_HARNESS — the arm exceeded its own gradient ceiling; no PASS/FAIL claim permitted
- Live transfer package remains v13–v24 **FAIL** (see PUBLISHABLE_CLAIMS §2b–2c).
