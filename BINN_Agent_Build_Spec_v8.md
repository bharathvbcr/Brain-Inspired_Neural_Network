# BINN Agent Build Spec (v8)

## An executable specification for building the substrate with coding agents

**Date:** 22 July 2026
**Builds on:** v2 (design), v3 (scaling), v4 (gaps), v5 (from-scratch stack), v6 (project plan), v7 (audit — all corrections folded in)
**Audience:** coding agents (and their orchestrator). This document is written to be handed to agents as the source of truth. Each work unit is self-contained, has an interface, and has a machine-checkable acceptance test.

> **Prior, read before starting.** The scientific bet most likely fails at gate G2. That is expected and fine — the point is to reach that verdict cheaply and trustworthily. Agents must optimize for *correctness and reproducibility of the experiment*, not for making the substrate look good. A clean negative is a successful outcome.

---

## 1. Goals (what "done" means at each level)

**North-star goal.** Determine, reproducibly, whether a sparse-assembly, locally-learned, event-driven substrate can learn competitively **without backpropagation** — and if so, whether it composes to a scaling curve.

**Concrete goals, in priority order:**

1. **G-BUILD** — a compiling, tested, deterministic Rust substrate (engine + areas + local learner + data + harness) with CI guardrails.
2. **G-CRUX (the one that matters)** — run experiment C1 and get a trustworthy verdict: what fraction of the dense-local-to-gradient-reference performance gap does local-assembly learning close?
3. **G-CONTINUAL** — measure forgetting under class-incremental streams (C2) and credit-assignment depth (C3).
4. **G-SCALE** — compose areas and fit the capability-vs-#areas curve (R2).
5. **G-EFFICIENCY** — honest work-per-accuracy vs. a matched baseline, overhead included.

**Non-goals (agents must NOT do these):** implement a production dense matmul / autograd engine; add a GUI; optimize prematurely; beat transformers on language; target neuromorphic hardware; introduce external ML framework dependencies into the BINN production path. An isolated test-only framework runner is permitted solely to validate the hand-written gradient reference against an independent implementation; it must share frozen data/config hashes and may not be linked into BINN. If a task seems to require anything broader, STOP and flag the orchestrator.

---

## 2. Global constraints (enforced, not aspirational)

These are checked in CI. A PR that violates any is rejected automatically.

| ID | Constraint | CI check |
|---|---|---|
| GC1 | No dense matmul / autograd in production path | grep test: no symbol matching `matmul`/`dense_layer`/`autograd`/`backward(` outside `*_baseline.rs` |
| GC2 | No external ML framework deps | `cargo tree` must not contain `torch`/`tch`/`candle`/`burn`/`dfdx` |
| GC3 | Determinism | CI executes a real same-seed output-fingerprint test in every stateful crate; file presence is insufficient |
| GC4 | Fixed encoders through crux | `binn-data` exposes no `train`/`fit` method on `Encoder`/`Decoder` until milestone P4+ |
| GC5 | Every hot path benchmarked | engine-step, k-WTA, and plasticity `criterion` benches exist **and compile** in CI |
| GC6 | No `unsafe` without documented invariant | clippy + a doc-comment lint |
| GC7 | Sparsity logged every run | harness refuses to emit results without an activity-sparsity field |

**Language/toolchain:** Rust stable, `cargo` workspace. Allowed crates: `rayon`, `crossbeam`, `rand`/`rand_chacha`, `bytemuck`, `criterion`, `proptest`, `serde`, `pyo3` (viz only, `binn-lab`). Nothing else without orchestrator approval.

---

## 3. Repository layout (create exactly this)

```
binn/
├── Cargo.toml                # workspace
├── README.md                 # points here
├── .github/workflows/ci.yml  # fmt, clippy, test, GC1-GC7 checks
├── binn-core/                # L2 numeric core
│   ├── src/{lib,buffer,rng,simd,sparse,scan,time}.rs
│   └── tests/
├── binn-engine/              # L3 event-driven substrate
│   ├── src/{lib,cell,synapse,queue,engine,spikelog}.rs
│   └── tests/
├── binn-areas/               # L4 composition
│   ├── src/{lib,area,wta,assembly,project,wiring,hub}.rs
│   └── tests/
├── binn-learn/               # L5 learning
│   ├── src/{lib,three_factor,modulators,eligibility}.rs
│   ├── src/bptt_baseline.rs  # labeled baseline ONLY (GC1 exempt)
│   └── tests/
├── binn-data/                # L6 data + fixed encoders + metrics
│   ├── src/{lib,synth,encoder,decoder,metrics,datasets}.rs
│   └── tests/
└── binn-lab/                 # L7 harness
    ├── src/{lib,runner,config,logging,plots}.rs
    └── experiments/{c1,c2,c3,r1,r2}.rs
```

Dependency direction strictly upward (lab → data → learn → areas → engine → core). No cycles.

---

## 4. Work units

Each unit is agent-assignable. Format: **ID · title · crate · depends-on · interface · acceptance test · done-when.** Acceptance tests are the contract — an agent's work is done when they pass in CI, not before.

### Phase P0 — foundation stone

**U01 · Numeric buffers & seeded RNG · `binn-core` · —**
- Interface: `Buffer<T>` (SoA column), `Csr { row_ptr, col }`, `Rng` (ChaCha, seeded), `Tick = u64`.
- Acceptance: RNG reproduces a golden sequence from a fixed seed; `Buffer` round-trips; `proptest` on CSR neighbor iteration.
- Done-when: tests green; GC3 determinism test present.

**U02 · SIMD cell math · `binn-core` · U01**
- Interface: `fn simd_leak_integrate(v: &mut [f32], input: &[f32], tau: &[f32], dt: Tick)`.
- Acceptance: SIMD result matches scalar reference within 1e-6 over random inputs.
- Done-when: `criterion` bench exists (GC5); parity test green.

**U03 · Chunked associative scan · `binn-core` · U01**
- Interface: `fn assoc_scan<F>(xs, combine: F) -> Vec<State>` — **chunked; parallelizes the linear sub-threshold recurrence only.**
- Acceptance: matches a sequential fold exactly on the linear recurrence; a doc-comment states plainly that the spike **reset is a sequential barrier** and this scan does NOT parallelize across reset events.
- Done-when: parity test green; doc note present (per v7 F1).

**U04 · Timing-wheel event queue · `binn-engine` · U01**
- Interface: `TimingWheel::{insert(at: Tick, ev), pop_earliest() -> Option<(Tick, Event)>}`.
- Acceptance: O(1) amortized (bench shows flat per-op cost vs. queue size); deterministic tie-break for equal `Tick`; property test vs. a naive binary-heap reference gives identical pop order.
- Done-when: bench + parity test green.

**U05 · Dendritic LIF cell · `binn-engine` · U02, U04**
- Interface: `Cell { v, theta, tau_m, branches: [f32; K], last: Tick }`; `Engine::{inject, step_until, spikes}`; lazy state update (integrate leak only when touched).
- Acceptance: **single-cell membrane matches the analytic LIF solution** within tolerance for constant current; an impulse decays and does not create permanent branch drive; adaptive threshold rises and relaxes; weighted spikes propagate across a 2→3-cell network at exact delays; partial stepping followed by an intermediate injection preserves queue order; same seed ⇒ identical spike train.
- Done-when: analytic membrane + impulse-decay + delayed-network-propagation + partial-horizon regression + determinism tests are green. **This is Gate G0. A subthreshold-only analytic check is not sufficient.**

### Phase P1 — the representation exists

**U06 · Area + k-WTA inhibition · `binn-areas` · U05**
- Interface: `Area { cells: Range<CellId>, k }`; per-cycle k-winners-take-all (fast partial-select / threshold).
- Acceptance: exactly ≤k cells fire per area per cycle; activity logged; a test asserts measured activity ≈ k/N.
- Done-when: k-WTA correctness test green; GC7 wiring in place.

**U07 · Assembly `project` with convergence · `binn-areas` · U06**
- Interface: `fn project(engine, src: &Assembly, dst: &Area) -> Assembly` is the event-driven scientific path; `project_reference` is a labeled algebraic oracle only; `Assembly { members: Vec<CellId> }`.
- Acceptance: event-driven source activation routes weighted/delayed impulses through `Engine`; k-WTA consumes measured delivered charge; repeated projection converges (successive overlap >0.9) across a disclosed seed sweep; CSR `nnz` is invariant throughout. The oracle cannot satisfy G1.
- Done-when: convergence test green. **This is Gate G1.**

**U08 · `associate` + wiring prior · `binn-areas` · U07**
- Interface: `fn associate(engine, a, b)`; `fn wire(role, pos, prior) -> Csr`.
- Acceptance: association potentiates existing random edges only and raises inter-assembly overlap; wiring is deterministic, respects a disclosed per-cell fan-out cap, runs in O(N × fan-out), and yields **>90% intra-area routed events** under a disclosed nonuniform spike workload.
- Done-when: both tests green.

### Phase P2 — the crux (highest priority after build)

**U09 · Eligibility traces · `binn-learn` · U05**
- Interface: `de/dt = -e/τ_e + STDP(pre, post)` stored on synapses.
- Acceptance: pre-before-post raises trace, post-before-pre lowers it; decay matches closed form.
- Done-when: STDP-sign test + decay test green.

**U10 · Three-factor plasticity · `binn-learn` · U09**
- Interface: `trait Learner { fn update(&mut self, engine, m: Modulators) }`; `ThreeFactor { eta, lambda, tau_e }`; `Modulators { reward, novelty, attention }`.
- Acceptance: on a coincidence task, weights move only where eligibility ∧ modulation coincide; **O(1) memory in sequence length** (assert no per-timestep allocation growth); ablations (no modulator / no eligibility) degrade learning as predicted.
- Done-when: learning + ablation + memory-flatness tests green.

**U11 · Gradient-trained BPTT reference (labeled) · `binn-learn` · U05 · GC1-exempt file**
- Interface: `BpttBaseline` in `bptt_baseline.rs` — surrogate-gradient learner used only as a **reference**, never called an upper bound.
- Acceptance: trains the identical frozen task; analytic gradients pass finite-difference checks; its config/data/parameter budget is disclosed; parity is checked against an isolated trusted implementation or a preregistered known-task target. A header states it must never be the production learner.
- Done-when: reference learns, gradient check passes, label says `gradient-reference`, and validation evidence is committed.

**U12 · Fixed encoders/decoders + metrics · `binn-data` · U01**
- Interface: `trait Encoder { fn encode(&self, x) -> Vec<SpikeEvent>; fn info_loss(&self) -> f32 }` (latency/population codes; **no `train`/`fit`** — GC4); `trait Decoder`; `Metrics::{work_per_accuracy, forgetting, sparsity, overlap}`.
- Acceptance: encoder is deterministic and reversible-enough that info_loss is measured and reported; efficiency uses disjoint counters: `source_spikes × routing_cost + synaptic_deliveries × delivery_cost + cell_updates × integration_cost + plasticity_updates × update_cost`. It must also report wall time and peak memory; this proxy must not be called energy.
- Done-when: metric unit tests green; info-loss meter reports a number.

**U13 · C1 experiment + harness · `binn-lab` · U10, U11, U12**
- Interface: `experiments/c1.rs`; `Runner` with a five-seed pilot and a final seed count chosen by power analysis (default ≥20), config hashing, structured logs, and plots via `pyo3` (viz only).
- Acceptance: one command reproduces C1 from a config hash; all conditions use the **same example identities in the same per-seed frozen train/test splits**. Cell, parameter, update, wall-time, and peak-memory budgets are disclosed; the primary comparison is repeated under preregistered parameter- and compute-matched budgets. Report local-assembly, dense-three-factor, a strong eligibility-based local reference (e-prop-compatible where applicable), and the gradient reference. Define per-seed `gap_closed = (A_local − A_dense_local) / (A_gradient_reference − A_dense_local)`; a non-positive denominator contributes zero positive evidence and invalidates a positive claim if it is systematic.
- **Done-when (Gate G2, KILL):** PASS requires the preregistered lower 95% confidence bound of `gap_closed` to exceed 0.5 **and** the absolute accuracy floor; defaults are stored in the hashed config. Otherwise FAIL → **stop and write U-NEG.** Five seeds can produce only a pilot verdict. Either outcome is project progress.

### Phase P3 — continual learning

**U14 · Class-incremental stream + C2 · `binn-data`+`binn-lab` · U13**
- Acceptance: stream has no task IDs and stores no raw data; forgetting curve computed; overlap is tested mechanistically with preregistered interventions (shuffle overlap while holding activity fixed, and force high/low overlap), not correlation alone.
- Done-when (Gate G3): forgetting is below a capacity/replay-matched gradient baseline and the overlap intervention changes forgetting in the predicted direction.

**U15 · Credit-depth task + C3 · `binn-lab` · U13**
- Acceptance: synthetic tasks with tunable compositional depth; report accuracy vs. depth; identify `D*` (max depth local credit crosses).
- Done-when: `D*` measured and reported.

**U21 · Offline consolidation / sleep replay · `binn-learn`+`binn-lab` · U14**
- Acceptance: compare no sleep, exact replay under a fixed memory budget, generative replay under the same budget, and offline local-plasticity consolidation; disclose replay compute and prohibit test-data generation.
- Done-when: retention/transfer curves and ablations identify whether offline consolidation adds value beyond matched replay.

**U22 · Active forgetting / synaptic pruning · `binn-learn`+`binn-lab` · U21**
- Acceptance: pruning is local and budgeted; compare magnitude, age, eligibility, and random pruning at matched sparsity; measure recovery, interference, and retained capacity.
- Done-when: a preregistered pruning rule improves the retention-capacity frontier or is rejected.

**U23 · Resting-state dynamics · `binn-engine`+`binn-lab` · U21**
- Acceptance: stimulus-free dynamics are generated without labels; quantify metastability, assembly reactivation, and transition structure against rate-, activity-, and spectrum-matched null models. Do not call it a Default Mode Network without homologous network-level evidence.
- Done-when: spontaneous dynamics are characterized and their causal contribution to consolidation is ablated.

### Phase P4 — scaling

**U16 · Multi-area composition + R1 · `binn-areas`+`binn-lab` · U08, U13**
- Acceptance: compose 3→10 areas via wiring prior + hubs; test whether composition **compounds** capability vs. merely adds.
- Done-when: R1 result reported.

**U17 · Scaling curve R2 · `binn-lab` · U16**
- Acceptance: sweep to hundreds of areas; **fit capability-vs-#areas curve**; report shape (healthy / plateau / degrade).
- Done-when (Gate G4, DECISION): curve fitted. PASS (healthy, non-plateauing) = *justifies the next order of magnitude* — NOT proof it continues to 10⁴–10⁶ areas (v7 F6). Plateau = redirect to edge-continual-learning product.

### Phase P5 — throughput & efficiency (optional, post-decision)

**U18 · Parallel engine · `binn-engine` · U05 · delta-stepping across independent cells; graph-partitioned.**
**U19 · Chunked scan training path · `binn-learn` · U03 · partial time-parallelism; reset stays sequential.**
**U20 · Work accounting · `binn-lab` · disjoint operation counters + measured wall time/peak memory vs. matched baseline. Hardware energy may be reported only when directly measured.**
- Done-when (Gate G5): work-per-accuracy beats matched dense baseline at disclosed sparsity. (No kill gate; optimization.)

---

## 5. Dependency & gate graph

```
U01─┬─U02─┐
    ├─U03 │        (P0)
    └─U04─┴─U05 ──[G0]── U06 ──U07 ──[G1]── U08          (P1)
                                    │
                         U09─U10────┤
                         U11────────┤─ U12 ─ U13 ──[G2 KILL]  (P2)
                                              │
                                    U14──[G3]─┤
                                    U15───────┤                (P3)
                                    U21─U22───┤
                                      └─U23───┤
                                              │
                                    U16 ─ U17 ─[G4 DECISION]   (P4)
                                              │
                              U18 U19 U20 ─[G5]               (P5)
```

**Hard rule for the orchestrator:** do not schedule any P3+ unit until G2 passes. If G2 fails, the remaining work is a single unit: **U-NEG** — write the reproducible negative-result note (config hashes, plots, the plateau comparison) and stop.

---

## 6. How to run the agents (orchestration guidance)

- **Parallelizable now (no cross-deps):** U01, U04 can start immediately; U02/U03 after U01. Assign to separate agents.
- **Serialize the gates.** G0, G1, G2 are barriers — one agent owns the gate's acceptance test and must confirm green before dependents start.
- **One agent = one work unit = one PR.** Each PR must pass CI (GC1–GC7 + the unit's acceptance test) before merge. No PR merges on a red gate.
- **Every unit produces a results note** when it involves an experiment (U13–U20): committed markdown with the config hash, the numbers, variance, and plots. A unit without its note is not done.
- **Verification agent (recommended):** a separate agent independently re-runs each gate's acceptance test from a clean checkout and confirms the result hash matches. This guards GC3 and the trustworthiness of G2/G4 specifically.
- **When blocked or tempted to violate a GC:** stop and escalate to the orchestrator rather than working around a constraint. The constraints are the experiment's validity; breaking one silently invalidates the result.

---

## 7. Definition of Done (per unit and overall)

**Per unit:** public API documented; unit + property + determinism tests green; `criterion` bench for any hot path; no `unsafe` without a documented invariant; CI green including GC1–GC7; results note committed if it is an experiment.

**Overall project:** either (a) G2 passes and G4 yields a fitted scaling curve with a clear go/no-go for the next order of magnitude, or (b) a clean, independently-reproduced negative at G2 or G4 with a committed note. Both are project success. The only failure state is an **ambiguous or non-reproducible** result — which the gates, seeds, and verification agent exist to prevent.

---

## 8. First tasks to hand out (copy-paste ready)

1. **Agent A → U01** (`binn-core` buffers/RNG/CSR + determinism test). 
2. **Agent B → U04** (`binn-engine` timing wheel + parity-vs-heap test). 
3. After U01: **Agent A → U02, U03** (SIMD + chunked scan, with the reset-barrier doc note). 
4. After U02+U04: **Gate owner → U05** (dendritic LIF cell + analytic-membrane test = **G0**). 

Everything downstream follows the dependency graph in §5. Reach **G2 (U13)** as fast as the graph allows — it is the whole point; the honest expectation is a negative, and getting there cheaply and reproducibly is the definition of winning.
