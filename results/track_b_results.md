# Track B Rescue Experiment Report

> **STALE — DO NOT CITE. This report is v130; the source is v131, and the
> 130→131 bump is the fix for the defect visible in this report.**
>
> The `Gap Closed Mean` of **1.0155** below means the arm beat the gradient
> reference it is supposed to be bounded by. `track_b_rescue.rs:39-49` names this
> exact value as the bug it replaces: both rescue harnesses computed
> `(acc − 0.5) / (grad − 0.5).max(1e-4)` with no clamp and no reference-separation
> gate. `runner.rs` has always clamped; these two did not.
>
> Under the current code this arm **cannot be reported as PASS**:
> `track_b_rescue.rs:311-319` emits a `HARNESS WARNING — ceiling inverted` and
> states that no PASS is permitted while it is present.
>
> The fix landed 2026-07-25 (`HARDENING_v12`) and the experiment was never
> re-run. See `AUDIT_2026-08-07_JULY_CAMPAIGN_SCORING_PATH.md` §2 and
> `TODO_2026-08-07_OPEN_WORK.md` §1.

**Protocol Version:** 130  
**Experiment ID:** `track-b-rescue` (**schedule / experiment name — not a `c1-*-<hex>` config hash**)  
**Schedule:** FULL SCIENTIFIC (n=20)  
**Substrate:** matched dense-LIF (SurrogateLifReference family)

```
claim_axis: matched contrast (supporting; hash-hygiene demoted)
object_under_test: MatchedRlRpe vs MatchedRlLearnedFb on dense-LIF
may_claim: Online learned B_i clears matched accuracy/gap thresholds on this schedule
must_not_claim: live k-WTA rescue; config-hash replay via --config-hash track-b-rescue
```

## Accuracy Summary (Mean ± SE)

| Arm | Mean Accuracy | SE | Gap Closed Mean | Gap Closed LCB (95%) | Clears Floor (≥0.65)? | Clears Gap LCB (>0.5)? |
|---|---:|---:|---:|---:|---|---|
| Baseline Flat (±1) | 0.5340 | 0.0258 | — | — | FAIL | — |
| Graded Broadcast | 0.7000 | 0.0562 | — | — | ✓ | — |
| Frozen REINFORCE×B_i | 0.9870 | 0.0130 | — | — | ✓ | — |
| **E1.1 Graded RPE Critic** | **0.5120** | 0.0120 | 0.0240 | **-0.0230** | **FAIL** | **FAIL** |
| **E1.3 Online Learned FB** | **1.0000** | 0.0000 | 1.0155 | **0.9988** | **✓** | **PASS (matched)** |
| Gradient Ceiling | 0.9930 | 0.0038 | 1.0000 | 1.0000 | ✓ | ✓ |

## Scientific Verdict

- E1.1 RPE Critic: FAIL (did not clear matched accuracy/gap thresholds)
- E1.3 Online Learned FB: **PASS (matched schedule)** — G2-numeric thresholds on dense-LIF only; **not** live Engine G2
