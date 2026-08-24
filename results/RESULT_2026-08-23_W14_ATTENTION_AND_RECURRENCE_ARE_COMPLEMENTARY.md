# Wave 14 — attention does not substitute for recurrence; it compounds with it

**Registered:** `PREREG_2026-08-23_RECURRENT_MEASUREMENT.md` (`0f338e4`),
**08:23 UTC, before any cell existed.** The analyser was committed at `da13bd0`,
**08:52 UTC — 22 minutes after the first cell landed at 08:30.** See §8; the
earlier wording of this line said both preceded the data, and that was wrong.
**Ran:** 2026-08-23, 4 × `c7g.16xlarge` spot, pinned binary `22d97c51ab02`.
**Status:** complete — 35/36 new cells emitted, one diverged, nothing retried,
no threshold moved. Fleet torn down.

---

## 1. Completion, and the gate it had to clear

| arm | completed | voided | diverged |
|---|---:|---:|---:|
| `ff+fixed` | 12/12 | 0 | 0 |
| `ff+fixed+attn` | 12/12 | 0 | 0 |
| `rec+alif` *(reused, wave 13)* | 11/12 | 0 | 1 |
| `rec+alif+attn` | 11/12 | 0 | 1 |

The two recurrent arms lost **different** seeds — `rec+alif` lost 5170002,
`rec+alif+attn` lost 5170012 — so the intersection is **10 pairs**, exactly the
registered minimum. M-0 cleared by zero margin, and one further loss on either
arm would have made M-1 and M-2 NOT EVALUABLE.

## 2. Paired gains, over seeds where both arms completed

| substrate | pairs | rate read-out | + attention d32/L4 | gain | per-pair range |
|---|---:|---:|---:|---:|---|
| `rec+alif` | 10 | 0.5262 | 0.7874 | **+0.2612** | +0.1886 to +0.4329 |
| `ff+fixed` | 12 | 0.7088 | 0.8289 | **+0.1201** | +0.0914 to +0.1484 |

## 3. Verdicts

**M-1 — SUPPORTED.** Attention helps a recurrent, adaptive substrate: gain
**+0.2612** against a bar of +0.05, positive in **10 of 10** pairs. Not one pair
goes the other way, and the smallest is +0.1886.

**M-2 — SUPPORTED, in the third direction.** gain(`rec+alif`) **+0.2612** vs
gain(`ff+fixed`) **+0.1201**, difference **+0.1411** against a two-sided bar of
0.03 — nearly five times the bar, and **larger on the recurrent substrate**.

Substitution is refuted on this axis too, and more strongly than on adaptation:
attention does not stand in for recurrence, it compounds with it. §5 is the
alternative explanation that must be dealt with before that reading is taken at
face value.

**M-3 — NOT SUPPORTED.** `rec+alif` alone reaches **0.5200** against the 0.80
gate, with **0 of 11** completing seeds over it. Recurrence plus adaptation does
not approach the gate without the read-out — and at this scale it sits well
*below* `ff+fixed`'s 0.7088.

**M-4 — the scale confound is closed.** `ff+fixed` at scale 0.4 is **0.7088**
against the archived **0.7062** at scale 1.0, a difference of **+0.0026**.

This is the result that licenses M-2. The wave was designed to run every arm at
0.4 so substrate and scale could not be confounded, and the registered risk was
that 0.4 would quietly weaken the feed-forward baseline and turn M-2 into a
comparison between a healthy arm and a crippled one. It did not: the baseline is
indistinguishable from its own archive at the default scale. **M-2 is reported
as clean, not scale-limited.**

## 4. What this settles, with wave 12

| axis | substrate without | with | gain |
|---|---:|---:|---:|
| adaptation *(wave 12, scale 1.0)* | `ff+fixed` 0.7062 | `ff+alif` 0.7018 | attention +0.1258 → +0.1285 |
| recurrence *(this wave, scale 0.4)* | `ff+fixed` 0.7088 | `rec+alif` 0.5262 | attention +0.1201 → **+0.2612** |

Adding **adaptation** changes the substrate's own score by −0.0044 and the
attention gain by +0.0027 — nothing, on both counts. Adding **recurrence and
adaptation together** costs the substrate 0.18 outright and roughly doubles what
attention recovers.

So the read-out's advantage is not a stand-in for temporal state in either
form. The mechanism claim from wave 9 — that 96% of it is contingent on
temporal order — stands unqualified by substrate.

## 5. The alternative explanation, and why it is not dismissed

**A gain measured from a lower base is not the same quantity as a gain measured
from a higher one.** `rec+alif` starts at 0.5262 with 0.4738 of headroom to a
perfect score; `ff+fixed` starts at 0.7088 with 0.2912. More room to recover is
the obvious reason a recovery is larger.

Normalising by headroom — **post-hoc, not registered, and reported for exactly
this reason**:

| substrate | gain | headroom | gain / headroom |
|---|---:|---:|---:|
| `rec+alif` | +0.2612 | 0.4738 | **0.551** |
| `ff+fixed` | +0.1201 | 0.2912 | **0.412** |

The ordering survives, and the ratio falls from 2.2× to 1.34×. So the
complementarity reading is not an artefact of the floor, but most of its apparent
size is. **The honest statement is the smaller one**, and the registered M-2
verdict — which is about the raw difference, as preregistered — should be read
next to it rather than instead of it.

Two more things the raw numbers do not say on their own:

* **The recurrent substrate does not win.** `rec+alif+attn` reaches 0.7874;
  `ff+fixed+attn` reaches 0.8289 at the same scale. Attention closes most of the
  0.18 the recurrent substrate gives away and does not close all of it. No
  verdict is issued on that ordering — it was not registered, and a factorial
  invites naming a winner afterwards.
* **The arm is numerically extreme.** 20 stability notes; `rec+alif` peak
  gradient norms run to **4.9e32**, against 1.13e8 for the largest cell anywhere
  in the recorded campaign. These are measurements taken on an arm that is
  barely holding together, which is what wave 13 established and why the
  operating point exists at all.

## 6. Survivorship, stated rather than assumed

Every comparison is paired on seed and computed only over pairs where both arms
completed, which removes the "two differently filtered subsets" failure wave 11
recorded. It does not remove everything: the ten surviving recurrent pairs are
the trajectories that did **not** diverge, and divergence is not random. A
recurrent gain measured over survivors could differ from one measured over all
twelve seeds if the arm could complete them, and nothing here can say by how
much.

The feed-forward comparison has no such exposure — 12/12 on both arms.

## 7. Scope

* **One scale, one width, one contract, one budget**, and the scale is 0.4 while
  the anchor is 1.0. M-4 shows the feed-forward baseline is unmoved by that, but
  nothing here establishes the same for the attention arms.
* **Nothing about `rec+fixed`.** Wave 13 measured it and it does not complete.
* **Ten pairs at the registered minimum.** The recurrent verdicts rest on the
  smallest sample the prereg permits.
* Not calibration. No comparison to macOS-recorded numbers.

## 8. Provenance

36 new cells from the pinned binary `22d97c51ab02`, which
`RESULT_2026-08-22_SOURCE_REPRODUCES_THE_PINNED_BINARY.md` established the
current source reproduces behaviourally on aarch64/glibc. `rec+alif` reused from
wave 13 rather than re-run — identical spec, deterministic instrument — and each
reused cell content-checked against the configuration it must be.

Verdicts computed once, from a settled wave, by an analyser mutation-tested
against both of its own failure modes: pooling instead of pairing, and a gate
that reports instead of blocking.

> **Correction, 2026-08-23.** This document, the analyser's docstring and the
> commit that added it all said the analyser was *frozen before the first cell
> landed*. It was not. Measured from the artefacts rather than from memory:
>
> | | wave 12 | wave 13 | wave 14 |
> |---|---|---|---|
> | prereg committed | 18:03 | 01:04 | **08:23** |
> | analyser committed | 18:06 | 01:06 | **08:52** |
> | first cell uploaded | 18:12 | 01:41 | **08:30** |
>
> Waves 12 and 13 are as claimed. Wave 14's analyser was committed **22 minutes
> after** its first cell, with twelve `ff+fixed` control cells already in the
> bucket.
>
> What is unaffected: every threshold was fixed in the prereg at 08:23, before
> any cell, and `test_the_registered_bars_are_the_ones_in_the_prereg` pins that
> the analyser carries those and no others. M-0's gate, M-1's and M-2's bars and
> the pairing rule are all registered text, so no bar was chosen with knowledge
> of a result. What existed when the analyser was written was a **count** — 12
> of 36 done, all of them the control arm — and no accuracy from it was read.
>
> What is affected is the claim. "Frozen before the first cell" is a stronger
> statement than "thresholds registered before the first cell", and only the
> second one is true here. The distinction is the whole point of freezing, so it
> is corrected rather than explained away.

The wave-14 plan was not archived to the campaign directory at launch. It was
regenerated from `plan_cells.py` and verified **sha256-identical** to the plan
that was uploaded, before any analysis ran.
