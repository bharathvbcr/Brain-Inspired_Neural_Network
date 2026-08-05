# Shakedown: the recurrent arms had never been run

**Date:** 2026-08-03
**Backend:** rust only.
**Command:** `shd-instrument train-cell --arm ...`
**Cost:** h128, 3 and 20 epochs. ~31 s per 3-epoch recurrent cell after the
kernel optimisation; the same cells cost 193 s before it.

> **Read §3.5 and §4b first.** This document was written forward as the runs
> came in, and its early sections were overturned by its later ones — always in
> the same direction, from an alarming reading to a milder one. §2 concluded
> `rec+fixed` "does not learn"; at 20 epochs it reaches 0.2633 with a
> monotonically falling loss, because 3 epochs sits entirely inside an
> early-training transient. §3.2 concluded no initialisation scale is stable;
> every cell in it is a 3-epoch cell, so it never addressed steady state.
>
> The sections are kept rather than rewritten away, because the sequence is the
> point: a cheap diagnostic chosen for cost happened to sample only the window
> in which the arm misbehaves.
>
> **The load-bearing sections are §3.6.0, §3.6.2 and §4b.** At h512 — the width
> `PREREG_2026-08-02_SHD_BPTT_CEILING_RECALIF` registers — **zero of three seeds
> produce a usable `rec+alif` cell.** Two abort mid-training on genuinely
> non-finite gradient entries; the third completes with a gradient norm reaching
> **7.36e29** while showing an entirely healthy accuracy and loss curve. h128 and
> h256 are clean on the same configuration.
>
> **§3.6.0 is a retraction**: the "420 of 640 non-finite steps" figure quoted
> throughout §3.6 and §3.6.2 was an artifact of the f32 norm overflow, not a
> property of the gradients. Read it before quoting any number from those
> sections. The campaign-blocking conclusion is unaffected.
>
> §4b is the instrument defects that let such a cell pass unremarked, found by
> trying to replicate rather than by any measurement. §3.6.1 records a prediction
> made before its replicate ran — the only prediction here that survived, and
> even it overstated severity in one direction while understating it in another.

---

## claim_axis

```
axis: instrument-validity
claim: The four arms were exercised end to end on real SHD data for the first
  time. The recurrent arms produce BPTT gradient excursions large enough to
  overflow the f32 norm computation, and until this session the instrument
  neither counted them nor wrote a parseable cell when they occurred. At h128
  the excursions are concentrated in early training and do not prevent learning;
  at h512 no seed produces a usable cell.
may_claim: That `rec+fixed` and `rec+alif` at h128 / published-2ms /
  adjacent-sum-5 produce peak gradient norms above 1e12; that the peak is
  carried by a single optimizer step out of 32 per epoch (§3.4); that
  `non_finite_events` was never incremented before this session and the pass
  predicate's use of it was vacuous (§3.3, §5); and that `json_f64` wrote `inf`,
  producing unparseable cells.
  Most importantly (§3.6.0, §3.6.2): that at **h512**, the width RECALIF
  registers, zero of three `rec+alif` seeds produce a usable cell — two abort on
  non-finite gradient entries, and the third reaches a gradient norm of 7.36e29
  while showing a healthy accuracy and loss curve.
must_not_claim: That the recurrent arms do not learn — §3.5 falsifies that. At
  e20 `rec+fixed` reaches 0.2633 and `rec+alif` 0.3785, both with monotonically
  falling loss. Nor anything about any arm's accuracy ceiling: these are e3 and
  e20 cells against a registered threshold of 0.80. Nor that no initialisation
  scale is stable — §3.2 swept only e3 cells, which sit entirely inside the
  early-training transient, so it does not address steady state at all. Nor that
  adaptation stabilises recurrence: at e20 `rec+alif` is the spikier arm.
  Nor that the h512 failure is a property of recurrence rather than of this
  instrument at this width: three seeds, one scale, e20, one contract. It blocks
  running RECALIF; it is not a result about recurrent architectures.
  Nor — see §3.6.0 — that any h512 gradient was non-finite: the two aborting
  seeds had non-finite *entries*, but seed 5170001's were finite throughout.
```

## 1. Why these cells were run

Two motivations, one planned and one not.

The planned one: the recurrent kernel had just been optimised
(`AMENDMENT_2026-08-03_RECURRENT_KERNEL_TRANSPOSE_AND_SPARSITY.md`) and needed a
bit-identity check at real training density, because fixture parity is not
evidence at density. Cells were captured before and after the change.

The unplanned one: those cells are the first time any recurrent arm has been run
on real data at all. `PREREG_2026-08-02_SHD_BPTT_CEILING_RECALIF` is built on
`rec+alif` at h512, and `PREREG_2026-08-02_SHD_TEMPORAL_INFORMATION` §5.6 has a
`W_rec` pilot, but no cell from either has ever executed.

## 2. Result at 3 epochs — superseded by §3.5

3 epochs, `h128 / published-2ms / adjacent-sum-5 / s5170001`, default
initialisation.

| arm | accuracy | mean loss | epoch mean gradient norm | classes | non-finite |
|---|---:|---:|---|---:|---:|
| `ff+alif` | 0.3291 | 2.745 | 0.128, 0.151, 0.154 | 18 | 0 |
| `rec+alif` | 0.1860 | 2.879 | 36.3, 9.11, 4.11 | 16 | 0 |
| `rec+fixed` | 0.0738 | 2.933 | **9.78e12**, 4.51e4, 5.11e4 | 13 | 0 |

`ff+alif` learns normally. `rec+alif` learns slowly, with a gradient norm two
orders of magnitude high but **decreasing monotonically** — 36.3 → 9.11 → 4.11.

`rec+fixed` does not learn. Its loss is flat and slightly non-monotonic
(2.934 → 2.945 → 2.920), accuracy 0.0738 against a 0.05 chance floor, and its
epoch-1 mean gradient norm is **9.78e12**.

## 3. Why nothing caught it

The mechanism is ordinary BPTT gradient explosion. The surrogate derivative
peaks at `MATCHED_SURROGATE_ALPHA * 0.5 = 2.5` at threshold
(`shd_matched.rs:195`), and the recurrent block is drawn Glorot-uniform, whose
whole design point is a spectral radius near 1. The product gives a per-timestep
backward gain above 1, compounded over the full sequence — several hundred
timesteps at a 2 ms frame, and variable per sample (366 for test sample 0).

What matters is not that it explodes but that **the instrument reports it as an
ordinary failure**. The pass predicate (`shd_instrument.rs:540`) is

```rust
let scientific = evaluation.accuracy >= 0.80
    && ... && diagnostics.non_finite_events == 0;
```

`9.78e12` is finite, so `non_finite_events` stays 0. The cell is marked
`CELL_FAIL` — but so is a merely undertrained cell, and so is every other cell in
the table above. **The instrument cannot distinguish "did not learn" from
"diverged numerically."**

That diagnosis was right for the wrong reason, and §4b has the real one:
`non_finite_events` stays 0 here not because `9.78e12` is finite, but because
**the counter was never incremented at all**. It would have read 0 for an
infinite norm too, and on seed 5170002 it did exactly that. Fixed; the clause is
no longer vacuous.

Adam hides it further. `mean_update_rms` is `7.4e-4` for `rec+fixed` against
`1.5e-3` for `ff+alif` — entirely unremarkable, because Adam divides by
`sqrt(v)` and normalises away the scale. A run of this kind completes, writes a
well-formed cell, and looks like a weak result rather than a broken one.

This is the specific risk to the RECALIF prereg: that campaign exists to measure
a **ceiling** for the recurrent arm. A ceiling depressed by gradient explosion
would read as a finding about recurrence when it is a fact about initialisation
and surrogate gain.

### 3.1 Reproduced independently

The `rec+fixed` row was reproduced by `scripts/w_rec_scale_pilot.py`, which
builds its own initialisation via `init --w-rec-scale 1.0` rather than reusing
the shakedown's weights. It returns accuracy `0.073763251` and epoch gradient
norms `9.78e12, 4.51e4, 5.11e4` — identical digits.

That is two things at once. The divergence is reproducible through an
independently constructed initialisation; and `--w-rec-scale 1.0` is confirmed
byte-preserving, since passing it explicitly yields a bit-identical cell to the
default path that predates the flag.

## 3.2 The scale pilot — conclusion did not survive §3.5

`scripts/w_rec_scale_pilot.py`, same cell geometry, 3 epochs, scales applied to
the same Glorot draw so the lineage is held fixed.

| arm | scale | peak \|grad\| | final \|grad\| | loss drop | fire | acc |
|---|---:|---:|---:|---:|---:|---:|
| `rec+fixed` | 1.0 | 9.78e12 | 5.11e4 | 0.0141 | 0.117 | 0.0738 |
| `rec+fixed` | 0.5 | 1.36e9 | 2.17e4 | 0.0653 | 0.789 | 0.1148 |
| `rec+fixed` | 0.25 | **1.49e17** | 1.49e17 | **-0.3975** | 0.306 | 0.0733 |
| `rec+fixed` | 0.1 | 4.19e10 | 110 | 0.0456 | 0.618 | 0.1175 |
| `rec+fixed` | 0.05 | 2.23e4 | 239 | 0.0599 | 0.059 | 0.1307 |
| `rec+alif` | 1.0 | **36.3** | 4.11 | 0.0987 | 0.081 | 0.1860 |
| `rec+alif` | 0.5 | 5.64e4 | 0.308 | 0.1762 | 0.099 | 0.2478 |
| `rec+alif` | 0.25 | 1.57e7 | 141 | 0.1693 | 0.090 | 0.2235 |
| `rec+alif` | 0.1 | **8.81** | 0.238 | 0.1596 | 0.116 | 0.2142 |
| `rec+alif` | 0.05 | **5.56e13** | 5.56e13 | 0.1766 | 0.063 | 0.1789 |

### Replicated across three seeds

**Read this section in light of §3.5: every cell here is e3, and e3 sits entirely
inside the early-training transient. What varies below is how bad the first three
epochs are, not whether the arm is stable.**

Seeds 5170002 and 5170003 were run identically. Of the 30 cells:

- **2 aborted outright** — `non-finite training value` at optimizer step 31
  (seed 5170002, `rec+fixed`, scale 0.05) and step 52 (seed 5170003,
  `rec+fixed`, scale 1.0). No cell file is produced at all. For a 24-cell
  campaign that is an operational failure mode, not just a bad number.
- **4 produced infinite gradient norms**, now written as JSON `null` rather than
  the unparseable `inf` (§3.3).
- **The max/mean ratio is ~32 nearly everywhere**, across all three seeds — the
  single-batch concentration in §3.4 is not seed-specific.
- **Which scales misbehave is not consistent across seeds.** At scale 1.0
  `rec+fixed` gives 9.78e12 (seed 1), infinite (seed 2) and an abort (seed 3).

That last point is the important one: **the scale axis does not survive seed
replication.** A ranking of scales from one seed is a ranking of trajectories.

**The expected result was a threshold scale below which things are stable. That
is not what happened.** The relationship is not monotonic and not close to it:
`rec+fixed` is worse at 0.25 (1.49e17) than at either 0.5 (1.36e9) or 0.1
(4.19e10), and `rec+alif` is worst of all at the *smallest* scale tested, 0.05
(5.56e13), while being well behaved at 1.0 (36.3) and 0.1 (8.81).

Three things follow.

**Rescaling the recurrent initialisation is not the fix.** Every `rec+fixed`
cell has a peak gradient norm of at least 2.2e4, against 0.15 for `ff+alif`.
There is no scale in this sweep at which the arm is numerically comfortable.

**§4's "adaptation stabilises recurrence" is weaker than it looked.** It held on
the single comparison available before this pilot. Across scales, `rec+alif` is
bounded at 2 of 5 and explodes at 3 of 5, including the worst cell in the table.
The reading it supports is that adaptation *sometimes* helps, not that it
controls the problem.

**Two cells never recover.** `rec+fixed` at 0.25 and `rec+alif` at 0.05 have
`final = peak`, so the explosion is still growing at the last epoch, and
`rec+fixed` at 0.25 ends with a *higher* loss than it started. Those two are
unambiguously void rather than merely bad.

The likelier lever is therefore gradient clipping, which the instrument does not
have, rather than initialisation scale. That is a change to the training loop and
is not being made here.

### 3.3 A telemetry gap this exposed

`epoch_mean_gradient_norm` is a **mean over optimizer steps**, not over samples:
the norm is taken on the batch-averaged gradient once per step, and with
`batch_size = 256` over 8156 training samples that is **32 steps per epoch**.
*(An earlier draft of this section said "mean over samples". The training loop at
`shd_instrument.rs:519` takes `gradient.l2_norm()` after `gradient.scale(...)`,
so the unit is the batch.)*

Either way the ambiguity is real and decides the fix. A mean of 1.49e17 over 32
steps is consistent with every step being catastrophic, and equally consistent
with 31 ordinary steps and one at 5e18. The first calls for gradient clipping;
the second calls for finding out what is in that batch. Nothing recorded
separated them.

**Closed.** `epoch_max_gradient_norm` is now recorded alongside the mean — the
peak batch norm within each epoch. It is additive to the cell schema, reads a
value the loop already computed, and does not disturb Gate F, whose comparison
list is explicit. The ratio `max / mean` is the discriminator: near 1 means
uniformly large gradients, near 32 means a single step carries the epoch.
Results in §3.4.

## 3.4 Where the peak sits, and a confound in this document's own design

`epoch_max_gradient_step` records which optimizer step held the epoch's peak.
Across 18 cells:

- **It is not step 0.** A first-batch warmup transient is ruled out.
- **It wanders across epochs** — `[27, 20, 27]`, `[24, 31, 2]`, `[22, 7, 27]`.
  `order` is reshuffled every epoch, so a peak that moves is consistent with
  particular *samples* carrying it.
- **Epoch 1's peak clusters late**: 18-31 out of 32 steps, in nearly every cell.

That last one is not a fact about the recurrent arm. It is a fact about the
learning-rate schedule, and it undermines this document's design.

`one_cycle_lr` (`shd_matched.rs:395`) ramps from `1e-3` to `5e-3` over
`progress <= 0.3`, where `progress = step / (total_steps - 1)` and `total_steps =
epochs * batches`. **The schedule is stretched over the whole run**, so the peak
learning rate arrives at a different point in training depending on the budget:

| budget | total steps | step at peak LR |
|---|---:|---:|
| e3 (this document) | 96 | **~29** — late in epoch 1 |
| e20 | 640 | ~192 |
| e100 (campaign) | 3200 | ~960 |

The peak LR is the same `5e-3` either way; what differs is how much adaptation
precedes it. At e3 the network reaches maximum learning rate after 29 optimizer
steps. Epoch 1's gradient peak lands exactly there.

**So a 3-epoch diagnostic may manufacture the instability it reports**, and every
number in §2 and §3.2 is from a 3-epoch cell. This was chosen to make the sweep
cheap and the choice may have decided the result. Tested directly in §3.5.

## 3.5 e3 versus e20 — the explosion is an early-training transient

Same initialisation, same seed, same scale; only the budget differs.

| cell | accuracy | loss first -> last | `non_finite_events` |
|---|---:|---|---:|
| `rec+fixed` e3 | 0.0738 | 2.934 -> 2.920 | 0 |
| `rec+fixed` e20 | **0.2633** | 2.952 -> **2.687** | **2** |
| `rec+alif` e3 | 0.1860 | 2.934 -> 2.836 | 0 |
| `rec+alif` e20 | **0.3785** | 2.959 -> **2.378** | 0 |

Per-epoch peak gradient norm, `rec+fixed` e20:

```
6.7e2, 2.1e6, inf, 29, 81, 1.5e2, 1.4e2, 1.4e2, 1.6e2, 1.5e2,
1.5e2, 1.7e2, 1.7e2, 1.7e2, 1.6e2, 1.7e2, 1.7e2, 1.8e2, 1.8e2, 1.8e2
```

**It explodes for three epochs and then stops.** From epoch 4 onward the gradient
norm sits between 29 and 1.8e2 and never moves again, and the loss falls
monotonically from 2.952 to 2.687.

**The hypothesis in §3.4 was wrong, and so was the framing of §2.** The short
budget did not *cause* the instability — the explosion is worse at e20, reaching
a genuinely infinite norm in epoch 3 where e3 only reached 9.8e12. But three
epochs turns out to be **entirely inside the transient**, so an e3 cell measures
the transient and nothing else. "`rec+fixed` does not learn" was an artifact of
looking only at epochs 1-3.

This has consequences for everything above it:

- **§2's headline is wrong as stated.** `rec+fixed` does learn; it needs to get
  past epoch 3 first. Accuracy 0.0738 at e3 versus 0.2633 at e20 from the same
  weights.
- **§3.2's scale sweep is confounded.** All ten cells are e3, so all ten sit in
  the transient. The sweep measured *transient severity*, not steady-state
  stability, and its non-monotonicity is consistent with chaotic variation in
  how bad the first three epochs are. It does not support "no scale is stable" —
  it does not address steady state at all.
- **The "adaptation stabilises recurrence" reading weakens further.** At e20
  `rec+alif` is the *spikier* of the two — `1.2e11` at epoch 8 and repeated
  excursions to 1e4-1e5 throughout — while `rec+fixed` goes quiet after epoch 3.
  `rec+alif` nonetheless ends with the better loss and accuracy.

What survives is narrower and still worth having: **the recurrent arms produce
gradient excursions large enough to overflow the f32 norm, and until this session
the instrument neither counted them nor wrote a parseable cell when they
happened.** That is independent of budget, and it is what §3.3 and §5 are about.

The `non_finite_events: 2` on the e20 cell is the repaired counter doing its job
on a real case: before this session it would have read 0.

## 3.6 Width — the campaign's own configuration fails, and looks fine doing it

Everything above is h128. `PREREG_2026-08-02_SHD_BPTT_CEILING_RECALIF` runs
`rec+alif` at **h512**, where the recurrent block has 16x the fan-in. e20, seed
5170001, scale 1.0, one cell each:

| arm | width | accuracy | loss first -> last | **non-finite steps** | epochs with an infinite peak |
|---|---:|---:|---|---:|---:|
| `rec+alif` | h128 | 0.3785 | 2.959 -> 2.378 | 0 / 640 | 0 / 20 |
| `rec+alif` | h256 | 0.3613 | 2.939 -> 2.278 | 0 / 640 | 0 / 20 |
| `rec+alif` | **h512** | 0.3507 | 2.926 -> **2.241** | **420 / 640** | **17 / 20** |
| `rec+fixed` | h256 | 0.2451 | 2.948 -> 2.743 | 1 / 640 | 1 / 20 |
| `rec+fixed` | h512 | 0.1197 | 3.034 -> 2.877 | 0 / 640 | 0 / 20 |

**At h512, two thirds of `rec+alif`'s optimizer steps compute a non-finite
gradient.** Not a transient: 17 of 20 epochs contain an infinite peak, including
the last. h128 and h256 are entirely clean, so the failure appears between h256
and h512 — consistent with the `O(hidden^2)` recurrent fan-in.

**The dangerous part is that the cell looks good.** Its loss falls smoothly and
monotonically to the lowest value of any cell measured in this document, and its
accuracy is the second highest. Every field a reader would check says "healthy,
undertrained". The only field that says otherwise is `non_finite_events`, and
**until today that field was never incremented** (§4b) — it would have read 0
here, exactly as it did for every one of the 296 completed cells.

So: had RECALIF run this morning, it would have produced 24 well-formed cells at
its registered width, reporting plausible accuracies computed from a gradient
that was non-finite on two thirds of steps, with nothing in the record to show
it. That is what the repaired counter is for, and this is the case that
demonstrates it.

### 3.6.1 Seed replication — stated before the result

The h512 row above is **one cell, one seed, one scale**. Every strong claim in
this document so far has been weakened by the next measurement, so the prediction
is recorded here before the replicate lands rather than after.

Seeds 5170002 and 5170003, `rec+alif`, h512, e20, scale 1.0 — identical to the
h512 row except for the seed.

**Prediction: both replicate**, with non-finite step counts in the same order of
magnitude (hundreds out of 640), because the proposed mechanism — `O(hidden^2)`
recurrent fan-in pushing the f32 norm past 3.4e38 — is a property of the width,
not of a particular draw. h128 and h256 gave exactly 0 and 0, so this is not a
noisy quantity near a boundary.

**What would falsify it:** either replicate coming back at 0/640, which would
make the h512 cell a seed accident and this entire section an overreaction — the
same failure mode as §2 and §3.2. §3.2 is the specific precedent: at scale 1.0,
`rec+fixed` gave 9.8e12, an infinite norm, and an outright abort on three seeds.
Recurrent cells have already been shown to vary wildly across seeds once.

### 3.6.0 RETRACTION — "420 non-finite steps" was a measurement artifact

*Added after the `l2_norm` fix (`AMENDMENT_2026-08-03_L2_NORM_CONDITIONAL_WIDENING.md`).
It supersedes the seed-5170001 numbers in §3.6 and §3.6.2 below, which are kept
as written so the error is legible.*

Re-running the identical cell on the fixed binary:

| | before | after |
|---|---|---|
| accuracy | 0.350707 | **0.350707** (bit-identical) |
| `epoch_mean_loss` trace | — | **identical** |
| `non_finite_events` | 420 / 640 | **0** |
| epochs with an infinite peak | 17 / 20 | **0** |
| peak gradient norm | `inf` | **7.36e29** |

**The gradients were never non-finite.** They were finite and enormous — peaking
at `7.36e29`, a value f32 represents without difficulty, whose *square* (`5.4e59`)
it cannot. The old `l2_norm` overflowed computing the square, returned infinity,
and the repaired `non_finite_events` counter faithfully counted 420 of them. The
counter was right about what it was given; what it was given was wrong.

That accuracy and the loss trace are bit-identical across the fix confirms the
norm is purely diagnostic and never feeds back into training — so the cell's
*result* was never in question, only its record.

**What this retracts:** every statement that `rec+alif` at h512 "computes a
non-finite gradient on 420 of 640 optimizer steps". It does not. Correct
statement: **its gradient norm reaches 7.36e29**, roughly 27 orders of magnitude
above the healthy `ff+alif` value of ~0.15.

**What survives unchanged:** seeds 5170002 and 5170003 still abort (§3.6.2).
Those are genuine — the guard is `!sample_gradient.all_finite()` on individual
gradient *entries*, which no norm computation touches. **Zero of three seeds
still produce a usable cell, and RECALIF is still blocked.** The reason is now
"one cell with a 1e29 gradient norm and two hard aborts" rather than "two thirds
non-finite", which is a different sentence with the same consequence.

This is the third time in this document that a measurement of the recurrent arm
turned out to be reporting an instrument defect rather than the arm. It is also
the second time the correction made the arm look *better* than first reported.

### 3.6.2 Result — confirmed, and worse than predicted

| seed | outcome |
|---|---|
| 5170001 | completes; 420/640 non-finite steps; accuracy 0.3507; **looks healthy** |
| 5170002 | **aborts** — non-finite training value at optimizer step 220 |
| 5170003 | **aborts** — non-finite training value at optimizer step 50 |

**Zero of three seeds produce a usable cell at h512.** The prediction was that
both replicates would complete with hundreds of non-finite steps. They did not
complete at all, which is the same conclusion arrived at more bluntly: neither
came back at 0/640, so the falsification condition did not fire.

The two aborts are a *different and more severe* failure than seed 5170001's.
The training loop's guard is

```rust
if !forward.loss.is_finite() || !sample_gradient.all_finite() { return Err(...) }
```

so an abort means an individual gradient **entry** went non-finite, not merely
the f32 norm of the whole vector. Seed 5170001 stayed under that bar for all 640
steps while overflowing the norm on 420 of them — which is why it produced a
plausible-looking cell instead of stopping.

So the h512 failure is not a seed accident, and this is the one prediction in
this document that survived contact with its replicate. The mechanism is a
property of the width: h128 and h256 give 0/640 on this configuration, h512
gives 420/640 or an abort.

**Consequence for `PREREG_2026-08-02_SHD_BPTT_CEILING_RECALIF`: it cannot run as
registered.** Its 24 cells are `rec+alif` at h512. On the evidence here, roughly
two thirds of them would abort and produce no cell, and the remainder would
report accuracies computed from a gradient that is non-finite on most steps.
Before 2026-08-03 the surviving cells would have carried `non_finite_events: 0`
and been indistinguishable from healthy ones (§4b).

That is a blocking result about the *instrument at that width*, not a finding
about recurrence, and it should not be written up as one.

`rec+fixed` at h512 fails differently and more visibly: no non-finite steps, but
a peak norm of 1.47e18 that does not decay, and the worst accuracy of any cell
here (0.1197 against 0.2633 at h128). Width hurts both arms; it just routes
`rec+alif` into overflow and `rec+fixed` into merely astronomic.

**This supersedes the reassurance in §3.5.** "The excursions decay after epoch 3"
is true at h128 and false at the width the campaign actually uses.

## 4. What this does and does not establish

*Rewritten after §3.2. The first version of this section, written from §2 alone,
said the contrast showed "adaptation stabilises recurrence" and that the scale
pilot would locate a safe initialisation. The pilot ran and supports neither
cleanly.*

It does **not** show the recurrent arms are unusable. `rec+alif` reaches 0.19-0.25
in 3 epochs, above `ff+alif`'s 0.33 only in the sense that neither is converged,
and its gradient norm is bounded at two of the five scales tested.

It does **not** show that a good initialisation scale exists. That was the
hypothesis this document was written to test, and §3.2 falsifies it in the range
swept: the response is non-monotonic across three orders of magnitude and the
worst cell in the sweep is at the smallest scale.

It **does** show that the recurrent arms cannot currently be run as a campaign
without an explosion check, and that the check cannot be "pick a better scale".
Of ten pilot cells, eight have a peak gradient norm above `1e4` and two are still
growing at the final epoch.

The mechanistic story — adaptive thresholds rise with recent firing and suppress
runaway recurrent excitation, which is a standard motivation for ALIF in
recurrent spiking networks — remains plausible and is *consistent* with
`rec+alif` being the only arm with any bounded cells. It is not established by
this data, and the `rec+alif` explosion at scale 0.05 is evidence against reading
it as a guarantee.

Separately, the pilot was **previously impossible to run**: the initialiser
hard-coded the Glorot scale under a comment saying the registered value "is set
by the G8 pilot, not here". `--w-rec-scale` now exposes it, defaulting to 1.0 so
every existing file reproduces byte-for-byte (verified in §3.1).

## 4b. Three instrument defects, found by trying to replicate

These are independent of the science above and outlast every revision to it.

**1. `non_finite_events` was never incremented.** It was declared on
`TrainDiagnostics`, read by the pass predicate

```rust
let scientific = evaluation.accuracy >= 0.80 && ... && diagnostics.non_finite_events == 0;
```

emitted into all 296 completed cells — and no code path ever wrote to it. The
clause was **vacuous**: it could not be false. The instrument's only guard
against numerical failure had never done anything. Now incremented when a step's
gradient norm or update RMS is non-finite; the `rec+fixed` e20 cell reports 2,
where it would previously have reported 0.

**2. `json_f64` wrote `inf`.** Not valid JSON, so a cell that diverged produced a
file no consumer could read — losing precisely the record that would explain the
divergence. `json_f32` had guarded this since it was written; `json_f64` and the
scalar `{:.9}` summaries had not. Both now emit `null`, which is what JSON has
for "no value". Finite values format identically, so no recorded cell moves.

**3. `l2_norm` accumulates in f32** (`shd_matched.rs:166`). Squaring entries near
1e19 overflows to infinity while every individual entry stays finite, which is
exactly why the existing per-sample `all_finite()` guard passed them through.

**Defect 3 is now fixed too, at no cost to the record.** The first reading here
was that it could not be: accumulating in f64 changes `mean_gradient_norm` in the
last ulp for `ff+fixed` and breaks Gate F against all 216 recorded cells, a
change to a registered result rather than a bug fix.

That framing was too pessimistic. The overflow only occurs when the **sum of
squares** exceeds `f32::MAX` while the norm itself does not — so the f64 path is
reachable only from values that were already infinite, and **every recorded cell
has a finite `mean_gradient_norm`**. Gating the widening on `!sum.is_finite()`
therefore leaves every representable value bit-identical and replaces only the
broken ones. Gate F: **13/13 PASS** on the rebuilt binary. Registered in
`AMENDMENT_2026-08-03_L2_NORM_CONDITIONAL_WIDENING.md`.

**It corrects the record, not the dynamics.** Only h512 seed 5170001 changes —
its 420 infinite norms become real numbers. The two seeds that abort do so on
`!sample_gradient.all_finite()`, an individual gradient *entry* going non-finite,
which this does not touch. §3.6.2 stands unchanged.

**Verified against the recorded cells.** `non_finite_events` is one of Gate F's
compared fields, so repairing it could in principle have moved 216 results. Gate
F was re-run on the fixed binary (`9ef41e39d81b`): **13/13 bit-identical, PASS**,
with `non_finite_events` still 0 on every `ff+fixed` cell. The counter fires on
the diverging recurrent cell and stays silent on the healthy feed-forward ones,
which is the behaviour wanted. The JSON guards format finite values identically,
so they cannot move a cell either.

## 5. Proposed validity gate — weaker than wanted, and why

*Rewritten after §3.5. The earlier version proposed voiding a cell whose peak
gradient norm exceeded `1e4`. That bound was set from e3 cells and would void
`rec+fixed` at e20 — a cell whose gradient goes infinite in epoch 3 and then
trains cleanly for seventeen more, ending with a monotonically falling loss. A
gate that discards the arm's best evidence is not a gate worth registering.*

**No magnitude threshold survives the transient.** The two candidate rules both
fail on measured cells:

| rule | fails on |
|---|---|
| peak `epoch_max_gradient_norm > 1e4` | voids `rec+fixed` e20, which recovers and learns |
| same rule over the final quarter of epochs | voids `rec+alif` e20 (tail excursions to 1.3e5), the **best** cell measured (0.3785) |

So a bound cannot responsibly be proposed from this data. What can be said is
narrower:

> **Gradient sanity (blocking, post-cell), shape to be determined.** A cell is
> void if it aborts, or if `non_finite_events > 0` *and* the loss is not falling.
> A void cell is not a `CELL_FAIL` result about the architecture; it is a cell
> that did not measure anything.

That much is safe: it voids the 2 of 30 pilot cells that aborted outright, and
does not void a cell that survives an excursion and goes on to learn. It is
deliberately weaker than what is wanted.

**What would settle the real bound.** A handful of `rec+alif` cells at the
campaign budget (e100) and campaign width (h512), looking at whether the
excursions decay as they do for `rec+fixed` at e20 or persist. That is the
measurement missing, and it is the one worth running before RECALIF — not
another scale sweep. It also directly tests the h512 extrapolation, which
nothing here touches.

This remains **unregistered and provisional**.

## 5b. Where the artifacts are

Several pilot runs exist because the harness was repaired between them. Which
directory backs which number:

| directory | binary | what it is |
|---|---|---|
| `w-rec-pilot/` | `32299af8` | first sweep, seed 5170001. **Superseded** — no max telemetry, and the run aborted at the first bad cell, discarding 19 others |
| `w-rec-pilot-v2-s*/` | `9ef41e39` | seed 5170001 complete; seeds 2 and 3 partial — the script crashed on `null` gradient values, which the fix in the same binary had just started emitting |
| `w-rec-pilot-v3-s*/` | `9ef41e39` | **canonical for seeds 5170002 / 5170003.** Tolerates aborts and null-valued cells |
| `lr-confound/` | `9ef41e39` | the e3-vs-e20 comparison in §3.5 |
| `width-check/` | `9ef41e39` | h256 / h512 at e20 |

The §3.2 table is seed 5170001 from `w-rec-pilot-v2`; the three-seed summary
draws seeds 2 and 3 from `v3`.

Two of those directories exist because of harness bugs of mine rather than
anything about the instrument: the first sweep raised on its first failing cell
and threw away the rest, and the second crashed on the `null` values that the
`json_f64` fix had just introduced — on precisely the cells the resilience fix
was written for. A failing cell is data in a diagnostic sweep.

## 6. Caveats

0. **This document was revised three times against its own data, and the
   revisions all went the same way: the early reading was too alarming.** §2 was
   written from e3 cells and said `rec+fixed` "does not learn"; §3.5 shows it
   reaches 0.2633 at e20. §3.2 concluded no initialisation scale is stable; it
   swept only e3 cells, which sit entirely inside the transient. §5 proposed a
   `1e4` gradient bound; that bound voids the arm's best cells. Read §3.5 and
   §4b before quoting anything above them. What survived every revision is §4b —
   the three instrument defects — and those were found by trying to replicate,
   not by the original measurement.
1. **h128 only, and 3 or 20 epochs.** Nothing here is a quantitative claim. The
   campaign this matters for runs h512/e100: 16x the recurrent fan-in and 5x the
   budget, neither of which is tested. §3.2's non-monotonicity is what chaotic
   trajectory variation produces, and the three-seed replication confirms the
   scale ranking does not survive reseeding.
2. **Every cell is `CELL_FAIL` and that is expected**, since the registered
   threshold is `accuracy >= 0.80`. The failures are not the finding; the
   gradient norms are.
3. **Mean, not max.** See §3.3 — pervasive instability and a few pathological
   samples are indistinguishable in what is recorded.
4. **The 1e4 bound in §5 is unregistered and provisional**, and the pilot removed
   the method that was supposed to set it.
5. **The stabilisation story in §4 is an interpretation**, not a measurement, and
   §3.2 weakened it.
6. **The scale sweep is coarse and one-dimensional** — five points on a single
   axis, holding the RNG lineage fixed. It does not rule out that some scale
   between the points tested, or a different initialisation *family* (orthogonal,
   spectral-radius-normalised) rather than a rescaled Glorot, behaves well.
   "Rescaling Glorot does not fix it" is the claim; "no initialisation fixes it"
   is not.
