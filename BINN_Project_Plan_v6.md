# BINN Project — Detailed Build Plan & Scopes (v6)

## The executable plan: modules, scopes, milestones, gates

**Date:** 22 July 2026
**Builds on:** v2 (substrate), v3 (scaling), v4 (gap map), v5 (from-scratch stack)
**Purpose:** turn the v5 blueprint into an execution plan a builder can work against — each module has an explicit scope (in *and* out), a public interface, acceptance criteria, and a place in a gated timeline with defined kill conditions.

> **Contingent on G2, and a likely-negative prior.** This is a detailed plan, not a validated roadmap. Everything past the crux gate (P2/G2) assumes local-assembly learning actually beats the plateau — a bet that, given decades of failed attempts to unseat backprop with local rules, most likely returns a *negative*. The plan is rational because it reaches that verdict cheaply, not because the verdict is expected to be yes.

---

## 1. Objectives, non-objectives, and the single success metric

**Objective.** Build, in Rust from scratch, a sparse-assembly, locally-learned, event-driven neural substrate, and use it to answer two empirical questions before any large spend: (C1) does local learning in a sparse-assembly regime beat the known local-learning plateau, and (R2) does composing assembly areas trace a usable scaling curve.

**Non-objectives (explicitly out of scope for this project).**
- Beating transformers on language/GPU benchmarks (v3: wrong turf).
- Neuromorphic-hardware deployment (simulation only; hardware is a later program).
- A general-purpose ML framework, a GUI, or a product. This is a research instrument.
- Biological fidelity beyond what the computation needs.

**The one metric that governs go/no-go.** Work-per-accuracy — *including per-event overhead* (queue ops, cache misses), not a naive linear-in-activity estimate — at a disclosed activity sparsity, on natively-temporal tasks, versus a matched well-tuned baseline. The software efficiency win is expected to be a modest constant factor; the large multipliers are neuromorphic-hardware-only and out of scope here. Everything is reported this way; nothing else counts as evidence.

**Definition of project success.** Either (a) C1 and R2 both pass → justified to scale, or (b) a clean, reproducible negative at C1 or R2 → publishable result, program redirects. Both are successes. The only failure is an ambiguous result from sloppy methodology.

---

## 2. Team, cadence, and conventions (assumptions)

This plan is written to be executable by **1–2 engineers** (scale timelines by team size). Conventions that apply to every module:

- **Language/tooling:** Rust stable, `cargo` workspace, `criterion` for benchmarks, `cargo test` + `proptest` for property tests. Plotting/analysis via a thin `pyo3` bridge used *only* in `binn-lab`.
- **Definition of Done (applies to every module):** public API documented; unit tests pass; property/determinism tests pass; a `criterion` benchmark exists for any hot path; no `unsafe` without a documented invariant; CI green.
- **Determinism gate (global):** same seed ⇒ bitwise-identical spike trains and weights, enforced by a test in every crate that has state.
- **Review artifact per milestone:** a short results note (plots + numbers + config hash) committed alongside code. No milestone is "done" without its note.

---

## 3. Workspace and module map

```
binn/
  binn-core/     L2  numeric core: buffers, RNG, SIMD, sparse, scan
  binn-engine/   L3  event-driven substrate: queue, cells, synapses, loop
  binn-areas/    L4  composition: Area, k-WTA, project/associate, wiring
  binn-learn/    L5  learning: three-factor plasticity; BPTT baseline (labeled)
  binn-data/     L6  data: synthetic event gen, learned encoders/decoders, metrics
  binn-lab/      L7  harness: experiment runner, seeds, logging, plots
```

Dependency direction is strictly upward (L7→…→L2); no cycles. Each crate below has: **Scope (in)**, **Scope (out)**, **Public interface**, **Acceptance criteria**.

---

## 4. Module scopes

### 4.1 `binn-core` (L2) — numeric core

**Scope (in).** Structure-of-arrays typed buffers for cell/synapse state; a seeded deterministic RNG; SIMD helpers for elementwise leak/integrate/threshold; a CSR sparse-connectivity type; a **chunked associative-scan** primitive (for *partial* time-parallel training of the linear sub-threshold dynamics only — the hard reset is a sequential barrier, so this parallelizes within chunks, not across them; v4 §3 #1); fixed-point *time* representation (integer ticks) to guarantee determinism.

**Scope (out).** No dense matrix-multiply. No autograd/tape. No neural concepts (no "neuron" here — just numbers). No file I/O.

**Public interface (sketch).**
```rust
pub struct Buffer<T> { /* SoA column */ }
pub struct Csr { row_ptr: Vec<u32>, col: Vec<u32> }           // connectivity
pub struct Rng(/* chacha state */);                            // seeded, deterministic
pub fn simd_leak_integrate(v: &mut [f32], input: &[f32], tau: &[f32], dt: Tick);
pub fn assoc_scan<F>(xs: &[State], combine: F) -> Vec<State>;  // chunked; sub-threshold only
pub type Tick = u64;                                           // integer time
```

**Acceptance criteria.** SIMD path matches scalar reference within 1e-6; `assoc_scan` matches a sequential fold exactly; RNG reproduces a golden sequence; **no symbol named `matmul`/`dense` exists** (grep test in CI — the guardrail from v5 §5).

---

### 4.2 `binn-engine` (L3) — event-driven substrate

**Scope (in).** Hierarchical **timing-wheel** event queue (O(1) amortized); **Cell** = LIF soma + K dendritic branches + adaptive threshold with **lazy state update**; **Synapse** = weight + trainable delay + eligibility trace (storage only; the update rule lives in L5); the **main event loop**; deterministic simultaneous-event tie-breaking.

**Scope (out).** No learning (traces are stored/decayed here, but `Δw` is L5). No area/assembly concepts (L4). No k-WTA (L4). No I/O.

**Public interface (sketch).**
```rust
pub struct Engine { cells: Cells, syn: Synapses, queue: TimingWheel, t: Tick }
impl Engine {
    pub fn inject(&mut self, cell: CellId, branch: u8, at: Tick);
    pub fn step_until(&mut self, t: Tick) -> SpikeLog;   // event-driven advance
    pub fn spikes(&self) -> &SpikeLog;
}
pub struct Cell { v: f32, theta: f32, tau_m: f32, branches: [f32; K], last: Tick }
```

**Acceptance criteria.** Single cell matches the analytic LIF membrane solution (P0 test); **cost scales with events, not cells** (benchmark: hold cells fixed, vary activity, show linear-in-events wall-clock); determinism test passes; a 10⁴-cell network runs stably for 10⁶ ticks without drift.

---

### 4.3 `binn-areas` (L4) — composition

**Scope (in).** **Area** (population + shared inhibition); **k-winners-take-all** per cycle (fast partial-select); Assembly Calculus **`project`** and **`associate`** with convergence guarantees tested; the **wiring prior** `wire(role, position) → Csr` that *generates* connectivity compactly; hub areas; an event-locality assertion (>90% intra-area). *Note:* the Assembly-Calculus guarantees tested here are *convergence* (assemblies stabilize) and separation of **well-separated** classes — they do **not** by themselves establish competitive learning on hard compositional tasks; that is the G2 bet, not a property this module proves.

**Scope (out).** No learning rule (L5 supplies plasticity; L4 calls it). No task/data (L6). No specific network topology hard-coded — topology comes from the wiring prior.

**Public interface (sketch).**
```rust
pub struct Area { cells: Range<CellId>, k: usize /* WTA cap */ }
pub fn project(engine: &mut Engine, src: &Assembly, dst: &Area) -> Assembly;
pub fn associate(engine: &mut Engine, a: &Assembly, b: &Assembly);
pub struct Assembly { members: Vec<CellId> }                  // sparse, size k
pub fn wire(role: AreaRole, pos: Pos, prior: &WiringPrior) -> Csr;
```

**Acceptance criteria.** Firing an assembly into a fresh area **converges** to a stable assembly (overlap between successive rounds → >0.9 within N rounds — the theory holds in code, P1 gate); two random assemblies of size k in N have overlap ≈ k²/N (sanity of sparsity); wiring prior reproduces identical connectivity from the same seed; measured intra-area event fraction >90% (v3 Blocker 3).

---

### 4.4 `binn-learn` (L5) — learning

**Scope (in).** The **online three-factor rule** `Δw = η·e·M − λ·w` (forward-only, O(1) in *memory* over sequence length — removes the backward-unroll half of B1, not the sequential forward simulation); **broadcast modulators** (reward, novelty, attention); multi-timescale eligibility; a **partial (chunked) parallel-in-time path** built on `binn-core::assoc_scan`; a **labeled** surrogate-gradient/BPTT learner used *only* as an upper-bound baseline. Throughput on a single stream stays time-sequential; parallelism comes from neurons, areas, and independent streams.

**Scope (out).** BPTT as a production learner (forbidden — using it to actually train the deliverable means the thesis failed, v4 rule). No task specifics. No dense gradients anywhere in the local path.

**Public interface (sketch).**
```rust
pub trait Learner { fn update(&mut self, engine: &mut Engine, m: Modulators); }
pub struct ThreeFactor { eta: f32, lambda: f32, tau_e: f32 }   // the real learner
pub struct BpttBaseline { /* labeled upper bound only */ }
pub struct Modulators { reward: f32, novelty: f32, attention: f32 }
```

**Acceptance criteria.** `ThreeFactor` learns a coincidence/temporal task with **no backward pass** (P2 gate); BPTT baseline runs and produces the labeled upper bound on the same task; ablations (remove modulator, remove eligibility) degrade learning as predicted; memory use is **O(1) in sequence length** for the local learner (proves B1 is dodged).

---

### 4.5 `binn-data` (L6) — data, encoders, metrics

**Scope (in).** **Synthetic event-stream generators** with known ground-truth structure; **fixed, information-preserving encoders/decoders for P0–P3** (model components with an **information-loss meter**, *not* learned — this avoids contaminating the crux with a trainable I/O boundary and resolves the tension with the no-autodiff rule); loaders for a few natively-temporal real datasets (audio/event-vision/time-series); the **metrics module** (accuracy, forgetting curve, work-per-accuracy, activity sparsity, assembly overlap). A *learned* encoder is deferred to post-crux and must be either local-rule-trained or an explicitly-labeled autodiff island.

**Scope (out).** No statically-encoded image benchmarks as primary evidence (allowed only as smoke tests, never headline). No dependency on external neuromorphic dataset packages if avoidable.

**Public interface (sketch).**
```rust
pub trait Encoder { fn encode(&self, x: &Sample) -> Vec<SpikeEvent>; fn info_loss(&self) -> f32; }
pub trait Decoder { fn decode(&self, spikes: &SpikeLog) -> Prediction; }
pub struct SyntheticStream { /* parametric, seeded, ground-truthed */ }
pub struct Metrics; // work_per_accuracy(), forgetting(), sparsity(), overlap()
```

**Acceptance criteria.** Synthetic tasks have provable ground truth and tunable difficulty/depth (needed for C3); encoder info-loss is measured and reported; every metric has a matched-baseline comparison built in; a fixed public config reproduces a dataset byte-for-byte.

---

### 4.6 `binn-lab` (L7) — experiment harness

**Scope (in).** Experiment runner for C1/C2/C3 and R0→R2; seed sweeps (≥5); config hashing; structured logging; auto-plots (spike rasters, weight evolution, forgetting curves, scaling curves); the results-note generator.

**Scope (out).** No model logic. No training rules. Python touched **only** for plotting/analysis via `pyo3`.

**Acceptance criteria.** One command reproduces any experiment from a config hash; outputs include variance and paired tests; plots regenerate deterministically; a results note is emitted per run.

---

## 5. Milestones, deliverables, gates

Vertical slices (v5 §4), each with an explicit **gate** — a pass/fail decision that authorizes (or halts) the next phase.

### P0 — Foundation stone *(scope: `binn-core` + `binn-engine`, single cell)*
- **Deliverables:** SoA buffers, seeded RNG, timing-wheel queue, one dendritic LIF cell.
- **Tests:** membrane matches analytic solution; determinism (seed → identical spike train).
- **Gate G0:** cell dynamics correct + deterministic. *If fail:* fix before anything else — no gate bypass.

### P1 — The representation exists *(scope: + `binn-areas`)*
- **Deliverables:** Area, k-WTA, `project`, `associate`, wiring prior.
- **Tests:** assembly-formation convergence; overlap ≈ k²/N; intra-area event fraction >90%.
- **Gate G1:** assemblies form and `project` converges in code. *If fail:* the representational primitive doesn't work → the whole v2 line is in doubt; investigate before building learning.

### P2 — The crux *(scope: + `binn-learn` + thin `binn-data`/`binn-lab`)* — **KILL GATE**
- **Deliverables:** three-factor learner; one temporal task; BPTT baseline (labeled); C1 experiment.
- **Tests / results:** local learning vs. (a) BPTT upper bound, (b) dense-local lower bound.
- **Gate G2 (kill):** local-assembly learning lands **closer to the BPTT bound than to the dense-local plateau**. *If fail:* **stop the program.** Publish the negative (sparse assemblies do not make local credit sufficient). ~2–3 months elapsed (realistic), ~6 weeks best case. This is the single most important gate, and the honest prior is that it fails.

### P3 — Continual learning *(scope: widen `binn-data`/`binn-lab`)*
- **Deliverables:** class-incremental stream (no task IDs, no raw-data storage); C2 + C3 experiments.
- **Gate G3:** forgetting materially below a matched backprop net, *and* forgetting correlates with low inter-task assembly overlap (mechanism confirmed, not just outcome).

### P4 — The scaling curve *(scope: `binn-areas` at scale)* — **DECISION GATE**
- **Deliverables:** compose 3 → hundreds of areas via the wiring prior; R1 (does composition compound?) then R2 (fit capability-vs-#areas).
- **Gate G4:** R2 curve is a healthy (non-plateauing) law over the tested range. *If pass:* this **justifies investing in the next order of magnitude** — it does *not* prove the law continues to 10⁴–10⁶ areas (curves can bend), so read it as "keep going," not "trillion-node question settled." A separate program handles the next rung. *If plateau:* honest product = a world-class continual-learning edge system; redirect. Either way, decided for GPU-weeks, not years.

### P5 — Throughput & efficiency, honestly *(scope: `binn-core` scan + parallel engine)*
- **Deliverables:** associative-scan time-parallel training path; parallel event engine (delta-stepping); energy/work accounting.
- **Gate G5:** work-per-accuracy beats matched dense baseline at disclosed sparsity; time-parallel path trains faster without changing results. (Optimization phase — no kill gate.)

---

## 6. Timeline (1–2 engineers)

**Read these as an aggressive best case, and plan in months.** Building a deterministic event engine, areas, a novel learner, a data/encoder layer, a harness, *and* running trustworthy experiments is realistically a 6–9 month effort to reach a defensible G2/G4 — the week columns below assume everything goes right and nothing needs re-derivation. P0→P2 alone is more likely 2–3 months for one engineer than the ~6 weeks shown.

| Phase | Focus | Best-case (cumulative) | Realistic (cumulative) |
|---|---|---|---|
| P0 | foundation stone | ~week 1 | ~weeks 1–2 |
| P1 | assemblies form | ~weeks 2–3 | ~month 1 |
| P2 | **crux / kill gate** | ~weeks 4–6 | ~months 2–3 |
| P3 | continual learning | ~weeks 7–9 | ~month 4 |
| P4 | **scaling curve / decision gate** | ~weeks 10–14 | ~months 5–7 |
| P5 | throughput & efficiency | ~weeks 15–18 | ~months 8–9 |

Front-loaded on purpose: the two gates that can end the program (G2, G4) come first, so the kill/decision verdicts are the *earliest* things you buy. Realistic worst case is a clean negative for ~2–3 months of one engineer (best case ~6 weeks) — still cheap for the question being asked.

---

## 7. Risk register (execution risks, distinct from the scientific risks in v2/v3)

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Scientific bet fails at G2/G4 | Medium–High | Ends program | *This is by design* — gates make it cheap and informative, not a loss |
| Timing-wheel / determinism bugs corrupt results silently | Medium | High | Determinism test in every crate; golden-trace regression tests |
| Sparsity not achieved → efficiency story void | Medium | High | k-WTA mandatory; activity logged every run; runs void if not ~1–2% |
| Scope creep toward a general framework | Medium | Medium | Non-objectives in §1; "no dense matmul" CI guardrail |
| Encoder hides/creates the result (D2) | Medium | High | Fixed info-preserving encoders through P2 (no trainable I/O boundary at the crux); info-loss meter; learned encoders only post-crux |
| Hot-path performance too slow to reach P4 scale | Medium | Medium | `criterion` benches from P0; SIMD + timing wheel early; profile before scaling |
| Single-engineer bus factor / context loss | Medium | Medium | Results note + config hash per milestone; documented invariants |

---

## 8. Traceability — plan back to the gaps

Each v4 gap is owned by a module and closed at a gate, so nothing is hand-waved:

| v4 gap | Owned by | Closed/tested at |
|---|---|---|
| A1 non-differentiability | `binn-learn` | G2 (learns with no backward pass) |
| A2 dead neurons | `binn-engine` | G0/G2 (homeostasis) |
| A3 depth decay | `binn-areas`+`binn-learn` | **G2 / G4** |
| B1 time-unrolling | `binn-learn`+`binn-core` | G2 (O(1)-memory, backward-unroll half only) / G5 (chunked scan, partial) |
| B2 batch parallelism | `binn-core` | G5 |
| C1/C3 ecosystem/backbones | whole stack / `binn-data` | by construction / G3 |
| C2 reproducibility | all | global determinism gate |
| D1/D2 data & encoding | `binn-data` | G2/G3 (info-loss meter) |
| E1 benchmark honesty | `binn-lab` | every gate (matched baseline) |
| E2 theory of advantage | `binn-areas` | G1 (convergence in code) |
| E3 conversion trap | `binn-areas` | by construction (native) |
| F2 hardware non-idealities | `binn-learn` | G5 (noise-aware, optional) |
| F3 connectivity | `binn-areas` | G1 (wiring prior, >90% local) |

---

## 9. First action

Create the `binn/` cargo workspace next to `Rust_MLKit/` with the six empty crates and CI (fmt, clippy, test, the "no dense matmul" grep guardrail). Then build **P0**: `binn-core` buffers + RNG + timing wheel, and `binn-engine`'s single dendritic LIF cell, with the analytic-membrane test and the determinism test. That is the whole first week, and it is the stone every gate above stands on.
