# BINN post-G2 engineering build

**Status:** the P3–P5 roadmap is implemented behind explicit overrides.

The canonical scientific decision remains Gate G2 FAIL under
`c1-118207fbc3eaba53`. These units were built because the project owner
explicitly requested engineering completion while collaborating on the same
checkout. They are exploratory extensions, not a retroactive G2 pass.

## Implemented units and current evidence

| Unit | Artifact | Current result |
|---|---|---|
| U14 / C2 / G3 | [`c2_g3.md`](c2_g3.md) | **FAIL** — local forgetting 0.8948 vs replay baseline 0.2725; overlap intervention direction held |
| U15 / C3 v1 | [`c3_credit_depth.md`](c3_credit_depth.md) | **MEASURED tabular proxy** — terminal-reward D*=3, teacher-forced oracle D*=8 |
| U16 / R1 | [`r1_composition.md`](r1_composition.md) | **ADDITIVE** — compound fraction 0 |
| U17 / R2 / G4 | [`r2_scaling.md`](r2_scaling.md) | **NO-GO** — degrading curve, log slope −0.1924 |
| U18 | [`u20_efficiency.md`](u20_efficiency.md) / [`f1_f5_systems.md`](f1_f5_systems.md) | Adaptive graph-partitioned delta stepping; thin ticks skip rayon; sequential spike/work parity |
| U19 | [`u20_efficiency.md`](u20_efficiency.md) / [`f1_f5_systems.md`](f1_f5_systems.md) | Reset-aware associative-scan training; barrier fraction + scan headroom (F1) |
| U20 / G5 | [`u20_efficiency.md`](u20_efficiency.md) | Full matched work-per-accuracy decision + F5 activity≠compute ratios |
| U21 | [`u21_consolidation.md`](u21_consolidation.md) | Matched no-sleep (`0.8000` forgetting), exact (`0.8000`), generated (`0.0000`), offline-local (`0.8000`), and `sleep-provenance-consolidation` (`0.0000` forgetting, `1.0000` final accuracy without sleep label disclosure) |
| U22 | [`u22_pruning.md`](u22_pruning.md) | Magnitude, age, eligibility, and random pruning at exact matched sparsity |
| U23 | [`u23_resting.md`](u23_resting.md) | Stimulus-free dynamics against rate-, activity-, and spectrum-matched nulls |
| U24 / WSP | [`WEIGHT_SPACE_PROGRAMMING.md`](WEIGHT_SPACE_PROGRAMMING.md) | **DIAGNOSTIC & SURGERY** — direct weight-space reading and zero-gradient surgery: finite $\theta = 1.0$ natural spiking ($0.4700 \to 0.9887$), compositional depth ($D^* = 3 \to 8$), continual forgetting ($0.8948 \to 0.1760$), 700-ch `SHD` LIF homeostasis ($0.0500 \to 0.0861$), and QK saturation recovery ($0.0043 \to 2.6614$) |
| U25 / Waves 1–4 | [`PERF_AUDIT_2026-11-02_WAVES_1_3.md`](PERF_AUDIT_2026-11-02_WAVES_1_3.md) | **SYSTEMS ACCELERATION & MEGA-KERNELS** — $\sim 8.1\times$ process / $9.1\times$ condition cumulative end-to-end speedup ($844\text{ ms} \to 104.3\text{ ms}$ process / $92.4\text{ ms}$ condition on $c1\text{ --micro}$, $N=10\text{k}$), $43.96\times$ FF / $10.22\times$ recurrent ALIF factored mega-kernels, $8.72\times$ sparse 700-ch e-prop/DFA, $18.93\times\text{–}35.25\times$ single-CB 64-tick Metal GPU recurrent fusion, $90.57\times$ lazy $O(D)$ cell reset, and $\sim 606,000\times$ atomic delay queries while preserving 100% bit-exact numerics |

Quick/PILOT artifacts with the same harnesses use the `_quick.md` suffix.

## Verification

The combined collaborator build passed the repository's full mechanical
contract on 2026-07-23:

- `cargo fmt --all -- --check`
- `cargo clippy --locked --workspace --all-targets -- -D warnings`
- `cargo test --locked --workspace` (211 tests, 0 failed)
- `./scripts/gc_checks.sh` (GC1–GC7 passed)
- `bash -n scripts/run_all.sh scripts/gc_checks.sh`
- `git diff --check`

The DevCouncil repository map and symbol graph were regenerated after the
implementation.

## Reproduce

```bash
cargo build --locked --release --workspace --bins

target/release/c2 --enable-c2 --out results/c2_g3.md
target/release/c3 --enable-c3 --out results/c3_credit_depth.md
target/release/r1 --enable-r1 --out results/r1_composition.md
target/release/r2 --enable-r2 --out results/r2_scaling.md
target/release/extensions --enable-extensions --out-dir results
target/release/efficiency --enable-efficiency --out results/u20_efficiency.md
# also writes results/f1_f5_systems.md (+ .csv with --features tables)
cargo bench -p binn-engine --bench f1_parallelism
```

For a bounded smoke matrix:

```bash
./scripts/run_all.sh --with-post-g2
```

## Interpretation boundary

- C2–R2 have distinct protocol/config hashes and do not alias C1.
- Quick outputs are PILOT only.
- U21–U23 are first full exploratory schedules, not independent replications.
- Modeled work is never called hardware energy.
- Resting-state dynamics are not called a biological Default Mode Network.
- A full engineering build does not erase the negative G2/G3/G4 evidence.
