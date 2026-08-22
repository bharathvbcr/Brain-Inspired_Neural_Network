# `ShdEpropCeiling` diagnosed — a constant predictor, by a *different* mechanism

**Found:** 2026-08-22, closing the last of the three references found at chance.
**Companion to** [`FINDING_2026-08-22_MATCHED_DEEP_GRADIENT_COLLAPSES_TO_SILENCE.md`](FINDING_2026-08-22_MATCHED_DEEP_GRADIENT_COLLAPSES_TO_SILENCE.md).

---

## 1. The diagnosis

| | |
|---|---|
| It predicts **one class for every sample** | `distinct preds 1`, `majority 60/60`, in **every** configuration tested |
| Its "accuracy" is the class balance | 0.1000 / 0.2000 / 0.2667 across three configurations, tracking the majority-class frequency exactly |
| The data and the forward pass are **fine** | `ShdSuperSpikeCeiling` reaches **1.0000** on the identical fixtures |
| It is not fixture ambiguity | identical result on the module's overlapping fixture and on a disjoint reconstruction of the sweep's |

`shd-scientific-sweep`'s reported **0.2140 against a chance of 0.2000** was never a
weak measurement. It was a constant predictor whose single class happened to hold
21.4% of that test set.

## 2. Same symptom, different mechanism — which is why it needed its own look

`RESULT_2026-08-20_DEEP_SNN_V134_CEILING_IS_AT_CHANCE.md` §"Not claimed" said
explicitly that a shared symptom is not a shared diagnosis. That was right:

| | `MatchedDeepGradient` | `ShdEpropCeiling` |
|---|---|---|
| symptom | constant predictor | constant predictor |
| hidden activity after training | **zero spikes** | present |
| credit signal | frozen readout, `logit == bias` | **modulator non-zero and data-dependent** (1.8867e-2 / 1.8727e-2 / 1.8520e-2 across seeds) |
| mechanism | **activity collapses to silence** | **credit flows and the readout collapses anyway** |

Had the first diagnosis been generalised, this one would have been wrong.

## 3. `ShdDfa` collapses too — and configuration does not explain it

An earlier revision of this section recorded DFA's collapse as *"not evidence that
`ShdDfa` is broken"*, because `shd-scientific-sweep` reported it at 1.0000 under a
different configuration. **That caution is now discharged by measurement.**

Varying width alone, on the sweep's own disjoint fixture construction:

| hidden | 16 | 24 | 32 | 48 | **64** | 96 | 128 |
|---|---|---|---|---|---|---|---|
| `ShdDfa` | 0.1000 | 0.1000 | 0.1000 | 0.1000 | **0.1000** | 0.1000 | 0.1000 |
| distinct predictions | 1 | 1 | 1 | 1 | **1** | 1 | 1 |

Including **the sweep's own `hidden = 64`**. Then varying budget alone at that width:

| epochs | 1 | 2 | 5 | 10 | 20 | **30** | 60 |
|---|---|---|---|---|---|---|---|
| `ShdDfa` | 0.1000 | 0.1000 | 0.1000 | 0.1000 | 0.1000 | **0.1000** | 0.1000 |
| `ShdEpropCeiling` | 0.1000 | 0.1000 | 0.1000 | 0.1000 | 0.1000 | **0.1000** | 0.1000 |
| `ShdSuperSpikeCeiling` | 0.1000 | 0.1000 | 0.1000 | 0.1000 | **1.0000** | **1.0000** | **1.0000** |

Including **the sweep's own 30 epochs**. Every arm starts at the same constant
predictor; **SuperSpike escapes it by epoch 20 and the two local arms never do**,
at any width or budget tested.

**So width and budget are ruled out, and both local arms in this module collapse
to constant predictors on a fixture a working reference solves.**

### What is still not explained

The sweep's reported DFA `1.0000` cannot be reconciled, because
`shd-scientific-sweep` **cannot be re-run** — it is refused by
`authorize_campaign(LocalLearning)` while the instrument is `Uncalibrated`, and
its report came from an older binary. The remaining differences are the example
counts (100/50 vs 120/60) and the arm seed. **This is an honest terminus, not a
resolution**: what is established is the behaviour of the current code, which is
what any future use of it would get.

It also makes the sweep's withdrawal firmer. That report was already void for
running on synthetic data labelled as SHD; its headline DFA number is now doubly
suspect.

## 4. What was changed

**Nothing in the implementation.** Three characterization tests
(`shd_eprop_baseline.rs`) **assert the broken behaviour**:

- `defect_the_eprop_ceiling_is_a_constant_predictor`
- `defect_is_localised_superspike_solves_the_same_fixtures`
- `defect_credit_flows_yet_the_readout_still_collapses` — which also fails if the
  mechanism ever becomes the *silence* one, so the two defects cannot be conflated
  later;
- `defect_neither_local_arm_escapes_the_constant_predictor` — across three widths;
- `defect_superspike_escapes_the_same_fixture_by_epoch_twenty`.

A repair makes these fail. That is the intended signal: it must be registered and
the record updated, not slipped in under a green build.

## 5. Why this matters more than the sweep did

`shd-scientific-sweep` is withdrawn — it never loaded SHD
(`DEFECT_2026-08-20_SHD_SWEEP_IS_SYNTHETIC.md`), so a defect visible only there
would be a curiosity. **But `ShdEpropCeiling` is also constructed in
`binn-lab/src/runner_shd_cal.rs:145`, the calibration runner.** Any comparison
that used it as a ceiling on that path was comparing against a constant predictor.

**No published claim is affected**, because the calibration matrix is not
authorised — `matrix_authorized` is `false` and always has been. This is a defect
found *before* it could reach a result, which is the order these things are
supposed to happen in.

## 6. Scope

- **Verified:** every row in §1 and §2, this session, by test, across three
  configurations and two fixture constructions.
- **Verified:** the `runner_shd_cal.rs` call site, by grep.
- **Not verified:** *why* the readout collapses while credit flows. Localised to
  the local arm rather than the forward or the data; not narrowed further.
- **Not explained:** the sweep's reported DFA 1.0000 (§3). The sweep cannot be
  re-run, so the discrepancy is recorded rather than resolved.
- **Not fixed.**
