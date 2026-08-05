> **RETRACTED 2026-07-25** — see `results/HARD_AUDIT_v12_2026-07-25.md`.
>
> The Verdict column was the literal string `PASS` for all four depth arms regardless of the measurement; rows read `FAIL | PASS`. The ceiling was 1-hidden-layer for every depth arm, so the depth collapse (1.00 -> 0.45) cannot be attributed to feedback alignment.
>
> Fixes landed in the same commit; re-run before citing any number from this file.

# Deep SNN Scaling Report

**Protocol Version:** 133  
**Experiment:** deep-snn-scaling  
**Schedule:** FULL SCIENTIFIC (n=20)  

## Scaling Accuracy Summary (Mean ± SE)

| Arm | Hidden Architecture | Mean Accuracy | SE | Clears Floor (≥0.65)? | Verdict |
|---|---|---:|---:|---|---|
| 1-Hidden-Layer Learned FB | 256 | 1.0000 | 0.0000 | ✓ | PASS |
| 2-Hidden-Layer Deep Learned FB | 256 → 256 | 0.4525 | 0.0327 | FAIL | PASS |
| 3-Hidden-Layer Deep Learned FB | 256³ | 0.5130 | 0.0283 | FAIL | PASS |
| **4-Hidden-Layer Deep Learned FB** | **256⁴** | **0.4500** | **0.0344** | **FAIL** | **PASS** |
| Single-Layer Gradient Ceiling | 256 | 0.9895 | 0.0057 | ✓ | CEILING |

## Verdict

- Deep 4-Hidden-Layer Feedback Alignment: FAIL
