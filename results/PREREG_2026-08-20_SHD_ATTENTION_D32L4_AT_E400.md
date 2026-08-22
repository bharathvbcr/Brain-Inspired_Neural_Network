# Preregistration — d32/L4 at e400 on the anchor, with convergence handled as a stability criterion

**Registered:** 2026-08-20, **before any e200 cell and before any d32/L4 cell above
e100 existed** — zero of each in the plan, in `results/`, and in `claims/`,
checked and recorded at registration time. The e100 rung for d32/L4 (0.8209,
n=12) and the `ff+fixed` control at e100/e400/e800 already exist from the
campaign and are **reused, not re-run**: same machine, same pinned binary
`22d97c51`, same seeds.
**Binary:** pinned to `22d97c51`, so these cells are directly comparable to the
campaign cells they extend. New instances download it and abort on hash mismatch.

---

## 1. Why this run exists, and what it is not

Wave 2 measured `ff+fixed+attn` at **d32/L4** reaching **0.8209** (n=12, e100,
h128, anchor), with **11 of 12 seeds at or above 0.80** and zero validity-gate
failures. The campaign's registered 0.80 accuracy gate — which this instrument
has never cleared, and against which converged `ff+fixed` sits at 0.7378 — is in
reach for the first time.

**But W1 tested `d32/L1` only.** That is the arm the campaign registered, and
wave 2 shows it is the *weakest* configuration in the sweep. So W1-1's NOT
SUPPORTED verdict applies to d32/L1 and says nothing about d32/L4. This run tests
the configuration that was never tested at the anchor budget.

**This run cannot calibrate the instrument, at any accuracy.** Calibration
criterion 5 requires a matched Python/Rust configuration and no Python mirror of
the attention axis exists (`scripts/shd_calibration/arms.py`). Criterion 4 is
about accuracy; criterion 5 is not, and no number here touches it.

## 2. Convergence, handled as *stability* and not as a signed gain

Three failures in this campaign shape the criterion below, and each is on record:

1. **W1-4's window scaled with the budget**
   (`DEFECT_2026-08-19_W1_4_THRESHOLD_IS_NOT_BUDGET_INVARIANT.md`). A statistic
   measured over "the final tenth of training" is not comparable across budgets.
2. **W5's signed final-doubling assumed monotonicity.** The attention arm does
   not have it: d32/L1 peaks at **e50 (0.7539)** and is *lower* at e400 (0.7483).
   A rule that asks "did it gain less than 0.01" scores a **decline** of 0.02 as
   converged, which is the opposite of true.
3. **Four thresholds were anchored to macOS values this campaign may not compare
   against** (`DEFECT_2026-08-20_THRESHOLDS_ANCHORED_TO_UNLICENSED_REFERENCES.md`).

So the criterion here is **absolute**, **same-machine**, and **budget-invariant**:

> **R-2.** The arm is *budget-stable at e400* if
> `|acc(e400) − acc(e200)| < 0.01`.

Absolute value, because a decline is instability too. Adjacent registered rungs,
because a doubling is budget-invariant where a fixed-fraction window is not. Both
terms measured in this run, because a historical anchor is not licensed.

This is a claim about **stability of the reported number**, not about optimisation
having converged. Those are different, and only the first is needed to report an
accuracy honestly.

## 3. Registered schedule

| axis | value |
|---|---|
| arm | `ff+fixed+attn`, **d_model 32, layers 4** |
| control | `ff+fixed` at matching budgets |
| contract / geometry | `published-2ms` / `adjacent-sum-5` (the anchor) |
| hidden | 128 |
| budgets | **e100** (exists), **e200** (new), **e400** (new) |
| seeds | **exactly 12**, 5170001–5170012, shared with the campaign |

36 new cells: d32/L4 at e200 and e400 (24), `ff+fixed` at e200 (12).

**The headline budget is e400, fixed here, before any e200 or e400 cell for this
arm exists.** If e100 or e200 scores higher, that is reported as part of the
ladder and **does not become the claim.** Substituting the best-looking rung after
the fact is the selection this clause exists to prevent, and the arm is already
known to peak early — so the temptation is real and specific.

## 4. Hypotheses

| ID | statement | threshold |
|---|---|---|
| **R-1** (primary) | d32/L4 clears the registered gate at the anchor budget | mean accuracy at e400 **≥ 0.80** *and* **≥ 9 of 12** seeds individually ≥ 0.80 |
| **R-2** (stability) | The e400 number is not a budget artefact | **\|acc(e400) − acc(e200)\| < 0.01** |
| **R-3** (contrast) | The gap W1 could not find at d32/L1 | mean(attn) − mean(`ff+fixed`) at e400 **≥ 0.05** |
| **R-4** (validity) | Nothing degenerate is being read as a pass | every reported cell passes §5 gates; a seed failing any gate does not count toward R-1 |

**R-1 and R-2 must both hold** for the accuracy to be reportable as an
architecture result. R-1 alone with R-2 failing means the number depends on where
training stopped — which is exactly what happened to the pilot's +0.1702 and must
not happen twice.

**R-3 is the direct successor to W1-1.** W1-1 measured +0.0421 at d32/L1 against a
0.05 bound. The same bound is applied here, unchanged, so the two are comparable.

## 5. Validity gates

Per cell, unchanged from the campaign: `non_finite_events == 0`;
`classes_predicted == 20`; `majority_prediction < 0.30`; `silent_fraction ≤ 0.95`;
`saturated_fraction ≤ 0.05`. Plus the campaign-wide rule that the instance's
cross-machine Gate F verdict is recorded whether it passes or fails.

## 6. What must not be claimed

- **That the instrument is calibrated.** Criterion 5 is untouched and unmet.
- **Any comparison to the macOS 0.7378 or 0.7032.** The cross-machine gate FAILs
  on every instance.
- **Scope beyond h128 / `adjacent-sum-5`.** Wave 3 measured the gain inverting by
  h1024 (−0.0159) and weakening to +0.0243 at `channels-700`, without seed
  consistency. Whatever this run finds inherits that scope exactly.
- **That d32/L4 is optimal.** It is the best of the six configurations wave 2
  tested at e100. L8, d64/L4 and the rest are unmeasured.

## 7. Stopping rule

**Twelve seeds, three rungs, verdicts computed once, reported whichever way they
fall.** No thirteenth seed, no additional configuration, and **no e800 rung** —
if R-2 fails, the honest report is "not budget-stable between e200 and e400", not
a longer ladder appended until something passes. That is the failure mode
`AMENDMENT_2026-08-03_CONVERGENCE_RULE_FINAL_DOUBLING.md` exists to prevent and
that wave 5 walked into.
