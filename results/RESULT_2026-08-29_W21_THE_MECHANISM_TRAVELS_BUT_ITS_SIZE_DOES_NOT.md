# Result — wave 21: the temporal-order mechanism travels, and its size is not the gain

**Registered:** [`PREREG_2026-08-27_THE_MECHANISM_ACROSS_THE_DESIGN_SPACE.md`](PREREG_2026-08-27_THE_MECHANISM_ACROSS_THE_DESIGN_SPACE.md),
before any cell of wave 21 existed and before any instance for it was launched.
**Analyser:** `scripts/aws/analyse_wave21.py`, frozen in the same commit as the
preregistration. Its output is
[`VERDICTS_W21.md`](shd_attention_campaign_v2/VERDICTS_W21.md) and every verdict
below is cross-checked against it by `scripts/check_verdicts_transcribed.py`.
**Binary:** the campaign's pinned `22d97c51ab02`, one binary across all
twenty-one waves.
**Status:** complete — **168/168 cells settled, zero divergences and zero voided
cells**, nothing retried, no threshold moved, no arm extended.

---

## 1. Verdicts

| ID | statement | verdict |
|---|---|---|
| **H21-1** | The mechanism is not unique to h128 | **MET** |
| **H21-2** | Where the gain inverts, the shuffle cost collapses | **NOT MET** |
| **H21-3** | The shuffle cost tracks the gain across width | **NOT MET** |
| **H21-4** | The mechanism survives a change of binning | **MET** |

Two of four. The preregistration named this combination in advance, and its
readings are quoted in §6 rather than reasoned out here.

## 2. The measurement

DiD(x) = (attention intact − attention shuffled) − (rate intact − rate
shuffled), seed-paired at every step. A seed missing from any of the four arms
is dropped from all four.

| operating point | quadruples | intact pairs | gain | **DiD** | positive |
|---|---:|---:|---:|---:|---:|
| h128 / `published-2ms` / `adjacent-sum-5` | 32 | 32 | +0.1275 | **+0.1205** | 32/32 |
| h256 / `published-2ms` / `adjacent-sum-5` | 12 | 12 | +0.0966 | **+0.0862** | 12/12 |
| h384 / `published-2ms` / `adjacent-sum-5` | 12 | 12 | +0.0760 | **+0.0767** | 12/12 |
| h512 / `published-2ms` / `adjacent-sum-5` | 12 | 12 | +0.0876 | **+0.0968** | 12/12 |
| h768 / `published-2ms` / `adjacent-sum-5` | 12 | 12 | +0.0560 | **+0.1881** | 12/12 |
| h1024 / `published-2ms` / `adjacent-sum-5` | 12 | **20** | **−0.1318** | **+0.1122** | 10/12 |
| h128 / `published-2ms` / `channels-700` | 12 | 12 | +0.1090 | **+0.1122** | 12/12 |
| h128 / `published-10ms` / `adjacent-sum-5` | 12 | 12 | +0.1491 | **+0.0959** | 12/12 |

**The two count columns are not the same seed set, and at one rung they differ.**
The **DiD** is paired across all four arms; the **gain** is paired across the two
intact arms only, which is what the frozen analyser's `gain()` computes and
documents — it is H21-3's corpus covariate and carries its own pair count. At
seven of eight rungs the two sets coincide and the distinction is invisible. At
**h1024 it does not**: the DiD is over **12** quadruples and the gain over **20**
intact pairs, because waves 18 and 19 extended that width's intact arms to twenty
seeds and only twelve carry a shuffled twin. Over the twelve quadruple seeds the
h1024 gain is **−0.1618**, not −0.1318.

This is disclosed rather than smoothed because a table whose columns are read as
one population is how a reader ends up quoting a gain and a DiD that were never
measured over the same trajectories. **H21-3's ρ is unaffected**: −0.1618 and
−0.1318 are both the smallest value on the ladder, so the rank is identical and
the correlation is the same either way.

The h128 anchor row is the corpus value the preregistration wrote down before
the wave existed — **+0.1205** — and the analyser reproduces it. That is a
consistency check on the pipeline, not a result of this wave.

## 3. Completion, which decides whether anything may be reported at all

The registered floor is **nine seed-paired quadruples**; below it an operating
point carries no numbers at all, not a mean and not a direction.

| operating point | quadruples | against floor |
|---|---:|---|
| h128 anchor | 32 | clear |
| h256, h384, h512, h768, h1024 | 12 each | clear |
| h128 / `channels-700` | 12 | clear |
| h128 / `published-10ms` | 12 | clear |

**Every operating point cleared, none of them marginally.** This wave lost no
cells: 168 planned, 168 settled, zero diverged and zero voided. That is unlike
wave 20, which cleared its floor at exactly 24 of 32 and would not have cleared
at 31 seeds.

## 4. H21-1 and H21-4 — the mechanism travels

**H21-1: MET** — DiD **+0.0862** (h256), **+0.0767** (h384), **+0.0968** (h512),
each against a +0.03 floor, each positive in **12/12** seeds. The hypothesis
required all three and got all three.

**H21-4: MET** — **+0.1122** on `channels-700` and **+0.0959** on
`published-10ms`, both 12/12, both against +0.03. The hypothesis required both.

Together these close the campaign's largest scope limit. Before this wave the
difference-in-differences existed at **2 of 21** operating points, both h128 /
`published-2ms` / `adjacent-sum-5`. It now exists at **9**, spanning every width
from 128 to 1024, two binning geometries and two temporal contracts.
`scripts/mechanism_coverage.py` recomputes this on every gate run and no longer
prints the one-width scope warning.

## 5. H21-2 and H21-3 — the size of the mechanism is not the gain

**H21-3: NOT MET** — Spearman ρ = **−0.143** over the six widths, against a bar
of **+0.829**, the n=6 one-tailed critical value at α = 0.05. The preregistration
set that bar as the smallest correlation six rungs can distinguish from chance at
all, and stated in advance that ρ = 0.7 would not be reported as a trend. −0.143
is not a weak positive; it is the absence of a relationship, with a sign pointing
the wrong way.

The clearest single case is **h768**: the **smallest** positive gain on the
ladder (+0.0560) carries the **largest** DiD in the wave (+0.1881).

**H21-2: NOT MET** — DiD(h1024) = **+0.1122**, positive in 10 of 12 seeds,
against a ceiling of +0.02. The bar was one-sided and would have been satisfied
by a negative DiD; it was not satisfied by a large positive one.

At h1024 the attention read-out **costs 0.1318 of accuracy relative to no
read-out at all** over the twenty intact pairs — **0.1618 over the twelve seeds
that also carry a shuffled twin**, which is the set the DiD is measured on — and
destroying temporal order still costs it **+0.1122 more than it costs the rate
arm**. On either seed set the sign is the same and the reading is the same: the
read-out is consuming temporal order while actively harming the network.

## 6. What the preregistration says this means

These readings were written before any cell of the wave existed, which is why
they are quoted rather than composed now:

> **H21-1 MET, H21-3 NOT MET** — "The mechanism is present at every width but its
> size is not predicted by the gain. The DiD is then a property of the read-out,
> not a quantitative account of what the gain is made of, and §0's framing must
> be weakened to say so."

> **H21-2 NOT MET** — "At h1024 the read-out still consumes temporal order while
> performing worse than no read-out at all. Nothing in the paper's account
> permits that, and it becomes the paper's leading open problem rather than a
> caveat."

> **H21-4 MET** — "The mechanism is about temporal order, not about
> `adjacent-sum-5`'s particular binning."

The wave therefore splits a claim the manuscript had bundled into one sentence.
**"The read-out's marginal contribution is order-dependent"** survives, and is
now measured at nine operating points instead of two. **"The gain is made of
temporal order"** — the reading behind *94.5% of the marginal contribution is
contingent on temporal order* — does **not** survive as a quantitative account,
because across width the size of the order-dependence carries no information
about the size of the gain.

## 7. What this does and does not say about the h1024 collapse

It sharpens the open problem and does not solve it. The read-out at h1024 has not
stopped doing the order-dependent thing; doing it has stopped helping.

**Overfitting is neither confirmed nor excluded by this wave.** The
preregistration's argument against it was conditional on the shuffle cost
collapsing — "overfitting predicts a *reduced* gain, not the disappearance of
order-dependence" — and the shuffle cost did not collapse, so that argument does
not fire in either direction. SHD still ships no validation set, so the
parsimonious alternative named in `PAPER_DRAFT.md` §3.5 remains unexcluded by
anything in the corpus.

Read with [`RESULT_2026-08-28_W18_19_THE_DEPTH_OPTIMUM_IS_INTERIOR.md`](RESULT_2026-08-28_W18_19_THE_DEPTH_OPTIMUM_IS_INTERIOR.md),
the h1024 row is a statement about the **`d32/L4` read-out at that width**, not
about that width: at the same width and budget, `d32/L2` gains +0.0405 in 20/20.
This wave measured the shuffle control at L4 only, so nothing here bounds the
shallower read-outs that do help at h1024, and their order-dependence is
unmeasured.

## 8. What may not be claimed

Carried from the preregistration §7, unchanged by the outcome:

- **Nothing against macOS-recorded numbers.** Cross-machine Gate F FAILs
  macOS-vs-Linux on every node of this campaign, by design. Every contrast above
  is between a treatment and a control that ran on the same fleet from the same
  pinned binary.
- **No claim about read-out depths other than `d32/L4`**, and none about the
  `channel-shuffled` operator.
- **No claim about the recurrent substrate.** `rec+alif` carries its own shuffle
  question and its own wave; none has been run.
- **No causal claim from H21-3.** A rank correlation over six rungs bounds how
  well the gain predicts the shuffle cost. Its failure bounds that prediction; it
  does not establish what the gain *is* made of, and no mechanism is offered.
- **Nothing about whether attention is the best read-out.**

Additionally, and specific to this outcome: **H21-3's failure may not be reported
as a weak or noisy positive.** ρ = −0.143 over six points is consistent with no
relationship, and the preregistration set its bar precisely to stop a
mid-magnitude ρ being narrated as a trend.

## 9. Provenance

168 cells, `w21mec`, produced on the `shd-attention-campaign-v2` fleet from the
pinned binary `22d97c51ab0204702ce44661683ff8c759c29d7f3379e2f6606b048f4f032104`
on `aarch64-unknown-linux-gnu`. The intact halves of all fourteen arms were
reused from the corpus at the same seeds and the same binary; only the
`bin-shuffled` halves are new, which is what makes the difference a difference of
differences rather than a drop.

Cells landed into `results/shd_attention_campaign_v2/` on 2026-08-29:
693 → 861 cells, 683 → 851 valid, both **+168**, so every wave-21 cell is valid
and the ten invalid cells are still the same ten `w13rec` cells. `CORPUS_BASELINE`
re-frozen in the same change; 0 modified and 0 deleted under the corpus.
