# The check that cannot fail

*Notes from a day of measuring a spiking network, in which four separate things
went wrong in exactly the same way.*

---

There is a particular kind of bug that doesn't crash, doesn't corrupt anything,
and doesn't produce a wrong number. It produces a **right-looking** number, from
a check that was never capable of returning any other answer.

I spent a day on a small spiking-neural-network research instrument and hit four
of them. They looked unrelated — a counter, a statistical test, a numerical
remedy, a convergence rule. They were the same bug wearing four costumes, and
the shape is worth naming because I don't think it's rare.

## 1. The counter that was never incremented

The instrument trains a network and writes a JSON cell for each run. Whether a
cell counts as scientifically valid runs through one predicate:

```rust
let scientific = evaluation.accuracy >= 0.80
    && ... && diagnostics.non_finite_events == 0;
```

`non_finite_events` is the numerical-sanity guard. If the gradients blew up, the
run is garbage, and this is what says so.

It was declared on the struct. It was read here. It was written into all 296
completed cells. **And no code path anywhere ever incremented it.**

The clause `non_finite_events == 0` could not be false. It had never been false.
It was decoration.

I found it the way you find these — not by reading the code, but by trying to
reproduce a result and having the reproduction die in a way the original hadn't.
The original cell had reported `"non_finite_events": 0` next to
`"mean_gradient_norm": inf`. Those two facts side by side in one file is what
made me look.

That `inf` was its own bug, incidentally: the JSON writer had a finiteness guard
on its 32-bit path and not on its 64-bit one, so a diverging run wrote a file no
parser could read. The record that would explain the divergence was the one
thing the divergence destroyed.

## 2. The test that couldn't see the effect

The scientific question was whether this network cares about the *order* of
events in time, or only about their aggregate statistics. You test it by
shuffling time and retraining — if accuracy doesn't move, order wasn't being
used.

We had preregistered the threshold: order is "negligible" if shuffling costs
≤ 0.02 accuracy **and** the 95% confidence intervals overlap. Both conditions,
fixed in advance, before any data.

Three seeds came back. Difference: **0.019876**. Under the bound. CIs
overlapping, barely. **Verdict: supported.** The network is order-invariant.

It passed by 0.000124 — 0.62% of its own bound. One of the three seeds exceeded
the threshold on its own.

So I registered an extension before running it: exactly three more seeds, verdict
recomputed once, reported whichever way it lands, **no seventh seed regardless of
outcome**. That last clause is the entire point. Adding seeds until the answer
looks right is optional stopping, and it would have made the preregistration
worthless.

Six seeds:

| criterion | value | verdict |
|---|---|---|
| difference ≤ 0.02 | 0.018919 | passes |
| CIs overlap | disjoint by 0.0120 | **fails** |

**Not supported.**

Here is what's interesting: *nothing about the effect changed*. The standard
deviation was essentially identical. What changed was the *t* multiplier — 4.303
at three seeds, 2.571 at six — so the intervals narrowed from ±0.0097 to ±0.0035
and a gap that had been there all along became visible.

The three-seed test wasn't wrong. It was **incapable of resolving the thing it
was measuring**, and it reported "no effect detected" in a way indistinguishable
from "no effect exists."

Had I stopped at three seeds I would have published *this network is a rate
coder*. It isn't. It's order-sensitive — just weakly, in an amount that happens
to sit below the bound we'd picked for "negligible" while being perfectly
reliable across every seed.

That's a more interesting result than either clean answer, and I only have it
because the stopping rule was fixed before the data arrived.

## 3. The fix applied downstream of the failure

A different arm of the same instrument — the recurrent one — wouldn't train at
all at the width we needed. Two of three random seeds died mid-run with
non-finite gradients; the third survived with a gradient norm around **1e29**,
roughly thirty orders of magnitude above healthy.

The textbook remedy is gradient clipping: if the gradient is too big, scale it
down. I implemented it, defaulted it off so nothing already recorded would move,
verified bit-identical behaviour when disabled, and ran it.

All three seeds died. And the one that had previously *survived* now died too.

The reason is embarrassing and completely structural. The training loop looks
like this:

```rust
for sample in batch {
    let g = backward(sample);
    if !g.all_finite() { return Err(...) }   // (1) it dies here
    total += g;
}
total /= batch.len();
if let Some(t) = clip { ... }                 // (2) clipping is here
```

**(1) happens before (2).** The failure is in a single sample's backward pass.
Clipping operates on the batch average, which doesn't exist yet. Clipping was
never reached on the step that failed. *No threshold value could have worked.*

Nor would moving it earlier help: rescaling a vector that already contains an
infinity gives `threshold/inf = 0`, then `0 × inf = NaN`. Clipping bounds
gradients that are **large**. It cannot repair ones that are **non-finite**.

I had spent the previous hour confidently telling my collaborator that clipping
was the lever. I'd written it into two planning documents as the recommended
next step. Both are now corrected, which is the cheapest part of being wrong.

## 4. The convergence rule that could never converge

Last one, and my favourite.

The instrument had a registered rule for deciding whether a network has trained
long enough. Run a ladder of budgets, then:

```python
first, last = rows[0], rows[-1]
if last.accuracy - first.accuracy > 0.01:
    print("UNDERTRAINED")
```

Compare the first and last rungs. If the whole ladder gained more than 1%, you
haven't trained enough.

Consider what happens as you collect more evidence. The first rung stays pinned
at the shortest budget. The last keeps climbing. So `gain` grows monotonically,
and **the more evidence you gather, the further you get from ever declaring
convergence.**

We had run 100 → 200 → 400 epochs. Total gain 0.0181. Verdict: undertrained,
and a headline result was withdrawn on the strength of it.

But look at the *per-doubling* gains: **+0.0119**, then **+0.0062**. The second
doubling was already under the threshold. The rule couldn't see it because it
never looked at a doubling.

I amended it to ask about the final doubling — keeping the 0.01 constant
untouched, because moving a registered threshold after seeing the data is the
one thing you absolutely may not do — and ran 800 epochs.

The final doubling bought **+0.000294**. One seed got *worse*. Meanwhile training
loss kept falling steeply, which is textbook overfitting and means the budget was
sufficient long ago.

The withdrawn result came back, at a corrected value, now genuinely converged
rather than merely truncated.

## The shape

Four failures:

- a guard whose condition was always true
- a statistical test whose resolution was coarser than its effect
- a remedy applied downstream of the thing it was meant to remedy
- a convergence criterion that diverged by construction

None of them produced a wrong number. Every one of them produced a
**confidently correct-looking** number from an apparatus structurally incapable
of producing any other.

I don't think there's a tool that catches this. I ran a linter across the whole
codebase: nineteen warnings, all stylistic, zero of these. There isn't a type
system for "this branch is unreachable given the semantics of your domain."

What worked was a question, asked mechanically and repeatedly:

> **Is there a path where this reports success without having done the work?**

Applied to a field, that means grepping for *writes*, not reads. Applied to a
CLI flag, it means passing the degenerate value and seeing whether anything
complains. Applied to a statistical test, it means asking what effect size the
test could actually resolve *before* interpreting a null. Applied to a fix, it
means checking that the fix executes on the failing path at all.

None of that requires understanding the science. It's mechanical, and it found
ten defects in a day, five of them in this one family.

## The uncomfortable part

The counter had been broken across 296 completed cells. The whole prior campaign
ran with its only numerical guard switched off, and every one of those cells
reported `"non_finite_events": 0` — truthfully, in the sense that the number
really was zero, and meaninglessly, in every sense that matters.

That campaign's conclusions happen to survive. I checked. But "happens to
survive" is not a property you get to claim in advance, and the reason I can
claim it now is that the recorded outputs were pinned byte-for-byte and could be
re-verified against a rebuilt binary — eleven times, across nine different
builds, while all of this was being changed underneath them.

The regression harness was the only thing standing between "we found some bugs"
and "we have no idea which of our results are real."

That, more than any of the four bugs, is the thing I'd want someone to take away.
