# Amendment — W2-1 registered an unsatisfiable threshold

**Registered:** 2026-08-19, **before any wave-2 cell existed.** At the time of
writing, 5 cells were complete and 25 claimed, **all of them wave 1**
(`w1__*`); zero `w2dim__*` or `w2lyr__*` cells had been claimed, let alone run.
That timing is the only thing that makes this amendment legitimate rather than a
threshold moved to fit its data, and it is checkable: the campaign's S3 bucket
records a claim marker per cell, and no `w2` marker predates this document.

**Amends:** `PREREG_2026-08-19_SHD_ATTENTION_CAMPAIGN.md` §2, hypothesis W2-1.
**Changes nothing else.** W2-2 and every wave-1, wave-3 and wave-4 hypothesis
carry forward untouched.

---

## The defect

As registered:

> **W2-1** — accuracy is monotone non-decreasing in `d_model` across **at least
> three of the four steps**

The sweep is `d_model ∈ {16, 32, 64, 128}`. Four values give **three**
transitions, not four. The threshold as written requires three successes out of
a set of four things that does not exist, and there is no reading of it that
both matches the sweep and preserves the intent.

A threshold that cannot be evaluated is worse than a loose one: at analysis time
the only options would have been to silently reinterpret it — which is
indistinguishable from choosing the interpretation the data favours — or to drop
W2-1 and lose the axis.

## The amendment

> **W2-1** — accuracy is monotone non-decreasing in `d_model` across **at least
> 2 of the 3 steps** (16→32, 32→64, 64→128).

## Why 2 of 3 and not 3 of 3

The registered intent was "the effect is not an artefact of one particular
width", which is a claim about trend, not about strict monotonicity. Requiring
all three steps would fail the hypothesis on a single seed-level inversion at
the top of the range — and saturation at the widest setting is the *expected*
behaviour if the effect is real and the read-out has enough capacity, so 3-of-3
would reject for the wrong reason.

2 of 3 is also the closest surviving reading of "a clear majority of steps",
which is what "three of four" was reaching for.

## What is reported regardless of the verdict

All three step deltas, with per-seed values, whichever way W2-1 falls. The
verdict is a summary of that table, not a replacement for it.
