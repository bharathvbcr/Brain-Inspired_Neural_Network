> **SUPERSEDED 2026-07-25** — see `results/HARD_AUDIT_v12_2026-07-25.md`.
>
> Gap-closed was computed unclamped as `(acc-0.5)/(grad-0.5).max(1e-4)`, producing 1.0244 -- the arm beat the ceiling it is meant to be bounded by. The Frozen-REINFORCE verdict cell was the hardcoded literal `PASS (matched)`. The 1.0000 +/- 0.0000 result reflects task saturation (`CoincidenceTask`, N_IN = 2), not credit-assignment quality.
>
> Fixes landed in the same commit; re-run before citing any number from this file.

# Matched Online-FB Schedule Contrast Report

**Protocol Version:** 131  
**Experiment ID:** live-transfer-rescue (historical name; **matched-only**)  
**Schedule:** FULL SCIENTIFIC (n=20)  
**Substrate:** matched dense-LIF — **not** live Engine / muted-θ / k-WTA  
**claim_axis:** exploratory matched schedule (not MUST live-transfer)  
**must_not_claim:** live k-WTA PASS; Gate G2 cleared on live C1; breaks transfer barrier  

## Matched Accuracy Summary (Mean ± SE)

| Arm | Mean Accuracy | SE | Gap Closed Mean | Gap Closed LCB (95%) | Clears Floor (≥0.65)? | Verdict |
|---|---:|---:|---:|---:|---|---|
| Baseline Flat (±1 Broadcast) | 0.5410 | 0.0190 | — | — | FAIL | FAIL |
| Frozen REINFORCE×B_i (v12) | 1.0000 | 0.0000 | — | — | ✓ | PASS (matched) |
| **Online Learned B_i Alignment** | **1.0000** | **0.0000** | **1.0244** | **0.9983** | **✓** | **PASS (matched schedule)** |
| Gradient Reference Ceiling | 0.9895 | 0.0057 | 1.0000 | 1.0000 | ✓ | CEILING |

## Scientific Verdict

- Online Learned B_i Alignment (matched-only): MATCHED thresholds cleared (dense-LIF schedule only — NOT live G2 / NOT live transfer)
- Live transfer package remains v13–v24 **FAIL** (see PUBLISHABLE_CLAIMS §2b–2c).
