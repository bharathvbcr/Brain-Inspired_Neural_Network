# What a time-axis read-out consumes: temporal order on SHD, by difference-in-differences

> **Provenance, current as of 2026-08-25.** Seven results have been withdrawn from
> this package and **none of them appears above**. Four went during the
> 2026-08-19→22 record repair:
> the `track-b-rescue` v130 PASS (`1.0000`, gap LCB `0.9988`) is withdrawn — the arm
> reports `INVALID_HARNESS` at v131 and v132; the depth-collapse result is withdrawn
> — every depth-matched ceiling is at chance; `shd-scientific-sweep` is withdrawn —
> it never loaded SHD; and the `live-transfer-rescue` arms are `INVALID_HARNESS`.
>
> Three gradient references were found at or near chance on tasks their own
> treatments solve, and two are now diagnosed:
> `MatchedDeepGradient` collapses to silence, `ShdEpropCeiling` is a constant
> predictor by a different mechanism. Neither is used above.
>
> Three more went on 2026-08-25, when the matched-architecture suites were re-run
> under the repaired input scale: the **discrete EventProp-style spike-adjoint
> FAIL** (0.5000 → 0.9450 / 0.8900 PASS — a spike-adjoint method had had no
> spikes to differentiate through), and **both RL broadcast contrasts** (0.5250 →
> 0.9100, 0.5113 → 0.7962). The lead negative survived on both forward graphs.
> [`RESULT_2026-08-25_MATCHED_ARCH_RERUN.md`](RESULT_2026-08-25_MATCHED_ARCH_RERUN.md).
>
> **The four `c1-*` config hashes this draft used to cite are retired.**
> `MATCHED_INPUT_SCALE` was never mixed into them, so each named two different
> experiments either side of the repair. They no longer resolve, deliberately.
>
> `scripts/record_checks.sh` machine-checks the **SHD attention-campaign**
> numbers — **125 assertions**, recomputed from the archived cells by
> `scripts/verify_published_numbers.py` against the wave-8, wave-9, wave-15/17
> and d32/L4 headline result documents, of which **13** are prose checks on
> this draft. The
> matched-architecture numbers in this abstract are **not** among them: they are
> attested by the on-disk hashed run records cited row-by-row in
> [`PAPER_RESULTS_TABLE.md`](PAPER_RESULTS_TABLE.md), not by that script.
> Full record:
> [`SUMMARY_2026-08-22_CAMPAIGN_AND_RECORD_REPAIR.md`](SUMMARY_2026-08-22_CAMPAIGN_AND_RECORD_REPAIR.md).

*Camera-ready draft (prose). Numbers: [`PAPER_RESULTS_TABLE.md`](PAPER_RESULTS_TABLE.md). Claims: [`PUBLISHABLE_CLAIMS.md`](PUBLISHABLE_CLAIMS.md). Figures: [`PAPER_FIGURE_SPEC.md`](PAPER_FIGURE_SPEC.md).*

---

## Abstract

Adding a time-axis attention read-out to a spiking network raises accuracy on
the Spiking Heidelberg Digits (SHD) from 0.7057 to **0.8332** (gain
**+0.1275**, positive in 32/32 seeds). That SHD depends on temporal information
is established; what has not been measured is which *component's* contribution
is the order-dependent one. We measure it as a seed-paired
**difference-in-differences on the gain** (the read-out × destruction
interaction) rather than on accuracy: when temporal order is destroyed by
permuting time bins independently per sample, in **both the training and test
splits**, the attention arm loses **+0.1347** while a rate read-out, which is
invariant to the permutation by construction, loses **+0.0142**, so the
read-out's advantage collapses by **+0.1205** (95% CI [+0.114, +0.127]), which
is **94.5%** of the gain at this point (95% CI 90.9–97.9%). Permuting only the
time axis the read-out sees, with the input and the rate read-out untouched,
gives nearly the same **+0.1159**. Across
**twenty-one operating points at two training budgets** the read-out's gain
depends on order **wherever it has a gain to lose**: at five points after long
training it has no gain above the +0.03 bar, and there the positive difference-in-differences reflects
attention falling below the rate read-out once order is gone, which we report
as a limitation rather than as support. Time reversal, which displaces spikes
*further* than shuffling but destroys no information, leaves the contrast flat,
so the cost is not brittleness to displacement; destroying cross-channel
synchrony as well costs up to **+0.1308** more at coarse bins, an upper bound
because that operator also raises the substrate's activity. Our registered
prediction that the effect's size tracks the gain across widths failed
(Spearman **ρ = −0.1429**), so the anchor's share does not transfer: we claim
presence everywhere and a proportion at one point. Every contrast was
preregistered with a numeric bar on a frozen configuration; the intervals, the
share and the gain-to-lose reading were added afterwards and are marked as such.
All numbers are recomputed from archived runs by scripts released with the
paper.

## Abstract — matched-architecture kill gate (secondary program)

Broadcast ±1 three-factor plasticity (surrogate eligibility multiplied by a single ±1 reward) fails a preregistered accuracy and gap bar when the dense leaky-integrate-and-fire forward pass is held identical to a SuperSpike backpropagation-through-time reference. The arm remains at chance on **both** matched forward graphs (feed-forward 0.5000, gap lower confidence bound 0.0000; recurrent 0.5100, LCB −0.0192), against a gradient reference at 1.0000, n = 20 seeds. Every other rule tested on that forward now clears the gate: graded direct feedback alignment (0.9925 / 0.9875), directional REINFORCE with frozen per-neuron feedback (0.9950 / 0.9812), broadcast graded error (0.9975), and a discrete EventProp-style spike-adjoint (0.9450 / 0.8900). **The task therefore separates one rule from a field that otherwise saturates, and it no longer ranks the field**: with every reference at exactly 1.0000, each of those passes reduces to "the arm scored above 0.75". Live k-WTA transfer of the matched REINFORCE and DFA families remains a scoped **negative** across twelve gap-close variants (v13–v24), best gap LCB 0.3127 against a 0.5 threshold.

> **Provenance of these numbers, and it is load-bearing.** The figures above are from a 2026-08-25 re-run under `MATCHED_INPUT_SCALE = 2.0`. Every previously published matched-architecture number was produced on a forward pass that emitted **zero spikes at any seed**, and the arms that most depended on spikes were the ones it most misrepresented: the discrete spike-adjoint read 0.5000 there and reads 0.9450 here, because a method that differentiates through spike times had none. That claim is **withdrawn**, along with the two RL broadcast contrasts (0.5250 → 0.9100 and 0.5113 → 0.7962). See [`RESULT_2026-08-25_MATCHED_ARCH_RERUN.md`](RESULT_2026-08-25_MATCHED_ARCH_RERUN.md).

**Two scope statements are load-bearing and are stated here rather than in a footnote.** First, the 80-epoch schedule undertrains *every* rule on it: raising the reference's budget alone lifts it from 0.9013 to 1.0000 by e640, so `gap_closed` at the canonical budget divides by a reference that is still climbing, and values above 1 are an artefact of the denominator rather than a result. The defensible reading of the ordering between local rules and BPTT at that budget is a statement about **learning speed**, not about a ceiling. Second, the matched task saturates: with every arm reaching 1.0000 at high budget it can no longer separate them, so no ceiling comparison on this task survives convergence.

**Honesty notes.** On the DFA schedule a broadcast-*graded* contrast reaches 0.9975, so the lead negative is ±1 three-factor specifically, not "any broadcast". Since every other rule tested is now at or near ceiling, the negative is specific in a stronger sense than that phrasing suggests. Locality as a necessary ingredient is evidenced on one-layer XOR, not by the coincidence task alone. We do not claim biological realism, Assembly Calculus success, or impossibility of local learning in principle.

## 0. What is new here, and what is not

> **Provenance of the citations in this section.** The literature positioning
> below was assembled by a search pass on 2026-08-27 and **was read against its
> primary sources on 2026-09-03**. Each source was retrieved as a PDF, the cited
> number located in the source's own words, and the bibliographic fields taken
> from the arXiv API and Crossref. **Three claims were wrong**: the frontier
> band's upper bound, the attribution of that band, and the description of Yu et
> al. (2025). All three are corrected below, and every check, including the
> failures, is recorded with the source hashes in
> [`CITATIONS_2026-09-03_SECTION_0_AGAINST_PRIMARIES.md`](CITATIONS_2026-09-03_SECTION_0_AGAINST_PRIMARIES.md)
> and gated by `scripts/check_section0_citations.py`. Four numbers seen during
> the original search (Pfa-SNN 96.26, Event-SSMA 95.90, SpikeSCR 95.60,
> d-cAdLIF 94.85) came only from a secondary comparison table; this pass did not
> reach their primaries either, and they remain **excluded** from every claim
> here. Percentages in this section are quoted from published papers and are
> **not** machine-checked against runs on disk, which is what every SHD number
> elsewhere in this paper is.


This paper's SHD result sits in a populated field and the boundary is stated
here rather than left to a reader.

**Not new: a time-axis attention mechanism in a spiking network on SHD.**
TA-SNN [@yao2021tasnn] applies squeeze-and-excitation attention over the
time axis and reports 91.08%; STSC-SNN [@yu2022stsc] places temporal
attention inside the synaptic connection and reports 92.36%. Attention as a
*temporal read-out* is older still outside spiking networks: attentive
statistics pooling [@okabe2018] is the same idea on speaker
embeddings. Re-weighting the read-out's sum over time is not new either:
loss shaping [@nowotny2025lossshaping] replaces the unweighted sum with a fixed
exponentially time-weighted one on SHD. The specific placement used here, a
learned attention block **only** at the read-out, added alongside a mean
spike-rate read-out, appears unoccupied, but a configuration gap is not a
mechanism. Note what the comparator is: the rate read-out is exactly invariant
to permuting time, by construction, and it is **not** the field's default
unweighted Σₜ softmax(u[t]), which reads the membrane and is order-sensitive.
This paper does not run that default; comparing against it is the experiment
§4.8 names first.

**Not new: that SHD depends on temporal information.** The dataset's own authors
constructed spike-count-only variants, patterns "without temporal information",
and report that no linear or nonlinear classifier on them could "surpass the 60 %
accuracy mark for the SHD" [@cramer2022]. The
Neuromorphic Sequential Arena [@chen2025nsa] removes temporal propagation
**model-side** rather than data-side and reports SHD falling 86.48 → 68.51 (its
Table S4). Two destruction operators on two different sides of the model, one
conclusion, both prior to this work. Our shuffle is not their counts-only
condition: it keeps each utterance's length and the synchrony within a bin, and a
rate read-out on bin-shuffled input still reaches 0.6934 at the anchor, above
their 60% mark. Count-preserving randomisation of spike
order has been proposed as a robustness benchmark in its own right
[@hindman2026tempo], and for N-MNIST it has been shown that timing is not needed
at all [@iyer2021nmnist]. None of these separates *order* from the other things
a spike train carries, which is what §3.5 does and what makes synchrony visible
there as a second term.

**Not new, and worth conceding plainly: the accuracy.** The SHD frontier runs
from **95.07 ± 0.24%** with learned delays [@hammouamri2024dcls], the best published
result from a model of this general class, to **96.3%**, held by a state-space
model [@soydan2024s7] whose SHD entry uses data augmentation and
continuously-valued rather than spiking output. Adaptation reaches **95.81 ±
0.56%** in between [@baronig2025]. This instrument's
0.8332 is not in that band and is not offered as if it were. It is close to the
best-effort no-delay recurrent baseline the dataset's authors themselves report
(83.2 ± 1.3% at 1024 neurons with augmentation), which is where an architecture
carrying no temporal kernel should land, and is the external corroboration §3.8
argues for from ablation alone.

**Prior work closest to the reversal control, and how it differs.**
@yu2025timing reverse time on SHD with an operator that, in their words,
"perturb[s] spatio-temporal spike patterns but leave[s] interspike intervals and
coincidence information unchanged" (the information-preserving property §3.5's
`reversed` control was built to have), and find that "when axonal delays are not
used, networks perform well under time reversal, whereas networks trained with
delays perform poorly." Theirs is a test-time perturbation of a model trained on
intact data; §3.5 reverses the training split as well, which leaves a task
isomorphic to the original, so a flat contrast there is expected of any model
without a built-in direction and is a control on the shuffle result rather than a
replication of theirs. Yu et al. also compare networks with and without learned
delays under increasing perturbation, and per-spike against per-neuron jitter:
a component-level contrast and a timing-versus-coincidence split, both close in
spirit to the difference-in-differences and synchrony term of §3.5.

**New: which component's contribution is the order-dependent one.** Every result
above measures how much *accuracy* survives destroying temporal structure. None
measures how much of a *specific component's marginal contribution* survives it.
The contrast in §3.5 is a difference-in-differences (attention's shuffle cost
against the rate read-out's own, on the same seeds, same splits, same
destruction operator), and we find no published equivalent for a read-out on a
neuromorphic benchmark. It is the interaction term of a 2 × 2 factorial design
(read-out × destruction), and the logic is that of a control task
[@hewitt2019control]: what matters is not accuracy under destruction but the
difference a component makes with and without the structure it is supposed to
use. Destroying order in training as well as test data has precedent in
language modelling [@sinha2021order]. And the outcome is not guaranteed by the
architecture: attention with positional encodings has been found nearly
insensitive to shuffled input in time-series forecasting [@zeng2023ltsf].

**The claim is presence everywhere, and a proportion at one point.** Across
twenty-one operating points at two training budgets, the read-out's gain depends
on order wherever it has a gain to lose (at every point after 100 epochs and at
sixteen of twenty-one after 400), and the *size* of the dependence does not track
the gain across widths (§3.5). At the anchor, destroying order removes 94.5% of
the gain (95% CI 90.9–97.9%). That share is a measurement at one point, not a
decomposition of the gain that carries to the others.

**New, and unsupported in either direction: the width collapse.** No published
work reports an attention read-out degrading with hidden width, and none reports
gradient pathology in an attention read-out over spike trains. Width normally
*helps* on SHD (Cramer et al.: 1024 neurons → 76.5%; Bittar & Garner [@bittar2022]: RadLIF at
3×128 → 3×1024 improves 92.88 → 94.62). §3.5's inversion is therefore an anomaly
against the baseline expectation with no citation to lean on. The parsimonious
alternative, overfitting on a training set of **8,156** samples, is excluded by
the runs themselves: the collapsing arm reaches a training-loss minimum and ends
far above it, where an arm that overfits keeps its training loss low
(Appendix E). Cramer et al. report overfitting on SHD repeatedly, and that noise
injection reduced it; what the collapse is instead remains unexplained.

**A caveat on the benchmark itself.** SHD ships no validation set. Baronig et al.
(2025), the SE-adLIF paper cited above and not a separate source, report the same
model at 95.81 ± 0.56 validating on the test set "to ensure comparability" and
93.79 ± 0.76 with 20% of training held out instead: a two-point gap that is a
property of the protocol, not of the model.
Differences below ~1.5 points between published SHD numbers are not reliably
meaningful, and that applies to this paper's comparisons as much as anyone's.
It applies to this paper's headline directly: every number here is test-set
accuracy, and the anchor configuration was carried forward because it cleared
0.80 on the test set, so **0.8332 is test-selected**. The paired contrasts are
not, because both arms of every pair are selected together.

---

## 1. Introduction

A read-out that weights a spiking network's hidden state over time buys accuracy
on SHD. Everyone who has added one reports that, and this paper reproduces it:
0.7057 to **0.8332** at the headline configuration, positive in 32 of 32 seeds
and at or above 0.80 in 32 of 32. The interesting question is not whether it
helps but **what it is consuming**, and an accuracy number cannot answer that,
because the mechanism and the improvement are confounded in it.

**The design.** Pair the attention arm against a rate read-out that is identical
in every other respect, on the same seeds and splits, and measure the gain.
Then destroy the temporal order of the input by permuting time bins
independently per sample, in the training split as well as the test split, so
that the task itself must be solved without temporal order rather than merely
being made harder, and
measure the gain again. The difference of those two differences is the
quantity this paper reports. It is a statement about a component's marginal
contribution, not about how much accuracy survives a corrupted input, and it is
what distinguishes this from prior work: that SHD depends on temporal
information has been shown from the data side (Cramer et al., 2022) and from the
model side (Chen et al., 2025), but neither localises the dependence to a part
of the network.

**Three controls decide whether "order" is a label or a claim.** Bin-shuffling
destroys order *and* displaces almost every spike, so on that operator alone the
result is not distinguishable from "the attention arm is the more brittle of the
two". **Time reversal** displaces further (0.9986 of entries relocated at a
mean 145.2 bins, against bin-shuffling's 0.9972 and 109.2) while preserving
counts, cross-channel synchrony and every interval; applied to the training
split too, it leaves a task isomorphic to the intact one. Its
difference-in-differences is negative at all three points measured, so the cost
is in what the displacement destroys and not in the displacement.
**Channel-shuffling** destroys order and synchrony together, and the gap between
the two operators is what synchrony is worth. **A second training budget**
separates the mechanism from the schedule it was measured on.

**Coverage, because a mechanism claim at one configuration is a configuration
result.** The contrast is measured at every one of the twenty-one operating
points this instrument defines (widths 128 to 1024, both binning schemes,
both channel geometries, five read-out configurations) and at two budgets at
each. Coverage also exposes the contrast's limit: where the read-out has no gain
to lose, a positive difference-in-differences means only that attention falls
below the rate read-out once order is gone, and we report those five points as
such rather than as support.

**What failed is reported at the same weight as what held.** The registered
prediction that the shuffle cost would *track* the gain across width is refuted
(Spearman ρ = −0.1429 against a one-tailed n = 6 bar of +0.829); the width with
the smallest positive gain carries the largest difference-in-differences. So the
claim this paper defends is that the read-out's contribution **is**
order-dependent wherever it has a gain to lose. At the anchor we also measure how
much of it is (94.5%), and explicitly do not carry that share to other points. Section 4.6 states the scope limits
that travel with it.

**What this paper does not claim.** Nothing here is a claim about biology,
about neuromorphic hardware, or about accuracy against the SHD frontier, which
this model does not approach (§0).

---

## 2. Methods

### 2.1 Matched dense-LIF control

Matched-arch protocols fix the SurrogateLifReference / dense-LIF forward and vary only the learner. Protocol v4 compares production-style **broadcast ±1 three-factor** updates to SuperSpike BPTT [@superspike2018]. Protocol v5 evaluates graded error with fixed-random DFA feedback [@dfa2016]. Protocol v12 evaluates directional REINFORCE [@reinforce1992] × frozen per-neuron `B_i` as the primary arm. Protocol v130 (`track-b-rescue`) evaluated continuous RPE critic scalar broadcast vs online learned feedback alignment ($B_i \leftarrow B_i + \eta_B r (a_i - p_i) x_i$), but was withdrawn under v131 due to ceiling-inversion defects. Protocol v28 (`c1-eventprop-5bb083d5e88d0ad2`) is a discrete EventProp-style spike-adjoint H2H vs SuperSpike on the same matched forward. Gates reuse Gate G2 numeric thresholds (accuracy floor 0.65; gap LCB > 0.5) under fresh hash families that do not reopen `c1-118207fbc3eaba53`.

### 2.2 Engine C1 / Gate G2

The C1 harness encodes coincidence sequences with a latency encoder, integrates with muted hidden thresholds on the canonical path, selects winners by membrane-score k-WTA, force-fires winners and readouts, and applies three-factor plasticity. Gradient and eligibility references train on the same frozen splits. Live transfer of matched credit families onto this substrate is the v13–v24 package. **Protocol v131 (`live-transfer-rescue`)** is a misnamed **matched-only** online-FB schedule contrast (no Engine / no muted-θ / no live k-WTA) and is **not** a live-transfer result.

### 2.3 SHD attention readout

**Task and substrate.** On the Spiking Heidelberg Digits [@cramer2022] (20
classes; 8,156 training and 2,264 test utterances; 700 input channels), a single
hidden layer of leaky integrate-and-fire (LIF) neurons is trained with surrogate
gradients [@zenke2021robustness] through full backpropagation through time. The
**rate arm** reads the mean spike count of each hidden unit over the utterance
through a linear layer. The **attention arm** adds a time-axis self-attention
block over the hidden spike train, whose mean-pooled output passes through a
second linear layer and is **added** to the rate arm's logits. Both arms of a
pair share every base weight at the same seed, so any difference between them is
attributable to the read-out, not to a perturbed spiking forward. Each attention
block's output projection starts at zero, so every block begins as an identity
residual; the final linear layer is initialised like every other weight, so the
two arms' logits already differ at the first step.

**The attention block** embeds the hidden spike vector at each step to width $d$
through a learned $[h, d]$ matrix and adds a fixed positional code: $d/2$
sine–cosine pairs over normalised position, with frequencies spaced
geometrically from 1 to 64 cycles per utterance. Without it, mean-pooled
attention would be permutation-invariant. It then applies $L$ single-head
blocks at width $d$, each a $q, k, v$ triple of shape $[d, d]$, scaled dot-product
attention ($1/\sqrt{d}$) and an output projection added back residually, with no
normalisation or MLP. It is **not causal**: a full $[T, T]$ row-softmax with no
mask, so the arm sees the whole utterance at inference. The time-mean of the
last block's output is the read-out's input. Every arm is trained by the same
gradient instrument; no locality claim is made for the read-out.

**Operating points.** An operating point is one configuration of hidden width
$h \in \{128, 256, 384, 512, 768, 1024\}$, binning, channel geometry and read-out
depth ($L \in \{1, 2, 3, 4\}$ at $d \in \{32, 64\}$). *Binning* is either
`published-Nms` (bins of N ms from the first event, so the number of steps $T$
varies by utterance; 358 steps on average at 2 ms) or `fixed-tN` (a fixed 1400 ms
window from the recording's start cut into N bins of 14.0, 5.6 or 2.8 ms, with
later events dropped). *Channels* are either all 700
(`channels-700`) or adjacent groups of five summed to 140 (`adjacent-sum-5`).
The **anchor** is $h = 128$, `published-2ms`, `adjacent-sum-5`, $d = 32$,
$L = 4$, 400 epochs. The substrate question of §3.7 also varies the substrate:
feed-forward or recurrent, fixed or adaptive threshold (`ff+fixed`, `ff+alif`,
`rec+fixed`, `rec+alif`).

**Destruction operators** are applied to the training and test splits alike,
each sample's permutation drawn once before training (so it is the same in every
epoch) and the two splits' permutations drawn from disjoint seed streams:
`bin-shuffled` permutes the time bins (one permutation for all channels);
`channel-shuffled` permutes each channel's bins independently; `reversed`
reverses time; `hidden-shuffled` permutes only the time axis the attention block
sees, so the positional code stays with the presented slot while the hidden
state moves. The input is never touched by `hidden-shuffled` and the rate arm
reads the unpermuted spike train, but the attention arm's credit flows back to
the substrate through the permuted view, so its substrate trains differently
from its intact twin's. Every manipulated run uses the same manipulation seed,
so all model seeds see the same permutations. Minibatch order is reshuffled
every epoch from the model seed.

| component | setting |
|---|---|
| neuron | LIF, $u_t = \alpha\, u_{t-1}(1 - s_{t-1}) + W_{\text{in}} x_t$, threshold 1, hard reset (not differentiated); $\alpha = e^{-\Delta t/\tau}$, $\tau = 10.05$ ms (retention 0.82 per 2 ms step) |
| surrogate | arctan derivative, $\sigma'(x) = (\beta/2)/(1 + (\pi\beta x/2)^2)$, $\beta = 5$ |
| input | integer event counts per bin and channel, unscaled |
| loss | softmax cross-entropy, batch mean, full BPTT |
| optimiser | Adam ($\beta_1 = 0.9$, $\beta_2 = 0.999$, $\epsilon = 10^{-8}$), L2 penalty $10^{-5}$ added to the gradient (not decoupled) on every weight including the attention block's, none on the output bias |
| learning rate | one-cycle: linear from $10^{-3}$ to $5 \times 10^{-3}$ over the first 30% of steps, then linear to $10^{-5}$ |
| batch, epochs | 256 (32 steps per epoch); 400 or 100 epochs |
| initialisation | Glorot-uniform for every weight matrix, each from its own seeded stream; output biases and attention output weights zero |
| regularisation | none: no dropout, no data augmentation; no gradient clipping at the anchor |
| seeds | 12 per arm per point (32 at the anchor), paired across arms |
| precision | 32-bit floating point throughout |

Table: Training and model settings, shared by both arms at every operating point.

**Evaluation.** SHD has no validation split, and no model selection was done on
one: every accuracy reported is **test-set accuracy after the final epoch**.
The consequences for the headline are stated in §0.

**Statistics.** Every interval is a paired percentile bootstrap over seeds
(10,000 resamples), which runs somewhat narrow at twelve seeds. The intervals
are conditional on one draw of the destruction permutations and on the one SHD
test set. No correction is made across the 42 point-and-budget contrasts: the
anchor's contrast is positive in 32 of 32 seeds and survives any correction,
while the smallest lower bounds elsewhere (+0.033 and +0.037) sit close enough
to the +0.03 bar that a corrected interval would likely cross it.

**Compute.** The instrument is written in Rust and runs on CPU only. The
paper's evidence comprises 3,934 runs totalling 12,916 hours of per-run wall
time on aarch64 and x86-64 cloud instances and one Apple-silicon machine; an
anchor attention run takes 3–11 hours depending on host and load, a rate run a
few minutes. Runs of the same configuration on the two cloud architectures are
bit-identical (129,253 recorded values across 83 run pairs), and the
Apple-silicon rate arm at the anchor lands 0.0006 from the cloud fleet's (§5).

### 2.4 Preregistration, frozen hashes, and kill-gates

Each experiment in this paper was run as a preregistered **wave**: a batch of
runs whose hypotheses, numeric bars, stopping rules and analysis script were
committed before its first run existed, a constraint the version history
attests rather than the prose [@bertinetto2021prereg]. Four quantities were
added afterwards and are not preregistered: the paired bootstrap intervals, the
anchor's share of the gain, the reading of each point by whether it has a gain
to lose (§3.5), and the headroom normalisation of §3.7. Three rules make that
record citable.

**Frozen configurations.** Every run is named by a hash over its configuration.
A result may not be reinterpreted under its old hash; a new hypothesis needs a
new hash. A hash found to name two different experiments is retired rather than
reinterpreted, and a test fails if a retired hash is cited as a live result.

**Failed predictions stay failed.** Where a registered prediction failed we
report it as failed; the size prediction of §3.5 is the worked case. A bar that
turns out to be wrong is reported together with the result it produced, and a
replacement bar governs only the next wave, on new runs. Re-reading the same
runs against a bar chosen afterwards is what this rule exists to prevent.

**A check that could not run must not look like one that passed.** A suite of
record checks recomputes every published number from the archived runs; one of
them reports every assertion that would still pass if the thing under test did
nothing, because silent success was the most common class in this project's own
defect register.

---

## 3. Results

### 3.1 Matched-architecture primary results

All figures below are the 2026-08-25 re-run at `MATCHED_INPUT_SCALE = 2.0`, n = 20, reported on both forward graphs. The forward is held fixed and only the update rule changes; the arm means and both halves of the gate are drawn by verdict rather than ranked (Figure 6) because the arms were historically split across two and the difference exceeds the registered 0.02 bar on two of them ([`RESULT_2026-08-25_MATCHED_ARCH_RERUN.md`](RESULT_2026-08-25_MATCHED_ARCH_RERUN.md)).

| arm | feed-forward | recurrent | verdict |
|---|---:|---:|---|
| **broadcast ±1 three-factor** | **0.5000** | **0.5100** | **FAIL**, both |
| graded DFA | 0.9925 | 0.9875 | PASS, both |
| broadcast graded error | 0.9975 | 0.9975 | — (contrast) |
| REINFORCE × frozen `B_i` | 0.9950 | 0.9812 | PASS, both |
| RL graded-reward broadcast | 0.8787 | 0.9100 | — (contrast) |
| RL ±1 broadcast | 0.7775 | 0.7962 | — (contrast) |
| discrete EventProp spike-adjoint | 0.9450 | 0.8900 | PASS, both |
| SuperSpike BPTT ceiling | 1.0000 | 1.0000 | reference |

Broadcast ±1 three-factor fails the matched gate at chance on both graphs, against a reference that reaches 1.0000 at the canonical budget. **It is the only rule tested that does.** That is a sharper negative than the previous record described and a much weaker instrument: with the ceiling pinned at 1.0000, `gap_closed` reduces every PASS above to "the arm scored above 0.75", and five of the seven arms sit between 0.88 and 1.00. The task separates one rule from the field; it does not rank the field, and no ordering among the passing arms is claimed.

**Three claims are withdrawn from this section, all by measurement.** The discrete EventProp-style spike-adjoint FAIL (0.5000 against SuperSpike 0.9150) is withdrawn: it reads 0.9450 / 0.8900 and PASSes on a forward that can spike, and the archived 0.5000 was a spike-adjoint method with no spikes to differentiate through. The two RL broadcast contrasts, cited as evidence that continuous magnitude without spatial directionality is insufficient, are withdrawn: they read 0.8787–0.9100 and 0.7775–0.7962. **Online learned feedback alignment (v130 schedule)** remains withdrawn as a credit-assignment result, though under both repairs it now PASSes by the registered rule at 1.0000 against a ceiling of 1.0000 with zero variance across 20 seeds, which `RESULT_2026-08-23_TRACK_B_REREAD.md` registers as a saturation result and explicitly not a credit-assignment one. The `live-transfer-rescue` binary is matched-only and **does not** constitute live k-WTA transfer.

Honesty note required in the main text: on the DFA matched schedule, a broadcast-graded contrast reaches 0.9975. The lead negative therefore concerns **broadcast ±1 three-factor** credit, not the claim that every broadcast scalar fails coincidence. Locality as a necessary ingredient is evidenced on XOR (Section 3.4), not by coincidence DFA alone. **Figure M is redrawn** ([`PAPER_FIGURE_SPEC.md`](PAPER_FIGURE_SPEC.md)): it plotted richness × addressability as a graded surface, and on the re-run it is a cliff with one cell below it. Two constraints are now on the spec: encode pass/fail/at-chance rather than a ramp, since with the reference at 1.0000 a ramp manufactures an ordering the task cannot support; and draw the two low-richness broadcast rules separately, because ±1 × surrogate eligibility is at chance while ±1 broadcast REINFORCE reaches 0.78, and collapsing them would be a stronger version of the error the 0.9863 disclosure was added to prevent.

### 3.2 Engine C1 secondary results

Canonical C1 fails Gate G2 (local 0.4912, gap LCB −0.0048); local-assembly and dense-local are **at chance on a two-class task**, which the figure marks with an explicit 0.50 line rather than letting bars from a zero baseline read as half the reference. Trial isolation and temporal positive-control sensitivities fail without clearing the accuracy floor. Capacity sensitivity clears the accuracy floor (0.6775) but leaves gap LCB at 0.0000 versus dense/gradient references; descriptive only, not a G2 PASS.

### 3.3 Live transfer, gap-close, and break-it

The ladder from the matched PASS down through every live arm is read on **two** axes, the 0.65 accuracy floor and the 0.5 gap-LCB gate, because clearing the floor is not clearing the gate, and with a substrate break between rung 1 and the rest, because rung 1 is the matched dense-LIF forward and nothing below it is. Live opt-in REINFORCE feedback fails G2 (local 0.4900, gap LCB 0.0737). Epoch matching alone does not rescue random-B live RFB (local 0.4838). Structured frozen B clears the accuracy floor (0.7262) with gap LCB 0.2567 but still fails the gap bar. Stacking epochs under structured B regresses (0.5200). Structured B on a capacity substrate yields the best gap LCB in the prior suite (0.3127) while clearing the floor (0.6825), remaining short of 0.5. Eligibility×REINFORCE and restored target teach clear the floor but do not beat structured B alone. Break-it protocols close remaining differentials without remassage: live graded DFA (v20) reaches local 0.7325 with gate LCB 0.2601 and chance LCB 0.3321: floor yes, gate no. Soft-WTA × structured B (v21) regresses to chance (0.5025). Matched three-factor under 4× epochs (v22) stays at 0.5000. Finite-θ under SFB (v23) clears the floor (0.6638) with LCB 0.2370. Continuous structured B (v24) does not beat sign-truncated v15 (0.6437 / 0.1380). Spiking-path true DFA rescue fails in one honest attempt (0.6513, gap LCB 0.0733). Differential closure: [`DIFF_CLOSURE.md`](DIFF_CLOSURE.md).

### 3.4 Task evidence

On one-layer `xor_thresh`, broadcast error stays at chance (0.5008) while DFA reaches 0.8267 against gradient 0.7733, a locality flip. On mid-init two-layer depth locality, broadcast error also succeeds (0.8158) alongside DFA (0.8250) and REINFORCE×B (0.8033); depth help is not treated as a locality-flip claim.

### 3.5 SHD attention read-out and mechanism

Every number in this section is seed-paired test-set accuracy after the final
epoch (§2.3), and every contrast was preregistered with a numeric bar before its
first run existed (§2.4). Appendix K lists every operating point with paired
bootstrap 95% intervals.

**The headline** (Figure 2). At the anchor (128 hidden units, `published-2ms`
binning, `adjacent-sum-5` channels, attention width 32 with four blocks, 400
epochs), the attention arm reaches **0.8332** against the rate arm's 0.7057 over
32 seeds, a gain of **+0.1275** (95% CI [+0.123, +0.132]), positive in 32 of 32
seeds and at or above 0.80 in 32 of 32. The twelve registered seeds gave 0.8320
against 0.7062 (+0.1258); twenty further seeds moved the gain by +0.0017.
([`RESULT_2026-08-20_D32L4_CLEARS_THE_080_GATE.md`](RESULT_2026-08-20_D32L4_CLEARS_THE_080_GATE.md);
[`RESULT_2026-08-27_W15_17_THE_COLLAPSE_IS_A_THRESHOLD.md`](RESULT_2026-08-27_W15_17_THE_COLLAPSE_IS_A_THRESHOLD.md))

**The conditional** (Figure 1). Under bin-shuffling, on the same 32 seeds, the
attention arm loses **+0.1347** and the rate arm **+0.0142**. The gain falls from
+0.1275 to **+0.0070**, so the read-out × shuffle interaction, the
difference-in-differences (DiD), is **+0.1205** (95% CI [+0.114, +0.127]),
positive in 32 of 32. The rate arm cannot pay for a permutation of time except
through the substrate, whose membrane retains 0.82 of its state per 2 ms step;
its +0.0142 is that substrate's own dependence on order.
([`RESULT_2026-08-21_W9_THE_MECHANISM_HOLDS_AT_THE_HEADLINE.md`](RESULT_2026-08-21_W9_THE_MECHANISM_HOLDS_AT_THE_HEADLINE.md))

**The same contrast with the substrate untouched.** `hidden-shuffled` permutes
only the time axis the read-out sees. The input is intact and the rate arm reads
the unpermuted spike train, so the rate arm pays exactly nothing: in 12 of 12
seed pairs, every recorded field is identical. (The attention arm's substrate is
not frozen; its credit returns through the permuted view.) The attention arm's gain falls from +0.1258 to +0.0099: DiD
**+0.1159**, positive in 12 of 12, close to what destroying order in the data
costs on the same seeds (+0.1208). What the read-out consumes, it reads off the
hidden layer's time axis.
([`RESULT_2026-09-05_W27_THE_TIMESCALE_IS_261_MS.md`](RESULT_2026-09-05_W27_THE_TIMESCALE_IS_261_MS.md))

**Twenty-one operating points, and where the read-out has no gain to lose.**
Preregistered waves measured the same DiD at every operating point the
instrument defines (widths 128 to 1024, both binning schemes, both channel
geometries, and read-out depths of one to four blocks at attention width 32 or
64) at 400 epochs and again at 100. The DiD clears its registered +0.03 bar at
every point and both budgets.
([`RESULT_2026-08-29_W21_THE_MECHANISM_TRAVELS_BUT_ITS_SIZE_DOES_NOT.md`](RESULT_2026-08-29_W21_THE_MECHANISM_TRAVELS_BUT_ITS_SIZE_DOES_NOT.md);
[`RESULT_2026-09-01_W22_THE_MECHANISM_AT_EVERY_OPERATING_POINT.md`](RESULT_2026-09-01_W22_THE_MECHANISM_AT_EVERY_OPERATING_POINT.md);
[`RESULT_2026-09-03_W25_THE_MECHANISM_WHERE_IT_WAS_UNMEASURED.md`](RESULT_2026-09-03_W25_THE_MECHANISM_WHERE_IT_WAS_UNMEASURED.md))

That statistic alone overstates the claim. The DiD is gain(intact) −
gain(shuffled), and where the intact gain is near zero a positive DiD records
only that the attention arm trains *below* the rate arm once order is gone, not
that the read-out bought anything order-dependent. At 100 epochs the read-out has
a gain at every point (the smallest lower bound of any intact-gain interval is
+0.034), so there the DiD measures an order-dependent benefit everywhere. At 400
epochs it has a gain above +0.03 at sixteen of the twenty-one points (fourteen
with the whole interval above it). At the other five (one block at widths 256,
512 and 1024, one block on `channels-700`, and four blocks at width 1024), the
intact gain lies between −0.1618 and +0.0243 and the shuffled gain between
−0.2740 and −0.0608. **The claim this paper defends is therefore that wherever
the read-out has a gain to lose, that gain depends on temporal order.** The
negative shuffled gains are a finding in their own right, and not confined to
those five: the shuffled gain's interval lies wholly below zero at twelve of the
twenty-one points after 400 epochs and at five after 100, and attention on a task
that has lost its order falls up to 0.27 below a read-out that ignores time
(Appendix K). Each training sample's permutation is drawn
once and held fixed (§2.3), so a positional code can fit it; this paper does not
test that account.

**What "order" means.** Bin-shuffling destroys order and also displaces almost
every spike, so on that operator alone the result could be brittleness to
displacement. **Time reversal** displaces spikes further: it relocates
**0.9986** of entries at a mean of **145.2 bins**, against bin-shuffling's 0.9972
and 109.2, while preserving per-channel counts, cross-channel synchrony and every
inter-spike interval, and because it is applied to the training split as well, it
leaves a task isomorphic to the intact one. At all three points measured its DiD
is negative (−0.0222, −0.0040, −0.0099; positive in 2, 4 and 3 of twelve), against
the same +0.03 bar. The second of those points, width 1024 with one block, is
one with no gain to lose at 400 epochs, so the control rests on the other two. The cost is in what the displacement destroys, not in the
displacement.
([`RESULT_2026-09-02_W24_ORDER_SYNCHRONY_AND_BUDGET.md`](RESULT_2026-09-02_W24_ORDER_SYNCHRONY_AND_BUDGET.md))

**Synchrony, as an upper bound.** `bin-shuffled` applies one permutation to all
channels, so order dies and within-bin synchrony survives; `channel-shuffled`
permutes each channel independently and destroys both. At the three `fixed-tN`
windows, destroying synchrony as well costs **+0.1308**, **+0.1038** and
**+0.0575** more (DiDs +0.2631, +0.2157 and +0.1736 against +0.1323, +0.1119 and
+0.1161), falling as bins get finer (14.0, 5.6, 2.8 ms); at both `published-2ms`
points the increment is within ±0.03 (−0.0117, −0.0181), the second of them
again at width 1024 with one block. **These increments are
upper bounds.** `channel-shuffled` also changes the substrate's activity: at
the three `fixed-tN` windows it more than doubles the rate arm's mean firing
rate (from 0.18–0.20 to 0.38–0.45) and pins up to 3.5% of its hidden units at
maximum firing, where `bin-shuffled` pins none. Part of each increment may
therefore be an activity effect. A re-measurement at matched activity
has not been run. The read-out reads synchrony as well as order at coarse bins;
how much of the increment is synchrony is open.

**What in the read-out consumes it** (waves 26–29, one operating point).
Removing only the positional encoding cuts the gain from +0.1258 to **+0.0506**,
while destroying order in the data leaves +0.0050, so something besides the
positional code carries order, and this paper does not identify it. Permuting
bins inside a sliding window reaches half the full shuffle's effect at a window
of **130.33 bins (261 ms)**, interpolated linearly between the measured windows
of 128 and 192 bins; that is about 36% of the mean utterance. Spike counts are
not what it consumes: under random spike deletion the advantage grows, to
+0.1640 at 90% deletion, and a two-sided registration of that contrast was NOT
MET before a one-sided re-registration on fresh seeds was MET (DiD −0.0467, 11
of 12). Appendix F gives the three manipulations in full.
([`RESULT_2026-09-06_W28_THE_READ_OUT_SURVIVES_WHAT_THE_SUBSTRATE_CANNOT.md`](RESULT_2026-09-06_W28_THE_READ_OUT_SURVIVES_WHAT_THE_SUBSTRATE_CANNOT.md);
[`RESULT_2026-09-09_W29_THE_ASYMMETRY_HAS_ITS_OWN_BAR.md`](RESULT_2026-09-09_W29_THE_ASYMMETRY_HAS_ITS_OWN_BAR.md))

**Presence everywhere, and a proportion at one point.** At the anchor,
destroying order removes **94.5%** of the gain (95% CI 90.9–97.9%, n = 32). That
share does not transfer. The registered prediction that the DiD would track the
gain across the six widths failed: Spearman ρ = **−0.1429** against the
one-tailed n = 6 critical value of +0.829. With six points only a near-perfectly
monotone relation could have passed, so the failure is weak evidence, and it is
carried mostly by width 768, whose gain (+0.0560) is the smallest positive one on
the ladder and whose DiD (+0.1881) is the largest. We claim that the read-out's
gain is order-dependent wherever it has one, and a share only where we measured
it.

**Scope.** *Depth:* at 400 epochs every shallower or wider read-out puts the
DiD within 0.05 of its four-block, width-32 twin, except h768 with two blocks,
0.1271 away and outside the registered 0.10 bar (Appendix K). *Width:* the gain falls from +0.1258 at width 128 to +0.0560 at 768
and inverts to −0.1618 at 1024, a threshold between 768 and 1024 at 6.9× the
largest step below it, which two- and three-block read-outs at that width do not
show (Appendix E). *Resolution:* holding a 1400 ms window fixed, the gain is
+0.1927, +0.1751 and +0.1474 at 14.0, 5.6 and 2.8 ms bins, shrinking by −0.0453
as bins get finer but never reversing (Appendix I). *Geometry:* the gain is
positive on both channel geometries, but 0.80 is cleared only on
`adjacent-sum-5` (0.7864 on `channels-700`). *Substrate:* every manipulated run
is feed-forward; the registered attempt on a recurrent substrate could not be
evaluated (§3.7).

---

### 3.6 The reference is undertrained, and the task saturates

Both facts bear directly on every gap number above and neither is a footnote.

Holding the forward, the frozen splits, the seed lineage (n = 20) and every arm
fixed, and sweeping **only** the reference's training budget, the SuperSpike
reference rises from **0.9013** at the canonical e80/lr0.05 to **0.9700** by e320
and **1.0000** by e640. The arm-versus-reference ordering therefore **inverts**
purely as a function of the reference's budget.

Two consequences:

1. **`gap_closed` at the canonical budget is not a ceiling-normalised quantity.**
   It divides by a reference that is still climbing. With the reference at 1.0000
   the DFA arm's gap-closed would be `(0.9387 − 0.5)/(1.0 − 0.5) = 0.877`, not
   `(0.9387 − 0.5)/(0.8963 − 0.5) = 1.107`. (Both figures are from the archived
   run; on the repaired instrument the reference *is* 1.0000 at the canonical
   budget, so this arithmetic is no longer hypothetical: it is what the
   denominator now is, and the ordering it produces is in §3.1.) Values above 1
   are an artefact of the
   denominator; clamping them, as `runner.rs` does, hides the cause rather than
   fixing it.
2. **Raising *both* budgets does not restore the ordering**: the arm stays at or
   above the reference at every budget tested, and the whole schedule saturates at
   1.0000. So the "arm exceeds ceiling" anomaly is **not** explained by reference
   undertraining alone; it survives matched compute, and that question is open.

The honest statement of what the matched comparison measures at the canonical
budget is therefore **learning speed**, not the distance to a ceiling. Any future
matched-architecture claim needs a task with headroom at convergence rather than
one where every arm reaches 1.0000.

*(Source: `RESULT_2026-08-19_A6_CEILING_HEALTH.md`, n = 20, 24 budget points per
suite. Run on `aarch64-unknown-linux-gnu`; the absolute reference values are not
directly comparable to the macOS-recorded ones, and the 0.90 → 1.00 effect is an
order of magnitude larger than that drift.)*

---

### 3.7 The read-out does not substitute for temporal state in the substrate

The +0.1258 of §3.5 has two readings the campaign could not separate, because every one of its 720 cells sat on a single substrate, `ff+fixed`: the read-out **adds** temporal structure no substrate of this kind represents, or it **substitutes** for the threshold adaptation and recurrence that `ff+fixed` happens not to have. ETLP's conclusion [@quintana2024etlp], that adaptation and a recurrent topology are what a spiking network needs for rich temporal structure, makes the second reading the live one. Three waves settle it (Figure S), and the limits below travel on the figure rather than in its caption alone.

**Adaptation makes no difference to the gain, or to anything else.** At the anchor (h128, `published-2ms`, `adjacent-sum-5`, e400, d32/L4, n=12), attention's gain is **+0.1258** on `ff+fixed` and **+0.1285** on `ff+alif`. The difference is **+0.0027** against a two-sided bar of 0.03, and is positive in **6 of 12** seeds, a coin flip. Adaptation alone does not help either: `ff+alif` reaches **0.7018** against `ff+fixed`'s 0.7062, better in **3 of 12** seeds, with **0 of 12** over the 0.80 gate. At this operating point threshold adaptation is inert. ([`RESULT_2026-08-23_W12_ATTENTION_DOES_NOT_SUBSTITUTE_FOR_ADAPTATION.md`](RESULT_2026-08-23_W12_ATTENTION_DOES_NOT_SUBSTITUTE_FOR_ADAPTATION.md))

**The recurrent substrate is measurable only at one operating point, and finding it was its own wave.** `rec+alif` completes **11 of 12** at the anchor budget only at surrogate scale 0.4; at the registered default of 1.0 it completes 8 of 12. `rec+fixed` completes 12 of 24 across both scales and fails by a different mechanism: **saturation**, with ten cells voided with up to 52% of hidden units pinned at maximum firing and none by divergence at scale 0.4. Adaptation is what prevents that, so on the recurrent substrate adaptation is *stabilising*, which is the opposite of the hypothesis that wave's own name asserted. ([`RESULT_2026-08-23_W13_RECURRENT_STABILITY.md`](RESULT_2026-08-23_W13_RECURRENT_STABILITY.md))

**On the recurrent substrate the gain roughly doubles.** With every arm run at scale 0.4 so substrate and scale cannot be confounded, and paired over the seeds where both arms completed:

| substrate | pairs | rate read-out | + attention d32/L4 | gain |
|---|---:|---:|---:|---:|
| `rec+alif` | 10 | 0.5262 | 0.7874 | **+0.2612** |
| `ff+fixed` | 12 | 0.7088 | 0.8289 | **+0.1201** |

Table: The recurrent substrate against the feed-forward one, every arm at surrogate scale 0.4, paired over the seeds where both arms completed.

The difference is **+0.1411** against a bar of 0.03, positive in **10 of 10** recurrent pairs. The scale is not doing the work: `ff+fixed` at 0.4 scores **0.7088** against **0.7062** archived at 1.0, a difference of +0.0026. ([`RESULT_2026-08-23_W14_ATTENTION_AND_RECURRENCE_ARE_COMPLEMENTARY.md`](RESULT_2026-08-23_W14_ATTENTION_AND_RECURRENCE_ARE_COMPLEMENTARY.md))

**So substitution is refuted on both axes**, and the read-out's advantage is indifferent to adaptation and *larger* where the substrate is recurrent.

**Whether the recurrent advantage is order-dependent is not measured.** Every manipulated run in this paper is feed-forward, and the wave registered to change that (72 recurrent cells under `intact`, `bin-shuffled` and `reversed`) **could not be evaluated**: ten failed on the instrument's non-finite-training guard, five of them the rate arm under time reversal, leaving the reversal contrast at three seed-paired quadruples against a floor of nine. What the attempt did establish is that `rec+alif` at the registered surrogate scale is numerically unstable under time reversal, which is a fact about the substrate and not about the read-out ([`RESULT_2026-09-03_W25_THE_MECHANISM_WHERE_IT_WAS_UNMEASURED.md`](RESULT_2026-09-03_W25_THE_MECHANISM_WHERE_IT_WAS_UNMEASURED.md)).

Four limits are load-bearing and are stated here rather than in a footnote.

1. **The recurrent gain is measured from a lower base.** `rec+alif` starts 0.18 below `ff+fixed` and has 0.4738 of headroom against 0.2912. Normalising by headroom (post-hoc, not registered), the ratio falls from 2.2× to **1.34×**. The ordering survives; most of its apparent size does not.
2. **The recurrent substrate does not win.** `rec+alif+attn` reaches 0.7874 against `ff+fixed+attn`'s 0.8289 at the same scale. Attention closes most of the gap the substrate gives away, and not all of it. No verdict is issued on that ordering.
3. **The recurrent arms are numerically extreme**, with peak gradient norms to 4.9e32 against 1.13e8 for the largest cell anywhere else in the campaign, and the comparison rests on **ten pairs, the registered minimum**. The two arms lost different seeds; one further loss on either would have made the comparison unreportable.
4. **Survivorship is reduced, not removed.** Pairing on seed compares the same trajectories rather than two differently filtered subsets, but the surviving recurrent pairs are those that did not diverge, and divergence is not random. The feed-forward comparison carries no such exposure at 12/12.


### 3.8 Where 0.8320 sits against the literature, and why the gap is a class difference

A read-out result on a standard benchmark invites one question immediately, and the paper answers it here rather than leaving it to a reader: published SHD accuracies for surrogate-gradient spiking networks sit around 0.90–0.95, and the pinned third-party reference used for calibration in this work reaches **0.9390 / 0.9368 / 0.9371** on the clean protocol. The instrument's converged attention arm reaches **0.8320**. The difference is about **0.087** against the best delay-free variant of that reference, and it is not a tuning gap.

Four preregistered ablations turned every knob on the reference that could plausibly account for it: depth, dropout, read-out style, normalisation. **Together they do not explain it, and two of them subtract**: removing the reference's second hidden layer *gains* 0.0145 and removing its batchnorm appears to gain 0.0058. Of everything measured, only dropout is a positive contributor the instrument lacks, and it is worth 0.0128 of the 0.087.

Reading the two forward passes term by term found what the configuration surface could not. The reference carries a **`Dcls1d` temporal kernel of 25 taps per synapse on every layer**, spanning 250 ms, and its membrane decays to a third in a single timestep; its temporal integration lives in the convolution and its neuron is close to a pointwise nonlinearity. The instrument is the mirror image: **no temporal kernel in any form**, and a membrane retaining 0.82 per step to compensate. These are not one architecture at two operating points ([`FINDING_2026-08-24_THE_FORWARD_PASSES_DIFFER_IN_KIND.md`](FINDING_2026-08-24_THE_FORWARD_PASSES_DIFFER_IN_KIND.md)).

That reframes this paper's own result rather than excusing it. The attention read-out is a mechanism over the time axis added to a substrate that has none, and §3.5's shuffle control shows that destroying order removes 94.5% of its gain at the anchor (95% CI 90.9–97.9%, n = 32). **The most likely reading of the +0.1258 is that the read-out is recovering some of what a learned temporal kernel supplies**. That is a claim about kind and, given the ablation below, not a claim that the two quantities are commensurable. It is a claim about what the read-out consumes, and is consistent with §3.7's finding that its advantage is larger, not smaller, where the substrate is recurrent.

**The kernel has now been ablated, and the arithmetic that motivated this section does not survive it** (preregistered, n = 3 per arm). Setting `max_delay = 1` makes all three `Dcls1d` constructions pointwise (a one-tap dilated convolution) and lets the five values mechanically derived from it follow, changing nothing else. The reference falls from **0.9387** to **0.6276**: the kernel is worth **K = +0.3111**.

That is **3.6× the 0.087 it was invoked to explain**, and the direction is what matters. Removing the kernel does not bring the reference *down to* the instrument's 0.8320; it drops it **0.2044 below** it. So the two systems are not one architecture plus a temporal kernel. They differ by at least two large terms of opposite sign (a kernel worth +0.31 to the reference, and something else worth roughly −0.20 to it relative to the instrument), and **the observed 0.087 is the net of offsetting differences, not the size of any one of them.**

**The compensating term is the membrane, and it is 81.5% of it** (preregistered, n = 3 per arm). The reference's neuron retains **0.0050** of its membrane per step (measured by construction, not inferred), so with the kernel removed it has no temporal integration at all, while the instrument retains 0.82. Setting `init_tau` to 55.5556 ms, which reproduces 0.8200 exactly and is the only value the series introduces, lifts the kernel-free reference from 0.6276 to **0.7942**: **M = +0.1666** of the 0.2044 that separated it from the instrument. The decomposition now closes: the reference's lead of **0.1067** over the instrument is **+0.3111 − 0.1666 − 0.0378**, and what is unexplained has fallen from 0.2044 to **0.0378**, below the 0.05 this programme calls material. The gate for that reading is that Arm B′ reproduced the kernel ablation's Arm B **to every recorded digit on a different instance**, so the two terms were measured against each other and not across a platform. The remaining 0.0378 is not identified: the synaptic filter, the optimiser, the schedule and the read-out are all untouched. ([`RESULT_2026-09-02_THE_MEMBRANE_ABLATION.md`](RESULT_2026-09-02_THE_MEMBRANE_ABLATION.md))

**What that changes, and what it does not.** The *arithmetic* attribution is withdrawn: this section may no longer read the 0.087 as the kernel's value, and the sentence above about the read-out "recovering a fraction of what a learned temporal kernel supplies" is a claim about kind, not about quantity. What is strengthened is the kernel itself. It is no longer supported by elimination and code-reading; it is a measured 0.3111, larger than every other term in the ablation series by an order of magnitude, and the single biggest structural fact about the reference. Its wall time corroborates that the manipulation reached the mechanism rather than a config value nothing reads: 5.06 h against 1.57 h on identical hardware, a 3.2× speedup from removing 25 taps and their 24 + 12 padding. ([`RESULT_2026-09-02_THE_KERNEL_ABLATION.md`](RESULT_2026-09-02_THE_KERNEL_ABLATION.md))

**Three limits on this section.** The ablation ran on aarch64 Linux rather than the machine that produced the pinned reference, so its Arm A is a **re-measurement** rather than a continuation of that series; the registration made this the gate, and Arm A reproduced the pinned 0.9376 at **0.9387**, a drift of 0.0011 against a bar of 0.010, with `torch` and the reference checkout pinned to the versions the original series used. The four *configuration* ablations were run at **n = 1** with a three-seed spread of 0.0022, so the batchnorm effect sits in the suggestive band and is not quoted as resolved. And every ablation here varies the *reference*, so each statement about the instrument remains an inference from a difference of differences.

---

## 4. Discussion and limitations

### 4.1 Lead claim and mechanism

The cleanest publishable claim is rule-topological: under a fixed dense-LIF forward, **broadcast ±1 three-factor** credit does not close the preregistered gap to SuperSpike BPTT, while graded DFA and REINFORCE×frozen-`B` do. That contrast supports modulator richness and feedback addressability as material factors on this gate (Figure M), without equating matched success to live sparse/k-WTA success.

Richness alone is not “locality”: on the DFA matched schedule, broadcast-*graded* reaches 0.9975. The lead FAIL is therefore specifically ±1 × surrogate eligibility under broadcast, not an indiscriminate “broadcast credit topology” ban. Addressability / locality as a necessary ingredient is evidenced by the one-layer XOR locality flip (broadcast graded fails; DFA solves), not by coincidence DFA alone.

**A6 Ceiling Health Caveat, and it has become the main limitation.** On the archived instrument the 80-epoch schedule undertrained the gradient reference (0.8963 / 0.9013 at e80, climbing to 1.0000 by e640; `RESULT_2026-08-19_A6_CEILING_HEALTH.md`), so gap-closed at e80 reflected *learning speed* rather than asymptotic capacity. On the repaired instrument the reference reaches **1.0000 at the canonical budget itself**, so there is no budget at which this task separates the arms. Every matched comparison in this paper is therefore bounded by a saturated reference, and the only claim that survives it is the one negative.

**Falsifier.** A matched ±1 three-factor arm that clears the accuracy floor *and* gap LCB under the same dense-LIF forward, splits, and Gate G2 numeric thresholds would overturn the lead claim. Silent threshold changes, hash remassage, or live-path substitutions do not count.

### 4.2 Transfer barrier and live negatives

The live transfer package is a scoped negative. Structured feedback is the strongest accuracy lever among tested gap-close arms, and capacity×structured yields the best prior gap LCB, but none (including live DFA, soft-WTA, mute-off, and continuous-B probes) clears Gate G2 (v13–v24). Matched undertraining does not explain the broadcast ±1 three-factor FAIL (v22 remains at chance). Reporting floor clearance without gap clearance would overclaim. Engine C1 remains a valid operationalized pipeline negative only when integrity caveats are explicit (Appendix A). Do **not** cite v131 / `live-transfer-rescue` as live-engine rescue: that binary is matched-only.

Soft-WTA × structured B on live C1 uses disclosed temperature `T=1` (v21). That probe is motivated by a hybrid soft→hard collapse whose transfer-collapse temperature is **T=2.0** on a separate hybrid ladder (`binn-hybrid-winner-temp-v1-fa7710de68ad7bfe`). Hybrid T=2.0 is **not** live v21; do not equate the appendix hybrid mechanism note with the live soft-WTA protocol.

### 4.3 Baselines and EventProp H2H

The primary gradient ceiling on the matched gate is **SuperSpike BPTT** on the fixed dense-LIF forward. True σ′ e-prop [@eprop2020] (`c1x-eprop-true-*`, true-surrogate mean 0.7125) is a methods footnote only, not a H2H claim that e-prop rescues broadcast ±1 insufficiency. **The discrete EventProp-style [@eventprop2021] spike-adjoint FAIL is withdrawn.** It was reported as mean 0.5000 against SuperSpike 0.9150, gap LCB 0.0000; on the repaired forward it reaches **0.9450** (feed-forward) and **0.8900** (recurrent) and **PASSes** on both. The archived number was a spike-adjoint method on a forward that emitted no spikes, and "discrete ≠ continuous Wunderlich–Pehle" was an explanation offered for a result with a different cause. This remains a **discrete** hard spike-gate adjoint and still is not continuous Wunderlich–Pehle (2021) hybrid EventProp; no comparison to that method is claimed in either direction. Hybrid exact-forward arms labeled “e-prop/DFA” remain eligibility × transported modulators unless the true-σ′ family is cited explicitly.

### 4.4 Efficiency honesty (F1 / F2 / F5)

These limitations constrain how far efficiency rhetoric should travel with the negative result:

- **F1 (scan barrier).** Sub-threshold membrane dynamics admit chunked associative scans; the hard spike reset remains a sequential, data-dependent barrier. Training-time parallelism is therefore *partial*, not a clean escape from recurrent-time cost.
- **F2 (forward remains sequential).** Online local learning removes the backward-unroll / activation-storage half of BPTT cost; it does **not** remove sequential forward simulation within a stream. Parallelism is across neurons, areas, and independent streams, not across time within one stream.
- **F5 (activity ≠ compute).** Sparse activity ratios do not translate linearly into work. Honest software efficiency is work-per-accuracy including per-event queue and pointer-chase overhead; large multipliers are a neuromorphic-hardware claim, not a CPU/GPU default.

### 4.5 Appendix-only: G3 / G4 / H0 (do not reopen G2)

Continual forgetting (C2 / Gate G3 FAIL), multi-area scaling (R2 / Gate G4 **NO-GO** degrade curve), and hybrid H0 (**HYBRID_NO_GO**) live in [`APPENDIX_POST_G2.md`](APPENDIX_POST_G2.md). They are post-G2 / exploratory. The camera-ready banner does **not** reopen Gate G2 or remassage `c1-118207fbc3eaba53`. G4 NO-GO redirects away from scaling more areas under the same ±1 three-factor substrate; any future Micro / isolate capacity stress is engineering headroom after G2 FAIL, not a Foundation unlock and not part of this MUST package.

### 4.6 Neuromorphic benchmark scope and non-claims

**What is claimed.** On SHD, a time-axis attention read-out's gain over a rate
read-out depends on the temporal order of the input wherever the read-out has a
gain to lose: at all twenty-one operating points after 100 epochs, and at
sixteen of twenty-one after 400 (§3.5, Appendix K). Removing order in the data
and removing it only from the read-out's view give nearly the same contrast at
the anchor (+0.1208 and +0.1159), and time reversal, which destroys no
information, gives none. At coarse bins the read-out also reads cross-channel
synchrony, by an amount this paper can only bound from above.

**What is not claimed, and why.**

- *Proportion.* The size of the contrast does not track the size of the gain
  (§3.5), so we do not decompose the gain into an order-dependent share and a
  remainder.
- *A strong comparator.* The rate read-out cannot see order by construction.
  The result shows that the component able to read order is the one whose
  contribution depends on it, and how strongly; whether it survives against
  read-outs that also see order is the first open problem (§4.8).
- *Generality beyond this substrate and dataset.* Every manipulated run is a
  single feed-forward LIF layer without delays or learned temporal kernels, on
  one dataset. The registered attempt on a recurrent substrate could not be
  evaluated: ten of its runs failed the instrument's non-finite-training guard
  (§3.7).
- *Depth and width.* Read-out depth is a scope axis at one width of five (h768,
  outside the registered 0.10 bar), and the gain inverts between widths 768 and
  1024 for reasons this paper does not identify (Appendix E).
- *The frontier.* 0.8332 is test-selected and roughly 0.12 below the published
  state of the art; §0 and §3.8 place it.
- *Calibration.* No number here is comparable to an externally recorded one: the
  instrument has no Python mirror of its attention axis (Appendix G).

We make no claim about biology, cortical realism, neuromorphic hardware, or the
impossibility of local learning in principle.

### 4.7 The withdrawal ledger

**Seven results have been withdrawn from the wider project this paper comes
from, three of them passes.** None is a number from the SHD attention campaign
reported above; they belong to the secondary programme (§3.1). We report
the count because a record that shows only surviving results tells a reader
nothing about how hard it was tested, and because the withdrawals are what
license the survivors. Two episodes produced them: a record repair that
re-read an online learned-feedback arm's **1.0000** as saturation rather than
credit assignment, and a re-run which found that every previously published
matched-architecture number had been produced on a forward pass emitting **zero
spikes at any seed**. The general point is not that defects occurred. It is that
**every one of them produced plausible numbers, and none was caught by an arm's
own accuracy**; each was caught by a check that asked whether a run could have
produced its number for the wrong reason. Appendix C gives the ledger entry by
entry.

### 4.8 Open problems

**Order-sensitive comparators.** The contrast of §3.5 should be repeated against
read-outs that can themselves see order (the membrane-based Σₜ softmax(u[t]),
max-over-time and last-step read-outs) at the anchor. That is the experiment
that separates "attention consumes order" from "the only order-sensitive
component consumes order".

**Negative shuffled gains.** At twelve points after long training, and five
after short, attention on a task without order falls below the rate read-out, by
up to 0.27. A fixed
per-sample permutation that a positional code can fit is one account; drawing a
fresh permutation each epoch would test it.

**Synchrony at matched activity.** The synchrony increments are upper bounds
until they are re-measured with the input scaled so that both destruction
operators leave the substrate equally active.

**The width threshold** between h768 and h1024, with three preregistered rescue
attempts negative (Appendix E), and **the recurrent substrate**, where the
contrast has not been measured.

**A second dataset.** Everything here is SHD. The Spiking Speech Commands set is
the natural second test.

**The reference residual.** The re-measured reference's lead over this model,
0.1067, was once assigned to its 25-tap learned temporal kernel. It is the net of
offsetting terms rather than the kernel's value: ablating the kernel costs the reference +0.3111, restoring the
instrument's membrane recovers +0.1666, and **0.0378** remains unidentified
(§3.8).

---

## 5. Reproducibility

The anonymised supplement contains the instrument's full source, every archived
run the paper cites, the preregistrations, and the record checks.
`python3 scripts/verify_published_numbers.py` recomputes every headline number
from the archived runs, `python3 scripts/check_every_number.py` traces every
four-decimal number in the manuscript to the runs or to a named record, and
`python3 scripts/did_confidence_intervals.py --paper` regenerates Appendix K;
each was run inside the anonymised copy before submission, together with a
compile of the whole workspace. The two external crates the workspace depends on
are included at the versions its lock file pins. Appendix H lists the commands.

Runs are preserved in five corpora, all read by the number checks: waves 1–7,
waves 8–25, the x86-64 scope runs, waves 26–28, and wave 29, run on Apple
silicon with Apple's libm rather than glibc. The supplement's `RECORDS.md` gives
the path of each corpus and maps every provenance marker in the text to its
record.

## LLM-use disclosure

A large language model (Anthropic's Claude, used through the Claude Code agent) was used throughout this work: to write and review the simulator, training and analysis code; to launch, monitor and audit the experiment campaigns; to write the preregistrations, the checks and the audits of the results; and to draft and edit this manuscript. Every number in the paper is recomputed from the archived runs by scripts in the accompanying record rather than taken from model output, and the author reviewed the text and takes full responsibility for its content.

---

## Appendix A — Integrity limitations (protocol v2)

Cross-trial STDP pairing times (`ThreeFactor.last_spike`) are retained on canonical C1 while eligibility traces are cleared. Membrane and dendritic reset is incomplete relative to the C3 production path. Hidden thresholds are set to infinity during the integrate window, suppressing natural spiking. Assembly Calculus `project` is not exercised on the canonical loop (it is wired under `c1-project*` and fails G2 there). Exact-forward arms labeled “e-prop/DFA” are hybrid eligibility × transported modulators unless the true σ′ e-prop family is cited explicitly.

## Appendix B — Post-G2 harvest (banner)

G3 / G4 / hybrid H0 numbers and hashes: [`APPENDIX_POST_G2.md`](APPENDIX_POST_G2.md). Mechanism figure cells: [`PAPER_FIGURE_SPEC.md`](PAPER_FIGURE_SPEC.md) Figure M. These rows do not reopen G2.

## Appendix C — The withdrawal ledger in full

Seven results have been withdrawn from this package. §4.7 states the count and
the lesson; this appendix is the ledger itself, because a count without
diagnoses is not auditable.

**Episode 1: the 2026-08-19→22 record repair.** Four results withdrawn, three
of them PASSes:

- **`track-b-rescue` v130 online learned feedback alignment, PASS at 1.0000 (gap
  LCB 0.9988): withdrawn.** At v131 the arm reports `INVALID_HARNESS`: a
  ceiling-inverted warning fires on 3 of 20 learned-feedback seeds and the code
  **refuses to emit a PASS while it is present**. Re-read under both repairs it
  is 1.0000 against a ceiling of 1.0000 with zero variance: a *saturation*
  result, not a credit-assignment one.
- **The depth-collapse / deep-SNN scaling result: withdrawn** (v134
  `INVALID_HARNESS`: every depth-matched gradient ceiling was at chance).
- **`shd-scientific-sweep`: withdrawn.** It ran on synthetic 24-channel /
  16-timestep data and **never loaded SHD**
  ([`DEFECT_2026-08-20_SHD_SWEEP_IS_SYNTHETIC.md`](DEFECT_2026-08-20_SHD_SWEEP_IS_SYNTHETIC.md)).
- **The `live-transfer-rescue` arms: `INVALID_HARNESS`**, and the protocol was
  misnamed: it is matched-only, not live-engine (§2.2).

Separately, three gradient *references* were found at or near chance on tasks
their own treatments solve; two are diagnosed (`MatchedDeepGradient` collapses to
silence, `ShdEpropCeiling` is a constant predictor by a different mechanism).
None is used in any claim here.

**Episode 2: the 2026-08-25 matched re-run.** Every previously published
matched-architecture number was produced on a forward pass that emitted **zero
spikes at any seed**. Three more results were withdrawn:

- **The discrete EventProp-style spike-adjoint FAIL** (0.5000 → 0.9450 / 0.8900
  PASS). The failure mode is instructive: *a method whose entire mechanism is the
  spike had no spikes to differentiate through*, while every other arm could
  still separate classes by sub-threshold membrane rate. The defect was
  maximally misleading precisely on the arm that most depended on the broken
  quantity, and the prior explanation offered for that number ("discrete hard
  spike-gate adjoint ≠ continuous Wunderlich–Pehle") was an explanation for an
  artefact, and is retired with it.
- **Both RL broadcast contrasts** (0.5250 → 0.9100; 0.5113 → 0.7962). The
  reading they supported, "continuous magnitude without spatial directionality
  is insufficient on this gate", no longer has evidence behind it.

**The lead negative survived both episodes** on both forward graphs
([`RESULT_2026-08-25_MATCHED_ARCH_RERUN.md`](RESULT_2026-08-25_MATCHED_ARCH_RERUN.md)).

**What the record repair changed structurally.** Configuration hashes that did
not mix in a semantically load-bearing constant (`MATCHED_INPUT_SCALE`) were
found to name *two* experiments each; they are now retired rather than silently
reinterpreted (§2.4). Ceiling-health checks were added such that a reference at
chance **fails the harness** instead of scoring the treatment against it.
Machine-checked prose assertions were added over the SHD campaign, and now
number 125.

## Appendix D — Non-claims: the matched and engine programs

§0 states the SHD programme's boundaries against the literature and §4.6 its
scope limits. This appendix records the non-claims of the secondary programme,
reproduced from the claim freeze, because summarising them away is how a claim
ladder degrades.

No biology or cortex. No Assembly Calculus PASS (`project` is wired under
`c1-project-*` and FAILs there). No natural-spiking G2 verdict (`c1-spike-*` are
`INVALID_HARNESS`). No neuromorphic-hardware claim. **No impossibility in
principle**: these are scoped, operationalised negatives about one substrate
under one set of rules. No reopening of frozen hashes by threshold massage. No
widening of the lead FAIL to "any broadcast". Undertraining is not the cause: the
v22 arm at four times the epochs is still at chance. **No ranking among the
passing matched arms**: the gate saturates, so it can name one failure and
cannot order the survivors. No live-engine rescue from a matched PASS. No claim
that the discrete spike-adjoint is a negative result, and none that it is
equivalent to continuous EventProp. No digital-brain or brain-equivalence claim.

On the SHD programme, four further non-claims that §0 does not cover. The
headline fraction is of the **gain**, not of accuracy. The 0.087 calibration
residual is not a tuning gap. No temporal-*resolution* mechanism is claimed;
that hypothesis was withdrawn, and the `fixed-tN` ladder moves the opposite way.
And no recurrent-substrate win is claimed anywhere (§3.7).

## Appendix E — The width collapse at h1024

The width collapse in full. §3.5 states its shape; the arithmetic is here.

**Scope limits** (Figure 3): gain inverts at width h1024, and on a six-rung ladder that inversion is a **threshold rather than a continuing slope**: +0.1258 (h128), +0.0966 (h256), +0.0760 (h384), +0.0876 (h512), +0.0560 (h768), −0.1618 (h1024). The drop into h1024 is **0.2178**, **6.9×** the largest gap below it (0.0316) and more than twice the registered 3× bar, so the collapse sits between **h768 and h1024** and the rungs below it remain positive. **Every rung of this ladder is measured at `d32/L4`, and the inversion is a property of that read-out depth rather than of the width.** At h1024 and the same budget, `d32/L2` gains **+0.0405** in **20/20** seeds and `d32/L3` gains **+0.0371** in 18/20, against L4's −0.1318 in 3/20; the optimum in depth at that width is interior, and it is not established which of L2 or L3 holds it; they differ by 0.0034. So this row bounds *deep* read-outs at h1024 and does not bound h1024. What makes an arm collapse is still unexplained: L3 sits above the registered gradient-norm sickness threshold at 1.347 and gains anyway, which is why the numerical account was refused ([`RESULT_2026-08-28_W18_19_THE_DEPTH_OPTIMUM_IS_INTERIOR.md`](RESULT_2026-08-28_W18_19_THE_DEPTH_OPTIMUM_IS_INTERIOR.md)). The decay above the collapse is **not strictly ordered**: h384 and h512 are not distinguishable at twelve seeds (paired difference −0.0116, sd 0.0253, negative in 7 of 12), so no monotonicity is claimed. Gain is positive across geometries (+0.1090 on `channels-700`, +0.1491 on `published-10ms`), but 0.80 clearance is geometry-specific (0.7864 on `channels-700`). ([`RESULT_2026-08-21_W8_HEADLINE_SCOPE_IS_MEASURED.md`](RESULT_2026-08-21_W8_HEADLINE_SCOPE_IS_MEASURED.md); [`RESULT_2026-08-27_W15_17_THE_COLLAPSE_IS_A_THRESHOLD.md`](RESULT_2026-08-27_W15_17_THE_COLLAPSE_IS_A_THRESHOLD.md))

**The collapse is late, and stopping early avoids it** (wave 23, preregistered). No cell at `h1024/d32/L4` had ever run at any budget but e400. At **e100** the same arm gains **+0.0827** over its rate control in **12 of 12** seeds (against **−0.1318** in 3 of 20 at e400, an improvement of **+0.2145** against a registered bar of +0.10) and reaches **0.8153** where e400 gives 0.5768. At **e200** the gain is +0.0564 in 11 of 12, so the degradation is progressive rather than a cliff. **The control is what makes this interpretable**: `d32/L2`, which does not collapse, moves only **+0.0149** over the same budget change against a ±0.03 bar, so the effect is specific to the collapsing arm and this is *not* the weaker statement that e400 is past the optimum for every deep read-out here. The arm also **keeps its fit**: 0 of 12 e100 cells end above 3× their own best training loss, where 63 of 68 do at e400. The rate arm is budget-insensitive throughout (0.7326 / 0.7390 / 0.7386), so the movement is in the attention arm and not its baseline. ([`RESULT_2026-08-30_W23_THE_COLLAPSE_IS_LATE.md`](RESULT_2026-08-30_W23_THE_COLLAPSE_IS_LATE.md))

**That excludes overfitting**, which this section previously recorded as "neither excluded nor supported": at e400, 63 of 68 intact `d32/L4` cells reach a training-loss minimum around epoch 39–99 and end **56×** above it, while `d32/L1` (0/20), `d32/L2` (4/32) and the rate arm (0/32) hold theirs, and an arm that overfits keeps a low training loss ([`FINDING_2026-08-29_THE_H1024_COLLAPSE_IS_A_LOST_FIT.md`](FINDING_2026-08-29_THE_H1024_COLLAPSE_IS_A_LOST_FIT.md)).

**Three things this does not settle.** Not *why* the fit is lost: H15-1's refusal of the gradient-norm account stands, so the collapse is **located, bounded in time, and still unexplained**. Not h1024 in general: one width, one read-out depth, two budgets. And not the headline, which stays `h128` at e400; what changes is that the anchor budget is now known to be past the optimum at this width and depth. One caveat travels with it: wave 21's h1024 difference-in-differences of +0.1122 was measured at e400 between two arms **both** in late collapse, and nothing says what it would be at e100.

**What the h1024 collapse is not.** Three preregistered levers (surrogate scale 0.5 and 0.25, and gradient clipping at 1000.0) were run at h1024/d32/L4 to test whether the inversion is an optimisation failure. All three are **negative and worse than the arm they were meant to rescue** (−0.2106, −0.2565, −0.0904). Clipping moves the median epoch-mean gradient norm from 55.494 to 11.660, a real effect in the intended direction, and accuracy does not follow. **No mechanism for the collapse is offered here**; what wave 23 adds above is a bound on *when* it happens, not an account of *why*.

**And it is not the temporal-order account in disguise.** If the inversion and the mechanism were the same phenomenon, then where the read-out buys nothing there should be no order-dependent benefit left to destroy. That prediction was registered before the cells existed and it **failed**: at h1024 the difference-in-differences is **+0.1122** in **10 of 12** seeds against a registered ceiling of +0.02, while the gain over those same twelve seeds is **−0.1618**. The verdict stands as registered; what it licenses changed with the amended claim (§3.5). This is a point with no gain to lose, and its difference-in-differences is carried by the shuffled arm: attention's shuffled gain is **−0.2740**, so once order is gone the attention arm falls further below the rate arm than it already sits. That is not the read-out consuming order where it performs worse than no read-out at all, and it is reported with the other four such points as a limitation. ([`RESULT_2026-08-29_W21_THE_MECHANISM_TRAVELS_BUT_ITS_SIZE_DOES_NOT.md`](RESULT_2026-08-29_W21_THE_MECHANISM_TRAVELS_BUT_ITS_SIZE_DOES_NOT.md))

## Appendix F — What the advantage runs through

This is §3.5's account of what the read-out consumes, in full: the three manipulations that ask what in the read-out consumes temporal order, rather than how much order the data carries. All are measured at a single operating point, and §3.5 states that limit where the result is used.

**What the advantage runs through** (waves 26–29, each preregistered). Everything above destroys order in the *data*; three further manipulations ask what in the read-out consumes it. **Position:** a `no-position` arm keeps every parameter and every spike and deletes only the read-out's positional encoding, and its gain falls to **+0.0506** from **+0.1258**, a cost of **0.0752**, above the +0.03 bar in **12 of 12**. Registered beside it as a question and answered *no*: deleting the read-out's access leaves +0.0506 where destroying order in the data leaves **+0.0050**, a difference of **0.0456** outside a ±0.03 band, so something besides the positional code carries order here and this paper does not say what. **A timescale:** permuting bins inside a sliding window destroys order below it and preserves it above, and across windows of 2 to 256 bins the difference-in-differences climbs **+0.0066 to +0.1260** against the full shuffle's +0.1208, reaching half its effect at **130.33 bins (261 ms)**, about 36% of the 716 ms mean utterance. **Not spike counts:** random deletion at p30 costs the rate arm **0.0124** and clears the 0.03 bar in **0 of 12**, so a null measured under it means nothing; laddering the rate makes **p70** the smallest sensitive rung (**0.0388**, 11/12) and p90 costs **0.1080** at 12/12, with no cell voided and p90 cells still predicting all twenty classes at **0.5982** against chance 0.05. Against that instrument the advantage does not fall but **grows**: +0.1390 at p70 and **+0.1640** at p90, the substrate losing 0.1080 where the read-out loses **0.0698**. **That clause is recorded NOT MET and is reported as NOT MET**: a two-sided band was registered for a one-sided question and fires identically when the advantage strengthens. **The asymmetry then got its own registration**: wave 29, two one-sided clauses on a seed block the frozen analyser enforces by value, so the replacement bar could not be evaluated on the cells that motivated it. On 48 cells the advantage is not hurt by count destruction (**12 of 12**) and **grows** by more than the campaign's 0.03 bar (DiD **−0.0467**, **11 of 12**), with the rate arm's p90 cost at **+0.1112** showing the instrument still feels its own hand. Those cells are `aarch64-apple-darwin`, not the glibc fleet (registered in an amendment committed before the first cell), so every clause is a within-block paired contrast, the platform gate puts the anchor **0.0006** from the fleet's, and no wave-29 accuracy is tabulated beside a fleet accuracy. What stands is the comparison: destroying order costs the read-out **0.1208**, destroying counts costs it nothing. **All three properties are measured at one operating point** (h128, feed-forward, `published-2ms`, `d32/L4`, e400), and none of them transfers to h1024 or to a recurrent substrate on this evidence; the difference-in-differences itself travels to twenty-one points, its decomposition does not. Read-out access is separable from input structure throughout: a `hidden-shuffled` operator permutes the time axis the read-out sees and leaves the substrate's input alone, and the rate arm pays **exactly 0.0000**, twelve of twelve cells byte-identical on every scientific field. ([`RESULT_2026-09-04_W26_SATURATION_IS_REAL_AND_NOT_SPECIFIC.md`](RESULT_2026-09-04_W26_SATURATION_IS_REAL_AND_NOT_SPECIFIC.md); [`RESULT_2026-09-05_W27_THE_TIMESCALE_IS_261_MS.md`](RESULT_2026-09-05_W27_THE_TIMESCALE_IS_261_MS.md); [`RESULT_2026-09-06_W28_THE_READ_OUT_SURVIVES_WHAT_THE_SUBSTRATE_CANNOT.md`](RESULT_2026-09-06_W28_THE_READ_OUT_SURVIVES_WHAT_THE_SUBSTRATE_CANNOT.md); [`RESULT_2026-09-09_W29_THE_ASYMMETRY_HAS_ITS_OWN_BAR.md`](RESULT_2026-09-09_W29_THE_ASYMMETRY_HAS_ITS_OWN_BAR.md))

## Appendix G — The calibration criteria and the 0.80 floor

Both paragraphs are about the third-party reference this model is calibrated against, not about the attention read-out; §4.6 states each conclusion where it is used.

Criteria 3 and 4 (`clean_reference` and `historical_reference`) were false for a **provenance** reason rather than an accuracy one: the six third-party PyTorch reference artifacts recorded a `source_fingerprint` frozen on 2026-07-27 that every later kernel edit had moved, while their recorded accuracies already met the requirement. Those six cells were re-run on 2026-08-23 and **every one reproduced its archived value to every recorded digit**: a 150-epoch stochastic PyTorch training run, on CPU, a month later, in a rebuilt environment. Both gates now read `true` and `matrix_authorized` is `true` ([`RESULT_2026-08-23_REFERENCE_RERUN.md`](RESULT_2026-08-23_REFERENCE_RERUN.md)). What still blocks calibration is criterion 5, the Python mirror of the attention axis, which does not exist; and `SHD_INSTRUMENT_STATE` remains a compile-time `Uncalibrated`, a second gate in series with the first.

**The 0.80 `CELL_PASS` floor should not be read as a standard the instrument is failing to meet.** It was derived from one configuration of a reference that is a different model class, and four preregistered ablations of that reference show three of its choices are neutral or harmful to it: its second hidden layer costs 0.0145, its batchnorm appears to cost 0.0058, and its non-spiking summed readout buys 0.0012. Only its dropout clearly earns its place ([`RESULT_2026-08-24_EVERY_CONFIGURABLE_DIFFERENCE_IS_MEASURED.md`](RESULT_2026-08-24_EVERY_CONFIGURABLE_DIFFERENCE_IS_MEASURED.md)). A fifth ablation, which removes the reference's temporal kernel rather than a configuration value, is worth **0.3111**, so the floor rests overwhelmingly on one architectural choice the instrument does not make (§3.8).

## Appendix H — Reproduction commands

Every claim in this paper is reachable from a clean checkout of the
supplement with:

```
cargo test --locked --workspace
./scripts/record_checks.sh      # recomputes every published number
python3 scripts/did_confidence_intervals.py --paper   # Appendix K
python3 scripts/build_paper.py  # this manuscript, page limit enforced
```

The first builds and tests the simulator; the second re-derives every number
the paper prints from the archived runs and fails on any mismatch.

## Appendix I — The resolution ladder

The resolution ladder in full, with the per-rung table. §3.5 states the direction and the withdrawn S-5 test where the result is used.

**Temporal resolution is an axis, and the gain falls as bins get finer** (Figure 4). The `published-Nms` test of this (S-5) was **refuted and is withdrawn**: that family moves bin width and sequence length together, so a single number cannot be attributed to either. Re-asked on `fixed-tN`, which holds a 1400 ms window fixed and varies only the number of frames, the read-out helps at **every** rung and the gain is monotone in resolution:

| contract | bin | `ff+fixed` | d32/L4 | gain | gain > 0 | ≥ 0.80 |
|---|---:|---:|---:|---:|---:|---:|
| `fixed-t100` | 14.0 ms | 0.6672 | 0.8599 | **+0.1927** | 12/12 | 12/12 |
| `fixed-t250` | 5.6 ms | 0.6844 | 0.8594 | **+0.1751** | 12/12 | 12/12 |
| `fixed-t500` | 2.8 ms | 0.7069 | 0.8543 | **+0.1474** | 12/12 | 12/12 |

Table: The resolution ladder: a fixed 1400 ms window cut into 100, 250 or 500 bins, n = 12 seeds per rung.

gain(t500) − gain(t100) = **−0.0453** against a two-sided bar of 0.03, so the advantage **shrinks with finer resolution**, the opposite of the direction S-5 predicted, on the axis S-5 could not isolate. The rate baseline rises +0.0397 across the same ladder, inside the registered 0.05 confound bar, while the attention arm moves only −0.0056: most of the shrinking gain is the rate read-out improving at finer bins, not the attention read-out losing accuracy. All three rungs clear the 0.80 gate at 12/12, the coarsest most comfortably. ([`RESULT_2026-08-22_W10_RESOLUTION_LADDER.md`](RESULT_2026-08-22_W10_RESOLUTION_LADDER.md))

## Appendix J — A converse result, and what it does not license

@xu2026reward report that a frozen, backprop-trained LLM already carries a sparse population encoding value and, downstream of it, a population signalling reward prediction error: a reward subsystem that is learned, layered and addressed rather than a global scalar. It is the closest published result to Gate G2 from the opposite direction, and the two are **converses rather than competitors**: theirs describes what a reward subsystem looks like *after* backprop has built one, while G2 finds that a broadcast ±1 scalar does not *build* one on a matched dense-LIF forward, where graded DFA (0.9925) and per-neuron RL (0.9950) both pass on the identical forward (§3.1, §4.1).

**It is cited for its result, with its own limits stated.** The evidence is correlational probing plus ablation on frozen models no larger than 14B; absolute value-neuron counts are not reported, only curves against pruning ratio; no IoU values are given for the transfer claim; and the causal ablation zeroes 1% of a single early residual layer, an intervention whose random-1% control is the weaker of the two it is compared against. The paper contains no spiking, no local learning rule, and no neuromodulation as a *training* signal, so nothing in it is evidence about BINN in either direction.

**What no experiment here has asked.** This paper establishes *that* broadcast ±1 fails and that graded DFA and per-neuron RL pass. It has never asked whether the passing arms *contain* a sparse value population and the failing arm does not. That is a probe on hidden-activity traces the matched runs already export, it costs no new instrumentation, and it is **unregistered and unrun**: no number is claimed for it here, and none may be cited until it exists under a config hash. The same applies to every other candidate in that reading note.

**Placement.** This belongs in §4.1 and is here instead for one measured reason: the main text is at 9 of 9 permitted pages, and inserting even a single sentence of it into §4.1 displaces the five full-width figures enough to cost a tenth page. The framing is the deliverable; the page is not available.


## Appendix K — Every operating point, with intervals

Every bin-shuffled operating point at both training budgets. *Bins*: `2ms` and
`10ms` are `published-Nms`, `t100`/`t250`/`t500` are `fixed-tN`; *chan.*:
`a5` is `adjacent-sum-5`, `700` is `channels-700`; $d$/$L$ is attention
width and number of blocks; *ep.* is epochs; $>0$ counts seeds with a positive
DiD. Gains are the
attention arm's accuracy minus the rate arm's over the same seeds; the DiD is
gain(intact) − gain(shuffled). Brackets are paired percentile-bootstrap 95%
intervals over seed quadruples (10,000 resamples). **†** marks an intact gain
below the +0.03 bar: a point where the read-out has no gain to lose, so a
positive DiD there reflects the attention arm falling below the rate arm once
order is destroyed (§3.5); **‡** marks a gain above the bar whose interval is
not. Width-1024 four-block gains are over the twelve quadruple seeds. The table
is the output of `scripts/did_confidence_intervals.py --paper`, and a test fails
if the two differ.

| width | bins | chan. | $d$/$L$ | ep. | $n$ | gain, intact | gain, shuffled | DiD | $>0$ |
|----:|----|-----|----|--:|-:|------------------------|-----------------------|-----------------------|----:|
| 128 | t100 | a5 | 32/4 | 400 | 12 | +0.193 [+0.184, +0.201] | +0.060 [+0.054, +0.067] | +0.132 [+0.119, +0.144] | 12/12 |
| 128 | t100 | a5 | 32/4 | 100 | 12 | +0.190 [+0.179, +0.199] | +0.075 [+0.070, +0.080] | +0.114 [+0.102, +0.127] | 12/12 |
| 128 | t250 | a5 | 32/4 | 400 | 12 | +0.175 [+0.169, +0.181] | +0.063 [+0.055, +0.070] | +0.112 [+0.103, +0.122] | 12/12 |
| 128 | t250 | a5 | 32/4 | 100 | 12 | +0.178 [+0.167, +0.189] | +0.079 [+0.073, +0.084] | +0.099 [+0.086, +0.113] | 12/12 |
| 128 | t500 | a5 | 32/4 | 400 | 12 | +0.147 [+0.139, +0.156] | +0.031 [+0.023, +0.040] | +0.116 [+0.106, +0.125] | 12/12 |
| 128 | t500 | a5 | 32/4 | 100 | 12 | +0.152 [+0.143, +0.161] | +0.041 [+0.033, +0.048] | +0.111 [+0.101, +0.124] | 12/12 |
| 128 | 10ms | a5 | 32/4 | 400 | 12 | +0.149 [+0.138, +0.160] | +0.053 [+0.046, +0.061] | +0.096 [+0.085, +0.107] | 12/12 |
| 128 | 10ms | a5 | 32/4 | 100 | 12 | +0.153 [+0.144, +0.162] | +0.061 [+0.057, +0.066] | +0.092 [+0.080, +0.103] | 12/12 |
| 128 | 2ms | a5 | 32/1 | 400 | 12 | +0.042 [+0.032, +0.052] | −0.049 [−0.056, −0.042] | +0.091 [+0.082, +0.099] | 12/12 |
| 128 | 2ms | a5 | 32/1 | 100 | 12 | +0.084 [+0.070, +0.097] | −0.015 [−0.020, −0.010] | +0.099 [+0.087, +0.111] | 12/12 |
| 128 | 2ms | a5 | 32/2 | 400 | 12 | +0.083 [+0.072, +0.095] | −0.031 [−0.037, −0.024] | +0.115 [+0.101, +0.129] | 12/12 |
| 128 | 2ms | a5 | 32/2 | 100 | 12 | +0.120 [+0.110, +0.129] | +0.002 [−0.004, +0.008] | +0.118 [+0.106, +0.131] | 12/12 |
| 128 | 2ms | a5 | 32/4 | 400 | 32 | +0.127 [+0.123, +0.132] | +0.007 [+0.003, +0.011] | +0.121 [+0.114, +0.127] | 32/32 |
| 128 | 2ms | a5 | 32/4 | 100 | 12 | +0.150 [+0.142, +0.158] | +0.024 [+0.016, +0.032] | +0.126 [+0.115, +0.136] | 12/12 |
| 128 | 2ms | a5 | 64/4 | 400 | 12 | +0.138 [+0.130, +0.145] | +0.005 [−0.000, +0.011] | +0.133 [+0.124, +0.142] | 12/12 |
| 128 | 2ms | a5 | 64/4 | 100 | 12 | +0.168 [+0.162, +0.174] | +0.030 [+0.021, +0.039] | +0.138 [+0.130, +0.147] | 12/12 |
| 128 | 2ms | 700 | 32/1 | 400 | 12 | +0.024 [+0.009, +0.039]† | −0.113 [−0.120, −0.106] | +0.137 [+0.120, +0.154] | 12/12 |
| 128 | 2ms | 700 | 32/1 | 100 | 12 | +0.059 [+0.045, +0.073] | −0.055 [−0.062, −0.048] | +0.114 [+0.104, +0.124] | 12/12 |
| 128 | 2ms | 700 | 32/4 | 400 | 12 | +0.109 [+0.086, +0.128] | −0.003 [−0.011, +0.006] | +0.112 [+0.088, +0.133] | 12/12 |
| 128 | 2ms | 700 | 32/4 | 100 | 12 | +0.150 [+0.140, +0.159] | +0.025 [+0.018, +0.032] | +0.125 [+0.112, +0.137] | 12/12 |
| 256 | 2ms | a5 | 32/1 | 400 | 12 | +0.015 [+0.005, +0.025]† | −0.061 [−0.073, −0.049] | +0.076 [+0.062, +0.091] | 12/12 |
| 256 | 2ms | a5 | 32/1 | 100 | 12 | +0.061 [+0.049, +0.075] | −0.022 [−0.028, −0.016] | +0.083 [+0.068, +0.101] | 12/12 |
| 256 | 2ms | a5 | 32/4 | 400 | 12 | +0.097 [+0.087, +0.107] | +0.010 [+0.003, +0.018] | +0.086 [+0.072, +0.101] | 12/12 |
| 256 | 2ms | a5 | 32/4 | 100 | 12 | +0.125 [+0.112, +0.137] | +0.025 [+0.018, +0.033] | +0.100 [+0.082, +0.116] | 12/12 |
| 384 | 2ms | a5 | 32/4 | 400 | 12 | +0.076 [+0.067, +0.086] | −0.001 [−0.010, +0.008] | +0.077 [+0.065, +0.088] | 12/12 |
| 384 | 2ms | a5 | 32/4 | 100 | 12 | +0.121 [+0.113, +0.129] | +0.032 [+0.024, +0.040] | +0.089 [+0.077, +0.100] | 12/12 |
| 512 | 2ms | a5 | 32/1 | 400 | 12 | +0.004 [−0.011, +0.019]† | −0.085 [−0.094, −0.076] | +0.089 [+0.073, +0.103] | 12/12 |
| 512 | 2ms | a5 | 32/1 | 100 | 12 | +0.057 [+0.044, +0.070] | −0.016 [−0.021, −0.012] | +0.073 [+0.060, +0.086] | 12/12 |
| 512 | 2ms | a5 | 32/4 | 400 | 12 | +0.088 [+0.078, +0.097] | −0.009 [−0.016, −0.001] | +0.097 [+0.083, +0.109] | 12/12 |
| 512 | 2ms | a5 | 32/4 | 100 | 12 | +0.107 [+0.101, +0.113] | +0.021 [+0.017, +0.025] | +0.086 [+0.081, +0.091] | 12/12 |
| 768 | 2ms | a5 | 32/2 | 400 | 12 | +0.042 [+0.034, +0.051] | −0.019 [−0.027, −0.011] | +0.061 [+0.051, +0.073] | 12/12 |
| 768 | 2ms | a5 | 32/2 | 100 | 12 | +0.067 [+0.057, +0.077] | +0.008 [−0.000, +0.015] | +0.059 [+0.049, +0.069] | 12/12 |
| 768 | 2ms | a5 | 32/4 | 400 | 12 | +0.056 [+0.022, +0.079]‡ | −0.132 [−0.182, −0.083] | +0.188 [+0.144, +0.234] | 12/12 |
| 768 | 2ms | a5 | 32/4 | 100 | 12 | +0.101 [+0.093, +0.108] | +0.029 [+0.023, +0.035] | +0.071 [+0.059, +0.082] | 12/12 |
| 1024 | 2ms | a5 | 32/1 | 400 | 12 | −0.016 [−0.022, −0.010]† | −0.083 [−0.092, −0.077] | +0.068 [+0.058, +0.078] | 12/12 |
| 1024 | 2ms | a5 | 32/1 | 100 | 12 | +0.044 [+0.034, +0.055] | −0.019 [−0.025, −0.012] | +0.063 [+0.055, +0.072] | 12/12 |
| 1024 | 2ms | a5 | 32/2 | 400 | 12 | +0.039 [+0.031, +0.048] | −0.027 [−0.033, −0.021] | +0.066 [+0.054, +0.078] | 12/12 |
| 1024 | 2ms | a5 | 32/2 | 100 | 12 | +0.055 [+0.044, +0.066] | +0.009 [+0.000, +0.016] | +0.047 [+0.033, +0.058] | 11/12 |
| 1024 | 2ms | a5 | 32/3 | 400 | 12 | +0.032 [+0.016, +0.046]‡ | −0.035 [−0.044, −0.027] | +0.068 [+0.049, +0.082] | 11/12 |
| 1024 | 2ms | a5 | 32/3 | 100 | 12 | +0.071 [+0.063, +0.079] | +0.021 [+0.015, +0.026] | +0.050 [+0.044, +0.056] | 12/12 |
| 1024 | 2ms | a5 | 32/4 | 400 | 12 | −0.162 [−0.210, −0.118]† | −0.274 [−0.360, −0.210] | +0.112 [+0.037, +0.203] | 10/12 |
| 1024 | 2ms | a5 | 32/4 | 100 | 12 | +0.083 [+0.073, +0.092] | +0.020 [+0.015, +0.026] | +0.063 [+0.052, +0.073] | 12/12 |

Table: Every operating point at both budgets, with paired bootstrap 95% intervals; † and ‡ as defined above.
