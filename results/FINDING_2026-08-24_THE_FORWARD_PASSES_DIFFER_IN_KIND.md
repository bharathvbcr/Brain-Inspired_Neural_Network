# Finding — the reference is a temporal-convolutional SNN and the instrument is a pointwise one

**Method:** reading `SnnDelays.forward` (`snn_delays.py:205-256`) against
`shd_matched.rs:265-300` term by term, after three config-surface ablations
failed to account for the gap.

**Result: two of the differences were never on my list, and one of them is the
mechanism.**

---

## 1. Term by term

| term | reference | instrument |
|---|---|---|
| **temporal kernel** | **`Dcls1d`, 25 taps per synapse, every layer** | **none — instantaneous** |
| **normalisation** | **`BatchNorm1d` over channels, every layer** | **none** |
| padding | asymmetric, left 24 / right 12 | n/a |
| depth | 2 hidden layers | 1 |
| dropout | 0.4, on spikes | none |
| threshold | 1.0 | 1.0 |
| reset | hard (`v_reset = 0`) | hard (`u × (1 − s)`) |
| bias | **False** | `b_out` present |
| readout | membrane `v_seq`, softmax-summed over time | spike rate, mean over time |

The reset rule, the threshold and the surrogate all match. **The parts I assumed
were the interesting differences are the parts that agree.**

## 2. The mechanism: two ways of handling time, not one done better

```
reference LIF tau  = 1.005 timesteps  ->  decay 0.3697 per step
instrument alpha at published-2ms     ->  decay 0.8195 per step
```

The reference's neuron has **almost no memory** — it decays to a third in a single
step. It does not need any, because every synapse carries a **25-tap kernel
spanning 250 ms**. Temporal integration happens in the *convolution*, and the
membrane is close to a pointwise nonlinearity.

The instrument is the mirror image: **no temporal kernel at all**, and a membrane
that retains 0.82 per step to compensate. All of its temporal integration is a
single exponential trace.

`grep -cE "kernel|conv|dilat" binn-learn/src/shd_matched.rs` → **0**.

These are not the same architecture with different hyper-parameters. One
integrates time with a learned 25-tap filter per synapse; the other with one
scalar decay shared by everything.

## 3. This explains why every ablation failed

My delay ablation set `lr_pos = 0`, `sigInit = 0`, `DCLSversion = 'max'`. **The 25
taps stayed.** It measured the value of *learning where the taps sit* — 0.0348 —
while leaving the temporal receptive field entirely intact.

That is why none of the four ablations moved the residual: **all of them were
inside a model that still had the mechanism the instrument lacks.** I was varying
decoration and the difference was structural.

`use_batchnorm = True` is the second thing I never enumerated. Every layer of the
reference is batch-normalised; the instrument has no normalisation anywhere.

That makes **five** enumeration errors in this investigation: claiming `augment`
and `stateful_synapse` were on when they are off; twice claiming knobs were not
knobs; and missing both the temporal kernel and batchnorm entirely. The prior I
recorded yesterday — *"a fourth omission is more likely than not"* — was correct
and, if anything, understated.

## 4. What this says about the instrument, and it is not a defect

The instrument reaching **0.7378** with a pointwise LIF and a single exponential
trace, against a 25-tap temporal-convolutional network at 0.9390, is not evidence
that the instrument is broken. It is evidence that the two are different model
classes, and the comparison has been reading a **class difference as a quality
gap** since the calibration criterion was written.

The `CELL_PASS` floor of 0.80 was set from a delays-based reference. It is not a
threshold the pointwise architecture was ever shown to be capable of, and
`FINDING_2026-08-23_THE_MATRIX_GRID_EXCLUDES_ITS_OWN_GATE.md` already showed the
matrix grid cannot reach it for a separate reason.

**And the instrument's own answer to this is already in the record.** The
attention read-out is a mechanism over the time axis, and it is what lifts the
instrument toward the delay-free reference. That was the campaign's headline
result all along; what is new here is *why* it works, and that it is the
instrument's substitute for the temporal kernel rather than an unrelated
improvement.

Paired by seed, on one substrate at a time, against the delay-free reference's
**0.9042**:

| contract | rate read-out | + attention d32/L4 | gain | reference |
|---|---:|---:|---:|---:|
| `published-2ms` (anchor) | 0.7062 | **0.8320** | +0.1258 | 0.9042 |
| `fixed-t100` | 0.6672 | **0.8599** | +0.1927 | 0.9042 |

> **CORRECTED 2026-08-25.** This paragraph read *"lifts the instrument from
> 0.7407 to **0.8821**"*. Both are **single best cells, and not of the same
> arm**: 0.7407 is the best of twelve `ff+fixed` cells at **h1024**, and 0.8821
> the best of twelve attention cells at **h128** on the **`fixed-t100`**
> contract. Two widths and two contracts, compared max to max, in a sentence
> that reads as one arm improving.
>
> The conclusion survives — the read-out is what closes most of the distance,
> on every paired contrast in the record — but the size does not. Max-to-max
> gives 0.7407 → 0.8821 and a near-parity with 0.9042 that no paired comparison
> supports. This document's own §4 argues the instrument is not defective but a
> different model class; that argument does not need, and is not helped by, the
> most flattering pair of cells in the archive.
>
> The rest of this finding is a structural reading of two forward passes and is
> unaffected: no number in §1–§3 is a comparison of arms.

## 5. What is not established

- **This is a structural reading, not a measurement.** The claim that the temporal
  kernel is *the* mechanism is supported by the code and by four failed ablations,
  not by an ablation that removed it. Removing it is not a config change — it is a
  different model — so no run on this config can test it.
- **Batchnorm is untested.** `use_batchnorm = False` is a genuine one-line ablation
  and was not run. It should be, before the temporal kernel is credited with the
  whole remainder.
- **No number here is new.** The accuracies are all previously recorded; this
  document only re-attributes them.
- **The 0.074 is still not decomposed** into kernel versus batchnorm versus
  anything else, and this finding does not decompose it — it identifies what the
  candidates actually are, which the previous list did not contain.
