# Rust defect register and audit coverage

**Date:** 2026-08-03
**Scope:** rust only. Python is deliberately out of scope and **unswept** — see §4.

This exists because defects were being recorded in whichever result document
happened to touch them, so nothing answered the two questions that matter:
**where do defects cluster**, and **what has actually been swept**. Anecdote is
not coverage. §3 is the honest gap list.

---

## 1. Defects found, by class

Ten defects, and they fall into six classes. The clustering is the useful part:
**five of ten are "the code reports success while measuring nothing"** — not
crashes, not wrong arithmetic, but silence where there should be a complaint.

| # | class | site | what it was | found by | status |
|---|---|---|---|---|---|
| 1 | **silent success** | `shd_instrument.rs` | `non_finite_events` declared, read by the pass predicate, written into all 296 cells — **never incremented**. The `== 0` clause could not be false | trying to replicate a diverging cell | FIXED |
| 2 | **silent success** | `shd_instrument.rs` | `"trained": false` was a hardcoded literal, so every trained-weights probe asserted it was untrained — including the one §4b of the positive control rests on | reading the artifact | FIXED — replaced with a weights fingerprint |
| 3 | **silent success** | `shd_instrument.rs` | `--epochs 0` wrote `mechanical_status: COMPLETE`, empty loss trace, `mean_gradient_norm: 0.0`, accuracy 0.0839 — an untrained model recorded as a finished cell | adversarial CLI probe | FIXED — `require_positive` |
| 4 | **silent success** | `shd_instrument.rs` | `init` accepted `--hidden 0`, `--classes 0`, `--n-inputs 0`, `--n-train 0` | adversarial CLI probe | FIXED |
| 5 | **silent truncation** | `shd_contract.rs` call site | `read_event_cache` clamps to file length, so `--samples 256` against a 100-sample cache silently reports a mean over 100 | reading the source | FIXED at call site — loud error |
| 6 | **numeric** | `shd_matched.rs` | `l2_norm` summed squares in f32; a norm of 1e20 (representable) has a square of 1e40 (not), so finite gradients reported `inf` | a cell that wrote unparseable JSON | FIXED — conditional f64 widening, Gate F 13/13 |
| 7 | **invalid output** | `shd_instrument.rs` | `json_f64` and the scalar `{:.9}` fields emitted the bare token `inf` — not valid JSON, so a diverging cell wrote a file nothing could read | a crashed pilot sweep | FIXED — emits `null` |
| 8 | **silent divergence** | `shd_matched_arms.rs` | two `argmax` implementations: shipped keeps the **last** maximum, the arm path kept the **first**. Diverge on ties, `-0.0`, and NaN. `prediction` feeds three Gate F fields | reading both implementations side by side | FIXED — unified |
| 9 | **correctness** | `shd_matched_arms.rs` | recurrent forward read partially-updated spikes while the backward differentiated the clean model — the gradient was not the gradient of the forward | earlier session | FIXED |
| 10 | **performance** | `shd_matched_arms.rs` | two regressions on `ff+fixed` (+9.5%, then +7%), invisible to every correctness gate because bit-identity held throughout | Gate F per-cell wall times in `runs.jsonl` | FIXED — residual ~4.5% accepted |

### What the clustering says

- **The dominant failure mode is a guard that cannot fire** (#1–#4). Every one
  produced a well-formed artifact claiming success. None would have been caught
  by a correctness test, because nothing was computed incorrectly — the code
  correctly did nothing and correctly said so in a field nobody could read.
- **Parallel implementations drift** (#8, #9). Both live in
  `shd_matched_arms.rs`, which exists precisely to mirror `shd_matched.rs`.
  Every mirrored pair is a defect site until pinned by a test that compares
  them on *degenerate* input, not just typical input.
- **f32 is used for accumulations that can leave its range** (#6). Fixed at the
  one site that overflowed; §3 lists the sites not yet checked.
- **Correctness gates are blind to performance** (#10). Only the wall-time
  history caught it, and only because it had been added that morning for an
  unrelated reason.

## 2. What has been swept

| file | lines | classes swept | result |
|---|---:|---|---|
| `binn-learn/src/shd_matched.rs` | 622 | numeric, silent-success, divergence | 1 defect (#6) |
| `binn-learn/src/shd_matched_arms.rs` | 1197 | all six | 3 defects (#8, #9, #10) |
| `binn-lab/experiments/shd_instrument.rs` | 875 | all six | 5 defects (#1–#4, #7) |
| `binn-learn/src/shd_temporal.rs` | 427 | silent-success, numeric, validation | **0 defects.** `counts_preserved` is genuinely computed and hard-fails; the `Default`-is-true subtlety was already documented with a regression test. One latent assumption pinned (integer counts must sum order-independently for gate 5.1's bit-comparison) |
| `binn-data/src/shd_contract.rs` | — | truncation, validation | 1 defect (#5) |

Tooling: `cargo clippy --all-targets` on `binn-learn`, `binn-lab`, `binn-data`
reports **19 warnings, all style** — no defect in this register was found by
clippy, and none would have been. The classes that matter here are semantic.

## 2b. The previously-unswept crates — swept 2026-08-03

`binn-engine`, `binn-areas`, `binn-core`, `shd_alif.rs`, `shared_bptt.rs` and
the baseline learners were swept with the same two questions that produced §1.

**Class A — fields read but never written (the `non_finite_events` class):
ZERO found.** Every numeric/bool struct field that is read is also written
somewhere. This is the class with the worst track record on the instrument path
(4 of 10 defects) and the unaudited crates are clean on it.

**Class D — panics on data paths: minimal.** `unwrap()`/`expect()` counts are 0
across `binn-engine/src/{cell,lib,parallel,spikelog,synapse,resting}.rs` and
`binn-areas/src/hub.rs`; `queue.rs` has 2.

**Class C — clamping: no true instances.** The `k.min(len)` sites in
`binn-areas/{area,wta}.rs` are k-WTA "select at most k", where clamping is the
defined semantics — not a silently shortened request like `read_event_cache`.

**Class B — f32 sum-of-squares: 7 sites, surveyed and assessed, NOT fixed.**

| site | role | assessment |
|---|---|---|
| `shd_alif.rs:220` `delta_l2` | **load-bearing** — scales `mods` | **fails closed.** `target_rms.is_finite()` is checked before scaling, so an overflow skips normalisation rather than corrupting weights |
| `shd_alif.rs:222` `actual_rms` | divisor of that scale | **partially unguarded.** If `actual_rms` overflows while `target_rms` is finite, `scale` becomes 0 and `mods` is zeroed — silent, and not covered by the existing guard |
| `shd_alif.rs:1044,1046` | same pattern, second site | same |
| `shared_bptt.rs:878` | RMS helper | diagnostic |
| `matched_deep_gradient.rs:490` | post-update RMS | diagnostic |
| `shd_eprop_baseline.rs:846` | RMS closure | diagnostic |

**Deliberately not fixed.** The conditional-widening fix from
`AMENDMENT_2026-08-03_L2_NORM_CONDITIONAL_WIDENING.md` applies verbatim to all
seven. It is not being applied because these files have no bit-identity
regression suite — there is no Gate F for the baselines — so a change here
cannot be shown harmless the way the instrument change could. Fixing untested
code to remove a latent defect can trade a known-dormant bug for an unknown live
one. The `actual_rms` sub-case is the one worth fixing first if anyone does.

**What this sweep did not do.** It applied generic defect-class greps, not a
semantic audit. The event-driven engine's own invariants — timing-wheel ordering,
event-queue correctness, `sparse.rs` CSR/CSC consistency — are untouched, and
those are where a BINN-proper defect would actually live. **BINN proper remains
semantically unaudited**, and no result in this repository currently depends on
it.

## 3. Gaps — what has NOT been swept

**This is the part that makes the register worth keeping.**

| area | lines | swept? | risk |
|---|---:|---|---|
| `binn-engine/`, `binn-areas/`, `binn-core/` | large | **NO** | Not on the instrument path, so no current result depends on them — but BINN proper lives here, and any future BINN claim rests on unaudited code |
| `binn-learn/src/shd_alif.rs` | 1219 | **NO** | Adjacent to the instrument; `MATCHED_DEFAULT_TAU_A`/`BETA_A` mirror it |
| `binn-learn/src/shared_bptt.rs` | 1120 | **NO** | — |
| other ~20 `binn-lab/experiments/*` binaries | ~8000 | **NO** | Several produced results in `results/` that nothing here re-verified |
| all `scripts/*.py` | — | **NO — deferred by instruction** | The python arm is superseded, but `gate_f_rust.py` and the calibration runner are live tooling |

**Specific unswept items inside swept files:**

1. **Other f32 accumulations.** `#6` was fixed at `l2_norm`. `softmax` is
   correctly max-subtracted in both copies, and `ArmAdam::update` already
   accumulates in f64. Not audited: the `evaluate()` rate/silent/saturated
   statistics, and `binn-data` framing accumulations.
2. **Non-finite logits.** `argmax` is now deterministic under NaN, but a cell
   whose logits are non-finite still reports a `prediction` and an `accuracy`
   as if meaningful. `non_finite_events` counts gradient/update excursions, not
   forward ones.
3. **Gate E / G7.** No cross-backend recurrent fixture exists. rust↔python
   recurrent agreement is **argued, not measured** — and this is a
   divergence-class gap (#8, #9), the class with the worst track record here.
   Blocked on the python sweep.
4. **The other Gate F compared fields.** #8 was found because `prediction`
   feeds three of them. The remaining compared fields have not been traced back
   to their producing code the same way.

## 4. How to extend this

The audit that produced #1–#4 was one question asked repeatedly: **"is there a
code path where this reports success without having done the work?"** Applied
to a field, it means: grep for writes, not just reads. Applied to a CLI, it
means: pass the degenerate value and see whether it complains.

The audit that produced #8 was: **"this file mirrors another — do they agree on
input nobody tests?"** Ties, signed zero, NaN, empty, single-element.

Neither requires understanding the science. Both are mechanical and both are
unfinished, per §3.
