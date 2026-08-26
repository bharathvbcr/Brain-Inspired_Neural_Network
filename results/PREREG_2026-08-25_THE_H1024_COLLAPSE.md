# Preregistration — the h1024 collapse, the width ladder, and the headline at n=32

**Registered:** 2026-08-25, **before any AWS instance for waves 15–17 exists and
before any cell of them has run.** Authorised by the maintainer, who asked for
additional hypotheses and larger samples.

**Protocol:** `shd-attention-campaign-v2`, waves `w15col`, `w16lad`, `w17hdl`.
**Binary:** the campaign's existing pinned binary
(`22d97c51ab0204702ce44661683ff8c759c29d7f3379e2f6606b048f4f032104`). One binary
across all seventeen waves; reused controls stay bit-comparable.
**Analyser:** `scripts/aws/analyse_wave15.py`, frozen in the same commit as this
document and before the first cell lands.

---

## 1. The observation this campaign exists to explain

The paper carries two load-bearing scope limits. One of them is *"the gain
inverts at width h1024 (−0.1618 at L4)"*
([`PAPER_DRAFT.md`](PAPER_DRAFT.md) §3.5, from
[`RESULT_2026-08-21_W8_HEADLINE_SCOPE_IS_MEASURED.md`](RESULT_2026-08-21_W8_HEADLINE_SCOPE_IS_MEASURED.md)).

Beside every one of those cells the archive also carries a gradient norm, and
until 2026-08-25 nobody read it. Derived this session from the committed cells:

| arm | epoch-mean norm, median | max norm | accuracy | sd |
|---|---:|---:|---:|---:|
| h128 d32/**L1** | 0.023 | 8.1e1 | 0.7483 | 0.0163 |
| h512 d32/**L1** | 0.023 | 3.4e2 | 0.7400 | 0.0276 |
| **h1024 d32/L1** | **0.025** | 3.3e2 | **0.7227** | **0.0094** |
| h512 d32/**L4** | 0.460 | 1.6e1 | 0.8233 | 0.0192 |
| **h1024 d32/L4** | **55.494** | **1.1e8** | **0.5768** | **0.0866** |
| h1024 d64/L4 (Azure) | — | 1.9e29 | 0.1278 | 0.1255 |
| h1024 rate-only | ~1.0 | 1.2e0 | 0.7386 | — |

Three things follow, and each is verified rather than argued:

1. **Accuracy tracks gradient magnitude almost perfectly.** Over the 24 h1024
   attention cells across both dimensions, Spearman ρ between accuracy and
   log₁₀ max gradient norm is **−0.970**. Within d32/L4 alone it is −0.867;
   within d64/L4, −0.895. Within the healthy h512/L4 arm it is −0.217, which is
   what no relationship looks like.
2. **It is not width.** `h1024` at **L1** is completely healthy — norm 0.025,
   accuracy sd 0.0094, the *tightest* of any arm in the table. The pathology is
   depth **and** width together.
3. **It is not the substrate.** The rate-only control at h1024, same seeds, same
   binary, has norms of order 1 and does not collapse. It appears only in the
   arm with the read-out attached.

And every collapsing cell **passes the preregistered validity gate**: 20 classes
predicted, majority prediction near 0.10, zero non-finite events. They are not
degenerate readouts. They are numerically sick, a condition
`scripts/cell_validity.py` reports and deliberately never voids — its own header
records `1.13e8` at h1024 as the largest norm in the corpus, without connecting
it to that arm's collapse.

**So the paper's scope limit may be measuring an optimisation failure and
calling it a property of the read-out.** That is the question.

## 2. What is at stake, stated plainly

If a lever restores the gain at h1024/L4, then *"the gain inverts at h1024"* is
false as a statement about the read-out and must be re-scoped to *"the deep
read-out is not trainable at this width under this optimiser"* — a different and
much narrower claim. If no lever restores it, the scope limit stands and is
strengthened, because the obvious alternative explanation will have been tested
and rejected.

Both directions change the paper. Neither is the outcome I am hoping for, and
the criteria below are fixed now so that it does not matter which arrives.

## 3. Wave 15 — the levers, and why these thresholds

**`--surrogate-scale` is the primary lever.** It reduces gradient magnitude at
source, and wave 13 established it as the lever that stabilises `rec+alif`
(11/12 completions at 0.4 against 8/12 at 1.0). Run at **0.5** and **0.25**.

**`--clip-grad-norm` is the secondary lever, and its threshold must be
justified**, because clipping is what destroyed wave 4
([`FINDING_2026-08-22_WAVE4_KILLED_ITS_OWN_CELLS.md`](FINDING_2026-08-22_WAVE4_KILLED_ITS_OWN_CELLS.md)).
The mechanism there was that a 1.0 threshold sat *below* the arm's typical norm,
so it bound on essentially every step — not outlier suppression but
unconditional renormalisation, which removes the Adam second-moment damping that
lets the arm absorb its own excursions and recover.

The threshold is therefore taken from **this arm's own** pooled epoch-mean
distribution, not from a healthy arm's:

| p50 | p75 | p90 | p95 | p99 |
|---:|---:|---:|---:|---:|
| 42.3 | 186.2 | 1298.9 | 3068.1 | 19798.3 |

**1000.0** binds on roughly a tenth of epochs, leaving the typical step
untouched. For contrast: a threshold of **20.0** would bind on about 65% of
epochs and is precisely the wave-4 regime. It is named here so the choice is
visibly a choice, and it is **not run**.

## 4. Wave 15 hypotheses

| ID | statement | criterion |
|---|---|---|
| **H15-1** | The collapse is an optimisation failure, not a capacity limit | at least one lever gives a paired gain over `ff+fixed` (unclipped, archived) of **≥ +0.05, positive in ≥ 9/12 seeds** at h1024/d32/L4 |
| **H15-2** | Recovery, if it happens, is numerical | in any arm meeting H15-1, the **median epoch-mean gradient norm falls below 1.0** — the scale of every healthy arm in §1 |
| **H15-3** | Depth is the axis | at h1024, the paired gain at **L2 lies between** L1's **−0.0159** and L4's **−0.1618**, exclusive of both by at least 0.01 |
| **H15-4** | Clipping is inert where it cannot bind | the twelve h512/d32/L4 cells at `clip=1000.0` are **byte-identical** to the archived `w8wid` cells on every scientific field |

H15-1 and H15-3 are independent. One passing does not rescue the other.

**H15-2 exists to stop a bad inference.** If accuracy recovers while the norms
stay at 55, the recovery is not the mechanism this campaign proposed and must be
reported as unexplained rather than as confirmation. Registering the mechanism
separately from the effect is what makes that distinguishable.

**H15-4 is a check on the instrument, not on the science.** Wave 4 showed the
clipping flag can be destructive when it binds; nothing has shown it is inert
when it does not. If those twelve cells differ from the archive at all, the flag
perturbs runs it should not touch, and **every clipped cell in this wave is
void** — including the ones that would have supported H15-1.

## 5. Wave 16 — the ladder, with no rung resting on four cells

The d32/L4 width ladder currently reads +0.1258 (h128, n=12), +0.0962 (h256,
**n=4**), +0.0876 (h512, n=12), −0.1618 (h1024, n=12). The h256 rung is four
cells recovered from the truncated Azure campaign — the only ones of their kind
anywhere — and between h512 and h1024 there is nothing.

h256, h384 and h768 are run at the full seed count, both arms. h256 is
regenerated rather than reused so that every rung is measured the same way; the
four Azure cells then become a cross-ISA check on the new ones rather than the
evidence itself.

| ID | statement | criterion |
|---|---|---|
| **H16-1** | The gain decays monotonically with width up to the collapse | gain(h128) > gain(h256) > gain(h384) > gain(h512) > gain(h768), each pair separated by at least 0.005 |
| **H16-2** | The collapse is a threshold, not the continuation of the slope | the drop from the last positive rung to h1024 exceeds **3×** the largest gap between adjacent rungs below it |

**H16-1 can fail without H16-2 failing**, and the reverse. A ladder that is not
monotone is a finding about the width axis; a collapse that is merely the slope
continuing is a finding about h1024.

## 6. Wave 17 — the headline and its mechanism at n=32

Every headline number rests on twelve seeds: 0.8320, +0.1258, 12/12 above 0.80,
and the bin-shuffle contrast that carries the whole mechanism claim.

Twelve was the terminal count for those registrations, and **this is not an
extension of them.** All four numbers already clear their bars comfortably;
nothing marginal is being rescued. It is a new registration at a larger n,
because the mechanism claim is the paper's strongest result and twelve seeds is
a thin base for it. Seeds 5170013–5170032 continue the same arithmetic sequence,
so no seed was chosen. The archived twelve are reused, giving **n = 32**.

| ID | statement | criterion |
|---|---|---|
| **H17-1** | The headline holds at n=32 | paired gain **≥ +0.05**, positive in **≥ 24/32**, and **≥ 24/32** seeds at or above 0.80 |
| **H17-2** | The mechanism holds at n=32 | (intact − bin-shuffled) for the attention arm **≥ +0.05**, positive in **≥ 24/32**, and at least **5×** the same quantity for the rate arm |

The 24/32 bars are the 9/12 proportion rounded to the nearer integer (24.0), so
the standard is neither raised nor lowered by the larger n.

## 7. Validity, unchanged

The preregistered per-cell gates are those of
`scripts/cell_validity.py` and are not modified for this campaign: zero
non-finite events, all 20 classes predicted, majority prediction below 0.30,
silent fraction at most 0.95, saturated fraction at most 0.05, and the cell must
have run the temporal condition its spec names.

**Gradient magnitude still does not void a cell.** It is the quantity under
study here, and voiding on it would decide the question by definition. It is
reported at both tiers, per cell, and H15-2 tests it explicitly.

Missing, timed-out, or invalid cells are reported as such and never silently
dropped. A wave that returns fewer than 9 valid seeds in any arm is reported as
**NOT EVALUABLE** for the hypotheses that arm serves, and no verdict is issued
from the survivors.

## 8. Named outcomes, every direction

- **O-1. A lever recovers the gain and the norms fall.** H15-1 and H15-2 both
  met. The paper's h1024 scope limit is re-scoped to trainability, and §3.5 is
  rewritten. The strongest single result of this campaign, and the one that
  costs the current draft the most.
- **O-2. No lever recovers the gain.** H15-1 not met. The scope limit stands and
  is stronger for having survived the test. The gradient correlation is then
  reported as a **symptom** of the collapse and explicitly not as its cause.
- **O-3. A lever recovers the gain and the norms do not fall.** H15-1 met, H15-2
  not. Reported as recovery by an unidentified mechanism. No causal claim, and
  the correlation in §1 does **not** get to be the explanation.
- **O-4. The levers move the h512 no-op control.** H15-4 fails; every clipped
  cell is void, and the wave is re-run without the clipping arm under its own
  amendment.
- **O-5. A lever makes h1024 worse.** Reported. Registered because wave 4's
  lever did exactly this and the outcome set that missed it is why this one
  enumerates both signs.
- **O-6. The n=32 headline disagrees with the n=12 headline.** Whichever way it
  falls, **n=32 is the reported number** and the n=12 result is superseded, not
  averaged with it. Pooling a small sample with the larger one that supersedes
  it is how a result gets to keep the half of itself that was luckier.

## 9. Stopping rule

The first complete run of this matrix is the result. Thresholds, seeds, widths,
depths, scales and clip thresholds do not move after outcomes are visible.

**No third lever is added without its own amendment**, which is the rule wave 11
honoured when its own completion expectation was not met, and it binds here for
the same reason: a campaign that keeps adding parameters until one works has
stopped testing a hypothesis.

## 10. What this cannot claim

- **Nothing about a fix.** A lever that recovers the gain shows the collapse was
  avoidable; it does not make the recovered configuration the one the paper
  reports, because it was not the registered operating point of any prior result.
- **Nothing about causation from §1's correlation.** ρ = −0.970 over 24 cells is
  an observation. H15-2 is what would let a causal reading be attempted, and only
  in conjunction with H15-1.
- **Nothing about other widths, depths or dimensions** than those run. d64 is not
  in this matrix: AZ8-6 was voided for degeneracy and re-asking it needs the
  outcome of H15-1 first, so it belongs to a successor registration.
- **Nothing about the recurrent substrate**, which is not in this matrix at all.
