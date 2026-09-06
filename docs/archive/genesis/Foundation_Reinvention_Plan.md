# Reinventing the Foundation of Neural Networks

## A biology-first computational substrate, and a plan to build and test it in Rust / native C++

**Author role:** Systems architect for a from-scratch neural substrate
**Date:** 22 July 2026
**Companion documents:** `chatgpt_ANN_Report.md`, `Grok_ANN_Report.md` (both are literature reviews anchored on *Language Models Need Sleep*, Behrouz et al. 2026)
**This document:** `Foundation_Reinvention_Plan.md`

---

## 0. What this document is, and why it exists

The two companion reports are careful and correct, but they stop at the same place. Both compare today's artificial neural networks (ANNs) to the brain, catalogue where they converge and diverge, and then propose *systems-level* additions — sleep phases, replay, consolidation, adaptive forgetting. They treat the neuron, the message it sends, and the way credit is assigned as **fixed background** and rebuild only the memory lifecycle on top.

That is the gap. If we want a genuinely new foundation, we have to go one level lower — beneath sleep, beneath memory, beneath the transformer — to the three primitives that every modern network silently inherited from 1943–1986 and never revisited:

1. **The unit.** A neuron is a scalar: weighted sum, one nonlinearity, one number out.
2. **The message.** Communication is a dense real-valued vector, emitted synchronously, every unit every step.
3. **The learning rule.** Credit is assigned by one global scalar loss differentiated backward through the whole graph.

These three choices are the actual foundation. They are also the three things the brain does *not* do. This document proposes rebuilding all three as a single coherent substrate — biology first, mathematics as the second step that formalizes the biology — and then lays out a concrete, falsifiable plan to build and test it in a systems language (Rust, with a native-C++ fallback), because the substrate we propose is fundamentally about **event-driven, sparse, local computation** that GPU tensor stacks are the wrong tool for.

The goal is not biological fidelity for its own sake, and not a brain simulator. The goal is a **new computational primitive** that keeps the engineering virtues we need (trainable, composable, scalable) while importing the three things that make brains work that ANNs threw away: **time as information, sparsity as the default, and locality of learning.**

### The one-sentence thesis

> Replace the scalar rate-neuron, the dense synchronous vector, and global backpropagation with a **compartmental stateful unit** that communicates through **sparse timed events** and learns from **locally available three-factor signals** — and the result is a substrate whose efficiency, continual-learning ability, and biological plausibility come from its foundation rather than being bolted on afterward.

---

## 1. The three choices modern networks never reopened

### 1.1 The unit: a neuron is not a scalar

The McCulloch–Pitts / perceptron neuron computes `y = φ(Σ wᵢxᵢ + b)`. Every innovation since — convolution, attention, gating, normalization — rearranges *how* those weighted sums are wired, but the atom is unchanged: a memoryless point that maps a vector to a scalar in one shot.

A biological neuron is almost the opposite of this atom:

- It has **internal state that persists in time** (membrane potential, calcium concentration, ion-channel dynamics). It is a leaky integrator, not a function.
- Its dendrites are **not a passive sum**. Distinct dendritic branches perform local nonlinear operations — coincidence detection, plateau potentials, branch-specific thresholds — before anything reaches the soma. A single pyramidal neuron is closer to a small two-layer network than to one node.
- It is **event-driven**: it does nothing until inputs push it past threshold, then emits a discrete event and resets.

The point-neuron abstraction was a reasonable simplification when compute was scarce and calculus was the only available training tool. But it discards the two properties that give biological units their power: **memory over time** and **local nonlinear structure inside the unit.** Neither report reopens this; both accept the scalar unit and add memory *between* units (buffers, adapters, MoE) instead of *inside* them.

### 1.2 The message: dense, synchronous, magnitude-coded

In an ANN, layer *L* produces a full real-valued vector, every element defined, every step, and passes all of it to layer *L+1*. Information lives in the **magnitude** of each number. Everything is **synchronous** (one global clock tick per layer) and **dense** (no element is ever "silent").

The brain communicates through **spikes**: identical, all-or-nothing events. Information is not in the height of a spike (they are all the same height) but in **which** neuron fired and **when** — the identity and the timing. Communication is **asynchronous** (no global clock; events happen when they happen) and **extremely sparse** (a cortical neuron fires on the order of a few times per second, not every millisecond). At any instant the overwhelming majority of the network is silent and costs nothing.

This is the single biggest architectural divergence, and it is the root of the efficiency gap. A dense synchronous network pays for every unit every step whether or not that unit had anything to say. An event-driven sparse network pays only for the events that actually occur. Both reports name sparsity as desirable ("conditional computation," MoE) but treat it as a *routing optimization on top of* dense magnitude-coded messages. They never make the message itself sparse and temporal.

### 1.3 The learning rule: one global loss, differentiated backward

Backpropagation requires a single differentiable scalar objective, a full forward graph held in memory, and an exactly coordinated backward pass that transports weight information in reverse. It works spectacularly. It is also the least biological part of the whole stack, for reasons both reports state well: weight transport, global synchrony, a frozen graph, and a single scalar teacher for the entire system.

A synapse in the brain has access to roughly three things, all **local**: what the presynaptic cell just did, what the postsynaptic cell just did, and a small number of **broadcast neuromodulatory signals** (dopamine, acetylcholine, noradrenaline) that say, globally and cheaply, "that was good / that was surprising / pay attention now." From these three factors alone, and slow-decaying **eligibility traces** that remember which synapses were recently active, the brain assigns credit across long delays without ever computing a gradient of a global loss.

The reports discuss the alternatives (feedback alignment, equilibrium propagation, e-prop) but frame them as *approximations to backprop* to be adopted eventually. The reframing this document makes: **local three-factor learning is not an approximation to backprop — it is a different and more general foundation**, of which supervised gradient descent is one special case (a single, dense, reward-everywhere modulator).

---

## 2. The reinvented foundation

We now define the substrate positively. Three pillars, each replacing one of the three choices above. The design rule throughout: **describe the mechanism in terms of what neurons do, then write the minimal math that captures it — not the other way around.**

### Pillar I — The unit: a compartmental, stateful, event-driven cell

Our primitive is not a scalar node. It is a small object we call a **cell**, with the following anatomy:

- **Dendritic compartments.** Each cell has *k* dendritic branches (k small, e.g. 2–8). Incoming events land on a specific branch. Each branch integrates its own inputs and applies a **local branch nonlinearity** — a threshold that, when crossed, produces a dendritic "plateau" event that is much stronger than any single input. This gives every cell internal two-stage computation (branch → soma) for free. It is what lets one cell detect *coincidences* ("inputs A and B together on the same branch") rather than just weighted totals.
- **A soma with membrane state.** The soma holds a scalar membrane potential `v` that **leaks toward rest over time** and **integrates** the plateau contributions from its branches. State persists between events — the cell remembers its recent history without any external memory module.
- **A threshold-and-reset output.** When `v` crosses threshold `θ`, the cell emits **one event** (a spike) to its downstream targets and resets `v`. Between threshold crossings it emits nothing and costs nothing.
- **An adaptive threshold.** `θ` itself rises after each spike and decays back down (spike-frequency adaptation / homeostasis). This is the cell's built-in gain control: it prevents runaway firing and keeps the network's overall activity near a target level without any global normalization layer.

The formal core (the "second step") is deliberately minimal — a leaky integrate-and-fire soma with nonlinear dendrites:

```
branch j:   d_j(t) = ρ( Σ_{i∈branch j} w_i · e_i(t) )        # ρ = branch nonlinearity (plateau)
soma:       dv/dt   = -(v - v_rest)/τ_m + Σ_j d_j(t)          # leaky integration
fire:       if v ≥ θ:  emit event;  v ← v_reset;  θ ← θ + α    # spike, reset, adapt
threshold:  dθ/dt   = -(θ - θ_0)/τ_θ                           # threshold relaxes back
```

Everything here is cheap, stateful, and local to one cell. There is no matrix multiply. The unit *is* the network's memory, its nonlinearity, and its gain control, all at once.

### Pillar II — The message: sparse timed events on a directed graph

There are no layers and no dense activation vectors. There is a **directed graph** of cells connected by weighted synapses, and a single global quantity: **simulation time.** Computation is the propagation of **events**:

- A cell that fires deposits one event onto each outgoing synapse. The event carries only two things: **which** synapse (identity) and its **delivery time** = now + that synapse's conduction delay.
- Events are held in a **time-ordered event queue**. The engine repeatedly pops the earliest event, delivers it to the target cell's specified branch, updates only that cell's state, and — if that cell crosses threshold — schedules its downstream events.
- Work is proportional to the **number of events**, not the number of cells. If 1% of cells are active in a given window, we do ~1% of the work. Silence is free.

This is the discrete-event / spiking substrate. Information is carried in **the pattern of which cells fire and when** — temporal codes (first-spike latency, spike-time coincidence, rank order) rather than magnitudes. Conduction **delays are first-class trainable parameters**, not nuisances: a network can learn to make two signals *arrive at the same branch at the same time* to trigger a coincidence, which is a computational primitive dense nets simply do not have.

Why this is the right foundation and not just an optimization: sparsity and asynchrony are properties of *the message format itself*, so every downstream benefit (energy, continual learning, temporal reasoning) falls out of the substrate instead of being engineered back in.

### Pillar III — The learning rule: local three-factor plasticity with neuromodulated consolidation

No global loss, no backward pass. Each synapse learns from quantities physically available at that synapse:

1. **A local coincidence signal (the eligibility trace).** When presynaptic and postsynaptic events occur close in time, the synapse sets a slowly decaying eligibility trace `e_ij` — "this synapse was recently part of something." This is spike-timing-dependent plasticity (STDP) captured as a trace: pre-before-post raises it, post-before-pre lowers it.
2. **A broadcast third factor (neuromodulation).** A small number of scalar signals `M(t)` are broadcast to all synapses — reward/value, novelty/surprise, and an attention/gate signal. These are cheap global messages, not gradients.
3. **The update.** The weight changes only where eligibility and modulation coincide:

```
eligibility:  de_ij/dt = -e_ij/τ_e + STDP(pre_i, post_j)
weight:       dw_ij/dt = η · e_ij · M(t)  −  λ · w_ij            # learn where eligible AND modulated; decay otherwise
```

This one rule subsumes several regimes. With `M ≡ 1` everywhere it is Hebbian self-organization. With `M` a reward prediction error it is reward-modulated learning (reinforcement). With `M` a locally-computed prediction error it becomes predictive/self-supervised learning. Backprop's "every weight gets a precise gradient every step" is the degenerate limit where the modulator is dense, exact, and everywhere — the expensive special case, not the foundation.

**Consolidation as a native property, not a bolted-on phase.** Because weights carry eligibility and a slow decay, the substrate already has fast and slow timescales inside every synapse. The "sleep/replay" story from the companion reports becomes a *scheduling policy over the same primitives* — during offline periods, replay recent event sequences and let neuromodulated plasticity move fast traces into slow weights — rather than a separate architecture. We get complementary learning systems for free because the unit and the synapse are themselves multi-timescale.

### 2.4 How the three pillars reinforce each other

The design is coherent, not three unrelated swaps:

- Stateful cells (Pillar I) are what make **timing** meaningful, which is what lets the **event message** (Pillar II) carry information in time.
- Event messages give plasticity its **local pre/post timing** signal (Pillar III's eligibility trace) directly — no need to reconstruct it.
- Local learning needs no stored forward graph, so the engine can be **pure streaming event processing** (Pillar II) with `O(active cells)` memory.
- Adaptive thresholds (Pillar I) keep activity sparse, which keeps the event queue short, which keeps the whole system cheap.

Pull any one pillar and the others lose their reason for existing. That interlock is the thing both reports' proposals lack, and it is the reason this is a *foundation* rather than a feature.

---

## 3. Why the brain is efficient, and how this substrate inherits it

The efficiency question deserves a precise answer, because "brains are efficient" is usually stated and never mechanized. The human brain runs on roughly **20 watts** — about the power of a dim light bulb — while performing perception, motor control, memory, and reasoning that the largest data-center models approach only in narrow slices and at many orders of magnitude more energy. The efficiency does not come from faster components; neurons are *slow* (millisecond timescales, meters-per-second signals). It comes from four structural facts, and our substrate is built to inherit each one.

**1. Event-driven sparsity — you pay only for what happens.** Cortical neurons fire a few times per second on average; at any millisecond, well under 1% of the network is active. A dense ANN computes every unit every step. Our substrate's cost is proportional to events, not cells (Pillar II), so a network that is 99% silent does 1% of the work — the same asymmetry the brain exploits. This is the single largest lever and it is structural, not tuned.

**2. Locality — memory and compute are the same place.** In a GPU, weights live in memory and must be shuttled to compute units every step; the energy goes into *moving data*, not computing. In the brain, the synapse both stores the weight and does the multiply, in place. Our learning rule (Pillar III) uses only quantities local to each synapse, so a faithful hardware or cache-friendly software implementation keeps weight, state, and update **co-located** — no global gradient tensor to materialize and stream.

**3. Time as a free computational resource.** Because information is in spike timing, the substrate computes with delays and coincidences instead of extra multiply-accumulates. A coincidence detector (two spikes arriving together on one dendritic branch) is a logical operation that costs one event, not a layer of matmul. Temporal coding lets a small network express functions that a rate network needs many units to approximate.

**4. Homeostasis instead of normalization.** Adaptive thresholds and synaptic decay (Pillars I and III) keep the network self-balancing — activity stays near a target level automatically. Dense nets need batch/layer normalization, careful initialization, and learning-rate schedules to avoid blowup; here stability is a property the units maintain locally and continuously.

The honest caveat, which we will hold ourselves to in the experiments: **event-driven efficiency only materializes at genuinely low activity and on hardware/software that does not pay for silence.** On a dense GPU simulating spikes densely, we would get the *worst* of both worlds. This is precisely why the build target is a systems language with explicit control over memory and scheduling, not a tensor framework — and why our benchmarks must measure **energy/work per solved task**, not just accuracy.

---

## 4. Why Rust (or native C++), not PyTorch

The substrate's defining operations are: a priority queue of events, pointer-chasing over a sparse irregular graph, per-cell state mutation, and fine-grained parallelism over active cells. None of these map well onto dense tensor kernels. The implementation language must give us:

- **Deterministic, allocation-free hot loops.** The event loop runs billions of times; we cannot afford garbage collection pauses or hidden allocations. Rust and C++ both give manual control; Rust adds memory safety without a GC, which matters when the graph is mutated concurrently.
- **Cache-friendly custom memory layout.** Performance here is dominated by memory access patterns over the sparse graph, not FLOPs. We need to lay out cells and synapses in structure-of-arrays form, control padding, and prefetch. This is exactly what systems languages are for.
- **Fearless parallelism.** Active cells in a time window can be updated in parallel; Rust's ownership model makes the data-race-free version enforceable at compile time (via `rayon`/`crossbeam`), and C++ can do it with more manual care.
- **A path to real neuromorphic / SIMD backends.** The same event-driven core can later target SIMD (`std::simd`), GPUs for the embarrassingly-parallel sub-steps, or neuromorphic chips — without rewriting the model logic.

**Recommendation: Rust as the primary language.** Rationale: memory safety on a concurrent mutable graph is worth a great deal and costs little; the ecosystem (`rayon`, `crossbeam`, `bytemuck`, `criterion` for benchmarking, `pyo3` for a Python front-end) covers everything we need; and this workspace already contains `Rust_MLKit/`, so we build on an existing Rust foundation. C++ remains the fallback if we need a specific library (e.g. a mature neuromorphic SDK) or hand-tuned intrinsics that are more mature in the C++ world. The core is small enough (a few thousand lines) that a port is feasible if ever required.

---

## 5. The engine: architecture for a Rust build

The system splits cleanly into a **simulation core** (fast, unsafe-if-needed, no ML opinions) and a **learning/experiment layer** (safe, high-level). Below is the concrete shape.

### 5.1 Core data structures (structure-of-arrays, cache-first)

```rust
// All cells stored columnar, indexed by CellId(u32). No per-cell heap objects.
struct Cells {
    v:        Vec<f32>,     // membrane potential
    theta:    Vec<f32>,     // adaptive threshold
    v_rest:   Vec<f32>,
    tau_m:    Vec<f32>,     // membrane time constant
    last_spk: Vec<f32>,     // time of last spike (for traces)
    branch:   Vec<[f32; K]>,// per-branch accumulator (K compartments)
}

// Synapses as a flat CSR-style adjacency: for each cell, a slice of outgoing edges.
struct Synapses {
    target:   Vec<CellId>,  // postsynaptic cell
    branch:   Vec<u8>,      // which dendritic branch it lands on
    weight:   Vec<f32>,
    delay:    Vec<f32>,     // conduction delay (trainable)
    elig:     Vec<f32>,     // eligibility trace
    row_ptr:  Vec<u32>,     // CSR: outgoing edges of cell i = target[row_ptr[i]..row_ptr[i+1]]
}

// Time-ordered pending deliveries.
struct EventQueue { /* bucketed timing wheel, see 5.3 */ }

struct Modulators { reward: f32, novelty: f32, attention: f32 } // broadcast third factors
```

Design notes: columnar layout means updating a batch of cells touches contiguous memory; CSR adjacency means firing a cell streams its out-edges linearly; eligibility lives *on the synapse* next to the weight so the update in Pillar III is a single local read-modify-write.

### 5.2 The main loop (event-driven, not clocked)

```
loop:
    event = queue.pop_earliest()          # (deliver_time, syn_id)
    advance_global_time_to(event.time)
    cell = synapse.target[syn_id]
    deposit synapse.weight onto cell.branch[synapse.branch]     # Pillar I integration
    apply branch nonlinearity; leak-integrate soma to current time
    update eligibility trace on syn_id                          # Pillar III (pre side)
    if cell.v >= cell.theta:
        emit spike: for each out-edge of cell, schedule event at now + delay
        update post-side eligibility on incoming synapses
        cell.v = v_reset; cell.theta += alpha                   # reset + adapt
    if modulator arrived this step:
        apply dw = eta * elig * M - lambda * w  over recently-eligible synapses
```

The loop never iterates over inactive cells. A **lazy state update** trick (store `last_update_time` per cell and integrate the leak only when the cell is next touched) means silent cells cost literally nothing until an event wakes them.

### 5.3 The event queue: a timing wheel

A naive binary-heap priority queue is `O(log N)` per event and becomes the bottleneck. Because delays are bounded and discretized, we use a **hierarchical timing wheel** (bucketed calendar queue): `O(1)` amortized insert and pop. This is the single most important performance decision in the engine and mirrors how high-throughput discrete-event simulators (and OS schedulers) work.

### 5.4 Parallelism

Two safe parallelization axes:

- **Within a time step (delta-stepping).** All events in the same small time bucket target independent cells most of the time; process them in parallel with `rayon`, resolving the rare same-target collisions by per-cell atomic accumulation or by partitioning cells across threads (owner thread applies updates).
- **Across independent sub-networks / trials.** Experiments run many seeds; embarrassingly parallel.

Graph partitioning (METIS-style) keeps most synapses intra-partition so cross-thread events are rare — the same principle neuromorphic hardware uses to keep communication local.

### 5.5 Front-end and tooling

- A thin **Python binding via `pyo3`** so experiments, plotting, and dataset loading live in Python while the hot loop stays in Rust.
- **`criterion`** for microbenchmarks (events/sec, cache misses via `perf`).
- **Deterministic seeded RNG** (`rand_chacha`) so every run is reproducible — non-negotiable for the falsifiable experiment program.
- A **spike-raster + weight-evolution visualizer** (export to the Python side) to actually *see* what the network is doing.

### 5.6 Phased build roadmap

| Phase | Milestone | What it proves | Rough scope |
|---|---|---|---|
| **P0** | Single cell: LIF soma + K dendritic branches, unit-tested against analytic solutions | The unit's dynamics are correct | days |
| **P1** | Event queue + graph + propagation; no learning yet | The engine runs, is deterministic, and cost scales with events not cells | 1–2 weeks |
| **P2** | Three-factor plasticity (eligibility + modulator) on the synapses | A network can learn a simple mapping with *no backprop* | 1–2 weeks |
| **P3** | Homeostasis, trainable delays, offline replay/consolidation scheduler | Continual learning + stability without normalization | 3–4 weeks |
| **P4** | Parallel timing-wheel engine, `pyo3` front-end, benchmarking harness | Scale + reproducible science | 3–4 weeks |
| **P5** | Larger tasks, energy/work accounting, baselines vs. dense nets | The efficiency and continual-learning claims, tested | ongoing |

Each phase is independently useful and independently falsifiable — if P2 cannot learn XOR-by-coincidence without backprop, the foundation is wrong and we find out in week three, not year two.

---

## 6. The experiment program (falsifiable, ladder-structured)

A foundation is only worth building if it makes **predictions that could fail.** Each experiment below states a hypothesis, the metric, the baseline, and the failure condition that would count as evidence *against* the substrate. The ladder climbs from single-unit correctness to the two claims that matter: **continual learning** and **energy per task.**

### E1 — The unit computes coincidences a point-neuron cannot (unit level)

- **Hypothesis:** A single cell with two dendritic branches learns to fire for "A and B within Δt on the same branch" and *not* for A or B alone, using only STDP-driven local plasticity.
- **Metric:** classification margin between coincident vs. non-coincident input pairs.
- **Baseline:** a single point-neuron (no branches) on the same task, which provably cannot separate XOR-like coincidence with one unit.
- **Fails if:** the compartmental cell needs a global error signal, or cannot exceed the point-neuron baseline.

### E2 — Learning with no backprop on a standard task (network level)

- **Hypothesis:** A small event-driven network trained *only* with three-factor local plasticity (modulator = reward signal) reaches non-trivial accuracy on a temporal-encoded benchmark (e.g. spiking MNIST / N-MNIST, or a keyword-spotting audio set).
- **Metric:** test accuracy vs. training events consumed.
- **Baseline:** a rate ANN of matched parameter count trained with backprop; and a surrogate-gradient spiking net.
- **Fails if:** local learning plateaus far below usable accuracy, or requires so many events that it is not competitive on a work-per-accuracy basis.

### E3 — Time carries information (representation level)

- **Hypothesis:** The network solves a task that is *only* separable in the temporal domain (e.g. detect a specific spike-time pattern / sequence order) that a rate code discards.
- **Metric:** accuracy on temporal-order tasks; degradation when spike timing is jittered.
- **Baseline:** the same architecture reading only firing rates (timing removed).
- **Fails if:** timing provides no advantage — i.e. the substrate is secretly just a rate network.

### E4 — Continual learning without catastrophic forgetting (the headline claim)

- **Hypothesis:** With eligibility traces + adaptive thresholds + offline replay, the network learns a sequence of tasks (class-incremental) and retains earlier tasks **without** storing raw data or task IDs, and **without** the catastrophic forgetting a matched backprop net shows.
- **Metric:** average accuracy over all tasks after the full sequence; forgetting curve; plasticity retained after 50+ tasks (the loss-of-plasticity test from Dohare et al. 2024).
- **Baseline:** backprop net with and without EWC/replay; the Behrouz-style sleep method as an aspirational upper bound.
- **Fails if:** forgetting matches the backprop baseline, or plasticity collapses over long task streams.

### E5 — Energy / work per solved task (the efficiency claim)

- **Hypothesis:** On a matched task and accuracy, the event-driven substrate uses **orders of magnitude fewer multiply-accumulate-equivalents** than the dense baseline, because it pays only for events.
- **Metric:** synaptic operations (events × fan-out) per inference at fixed accuracy; wall-clock and estimated joules on CPU; projected energy on neuromorphic hardware.
- **Baseline:** dense ANN op-count for the same accuracy.
- **Fails if:** at usable accuracy the network is *not* sparse (activity too high), erasing the event-driven advantage — the most likely and most important way this could fail.

### E6 — Structural plasticity and self-organization (stretch)

- **Hypothesis:** Allowing synapses to be pruned (low weight → removed) and grown (new connections between co-active cells) improves capacity and efficiency over a fixed graph, mirroring biological rewiring.
- **Metric:** capacity and energy vs. a fixed-topology control.
- **Fails if:** structural changes destabilize learning or provide no benefit.

### Verification discipline

Every experiment: fixed seeds, ≥5 runs, report variance and paired tests (not point estimates — a specific criticism the ChatGPT report leveled at the anchor paper, which we adopt as our own standard). Every efficiency claim reported as **work-per-accuracy**, with the sparsity level disclosed. A result that only holds at one hand-tuned operating point does not count.

---

## 7. What would falsify the whole thesis

Intellectual honesty requires naming the ways this foundation could simply be wrong:

1. **Local learning may not scale.** If three-factor plasticity cannot climb past toy tasks no matter how it is arranged (E2/E4 plateau), then backprop's global gradient is doing something irreplaceable and the "foundation" is a dead end for capability. This is the central risk.
2. **The sparsity might not survive accuracy.** Efficiency depends on genuinely low activity. If reaching competitive accuracy forces the network dense (E5 fails), the event-driven advantage evaporates and we have a slower spiking net with no payoff.
3. **Temporal codes may add nothing over rates for the tasks we care about.** If E3 shows no timing advantage, the extra machinery is unjustified complexity.
4. **The engineering may not be worth it.** Even if it works, if it lands within a small constant factor of a well-tuned sparse dense net, the ecosystem cost of a new substrate is not justified.

The plan is explicitly designed so each of these is tested **early and cheaply** — P2/E2 within weeks. We are trying to *break* the idea fast, not defend it slowly.

---

## 8. How this differs from — and completes — the two companion reports

Both reports converge on the same recommendation set: multi-timescale memory, complementary learning systems, adaptive forgetting, sparse conditional compute, and a wake/sleep consolidation cycle. Every one of those is **correct and desirable** — and every one of them *falls out for free* from the substrate proposed here rather than being added as a separate mechanism:

- Multi-timescale memory → native, because cells have state and synapses have eligibility + slow weights.
- Complementary learning systems → native, because fast traces and slow weights already coexist per-synapse.
- Adaptive forgetting → native, because synaptic decay and structural pruning are built-in operators.
- Sparse conditional compute → native, because the message format is itself sparse and event-driven.
- Wake/sleep consolidation → a *scheduling policy* over the same primitives, not a new architecture.

The reports rebuild the **memory lifecycle** on top of the old atom. This document rebuilds the **atom, the message, and the learning rule**, so that the memory lifecycle they want becomes a consequence instead of a construction. That is the sense in which this is a foundation: get the bottom three primitives right, and the systems-level virtues everyone agrees on stop being features you engineer and start being behaviors the substrate exhibits.

---

## 9. Immediate next actions

1. Stand up a Rust crate (`neura-core`) alongside `Rust_MLKit/`: implement the P0 single cell with dendritic branches and unit tests against analytic LIF solutions.
2. Build the P1 timing-wheel event engine and confirm cost scales with events, not cells (this is the make-or-break performance property).
3. Implement P2 three-factor plasticity and run **E1 and E2** — the two experiments that, if they fail, tell us to stop. Target: a network learning coincidence detection and a small classification task with **zero backpropagation**, within the first month.

Everything after P2 is worth building only if E1 and E2 pass. Start there.

