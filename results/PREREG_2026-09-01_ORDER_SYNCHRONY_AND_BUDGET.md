# Preregistration — is the shuffle cost about order, and does it survive a second budget

**Registered:** 2026-09-01, **before any cell of wave 24 exists and before any
instance for it is launched.** Attested by git history: this file and
`scripts/aws/analyse_wave24.py` are committed together, and the first cell is
produced afterwards.

**Analyser:** `scripts/aws/analyse_wave24.py`, frozen in the same commit as this
file, and the authority on every verdict below.

**Pinned binary:** `3afd4434431a75a26cc9d5fa46831341fc2f1dd0ef08dc308e18ca139b576364`,
already published at `s3://binn-campaign-v2-511192439661-us-east-1/input/binary.sha256`
and the binary every wave-22 cell was produced by. This wave launches into that
same bucket, so `bootstrap.sh` downloads the pin and aborts on mismatch rather
than rebuilding. That is what makes §2's reuse legitimate; see §5.

---

## 1. The two things the paper cannot currently answer

### 1.1 The lead claim names a mechanism it has not isolated

§3.5's difference-in-differences is attention's accuracy cost under
`bin-shuffled` against the rate read-out's own cost, on shared seeds. Wave 22
took it to 21 of 21 operating points. Every one of those DiDs is built from a
**single** destruction operator.

`bin-shuffled` applies one permutation of the time bins per sample. It destroys
temporal order — and it also *displaces almost every spike*. Those two things
are not separated anywhere in the corpus, so the sentence "attention reads
temporal order" is, on the present evidence, indistinguishable from "attention
is more brittle than the rate read-out to having its input moved about."
A reviewer who notices this can delete the paper's mechanism claim without
disputing a single number in it.

**`reversed` separates them, and it is already implemented.**
`binn-learn/src/shd_temporal.rs:196` reverses the frame sequence. Per its own
audit, reversal relocates essentially every entry and carries a large mean bin
displacement — *the same kind of insult as `bin-shuffled`* — while preserving
per-channel counts, cross-channel synchrony, and every inter-spike interval
magnitude. And because `apply_temporal` is applied to the training split as
well as the evaluation split (`shd_instrument.rs:686`, `:694`), a globally
reversed task is **isomorphic to the intact task**: no information is lost, only
the arrow of time is inverted.

So reversal is a *displacement-matched, information-preserving* control. If the
DiD is brittleness, reversal reproduces it. If the DiD is structure, reversal
costs nothing. Nothing else in the corpus makes that cut.

> **Correction to an earlier reading, recorded because it drove the plan.**
> This wave was scoped on my claim that `channel-shuffled` was the
> order-preserving control. It is not. `shd_temporal.rs:230` draws an
> independent permutation **per channel**, which destroys temporal order *and*
> cross-channel synchrony — strictly more than `bin-shuffled`, not less. The
> module docstring says so directly. `channel-shuffled` is still worth running,
> for a different question (§1.3), but the confound above is answered by
> `reversed`.

### 1.2 The claim rests entirely on one budget

No `bin-shuffled` cell has ever run at any budget but `e400`, at any operating
point, in any wave. The mechanism claim is therefore stated at one point on an
axis that `PREREG_2026-08-29_THE_COLLAPSE_IS_LATE.md` has already shown is not
inert: at h1024 the fit is *lost* late, so e400 and e100 are different regimes
for at least one arm of at least one point. Whether the DiD is a property of the
model or a property of training it for 400 epochs is currently unknown, and the
manuscript discloses this as a limit rather than testing it.

### 1.3 A third question the same cells answer for free

`bin-shuffled` destroys order and preserves within-bin synchrony;
`channel-shuffled` destroys both. The difference between their DiDs is the
contribution of **cross-channel synchrony**, which the paper does not mention
and which most published shuffle controls conflate with order. This is
registered below as a question, not a prediction.

## 2. Design

Four temporal conditions — `intact`, `bin-shuffled`, `reversed`,
`channel-shuffled` — at `e400` and `e100`, 12 seeds, on the wave-22 pinned
binary.

**At `e400` the `intact` and `bin-shuffled` arms are not re-run.** They exist,
at 12 seeds, from wave 22, on the identical pinned binary and in the identical
bucket. Re-running them would cost 288 slot-hours to reproduce cells that
`bootstrap.sh`'s own pin guarantees are the same experiment. The one-binary rule
is about the binary, and the binary is unchanged. §5 makes this checkable rather
than asserted.

**At `e100` nothing exists**, so all four conditions run.

### The points

Three at `e400`, all drawn from wave 22 so the reuse above is available. The
rate arm carries no read-out depth, so one `ff+fixed` set serves each
`(width, contract, geometry)`.

| # | point | read-out | wave-22 DiD |
|---|---|---|---:|
| P1 | h128 / `published-2ms` / `adjacent-sum-5` | `d32/L2` | +0.1145 |
| P2 | h1024 / `published-2ms` / `adjacent-sum-5` | `d32/L1` | +0.0675 |
| P3 | h128 / `fixed-t250` / `adjacent-sum-5` | `d32/L4` | +0.1119 |

P2 is the point that must be in this wave. It is the **dissociation**: the gain
there is *negative* (−0.0159) while the DiD is firmly positive (+0.0675). It is
the single point at which "attention helps because it reads order" is hardest to
tell apart from "attention is simply more fragile", and it is where a reviewer
will look first. P1 is the nearest neighbour of the manuscript's headline
geometry, differing only in read-out depth. P3 moves the resolution axis.

Two at `e100`, the two anchor-contract points:

| # | point | read-out |
|---|---|---|
| Q1 | h128 / `published-2ms` / `adjacent-sum-5` | `d32/L2` |
| Q2 | h1024 / `published-2ms` / `adjacent-sum-5` | `d32/L1` |

`fixed-t250` is not carried to `e100`. The budget question is about the
mechanism, not about resolution, and two widths at the extremes of the ladder
answer it more cleanly than a third contract at one width.

### Cell count

| block | conditions | attention | rate | cells |
|---|---|---:|---:|---:|
| `e400`, P1–P3 | `reversed`, `channel-shuffled` | 72 | 72 | **144** |
| `e100`, Q1–Q2 | all four | 96 | 96 | **192** |

**336 cells.** Attention counts are points × conditions × 12 seeds; rate counts
are geometries × conditions × 12 seeds (3 geometries at `e400`, 2 at `e100`).

### What is deliberately not here

`fixed-t500` is the third resolution rung and is left out. Its attention cell
runs a measured **18.44 h** against `fixed-t250`'s 5.71 h — the `T²` term — so
including it would add ~306 slot-hours, more than half the wave again, and would
set the wave's makespan by itself. That is a cost decision, it is recorded here
rather than in a commit message, and it is reversible: adding P4 = h128 /
`fixed-t500` / `d32/L4` costs one line in `plan_cells.py`.

## 3. Hypotheses

Throughout, **DiD(X)** at a point is
`(attn_intact − attn_X) − (rate_intact − rate_X)` on seed-paired quadruples,
which is the paper's estimator unchanged.

**H24-1 — the cost is not generic perturbation brittleness.**
At each of P1, P2, P3: **DiD(`reversed`) < +0.03**.

This is the wave-21/22 bar, unchanged and now used from the other side. There it
was the threshold a real effect had to clear; here it is the threshold a
displacement-matched null must *fail* to clear. Reusing it is deliberate — a bar
invented for this wave would be a bar chosen after seeing which way the argument
needed to go. The ratio DiD(`reversed`) / DiD(`bin-shuffled`) is reported
alongside and carries no verdict.

**H24-2 — does synchrony contribute?** *Registered as a question, not a
prediction.* At each of P1, P2, P3: is
**|DiD(`channel-shuffled`) − DiD(`bin-shuffled`)| ≤ 0.03**?
The campaign has never separated order from synchrony and has no basis for a
directional prediction.

**H24-3 — the contrast survives a second budget.**
At Q1 and Q2, at `e100`: **DiD(`bin-shuffled`) > +0.03**, positive in **≥ 9 of
12** seed quadruples. The H22-1 bar, unchanged.

**H24-4 — the order control holds at the second budget too.**
At Q1 and Q2, at `e100`: **DiD(`reversed`) < +0.03**.
Without this, a small DiD(`reversed`) at `e400` could be read as "reversal is
cheap because 400 epochs is past convergence" rather than as a statement about
structure.

**H24-5 — coverage is mechanical.**
All 336 cells land with `mechanical_status` clean and `non_finite_forward: 0`.
This can fail only by cells failing to run.

## 4. Named outcomes, in every direction

| outcome | reading |
|---|---|
| H24-1 MET at all three | The mechanism claim survives its hardest control. The manuscript may say **temporal order** without hedging, and gains a displacement-matched null that no comparable paper reports. This is the outcome that most strengthens §3.5. |
| H24-1 NOT MET at any point | **The paper's mechanism sentence is wrong as written and is rewritten in the abstract, not in a footnote.** "Attention reads temporal order" becomes "attention is more sensitive than the rate read-out to input perturbation", the DiD keeps every number it has, and the interpretation is withdrawn. This is a real possibility and the wave exists because it is. |
| H24-1 MET at P1 and P3, NOT MET at P2 | The dissociation point is brittleness and the rest is order. The paper's claim acquires a boundary at h1024, alongside the collapse that already lives there — and §3.8's h1024 story and §3.5's mechanism story become one story. |
| H24-2 answered ≤ 0.03 | Order alone. The paper's language is already correct and gains a citation-grade control against the "shuffles conflate order and synchrony" objection. |
| H24-2 answered > 0.03, `channel` costlier | Attention reads cross-channel synchrony as well as order. The mechanism is *broader* than claimed, which is a new result and gets its own subsection. |
| H24-2 answered > 0.03, `channel` cheaper | Destroying strictly more structure costs strictly less. That is not a mechanism finding; it is evidence the manipulation interacts with training in a way nothing here models, and it is reported as a defect to investigate, not as a result. |
| H24-3 MET at both | The "all of it is `e400`" limit — currently the second sentence of §4.6's scope paragraph — is retired by measurement. |
| H24-3 NOT MET at either | The DiD is a property of the long-budget regime. This is the more interesting failure: it ties the mechanism to the same late-training dynamics as the h1024 collapse, and the paper says so. |
| H24-4 NOT MET while H24-1 MET | Reversal is cheap at `e400` and not at `e100`, which would mean the `e400` null is a convergence artefact and H24-1's support is weaker than it looks. Both are then reported together and neither is quoted alone. |

## 5. The one-binary rule, and how this wave keeps it while reusing cells

`bootstrap.sh` states it: *"a campaign whose cells came from more than one
binary is not one experiment."* Wave 22 obeyed it by running all four arms —
504 cells where 180 would have answered the question — because the archived
corpus predated the forward-finiteness guard.

This wave reuses instead, and the difference is that **the binary has not
changed.** Enforcement is not a promise in this document:

1. `bootstrap.sh:78` downloads `input/binary.sha256` from the campaign bucket if
   it exists, downloads the pinned binary, and **aborts** if the sha256 differs.
   It rebuilds only when no pin exists. The pin in
   `binn-campaign-v2-511192439661-us-east-1` is
   `3afd4434431a75a26cc9d5fa46831341fc2f1dd0ef08dc308e18ca139b576364`.
2. All four wave-22 instances recorded that sha in their gate objects
   (`gates/i-*.json`). There is no second binary in that bucket's history.
3. `analyse_wave24.py` keys every cell by its **wave label** and prints, for
   every arm of every contrast, which wave supplied it. A reused arm and a
   fresh arm cannot be swapped by filename order, and the provenance is on the
   face of the output rather than in this file.

If the pin in that bucket is ever found to differ from the value above, the
reuse is void and the `e400` block must be re-run in full. That is the
condition, stated before the data.

## 6. Cost and duration, and the basis for both

Estimated from the **measured** median `wall_secs` of wave-22 cells at the same
configurations, on the same fleet and the same binary — not from
`estimate_cost.py`'s extrapolation, which the archive shows over-predicts.

| block | cells | slot-hours |
|---|---:|---:|
| `e400` attention (P1–P3) | 72 | 398 |
| `e400` rate | 72 | 40 |
| `e100` attention (Q1–Q2) | 96 | 131 |
| `e100` rate | 96 | 18 |
| **total** | **336** | **587** |

`e100` cells are priced at one quarter of their `e400` medians; the budget is
the only thing that changes and training is linear in epochs.

- **Wall clock:** 587 slot-hours across 64 slots is **9.2 h** of ideal
  packing. The longest single cell is 5.71 h, so the tail cannot dominate the
  way `fixed-t500` would have. **Expect 11–13 h** on the four-node fleet.
- **Cost:** **≈ $26** of EC2 spot, plus a few dollars of EBS and transfer.

**The estimator is calibrated, and here is the calibration.** The same
arithmetic applied to wave 22's 2,423 measured slot-hours predicts **$108**.
Wave 22's actual EC2 spot charge, from Cost Explorer for 2026-08-30 and
2026-08-31, was **$107.66**. The model is stated with its check because the last
ETA in this campaign was quoted without one and was wrong by a factor of two.

## 7. What this wave still will not answer

- **Three of 21 points at `e400`, two at `e100`.** H24-1 is a claim about the
  points measured. It does not license "the mechanism is order everywhere."
- **Two budgets, not a curve.** `e100` and `e400` bracket nothing in between and
  nothing beyond.
- **Reversal cannot separate order from direction.** A read-out that used
  interval structure but ignored the arrow of time would produce
  DiD(`reversed`) ≈ 0 and would satisfy H24-1 without reading order in the
  strong sense. The wave therefore supports "attention reads temporal
  structure", and the manuscript must not upgrade that to "reads the direction
  of time" on this evidence.
- **The rate arm's own cost is subtracted throughout**, so every number here is
  attention's *excess* sensitivity, as everywhere else in the paper.
- **`rec+alif` is untouched.** The recurrent substrate carries its own shuffle
  question and its own wave.
