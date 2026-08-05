# Convergence and Divergence Between Artificial Neural Networks and the Biological Human Brain

## A Comparative Literature Review of Mechanisms, Architecture, and Offline Cognition

**Author role:** Principal Computational Neuroscientist & AI Architect (synthesis report)  
**Date:** 22 July 2026  
**Primary anchor paper:** Behrouz, A., Hashemi, F., Javanmard, A., & Mirrokni, V. (2026). *Language Models Need Sleep: Learning to Self-Modify and Consolidate Memories.* arXiv:2606.03979  
**Document:** `Grok_ANN_Report.md`

---

## Abstract

Artificial neural networks (ANNs) and the human brain share a family resemblance: hierarchical representation, massive parallelism, and experience-dependent change. That resemblance is often overstated. This review evaluates **convergence** and **divergence** between contemporary deep learning systems—especially large language models (LLMs)—and biological intelligence along three axes: (1) **mechanisms** of learning, memory consolidation, and forgetting; (2) **architectural specialization**, from cortical modularity to Mixture-of-Experts (MoE) systems; and (3) **offline processing**, including sleep-dependent consolidation, dream-like generative rehearsal, and resting-state / Default Mode Network (DMN) dynamics. We integrate classical complementary learning systems theory with modern continual-learning and brain-inspired replay literature, and critically appraise the recent *Sleep* paradigm for LLMs (Behrouz et al., 2026), which operationalizes wake/sleep cycles, upward knowledge distillation (“Knowledge Seeding”), and reinforcement-learned “Dreaming.”

The central thesis is that ANNs and brains exhibit **partial computational homology**—especially regarding hierarchical features, modular conditional computation, and the need for offline reorganization to manage the stability–plasticity dilemma—while remaining **mechanistically divergent** in credit assignment, energy-efficient sparse dynamics, developmental embodiment, and multi-state cognition. Next-generation brain-inspired architectures should treat multi-timescale memory, adaptive capacity, structured forgetting, and offline generative consolidation as first-class design primitives, without equating engineering metaphors with biological identity.

**Keywords:** artificial neural networks; complementary learning systems; synaptic plasticity; backpropagation; experience replay; sleep consolidation; catastrophic forgetting; Mixture of Experts; Default Mode Network; continual learning; brain-inspired AI

---

## 1. Introduction

### 1.1 Why the comparison still matters

Deep learning’s practical success—vision, speech, language, scientific discovery—has reopened an old question: are modern ANNs merely loosely “brain-inspired,” or do they capture computational principles of biological intelligence (Kriegeskorte, 2015; Richards et al., 2019; Sejnowski, 2020)? The answer depends on the level of analysis. At the level of *task performance*, ANNs can match or exceed humans in narrow domains. At the level of *algorithms*, some problems (hierarchical credit assignment, interference across sequential tasks, abstraction from sparse experience) appear shared. At the level of *implementation*, spikes, neuromodulators, glial support, embodiment, and sleep architecture have no direct counterpart in standard transformers trained with backpropagation.

This report adopts a deliberate taxonomy for every claimed parallel:

| Label | Meaning | Example |
|-------|---------|---------|
| **Functional analogy** | Same problem, different substrate | Replay reduces interference in both systems |
| **Computational homology** | Similar algorithm-level structure | Fast episodic store + slow semantic store |
| **Mechanistic identity** | Shared physical mechanism | Rare for silicon ANNs vs. wetware |

Confusing these levels produces both hype (“models dream like humans”) and nihilism (“ANNs have nothing to do with brains”). Productive science sits between them (Marblestone, Wayne, & Körding, 2016).

### 1.2 Historical arc in brief

McCulloch–Pitts formal neurons, Rosenblatt’s perceptron, and connectionism established the idea that intelligence might emerge from networks of simple adaptive units. The deep learning revival scaled gradient-based optimization, differentiable architectures, and data, often *abandoning* biological constraints (non-local weight transport, dense matrix multiplies, train/test split). Neuroscience, in parallel, refined multi-timescale plasticity, systems consolidation, and large-scale network organization. The most durable theoretical bridge remains **Complementary Learning Systems (CLS)** (McClelland, McNaughton, & O’Reilly, 1995; O’Reilly et al., 2011): a fast hippocampal-like system for episodic encoding and a slow neocortical system for interleaved, structure-extracting learning.

### 1.3 Scope and the anchor paper

This review focuses on three user-specified axes and uses Behrouz et al. (2026), *Language Models Need Sleep*, as a contemporary engineering realization of sleep-inspired consolidation in LLMs. The paper argues that continual learners should abandon static train/test phases in favor of **wake** (active data interaction) and **sleep** (internal consolidation and self-improvement), with two sleep substages: **memory consolidation via Knowledge Seeding** and **Dreaming** via synthetic curricula optimized with reinforcement learning.

### 1.4 Guiding thesis

> ANNs and brains share partial computational homology on hierarchical representation, modular specialization, and offline consolidation under stability–plasticity pressure. They diverge mechanistically on credit assignment, sparse event-driven dynamics, development and embodiment, and multi-state (wake/sleep/rest) cognition. Sleep-inspired LLM methods, including Behrouz et al. (2026), are best read as **functional designs informed by neuroscience**, not as biological models of sleep or dreaming.

---

## 2. Axis I — Biological vs. Artificial Mechanisms

### 2.1 Synaptic plasticity versus backpropagation

#### 2.1.1 Biological learning rules

Cortical synapses change under local conditions: pre- and postsynaptic activity, dendritic voltage, and neuromodulatory third factors that gate plasticity according to reward, surprise, or attention (Roelfsema & Holtmaat, 2018). Spike-timing-dependent plasticity (STDP) and related rules couple timing to potentiation or depression. Importantly, biological networks must assign credit across many layers and long delays without an obvious implementation of reverse-mode automatic differentiation through a global computational graph.

#### 2.1.2 Backpropagation and its biological criticisms

Backpropagation (BP) multiplies error signals by the transpose of feedforward weights—the classic **weight transport** problem. Cortical feedback pathways exist but are not simple weight-symmetric copies of feedforward connections. BP also typically assumes continuous activations, synchronized phases of forward and backward pass, and non-local storage of intermediate activations.

#### 2.1.3 Bridging algorithms

Research has narrowed—but not closed—the gap:

- **Feedback alignment** shows that fixed random feedback weights can support useful deep learning, relaxing exact symmetry (Lillicrap et al., 2016).
- **Equilibrium Propagation** unifies inference and learning in energy-based models with a second nudged phase (Scellier & Bengio, 2017).
- **e-prop** and related eligibility-trace methods approach credit assignment in spiking recurrent networks (Bellec et al., 2020).
- Spiking deep learning surveys document both progress and remaining performance/efficiency trade-offs (Pfeiffer & Pfeil, 2018; Eshraghian et al., 2023).

**Convergence:** Hierarchical representation learning and credit assignment are shared *problems*; approximate nonlocal error signals may be computable without exact BP.

**Divergence:** Production ANNs still rely on reverse-mode AD at scale. Biological learning is continuous, online, sparse, and metabolically constrained. Mechanistic identity has not been established.

### 2.2 Memory consolidation: hippocampal replay versus experience and generative replay

#### 2.2.1 Biological systems consolidation

CLS theory posits that the hippocampus rapidly encodes episodes while neocortex integrates structure through interleaved learning (McClelland et al., 1995; O’Reilly et al., 2011). Sleep is not passive downtime: it is a structured brain state for systems consolidation (Born & Wilhelm, 2011; Brodt et al., 2023).

Key physiological ingredients include:

- **Sharp wave–ripples (SWRs)** in hippocampus as highly synchronous “offline” events supporting replay and planning-related reactivation (Buzsáki, 2015).
- **Nested coupling** of slow oscillations, spindles, and ripples coordinating hippocampal–neocortical dialogue (Staresina et al., 2015).
- **Selective prioritization**: human hippocampal replay during rest can favor weakly learned information and predict later performance (Schapiro et al., 2018).
- **Cascaded systems** linking replay to DMN-related reorganization (Kaefer et al., 2022).

NREM (especially slow-wave) sleep is strongly associated with declarative consolidation and synaptic renormalization hypotheses; REM is more often linked to integration with existing semantic/emotional networks and flexible recombination—though dream content ↔ consolidation links remain mixed (Bloxham & Horton, 2024).

#### 2.2.2 Artificial replay families

In machine learning, **experience replay** stores past samples and interleaves them with new data (canonical in deep RL). **Generative replay** trains a generator to sample past task distributions without storing raw exemplars—closer, at a functional level, to abstractive biological replay (van de Ven, Siegelmann, & Tolias, 2020). **Sleep-like unsupervised replay** can reduce catastrophic forgetting by offline reactivation dynamics (Tadros et al., 2022).

| Process | Biology | ANN / LLM analog | Homology strength |
|---------|---------|------------------|-------------------|
| Rapid encoding | Hippocampal episodic binding | In-context learning; short-term activations / KV cache | Functional–computational |
| Offline reactivation | SWR replay, nested oscillations | Experience / generative replay; sleep-time compute | Computational (partial) |
| Transfer to long-term store | Systems consolidation to neocortex | Distillation into weights; adapters; slow modules | Functional–computational |
| Abstraction, not raw copy | Schema extraction during sleep | Generative replay; self-distillation | Functional |
| Novelty recombination | REM-associated integration (hypothesized) | Synthetic data / “dreaming” curricula | Metaphorical–functional |

#### 2.2.3 Deep dive: Behrouz et al. (2026) — *Language Models Need Sleep*

**Problem framing.** LLMs excel at in-context learning but struggle to transfer fragile session knowledge into durable parameters without expensive retraining or catastrophic forgetting. Behrouz et al. analogize this to **anterograde amnesia**: intact remote knowledge (pretraining) plus intact short-term context, but impaired formation of new long-term memories after “end of pretraining.”

**Lifecycle redesign.** Rather than train vs. test, a continual learner alternates:

1. **Wake / active time** — receive and process external data; high-frequency modules update.
2. **Sleep time** — little or no external data; consolidate and self-improve.

This sits on a **Continuum Memory System (CMS)** / Nested Learning view: components update at different frequencies, from attention-like short-term memory to near-static long-term MLPs, with intermediate bands.

**Sleep stage 1 — Memory consolidation (NREM-inspired).**

- **Parameter expansion:** before consolidating a faster block into a slower one, add capacity (e.g., new low-rank experts in sparse MoE blocks) so new knowledge does not overwrite old parameters.
- **Knowledge Seeding (upward distillation):** a *smaller* model state (teacher) distills into a *larger* expanded student—opposite the usual large→small compression story. Only newly expanded parameters are trained, protecting prior knowledge.
- **Generalized Distillation + Learning to Imitate:** combine on-policy distillation with RL rewards for semantic and token-level imitation of teacher continuations.
- **Reset / prune:** after consolidation, reset high-frequency low-rank parameters to free capacity—explicitly compared to **synaptic pruning**.

**Sleep stage 2 — Dreaming (REM-inspired self-modification).**

- Generate synthetic “dreams” conditioned on task context.
- Inject randomness via MoE routers (sample extra random experts) to explore beyond the model’s habitual knowledge.
- Score dreams by gradient importance; keep top-*k* plus diversity samples.
- Inner-loop SFT (e.g., LoRA) on each dream; outer-loop RL (ReST-style) rewards dreams that improve downstream performance—building on SEAL-like self-edit ideas while addressing CF risk via prior consolidation.

**Empirical claims (as reported).** Gains on class-incremental text classification (CLINC, Banking, DBpedia) vs. ICL, EWC, InCA, and multi-level Hope without explicit distillation; improved long-context behaviors (RULER MK-NIAH, LongHealth, QASPER) with more consolidation stages; better sequential learning of novel languages; strong BABILong scaling in their setup; knowledge incorporation and few-shot ARC improvements when full sleep (consolidation + dreaming) is used.

**Critical appraisal.**

| Strength | Limitation |
|----------|------------|
| Makes offline consolidation a first-class lifecycle, not an afterthought | “Sleep/Dream” are engineering metaphors, not physiological models |
| Upward distillation + frozen old params is a principled anti-interference move | Capacity growth and sleep schedules are still largely designed, not autonomously homeostatic |
| Generative dreaming aims at abstraction and self-curriculum | Inner-loop SFT remains costly; scale to frontier models is open |
| Explicit pruning/reset of temporary adapters | Biological pruning is activity-, neuromodulator-, and development-dependent |
| Aligns with CLS and generative replay traditions | Limited joint validation against neural sleep markers or human memory experiments |
| Rejects pure ICL as sufficient long-term learning | Still gradient-based, transformer-centric substrate |

**Judgment:** Behrouz et al. (2026) is among the most complete *systems* statements of sleep-inspired continual learning for LLMs to date—**computational homology** with systems consolidation is real; **mechanistic identity** with NREM/REM is not.

### 2.3 Catastrophic forgetting versus active forgetting and synaptic pruning

#### 2.3.1 The ANN failure mode

When parameters are shared and updates for task *B* overwrite features needed for task *A*, networks exhibit **catastrophic forgetting** (historical literature from McCloskey & Cohen; French). Continual learning surveys document regularization (EWC), replay, dynamic architecture, and isolation strategies (Wang et al., 2023; Zhou et al., 2023). Context-dependent gating plus synaptic stabilization can alleviate forgetting in ways loosely inspired by neural circuit mechanisms (Masse, Grant, & Freedman, 2018). A deeper recent finding is **loss of plasticity**: even when old performance is protected, deep networks can become hard to train on new tasks over long continual regimes (Dohare et al., 2024).

#### 2.3.2 Biological forgetting is often adaptive

Brains forget. Active forgetting, synaptic downscaling during sleep (synaptic homeostasis hypothesis), and developmental/adult pruning remove redundant or weak synapses, preserve metabolic budgets, and improve signal-to-noise. Forgetting can be **goal-directed**, not merely a bug.

#### 2.3.3 Comparative synthesis

| Dimension | Biological forgetting | ANN catastrophic forgetting |
|-----------|----------------------|----------------------------|
| Typical role | Adaptive resource management | Uncontrolled interference |
| Timing | Sleep, development, neuromodulation | Online gradient steps on new tasks |
| Selectivity | Often use-dependent / tagged | Often global, non-selective |
| Desired analog in AI | Structured pruning, utility-weighted retention | Stability without freezing all plasticity |

Behrouz et al.’s reset of consolidated high-frequency adapters is closer to **adaptive capacity recycling** than to pure EWC-style protection. Still, most production LLMs lack principled active-forgetting objectives.

**Section verdict (Axis I):** Strongest homology lies in *two-stage memory* and *replay/consolidation* as solutions to interference. Strongest divergence remains *how* synapses/parameters change (local multi-factor plasticity vs. BP) and *why* forgetting occurs (adaptive vs. accidental).

---

## 3. Axis II — Architectural Specialization

### 3.1 Biological modularity and cortical organization

The cortex exhibits multi-scale modularity:

- **Microcircuit motifs** and columnar organization (classic Mountcastle program; modern literature debates whether the “column” is a universal computational atom versus a useful descriptive scale).
- **Large-scale modular networks** measurable with graph-theoretic tools on structural and functional connectomes (Sporns & Betzel, 2015).
- **Macroscale gradients** from unimodal sensorimotor cortex to transmodal association cortex, with the DMN near the abstract pole (Margulies et al., 2016).
- **Hub architecture** integrating specialized systems (Buckner et al., 2009; Seeley et al., 2007 for salience vs. executive control dissociation).

Specialization coexists with flexible recombination: the same regions participate in multiple coalitions depending on task and state.

### 3.2 Artificial modularity: modular nets and Mixture of Experts

Engineering modularity evolved under capacity and compute constraints:

- **Modular and progressive networks** allocate new modules for new tasks, reducing interference.
- **Hard attention to task** and related isolation mechanisms (Serrà et al., 2018).
- **Sparsely gated Mixture of Experts** dramatically increases parameter capacity while activating only a subset of experts per token/example (Shazeer et al., 2017), extended in large language and multimodal systems (e.g., GLaM; DeepSeekMoE, Dai et al., 2024).
- Interpretive work suggesting transformer FFNs behave like mixtures of specialized keys (Zhang et al., 2022).
- Broader **cognitive architectures** literature (Kotseruba & Tsotsos, 2018) situates modular ANNs within decades of systems AI.

Behrouz et al. (2026) explicitly use **sparse MoE blocks within CMS**, expanding experts during sleep for consolidation targets—linking modular capacity growth to offline learning.

### 3.3 Comparative table: cortex vs. modular ANNs

| Biological property | Closest ANN construct | Fidelity |
|---------------------|----------------------|----------|
| Sparse, mostly local connectivity | Sparse MoE; sparsity regularizers; neuromorphic graphs | Partial |
| Functional specialization | Expert specialization; multi-task MoE gates | Functional analogy |
| Dynamic coalition formation | Attention + routers | Partial |
| Developmental wiring & myelination | Progressive nets; curriculum; parameter expansion | Weak–moderate |
| Thalamocortical gating | Input routing / early fusion modules | Metaphorical |
| Metabolic cost as hard constraint | FLOPs/latency objectives; mixture sparsity | Partial (different physics) |
| Embodiment-shaped modules | Multimodal transformers; robot policies | Early / partial |

### 3.4 Convergence and divergence

**Convergence:** Conditional computation is a shared *principle*: not all capacity should fire for every input. Both systems benefit from specialization plus integrative pathways.

**Divergence:** Biological modules emerge through evolution, development, and embodied interaction with energy limits. MoE modules are optimized for next-token or task losses under hardware kernels. Cortical “routing” is continuous, recurrent, and state-dependent (arousal, neuromodulators); ANN routers are typically discrete top-*k* selections trained with load-balancing auxiliaries.

**Section verdict (Axis II):** MoE and modular nets are the best current **architectural** analogs of biological specialization, but they capture *conditional capacity use*, not developmental cortical ontogeny.

---

## 4. Axis III — Subconscious Processing, Dream-like Consolidation, and Resting-State Networks

### 4.1 Resting-state networks and the DMN

The **Default Mode Network**—medial prefrontal, posterior cingulate/precuneus, angular gyrus, and related regions—shows elevated activity during rest and self-referential cognition, and is typically suppressed during externally directed tasks (Raichle lineage; Sheline et al., 2009; Utevsky et al., 2014). Resting-state fMRI reveals dynamic connectivity (Allen et al., 2012) and hub structure relevant to integration and disease (Buckner et al., 2009). The DMN sits toward the abstract end of cortical gradients (Margulies et al., 2016).

Functionally, DMN-related processes have been linked to autobiographical memory, prospection, social cognition, and constructive simulation—not to a single “unconscious module.” Replay research increasingly situates offline reactivation within cascaded systems involving DMN dynamics (Kaefer et al., 2022).

### 4.2 Do deep networks have a DMN?

Standard deep networks do **not** maintain endogenous resting-state dynamics when “idle.” Between training steps or user queries, a deployed LLM is typically inert: no spontaneous attractor itinerancy, no metabolic baseline, no self-generated thought stream.

Closest engineering analogs:

| Biological offline phenomenon | ANN / LLM analog | Notes |
|------------------------------|------------------|-------|
| Resting-state spontaneous activity | Rarely modeled; some generative RNNs / predictive coding nets | Not standard in production LLMs |
| DMN constructive simulation | Chain-of-thought, self-reflection prompts, world-model rollouts | Prompted, not endogenous |
| NREM consolidation | Replay buffers; sleep-time distillation (Behrouz 2026) | Strongest engineering parallel |
| REM recombination | Synthetic data dreaming; exploratory generation | Functional analogy only |
| Subconscious processing | Parallel shallow features; unattended activations | Rhetorical dual-process language is mostly metaphor |

### 4.3 Dreaming: science versus systems engineering

**Neuroscience.** Dreaming is most associated with REM but not exclusive to it. Links between dream *content* and overnight memory gains are inconsistent; methodological challenges are substantial (Bloxham & Horton, 2024). A safer claim is that sleep stages implement different computational modes—global downscaling and systems transfer (NREM-biased) versus integration and flexibility (REM-biased)—that *sometimes* correlate with dream reports.

**Machine learning.** “Dreaming” in ANNs usually means **generative rehearsal**: sampling from a model of past experience or latent structure to train without new external labels. Behrouz et al.’s Dreaming stage is explicit self-improvement: generate, filter, fine-tune, reinforce. This is closer to **curriculum synthesis and policy improvement** than to phenomenology.

### 4.4 Subconscious processing: a careful stance

Biological cognition includes parallel, preconscious, and automatic processes alongside reportable awareness. ANNs have layered distributed representations and can perform substantial computation not exposed as intermediate language—but this is **not** a validated model of human unconscious processing. Dual-process AI rhetoric should be treated as pedagogical metaphor unless tied to specific architectural claims (e.g., System-1 cached heuristics vs. System-2 search).

**Section verdict (Axis III):** Offline **consolidation and generative recombination** are the legitimate bridge. DMN and subconscious cognition currently lack strong mechanistic ANN counterparts; sleep-time compute is the most productive research path.

---

## 5. Integrated Comparison Matrices

### 5.1 Mechanisms at a glance

| Mechanism | Biology (state of knowledge) | ANN SOTA | Convergence | Open gap |
|-----------|------------------------------|----------|-------------|----------|
| Credit assignment | Local multi-factor plasticity; feedback pathways | Backprop; feedback alignment; EqProp; e-prop | Shared problem | Scalable local rules matching BP performance |
| Episodic binding | Hippocampal index / pattern separation | ICL, episodic memory modules, retrieval | Functional | True one-shot binding with controlled interference |
| Systems consolidation | Sleep SWRs, nested oscillations | Replay, generative replay, Knowledge Seeding | Computational homology | Homeostatic scheduling of consolidation |
| Generative recombination | REM / offline simulation (partially understood) | Dreaming / synthetic curricula | Functional | Grounded novelty without model collapse |
| Forgetting | Active, adaptive, pruned | Catastrophic interference; regularization | Weak | Utility-aware active forgetting |
| Plasticity maintenance | Lifelong, neuromodulated | Loss of plasticity in deep CL (Dohare 2024) | Problem shared | Recoverable plasticity at scale |

### 5.2 Architecture at a glance

| Feature | Cortex | Modular ANN / MoE | Notes |
|---------|--------|-------------------|-------|
| Specialization | High, multi-scale | Expert FFNs / modules | Analogy good for capacity |
| Integration | Hubs, gradients, thalamocortical loops | Attention, shared trunks | Partial |
| Sparsity | Extreme spike sparsity | Conditional expert activation | Different units (spikes vs. tokens) |
| Growth | Synaptogenesis, pruning | Progressive nets, sleep expansion | Behrouz links these |
| Energy objective | Hard biological constraint | Soft FLOPs/latency proxy | Physics diverges |

---

## 6. Fundamental Limitations of Current ANNs as Models of Biological Intelligence

The following limitations are structural, not merely temporary engineering deficits.

### 6.1 Learning algorithm gap

Global BP remains the workhorse. Biologically plausible alternatives work in restricted settings but have not displaced BP for frontier models. Without local learning, neuromorphic deployment and online lifelong adaptation remain strained.

### 6.2 Missing multi-state cognition as a first-class design

Brains cycle through wake, NREM, REM, and resting dynamics with distinct computational modes. Most ANNs have one mode: forward inference, plus an external training loop. Behrouz et al. (2026) and sleep-like replay papers attack this gap but are not yet standard LLM infrastructure.

### 6.3 Energy, sparsity, and temporal codes

Biological computation is sparse and event-driven. Dense GPU matmuls achieve accuracy with enormous energy budgets. Spiking networks and neuromorphic hardware narrow the gap (Eshraghian et al., 2023) but lag in ecosystem and peak task performance.

### 6.4 Embodiment and closed-loop grounding

Human concepts are shaped by sensorimotor contingencies, interoception, and social interaction. Disembodied LLMs can imitate linguistic competence without comparable grounding, limiting robust causal reasoning about the physical and social world.

### 6.5 Development and inductive bias

Brains grow under genetic and experiential curricula with strong architectural priors. ANNs typically start from random weights (plus data) or large unsupervised corpora, without a developmental physics of the body.

### 6.6 Sample efficiency and continual competence

Humans learn new skills with few examples without destroying old ones. ANNs often need vast data and still face CF and plasticity loss (Dohare et al., 2024). ICL helps but does not fully solve long-term parameter integration—the problem Behrouz et al. target.

### 6.7 Interpretive and scientific risk

Anthropomorphic vocabulary (memory, attention, sleep, dream, understanding) can smuggle in unjustified mechanistic claims. Good practice: keep functional language, demand algorithmic specificity, and separate capability evaluation from biological modeling claims (Richards et al., 2019).

---

## 7. Theoretical Frameworks for Next-Generation Brain-Inspired Architectures

The goal is not to copy the brain neuron-for-neuron. It is to import **constraints that make biological intelligence robust**—multi-timescale memory, offline reorganization, sparse conditional compute, adaptive forgetting—into systems that remain engineerable.

### 7.1 Framework F1 — Multi-timescale Continuum Memory

**Core idea:** Parameters and states exist on a frequency spectrum (Nested Learning / CMS): high-frequency short-term memory (context, fast adapters) → mid-frequency → low-frequency semantic weights.

**Design rules:**

1. Never update all timescales uniformly.
2. Consolidate high-frequency content into lower-frequency stores *before* overwriting.
3. Evaluate retention as a function of timescale, not only aggregate accuracy.

### 7.2 Framework F2 — CLS 2.0: Dual-process offline cycle

**Core idea:** Maintain an explicit fast episodic buffer (hippocampal analog) and a slow generalizing store (neocortical analog), coupled by **abstractive generative replay** (van de Ven et al., 2020) rather than only exemplar buffers.

**Design rules:**

1. Fast store: high plasticity, high interference risk, rapid binding.
2. Slow store: low plasticity, interleaved updates, schema extraction.
3. Offline phase: generative rehearsal prioritized by uncertainty / weak learning (cf. Schapiro et al., 2018).

### 7.3 Framework F3 — Adaptive capacity and structured forgetting

**Core idea:** Capacity expansion, pruning, and forgetting are optimization operators, not failures.

**Design rules:**

1. Expand sparsely when consolidation would cause interference (Behrouz-style expert growth).
2. Reset or prune temporary high-frequency parameters after successful transfer.
3. Optimize utility-weighted retention; deliberately forget low-utility detail.
4. Monitor and restore plasticity (response to Dohare et al., 2024).

### 7.4 Framework F4 — Sparse conditional modular compute

**Core idea:** MoE-like specialization with integrative hubs.

**Design rules:**

1. Default to sparse expert activation.
2. Train routers for specialization *and* load balance, with occasional exploratory routing during offline “dream” phases (as in Behrouz random expert injection).
3. Maintain cross-module integrative pathways used preferentially during offline simulation (a demythologized DMN analog).

### 7.5 Framework F5 — Local credit with global neuromodulation

**Core idea:** Hybrid learning: local three-factor / feedback-alignment / equilibrium-style updates, sparsely gated by global RL-like modulators.

**Design rules:**

1. Prefer algorithms deployable without full weight transport when targeting neuromorphic hardware.
2. Use global signals for *when* to consolidate, not for every synapse’s gradient.

### 7.6 Framework F6 — Resting-state generative dynamics

**Core idea:** Idle time is computational time: world-model rollouts, counterfactuals, self-consistency repair.

**Design rules:**

1. Schedule endogenous generative activity under compute budgets.
2. Separate externally driven wake updates from internally driven sleep updates.
3. Measure benefits on continual learning and calibration, not only single-task scores.

### 7.7 Architect’s checklist

1. Separate **wake acquisition** from **sleep consolidation**.
2. Maintain a **multi-timescale memory continuum**.
3. Prefer **generative abstractive replay** when capacity is limited.
4. Treat **forgetting and pruning as optimization**.
5. **Grow capacity sparsely**; freeze what is consolidated.
6. Prefer **local or approximate credit assignment** for long-horizon online systems.
7. Benchmark with **continual learning + sample efficiency + energy**, not only peak static accuracy.
8. Label brain metaphors with the analogy taxonomy (functional / computational / mechanistic).

### 7.8 Research agenda (near term)

| Priority | Experiment | Why |
|----------|------------|-----|
| P1 | Ablate consolidation vs. dreaming stages on standardized CL suites | Isolate offline contributions (Behrouz-style) |
| P2 | Compare generative vs. exemplar replay under equal memory budgets | Test CLS predictions in modern nets |
| P3 | Measure plasticity retention over 100+ task sequences | Address Dohare loss-of-plasticity |
| P4 | Joint neuro-AI benchmarks (e.g., Neurobench-style + LLM CL) | Ground metaphors in measurable constraints |
| P5 | Sleep-schedule policies as meta-learned controllers | Move beyond hand-set consolidation frequencies |
| P6 | Human sleep intervention ↔ model sleep ablation parallels | Strongest test of claimed homology |

---

## 8. Discussion

### 8.1 Where the fields productively co-evolve

Three areas show genuine two-way traffic:

1. **Hierarchical sensory models** — DNNs as tools for vision neuroscience (Kriegeskorte, 2015; Richards et al., 2019).
2. **Replay and continual learning** — CLS and sleep physiology informing generative replay and sleep-like training (van de Ven et al., 2020; Tadros et al., 2022; Behrouz et al., 2026).
3. **Modularity and conditional compute** — MoE systems echoing sparse specialized pathways without copying cortical columns.

### 8.2 Where metaphors mislead

- **“Attention”** in transformers is not selective attention in psychology/neuroscience.
- **“Memory”** in LLMs conflates parameters, context windows, and retrieval stores.
- **“Sleep” and “dreaming”** in Behrouz et al. are lifecycle phases for consolidation and synthetic self-training—not models of NREM physiology or dream experience.
- **“Subconscious”** is rarely operationalized in ANN papers with testable predictions.

### 8.3 The special role of the 2026 Sleep paradigm

Behrouz et al. matter because they integrate several previously separate threads—multi-timescale memory, continual learning, distillation, MoE capacity growth, and RL-based self-improvement—into a single **wake/sleep lifecycle** for LLMs. Relative to Nested Learning’s emphasis on *online* consolidation, Sleep emphasizes *offline* systems consolidation and recursive self-modification. That is the right conceptual move for lifelong agents. The remaining scientific task is to harden the analogy: which predictions about prioritization, abstraction, and interference match neural data, and which are purely engineering?

### 8.4 Limitations of this review

- Narrative synthesis, not a PRISMA meta-analysis of effect sizes.
- Rapidly moving arXiv literature (including the anchor) may be revised.
- Citation coverage prioritizes high-signal theoretical and empirical landmarks over exhaustive enumeration.
- No new experiments were run; empirical claims from primary papers are reported as published.

---

## 9. Conclusions

1. **Mechanisms:** Synaptic plasticity and backpropagation solve related credit-assignment problems with different constraints. Systems consolidation via hippocampal–neocortical dialogue has a clear computational homolog in replay and sleep-time distillation; Behrouz et al. (2026) is a leading LLM instantiation. Catastrophic forgetting is not the same as adaptive biological forgetting—future systems need structured forgetting and plasticity maintenance.

2. **Architecture:** Cortical modularity and MoE/modular nets share the principle of specialized, conditionally active capacity, but diverge in development, energy physics, and integrative dynamics.

3. **Offline cognition:** NREM/REM and DMN research describe multi-state endogenous computation that standard ANNs lack. Sleep-time compute and generative “dreaming” are the most promising engineering bridges; DMN and subconscious parallels remain largely metaphorical.

4. **Limits:** Local learning, multi-state dynamics, energy sparsity, embodiment, development, and lifelong plasticity still separate ANNs from biological intelligence.

5. **Path forward:** Build architectures around continuum memory, CLS-style dual stores, adaptive capacity, sparse modules, hybrid local/global learning, and scheduled generative rest—evaluated under continual, efficient, and grounded metrics.

**Final judgment:** The brain is not a deep net, and a deep net is not a brain. They are **convergent solutions to overlapping computational problems**, with enough homology to transfer principles—and enough divergence that biological fidelity must be earned mechanism by mechanism, not declared by metaphor.

---

## References

Allen, E. A., Damaraju, E., Plis, S. M., Erhardt, E. B., Eichele, T., & Calhoun, V. D. (2012). Tracking whole-brain connectivity dynamics in the resting state. *Cerebral Cortex, 24*(3), 663–676. https://doi.org/10.1093/cercor/bhs352

Behrouz, A., Hashemi, F., Javanmard, A., & Mirrokni, V. (2026). Language models need sleep: Learning to self-modify and consolidate memories. *arXiv preprint* arXiv:2606.03979. https://doi.org/10.48550/arXiv.2606.03979

Bellec, G., Scherr, F., Subramoney, A., Hajek, E., Salaj, D., Legenstein, R., & Maass, W. (2020). A solution to the learning dilemma for recurrent networks of spiking neurons. *Nature Communications, 11*, 3625. https://doi.org/10.1038/s41467-020-17236-y

Bloxham, A., & Horton, C. L. (2024). Enhancing and advancing the understanding and study of dreaming and memory consolidation: Reflections, challenges, theoretical clarity, and methodological considerations. *Consciousness and Cognition, 123*, 103719. https://doi.org/10.1016/j.concog.2024.103719

Born, J., & Wilhelm, I. (2011). System consolidation of memory during sleep. *Psychological Research, 76*, 192–203. https://doi.org/10.1007/s00426-011-0335-6

Brodt, S., Inostroza, M., Niethard, N., & Born, J. (2023). Sleep—A brain-state serving systems memory consolidation. *Neuron, 111*(7), 1050–1075. https://doi.org/10.1016/j.neuron.2023.03.005

Buckner, R. L., Sepulcre, J., Talukdar, T., Krienen, F. M., Liu, H., Hedden, T., Andrews-Hanna, J. R., Sperling, R. A., & Johnson, K. A. (2009). Cortical hubs revealed by intrinsic functional connectivity: Mapping, assessment of stability, and relation to Alzheimer’s disease. *Journal of Neuroscience, 29*(6), 1860–1873. https://doi.org/10.1523/JNEUROSCI.5062-08.2009

Buzsáki, G. (2015). Hippocampal sharp wave-ripple: A cognitive biomarker for episodic memory and planning. *Hippocampus, 25*(10), 1073–1188. https://doi.org/10.1002/hipo.22488

Dai, D., Deng, C., Zhao, C., Xu, R., Gao, H., Chen, D., Li, J., Zeng, W., Yu, X., Wu, Y., Xie, Z., Li, Y. K., Huang, P., Luo, F., Ruan, C., Sui, Z., & Liang, W. (2024). DeepSeekMoE: Towards ultimate expert specialization in Mixture-of-Experts language models. In *Proceedings of the 62nd ACL* (pp. 1280–1297). https://doi.org/10.18653/v1/2024.acl-long.70

Dohare, S., Hernandez-Garcia, J. F., Lan, Q., Rahman, P., Mahmood, A. R., & Sutton, R. S. (2024). Loss of plasticity in deep continual learning. *Nature, 632*, 768–774. https://doi.org/10.1038/s41586-024-07711-7

Eshraghian, J. K., Ward, M., Neftci, E., Wang, X., Lenz, G., Dwivedi, G., Bennamoun, M., Jeong, D. S., & Lü, W. (2023). Training spiking neural networks using lessons from deep learning. *Proceedings of the IEEE, 111*(9), 1016–1054. https://doi.org/10.1109/JPROC.2023.3308088

Kaefer, K., Stella, F., McNaughton, B. L., & Battaglia, F. P. (2022). Replay, the default mode network and the cascaded memory systems model. *Nature Reviews Neuroscience, 23*, 628–640. https://doi.org/10.1038/s41583-022-00620-6

Kotseruba, I., & Tsotsos, J. K. (2018). 40 years of cognitive architectures: Core cognitive abilities and practical applications. *Artificial Intelligence Review, 53*, 17–94. https://doi.org/10.1007/s10462-018-9646-y

Kriegeskorte, N. (2015). Deep neural networks: A new framework for modeling biological vision and brain information processing. *Annual Review of Vision Science, 1*, 417–446. https://doi.org/10.1146/annurev-vision-082114-035447

Lillicrap, T. P., Cownden, D., Tweed, D. B., & Akerman, C. J. (2016). Random synaptic feedback weights support error backpropagation for deep learning. *Nature Communications, 7*, 13276. https://doi.org/10.1038/ncomms13276

Marblestone, A. H., Wayne, G., & Körding, K. P. (2016). Toward an integration of deep learning and neuroscience. *Frontiers in Computational Neuroscience, 10*, 94. https://doi.org/10.3389/fncom.2016.00094

Margulies, D. S., Ghosh, S. S., Goulas, A., Falkiewicz, M., Huntenburg, J. M., Langs, G., Bezgin, G., Eickhoff, S. B., Castellanos, F. X., Petrides, M., Jefferies, E., & Smallwood, J. (2016). Situating the default-mode network along a principal gradient of macroscale cortical organization. *Proceedings of the National Academy of Sciences, 113*(44), 12574–12579. https://doi.org/10.1073/pnas.1608282113

Masse, N. Y., Grant, G. D., & Freedman, D. J. (2018). Alleviating catastrophic forgetting using context-dependent gating and synaptic stabilization. *Proceedings of the National Academy of Sciences, 115*(44), E10467–E10475. https://doi.org/10.1073/pnas.1803839115

McClelland, J. L., McNaughton, B. L., & O’Reilly, R. C. (1995). Why there are complementary learning systems in the hippocampus and neocortex: Insights from the successes and failures of connectionist models of learning and memory. *Psychological Review, 102*(3), 419–457. https://doi.org/10.1037/0033-295X.102.3.419

O’Reilly, R. C., Bhattacharyya, R., Howard, M. D., & Ketz, N. (2011). Complementary learning systems. *Cognitive Science, 38*(6), 1229–1248. https://doi.org/10.1111/j.1551-6709.2011.01214.x

Pfeiffer, M., & Pfeil, T. (2018). Deep learning with spiking neurons: Opportunities and challenges. *Frontiers in Neuroscience, 12*, 774. https://doi.org/10.3389/fnins.2018.00774

Richards, B. A., Lillicrap, T. P., Beaudoin, P., Bengio, Y., Bogacz, R., Christensen, A., Clopath, C., Costa, R. P., de Berker, A., Ganguli, S., Gillon, C. J., Hafner, D., Kepecs, A., Kriegeskorte, N., Latham, P., Lindsay, G. W., Miller, K. D., Naud, R., Pack, C. C., … Kording, K. P. (2019). A deep learning framework for neuroscience. *Nature Neuroscience, 22*, 1761–1770. https://doi.org/10.1038/s41593-019-0520-2

Roelfsema, P. R., & Holtmaat, A. (2018). Control of synaptic plasticity in deep cortical networks. *Nature Reviews Neuroscience, 19*, 166–180. https://doi.org/10.1038/nrn.2018.6

Scellier, B., & Bengio, Y. (2017). Equilibrium propagation: Bridging the gap between energy-based models and backpropagation. *Frontiers in Computational Neuroscience, 11*, 24. https://doi.org/10.3389/fncom.2017.00024

Schapiro, A. C., McDevitt, E. A., Rogers, T. T., Mednick, S. C., & Norman, K. A. (2018). Human hippocampal replay during rest prioritizes weakly learned information and predicts memory performance. *Nature Communications, 9*, 3920. https://doi.org/10.1038/s41467-018-06213-1

Seeley, W. W., Menon, V., Schatzberg, A. F., Keller, J., Glover, G. H., Kenna, H., Reiss, A. L., & Greicius, M. D. (2007). Dissociable intrinsic connectivity networks for salience processing and executive control. *Journal of Neuroscience, 27*(9), 2349–2356. https://doi.org/10.1523/JNEUROSCI.5587-06.2007

Sejnowski, T. J. (2020). The unreasonable effectiveness of deep learning in artificial intelligence. *Proceedings of the National Academy of Sciences, 117*(48), 30033–30038. https://doi.org/10.1073/pnas.1907373117

Serrà, J., Surís, D., Miron, M., & Karatzoglou, A. (2018). Overcoming catastrophic forgetting with hard attention to the task. *arXiv preprint* arXiv:1801.01423. https://doi.org/10.48550/arXiv.1801.01423

Shazeer, N., Mirhoseini, A., Maziarz, K., Davis, A., Le, Q., Hinton, G., & Dean, J. (2017). Outrageously large neural networks: The sparsely-gated mixture-of-experts layer. *arXiv preprint* arXiv:1701.06538. https://doi.org/10.48550/arXiv.1701.06538

Sheline, Y. I., Barch, D. M., Price, J. L., Rundle, M. M., Vaishnavi, S. N., Snyder, A. Z., Mintun, M. A., Wang, S., Coalson, R. S., & Raichle, M. E. (2009). The default mode network and self-referential processes in depression. *Proceedings of the National Academy of Sciences, 106*(6), 1942–1947. https://doi.org/10.1073/pnas.0812686106

Sporns, O., & Betzel, R. F. (2015). Modular brain networks. *Annual Review of Psychology, 67*, 613–640. https://doi.org/10.1146/annurev-psych-122414-033634

Staresina, B. P., Bergmann, T. O., Bonnefond, M., van der Meij, R., Jensen, O., Deuker, L., Elger, C. E., Axmacher, N., & Fell, J. (2015). Hierarchical nesting of slow oscillations, spindles and ripples in the human hippocampus during sleep. *Nature Neuroscience, 18*, 1679–1686. https://doi.org/10.1038/nn.4119

Tadros, T., Krishnan, G. P., Ramyaa, R., & Bazhenov, M. (2022). Sleep-like unsupervised replay reduces catastrophic forgetting in artificial neural networks. *Nature Communications, 13*, 7742. https://doi.org/10.1038/s41467-022-34938-7

Utevsky, A. V., Smith, D. V., & Huettel, S. A. (2014). Precuneus is a functional core of the default-mode network. *Journal of Neuroscience, 34*(3), 932–940. https://doi.org/10.1523/JNEUROSCI.4227-13.2014

van de Ven, G. M., Siegelmann, H. T., & Tolias, A. S. (2020). Brain-inspired replay for continual learning with artificial neural networks. *Nature Communications, 11*, 4069. https://doi.org/10.1038/s41467-020-17866-2

Wang, L., Zhang, X., Su, H., & Zhu, J. (2023). A comprehensive survey of continual learning: Theory, method and application. *arXiv preprint* arXiv:2302.00487. https://doi.org/10.48550/arXiv.2302.00487

Zhang, Z., Lin, Y., Liu, Z., Li, P., Sun, M., & Zhou, J. (2022). MoEfication: Transformer feed-forward layers are mixtures of experts. In *Findings of the ACL* (pp. 877–890). https://doi.org/10.18653/v1/2022.findings-acl.71

Zhou, D.-W., Wang, Q., Qi, Z., Ye, H.-J., Zhan, D.-C., & Liu, Z. (2023). Class-incremental learning: A survey. *arXiv preprint* arXiv:2302.03648. https://doi.org/10.48550/arXiv.2302.03648

---

## Appendix A — Glossary

| Term | Definition in this report |
|------|---------------------------|
| **CLS** | Complementary Learning Systems: fast episodic + slow semantic stores |
| **CMS** | Continuum Memory System: multi-frequency parameter updates |
| **CF** | Catastrophic forgetting |
| **DMN** | Default Mode Network |
| **GKD** | Generalized Knowledge Distillation |
| **ICL** | In-context learning |
| **MoE** | Mixture of Experts |
| **NREM / REM** | Non-rapid / rapid eye movement sleep |
| **SKS** | Self-Knowledge Seeding (Behrouz et al.) |
| **SWR** | Sharp wave–ripple |
| **STDP** | Spike-timing-dependent plasticity |

## Appendix B — Anchor paper quick reference

| Item | Detail |
|------|--------|
| Citation | Behrouz et al., arXiv:2606.03979 (v2, 10 Jul 2026) |
| Title | Language Models Need Sleep: Learning to Self-Modify and Consolidate Memories |
| Core claim | Continual LLMs need wake/sleep cycles: consolidate short-term knowledge into long-term parameters, then dream for self-improvement |
| Stage 1 | Knowledge Seeding (upward distillation + RL imitation) with parameter expansion |
| Stage 2 | Dreaming (synthetic curriculum + RL, SEAL-inspired, with novelty via random experts) |
| Biological inspirations cited | Neuroplasticity; online vs. offline consolidation; NREM systems consolidation; REM integration; synaptic pruning |
| Best use in this review | Exemplar of computational homology for systems consolidation in modern LLMs |

## Appendix C — Search notes (reproducibility)

- Anchor full text: arXiv HTML for 2606.03979 (accessed session date 2026-07-22).
- Literature discovery: multi-provider academic search (OpenAlex, arXiv) via WisDev for plasticity/BP, hippocampal replay, continual learning/forgetting, modularity/MoE, DMN/resting state, and brain-inspired deep learning.
- Evidence queries: biological plausibility of backprop; SWR systems consolidation; adaptive vs. catastrophic forgetting.
- Inclusion preference: high-citation foundational works + 2016–2026 SOTA bridging neuroscience and ML.

---

*End of `Grok_ANN_Report.md`*
