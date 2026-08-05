# Broadcast ±1 three-factor credit fails a matched dense-LIF gate

*Camera-ready draft (prose). Numbers: [`PAPER_RESULTS_TABLE.md`](PAPER_RESULTS_TABLE.md). Claims: [`PUBLISHABLE_CLAIMS.md`](PUBLISHABLE_CLAIMS.md). Figures: [`PAPER_FIGURE_SPEC.md`](PAPER_FIGURE_SPEC.md).*

---

## Abstract

Broadcast ±1 three-factor plasticity—surrogate eligibility multiplied by a single ±1 reward—fails a preregistered accuracy and gap bar when the dense leaky-integrate-and-fire forward pass is held identical to a SuperSpike backpropagation-through-time reference. Under matched-architecture protocol v4 (`c1-match-5dc6822e71229e9e`), the broadcast ±1 three-factor arm remains at chance (mean accuracy 0.5000) with gap lower confidence bound 0.0000, while the gradient ceiling reaches 0.8963. On that same forward family, graded direct feedback alignment, directional REINFORCE with frozen per-neuron feedback, and **online learned feedback alignment** clear the matched gate (`c1-dfa-c8c4fe0899908b84`: primary 0.9387, gap LCB 0.6894; `c1-rl-42eddc9c801308e9`: primary 0.9200, gap LCB 0.6846; **`track-b-rescue` v130** schedule ID — not a `c1-*-<hex>` hash: primary **1.0000**, gap LCB **0.9988** PASS matched). Continuous reward prediction error (RPE) scalar broadcast without spatial feedback alignment remains at chance (0.5120, LCB -0.0230 FAIL). A discrete EventProp-style spike-adjoint H2H on the same matched forward **FAIL**s (`c1-eventprop-5bb083d5e88d0ad2`: mean 0.5000 vs SuperSpike 0.9150, gap LCB 0.0000; discrete ≠ continuous Wunderlich–Pehle). Live k-WTA transfer of matched REINFORCE / DFA families remains a scoped **negative** (v13–v24 FAIL). The misnamed `live-transfer-rescue` v131 binary is **matched-only** and must not be read as live-engine PASS. Honesty note: on the DFA schedule a broadcast-*graded* contrast also reaches 0.9863, so the lead FAIL is ±1 three-factor, not “any broadcast.” Locality as a necessary ingredient is evidenced on one-layer XOR, not by coincidence alone. We do not claim biological realism, Assembly Calculus success, or impossibility of local learning in principle.

---

## 1. Introduction

Claims that sparse assemblies can learn without backpropagation are only as strong as the object under test. Much of the rhetoric around local synaptic learning mixes rule topology, neuromodulator richness, and spiking front-end engineering. We separate those factors with a preregistered matched-architecture kill gate: the dense-LIF forward, width, frames, readout, splits, and seeds are held fixed, and only the update rule changes.

The narrow primary hypothesis is that a **broadcast ±1 three-factor** rule—surrogate eligibility multiplied by a single ±1 reward—is insufficient to recover SuperSpike BPTT accuracy on a coincidence discrimination task under fixed Gate G2 thresholds. A contrast hypothesis is that richer or more local credit (graded DFA; REINFORCE × frozen feedback weights; online learned feedback alignment) can clear the same matched gate. A transfer hypothesis asks whether matched PASS transfers to live muted-θ / k-WTA C1; the honest package answer is **no** (v13–v24 FAIL).

We treat the hashed C1 production loop as a secondary, softer negative: it fails its operationalized gate under static scalar modulation, but integrity caveats reduce how far that failure alone can be generalized. Throughout, we refuse biology, neuromorphic hardware, and impossibility claims. Mechanism evidence is summarized as richness × addressability (Figure M; Section 3.1 / 3.4).

---

## 2. Methods

### 2.1 Matched dense-LIF control

Matched-arch protocols fix the SurrogateLifReference / dense-LIF forward and vary only the learner. Protocol v4 compares production-style **broadcast ±1 three-factor** updates to SuperSpike BPTT. Protocol v5 evaluates graded error with fixed-random DFA feedback. Protocol v12 evaluates directional REINFORCE × frozen per-neuron `B_i` as the primary arm. **Protocol v130 (`track-b-rescue`)** is a matched schedule ID (not a `c1-*-<hex>` config hash) evaluating continuous RPE critic scalar broadcast (`MatchedRlRpe`) vs online learned feedback alignment (`MatchedRlLearnedFb`, $B_i \leftarrow B_i + \eta_B r (a_i - p_i) x_i$). Protocol v28 (`c1-eventprop-5bb083d5e88d0ad2`) is a discrete EventProp-style spike-adjoint H2H vs SuperSpike on the same matched forward. Gates reuse Gate G2 numeric thresholds (accuracy floor 0.65; gap LCB > 0.5) under fresh hash families that do not reopen `c1-118207fbc3eaba53`.

### 2.2 Engine C1 / Gate G2

The C1 harness encodes coincidence sequences with a latency encoder, integrates with muted hidden thresholds on the canonical path, selects winners by membrane-score k-WTA, force-fires winners and readouts, and applies three-factor plasticity. Gradient and eligibility references train on the same frozen splits. Live transfer of matched credit families onto this substrate is the v13–v24 package. **Protocol v131 (`live-transfer-rescue`)** is a misnamed **matched-only** online-FB schedule contrast (no Engine / no muted-θ / no live k-WTA) and is **not** a live-transfer result.

---

## 3. Results

### 3.1 Matched-architecture primary results

Broadcast ±1 three-factor fails the matched gate at chance (0.5000) with gap LCB 0.0000 while the SuperSpike BPTT gradient ceiling learns (0.8963). Graded DFA passes (0.9387, gap LCB 0.6894). REINFORCE × frozen feedback passes as primary (0.9200, gap LCB 0.6846). **Online learned feedback alignment (v130 schedule) reaches 1.0000 mean accuracy with 0.9988 95% LCB gap closed (PASS matched)**, recovering the gradient ceiling on dense-LIF. Continuous scalar RPE broadcast remains at chance (0.5120, LCB -0.0230 FAIL), confirming that continuous magnitude without spatial directionality is information-theoretically insufficient on this gate. Discrete EventProp-style spike-adjoint H2H **FAIL**s at chance (0.5000, gap LCB 0.0000) against SuperSpike 0.9150 (`c1-eventprop-5bb083d5e88d0ad2`). The v131 `live-transfer-rescue` binary is matched-only (online-FB schedule contrast; mean 1.0000 / LCB 0.9983 on dense-LIF) and **does not** constitute live k-WTA transfer.

Honesty note required in the main text: on the DFA matched schedule, a broadcast-graded contrast also reaches high accuracy (0.9863). The lead negative therefore concerns **broadcast ±1 three-factor** credit, not the claim that every broadcast scalar fails coincidence. Locality as a necessary ingredient is evidenced on XOR (Section 3.4), not by coincidence DFA alone. Figure M plots this as a richness × addressability mechanism panel (including the 0.9863 cell and the XOR locality flip).

### 3.2 Engine C1 secondary results

Canonical C1 fails Gate G2 (local 0.4912, gap LCB −0.0048). Trial isolation and temporal positive-control sensitivities fail without clearing the accuracy floor. Capacity sensitivity clears the accuracy floor (0.6775) but leaves gap LCB at 0.0000 versus dense/gradient references—descriptive only, not a G2 PASS.

### 3.3 Live transfer, gap-close, and break-it

Live opt-in REINFORCE feedback fails G2 (local 0.4900, gap LCB 0.0737). Epoch matching alone does not rescue random-B live RFB (local 0.4838). Structured frozen B clears the accuracy floor (0.7262) with gap LCB 0.2567 but still fails the gap bar. Stacking epochs under structured B regresses (0.5200). Structured B on a capacity substrate yields the best gap LCB in the prior suite (0.3127) while clearing the floor (0.6825), remaining short of 0.5. Eligibility×REINFORCE and restored target teach clear the floor but do not beat structured B alone. Break-it protocols close remaining differentials without remassage: live graded DFA (v20) reaches local 0.7325 with gate LCB 0.2601 and chance LCB 0.3321—floor yes, gate no. Soft-WTA × structured B (v21) regresses to chance (0.5025). Matched three-factor under 4× epochs (v22) stays at 0.5000. Finite-θ under SFB (v23) clears the floor (0.6638) with LCB 0.2370. Continuous structured B (v24) does not beat sign-truncated v15 (0.6437 / 0.1380). Spiking-path true DFA rescue fails in one honest attempt (0.6513, gap LCB 0.0733). Differential closure: [`DIFF_CLOSURE.md`](DIFF_CLOSURE.md).

### 3.4 Task evidence

On one-layer `xor_thresh`, broadcast error stays at chance (0.5008) while DFA reaches 0.8267 against gradient 0.7733—a locality flip. On mid-init two-layer depth locality, broadcast also succeeds (0.8158) alongside DFA (0.8250) and REINFORCE×B (0.8033); depth help is not treated as a locality-flip claim.

---

## 4. Discussion and limitations

### 4.1 Lead claim and mechanism

The cleanest publishable claim is rule-topological: under a fixed dense-LIF forward, **broadcast ±1 three-factor** credit does not close the preregistered gap to SuperSpike BPTT, while graded DFA and REINFORCE×frozen-`B` do. That contrast supports modulator richness and feedback addressability as material factors on this gate (Figure M), without equating matched success to live sparse/k-WTA success.

Richness alone is not “locality”: on the DFA matched schedule, broadcast-*graded* also reaches 0.9863. The lead FAIL is therefore specifically ±1 × surrogate eligibility under broadcast, not an indiscriminate “broadcast credit topology” ban. Addressability / locality as a necessary ingredient is evidenced by the one-layer XOR locality flip (broadcast graded fails; DFA solves), not by coincidence DFA alone.

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

### 4.6 Synthetic primary and non-claims

The primary kill-gate is a synthetic coincidence discrimination task on matched dense-LIF and on the coded C1 pipeline. We do not claim a standard neuromorphic benchmark PASS/FAIL as the lead result. We do not claim cortical realism, Assembly Calculus PASS, neuromorphic deployment, or impossibility of local learning in principle. New integrity or credit hypotheses require new protocol versions and hashes; they must not reopen `c1-118207fbc3eaba53` by silent threshold change.

---

## 5. Reproducibility

Scientific hashes and commands are listed in [`REPRO_ARTIFACT_CHECKLIST.md`](REPRO_ARTIFACT_CHECKLIST.md) and [`PAPER_RESULTS_TABLE.md`](PAPER_RESULTS_TABLE.md). Rebuild with `cargo test --locked --workspace` from `binn/`. Camera-ready citations must point at on-disk notes or exact `--config-hash` replays. No new experiment hashes are introduced by this MUST packaging pass.

---

## Appendix A — Integrity limitations (protocol v2)

Cross-trial STDP pairing times (`ThreeFactor.last_spike`) are retained on canonical C1 while eligibility traces are cleared. Membrane and dendritic reset is incomplete relative to the C3 production path. Hidden thresholds are set to infinity during the integrate window, suppressing natural spiking. Assembly Calculus `project` is not exercised on the canonical loop (it is wired under `c1-project*` and fails G2 there). Exact-forward arms labeled “e-prop/DFA” are hybrid eligibility × transported modulators unless the true σ′ e-prop family is cited explicitly.

## Appendix B — Post-G2 harvest (banner)

G3 / G4 / hybrid H0 numbers and hashes: [`APPENDIX_POST_G2.md`](APPENDIX_POST_G2.md). Mechanism figure cells: [`PAPER_FIGURE_SPEC.md`](PAPER_FIGURE_SPEC.md) Figure M. These rows do not reopen G2.
