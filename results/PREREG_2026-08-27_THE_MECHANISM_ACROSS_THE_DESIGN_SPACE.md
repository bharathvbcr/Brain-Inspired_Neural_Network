# Preregistration — does the temporal-order mechanism exist anywhere but h128?

**Registered:** 2026-08-27, **before any cell of wave 21 exists and before any
instance for it is launched.** Authorised by the maintainer, who asked for
additional hypotheses and larger samples and for the fleet not to sit idle.

**Protocol:** `shd-attention-campaign-v2`, wave `w21mec`.
**Binary:** the campaign's existing pinned binary
(`22d97c51ab0204702ce44661683ff8c759c29d7f3379e2f6606b048f4f032104`). One binary
across all twenty-one waves; reused controls stay bit-comparable.
**Analyser:** `scripts/aws/analyse_wave21.py`, frozen in the same commit as this
document and before the first cell lands.

---

## 1. Why this wave exists

[`PAPER_DRAFT.md`](PAPER_DRAFT.md) leads with a difference-in-differences:
attention's accuracy cost under bin-shuffling, measured against the rate
read-out's own cost, on the same seeds and the same destruction operator.
**+0.1347 against +0.0142, a 9.5× factor at n=32.** §0 states that no published
equivalent exists for any read-out on any neuromorphic benchmark, and that this
is the paper's contribution.

`scripts/mechanism_coverage.py` derives where that contrast can actually be
computed. The answer:

> **2 of 19 operating points**, both of them h128 / `published-2ms` /
> `adjacent-sum-5`. Seventeen rungs carry the intact arms and **no destruction
> control at all**.

The campaign's best-evidenced claim is therefore also its narrowest, and its
generality is the first thing a reviewer will attack. Nothing in the corpus
answers them.

## 2. The prediction that makes this more than a coverage exercise

The gain ladder over width, `ff+fixed+attn` d32/L4 against `ff+fixed`, anchor
geometry and contract, seed-paired, derived from the corpus:

| width | n | rate | attention | gain | positive |
|---:|---:|---:|---:|---:|---:|
| 128 | 32 | 0.7057 | 0.8332 | **+0.1275** | 32/32 |
| 256 | 12 | 0.7240 | 0.8206 | +0.0966 | 12/12 |
| 384 | 12 | 0.7336 | 0.8096 | +0.0760 | 12/12 |
| 512 | 12 | 0.7357 | 0.8233 | +0.0876 | 12/12 |
| 768 | 12 | 0.7386 | 0.7946 | +0.0560 | 11/12 |
| 1024 | 12 | 0.7386 | 0.5768 | **−0.1618** | 1/12 |

§3.5 calls the h1024 inversion an anomaly "with no citation to lean on", and
names overfitting as a parsimonious alternative it does not exclude.

**If the paper's own thesis is right, the ladder predicts its own control.** The
claim is that what the read-out buys is temporal order. Where it buys a lot,
destroying order should cost a lot. Where it buys nothing — h1024, where the
gain is *negative* — there should be no order-dependent benefit left to destroy,
and the shuffle cost should collapse with it.

That is a real prediction: it is stated before the cells exist, it is derived
from the thesis rather than fitted to the data, and it can fail. A large shuffle
cost at h1024 would mean the read-out is still consuming temporal order while
losing accuracy, which the paper's account does not permit.

## 3. Design

Two arms per operating point, twelve seeds, everything else reused from the
corpus at the same seeds and the same pinned binary:

- **`ff+fixed` bin-shuffled** — the rate read-out's own shuffle cost. This is
  what makes the contrast a difference *of differences* rather than a drop, and
  it exists at only one operating point today. At 4–24 minutes a cell it is the
  cheap half.
- **`ff+fixed+attn` d32/L4 bin-shuffled.**

Seven operating points — five that complete the width ladder wave 16 filled for
the gain, and two that move something other than width, because "generalises
across width" and "generalises across binning" are different claims and the
paper supports neither:

| operating point | why |
|---|---|
| h256, h384, h512, h768 / anchor | the ladder between the headline and the collapse |
| h1024 / anchor | §2's prediction, where the gain inverts |
| h128 / `channels-700` | a different binning geometry at the headline width |
| h128 / `published-10ms` | a different temporal resolution at the headline width |

**168 new cells, ≈272 slot-hours, ≈10 h on the 26-slot fleet.** Calibrated from
measured `wall_secs` on these exact configurations, **not** from
`estimate_cost.py`, which over-predicts by a median 3.08× and prints that ratio
with every run.

The intact halves of all fourteen arms are already in the corpus at n = 12.
Nothing in this wave re-runs them.

## 4. Hypotheses

Write **DiD(x)** for the seed-paired difference of differences at operating
point *x*: (attention intact − attention shuffled) − (rate intact − rate
shuffled). At h128 it is **+0.1205**.

| ID | statement | criterion |
|---|---|---|
| **H21-1** | The mechanism is not unique to h128 | DiD **≥ +0.03**, positive in **≥ 9/12** seeds, at **each** of h256, h384, h512 |
| **H21-2** | Where the gain inverts, the shuffle cost collapses | DiD(h1024) **≤ +0.02** |
| **H21-3** | The shuffle cost tracks the gain across width | Spearman ρ between the six per-width gains and their DiDs **≥ +0.829** |
| **H21-4** | The mechanism survives a change of binning | DiD **≥ +0.03**, positive in **≥ 9/12**, at **both** `channels-700` and `published-10ms` |

H21-1 through H21-4 are independent. Any one passing rescues none of the others.

**On H21-3's bar.** 0.829 is the critical value of Spearman's ρ at n = 6 for a
one-tailed test at α = 0.05. It is the smallest correlation that six rungs can
distinguish from chance at all, which is why it is the bar rather than a number
chosen to be comfortably above or below the pilot. Six points is a weak
instrument and this states so in advance: ρ = 0.7 here is **not** a positive
result, and will not be reported as a trend.

**On H21-2's direction.** The bar is one-sided and permits a *negative* DiD. If
attention at h1024 is harmed by intact temporal structure — plausible, since its
accuracy there is 0.5768 against the rate arm's 0.7386 — bin-shuffling could
improve it, and DiD(h1024) would be below zero. That satisfies H21-2 and is
reported as satisfying it, because the prediction is about the *absence* of an
order-dependent benefit, not about its sign.

**H21-1's floor of +0.03** is one quarter of the h128 value. It is set below any
plausible dilution so that the hypothesis fails only if the mechanism is close to
absent, not merely smaller.

## 5. Named outcomes, in every direction

| observed | reading |
|---|---|
| H21-1 **MET**, H21-3 **MET** | The mechanism generalises across width and its size tracks the gain. §3.5's contrast becomes a law over the ladder rather than a measurement at a point, and the paper's lead claim is no longer a single-configuration result. |
| H21-1 **MET**, H21-3 **NOT MET** | The mechanism is present at every width but its size is not predicted by the gain. The DiD is then a property of the read-out, not a quantitative account of what the gain is made of, and §0's framing must be weakened to say so. |
| H21-1 **NOT MET** | The contrast does not survive leaving h128. The paper's lead claim is a single-operating-point result and its abstract must say so in those words. This is the outcome that would most change the manuscript. |
| H21-2 **MET** | The width collapse and the temporal-order account are the same phenomenon. §3.5's anomaly stops needing a citation to lean on, and the overfitting alternative it names is no longer parsimonious — overfitting predicts a *reduced* gain, not the disappearance of order-dependence. |
| H21-2 **NOT MET** | At h1024 the read-out still consumes temporal order while performing worse than no read-out at all. Nothing in the paper's account permits that, and it becomes the paper's leading open problem rather than a caveat. |
| H21-4 **MET** | The mechanism is about temporal order, not about `adjacent-sum-5`'s particular binning. |
| H21-4 **NOT MET** at `channels-700` only | The effect is entangled with the 140-channel reduction. Reported as a scope limit on geometry, not on the mechanism. |
| H21-4 **NOT MET** at `published-10ms` only | The effect depends on temporal resolution. Given that 10 ms binning already destroys most within-bin order, this is the weakest of the four and is read as such. |
| any arm below 9 valid seeds | **NOT EVALUABLE** for that operating point, carrying no numbers at all. |

## 6. Stopping rule

Every cell runs. No arm is extended, truncated or re-seeded on the basis of what
it shows. **An operating point returning fewer than 9 seed-paired quadruples is
NOT EVALUABLE and carries no numbers** — not a mean, not a direction.

Validity is `scripts/cell_validity.py` unmodified. The bin-shuffle manipulation
is audited by the instrument itself and by `validity_problems`; a cell whose
`temporal_condition` disagrees with its plan entry is void, which is the check
that stops a silently-intact cell from being scored as a destroyed one.

## 7. What may not be claimed

- **Nothing against macOS-recorded numbers.** Cross-machine Gate F FAILs
  macOS-vs-Linux on every node of this campaign, by design. Every contrast here
  is between a treatment and a control that ran on the same fleet from the same
  binary.
- **No claim about read-out depths other than d32/L4**, and none about the
  `channel-shuffled` operator, which destroys a different structure and answers
  a different question.
- **No claim about the recurrent substrate.** `rec+alif` carries its own shuffle
  question, its own divergence rate, and its own wave.
- **No causal claim from H21-3.** A rank correlation over six rungs bounds how
  well the gain predicts the shuffle cost; it does not establish that the gain
  is *made of* the shuffle cost.
- **Nothing about whether attention is the best read-out.** This wave measures
  what one read-out consumes, not whether another would consume less.
