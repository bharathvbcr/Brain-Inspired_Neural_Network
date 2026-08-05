# BINN C1 audit resolution (v11)

**Date:** 22 July 2026  
**Scope:** Resolve confirmed C1 correctness, fairness, reproducibility, and host-verification gaps from `BUILD_AUDIT_v10.md`.

## Resolved

1. **False-PASS gate route:** `gap_closed` is clamped to `[0, 1]`; a seed contributes zero unless `gradient_reference - dense_local >= 0.15`. The threshold is part of `Config` and its hash. Regression tests cover both weak-reference inflation and ratios above one.
2. **One-sided local credit:** wrong predictions now depress the selected readout and positively teach the correct readout, with eligibility cleared between the two modulatory updates.
3. **Dead readout spike path:** incoming readout weights are scaled so one connected hidden winner can cross the shared `theta = 1` threshold. A direct engine-level regression test proves the readout spikes.
4. **Idle-edge decay:** weight decay applies only when a synapse has nonzero eligibility, so inactive readout edges are not repeatedly bled toward zero.
5. **Unreachable sparse readout fallback:** required readout edges originate from hidden cells, where winner spikes occur in the decision window.
6. **Matched-control topology:** the dense control now uses the same edge roles as local-assembly (input-to-hidden, hidden-to-hidden, hidden-to-readout; no readout-to-hidden edges). Parameter matching preserves all shared I/O edges, samples only hidden-to-hidden edges when the budget permits, uses the experiment seed, and hits the exact requested `nnz`.
7. **Misleading matched-control label:** results now call this arm parameter-matched with measured compute disclosure; they no longer claim a separately compute-matched arm.
8. **Protocol/hash aliasing:** C1 scientific protocol version `2` is mixed into config hashes and printed in results. Algorithm changes can no longer silently reuse the pre-fix config hash.
9. **Repository quality gates:** Rust formatting drift and the strict-Clippy `manual_is_multiple_of` finding are fixed.
10. **RPE ambiguity:** hard two-sided `+1/-1` reward is explicitly retained as the protocol; soft reward-prediction error remains a future experimental variant, not a partially applied fix.

## Verification

- `cargo fmt --all -- --check` — PASS
- `cargo clippy --locked --workspace --all-targets -- -D warnings` — PASS
- `cargo test --locked --workspace` — PASS (139 tests)
- `./scripts/gc_checks.sh` — PASS (GC1-GC7)
- C1 quick protocol-v2 run — `PILOT`, valid harness
- C1 full protocol-v2 run — `FAIL`, valid harness
- Config-hash replay — identical seeds, per-seed accuracies, summaries, and verdict

## Protocol-v2 result

- Config hash: `c1-118207fbc3eaba53`
- Mean local-assembly accuracy: `0.4912`
- Mean dense-local accuracy: `0.5000`
- Mean gradient-reference accuracy: `0.8938`
- Mean eligibility-reference accuracy: `1.0000`
- Mean parameter-matched dense accuracy: `0.5000`
- Mean normalized gap closed: `0.0189`
- Lower 95% normal-approximation bound: `-0.0048`
- Positive control: `0.9488` (valid; threshold `0.9000`)
- Activity sparsity: `0.0156` (valid band `[0.0050, 0.0300]`)
- Verdict: **FAIL**

Artifacts:

- `results/runs/2026-07-22/c1_g2_quick_v11.md`
- `results/runs/2026-07-22/c1_g2_full_v11.md`
- `results/runs/2026-07-22/c1_g2_full_v11_replay.md`

## Deliberate non-fixes / future scope

- The preregistered `z = 1.96` normal approximation remains unchanged. Switching to Student-t after observing results would alter the protocol; a future protocol may preregister that choice.
- C2, C3, R1, and R2 remain planned work units, not C1 defect fixes. The v8 program stops at G2 after a trustworthy negative, so they are not scheduled by this result.
- Optional plotting: enable with `--features plots` plus the documented Python ≤3.13 venv (`./scripts/setup_plots_venv.sh` / `./scripts/run_c1_plots.sh`). See `results/U-NEG_protocol_v2.md` for attached C1 figures.
