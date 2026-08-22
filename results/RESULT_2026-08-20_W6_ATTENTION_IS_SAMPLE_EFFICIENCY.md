# The attention read-out reaches its accuracy by epoch 20 and never improves — the result is sample efficiency, not a higher ceiling

> ## REFINED 2026-08-20 — the convergence point is now bracketed at (5, 10]
>
> §5 of this document flagged that e20 was the ladder's floor and the true
> convergence point was unmeasured. Wave 7 measured it:
> `RESULT_2026-08-20_W7_CONVERGENCE_IS_BRACKETED.md`.
>
> **e5 reaches 90.3% of the arm's e400 accuracy (fails the 0.95 criterion) and
> e10 reaches 98.1% (passes).** Convergence is therefore in `(5, 10]` epochs, not
> "20". Every W6 verdict below stands; the "20 epochs" figure is an upper bound
> that has since been tightened.

**Protocol:** `PREREG_2026-08-19_SHD_ATTENTION_LEARNING_CURVE.md` (wave 6),
registered before any e20 or e50 cell existed. Verdicts computed once, at the
registered n=12.
**Cells:** 48 of 48, **0 voided**, plus the e400 rung from wave 1 (60/60).
**Backend:** rust on Linux/aarch64, binary `22d97c51`, one machine throughout —
which is the entire point of this wave.

```
claim_axis: architecture
may_claim: That on this forward model, at this contract and width, the attention
  read-out reaches ~0.748 within 20 epochs and does not improve with twenty times
  that budget, while the control needs the full budget to reach 0.706.
must_not_claim: Any comparison against the macOS-recorded 0.7032 / 0.7378 - every
  instance FAILED the cross-machine gate. That attention "converges at epoch 20":
  e20 is this ladder's floor and it was already converged there, so the true
  point is somewhere at or below it and unmeasured. Anything about local
  learning - these are BPTT reference arms.
```

---

## 1. The ladder

| epochs | `ff+fixed` | `ff+fixed+attn` | gain |
|---:|---:|---:|---:|
| 20 | 0.5851 | **0.7479** | **+0.1627** |
| 50 | 0.6484 | **0.7539** | +0.1055 |
| 400 | 0.7062 | **0.7483** | +0.0421 |

n=12 at every cell, seeds shared across rungs and arms.

## 2. The registered verdicts — all three supported

| ID | measured | threshold | verdict |
|---|---:|---|---|
| **W6-1** | attn(e20)/attn(e400) = **0.9995** | ≥ 0.95 | **SUPPORTED** |
| **W6-2** | ff+fixed(e20)/ff+fixed(e400) = **0.8286** | < 0.90 | **SUPPORTED** |
| **W6-3** | gain(e20) − gain(e400) = **+0.1207** | ≥ 0.05 | **SUPPORTED — a budget effect** |

**0.9995.** The attention arm is at 99.95% of its 400-epoch accuracy after 20
epochs. Twenty times more training buys it **+0.0004**. The control, on the same
data with the same seeds, is at 82.9% and needs the whole budget.

## 3. What this settles

**The pilot was right about what it measured.** Its +0.1702 at e20 replicates
here same-machine at **+0.1627**; the 0.0075 difference is the scale of the
cross-machine divergence already measured
(`MEASUREMENT_2026-08-19_CROSS_MACHINE_BIT_EXACTNESS.md`). The pilot's error was
never the number — it was that a 20-epoch contrast reads as an architecture
result. It is a *budget* result, and W6-3 now says so with a threshold registered
in advance.

**W1-1's failure is explained rather than merely recorded.** The converged gap is
+0.0421 because both arms end up near the same place; the attention arm simply
gets there twenty times sooner. A hypothesis about the converged gap was asking
the wrong question of this arm, and it is the wave-1 negative plus this ladder
together that make that visible.

**W1-4's UNDERTRAINED verdict is confirmed backwards.** The attention arm peaks at
**e50 (0.7539)** and is *lower* at e400 (0.7483). Accuracy declining while loss
falls fast (−0.246) is overfitting, not undertraining, exactly as
`DEFECT_2026-08-19_W1_4_THRESHOLD_IS_NOT_BUDGET_INVARIANT.md` said before any of
these cells existed. W1-4's verdict stands as registered and reported; its
*interpretation* is now measured to be inverted. Wave 5's e800 rung will state it
in the instrument's own budget-invariant terms.

## 4. What the paper can claim

Not *"attention raises the ceiling"* — wave 1 killed that at n=12, and the
ceiling language must not appear.

What is supported, on this forward model at this contract and width:

1. **Sample efficiency.** The attention read-out reaches its accuracy within 20
   epochs and holds it; the rate-only control needs 400 to reach a lower one.
   +0.1627 at e20, all twelve seeds.
2. **The mechanism is temporal order.** On bin-shuffled data the attention arm is
   *worse* than its control by −0.0492 in **12 of 12 seeds**, against a base-arm
   order sensitivity of 0.0128 — a 7x larger order-derived component
   (`RESULT_2026-08-19_W1_ATTENTION_AT_CONVERGENCE.md` §5).
3. **Neither is capacity.** The h192 control carries more parameters and buys
   +0.0119 against attention's +0.0421.

That is a narrower claim than the one this campaign set out to test, and it is
the one the evidence supports. It is also arguably the more useful one: a
read-out that gets a spiking network to its accuracy twenty times sooner is worth
more than 0.04 of accuracy it does not deliver.

## 5. The obvious next question, deliberately unasked here

The ladder's floor is e20 and the arm was already converged at it, so **where it
actually converges is unmeasured** — it could be e5 or e10. That needs rungs
below e20 and a separate registration. This document does not authorise them, and
"reaches its accuracy by epoch 20" is the strongest statement these cells
support.
