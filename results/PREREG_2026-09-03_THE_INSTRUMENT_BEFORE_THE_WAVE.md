# Preregistration — the estimators, bars and void rules for five instruments that do not yet have a wave

**Registered:** 2026-09-03, **before any cell using any instrument below
exists**, and before any wave is designed around one. The instruments were built
first (commits `9a237f6`, `d4dcc1b`, `b812aea`, `5cd8257`, `2f09c91`); this
fixes what each of them is allowed to conclude, so that a later wave inherits its
bars instead of choosing them after seeing a curve.

This is **not a wave registration.** It has no cell count, no fleet, no cost and
no launch. A wave using any of these must still register its own design, its own
seeds and its own analyser. What it may not do is re-derive an estimator or a
bar that is fixed here.

---

## 0. The binary changes, and that is the most consequential fact in this document

Every instrument below is new code in `shd-instrument`. The campaign's pinned
binary is
`3afd4434431a75a26cc9d5fa46831341fc2f1dd0ef08dc308e18ca139b576364`, and it does
not contain any of it. Two consequences, both blocking:

1. **No wave using these instruments may pair against an archived half.** The
   wave-22 rule — every wave self-contained on one binary, both arms of every
   comparison produced by it — applies with no exemption. An `intact` cell from
   `w22cov` and a `window-shuffled-w8` cell from a new binary are not a pair,
   and a difference-in-differences over them would be measuring the compiler
   alongside the manipulation.
2. **The reproduction gate runs before any wave is credited, not after.** The new
   binary re-runs the anchor cells at n=3 and matches every recorded digit, as
   the kernel ablation's binary did. Every default path in this change is
   asserted bit-identical in the Rust test suite — `--tau-m` absent, `--temporal
   intact`, the default read-out, no probe, no split — but a passing unit test is
   a claim about a fixture and the gate is a claim about the corpus. If the gate
   fails, nothing below runs until it is understood.

Neither of these is negotiable by a wave that is short of slot-hours.

---

## 1. The window-shuffle timescale estimator

`--temporal window-shuffled-wN` permutes bins only within disjoint N-bin
windows. Sweeping N is a *timescale* measurement only if "timescale" is defined
before the curve exists; otherwise it is a post-hoc read of whichever rung looks
like an elbow.

**Registered estimator.** Let `DiD(N)` be the campaign's unchanged estimator —
`(attn_intact − attn_wN) − (rate_intact − rate_wN)` on seed-paired quadruples —
and `DiD(full)` the same quantity for `bin-shuffled`, measured **in the same
wave, on the same binary**. Define

> **τ½ = the smallest N on the registered ladder at which DiD(N) ≥ ½·DiD(full)**,
> with linear interpolation between the bracketing rungs and reporting in
> **bins**, converted to ms by the cell's own `dt_ms`.

**Registered ladder:** N ∈ {2, 4, 8, 16, 32, 64, 128}, plus `bin-shuffled` as
the N ≥ steps rung. Fixed here so a wave cannot add a rung after seeing where
the curve turns.

**When τ½ is not reported.** If `DiD(full) ≤ +0.03` — the campaign's bar — there
is no order effect to locate and τ½ is **undefined, not large**. If `DiD(N)` is
not monotone in N to within ±0.02, the ladder is reported as a table and τ½ is
**withheld**: a half-maximum on a non-monotone curve is an artefact of which
crossing you take.

**What τ½ is not.** It is not a membrane time constant, not a correlation time
of the stimulus, and not comparable across contracts without the `dt_ms`
conversion. A wave reporting it in bins at `fixed-t100` and in bins at
`published-2ms` would be reporting two different quantities under one name.

---

## 2. Spike dropout, and what its null would mean

`--temporal spike-dropout-pN` deletes each spike independently at N%, from a
mask drawn once per sample before the first epoch.

**The frozen mask is the registration.** Redrawn each epoch, dropout is noise
augmentation rather than information removal, and it would lift both arms while
wearing an information-removal label. The literature anchor, as far as this
record has verified it: Cramer et al. (2022) report that noise injection *"was
effective in decreasing overfitting"* on SHD
([`CITATIONS_2026-09-03_SECTION_0_AGAINST_PRIMARIES.md`](CITATIONS_2026-09-03_SECTION_0_AGAINST_PRIMARIES.md)).
They do not report on spike dropout specifically and this document does not
attribute that to them — the direction of the risk is enough to register against,
and overstating the source would be the same error §8 corrects. `apply_temporal` is called once per sample and the
realisation is a pure function of the seed;
`dropout_is_a_frozen_mask_not_per_epoch_noise` pins it and the binomial
retention band would reject a per-epoch redraw by thousands of sigmas.

**Why this operator exists.** Every other registered manipulation preserves
per-channel counts exactly. A null under all of them is therefore ambiguous
between two readings — *the read-out uses only rate*, and *the measurement is
insensitive* — and the campaign has no instrument that separates them. Dropout
is the one operator that moves rate, so it separates them:

| dropout cost | shuffle cost | reading |
|---|---|---|
| large | large | the read-out uses both; neither null is instrumental |
| large | ~0 | the read-out uses rate and not order — a real, publishable narrowing |
| ~0 | ~0 | **the measurement is insensitive.** No shuffle result from this instrument means anything, and that is a finding about the instrument |
| ~0 | large | the read-out uses order and is robust to rate loss |

**Registered bar.** At the anchor (`h128 / published-2ms / adjacent-sum-5 /
d32L4 / e400`), `p30`: the paired **rate-arm** accuracy drop from `intact` is
**> 0.03**, in **≥ 9 of 12** seeds. This is a check on the manipulation, not on
the hypothesis — a dropout that costs the rate arm nothing has not removed
information the network was using, and no reading in the table above applies.

---

## 3. Hidden shuffle, and the only exact zero in this campaign

`--temporal hidden-shuffled` permutes the hidden spike train's time axis for the
read-out, after the substrate has run.

**Registered validity gate, and it is an equality.** A `rate` arm under
`hidden-shuffled` must produce a cell **byte-identical on every scientific field**
to its `intact` twin at the same seed. Not "within 0.001" — identical. The rate
read-out accumulates from the unpermuted buffer and the spiking loop never sees
the permutation, so the equality holds by construction and
`a_hidden_shuffle_costs_the_rate_read_out_exactly_nothing` asserts it across all
four base arms.

**If that equality fails in a wave, every cell of that wave is void**, including
the attention cells that would have supported whatever it was run for. There is
no reading under which a rate arm noticing this manipulation is informative
about the read-out; it is the instrument reporting that it is not the instrument
described here.

**What it separates.** Every input-space operator confounds two things: temporal
structure the substrate *built*, and the substrate having been *driven* by
ordered input. `hidden-shuffled` leaves the second intact and destroys only the
first. **Registered as a question, not a prediction.** The campaign has no basis
for predicting whether `DiD(hidden-shuffled)` matches `DiD(bin-shuffled)` — a
match would say the read-out's advantage is entirely about the order of the
hidden representation, and a gap would say part of it is about what ordered
input does to the substrate.

---

## 4. The no-position arm, and the weaker claim it can make

`--attn-readout no-position` zeroes the positional code. Mean-pooled attention
is then permutation-invariant, so the arm keeps every parameter and every
pairwise interaction and loses the only path to order.

**Registered bar.** At the anchor, `DiD` computed with the `no-position` arm in
place of the rate arm is **≥ +0.03 smaller** than the standard `DiD`, in **≥ 9
of 12** seeds. That is: removing the read-out's access to order costs at least
as much as removing order from the data.

**One thing it cannot say, recorded here so a wave does not claim it.** The
no-position arm is order-invariant *to rounding*, not exactly: mean pooling
reassociates when the trace is reordered, and the first version of its test
failed by one ULP. So this arm can report "smaller than rounding" and never
"exactly zero". `hidden-shuffled`'s rate arm is the one that reports an exact
zero, and it is the one a wave should read a zero from.

**RNG lineage.** `positional_code` draws nothing, and
`no_read_out_variant_moves_the_initialisation` asserts that a `no-position` arm
and a default arm at the same seed are bit-identical in every parameter and equal
in parameter count. Seed pairing against intact arms depends on this and it is
checked rather than reasoned.

---

## 5. QK-norm — a read-out, with a bar at h128 before it is read at h1024

`--attn-readout qk-norm` L2-normalises the query and key before the score, and
replaces the `1/sqrt(d)` scale with `sqrt(d)`.

**The scale change is registered, not tuned.** `qhat · khat` is bounded by 1, so
dividing by `sqrt(d)` puts every score inside ±0.18 at the registered d=32, the
softmax is within rounding of uniform, and the arm is *dead* rather than
*bounded*. `sqrt(d)` restores an O(1) score while `||q|| ||k||` no longer enters
at all. It is fixed here. **If a wave sweeps it, the arm stops being a test of
the collapse hypothesis and becomes a search for a working read-out**, and must
be reported as the latter.

**Registered h128 bar, and it runs first.** At the anchor, the `qk-norm`
attention arm's accuracy is within **±0.03** of the archived default arm's, and
`DiD(bin-shuffled)` under `qk-norm` is within **±0.03** of the default's.

- **Both inside the band:** QK-norm is the same instrument with the score term
  bounded, and its h1024 result may be read against the default arm's.
- **Either outside:** it is a **different read-out**. Nothing measured with it
  transfers to the paper's headline, the h1024 result is reported as a property
  of a different arm, and the collapse claim is not resolved by it.

The h128 bar is not a formality and is not conditional on the h1024 result. It
runs in the same wave, and a wave that produces only the h1024 half is
uninterpretable.

---

## 6. The h1024 saturation prediction, registered before the probe is pointed at it

`--probe-epochs L --probe-out FILE` writes read-out diagnostics on a bounded
evaluation subset at chosen epochs. The account this is built to test:

> The h1024/`d32l4` collapse is a lost fit — training loss reaches a minimum
> around epoch 39–99 and ends 56× above it in 63 of 68 cells
> ([`FINDING_2026-08-29_THE_H1024_COLLAPSE_IS_A_LOST_FIT.md`](FINDING_2026-08-29_THE_H1024_COLLAPSE_IS_A_LOST_FIT.md)),
> and truncating to e100 avoids it — **+0.0827** against e400's **−0.1318**,
> while `d32l2` moves only **+0.0149** between the two budgets
> ([`RESULT_2026-08-30_W23_THE_COLLAPSE_IS_LATE.md`](RESULT_2026-08-30_W23_THE_COLLAPSE_IS_LATE.md)).
> *Why* it goes late is unexplained. The proposed mechanism is softmax
> saturation: `||q|| ||k||` grows without bound, attention rows become deltas,
> and the read-out degenerates into a single-timestep gather.

**Registered prediction — H-SAT.** Probing `ff+fixed+attn` at `h1024`,
`published-2ms`, `adjacent-sum-5`, `e400`, at epochs **{1, 25, 50, 100, 200,
400}**, on all four `d32l4` blocks:

1. **Entropy falls where the fit is lost and not elsewhere.** Mean
   `normalised_entropy` at epoch 400 is **< 0.30** at `h1024/d32l4`, and
   **> 0.60** at both `h1024/d32l2` and `h1024/d32l4` restricted to epoch 100.
2. **The fall is late.** At `h1024/d32l4`, `normalised_entropy` at epoch 100 is
   **> 0.60** and at epoch 400 is **< 0.30** — the collapse is inside the
   window wave 23 located, not present from the start.
3. **The named term grows.** Median `q_norm × k_norm` over the probed layers
   rises by a factor of **≥ 10** between epoch 1 and epoch 400 at
   `h1024/d32l4`, and by **< 3×** at `h1024/d32l2`.

**All three are required.** Entropy collapsing without the norms growing is a
different mechanism and must be reported as unexplained rather than as
confirmation — the same discipline `H15-2` established, for the same reason.

**Named outcomes, in every direction.**

| outcome | reading |
|---|---|
| all three met | the collapse has a mechanism, "located but unexplained" is retired, and QK-norm's h1024 result becomes a prediction rather than a hope |
| 1 and 2 met, 3 not | the read-out saturates and the proposed cause is wrong. Reported as a **new** open problem, narrower than the current one |
| 1 not met | the read-out does **not** saturate at h1024/d32l4. The saturation account is withdrawn, QK-norm's registration in §5 loses its motivation, and its h1024 arm is reported as a null with the motivation retracted |
| entropy already < 0.30 at epoch 1 | the probe is measuring initialisation, not training. **Void**: the epoch grid is wrong and no verdict is taken from it |

**The probe cannot alter what it observes**, and that is asserted end to end
rather than left to the borrow checker: `probing_does_not_change_the_cell` runs a
full cell twice and compares every measured field byte for byte. If a wave finds
a probed cell differing from an unprobed twin, **every probed cell in it is
void**, on the H15-4 principle.

---

## 7. The τ_m ladder, and its saturation void rule

`--tau-m MS` makes the membrane constant a per-cell parameter. The calibrated
value is 10.05 ms and the default path is asserted bit-identical across all eight
arms.

**Registered void rule — the standing gate, unchanged, and no exemption.** A
cell whose `saturated_fraction` exceeds **0.05** is void. That is
`cell_validity.SATURATED_MAX` as it already stands, and this ladder does not
move it.

I first registered a looser threshold here — 0.20, on the reasoning that a τ
ladder is *expected* to move firing rates and a rung that merely fires more is
not a broken cell. That was wrong for a structural reason, not a scientific one.
`cell_validity.py` is the authority on whether a cell is valid, and a
registration that declared a different threshold would have created a second
owner disagreeing with the first: a cell at 0.12 would have been non-void by this
document and void by the gate that actually decides. That is the exact defect
this session's operator-invariant work removed from the temporal manipulations,
and re-introducing it in a preregistration would be worse than leaving it in
code.

**Precedent for what an exemption looks like, and why this ladder gets none.**
Ten `w13rec` cells voided by saturation at `saturated_fraction` 0.055 to
**0.523** — up to 52% of hidden units pinned
([`PAPER_RESULTS_TABLE.md`](PAPER_RESULTS_TABLE.md) §"Reading"). Wave 25's first
manipulated recurrent cell voided the same way at 0.305, and the exemption that
followed is scoped **by substrate** — recurrent — never by wave or by threshold
(`test_the_saturation_exemption_is_by_substrate_and_not_by_wave`). A τ ladder on
a feed-forward arm falls under no such exemption and asks for none.

**A rung is void as a whole, not per cell.** If more than 3 of 12 seeds at a rung
void, the rung is void: a rung reported from its surviving seeds is a
survivorship statistic, and the survivors are exactly the seeds whose
initialisation happened to fire less.

**If the long rungs void wholesale, that is the result.** The ladder is then
reported as bounded by the standing gate — "τ above X cannot be measured on this
arm without saturating it" — and **not** re-run under a loosened one. A wave that
wants an exemption registers it separately, with a substrate-scoped argument of
the kind wave 25's carries, and does not get it from here.

**Registered as a threshold, never as a prediction about which rung trips it.**
I wrote this rule expecting "longer τ raises firing rates", and the first
monotonicity test failed against the campaign's own Glorot weights — at dt = 4 ms
the total rate went 7.675 at τ = 2.5 and 6.775 at τ = 5. With mixed-sign input a
larger `alpha` retains inhibition as faithfully as excitation, so the aggregate
is not monotone in τ. The direction holds under purely excitatory drive and SHD's
weights are not constrained to be. A ladder registered on the prediction rather
than on the measurement would have been registered on something false.

**Registered ladder:** τ ∈ {2.5, 5.0, 10.05, 20.0, 40.0} ms. 10.05 is a rung and
not a baseline appended to the ladder, so the calibrated point is measured the
same way as the others.

---

## 8. The speaker-held-out validation split, and what it must not be used for

`--train-speakers FILE --val-speakers a,b` holds every trial from those speakers
out of training and reports `val_accuracy` beside `accuracy`.

**The measurement that motivates it.** From `data/shd/shd_test.h5`'s
`extra/speaker`: speakers 4 and 5 appear **only** in test and account for
**1,840 of its 2,264 trials — 81.3%**; the remaining 424 are held-out trials from
the ten speakers that are also in train (0–3, 6–11). A validation split drawn at
random from train therefore contains **zero** unseen speakers while standing in
for a set that is 81.3% unseen speakers.

*(An earlier review note of mine said the test set "is two unseen speakers". It
is 81.3%. The correction sharpens the argument rather than weakening it, and it
is recorded here because the number is now load-bearing.)*

**Registered use.** Model selection only — an epoch, a read-out variant, a
membrane rung. Any number selected on the validation split is reported with the
split named and the held-out speakers listed; `val_speakers` is in the cell for
exactly that.

**Registered non-use.** No headline number, no `DiD`, and no bar in this document
is computed on the validation split. Every estimator here remains on the official
test set, so that this change adds a selection surface and moves no published
quantity.

**Registered held-out pair:** speakers **6 and 7**, giving n_train 6,939 and
n_val 1,217 of 8,156 — measured with `shd-instrument speaker-split`, not
computed by hand. Fixed here so a wave cannot choose the pair that makes its
validation curve behave.

**What it does not reproduce, stated rather than corrected.** Holding out whole
speakers gives a validation set that is 100% unseen-speaker against the test
set's 81.3%, so it is *harder* than the test set rather than matched to it. That
is the safe direction. Matching it would need a second selection mechanism with
its own seed and its own argument for why it does not leak, and none is
registered.

---

## 9. Wave discipline, unchanged and restated because five new instruments make it easier to break

1. **One binary per wave, and every comparison self-contained inside it.** The
   wave-22 rule. §0 says why it binds harder than usual here.
2. **The analyser is frozen and committed before the first cell exists**, in the
   same commit as the wave's own registration, and its output is the authority
   on every verdict. Practice since wave 21.
3. **No threshold moves after a cell lands.** A bar that turns out to be wrong is
   reported as a wrong bar with the result it produced, and a new registration
   changes it for the next wave.
4. **A check that could not run is never reported as a check that ran.** Cells
   predating the per-operator audit fields are counted by
   `cell_validity.temporal_fields_absent`, not waved through, and `hidden-shuffled`
   without its one distinguishing field is a problem rather than a pass.
5. **Voids are reported, not dropped.** §7's rung rule and §6's probe rule both
   name what happens to the cells they void.

---

## 10. What this document does not do

- **It registers no wave.** Nothing here schedules a cell, and no cost estimate
  appears because nothing is being spent.
- **It does not predict any headline.** §1, §3 and §5's h1024 half are registered
  as questions; only §2, §4, §6 and the bars in §5's h128 half are predictions.
- **It does not license reuse.** §0 forbids exactly the reuse that would make
  these waves cheap.
- **It does not address the gain/DiD dissociation**, which remains the paper's
  leading open problem and still has no design.
