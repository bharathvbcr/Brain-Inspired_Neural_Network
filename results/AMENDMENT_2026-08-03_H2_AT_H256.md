# Amendment: measure H2 at h256, where the instrument works

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
