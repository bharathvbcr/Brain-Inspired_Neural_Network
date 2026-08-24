# Track B Rescue Experiment Report

> ## SUPERSEDED 2026-08-23 by `RESULT_2026-08-23_TRACK_B_REREAD.md`.
>
> These numbers came from a gradient ceiling built with the wrong constructor —
> recurrent, while every treatment arm is feedforward — over a shared forward
> that could not emit a spike at any seed. Both are repaired
> (`RESULT_2026-08-23_MATCHED_ARCH_REPAIR.md`).
>
> The harness warning below was right that something was wrong and wrong about
> what: it blamed "a saturated task or an undertrained ceiling", and the
> inversions came from the ceiling carrying a `hidden × hidden` matrix no arm had.
> It was right about saturation, which the re-read confirms and makes the headline.
>
> Under a corrected instrument, on a disjoint seed block registered before it ran:
> **0 of 20 inverted, E1.1 FAIL (0.5715), E1.3 PASS** — with E1.3, the ceiling and
> a third arm all at exactly 1.0000, variance 0.000000. The PASS is real by the
> registered rule and establishes nothing about credit assignment.


**Protocol Version:** 132  
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
