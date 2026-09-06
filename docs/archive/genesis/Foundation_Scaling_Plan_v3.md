# Scaling the Foundation — v3

## Can the assembly substrate go from one node to trillions? And should the target be the LLM or the brain?

**Date:** 22 July 2026
**Builds on:** `Foundation_Reinvention_v2.md` (the sparse-assembly, local-learning substrate)
**Question addressed:** whether the v2 foundation scales single node → millions → billions → trillions, and what "reinventing the whole LLM or brain" concretely requires.

> **Contingent on G2.** Everything in this document assumes the v2 crux experiment (does sparse-assembly local learning beat the local-learning plateau?) passes. If it does not, the substrate does not learn competitively and this entire scaling analysis is moot. Read this as "what scaling looks like *if the foundation works*," not as a validated roadmap. The honest prior is that the crux most likely fails; this document is the payoff branch, not the expected one.

---

## 0. The honest headline

Two things are true at once, and keeping them separate is the whole point of this document:

1. **Node count is not the barrier.** Event-driven hardware that holds *billions* of neurons and *hundreds of billions* of synapses already exists today (Intel Hala Point: 1.15B neurons, 128B synapses, ~20 petaops, in a six-rack-unit chassis). The brain has ~86B neurons and ~100T synapses at 20W. So the physical substrate for brain-scale, event-driven computation is already within ~2 orders of magnitude of the brain on *neuron* count (86B vs 1.15B) and ~3 orders on *synapse* count (~100T vs 128B) — the synapses are the binding resource, so call it ~3 orders — not science fiction. If the only question were "can we instantiate a trillion nodes," the answer is *nearly, and soon.*

2. **Learning at scale is the barrier — and it is unproven.** Whether the v2 local-learning rule keeps working as you compose more and more assembly areas is unknown. The current literature is consistent and discouraging on exactly this: local learning rules have "poor scalability for deep networks." Dense-backprop LLMs scale because they have a *scaling law* — a predictable power-law return on adding parameters and data. **No such scaling law is known for a sparse-assembly, locally-learned substrate.** That is the thing that has to be discovered, and it may not exist.

So the answer to "do I think it can scale to billions/trillions" is: **the machine can; whether the learning does is the real research bet, and it must be earned one order of magnitude at a time.** Anyone who tells you otherwise is selling the hardware press release as if it were the algorithm.

And "reinvent the LLM *or* the brain" is a false *or* that hides a strategic choice — Section 5 argues you should pick the brain target, because on the LLM's turf (language benchmarks, GPU economics) the substrate loses, while on the brain's turf (continual, embodied, 20W) it is the natural winner and the hardware already exists.

---

## 1. What "scaling" even means here — it is not "more parameters"

Dense LLMs scale by making one homogeneous object bigger: more layers, wider matrices, more tokens. The scaling law (test error ∝ N^−α, α ≈ 0.1) is a property of that homogeneous growth. You cannot copy this recipe, because the assembly substrate does not grow by widening a matmul.

The assembly substrate scales the way the brain does: **by composing more areas, not by inflating one graph.** The brain is not one giant fully-connected network; it is thousands of specialized areas (each a modest population) wired by sparse long-range tracts, with hub regions integrating them. Scaling = *add areas and wire them*, recursively. This has three consequences that shape everything below:

- **The unit of scale is the area, not the neuron.** A "trillion-node brain" is really ~10⁴–10⁶ areas of ~10⁶ cells each, composed. You never train a trillion-node monolith; you train and compose areas.
- **Connectivity must stay sparse and mostly local.** All-to-all wiring at 10⁹ nodes is 10¹⁸ synapses — impossible and unnecessary. The brain keeps ~10⁴ synapses per neuron, mostly local, with rare long-range hubs (a small-world graph). The substrate must adopt the same wiring discipline or it dies at the million-node mark on memory alone.
- **Scaling is a question about composition, not size.** "Does it scale" becomes "does composing area A and area B and area C ... produce compounding capability, or does credit assignment and interference break as areas multiply?" That is an empirical question with a possible negative answer.

---

## 2. Does the hardware exist? Mostly yes — a scale ladder

| Tier | Nodes | Synapses | Hardware that runs it *today* | Status |
|---|---|---|---|---|
| Single cell | 1 | ~10³ | anything | trivial |
| Microcircuit | 10³–10⁴ | 10⁶ | laptop CPU (our Rust engine) | easy |
| Area | 10⁶ | 10⁹ | one workstation / one Loihi-2 chip | feasible now |
| Multi-area system | 10⁸ | 10¹¹ | SpiNNaker2 (~175M neurons); GPU cluster (dense sim) | feasible now |
| Brain-fraction | 10⁹ | 1.3×10¹¹ | **Hala Point exists at exactly this scale** | hardware ready |
| Brain-scale | 8.6×10¹⁰ | 10¹⁴ | ~75 Hala-Point-class systems by neurons; ~780× by synapses | ~2 orders (neurons) / ~3 orders (synapses) away |
| "Trillion+" (GPT-synapse-count analog) | 10¹¹–10¹² cells | 10¹⁴–10¹⁵ | not yet in one system; plausible this decade | hardware gap, closing |

The takeaway: **the hardware is not the wall.** A billion-node event-driven system is a purchasable reality; brain-scale is an engineering-and-money problem, not a physics one. This is the opposite of the situation for local *learning*, where we have no evidence the algorithm holds at these scales. The scarce resource is a working learning rule at scale, not transistors.

---

## 3. The three real blockers to scale (ranked)

### Blocker 1 — A scaling law for local learning (the make-or-break)

Dense nets scale because adding capacity reliably lowers loss. For the assembly substrate we do not know the shape of the curve — or whether it monotonically improves at all. Three possibilities, and we cannot yet rule any out:

- **(a) It scales sublinearly but usefully** — composing areas compounds capability with a discoverable exponent. Then we have a new scaling law and a real path to brain-scale.
- **(b) It plateaus** — beyond some number of composed areas, local credit assignment can no longer coordinate them and capability saturates. This is what the "poor scalability for deep networks" literature warns of. Then the substrate is a *specialist* technology (great for edge, continual learning, sensing) but not a brain replacement.
- **(c) It degrades** — interference between areas grows faster than capability. Then it fails outright at scale.

**This is the single most important unknown in the entire program, and it cannot be reasoned about — only measured.** The plan in Section 4 is built to measure it at each order of magnitude and stop the moment the curve turns to (b) or (c).

### Blocker 2 — Wiring at scale (the connectome / genomic-bottleneck problem)

You cannot *learn* a 10¹⁴-entry connectivity matrix from data, and you cannot store it densely. The brain solves this with a **compact wiring rule**: a genome of ~10⁸ bytes specifies the developmental program that grows 10¹⁴ synapses. The connectivity is *generated*, not *learned* — a "genomic bottleneck" that forces the wiring to be compressible and therefore generalizable. The substrate needs the analog: **a parametric wiring prior** (a small program that, given an area's role and position, generates its local + long-range connectivity), not a learned adjacency. Without this, the system is un-buildable past ~10⁷ nodes. v2 did not address this; it is essential for scale.

### Blocker 3 — Communication at scale (the event-routing bottleneck)

At billions of nodes, moving spike events between areas becomes the cost center (this is why neuromorphic chips use address-event representation and keep wiring local). Scaling requires that the **vast majority of events stay intra-area / intra-chip**, with only sparse, low-rate long-range traffic between hubs — exactly the brain's white-matter economy. This is a graph-partitioning and locality constraint that the wiring prior (Blocker 2) must respect. It is solvable (Hala Point does it), but it dictates that the architecture be *born* modular and local, not partitioned after the fact.

---

## 4. The scaling ladder — one order of magnitude at a time, each with a kill criterion

The discipline: **never scale up until the current tier has answered its question.** Each rung is cheap relative to the next, and each has a failure mode that stops the program before it burns a year.

| Rung | Scale | Question it answers | Mechanism added | Kills the program if… |
|---|---|---|---|---|
| **R0** | 1 area, 10⁶ cells | Does local learning match backprop *within* one area? (v2's crux C1) | k-WTA assemblies + three-factor plasticity | plateaus at dense-local baseline |
| **R1** | 3–10 areas, 10⁷ | Does composing areas *compound* capability, or just add? | inter-area `project`/`associate`; wiring prior v1 | capability is flat in #areas |
| **R2** | 10²–10³ areas, 10⁸ | Is there a measurable scaling law (capability vs #areas)? | hub areas; long-range sparse tracts | curve plateaus or degrades (Blocker 1b/c) |
| **R3** | 10⁹, on Hala-Point-class HW | Does the law hold on real event-driven hardware at low activity/energy? | AER routing; genomic-bottleneck wiring | activity not sparse → no efficiency; or law breaks |
| **R4** | 10¹⁰–10¹² | Brain-scale composition; continual lifelong learning | full developmental wiring program; consolidation scheduler | forgetting/interference returns at scale |

The crucial property: **R2 gives the first strong signal cheaply — it does not settle the question.** If, at a few hundred areas on a single workstation, capability-vs-composition traces out a healthy curve, that *justifies investing in the next order of magnitude* — it is not proof the law continues to 10⁴–10⁶ areas, since scaling curves can bend. If it plateaus, you have learned the substrate is a specialist, not a brain, for the cost of a few GPU-weeks. Either outcome is informative and neither requires trillion-node hardware to obtain; but read a healthy R2 as "keep going," not "solved."

---

## 5. LLM or brain? Pick the brain — and here is why

"Reinvent the whole LLM or the brain" are genuinely different targets with different success metrics, hardware economics, and odds. Choosing is the most important strategic decision in the program.

### Why *not* to fight the LLM head-on

- **You would be fighting on the transformer's home turf.** The metrics (perplexity, MMLU, code) reward exactly what dense attention + backprop + web-scale text does best, and that stack is hyper-optimized on GPUs that the transformer was co-designed with. A locally-learned spiking substrate would be years behind on those metrics and running on hardware the benchmarks don't credit.
- **In-context learning, long-range composition over sequences, and web-scale knowledge absorption have no demonstrated assembly analog at scale.** Building an "assembly LLM" means reinventing attention (as assembly binding), sequence memory (as temporal assemblies), and knowledge storage (as consolidated slow weights) — each an open research problem — just to *tie* a transformer. That is the worst kind of bet: enormous effort to reach parity on someone else's metric.

### Why the brain target is winnable

- **The success metrics are the substrate's native strengths:** continual lifelong learning without forgetting, sample efficiency, robustness, and energy (20W). These are precisely where transformers are *weak* and where sparse-assembly local learning is *strong*. You compete where you win, not where you lose.
- **The hardware already exists at the right scale** (Section 2), and it is event-driven — so the substrate's efficiency is realized rather than projected.
- **It is a different product, not a worse chatbot:** an always-on, embodied, continually-adapting agent that learns from its own stream of experience at tens of watts. Nothing in the transformer world is trying to be this, so parity is not the bar.

### The honest middle path

Pursue the **brain target** as the primary program. Treat "language" not as the goal but as *one capability an embodied assembly agent eventually acquires* — grounded, continual, and small, rather than pretrained on the web. If the R2 scaling law turns out strong, *then* revisit whether an assembly substrate can challenge LLMs on language; do not start there. This keeps the program aimed at what the foundation is actually good for, and treats LLM-competition as an optional later fork contingent on the scaling law existing.

---

## 6. The compositional scaling mechanism, concretely

For the plan to be buildable, "compose areas" needs to be a real operation, not a slogan. The ingredients:

- **Area** = a population of ~10⁶ cells with k-WTA inhibition; hosts assemblies; the reusable scaling unit (the "cortical area / column" analog).
- **Inter-area projection** = Assembly-Calculus `project`: firing an assembly in area A into area B creates a stable corresponding assembly in B. This is how a concept moves and transforms across the hierarchy — the substrate's version of a "layer," but learned locally and Hebbianly with convergence guarantees.
- **Association** = binding co-active assemblies across areas — the substrate's version of relational/attention-like composition.
- **Wiring prior (the genomic bottleneck)** = a small parametric program `wire(role, position) → local + sparse long-range connectivity`. Areas are *grown*, not hand-wired; this is what makes 10⁹+ nodes describable in a compact, generalizable form and keeps events local (Blocker 2 & 3).
- **Hub areas** = a few densely-connected integrator areas (the DMN/association-cortex analog) that the reports' "global workspace" idea maps onto — providing long-range integration without all-to-all cost.
- **Consolidation scheduler** = offline replay moving fast eligibility to slow weights across areas — the companion reports' "sleep," now operating over a composed multi-area system so that scaling and continual learning are the same mechanism.

Scaling is then literally: instantiate more areas via the wiring prior, connect them through hubs, let local plasticity + periodic consolidation organize them. The research question of Section 4 is whether *that loop* compounds capability.

---

## 7. Verdict on trillions

- **Physically:** yes within the decade. Billion-node event-driven hardware exists now; brain-scale is ~75× today's largest single system by *neuron* count but ~780× by *synapse* count (the binding resource) — an engineering/funding problem, not a physics wall, but a ~3-order-of-magnitude one.
- **Algorithmically:** unknown, and this is the real answer to your question. There is no demonstrated scaling law for local learning, and the prior evidence leans toward plateaus at depth. The program is explicitly designed to find the shape of that curve at R2 (a few hundred areas, cheap) before betting on hardware.
- **Strategically:** aim at the brain, not the LLM. The substrate's native metrics are the brain's, the hardware is event-driven, and you avoid a losing fight on the transformer's optimized home ground.

The one-line synthesis: **the barrier to scaling this foundation is not the number of nodes — it is whether locally-learned assembly composition has a scaling law at all. Discover that curve at the hundred-area scale first; everything else, including trillion-node hardware and any LLM ambitions, is downstream of that single empirical result.**

---

## 8. Tie-in to `neura-core` (extends the v2 build plan)

Two additions to the v2 milestones, both required for scale and both testable cheaply:

- **M5 — Multi-area composition.** Implement `Area`, inter-area `project`/`associate`, and hub areas. Run **R1** (does composition compound?). This is the first scaling signal and needs only a workstation.
- **M6 — Wiring prior (genomic bottleneck).** Implement `wire(role, position)` as a compact parametric generator; verify events stay >90% intra-area (Blocker 3) and that areas can be grown, not stored. Run **R2** — trace capability vs. number of areas and *fit the scaling curve*. This single experiment is the highest-information result in the whole program.

If R2's curve is healthy, scaling to billion-node hardware is justified. If it plateaus, you have discovered — for a few GPU-weeks — that the honest product is a world-class continual-learning edge system, not a brain. Both are wins; only one is a brain; you find out which without needing a trillion nodes to ask.
