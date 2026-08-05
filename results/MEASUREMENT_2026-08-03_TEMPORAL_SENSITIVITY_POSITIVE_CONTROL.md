# Positive control: timing reaches the hidden representation under every framing

**Date:** 2026-08-03
**Backend:** rust only.
**Command:** `shd-instrument temporal-sensitivity` (new; `CampaignKind::HarnessValidation`)
**Artifacts:** `results/shd_instrument_v4/temporal-sensitivity/*.json`
**Cost:** ~2.2 s per configuration. Six configurations, untrained.

---

## claim_axis

```
axis: instrument-validity
claim: The SHD framing pipeline propagates temporal structure to the hidden
  representation and the loss, under all three contracts and both geometries.
  A null accuracy result from time-shuffling therefore cannot be dismissed as an
  artifact of preprocessing.
may_claim: That the causal channel from spike timing to the loss exists and is
  large, both at initialization and after training; and that training amplifies
  it ~3.6x at the anchor.
must_not_claim: That temporal order is *necessary* for the task. Every
  trained-weights number is a test-time perturbation of an intact-trained model
  and is therefore confounded with distribution shift (§4b.1). Only
  PREREG_2026-08-02_SHD_TEMPORAL_INFORMATION, which trains each condition on its
  own data, can speak to necessity. Nothing here is an accuracy measurement.
```

## 1. Why this control was needed

`PREREG_2026-08-02_SHD_TEMPORAL_INFORMATION` concludes H1 from a **null**:
shuffling time does not move accuracy. A null is interpretable only if the
measurement could have detected an effect.

The prereg's validity gates cover a lot — bit-identical per-channel counts
(5.1), a trained-regime floor (5.2), degeneracy, determinism — but none of them
establishes that timing ever reaches the loss. Gate 5.1 proves the manipulation
changed timing rather than rate. It says nothing about whether the *pipeline*
carries timing forward.

The failure mode is concrete and was not hypothetical. Framing at 2 ms with an
`adjacent-sum-5` geometry could attenuate temporal structure before the network
sees it. Then all four conditions score alike, H1 "passes", and the very same
artefact independently explains the resolution invariance that motivated the
campaign (T = 100/250/500 → 0.6557/0.6570/0.6536). The experiment would have been
measuring its own preprocessing, and the headline result — a rate-code ceiling —
would have been an artefact.

35 h of compute rested on an assumption that cost 2 s to test.

## 2. Method

Forward pass only, against the **registered untrained initialization**, comparing
each manipulated condition to `intact` on the same sample:

- `mean_spike_hamming` — fraction of hidden `(t, h)` spike positions that differ.
- `mean_membrane_rel_l2` — `‖u_cond − u_intact‖₂ / ‖u_intact‖₂`.
- `mean_rate_rel_l1` — relative change in the per-unit rates the readout sees.
- `prediction_changed_fraction` — see the caveat in §5; the weakest metric here.

Untrained on purpose: this isolates *can timing reach the loss* from *does the
trained model choose to use it*. 256 test samples, `h512`, seed 5170001.

`intact` returns exactly `0.000000` on every metric — a control on the control,
confirming the identity condition is a true identity and the comparison harness
is not manufacturing divergence.

## 3. Result — the control passes everywhere

| contract | geometry | condition | spike Hamming | membrane rel L2 |
|---|---|---|---:|---:|
| published-2ms | adjacent-sum-5 | bin-shuffled | 0.0743 | 0.957 |
| published-2ms | adjacent-sum-5 | channel-shuffled | 0.0782 | 0.969 |
| published-2ms | adjacent-sum-5 | reversed | 0.0773 | 1.151 |
| published-2ms | channels-700 | bin-shuffled | 0.0123 | 1.091 |
| published-2ms | channels-700 | channel-shuffled | 0.0112 | 1.103 |
| published-2ms | channels-700 | reversed | 0.0150 | 1.208 |
| published-10ms | adjacent-sum-5 | bin-shuffled | 0.1874 | 1.012 |
| published-10ms | adjacent-sum-5 | channel-shuffled | 0.2271 | 1.021 |
| published-10ms | adjacent-sum-5 | reversed | 0.1935 | 1.147 |
| published-10ms | channels-700 | bin-shuffled | 0.0736 | 1.094 |
| published-10ms | channels-700 | channel-shuffled | 0.0732 | 1.106 |
| published-10ms | channels-700 | reversed | 0.0809 | 1.204 |
| fixed-t100 | adjacent-sum-5 | bin-shuffled | 0.1452 | 1.191 |
| fixed-t100 | adjacent-sum-5 | channel-shuffled | 0.2284 | 1.204 |
| fixed-t100 | adjacent-sum-5 | reversed | 0.1861 | 1.411 |
| fixed-t100 | channels-700 | bin-shuffled | 0.0604 | 1.226 |
| fixed-t100 | channels-700 | channel-shuffled | 0.0562 | 1.240 |
| fixed-t100 | channels-700 | reversed | 0.0705 | 1.412 |

**Membrane relative L2 sits between 0.96 and 1.41 in every one of the eighteen
cells.** The hidden trace moves by an amount comparable to its own magnitude.
The pipeline does not destroy temporal information under any framing tested.

**The campaign is valid to run.** A trained null would be a real null.

## 4. Two findings beyond the control's remit

These are descriptive and were not registered in advance. They are hypotheses for
the campaign to test, not results.

### 4.1 A dissociation that sharpens the rate-code hypothesis

The representation is strongly timing-sensitive at *every* resolution, including
the finest. Yet accuracy is famously flat across resolution: a 5× change in
temporal resolution moves it by 0.002.

So the invariance cannot be that timing never enters the network — it plainly
does, at all three contracts. The invariance has to arise **downstream**, at the
readout. That is a sharper and more falsifiable claim than "the model is a rate
code": it locates the discarding of temporal information at a specific stage, and
predicts that hidden-layer probes would recover timing information the readout
throws away.

### 4.2 The order/synchrony decomposition is already visible before training

`channel-shuffled` disturbs the representation more than `bin-shuffled` in five
of six configurations, and by a wide margin at the coarser framings
(0.2284 vs 0.1452 at `fixed-t100/adjacent-sum-5`). Both destroy temporal order;
only `channel-shuffled` also destroys cross-channel synchrony.

That contrast is the genuinely novel part of the prereg's design — most published
shuffle controls conflate the two. It is encouraging that the manipulation
separates them mechanically at initialization, since it means the campaign's
contrast has something to detect.

## 4b. Trained weights — the prediction in §4.1 was wrong

Run 2026-08-03 against the trained anchor (`published-2ms / adjacent-sum-5 /
h512 / e100 / s5170001`, reproduced bit-identically at accuracy `0.716431095`
before probing).

| condition | spike Hamming | membrane rel L2 | pred. changed | mean abs ΔLoss |
|---|---:|---:|---:|---:|
| | untrained → trained | untrained → trained | trained | trained |
| bin-shuffled | 0.0743 → **0.2692** | 0.957 → 0.980 | **0.543** | 2.451 |
| channel-shuffled | 0.0782 → **0.2830** | 0.969 → 0.879 | **0.660** | **10.764** |
| reversed | 0.0773 → **0.2713** | 1.151 → 1.240 | **0.078** | **0.181** |

Two things fall out, and neither is what §4.1 predicted.

**Training makes the representation ~3.6× *more* timing-sensitive**, not less
(spike Hamming 0.074 → 0.269). §4.1 guessed the invariance might live at the
readout while the hidden layer stayed sensitive. Instead the hidden layer becomes
*markedly more* sensitive, and the trained readout is anything but indifferent:
shuffling flips 54–66 % of its predictions.

**A sharp dissociation between global order and local structure.** Reversal —
which preserves per-channel counts, inter-spike intervals, within-bin synchrony
and all local structure, flipping only global order — is nearly harmless:
`ΔLoss = 0.181`, and predictions change *less* than at initialization
(0.078 vs 0.125). Shuffling, which destroys local structure, is catastrophic:
`ΔLoss = 2.451` for bin-shuffled and `10.764` for channel-shuffled, a **59×
spread from reversal to channel-shuffling**.

So the learned solution is close to invariant to *global temporal order* while
being strongly dependent on *local temporal structure and cross-channel
synchrony*. That is a more specific characterisation than either "rate code" or
"timing code", and the bin/channel contrast — 2.451 vs 10.764, a 4.4× gap —
locates most of the dependence in cross-channel synchrony rather than in order
within a channel.

### 4b.1 Why this does NOT answer H1 — a correction

I described this probe beforehand as possibly answering the campaign's core
question without the 35 h run. **That was wrong, and the distinction matters.**

This probe perturbs at **test time** a model trained only on intact data. The
campaign trains *and* tests each condition on its own manipulated data. Those ask
different questions:

- **This probe:** is the learned solution sensitive to temporal perturbation?
  → Yes, strongly.
- **H1:** is temporal order *necessary information* for the task — can a model
  trained on shuffled data reach the same accuracy?
  → Untouched by this measurement.

A model can be highly sensitive to test-time shuffling purely because shuffled
input is **out of distribution**, and still be trainable to identical accuracy on
shuffled data, if the task never needed order. The large `ΔLoss` values above are
exactly what distribution shift produces, and cannot be separated from genuine
timing-dependence by this design.

The two measurements are complementary, not redundant: this one characterises the
*learned solution*, H1 characterises *task requirements*. **The 24-cell campaign
is still required**, and §4b is a hypothesis-generator for it, not a substitute.
Concretely it predicts `reversed ≈ intact` in accuracy, with any H1 failure
concentrated in the shuffled conditions.

## 4c. Replication across seeds and widths — caveat 3 discharged

Caveat 3 below said the control ran at a single seed and a single width, which is
enough for "does the channel exist" and not enough for anything quantitative.
`scripts/temporal_sensitivity_sweep.py` replicates it across the full set of
registered initialisations: 3 seeds x 3 widths x 2 geometries = **18
configurations**, 256 test samples each, `published-2ms`, untrained.

| condition | spike Hamming min / median / max | membrane rel L2 min / median / max |
|---|---|---|
| bin-shuffled | 0.0123 / 0.0484 / 0.1223 | 0.9264 / 1.0212 / 1.1348 |
| channel-shuffled | 0.0112 / 0.0497 / 0.1330 | 0.9276 / 1.0322 / 1.1539 |
| reversed | 0.0150 / 0.0516 / 0.1236 | 1.1345 / 1.1812 / 1.2262 |

**Gate 5.0 passes in all 18, and the reported figure is the sweep minimum, not a
mean** — a single weak configuration cannot hide behind an average. The worst
membrane rel L2 anywhere in the sweep is `0.9264` against a `0.1` floor.

The identity condition returns exactly `0.000000` in every one of the 18, which
is the control on the control: had the comparison harness been manufacturing
divergence, it would show up here.

Two observations, both descriptive:

- **The spike-Hamming spread is wide (0.011 to 0.133) and the membrane spread is
  narrow (0.93 to 1.23).** The membrane metric is the more stable one across
  configurations, which is a reason to keep treating it as the load-bearing one
  rather than spike Hamming.
- **The order of the three conditions by membrane divergence is the same in every
  configuration**: `reversed` largest, then `channel-shuffled`, then
  `bin-shuffled`. §4.2 read the bin-vs-channel gap off six configurations; it
  survives eighteen.

This discharges caveat 3 for the *existence and magnitude* of the channel at
initialisation. It does not touch caveats 1, 4 or 6 — still untrained, still not
an accuracy measurement, and the trained-weights numbers in §4b are still a
single configuration and still confounded with distribution shift.

## 5. Caveats, stated plainly

1. **Untrained.** This establishes that the causal channel exists, not that a
   trained network uses it. A trained network could learn weights that discard
   timing — which is exactly H1.
2. **`prediction_changed_fraction` is weak evidence here and should not be
   quoted.** At initialization the readout is near-chance, so predictions are
   unstable under any perturbation and a flip carries little information. It is
   reported for completeness. The spike Hamming and membrane L2 are the load
   bearing metrics.
3. ~~**Single seed, single width, 256 test samples.**~~ **Discharged by §4c**,
   which replicates across 3 seeds x 3 widths x 2 geometries and reports sweep
   minima rather than means. Applies to the untrained control only; §4b's
   trained-weights numbers remain a single configuration.
4. **Not an accuracy measurement.** Nothing here says how much accuracy would
   move. Representation divergence and accuracy divergence are different
   quantities, and the gap between them is the campaign's subject.
5. **§4 and §4b are post-hoc.** None of these findings was registered. They are
   hypotheses generated by this control, not confirmed results.
6. **Test-time perturbation is confounded with distribution shift** (§4b.1). Every
   trained-weights number above is measured on input the model never saw in
   training. This confound is not separable within this design and is the reason
   §4b cannot stand in for H1.
7. **§4.1's prediction was falsified by §4b** and is retained only as a record of
   what was expected. Training increased timing sensitivity rather than leaving it
   to the readout.

## 6. Consequence for the prereg

This should become a **blocking validity gate** in
`PREREG_2026-08-02_SHD_TEMPORAL_INFORMATION` §5, ahead of the trained-regime
gate, since it is orders of magnitude cheaper and can void the design outright:

> **5.0 Pipeline sensitivity (blocking, pre-run).** For the campaign's contract
> and geometry, `temporal-sensitivity` must show `mean_membrane_rel_l2 ≥ 0.1` and
> `mean_spike_hamming > 0` for every non-identity condition against the untrained
> initialization. If the pipeline does not propagate timing, a null accuracy
> result is uninterpretable and the campaign must not be run.

Measured at the anchor: `0.957 / 0.969 / 1.151` against a `0.1` bound. Passes
with an order of magnitude to spare.
