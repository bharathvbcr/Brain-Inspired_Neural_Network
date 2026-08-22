# Amendment: re-run wave 4 without gradient clipping

**Registered:** 2026-08-22, **before any cell of the re-run executes.**
**Amends:** `PREREG_2026-08-19_SHD_ATTENTION_CAMPAIGN.md` W4, which registers
`--clip-grad-norm 1.0` for all 24 cells.
**Why:** `FINDING_2026-08-22_WAVE4_KILLED_ITS_OWN_CELLS.md`.

---

## 1. What changes, and it is exactly one thing

`clip_grad_norm` goes from `1.0` to `None`. **Nothing else moves**: same arm
strings (`rec+alif`, `rec+alif+attn`), same width 256, same budget e100, same
surrogate-scale ladder `{1.0, 0.4}`, same 6 seeds, same contract
`published-2ms`, same geometry `adjacent-sum-5`, same attention configuration
`d32/L1`, same binary.

This is not a new design. It restores the configuration under which thirteen
`rec+alif` cells at this exact width and budget already completed with zero
non-finite events.

## 2. Why an amendment rather than an edit

`clip_grad_norm = 1.0` is a registered protocol parameter. Changing it silently
and re-running would be selecting a setting after seeing that the first one
failed, which is the thing preregistration exists to prevent. So it is registered,
with the reason and the falsifiable expectation below, before any cell runs.

The diverged wave is **kept**, not deleted. `wave4_recurrent` in
`scripts/aws/plan_cells.py` is unchanged and still reproduces the 24 aborts; the
re-run is a new wave, `wave11_recurrent_unclipped`.

## 3. Falsifiable expectation, recorded before running

Registered as a **completion** expectation, not an accuracy one. The question
this amendment answers is whether the clip flag caused the divergence — the
scientific question about attention on the recurrent arm is answered by §4.

**Expected:** at least 18 of 24 cells complete with `non_finite_events = 0`.

The bar is 18, not 24, because the marginality is real and independent of
clipping: the unclipped h256/e100 record is 13 of 15, and completing cells still
show gradient peaks to 3.93e33. Demanding 24/24 would be demanding better than
the arm has ever managed unclipped.

**If fewer than 18 complete**, the diagnosis in the finding is incomplete —
clipping was then not the whole cause — and the re-run is reported as such rather
than patched with a third parameter. No further lever is added without its own
amendment.

## 4. Registered hypotheses for the science, unchanged in form from W4

Only evaluable if the completion expectation in §3 holds. Both are two-sided;
there is no directional theory about attention on a recurrent arm.

| id | hypothesis | bar |
|---|---|---|
| **T4-1** | the recurrent arm produces usable cells at all | ≥ 18/24 complete, and mean accuracy of the completing `rec+alif` cells is above chance (0.05) by more than the 0.05 margin `CeilingHealth` already uses |
| **T4-2** | the attention read-out changes recurrent-arm accuracy | \|mean(`rec+alif+attn`) − mean(`rec+alif`)\| ≥ 0.05 at the same surrogate scale, with ≥ 10 of 12 seeds agreeing in sign |
| **T4-3** | surrogate scale matters for the recurrent arm | \|mean(ss1.0) − mean(ss0.4)\| ≥ 0.05, pooled across arms |

## 5. Named outcomes

- **§3 holds and T4-2 is met** → the read-out has a measurable effect on the
  recurrent arm, in whichever direction it falls. First such measurement.
- **§3 holds and T4-2 is not met** → the read-out's effect on the recurrent arm
  is below 0.05 at this width and budget. A real negative, and the first honest
  one for this arm.
- **§3 holds but T4-1 fails** — cells complete but sit at chance → the arm is
  trainable but not learning this task at h256/e100. Reported as a property of
  the arm at this operating point, not as a refutation of recurrence.
- **§3 fails** → see §3. The finding's diagnosis is incomplete and is corrected.

## 6. What this may not claim

- **It does not resurrect the W4 verdict.** That document is withdrawn. This is a
  new wave with a new plan and its own cells; nothing here re-validates it.
- **It does not compare against the 13 archived cells.** Those come from a
  different campaign, a different binary and a different contract mix. They are
  evidence that the arm completes unclipped, which is all they are cited for.
- **It does not touch the headline claim.** The paper's attention result is
  `ff+fixed+attn` at d32/L4 and does not depend on the recurrent arm either way.
- **It says nothing about h512.** `MEASUREMENT_2026-08-03_GRADIENT_CLIPPING_DOES_NOT_FIX_H512.md`
  stands: at h512 the arm aborts with or without clipping. This wave is h256.
