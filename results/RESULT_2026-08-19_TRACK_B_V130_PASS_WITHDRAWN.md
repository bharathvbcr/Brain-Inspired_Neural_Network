# The `track-b-rescue` v130 PASS is withdrawn — the arm reports INVALID_HARNESS at v131

**Date:** 2026-08-19
**Discharges:** `TODO_2026-08-07_OPEN_WORK.md` §1, first two items.
**Source of the defect:** `AUDIT_2026-08-07_JULY_CAMPAIGN_SCORING_PATH.md`.
**Fresh report:** `results/track_b_results_v131.md`.

---

## 1. What was predicted, and what happened

The open-work register named three possible outcomes in advance:

> warning fires and the PASS is withdrawn; warning does not fire and the arm
> stands on a clamped bound; or the arm no longer reaches 1.0000.

**The first outcome occurred.** The re-run at protocol v131 emits:

> **HARNESS WARNING — ceiling inverted.** 0 of 20 RPE seeds and **3 of 20
> learned-FB seeds produced a raw gap-closed above 1.0**, i.e. the arm beat the
> gradient reference it is supposed to be bounded by. This indicates a saturated
> task or an undertrained ceiling, not a credit-assignment result. […] no PASS is
> permitted while this warning is present.

## 2. The correction, field by field

| arm | v130 (on disk, stale) | v131 (current code) |
|---|---|---|
| E1.3 Online Learned FB | accuracy 1.0000, gap-closed **0.9988**, **PASS (matched)** | accuracy 1.0000, gap-closed 1.0000, **INVALID_HARNESS** |
| E1.1 Graded RPE Critic | FAIL | **INVALID_HARNESS** |
| Gradient Ceiling | 0.9930 ± 0.0038 | 0.9930 ± 0.0038 (unchanged) |

The accuracy did not move. **What moved is whether the number is allowed to mean
anything**, and the answer is that it is not: three seeds in twenty had the local
arm outscoring the gradient reference that bounds it. A gap-closed statistic
computed against an inverted ceiling is not a measurement of credit assignment.

## 3. What this costs the paper

`PAPER_DRAFT.md` opens on this claim. The abstract, the results table, the
skeleton, the publishable-claims register, the claim freeze and the repro
checklist all cite the v130 PASS; each already carries a 2026-08-07 citation
warning, and those warnings are now **discharged into a verdict** rather than
left pending.

**The remaining work is authorial, not computational.** The banners now state the
outcome, but the abstract still argues from a PASS that no longer exists, and
rewriting a paper's central claim is not a mechanical edit. What the record now
supports is narrower and is stated here so the rewrite has something to stand on:

- The E1.3 arm reaches ceiling accuracy on the matched dense-LIF schedule.
- Its **gap-closed statistic is uninterpretable** on this task at this budget,
  because the reference it is normalised against is not a ceiling in 3/20 seeds.
- Nothing here bears on the live Engine G2 result, which is unchanged.
- The DFA (`c1-dfa-c8c4fe0899908b84`) and RL (`c1-rl-42eddc9c801308e9`) matched
  PASSes are **not** affected — they ran through the clamped `runner.rs` path, as
  the 2026-08-07 audit already established.

## 4. The pattern this closes

The register's own words: **a fix that is not re-run is not a fix.** The
clamp-and-separation gate was written on 2026-07-25 and the report it invalidates
sat on disk for 25 days, cited six times, while the corrected behaviour lived
only in a Rust source file. The gate worked the moment it was executed.

`ei-inhibition-sweep` was re-run in the same pass and is at
`results/ei_inhibition_results_v135.md`; its numbers also differ materially from
the stale v133 report on disk. `deep-snn-scaling` at v134 is still running.
