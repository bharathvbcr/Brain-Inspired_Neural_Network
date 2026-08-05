# Matched Online-FB Schedule Contrast Report

**Protocol Version:** 131  
**Experiment ID:** `live-transfer-rescue` (historical name; **matched-only**)  
**Schedule:** FULL SCIENTIFIC (n=20)  
**Substrate:** matched dense-LIF — **not** live Engine / muted-θ / k-WTA  

```
claim_axis: exploratory matched schedule (not MUST live-transfer)
object_under_test: MatchedRlLearnedFb vs flat / frozen RFB / SuperSpike on dense-LIF
may_claim: Matched online learned-B_i clears matched accuracy/gap thresholds on this schedule
must_not_claim: live k-WTA PASS; Gate G2 cleared on live C1; “breaks the substrate transfer barrier”
```

## Matched Accuracy Summary (Mean ± SE)

| Arm | Mean Accuracy | SE | Gap Closed Mean | Gap Closed LCB (95%) | Clears Floor (≥0.65)? | Verdict |
|---|---:|---:|---:|---:|---|---|
| Baseline Flat (±1 Broadcast) | 0.5410 | 0.0190 | — | — | FAIL | FAIL |
| Frozen REINFORCE×B_i (v12) | 1.0000 | 0.0000 | — | — | ✓ | PASS (matched) |
| **Online Learned B_i Alignment** | **1.0000** | **0.0000** | **1.0244** | **0.9983** | **✓** | **PASS (matched schedule)** |
| Gradient Reference Ceiling | 0.9895 | 0.0057 | 1.0000 | 1.0000 | ✓ | CEILING |

## Scientific Verdict

- Online Learned B_i Alignment: **PASS (matched schedule only)** — dense-LIF thresholds cleared; **NOT** live G2 / **NOT** live transfer.
- Live transfer package remains **v13–v24 FAIL** ([`PUBLISHABLE_CLAIMS.md`](PUBLISHABLE_CLAIMS.md) §2b–2c; [`PAPER_DRAFT.md`](PAPER_DRAFT.md) §4.2).
