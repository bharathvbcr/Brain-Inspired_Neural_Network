# BINN

Brain-inspired neural network substrate (greenfield Rust workspace).

**Source of truth:** [`../BINN_Agent_Build_Spec_v8.md`](../BINN_Agent_Build_Spec_v8.md)

Module scopes and API sketches: [`../BINN_Project_Plan_v6.md`](../BINN_Project_Plan_v6.md) §4.

## Workspace layout

| Crate | Layer | Role |
|---|---|---|
| `binn-core` | L2 | Numeric core: buffers, RNG, SIMD, sparse, scan |
| `binn-engine` | L3 | Event-driven substrate: queue, cells, synapses |
| `binn-areas` | L4 | Composition: Area, k-WTA, project/associate, wiring |
| `binn-learn` | L5 | Three-factor plasticity; labeled BPTT baseline only |
| `binn-data` | L6 | Synthetic events, fixed encoders/decoders, metrics |
| `binn-lab` | L7 | Experiment harness, seeds, logging, plots |

Dependency direction is strictly upward: `lab → data → learn → areas → engine → core`.

## Build & test

```bash
cd binn
cargo test --locked --workspace
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
./scripts/gc_checks.sh
```

## Global constraints (CI)

GC1–GC7 are enforced by `.github/workflows/ci.yml` and `scripts/check_gc*.sh`. See the v8 spec §2.
