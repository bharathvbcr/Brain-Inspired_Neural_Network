# Amendment: measure H2 at h256, where the instrument works

> ## OUTCOME: H2 is STILL NOT RUN. h256 is not clean at the campaign budget.
> ## The §5 stopping rule fired. See §7.

**Registered:** 2026-08-03, before any H2 cell was run.
**Amends:** `PREREG_2026-08-02_SHD_TEMPORAL_INFORMATION.md` §3 — width for the
`rec+alif` arm only.
**Result extended:** `RESULT_2026-08-03_SHD_TEMPORAL_INFORMATION_H1.md`, where
H2 is recorded as NOT RUN.

---

## 1. Why

> **H2:** `rec+alif` degradation under `bin-shuffled` exceeds `ff+fixed`'s by
> **≥ 0.05** absolute, with disjoint CIs.

H2 is registered at **h512**, and h512 is the one width where the recurrent arm
does not run. Measured 2026-08-03, `rec+alif` / h512 / e20, three seeds: two
abort mid-training on non-finite per-sample gradients (steps 220, 50) and the
third reaches a gradient norm of 7.36e29. **Zero of three seeds produce a usable
cell.**

Three interventions failed to change this — rescaling `W_rec`, the f64 `l2_norm`
fix, and batch gradient clipping, the last of which *cannot* work because the
abort fires upstream of where it acts
(`MEASUREMENT_2026-08-03_GRADIENT_CLIPPING_DOES_NOT_FIX_H512.md`).

**h256 is clean.** Same arm, same budget, same contract and geometry:
`non_finite_events` = 0/640, no aborts, accuracy 0.3613, loss falling
monotonically. h128 is clean too. The failure appears between h256 and h512 and
tracks the `O(hidden²)` recurrent fan-in.

## 2. What changes

**Width for the H2 comparison moves from h512 to h256.** Everything else is
unchanged: `published-2ms`, `adjacent-sum-5`, e100, seeds 5170001-3, the four
conditions, the manipulation seeding, and **the registered ≥ 0.05 threshold**.

The threshold is deliberately not touched. Moving a registered effect-size bound
after the fact is the thing this repo's culture forbids, and nothing about the
width change justifies it.

## 3. Both arms are re-run at h256

H2 is a **comparison between arms**. Comparing `rec+alif` at h256 against
`ff+fixed` at h512 would confound arm with width, so **`ff+fixed` is re-run at
h256 as well** — 12 cells per arm, 24 total.

This means H2 is evaluated against a fresh `ff+fixed` baseline at h256, not
against the H1 result at h512. The two are reported separately and must not be
mixed.

## 4. What this costs the claim

**H2 at h256 is a weaker claim than H2 at h512 would have been**, and the
weakening must be stated wherever it is reported:

- h256 is not the width at which `ff+fixed` reaches its ceiling. The converged
  ceiling is 0.7378 and saturates at h512
  (`RESULT_2026-08-03_BUDGET_CONVERGENCE_CEILING_RESTORED.md`), so h256 sits
  below saturation on the width axis.
- A null H2 at h256 therefore **cannot** be read as "recurrence does not help at
  scale". It can only be read as "recurrence does not help at h256".
- A positive H2 at h256 is the stronger direction: if recurrence buys ≥ 0.05
  even below width saturation, that is informative.

## 5. Stopping rule

**Three seeds, one verdict, reported whichever way it falls. No seed extension
for H2** without a further amendment carrying its own pre-registered count —
the H1 seed extension is precedent for how that must be done
(`AMENDMENT_2026-08-03_H1_SEED_EXTENSION.md`).

If any `rec+alif` cell aborts at h256, H2 is reported as **NOT RUN** again
rather than evaluated on surviving cells. Partial-arm evaluation is exactly the
silent-partial failure the verdict tool was hardened against.

## 6. What this does not do

- It does **not** rescue `PREREG_2026-08-02_SHD_BPTT_CEILING_RECALIF`, which
  registers h512 for a *ceiling* claim. A ceiling measured below width
  saturation is not a ceiling. That prereg needs its own decision.
- It does **not** change H1 or H3, both settled at h512/e100 over 6 seeds.


## 7. OUTCOME — the premise was wrong, and §5 caught it

**H2 remains NOT RUN.** Of 12 `rec+alif` cells at h256/e100, **two aborted** on
non-finite per-sample gradients — seed 5170002, `intact` at optimizer step 374
and `bin-shuffled` at step 727. `ff+fixed` completed 12/12.

Per §5 — *"If any `rec+alif` cell aborts at h256, H2 is reported as NOT RUN
rather than evaluated on surviving cells"* — the 10 surviving cells were **not**
used to compute the H2 statistic. The verdict script refused before printing one.
Evaluating an arm on the subset of seeds that happened to survive is exactly the
selection effect that would make the number meaningless, and it is the
silent-partial failure the tooling was hardened against earlier the same day.

### Why §1's premise was wrong

§1 asserted "h256 is clean", on the evidence of a width check at **e20**:
`non_finite_events` 0/640, no aborts, accuracy 0.3613.

| budget | optimizer steps | result |
|---|---:|---|
| e20 (the evidence for §1) | 640 | clean |
| e100 (the campaign budget) | 3200 | **2 of 12 abort, at steps 374 and 727** |

**The e20 probe covered one fifth of the trajectory the campaign actually runs.**
A "clean" verdict from it was never evidence about e100, and I treated it as
though it were.

This is the **third** time in one day that a cheap short-budget probe gave a
verdict that did not survive the real budget:

1. The e3 scale pilot sampled only the early-training transient, making
   `rec+fixed` look like it could not learn at all — it reaches 0.2633 at e20.
2. The registered convergence rule looked undertrained at e400 and was
   overfitting by e800.
3. This: h256 clean at e20, aborting at e100.

The pattern is worth naming, because it is cheaper to notice than to rediscover:
**a short-budget probe answers a question about short budgets.** When the
quantity of interest is a property of the whole trajectory — stability,
convergence, a ceiling — a probe that stops early does not approximate it, it
measures something else.

### What would actually be needed for H2

Narrowing the width was a guess that h512's failure was purely about fan-in. It
is not: h256 fails too, just later and less often. The remaining candidates from
`MEASUREMENT_2026-08-03_GRADIENT_CLIPPING_DOES_NOT_FIX_H512.md` §4 — truncated
BPTT, lower surrogate gain, spectral-radius-normalised initialisation — attack
the compounding in the per-sample backward, which is where the failure actually
lives. **h128 is the only width with no observed failure**, and it has not been
tested at e100 either; on this evidence it should not be assumed clean until it
is.

### What was produced

The 12 `ff+fixed` cells at h256/e100 are valid and complete. They are **not**
reported as a result here: they were run as H2's matched baseline, and with H2
unrun they answer no registered question. They remain on disk under
`results/shd_instrument_v4/h2-campaign/` for whoever resumes this.
