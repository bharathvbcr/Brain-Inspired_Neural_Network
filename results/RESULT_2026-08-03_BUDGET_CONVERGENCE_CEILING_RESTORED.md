# Budget AND width are saturated: the ceiling claim is restored at 0.7378

**Date:** 2026-08-03
**Amendment governing the rule:** `AMENDMENT_2026-08-03_CONVERGENCE_RULE_FINAL_DOUBLING.md`, registered before the e800 cells ran
**Backend:** rust only. Binary `8c169a659c3c`, Gate F 13/13 PASS
**Cells:** e800 × 3 seeds, `published-2ms / adjacent-sum-5 / h512`; compared against the existing e400 × 3
**Supersedes:** the withdrawal in `MEASUREMENT_2026-08-03_SHD_BUDGET_AND_ERRATA.md`

---

## claim_axis

```
axis: architecture-ceiling
claim: At published-2ms / adjacent-sum-5, the matched SHD BPTT instrument
  converges to 0.7378 and does not reach the registered 0.80 gate. Both scaling
  axes are closed: doubling epochs 400->800 buys 0.000294 while training loss
  keeps falling (overfitting, not undertraining), and doubling width 512->1024
  at the converged budget buys 0.000883.
may_claim: That e400 is SUFFICIENT by the amended rule (gain 0.000294 <= 0.01);
  that e800 gives 0.737191 +/- 0.003825 over 3 seeds; that the shortfall to the
  registered gate is 0.0622; that h512 is SUFFICIENT on width by the same bound
  (gain 0.000883); and that all three seeds show training loss still falling
  ~6.4% per final decile while test accuracy is flat or declining.
must_not_claim: That BPTT cannot reach 0.80 on this task in general. Budget
  and width are now BOTH saturated (§2, §4), but this is **one contract and
  one geometry** — `published-2ms / adjacent-sum-5`. Geometry is untested at
  convergence and is now the binding scope limit. Nor a SOTA comparison:
  0.7378 against a 0.939 reference. Nothing about BINN.
```

## 1. The ladder

Seed 5170001, `published-2ms / adjacent-sum-5 / h512`:

| epochs | test acc | train loss | gain vs previous |
|---:|---:|---:|---:|
| 100 | 0.716431 | 0.214585 | — |
| 200 | 0.728357 | 0.097903 | **+0.011926** |
| 400 | 0.734541 | 0.045623 | **+0.006184** |
| 800 | 0.732774 | 0.033621 | **−0.001767** |

Three seeds, final doubling:

| seed | e400 | e800 | delta |
|---|---:|---:|---:|
| 5170001 | 0.734541 | 0.732774 | −0.001767 |
| 5170002 | 0.737633 | 0.739399 | +0.001767 |
| 5170003 | 0.738516 | 0.739399 | +0.000883 |
| **mean** | **0.736896** | **0.737191** | **+0.000294** |

## 2. Verdict — SUFFICIENT, and specifically OVERFITTING

**Amended rule (§3 of the amendment), 0.01 bound unchanged from the registered
version:**

> `mean_accuracy(2B) − mean_accuracy(B) ≤ 0.01` → B is SUFFICIENT.

`0.000294 ≤ 0.01` → **e400 is SUFFICIENT.** The ladder is flat at 34× below the
bound.

Which branch it lands in matters, and the registered rule distinguishes them:

| seed | train loss e400 → e800 | final-decile train improvement | test accuracy |
|---|---|---:|---|
| 5170001 | 0.0456 → 0.0336 | −6.07% | down |
| 5170002 | 0.0453 → 0.0333 | −6.74% | flat |
| 5170003 | 0.0459 → 0.0334 | −6.56% | flat |

**Training loss is still falling steeply in all three while test accuracy has
stopped moving.** That is the registered rule's OVERFITTING branch, whose text
says exactly what it means here:

> *"OVERFITTING. Training loss keeps falling while test accuracy does not. The
> budget is sufficient; the still-falling training loss is not evidence against
> [the ceiling]."*

This resolves the §3.1 ambiguity in `HANDOFF_2026-08-02.md` — which said "both
are happening" — with a boundary: **undertrained through e400, overfitting by
e800.** The still-falling training loss that motivated the original withdrawal
was never evidence of undertraining once test accuracy stopped tracking it.

## 3. The restored figure

**0.737780 ± 0.000675** at h1024/e400 — the widest and best-converged point.
h512 gives 0.736896 ± 0.002087 at e400 and 0.737191 ± 0.003825 at e800. All
three agree within noise, as a converged quantity should.

**Shortfall to the registered `accuracy >= 0.80` gate: 0.0622.**

The previously quoted 0.7151 was an e100 figure and is superseded — it
understated the converged value by 0.021. Quote **0.7372** for this
configuration — or **0.7378** at h1024 — and note that this is now a *converged*
number on both axes rather than a budget-limited one, which is the substantive
change.

## 4. The width axis — CLOSED, measured at the converged budget

*This section originally reported width as unsaturated and named it the binding
scope limit. It has since been measured and that limit is gone.*

Nine further cells — h128 / h256 / h1024 at **e400**, the sufficient budget,
three seeds each — against the existing h512:

| hidden | mean acc | sd | gain vs previous doubling |
|---:|---:|---:|---:|
| 128 | 0.703180 | 0.002208 | — |
| 256 | 0.721731 | 0.004611 | +0.018551 |
| 512 | 0.736896 | 0.002087 | +0.015165 |
| 1024 | **0.737780** | 0.000675 | **+0.000883** |

**h512 → h1024 buys +0.000883** against the same 0.01 bound. **Width is
saturated at h512.**

**The two axes interact, and that corrects an earlier misreading.** At e100 the
width curve gave 0.6588 / 0.6751 / 0.6928 — *+0.017 per doubling and still
rising* — which is what motivated calling width the binding limit. At the
converged budget it flattens. Wider networks at a fixed short budget look better
largely because they reach a given loss in fewer epochs; train every width to
convergence and the advantage disappears. **The apparent width trend was
substantially a budget artefact**, which is only visible now that the budget
axis is closed.

## 5. What this unblocks and what it does not

- **`SHD_BPTT_CEILING_NEGATIVE_RESULT.md`**: the word "ceiling" is supportable
  again at 0.7378, with **no width qualifier** — §4 closed it. The remaining
  qualifier is contract/geometry. ~~The blocking caveat at its top can be
  replaced, not simply deleted.~~ **DONE 2026-08-04**: the layered
  withdraw/restore banner was replaced with a single current-position block that
  retains the withdrawal history, and the stale body was brought current — title,
  `claim_axis`, §1, §4 (see below), §5 gap decomposition, §7 threats, §8, and
  provenance.
- **H1's e100 caveat is now quantified rather than vague.** The temporal
  campaign ran at e100, which is measurably undertrained by 0.021 accuracy
  against convergence. H1's contrast is still internally valid — all conditions
  share the budget — but "does the order effect survive at a converged budget?"
  is now a sharp, answerable question rather than a hand-wave. Answering it
  means re-running the 24 cells at e400.
- **H2 is untouched.** rec+alif at h512 still produces no usable cell, and
  clipping cannot fix it
  (`MEASUREMENT_2026-08-03_GRADIENT_CLIPPING_DOES_NOT_FIX_H512.md`). What *has*
  changed is what a rec+alif number would be worth: with the ff+fixed comparison
  point now converged rather than budget-limited, a rec+alif result at the same
  budget would decide the architecture question on its own, which it could not
  before.
- **It withdraws the matrix result's rate-coding inference.** That inference
  rested on resolution invariance at a short budget; the direct test refuted it
  (H1 NOT SUPPORTED). `SHD_BPTT_CEILING_NEGATIVE_RESULT.md` §4.3 now carries the
  reconciliation — **within-bin coincidence detection** is invariant to bin
  resolution *and* order-sensitive, which the two results jointly require.

## 6. Caveats

1. **One contract, one geometry.** `published-2ms / adjacent-sum-5`. The
   resolution invariance in the campaign suggests the contract matters little;
   `channels-700` is untested at convergence.
2. **Geometry untested at convergence** — now the load-bearing caveat.
   `adjacent-sum-5` only; `channels-700` is unrun at convergence. Width and
   budget are both closed (§2, §4).
3. **Three seeds**, df=2. The e800 95% CI is [0.7277, 0.7467], wide.
4. **Overfitting was diagnosed, not measured against a validation split.** SHD
   holds out speakers, and no separate validation set was used; the diagnosis
   rests on train-loss-falling with test-accuracy-flat, which is the registered
   criterion but is weaker than an explicit early-stopping curve.
5. **Instrument, not BINN.**
6. **Not a SOTA comparison.** 0.7378 against 0.939.
