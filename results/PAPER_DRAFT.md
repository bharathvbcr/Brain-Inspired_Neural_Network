# Broadcast ±1 three-factor credit fails a matched dense-LIF gate

> **Provenance, current as of 2026-08-22.** Four results were withdrawn from this
> package during the 2026-08-19→22 record repair and **none of them appears above**:
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
> `scripts/record_checks.sh` machine-checks the **SHD attention-campaign**
> numbers — twelve of them, recomputed from the archived cells by
> `scripts/verify_published_numbers.py` against the wave-8, wave-9, and d32/L4
> headline result documents, plus four prose checks on this draft. The
> matched-architecture numbers in this abstract are **not** among them: they are
> attested by the on-disk hashed run records cited row-by-row in
> [`PAPER_RESULTS_TABLE.md`](PAPER_RESULTS_TABLE.md), not by that script.
> Full record:
> [`SUMMARY_2026-08-22_CAMPAIGN_AND_RECORD_REPAIR.md`](SUMMARY_2026-08-22_CAMPAIGN_AND_RECORD_REPAIR.md).

*Camera-ready draft (prose). Numbers: [`PAPER_RESULTS_TABLE.md`](PAPER_RESULTS_TABLE.md). Claims: [`PUBLISHABLE_CLAIMS.md`](PUBLISHABLE_CLAIMS.md). Figures: [`PAPER_FIGURE_SPEC.md`](PAPER_FIGURE_SPEC.md).*

---

## Abstract

Broadcast ±1 three-factor plasticity — surrogate eligibility multiplied by a single ±1 reward — fails a preregistered accuracy and gap bar when the dense leaky-integrate-and-fire forward pass is held identical to a SuperSpike backpropagation-through-time reference. Under matched-architecture protocol v4 (`c1-match-5dc6822e71229e9e`) the broadcast ±1 arm remains at chance (mean accuracy 0.5000, gap lower confidence bound 0.0000) while the gradient reference reaches 0.8963. On that same forward family, graded direct feedback alignment and directional REINFORCE with frozen per-neuron feedback clear the matched gate (`c1-dfa-c8c4fe0899908b84`: 0.9387, gap LCB 0.6894; `c1-rl-42eddc9c801308e9`: 0.9200, gap LCB 0.6846). Continuous reward-prediction-error scalar broadcast without spatial feedback alignment remains at chance (0.5120, LCB −0.0230). A discrete EventProp-style spike-adjoint head-to-head on the same matched forward also fails (`c1-eventprop-5bb083d5e88d0ad2`: 0.5000 against SuperSpike 0.9150; discrete ≠ continuous Wunderlich–Pehle). Live k-WTA transfer of the matched REINFORCE and DFA families remains a scoped **negative** across twelve gap-close variants (v13–v24), best gap LCB 0.3127 against a 0.5 threshold.

**Two scope statements are load-bearing and are stated here rather than in a footnote.** First, the 80-epoch schedule undertrains *every* rule on it: raising the reference's budget alone lifts it from 0.9013 to 1.0000 by e640, so `gap_closed` at the canonical budget divides by a reference that is still climbing, and values above 1 are an artefact of the denominator rather than a result. The defensible reading of the ordering between local rules and BPTT at that budget is a statement about **learning speed**, not about a ceiling. Second, the matched task saturates: with every arm reaching 1.0000 at high budget it can no longer separate them, so no ceiling comparison on this task survives convergence.

**Honesty notes.** On the DFA schedule a broadcast-*graded* contrast also reaches 0.9863, so the lead negative is ±1 three-factor specifically, not "any broadcast". Locality as a necessary ingredient is evidenced on one-layer XOR, not by the coincidence task alone. We do not claim biological realism, Assembly Calculus success, or impossibility of local learning in principle.

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

The benchmark tests temporal structure preservation across depth ($L \in \{1, 2, 4\}$), width ($h \in \{128, 512, 1024\}$), binning geometries (`adjacent-sum-5`, `channels-700`, `published-10ms`), temporal resolution at fixed window (`fixed-t100/t250/t500`), temporal shuffling controls (`bin-shuffled`, `channel-shuffled`), and — for the substitution question of §3.7 — the **spiking substrate itself**: $\{$feed-forward, recurrent$\} \times \{$fixed threshold, adaptive threshold$\}$, written `ff+fixed`, `ff+alif`, `rec+fixed`, `rec+alif`.

---

## 3. Results

### 3.1 Matched-architecture primary results

Broadcast ±1 three-factor fails the matched gate at chance (0.5000) with gap LCB 0.0000 while the SuperSpike BPTT gradient ceiling learns (0.8963). Graded DFA passes (0.9387, gap LCB 0.6894). REINFORCE × frozen feedback passes as primary (0.9200, gap LCB 0.6846). **Online learned feedback alignment (v130 schedule)** was withdrawn under v131 after reporting `INVALID_HARNESS` (ceiling-inverted warning on 3/20 seeds). Continuous scalar RPE broadcast remains at chance (0.5120, LCB -0.0230 FAIL), confirming that continuous magnitude without spatial directionality is information-theoretically insufficient on this gate. Discrete EventProp-style spike-adjoint H2H **FAIL**s at chance (0.5000, gap LCB 0.0000) against SuperSpike 0.9150 (`c1-eventprop-5bb083d5e88d0ad2`). The `live-transfer-rescue` binary is matched-only and **does not** constitute live k-WTA transfer; at v132 **every arm reports `INVALID_HARNESS`** — the ceiling-inverted warning fires on 3 of 20 seeds, with the arms at 1.0000 against a reference of 0.9895 — so no number from it is cited here.

Honesty note required in the main text: on the DFA matched schedule, a broadcast-graded contrast also reaches high accuracy (0.9863). The lead negative therefore concerns **broadcast ±1 three-factor** credit, not the claim that every broadcast scalar fails coincidence. Locality as a necessary ingredient is evidenced on XOR (Section 3.4), not by coincidence DFA alone. Figure M plots this as a richness × addressability mechanism panel (including the 0.9863 cell and the XOR locality flip).

### 3.2 Engine C1 secondary results

Canonical C1 fails Gate G2 (local 0.4912, gap LCB −0.0048). Trial isolation and temporal positive-control sensitivities fail without clearing the accuracy floor. Capacity sensitivity clears the accuracy floor (0.6775) but leaves gap LCB at 0.0000 versus dense/gradient references—descriptive only, not a G2 PASS.

### 3.3 Live transfer, gap-close, and break-it

Live opt-in REINFORCE feedback fails G2 (local 0.4900, gap LCB 0.0737). Epoch matching alone does not rescue random-B live RFB (local 0.4838). Structured frozen B clears the accuracy floor (0.7262) with gap LCB 0.2567 but still fails the gap bar. Stacking epochs under structured B regresses (0.5200). Structured B on a capacity substrate yields the best gap LCB in the prior suite (0.3127) while clearing the floor (0.6825), remaining short of 0.5. Eligibility×REINFORCE and restored target teach clear the floor but do not beat structured B alone. Break-it protocols close remaining differentials without remassage: live graded DFA (v20) reaches local 0.7325 with gate LCB 0.2601 and chance LCB 0.3321—floor yes, gate no. Soft-WTA × structured B (v21) regresses to chance (0.5025). Matched three-factor under 4× epochs (v22) stays at 0.5000. Finite-θ under SFB (v23) clears the floor (0.6638) with LCB 0.2370. Continuous structured B (v24) does not beat sign-truncated v15 (0.6437 / 0.1380). Spiking-path true DFA rescue fails in one honest attempt (0.6513, gap LCB 0.0733). Differential closure: [`DIFF_CLOSURE.md`](DIFF_CLOSURE.md).

### 3.4 Task evidence

On one-layer `xor_thresh`, broadcast error stays at chance (0.5008) while DFA reaches 0.8267 against gradient 0.7733—a locality flip. On mid-init two-layer depth locality, broadcast also succeeds (0.8158) alongside DFA (0.8250) and REINFORCE×B (0.8033); depth help is not treated as a locality-flip claim.

### 3.5 SHD attention read-out and mechanism

Across 700+ cells (n=12 per contrast, 0 voided), the time-axis attention read-out establishes five findings on SHD:

1. **Headline accuracy:** `ff+fixed+attn` at `d32/L4` at `e400` reaches **0.8320** with **12/12 seeds ≥ 0.80**, budget-stable (|e400−e200|=0.0002), providing a **+0.1258** gain over the rate readout `ff+fixed` (0.7062). ([`RESULT_2026-08-20_D32L4_CLEARS_THE_080_GATE.md`](RESULT_2026-08-20_D32L4_CLEARS_THE_080_GATE.md))
2. **Temporal order is the mechanism:** Under bin-shuffling, the attention arm drops **+0.1337** (from 0.8320 to 0.6983) across **12 of 12 seeds**, while the plain arm drops only **+0.0128** (from 0.7062 to 0.6934)—a **10× factor**. The attention advantage collapses from +0.1258 to +0.0050; **96% of the readout benefit is contingent on temporal order**. ([`RESULT_2026-08-21_W9_THE_MECHANISM_HOLDS_AT_THE_HEADLINE.md`](RESULT_2026-08-21_W9_THE_MECHANISM_HOLDS_AT_THE_HEADLINE.md))
3. **Sample efficiency:** Attention reaches 98.1% of e400 accuracy by 10 epochs (0.7337), bracketing convergence at `(5, 10]` epochs. ([`RESULT_2026-08-20_W7_CONVERGENCE_IS_BRACKETED.md`](RESULT_2026-08-20_W7_CONVERGENCE_IS_BRACKETED.md); refines [`RESULT_2026-08-20_W6_ATTENTION_IS_SAMPLE_EFFICIENCY.md`](RESULT_2026-08-20_W6_ATTENTION_IS_SAMPLE_EFFICIENCY.md))
4. **Scope limits:** Gain inverts at width h1024 (−0.1618 at L4). Gain is positive across geometries (+0.1090 on `channels-700`, +0.1491 on `published-10ms`), but 0.80 clearance is geometry-specific (0.7864 on `channels-700`). ([`RESULT_2026-08-21_W8_HEADLINE_SCOPE_IS_MEASURED.md`](RESULT_2026-08-21_W8_HEADLINE_SCOPE_IS_MEASURED.md))
5. **Temporal resolution is an axis, and the gain falls as bins get finer.** The `published-Nms` test of this (S-5) was **refuted and is withdrawn**: that family moves bin width and sequence length together, so a single number cannot be attributed to either. Re-asked on `fixed-tN`, which holds a 1400 ms window fixed and varies only the number of frames, the read-out helps at **every** rung and the gain is monotone in resolution:

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
   `(0.9387 − 0.5)/(0.8963 − 0.5) = 1.107`. Values above 1 are an artefact of the
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

**So substitution is refuted on both axes**, and the read-out's advantage is indifferent to adaptation and *larger* where the substrate is recurrent. Read with §3.5's shuffle result — 96% of the advantage contingent on temporal order — the claim the paper supports is about what the read-out consumes, not about a deficiency of one substrate.

Four limits are load-bearing and are stated here rather than in a footnote.

1. **The recurrent gain is measured from a lower base.** `rec+alif` starts 0.18 below `ff+fixed` and has 0.4738 of headroom against 0.2912. Normalising by headroom — post-hoc, not registered — the ratio falls from 2.2× to **1.34×**. The ordering survives; most of its apparent size does not.
2. **The recurrent substrate does not win.** `rec+alif+attn` reaches 0.7874 against `ff+fixed+attn`'s 0.8289 at the same scale. Attention closes most of the gap the substrate gives away, and not all of it. No verdict is issued on that ordering.
3. **The recurrent arms are numerically extreme**, with peak gradient norms to 4.9e32 against 1.13e8 for the largest cell anywhere else in the campaign, and the comparison rests on **ten pairs, the registered minimum**. The two arms lost different seeds; one further loss on either would have made the comparison unreportable.
4. **Survivorship is reduced, not removed.** Pairing on seed compares the same trajectories rather than two differently filtered subsets, but the surviving recurrent pairs are those that did not diverge, and divergence is not random. The feed-forward comparison carries no such exposure at 12/12.


---

## 4. Discussion and limitations

### 4.1 Lead claim and mechanism

The cleanest publishable claim is rule-topological: under a fixed dense-LIF forward, **broadcast ±1 three-factor** credit does not close the preregistered gap to SuperSpike BPTT, while graded DFA and REINFORCE×frozen-`B` do. That contrast supports modulator richness and feedback addressability as material factors on this gate (Figure M), without equating matched success to live sparse/k-WTA success.

Richness alone is not “locality”: on the DFA matched schedule, broadcast-*graded* also reaches 0.9863. The lead FAIL is therefore specifically ±1 × surrogate eligibility under broadcast, not an indiscriminate “broadcast credit topology” ban. Addressability / locality as a necessary ingredient is evidenced by the one-layer XOR locality flip (broadcast graded fails; DFA solves), not by coincidence DFA alone.

**A6 Ceiling Health Caveat.** The 80-epoch schedule undertrains the gradient reference (0.8963 / 0.9013 at e80, climbing to 1.0000 at e640; `RESULT_2026-08-19_A6_CEILING_HEALTH.md`). Therefore, gap closed values at e80 reflect *learning speed* on a saturating task rather than asymptotic representation capacity.

**Falsifier.** A matched ±1 three-factor arm that clears the accuracy floor *and* gap LCB under the same dense-LIF forward, splits, and Gate G2 numeric thresholds would overturn the lead claim. Silent threshold changes, hash remassage, or live-path substitutions do not count.

### 4.2 Transfer barrier and live negatives

The live transfer package is a scoped negative. Structured feedback is the strongest accuracy lever among tested gap-close arms, and capacity×structured yields the best prior gap LCB, but none—including live DFA, soft-WTA, mute-off, and continuous-B probes—clear Gate G2 (v13–v24). Matched undertraining does not explain the broadcast ±1 three-factor FAIL (v22 remains at chance). Reporting floor clearance without gap clearance would overclaim. Engine C1 remains a valid operationalized pipeline negative only when integrity caveats are explicit (Appendix A). Do **not** cite v131 / `live-transfer-rescue` as live-engine rescue: that binary is matched-only.

Soft-WTA × structured B on live C1 uses disclosed temperature `T=1` (v21). That probe is motivated by a hybrid soft→hard collapse whose transfer-collapse temperature is **T=2.0** on a separate hybrid ladder (`binn-hybrid-winner-temp-v1-fa7710de68ad7bfe`). Hybrid T=2.0 is **not** live v21; do not equate the appendix hybrid mechanism note with the live soft-WTA protocol.

### 4.3 Baselines and EventProp H2H

The primary gradient ceiling on the matched gate is **SuperSpike BPTT** on the fixed dense-LIF forward. True σ′ e-prop (`c1x-eprop-true-*`, true-surrogate mean 0.7125) is a methods footnote only—not a H2H claim that e-prop rescues broadcast ±1 insufficiency. **Discrete EventProp-style spike-adjoint** was run H2H under protocol v28 (`c1-eventprop-5bb083d5e88d0ad2`): mean **0.5000** vs SuperSpike **0.9150**, gap LCB **0.0000**, **FAIL**. Honesty: this is a **discrete** hard spike-gate adjoint on the matched dense-LIF forward — **not** continuous Wunderlich–Pehle (2021) hybrid EventProp. Hybrid exact-forward arms labeled “e-prop/DFA” remain eligibility × transported modulators unless the true-σ′ family is cited explicitly.

### 4.4 Efficiency honesty (F1 / F2 / F5)

These limitations constrain how far efficiency rhetoric should travel with the negative result:

- **F1 (scan barrier).** Sub-threshold membrane dynamics admit chunked associative scans; the hard spike reset remains a sequential, data-dependent barrier. Training-time parallelism is therefore *partial*, not a clean escape from recurrent-time cost.
- **F2 (forward remains sequential).** Online local learning removes the backward-unroll / activation-storage half of BPTT cost; it does **not** remove sequential forward simulation within a stream. Parallelism is across neurons, areas, and independent streams—not across time within one stream.
- **F5 (activity ≠ compute).** Sparse activity ratios do not translate linearly into work. Honest software efficiency is work-per-accuracy including per-event queue and pointer-chase overhead; large multipliers are a neuromorphic-hardware claim, not a CPU/GPU default.

### 4.5 Appendix-only: G3 / G4 / H0 (do not reopen G2)

Continual forgetting (C2 / Gate G3 FAIL), multi-area scaling (R2 / Gate G4 **NO-GO** degrade curve), and hybrid H0 (**HYBRID_NO_GO**) live in [`APPENDIX_POST_G2.md`](APPENDIX_POST_G2.md). They are post-G2 / exploratory. The camera-ready banner does **not** reopen Gate G2 or remassage `c1-118207fbc3eaba53`. G4 NO-GO redirects away from scaling more areas under the same ±1 three-factor substrate; any future Micro / isolate capacity stress is engineering headroom after G2 FAIL, not a Foundation unlock and not part of this MUST package.

### 4.6 Neuromorphic benchmark scope and non-claims

The SHD attention readout results are scoped to **h128 / `published-2ms` / `adjacent-sum-5`**. We do not claim calibration. The blocking gates are criteria 3 and 4 — `clean_reference` and `historical_reference` — and they are false for a **provenance** reason, not an accuracy one: the six third-party PyTorch reference artifacts record a `source_fingerprint` frozen on 2026-07-27 that every later kernel edit has moved, while their recorded accuracies (clean 0.9390 / 0.9368 / 0.9371 against a 0.80 floor) already meet the requirement. Criterion 5, the Python mirror, is **not currently reachable**, because `matrix_authorized` conjoins those two gates ahead of it ([`FINDING_2026-08-21_CALIBRATION_GAP_IS_PROVENANCE_NOT_ACCURACY.md`](FINDING_2026-08-21_CALIBRATION_GAP_IS_PROVENANCE_NOT_ACCURACY.md)). We do not claim cortical realism, Assembly Calculus PASS, neuromorphic deployment, or impossibility of local learning in principle.

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
