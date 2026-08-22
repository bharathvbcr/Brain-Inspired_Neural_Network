# Deep SNN Scaling Report

> **SUPERSEDED AND WITHDRAWN — 2026-08-20.** The v134 re-run is on disk at
> [`deep_snn_results_v134.md`](deep_snn_results_v134.md) and it does not restate
> this result; it removes the basis for one. **Every depth-matched gradient
> ceiling in the v134 run sits at chance** (0.4880 / 0.5000 / 0.5000 / 0.5000 on a
> two-class task), including at depth 1, on the same frozen splits that the
> learned-feedback arm solves at 1.0000 in the same process. The suite is
> `INVALID_HARNESS`.
>
> **The 1.0000 → 0.4525 collapse below may not be cited** — not as local learning
> failing with depth, not as anything. Neither may the 1-hidden-layer 1.0000: it
> is a treatment with no reference.
>
> See [`RESULT_2026-08-20_DEEP_SNN_V134_CEILING_IS_AT_CHANCE.md`](RESULT_2026-08-20_DEEP_SNN_V134_CEILING_IS_AT_CHANCE.md).
> The original staleness banner is retained below for the record.
>
> ---
>
> **STALE — this report is v132; the source is v134.** It predates both
> 2026-07-25 fixes recorded in `deep_snn_scaling.rs:7-21`: the hardcoded `PASS`
> verdict string, and the use of a **1-hidden-layer ceiling for every depth arm**,
> which means the collapse below was measured against the wrong reference.
>
> **Separately, and not a defect:** `CoincidenceTask` has `N_IN = 2`, so a 256⁴
> stack on two-dimensional near-noiseless input has no depth structure to exploit
> (`deep_snn_scaling.rs:22-26`). A depth result on this task is weak evidence
> either way, and the 1.0000 → 0.4525 collapse **must not** be cited as local
> learning failing with depth.
>
> Note also that the 1-hidden-layer arm at 1.0000 exceeds its own gradient
> ceiling of 0.9895. See `AUDIT_2026-08-07_JULY_CAMPAIGN_SCORING_PATH.md` §3 and
> `TODO_2026-08-07_OPEN_WORK.md` §1.


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
