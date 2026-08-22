# Amendment: expose the surrogate gain, to make the recurrent arm measurable

> ## OUTCOME: the registered expectation in §4 is MET — 3/3 complete,
> ## `non_finite_events` = 0, loss falls. **But the arm is still numerically
> ## marginal**: one seed peaks at 3.93e33, ~5 orders from f32 overflow. See §6.

**Registered:** 2026-08-05, before implementation.
**Authorized:** by the human, explicitly, as a model change.
**Amends:** `MATCHED_SURROGATE_ALPHA` usage — via a flag, not by moving the
constant.
**Bears on:** `PREREG_2026-08-02_SHD_TEMPORAL_INFORMATION` H2 (NOT RUN twice) and
`PREREG_2026-08-02_SHD_BPTT_CEILING_RECALIF` (cannot execute).

---

## 1. Why this and not clipping or width

Four interventions have been tried against the recurrent failure and all four
failed, each for a recorded reason:

| intervention | acts on | outcome |
|---|---|---|
| rescale `W_rec` init | starting point | non-monotonic over 3 orders of magnitude; ranking does not survive reseeding |
| f64 `l2_norm` | the *record* | corrected reporting; dynamics untouched by construction |
| batch gradient clipping | the update step | **never reached** — abort fires on a per-sample gradient, upstream |
| narrow to h256 | fan-in | **fails too**, at e100: 2 of 12 cells abort at steps 374 and 727 |

The failure is compounding **inside the per-sample backward pass**. Only
something that reduces the per-timestep backward gain can address it.

`surrogate_derivative` is a Lorentzian with peak `α/2` at threshold, and
`α = MATCHED_SURROGATE_ALPHA = 5.0`, so the **peak per-timestep gain is 2.5**.
Multiplied by a recurrent block with spectral radius near 1 (Glorot's design
point) and compounded over the several hundred timesteps of a 2 ms frame, a
per-step gain above 1 is sufficient on its own to explain overflow.

## 2. The change

A `--surrogate-scale` flag multiplying `α`:

```
alpha_effective = MATCHED_SURROGATE_ALPHA * surrogate_scale
peak gain       = alpha_effective / 2
```

**Default 1.0, which is bit-identical to before the flag existed.** The
registered constant is not moved; it is multiplied by 1.0 unless a run asks
otherwise. Gate F over the 216 recorded cells is the binding check, as with
every other change today.

### The one value that is principled

**`--surrogate-scale 0.4` gives `α = 2.0` and a peak gain of exactly 1.0.**

That is not a tuned number. A peak per-timestep gain of 1 is the boundary
between contraction and expansion in the compounded backward, so it is the
natural first value to test, and it was chosen **before running anything**.

If 0.4 does not work, the next value is **not** to be chosen by trying values
until a campaign succeeds. Any further value requires its own amendment stating
why, on the same terms as this one.

## 3. What this costs — stated plainly, because it is not small

**A gradient computed with a different surrogate is a different gradient.** It is
still BPTT and still a surrogate method, but:

- **The 216 recorded cells and the 0.7378 ceiling were measured at α = 5.0.** A
  recurrent number obtained at α = 2.0 is **not comparable to them**. Reporting
  "recurrence reaches X" against "feed-forward reaches 0.7378" would be
  comparing two different training procedures.
- Therefore **any H2 or RECALIF comparison must run BOTH arms at the same
  surrogate scale**, exactly as `AMENDMENT_2026-08-03_H2_AT_H256.md` §3 required
  both arms at the same width. A matched `ff+fixed` baseline at α = 2.0 is part
  of the measurement, not optional.
- **A ceiling measured at a reduced surrogate gain is a ceiling for that
  method**, and must be labelled that way wherever it appears.

## 4. Falsifiable expectation, recorded before running

At `rec+alif / h256 / e100 / --surrogate-scale 0.4`, three seeds:

- **Expected:** all 3 seeds complete, `non_finite_events` = 0, loss falls
  monotonically. The h256 failures occurred at optimizer steps 374 and 727, so a
  run that reaches step 3200 without aborting is a clear pass.
- **If any seed still aborts:** the compounding hypothesis is incomplete —
  reducing the dominant per-timestep term by 2.5× was not sufficient — and the
  next candidate is truncated BPTT, which bounds the *number* of compounding
  steps rather than their size. That would be a further amendment, not a retry
  at a smaller scale.

## 5. Stopping rule

**One value (0.4), three seeds, reported whichever way it falls.** If it works,
H2 is run at that scale with a matched `ff+fixed` baseline. If it does not, it
is reported as a failed intervention alongside the other four, and H2 stays NOT
RUN.

No scale sweep. The `W_rec` scale pilot already demonstrated what sweeping a
stability parameter across three orders of magnitude buys: a non-monotonic
response that does not survive reseeding.


## 6. OUTCOME

`rec+alif / h256 / e100 / --surrogate-scale 0.4`, three seeds:

| seed | accuracy | `non_finite_events` | peak gradient norm | loss first → last |
|---|---:|---:|---:|---|
| 5170001 | 0.5141 | 0 | 1.17e12 | 2.941 → 1.606 |
| 5170002 | 0.4386 | 0 | **3.93e33** | 2.945 → 1.950 |
| 5170003 | 0.4567 | 0 | 3.08e10 | 2.946 → 1.839 |

**The §4 expectation is met.** All three reach optimizer step 3200 — the h256
failures at scale 1.0 aborted at steps 374 and 727 — with zero non-finite events
and a large monotone loss drop. Accuracy 0.44-0.51, against 0.3613 for the same
arm at e20/scale 1.0.

**But this is a qualified success and should not be reported as a fix.** The
peak gradient norms are still enormous:

- healthy `ff+alif` peaks at ~0.15;
- seed 5170002 peaks at **3.93e33**, which is **4.9 orders of magnitude** from
  f32 overflow;
- the other two sit 26-28 orders clear.

So the intervention **moved the arm from "aborts" to "completes"**, and did not
move it to "numerically healthy". Seed 5170002 is one bad trajectory away from
the failure this was meant to remove, and the spread across three seeds spans 23
orders of magnitude — the same chaotic, non-reproducible-across-seeds behaviour
the `W_rec` scale pilot found.

### What follows

H2 **can** now be attempted at this scale with a matched `ff+fixed` baseline, per
§5. Two things must travel with any number it produces:

1. **The gradient is not the registered one.** α = 2.0, not 5.0. Nothing from
   this scale is comparable to the 216 recorded cells or to the 0.7378 ceiling.
2. **The arm remains marginal.** A completed cell at 3.93e33 is not evidence of
   a healthy optimisation, and a ceiling claim from this configuration would be
   unsafe. H2 asks a *relative* question — does recurrence degrade less under
   shuffling than feed-forward — which is more defensible here than an absolute
   ceiling, but the margin should be stated wherever it appears.

**No smaller scale will be tried to chase the margin.** §5's stopping rule holds:
one value, reported as measured. If the marginality matters enough to fix, the
next step is truncated BPTT — which bounds the *number* of compounding steps
rather than their size, and is a different intervention rather than more of this
one.
