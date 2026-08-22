# Repair result — the silent initialisation, and what it exposed

**Prereg:** `PREREG_2026-08-22_SILENT_INITIALISATION_REPAIR.md`, registered before
the repair and before any post-repair number existed.

---

## 1. Registered criteria

| id | criterion | outcome |
|---|---|---|
| **F-1** | hidden layer not silent at init, rate in `[0.001, 0.500]` | **MET** — 0.087 (`shd_eprop_baseline`), 0.0625 (`matched_deep_gradient`) |
| **F-2** | the working reference is not broken | **MET** — `ShdSuperSpikeCeiling` still 1.0000 at 20 epochs |
| **F-3** | `MatchedDeepGradient` learns its fixture at depth 1 | **MET at the registered seed only** — 0.5000 → **1.0000** at seed 7, but **0.5000 at seeds 3 and 11**. See the correction below. |
| **F-4** | readouts no longer constant | **MET** — `ShdEpropCeiling` and `ShdDfa` both use their whole output layer, at every width |
| **F-5** | no arm silently advantaged | **MET** — one shared constant per module |

## 2. The correction the prereg had to make first

`FINDING_2026-08-22_SHD_EPROP_CEILING_IS_A_CONSTANT_PREDICTOR.md` claimed the two
defects had **different mechanisms**, on the grounds that e-prop's modulator was
non-zero and data-dependent, which was read as "credit flows".

**That was wrong.** Measuring the hidden layer directly: mean firing rate
`0.00000000` untrained *and* after 60 epochs, `sum|w_out|` unchanged at 4.106466,
logits bit-identical across classes. The modulator is non-zero below threshold
because the **surrogate derivative** is non-zero there — not because anything
spiked. Both defects were the same silent initialisation.

Generalising is what I had warned against; the error was reading a proxy instead
of measuring the thing.

## 3. What the repair changed

Only the operating point. No rule, threshold, surrogate, readout or optimiser was
touched.

| module | before | after | initial rate |
|---|---|---|---|
| `shd_eprop_baseline::ShdArch` | `0.35 / √n_in` | `2.8 / √n_in` | 0.000000 → **0.087** |
| `matched_deep_gradient` | `0.5` | `8.0` | 0.000000 → **0.0625** |

The scales were chosen by sweep against the registered activity band, not tuned to
an outcome: 6× gives 0.020, 16× gives 0.454, 24× saturates at 0.81.

## 4. Two results the prereg named in advance, and both happened

### Silence was necessary but not sufficient for the deep path

`MatchedDeepGradient` at depth 1 goes 0.5000 → **1.0000**. Depths 2–4 **still fail**
— and breaking the silence alone was measurably not enough: at 2×–8× the layer
spikes (rate 0.031–0.063) and accuracy stays exactly 0.5000; only 16× learns.

Per prereg §5 this is **kept and documented as a residual, not declared fixed**.
`repaired_at_depth_one_residual_defect_at_greater_depth` asserts depth 1 works and
depths 2–4 do not, so repairing the deep path fails the test and must be
registered.

### The repair exposed a guard that could not fail

`dfa_and_eprop_modulator_scales_stay_comparable` asserted the DFA/e-prop modulator
RMS ratio stays within 3.5. **It was passing vacuously.** With the network silent,
both modulators came from the same sub-threshold surrogate values and the ratio was
**1.03** — the check could not fail whatever the rules did.

Spiking, it is **5.08**, and the cause is real and always was: e-prop transports the
readout (`δ_i = Σ_k wout[k,i]·(p_k − y_k)`), so its modulator grows with `wout` as
the arm learns, while DFA's random feedback is fixed. Now that the arms actually
learn, they step at different rates — exactly the defect this module's header warns
about.

**The tolerance was not widened.** The test now asserts the defect, with the
repair path named (`MatchedDeepGradient`'s `normalise_transport` is the
established remedy) and a note that fixing it changes the e-prop rule's step scale
and needs its own registration.

This is the third vacuous check found this week, and the first one hiding inside a
scientific guard rather than a smoke test.

## 5. What this does *not* do

- **Revives nothing.** `deep-snn-scaling` and `shd-scientific-sweep` stay
  withdrawn. A repaired instrument does not retroactively validate a report
  produced by the broken one.
- **Affects no live claim.** `matrix_authorized` is `false`; the attention
  campaign uses neither type.
- **Is a provenance event.** Every recorded number from these modules came from
  the pre-repair initialisation and is not comparable to anything produced after.

## 6. Verification

623 Rust tests pass, 0 fail · `fmt` clean · clippy `-D warnings` clean ·
GC1–GC7 pass · record checks green (24 tooling tests, 16/16 published numbers).

## 6a. Correction, 2026-08-22 — F-3 was validated on one seed

F-3 above is recorded as MET on the strength of seed 7. It does not generalise:

| seed | depth 1 |
|---|---:|
| 7 (registered) | 1.0000 |
| 29 | 1.0000 |
| **3** | **0.5000** |
| **11** | **0.5000** |

Depth 1 has no hidden-to-hidden weights, so nothing in the later repair caused
this — it was already true when F-3 was recorded and was simply never checked.
The honest reading is *"this module can sometimes solve its own fixture"*, not
*"the module works at depth 1"*. Pinned by
`the_depth_one_repair_does_not_hold_across_seeds` so the record cannot drift back.

No live claim changes: the module is now retired from every experiment.

## 7. Scope

- **Verified:** every number above, this session, by test.
- **Superseded 2026-08-22:** both residuals are closed in
  `RESULT_2026-08-22_DEEP_PATH_AND_TRANSPORT_SCALE.md`. The e-prop transport scale
  was repaired (ratio 5.0810 → 1.8940, tolerance unmoved). The deep path was
  **not** repairable — raising the hidden scale removed the silence at every
  layer and the arm still scored 0.5000 at depths 2–4, which localised the defect
  to the credit rule. `deep-snn-scaling` v136 runs on `shared_bptt` instead, and
  `MatchedDeepGradient` is retired from every experiment.
- **Not verified:** the initial firing rate on **real SHD data**. The band was
  measured on the synthetic fixtures; the real-corpus rate cannot be checked while
  the calibration matrix is unauthorised.
