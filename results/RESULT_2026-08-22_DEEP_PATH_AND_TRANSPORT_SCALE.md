# Result — the deep path and the e-prop transport scale

**Registered:** `PREREG_2026-08-22_DEEP_PATH_AND_TRANSPORT_SCALE.md`, including the
§7a amendment, all before any number below existed.
**Run:** 2026-08-22.

Both residuals left open by `RESULT_2026-08-22_SILENT_INITIALISATION_REPAIR.md` are
closed. One was repaired; the other turned out not to be repairable, and the
instrument was replaced instead.

---

## 1. Criteria

| id | criterion | outcome |
|---|---|---|
| **D-1** | no layer silent at initialisation, every depth 1..=4, widths 16 and 256 | **MET** — min rate 0.0391, max 0.0972 |
| **D-2** | depth-1 result not lost; plain reference unchanged | **MET at the registered seed** — see §4, this is weaker than it looks |
| **D-3** | the deep *ceiling* learns its own fixture at every depth | **NOT MET** — 0.5000 at depths 2–4, and not fixable by initialisation |
| **D-4** | the deep treatment readout can express a boundary | **MET by replacement** — the shared stack has a trained readout bias |
| **D-5** | one shared place per arm, pinned by a test | **MET** — `deep_hidden_scale_is_the_smallest_rung_inside_the_activity_band` |
| **E-1** | modulator parity after training, tolerance **unchanged** at 3.5 | **MET** — 1.8940 |
| **E-2** | the parity guard is not vacuous | **MET** — raw transport measures 5.0810 and still fails |
| **E-3** | no untouched arm changes | **MET** — pinned by `the_transport_rescale_reaches_only_the_eprop_ceiling` |

§5's second named outcome — *"D-1 holds, D-3 fails → the residual is re-scoped to
the learning path rather than the initialisation"* — is the one that fired.

## 2. Residual B: the transport scale is repaired

`ShdEpropCeiling` transported through `wout`, which is trained, while `ShdDfa`
transports through a frozen matrix at `shd_out_scale`. Parity was set at
initialisation and drifted as `wout` grew.

| | modulator RMS ratio, DFA vs e-prop |
|---|---:|
| while the hidden layer was silent (the vacuous era) | 1.03 |
| after the initialisation repair, raw transport | **5.0810** |
| after rescaling the transport matrix to `shd_out_scale` | **1.8940** |

The tolerance was **not moved**. It is the same `MODULATOR_PARITY_TOLERANCE = 3.5`
the guard failed against. `with_raw_transport()` keeps the old rule reachable, and
`defect_raw_transport_outgrows_dfa_feedback_once_the_network_spikes` asserts it
still violates 3.5 — so the repaired guard is demonstrably able to fail, rather
than satisfied by construction. That mattered: normalising the *modulator* to unit
RMS would have driven the ratio to exactly 1 and produced a check that could never
fail, which is the failure mode this workspace keeps finding.

1.8940 is not 1.0. What remains is the genuine difference between transporting a
trained matrix's direction and a frozen random one, which is the thing the
comparison exists to measure.

## 3. Residual A: the deep path is not repairable, and was replaced

`DEEP_HIDDEN_SCALE` was raised from `0.3` to `9.6` — the smallest rung of a
doubling ladder whose initial firing rate is inside `[0.001, 0.500]` at every
layer, depth and width. Accuracy was not an input to that choice, and a test
re-runs the selection against the real constructor, asserting both that 9.6
qualifies and that **every smaller rung does not**.

Silence is gone. The failure is not.

| depth | last-layer class separation at init | after 200 epochs | accuracy |
|---|---:|---:|---:|
| 2 | 5 | **0** | 0.5000 |
| 3 | 6 | **0** | 0.5000 |
| 4 | — | **0** | 0.5000 |

The class signal reaches every layer at initialisation and **training destroys
it**. The layers saturate to identical, class-blind patterns — units pinned at 6
spikes or at 0, the same for both classes — and the logits become bit-identical.

The mechanism: the inter-layer code is a non-negative rate and the eligibility
trace is therefore sign-definite, so for a given post-synaptic unit every incoming
weight moves the same way. The unit can learn a scalar gain on its whole input and
nothing else, and with no weight decay that gain runs away until the layer
saturates. **That is a property of the credit rule and the code, not of the
operating point**, and no initialisation reaches it.

So the suite was moved to `binn_learn::shared_bptt`, which was written in this
workspace as the validated replacement for exactly this ceiling and **had no
callers**. `train_learned_feedback_adam` was added to it so the treatment and the
ceiling share an optimiser; everything else it needed already existed and was
tested.

`DEEP_HIDDEN_SCALE` is kept at 9.6. The retired module now demonstrably spikes at
every layer **and still fails**, which localises its defect to the learning rule
instead of hiding it behind silence.

## 4. A correction: the depth-1 repair was validated on one seed

`RESULT_2026-08-22_SILENT_INITIALISATION_REPAIR.md` reports criterion F-3 met —
`MatchedDeepGradient` at depth 1 scoring 1.0000 against 0.5000 before. That number
is real, and it does not generalise:

| seed | depth 1 |
|---|---:|
| 7 (the registered seed) | 1.0000 |
| 29 | 1.0000 |
| **3** | **0.5000** |
| **11** | **0.5000** |

Depth 1 has no hidden-to-hidden weights, so `DEEP_HIDDEN_SCALE` cannot have caused
this — it was already true and was never checked. **F-3 was seed-lucky.** The
depth-1 repair should be read as "this module can sometimes solve its own fixture",
not "the module works at depth 1". It changes no live claim, because the module is
now retired from every experiment, but the earlier result overstates what was
established and this record corrects it.

## 5. The replacement measures the thing the old suite could not

`deep-snn-scaling` **v136**, n=20 seeds, hidden 128, 60 epochs, `ACCURACY_FLOOR`
and `REQUIRED_SEEDS` unchanged. Treatment and ceiling share one forward graph, one
initialisation, one optimiser and one step size, and differ only in whether the
gradients are true or feedback-projected.

| depth | treatment | SE | ceiling | SE | gap | input modulator RMS | ceiling health | verdict |
|---:|---:|---:|---:|---:|---:|---:|---|---|
| 1 | 0.9920 | 0.0009 | 0.9945 | 0.0011 | −0.0025 | 2.648e-1 | ok | PASS |
| 2 | 1.0000 | 0.0000 | 1.0000 | 0.0000 | +0.0000 | 5.620e-3 | ok | PASS |
| 3 | 0.9740 | 0.0022 | 1.0000 | 0.0000 | −0.0260 | 2.976e-1 | ok | PASS |
| 4 | 0.9780 | 0.0052 | 1.0000 | 0.0000 | −0.0220 | 3.914e-1 | ok | PASS |

Every ceiling is healthy and every gap is inside the registered 0.05 tolerance.
§7's **first** registered outcome fires: *no depth penalty for learned feedback is
detected on this task.*

**A first draft of this section said "the gap does not grow with depth". That
overstates the table.** From depth 1 to depth 4 the gap moves −0.0025 → −0.0220,
so it grows by 0.0195 — a fifth of the way to the tolerance, on four points. What
is true, and all that is claimed, is that it stays inside the registered
threshold. Whether a trend exists is not answerable from four depths at this
precision, and is not asserted.

The optimiser choice is documented rather than assumed. The full registered SGD
ladder, both arms, every depth: depth 1 reaches 1.0000 at `lr = 1e-1`, and **every
rung at depths 2–4 leaves both arms at exactly 0.5000**. A reference that cannot
learn bounds nothing, which is why the headline pair is matched at Adam.

## 6. What this may not claim

- **It does not revive v134 or v135.** Those are withdrawn and stay withdrawn.
  Nothing here is comparable with them; the arms are different types.
- **It is weak evidence about depth, and the ceiling says so.** The ceiling reaches
  **exactly 1.0000** at depths 2–4. A saturated reference has no headroom, so
  "the treatment tracks its ceiling" is close to "both arms solved an easy task".
  `CoincidenceTask` has `N_IN = 2`; the negative depth result is a statement about
  this task, not about deep credit assignment. Moving the suite to an input-rich
  task remains open work.
- **The depth-1 identity is structural, not evidence.** At depth 1 the learned
  feedback aligns to the readout, so the treatment *is* the true gradient there.
  The 0.9920 / 0.9945 agreement is a consistency check, not a finding.
- **It does not touch the calibration matrix.** `matrix_authorized` is false and
  stays false; `SHD_INSTRUMENT_STATE` is untouched.
- **It is a provenance event for three binaries.** Every earlier number from
  `deep-snn-scaling`, `shd-scientific-sweep` and `c1 --shd-cal` came from the
  pre-repair operating point and is not comparable with anything produced now.

## 7. Bit-identity

None of the changed types is reachable from `shd-instrument`, which is the binary
`scripts/gate_f_rust.py` regresses. Verified by import graph and by running the
gate — see §8 of the summary. The 216 recorded cells are unaffected.

## 8. Artifacts

- Report: `results/deep_snn_scaling_v136.md`
- Ceiling and treatment: `binn-learn/src/shared_bptt.rs`
- Retired pair, kept as pinned characterisation:
  `binn-learn/src/matched_deep_gradient.rs`, `binn-learn/src/matched_rl_baseline.rs`
- Transport repair: `binn-learn/src/shd_eprop_baseline.rs`
