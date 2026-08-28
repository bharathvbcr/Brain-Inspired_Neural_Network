# Result — wave 20: the recurrent claim survives a tripled sample, and survivorship is not shaping it

**Registered:** [`PREREG_2026-08-27_THE_RECURRENT_CLAIM_AT_THIRTY_TWO_SEEDS.md`](PREREG_2026-08-27_THE_RECURRENT_CLAIM_AT_THIRTY_TWO_SEEDS.md),
before any cell of wave 20 existed and before any instance for it was launched.
**Analyser:** `scripts/aws/analyse_wave20.py`, frozen in the same commit as the
preregistration. Its output is
[`VERDICTS_W20.md`](shd_attention_campaign_v2/VERDICTS_W20.md) and every verdict
below is cross-checked against it by `scripts/check_verdicts_transcribed.py`.
**Binary:** the campaign's pinned `22d97c51ab02`, one binary across all twenty
waves.
**Status:** complete — **80/80 cells settled**, nothing retried, no threshold
moved, no arm extended.

---

## 1. Verdicts

| ID | statement | verdict |
|---|---|---|
| **H20-1** | The recurrent substrate's larger gain survives a tripled sample | **MET** |
| **H20-2** | The comparison is no longer one loss from unreportable | **MET** |
| **H20-3** | Survivorship is not shaping the gain | **MET** |
| **H20-4** | The advantage survives headroom normalisation | **MET** |

All four. The preregistration's own reading of that combination:

> §3.7 stands and limits 3 and 4 are retired. The recurrent result is the
> campaign's best-evidenced contrast.

## 2. Completion, which decides whether anything may be reported at all

| arm | valid of 32 |
|---|---:|
| `rec+alif` | 27 |
| `rec+alif+attn` | 29 |
| **usable pairs** | **24** |

Six cells diverged: `rec+alif` at seeds 5170013, 5170017, 5170020 and 5170029,
and `rec+alif+attn` at 5170018 and 5170022. **The two arms lost disjoint
seeds**, which is why 27 and 29 valid arms yield 24 pairs rather than 27.

**H20-2: MET** — 24 pairs against a floor of 24/32, and not one more. The
registered stopping rule made this gate on everything else: *"A comparison
below 24 pairs carries no numbers at all"*, and H20-1's verdict would have been
suppressed regardless of its arithmetic. For most of the wave the margin was
zero — at one point the whole comparison turned on a single cell still running.
It cleared exactly at the bar.

That is a pass and it is not a comfortable one. The honest reading is that this
operating point supports the claim at thirty-two seeds and would not have
supported it at thirty-one.

## 3. H20-1 — the gain survives

| substrate | pairs | rate read-out | + attention | gain |
|---|---:|---:|---:|---:|
| `rec+alif` | 24 | 0.5187 | 0.7944 | **+0.2757** |
| `ff+fixed` | 32 | 0.7086 | 0.8275 | **+0.1189** |

**H20-1: MET** — difference of gains **+0.1551** over 24 seed-paired
comparisons, positive in **24/24**, against registered bars of ≥ +0.03 and
≥ 24/32.

The pilot in [`PAPER_DRAFT.md`](PAPER_DRAFT.md) §3.7 was **+0.1391** over ten
pairs. At 24 pairs the difference is **larger**, not smaller, and it is positive
in every pair rather than in ten of ten. Twenty further seeds did not erode it.

## 4. H20-3 — the question the wave was really about

The preregistration registered a bar **its own pilot already failed**, so that
the failure could not later be presented as a surprise or the threshold as
post-hoc. Over the ten surviving pairs that existed when it was written,
Spearman ρ between the paired gain and log₁₀ of the cell's peak gradient norm
was **−0.648**: among cells that completed, those closest to diverging showed
the *smaller* gains. If that held, the gain was measured on a subsample selected
for the property that predicts it, and §3.7's limit 4 was a defect rather than a
caveat.

**H20-3: MET** — ρ = **−0.274** over 24 completing pairs, against a bar of
≥ −0.30.

The prediction was that −0.648 was small-sample noise. It was. The margin is
0.026 and the bar is cleared, not comfortably.

**No causal claim follows in either direction.** The preregistration is explicit:
a correlation between gain and gradient norm among survivors bounds how much the
selection could be doing; it does not establish that it is doing it, and its
absence does not establish that nothing is.

## 5. H20-4 — the post-hoc number is now registered

| substrate | base | headroom | gain | gain / headroom |
|---|---:|---:|---:|---:|
| `rec+alif` | 0.5187 | 0.4813 | +0.2757 | 0.5728 |
| `ff+fixed` | 0.7086 | 0.2914 | +0.1189 | 0.4080 |

**H20-4: MET** — ratio **1.404x** against a registered bar of > 1.0x.

§3.7's limit 1 computed this as 1.34x and labelled it *"post-hoc, not
registered"*. The bar was set at 1.0 rather than at 1.34 precisely so the
pilot's own value was not the thing being fitted. **1.404x is the registered
measurement and supersedes the post-hoc figure.**

## 6. What the wave also measured, and does not explain

Twenty-nine cells carry **peak gradient norms above anything in the recorded
campaign** (previous maximum 1.13e8), ranging to 4.584e+35 — within five orders
of f32 overflow, the numerically marginal regime of
[`AMENDMENT_2026-08-05_SURROGATE_GAIN_FOR_RECURRENT.md`](AMENDMENT_2026-08-05_SURROGATE_GAIN_FOR_RECURRENT.md),
whose own §6 records a seed peaking at 3.93e33 and calls the arm marginal on
that basis.

**Gradient magnitude voided no cell**, by registration: it is the covariate
H20-3 is measured on, and voiding on it would decide the survivorship question
by construction. This is a measurement about the arm's numerical regime and it
is why its completion rate is what it is. Nothing here explains it, and
[`MEASUREMENT_2026-08-27_THE_RECURRENT_FOLLOW_UP_IS_NOT_WARRANTED.md`](MEASUREMENT_2026-08-27_THE_RECURRENT_FOLLOW_UP_IS_NOT_WARRANTED.md)
records why no further lever is registered: clipping was tested in wave 11 and
*causes* divergence, surrogate scale was tested in wave 13 and 0.4 is already
the stabilising setting, and adaptation was tested in the same wave and
stabilises rather than destabilises.

## 7. What may not be claimed

Carried unchanged from the preregistration's §7.

- **Nothing against macOS-recorded numbers.** Cross-machine Gate F FAILs
  macOS-vs-Linux on every node of this campaign, by design. Every contrast above
  is against a control that ran beside its treatment on the same machine.
- **No claim that the recurrent substrate wins.** §3.7's limit 2 stands
  untouched: `rec+alif+attn` reaches 0.7944 against `ff+fixed+attn`'s 0.8275.
  The recurrent substrate gains more from the read-out and still scores lower.
- **No claim about surrogate scale 1.0.** Every arm here is at 0.4.
- **No causal claim from H20-3**, in either direction.

## 8. Provenance

80 planned, 74 complete, 6 diverged, 0 voided. One pinned binary. Every
instance's Gate F verdict is FAIL cross-machine, recorded in `gates/` and
reported by `scripts/aws/watch_campaign.py` as instances joined. The 74 settled
cells are landed in `results/shd_attention_campaign_v2` and the archive's
baseline digest is re-frozen in `scripts/test_campaign_tooling.py`, with the
count moving 619 → 693 and the valid count 609 → 683 — both +74, so the ten
invalid cells are the same ten `w13rec` cells as before and no archived verdict
moved.
