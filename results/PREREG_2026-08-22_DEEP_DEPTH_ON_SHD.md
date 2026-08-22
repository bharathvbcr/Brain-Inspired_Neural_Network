# Preregistration — does learned feedback alignment track its ceiling at depth, on a task with headroom?

**Registered:** 2026-08-22, **before any accuracy from `shd-depth-scaling` exists,
and before the campaign has been permitted to run at all.** The only numbers that
existed at registration time are the harness-validation measurements in §4 —
initialisation firing rates and per-example cost — which involve no training and
no accuracy, and which are reported here rather than withheld.

Follows `PREREG_2026-08-22_DEEP_PATH_AND_TRANSPORT_SCALE.md` and its result,
`RESULT_2026-08-22_DEEP_PATH_AND_TRANSPORT_SCALE.md`, which closed the deep-path
residual by replacing the instrument and then said plainly what the replacement
could not establish. This document opens the piece it named as open work.

---

## 0. This campaign is refused by the authorization gate, and that is the correct outcome

`shd-depth-scaling run` trains a non-gradient credit rule on SHD and reports its
accuracy. It is therefore `CampaignKind::LocalLearning`, which
`binn_lab::authorize_campaign` refuses while `SHD_INSTRUMENT_STATE` is
`Uncalibrated`. Verified, not assumed:

```
$ ./target/release/shd-depth-scaling run --quick
SHD instrument is UNCALIBRATED; blocked local-learning campaign. Only calibration,
parity, and harness-validation work is authorized. See results/SHD_INSTRUMENT_STATUS.md
$ echo $?
2
```

`SHD_INSTRUMENT_STATUS.md` blocks "new SHD local-learning or architecture-ablation
campaigns" outright. The siblings `shd-arch-ablation`, `shd-frozen-attention`,
`shd-scientific-sweep`, `shortcut-accessibility-contrast`,
`temporal-deep-campaign` and `temporal-eligibility-diagnostic` are refused
identically; `transfer-falsifier` and `temporal-optimizer-control` are refused
under their own kinds. All nine — the six local-learning siblings, the two
refused under their own kinds, and this new one — are now pinned by
`binn-lab/tests/campaign_gate_refuse.rs`, which spawns each real binary and
requires both a nonzero exit **and** the gate's own message, so a binary that
exits nonzero for an unrelated reason cannot pass by accident. Before this file
nothing checked the binaries at all: `instrument_status`'s own test exercises
`authorize_campaign` as a function, which stays green whether or not any binary
calls it.

`SHD_INSTRUMENT_STATE` is a compile-time constant with no flag and no environment
override, by design. **It is not to be flipped to run this.** That constant *is*
the claim that the instrument measures what it says it measures; flipping it
would falsify the work rather than unblock it. The precedent is
`BLOCKER_2026-08-19_FROZEN_ATTENTION_LOCAL_ARM.md`, and this experiment sits in
the same place: implemented, unit-tested, wired, and waiting on calibration
criteria 4 and 5.

This document is registered anyway, and now, because a preregistration written
after the gate opens is worth less than one written before. Nothing below may be
revised once the campaign becomes runnable.

## 1. The question, and why the last instrument could not answer it

`deep-snn-scaling` v136 compared `train_learned_feedback_adam` (treatment) against
`train_bptt` (ceiling) on `binn_learn::shared_bptt` across depths 1–4, and found
the treatment tracking its ceiling at every depth. Its own report withdraws most
of the weight of that finding:

| depth | treatment | ceiling | gap |
|---:|---:|---:|---:|
| 1 | 0.9920 | 0.9945 | −0.0025 |
| 2 | 1.0000 | **1.0000** | +0.0000 |
| 3 | 0.9740 | **1.0000** | −0.0260 |
| 4 | 0.9780 | **1.0000** | −0.0220 |

> A saturated reference has no headroom, so "the treatment tracks its ceiling" is
> close to "both arms solved an easy task". `CoincidenceTask` has `N_IN = 2`; the
> negative depth result is a statement about this task, not about deep credit
> assignment. Moving the suite to an input-rich task remains open work.

The scientific question is unchanged: **does learned feedback alignment track its
depth-matched BPTT ceiling as depth grows?** What changes is that it is asked
where the ceiling has somewhere to fall from. There is no directional theory, so
the outcome is registered two-sided in §7.

## 2. The instrument, and what is new

Reused **exactly as v136 uses it**, unmodified:

* `binn_learn::shared_bptt::train_bptt` — the Adam ceiling, exact reverse mode;
* `binn_learn::shared_bptt::train_learned_feedback_adam` — the matched treatment,
  same optimiser, same frozen hyper-parameters, differing only in whether the
  gradients are true or feedback-projected;
* one `SharedTemporalNet::new` for both arms, so they share a forward graph and
  an initialisation;
* `feedback_modulator_rms`, read per hidden layer at the trained parameters on
  the held-out split, without applying an update;
* `binn_lab::guards::CeilingHealth` as the owner of the "is this reference
  usable at all" question, and `Verdict::evaluate_mean` as the owner of the
  arm verdict. Neither is reimplemented.

New, and only this:

* `binn_lab::shd_dense` — the last hop from a framed SHD utterance to
  `DenseTemporalExample`. The canonical reader `binn_data::read_event_cache` and
  the canonical framing `binn_data::frame_events` are **called**, never copied;
  both are on the `shd-instrument` bit-identity path. The converter is the SHD
  sibling of `binn_lab::samples_to_dense_temporal_examples` and agrees with it on
  layout, label type and the pad/truncate rule.
* `binn-lab/experiments/shd_depth_scaling.rs` — the campaign, plus an
  `activity-probe` command that trains nothing.

Nothing on the Gate F path is touched. `scripts/gate_f_rust.py` regresses the
`shd-instrument` binary; none of the new code is reachable from it. That is
checked rather than assumed — `python scripts/gate_f_rust.py --cheapest 3` was
run against the current binary and reports **3/3 bit-identical**, on exactly the
`fixed-t100 / adjacent-sum-5 / h128 / e20` cells whose accuracies section 5
cites.

## 3. The registered operating point — no free parameter is set by this experiment

| knob | registered value | the owner it is inherited from |
|---|---|---|
| contract | `fixed-t100` (dt = 14 ms) | the 216 recorded cells, `rust__fixed-t100__…` |
| geometry | `adjacent-sum-5`, **140** inputs | the same recorded cells |
| classes | 20, chance 0.05 | `binn_data::SHD_N_CLASSES`, `SHD_CHANCE` |
| timesteps | 100 | the contract's own frame count |
| `alpha` | `exp(-dt_ms / MATCHED_PHYSICAL_TAU_MS)` = 0.2483 | `shd_matched::loss_and_gradient` |
| threshold | `THETA_REST` = 1.0 | `binn_engine::cell` |
| surrogate beta | `DEFAULT_MATCHED_BETA` = 5.0 | `matched_local_baseline` |
| input values | raw event counts, **unscaled** | `shd_matched::loss_and_gradient` |
| optimiser | Adam at `shared_bptt::ADAM_LR` = 1e-3 | `shared_bptt`, frozen |
| feedback rate | 0.01 | v136, unchanged |
| activity band | `[0.001, 0.500]` | `binn_learn::shd_alif` |
| collapse check | distinct predicted classes per arm | `shd_alif::MAJORITY_PRED_MAX` |

**Depth grid:** 1, 2, 3, 4 — the same grid as v136, so the two runs are readable
side by side. **Width:** 128 hidden units at every layer (64 in the pilot), which
is the width of every recorded instrument cell. **Epochs:** 20, matching the
`__e20__` budget of the recorded cells. **Splits:** the first 2000 training and
500 test utterances of the event caches, which are the caps `shd-arch-ablation`
and `shd-frozen-attention` already register. The caches are shuffled, and the
realised class histogram of the evaluation split is reported so that an
unbalanced prefix is visible rather than silent.

**Seeds: `REQUIRED_SEEDS = 12`, not v136's 20.** The reason is cost and is stated
before the fact: from §4, twenty seeds would cost ≈ 18 CPU-hours against twelve
seeds' ≈ 11. Twelve matches `shd-frozen-attention`'s registered seed count on the
same dataset. Because the SHD split is fixed, seeds vary **initialisation only** —
there is no data resampling, so the standard errors reported do not include
sampling variability of the data. That is a real limitation of this design and is
restated in §8 rather than buried.

**No SGD step-size ladder is run.** v136 ran the full registered ladder for both
arms and recorded that at depth ≥ 2 the SGD ceiling sits at exactly chance at
every rung while the Adam ceiling learns. A reference that cannot learn bounds
nothing. That finding is cited rather than re-purchased at ~70× the cost per cell.

## 4. Measured before registering — activity and cost, no training, no accuracy

`shd-depth-scaling activity-probe` requests `CampaignKind::HarnessValidation`,
the same class as `shd-instrument temporal-sensitivity`, and is authorized. It
applies **no parameter update** — `SharedTemporalNet::parameter_fingerprint` is
compared before and after every measurement and the probe fails if it moved — and
computes no accuracy. Cost is a timing, not a result: `scripts/gate_f_rust.py`
excludes `wall_secs` from its compared fields for exactly that reason.

This measurement is made *before* registration deliberately. Two results in this
workspace were withdrawn because a stack was silent above layer 1 and nobody
looked. Registering a depth experiment without knowing whether its layers spike
would repeat that.

**Initialisation firing rate**, hidden 128, 200 SHD utterances, `fixed-t100` /
`adjacent-sum-5`. Layer `L` is read from a depth-`L+1` model, whose first `L+1`
layers are bit-identical to any deeper model's at the same seed — pinned by
`prefix_layers_are_identical_across_depths`.

| layer | mean firing rate | silent fraction | saturated fraction | band `[0.001, 0.500]` |
|---:|---:|---:|---:|---|
| 0 | 0.0953 | 0.0000 | 0.0000 | inside |
| 1 | 0.0074 | 0.0000 | 0.0000 | inside |
| 2 | 0.0066 | 0.0000 | 0.0000 | inside |
| 3 | 0.0066 | 0.0000 | 0.0000 | inside |

**Every layer spikes at every depth, and none saturates.** This is the check that
the two withdrawn deep instruments failed. It is *not* a claim that the deep
layers carry class information after training — that is the failure mode the v136
record traced to the credit rule, and no initialisation measurement can speak to
it.

Note the 14× drop between layer 0 and layer 1: the inter-layer code is a
cumulative rate and is intrinsically an order of magnitude smaller than the raw
event counts the first layer sees. It is inside the band and it is not repaired
here, because repairing it would mean changing `shared_bptt`'s initialisation,
which v136 shares. The number is registered so that it is on the record before
any accuracy exists.

**Cost per example**, forward plus gradient, single-threaded, no optimiser step,
60 examples per cell, two independent runs:

| depth | ceiling (true BPTT), s | treatment (feedback-projected), s |
|---:|---:|---:|
| 1 | 0.0045 / 0.0038 | 0.0028 / 0.0023 |
| 2 | 0.0123 / 0.0100 | 0.0050 / 0.0045 |
| 3 | 0.0151 / 0.0145 | 0.0064 / 0.0060 |
| 4 | 0.0273 / 0.0224 | 0.0093 / 0.0082 |

Registered schedule cost, from the upper of each pair: 0.083 s per example
summed over all eight cells of one seed; 40 000 examples per cell; **≈ 55 CPU-
minutes per seed, ≈ 11 CPU-hours for twelve seeds**, and — with 96 independent
cells on 18 cores, the longest single cell being ≈ 18 minutes — an expected wall
time of **roughly 45–75 minutes**. The pilot (3 seeds, hidden 64, 5 epochs, 400
train / 200 test) is ≈ 3 CPU-minutes, well under a minute of wall time.

## 5. Registered validity gates, fixed before any accuracy exists

Each of these independently voids the depth reading. All three are emitted as a
banner **before any number** in the report, so a reader who stops at the first
table cannot miss them.

| id | gate | bar |
|---|---|---|
| **V-1** | the reference is usable | `CeilingHealth::evaluate(ceiling, treatment, realised majority-class rate)` is `Ok` at **every** depth |
| **V-2** | **the reference has headroom** | ceiling mean ≤ `HEADROOM_MAX = 0.95` at **every** depth |
| **V-3** | the operating point is inside the activity band | initialisation mean firing rate in `[0.001, 0.500]` for **every** layer at every depth |
| **V-4** | neither arm is a constant predictor | every seed's arm predicts more than one distinct class on the held-out split |
| **V-5** | the evaluation split is what it claims | all 20 classes present; realised majority-class rate reported and used as the chance argument |
| **V-6** | power | `n_seeds ≥ REQUIRED_SEEDS = 12`, else the verdict is `Underpowered` |

**V-2 is the gate this experiment exists to add.** v136 had no headroom
requirement, so a ceiling reading exactly 1.0000 still produced a table of gaps.
It is set at 0.95 rather than at some tighter value because it is a defect
detector, not a quality bar — the same reasoning `CEILING_ABOVE_CHANCE_MARGIN`
gives for 0.05. The recorded `fixed-t100 / adjacent-sum-5 / h128 / e20`
instrument cells sit at 0.600–0.627 accuracy on the full split, so a healthy
ceiling here is expected to have ample headroom; V-2 is registered against the
possibility that it does not.

## 6. Registered thresholds

| constant | value | provenance |
|---|---|---|
| `ACCURACY_FLOOR` | **0.25** | five times the 0.05 chance rate of a 20-class task; the bar for citing a **treatment** arm as a positive |
| `GAP_TOLERANCE` | **0.05** | unchanged from v136 |
| `HEADROOM_MAX` | **0.95** | new; see V-2 |
| `REQUIRED_SEEDS` | **12** | §3 |
| chance | 0.05, and the realised majority-class rate as the operative baseline | `guards::CeilingHealth` documents the latter as correct for an unbalanced split |

`ACCURACY_FLOOR` decides only whether an arm is citable as a positive. It does
**not** decide the depth question, which is decided by the gap and its drift.
The 0.234 that this project's DFA arm reached on SHD under a different protocol
is context for why 0.25 is not a trivial bar; it is **not** a comparison, and no
claim of the form "this beats the DFA arm" may be made from it.

## 7. Named outcomes, registered two-sided before the campaign runs

Read in order; the first that applies is the outcome.

- **O-0 — any of V-1…V-6 fails.** No depth verdict is issued, in either
  direction. The report prints the banner and the run is a statement about the
  instrument, not about credit assignment. In particular, **a saturated ceiling
  (V-2) is reported as "this task did not provide headroom either", not as
  "the treatment tracks its ceiling"** — the reading v136 was forced into.

- **O-1 — the treatment tracks its ceiling at every depth.** All gates hold and
  `|treatment − ceiling| ≤ 0.05` at every depth in 1…4 → *no depth penalty for
  learned feedback alignment is detected on SHD*, this time against a reference
  with headroom. This is a negative result about the depth penalty and is
  reported as such; it is not evidence that feedback alignment equals BPTT.

- **O-2 — the gap is negative and grows with depth.** All gates hold, the gap is
  negative at depth 4, and `gap(4) − gap(1) < −0.05` → *a depth penalty for
  learned feedback alignment, measured for the first time against a reference
  that is neither dead nor saturated.*

- **O-3 — the gap is negative but flat.** All gates hold, some depth exceeds the
  0.05 tolerance, but `|gap(4) − gap(1)| ≤ 0.05` → *a constant cost of feedback
  projection, not a depth effect.* No scaling claim may be made.

- **O-4 — the gap is positive beyond tolerance at any depth.** The treatment
  exceeds the reference that is supposed to bound it. `CeilingHealth` already
  classifies this as `Inverted` and V-1 fires, so this lands in O-0; it is named
  separately here so that it cannot later be presented as a finding.

- **O-5 — all gates hold and both arms are below `ACCURACY_FLOOR`.** The
  registered budget — 2000 utterances, 20 epochs, this architecture — is too
  small for either arm to learn SHD, and the comparison is between two arms that
  both failed. Reported as a negative result about **the budget and the
  architecture**, not about feedback alignment, and the gap is not interpreted.

- **O-6 — the modulator RMS collapses with depth.** If the per-layer
  `feedback_modulator_rms` reaching layer 0 falls by more than an order of
  magnitude between depth 1 and depth 4, then any gap is confounded with
  effective step size, and O-1…O-3 are reported **with that caveat attached to
  the headline**, not in a footnote.

## 8. What this may not claim, whatever it produces

- **It may not be compared with `deep-snn-scaling` v136.** Different task,
  different input dimensionality, different class count, different chance rate,
  different data budget. The two share a module, not a measurement. A gap of
  −0.02 here and −0.02 there are not the same quantity.
- **It may not be compared with the 216 recorded instrument cells.** Those are a
  different architecture (`shd_matched_arms`), a different optimiser schedule
  (minibatched one-cycle), a different readout, and the **full** 8156/2264 split.
  The 0.600–0.627 quoted in §5 is a headroom expectation, not a reference.
- **It says nothing about locality.** The treatment is feedback-alignment through
  a learned matrix, not a local rule in the sense Gate G2 asks about.
- **Seeds do not measure data variability.** The split is fixed; twelve seeds
  measure twelve initialisations of one split. A confidence interval from this
  design is narrower than the true uncertainty about SHD.
- **It does not touch the calibration matrix.** `SHD_INSTRUMENT_STATE` is
  untouched and stays `Uncalibrated`. No recorded cell is re-derived, and Gate F
  must stay bit-identical across this change.
- **A run that never happens claims nothing at all.** Until calibration criteria
  4 and 5 are met, the deliverable is a built, tested and refused binary — and
  the correct summary of this experiment is that it has produced no accuracy
  whatsoever.
