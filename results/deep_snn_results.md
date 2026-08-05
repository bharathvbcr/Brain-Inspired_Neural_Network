# Deep SNN Scaling Report

**Protocol Version:** 132  
**Experiment:** deep-snn-scaling  
**Schedule:** FULL SCIENTIFIC (n=20)  

```
claim_axis: exploratory (not MUST / not Gate G2)
object_under_test: matched Learned FB depth (1 vs 2 hidden)
must_not_claim: Gate G2; live k-WTA; Foundation depth unlock
```

## Scaling Accuracy Summary (Mean ± SE)

| Arm | Hidden Architecture | Mean Accuracy | SE | Clears Floor (≥0.65)? | Verdict |
|---|---|---:|---:|---|---|
| 1-Hidden-Layer Learned FB | 256 | 1.0000 | 0.0000 | ✓ | PASS (exploratory) |
| **2-Hidden-Layer Deep Learned FB** | **256 → 256** | **0.4525** | **0.0327** | **FAIL** | **FAIL** |
| Single-Layer Gradient Ceiling | 256 | 0.9895 | 0.0057 | ✓ | CEILING |

## Verdict

- Deep 2-Hidden-Layer Feedback Alignment: **FAIL** (floor not cleared; prior row Verdict=PASS was a report integrity bug).
