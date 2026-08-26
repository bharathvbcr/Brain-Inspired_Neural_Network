# Amendment — the primary lever's rationale was wrong, and it was derivable before registering

**Amends:** [`PREREG_2026-08-25_THE_H1024_COLLAPSE.md`](PREREG_2026-08-25_THE_H1024_COLLAPSE.md) §3.
**Registered:** 2026-08-26, with **5 of the 24 lever cells complete** — below the
9-valid floor at which any arm becomes evaluable, so no hypothesis is decided
here and none is changed.

**No hypothesis, threshold, criterion, seed, or outcome moves.** What is
corrected is the *reason given* for choosing the primary lever. The outcome that
the early cells point toward, **O-5 — "a lever makes h1024 worse"** — was
already enumerated in §8 and is registered exactly as it stands.

---

## 1. What the preregistration claimed

> **`--surrogate-scale` is the primary lever.** It reduces gradient magnitude at
> source, and wave 13 established it as the lever that stabilises `rec+alif`.

The second clause is true. **The first is false for this arm**, and the source
says so plainly enough that it should have been read before the lever was
chosen rather than after the cells started landing.

## 2. What the scale actually does

`surrogate_derivative_scaled` (`binn-learn/src/shd_matched.rs:253`) is

```
alpha = MATCHED_SURROGATE_ALPHA * scale          // 5.0 * scale
        (alpha / 2)
  ---------------------------------
   1 + ((pi/2) * alpha * u)^2
```

a Lorentzian in `u = membrane - threshold`. Its **area is invariant in
`scale`** — the parameter trades peak height against width and moves no
probability mass:

| scale | alpha | peak gain | half-width \|u\| | area |
|---:|---:|---:|---:|---:|
| 1.00 | 5.00 | 2.5000 | 0.1273 | 1.0 |
| 0.50 | 2.50 | 1.2500 | 0.2546 | 1.0 |
| **0.40** | 2.00 | **1.0000** | 0.3183 | 1.0 |
| 0.25 | 1.25 | 0.6250 | 0.5093 | 1.0 |

So lowering the scale does not reduce gradient magnitude. It **lowers the
per-unit peak and broadens the band of units that receive any gradient at all**,
by the same factor.

## 3. Why that stabilises a recurrent arm and need not stabilise this one

The module's own doc comment, three lines above the function, states the case it
was exposed for:

> the peak gain is the dominant per-timestep term in the **compounded recurrent
> backward**: at the registered `alpha = 5.0` it is 2.5, and a per-step gain
> above 1 against a recurrent block with spectral radius near 1 is sufficient on
> its own to overflow f32 over several hundred timesteps.

That is a statement about **compounding**, and the table above shows why wave 13
landed where it did: **scale 0.4 is exactly the point at which the peak gain
reaches 1.0**. Below a per-step gain of 1 the recursion contracts instead of
expanding. `rec+alif` was stabilised because the lever addressed the mechanism
that was destabilising it.

**The h1024 `ff+fixed+attn` arm has no such recursion through the spiking
layer.** It is feed-forward, and §1 of the preregistration located its pathology
in depth *and* width **with the read-out attached** — `h1024` at `L1` is
completely healthy. There is no per-timestep compounding term for a lower peak
gain to tame. What the broader band does instead is put **more of 1024 hidden
units inside the active window at every step**, and a sum of more non-zero terms
is a larger gradient, not a smaller one.

I applied a lever validated against one mechanism to an arm that fails by
another, and wrote down a rationale that named the mechanism it does not have.

## 4. What the early cells show, and what that is worth

Five of the twenty-four lever cells, **against a floor of nine**:

| cell | accuracy | median epoch-mean norm | max norm |
|---|---:|---:|---:|
| `ss0.25` s5170001 | 0.5685 | 188.4 | 2.90e6 |
| `ss0.25` s5170002 | 0.1837 | 10,588.2 | 3.66e19 |
| `ss0.25` s5170003 | 0.3132 | 22,183.7 | 4.89e9 |
| `ss0.25` s5170005 | 0.4474 | 1,982.7 | 3.22e9 |
| `ss0.5` s5170003 | 0.5738 | 86.1 | 6.98e6 |
| *archived, scale 1.0, n=12* | *0.5768* | *55.5* | *1.13e8* |

**This is not a result and is not reported as one.** Five cells is below the
registered floor; the arm carries no verdict until it is complete, and the
analyser will refuse to print a mean for it before then. It is recorded here for
one reason only: it is what sent me back to the source, and the correction is
worth more if it is dated before the arm finishes than after.

The direction is what §2 predicts — lower scale, broader band, larger norms —
and every one of these cells passes the validity gate, which is the design
(magnitude never voids, because magnitude is the quantity under study).

## 5. The prediction, made now rather than after

Recorded because the preregistration's original prediction rested on the wrong
mechanism and this one should be falsifiable on the same cells:

- **`ss0.5` and `ss0.25` will not meet H15-1.** The gain will not reach +0.05,
  and the median epoch-mean norm will not fall below 1.0, so H15-2 will not be
  met either.
- **`ss0.25` will be worse than `ss0.5`**, which will be worse than the archived
  scale 1.0, monotonically in the width of the surrogate band.
- **The clipping arm remains live.** `--clip-grad-norm 1000.0` bounds the norm
  directly rather than reshaping the surrogate, so it addresses the observed
  quantity whatever the underlying mechanism turns out to be. H15-1 therefore
  now rests mainly on it.

If instead a scale arm *does* recover the gain, §2 is wrong about this arm and
the recovery needs a mechanism nobody here has proposed — which is outcome O-3,
already registered, and it would be reported as unexplained.

## 6. What does not change

- **No third lever is added.** The preregistration's §9 stopping rule binds:
  *"a campaign that keeps adding parameters until one works has stopped testing
  a hypothesis."* Discovering that the primary lever was chosen for a wrong
  reason is not a licence to go looking for a better one mid-campaign.
- **The bars, seeds, widths, depths, scales and clip threshold are untouched.**
- **The h1024 collapse question is unaffected.** Whether it is an optimisation
  failure or a capacity limit is exactly as open as it was; what has changed is
  that one of the two levers aimed at it is now expected to fail, and for a
  stated reason rather than by surprise.
- **If no lever recovers the gain, outcome O-2 fires** and the paper's h1024
  scope limit stands — strengthened by having been tested, and *weakened* as
  evidence by the fact that one of the two tests was aimed at the wrong
  mechanism. Both halves of that belong in the result.
