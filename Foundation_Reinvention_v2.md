# Reinventing the Foundation of Neural Networks — v2

## An honest re-audit, and a foundation rebuilt around the problem that actually matters

**Date:** 22 July 2026
**Supersedes:** `Foundation_Reinvention_Plan.md` (v1), which is kept as a record
**Companion reviews:** `chatgpt_ANN_Report.md`, `Grok_ANN_Report.md`

---

## 0. Audit of v1 — what it got right, wrong, and dishonest

Before rebuilding, an unsparing grade of the first draft. v1 proposed a substrate of three pillars — a compartmental stateful cell, sparse timed events, and local three-factor plasticity — and claimed this was a "reinvented foundation." Checked against the current literature (mid-2026), that claim does not survive contact.

| v1 claim | Verdict | Why |
|---|---|---|
| The compartmental, stateful, event-driven **unit** is a new primitive | **Derivative** | This is a spiking neuron with nonlinear dendrites. It already exists and is actively published: dendritic spiking neurons (DendSN), L5-pyramidal-cell models, "single neuron ≈ deep network" results. Good idea, not a new one. |
| Sparse **timed events** are a new message format | **Derivative** | This is the definition of a spiking neural network (SNN). Timing wheels / discrete-event spike simulation are standard. |
| **Local three-factor plasticity** is a more general foundation than backprop | **True but incomplete** | The framing is right, but v1 asserted it *works* and never confronted the fact that this is exactly where every biologically-plausible method has failed to scale. |
| Event-driven gives **orders-of-magnitude** efficiency | **Overclaimed** | On CPU/GPU the measured advantage is ~65–75% (a small constant factor). The large multipliers are *neuromorphic-hardware projections*, not demonstrated results. v1 stated the projection as fact. |
| Systems-level virtues (continual learning, consolidation) "fall out for free" | **Unearned** | They only fall out *if local learning works at depth*. v1 built the roof and skipped the load-bearing wall. |
| Leads with the unit and the message | **Wrong emphasis** | These are the well-trodden, easy parts. The actual foundation — the thing that has blocked the field for 40 years — is credit assignment. v1 spent its first three sections on the parts that are already solved and hand-waved the part that isn't. |

**The one honest sentence about v1:** it is a competent synthesis of known neuromorphic components, dressed as a reinvention, that avoids the only problem whose solution would actually constitute a new foundation. A reviewer would reject it as "SNNs with extra steps."

The rest of this document fixes that by inverting the priority: **lead with credit assignment, choose the representation and substrate specifically to make credit assignment local and sufficient, and be explicit about what is known versus what is the actual bet.**

---

## 1. Re-diagnosis: the foundation is credit assignment, not the neuron

Strip away the vocabulary and there are three problems any learning substrate must solve, in increasing order of difficulty:

1. **Substrate / efficiency** — how signals are represented and moved (dense vs. event-driven). *Largely solved; the trade-offs are understood.*
2. **Representation** — what a "unit of meaning" is (a scalar activation vs. a distributed population). *Partially solved.*
3. **Credit assignment** — how, when the system does something good or bad, each of billions of synapses knows how to change. *This is the wall.*

Backpropagation is the only method that solves #3 at scale, and it does so with machinery the brain cannot plausibly have: a global scalar loss, a frozen forward graph held in memory, an exact backward pass, and weight transport. Every attempt to replace it — feedback alignment, equilibrium propagation, predictive coding, forward-forward, e-prop, target propagation — works in restricted settings and then **plateaus or destabilizes as networks get deeper.** The current literature is blunt about this: state-of-the-art spiking networks reach ANN accuracy only by training with surrogate-gradient backpropagation-through-time (i.e., they re-import backprop); fully local rules "struggle to maintain the cross-layer coordination needed for coherent global learning," and predictive coding "faces gradient explosion and vanishing in deep networks."

This is the whole game. A "new foundation" that does not have a credible answer to local credit assignment is not a new foundation — it is a new front-end on the same unsolved problem. v1 was exactly that.

So v2 makes a single strategic commitment: **do not try to approximate backprop's global gradient with local machinery.** That race has been run for a decade and the local runners keep losing. Instead, change the conditions of the race — pick a representation in which a *local* learning signal is *sufficient*, because the geometry of the representation removes the need for precise cross-layer gradient coordination in the first place.

---

## 2. The reframed thesis: representation-first

> **Bet:** If the network's internal representations are *sparse, high-dimensional, distributed assemblies* (populations, not single units), then (a) different concepts occupy near-orthogonal subspaces, so learning one thing barely disturbs another; (b) a simple local Hebbian-plus-neuromodulator rule becomes *sufficient* for credit assignment, because there is little interference to untangle and the "which units were responsible" question is answered by which assembly was active; and (c) continual learning and energy efficiency follow as consequences of the sparsity, not as separate mechanisms.

The three primitives, now in **dependency order** (each one exists to serve the one above it):

1. **Representation — the sparse distributed assembly.** The unit of meaning is not a neuron and not a scalar. It is an *assembly*: a large, sparse set of co-firing cells that represents a concept, drawn from a much larger population so that any two assemblies overlap only slightly. This is the primitive of the **Assembly Calculus** (Papadimitriou, Vempala, Maass, and colleagues), which defines biologically-realizable operations on assemblies — **projection** (copy a concept into a new area), **association** (bind two co-active concepts), and **merge** — and proves these compile down to spiking neurons with Hebbian plasticity, and can perform space-bounded computation, classification, and even parsing/language. It is the missing representational layer v1 never considered.

**Important scope limit (do not over-read this).** What Assembly Calculus actually *proves* is narrower than what this document's central bet needs. Its learning/classification guarantees hold for **"reasonably separated" classes** and for **bounded computation** (on the order of a few hundred parallel steps); it also has documented expressiveness limits (an Assembly-Calculus parser is provably *weaker than a finite automaton*). The hard regime real ML lives in — not-well-separated, deep, compositional tasks — is exactly where these guarantees run out. So Assembly Calculus makes the representation-first bet *coherent and biologically grounded*, but it does **not** de-risk the crux. Treat the sufficiency of local credit at task scale as a genuinely open empirical question (tested at G2), not as something the theory has settled.

2. **Credit assignment — local, made sufficient by (1).** Because assemblies are near-orthogonal, learning is local Hebbian plasticity gated by a small number of broadcast neuromodulators (reward, novelty, attention). No target propagation, no backward pass. The claim is not that this is a clever approximation to a gradient; it is that in a sparse-assembly regime the gradient's job — deconflicting overlapping credit — is mostly unnecessary, so a local rule can do it.

3. **Substrate — event-driven dendritic cells, chosen to make (1) and (2) natural.** Spiking, stateful, compartmental cells are adopted **not** as the headline innovation (v1's mistake) but because they are the cheapest physical realization of sparse assemblies and local timing-based plasticity: sparsity is native, coincidence detection forms assemblies, timing gives the local pre/post signal for free, and silence is free so the whole thing is efficient.

Notice the causal chain is now tight and runs *downward*: pick the representation → local learning becomes viable → the event-driven substrate is the natural implementation → efficiency and continual learning are byproducts. v1's chain ran the other way and stalled because it never established step 2.

---

## 3. The mechanism — biology first, math second

### 3.1 Representation: assemblies, not activations

Biologically: a concept in cortex is not one "grandmother cell" and not a dense pattern over all neurons. It is a **cell assembly** (Hebb's original idea) — a sparse, recurrently-connected population that ignites as a unit. Sparse coding is a measured property of cortex, and it is what keeps representations separable.

Formally (second step): a population of `N` cells; a concept is a binary-ish activity vector with only `k ≪ N` active (e.g. `k/N ≈ 1–2%`). Two random assemblies of size `k` in dimension `N` have expected overlap `k²/N`, which is tiny when `N` is large — this near-orthogonality is the entire reason local learning can work. The Assembly Calculus operations become:

```
project(A → area Y):   repeatedly fire A into Y; the k most-excited cells in Y,
                        strengthened by Hebbian plasticity, converge to a stable
                        assembly A' that now "means" A in Y.
associate(A, B):       co-firing A and B increases overlap of their assemblies,
                        binding the concepts.
```

These are not metaphors; they are proven to converge under a random-connectivity + Hebbian-plasticity + k-cap (top-k inhibition) model. That *convergence guarantee* is the kind of foundational result v1 lacked — but note (per §3.1's scope limit) that convergence of `project` is not the same as competitive *learning* on hard tasks; the former is proven, the latter is the bet.

### 3.2 The unit: a cell that makes assemblies cheap

The cell is v1's compartmental spiking unit — kept, but demoted to "implementation detail that serves the representation." Its two relevant jobs:

- **k-winners-take-all via inhibition.** A shared inhibitory pool enforces that only ~k cells win per area per cycle. This is what makes representations sparse and assemblies well-defined. (Biology: fast feedforward/lateral inhibition. Math: a top-k / divisive-normalization operator.)
- **Dendritic coincidence detection.** Nonlinear dendritic branches let a cell fire only when *specific combinations* of assembly members are active together — this is how associations and higher-order features form. This is now well-supported: a single pyramidal neuron can implement functions that need a multi-layer perceptron, so putting this power *inside* the unit is buying real representational capacity, not decoration.

Minimal soma dynamics (leaky integrate-and-fire with adaptive threshold) are unchanged from v1 and are standard; they are not where the novelty lives, and v2 stops pretending they are.

### 3.3 Credit assignment: the actual proposal

Three-factor plasticity, but with an explicit account of *why it is sufficient here* rather than an assertion that it works:

```
eligibility:  de_ij/dt = -e_ij/τ_e + STDP(pre_i, post_j)      # local, timing-based
weight:       Δw_ij     = η · e_ij · M(t) − λ · w_ij           # gated by broadcast modulator
```

The load-bearing argument (this is the thesis, stated so it can be attacked):

- In a **dense** representation, many overlapping units are partly responsible for any output, so a local rule cannot tell which to change — you need the gradient to apportion blame. This is why local rules fail in dense nets.
- In a **sparse-assembly** representation, the active assembly *is* the set of responsible units. A modulator that says "that was good" applied via Hebbian eligibility strengthens exactly the assembly that produced the outcome, and — because other assemblies are near-orthogonal — barely touches them. Credit assignment reduces to "reward the pathway that just fired," which is local and needs no backward pass.

**This is the entire bet, and it may be wrong.** It is plausible for shallow association and for tasks that decompose into assembly operations. It is *unproven* for deep compositional credit assignment (many layers of transformation before a reward). Section 5 is designed to test exactly this, early.

### 3.4 What comes for free if the bet holds

- **Continual learning without catastrophic forgetting:** near-orthogonal assemblies mean new learning writes to different subspaces — the interference that causes forgetting is structurally suppressed. (This is why sparse/assembly methods empirically forget less.)
- **Consolidation / "sleep":** offline replay of assembly activations moves fast eligibility into slow weights — the companion reports' whole sleep story becomes a scheduling policy, as v1 argued, but now resting on a representation that actually supports it.
- **Efficiency:** k/N ≈ 1–2% activity means far less work in an event-driven engine — but *not* literally 1–2%. Each event carries queue and cache-miss overhead, so the realized software speedup is a modest constant factor (the ~65–75% band typical of SNNs on CPU/edge); the large multipliers appear only on event-driven neuromorphic hardware. Report efficiency as **work-per-accuracy including per-event overhead**, never as linear-in-activity savings.

---

## 4. Novelty ledger — what is known vs. what is the bet

Intellectual honesty requires separating the borrowed parts from the actual contribution. This is the section v1 should have had.

| Component | Status | Prior art |
|---|---|---|
| Spiking, event-driven substrate | **Known** | Decades of SNN / neuromorphic work |
| Dendritic / compartmental units | **Known, active** | DendSN (2024–25); single-neuron-as-deep-net results (Beniaguev; 2025 L5PC studies) |
| Three-factor / eligibility-trace plasticity | **Known** | e-prop (Bellec 2020); reward-modulated STDP |
| Assembly representation + operations | **Known, under-used in ML** | Assembly Calculus (Papadimitriou, Vempala, Maass, Legenstein) |
| Sparse coding → low interference → less forgetting | **Known empirically** | Sparse/orthogonal continual-learning literature |
| **The synthesis claim: a sparse-assembly representation makes purely-local three-factor learning *sufficient* for credit assignment, so a spiking dendritic substrate can learn competitively without any backprop surrogate** | **The bet — this is what would be new** | Not established; adjacent pieces exist but the sufficiency claim at task scale is open |

So the honest positioning is: **every brick exists; the building does not.** The contribution is a specific, falsifiable architectural hypothesis about *why* the combination should escape the local-learning plateau — plus an engine and an experiment program designed to kill or confirm it fast. That is a legitimate research contribution and, unlike v1, it is not pretending the bricks are the building.

---

## 5. Experiments — attack the crux first

v1's experiment ladder started with trivialities (coincidence detection — known to work) and reached the hard question late. v2 front-loads the two experiments that could *falsify the thesis*, because if they fail there is no point building the rest.

### C1 — The crux: does sparse-assembly local learning beat the local-learning plateau?

- **Setup:** a multi-area assembly network trained with *only* local three-factor plasticity (no surrogate gradient, no BPTT) on a task with genuine compositional depth (e.g. sequential/relational classification, or a class-incremental image benchmark in assembly encoding).
- **Baselines:** (a) the same substrate trained with surrogate-gradient BPTT — the "cheating" upper bound; (b) a fully-local rule in a *dense* representation — the known-to-plateau lower bound; (c) a matched backprop ANN.
- **Hypothesis:** local learning in the sparse-assembly regime lands much closer to the BPTT upper bound than the dense-local lower bound — i.e., the representation, not the gradient, is what mattered.
- **Kills the thesis if:** sparse-assembly local learning plateaus at the same place dense-local does. Then sparsity was not the missing ingredient and the foundation is wrong. **This is the single most important experiment; run it first.**

### C2 — Interference: does the assembly geometry actually suppress forgetting?

- **Setup:** class-incremental stream, no task IDs, no stored raw data. Measure forgetting curve and measured assembly overlap between tasks.
- **Hypothesis:** forgetting is low *and* correlates with low inter-task assembly overlap — establishing the causal mechanism, not just the outcome.
- **Kills the thesis if:** forgetting matches a dense backprop net, or overlap is high despite sparsity (assemblies collapse together).

### C3 — Depth of credit: how many transformations can local credit cross?

- **Setup:** synthetic tasks with a tunable number of compositional stages between input and reward. Measure accuracy vs. depth.
- **Hypothesis:** local-assembly credit assignment holds to some depth `D*`; find `D*`.
- **Value even if modest:** if `D*` is small but > 1, the honest conclusion is "local for shallow, gradient for deep" — a hybrid, which is still a real result.

Only after C1–C3 do the v1-style experiments (temporal coding advantage, energy/work per task, structural plasticity) become worth running. Discipline unchanged: ≥5 seeds, variance and paired tests reported, efficiency always as work-per-accuracy with disclosed sparsity.

---

## 6. Efficiency — the corrected claim

v1 said "orders of magnitude." The honest version:

- **On CPU/GPU:** expect a small-constant-factor win (roughly the ~65–75% energy reductions reported for SNNs on edge tasks), *if* activity is genuinely sparse. Possibly no win, or a loss, if simulating events densely.
- **On neuromorphic hardware** (Loihi-class, event-driven, memory-compute co-located): the large multipliers become physically available, because silence truly costs nothing and weights are not shuttled. These remain **projections until measured.**
- **What we will actually report:** synaptic operations per inference at fixed accuracy, wall-clock, and measured activity sparsity — never a hardware projection stated as a result. The efficiency case rests entirely on C1–C3 succeeding at low activity; without competitive accuracy at ~1–2% activity, there is no efficiency story.

---

## 7. Rust build plan — reprioritized to serve C1 first

Same engine as v1, re-sequenced so the crux experiment is reachable as early as possible.

- **`neura-core` (Rust crate, alongside `Rust_MLKit/`):**
  - Columnar cell state (SoA), CSR sparse synapses, eligibility co-located with weights.
  - **k-winners-take-all inhibition per area** — new priority vs. v1, because it is what creates assemblies (the representation the whole thesis depends on).
  - Hierarchical timing-wheel event queue; lazy per-cell state update so silence is free.
  - Assembly Calculus operations (`project`, `associate`) as first-class engine ops with convergence unit-tests.
  - Deterministic seeded RNG; `pyo3` front-end for experiments; `criterion` benchmarks.

- **Milestone order:**
  1. **M0** — single dendritic cell + analytic unit tests (days).
  2. **M1** — area with k-WTA inhibition; verify assemblies form and `project`/`associate` converge (this is the representation; ~2 weeks).
  3. **M2** — three-factor plasticity + broadcast modulators; **run C1** (the crux) as soon as this lands (~3–4 weeks in).
  4. **M3** — continual-learning harness; run C2, C3.
  5. **M4** — parallel engine, energy/work accounting; efficiency experiments.

If C1 fails at M2, we stop — total spend ~1 month, and we have a clean negative result about sparse-assembly local learning, which is itself publishable.

---

## 8. Risk register — the honest version

**State the prior plainly:** replacing backprop with a local rule that scales has been attempted for decades and has not succeeded. The honest Bayesian prior is that **the crux (G2) most likely returns a negative.** This is not a reason to abandon the program — the entire design is to buy that answer cheaply — but the *expected* outcome is likely-negative, and the enthusiasm elsewhere in this document should be read against that prior.

1. **The crux may fail (highest risk).** Sparse assemblies may not make local credit assignment sufficient beyond shallow tasks. C1/C3 test this in month one. If they fail, the thesis is dead and we report why.
2. **Assemblies may not stay separable under load.** As concepts accumulate, k-WTA competition may force overlap and reintroduce interference (C2 tests this).
3. **Efficiency may be neuromorphic-only.** If so, the software artifact is a research tool, not a deployable speedup — worth stating up front to avoid v1's overclaim.
4. **It may reduce to known work.** If the "sufficiency" bet fails and we fall back on surrogate gradients to make it train, we have merely rebuilt a dendritic SNN — a fine engineering artifact but not a new foundation. We must not quietly cross that line; using BPTT anywhere except as a labeled upper-bound baseline means the thesis has failed.

---

## 9. What changed from v1, in one paragraph

v1 reinvented the parts that were already solved (the spiking dendritic unit, the event message) and asserted away the part that isn't (local credit assignment), then overclaimed efficiency. v2 names credit assignment as *the* foundation, refuses to approximate backprop, and instead bets on a **representation** — sparse distributed assemblies with a convergence theory behind them — that could make purely local learning sufficient. It labels every borrowed component honestly, isolates the one genuinely novel claim, front-loads the experiment that can kill that claim within a month, and corrects the efficiency story to what is actually measurable. Whether the bet is right is unknown — but now the document is about the right problem.
