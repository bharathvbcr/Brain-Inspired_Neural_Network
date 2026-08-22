# `MatchedDeepGradient` diagnosed — training drives the network silent

**Found:** 2026-08-22.
**Closes the "why" left open by**
[`RESULT_2026-08-20_DEEP_SNN_V134_CEILING_IS_AT_CHANCE.md`](RESULT_2026-08-20_DEEP_SNN_V134_CEILING_IS_AT_CHANCE.md),
which established *that* the depth-matched ceiling sits at chance and explicitly
recorded *why* as unverified.

---

## 1. The diagnosis

| step | evidence |
|---|---|
| It never learns **anything** | on the module's **own separable fixture**, 200 epochs, depths 1–4: accuracy is **0.5000 before and after**, class 1 predicted in **0 / 20** cases |
| The defect is **this implementation** | `MatchedGradient` on **identical data, learning rate, surrogate width and seed** reaches **1.0000** |
| It is **not** the task | the fixture is separable and the module's own doc says so |
| It is **not** depth | depth 1 fails exactly as depth 4 does |
| The mechanism is **activity collapse** | with input drive ×8 the layer spikes at initialisation; after 200 epochs it emits **zero** spikes |
| The consequence | `rate = s[last][j] / T` is then 0, so the readout freezes and `logit == by`. Both classes produce the **bit-identical** logit `-0.012575` — a constant predictor by construction |

**Root cause: training moves the hidden weights into a regime where the network
stops spiking, and a rate readout over a silent layer is a constant.**

## 2. Two wrong hypotheses, both killed by measurement

Recording these because each was plausible and each would have produced a
confident, wrong write-up.

**"The network is silent at initialisation."** True at the default scale — the
depth-1 layer emits 0.0 spikes untrained — and it looks like a complete
explanation. **It is not.** Raising input drive to ×2, ×4, ×8, ×16 produces 2–4
spikes at init and accuracy stays at **0.5000** at every gain. Silence at init is
a *symptom*.

**"A rate readout cannot separate classes that differ only in timing."**
Structurally true of the fixture, and it predicts exactly the observed constant
predictor. **Also not the cause** — the measurement that was supposed to confirm
it instead showed **zero spikes after training even at ×8 drive**, where the same
configuration had spiked at init. The readout never got the chance to fail on
timing; there was nothing to read.

The correct diagnosis only appeared because both were tested rather than
concluded.

## 3. What was changed

**Nothing in the implementation.** `MatchedDeepGradient` is not fixed. Fixing a
scientific reference changes what every comparison against it means, so it needs a
registration and a re-run, not a quiet patch — and `deep-snn-scaling` is
`INVALID_HARNESS` regardless, so there is no live result waiting on it.

**Three characterization tests were added** (`matched_deep_gradient.rs`) that
**assert the broken behaviour**:

- `defect_the_deep_ceiling_never_learns_its_own_fixture`
- `defect_is_localised_the_plain_reference_solves_the_same_fixture`
- `defect_training_collapses_activity_to_zero`

They pin the defect so it cannot silently change. **If someone repairs this type,
these tests fail** — which is the intended signal: the repair must be registered
and the record updated, not slipped in under a green build.

## 4. What this would take to fix, and what it would buy

The failure is in learning dynamics, not in a single line: something in the
transported-delta path (`rms_normalise` followed by re-multiplication by
`delta_out`) or the input-layer update is systematically driving weights toward
silence. Isolating it means instrumenting the weight trajectory per layer per
epoch, which is a debugging project rather than an experiment.

**It is probably not worth doing.** `shared_bptt` already exists as the validated
replacement — `shared_bptt.rs:3` says so explicitly — and it carries tests that
can fail, including `depth_one_bptt_overfits_easy_fixture`. The depth question
should be answered with that instrument, and that instrument is blocked only by
the calibration gate, not by this defect.

## 5. Scope

- **Verified:** every row in §1, this session, by test.
- **Verified:** both refuted hypotheses in §2, by the measurements that refuted
  them.
- **Not verified:** which specific update drives the collapse. Localised to
  `MatchedDeepGradient`'s learning dynamics; not narrowed further.
- **Not claimed:** that `ShdEpropCeiling` — the other reference found at chance —
  shares this mechanism. It has not been diagnosed. Same symptom is not same cause.
- **Not fixed.**
