# Building BINNs From Scratch — v5

## Own the whole stack. Every gap becomes a component you write, not a dependency you inherit.

**Date:** 22 July 2026
**Builds on:** v2 (assembly + local-learning substrate), v3 (scaling), v4 (the full gap map)
**Decision this document encodes:** build brain-inspired neural networks entirely in code, no PyTorch / CUDA / neuromorphic-SDK dependency. Ecosystem gravity (v4 gap C1/C3) stops being a headwind and becomes an architectural choice: we own every layer, from the numeric core to the benchmark harness.

> **Contingent on G2.** This build plan is worth executing to reach the crux gate cheaply, but everything past P2 assumes the crux (does local learning beat the plateau?) passes. If it fails, most of P3–P5 is moot. Build to learn that answer, not on the assumption it is yes.

---

## 0. What "from scratch" buys you, and what it costs

Building the whole stack yourself is usually a mistake — you re-implement a decade of other people's work badly. It is the *right* move here for one specific reason: **the ANN ecosystem is built around the three things we are rejecting** (dense tensors, global autodiff, synchronous layers). PyTorch's autograd, CUDA's matmul kernels, and the layer abstraction are not neutral infrastructure we can borrow — they *are* the old foundation. Depending on them would quietly drag the design back toward dense backprop. So owning the stack is not vanity; it is how we keep the substrate honest.

The cost is real and must be respected: we give up autodiff, pretrained weights, mature debuggers, and a community. The plan below is structured so that we only rebuild what we *must* own, reuse boring infrastructure (a language, a build system, plotting) freely, and never rebuild something just because we can.

**Rule of thumb for the whole project:** rebuild anything that encodes the old foundation's assumptions; reuse anything that doesn't. A CSV parser is not the enemy. Autograd is.

**Prior, stated plainly:** owning the stack does not raise the odds the core bet works. Replacing backprop with a local rule that scales has been tried for decades without success, so the honest expectation is that the crux gate (G2) most likely returns a *negative*. The from-scratch effort is justified not by optimism but because it is the cheapest way to get a *trustworthy* answer to that question. Build to learn the answer fast, not to be proven right.

---

## 1. The stack we are writing, bottom to top

Seven layers. Each is a crate/module in one Rust workspace (`binn/`), each independently testable. Ecosystem gravity is bypassed because *nothing above the OS and the Rust standard library is assumed.*

```
L7  Experiment harness      — runs the v2/v3 experiments, logs, plots
L6  Benchmark & data        — our own datasets, encoders, decoders, metrics
L5  Learning                — local three-factor plasticity; NO autodiff
L4  Composition             — areas, projection, association, wiring prior
L3  Substrate engine        — event queue, cells, synapses, simulation loop
L2  Numeric core            — our tensors/buffers, RNG, SIMD, parallelism
L1  Platform                — Rust, cargo, OS threads (the only "borrowed" layer)
```

The key inversion vs. a normal ML stack: there is **no autodiff layer and no dense-tensor-graph layer.** Those two are where ecosystem gravity lives, and we simply do not have them. L2 is not a differentiable tensor library; it is plain typed buffers with fast math. Learning (L5) is local and forward-only, so it never needs a backward graph.

---

## 2. Layer by layer — what to build, and the gap each one closes

### L1 — Platform (borrow, don't build)

Rust + cargo + OS threads. This is the only layer we take off the shelf, and it carries none of the old foundation's assumptions. Optional: `rayon` (parallel iterators) and `rand` — small, foundation-neutral. Everything above is ours.

### L2 — Numeric core (`binn-core`)

- Plain **structure-of-arrays buffers** for cell and synapse state (from v2): `Vec<f32>` columns, cache-friendly, no autograd tape.
- Our own **seeded RNG** (`rand_chacha` or hand-rolled) — determinism is non-negotiable for reproducibility (v4 gap C2).
- **SIMD** math via `std::simd` for the per-cell integrate/leak/threshold updates.
- A tiny **sparse-matrix / CSR** representation for connectivity. No dense matmul primitive exists in the whole codebase — deliberately, so we can never accidentally write a dense layer.

*Closes:* the "borrowed tensor stack pulls you back to dense" problem. You cannot write a transformer in a library that has no dense matmul.

### L3 — Substrate engine (`binn-engine`)

The v2 event-driven core, built for real this time:

- **Hierarchical timing-wheel event queue** — O(1) insert/pop (v2 §5.3).
- **Cells** with LIF soma + K dendritic branches + adaptive threshold; **lazy state update** so silent cells cost nothing.
- **Synapses** with weight, trainable delay, and eligibility trace co-located.
- **Main loop**: pop event → deposit on branch → integrate → maybe fire → schedule downstream. Work ∝ events, not cells.
- **Determinism**: a fixed tie-breaking rule for simultaneous events, seeded throughout.

*Closes v4 gaps:* B3 (few-spike/temporal by construction), and the substrate half of A2 (homeostasis lives here).

### L4 — Composition (`binn-areas`)

The v3 scaling machinery, as code primitives:

- **`Area`**: population + k-winners-take-all inhibition. This is what makes assemblies exist. Implement k-WTA as a fast partial-sort / threshold over the area's active cells each cycle.
- **`project(A → B)`** and **`associate(A, B)`**: the Assembly Calculus operations, with **convergence unit-tests** (fire repeatedly, assert the target assembly stabilizes). These are the theory-backed core (v4 gap E2 — we have real theorems here; use them).
- **Wiring prior** `wire(role, position) → connectivity`: a small parametric generator so 10⁶+ nodes are *grown from a compact program*, not stored (v3 Blocker 2, v4 gap F3). Assert >90% of events stay intra-area (v3 Blocker 3).

*Closes:* the scaling-mechanics gaps; gives us the compositional unit of scale.

### L5 — Learning (`binn-learn`) — the autodiff-free training path

This is where we most decisively bypass the ecosystem, and where v4's biggest miss (B1, sequential time-unrolling) gets its workaround.

- **Online local three-factor rule** (v2 §3.3): `Δw = η·e·M − λ·w`. Forward-only, O(1) in *memory* over sequence length, **no backward pass, no unrolled graph, no stored activations**. This structurally dodges A1 (we never differentiate a spike) and the *backward-unroll half* of B1 (we never unroll time for credit) — but it does **not** remove the sequential forward simulation; single-stream throughput stays time-sequential, so throughput comes from neuron/area/stream parallelism, not from time.
- **Broadcast modulators** `M(t)`: reward, novelty, attention — a handful of global scalars, cheap.
- **The parallel-in-time path (v4 §3 #1) — partial, not a clean escape.** The sub-threshold membrane dynamics are linear and *can* be computed with an associative scan, but the **hard reset after a spike is a sequential, data-dependent barrier that provably blocks full parallelization.** So the honest primitive is a **chunked scan** (parallel within a chunk, sequential across chunks), or reset-removal at a modeling cost. We write the scan in L2 and use it where the trade-off pays — but the plan does **not** rely on it to solve throughput; neuron/area/stream parallelism is the real lever.
- **Baseline harness:** a *labeled* surrogate-gradient / BPTT path, built only as an **upper-bound baseline** to measure the local rule against — never as the production learner. (If we ever ship the BPTT path as the real trainer, the thesis has failed; v4 rule.)

*Closes v4 gaps:* A1 (no differentiability needed), B1/B2 (no time-unrolling; parallelize across neurons/streams), A3 is *tested* here (the crux).

### L6 — Benchmark & data (`binn-data`) — own the encode/decode boundary

v4 gap D1/D2 said much of the measured BINN gap lives in data and the spike-encoding boundary. From scratch, we own that boundary instead of inheriting a bad one:

- **Encoders and decoders as first-class, measured components** — not hidden preprocessing. Resolve the tension with the no-autodiff rule explicitly: **for P0–P3 (through the crux), use fixed, hand-designed, information-preserving encoders/decoders** (e.g. latency/population codes with a measured info-loss meter). A "learned, co-trained" encoder is deferred, and if pursued later must pick a lane — either it learns via the *same* local three-factor rule (consistent but unproven), or it is an explicitly-labeled small **autodiff island** at the I/O boundary that is *not* part of the pure substrate. Do not let a learned encoder silently contaminate the crux result.
- **Natively-temporal data first:** audio, event-vision streams, sensor/time-series, synthetic spike tasks — domains where encoding is nearly free. Do **not** start on statically-encoded MNIST and draw conclusions.
- **Our own dataset loaders and synthetic generators** — no dependency on the single-source neuromorphic dataset ecosystem. Generate large synthetic event streams with known structure so tasks have ground truth.
- **Matched-baseline metrics:** every result reported as work-per-accuracy against a well-tuned dense baseline on the *same* task, with sparsity disclosed (v4 gap E1).

*Closes v4 gaps:* D1, D2, E1, and part of C3 (we build reusable pretrained assembly modules here).

### L7 — Experiment harness (`binn-lab`)

- Runs the v2/v3 experiment ladder: C1 (crux: local vs. plateau), C2 (interference/forgetting), C3 (credit depth), then R0→R2 (scaling curve).
- Deterministic, ≥5 seeds, variance + paired tests, auto-generated plots (spike rasters, weight evolution, scaling curves).
- A thin Python view via `pyo3` **only for plotting and analysis** — the model never depends on Python. (This is foundation-neutral reuse, not ecosystem gravity.)

---

## 3. The gap-to-workaround table (the answer to your ask)

Every gap from v4, mapped to the concrete thing we build instead of the thing we'd normally depend on.

| v4 gap | Normal (ecosystem) solution | From-scratch workaround (what we code) | Layer |
|---|---|---|---|
| A1 spikes non-differentiable | surrogate gradients | forward-only local rule; never differentiate a spike | L5 |
| A2 dead neurons | gradient tricks | homeostatic adaptive thresholds keep cells in range | L3 |
| A3 local rules decay with depth | give up, use backprop | sparse-assembly representation makes local credit sufficient (the bet) | L4/L5 |
| A4 long-delay credit | BPTT over long windows | multi-timescale eligibility + neuromodulated consolidation | L5 |
| **B1 sequential time-unrolling** | GPU BPTT | online local learning removes backward-unroll (not forward-sequential); throughput via neuron/area/stream parallelism; chunked scan is partial (reset barrier) | L5/L2 |
| B2 no batch parallelism | huge GPU batches | parallelize over neurons, areas, and many streams | L2/L5 |
| B3 latency vs accuracy | many time steps | temporal/few-spike assembly codes by construction | L3/L4 |
| C1 PyTorch/CUDA dominance | use PyTorch | own numeric core; no dense-matmul primitive exists | L2 |
| C2 reproducibility | framework seeds | deterministic engine, fixed time-base, logged configs | L2/L7 |
| C3 no pretrained backbones | download weights | build reusable pretrained assembly "cortex" modules | L6 |
| D1 no large event data | scrape/convert | own synthetic event generators + natively-temporal domains | L6 |
| D2 encode/decode boundary | fixed rate encoder | fixed info-preserving encoders + info-loss meter for P0–P3; learned encoder only later (local-rule or labeled autodiff island) | L6 |
| E1 dishonest benchmarks | cherry-pick | matched baseline, work-per-accuracy, disclosed sparsity | L7 |
| E2 no theory of advantage | hand-wave | lean on Assembly Calculus convergence/computation theorems | L4 |
| E3 conversion trap | ANN→SNN convert | natively assembly-based; never convert | L4 |
| F2 hardware non-idealities | ignore (sim only) | noise/quantization-aware training; sparse codes are noise-robust | L5 |
| F3 connectivity limits | dense storage | wiring prior generates connectivity; >90% events local | L4 |

Seventeen gaps, seventeen things we write. None of them requires the ANN ecosystem; several are *easier* without it.

---

## 4. Build order — smallest runnable thing first

Do not build the layers top-to-bottom or bottom-to-top. Build the **thinnest vertical slice** that can answer the make-or-break question, then widen.

| Phase | Vertical slice | Layers touched | Answers |
|---|---|---|---|
| **P0** | one cell, unit-tested vs analytic LIF | L2, L3 | dynamics correct |
| **P1** | one area forms stable assemblies; `project`/`associate` converge | L2–L4 | the representation exists (theory holds in code) |
| **P2** | local three-factor learning on one temporal task, no backprop | L2–L6 (thin) | **C1: does local beat the plateau?** — the crux |
| **P3** | continual stream, forgetting measured | +L6/L7 | C2: does assembly geometry suppress forgetting? |
| **P4** | compose 3→100s of areas; fit capability-vs-#areas curve | +L4 scale | **R2: is there a scaling law?** — the whole ballgame |
| **P5** | associative-scan time-parallel path; parallel engine; energy accounting | +L2/L5 | throughput + efficiency, honestly measured |

**Stop-loss:** if P2 (C1) fails, the local-learning bet is dead — stop, ~1 month in, publish the negative. If P4 (R2) plateaus, the honest product is a continual-learning edge system, not a brain — still a win, discovered for GPU-weeks not years.

---

## 5. Design commitments that keep "from scratch" from drifting back

Owning the stack only helps if we hold a few lines. These are the guardrails:

1. **No dense matmul primitive in the codebase.** If you need one, you're building an ANN. The absence is a feature.
2. **No global loss, no backward graph in the production path.** BPTT exists only as a labeled baseline in L5.
3. **Sparsity is enforced, not hoped for.** k-WTA is mandatory in every area; log activity every run; if activity isn't ~1–2%, the efficiency story is void and the run doesn't count.
4. **Encoders/decoders are model, not preprocessing.** They are trained and measured; the boundary is never hidden.
5. **Every claim is matched-baseline + work-per-accuracy + disclosed sparsity.** No toy-benchmark victory laps.
6. **Determinism always.** Seeded, fixed time-base, logged config, ≥5 seeds. A result that doesn't reproduce isn't a result.

---

## 6. Concrete first move

Stand up the workspace next to `Rust_MLKit/`:

```
binn/
  binn-core/     # L2: SoA buffers, seeded RNG, SIMD, CSR sparse — NO dense matmul
  binn-engine/   # L3: timing-wheel queue, LIF+dendrite cells, event loop
  binn-areas/    # L4: Area, k-WTA, project/associate, wiring prior
  binn-learn/    # L5: three-factor plasticity; BPTT baseline (labeled)
  binn-data/     # L6: synthetic event gen, learned encoders/decoders, metrics
  binn-lab/      # L7: experiment runner, seeds, plots (pyo3 for viz only)
```

**P0 deliverable (days):** `binn-core` + `binn-engine` with a single dendritic LIF cell, unit-tested against the analytic solution of the membrane equation, plus a determinism test (same seed → identical spike train). That is the foundation stone; everything else stacks on it.

Then P1 (assemblies form + `project` converges) is the first moment the *theory* becomes running code — and P2 right after is the experiment that tells us whether this whole line of thinking is right.

---

## 7. One-paragraph bottom line

Building from scratch is the correct response to your goal precisely because the gaps that most disadvantage brain-inspired networks — autodiff, dense kernels, fixed encoders, borrowed datasets, pretrained backbones — are all *ecosystem inheritances*, and you are choosing not to inherit them. The plan owns seven thin layers, deliberately omits the two (dense tensors, global autodiff) that encode the old foundation, maps all seventeen v4 gaps to concrete components you write, and sequences the build so the two questions that could kill the idea (C1 local-learning, R2 scaling-law) are answered in the first weeks for the cost of a workstation. The discipline that makes it work is negative: no dense matmul, no backward graph in production, enforced sparsity, honest baselines. Build the single cell first.
