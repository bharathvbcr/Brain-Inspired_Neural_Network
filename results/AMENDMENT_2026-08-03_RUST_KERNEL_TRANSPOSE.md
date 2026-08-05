# Amendment — rust instrument kernel, input-drive transposition

**Registered:** 2026-08-03, after the change, before any affected cell is run.
**Amends:** `shd_instrument_v4`, rust backend, `binn-learn/src/shd_matched_arms.rs`.
**Supersedes nothing.** Unlike the 2026-08-02 python amendment, this change is
bit-identical and no completed cell is superseded.

```
claim_axis: Integrity
object_under_test: A performance change to the rust instrument kernel, and
  whether it preserves the 216 completed rust cells bit-for-bit.
may_claim: The change is bit-identical on every cell tested, at both geometries,
  both widths tested, and T=100/500, including the per-epoch loss and gradient
  traces. Measured 2.8x faster at the anchor geometry.
must_not_claim: That any accuracy, verdict, or scientific conclusion changed;
  that bit-identity is proven for the 206 cells not re-run.
```

---

## 1. What changed

Two edits to `loss_and_gradient_arm`, both performance-motivated, neither
scientific. Feed-forward arms only; the recurrent path is untouched.

**A. Input-drive transposition.** `w_in` is stored `[hidden, n_inputs]`, so the
drive loop

```rust
for h in 0..hidden {
    let row = h * n_inputs;
    for &(channel, count) in frame { current += w_in[row + channel] * count; }
}
```

gathers scattered elements from a *different row per hidden unit*. Each timestep
therefore sweeps the whole `w_in` matrix (287 KB at the anchor) in a strided
pattern, which makes the loop memory-bound rather than compute-bound. The kernel
now pivots to a transposed copy `[n_inputs, hidden]`, so each event becomes a
contiguous `hidden`-length AXPY that vectorises. The backward scatter into
`gradient.w_in` is pivoted the same way and folded back to canonical layout once
per sample.

**B. Dead-store elimination.** `thresholds` and `previous_spike_log` are each
`t_steps * hidden` — 1 MB apiece at the anchor, allocated and filled per sample.
`thresholds` is the constant `MATCHED_THRESHOLD` unless the arm adapts, and
`previous_spike_log` is read only by the recurrent backward. Both are now
allocated only for the arms that need them.

## 2. Why this is bit-identical, and why that argument is not sufficient on its own

Each `current[h]` receives the same addends in the same order — decay first, then
events in frame order — because only the *loop nesting* changed, not the
sequence of operations on any individual accumulator. The lanes are independent
chains, so vectorising across `h` cannot reassociate any one of them, and rustc
does not enable fast-math. On the backward side, for any fixed `(h, channel)` the
addends still arrive in reverse-`t` order and in frame order within a timestep.
The fold back to `[hidden, n_inputs]` is pure data movement. Adam, `l2_norm`,
`add_assign` and the weight file format never see anything but the canonical
layout.

**That argument is exactly the argument that failed on 2026-08-02.** The python
kernel change was also believed safe on reasoning plus fixture parity, and a full
cell disproved it: BLAS reassociated where a sequential `+=` did not, a one-ulp
difference crossed the hard spike threshold, and a single flipped spike compounded
through Adam. The difference here is that no summation order changes at all — but
that distinction is only worth what the measurement says, so the claim rests on
§3, not on §2.

## 3. Evidence

**Unit level.** `cargo test -p binn-learn shd_matched_arms` — 5/5 pass, including
`ff_fixed_matches_shipped_reference`, which asserts the arm path reproduces the
shipped `shd_matched::loss_and_gradient` bit-for-bit.

**Cell level, against recorded cells.** `scripts/gate_f_rust.py` (new, §4)
re-runs completed rust cells from the pinned initialization artifacts and demands
every scientific field match bit-exactly.

All 13 are measured and passing **against the installed binary** —
`target/release/shd-instrument`, `sha256 6f6dbbc9fd58…`, which is the binary that
will produce future cells. `results/shd_instrument_v4/gate-f-rust/report.json`:
`"status": "PASS"`, `"cells": 13`, `"failures": 0`.

The suite was run twice — once against the scratch build before installation and
once against the installed path afterwards — and passed 13/13 both times.

| cells | budget | result |
|---|---|---|
| 6 cheapest recorded (`fixed-t100` / `published-10ms`, adjacent-sum-5, h128, 3 seeds) | e20 | **6/6 BIT_IDENTICAL** |
| `fixed-t100 / channels-700 / h128` | e20 | **BIT_IDENTICAL** |
| `fixed-t100 / channels-700 / h512` | e20 | **BIT_IDENTICAL** |
| `published-2ms / adjacent-sum-5 / h512` | e20 | **BIT_IDENTICAL** |
| `published-2ms / channels-700 / h512` | e20 | **BIT_IDENTICAL** |
| `fixed-t100 / adjacent-sum-5 / h128` | e100 | **BIT_IDENTICAL** |
| `published-10ms / adjacent-sum-5 / h128` | e100 | **BIT_IDENTICAL** |
| `published-2ms / adjacent-sum-5 / h512` — **the anchor** | e100 | **BIT_IDENTICAL** |
| **total** | | **13/13 PASS** |

**Cell level, direct A/B at the anchor geometry** (`published-2ms` /
`adjacent-sum-5` / h512 / s5170001, 3 epochs, full 8156/2264 split), shipped
binary against optimised binary:

| field | shipped | optimised |
|---|---:|---:|
| accuracy | 0.483657244 | 0.483657244 |
| mean_loss | 2.272878203 | 2.272878203 |
| mean_gradient_norm | 0.350686894 | 0.350686894 |
| mean_update_rms | 0.001290036 | 0.001290036 |
| mean_firing_rate | 0.138151444 | 0.138151444 |
| majority_prediction | 0.143109541 | 0.143109541 |

Every field in the cell schema matched, and so did `epoch_mean_loss` and
`epoch_mean_gradient_norm` element-for-element. Those traces are the sensitive
quantity: a single flipped spike anywhere in 3 × 8156 samples would separate them.

Note that the trace comparison comes from this A/B, **not** from Gate F against
recorded cells — the recorded cells predate the convergence telemetry and carry
no traces, so `gate_f_rust.py` reports `compared_traces: 0` for them and falls
back to the scalar fields. Both forms of evidence are needed; neither subsumes
the other.

**Coverage limit, stated plainly.** The suite is 13 of 216 cells; the other 203
are argued, not measured. Coverage spans both geometries (n_inputs 140 and
700), h128 and h512, T=100 and T=500, and both epoch budgets — the axes the
transposition shape depends on, plus the axis the *failure mode* depends on.

The epoch axis is the one that matters most and was almost missed: the defect
that disqualified the 2026-08-02 python kernel change compounds with epochs, so a
suite consisting only of `e20` cells is weak evidence regardless of how many
cells it contains. The e100 set therefore includes
`rust__published-2ms__adjacent-sum-5__h512__e100__s5170001` — the anchor itself,
which is the headline configuration at the widest layer, the longest budget in
the matrix, and the finest temporal resolution.

## 4. A gap this change exposed: the rust arm had no runnable gate

`AMENDMENT_2026-08-02` §5.3 puts Gate F in force for the rust arm. The registered
implementation does not support it — `gates_ef.py::gate_f_cell` raises
*"gate-f regresses the python arm; the rust arm is checked by gate-e"* — and Gate
E is unimplemented, raising *"GATE E BLOCKED - no arm fixtures yet"*
(`GATE_EF_WORK.md` G7). So the obligation existed with nothing behind it.

`scripts/gate_f_rust.py` implements it: `--cell`, `--cheapest N`, `--all`,
`--binary`, writing to `gate-f-rust/` and touching nothing under
`initialization/` or `cells/`.

**One design note, learned by hitting it.** The first version wrote only
`gate-f-rust/report.json`, holding the latest invocation. A later 7-cell run
against the installed binary silently destroyed the 13-cell report this
amendment cites — quiet record loss of exactly the kind this apparatus exists to
prevent. Every run is now also appended to `gate-f-rust/runs.jsonl`, and each
record carries `binary_sha256`, so evidence accumulates and is attributable to a
specific binary rather than to a path that gets rebuilt. Per-cell outputs were
never at risk; only the summary was.

## 5. A provenance hole, reported not fixed

`relevant_source_fingerprint()` was recomputed after this change and is
**unchanged** at `4b85606d11fb3d523bd421afcfe312327e5b739029c28d8958ee3678364af7d9`.

`SOURCE_PATHS` (`runner.py:54`) covers `binn-learn/src/shd_matched.rs` but **not
`binn-learn/src/shd_matched_arms.rs`**, which is the module `shd_instrument.rs`
actually routes through, nor `binn-learn/src/shd_temporal.rs`. The compute kernel
that produces every rust number can therefore be rewritten without moving the
fingerprint that is supposed to detect exactly that.

This is the mirror image of the problem in
`MEASUREMENT_2026-08-03_SHD_BUDGET_AND_ERRATA.md` §3, where the fingerprint is
*too broad* and invalidates a third-party reference it should not guard. Both
point at the same fix — scope the fingerprint to what each artifact actually
depends on — and both require a registered amendment written **before** the
change, since altering `SOURCE_PATHS` changes what every stored manifest means.
No change is made here.

## 6. Measured effect

Anchor geometry, `published-2ms / adjacent-sum-5 / h512`, 3 epochs, wall clock:

| binary | wall |
|---|---:|
| shipped | 86 s |
| + transposition | 34 s |
| + dead-store elimination | **31 s** |

**2.8× at the anchor.** Both binaries were timed under identical load (two e400
probe cells running concurrently), so the ratio is meaningful but the absolute
numbers are inflated relative to an idle machine.

Independently confirmed on a full recorded cell: the Gate F re-run of
`rust__published-2ms__adjacent-sum-5__h512__e100__s5170001` — the anchor, 100
epochs over the full 8156/2264 split — completed in **778 s** against the
**2127 s** recorded for the same cell, while producing bit-identical output.
That is **2.7×**, agreeing with the 3-epoch A/B. The same cell ran in 691 s on an
otherwise idle machine (**3.1×**); the spread between 691 s and 795 s across
repeats is load, not variance in the optimisation.

Projected on the `PREREG_2026-08-02_SHD_TEMPORAL_INFORMATION` §9 budget, the
ff+fixed half of that campaign drops from ≈4.7 h to ≈1.7 h. The rec+alif half is
**not** improved — the recurrent path is deliberately untouched, and in any case
it is blocked by `DEFECT_2026-08-03_RECURRENT_ARM_FORWARD_BACKWARD_MISMATCH.md`.

## 7. Obligations

1. ~~Do not install until the in-flight e400 cells finish.~~ **DISCHARGED.** The
   two e400 probe cells were run to completion against the shipped binary — they
   are therefore attributable to a single binary — and the optimised build was
   installed at `target/release/shd-instrument` only afterwards, then re-verified
   13/13. The previous shipped binary is preserved outside the repo; rebuild from
   git history if a comparison is ever needed.
2. **Widen Gate F coverage before the next matrix-scale campaign.** 10/216 is
   enough to license this amendment, not enough to license a new campaign.
3. ~~Do not extend this transposition to the recurrent arms — their
   `previous_s[j]` read aliases the `previous_s[h]` write.~~ **That aliasing was
   a bug and is now fixed** (`DEFECT_2026-08-03_RECURRENT_ARM_FORWARD_BACKWARD_MISMATCH.md`).
   The recurrent arms keep the single-pass loop for now, but that is a
   performance choice rather than a correctness constraint: with the drive
   reading a clean `s(t-1)` snapshot, the loop can be split and transposed the
   same way whenever the rec arms become the bottleneck — which they will, at
   ~6.5× the ff+fixed cost.

---

**Artifacts.**
`binn-learn/src/shd_matched_arms.rs` — kernel.
`scripts/gate_f_rust.py` — rust-arm Gate F.
`results/shd_instrument_v4/gate-f-rust/report.json` — the passing runs quoted in §3.
