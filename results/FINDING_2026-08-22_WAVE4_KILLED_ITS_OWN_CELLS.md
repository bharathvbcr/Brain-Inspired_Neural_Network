# Finding — wave 4 did not measure the recurrent arm, it killed it

**Found:** 2026-08-22, diagnosing why `rec+alif` diverged 24 of 24.

**Withdraws:** `RESULT_2026-08-20_W4_RECURRENT_ARM_IS_UNUSABLE.md`.

**Consequence:** `rec+alif` is not unusable. It completes at exactly the width and
budget wave 4 used, and thirteen such cells were already on disk when wave 4 ran.

---

## 1. The failure is a protocol parameter, not the arm

All 24 wave-4 logs carry one line:

```
shd-instrument: non-finite training value at optimizer step N
```

That is the per-sample guard at `binn-lab/experiments/shd_instrument.rs:777`,
firing on `!sample_gradient.all_finite()`. Abort steps span 53–830, so it is
trajectory-dependent rather than a fixed structural failure.

Reproduced locally through the campaign's own `run_cell.py` against the
campaign's own `plan.json`. Then a **paired control** — same binary, same seed,
byte-identical initial weights, same data order, same surrogate scale, differing
only in `--clip-grad-norm`:

| optimizer step | clipped, max\|g.w_in\| | unclipped, max\|g.w_in\| |
|---:|---:|---:|
| 0–24 | identical | identical |
| 40 | 3.88e2 | 6.20e-2 |
| 56 | 9.23e5 | 5.19e-2 |
| 96 | 7.62e18 | 2.65e0 |
| 244 | **overflow, 3586 non-finite** | 1.53e-1 (healthy) |

The unclipped run completed all 100 epochs. **`--clip-grad-norm 1.0` is what
diverged the cells.**

Two hypotheses were tested and falsified. It is not weight growth: `max|W_rec|`
is indistinguishable between the two runs (0.167 vs 0.167 at step ~110). It is
not a forward-pass blow-up: the loss at the aborting step is 2.72, and `w_out` /
`b_out` — the only gradients that do not pass through the BPTT recursion — stay
at ~1e-1 throughout. The first tensor to overflow is `gradient.base.w_in`, fed by
the `du` recursion at `binn-learn/src/shd_matched_arms.rs:882`.

**Why clipping is destructive here.** The 1.0 threshold was taken from the
healthy `ff+fixed` scale (epoch mean norm 0.20–0.29). The recurrent arm's own
epoch-mean batch gradient norm exceeds 1.0 in **100 of 100 epochs** (median 4.1e3
at surrogate scale 0.4). So clipping binds on essentially every step: it is not
outlier suppression, it is unconditional renormalisation to a constant norm.
Under Adam that removes the second-moment damping which is exactly what lets the
unclipped arm absorb its excursions and recover — the recorded pilot's epoch
peaks go 522 → 4.2e5 → 1.17e12 → 130 → 62, and it finishes.

## 2. Thirteen completing cells were already on disk

`results/shd_instrument_v4/` holds **13 `rec+alif` cells at exactly h256/e100**,
every one with `clip_grad_norm: null` and `non_finite_events: 0`:

| location | n | accuracy range |
|---|---:|---|
| `h2-campaign/` | 10 | 0.2893 – 0.4910 |
| `surrogate-pilot/` | 3 | 0.4386 – 0.5141 |

Against wave 4's 0 of 24. **13/15 unclipped complete versus 0/24 clipped**
(Fisher exact p = 1.3e-8); restricted to the intact condition, 5/6 versus 0/12
(p = 7e-4).

The two unclipped local reruns reproduced the recorded 2026-08-05 cells
**bit-identically** (seed 5170001 accuracy 0.514134276, peak norm 1.174e12), so
there is no binary drift and no macOS/Linux drift on this path.

## 3. How the design came to include the thing that breaks it

Two citation defects, in sequence.

**The misquote.** `RESULT_2026-08-20_W4_RECURRENT_ARM_IS_UNUSABLE.md:48` states:

> `TODO_2026-08-07_OPEN_WORK.md` §4 records `rec+alif` producing *zero usable
> cells* at h512, with activation peaks from 3.08e10 to 3.93e33.

`TODO_2026-08-07_OPEN_WORK.md:115` actually reads:

> **3/3 seeds complete with zero non-finite events**, but peaks span 3.08e10 to
> 3.93e33

"Zero non-finite events" became "zero usable cells" — the exact inversion of the
source. Those peaks belong to three **completing** cells.

**The inverted lever.** `scripts/aws/plan_cells.py:116` justifies the wave-4
design:

> The surrogate-scale ladder and clipping are the two levers the record says were
> needed to get any recurrent cell to complete at all

The record says the opposite, in a document whose title is
`MEASUREMENT_2026-08-03_GRADIENT_CLIPPING_DOES_NOT_FIX_H512.md` — *"Gradient
clipping does not rescue rec+alif at h512, and cannot."* Its §1 records clipping
taking seed 5170001 from completing to aborting, and its §2 shows the abort fires
**upstream of the clip site**, so no threshold could ever have helped.

So: a misreading turned a success into a failure; the misreading justified adding
clipping; the clipping diverged every cell; and the divergence was written up as
the arm being unusable — citing the same misreading.

## 4. What is withdrawn, and what is not

**Withdrawn:** `RESULT_2026-08-20_W4_RECURRENT_ARM_IS_UNUSABLE.md` in full. Its
title claim is false and its evidence measures a protocol parameter.

**Not withdrawn — the marginality is real.** With membrane decay α = 0.8195 and a
Glorot `w_rec` of spectral radius ≈ 1.00, the worst-case per-timestep backward
gain is 1.82 at surrogate scale 0.4, while f32 survives at most 1.274 over the
~366 timesteps of a 2 ms frame. Only the Lorentzian surrogate's fall-off keeps
typical gain below that, which is why the seed-to-seed spread is chaotic and why
*completing* cells still show peaks to 3.93e33. That belongs on the record as
numerical marginality. It is not what produced 0 of 24.

**Not established:** whether the attention read-out helps the recurrent arm. That
was wave 4's actual question and it remains unanswered, because no wave-4 cell
produced a number. Re-running it needs an amendment, since `clip_grad_norm = 1.0`
is a registered protocol parameter — see
`AMENDMENT_2026-08-22_WAVE4_WITHOUT_CLIPPING.md`.

## 5. The class of defect

This is the third time this week a check or a design has been built on a claim
nobody re-read at the source. The instance is a misquote; the class is that
`plan_cells.py` docstrings cite the record in prose that nothing verifies.

`scripts/verify_published_numbers.py` recomputes published numbers from cells. No
equivalent checks that a *design rationale* still matches the document it cites,
and one existed here that inverted its source. That is named, not fixed.

## 6. Caveat, unverified

The local clipped run aborted at step 244; the AWS wave-4 cell at 151. Since the
unclipped runs bit-match the Linux record exactly, this is not general binary or
platform drift. It may be the different wave-4 binary (`22d97c51`) on the clip
path specifically, or a platform difference confined to that path. Undetermined.
It does not bear on the verdict: the clipped/unclipped contrast is same-machine,
same-binary, same-seed.
