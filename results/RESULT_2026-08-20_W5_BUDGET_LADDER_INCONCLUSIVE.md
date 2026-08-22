# The attention arm's final doubling buys +0.0064 — but the control fails its own convergence bound, so W5-1 is uninterpretable as registered

**Protocol:** `PREREG_2026-08-19_SHD_ATTENTION_BUDGET_LADDER.md` (wave 5),
registered before any e800 cell existed. Verdicts computed once at n=12.
**Cells:** 24 of 24, **0 voided**, joining the e400 rung from wave 1.
**Backend:** rust on Linux/aarch64, binary `22d97c51`, one machine.

---

## 1. The ladder

| arm | e400 | e800 | final doubling |
|---|---:|---:|---:|
| `ff+fixed` | 0.7062 | 0.7164 | **+0.010233** |
| `ff+fixed+attn` | 0.7483 | 0.7546 | **+0.006368** |

n=12 at every cell, seeds shared across rungs and arms.

## 2. The registered verdicts

| ID | measured | threshold | verdict |
|---|---:|---|---|
| **W5-1** | attn final doubling **+0.006368** | < 0.01 | **SUPPORTED** — converged at e400 |
| **W5-2** | control final doubling **+0.010233** | < 0.01 | **NOT SUPPORTED** |
| **W5-3** | contrast +0.0421 → +0.0382, moved 0.0039 | within 0.02 | **SUPPORTED** |

## 3. W5-2's failure disqualifies W5-1, by prior agreement

This document's §3 registered W5-2 as *a control on the control* and stated the
consequence in advance:

> If `ff+fixed` fails its own registered convergence bound on this machine while
> the macOS record has it at +0.000294, something about the Linux runs differs
> beyond the ~0.005 divergence already measured, **and W5-1 is uninterpretable
> until that is explained.**

The control's final doubling is **+0.010233** against a **0.01** bound. It fails.

**So W5-1 does not settle the convergence question, and this campaign still has no
budget-invariant statement about whether the attention arm is converged.** That is
the registered consequence and it is applied, not softened, even though W5-1's own
number is clean and comfortably inside its bound.

## 4. What is and is not known about the discrepancy

The failure is **marginal**: +0.010233 against 0.01 exceeds the bound by 0.000233,
about 2%. A bound this close to the measurement is not a strong signal in either
direction, and that cuts both ways — it is not evidence the runs are sound either.

Two candidate explanations, **both unverified, neither claimed**:

1. **Seed count.** The macOS +0.000294 is an n=3 figure
   (`RESULT_2026-08-03_BUDGET_CONVERGENCE_CEILING_RESTORED.md`); this is n=12. A
   three-seed estimate of a small difference carries wide error, so the two
   numbers may not be in conflict at all.
2. **Genuinely different late-training behaviour** on this libm, compounding over
   800 epochs — a larger effect than the ~0.005 the cross-machine gate measured at
   e20, but measured at a very different budget.

Distinguishing them needs an **e1600 rung**, and §5 of the prereg explicitly
refuses to authorise one: *"If W5-1 fails, the honest report is 'not converged at
e400 or e800', not a longer ladder appended until it passes."* The same refusal
applies to appending a rung to rescue W5-2. **A separate registration is
required.**

## 5. What still stands, independently of this wave

W1-4's **UNDERTRAINED** verdict on the attention arm remains contradicted by
evidence that does not depend on wave 5 at all:

- the arm peaks at **e50 (0.7539)** and is *lower* at e400 (0.7483) and at e800
  (0.7546 — within noise of e50 after 750 more epochs);
- accuracy flat-to-declining while loss falls fast is the **overfitting** branch of
  the instrument's own rule, which reads *"the budget is sufficient"*;
- wave 7 brackets convergence at **(5, 10] epochs**, four hundred epochs before
  W1-4 claims the arm is still learning.

**W5-3 also holds**: the attention contrast is +0.0421 at e400 and +0.0382 at
e800, moving 0.0039. The wave-1 finding is not an artefact of stopping at e400 —
it is the same contrast at twice the budget.

## 6. Consequence for the paper

Nothing in the paper may describe either arm as "converged" on the strength of
this wave. The supportable phrasing is **"measured at e400 and e800, with the
contrast stable across both"**, which is what W5-3 licenses and is sufficient for
the sample-efficiency claim — that claim rests on waves 6 and 7, which are
unaffected.
