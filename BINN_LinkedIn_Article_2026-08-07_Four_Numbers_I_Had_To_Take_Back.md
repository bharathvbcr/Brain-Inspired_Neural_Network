# Four Numbers I Had to Take Back, and One Prediction I Lost

## What happened when I actually ran the kill gates I published two weeks ago

Two weeks ago I published an article asking why brain-inspired neural networks haven't replaced ANNs. It ended with a promise rather than a result:

> I do not yet know whether this architecture is better than an ANN. I know what evidence I would accept against it.

I laid out six gates the project has to pass, said plainly that the implementation and its benchmark evidence did not exist yet, and invited people to attack the design before I built it.

This is the first instalment of that evidence. It is not the result anyone was waiting for, including me.

The short version: the central kill gate has already run once, on easy tasks, and the answer was negative in an interesting way. Chasing down *why* it was negative sent me back to build a proper measuring instrument first. Building that instrument took three weeks, cost me four retracted claims and one prediction I registered in advance and lost, and left me suspicious of some of my own earlier results that look much better than these do.

That last part is the one I would read if I were you.

## What the kill gate already said

Gate 2 is the one that matters. It asks whether a sparse-assembly learner using local credit assignment can get close to what backpropagation achieves on the same task, the same architecture and the same parameter budget. If it stays closer to the other local baselines than to the gradient reference, I stop the central program.

I ran a version of it in July, on deliberately easy tasks: a coincidence-detection problem and XOR, both binary, so chance sits at 0.5, across twelve to twenty seeds. The gate required a primary mean of at least 0.65 and a lower confidence bound on the closed gap above 0.5.

On a matched dense network of leaky integrate-and-fire neurons, three local rules cleared it. Direct feedback alignment with graded error and fixed random feedback reached 0.9387, with a gap lower bound of 0.6894. A REINFORCE rule multiplied by per-neuron feedback reached 0.9200, bound of 0.6846. An online learned feedback rule reached 1.0000, bound of 0.9988. Meanwhile the rule the production system actually used, a broadcast plus-or-minus-one three-factor rule, sat at exactly chance, 0.5000, with a bound of 0.0000.

So far, so encouraging. Local learning works, the production rule was just badly designed, swap it and move on.

Then I ran the same thing on the live event-driven engine, which is the system I actually care about, and everything failed.

Not narrowly. Twelve variants, each one a serious attempt at a rescue: structured feedback matrices, capacity sensitivity, eligibility traces crossed with REINFORCE, live direct feedback alignment, soft winner-take-all competition, continuous feedback, finite thresholds. Several of them cleared the accuracy floor comfortably, at 0.7325, 0.7262 and 0.7125. Not one cleared the gate. The best gap lower bound across the entire sweep was 0.3127, against a threshold of 0.5. The canonical protocol came in at 0.4912 with a bound of negative 0.0048.

**A rule that passes on the matched dense substrate dies on the live sparse engine.** That gap is the most interesting thing in the project, and it is written down in my own results table as a disclaimer rather than as a finding.

Nobody has isolated why. The candidates are all named in the appendix, and they are specific: a sticky last-spike variable, a partial membrane reset, muted thresholds, hard winner-take-all competition instead of soft. Any one of them could be the whole effect. None of them has been tested on its own.

Along the way that campaign did produce two results I still like.

The first is a decomposition. I wanted to know what the failing broadcast reward was actually missing, so I built a ladder where each rule adds one ingredient. The plain plus-or-minus-one reward sits at 0.53, which is chance. Grading the reward, so it carries magnitude rather than just sign, lifts it to 0.81. Signing it with the exploratory action instead, leaving it binary, lifts it to 0.73. Doing both and adding per-neuron feedback gives 0.84. A full supervised error gives 0.94, and routing that error through random feedback gives 0.95. So the production rule was not failing because credit has to be local. It was failing because a binary unsigned scalar is an impoverished teaching signal, and *either* repair substantially rescues it.

That result did not survive the port cleanly, which is worth saying. In the Rust implementation, gradation alone stopped working, and only the directional version with per-neuron feedback carried over. The preview and the production harness disagreed, and the production harness wins.

The second is XOR. Broadcast credit gets 0.5008, which is chance. Direct feedback alignment gets 0.8267. Exact gradient descent gets 0.7733. A local rule beating backpropagation on the task specifically chosen to require locality is a small, clean, genuinely surprising result, and it is the one piece of evidence in that campaign I would still defend in a hostile room.

There is also a depth result: one hidden layer of learned feedback reaches 1.0000, and adding a second collapses it to 0.4525, below the floor, while the gradient ceiling stays at 0.9895. That is exactly the failure mode my published article named as the first reason local learning might not scale. I am not going to claim it yet, because on the same family of tasks plain broadcast credit also handles the two-layer case at 0.8158, so the collapse is not cleanly attributable to locality.

## Why any of that sent me back to the workbench

Look at what those numbers have in common. Chance is 0.5. The tasks are toy. And the gate is a *comparison*, which means its verdict is only as trustworthy as the reference arm I am comparing against.

There is a failure mode in this kind of project that I wanted to design out from the start. The baseline gets built carelessly, the hypothesis beats it, and the result turns out to be an artefact of the comparison rather than a fact about the world. You can find that paper in almost every subfield. The mirror image is just as bad: the baseline is built carelessly in the other direction, the hypothesis loses, and you kill a good idea for no reason.

The only defence I know is to hold the reference arm to exactly the standard you would demand of the thing you are hoping for. Pre-register it. Give it real compute. Let it fail your gates. Take your own numbers back when the evidence says to.

So before running Gate 2 on anything harder than XOR, I built the reference arm properly. It is a matched backpropagation-through-time network on a real task, and everything from here to the end of the instrument story is about *that*, not about BINN.

It took three weeks and it did not go well.

## The setup: a deliberately weak network

The object under test is not impressive, and that is the point.

One hidden layer of leaky integrate-and-fire neurons. Fixed thresholds. No recurrence, no adaptive threshold, no learned delays. Trained by backpropagation-through-time on the Spiking Heidelberg Digits dataset, which is spoken digits recorded as spike trains: twenty classes, 8,156 training samples.

It reaches about 0.74 accuracy. The published state of the art on this task is about 0.94.

I did not build it to compete. I built it as an instrument, something simple enough that when it fails you can say precisely what failed. The research question was never whether I could win a benchmark. It was this: if you give this architecture the strongest possible credit assignment, how far does it get, and what is it actually using to get there?

I pre-registered a bar of 0.80 accuracy. That number is not arbitrary. Published e-prop, which is a *local* learning rule, reaches 0.808 on this task. So the bar encodes the same question Gate 2 asks, in a harder setting: can exact gradients on this simple forward match what a local rule achieves on a better one?

## 216 experiments, zero passes

The matrix was 216 cells. Six data encodings, two input geometries, three widths, two training budgets, three random seeds.

Not one cell cleared 0.80.

What made it interesting was the shape of the failure. Every *other* registered gate passed in all 216 cells. All twenty classes predicted. No majority-class collapse. Healthy firing rates. Zero saturated neurons. No numerical blowups.

This was not a broken network. It was a healthy network that trains beautifully and simply stops at 0.72.

A clean negative result. I wrote it up as a ceiling.

## Taking it back

Then the convergence probe came back.

I had trained for 100 epochs. At 200 epochs accuracy was 0.728. At 400 it was 0.735. Still climbing.

My "ceiling" was measuring my patience, not the architecture.

The pre-registered rule was unambiguous about what to do. The ceiling claim had to be withdrawn or re-measured at the longer budget. I withdrew it. Every use of the word "ceiling" came out of the document, and the title changed to name the budget explicitly, because what I had actually measured was BPTT-at-100-epochs, which is a different and much less interesting thing than BPTT.

That is the part of research nobody posts about. You had a result. Now you have a footnote.

## The rule that could never say yes

So I extended the ladder to 800 epochs, ran it back through the registered convergence rule, and the rule said UNDERTRAINED again.

I went and read the rule.

It compared the first and last rungs of the ladder. Which means that as you extend the ladder, the first rung stays pinned at 100 epochs while the last one keeps climbing, so the measured "gain" gets monotonically *larger* the more evidence you collect.

A rule designed to detect convergence that got further from declaring it with every experiment I ran.

It had never been capable of returning "converged". It was decoration.

The fix was not to move the threshold. Moving a registered threshold after seeing your data is precisely the failure this whole apparatus exists to prevent. The fix was to change the question and keep the number. Instead of asking whether the whole ladder gained more than 0.01, which is trivially true and increasingly so, ask whether the *next doubling* still buys anything.

Same 0.01 constant. Different pair compared. Registered before running the new cells.

## The ceiling comes back, and a scaling law dies

Under the amended rule, one seed's ladder runs 0.7164 at 100 epochs, 0.7284 at 200, 0.7345 at 400, and 0.7328 at 800, while training loss falls from 0.215 to 0.034 across the same span.

The verdict is taken across three seeds, not one. Doubling from 400 to 800 epochs buys 0.000294, which is thirty-four times below the threshold, while training loss keeps falling about 6.4% per final decile in every single seed.

That is not undertraining. That is overfitting. The budget is sufficient. The ceiling is real.

Then came the part I did not expect.

At the short budget, width had looked like a live scaling axis. Going 128 to 256 to 512 hidden units gave 0.6676, then 0.6842, then 0.7042. That is roughly 0.017 per doubling, and mildly *accelerating*: the second doubling bought more than the first. A clean scaling story, and one that invited an obvious extrapolation. The next doubling should be worth another 0.02.

Re-run at the converged budget, the same widths give 0.7032, 0.7217 and 0.7369, and the doubling to 1024 units gives 0.7378. The gains are 0.0186, then 0.0152, then 0.0009.

The extrapolation was off by more than twentyfold.

The mechanism is simple once you see it. Wider networks reach a given loss in fewer epochs. At a fixed short budget that reads as a capacity advantage, and the largest width you tested is the one furthest from having finished. Train every width to convergence and the advantage evaporates.

My scaling law was a budget artefact. And it was only visible as one *after* the budget axis was closed. The two axes had been confounded the entire time.

Converged on both axes, the number is 0.7378, and the shortfall to the bar is 0.062.

## The prediction I wrote down, and lost

Six days later the same confound appeared to be running in the other direction, and this time I saw it coming.

The instrument takes 700 cochlear channels. One input geometry sums every five adjacent channels into 140 inputs. The other presents all 700 raw. At the short budget the 700-channel version was consistently *worse*, by 0.0259 at the anchor configuration, and the gap grew with width.

That is the identical shape as the width curve I had just been fooled by. The 700-channel geometry has five times the input parameters to fit from the same 8,156 samples, so it needs more epochs to reach the same loss. At a fixed short budget it would look worse for a reason that has nothing to do with the information it carries.

Having been caught once, the temptation was to assume it and move on. Instead I registered the prediction, with its mechanism and a number:

> The short-budget geometry gap is substantially a budget artefact, and will narrow at convergence. Threshold: the gap at 400 epochs must come in below 0.025913, the measured short-budget gap. If the gap is information-bearing it should be stable or grow. A *stable* gap counts as evidence against the mechanism.

In the same document, before any cell ran, I also fixed the rule for when I would be allowed to stop. If the 95% confidence interval came back entirely below 0.78, no further axis-closure runs were required, because no single doubling in this instrument has ever bought more than 0.0186, so two doublings could not bridge a shortfall that large.

Three cells. Paired on epoch shuffle order with the existing runs, so the contrast could not come down to a difference in how the data was shuffled.

The 140-input geometry finished at 0.7369, with a confidence interval of 0.7317 to 0.7421. The 700-input geometry finished at 0.7086, interval 0.6984 to 0.7188. The gap at convergence is 0.0283. It did not narrow. It widened slightly, and the intervals do not overlap.

The prediction failed.

The mechanism I described is real. Five times the input parameters genuinely does slow fitting. It simply was not what was driving this gap. Presenting all 700 channels separately does not recover information that the summing discards. It loses accuracy, which is the opposite of the concern the experiment was run to rule out.

The interval's upper bound came in at 0.7188, below the pre-fixed 0.78, so the stopping rule closed the question rather than my judgement closing it after seeing the answer.

I want to be precise about what this taught me, because it is not simply that I was wrong. It is narrower and more useful. **Being burned by a confound is not a licence to assume it everywhere.** Six days earlier a scaling curve had turned out to be an artefact of a short budget. That made me *expect* the next gap to be one too, and an expectation formed by a recent burn feels exactly like insight from the inside. Writing it down as a falsifiable prediction with a number is the only thing that converted a confident wrong intuition into a cheap three-cell refutation instead of a claim.

The ceiling survives. Its scope qualifier narrows from "one contract, one geometry" to "one contract".

## What is the network actually using?

Running alongside all of this was a different question.

This network is fed spike trains, data whose whole premise is that *timing* carries information. Does it use any of it?

The standard way to test this is a shuffle control. Scramble time, see what breaks. It is a good instinct, and it is almost always run in a way that cannot answer the question.

Because "timing" is not one thing. There are at least two. There is order, meaning the sequence in which things happen. And there is cross-channel synchrony, meaning which channels fire *together*.

Now consider two ways to scramble a spike raster while keeping every channel's total spike count exactly fixed.

Shuffle the time bins using one permutation shared by every channel. Order is destroyed. But channels that fired together still fire together, so synchrony survives perfectly.

Shuffle the time bins using an independent permutation per channel. Order dies, and synchrony dies with it.

The usual control does the second one and reports that timing matters. But it destroyed two things at once, so the number it returns is a sum you cannot decompose.

Run both, and the difference isolates synchrony.

Four conditions, six seeds, trained *and* tested on manipulated data rather than perturbed only at test time, which would have confounded the effect with distribution shift.

Playing the digits backwards, which destroys the direction of time entirely, costs 0.0032. Destroying temporal order costs 0.0189. Destroying order and cross-channel synchrony together costs 0.1437. Subtract, and synchrony on its own is worth 0.1248, which is 6.6 times the order effect.

Direction is worth nothing. Order is worth a little. Synchrony is worth almost all of it.

The reversal number is the one I keep coming back to. Playing a spoken digit backwards destroys the entire global sequence, and this network barely notices, while a shuffle that preserves local structure but scrambles bin order costs six times more. Whatever it is reading, it is not a sequence. It is reading coincidence.

## The flip

Here is where the story turns back on itself.

I had pre-registered the hypothesis that this network is a rate coder, meaning it ignores order entirely. I defined it with two criteria that both had to hold: the accuracy difference had to sit within an equivalence bound of 0.02, *and* the 95% confidence intervals had to overlap.

At three seeds it passed. The difference was 0.0199, inside the bound by 0.62%, with the intervals overlapping by a hair. I could have published "this network is a rate coder" that afternoon.

A boundary pass on three seeds is not something to build a claim on. So I registered an amendment with a binding stopping rule:

> Exactly three seeds are added. The verdict is recomputed once, on all six, and reported whichever way it falls. No further seeds will be added, regardless of outcome.

I also wrote down, in advance, that this would make my own hypothesis *harder* to pass, because more seeds narrow the confidence intervals, and narrower intervals overlap less.

Then I ran them. The verdict flipped to NOT SUPPORTED.

And the mechanism is the opposite of what "my result changed when I added data" usually implies. The scatter barely moved. The standard deviation was 0.0039 at three seeds and 0.0033 at six. What changed is that the critical t multiplier fell from 4.303 to 2.571, the intervals tightened from roughly ±0.0098 to ±0.0034, and the same real gap between the same two means became visible.

Nothing about the effect changed. The resolution did.

That flip is only reportable because the seed count was committed first. A three-seed pass becoming a six-seed failure, without a stopping rule fixed in advance, is indistinguishable from sampling until you like the answer. With one, it is just a better measurement superseding a worse one.

## The disagreement that turned out to be the finding

The two criteria disagreed, and that disagreement is more interesting than either verdict on its own.

On the equivalence bound, the difference of 0.0189 passes, by 0.0011. On the interval test, the intervals are disjoint by 0.0120, and it fails.

They disagree because they are asking different questions.

The equivalence bound asks whether the effect is big enough to care about. It is not. It sits below the threshold I registered for practical negligibility.

The interval test asks whether the effect is real. It is, unambiguously. All six seeds are positive, ranging from 0.0150 to 0.0203.

The honest summary is neither "order matters" nor "order doesn't matter". It is that **order matters a little, and consistently**, and both halves of that sentence are load-bearing. A study that ran only one of the two tests would have reported half the story as the whole story, and I would never have known which half I had.

## Does any of that survive convergence?

Everything in the last two sections was measured at 100 epochs, which by then I knew to be undertrained by 0.021 accuracy. That is the first objection any reviewer would raise, and it is a fair one. The contrast is internally valid, since every condition shares the budget, but whether the order effect survives, grows or vanishes with training was untested.

So I re-ran the whole thing at the converged budget. Twenty-four cells, six seeds, zero aborts. Before running it I named all four possible outcomes and what each would mean, so that whichever one landed could not be narrated after the fact.

The verdict is identical. The difference is 0.0127, comfortably inside the 0.02 equivalence bound, and the intervals are still disjoint. Still NOT SUPPORTED, reached the same way. Real, and small, at both budgets. The budget objection is discharged.

But the components moved, and not in the direction I would have guessed. The order effect shrank from 0.0189 to 0.0127. The synchrony increment *grew*, from 0.1248 to 0.1336. The cost of playing digits backwards grew too, from 0.0032 to 0.0091. The ratio of synchrony to order went from 6.6 times to 10.5 times.

Training moves the solution further toward synchrony and away from order. The decomposition is not merely preserved at convergence. It sharpens.

That last observation was **not registered**, and I am labelling it accordingly. It is descriptive, it is a hypothesis for future work, and the direction should be checked at a third budget before anyone leans on it, including me.

## The arm that still doesn't work

One thing has resisted everything.

The recurrent, adaptive-threshold variant, which is the architecture you would actually reach for if you wanted to exploit timing, produces exploding gradients that abort training. Five interventions, each recorded with its reason.

Rescaling the recurrent initialisation moved the failure around non-monotonically across three orders of magnitude, and the ranking did not survive reseeding. Switching the gradient-norm logging to double precision fixed the *record* and, by construction, touched nothing about the dynamics. Batch gradient clipping was never even reached, because the abort fires on a per-sample gradient upstream of it. Halving the width failed too, with two of twelve cells aborting at optimizer steps 374 and 727.

The fifth is the only one that moved anything, and it is the one worth explaining. The surrogate derivative has a peak gain of 2.5 per timestep. Multiply that through a recurrent block over several hundred timesteps and a per-step gain above 1 is enough on its own to explain the overflow. So the value to test was not tuned. It was the one that makes the peak per-timestep gain exactly 1.0, the boundary between contraction and expansion, chosen before running anything, with a registered commitment to one value, three seeds, and no sweep.

All three seeds now reach the end of training with zero non-finite events and a large monotone drop in loss, at accuracies between 0.44 and 0.51.

And it should not be reported as a fix. A healthy feed-forward run peaks at a gradient norm of about 0.15. These three peak at 3.08e10, 1.17e12, and 3.93e33. The last of those is under five orders of magnitude from overflowing single precision, and the spread across three seeds spans twenty-three orders of magnitude. The intervention moved the arm from "aborts" to "completes". It did not move it to "numerically healthy", and one bad trajectory would put it back.

Two things travel with any number this configuration ever produces. The gradient is not the registered one, so nothing from it is comparable to the 216 cells or to the 0.7378 ceiling. And no smaller value will be tried to chase the margin. If it matters enough to fix, the next step is truncated backpropagation, which bounds the *number* of compounding steps rather than their size, and is a different intervention rather than more of this one.

## The part that actually got faster

Not everything in three weeks was subtraction.

The recurrent training cell went from 193.0 seconds to 30.6, a factor of 6.3, and it came out of correctness work rather than optimisation work. The forward and backward passes of that arm disagreed, which is to say the gradient was not the gradient of the forward. Fixing the disagreement made it both correct and much faster.

The largest single waste found in the hot path is a plasticity step that was deep-copying its entire sparse connectivity matrix, in both row and column form, on every single update, purely to avoid a borrow conflict. At the sizes this runs at that is roughly thirty megabytes of memory copying per step, for nothing. There are seven more ranked candidates behind it, covering the event queue, a per-call memset, three exponentials per cell update whose arguments are provably constant, and a sort whose ordering is unobservable.

Two caveats, because a performance section without them is marketing. That audit was done by reading code, with no compiler available in the environment it ran in, so nothing in it was compiled, benchmarked or profiled. Its own opening paragraph says the ranking is a hypothesis and that the first action should be to profile rather than to start at the top and work down. And the one change with a genuinely silent failure mode, a bitmask rewrite of the event queue where a stale bit would make the simulation quietly drop events, now carries a property-based test that checks the fast path against the brute-force path after every single operation.

The feed-forward path also got 4.5% *slower* in the same pass, and that regression is documented rather than hidden, because removing it needs a structural change with its own verification burden.

There is one performance result I have to withdraw rather than report. An earlier scaling table showed GPU throughput against CPU across four, eight and sixteen areas. The backend selector was a boolean that the kernel never actually read, so both columns executed byte-identical CPU code. The reported speedups of 0.97, 0.62 and 0.99 are measurement noise between two runs of the same thing. No GPU code has ever executed in this repository. The CPU column is a valid single-backend scaling series and the other two columns should never be cited.

## The scores I trust least are my best ones

Now the part I have been circling.

Go back to that July campaign. Three arms score exactly 1.0000, with a standard error of exactly 0.0000 across twenty seeds. In each case they equal or beat the exact-gradient ceiling they are measured against: online learned feedback at 1.0000 against a gradient ceiling of 0.9930, single-hidden-layer learned feedback at 1.0000 against 0.9895. And on a five-class version of the spoken-digits task, graded feedback alignment reports 1.0000 while the true e-prop reference on the same harness reports 0.2140, against a chance level of 0.2000.

A local rule matching exact gradients is surprising and worth investigating. A local rule *beating* exact gradients, at zero variance, while the gradient reference on the same harness sits at chance, is not a discovery. It is the signature of a harness that is measuring something other than what it claims to measure.

So I went and audited it. What I found is worse than a bug, and more interesting.

I had already caught it. On the twenty-fifth of July I found that two of those harnesses were computing the closed gap without a clamp and without a check that the reference was even separated from chance, and that this is how values like 1.0155, meaning "closed 102% of the gap to the ceiling", got into shipped reports. I wrote the fix. I wrote a paragraph in the source explaining precisely why a value above 1 is a harness warning rather than a result. I made the new code refuse to emit a PASS while that warning is present. I bumped the protocol version.

And then I never re-ran the experiment.

The report on disk is still the old protocol version. It still says 1.0155. It still says PASS. And that PASS is currently cited in six documents, including the abstract of the paper draft, where it appears in bold as the headline matched result.

The version number was sitting in the report header the whole time, one digit off from the source that would have refused to produce it.

Two of the three suspicious arms turn out to be that same story: fixed in code, never regenerated. The third is worse. It is the five-class digits sweep, its report matches its source exactly, and its entire verdict logic is a per-arm check against chance. No ceiling comparison of any kind. So an arm at 1.0000 and a reference at 0.2140 both print "beats chance" and the report ships without comment. The mechanism is even hypothesised in my own codebase, in a comment in a *different* file, which names this suite by name as the example of a ceiling whose credit signal is too weak to be a ceiling. The instrumentation that would have tested it was added to the wrong binary.

I take this seriously rather than shrugging because of the precedent. The audit of the instrument in this article turned up ten defects. Five of them are the same class: code reporting success while measuring nothing. The numerical guard that was supposed to catch diverging runs had a counter that was never incremented, so the only protection in the pass predicate was silently vacuous across all 296 cells. A flag recording whether a model had been trained was hardcoded to false, so every trained-weights probe asserted it was untrained. Passing zero epochs was accepted and written out as a completed run. Two different implementations of the same argmax disagreed on ties, on negative zero, and on NaN, and one of them fed three fields of the determinism gate. A linter found none of them. They are semantic, and you only catch them by attacking your own code as though someone else wrote it and wanted to fool you.

That audit happened because I was building a *baseline* and forced myself to hold it to the standard of the hypothesis. The arms scoring 1.0000 have had none of it.

## What I'd take from this

**A short budget makes everything look like a scaling law.** My width curve was clean, monotone and accelerating. It was an artefact of stopping early. If you are reading a scaling result, ask what happens at convergence. If you are producing one, close the budget axis before you believe your own curve.

**Having been fooled by a confound is not evidence that the confound is everywhere.** The second time I saw that pattern I was sure I knew what it was. Writing the prediction down with a threshold is what stopped an overcorrection from quietly becoming a claim.

**"Significant" and "meaningful" are different measurements, and you need both.** One number, two tests, two legitimate answers. Reporting either alone would have been true and misleading.

**Write the stopping rule down before you need it.** Not because it makes you virtuous, but because it is the only thing that makes an inconvenient flip publishable rather than embarrassing.

**Audit the results you like most, first, and then check that the audit landed.** Almost every hour of scrutiny in this project went into a number that disappointed me. The numbers that delighted me were filed and cited. In the one case where I did catch a flattering result, I fixed the code, wrote up why, and then left the old report in place and kept citing it for two weeks. Finding a defect and repairing a record are two different jobs, and only the first one is satisfying.

**Re-derive your own numbers from the raw data.** I recomputed every figure in the instrument sections above from the per-cell records rather than trusting my own write-ups. Everything reproduced. But the exercise surfaced a caveat I had missed. The synchrony-destroying shuffle also pushes the hidden layer's saturated-neuron fraction from exactly zero to about 3%, and mean firing rate from 0.21 to 0.28. That is inside my registered gate, but it is not nothing. So some unknown share of that 0.1336 is an activity-regime effect rather than pure information loss, and it belongs in the write-up as an upper bound. It is unchanged at the converged budget. The order effect is untouched by it, because that shuffle leaves saturation at exactly zero in every seed.

## What none of this claims

The July gate ran on toy tasks where chance is 0.5. Nothing there is a statement about hard problems, and the live-engine failure is a failure on easy tasks, which is informative but narrow.

The instrument sections are about a matched backpropagation baseline, not about BINN, sparse assemblies or local learning. That baseline is one deliberately simple network at 0.7378 against a published reference of about 0.94, so nothing there is a state-of-the-art comparison. It says nothing about architectures built to exploit timing. The recurrent variant is now marginally measurable rather than unmeasurable, which is a different claim from working. Two input geometries are closed at convergence, but the six timing contracts are closed only at the short budget, so "one contract" remains a live scope limit. And the effects are small in absolute terms: 0.0127 on a baseline of 0.7374. Reliable is not the same as large.

Two pieces of bookkeeping that would be easy to omit and shouldn't be. The harness authorization flag covering the original 216-cell matrix went false on the third of August and has not been restored, so every extension since inherits that as a recorded threat rather than a clean bill of health. And the bit-identical determinism suite is mid-rerun on the current binary, at seven of thirteen fixtures with zero failures so far. Seven is not thirteen, and I am not going to describe it as though it were.

## What happens next

Seven things, in the order I intend to do them.

**Re-run the fixes I already wrote.** The audit above is done; the repair is not. Three experiments need re-running against the code that is already in the repository, the ceiling-inversion guard needs porting into the fourth, and the six documents citing a forbidden PASS need correcting or withdrawing. None of this is new science. It is the difference between having fixed something and having fixed it, and I am putting it first because it can invalidate everything below.

**Ask the ceiling-health question of the two arms still standing.** This is the one that worries me. The transfer gap does not depend on the compromised arm, because the two matched passes that carry it went through a code path that always clamped. But both of them also beat their own gradient reference, at 0.9387 against 0.8963 and 0.9200 against 0.8887, and on one of those schedules a broadcast *control* reaches 0.9863 against that same 0.8963. Two arms over the ceiling and one of them is a control. That is milder than the case above and it is bounded rather than unbounded, but a gradient reference that a control beats by 0.09 on a binary task looks undertrained, and if it is, then the matched side of the transfer gap is measuring something other than what I think. This has to be settled before the next item is worth designing.

**Commit the record.** The preregistrations, the amendments and the cells are still sitting uncommitted in a repository whose last commit predates most of this work. The ordering that carries all the epistemic weight, that a rule was registered *before* the run it governs, is currently attested by prose and file modification times. Those times are consistent, and in a couple of cases I can point at the exact gap: the seed amendment was written fourteen minutes before the first new cell, the convergence amendment ninety-eight minutes before the 800-epoch cells. But modification times are not tamper-evident, and an article built entirely on "I wrote it down first" should not rest on evidence I could trivially forge.

**Decompose the transfer gap.** This is the real scientific prize sitting in the repository. A local rule passes on the matched dense substrate and fails on the live sparse engine, and there are four named suspects: the sticky last-spike variable, the partial membrane reset, the muted thresholds, and hard winner-take-all competition. They have never been tested individually. That is a preregistration with one factor per arm, and it is the same move that made the order and synchrony result work.

**Attack my own biggest caveat.** The 0.1336 synchrony number is an upper bound, because the shuffle that destroys synchrony also shifts the network into a slightly different activity regime. I know the shape of the fix and it is not a re-analysis, it is a new experiment: a registered input-scale normalisation that holds firing statistics matched across conditions, then a re-measurement. If the effect survives at matched activity, the decomposition stands. If it collapses, then one of the headline findings of this article was substantially an artefact.

**Ask the relative question of the recurrent arm.** With the arm now completing, the question I can defensibly ask is not what it reaches, since a ceiling measured at a modified surrogate gain is a ceiling for that method and nothing more. The defensible question is relative: does recurrence degrade *less* under shuffling than the feed-forward arm does? That needs a matched feed-forward baseline at the same surrogate gain, which is part of the measurement rather than an optional extra.

**Then run Gate 2 on a hard task.** That is what the instrument was built for. The baseline now exists, it is measured, its budget and width and geometry axes are closed, and it is worse than a local learning rule achieves on a better architecture. Which means it is a reference the hypothesis can meaningfully lose to, and that is all a baseline has to be.

Roughly eight thousand lines of the actual brain-inspired system have never had the treatment the baseline just got. Whatever Gate 2 eventually says, it is worth nothing until they have. I expect that audit to be worse than this one, because I wrote that code while I still believed the idea.

## Where I want to be wrong

The last article asked for objections that would change a gate, reveal a missing baseline, or identify a cheaper falsification. Same ask, now with something concrete to aim at.

Is the order and synchrony decomposition already standard somewhere, and have I reinvented it badly?

What is the cheapest experiment that would show the synchrony effect is an activity-regime artefact rather than an information effect, given that this is the one caveat I have not been able to design my way out of?

Has anyone else measured a local rule that clears a matched-substrate gate and then fails on an event-driven engine, and did you find out which mechanism was responsible?

And if you have seen a learning rule beat exact gradients at zero variance, what turned out to be wrong with the harness?

Negative results, counterexamples and papers are all welcome. If any part of this still protects the idea instead of testing it, tell me where.

#SpikingNeuralNetworks #NeuromorphicComputing #ComputationalNeuroscience #MachineLearningResearch #ReproducibleResearch
