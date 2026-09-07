# What a time-axis read-out buys is temporal order: a difference-in-differences on SHD

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

Adding a time-axis attention read-out to a spiking network raises SHD accuracy
from 0.7057 to **0.8332** (gain **+0.1275**, positive in 32/32 seeds). That much
is unsurprising. The result this paper is built on is the **conditional**: when
the temporal order of the input is destroyed by permuting time bins —
independently per sample, in **both the training and test splits**, so the task
itself becomes rate-solvable — the read-out's *advantage over a rate read-out*
collapses by **+0.1347**, while the rate read-out loses only **+0.0142** of its
own. That SHD depends on temporal information is established; what has not been
measured is which *component's* contribution is the order-dependent one. This is
a difference-in-differences on the **gain**, not on accuracy.

Preregistered waves carried the control to **all twenty-one operating points**
of this instrument, at **two training budgets**, and every one clears its +0.03
bar. Two further controls decide what "order" means. Time reversal displaces
spikes *further* than shuffling while destroying no information, and its
difference-in-differences is flat — so the cost is not brittleness to
displacement. Destroying cross-channel synchrony *as well as* order costs
**+0.1308**, **+0.1038** and **+0.0575** more at three binning contracts and
nothing at two others, so **the read-out reads synchrony as well as order, and
how much depends on the contract**.

The paper's own registered prediction failed: the size of the effect does not
track the gain (Spearman **ρ = −0.1430** against a bar of +0.829). We claim
presence, not proportion, and report four scope limits — width, read-out depth,
geometry, and a substrate on which the question could not be asked.

Two things make those numbers citable, and we report both as results. Every
claim carries a numeric bar fixed before the run, on a frozen configuration
hash, behind a kill-gate that is permanent once failed (§2.4). And **seven
results have been withdrawn from this package, three of them PASSes** —
including a 1.0000 — across two repair episodes, one of which found an entire
matched-architecture suite running on a forward pass that emitted **zero spikes
at any seed**. Every one of those defects produced plausible numbers, and none
was caught by an arm's own accuracy (§4.7).

## Abstract — matched-architecture kill gate (secondary program)

Broadcast ±1 three-factor plasticity — surrogate eligibility multiplied by a single ±1 reward — fails a preregistered accuracy and gap bar when the dense leaky-integrate-and-fire forward pass is held identical to a SuperSpike backpropagation-through-time reference. The arm remains at chance on **both** matched forward graphs (feed-forward 0.5000, gap lower confidence bound 0.0000; recurrent 0.5100, LCB −0.0192), against a gradient reference at 1.0000, n = 20 seeds. Every other rule tested on that forward now clears the gate: graded direct feedback alignment (0.9925 / 0.9875), directional REINFORCE with frozen per-neuron feedback (0.9950 / 0.9812), broadcast graded error (0.9975), and a discrete EventProp-style spike-adjoint (0.9450 / 0.8900). **The task therefore separates one rule from a field that otherwise saturates, and it no longer ranks the field**: with every reference at exactly 1.0000, each of those passes reduces to "the arm scored above 0.75". Live k-WTA transfer of the matched REINFORCE and DFA families remains a scoped **negative** across twelve gap-close variants (v13–v24), best gap LCB 0.3127 against a 0.5 threshold.

> **Provenance of these numbers, and it is load-bearing.** The figures above are from a 2026-08-25 re-run under `MATCHED_INPUT_SCALE = 2.0`. Every previously published matched-architecture number was produced on a forward pass that emitted **zero spikes at any seed**, and the arms that most depended on spikes were the ones it most misrepresented — the discrete spike-adjoint read 0.5000 there and reads 0.9450 here, because a method that differentiates through spike times had none. That claim is **withdrawn**, along with the two RL broadcast contrasts (0.5250 → 0.9100 and 0.5113 → 0.7962). See [`RESULT_2026-08-25_MATCHED_ARCH_RERUN.md`](RESULT_2026-08-25_MATCHED_ARCH_RERUN.md).

**Two scope statements are load-bearing and are stated here rather than in a footnote.** First, the 80-epoch schedule undertrains *every* rule on it: raising the reference's budget alone lifts it from 0.9013 to 1.0000 by e640, so `gap_closed` at the canonical budget divides by a reference that is still climbing, and values above 1 are an artefact of the denominator rather than a result. The defensible reading of the ordering between local rules and BPTT at that budget is a statement about **learning speed**, not about a ceiling. Second, the matched task saturates: with every arm reaching 1.0000 at high budget it can no longer separate them, so no ceiling comparison on this task survives convergence.

**Honesty notes.** On the DFA schedule a broadcast-*graded* contrast reaches 0.9975, so the lead negative is ±1 three-factor specifically, not "any broadcast" — and since every other rule tested is now at or near ceiling, the negative is specific in a stronger sense than that phrasing suggests. Locality as a necessary ingredient is evidenced on one-layer XOR, not by the coincidence task alone. We do not claim biological realism, Assembly Calculus success, or impossibility of local learning in principle.

## 0. What is new here, and what is not

> **Provenance of the citations in this section.** The literature positioning
> below was assembled by a search pass on 2026-08-27 and **was read against its
> primary sources on 2026-09-03**. Each source was retrieved as a PDF, the cited
> number located in the source's own words, and the bibliographic fields taken
> from the arXiv API and Crossref. **Three claims were wrong**: the frontier
> band's upper bound, the attribution of that band, and the description of Yu et
> al. (2025). All three are corrected below, and every check — including the
> failures — is recorded with the source hashes in
> [`CITATIONS_2026-09-03_SECTION_0_AGAINST_PRIMARIES.md`](CITATIONS_2026-09-03_SECTION_0_AGAINST_PRIMARIES.md)
> and gated by `scripts/check_section0_citations.py`. Four numbers seen during
> the original search (Pfa-SNN 96.26, Event-SSMA 95.90, SpikeSCR 95.60,
> d-cAdLIF 94.85) came only from a secondary comparison table; this pass did not
> reach their primaries either, and they remain **excluded** from every claim
> here. Percentages in this section are quoted from published papers and are
> **not** machine-checked against cells on disk, which is what every SHD number
> elsewhere in this paper is.


This paper's SHD result sits in a populated field and the boundary is stated
here rather than left to a reader.

**Not new: a time-axis attention mechanism in a spiking network on SHD.**
TA-SNN (Yao et al., ICCV 2021) applies squeeze-and-excitation attention over the
time axis and reports 91.08%; STSC-SNN (Yu et al., 2022) places temporal
attention inside the synaptic connection and reports 92.36%. Attention as a
*temporal read-out* is older still outside spiking networks — attentive
statistics pooling (Okabe et al., Interspeech 2018) is the same idea on speaker
embeddings. The specific placement used here — attention **only** at the
read-out, replacing the field's default unweighted Σₜ softmax(u[t]) — appears
unoccupied, but a configuration gap is not a mechanism.

**Not new: that SHD depends on temporal information.** The dataset's own authors
constructed spike-count-only variants — patterns "without temporal information" —
and report that no linear or nonlinear classifier on them could "surpass the 60 %
accuracy mark for the SHD" (Cramer et al., IEEE TNNLS 33(7):2744–2757, 2022). The
Neuromorphic Sequential Arena (IJCAI 2025) removes temporal propagation
**model-side** rather than data-side and reports SHD falling 86.48 → 68.51 (its
Table S4). Two destruction operators on two different sides of the model, one
conclusion, both prior to this work. Neither separates *order* from the other
things a spike train carries, which is what §3.5 does and what makes synchrony
visible there as a second term.

**Not new, and worth conceding plainly: the accuracy.** The SHD frontier runs
from **95.07 ± 0.24%** — learned delays (DCLS, ICLR 2024), the best published
result from a model of this general class — to **96.3%**, held by a state-space
model (S7, arXiv:2410.03464) whose SHD entry uses data augmentation and
continuously-valued rather than spiking output. Adaptation reaches **95.81 ±
0.56%** in between (SE-adLIF, Nature Communications 2025). This instrument's
0.8332 is not in that band and is not offered as if it were. It is close to the
best-effort no-delay recurrent baseline the dataset's authors themselves report
(83.2 ± 1.3% at 1024 neurons with augmentation), which is where an architecture
carrying no temporal kernel should land, and is the external corroboration §3.8
argues for from ablation alone.

**Prior work that corroborates the reversal control, rather than the shuffle
result.** Yu et al. (arXiv:2507.16043, 2025) reverse time on SHD with an operator
that, in their words, "perturb[s] spatio-temporal spike patterns but leave[s]
interspike intervals and coincidence information unchanged" — the same
information-preserving property §3.5's `reversed` control was built to have. Their
finding is that "when axonal delays are not used, networks perform well under time
reversal, whereas networks trained with delays perform poorly." This instrument
has no delays and no temporal kernel, and its reversal difference-in-differences is
flat at all three points measured. **That is an independent prediction of what
§3.5 observes**, and it is the reason the reversal result is reported here as a
control that behaved as the literature says it should rather than as a surprise.

**New: which component's contribution is the order-dependent one.** Every result
above measures how much *accuracy* survives destroying temporal structure. None
measures how much of a *specific component's marginal contribution* survives it.
The contrast in §3.5 is a difference-in-differences — attention's shuffle cost
against the rate read-out's own, on the same seeds, same splits, same
destruction operator — and we find no published equivalent for any read-out on
any neuromorphic benchmark.

**The claim is about presence, not proportion.** Measured across **all
twenty-one** operating points and at two training budgets, the order-dependence is
present everywhere and its *size* is uncorrelated with the gain (§3.5). The contribution is therefore a statement about what the
read-out consumes, and explicitly not a decomposition of the gain into an
order-dependent share and a remainder.

**New, and unsupported in either direction: the width collapse.** No published
work reports an attention read-out degrading with hidden width, and none reports
gradient pathology in an attention read-out over spike trains. Width normally
*helps* on SHD (Cramer et al.: 1024 neurons → 76.5%; Bittar & Garner: RadLIF at
3×128 → 3×1024 improves 92.88 → 94.62). §3.5's inversion is therefore an anomaly
against the baseline expectation with no citation to lean on, and the parsimonious
alternative — overfitting on the **8,156** training samples every cell in this
campaign records loading — is **not excluded by anything in this paper**. Cramer
et al. report overfitting on SHD repeatedly, and that noise injection reduced it.

**A caveat on the benchmark itself.** SHD ships no validation set. Baronig et al.
(2025) — the SE-adLIF paper cited above, not a separate source — report the same
model at 95.81 ± 0.56 validating on the test set "to ensure comparability" and
93.79 ± 0.76 with 20% of training held out instead: a two-point gap that is a
property of the protocol, not of the model.
Differences below ~1.5 points between published SHD numbers are not reliably
meaningful, and that applies to this paper's comparisons as much as anyone's.

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
Then destroy the temporal order of the input — permuting time bins
independently per sample, in the training split as well as the test split, so
that the task itself becomes rate-solvable rather than merely harder — and
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
two". **Time reversal** displaces further — 0.9986 of entries relocated at a
mean 145.2 bins, against bin-shuffling's 0.9972 and 109.2 — while preserving
counts, cross-channel synchrony and every interval; applied to the training
split too, it leaves a task isomorphic to the intact one. Its
difference-in-differences is negative at all three points measured, so the cost
is in what the displacement destroys and not in the displacement.
**Channel-shuffling** destroys order and synchrony together, and the gap between
the two operators is what synchrony is worth. **A second training budget**
separates the mechanism from the schedule it was measured on.

**Coverage, because a mechanism claim at one configuration is a configuration
result.** The contrast is measured at every one of the twenty-one operating
points this instrument defines — widths 128 to 1024, both binning contracts,
both channel geometries, five read-out depths — and at two budgets at each.

**What failed is reported at the same weight as what held.** The registered
prediction that the shuffle cost would *track* the gain across width is refuted
(Spearman ρ = −0.1430 against a one-tailed n = 6 bar of +0.829); the width at
which the gain is smallest carries the largest difference-in-differences. So the
claim this paper defends is that the read-out's contribution **is**
order-dependent, and explicitly not that the gain decomposes into an
order-dependent share and a remainder. Section 4.1 states the four scope limits
that travel with it.

**A secondary programme is reported in the appendix.** A preregistered
matched-architecture kill gate holds a dense-LIF forward, width, frames,
read-out, splits and seeds fixed and changes only the update rule, and finds
that a broadcast ±1 three-factor rule fails a task every other rule tested
saturates, while live muted-θ / k-WTA transfer fails outright. It is the
programme this manuscript was originally built around; it is supporting material
now, and it is bounded by a reference that saturates the task (Appendix,
§3.6). Throughout, we refuse claims about biology, neuromorphic hardware, and
the impossibility of local learning in principle.

---

## 2. Methods

### 2.1 Matched dense-LIF control

Matched-arch protocols fix the SurrogateLifReference / dense-LIF forward and vary only the learner. Protocol v4 compares production-style **broadcast ±1 three-factor** updates to SuperSpike BPTT. Protocol v5 evaluates graded error with fixed-random DFA feedback. Protocol v12 evaluates directional REINFORCE × frozen per-neuron `B_i` as the primary arm. Protocol v130 (`track-b-rescue`) evaluated continuous RPE critic scalar broadcast vs online learned feedback alignment ($B_i \leftarrow B_i + \eta_B r (a_i - p_i) x_i$), but was withdrawn under v131 due to ceiling-inversion defects. Protocol v28 (`c1-eventprop-5bb083d5e88d0ad2`) is a discrete EventProp-style spike-adjoint H2H vs SuperSpike on the same matched forward. Gates reuse Gate G2 numeric thresholds (accuracy floor 0.65; gap LCB > 0.5) under fresh hash families that do not reopen `c1-118207fbc3eaba53`.

### 2.2 Engine C1 / Gate G2

The C1 harness encodes coincidence sequences with a latency encoder, integrates with muted hidden thresholds on the canonical path, selects winners by membrane-score k-WTA, force-fires winners and readouts, and applies three-factor plasticity. Gradient and eligibility references train on the same frozen splits. Live transfer of matched credit families onto this substrate is the v13–v24 package. **Protocol v131 (`live-transfer-rescue`)** is a misnamed **matched-only** online-FB schedule contrast (no Engine / no muted-θ / no live k-WTA) and is **not** a live-transfer result.

### 2.3 SHD attention readout

On the Spiking Heidelberg Digits dataset, we evaluate a time-axis self-attention read-out (`+attn`) over LIF hidden activations, comparing against a standard mean-rate read-out. The read-out is **additive**: at $W_a = 0$ the arm reduces exactly to its non-attention counterpart, so any difference between them is attributable to the read-out and not to a perturbed spiking forward.

Two properties of the block are stated explicitly because they bound what the result can mean. It is **not causal** — every timestep attends to every other, through a full $[T, T]$ row-softmax with no mask, so the arm consumes the whole utterance at inference. And it is **single-head**: one $q, k, v$ triple of shape $[d, d]$ per block, with depth supplied by stacking $L$ blocks rather than by splitting heads. Positional information is a fixed sinusoidal code over normalised position; without it, mean-pooled attention is permutation-invariant and the block would be blind to the order it exists to use. Every arm here is trained by the matched BPTT instrument, not by the local rule under test elsewhere in this paper; the read-out is a gradient reference, and no claim of locality is made for it.

The benchmark tests temporal structure preservation across depth ($L \in \{1, 2, 4\}$), width ($h \in \{128, 256, 384, 512, 768, 1024\}$), binning geometries (`adjacent-sum-5`, `channels-700`, `published-10ms`), temporal resolution at fixed window (`fixed-t100/t250/t500`), temporal shuffling controls (`bin-shuffled`, `channel-shuffled`), and — for the substitution question of §3.7 — the **spiking substrate itself**: $\{$feed-forward, recurrent$\} \times \{$fixed threshold, adaptive threshold$\}$, written `ff+fixed`, `ff+alif`, `rec+fixed`, `rec+alif`.

### 2.4 Preregistration, frozen hashes, and kill-gates

Four disciplines separate this record from an ablation sweep. Each is enforced
by tooling rather than by intention, which is the only reason any of them can be
cited as evidence.

**Frozen configuration hashes.** Every scientific run is named by a hash over its
configuration. A protocol may not be reinterpreted under its old hash; a new
hypothesis requires a new hash. `c1-118207fbc3eaba53` (Gate G2) and
`r2-afafa0fa6f43e3fc` (Gate G4) are frozen and are reopened by no downstream
result here. When a hash is found to name two experiments — as four `c1-*`
hashes were, having never mixed in the load-bearing `MATCHED_INPUT_SCALE` — it is
retired rather than quietly reinterpreted, and `scripts/test_published_hashes_resolve.py`
fails if a retired hash is cited anywhere in the repository as a live result.

**Kill-gates are permanent.** G2 FAIL is terminal: every downstream experiment
(C2, C3, R1, R2) requires an explicit opt-in flag (`--enable-c2`, and so on) to
run at all, so that post-G2 work cannot be mistaken for a rescue of G2.

**Preregistered bars and falsifiers.** Each claim carries a numeric bar fixed
before the run and a stated falsifier, registered in a PREREG document whose
analyser is committed before the first cell of that wave exists — a constraint
the git history attests rather than the prose. Where a preregistered prediction
failed we report it as failed; the registered size prediction of §3.5 is the
worked case. A bar that turns out to be wrong is reported as a wrong bar
together with the result it produced, and a replacement bar governs the *next*
wave on new cells. Re-reading the same cells against a bar chosen afterwards is
the failure this rule exists to prevent.

**A check that could not run must not report what a check that ran and passed
reports.** `scripts/record_checks.sh` runs twelve gates over the manuscript and
the archived cells, with the coverage boundary stated in §0. One audits the
others: `scripts/find_weak_checks.py` reports every assertion that would still
print `ok` if the thing under test did nothing — an ablation whose variants all
equal the baseline, a shuffle that is the identity, a loop over a possibly empty
collection. It exists because silent success is the dominant class in this
instrument's own defect register, **five of its ten defects**
([`AUDIT_2026-08-03_RUST_DEFECT_REGISTER.md`](AUDIT_2026-08-03_RUST_DEFECT_REGISTER.md);
[`DEFECT_2026-08-29_THE_EVALUATION_FORWARD_WAS_NEVER_CHECKED.md`](DEFECT_2026-08-29_THE_EVALUATION_FORWARD_WAS_NEVER_CHECKED.md)).
Its count is a standing figure that may not rise unexplained.

---

## 3. Results

### 3.1 Matched-architecture primary results

All figures below are the 2026-08-25 re-run at `MATCHED_INPUT_SCALE = 2.0`, n = 20, reported on both forward graphs. The forward is held fixed and only the update rule changes (Figure 5); the arm means and both halves of the gate are drawn by verdict rather than ranked (Figure 6) because the arms were historically split across two and the difference exceeds the registered 0.02 bar on two of them ([`RESULT_2026-08-25_MATCHED_ARCH_RERUN.md`](RESULT_2026-08-25_MATCHED_ARCH_RERUN.md)).

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

**Three claims are withdrawn from this section, all by measurement.** The discrete EventProp-style spike-adjoint FAIL (0.5000 against SuperSpike 0.9150) is withdrawn: it reads 0.9450 / 0.8900 and PASSes on a forward that can spike, and the archived 0.5000 was a spike-adjoint method with no spikes to differentiate through. The two RL broadcast contrasts, cited as evidence that continuous magnitude without spatial directionality is insufficient, are withdrawn: they read 0.8787–0.9100 and 0.7775–0.7962. **Online learned feedback alignment (v130 schedule)** remains withdrawn as a credit-assignment result, though under both repairs it now PASSes by the registered rule at 1.0000 against a ceiling of 1.0000 with zero variance across 20 seeds — which `RESULT_2026-08-23_TRACK_B_REREAD.md` registers as a saturation result and explicitly not a credit-assignment one. The `live-transfer-rescue` binary is matched-only and **does not** constitute live k-WTA transfer.

Honesty note required in the main text: on the DFA matched schedule, a broadcast-graded contrast reaches 0.9975. The lead negative therefore concerns **broadcast ±1 three-factor** credit, not the claim that every broadcast scalar fails coincidence. Locality as a necessary ingredient is evidenced on XOR (Section 3.4), not by coincidence DFA alone. **Figure M is redrawn** ([`PAPER_FIGURE_SPEC.md`](PAPER_FIGURE_SPEC.md)): it plotted richness × addressability as a graded surface, and on the re-run it is a cliff with one cell below it. Two constraints are now on the spec — encode pass/fail/at-chance rather than a ramp, since with the reference at 1.0000 a ramp manufactures an ordering the task cannot support; and draw the two low-richness broadcast rules separately, because ±1 × surrogate eligibility is at chance while ±1 broadcast REINFORCE reaches 0.78, and collapsing them would be a stronger version of the error the 0.9863 disclosure was added to prevent.

### 3.2 Engine C1 secondary results

Canonical C1 fails Gate G2 (local 0.4912, gap LCB −0.0048) (Figure 7); local-assembly and dense-local are **at chance on a two-class task**, which the figure marks with an explicit 0.50 line rather than letting bars from a zero baseline read as half the reference. Trial isolation and temporal positive-control sensitivities fail without clearing the accuracy floor. Capacity sensitivity clears the accuracy floor (0.6775) but leaves gap LCB at 0.0000 versus dense/gradient references—descriptive only, not a G2 PASS.

### 3.3 Live transfer, gap-close, and break-it

The ladder from the matched PASS down through every live arm (Figure 8) is drawn on **two** axes — the 0.65 accuracy floor and the 0.5 gap-LCB gate — because clearing the floor is not clearing the gate, and with a substrate break between rung 1 and the rest, because rung 1 is the matched dense-LIF forward and nothing below it is. Live opt-in REINFORCE feedback fails G2 (local 0.4900, gap LCB 0.0737). Epoch matching alone does not rescue random-B live RFB (local 0.4838). Structured frozen B clears the accuracy floor (0.7262) with gap LCB 0.2567 but still fails the gap bar. Stacking epochs under structured B regresses (0.5200). Structured B on a capacity substrate yields the best gap LCB in the prior suite (0.3127) while clearing the floor (0.6825), remaining short of 0.5. Eligibility×REINFORCE and restored target teach clear the floor but do not beat structured B alone. Break-it protocols close remaining differentials without remassage: live graded DFA (v20) reaches local 0.7325 with gate LCB 0.2601 and chance LCB 0.3321—floor yes, gate no. Soft-WTA × structured B (v21) regresses to chance (0.5025). Matched three-factor under 4× epochs (v22) stays at 0.5000. Finite-θ under SFB (v23) clears the floor (0.6638) with LCB 0.2370. Continuous structured B (v24) does not beat sign-truncated v15 (0.6437 / 0.1380). Spiking-path true DFA rescue fails in one honest attempt (0.6513, gap LCB 0.0733). Differential closure: [`DIFF_CLOSURE.md`](DIFF_CLOSURE.md).

### 3.4 Task evidence

On one-layer `xor_thresh`, broadcast error stays at chance (0.5008) while DFA reaches 0.8267 against gradient 0.7733—a locality flip (Figure 9). On mid-init two-layer depth locality, broadcast error also succeeds (0.8158) alongside DFA (0.8250) and REINFORCE×B (0.8033); depth help is not treated as a locality-flip claim.

### 3.5 SHD attention read-out and mechanism

Across 1,000+ cells (n=12 per contrast, extended to n=32 where noted), the time-axis attention read-out establishes five findings on SHD. Every feed-forward contrast below is 0 voided; the ten cells this campaign has voided are all `rec+fixed`, by saturation, and are reported in §3.7:

1. **Headline accuracy** (Figure 2): `ff+fixed+attn` at `d32/L4` at `e400` reaches **0.8320** with **12/12 seeds ≥ 0.80**, budget-stable (|e400−e200|=0.0002), providing a **+0.1258** gain over the rate readout `ff+fixed` (0.7062). ([`RESULT_2026-08-20_D32L4_CLEARS_THE_080_GATE.md`](RESULT_2026-08-20_D32L4_CLEARS_THE_080_GATE.md)) **Confirmed at n=32**: 0.8332 against 0.7057, gain **+0.1275**, positive in **32/32** and **32/32 at or above 0.80**. Twenty seeds beyond the registered twelve move the gain by +0.0017. ([`RESULT_2026-08-27_W15_17_THE_COLLAPSE_IS_A_THRESHOLD.md`](RESULT_2026-08-27_W15_17_THE_COLLAPSE_IS_A_THRESHOLD.md))
2. **Temporal order is the mechanism** (Figure 1): under bin-shuffling, the attention arm drops **+0.1337** (from 0.8320 to 0.6983) across **12 of 12 seeds**, while the plain arm drops only **+0.0128** (from 0.7062 to 0.6934)—a **10× factor**. The attention advantage collapses from +0.1258 to +0.0050; **96% of the readout benefit is contingent on temporal order** (94.5% at n=32, where the advantage falls +0.1275 → +0.0070). ([`RESULT_2026-08-21_W9_THE_MECHANISM_HOLDS_AT_THE_HEADLINE.md`](RESULT_2026-08-21_W9_THE_MECHANISM_HOLDS_AT_THE_HEADLINE.md)) **Confirmed at n=32**: the attention arm's shuffle cost is **+0.1347**, positive in **32/32**, against the rate arm's +0.0142 — a **9.5× factor**. Twenty further seeds move it by +0.0010. ([`RESULT_2026-08-27_W15_17_THE_COLLAPSE_IS_A_THRESHOLD.md`](RESULT_2026-08-27_W15_17_THE_COLLAPSE_IS_A_THRESHOLD.md))

   **The mechanism is not unique to the anchor.** A preregistered wave carried the same operator, the same seeds and the same pinned binary to seven further operating points — 168 cells, **zero divergences and zero voided**, every point clearing the registered floor of nine seed-paired quadruples:

   | operating point | quadruples | gain | **DiD** | positive |
   |---|---:|---:|---:|---:|
   | h256 | 12 | +0.0966 | **+0.0862** | 12/12 |
   | h384 | 12 | +0.0760 | **+0.0767** | 12/12 |
   | h512 | 12 | +0.0876 | **+0.0968** | 12/12 |
   | h768 | 12 | +0.0560 | **+0.1881** | 12/12 |
   | h1024 | 12 | −0.1318 | **+0.1122** | 10/12 |
   | h128 / `channels-700` | 12 | +0.1090 | **+0.1122** | 12/12 |
   | h128 / `published-10ms` | 12 | +0.1491 | **+0.0959** | 12/12 |

   **Coverage is complete, and the dissociation is wider than one width** (wave 22, preregistered). The twelve operating points that previously carried intact arms with no `bin-shuffled` twin were measured on a **self-contained** 504-cell wave — all four arms on one binary, so no difference-in-differences is built across two. **All twelve clear the registered +0.03 bar**, the smallest at **+0.0610** and the weakest positive count at **11 of 12**, so `scripts/mechanism_coverage.py` now reports **21 of 21**. At **h1024/`d32`/L1 the read-out makes accuracy worse — gain −0.0159 — and destroying temporal order still costs it +0.0675 in 12 of 12 seeds**; at h512 the gain is **+0.0043**, indistinguishable from nothing, against a DiD of **+0.0893**; and `channels-700` gains **+0.0243** while carrying the wave's largest DiD, **+0.1369**. The dissociation above was found at h1024 and recorded as this paper's leading open problem; it is **not an h1024 pathology**, and it recurs at widths and geometries where nothing collapses. ([`RESULT_2026-09-01_W22_THE_MECHANISM_AT_EVERY_OPERATING_POINT.md`](RESULT_2026-09-01_W22_THE_MECHANISM_AT_EVERY_OPERATING_POINT.md))

   **The operator that carries every one of those numbers is now controlled** (wave 24, preregistered). Until this point every difference-in-differences in this paper rested on a single destruction operator, and `bin-shuffled` destroys temporal order *and* displaces almost every spike. On that evidence "the read-out consumes temporal order" was not distinguishable from "the read-out is more brittle than the rate arm to having its input moved about." The control is **time reversal**, which by the instrument's own per-cell manipulation audit relocates **0.9986** of entries at a mean displacement of **145.2 bins** — against bin-shuffling's 0.9972 and 109.2, so it moves spikes *a third further* — while preserving per-channel counts, cross-channel synchrony and every inter-spike interval. Because the manipulation is applied to the training split as well, a globally reversed task is isomorphic to the intact one: the displacement is maximal and the information loss is nil. **At all three points measured the difference-in-differences under reversal is negative** — −0.0222 at h128/`d32`/L2, −0.0040 at h1024/`d32`/L1, −0.0099 at h128/`fixed-t250`/`d32`/L4 — and positive in 2, 4 and 3 of twelve seed quadruples, against the same +0.03 bar the shuffle clears everywhere. The cost is not in the displacement. It is in what the displacement destroys. **This is the one result in the paper with an independent prediction behind it**: Yu et al. (2025) report that delay-free networks are robust to a time reversal that preserves intervals and coincidences, and this instrument has no delays (§0). ([`RESULT_2026-09-02_W24_ORDER_SYNCHRONY_AND_BUDGET.md`](RESULT_2026-09-02_W24_ORDER_SYNCHRONY_AND_BUDGET.md))

   **But order is not the whole of it at every operating point.** `bin-shuffled` applies one permutation to every channel, so temporal order dies and within-bin synchrony survives; `channel-shuffled` permutes each channel independently and destroys both. Their difference is what cross-channel synchrony contributes, and the same wave registered it as a question with no directional prediction. At both `published-2ms` points the answer is **order alone** — the two DiDs differ by −0.0117 and −0.0181, inside a ±0.03 band. At **`fixed-t250` destroying synchrony as well as order doubles the cost: +0.2157 against +0.1119**, a difference of **+0.1038**. **The read-out reads cross-channel synchrony as well as temporal order**, and the paper's one-word summary of the mechanism is incomplete. Wave 24 could say that of one operating point; the next paragraph says it of the whole resolution ladder.

   **Synchrony is an axis, not a single point.** Wave 24 found the term at `fixed-t250` and nowhere else, and both remaining rungs of that ladder now answer the same way: destroying cross-channel synchrony on top of temporal order costs **+0.1308** more at `fixed-t100` (DiD +0.2631 against +0.1323) and **+0.0575** more at `fixed-t500` (+0.1736 against +0.1161), while both `published-2ms` points sit inside the ±0.03 band and slightly negative. **The read-out reads cross-channel synchrony as well as temporal order, and how much depends on the contract.** As an observation carrying no registered claim, the term falls monotonically as bins get finer — +0.1308, +0.1038, +0.0575 at 14.0, 5.6 and 2.8 ms, and absent at 2 ms — which is the direction a bin-width account predicts, because a wider bin holds more coincident spikes for `channel-shuffled` to destroy and `bin-shuffled` to preserve. Three rungs are three points, no curve is fitted, and the `published-2ms` points differ in sequence length as well as bin width so they are not a fourth rung. ([`RESULT_2026-09-03_W25_THE_MECHANISM_WHERE_IT_WAS_UNMEASURED.md`](RESULT_2026-09-03_W25_THE_MECHANISM_WHERE_IT_WAS_UNMEASURED.md))

   **What the advantage runs through** (waves 26–28, each preregistered). Everything above destroys order in the *data*; three further manipulations ask what in the read-out consumes it, all at one operating point — h128, feed-forward, `published-2ms`, `d32/L4`, e400. **Position** carries part of it but not all: deleting only the read-out's positional encoding drops the gain to **+0.0506** from **+0.1258**, a cost of **0.0752** in 12 of 12, yet destroying order in the data leaves just **+0.0050**, so something besides the positional code carries order here and this paper does not say what. **The timescale is long**: permuting bins inside a sliding window reaches half the full shuffle's effect at **130.33 bins — 261 ms**, about 36% of the 716 ms mean utterance. **It is not spike counts**: under random deletion the advantage does not fall but **grows**, to **+0.1640** at p90, the substrate losing 0.1080 where the read-out loses **0.0698**. That clause was registered two-sided for a one-sided question and **is recorded NOT MET and reported as NOT MET**. What stands is the comparison — destroying order costs the read-out **0.1208**, destroying counts costs it nothing. None of this decomposition transfers to h1024 or to a recurrent substrate on this evidence: the difference-in-differences travels to twenty-one points, its decomposition does not. **Appendix F** carries the three manipulations in full.

   **The budget limit is retired.** Wave 24 measured the contrast at `e100` at two of the twenty-one — **+0.1183** at h128/`d32`/L2 and **+0.0633** at h1024/`d32`/L1, each positive in **12 of 12** seed quadruples and each within 0.005 of its own `e400` value, with reversal still flat at both (−0.0068, −0.0307). Wave 25 measured the other nineteen. **All nineteen clear the +0.03 bar**, seventeen at 12/12 seed quadruples and the weakest at 11/12, ranging from **+0.0469** (h1024 `d32l2`) to **+0.1381** (h128 `d64l4`). **Every one of the twenty-one operating points now carries the difference-in-differences at two budgets, and the mechanism is not a property of the anchor budget.** Two budgets are still two budgets; nothing here maps the contrast as a function of training length.

   **The depth question that wave 22 could not evaluate is now answered, and it is answered NOT MET.** H22-3 was **NOT EVALUABLE** because its analyser needs a `d32l4` twin drawn from the wave's own cells and the plan contained none — the self-containment that makes every other verdict sound is what starved it, and neither the analyser nor the corpus was touched afterwards ([`DEFECT_2026-08-31_H22_3_CANNOT_BE_EVALUATED.md`](DEFECT_2026-08-31_H22_3_CANNOT_BE_EVALUATED.md)). Wave 25 ran the missing twins as a registered wave of its own rather than as a repair, and against them the contrast is largely indifferent to read-out depth: h128 (`d32l2` **+0.0063**, `d64l4` **+0.0123**), h256 (**+0.0104**) and h512 (**+0.0075**) all sit inside 0.013 of their twin, against a registered bar of 0.10. **h768 `d32l2` is +0.1271 from its twin and outside it**, so the question is answered **NOT MET** and **read-out depth is a scope axis for the mechanism claim, at one width of five.** h768 is also where the campaign records its largest difference-in-differences (+0.1881) and its smallest positive gain (+0.0560); this paper does not explain either, and one point outside a bar is a scope limit to disclose rather than a depth effect to characterise.

   **The `gain` and `DiD` columns are over the same seeds everywhere except h1024**, where the DiD is over the twelve quadruples and the gain over the twenty intact pairs waves 18–19 extended that width to; over the twelve quadruple seeds the h1024 gain is **−0.1618**. The rank is identical either way.

   **The size of the effect is not the gain.** Spearman ρ between the six per-width gains and their DiDs is **−0.1430** against a preregistered bar of **+0.829**, the n = 6 one-tailed critical value — not a weak positive, absent and faintly negative. h768 buys the least on the ladder and carries the largest DiD in the wave. The registered reading is that the difference-in-differences is a property of the read-out and **not a quantitative account of what the gain is made of**. ([`RESULT_2026-08-29_W21_THE_MECHANISM_TRAVELS_BUT_ITS_SIZE_DOES_NOT.md`](RESULT_2026-08-29_W21_THE_MECHANISM_TRAVELS_BUT_ITS_SIZE_DOES_NOT.md))
3. **Sample efficiency:** Attention reaches 98.1% of e400 accuracy by 10 epochs (0.7337), bracketing convergence at `(5, 10]` epochs. **The denominator is the `d32/L1` arm at convergence (0.7483), not the `d32/L4` headline** — against the headline's 0.8320 the same cell is 88.2%. The two operating points differ: the L1 ladder's e400 gain is +0.0421, not +0.1258. ([`RESULT_2026-08-20_W7_CONVERGENCE_IS_BRACKETED.md`](RESULT_2026-08-20_W7_CONVERGENCE_IS_BRACKETED.md); refines [`RESULT_2026-08-20_W6_ATTENTION_IS_SAMPLE_EFFICIENCY.md`](RESULT_2026-08-20_W6_ATTENTION_IS_SAMPLE_EFFICIENCY.md))
4. **Scope limits:** the gain inverts at width h1024 — +0.1258 at h128 falling to +0.0560 at h768 and then to **−0.1618** — and that inversion is a **threshold rather than a continuing slope**, sitting between h768 and h1024 at **6.9×** the largest gap below it. It is a property of read-out *depth* at that width rather than of the width: `d32/L2` and `d32/L3` both gain there. Three preregistered rescue levers are negative and no mechanism is offered. The difference-in-differences nonetheless stays **positive** at h1024 (**+0.1122**, 10 of 12 seeds) — the read-out consumes temporal order while performing worse than no read-out at all, which §4.8 records as this paper's leading open problem. **Appendix E** carries the ladder, the budget dependence, the rescue attempts and what each does not settle. Gain is positive across geometries (+0.1090 on `channels-700`, +0.1491 on `published-10ms`), but 0.80 clearance is geometry-specific (0.7864 on `channels-700`).
5. **Temporal resolution is an axis, and the gain falls as bins get finer.** The `published-Nms` test of this (S-5) was **refuted and is withdrawn**: that family moves bin width and sequence length together, so a single number cannot be attributed to either. Re-asked on `fixed-tN`, which holds a 1400 ms window fixed and varies only the frame count, the read-out helps at **every** rung and the gain is monotone in resolution — **+0.1927** at 14.0 ms bins, **+0.1751** at 5.6 ms, **+0.1474** at 2.8 ms, each 12/12 seeds positive and 12/12 over the 0.80 gate. gain(t500) − gain(t100) = **−0.0453** against a two-sided bar of 0.03, so the advantage **shrinks** as bins get finer while never reversing. **Appendix I** carries the ladder.

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
   budget, so this arithmetic is no longer hypothetical — it is what the
   denominator now is, and the ordering it produces is in §3.1.) Values above 1
   are an artefact of the
   denominator; clamping them, as `runner.rs` does, hides the cause rather than
   fixing it.
2. **Raising *both* budgets does not restore the ordering** — the arm stays at or
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

The +0.1258 of §3.5 has two readings the campaign could not separate, because every one of its 720 cells sat on a single substrate, `ff+fixed`: the read-out **adds** temporal structure no substrate of this kind represents, or it **substitutes** for the threshold adaptation and recurrence that `ff+fixed` happens not to have. ETLP's conclusion — that adaptation and a recurrent topology are what a spiking network needs for rich temporal structure — makes the second reading the live one. Three waves settle it (Figure S), and the limits below travel on the figure rather than in its caption alone.

**Adaptation makes no difference to the gain, or to anything else.** At the anchor (h128, `published-2ms`, `adjacent-sum-5`, e400, d32/L4, n=12), attention's gain is **+0.1258** on `ff+fixed` and **+0.1285** on `ff+alif`. The difference is **+0.0027** against a two-sided bar of 0.03, and is positive in **6 of 12** seeds — a coin flip. Adaptation alone does not help either: `ff+alif` reaches **0.7018** against `ff+fixed`'s 0.7062, better in **3 of 12** seeds, with **0 of 12** over the 0.80 gate. At this operating point threshold adaptation is inert. ([`RESULT_2026-08-23_W12_ATTENTION_DOES_NOT_SUBSTITUTE_FOR_ADAPTATION.md`](RESULT_2026-08-23_W12_ATTENTION_DOES_NOT_SUBSTITUTE_FOR_ADAPTATION.md))

**The recurrent substrate is measurable only at one operating point, and finding it was its own wave.** `rec+alif` completes **11 of 12** at the anchor budget only at surrogate scale 0.4; at the registered default of 1.0 it completes 8 of 12. `rec+fixed` completes 12 of 24 across both scales and fails by a different mechanism — **saturation**, ten cells voided with up to 52% of hidden units pinned at maximum firing, none by divergence at scale 0.4. Adaptation is what prevents that, so on the recurrent substrate adaptation is *stabilising*, which is the opposite of the hypothesis that wave's own name asserted. ([`RESULT_2026-08-23_W13_RECURRENT_STABILITY.md`](RESULT_2026-08-23_W13_RECURRENT_STABILITY.md))

**On the recurrent substrate the gain roughly doubles.** With every arm run at scale 0.4 so substrate and scale cannot be confounded, and paired over the seeds where both arms completed:

| substrate | pairs | rate read-out | + attention d32/L4 | gain |
|---|---:|---:|---:|---:|
| `rec+alif` | 10 | 0.5262 | 0.7874 | **+0.2612** |
| `ff+fixed` | 12 | 0.7088 | 0.8289 | **+0.1201** |

The difference is **+0.1411** against a bar of 0.03, positive in **10 of 10** recurrent pairs. The scale is not doing the work: `ff+fixed` at 0.4 scores **0.7088** against **0.7062** archived at 1.0, a difference of +0.0026. ([`RESULT_2026-08-23_W14_ATTENTION_AND_RECURRENCE_ARE_COMPLEMENTARY.md`](RESULT_2026-08-23_W14_ATTENTION_AND_RECURRENCE_ARE_COMPLEMENTARY.md))

**So substitution is refuted on both axes**, and the read-out's advantage is indifferent to adaptation and *larger* where the substrate is recurrent.

**This paragraph used to read that finding through §3.5's shuffle result and conclude that the claim is "about what the read-out consumes". That reading is withdrawn.** Every manipulated cell in this corpus is feed-forward, and the wave registered to fix that — 72 recurrent cells under `intact`, `bin-shuffled` and `reversed` — **could not be evaluated**: ten failed on the instrument's non-finite-training guard, five of them the rate arm under time reversal, leaving the reversal contrast at three seed-paired quadruples against a floor of nine. **Whether the recurrent read-out's advantage is order-dependent is unmeasured**, and this section no longer asserts it. What the attempt did establish is that `rec+alif` at the registered surrogate scale is numerically unstable under time reversal, which is a fact about the substrate and not about the read-out ([`RESULT_2026-09-03_W25_THE_MECHANISM_WHERE_IT_WAS_UNMEASURED.md`](RESULT_2026-09-03_W25_THE_MECHANISM_WHERE_IT_WAS_UNMEASURED.md)).

Four limits are load-bearing and are stated here rather than in a footnote.

1. **The recurrent gain is measured from a lower base.** `rec+alif` starts 0.18 below `ff+fixed` and has 0.4738 of headroom against 0.2912. Normalising by headroom — post-hoc, not registered — the ratio falls from 2.2× to **1.34×**. The ordering survives; most of its apparent size does not.
2. **The recurrent substrate does not win.** `rec+alif+attn` reaches 0.7874 against `ff+fixed+attn`'s 0.8289 at the same scale. Attention closes most of the gap the substrate gives away, and not all of it. No verdict is issued on that ordering.
3. **The recurrent arms are numerically extreme**, with peak gradient norms to 4.9e32 against 1.13e8 for the largest cell anywhere else in the campaign, and the comparison rests on **ten pairs, the registered minimum**. The two arms lost different seeds; one further loss on either would have made the comparison unreportable.
4. **Survivorship is reduced, not removed.** Pairing on seed compares the same trajectories rather than two differently filtered subsets, but the surviving recurrent pairs are those that did not diverge, and divergence is not random. The feed-forward comparison carries no such exposure at 12/12.


### 3.8 Where 0.8320 sits against the literature, and why the gap is a class difference

A read-out result on a standard benchmark invites one question immediately, and the paper answers it here rather than leaving it to a reader: published SHD accuracies for surrogate-gradient spiking networks sit around 0.90–0.95, and the pinned third-party reference used for calibration in this work reaches **0.9390 / 0.9368 / 0.9371** on the clean protocol. The instrument's converged attention arm reaches **0.8320**. The difference is about **0.087** against the best delay-free variant of that reference, and it is not a tuning gap.

Four preregistered ablations turned every knob on the reference that could plausibly account for it — depth, dropout, read-out style, normalisation. **Together they do not explain it, and two of them subtract**: removing the reference's second hidden layer *gains* 0.0145 and removing its batchnorm appears to gain 0.0058. Of everything measured, only dropout is a positive contributor the instrument lacks, and it is worth 0.0128 of the 0.087.

Reading the two forward passes term by term found what the configuration surface could not. The reference carries a **`Dcls1d` temporal kernel of 25 taps per synapse on every layer**, spanning 250 ms, and its membrane decays to a third in a single timestep — its temporal integration lives in the convolution and its neuron is close to a pointwise nonlinearity. The instrument is the mirror image: **no temporal kernel in any form**, and a membrane retaining 0.82 per step to compensate. These are not one architecture at two operating points ([`FINDING_2026-08-24_THE_FORWARD_PASSES_DIFFER_IN_KIND.md`](FINDING_2026-08-24_THE_FORWARD_PASSES_DIFFER_IN_KIND.md)).

That reframes this paper's own result rather than excusing it. The attention read-out is a mechanism over the time axis added to a substrate that has none, and §3.5's shuffle control shows 94.5% of its benefit is contingent on temporal order at n=32. **The most likely reading of the +0.1258 is that the read-out is recovering some of what a learned temporal kernel supplies** — a claim about kind, and, given the ablation below, not a claim that the two quantities are commensurable — which is a claim about what the read-out consumes, and is consistent with §3.7's finding that its advantage is larger, not smaller, where the substrate is recurrent.

**The kernel has now been ablated, and the arithmetic that motivated this section does not survive it** (preregistered, n = 3 per arm). Setting `max_delay = 1` makes all three `Dcls1d` constructions pointwise — a one-tap dilated convolution — and lets the five values mechanically derived from it follow, changing nothing else. The reference falls from **0.9387** to **0.6276**: the kernel is worth **K = +0.3111**.

That is **3.6× the 0.087 it was invoked to explain**, and the direction is what matters. Removing the kernel does not bring the reference *down to* the instrument's 0.8320; it drops it **0.2044 below** it. So the two systems are not one architecture plus a temporal kernel. They differ by at least two large terms of opposite sign — a kernel worth +0.31 to the reference, and something else worth roughly −0.20 to it relative to the instrument — and **the observed 0.087 is the net of offsetting differences, not the size of any one of them.**

**The compensating term is the membrane, and it is 81.5% of it** (preregistered, n = 3 per arm). The reference's neuron retains **0.0050** of its membrane per step — measured by construction, not inferred — so with the kernel removed it has no temporal integration at all, while the instrument retains 0.82. Setting `init_tau` to 55.5556 ms, which reproduces 0.8200 exactly and is the only value the series introduces, lifts the kernel-free reference from 0.6276 to **0.7942**: **M = +0.1666** of the 0.2044 that separated it from the instrument. The decomposition now closes — the reference's lead of **0.1067** over the instrument is **+0.3111 − 0.1666 − 0.0378** — and what is unexplained has fallen from 0.2044 to **0.0378**, below the 0.05 this programme calls material. The gate for that reading is that Arm B′ reproduced the kernel ablation's Arm B **to every recorded digit on a different instance**, so the two terms were measured against each other and not across a platform. The remaining 0.0378 is not identified: the synaptic filter, the optimiser, the schedule and the read-out are all untouched. ([`RESULT_2026-09-02_THE_MEMBRANE_ABLATION.md`](RESULT_2026-09-02_THE_MEMBRANE_ABLATION.md))

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

The live transfer package is a scoped negative. Structured feedback is the strongest accuracy lever among tested gap-close arms, and capacity×structured yields the best prior gap LCB, but none—including live DFA, soft-WTA, mute-off, and continuous-B probes—clear Gate G2 (v13–v24). Matched undertraining does not explain the broadcast ±1 three-factor FAIL (v22 remains at chance). Reporting floor clearance without gap clearance would overclaim. Engine C1 remains a valid operationalized pipeline negative only when integrity caveats are explicit (Appendix A). Do **not** cite v131 / `live-transfer-rescue` as live-engine rescue: that binary is matched-only.

Soft-WTA × structured B on live C1 uses disclosed temperature `T=1` (v21). That probe is motivated by a hybrid soft→hard collapse whose transfer-collapse temperature is **T=2.0** on a separate hybrid ladder (`binn-hybrid-winner-temp-v1-fa7710de68ad7bfe`). Hybrid T=2.0 is **not** live v21; do not equate the appendix hybrid mechanism note with the live soft-WTA protocol.

### 4.3 Baselines and EventProp H2H

The primary gradient ceiling on the matched gate is **SuperSpike BPTT** on the fixed dense-LIF forward. True σ′ e-prop (`c1x-eprop-true-*`, true-surrogate mean 0.7125) is a methods footnote only—not a H2H claim that e-prop rescues broadcast ±1 insufficiency. **The discrete EventProp-style spike-adjoint FAIL is withdrawn.** It was reported as mean 0.5000 against SuperSpike 0.9150, gap LCB 0.0000; on the repaired forward it reaches **0.9450** (feed-forward) and **0.8900** (recurrent) and **PASSes** on both. The archived number was a spike-adjoint method on a forward that emitted no spikes, and "discrete ≠ continuous Wunderlich–Pehle" was an explanation offered for a result with a different cause. This remains a **discrete** hard spike-gate adjoint and still is not continuous Wunderlich–Pehle (2021) hybrid EventProp; no comparison to that method is claimed in either direction. Hybrid exact-forward arms labeled “e-prop/DFA” remain eligibility × transported modulators unless the true-σ′ family is cited explicitly.

### 4.4 Efficiency honesty (F1 / F2 / F5)

These limitations constrain how far efficiency rhetoric should travel with the negative result:

- **F1 (scan barrier).** Sub-threshold membrane dynamics admit chunked associative scans; the hard spike reset remains a sequential, data-dependent barrier. Training-time parallelism is therefore *partial*, not a clean escape from recurrent-time cost.
- **F2 (forward remains sequential).** Online local learning removes the backward-unroll / activation-storage half of BPTT cost; it does **not** remove sequential forward simulation within a stream. Parallelism is across neurons, areas, and independent streams—not across time within one stream.
- **F5 (activity ≠ compute).** Sparse activity ratios do not translate linearly into work. Honest software efficiency is work-per-accuracy including per-event queue and pointer-chase overhead; large multipliers are a neuromorphic-hardware claim, not a CPU/GPU default.

### 4.5 Appendix-only: G3 / G4 / H0 (do not reopen G2)

Continual forgetting (C2 / Gate G3 FAIL), multi-area scaling (R2 / Gate G4 **NO-GO** degrade curve), and hybrid H0 (**HYBRID_NO_GO**) live in [`APPENDIX_POST_G2.md`](APPENDIX_POST_G2.md). They are post-G2 / exploratory. The camera-ready banner does **not** reopen Gate G2 or remassage `c1-118207fbc3eaba53`. G4 NO-GO redirects away from scaling more areas under the same ±1 three-factor substrate; any future Micro / isolate capacity stress is engineering headroom after G2 FAIL, not a Foundation unlock and not part of this MUST package.

### 4.6 Neuromorphic benchmark scope and non-claims

The SHD attention readout results are anchored at **h128 / `published-2ms` /
`adjacent-sum-5`**, and the **mechanism** result is no longer scoped to it: the
destruction control exists at **21 of 21 operating points**, covering widths 128
through 1024, both contracts and both geometries, and
`scripts/mechanism_coverage.py` recomputes that on every gate run. Wave 22
measured the twelve points that previously carried intact arms with no
`bin-shuffled` twin, and every one of them clears the registered bar
([`RESULT_2026-09-01_W22_THE_MECHANISM_AT_EVERY_OPERATING_POINT.md`](RESULT_2026-09-01_W22_THE_MECHANISM_AT_EVERY_OPERATING_POINT.md)). **The budget limit that stood beside that coverage is
discharged:** wave 24 measured the contrast at `e100` at two of the twenty-one
and wave 25 at the other nineteen, and every one of the twenty-one clears the
registered bar at both budgets. Two budgets are not a curve, and nothing
maps the contrast as a function of training length. Wave 24 also supplies what
the mechanism claim had never had — a destruction control that is
displacement-matched and information-preserving — and finds a second term that
wave 25 then measured across the resolution ladder: destroying cross-channel
synchrony as well as order costs +0.1308, +0.1038 and +0.0575 more than
destroying order alone at `fixed-t100`, `fixed-t250` and `fixed-t500`, and
nothing at either `published-2ms` point, so **"temporal order" is an incomplete
description of the mechanism** and how incomplete depends on the contract
([`RESULT_2026-09-02_W24_ORDER_SYNCHRONY_AND_BUDGET.md`](RESULT_2026-09-02_W24_ORDER_SYNCHRONY_AND_BUDGET.md);
[`RESULT_2026-09-03_W25_THE_MECHANISM_WHERE_IT_WAS_UNMEASURED.md`](RESULT_2026-09-03_W25_THE_MECHANISM_WHERE_IT_WAS_UNMEASURED.md)).
**Two limits replace it.** Read-out depth is a scope axis: four of five widths
put the contrast within 0.013 of their `d32l4` twin and h768 is 0.1271 away,
outside the registered 0.10 bar. And every manipulated cell in the corpus is
feed-forward — the registered attempt to measure the recurrent substrate under
shuffling and reversal returned **NOT EVALUABLE**, ten of its cells lost to the
instrument's non-finite-training guard.

**We do not claim calibration**, and what blocks it is criterion 5 — a Python
mirror of the attention axis, which does not exist and is not attempted here
(§4.8). The two criteria that previously read `false` no longer do, and they
were false for a provenance reason rather than an accuracy one; nor should **the
0.80 `CELL_PASS` floor be read as a standard the instrument is failing to
meet**. Both are matters of the third-party reference artifact rather than of
the read-out, and **Appendix G** carries them.

We do not claim cortical realism, Assembly Calculus PASS, neuromorphic deployment, or impossibility of local learning in principle.

Seven suites are explicitly **withdrawn**. §4.7 states the count and what it
cost; Appendix C is the ledger, entry by entry.

### 4.7 The withdrawal ledger

**Seven results have been withdrawn from this package, three of them PASSes.**
We report the count in the main text because a record that shows only surviving
results tells a reader nothing about how hard the instrument was tried, and
because the withdrawals are what license the survivors. The episode-by-episode
ledger, with the diagnosis for each, is
Appendix C.

Two episodes produced them. The **2026-08-19→22 record repair** withdrew four,
including this project's most spectacular number — an online learned-feedback
arm at **1.0000**, which a ceiling-health repair re-reads as saturation rather
than credit assignment. The **2026-08-25 matched re-run** withdrew three more on
the finding that every previously published matched-architecture number had been
produced on a forward pass emitting **zero spikes at any seed**. **The lead
negative survived both episodes on both forward graphs, which is the only reason
it is reported at all.**

The generalisable point is not that defects occurred. It is that **every one of
these defects produced plausible numbers, and none was caught by an arm's own
accuracy.** All were caught by instrument-health checks that ask a different
question — could this run have produced this number for the wrong reason? The
zero-spike forward is the cautionary case: it ran for weeks, produced
publishable-looking results across an entire suite, and was invisible to every
experiment's own verdict. A harness that scores whatever it is given will
eventually score an artefact.

### 4.8 Open problems

We state these as open rather than as future work, because each is a thing this
paper's own evidence cannot currently settle.

**The gain/DiD dissociation** is the leading one. The order-dependence is present at all twenty-one operating points, but its *size* is uncorrelated with the size of the gain it is supposed to explain (Spearman $\rho$ = −0.1430 over the six widths), and two waves of narrowing widened the puzzle rather than closing it. Nothing here explains why a read-out whose contribution depends everywhere on temporal order should contribute an amount unrelated to how much order it consumes. **The h1024 threshold** is located between h768 and h1024, unexplained, with three rescue levers negative and order-dependence persisting through the collapse (Appendix E). **Calibration** is unmet: criterion 5, a Python mirror of the attention axis, does not exist and is not attempted here, so no number in this paper is comparable to an externally recorded one. **The kernel attribution** — the 0.087 calibration residual assigned to a 25-tap learned temporal kernel — is by elimination, not by an ablation that added one (§3.8). And **whether any local rule crosses the k-WTA transfer barrier** is open, given that a structured feedback matrix moves accuracy but not the gap (§4.2).

---

## 5. Reproducibility

Scientific hashes and commands are listed in [`REPRO_ARTIFACT_CHECKLIST.md`](REPRO_ARTIFACT_CHECKLIST.md) and [`PAPER_RESULTS_TABLE.md`](PAPER_RESULTS_TABLE.md). Camera-ready citations must point at on-disk notes or exact `--config-hash` replays.

The full record rebuilds and re-checks itself from a clean checkout with `cargo test --locked --workspace`, `./scripts/gc_checks.sh` (GC1-GC7) and `./scripts/record_checks.sh` (twelve gates, 125 SHD-campaign assertions); **Appendix H** lists the commands, including the canonical Gate G2 replay.

Post-G2 gates require explicit opt-in (`--enable-c2`, `--enable-c3`,
`--enable-r1`, `--enable-r2`); see §2.4. The current matched figures come from
the 2026-08-25 re-run recorded in
[`RESULT_2026-08-25_MATCHED_ARCH_RERUN.md`](RESULT_2026-08-25_MATCHED_ARCH_RERUN.md),
not from the four hashes retired with it.

Attention-campaign cells are preserved in four corpora, all four of which the
number sweep reads: `results/shd_attention_campaign_v1/cells` (waves 1–7 plus
`r1cal`), `results/shd_attention_campaign_v2` (waves 8–25), the Azure d32/L4
scope cells under `results/azure-d32l4-scope-v1/results`, and
`results/shd_attention_campaign_v3` (waves 26–28). A corpus that exists but is
not in that list is a corpus whose cells derive nothing, which is a failure this
paper has had once and now tests against.

---

## Appendix A — Integrity limitations (protocol v2)

Cross-trial STDP pairing times (`ThreeFactor.last_spike`) are retained on canonical C1 while eligibility traces are cleared. Membrane and dendritic reset is incomplete relative to the C3 production path. Hidden thresholds are set to infinity during the integrate window, suppressing natural spiking. Assembly Calculus `project` is not exercised on the canonical loop (it is wired under `c1-project*` and fails G2 there). Exact-forward arms labeled “e-prop/DFA” are hybrid eligibility × transported modulators unless the true σ′ e-prop family is cited explicitly.

## Appendix B — Post-G2 harvest (banner)

G3 / G4 / hybrid H0 numbers and hashes: [`APPENDIX_POST_G2.md`](APPENDIX_POST_G2.md). Mechanism figure cells: [`PAPER_FIGURE_SPEC.md`](PAPER_FIGURE_SPEC.md) Figure M. These rows do not reopen G2.

## Appendix C — The withdrawal ledger in full

Seven results have been withdrawn from this package. §4.7 states the count and
the lesson; this appendix is the ledger itself, because a count without
diagnoses is not auditable.

**Episode 1 — the 2026-08-19→22 record repair.** Four results withdrawn, three
of them PASSes:

- **`track-b-rescue` v130 online learned feedback alignment, PASS at 1.0000 (gap
  LCB 0.9988) — withdrawn.** At v131 the arm reports `INVALID_HARNESS`: a
  ceiling-inverted warning fires on 3 of 20 learned-feedback seeds and the code
  **refuses to emit a PASS while it is present**. Re-read under both repairs it
  is 1.0000 against a ceiling of 1.0000 with zero variance — a *saturation*
  result, not a credit-assignment one.
- **The depth-collapse / deep-SNN scaling result — withdrawn** (v134
  `INVALID_HARNESS`: every depth-matched gradient ceiling was at chance).
- **`shd-scientific-sweep` — withdrawn.** It ran on synthetic 24-channel /
  16-timestep data and **never loaded SHD**
  ([`DEFECT_2026-08-20_SHD_SWEEP_IS_SYNTHETIC.md`](DEFECT_2026-08-20_SHD_SWEEP_IS_SYNTHETIC.md)).
- **The `live-transfer-rescue` arms — `INVALID_HARNESS`**, and the protocol was
  misnamed: it is matched-only, not live-engine (§2.2).

Separately, three gradient *references* were found at or near chance on tasks
their own treatments solve; two are diagnosed (`MatchedDeepGradient` collapses to
silence, `ShdEpropCeiling` is a constant predictor by a different mechanism).
None is used in any claim here.

**Episode 2 — the 2026-08-25 matched re-run.** Every previously published
matched-architecture number was produced on a forward pass that emitted **zero
spikes at any seed**. Three more results were withdrawn:

- **The discrete EventProp-style spike-adjoint FAIL** (0.5000 → 0.9450 / 0.8900
  PASS). The failure mode is instructive: *a method whose entire mechanism is the
  spike had no spikes to differentiate through*, while every other arm could
  still separate classes by sub-threshold membrane rate. The defect was
  maximally misleading precisely on the arm that most depended on the broken
  quantity — and the prior explanation offered for that number ("discrete hard
  spike-gate adjoint ≠ continuous Wunderlich–Pehle") was an explanation for an
  artefact, and is retired with it.
- **Both RL broadcast contrasts** (0.5250 → 0.9100; 0.5113 → 0.7962). The
  reading they supported — "continuous magnitude without spatial directionality
  is insufficient on this gate" — no longer has evidence behind it.

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
principle** — these are scoped, operationalised negatives about one substrate
under one set of rules. No reopening of frozen hashes by threshold massage. No
widening of the lead FAIL to "any broadcast". Undertraining is not the cause: the
v22 arm at four times the epochs is still at chance. **No ranking among the
passing matched arms** — the gate saturates, so it can name one failure and
cannot order the survivors. No live-engine rescue from a matched PASS. No claim
that the discrete spike-adjoint is a negative result, and none that it is
equivalent to continuous EventProp. No digital-brain or brain-equivalence claim.

On the SHD programme, four further non-claims that §0 does not cover. The
headline fraction is of the **gain**, not of accuracy. The 0.087 calibration
residual is not a tuning gap. No temporal-*resolution* mechanism is claimed —
that hypothesis was withdrawn, and the `fixed-tN` ladder moves the opposite way.
And no recurrent-substrate win is claimed anywhere (§3.7).

## Appendix E — The width collapse at h1024

This is finding 4 of §3.5 in full. The main text states the shape of the collapse and the open problem it leaves; the arithmetic is here.

**Scope limits** (Figure 3): gain inverts at width h1024, and on a six-rung ladder that inversion is a **threshold rather than a continuing slope**: +0.1258 (h128), +0.0966 (h256), +0.0760 (h384), +0.0876 (h512), +0.0560 (h768), −0.1618 (h1024). The drop into h1024 is **0.2178**, **6.9×** the largest gap below it (0.0316) and more than twice the registered 3× bar, so the collapse sits between **h768 and h1024** and the rungs below it remain positive. **Every rung of this ladder is measured at `d32/L4`, and the inversion is a property of that read-out depth rather than of the width.** At h1024 and the same budget, `d32/L2` gains **+0.0405** in **20/20** seeds and `d32/L3` gains **+0.0371** in 18/20, against L4's −0.1318 in 3/20; the optimum in depth at that width is interior, and it is not established which of L2 or L3 holds it — they differ by 0.0034. So this row bounds *deep* read-outs at h1024 and does not bound h1024. What makes an arm collapse is still unexplained: L3 sits above the registered gradient-norm sickness threshold at 1.347 and gains anyway, which is why the numerical account was refused ([`RESULT_2026-08-28_W18_19_THE_DEPTH_OPTIMUM_IS_INTERIOR.md`](RESULT_2026-08-28_W18_19_THE_DEPTH_OPTIMUM_IS_INTERIOR.md)). The decay above the collapse is **not strictly ordered** — h384 and h512 are not distinguishable at twelve seeds (paired difference −0.0116, sd 0.0253, negative in 7 of 12) — so no monotonicity is claimed. Gain is positive across geometries (+0.1090 on `channels-700`, +0.1491 on `published-10ms`), but 0.80 clearance is geometry-specific (0.7864 on `channels-700`). ([`RESULT_2026-08-21_W8_HEADLINE_SCOPE_IS_MEASURED.md`](RESULT_2026-08-21_W8_HEADLINE_SCOPE_IS_MEASURED.md); [`RESULT_2026-08-27_W15_17_THE_COLLAPSE_IS_A_THRESHOLD.md`](RESULT_2026-08-27_W15_17_THE_COLLAPSE_IS_A_THRESHOLD.md))

**The collapse is late, and stopping early avoids it** (wave 23, preregistered). No cell at `h1024/d32/L4` had ever run at any budget but e400. At **e100** the same arm gains **+0.0827** over its rate control in **12 of 12** seeds — against **−0.1318** in 3 of 20 at e400, an improvement of **+0.2145** against a registered bar of +0.10 — and reaches **0.8153** where e400 gives 0.5768. At **e200** the gain is +0.0564 in 11 of 12, so the degradation is progressive rather than a cliff. **The control is what makes this interpretable**: `d32/L2`, which does not collapse, moves only **+0.0149** over the same budget change against a ±0.03 bar, so the effect is specific to the collapsing arm and this is *not* the weaker statement that e400 is past the optimum for every deep read-out here. The arm also **keeps its fit**: 0 of 12 e100 cells end above 3× their own best training loss, where 63 of 68 do at e400. The rate arm is budget-insensitive throughout (0.7326 / 0.7390 / 0.7386), so the movement is in the attention arm and not its baseline. ([`RESULT_2026-08-30_W23_THE_COLLAPSE_IS_LATE.md`](RESULT_2026-08-30_W23_THE_COLLAPSE_IS_LATE.md))

**That excludes overfitting**, which this section previously recorded as "neither excluded nor supported": at e400, 63 of 68 intact `d32/L4` cells reach a training-loss minimum around epoch 39–99 and end **56×** above it, while `d32/L1` (0/20), `d32/L2` (4/32) and the rate arm (0/32) hold theirs — and an arm that overfits keeps a low training loss ([`FINDING_2026-08-29_THE_H1024_COLLAPSE_IS_A_LOST_FIT.md`](FINDING_2026-08-29_THE_H1024_COLLAPSE_IS_A_LOST_FIT.md)).

**Three things this does not settle.** Not *why* the fit is lost — H15-1's refusal of the gradient-norm account stands, so the collapse is **located, bounded in time, and still unexplained**. Not h1024 in general: one width, one read-out depth, two budgets. And not the headline, which stays `h128` at e400; what changes is that the anchor budget is now known to be past the optimum at this width and depth. One caveat travels with it: wave 21's h1024 difference-in-differences of +0.1122 was measured at e400 between two arms **both** in late collapse, and nothing says what it would be at e100.

**What the h1024 collapse is not.** Three preregistered levers — surrogate scale 0.5 and 0.25, and gradient clipping at 1000.0 — were run at h1024/d32/L4 to test whether the inversion is an optimisation failure. All three are **negative and worse than the arm they were meant to rescue** (−0.2106, −0.2565, −0.0904). Clipping moves the median epoch-mean gradient norm from 55.494 to 11.660, a real effect in the intended direction, and accuracy does not follow. **No mechanism for the collapse is offered here**; what wave 23 adds above is a bound on *when* it happens, not an account of *why*.

**And it is not the temporal-order account in disguise.** If the inversion and the mechanism were the same phenomenon, then where the read-out buys nothing there should be no order-dependent benefit left to destroy. That prediction was registered before the cells existed and it **failed**: at h1024 the difference-in-differences is **+0.1122** in **10 of 12** seeds against a registered ceiling of +0.02, while the gain over those same twelve seeds is **−0.1618**. The read-out consumes temporal order while performing worse than no read-out at all. Nothing in this paper's account permits that, and per the preregistration it is the paper's **leading open problem** rather than a caveat. It also leaves the overfitting alternative exactly where it was: that argument was conditional on the shuffle cost collapsing, and it did not. ([`RESULT_2026-08-29_W21_THE_MECHANISM_TRAVELS_BUT_ITS_SIZE_DOES_NOT.md`](RESULT_2026-08-29_W21_THE_MECHANISM_TRAVELS_BUT_ITS_SIZE_DOES_NOT.md))

## Appendix F — What the advantage runs through

This is the waves 26–28 block of §3.5 finding 2 in full: the three manipulations that ask what in the read-out consumes temporal order, rather than how much order the data carries. All are measured at a single operating point, and §3.5 states that limit where the result is used.

**What the advantage runs through** (waves 26–28, each preregistered). Everything above destroys order in the *data*; three further manipulations ask what in the read-out consumes it. **Position:** a `no-position` arm keeps every parameter and every spike and deletes only the read-out's positional encoding, and its gain falls to **+0.0506** from **+0.1258** — a cost of **0.0752**, above the +0.03 bar in **12 of 12**. Registered beside it as a question and answered *no*: deleting the read-out's access leaves +0.0506 where destroying order in the data leaves **+0.0050**, a difference of **0.0456** outside a ±0.03 band, so something besides the positional code carries order here and this paper does not say what. **A timescale:** permuting bins inside a sliding window destroys order below it and preserves it above, and across windows of 2 to 256 bins the difference-in-differences climbs **+0.0066 to +0.1260** against the full shuffle's +0.1208, reaching half its effect at **130.33 bins — 261 ms**, about 36% of the 716 ms mean utterance. **Not spike counts:** random deletion at p30 costs the rate arm **0.0124** and clears the 0.03 bar in **0 of 12**, so a null measured under it means nothing; laddering the rate makes **p70** the smallest sensitive rung (**0.0388**, 11/12) and p90 costs **0.1080** at 12/12, with no cell voided and p90 cells still predicting all twenty classes at **0.5982** against chance 0.05. Against that instrument the advantage does not fall but **grows** — +0.1390 at p70 and **+0.1640** at p90, the substrate losing 0.1080 where the read-out loses **0.0698**. **That clause is recorded NOT MET and is reported as NOT MET**: a two-sided band was registered for a one-sided question and fires identically when the advantage strengthens, so the asymmetry needs its own registration before it is a claim. What stands is the comparison — destroying order costs the read-out **0.1208**, destroying counts costs it nothing. **All three properties are measured at one operating point** — h128, feed-forward, `published-2ms`, `d32/L4`, e400 — and none of them transfers to h1024 or to a recurrent substrate on this evidence; the difference-in-differences itself travels to twenty-one points, its decomposition does not. Read-out access is separable from input structure throughout: a `hidden-shuffled` operator permutes the time axis the read-out sees and leaves the substrate's input alone, and the rate arm pays **exactly 0.0000**, twelve of twelve cells byte-identical on every scientific field. ([`RESULT_2026-09-04_W26_SATURATION_IS_REAL_AND_NOT_SPECIFIC.md`](RESULT_2026-09-04_W26_SATURATION_IS_REAL_AND_NOT_SPECIFIC.md); [`RESULT_2026-09-05_W27_THE_TIMESCALE_IS_261_MS.md`](RESULT_2026-09-05_W27_THE_TIMESCALE_IS_261_MS.md); [`RESULT_2026-09-06_W28_THE_READ_OUT_SURVIVES_WHAT_THE_SUBSTRATE_CANNOT.md`](RESULT_2026-09-06_W28_THE_READ_OUT_SURVIVES_WHAT_THE_SUBSTRATE_CANNOT.md))

## Appendix G — The calibration criteria and the 0.80 floor

Two paragraphs lifted from §4.6 on 2026-09-07. Both are about the third-party reference artifact this instrument is calibrated against, not about the attention read-out, and §4.6 states each conclusion where it is used.

Criteria 3 and 4 — `clean_reference` and `historical_reference` — were false for a **provenance** reason rather than an accuracy one: the six third-party PyTorch reference artifacts recorded a `source_fingerprint` frozen on 2026-07-27 that every later kernel edit had moved, while their recorded accuracies already met the requirement. Those six cells were re-run on 2026-08-23 and **every one reproduced its archived value to every recorded digit** — a 150-epoch stochastic PyTorch training run, on CPU, a month later, in a rebuilt environment. Both gates now read `true` and `matrix_authorized` is `true` ([`RESULT_2026-08-23_REFERENCE_RERUN.md`](RESULT_2026-08-23_REFERENCE_RERUN.md)). What still blocks calibration is criterion 5, the Python mirror of the attention axis, which does not exist; and `SHD_INSTRUMENT_STATE` remains a compile-time `Uncalibrated`, a second gate in series with the first.

**The 0.80 `CELL_PASS` floor should not be read as a standard the instrument is failing to meet.** It was derived from one configuration of a reference that is a different model class, and four preregistered ablations of that reference show three of its choices are neutral or harmful to it: its second hidden layer costs 0.0145, its batchnorm appears to cost 0.0058, and its non-spiking summed readout buys 0.0012. Only its dropout clearly earns its place ([`RESULT_2026-08-24_EVERY_CONFIGURABLE_DIFFERENCE_IS_MEASURED.md`](RESULT_2026-08-24_EVERY_CONFIGURABLE_DIFFERENCE_IS_MEASURED.md)). A fifth ablation, which removes the reference's temporal kernel rather than a configuration value, is worth **0.3111** — so the floor rests overwhelmingly on one architectural choice the instrument does not make (§3.8).

## Appendix H — Reproduction commands

Every claim in this paper is reachable from a clean checkout with:

```
cargo test --locked --workspace
cargo clippy --locked --workspace --all-targets -- -D warnings
./scripts/gc_checks.sh          # GC1-GC7, the enforced global constraints
./scripts/record_checks.sh      # twelve gates, 125 SHD-campaign assertions
python3 scripts/build_paper.py  # this manuscript, page-limit enforced
cargo run --locked --release -p binn-lab --bin c1 -- \
  --config-hash c1-118207fbc3eaba53 --out results/c1_g2_replay.md   # Gate G2
```

Post-G2 gates require explicit opt-in (`--enable-c2`, `--enable-c3`, `--enable-r1`, `--enable-r2`); see §2.4.

## Appendix I — The resolution ladder

Finding 5 of §3.5 in full, with the per-rung table. §3.5 states the direction and the withdrawn S-5 test where the result is used.

**Temporal resolution is an axis, and the gain falls as bins get finer** (Figure 4). The `published-Nms` test of this (S-5) was **refuted and is withdrawn**: that family moves bin width and sequence length together, so a single number cannot be attributed to either. Re-asked on `fixed-tN`, which holds a 1400 ms window fixed and varies only the number of frames, the read-out helps at **every** rung and the gain is monotone in resolution:

| contract | bin | `ff+fixed` | d32/L4 | gain | gain > 0 | ≥ 0.80 |
|---|---:|---:|---:|---:|---:|---:|
| `fixed-t100` | 14.0 ms | 0.6672 | 0.8599 | **+0.1927** | 12/12 | 12/12 |
| `fixed-t250` | 5.6 ms | 0.6844 | 0.8594 | **+0.1751** | 12/12 | 12/12 |
| `fixed-t500` | 2.8 ms | 0.7069 | 0.8543 | **+0.1474** | 12/12 | 12/12 |

gain(t500) − gain(t100) = **−0.0453** against a two-sided bar of 0.03, so the advantage **shrinks with finer resolution** — the opposite of the direction S-5 predicted, on the axis S-5 could not isolate. The baseline drifts +0.0397 across the same ladder, inside the 0.05 confound bar, so this is a property of the read-out and not of the substrate beneath it. All three rungs clear the 0.80 gate at 12/12, the coarsest most comfortably. ([`RESULT_2026-08-22_W10_RESOLUTION_LADDER.md`](RESULT_2026-08-22_W10_RESOLUTION_LADDER.md))
