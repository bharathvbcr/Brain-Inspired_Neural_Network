# Preregistration — budget ladder for the attention arm (wave 5)

**Registered:** 2026-08-19, **before any e800 cell existed** — zero in the plan,
zero in `results/`, zero in `claims/`, checked and recorded at registration time.
**Adds to:** `PREREG_2026-08-19_SHD_ATTENTION_CAMPAIGN.md` as wave 5. Changes no
existing hypothesis, threshold, arm or stopping rule.
**Exists because of:** `DEFECT_2026-08-19_W1_4_THRESHOLD_IS_NOT_BUDGET_INVARIANT.md`.

---

## 1. Why

W1-4 asks whether the attention arm is converged at e400 and answers it with
`tail_loss_improvement > −0.02`. That statistic is measured over the final tenth
of training, so its window is 2 epochs at e20 and 40 at e400; the bound was
calibrated on the 2-epoch version and rejects even the known-converged `ff+fixed`
reference at e400. It is being reported as registered and is being disclosed as
uninformative in the same breath.

That leaves the actual question — *is the attention arm converged?* — unanswered.
This wave answers it with the instrument's **own registered rule**, the one that
closed the budget axis for `ff+fixed`:

> `AMENDMENT_2026-08-03_CONVERGENCE_RULE_FINAL_DOUBLING.md` — compare the **final
> doubling** of the budget ladder against a fixed **0.01** bound. A final
> doubling that buys less than 0.01 means the budget is sufficient.

That rule is budget-invariant by construction: it compares two adjacent rungs,
not a window whose length moves with the schedule.

## 2. Registered schedule

24 cells, at the anchor (`published-2ms` / `adjacent-sum-5`), h128, 12 seeds:

| arm | epochs | role |
|---|---:|---|
| `ff+fixed+attn` (d32, L1) | 800 | the rung under test |
| `ff+fixed` | 800 | control at the same rung |

The e400 rung already exists as wave 1. The ladder is therefore e400 → e800 for
both arms, with seeds shared across rungs and across arms.

## 3. Hypotheses

| ID | statement | threshold |
|---|---|---|
| **W5-1** (primary) | The attention arm is converged at e400 | mean accuracy gain from e400 → e800 is **< 0.01** |
| **W5-2** | The control is converged at e400 on this machine | same bound, applied to `ff+fixed`; the recorded macOS value for this doubling is **+0.000294** |
| **W5-3** | The wave-1 contrast survives the longer budget | mean(attn) − mean(ff+fixed) at e800 is **within 0.02** of the same contrast at e400 |

**W5-1 is what W1-4 was reaching for.** If it holds, wave 1's accuracy is a
property of the architecture and may be described as converged. If it does not,
wave 1's numbers are a budget artefact and must be reported as measured-at-e400
and nothing more — which is exactly what the ceiling claim was forced to do on
2026-08-03 before the budget probe closed.

**W5-2 is a control on the control.** If `ff+fixed` fails its own registered
convergence bound on this machine while the macOS record has it at +0.000294,
something about the Linux runs differs beyond the ~0.005 divergence already
measured, and W5-1 is uninterpretable until that is explained.

## 4. What must not be claimed

That e800 is "the converged budget" in general. It is one rung. The registered
rule tests the *final* doubling available, and if a further rung is ever needed
that is a further registration.

That W5-1 rescues W1-4. W1-4's verdict stands as registered and reported; this
wave measures the underlying question with a different, budget-invariant test,
and the two are reported separately.

## 5. Stopping rule

**Twelve seeds, one rung, verdicts computed once.** No e1600 rung is authorised
by this document. If W5-1 fails, the honest report is "not converged at e400 or
e800", not a longer ladder appended until it passes — the failure mode the
2026-08-03 amendment exists to prevent.
