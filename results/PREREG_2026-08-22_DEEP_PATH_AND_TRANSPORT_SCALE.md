# Preregistration — repairing the deep credit path and the e-prop transport scale

**Registered:** 2026-08-22, **before either repair and before any post-repair
number exists.** Authorised by the maintainer, who asked for both residuals to be
closed rather than left pinned.

Follows `PREREG_2026-08-22_SILENT_INITIALISATION_REPAIR.md`, which repaired the
*input* layer of these same modules and left two residuals open by name. This
document closes both.

---

## 1. Residual A — the deep path is silent above layer 1

The earlier repair raised `MatchedDeepGradient`'s `in_scale` from `0.5` to `8.0`
and fixed depth 1. Depth ≥ 2 stayed at exactly 0.5000 and was pinned as an
unexplained residual. It is no longer unexplained.

Measured on the module's own separable fixture, seed 7, width 8, at
initialisation — layer spike totals for one positive and one negative example:

| depth | layer 0 (pos / neg) | layers ≥ 1 | last-layer class separation | logits |
|---|---|---|---|---|
| 1 | 4 / 9 | — | 5.0000 | −0.031, −0.088 |
| 2 | 4 / 9 | **0 / 0** | **0.0000** | 0.0, 0.0 |
| 3 | 4 / 9 | **0 / 0** | **0.0000** | 0.0, 0.0 |
| 4 | 4 / 9 | **0 / 0** | **0.0000** | 0.0, 0.0 |

Layer 0 spikes and separates the classes cleanly. **Every layer above it emits
zero spikes at initialisation**, so the readout reads a zero rate vector and both
classes produce a bit-identical logit. Training does not escape: at 200 epochs
the last layer's class separation is still exactly `0.0000` at depths 2–4 and the
logits are still bit-identical.

The cause is the same one the previous preregistration named, one layer up. The
inter-layer signal is a rate, `s[l−1][i] / T ∈ [0, ~0.15]`, but `w_hh` is
initialised at `h_scale = 0.3/√n_prev` — a scale for a unit-variance input. The
realised drive is roughly an order of magnitude below threshold, so the same
circular trap applies: **no spikes → vanishing eligibility → no weight growth →
no spikes.** The earlier repair fixed `in_scale` and left `h_scale` untouched.

## 2. Residual A extends to the treatment arm — the comparison, not just the ceiling

`deep_snn_scaling` compares `MatchedDeepGradient` (ceiling) against
`MatchedRlDeepLearnedFb` / `MatchedRl3LayerLearnedFb` / `MatchedRl4LayerLearnedFb`
(treatment). **The treatment arms carry the same defect and one more.** Measured
on the same fixture, seed 7:

| width | layer 1 (pos / neg) at init | layer 2 at init | after 200 epochs |
|---|---|---|---|
| 16 | **0 / 0** | **0 / 0** | acc 0.5000, layer-2 separation 14, logits 0.117 / 0.071 |
| 256 | **0 / 0** | **0 / 0** | acc 0.5000, layer-2 separation **735**, logits 8.339 / 5.326 |

Two distinct defects are visible here:

1. **Silent initialisation**, as in the ceiling. The deep treatment arms still
   carry `in_scale = 0.5` — the value the earlier repair replaced in the ceiling —
   *and* the same `h_scale = 0.3/√n_prev`.
2. **No readout bias.** The deep treatment arms compute `logit = Σ w_out[j]·rate`
   starting from a literal `0.0`; there is no bias term in the struct, the
   training step or the evaluation. Their own depth-1 sibling has one
   (`MatchedArch::by`, updated in `train_minibatches_learned_fb`), and so does the
   ceiling (`MatchedDeepGradient::by`). Because every rate feature is
   non-negative, a readout with no bias has its decision boundary pinned at the
   origin. At width 256 the hidden representation separates the classes by **735
   spikes** and the arm still scores exactly 0.5000, because both logits land on
   the same side of zero.

**Consequence for the record.** `deep-snn-scaling` v134/v135 compared a broken
ceiling against a broken treatment. Its depth-collapse finding is already
withdrawn (`PAPER_DRAFT.md`, `PUBLISHABLE_CLAIMS.md` item 18); this identifies the
cause. Repairing only the ceiling would be **worse than repairing neither**: it
would produce a valid reference standing over a treatment that cannot express a
decision boundary, and the resulting "learned feedback fails at depth" would be an
artifact with no warning label. Both sides are therefore repaired together, or
neither is.

## 3. Residual B — the e-prop transport scale

`ShdDfa` transports through a **frozen** feedback matrix `B`, initialised at
`shd_out_scale(h) = 0.2/√h`. `ShdEpropCeiling` transports through `wout`, which
**is trained**. Parity was established at initialisation and then drifts as `wout`
grows: measured after the silent-initialisation repair, the hidden-modulator RMS
ratio is **5.08**, against a tolerance of `MODULATOR_PARITY_TOLERANCE = 3.5`.

While the hidden layer was silent the ratio was **1.03** — both modulators were
driven by the same sub-threshold surrogate values, so the guard could not fail
whatever the rules did. The repair did not create the violation; it exposed it.

## 4. The repairs

### A. Deep path

1. `MatchedDeepGradient::new` — raise `h_scale` so every hidden layer is inside
   the activity band at initialisation.
2. The three deep treatment arms — raise `in_scale` and `h_scale` by the same
   rule, and **add a trained readout bias**, matching both their depth-1 sibling
   and the ceiling.

The scale constants are chosen by the **activity-band rule alone**, stated here
before the sweep is run: pick the smallest value on a fixed geometric ladder whose
initial mean firing rate lies inside `[ACTIVITY_MIN, ACTIVITY_MAX] = [0.001,
0.500]` at every depth in 1..=4 and at both widths 16 and 256. **Accuracy is not
an input to this choice.** The ladder and the realised rates are reported whatever
they show.

The readout bias is a structural change, not an operating point, and is declared
as such: it restores a degree of freedom the arm's own depth-1 sibling already
has. No learning rule, threshold, surrogate, reset or optimiser is touched.

### B. Transport scale

`ShdEpropCeiling` gains a `normalise_transport` mode, **on by default**, that
rescales the transport matrix to the same RMS as `ShdDfa`'s frozen feedback,
`shd_out_scale(h)`, before transporting. This removes exactly the identified
artifact — growth of the transport matrix norm — and nothing else. The modulator
stays `δ_i = Σ_k wout[k,i]·δ_k` in direction and stays data-dependent; only the
matrix norm is pinned to the scale the comparison was built on.

`with_raw_transport()` preserves the unnormalised rule so the pathology stays
demonstrable, mirroring `MatchedDeepGradient::with_raw_transport`.

**`MODULATOR_PARITY_TOLERANCE` stays at 3.5.** The repair is required to satisfy
the existing tolerance; the tolerance is not moved to fit the repair.

## 5. Registered acceptance criteria, fixed before either repair

| id | criterion | bar |
|---|---|---|
| **D-1** | no layer is silent at initialisation | mean firing rate in `[0.001, 0.500]` for **every** layer, every depth 1..=4, both widths 16 and 256, in the ceiling and in all three deep treatment arms |
| **D-2** | the already-repaired depth-1 result is not lost | `MatchedDeepGradient` depth 1 ≥ 0.90 (prereg F-3), `MatchedGradient` plain reference > 0.99 (unchanged) |
| **D-3** | the deep ceiling learns its own separable fixture | `MatchedDeepGradient` accuracy ≥ 0.90 at **every** depth 1..=4 |
| **D-4** | the deep treatment readout can express a boundary | a trained bias exists in all three deep arms, and each produces more than one distinct predicted class on the fixture |
| **D-5** | the repair is applied in one shared place per arm | verified by inspection and pinned by a test that fails if a deep arm's init scales diverge from the ceiling's |
| **E-1** | modulator parity holds after training | `ratio(ShdDfa, ShdEpropCeiling) ≤ 3.5`, the **existing** tolerance |
| **E-2** | the parity guard is not vacuous | with `with_raw_transport()` the same measurement **violates** 3.5, and the modulator differs across different data |
| **E-3** | no untouched arm changes | `ShdBroadcastPm1`, `ShdRlReinforceFb`, `ShdSuperSpikeCeiling` produce bit-identical accuracies before and after |

## 6. Named outcomes

- **D-1…D-5 all hold** → the deep path is repaired. `deep-snn-scaling` is
  re-run at a new protocol version under §7, and the residual is closed in the
  record.
- **D-1 holds, D-3 fails** → the deep stack spikes but still cannot learn. The
  repair is kept only if D-2 holds, the residual is re-scoped to the *learning*
  path rather than the initialisation, and no depth claim is made.
- **D-2 fails** → the change is **reverted**. Breaking the repaired depth-1
  result to reach depth 2 is not progress.
- **E-1 holds and E-2 holds** → the transport scale is repaired and the guard is
  proven able to fail.
- **E-1 holds but E-2 fails** → the fix has made the guard vacuous. It is
  **rejected**, whatever the ratio reads, and the residual stays open.
- **E-3 fails** → the change has leaked into arms it must not touch; revert and
  re-scope.

## 7. Outcomes for the re-run depth experiment, registered before it runs

Only if D-1…D-5 hold. `deep-snn-scaling` is re-run at **protocol v136**, with
`REQUIRED_SEEDS = 20` and `ACCURACY_FLOOR = 0.65` **unchanged**, and
`CeilingHealth` deciding harness validity as it does today. There is no
directional theory here, so the outcome is registered two-sided:

- **Treatment tracks its depth-matched ceiling at every depth** (mean gap ≤ 0.05,
  ceiling healthy at every depth) → no depth penalty for learned feedback is
  detected on this task.
- **Treatment falls below its ceiling and the gap grows with depth** → a depth
  penalty for learned feedback, measured for the first time against a valid
  reference.
- **Treatment tracks its ceiling but both are below the 0.65 floor** → the task
  has no depth structure to exploit. This is the outcome the experiment's own
  header already warns about (`CoincidenceTask` has `N_IN = 2`), and it is
  reported as a negative result about the *task*, not about feedback alignment.
- **`CeilingHealth` reports anything other than `Ok` at any depth** → the suite
  stays withdrawn and no depth verdict is issued.

## 7a. Amendment, registered 2026-08-22 before the full run

Three things changed between registering §7 and running it. All three are recorded
here **before any full-run number exists**; only the `--quick` pilot (n=5,
hidden=64, 20 epochs) had been run, and only on the ceiling-side questions below.

### The instrument is replaced, not repaired

§5's outcome *"D-1 holds, D-3 fails"* fired exactly as written. With
`DEEP_HIDDEN_SCALE = 9.6` every layer is inside the activity band at every depth
and both widths (min rate 0.039, max 0.097 — the smallest rung of the registered
doubling ladder that qualifies), and `MatchedDeepGradient` **still scores 0.5000
at depths 2–4**. The mechanism is now visible and is not the initialisation:

| depth | class separation at init | after 200 epochs |
|---|---:|---:|
| 2 | layer 1 = 5 | **0** |
| 3 | layer 1 = 5, layer 2 = 6 | **0, 0** |

The class signal reaches every layer at initialisation and **training destroys
it**. The layers saturate to identical, class-blind patterns (units pinned at 6
spikes or 0, for both classes). The eligibility is sign-definite — the
inter-layer code is a non-negative rate and the trace never goes negative — so a
hidden unit can learn only a scalar gain on its whole input, which then runs away
with no weight decay. That is a defect in the credit rule and the code, not in the
operating point, and it is not reachable by any initialisation.

A further finding, registered here because it weakens an earlier claim of mine:
**F-3 was seed-lucky.** `MatchedDeepGradient` at depth 1 scores 1.0000 at seed 7
and 29 but **0.5000 at seeds 3 and 11**. The depth-1 repair recorded in
`RESULT_2026-08-22_SILENT_INITIALISATION_REPAIR.md` was validated on one seed and
does not hold across seeds. Depth 1 is unaffected by `DEEP_HIDDEN_SCALE` (there
are no hidden-to-hidden weights at depth 1), so this was already true before this
change; it was simply never checked.

So `deep-snn-scaling` v136 runs on `binn_learn::shared_bptt`, which was written in
this workspace as the validated replacement for exactly this ceiling and **had no
callers**. It supplies a genuinely shared forward, an explicit readout bias, exact
reverse-mode gradients, and a matched feedback-alignment treatment. The
`MatchedDeepGradient` / `MatchedRl*LearnedFb` pair is retired from this experiment
and kept only as the pinned characterisation of a withdrawn result.

`DEEP_HIDDEN_SCALE` is kept at 9.6 under §6's rule (D-2 holds at the registered
seed). Keeping it makes the retired module's remaining defect honest: it now
demonstrably spikes at every layer **and still fails**, which localises the defect
to the learning rule rather than leaving it hidden behind silence.

### The optimiser is matched at Adam, and the choice read the ceiling only

`shared_bptt` offers an SGD-matched pair (`train_learned_feedback` /
`train_bptt_sgd`) and an Adam ceiling (`train_bptt`). The SGD pair is only useful
where SGD can train the architecture at all, and on this stack it cannot. Pilot,
every rung of the registered ladder `{1e-3, 3e-3, 1e-2, 3e-2, 1e-1}`:

- depth ≥ 2: the **ceiling** sits at exactly 0.5000 at every rung;
- the Adam ceiling reaches 0.9000 and 0.9133 at depths 3 and 4 on the same data.

A reference that cannot learn bounds nothing, so an SGD-matched comparison would
measure the optimiser's failure and nothing else. `train_learned_feedback_adam`
was therefore added to `shared_bptt` — the same feedback treatment under the same
Adam, composed from the module's existing tested pieces — so the headline pair is
optimiser-matched **at the optimiser that works**, differing only in whether the
gradients are true or feedback-projected.

**The selection read the ceiling arm only, never the treatment**, on the same
principle as §4's step-size rule. Adam runs at the module's frozen `ADAM_LR`; no
hyper-parameter was tuned on either arm. The full SGD ladder is reported for both
arms in every run, so the reader can see what the optimiser choice excluded.

### What did not change

`REQUIRED_SEEDS = 20`, `ACCURACY_FLOOR = 0.65`, `CHANCE = 0.5`, and
`CeilingHealth` deciding harness validity. The four named outcomes in §7 stand
exactly as registered, and the Adam ceiling is the reference they are read
against.

## 8. What this may not claim

- **It does not revive v134 or v135.** Those numbers came from the broken pair.
  A re-run under v136 is a new experiment with its own preregistration, not a
  vindication of a withdrawn one.
- **It does not touch any recorded cell.** The bit-identity gate regresses the
  `shd-instrument` binary; none of the repaired types is reachable from it, so
  Gate F must stay 10/10 bit-identical across this change. That is checked, not
  assumed.
- **It does not affect the calibration matrix.** `matrix_authorized` is `false`
  and stays false; nothing here moves `SHD_INSTRUMENT_STATE`.
- **It is a provenance event for three more binaries.** Every number previously
  produced by `deep-snn-scaling`, `shd-scientific-sweep` and `c1 --shd-cal` came
  from the pre-repair operating point and is not comparable with anything
  produced after it.
