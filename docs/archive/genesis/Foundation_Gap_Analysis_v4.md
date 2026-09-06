# Why Brain-Inspired Neural Networks Lag ANNs — the complete gap map, and how to close it

## v4 — the gaps v1–v3 missed

**Date:** 22 July 2026
**Builds on:** v2 (assembly + local-learning substrate), v3 (scaling)
**Purpose:** an honest, complete taxonomy of *why* brain-inspired neural networks (BINNs / spiking nets) underperform ANNs, with a mitigation for each — and a blunt marking of which gaps my earlier documents already address, which they silently inherit, and which are new gaps I had not surfaced.

> **Note.** The gap *taxonomy* below holds regardless of our specific bet — it is about the field. The *mitigations* tied to our substrate are contingent on the v2 crux (G2) passing; where they are, they inherit v2's likely-negative prior.

---

## 0. Admission first

v1–v3 concentrated on two gaps: credit assignment (v2) and the scaling law (v3). Those are real and central, but they are not the whole story. Checked against the current literature, BINNs lag for **at least six families** of reasons, and I had only seriously treated one and a half of them. The most important omission is a training-throughput gap that is the direct analog of why RNNs lost to transformers — and it is arguably a bigger practical reason BINNs lag than credit assignment is. This document is the correction.

A running tag on every gap:
- **[DODGED]** — the v2/v3 approach structurally avoids this.
- **[INHERITED]** — v2/v3 is still fully exposed; I had not addressed it.
- **[NEW]** — a gap I had not surfaced at all in v1–v3.

---

## 1. The six families of why BINNs lag

### Family A — Learning & credit assignment

This is the one v2 tackled, but it has sub-gaps I had folded together too quickly:

- **A1. Non-differentiability of spikes. [INHERITED at the baseline level]** A spike is a step function; its derivative is zero almost everywhere and infinite at threshold. Backprop cannot flow through it. The field's workaround — *surrogate gradients* (pretend the spike has a smooth derivative) — works but is "less mature and efficient" than ANN training and is philosophically a patch. *Mitigation:* v2's local three-factor rule sidesteps differentiability entirely — but only if it actually learns (the unproven bet). So this is dodged in principle, inherited in practice until the crux experiment passes.
- **A2. Dead-neuron problem & training instability. [INHERITED]** With surrogate gradients, neurons that stop firing get no gradient and never recover; training needs careful regularization. *Mitigation:* homeostatic adaptive thresholds (v2's Pillar I) keep neurons in their dynamic range and directly attack this — one place the biology genuinely helps.
- **A3. STDP/local rules decay with depth. [PARTIALLY ADDRESSED]** Pure STDP "diminishes in deeper networks." This is exactly the plateau v2's sparse-assembly bet is designed to escape; it remains the make-or-break unknown.
- **A4. Long-delay temporal credit. [INHERITED]** Assigning credit for a reward that arrives long after the responsible spikes is hard; eligibility traces help but have limited horizon. *Mitigation:* longer, multi-timescale eligibility traces + neuromodulated consolidation; still an open problem at long horizons.

### Family B — Training throughput & temporal parallelism *(the big miss)*

- **B1. Sequential time-unrolling. [NEW — the omission that matters most]** ANNs on GPUs parallelize across the entire sequence and batch at once. A spiking net has *recurrent membrane state*, so to train it with backprop-through-time you must unroll it step by step through time — inherently sequential, exactly like an RNN. This is a major reason SNN training is "slow (as time is sequentially simulated)." **It is the same structural weakness that made RNNs lose to transformers**, and I never named it. *Mitigation (three routes):* (i) *online local learning* — v2's rule needs no backward unroll and is O(1) in *memory* over sequence length, so it removes the **backward-unroll half** of B1 **[HALF-DODGES]**; but it does **not** remove the sequential *forward* simulation — events are still processed in time order, so single-stream throughput stays time-sequential; (ii) therefore *parallelize across neurons, areas, and independent streams* instead of across time — this is the real throughput lever; (iii) the transformer-style *parallel-in-time* trick (associative-scan recurrences, as in state-space models) helps **only partially**: the sub-threshold membrane dynamics are linear and scan-parallelizable, but the **hard reset after a spike introduces a sequential, data-dependent barrier that provably blocks full parallelization**. Practical options are chunked scans (parallel within a chunk, sequential across chunks) or removing the reset entirely (which discards the spike-reset dynamics). Treat parallel-in-time as a partial optimization with a modeling trade-off, not a clean fix.
- **B2. No batch parallelism in the brain. [NEW]** Biology learns from one stream, online; GPUs love big batches. A faithful BINN forfeits the batch-size lever that makes ANN training throughput-efficient. *Mitigation:* population/area parallelism and many parallel environments (as in RL) substitute for batch; accept that the win is sample-efficiency, not throughput.
- **B3. Latency vs. accuracy trade-off. [INHERITED]** Rate-coded SNNs need many time steps to average out spikes, adding latency; temporal codes cut steps but are harder to train. *Mitigation:* commit to temporal/few-spike codes (v2's assemblies are already sparse/temporal), accept the training difficulty as the price.

### Family C — Ecosystem, tooling & talent

- **C1. A decade of PyTorch/CUDA vs. immature neuromorphic stacks. [INHERITED — I ignored this]** Intel's Lava and IBM's toolchains are "functional but immature relative to PyTorch and CUDA," which have a decade of investment and millions of practitioners. The BINN talent pool is "measured in the hundreds globally." This is a compounding disadvantage independent of any science. *Mitigation:* do **not** build a new framework from zero — build `neura-core` with first-class Python/`pyo3` bindings, interoperate with PyTorch tensors, and reuse existing autodiff for the *baseline* comparisons. Meet the ecosystem where it is.
- **C2. Reproducibility & simulator dependence. [INHERITED]** Results vary by simulator, time-step, and encoding; hard to compare. *Mitigation:* deterministic seeded engine (already in v2's plan), fixed time-base, published configs — non-negotiable.
- **C3. No pretrained backbones / foundation models. [NEW]** ANNs stand on ImageNet/LLM pretrained weights; every BINN project starts from scratch. This alone can account for a large accuracy gap that has nothing to do with the substrate. *Mitigation:* build reusable pretrained assembly "cortex" modules that downstream tasks fine-tune; and allow ANN-pretrained features to *initialize* areas where honest (label it clearly).

### Family D — Data & the encoding boundary

- **D1. No ImageNet-scale event datasets. [NEW]** Neuromorphic work leans on essentially one dataset ecosystem (Tonic) and mostly validates on MNIST/CIFAR; ImageNet-scale event data is rare. Chicken-and-egg: no data → weak algorithms → no incentive to make data. *Mitigation:* generate large synthetic event streams; convert existing large corpora to spikes with a fixed public encoder; pick *natively event* domains (audio, DVS vision, sensor/robotics, finance ticks) where the data is already temporal and abundant.
- **D2. The encode/decode boundary is lossy and unstandardized. [NEW — a subtle, important one]** Real data is not spikes; you must *encode* it into spikes and *decode* spikes into answers. There is "no universal spike encoding standard" (rate vs. temporal vs. phase), and a bad encoder throws away information before the network ever sees it. Much of the measured BINN gap lives in this boundary, not in the network. *Mitigation:* treat the encoder/decoder as *learned, first-class parts of the model*, co-trained with the substrate; standardize on one encoding within the project and measure encoder-induced loss explicitly.

### Family E — Benchmarking honesty & missing theory

- **E1. Toy-benchmark wins don't transfer. [INHERITED]** Many "SNN beats ANN" results are on small or DVS-specific datasets; on hard tasks ANNs still win. *Mitigation:* always report against a *matched, well-tuned* ANN on the *same* task and *work-per-accuracy* — the discipline v2/v3 already commit to.
- **E2. No theory of the spiking advantage. [NEW]** There is still no crisp account of *what temporal/spiking computation buys you* that rates cannot, for general tasks. Without theory, BINNs are a bag of tricks and every gain looks incidental. *Mitigation:* this is where v2's Assembly Calculus is a genuine asset — it is one of the few BINN frameworks with *convergence and computational-power theorems*. But be precise about their reach: those results cover assembly *convergence*, *bounded* computation, and classification of **well-separated** classes — not competitive learning on hard, not-well-separated, compositional tasks (and the framework has known expressiveness limits, e.g. an AC parser is weaker than a finite automaton). So lead with it to ground the representation, and make the claim (sparse assemblies → low interference → sufficient local credit) explicit and testable — but treat it as an *open hypothesis the theory motivates*, not one the theory proves.
- **E3. The ANN-to-SNN conversion trap. [DODGED]** Much of the field trains an ANN then converts it to spikes, which inherits every ANN design assumption and never explores native spiking computation. *Mitigation:* v2 is natively spiking/assembly-based and never converts — this trap is structurally avoided.

### Family F — Economics, hardware non-idealities, connectivity

- **F1. Transformer capital gravity. [PARTIALLY ADDRESSED in v3]** The workloads where neuromorphic wins ("sparse temporal signals, event-driven sensors, certain optimization") are "real but niche relative to transformer-dominated workloads." Investment, and therefore progress, follows the transformer. *Mitigation:* v3's answer — aim at the brain/embodied-agent target where these are the *native* strengths, not at LLM turf.
- **F2. Analog hardware non-idealities. [NEW]** On analog/in-memory neuromorphic chips, device mismatch, noise, and limited precision degrade results; algorithms tuned in simulation break on silicon. *Mitigation:* train with injected noise/quantization (noise-aware training); keep the algorithm robust by design (sparse codes are naturally noise-tolerant).
- **F3. Memory & connectivity constraints. [ADDRESSED in v3]** Fan-out and on-chip memory limit how densely areas can wire. *Mitigation:* v3's wiring prior + event-locality (>90% intra-area) directly targets this.

---

## 2. The scoreboard — what I had and hadn't covered

| Gap | Tag | Covered before v4? |
|---|---|---|
| A1 non-differentiability | dodged-in-principle | implied, not named |
| A2 dead neurons | addressed | implied (homeostasis) |
| A3 depth decay of local rules | the core bet | yes (v2) |
| A4 long-delay credit | inherited | partially |
| **B1 sequential time-unrolling** | **new — biggest miss** | **no** |
| B2 no batch parallelism | new | no |
| B3 latency/accuracy | inherited | partially |
| C1 tooling/ecosystem | inherited | no |
| C2 reproducibility | addressed | yes |
| **C3 no pretrained backbones** | **new** | **no** |
| **D1 no large event data** | **new** | **no** |
| **D2 encode/decode boundary** | **new — subtle** | **no** |
| E1 benchmark honesty | addressed | yes |
| **E2 no theory of advantage** | **new** | partially (assembly theory) |
| E3 conversion trap | dodged | yes |
| F1 economics | addressed | yes (v3) |
| **F2 hardware non-idealities** | **new** | **no** |
| F3 connectivity | addressed | yes (v3) |

Seven genuinely new or unaddressed gaps (B1, B2, C1, C3, D1, D2, E2, F2). So — to answer your question directly — **no, I had not thought of all the gaps.** The three that most change the plan are below.

---

## 3. The three gaps that actually decide it

### #1 — Training throughput / temporal parallelism (B1)

The deepest practical reason BINNs lag is not that they can't learn — it's that they learn *slowly*, because recurrent state forces sequential simulation, the same wall RNNs hit. **Mitigation strategy (this is the important one):**

- For **learning**, commit to *online local plasticity* (v2), which needs no backward unroll and is O(1) in *memory* over sequence length. This removes the backward-unroll half of B1 — a real advantage *if* the scaling bet holds — but note it does **not** speed up the sequential forward simulation.
- The **primary throughput lever** is therefore parallelism across **neurons, areas, and independent streams**, never across time within a single stream.
- The state-space-model trick (associative-scan recurrences) is a **partial** help only: sub-threshold dynamics parallelize, but the hard reset is a sequential barrier that blocks full parallelization. Use chunked scans (or reset-removal, at a modeling cost) where worthwhile; do not count on it to rescue throughput.

### #2 — The encode/decode boundary (D2)

A large chunk of the measured gap is information lost turning data into spikes and spikes into answers, not the network's fault. **Mitigation:** treat the encoder and decoder as *first-class, measured components* — but resolve the tension with the no-autodiff rule explicitly: use **fixed, information-preserving encoders/decoders through the crux experiments** (with an info-loss meter), and defer any *learned* encoder to post-crux, where it must either train via the same local rule or be an explicitly-labeled autodiff island. Choose natively-temporal domains (audio, event-vision, sensor, market microstructure) where encoding is nearly free. Do not benchmark on statically-encoded MNIST and conclude anything about the substrate.

### #3 — Ecosystem & foundation-model gravity (C1 + C3)

Even a perfect algorithm loses to a decade of PyTorch/CUDA tooling and pretrained backbones. **Mitigation:** interoperate rather than replace — Python bindings, PyTorch-tensor I/O, reuse autodiff for baselines only; build *reusable pretrained assembly modules* so projects don't start from zero; and accept that closing this gap is as much a community/tooling effort as a scientific one.

---

## 4. What this changes about the program

Nothing in v2/v3 is invalidated, but the plan gets three additions and one reframing:

1. **Add a *partial* parallel-in-time path** (chunked associative-scan over the linear sub-threshold dynamics; the reset stays sequential) alongside online local learning, and rely mainly on neuron/area/stream parallelism for throughput (mitigates the tractable part of B1/B2).
2. **Keep encoders/decoders fixed and information-preserving through the crux** (measured, not learned), deferring learned encoders to post-crux; pick natively-temporal data (mitigates D1/D2 without contaminating the crux).
3. **Build on the existing ecosystem** — bindings and interop, plus reusable pretrained assembly backbones (mitigates C1/C3).
4. **Reframing:** the honest competitive claim is *not* "BINNs will beat ANNs on accuracy." It is "on natively-temporal, continual, energy-constrained tasks, a sparse-assembly substrate with online local learning wins on work-per-task and lifelong adaptation — and that is a different market, not a worse chatbot." Every gap above is either mitigated on that turf or irrelevant to it.

---

## 5. Blunt bottom line

BINNs lag ANNs for a **stack** of reasons, only some of which are about the brain-inspired algorithm itself. Ranked by how much they actually explain the gap: (1) slow sequential training from recurrent state — the RNN-vs-transformer wall; (2) the lossy, unstandardized spike-encoding boundary; (3) a decade of tooling, data, and pretrained-model advantage the ANN world has and the BINN world doesn't; (4) local credit assignment decaying with depth; (5) benchmark and theory immaturity; (6) hardware non-idealities and economics. My earlier documents seriously engaged only #4. The mitigations exist for every one of them, but several are *ecosystem and engineering* problems, not scientific ones — which means the fastest way to close the gap is often to stop trying to beat ANNs on their turf and metrics, and compete where the substrate's structural advantages are the scoring function.
