# Amendment: extend H1 from 3 seeds to 6

**Registered:** 2026-08-03, **after** the 3-seed result was read, **before** any
additional cell was run.
**Amends:** `PREREG_2026-08-02_SHD_TEMPORAL_INFORMATION.md` §3 (seed count only).
**Result being extended:** `RESULT_2026-08-03_SHD_TEMPORAL_INFORMATION_H1.md`.

---

## 1. Why

H1 passed its registered equivalence bound by **0.000124 — 0.62% of the bound**
— with 1 of 3 seeds exceeding it individually and the 95% CIs overlapping by
0.0006. A different seed triple could plausibly return NOT SUPPORTED. A boundary
pass on n = 3 is not something to build a claim on.

## 2. What changes

Seeds **5170004, 5170005, 5170006** are added, giving 6 seeds × 4 conditions =
24 `ff+fixed` cells. Nothing else changes: same contract, geometry, width,
budget, arm, manipulation seeding, and the same thresholds in §4 of the prereg.

Initialisations are generated exactly as the originals were —
`init --n-inputs 140 --hidden 512 --classes 20 --seed N --epochs 100 --n-train 8156`
— and written under the campaign directory. The registered
`initialization/` tree is not touched.

## 3. Stopping rule — binding, and the point of this document

**Exactly three seeds are added. The verdict is recomputed once, on all six, and
reported whichever way it falls. No further seeds will be added to H1 after
this, regardless of outcome.**

This is stated in advance because the failure mode is obvious and severe:
extending a boundary result until it lands on the preferred side is optional
stopping, and it would make the pre-registration worthless. Committing to n = 6
before seeing seed 4 is what keeps the test honest.

If the 6-seed verdict is NOT SUPPORTED, that supersedes the 3-seed SUPPORTED
verdict. The earlier result is not withdrawn or hidden — both are reported, with
the 6-seed one binding.

## 4. Direction of the effect on difficulty

Adding seeds makes H1 **harder**, not easier, on one of its two criteria:

| criterion | n=3 | n=6 | effect |
|---|---|---|---|
| CI overlap | df=2, t=4.303 | df=5, t=**2.571** | CIs **narrow** → overlap less likely → harder |
| mean difference vs 0.02 | — | — | can move either way |

So this extension cannot make H1 pass "by accident" on the CI criterion; it can
only tighten it. That asymmetry is a reason to trust a 6-seed pass more than a
3-seed one, and it is why this is worth the compute.

## 5. What this does not touch

- **H2** remains NOT RUN. `rec+alif` at h512 produces zero usable cells; that is
  unchanged by seeds.
- **H3**'s registered conditional status is unchanged. It stays non-confirmatory
  if H1 passes, per the original plan and §4.2 of the result.
- **The e100 budget caveat is unaddressed by this amendment.** More seeds say
  nothing about whether timing becomes usable with more training. That is a
  separate question and needs a separate registered extension.
