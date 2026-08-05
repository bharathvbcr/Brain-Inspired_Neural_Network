# What the shuffle destroyed

*Two results from a small spiking-network instrument: temporal order turns out to
be worth almost nothing, cross-channel synchrony is worth 6.6× more — and an
accuracy ceiling I withdrew as an artefact turned out to be real after all.*

---

Both results come from the same object: a deliberately simple feed-forward LIF
network with fixed thresholds, trained by backpropagation-through-time on the
Spiking Heidelberg Digits dataset. One hidden layer. No recurrence, no adaptive
threshold, no learned delays. It reaches about 0.74 where the published
state of the art reaches 0.94.

That gap is the point. This is an instrument for asking *what a simple spiking
network actually uses*, not a contender. Every number below is about the
instrument.

## The control everyone runs, and the one nobody separates

If you want to know whether a network trained on spike trains cares about *when*
things happened, the standard move is to shuffle time and see what breaks. It is
a good instinct and it is almost always run in a way that cannot answer the
question.

The problem: there is more than one thing living in "when". Consider two ways to
scramble a spike raster while keeping every channel's total spike count exactly
fixed:

- **Shuffle bins with one shared permutation.** Every channel gets the *same*
  reordering. Sequence order is destroyed. But if two channels fired in the same
  bin before, they still fire in the same bin after — cross-channel synchrony
  survives perfectly intact.
- **Shuffle bins with an independent permutation per channel.** Order dies *and*
  so does synchrony. Channels that used to coincide no longer do.

The usual shuffle control does the second one and reports "timing matters." But
the second manipulation destroys two things, so the number it returns is a sum
of two effects with no way to split them. Run both and subtract, and the
difference isolates synchrony.

So: four conditions, trained *and tested* on manipulated data — not a test-time
perturbation, which would confound the effect with distribution shift. Six seeds,
24 cells.

| condition | destroys | preserves |
|---|---|---|
| `intact` | — | — |
| `reversed` | direction | order magnitude, synchrony, counts |
| `bin-shuffled` | order | counts, within-bin synchrony |
| `channel-shuffled` | order **and** synchrony | counts |

The manipulation is not trusted; it is checked. Per-channel totals are recomputed
before and after and compared bit-for-bit, and the code returns an error rather
than a result if they differ. A manipulation that preserved counts while
relocating nothing would pass that check and mean nothing, so displacement is
audited separately: 99.7% of entries changed bin, mean displacement 109 bins.

## The result

| condition | mean | sd | 95% CI |
|---|---:|---:|---|
| `intact` | 0.7158 | 0.0033 | [0.7123, 0.7192] |
| `reversed` | 0.7126 | 0.0046 | [0.7078, 0.7175] |
| `bin-shuffled` | 0.6968 | 0.0033 | [0.6934, 0.7003] |
| `channel-shuffled` | 0.5721 | 0.0051 | [0.5667, 0.5775] |

| manipulation | cost |
|---|---:|
| reverse time | **0.0032** |
| destroy order | **0.0189** |
| destroy order **and** synchrony | **0.1437** |
| **synchrony-specific increment** | **0.1248 — 6.6× the order effect** |

Direction is worth nothing. Order is worth a little. Synchrony is worth most of
it.

Reversal costing 0.0032 is worth sitting with. Playing the audio backwards
destroys the entire global sequence of a spoken digit and this network barely
notices — while a shuffle that keeps local structure but scrambles bin order
costs six times more. Whatever it is reading, it is not a sequence.

## The disagreement that is the finding

The pre-registered hypothesis H1 said the network was a rate coder, and defined
that with two criteria that had to *both* hold: the intact-minus-shuffled
difference within an equivalence bound of 0.02, **and** overlapping 95%
confidence intervals.

| criterion | value | verdict |
|---|---|---|
| mean difference ≤ 0.02 | 0.018919 | **passes**, by 0.0011 |
| CIs overlap | [0.7123, 0.7192] vs [0.6934, 0.7003] | **fails** — disjoint by 0.0120 |

H1 is not supported. And the two criteria disagreeing is more interesting than
either verdict would have been alone, because they are asking different
questions. The equivalence bound asks *is this effect big enough to care about?*
— no, it is below the threshold I registered for practical negligibility. The
CI test asks *is this effect real?* — yes, unambiguously: all six seeds positive,
ranging 0.0150 to 0.0203.

The honest summary is neither "order matters" nor "order doesn't matter." It is
**order matters a little, and consistently**. Both halves of that sentence are
load-bearing, and a study that ran only one of the two tests would have reported
half of it as the whole story.

## The part I could only report because I wrote it down first

Here is what actually happened. At three seeds, H1 **passed** — the difference
was 0.019876 and the CIs overlapped by 0.0006. A boundary pass: 0.62% inside its
own bound. I could have published "this network is a rate coder" right there.

Instead I registered an amendment: add exactly three seeds, recompute once,
report whichever way it falls, no seventh seed. Then I ran them, and the verdict
flipped to NOT SUPPORTED.

The mechanism is pure statistics and worth understanding, because it is the
opposite of what "the result changed when I added data" usually implies. The
standard deviation barely moved — 0.0033 at both n=3 and n=6. What changed is
that *t* fell from 4.303 to 2.571, so the intervals narrowed from ±0.0097 to
±0.0035, and the same real gap between the same two means became visible.
**Nothing about the effect changed; the resolution did.**

Note the direction: adding seeds made H1 *harder* to pass on the CI criterion,
necessarily, because narrower intervals overlap less. I wrote that asymmetry into
the amendment before running anything. That is the whole reason the flip is
reportable. A 3-seed SUPPORTED becoming a 6-seed NOT SUPPORTED, without a
committed stopping rule, is indistinguishable from sampling until you like the
answer. With one, it is just a better measurement superseding a worse one.

## The other result: a ceiling I withdrew, then got back

Running in parallel: 216 cells of matched BPTT across six data contracts, two
geometries, three widths, two budgets, three seeds. The gate was accuracy ≥ 0.80,
set because published e-prop — a *local* learning rule — reaches 0.808 on this
task.

**Zero of 216 cells cleared it.** Every other registered gate passed in all 216:
all 20 classes predicted, no majority-class collapse, healthy firing rates, no
non-finite events, zero saturation. Not a broken network. A network that trains
fine and tops out at 0.72.

I wrote that up as a ceiling. Then the convergence probe came back and said the
100-epoch budget was binding — accuracy was still climbing at e200 and e400 — so
the ceiling claim was withdrawn. What I had measured was BPTT-at-100-epochs, not
BPTT.

Then the convergence rule itself turned out to be broken, in a way I have written
about separately: it compared the *first and last* rungs of the ladder, so
extending the ladder could only ever make the gain larger. A rule for detecting
convergence that got monotonically further from declaring it the more evidence
you collected. Amended to ask about the final doubling instead, with the 0.01
threshold untouched — moving the number would have been the exact forking path
this apparatus exists to prevent.

Under the amended rule, e800:

| epochs | test acc | train loss | gain |
|---:|---:|---:|---:|
| 100 | 0.716431 | 0.2146 | — |
| 200 | 0.728357 | 0.0979 | +0.011926 |
| 400 | 0.734541 | 0.0456 | +0.006184 |
| 800 | 0.732774 | 0.0336 | **−0.001767** |

Across three seeds the final doubling buys **+0.000294** — 34× below the bound —
while training loss keeps falling about 6.4% per final decile in every seed. That
is not undertraining. That is overfitting, and the budget is sufficient.

## The scaling curve that wasn't

The best part came last. At the 100-epoch budget, width looked like a live axis:
h128 → h256 → h512 gave 0.659 / 0.675 / 0.693, about +0.017 per doubling and
mildly *accelerating*. A clean scaling story. Just build it wider.

At the converged budget it flattens:

| hidden | mean acc | gain |
|---:|---:|---:|
| 128 | 0.703180 | — |
| 256 | 0.721731 | +0.018551 |
| 512 | 0.736896 | +0.015165 |
| 1024 | **0.737780** | **+0.000883** |

Wider networks reach a given loss in fewer epochs. At a fixed short budget that
reads as a capacity advantage. Train every width to convergence and it
evaporates. **The width trend was substantially a budget artefact** — and it was
only visible as one after the budget axis was closed, because the two axes were
confounded the entire time.

Converged on both axes: **0.7378 ± 0.0007. Shortfall to the 0.80 gate: 0.0622.**

Exact gradients on this forward do not reach a bar that a *local* rule clears on
this task. The binding constraint is the architecture — no recurrence, no
threshold adaptation, no learned delays — and it now carries no budget and no
width qualifier. What it does still carry is one data contract and one geometry.

## What none of this says

This is one instrument, and a deliberately weak one: 0.7378 against a 0.939
reference, so nothing here is a state-of-the-art comparison. It says nothing
about architectures that could actually exploit timing — the recurrent,
adaptive-threshold arm produced zero usable cells at this width and remains
unmeasured, not refuted. The temporal decomposition ran at 100 epochs, which is
now *known* undertrained by 0.021, and whether the order effect grows or shrinks
with training is untested. Three seeds is three seeds; the e800 interval is wide.

One caveat I found while re-checking the cells and which is worth stating
plainly: the channel-shuffle does not leave the network's operating regime alone.
Hidden firing rate rises from 0.199 to about 0.26 and saturated fraction from
exactly zero to 0.023–0.035 across all six seeds — inside the registered ≤ 0.05
gate, but not nothing. Some unknown share of that 0.1248 is the network being
pushed toward saturation rather than pure information loss. The order effect is
clean: `bin-shuffled` has a saturated fraction of exactly zero in every seed, so
the H1 number is untouched by this. The synchrony magnitude should be read as an
upper bound until it is checked at matched activity.

And the effects are small in absolute terms. 0.0189 on a 0.7158 baseline.
Reliable is not the same as large.

## How to check it

Every number above re-derives from the raw per-cell JSON on disk — I recomputed
all of them independently rather than trusting the write-ups, including the
validity gates, both standard-deviation conventions, the 3-seed-versus-6-seed
flip, and the full 216-cell gate audit. The three `intact` cells in the temporal
campaign are **bit-identical** to the corresponding cells in the recorded matrix,
which is the check that the manipulation harness adds no artefact to its own
control arm. The instrument's determinism gate reproduces 13/13 cells
bit-identically across nine distinct compiled binaries, and 160 tests pass.

The weak link was never the arithmetic. It was that the pre-registrations, the
amendments, and the cells all sat in a working tree with nothing committed — so
the ordering that makes these results honest, *the rule was fixed before the
run*, was attested by prose and file timestamps rather than by anything
tamper-evident. The timestamps are consistent: the seed-extension amendment
predates the first new cell by 14 minutes, the convergence amendment predates the
e800 cells by an hour and a half. But mtimes are not evidence.

That is now fixed — the record is committed, and the working rule from here is
**register, commit, then run**. It is worth naming as a general point rather than
a housekeeping note: for work whose entire claim to trustworthiness is *"I wrote
the rule down first,"* the version-control history is not incidental to the
result. It is part of the result.
