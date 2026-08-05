# Gradient clipping does not rescue rec+alif at h512, and cannot

**Date:** 2026-08-03
**Backend:** rust only. Binary `ec1b5d9db6f7`.
**Cost:** 3 cells, `rec+alif / h512 / e20 / published-2ms / adjacent-sum-5`, `--clip-grad-norm 1.0`.

---

## claim_axis

```
axis: instrument-validity
claim: Batch-level global-norm gradient clipping does not make rec+alif
  trainable at h512. The failure it was meant to address occurs upstream of
  where clipping acts, so no threshold could have worked.
may_claim: That with --clip-grad-norm 1.0 all three seeds abort on a non-finite
  per-sample gradient (steps 43, 34, 76); that seed 5170001, which completed
  without clipping, aborts with it; and that the abort site precedes the
  clipping site in the training loop.
must_not_claim: That gradient clipping is useless in general, or that it is
  incorrectly implemented — it is verified bit-identical when off and correct
  when on. Only that it does not address this failure. Nor that rec+alif is
  untrainable in principle: the untried interventions in §4 are model changes,
  not tuning.
```

## 1. Result

| seed | without clipping (2026-08-03) | with `--clip-grad-norm 1.0` |
|---|---|---|
| 5170001 | completed, 420 large-norm steps, acc 0.3507 | **ABORT at step 43** |
| 5170002 | ABORT at step 220 | **ABORT at step 34** |
| 5170003 | ABORT at step 50 | **ABORT at step 76** |

Clipping did not rescue any seed, and **made seed 5170001 strictly worse** — it
previously completed, and now aborts. Two of three seeds abort *earlier* than
without it.

## 2. Why it cannot work here — the structural reason

The training loop:

```rust
for &index in batch {
    let (forward, sample_gradient) = shd_matched_loss_and_gradient_arm(&weights, &train[index])?;
    if !forward.loss.is_finite() || !sample_gradient.all_finite() {
        return Err(...);              // <-- (1) the abort fires HERE
    }
    gradient.add_assign(&sample_gradient);
}
gradient.scale(1.0 / batch.len() as f32);
let gradient_norm = gradient.l2_norm();
if let Some(threshold) = clip_grad_norm { ... }   // <-- (2) clipping acts HERE
```

**(1) precedes (2).** The abort is triggered by an *individual sample's*
gradient containing a non-finite entry, during the per-sample backward pass.
Clipping operates on the batch-averaged gradient, which does not exist yet.
**No threshold value could have changed this outcome**, because clipping is
never reached on the step that fails.

Nor would moving clipping inside the loop help. Rescaling a vector that already
contains an infinity does not recover it: `threshold / inf` is `0`, and
`0 * inf` is `NaN`. Once an entry overflows in the backward accumulation, the
information is gone. Clipping bounds gradients that are *large*; it cannot
repair ones that are *non-finite*.

**Why it made seed 5170001 worse.** Clipping changes the weight trajectory as
soon as it first binds. The run then explores a different region and reaches a
sample whose backward overflows sooner. This is not clipping "causing"
instability — it is a chaotic trajectory being perturbed into a different part of
the same unstable basin.

## 3. What this closes

Three interventions have now been tried against the h512 `rec+alif` failure, and
all three have failed for the same underlying reason — they address the
*symptom* at the wrong stage:

| intervention | acts on | result |
|---|---|---|
| rescale `W_rec` initialisation | the starting point | non-monotonic across 3 orders of magnitude; ranking does not survive reseeding |
| f64 `l2_norm` | the *record* of the gradient | corrected reporting; dynamics untouched by construction |
| batch-level clipping | the update step | **never reached; abort is upstream** |

The failure is in the **per-sample backward pass**: BPTT through several hundred
timesteps with a surrogate derivative peaking at 2.5 and a recurrent block whose
spectral radius is near 1, at a width with 16x the fan-in of h128. The product
of per-timestep gains exceeds 1 and compounds until an entry leaves f32 range.
Nothing applied after that pass can undo it.

## 4. What has not been tried

These are **model changes** and each needs registration before use — none is
tuning, and none should be attempted to make a campaign come out well:

1. **Truncated BPTT.** Bound the backward window. Directly attacks the
   compounding, and is the standard remedy. Changes what gradient is computed,
   so it changes what the ceiling means.
2. **Lower surrogate gain.** `MATCHED_SURROGATE_ALPHA` gives a peak derivative
   of 2.5; reducing it reduces per-timestep gain. Registered constant.
3. **Spectral-radius-normalised recurrent initialisation** rather than a
   rescaled Glorot draw — the one initialisation family §6.6 of the stability
   measurement explicitly did not rule out.
4. **A narrower width.** h128 and h256 are clean. RECALIF registers h512
   specifically, so this is an amendment to that prereg, not a workaround.

## 5. The implementation is kept

`--clip-grad-norm` stays in the instrument. It is verified:

- **default-off is bit-identical** — a recorded cell reproduces exactly
  (`0.589222615`) with `clip_grad_norm: null`, `clipped_steps: 0`;
- non-finite and non-positive thresholds are rejected;
- `clipped_steps` and `unclippable_steps` are recorded, so a clipped run can
  never be mistaken for an unclipped one;
- the scale is computed in f64, because at h512 the norms reaching that point
  are ~1e29 and `threshold / norm` in f32 would flush to zero.

It is correct and may be useful elsewhere. It simply does not solve this.

## 6. Consequence

**H2 remains NOT RUN**, and the reason is now better understood rather than
resolved. `PREREG_2026-08-02_SHD_BPTT_CEILING_RECALIF` still cannot execute as
registered. The next attempt should be truncated BPTT or a narrower width, both
registered in advance — not another threshold.
