# Build Audit (v9) — is the BINN substrate built properly?

> **SUPERSEDED (2026-07-23):** C2–R2 are **no longer** empty `fn main(){}` stubs. For current override full-run evidence see [`OVERRIDE_FULL_RUNS.md`](OVERRIDE_FULL_RUNS.md) and [`POST_G2_BUILD.md`](POST_G2_BUILD.md); later harness audits include [`BUILD_AUDIT_v11_RESOLUTION.md`](BUILD_AUDIT_v11_RESOLUTION.md). This file is retained as a historical snapshot — do not cite its “empty stub” claim for the live tree.

**Date:** 22 July 2026
**Auditor:** static review (no Rust toolchain available in the review sandbox — see limits)
**Verdict in one line:** the **scaffold is built well and comprehensively; the headline G2 result is NOT yet trustworthy** — it is confounded by a degenerate control and a stale results note, which by the v8 spec's own standard ("the only failure state is an ambiguous result") must be fixed before the negative can be believed.

---

## 0. FIXES APPLIED (22 Jul 2026) — code changed, host verification still required

The three confounds from §3 have been fixed in source. **These edits were made without a Rust toolchain in the review sandbox and are NOT compiler-verified** — run `cargo test --locked --workspace` and `cargo run -p binn-lab --bin c1` on a host to confirm and to regenerate the results note.

| Issue (§) | Fix applied | File(s) |
|---|---|---|
| 3.2 Task label imbalance (wrap bug) | Rewrote `CoincidenceTask::next_trial` to place peaks with **linear (non-wrapping) distance**; constructor now guarantees `sequence_len ≥ max_lag+2`; strict alternation ⇒ exact 50/50 labels (chance = 0.5). Added `coincidence_task_is_label_balanced` test. | `binn-data/src/datasets.rs` |
| 3.1 Degenerate dense-local control | Both conditions now apply the **same k-winner budget**; dense differs from local only in connectivity (dense all-to-all vs sparse assembly), so it is a fair chance-level control instead of an over-firing constant predictor. | `binn-lab/src/runner.rs` |
| 3.3 No positive/sanity control | Added `run_positive_control` (same pipeline, trivially separable task) surfaced in `C1Report.positive_control_mean` and the results note, with the rule: **low positive-control accuracy ⇒ harness under-powered ⇒ G2 verdict is a void artifact.** | `binn-lab/src/runner.rs` |
| 3.5 Stale results note | `results/c1_g2.md` prepended with a **SUPERSEDED / do-not-cite** banner and re-run instructions. Not regenerated here (no toolchain — numbers must not be fabricated). | `results/c1_g2.md` |
| 3.4 Reference on different data | Verified already addressed on-disk: `gradient_examples` draws from the **same `CoincidenceTask` and same seed convention** as the spiking conditions, so the balance fix flows to it automatically. | (no change) |

**Remaining host steps (cannot be done in this sandbox):** run `cargo test`, `cargo clippy -D warnings`, `./scripts/gc_checks.sh` (GC2/GC3/GC5/GC7), then `cargo run -p binn-lab --bin c1` to produce a trustworthy, current G2 verdict. If the positive control comes back low, fix the harness (thresholds/encoding) before trusting any FAIL.

---

## 1. What is built properly ✅

- **Full workspace, all six crates** (`binn-core … binn-lab`), ~7,300 lines of Rust, correct upward-only dependency direction, module files match spec §3.
- **101 unit tests** across crates (binn-core 22, binn-engine 25, binn-areas 12, binn-learn 15, binn-data 15, binn-lab 12) — U01–U13 have real coverage.
- **Guardrail scripts exist** and the statically-checkable ones pass: **GC1** (no dense matmul / autograd in production path), **GC4** (encoders expose no `train`/`fit`), **GC6** (no undocumented `unsafe`).
- **Harness discipline is real:** the C1 runner sweeps 20 seeds, hashes config, emits GC7 structured logs, computes variance + a paired statistic, and the gate logic has its own unit tests.
- The G2 gate correctly returns **FAIL**, which is *directionally consistent with the prior* that local learning most likely fails.

> Correction to an earlier concern: a first read suggested `runner.rs` had mismatched struct/function schemas and wouldn't compile. That was a **stale file-cache artifact** — the on-disk file is internally consistent (new normalized-gap gate). No action needed there.

---

## 2. What could NOT be verified here ⚠️

The review sandbox has **no Rust toolchain** (rustup blocked, no root for apt), so the following were **not** run and must be confirmed on the host:

```bash
cd binn
cargo test --locked --workspace          # confirm all 101 tests pass
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
./scripts/gc_checks.sh                    # GC2 (deps), GC3 (determinism), GC5 (benches), GC7
```

Until these are green on the host, "it builds and the guardrails pass" is **unconfirmed**, not established.

---

## 3. Why the G2 negative is not yet trustworthy ❌ (the important part)

The result file reports mean accuracy **local-assembly 0.10, dense-local 0.00, gradient-reference 0.87**. Three problems make this an *ambiguous* negative rather than a clean one:

**3.1 The dense-local "plateau control" is degenerate.** In `run_trial`, the dense condition skips k-WTA and lets **every** charged hidden cell fire as a "winner," which over-drives the readout so it predicts a near-constant. That is why dense-local is pinned at *exactly 0.0 on all 20 seeds*. A fair chance-level control on a balanced binary task should sit near **~0.5**, not 0.0. So this control does not measure "local learning without assembly structure plateaus at chance" — it measures an over-firing artifact. The core comparison ("is local closer to the reference than to the plateau?") is therefore built on a broken plateau.

**3.2 The task labels are probably imbalanced.** In `CoincidenceTask::next_trial`, positive examples are built as `t1 = (t0 + offset) % len`; the wrap-around can push an intended-positive pair to an actual lag `> max_lag`, flipping it to label 0. So "chance" is not 0.5, and sub-0.5 accuracies partly reflect a constant predictor against a skewed set rather than failed learning. There is no test asserting ~50/50 label balance in the test split.

**3.3 There is no positive / sanity control.** Nothing in `binn-lab` demonstrates that the local pathway *can* drive the readout correctly on an easy, clearly-separable case. That control is exactly what distinguishes "sparse-assembly local learning genuinely can't do this" (a real negative) from "the harness is mis-tuned so nothing crosses threshold" (an artifact). Without it, a floor result is uninterpretable.

**3.4 Secondary:** the gradient reference (`BpttBaseline::train_coincidence`) is a **separate non-spiking model** — it never touches `Engine`. The code honestly labels it a *reference, not an attainable upper bound*, but the "gap closed" metric still compares across two different model families.

**3.5 Stale artifact:** `results/c1_g2.md` is rendered in the **old format** ("bptt", "paired mean … t=-7.100") while the current code uses a stricter gate (normalized gap-closed lower-CB ≥ 0.5 **and** mean accuracy ≥ 0.65) and new rendering. The committed note predates the current harness and must be regenerated. (Note: the verdict stays FAIL under both gates, because mean local accuracy 0.10 fails the 0.65 floor regardless — so the *direction* is stable, but the note is not current.)

---

## 4. Fix list before the G2 verdict can be trusted

1. **Run the host toolchain checks** (§2) and confirm 101 tests + GC2/GC3/GC5/GC7 pass.
2. **Repair the dense-local control** so it is a fair chance-level baseline — apply the same winner-selection / readout-drive normalization as the local condition, minus the assembly/k-WTA structure, so it lands near ~0.5 rather than saturating.
3. **Fix task label balance** (the wrap bug) and add a test asserting ~50/50 in the test split; print the empirical chance rate in the note.
4. **Add a positive/sanity control** (an easy separable case the local rule provably solves) so a floor result is interpretable.
5. **Regenerate `results/c1_g2.md`** with the current binary and the fixed controls; only then treat the verdict as trustworthy.

---

## 5. Bottom line

Engineering: **solid and on-spec.** Scientific result: **not yet trustworthy.** The FAIL is very likely to survive these fixes (it matches the honest prior), but right now it cannot be cleanly attributed to the thesis rather than to a degenerate control and a skewed task — which is precisely the "ambiguous result" the v8 spec says to prevent. Fix the control, balance the task, add a positive control, re-run, and *then* the negative is publishable.

---

## 6. FIXES APPLIED (round 2, 22 Jul 2026) — bench relocation + guardrail hardening

Follow-up completeness audit found two organizational/guardrail issues (not correctness bugs — the core substrate is genuinely implemented, no stubs in the P0–P2 path). Both fixed:

| Issue | Fix | Files |
|---|---|---|
| Two criterion benches were hidden in **misleadingly-named `tests/determinism.rs`** files (they are benchmarks, not tests), which required `autotests=false` and obscured where the real determinism tests live. | Moved to real bench paths: `binn-core/benches/simd_leak_integrate.rs`, `binn-engine/benches/timing_wheel.rs`; updated `[[bench]]` paths; removed the now-unneeded `autotests=false`; fixed the misleading Cargo comments. | `binn-core/Cargo.toml`, `binn-engine/Cargo.toml`, moved `.rs` files |
| **GC3 could pass vacuously**: `cargo test -p pkg <name>` exits 0 even if `<name>` matches zero tests. | Hardened `check_gc3.sh` to use `-- --exact` and assert `test result: ok. [1-9]+ passed` — a missing/renamed test now FAILS the guardrail. | `scripts/check_gc3.sh` |
| **GC5 only guarded 3 of 5 benches** (simd + timing-wheel unguarded, so they could be faked). | Hardened `check_gc5.sh` to require all five bench files, non-empty, each containing `criterion_main!`. | `scripts/check_gc5.sh` |

Verified statically after the change: all five bench files exist at their declared paths and contain `criterion_main!`; GC1/GC4/GC6 still pass; no `tests/determinism.rs` remains in binn-core/binn-engine (their determinism tests are inline in `src/`).

**Still requires a host run (no toolchain in the review sandbox):** `cargo test --locked --workspace`, `cargo clippy --all-targets -- -D warnings`, `cargo bench --workspace --no-run`, and `./scripts/gc_checks.sh` to confirm everything compiles and all guardrails pass green.

### Completeness verdict
The foundation required to run the crux (P0–P2: core, engine, areas, learn, data, C1 harness) is **fully and genuinely built** — real timing wheel, dendritic cell, three-factor plasticity, wiring/project, assoc_scan; ~7,300 LOC; 100+ tests; no stubs. The downstream experiments **C2/C3/R1/R2 are intentionally empty `fn main(){}` stubs**, correctly gated behind G2 per the spec (do not build P3+ until the crux passes). Nothing is under-built for the current stage.
