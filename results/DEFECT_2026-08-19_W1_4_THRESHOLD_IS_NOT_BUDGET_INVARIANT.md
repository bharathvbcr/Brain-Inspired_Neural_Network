# W1-4's threshold is not budget-invariant — disclosed before the cells it judges existed

**Date:** 2026-08-19
**Affects:** `PREREG_2026-08-19_SHD_ATTENTION_CAMPAIGN.md` §1, hypothesis W1-4.
**Disclosed:** with **14 control cells complete and ZERO attention cells
complete**. The verdict W1-4 renders is on attention cells, none of which exist
yet. Verifiable: `s3://binn-campaign-.../results/` contains no `*attn*` object at
the time of writing, and the campaign log records the order.

**This document does not change the threshold.** W1-4 will be computed and
reported exactly as registered. What follows is the disclosure that its verdict
is partly an artefact of how the statistic is built, so that the verdict is not
read as more than it is.

---

## 1. What was registered

> **W1-4** — Converged, not undertrained: `tail_loss_improvement` **> −0.02** in
> every attention cell.

The threshold was calibrated against the pilot, where the attention arms sat at
**−0.149** and the controls at **−0.011** at **e20**. A bound of −0.02 cleanly
separated those.

## 2. The defect

`tail_loss_improvement` is the fractional loss change across the final **tenth**
of training (`shd_instrument.rs:781`):

```rust
let tail = (epochs / 10).max(1);
let earlier = epoch_loss[epoch_loss.len() - tail - 1];
let later   = epoch_loss[epoch_loss.len() - 1];
(later - earlier) / earlier
```

The window **scales with the budget**. At e20 it spans 2 epochs; at e400 it spans
**40**. A fractional change measured over 40 epochs is not comparable to one
measured over 2, and −0.02 was calibrated on the 2-epoch version.

The converged control cells now on disk make this concrete. `ff+fixed` at
h128/e400 — an arm the record already calls converged, whose budget axis closed
at +0.000294 for a final doubling — reports `tail_loss_improvement` between
**−0.031 and −0.036**. By W1-4's bound, the arm this project has spent months
establishing as converged would be declared **undertrained**.

A threshold that rejects the known-converged reference is measuring its own
window length.

## 3. This is a repeat of a defect this project already found and fixed

`AMENDMENT_2026-08-03_CONVERGENCE_RULE_FINAL_DOUBLING.md` records that the
original convergence rule *"compared the endpoints of the ladder, so extending
the ladder could never escape UNDERTRAINED — the measured gain grew
monotonically the more evidence was collected."* The amendment replaced it with a
**final-doubling** test against a fixed 0.01 bound, precisely because that
comparison does not move with the budget.

W1-4 reintroduced the same class of error from a different direction: not a
moving comparison, but a moving *window*. The registered instrument already
carries the correct tool and W1-4 did not use it.

## 4. What will be reported

1. **W1-4 exactly as registered**, whichever way it falls, with the raw
   `tail_loss_improvement` values for both arms.
2. **The control arm's value beside it**, always. `attention −0.14 vs control
   −0.03` and `attention −0.04 vs control −0.03` are completely different
   findings, and the registered bound cannot distinguish them.
3. **The registered consequence, applied as written.** If W1-4 fails, W1-1 is
   reported as **UNTESTED**, per the prereg's named outcomes. That consequence is
   not being softened here.
4. This defect note, cited from the result.

## 5. What is *not* being done

- The threshold is **not** being amended. Wave-1 control cells already exist;
  changing a bound after any of its wave's data has landed is indistinguishable
  from choosing the bound that gives the preferred answer, whatever the
  justification. The W2-1 amendment was legitimate because **no** wave-2 cell had
  been claimed; that is not the situation here.
- The cells are **not** being discarded. They are valid measurements of what they
  measure.

## 6. The correct test, for separate registration

A budget-invariant convergence test on the attention arm, using the instrument's
own registered rule: run the arm at e400 and e800, and require the **final
doubling** to buy less than the registered **0.01** in accuracy. That is the test
that closed the budget axis for `ff+fixed`, it is already registered, and it
costs one extra ladder rung per seed.

Until that runs, no claim in this campaign may describe the attention arm as
"converged". It may only report the budget it was measured at.
