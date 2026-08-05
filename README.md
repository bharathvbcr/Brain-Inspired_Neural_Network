# BINN

**Brain-inspired neural network substrate** — a from-scratch Rust research instrument for one falsifiable question:

> Can a sparse-assembly, locally learned, event-driven network learn competitively **without backpropagation**?

BINN is not a product, ML framework, or neuromorphic deployment stack. It is a deterministic experiment harness with a pre-registered kill-gate. A clean negative is a successful outcome.

## The story

The project started from a belief that sounds obvious: if the brain learns through sparse activity, local plasticity, and continual adaptation, then a network built that way should eventually have real advantages over today's ANNs.

Then the uncomfortable question: **if that is true, why haven't brain-inspired networks already taken over?**

Blaming backpropagation alone is too easy. Dense matmul maps onto GPUs. Large batches expose parallelism. PyTorch, CUDA, pretrained checkpoints, and shared benchmarks let every new project inherit years of engineering. ANNs win because algorithm, hardware, data, and tooling reinforce one another — not because researchers ignored biology.

That reframed the work. BINN is not "make a network look more like a brain." It asks:

> Which biological principles create a measurable computational advantage — and what is the cheapest experiment that could prove the hypothesis wrong?

Scrutiny narrowed the bet. Spiking cells, dendritic compartments, event-driven simulation, and three-factor plasticity already exist as research directions. The unbuilt claim is sharper: **sparse assemblies can make local credit assignment sufficient on difficult tasks.** Adversarial review also forced concrete corrections — spike resets keep forward time sequential; the crux must use fixed encoders (no co-trained autodiff front-end); Assembly Calculus does not prove deep local learning.

So the program became a gated Rust substrate rather than a presumed foundation. Given decades of failed attempts to unseat backprop with local rules, the prior was that the central gate would fail. The point of building was a trustworthy verdict at bounded cost — not confidence theater.

### What the substrate is

Three ingredients, tested together rather than as vibes:

1. **Compartmental, stateful cells** — LIF soma + dendritic branches, lazy integrate-and-fire on an event queue
2. **Sparse timed events and assemblies** — k-WTA areas, project/associate wiring, cost that should scale with activity not cell count
3. **Local three-factor plasticity** — eligibility traces + a modulator; no dense matmul or autograd on the production path

Success metric: work-per-accuracy (including queue and cache overhead) at disclosed sparsity, versus matched gradient and eligibility references. Either a clear pass that licenses scaling, or a reproducible negative that ends the central program. Both count as success; ambiguity from sloppy methodology does not.

### What happened

Gate G2 (experiment C1) returned **FAIL** under protocol v2 hash `c1-118207fbc3eaba53`. Local-assembly learning stayed near chance while gradient and eligibility references succeeded on the same splits. The harness was valid; the FAIL is the scientific decision, not a broken run. The scheduled program stops before P3+; later experiments exist only behind explicit overrides.

## Status


| Gate | Result | Note |
|---|---|---|
| **G2 (C1 crux)** | **FAIL** | Local three-factor / assembly learning did not clear the preregistered gap or accuracy floor |
| **G3 (C2 continual)** | **FAIL** | Local forgetting 0.8948 vs replay baseline 0.2725 |
| C3 v1 credit proxy | **Measured** | Tabular terminal-reward D*=3 vs teacher-forced oracle D*=8 |
| Credit repreregistration | Implemented | Exact-forward C1 + production-engine C3 v2; separate hashes |
| R1 composition | **Additive** | No tested area count compounded capability |
| **G4 (R2 scale)** | **NO-GO** | Degrading curve, slope −0.1924 vs ln(#areas) |
| U21–U23 | Built + measured | Replay, pruning, and resting-state extensions remain exploratory |
| U18–U20 | Built | Partitioned engine, reset-aware scan path, and G5 accounting |

Canonical packaging: [`results/U-NEG_protocol_v2.md`](results/U-NEG_protocol_v2.md)
Config hash: `c1-118207fbc3eaba53` · protocol v2 · n=20 seeds

**Paper package (matched + transfer):** [`results/PAPER_DRAFT.md`](results/PAPER_DRAFT.md) · [`results/PAPER_RESULTS_TABLE.md`](results/PAPER_RESULTS_TABLE.md) · verified replays [`results/PAPER_VERIFY.md`](results/PAPER_VERIFY.md)

```bash
# Matched-arch primary claims (scientific hash replay)
cargo run --locked --release -p binn-lab --bin c1 -- --matched-arch \
  --config-hash c1-match-5dc6822e71229e9e
cargo run --locked --release -p binn-lab --bin c1 -- --matched-dfa \
  --config-hash c1-dfa-c8c4fe0899908b84
cargo run --locked --release -p binn-lab --bin c1 -- --matched-rl \
  --config-hash c1-rl-42eddc9c801308e9
# Live RFB / structured-B transfer
cargo run --locked --release -p binn-lab --bin c1 -- --reinforce-fb \
  --config-hash c1-660401d74db3c88d
cargo run --locked --release -p binn-lab --bin c1 -- --structured-fb \
  --config-hash c1-493ddd56f8714fb6
cargo run --locked --release -p binn-lab --bin c1 -- --structured-fb-teach \
  --config-hash c1-dfab4a7ec19f17c2
```

## Quick start

```bash
cd binn

# Build + test + global constraints (GC1–GC7)
cargo test --locked --workspace
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
./scripts/gc_checks.sh

# C1 / Gate G2 — pilot (not a scientific verdict)
cargo run --locked --release -p binn-lab --bin c1 -- --quick

# C1 full scientific schedule (n=20) → writes results note
cargo run --locked --release -p binn-lab --bin c1 -- --out results/c1_g2.md

# Exact replay from a config hash
cargo run --locked --release -p binn-lab --bin c1 -- \
  --config-hash c1-118207fbc3eaba53 --out results/c1_g2_replay.md
```

Full matrix (build, lint, tests, GC, C1 quick/full/replay):

```bash
./scripts/run_all.sh
# optional: --with-benches --with-plots --with-post-g2
```

## Workspace

Strict upward dependency: `lab → data → learn → areas → engine → core`.

| Crate | Layer | Role |
|---|---|---|
| `binn-core` | L2 | Numeric core: buffers, RNG, SIMD, sparse CSR, scan |
| `binn-engine` | L3 | Event-driven substrate: timing wheel, LIF cells, synapses |
| `binn-areas` | L4 | Composition: Area, k-WTA, project/associate, wiring |
| `binn-learn` | L5 | Three-factor plasticity; labeled BPTT / e-prop baselines only |
| `binn-data` | L6 | Synthetic events, fixed encoders/decoders, metrics |
| `binn-lab` | L7 | Experiment harness, seeds, config hashes, logging, plots |

## Experiments

| Binary | Gate | Default | Purpose |
|---|---|---|---|
| `c1` | G2 | Always runnable | Crux: does local-assembly learning close the gap to a gradient reference? |
| `c2` | G3 | Opt-in | Class-incremental forgetting |
| `c3` | — | Opt-in | Credit-assignment depth |
| `credit-assignment` | — | Always runnable | Exact-forward C1 matched/RPE/e-prop/DFA repreregistration |
| `c3-production` | — | Opt-in | Production-engine C3 v2 depth sweep |
| `r1` | — | Opt-in | Area-count sweep |
| `r2` | G4 | Opt-in | Capability vs #areas scaling curve |
| `extensions` | — | Opt-in | U21 consolidation, U22 pruning, U23 resting-state notes |
| `efficiency` | G5 | Opt-in | U18–U20 + P2 F1/F5: adaptive partitioned engine, reset-barrier headroom, activity≠compute accounting |

Credit-assignment repreregistration: the frozen protocol and interpretation
contract are in
[`results/CREDIT_ASSIGNMENT_PREREGISTRATION.md`](results/CREDIT_ASSIGNMENT_PREREGISTRATION.md).
The completed held-out reports are
[`results/credit_assignment.md`](results/credit_assignment.md) and
[`results/c3_v2_production.md`](results/c3_v2_production.md). All exact-forward
C1 arms failed the unchanged G2 contract; production C3 v2 measured broadcast
and RPE D*=3 versus matched-oracle D*=8. Canonical C1 protocol-v2 remains
unchanged and failed.

C1 conditions (see [`results/c1_g2.md`](results/c1_g2.md)):

- `local-assembly` — three-factor rule + sparse assemblies + k-WTA
- `dense-local` / `dense-matched` — same local rule without assembly structure
- `gradient-reference` — surrogate-LIF BPTT (primary positive control)
- `eligibility-reference` — e-prop-compatible local reference

### Post-G2 overrides (opt-in only)

G2 FAIL stands. These binaries refuse to run unless explicitly enabled; enabling them does **not** reopen the kill-gate.

| Experiment | Docs | Enable |
|---|---|---|
| C2 / U14 | [`results/C2_OVERRIDE.md`](results/C2_OVERRIDE.md) | `--enable-c2` |
| C3 / U15 | [`results/C3_OVERRIDE.md`](results/C3_OVERRIDE.md) | `--enable-c3` |
| C3 v2 / credit | [`results/CREDIT_ASSIGNMENT_PREREGISTRATION.md`](results/CREDIT_ASSIGNMENT_PREREGISTRATION.md) | `--enable-c3-v2` |
| R1 / U16 | [`results/R1_OVERRIDE.md`](results/R1_OVERRIDE.md) | `--enable-r1` |
| R2 / U17 | [`results/R2_OVERRIDE.md`](results/R2_OVERRIDE.md) | `--enable-r2` |
| U21–U23 | [`results/POST_G2_BUILD.md`](results/POST_G2_BUILD.md) | `--enable-extensions` |
| U18–U20 / G5 | [`results/POST_G2_BUILD.md`](results/POST_G2_BUILD.md) | `--enable-efficiency` |

Also accepted: `--override-g2-for <id>` or `BINN_OVERRIDE_G2_FOR=<id>`.

Full post-G2 runs:

```bash
cargo run --release -p binn-lab --bin c2 -- --enable-c2 --out results/c2_g3.md
cargo run --release -p binn-lab --bin c3 -- --enable-c3 --out results/c3_credit_depth.md
cargo run --release -p binn-lab --bin credit-assignment -- --out results/credit_assignment.md
cargo run --release -p binn-lab --bin c3-production -- --enable-c3-v2 --out results/c3_v2_production.md
cargo run --release -p binn-lab --bin r1 -- --enable-r1 --out results/r1_composition.md
cargo run --release -p binn-lab --bin r2 -- --enable-r2 --out results/r2_scaling.md
cargo run --release -p binn-lab --bin extensions -- --enable-extensions --out-dir results
cargo run --release -p binn-lab --bin efficiency -- --enable-efficiency --out results/u20_efficiency.md
```

## Optional plots

CI keeps plotting off. Local figures use **plotters** (optional `plots` feature) — no Python / matplotlib / pyo3. Do not bump scientific defaults or hash `c1-118207fbc3eaba53` for plotting.

```bash
# C1 raster / weight traces
cargo run --locked --release -p binn-lab --features plots --bin c1 -- --quick
# or: ./scripts/run_c1_plots.sh --quick

# Camera-ready paper figures (figM / fig1 / fig3 / graphical abstract)
cargo run --locked --release -p binn-lab --features plots --bin paper-figures -- \
  --out results/runs/2026-07-23-paper-hard-both/figures
```

Optional Polars table harvest: `--features tables` (see `binn_lab::harvest`).

Figure copies: [`results/plots/`](results/plots/) and camp `figures/`.

### Offline spike / assembly viewer

Opt-in JSONL trace export plus a self-contained HTML viewer. Export is off unless `--export-trace` is set (scientific schedules stay lean).

```bash
# C1: spikes, k-WTA, assembly overlap, weights / eligibility
cargo run -p binn-lab --bin c1 -- --quick --export-trace results/c1_trace.jsonl

# Optional: one condition + seed (default path when flag alone: results/c1_trace.jsonl)
cargo run -p binn-lab --bin c1 -- \
  --quick --isolate-condition local-assembly --seed 1 --export-trace

# R1: static topology / coupling flow only (no Engine spikes)
cargo run -p binn-lab --bin r1 -- --enable-r1 --quick --export-trace
```

Open [`results/viewer.html`](results/viewer.html) in a browser → Load JSONL via the file picker. R1 traces have no spike replay; the flow panel uses static edge thickness from `nnz` / coupling.

## Constraints (CI)

GC1–GC7 are enforced by `.github/workflows/ci.yml` and `scripts/check_gc*.sh` (see build spec §2):

| ID | Rule |
|---|---|
| GC1 | No dense matmul / autograd on the production path |
| GC2 | No external ML framework deps (`torch` / `candle` / …) |
| GC3 | Same seed ⇒ identical outputs (fingerprint tests) |
| GC4 | Fixed encoders through the crux |
| GC5 | Hot paths have compiling `criterion` benches |
| GC6 | No undocumented `unsafe` |
| GC7 | Every run logs activity sparsity |

## Docs

| Document | Role |
|---|---|
| [`BINN_Agent_Build_Spec_v8.md`](BINN_Agent_Build_Spec_v8.md) | **Source of truth** for agents — goals, GC rules, work units |
| [`BINN_Project_Plan_v6.md`](BINN_Project_Plan_v6.md) | Module scopes and API sketches (§4) |
| [`results/U-NEG_protocol_v2.md`](results/U-NEG_protocol_v2.md) | Publishable G2 FAIL packaging |
| [`results/c1_g2.md`](results/c1_g2.md) | Canonical C1 numbers |
| [`results/SENSITIVITY_PROTOCOLS.md`](results/SENSITIVITY_PROTOCOLS.md) | Tier-B confound probes (do not reopen G2) |
| [`results/CREDIT_ASSIGNMENT_PREREGISTRATION.md`](results/CREDIT_ASSIGNMENT_PREREGISTRATION.md) | Exact-forward C1 and production C3 v2 preregistration |
| [`results/ANE_FIT_AUDIT.md`](results/ANE_FIT_AUDIT.md) | Closed decision: skip maderix/ANE (no NPU backend) |

License: MIT OR Apache-2.0.
