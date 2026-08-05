# The Number I Had to Take Back — Twice

## What a spiking network taught me about the difference between an effect being small and an effect being unreal

Three seeds said my network was a rate coder.

Six seeds said it wasn't.

I had written down, in advance, that six would be the answer. That single decision — made before I ran seeds four, five and six — is the reason I can tell you this story instead of quietly publishing the version I preferred.

This is a story about two results from a small spiking neural network, and about how close I came to reporting both of them wrong.

---

## The setup: a deliberately weak network

The object under test is not impressive, and that is the point. One hidden layer of leaky integrate-and-fire neurons. Fixed thresholds. No recurrence, no adaptive threshold, no learned delays. Trained by backpropagation-through-time on the Spiking Heidelberg Digits dataset — spoken digits, recorded as spike trains.

It reaches about 0.74 accuracy. The published state of the art on this task is 0.94.

I did not build it to compete. I built it as an **instrument** — something simple enough that when it fails, you can say precisely what failed. The research question was never "can I win a benchmark." It was:

> If you give this architecture the strongest possible credit assignment, how far does it get — and what is it actually using to get there?

I pre-registered a bar: accuracy ≥ 0.80. That number is not arbitrary. Published **e-prop**, a *local* learning rule, reaches 0.808 on this task. So the bar encodes a real question: can exact gradients on this simple forward match what a local rule achieves on a better one?

## Act I: 216 experiments, zero passes

The matrix was 216 cells — six data encodings, two input geometries, three widths, two training budgets, three random seeds.

**Not one cell cleared 0.80.**

What made it interesting was the shape of the failure. Every *other* registered gate passed in all 216 cells. All twenty classes predicted. No majority-class collapse. Healthy firing rates. Zero saturated neurons. No numerical blowups.

This was not a broken network. It was a healthy network that trains beautifully and simply stops at 0.72.

A clean negative result. I wrote it up as a ceiling.

## Act II: taking it back

Then the convergence probe came back.

I had trained for 100 epochs. At 200 epochs, accuracy was 0.728. At 400, it was 0.735. Still climbing.

My "ceiling" was measuring my patience, not the architecture.

The pre-registered rule was unambiguous about what to do: *the ceiling claim must be withdrawn or re-measured at the longer budget.* I withdrew it. Every use of the word "ceiling" came out of the document, and the title was changed to name the budget explicitly — because what I had actually measured was BPTT-at-100-epochs, which is a different and much less interesting thing than BPTT.

That is the part of research nobody posts about. You had a result. Now you have a footnote.

## Act III: the rule that could never say yes

So I extended the ladder to 800 epochs, ran it back through the registered convergence rule, and the rule said **UNDERTRAINED** again.

I went and read the rule.

It compared the **first and last rungs of the ladder**. Which means: as you extend the ladder, the first rung stays pinned at 100 epochs while the last one keeps climbing — so the measured "gain" gets *monotonically larger* the more evidence you collect.

A rule designed to detect convergence that got further from declaring it with every experiment I ran.

It had never been capable of returning "converged." It was decoration.

The fix was not to move the threshold. Moving a registered threshold after seeing your data is precisely the failure this whole apparatus exists to prevent. The fix was to change the **question** and keep the number: instead of *"did the whole ladder gain more than 0.01?"* — trivially yes, and increasingly so — ask **"does the next doubling still buy anything?"**

Same 0.01 constant. Different pair compared. Registered before running the new cells.

## Act IV: the ceiling comes back, and a scaling law dies

Under the amended rule:

| epochs | test accuracy | training loss |
|---:|---:|---:|
| 100 | 0.7164 | 0.2146 |
| 200 | 0.7284 | 0.0979 |
| 400 | 0.7345 | 0.0456 |
| 800 | 0.7328 | 0.0336 |

Across three seeds, doubling from 400 to 800 epochs buys **+0.000294** — thirty-four times below the threshold — while training loss keeps falling about 6.4% per final decile in every single seed.

That is not undertraining. That is overfitting. The budget is sufficient. The ceiling is real.

Then came the part I did not expect.

At the short budget, **width** had looked like a live scaling axis. 128 → 256 → 512 hidden units gave 0.659 → 0.675 → 0.693. About +0.017 per doubling, and mildly *accelerating*. A clean scaling story. Just build it wider.

Re-run at the converged budget:

| hidden units | accuracy | gain |
|---:|---:|---:|
| 128 | 0.7032 | — |
| 256 | 0.7217 | +0.0186 |
| 512 | 0.7369 | +0.0152 |
| **1024** | **0.7378** | **+0.0009** |

It flattens completely.

The mechanism is simple once you see it: **wider networks reach a given loss in fewer epochs.** At a fixed short budget, that reads as a capacity advantage. Train every width to convergence and the advantage evaporates.

My scaling law was a budget artefact. And it was only visible as one *after* the budget axis was closed — the two axes had been confounded the entire time.

**Converged on both axes: 0.7378. Shortfall to the bar: 0.062.**

Exact gradients on this forward do not reach a bar that a *local* rule clears on a better forward. The binding constraint is the architecture, not the credit assignment — and that conclusion now carries no budget qualifier and no width qualifier.

## Act V: what is the network actually using?

Running alongside all of this was a different question. This network is fed spike trains — data whose whole premise is that *timing* carries information. Does it use any of it?

The standard way to test this is a shuffle control: scramble time, see what breaks. It is a good instinct, and it is almost always run in a way that cannot answer the question.

Because "timing" is not one thing. There are at least two:

- **Order** — the sequence in which things happen.
- **Cross-channel synchrony** — which channels fire *together*.

Now consider two ways to scramble a spike raster while keeping every channel's total spike count exactly fixed:

**Shuffle time bins using one permutation shared by every channel.** Order is destroyed. But channels that fired together still fire together — synchrony survives perfectly.

**Shuffle time bins using an independent permutation per channel.** Order dies *and* synchrony dies with it.

The usual control does the second one and reports "timing matters." But it destroyed two things at once, so the number it returns is a sum you cannot decompose.

Run both, and the difference isolates synchrony.

## The result

Four conditions. Six seeds. Trained *and* tested on manipulated data — not a test-time perturbation, which would confound the effect with distribution shift.

| what I destroyed | accuracy cost |
|---|---:|
| direction of time (played backwards) | **0.0032** |
| temporal order | **0.0189** |
| order **and** cross-channel synchrony | **0.1437** |
| **→ synchrony alone** | **0.1248 — 6.6× the order effect** |

Direction is worth nothing. Order is worth a little. **Synchrony is worth almost all of it.**

The reversal number is the one I keep coming back to. Playing a spoken digit backwards destroys the entire global sequence, and this network barely notices — while a shuffle that preserves local structure but scrambles bin order costs six times more. Whatever it is reading, it is not a sequence. It is reading coincidence.

## Act VI: the flip

Here is where the story turns back on itself.

I had pre-registered the hypothesis that this network is a **rate coder** — that it ignores order entirely. I defined it with two criteria that both had to hold: the accuracy difference within an equivalence bound of 0.02, **and** overlapping 95% confidence intervals.

At three seeds, it **passed**. Difference of 0.0199, inside the bound by 0.62%, intervals overlapping. I could have published "this network is a rate coder" that afternoon.

A boundary pass on three seeds is not something to build a claim on. So I registered an amendment with a binding stopping rule:

> **Exactly three seeds are added. The verdict is recomputed once, on all six, and reported whichever way it falls. No further seeds will be added, regardless of outcome.**

I also wrote down, in advance, that this would make the hypothesis **harder** to pass — because more seeds narrow the confidence intervals, and narrower intervals overlap less.

Then I ran them. The verdict flipped to **NOT SUPPORTED**.

And the mechanism is the opposite of what "my result changed when I added data" usually implies. The standard deviation barely moved — 0.0033 at three seeds, 0.0033 at six. What changed is that the *t*-statistic fell from 4.303 to 2.571, the intervals narrowed from ±0.0097 to ±0.0035, and the same real gap between the same two means became visible.

**Nothing about the effect changed. The resolution did.**

That flip is only reportable because the seed count was committed first. A three-seed pass becoming a six-seed failure, without a stopping rule fixed in advance, is indistinguishable from sampling until you like the answer. With one, it is just a better measurement superseding a worse one.

## The disagreement that turned out to be the finding

The two criteria disagreed, and that disagreement is more interesting than either verdict alone:

| criterion | value | verdict |
|---|---|---|
| difference ≤ 0.02 | 0.0189 | **passes**, by 0.0011 |
| confidence intervals overlap | disjoint by 0.0120 | **fails** |

They disagree because **they are asking different questions.**

The equivalence bound asks: *is this effect big enough to care about?* No — it is below the threshold I registered for practical negligibility.

The interval test asks: *is this effect real?* Yes — unambiguously. All six seeds positive, ranging 0.0150 to 0.0203.

The honest summary is neither "order matters" nor "order doesn't matter." It is **order matters a little, and consistently** — and both halves of that sentence are load-bearing. A study that ran only one of the two tests would have reported half the story as the whole story, and I would never have known which half I had.

## What I'd take from this

**A short budget makes everything look like a scaling law.** My width curve was clean, monotone, and accelerating. It was an artefact of stopping early. If you are reading a scaling result, ask what happens at convergence — and if you are producing one, close the budget axis before you believe your own curve.

**"Significant" and "meaningful" are different measurements, and you need both.** One number, two tests, two legitimate answers. Reporting either alone would have been true and misleading.

**Write the stopping rule down before you need it.** Not because it makes you virtuous — because it is the only thing that makes an inconvenient flip *publishable* rather than embarrassing.

**Re-derive your own numbers from the raw data.** I recomputed every figure above from the per-cell records rather than trusting my own write-ups. Everything reproduced. But the exercise surfaced a caveat I had missed: the synchrony-destroying shuffle also pushes the network's saturated-neuron fraction from exactly zero to about 3% — inside my registered gate, but not nothing. So some unknown share of that 0.1248 is an activity-regime effect rather than pure information loss, and it belongs in the write-up as an upper bound. The order effect is untouched by this; that shuffle leaves saturation at exactly zero.

## What none of this claims

This is one deliberately simple instrument: 0.7378 against a 0.939 reference, so nothing here is a state-of-the-art comparison. It says nothing about architectures built to exploit timing — the recurrent, adaptive-threshold variant produced zero usable runs at this width and remains **unmeasured, not refuted**. The temporal experiment ran at a budget I now know to be undertrained by 0.021, so whether the order effect grows with training is an open question. And the effects are small in absolute terms: 0.0189 on a 0.7158 baseline. *Reliable* is not the same as *large*.

The broader project this instrument serves — whether sparse assemblies can make local credit assignment sufficient on hard tasks — is a separate question with its own kill gate, and nothing above speaks to it.

---

To anyone working on spiking networks, neuromorphic systems, or temporal credit assignment: I would genuinely like to know where this is wrong.

**Is the order/synchrony decomposition already standard somewhere and I have reinvented it badly? What is the cheapest experiment that would show the synchrony effect is an activity-regime artefact rather than an information effect? And which missing baseline would make any of this credible?**

Negative results, counterexamples, and papers all welcome. If any part of this still protects the idea instead of testing it, tell me where.

#SpikingNeuralNetworks #NeuromorphicComputing #ComputationalNeuroscience #MachineLearningResearch #ReproducibleResearch
