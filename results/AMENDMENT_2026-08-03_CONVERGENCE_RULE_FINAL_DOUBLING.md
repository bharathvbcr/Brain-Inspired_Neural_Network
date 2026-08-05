# Amendment: ask the convergence question of the final doubling, not the ladder endpoints

**Registered:** 2026-08-03, **before** the e800 cells were run.
**Amends:** the budget rule at `scripts/probe.py:430`.
**Bears on:** `SHD_BPTT_CEILING_NEGATIVE_RESULT.md` (claim currently withdrawn),
and caveat §6.1 of `RESULT_2026-08-03_SHD_TEMPORAL_INFORMATION_H1.md`.

---

## 1. The defect in the registered rule

```python
first, last = rows[0], rows[-1]
gain = last[1] - first[1]
if gain > 0.01:
    print("UNDERTRAINED. ...")
```

It compares the **first and last rungs of whatever ladder is passed**. The
consequence is that **extending the ladder can never escape UNDERTRAINED**,
because the first rung stays fixed at e100 while the last keeps climbing. Adding
e800, e1600, e3200 makes `gain` monotonically *larger*. A rule intended to detect
convergence gets further from declaring it the more evidence you collect.

Measured, at `published-2ms / adjacent-sum-5 / h512 / s5170001`:

| ladder | rows[0] → rows[-1] | gain | registered verdict |
|---|---|---:|---|
| e100, e200 | 0.716431 → 0.728357 | +0.011926 | UNDERTRAINED |
| e100, e200, e400 | 0.716431 → 0.734541 | +0.018110 | UNDERTRAINED |
| e100, …, e800 | 0.716431 → ? | larger still | UNDERTRAINED, necessarily |

Meanwhile the **per-doubling** gains are decelerating clearly:

| doubling | gain |
|---|---:|
| e100 → e200 | **+0.011926** |
| e200 → e400 | **+0.006184** |

The second doubling already buys less than the 0.01 threshold. The rule cannot
see this because it never looks at a doubling.

## 2. What the rule was for

"Is 0.7151 a ceiling, or an artefact of stopping at e100?" The right question is
not *"did the whole ladder gain more than 0.01?"* — trivially yes, and
increasingly so. It is **"does the next doubling still buy anything?"** A budget
B is sufficient when going to 2B buys ≤ 0.01.

## 3. Amended rule

> **Convergence (blocking, post-ladder).** For budgets B and 2B measured at the
> same configuration across the same seeds, B is **SUFFICIENT** if
> `mean_accuracy(2B) − mean_accuracy(B) ≤ 0.01`. Otherwise B is
> **UNDERTRAINED** and the ladder must be extended. The verdict is reported for
> the specific B, never for "the model".

The 0.01 constant is **unchanged** from the registered rule. Only the pair being
compared changes. This is deliberate: moving a registered threshold post-hoc is
the thing the repo's culture forbids, so it is not being moved.

## 4. Pre-specified test

**Run e800 at seeds 5170001-3.** Compare against the existing e400 cells at the
same three seeds (`0.734541`, `0.737633`, `0.738516`; mean `0.736897`).

- If `mean(e800) − mean(e400) ≤ 0.01` → **e400 is SUFFICIENT**, and 0.7369
  becomes a defensible ceiling figure for this configuration.
- If `> 0.01` → e400 is undertrained too; the ladder extends to e1600 and the
  same test repeats.

**Stopping rule:** the ladder extends by doubling until the criterion is met or
until e1600, whichever comes first. If e1600 has not converged, that is reported
as "not converged within the budget explored" — **not** as a ceiling. No
threshold is adjusted to force a verdict.

## 5. What this does not do

- It does **not** restore the ceiling claim. It provides the test that could.
  The claim stays withdrawn until a SUFFICIENT verdict exists.
- It does **not** change H1's result. H1 compares conditions at a shared budget
  and is internally valid regardless. What a SUFFICIENT verdict at e400 would do
  is tell us whether the e100 caveat in §6.1 of that result matters — and that
  question needs the temporal campaign re-run at the sufficient budget to
  answer, which this amendment does not authorise.
- It does **not** touch `probe.py`. The python arm is out of scope by
  instruction; the amended rule is applied in analysis over cells produced by the
  rust binary, which is what has generated every cell today.
