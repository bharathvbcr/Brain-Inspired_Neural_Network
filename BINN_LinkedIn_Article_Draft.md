# Why Haven't Brain-Inspired Neural Networks Replaced ANNs?

## The question that changed how I am approaching the problem

I began this project with a simple belief:

> If the brain learns continually through sparse activity and mostly local adaptation, then a neural network built around those principles should eventually have important advantages over today's artificial neural networks.

Then I asked the question that made the idea uncomfortable:

**If that is true, why haven't brain-inspired neural networks already taken over?**

At first, I blamed backpropagation. It seemed obvious that global gradients were the central limitation: biologically implausible, expensive to coordinate, and vulnerable to catastrophic forgetting during sequential learning.

But the deeper I went, the less satisfying that answer became.

Backpropagation is not merely an old assumption waiting to be replaced. It is extremely effective at assigning credit across deep networks. Dense matrix multiplication maps remarkably well onto GPUs. Large batches expose parallelism. PyTorch, CUDA, pretrained checkpoints, datasets, and established benchmarks allow every new project to inherit years of engineering.

ANNs did not win because researchers ignored biology. They won because the algorithm, hardware, data, and software stack reinforce one another.

That realization changed my project. I am no longer asking, "How can I make a neural network look more like a brain?"

I am asking:

> **Which biological principles create a measurable computational advantage, and what is the cheapest experiment that could prove my hypothesis wrong?**

This article describes where that question has led me. I use **BINN—brain-inspired neural network—as shorthand for this project**, not as a claim that all spiking, neuromorphic, or biologically motivated systems are one unified field.

Most importantly, this is not a victory announcement. The `binn/` implementation and its benchmark evidence do not exist yet. What exists today is an architecture, a build plan, and a set of explicit kill gates.

## How scrutiny changed the hypothesis

The most honest version of this story is not that I had an idea and then wrote a plan. It is that each attempt to make the idea precise exposed another reason it might fail.

I initially focused on three familiar biological substitutions: a compartmental cell instead of a scalar unit, sparse timed events instead of dense activation vectors, and local plasticity instead of backpropagation. I treated their combination as a reinvented foundation and implied that efficiency, continual learning, and consolidation would follow naturally.

That was too confident. Spiking cells, nonlinear dendrites, event-driven simulation, and three-factor plasticity are established research directions. A more defensible novelty statement is: **every brick already exists; the unbuilt structure is the hypothesis that sparse assemblies can make local credit assignment sufficient on difficult tasks.**

That realization made every scaling claim contingent on the local-learning hypothesis surviving. A broader gap analysis then exposed problems I had underweighted: sequential forward simulation, lossy spike encoding, weak data infrastructure, and the gravitational advantage of the ANN ecosystem. The proposal consequently became a gated Rust research instrument rather than a presumed path to a new foundation.

An adversarial audit of the argument produced three corrections that now shape the project:

1. A hard spike reset blocks clean parallelization through time. Associative scans can help with sub-threshold dynamics or within chunks, but they do not remove the sequential dependency.
2. “Learned, co-trained encoders” conflict with a no-autodiff production path. The crux experiments should therefore begin with fixed, information-preserving encoders whose loss is measured explicitly.
3. Assembly Calculus proves useful convergence and bounded-computation results, but it does not prove that local learning will solve deep, poorly separated, compositional tasks.

This scrutiny lowered my confidence while improving the experiment. Given the base rate of attempts to replace backpropagation with local learning, my prior is now that the central kill gate is more likely to fail than pass. The reason to proceed is not confidence in a breakthrough. It is the chance to obtain a clean answer at bounded cost.

## The first lesson: ANN drawbacks are trade-offs, not proof of failure

It is easy to write a list of differences between ANNs and brains and treat every difference as an ANN defect. That is not a fair comparison.

Still, several properties of conventional ANNs motivate searching for a complementary substrate.

### Global credit assignment

Standard backpropagation requires a differentiable objective and coordinated forward and backward computation. In its conventional form, it also raises the weight-transport problem: distant parameters receive precisely structured error information that biological synapses do not obviously possess.

Feedback alignment and related approaches show that exact symmetric feedback may not always be necessary. But a general, biologically grounded alternative that matches backpropagation across deep, difficult tasks has not been established. [Lillicrap et al., 2016](https://www.nature.com/articles/ncomms13276)

### Dense scheduled computation

Conventional accelerators are designed to make dense operations efficient. That is a tremendous advantage, but it means the system usually pays for the operations scheduled by the graph even when only a small part of the learned representation matters for a particular input.

Mixture-of-Experts and activation sparsity reduce this cost. So sparsity is not unique to brain-inspired systems. The sharper question is whether **events can become the native unit of computation**, so inactive cells generate no work rather than zeros inside a dense operation.

### Continual adaptation

When a shared parameter set is updated sequentially, new gradients can interfere with earlier solutions. Replay, parameter isolation, and regularization methods such as Elastic Weight Consolidation can reduce catastrophic forgetting, but lifelong adaptation is not yet a default property of most deployed models. [Kirkpatrick et al., 2017](https://doi.org/10.1073/pnas.1611835114)

Sleep-inspired replay has recovered older representations in experimental ANNs, and recent language-model research uses multi-timescale memory and synthetic rehearsal. These are promising computational ideas, although they remain very different from biological sleep. [Tadros et al., 2022](https://www.nature.com/articles/s41467-022-34938-7), [Behrouz et al., 2026](https://arxiv.org/html/2606.03979v2)

These limitations do not show that ANNs are a dead end. They define the conditions under which another substrate might deserve attention.

## The second lesson: biological plausibility does not automatically produce performance

My initial thinking focused on local credit assignment and neural assemblies. A broader gap analysis forced me to confront problems I had underestimated.

### 1. Local learning can fail with depth

Hebbian, STDP, and three-factor rules use information available near each synapse. That locality is attractive, but it does not guarantee that early components receive useful credit for a distant outcome. Whether sparse assemblies can change this is the central scientific bet—not an established result.

### 2. Time can destroy throughput

Spiking and recurrent systems preserve membrane state, which makes temporal computation possible. But training them with backpropagation-through-time requires step-by-step unrolling. This sacrifices the temporal parallelism that helped transformers displace traditional recurrent networks.

Online local learning removes the backward unroll and its sequence-length activation memory, but not the sequential forward simulation. A biologically inspired system can therefore have elegant dynamics and still lose because a single temporal stream is too slow to process. Practical parallelism must come primarily from neurons, areas, or independent streams—not from pretending the time dependency disappeared.

### 3. The encoder may lose the experiment before the network begins

Images, text, and tabular values do not naturally arrive as spikes. Rate codes, latency codes, and population codes retain different information. If the encoder discards task-relevant structure, the network is blamed for a failure created at its input boundary.

That is why encoders and decoders must be visible, measured components—and why early headline experiments should use natively temporal data such as audio, event vision, or sensor streams. For the crux experiment, I will use fixed, information-preserving encoders rather than introduce a learned I/O model that could create or hide the result. A locally learned encoder or an explicitly labeled autodiff boundary can be tested later.

### 4. Ecosystem gravity is a scientific variable

ANN projects inherit optimized kernels, debuggers, pretrained models, public datasets, and a huge community. Brain-inspired projects often begin from scratch on incompatible simulators.

When one system inherits a mature civilization and another inherits a blank directory, accuracy is not the only difference being measured.

### 5. Sparse activity does not guarantee efficient execution

If a spiking model is simulated with dense kernels, silence may still cost almost as much as activity. The event-driven advantage exists only when the runtime avoids work for inactive units and when competitive accuracy does not require high firing rates.

### 6. There is no demonstrated general scaling law

Assembly-based computation has promising mathematical foundations, but we do not yet know whether composing more areas produces a predictable capability curve. Without that evidence, scaling from a small simulation to a brain-sized architecture is speculation.

This is why I no longer think the right pitch is "the brain is efficient, therefore my model will be efficient."

The right question is: **where does the proposed advantage appear in a measurement, compared with what baseline, and under which failure condition?**

## The approach I now want to test

My hypothesis is that three primitives must change together.

That creates an obvious danger: a complicated system can look "brain-inspired" while making it impossible to identify which mechanism did useful work. I therefore want every biological addition to earn its place through ablation, not analogy.

### 1. The unit

Replace the memoryless scalar activation with a compartmental, stateful cell: a leaky integrate-and-fire soma, nonlinear dendritic branches, persistent membrane state, and a homeostatic adaptive threshold.

The dendrites are not included for visual biological resemblance. They provide local coincidence detection and a second level of computation inside the cell. Recent artificial dendritic models have shown gains in robustness and parameter efficiency, though they do not establish that my proposed system will scale. [Chavlis and Poirazi, 2025](https://www.nature.com/articles/s41467-025-56297-9)

### 2. The message

Replace dense synchronous activation vectors with sparse timed events on a directed graph. A firing cell schedules events to its downstream synapses; a silent cell schedules nothing.

The runtime must therefore be event-driven. Otherwise, sparsity is only representational and does not become a systems advantage.

### 3. The learning rule

Replace the production backward graph with local three-factor plasticity. Each synapse uses pre- and postsynaptic timing to maintain an eligibility trace, then updates when a small set of broadcast modulators—such as reward, novelty, or attention—arrive.

The proposed update is forward-only and uses memory that is constant in sequence length. That removes the backward-unroll cost of backpropagation-through-time, but the event stream still advances sequentially. It also trades gradient-based credit for the harder question: **will local credit remain sufficient as the computation deepens?**

### The compositional layer: sparse neural assemblies

Individual spikes are too low-level to provide a theory of cognition. I plan to use sparse assemblies as the intermediate representation.

The Assembly Calculus defines operations including projection, association, pattern completion, and merge, supported by mathematical analysis and simulation. It gives the project a theory-backed representational primitive rather than an informal claim that "the brain uses populations." But the proven regime is narrower than the proposed architecture needs: convergence and bounded computation do not establish competitive learning on deep, poorly separated, compositional tasks. [Papadimitriou et al., 2020](https://pubmed.ncbi.nlm.nih.gov/32518114/)

The key hypothesis is that sparse assembly geometry may reduce interference enough for local learning to remain useful at greater depth.

That sentence contains the entire scientific risk: **may**.

Before treating the combined architecture as evidence, I will remove its proposed mechanisms one at a time: dendritic branches, adaptive thresholds, trainable delays, assembly competition, eligibility timescales, and modulatory signals. A component that produces no reproducible advantage should leave the core architecture, however biologically appealing it may be.

## Why I am building a small runtime from scratch

Building an ML stack from scratch is usually a mistake. I am doing it here because dense tensors, synchronous layers, and autograd are not neutral implementation details for this experiment. They can pull the design back toward the computational assumptions I am trying to test.

The planned Rust workspace therefore has no dense-matrix-multiplication primitive and no production autograd engine. It will include:

- Deterministic state buffers and sparse connectivity.
- A time-ordered event queue.
- Compartmental cells and local synaptic state.
- Sparse areas with k-winners-take-all inhibition.
- Assembly projection and association.
- Forward-only three-factor plasticity.
- Fixed, measured encoders and decoders through the crux gate; learned I/O only as a later, separately labeled experiment.
- A reproducible experiment harness.

This does **not** mean refusing conventional tools where they are appropriate. Well-tuned ANN and surrogate-gradient models will be mandatory labeled baselines. Plotting, data loading, and analysis do not need to be reinvented merely to make the project look independent.

## The experiment must be able to say no

The project is organized around gates, not milestones that assume success.

### Gate 0: correct dynamics

A single dendritic LIF cell must match its analytic membrane solution. Identical seeds must generate identical spike trains.

### Gate 1: a working representation

Assemblies must form, stabilize, project into new areas, and associate under controlled conditions. Observed convergence must match the theoretical model closely enough to justify continuing.

### Gate 2: the local-learning kill gate

On the same temporal task and parameter budget, the sparse-assembly learner will be compared with:

- A matched dense local-learning baseline.
- Strong eligibility-trace local learners and feedback-alignment-style credit-assignment baselines where applicable.
- A matched surrogate-gradient spiking network.
- A well-tuned backpropagation or BPTT reference.

If the assembly learner remains closer to the strongest non-assembly local baseline than to the BPTT reference, I stop the central program and report the negative result transparently. A negative may be scientifically useful; I do not assume that it is automatically publishable.

### Gate 3: continual learning with a mechanism

The system must learn a stream without task IDs or raw-data rehearsal, under the same information constraints as its baselines. A separate replay-allowed track can compare consolidation strategies without quietly giving one learner more memory than another. Lower forgetting must correlate with reduced overlap between task assemblies. A performance change without a supported mechanism is not enough.

### Gate 4: a scaling curve

The project will compose three, tens, and eventually hundreds of areas. If capability plateaus across the tested range, claims about scaling toward larger cognitive systems will be rejected. A healthy curve would justify testing the next order of magnitude; it would not prove that the same curve continues to brain scale.

### Gate 5: an honest efficiency result

At matched accuracy, every result must disclose activity sparsity, event counts, synaptic operations, wall-clock cost, memory, and uncertainty across independent seeds.

If competitive accuracy forces dense firing, the efficiency thesis fails.

The governing comparison is **work per achieved accuracy at a disclosed activity sparsity**, including queueing, routing, and irregular-memory overhead—not accuracy alone and not a projected neuromorphic energy number detached from the actual workload. I will also report the accuracy-work Pareto frontier and the underlying measurements separately, so a single ratio cannot hide a slower, less accurate, or less stable system.

## Where I need researchers to challenge this

The project is still early enough that criticism can change the architecture rather than become a footnote after the experiments.

I would value scrutiny from several communities:

- **Computational neuroscientists:** Which biological mechanisms in this design are computationally essential, and which are decorative or incorrectly simplified?
- **Spiking-network researchers:** Is the local-learning kill gate fair? Which strong baselines or temporal tasks would make the comparison credible?
- **Learning theorists:** Under what assumptions could sparse assemblies reduce interference or extend useful local credit? What counterexample would invalidate the idea fastest?
- **Continual-learning researchers:** Is learning without task IDs and raw-data rehearsal the right test? Which forgetting and transfer metrics should be mandatory?
- **Systems and neuromorphic researchers:** What should count as work in an event-driven runtime? Where will queueing, irregular memory access, or communication erase the theoretical advantage?
- **Researchers with negative results:** Which approaches looked promising in simulation but failed with depth, noise, scale, or real hardware?

I am especially interested in objections that change a gate, reveal a missing baseline, or identify a cheaper falsification experiment.

Please share papers I may have missed, failed experiments that were never published, benchmark recommendations, or a direct argument for why this substrate cannot work. If any part of the experiment still protects the hypothesis instead of testing it, I would like to know where.

I do not yet know whether this architecture is better than an ANN.

I know what evidence I would accept against it.

And I believe inviting that scrutiny before implementation is the most scientifically useful place to begin.

---

## Selected research

- [Brain computation by assemblies of neurons](https://pubmed.ncbi.nlm.nih.gov/32518114/) — Papadimitriou et al., PNAS, 2020.
- [Random synaptic feedback weights support error backpropagation for deep learning](https://www.nature.com/articles/ncomms13276) — Lillicrap et al., *Nature Communications*, 2016.
- [Overcoming catastrophic forgetting in neural networks](https://doi.org/10.1073/pnas.1611835114) — Kirkpatrick et al., PNAS, 2017.
- [Sleep-like unsupervised replay reduces catastrophic forgetting in artificial neural networks](https://www.nature.com/articles/s41467-022-34938-7) — Tadros et al., *Nature Communications*, 2022.
- [Dendrites endow artificial neural networks with accurate, robust and parameter-efficient learning](https://www.nature.com/articles/s41467-025-56297-9) — Chavlis and Poirazi, *Nature Communications*, 2025.
- [Language Models Need Sleep](https://arxiv.org/html/2606.03979v2) — Behrouz et al., arXiv preprint, 2026.
