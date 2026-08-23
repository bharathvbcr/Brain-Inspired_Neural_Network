# Wave 13 — there is an operating point, and adaptation is what makes it one

**Registered:** `PREREG_2026-08-23_RECURRENT_STABILITY.md` (`1942fd7`); analyser
frozen at `7d46d05`, both before the first cell existed.
**Ran:** 2026-08-23, 3 × `c7g.16xlarge` spot, pinned binary `22d97c51ab02`.
**Status:** complete — **48/48 cells settled**, nothing retried, no threshold
moved.

---

## 1. Completion, which is the measurement

| arm | surrogate scale | completed | voided | diverged |
|---|---:|---:|---:|---:|
| `rec+alif` | **0.4** | **11/12** | 0 | 1 |
| `rec+alif` | 1.0 | 8/12 | 0 | 4 |
| `rec+fixed` | 0.4 | 7/12 | 5 | 0 |
| `rec+fixed` | 1.0 | 5/12 | 5 | 2 |

## 2. Verdicts

**R-1 — SUPPORTED, by the smallest possible margin.** `rec+alif` at scale 0.4
completes **11 of 12** against a bar of 11 of 12. One further divergence would
have made it NOT SUPPORTED. That is a pass, and it is not a comfortable one; §5
says what follows from that.

**R-2 — SUPPORTED, with the sign opposite to the hypothesis's own name.**
`rec+alif` 19/24 against `rec+fixed` 12/24, a difference of **+7** on a
two-sided bar of 6. The hypothesis was called *"adaptation is what
destabilises"*. **Adaptation stabilises.** Removing it makes the recurrent arm
markedly less usable, not more.

**R-3 — NOT SUPPORTED.** Surrogate scale 0.4 completes 18/24 against 1.0's
13/24, a difference of **+5** against a bar of 6. Directionally the finer scale
helps — and every one of the 0.4 conditions beats its 1.0 counterpart — but not
by the registered margin, so no claim is made. `AMENDMENT_2026-08-05` argued the
per-timestep backward gain is the axis at h512; at h128 that is unconfirmed
rather than contradicted.

## 3. The two arms fail in different ways, and that explains R-2

Not one of the ten voided cells is `rec+alif`, and not one of them was voided for
anything but saturation:

| arm | how it fails | count |
|---|---|---:|
| `rec+fixed` | `saturated_fraction` above the 0.05 gate — 0.055 to **0.523** | 10 |
| `rec+alif` | divergence: the per-sample guard fires | 5 |

`rec+fixed` does not blow up. It **jams**: by the worst cell, more than half the
hidden units sit pinned at maximum firing, and the validity gate voids it. Its
`ss0.4` condition diverged **zero** times and still only reached 7/12, entirely
through saturation.

Threshold adaptation is precisely the mechanism against that — the firing
threshold rises with recent activity, so a unit that saturates raises its own
bar. The wave was designed on the premise that adaptation was the numerical
liability. It is the opposite: adaptation is what keeps the recurrent arm off
the saturation failure mode, and it costs some divergence in exchange.

**A preregistration miss, stated rather than smoothed.** §4 of the prereg named
outcomes for "R-2 favours `rec+fixed`" and for "R-2 flat" and did **not** name
the third possibility, which is the one that happened. The threshold and the
two-sidedness were registered correctly, so the verdict is well defined; the
interpretation table was incomplete, and the reading in this section was written
after seeing the sign. It is offered as a mechanism that fits, not as a
registered result.

## 4. Diagnostics

| arm | scale | peak ‖g‖ of completing cells | abort steps of diverged cells |
|---|---:|---|---|
| `rec+alif` | 0.4 | 1.30e+09 – 4.95e+32 | 8056 |
| `rec+alif` | 1.0 | 7.19e+07 – 1.02e+37 | 367, 3488, 3864, 7895 |
| `rec+fixed` | 0.4 | 2.24e+10 – 1.58e+23 | — |
| `rec+fixed` | 1.0 | 2.48e+14 – 3.80e+35 | 63, 428 |

Every completing cell is far above the 1e9 stability tier — 30 stability notes,
registered in advance as expected and non-voiding, because a recurrent arm up
there is the phenomenon under study rather than a defect. The single `ss0.4`
divergence aborts at step **8056** of 12,800, against wave 11's range of
438–1035 at h256/e100: at this width and scale the arm survives most of the
budget before it fails, where before it failed near the start.

## 5. What follows, and what it costs

**The operating point is `rec+alif`, h128, `published-2ms`, `adjacent-sum-5`,
e400, surrogate scale 0.4.**

Two constraints on any measurement wave run there, both of which narrow what it
can claim:

* **Scale 0.4 is a deviation from the registered default**, and every anchor
  control — the 12 `ff+fixed` and 12 `ff+fixed+attn` cells wave 12 reused — ran
  at 1.0. So a measurement there supports the **within-substrate** comparison
  the substitution question needs, `gain(rec+alif) = mean(rec+alif+attn) −
  mean(rec+alif)` with both arms at 0.4. It does **not** support comparing that
  gain to `gain(ff+fixed)` without conceding that scale is confounded with
  substrate. Making that comparison clean would need `ff+fixed ± attn` re-run at
  0.4, which is 24 more cells.
* **11/12 is the completion rate of the arm without attention.** Wave 11 saw
  `rec+alif+attn` diverge as well, and nothing here measures the attention arm's
  completion. A measurement wave has to carry both arms at 12 seeds and can be
  voided by either.

**Accuracies of completing cells are not a measurement**, and are recorded here
with their completion counts for that reason:

| arm | scale | n | mean | min | max |
|---|---:|---:|---:|---:|---:|
| `rec+alif` | 0.4 | 11 | 0.5200 | 0.3913 | 0.6144 |
| `rec+alif` | 1.0 | 8 | 0.5288 | 0.4457 | 0.5910 |
| `rec+fixed` | 0.4 | 7 | 0.4972 | 0.3944 | 0.5994 |
| `rec+fixed` | 1.0 | 5 | 0.5448 | 0.4814 | 0.6179 |

The four means sit within 0.05 of each other while completion ranges from 5/12
to 11/12. An arm that diverges more often can look better because only its
luckier trajectories survive to be scored, which is wave 11's recorded lesson,
and no comparison between these rows is a result.

## 6. Scope

* h128, `published-2ms`, `adjacent-sum-5`, e400 only. Nothing about other widths
  or contracts.
* **Nothing about attention on a recurrent substrate.** That is the measurement
  wave and it is not this one.
* Nothing about `--clip-sample-grad-norm`, deliberately: introducing an untried
  parameter into the wave characterising the baseline would have confounded it.
  It remains the next lever if the measurement wave's completion fails.
* Not calibration. No comparison to macOS-recorded numbers.

## 7. Provenance

48 cells from the pinned binary `22d97c51ab02`, which
`RESULT_2026-08-22_SOURCE_REPRODUCES_THE_PINNED_BINARY.md` established is
behaviourally reproduced by the current source on aarch64/glibc.

Verdicts computed once, from a settled wave, by an analyser frozen before the
first cell landed and tested by mutation against both bugs that wave 11's frozen
analyser carried. One change was made to the analyser after the data landed: the
voided-cell list truncated ids before the part that distinguishes them, so every
line rendered identically. Display only — no verdict reads that list, and the
verdicts are byte-identical before and after.

Three spot instances, ~1.5 h, torn down after collection. Confirmed no instances
running.
