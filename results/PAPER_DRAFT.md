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
from 0.7057 to **0.8332** (gain **+0.1275**, positive in 32/32 seeds, 32/32 at or
above 0.80). That much is unsurprising and not new. The result this paper is
built on is the **conditional**: when the temporal order of the input is
destroyed by permuting time bins — independently per sample, in **both the
training and test splits**, so the task itself becomes rate-solvable — the
read-out's *advantage over a rate read-out* collapses by **+0.1347**, while the
rate read-out loses only **+0.0142** of its own. A **9.5× ratio, 32/32 seeds**.

The distinction matters because "SHD depends on temporal order" is already
established — Cramer et al. (2022) could not exceed 60% on spike-count-only SHD,
and two 2025 studies reach the same conclusion with model-side and spike-time
operators. What has not been measured is which *component's* contribution is the
order-dependent one. This is a difference-in-differences on the **gain**, not on
accuracy, and it is what the read-out is for.

**The contrast is not confined to the configuration it was registered at.** A
preregistered wave carried the same destruction control to seven further
operating points: the difference-in-differences clears its +0.03 bar in **12 of
12 seeds** at h256 (**+0.0862**), h384 (**+0.0767**) and h512 (**+0.0968**), and
at both alternative binnings — `channels-700` (**+0.1122**) and `published-10ms`
(**+0.0959**). Coverage goes from **2 of 21 operating points to 9**, spanning
every width from 128 to 1024 and two contracts and geometries. The read-out's
contribution is order-dependent across the design space, not at a point.

**What that wave also refuted is ours.** The same preregistration asked whether
the shuffle cost *tracks* the gain across width, and it does not: Spearman
**ρ = −0.1430** over the six rungs against a bar of **+0.829**, the n = 6
one-tailed critical value. h768 carries the **smallest** positive gain on the
ladder (+0.0560) and the **largest** difference-in-differences in the wave
(**+0.1881**). So *"the read-out's contribution is order-dependent"* survives and
is now measured at nine points; *"the gain is made of temporal order"* does not
survive as a quantitative account, and this paper does not assert it.

We report three scope limits against it. The gain **inverts at width h1024**
(−0.1618), and on a six-rung ladder that inversion is a threshold — a 0.2178
drop, 6.9× the largest gap below it — not a continuing slope; three preregistered
rescue levers all fail, so the collapse is **located but unexplained**. The 0.80
clearance is geometry-specific. And **0.8332 is not competitive**: the SHD
frontier sits at 95–96.4% via learned delays, adaptation, and spiking
transformers. This instrument carries **no temporal kernel of any kind**, and it
lands where the literature puts a no-delay recurrent SNN. Four preregistered
ablations fail to explain the 0.087 residual against a delay-free reference; the
term-by-term reading attributes it to a 25-tap learned kernel per synapse that
the reference has and the instrument does not. That attribution rests on
elimination and code-reading, **not on an ablation that added the kernel**, and
is the paper's weakest load-bearing inference.

## Abstract — matched-architecture kill gate (secondary program)

Broadcast ±1 three-factor plasticity — surrogate eligibility multiplied by a single ±1 reward — fails a preregistered accuracy and gap bar when the dense leaky-integrate-and-fire forward pass is held identical to a SuperSpike backpropagation-through-time reference. The arm remains at chance on **both** matched forward graphs (feed-forward 0.5000, gap lower confidence bound 0.0000; recurrent 0.5100, LCB −0.0192), against a gradient reference at 1.0000, n = 20 seeds. Every other rule tested on that forward now clears the gate: graded direct feedback alignment (0.9925 / 0.9875), directional REINFORCE with frozen per-neuron feedback (0.9950 / 0.9812), broadcast graded error (0.9975), and a discrete EventProp-style spike-adjoint (0.9450 / 0.8900). **The task therefore separates one rule from a field that otherwise saturates, and it no longer ranks the field**: with every reference at exactly 1.0000, each of those passes reduces to "the arm scored above 0.75". Live k-WTA transfer of the matched REINFORCE and DFA families remains a scoped **negative** across twelve gap-close variants (v13–v24), best gap LCB 0.3127 against a 0.5 threshold.

> **Provenance of these numbers, and it is load-bearing.** The figures above are from a 2026-08-25 re-run under `MATCHED_INPUT_SCALE = 2.0`. Every previously published matched-architecture number was produced on a forward pass that emitted **zero spikes at any seed**, and the arms that most depended on spikes were the ones it most misrepresented — the discrete spike-adjoint read 0.5000 there and reads 0.9450 here, because a method that differentiates through spike times had none. That claim is **withdrawn**, along with the two RL broadcast contrasts (0.5250 → 0.9100 and 0.5113 → 0.7962). See [`RESULT_2026-08-25_MATCHED_ARCH_RERUN.md`](RESULT_2026-08-25_MATCHED_ARCH_RERUN.md).

**Two scope statements are load-bearing and are stated here rather than in a footnote.** First, the 80-epoch schedule undertrains *every* rule on it: raising the reference's budget alone lifts it from 0.9013 to 1.0000 by e640, so `gap_closed` at the canonical budget divides by a reference that is still climbing, and values above 1 are an artefact of the denominator rather than a result. The defensible reading of the ordering between local rules and BPTT at that budget is a statement about **learning speed**, not about a ceiling. Second, the matched task saturates: with every arm reaching 1.0000 at high budget it can no longer separate them, so no ceiling comparison on this task survives convergence.

**Honesty notes.** On the DFA schedule a broadcast-*graded* contrast reaches 0.9975, so the lead negative is ±1 three-factor specifically, not "any broadcast" — and since every other rule tested is now at or near ceiling, the negative is specific in a stronger sense than that phrasing suggests. Locality as a necessary ingredient is evidenced on one-layer XOR, not by the coincidence task alone. We do not claim biological realism, Assembly Calculus success, or impossibility of local learning in principle.

## 0. What is new here, and what is not

> **Provenance of the citations in this section.** The literature positioning
> below was assembled by a search pass on 2026-08-27, not by the author reading
> each source. Numbers marked here were reported as extracted from the primary
> PDF or a Crossref record; a further set encountered during that search
> (Pfa-SNN 96.26, Event-SSMA 95.90, SpikeSCR 95.60, d-cAdLIF 94.85) came only
> from a secondary comparison table and is **deliberately excluded** from the
> claims below. **Every citation in this section must be checked against its
> primary source before submission.** Unlike every SHD number in this paper,
> none of them is machine-verified against cells on disk, and
> `scripts/check_every_number.py` does not sweep this document.


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

**Not new: that SHD depends on temporal order.** The dataset's own authors
constructed spike-count-only variants and could not exceed **60%** on SHD
(Cramer et al., IEEE TNNLS 33(7), 2022). The Neuromorphic Sequential Arena
(IJCAI 2025) removes temporal processing model-side and reports SHD falling
86.48 → 68.51. Yu et al. (arXiv:2507.16043, 2025) randomise spike times while
preserving counts, and separately reverse time, on SHD directly. Three
independent destruction operators, one conclusion, all of it prior to this work.

**Not new, and worth conceding plainly: the accuracy.** The SHD frontier is
95–96.4%, reached by learned delays (DCLS, ICLR 2024), adaptation (SE-adLIF,
Nature Communications 2025), and spiking transformers. This instrument's 0.8332
is not in that band and is not offered as if it were. It is close to the
best-effort no-delay recurrent baseline the dataset's authors themselves report
(83.2 ± 1.3% at 1024 neurons with augmentation), which is where an architecture
carrying no temporal kernel should land, and is the external corroboration §3.8
argues for from ablation alone.

**New: which component's contribution is the order-dependent one.** Every result
above measures how much *accuracy* survives destroying temporal structure. None
measures how much of a *specific component's marginal contribution* survives it.
The contrast in §3.5 is a difference-in-differences — attention's shuffle cost
against the rate read-out's own, on the same seeds, same splits, same
destruction operator — and we find no published equivalent for any read-out on
any neuromorphic benchmark.

**The claim is about presence, not proportion.** Measured across nine operating
points, the order-dependence is present everywhere and its *size* is uncorrelated
with the gain (§3.5). The contribution is therefore a statement about what the
read-out consumes, and explicitly not a decomposition of the gain into an
order-dependent share and a remainder.

**New, and unsupported in either direction: the width collapse.** No published
work reports an attention read-out degrading with hidden width, and none reports
gradient pathology in an attention read-out over spike trains. Width normally
*helps* on SHD (Cramer et al.: 1024 neurons → 76.5%; Bittar & Garner: 3×128 →
3×1024 improves 92.88 → 94.62). §3.5's inversion is therefore an anomaly against
the baseline expectation with no citation to lean on, and the parsimonious
alternative — overfitting on 8,156 training samples, which Cramer et al. document
as severe — is **not excluded by anything in this paper**.

**A caveat on the benchmark itself.** SHD ships no validation set. Baronig et al.
(2025) report the same model at 95.81 ± 0.56 validating on test "to ensure
comparability" and 93.79 ± 0.76 with a proper held-out split — a two-point gap.
Differences below ~1.5 points between published SHD numbers are not reliably
meaningful, and that applies to this paper's comparisons as much as anyone's.

---

## 1. Introduction

Claims that sparse assemblies can learn without backpropagation are only as strong as the object under test. Much of the rhetoric around local synaptic learning mixes rule topology, neuromodulator richness, and spiking front-end engineering. We separate those factors with a preregistered matched-architecture kill gate: the dense-LIF forward, width, frames, readout, splits, and seeds are held fixed, and only the update rule changes.

The narrow primary hypothesis is that a **broadcast ±1 three-factor** rule—surrogate eligibility multiplied by a single ±1 reward—is insufficient to recover SuperSpike BPTT accuracy on a coincidence discrimination task under fixed Gate G2 thresholds. A contrast hypothesis is that richer or more local credit (graded DFA; REINFORCE × frozen feedback weights) can clear the same matched gate. A transfer hypothesis asks whether matched PASS transfers to live muted-θ / k-WTA C1; the honest package answer is **no** (v13–v24 FAIL). On standard neuromorphic audio benchmarks (SHD), we test whether temporal self-attention readouts can unlock temporal order in spiking representations.

We treat the hashed C1 production loop as a secondary, softer negative: it fails its operationalized gate under static scalar modulation, but integrity caveats reduce how far that failure alone can be generalized. Throughout, we refuse biology, neuromorphic hardware, and impossibility claims. Mechanism evidence is summarized as richness × addressability (Figure M; Section 3.1 / 3.4) and temporal order (Section 3.5).

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

---

## 3. Results

### 3.1 Matched-architecture primary results

All figures below are the 2026-08-25 re-run at `MATCHED_INPUT_SCALE = 2.0`, n = 20, reported on both forward graphs because the arms were historically split across two and the difference exceeds the registered 0.02 bar on two of them ([`RESULT_2026-08-25_MATCHED_ARCH_RERUN.md`](RESULT_2026-08-25_MATCHED_ARCH_RERUN.md)).

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

Canonical C1 fails Gate G2 (local 0.4912, gap LCB −0.0048). Trial isolation and temporal positive-control sensitivities fail without clearing the accuracy floor. Capacity sensitivity clears the accuracy floor (0.6775) but leaves gap LCB at 0.0000 versus dense/gradient references—descriptive only, not a G2 PASS.

### 3.3 Live transfer, gap-close, and break-it

Live opt-in REINFORCE feedback fails G2 (local 0.4900, gap LCB 0.0737). Epoch matching alone does not rescue random-B live RFB (local 0.4838). Structured frozen B clears the accuracy floor (0.7262) with gap LCB 0.2567 but still fails the gap bar. Stacking epochs under structured B regresses (0.5200). Structured B on a capacity substrate yields the best gap LCB in the prior suite (0.3127) while clearing the floor (0.6825), remaining short of 0.5. Eligibility×REINFORCE and restored target teach clear the floor but do not beat structured B alone. Break-it protocols close remaining differentials without remassage: live graded DFA (v20) reaches local 0.7325 with gate LCB 0.2601 and chance LCB 0.3321—floor yes, gate no. Soft-WTA × structured B (v21) regresses to chance (0.5025). Matched three-factor under 4× epochs (v22) stays at 0.5000. Finite-θ under SFB (v23) clears the floor (0.6638) with LCB 0.2370. Continuous structured B (v24) does not beat sign-truncated v15 (0.6437 / 0.1380). Spiking-path true DFA rescue fails in one honest attempt (0.6513, gap LCB 0.0733). Differential closure: [`DIFF_CLOSURE.md`](DIFF_CLOSURE.md).

### 3.4 Task evidence

On one-layer `xor_thresh`, broadcast error stays at chance (0.5008) while DFA reaches 0.8267 against gradient 0.7733—a locality flip. On mid-init two-layer depth locality, broadcast also succeeds (0.8158) alongside DFA (0.8250) and REINFORCE×B (0.8033); depth help is not treated as a locality-flip claim.

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

   **The `gain` and `DiD` columns are over the same seeds everywhere except h1024**, where the DiD is over the twelve quadruples and the gain over the twenty intact pairs waves 18–19 extended that width to; over the twelve quadruple seeds the h1024 gain is **−0.1618**. The rank is identical either way.

   **The size of the effect is not the gain.** Spearman ρ between the six per-width gains and their DiDs is **−0.1430** against a preregistered bar of **+0.829**, the n = 6 one-tailed critical value — not a weak positive, absent and faintly negative. h768 buys the least on the ladder and carries the largest DiD in the wave. The registered reading is that the difference-in-differences is a property of the read-out and **not a quantitative account of what the gain is made of**. ([`RESULT_2026-08-29_W21_THE_MECHANISM_TRAVELS_BUT_ITS_SIZE_DOES_NOT.md`](RESULT_2026-08-29_W21_THE_MECHANISM_TRAVELS_BUT_ITS_SIZE_DOES_NOT.md))
3. **Sample efficiency:** Attention reaches 98.1% of e400 accuracy by 10 epochs (0.7337), bracketing convergence at `(5, 10]` epochs. **The denominator is the `d32/L1` arm at convergence (0.7483), not the `d32/L4` headline** — against the headline's 0.8320 the same cell is 88.2%. The two operating points differ: the L1 ladder's e400 gain is +0.0421, not +0.1258. ([`RESULT_2026-08-20_W7_CONVERGENCE_IS_BRACKETED.md`](RESULT_2026-08-20_W7_CONVERGENCE_IS_BRACKETED.md); refines [`RESULT_2026-08-20_W6_ATTENTION_IS_SAMPLE_EFFICIENCY.md`](RESULT_2026-08-20_W6_ATTENTION_IS_SAMPLE_EFFICIENCY.md))
4. **Scope limits** (Figure 3): gain inverts at width h1024, and on a six-rung ladder that inversion is a **threshold rather than a continuing slope**: +0.1258 (h128), +0.0966 (h256), +0.0760 (h384), +0.0876 (h512), +0.0560 (h768), −0.1618 (h1024). The drop into h1024 is **0.2178**, **6.9×** the largest gap below it (0.0316) and more than twice the registered 3× bar, so the collapse sits between **h768 and h1024** and the rungs below it remain positive. **Every rung of this ladder is measured at `d32/L4`, and the inversion is a property of that read-out depth rather than of the width.** At h1024 and the same budget, `d32/L2` gains **+0.0405** in **20/20** seeds and `d32/L3` gains **+0.0371** in 18/20, against L4's −0.1318 in 3/20; the optimum in depth at that width is interior, and it is not established which of L2 or L3 holds it — they differ by 0.0034. So this row bounds *deep* read-outs at h1024 and does not bound h1024. What makes an arm collapse is still unexplained: L3 sits above the registered gradient-norm sickness threshold at 1.347 and gains anyway, which is why the numerical account was refused ([`RESULT_2026-08-28_W18_19_THE_DEPTH_OPTIMUM_IS_INTERIOR.md`](RESULT_2026-08-28_W18_19_THE_DEPTH_OPTIMUM_IS_INTERIOR.md)). The decay above the collapse is **not strictly ordered** — h384 and h512 are not distinguishable at twelve seeds (paired difference −0.0116, sd 0.0253, negative in 7 of 12) — so no monotonicity is claimed. Gain is positive across geometries (+0.1090 on `channels-700`, +0.1491 on `published-10ms`), but 0.80 clearance is geometry-specific (0.7864 on `channels-700`). ([`RESULT_2026-08-21_W8_HEADLINE_SCOPE_IS_MEASURED.md`](RESULT_2026-08-21_W8_HEADLINE_SCOPE_IS_MEASURED.md); [`RESULT_2026-08-27_W15_17_THE_COLLAPSE_IS_A_THRESHOLD.md`](RESULT_2026-08-27_W15_17_THE_COLLAPSE_IS_A_THRESHOLD.md))

   **What the h1024 collapse is not.** Three preregistered levers — surrogate scale 0.5 and 0.25, and gradient clipping at 1000.0 — were run at h1024/d32/L4 to test whether the inversion is an optimisation failure. All three are **negative and worse than the arm they were meant to rescue** (−0.2106, −0.2565, −0.0904). Clipping moves the median epoch-mean gradient norm from 55.494 to 11.660, a real effect in the intended direction, and accuracy does not follow. **The collapse is located but not explained**; nothing in this paper offers a mechanism for it.

   **And it is not the temporal-order account in disguise.** If the inversion and the mechanism were the same phenomenon, then where the read-out buys nothing there should be no order-dependent benefit left to destroy. That prediction was registered before the cells existed and it **failed**: at h1024 the difference-in-differences is **+0.1122** in **10 of 12** seeds against a registered ceiling of +0.02, while the gain over those same twelve seeds is **−0.1618**. The read-out consumes temporal order while performing worse than no read-out at all. Nothing in this paper's account permits that, and per the preregistration it is the paper's **leading open problem** rather than a caveat. It also leaves the overfitting alternative exactly where it was: that argument was conditional on the shuffle cost collapsing, and it did not. ([`RESULT_2026-08-29_W21_THE_MECHANISM_TRAVELS_BUT_ITS_SIZE_DOES_NOT.md`](RESULT_2026-08-29_W21_THE_MECHANISM_TRAVELS_BUT_ITS_SIZE_DOES_NOT.md))
5. **Temporal resolution is an axis, and the gain falls as bins get finer** (Figure 4). The `published-Nms` test of this (S-5) was **refuted and is withdrawn**: that family moves bin width and sequence length together, so a single number cannot be attributed to either. Re-asked on `fixed-tN`, which holds a 1400 ms window fixed and varies only the number of frames, the read-out helps at **every** rung and the gain is monotone in resolution:

   | contract | bin | `ff+fixed` | d32/L4 | gain | gain > 0 | ≥ 0.80 |
   |---|---:|---:|---:|---:|---:|---:|
   | `fixed-t100` | 14.0 ms | 0.6672 | 0.8599 | **+0.1927** | 12/12 | 12/12 |
   | `fixed-t250` | 5.6 ms | 0.6844 | 0.8594 | **+0.1751** | 12/12 | 12/12 |
   | `fixed-t500` | 2.8 ms | 0.7069 | 0.8543 | **+0.1474** | 12/12 | 12/12 |

   gain(t500) − gain(t100) = **−0.0453** against a two-sided bar of 0.03, so the advantage **shrinks with finer resolution** — the opposite of the direction S-5 predicted, on the axis S-5 could not isolate. The baseline drifts +0.0397 across the same ladder, inside the 0.05 confound bar, so this is a property of the read-out and not of the substrate beneath it. All three rungs clear the 0.80 gate at 12/12, the coarsest most comfortably. ([`RESULT_2026-08-22_W10_RESOLUTION_LADDER.md`](RESULT_2026-08-22_W10_RESOLUTION_LADDER.md))

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

The +0.1258 of §3.5 has two readings the campaign could not separate, because every one of its 720 cells sat on a single substrate, `ff+fixed`: the read-out **adds** temporal structure no substrate of this kind represents, or it **substitutes** for the threshold adaptation and recurrence that `ff+fixed` happens not to have. ETLP's conclusion — that adaptation and a recurrent topology are what a spiking network needs for rich temporal structure — makes the second reading the live one. Three waves settle it.

**Adaptation makes no difference to the gain, or to anything else.** At the anchor (h128, `published-2ms`, `adjacent-sum-5`, e400, d32/L4, n=12), attention's gain is **+0.1258** on `ff+fixed` and **+0.1285** on `ff+alif`. The difference is **+0.0027** against a two-sided bar of 0.03, and is positive in **6 of 12** seeds — a coin flip. Adaptation alone does not help either: `ff+alif` reaches **0.7018** against `ff+fixed`'s 0.7062, better in **3 of 12** seeds, with **0 of 12** over the 0.80 gate. At this operating point threshold adaptation is inert. ([`RESULT_2026-08-23_W12_ATTENTION_DOES_NOT_SUBSTITUTE_FOR_ADAPTATION.md`](RESULT_2026-08-23_W12_ATTENTION_DOES_NOT_SUBSTITUTE_FOR_ADAPTATION.md))

**The recurrent substrate is measurable only at one operating point, and finding it was its own wave.** `rec+alif` completes **11 of 12** at the anchor budget only at surrogate scale 0.4; at the registered default of 1.0 it completes 8 of 12. `rec+fixed` completes 12 of 24 across both scales and fails by a different mechanism — **saturation**, ten cells voided with up to 52% of hidden units pinned at maximum firing, none by divergence at scale 0.4. Adaptation is what prevents that, so on the recurrent substrate adaptation is *stabilising*, which is the opposite of the hypothesis that wave's own name asserted. ([`RESULT_2026-08-23_W13_RECURRENT_STABILITY.md`](RESULT_2026-08-23_W13_RECURRENT_STABILITY.md))

**On the recurrent substrate the gain roughly doubles.** With every arm run at scale 0.4 so substrate and scale cannot be confounded, and paired over the seeds where both arms completed:

| substrate | pairs | rate read-out | + attention d32/L4 | gain |
|---|---:|---:|---:|---:|
| `rec+alif` | 10 | 0.5262 | 0.7874 | **+0.2612** |
| `ff+fixed` | 12 | 0.7088 | 0.8289 | **+0.1201** |

The difference is **+0.1411** against a bar of 0.03, positive in **10 of 10** recurrent pairs. The scale is not doing the work: `ff+fixed` at 0.4 scores **0.7088** against **0.7062** archived at 1.0, a difference of +0.0026. ([`RESULT_2026-08-23_W14_ATTENTION_AND_RECURRENCE_ARE_COMPLEMENTARY.md`](RESULT_2026-08-23_W14_ATTENTION_AND_RECURRENCE_ARE_COMPLEMENTARY.md))

**So substitution is refuted on both axes**, and the read-out's advantage is indifferent to adaptation and *larger* where the substrate is recurrent. Read with §3.5's shuffle result — 94.5% of the advantage contingent on temporal order at n=32 — the claim the paper supports is about what the read-out consumes, not about a deficiency of one substrate.

Four limits are load-bearing and are stated here rather than in a footnote.

1. **The recurrent gain is measured from a lower base.** `rec+alif` starts 0.18 below `ff+fixed` and has 0.4738 of headroom against 0.2912. Normalising by headroom — post-hoc, not registered — the ratio falls from 2.2× to **1.34×**. The ordering survives; most of its apparent size does not.
2. **The recurrent substrate does not win.** `rec+alif+attn` reaches 0.7874 against `ff+fixed+attn`'s 0.8289 at the same scale. Attention closes most of the gap the substrate gives away, and not all of it. No verdict is issued on that ordering.
3. **The recurrent arms are numerically extreme**, with peak gradient norms to 4.9e32 against 1.13e8 for the largest cell anywhere else in the campaign, and the comparison rests on **ten pairs, the registered minimum**. The two arms lost different seeds; one further loss on either would have made the comparison unreportable.
4. **Survivorship is reduced, not removed.** Pairing on seed compares the same trajectories rather than two differently filtered subsets, but the surviving recurrent pairs are those that did not diverge, and divergence is not random. The feed-forward comparison carries no such exposure at 12/12.


### 3.8 Where 0.8320 sits against the literature, and why the gap is a class difference

A read-out result on a standard benchmark invites one question immediately, and the paper answers it here rather than leaving it to a reader: published SHD accuracies for surrogate-gradient spiking networks sit around 0.90–0.95, and the pinned third-party reference used for calibration in this work reaches **0.9390 / 0.9368 / 0.9371** on the clean protocol. The instrument's converged attention arm reaches **0.8320**. The difference is about **0.087** against the best delay-free variant of that reference, and it is not a tuning gap.

Four preregistered ablations turned every knob on the reference that could plausibly account for it — depth, dropout, read-out style, normalisation. **Together they do not explain it, and two of them subtract**: removing the reference's second hidden layer *gains* 0.0145 and removing its batchnorm appears to gain 0.0058. Of everything measured, only dropout is a positive contributor the instrument lacks, and it is worth 0.0128 of the 0.087.

Reading the two forward passes term by term found what the configuration surface could not. The reference carries a **`Dcls1d` temporal kernel of 25 taps per synapse on every layer**, spanning 250 ms, and its membrane decays to a third in a single timestep — its temporal integration lives in the convolution and its neuron is close to a pointwise nonlinearity. The instrument is the mirror image: **no temporal kernel in any form**, and a membrane retaining 0.82 per step to compensate. These are not one architecture at two operating points ([`FINDING_2026-08-24_THE_FORWARD_PASSES_DIFFER_IN_KIND.md`](FINDING_2026-08-24_THE_FORWARD_PASSES_DIFFER_IN_KIND.md)).

That reframes this paper's own result rather than excusing it. The attention read-out is a mechanism over the time axis added to a substrate that has none, and §3.5's shuffle control shows 94.5% of its benefit is contingent on temporal order at n=32. **The most likely reading of the +0.1258 is that the read-out is recovering a fraction of what a learned temporal kernel supplies** — which is a claim about what the read-out consumes, and is consistent with §3.7's finding that its advantage is larger, not smaller, where the substrate is recurrent.

**Three limits on this section.** The kernel's contribution is supported by elimination and by reading the code, **not by an ablation that removed it** — removing 25 taps per synapse is a different model, not a configuration, and `scripts/test_shd_calibration.py::ReferenceKernelInvariantTests` makes the two underlying claims executable: no documented `model_type` removes the kernel, and binning cannot be varied without moving its width. The ablations were run at **n = 1** with a three-seed spread of 0.0022, so the batchnorm effect sits in the suggestive band and is not quoted as resolved. And they vary the *reference*, so every statement about the instrument is an inference from a difference of differences.

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
destruction control exists at **9 of 21 operating points**, covering widths 128
through 1024, both contracts and both geometries, and
`scripts/mechanism_coverage.py` recomputes that on every gate run. Twelve
operating points still carry intact arms with no `bin-shuffled` twin, and no
mechanism claim is made at those. We do not claim calibration, and the reason has
changed.

Criteria 3 and 4 — `clean_reference` and `historical_reference` — were false for a **provenance** reason rather than an accuracy one: the six third-party PyTorch reference artifacts recorded a `source_fingerprint` frozen on 2026-07-27 that every later kernel edit had moved, while their recorded accuracies already met the requirement. Those six cells were re-run on 2026-08-23 and **every one reproduced its archived value to every recorded digit** — a 150-epoch stochastic PyTorch training run, on CPU, a month later, in a rebuilt environment. Both gates now read `true` and `matrix_authorized` is `true` ([`RESULT_2026-08-23_REFERENCE_RERUN.md`](RESULT_2026-08-23_REFERENCE_RERUN.md)). What still blocks calibration is criterion 5, the Python mirror of the attention axis, which does not exist; and `SHD_INSTRUMENT_STATE` remains a compile-time `Uncalibrated`, a second gate in series with the first.

**The 0.80 `CELL_PASS` floor should not be read as a standard the instrument is failing to meet.** It was derived from one configuration of a reference that is a different model class, and four preregistered ablations of that reference show three of its choices are neutral or harmful to it: its second hidden layer costs 0.0145, its batchnorm appears to cost 0.0058, and its non-spiking summed readout buys 0.0012. Only its dropout clearly earns its place ([`RESULT_2026-08-24_EVERY_CONFIGURABLE_DIFFERENCE_IS_MEASURED.md`](RESULT_2026-08-24_EVERY_CONFIGURABLE_DIFFERENCE_IS_MEASURED.md)).

We do not claim cortical realism, Assembly Calculus PASS, neuromorphic deployment, or impossibility of local learning in principle.

The following suites are explicitly **withdrawn**:
1. `track-b-rescue` v130 online learned FB PASS (withdrawn under v131 `INVALID_HARNESS`).
2. `deep-snn-scaling` depth collapse (withdrawn under v134; all ceilings at chance).
3. `shd-scientific-sweep` (withdrawn; synthetic data).

---

## 5. Reproducibility

Scientific hashes and commands are listed in [`REPRO_ARTIFACT_CHECKLIST.md`](REPRO_ARTIFACT_CHECKLIST.md) and [`PAPER_RESULTS_TABLE.md`](PAPER_RESULTS_TABLE.md). Rebuild with `cargo test --locked --workspace` from `binn/`. Camera-ready citations must point at on-disk notes or exact `--config-hash` replays. Attention campaign artifacts are preserved under `results/shd_attention_campaign_v1/` (waves 1–7 plus `r1cal`) and `results/shd_attention_campaign_v2/` (wave 8 as `w8*__`, wave 9 as `w9dim__` / `w9shf__`).

---

## Appendix A — Integrity limitations (protocol v2)

Cross-trial STDP pairing times (`ThreeFactor.last_spike`) are retained on canonical C1 while eligibility traces are cleared. Membrane and dendritic reset is incomplete relative to the C3 production path. Hidden thresholds are set to infinity during the integrate window, suppressing natural spiking. Assembly Calculus `project` is not exercised on the canonical loop (it is wired under `c1-project*` and fails G2 there). Exact-forward arms labeled “e-prop/DFA” are hybrid eligibility × transported modulators unless the true σ′ e-prop family is cited explicitly.

## Appendix B — Post-G2 harvest (banner)

G3 / G4 / hybrid H0 numbers and hashes: [`APPENDIX_POST_G2.md`](APPENDIX_POST_G2.md). Mechanism figure cells: [`PAPER_FIGURE_SPEC.md`](PAPER_FIGURE_SPEC.md) Figure M. These rows do not reopen G2.
