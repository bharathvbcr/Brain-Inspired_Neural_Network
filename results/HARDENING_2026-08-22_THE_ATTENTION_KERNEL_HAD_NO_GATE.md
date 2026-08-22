# Gate F had no attention arm, and the attention pin was split by optimisation level

**Date:** 2026-08-22
**Branch:** `harden-before-cross-arch`, developed in an isolated worktree because
another session held the live tree.
**Scope:** everything a cross-architecture wave would rest on — the attention
backward, Gate F's coverage, the clipping that the recurrent arms need, and the
per-cell validity gate.

---

## 1. The finding that stopped the line — **corrected: the pin was profile-split, not broken**

> **Correction, 2026-08-22 (later the same day).** The measurement below is
> real, but its conclusion is wrong, and so was the conclusion of the commit it
> replaced. `every_attention_arm_forward_and_backward_is_bit_pinned` produces
> **different hashes at different optimisation levels**. Same source, same
> machine, only `-C opt-level` varying:
>
> | opt-level | 0 | 1 | 2 | 3 |
> |---|---|---|---|---|
> | side | unoptimised | unoptimised | optimised | optimised |
>
> LTO and `codegen-units` are not involved — the flip reproduces in the plain
> test profile with only `-C opt-level` changed. So both pins were correct, each
> for the profile it was taken under, and each looked broken from the other:
>
> | commit | `cargo test` | `cargo test --release` |
> |---|---|---|
> | `0cc0522` (pre-re-pin constants) | **pass** | **fail** |
> | `7f908c7` (re-pinned constants) | **fail** | **pass** |
>
> This section measured under `--release` (see §9) and concluded the old
> constants "match no committed state". They match it exactly at `-C
> opt-level=0`. The claim that the pin "had never passed" is withdrawn: it
> passed at `597aeba` and `0cc0522` in debug, which is where the three commit
> messages asserting a clean suite were looking. Nobody captured hashes from an
> uncommitted tree; there was no missing kernel revision.
>
> **The kernel never moved.** Three independent checks: `7f908c7`'s edit to
> `shd_attention.rs` is +33 lines entirely inside `mod tests` and its
> non-comment, non-constant diff to `shd_matched_arms.rs` is empty; no commit
> since touches `shd_attention.rs`, `shd_matched_arms.rs` or `shd_matched.rs`;
> and the `ff-fixed-attn` cell recorded by this very commit still reproduces
> **1/1 bit-identical** under `scripts/gate_f_rust.py`.
>
> **The recorded corpus is unaffected.** `shd-instrument` is a release build, so
> every recorded cell and every Gate F re-run sits on the optimised side — the
> side currently pinned. Nothing in `results/` needs re-recording.
>
> **Repair.** The pin now records **both** sides and fails unless the output
> matches one of them exactly, with a companion assertion that all four arms
> land on the same side. Verified passing at opt-levels 0, 1, 2, 3 and
> `--release`, and verified to still fail in both debug and release when the
> attention kernel is deliberately perturbed.
>
> **Mechanism, identified the same day.** `positional_code` calls `.sin()` and
> `.cos()` on the same argument. At opt-level >= 2 LLVM merges that pair into
> Darwin's `__sincosf_stret`; at 0 and 1 it emits separate `sinf` and `cosf`.
> Confirmed in the emitted assembly (`__sincosf_stret` present at O3, absent at
> O0), and the two routines disagree by 1 ULP on 12 of this fixture's 120
> phases — reproduced in a standalone C program calling both on the same `f32`
> bit patterns, matching the Rust probe bit-for-bit. A stage-by-stage bisection
> puts the first divergence in `z[0]`, the embedded input, while the spike train
> hashes identically; it propagates from there. This is also why only the
> attention read-out moves (`positional_code` is the only sin/cos in the arm
> path) and why `--fp-contract=off` changed nothing — contraction was never
> involved.
>
> **Linux is a third set of values — measured, not inferred.** The split is
> Darwin-specific, and Linux shares neither side:
>
> | platform | opt-level | path taken | values |
> |---|---|---|---|
> | Darwin | 0–1 | separate `sinf`/`cosf` | unoptimised pin |
> | Darwin | ≥2 | `__sincosf_stret` | optimised pin |
> | glibc 2.41 | any | `sincosf` | **neither** |
>
> On glibc, `sincosf` agrees with separate `sinf`/`cosf` on all 120 phases, so
> Linux has no opt-level split. But glibc's values are not Darwin's: across the
> 40 positional rows, Linux differs from the unoptimised side in 8 and from the
> optimised side in 4. Verified under Debian glibc 2.41 on **both** x86_64 and
> aarch64, which agree with each other exactly — a libm difference, not an
> architecture one. The harness was validated by first confirming the C
> reproduction matches the Rust optimised side bit-for-bit on Darwin.
>
> Two consequences, both real. The pin **will fail on a Linux host**, matching
> neither side — that is the pin working, and it needs a third *measured* side
> before it runs in Linux CI. And attention-arm cells recorded on Linux cannot
> be bit-identical to macOS-recorded ones at any optimisation level, so Gate F
> comparisons of attention cells are meaningful only within one platform until
> `positional_code` stops depending on the platform's libm — which changes
> release numerics and is therefore a provenance event, not a cleanup.
>
> **What survives.** §3 is independent and stands: Gate F's corpus really was
> 296 cells with every rust cell `ff+fixed`, `parse_cell_id` really had no field
> for an arm, and `regress_cell` really did invoke `train-cell` without `--arm`.
> The attention path genuinely had no recorded-cell gate. §2's independent
> derivation also stands, and is what proved the kernel itself is sound.

Superseded text, retained so the error is auditable rather than erased:

~~`every_attention_arm_forward_and_backward_is_bit_pinned` **fails at `HEAD`**,
for all four attention arms. It also fails at `516e9c7`, `fcfadbd` and
`a3dafd1` — every commit in which it has existed, so no committed state produces
the pinned hashes; they were captured from a working tree that was never
committed.~~

### Why Gate F's coverage gap still mattered

Gate F is the only bit-identity gate against recorded cells. Its corpus is 296
cells, 216 of them rust, and **every one is `ff+fixed`**. `parse_cell_id` splits
a six-field id that has no place for an arm, and `regress_cell` invoked
`train-cell` without `--arm`. So the gate could not express a cell on any other
arm even in principle.

Waves 1–10 are not in question: every cell came from one binary, hash-verified
per instance. What was unverified is whether **today's source still produces
what that binary produced** on the attention path — which is exactly the
comparability a new wave needs against the 96 reused controls. That gap was
real, and §3 closes it.

---

## 2. The pin was not repaired by re-pinning

Re-pinning from the kernel would have recorded whatever the kernel now does as
correct by definition. `binn-learn/tests/attention_w_in_independent.rs`
reimplements the arm forward and backward from the equations documented in
`shd_matched_arms.rs`, in a separate crate target that reaches only the public
API — no transposed layouts, no sparse skip over silent units, no scratch
staging, no prepared weight layout.

The argument has three steps, in this order:

1. **Calibrate on what is already covered.** The reimplementation reproduces all
   four *base* arms **bit-exactly** — the arms Gate F regresses over 296 recorded
   cells and `every_arm_forward_and_backward_is_bit_pinned` covers.
2. **Apply to what is not.** The same reference, unchanged, reproduces all four
   attention arms bit-exactly: membrane, spikes, logits, `grad_w_in`,
   `grad_w_out`, `grad_b_out`, `grad_w_rec`.
3. **Show it can fail.** With attention's contribution to `dL/ds` withheld, the
   comparison breaks by far more than rounding. A check that passes either way
   is not a check.

Underneath, `ds_attn` is checked against central differences of the attention
forward at **every index** of the spike train — 96 of 96 — using no backward code
at all, so the two layers share no assumption. 84 entries were large enough for
f32 central differencing to resolve; the worst relative deviation among those is
**7.2e-3**, against a 2e-2 bar. That bar is separate on purpose: a single loose
absolute tolerance would accept a systematically wrong gradient on small entries,
and an earlier version of this check would have.

**Conclusion: the kernel is right.** That half held, and it is what settled the
question — this derivation passes against the kernel on both sides of the
optimisation split. The second half, "and the pin was stale", was wrong: the
pin matched the kernel exactly, at `-C opt-level=0`. See the correction in §1.

---

## 3. Gate F now covers arms it could not express

Cell ids take two optional components — an arm, and `d<dim>l<layers>` for
attention arms:

```
rust__<contract>__<geometry>__h<hidden>__e<epochs>__s<seed>
rust__...__s<seed>__<arm>
rust__...__s<seed>__<arm>__d<dim>l<layers>
```

Malformed combinations are refused rather than defaulted: an attention arm with
no shape, a shape on a plain arm, or the wrong number of components. All 296
legacy ids keep resolving to the artifacts they were recorded from, and `--arm`
is passed explicitly so a mismatched weight file becomes an error instead of a
silently different cell.

Two reference cells recorded at `fixed-t100__adjacent-sum-5__h128__e20__s5170001`:

| arm | accuracy | classes | silent | saturated | wall |
|---|---:|---:|---:|---:|---:|
| `ff+alif` | 0.6175 | 20 | 0.000 | 0.000 | 55 s |
| `ff+fixed+attn` d32/L1 | 0.7637 | 20 | 0.000 | 0.000 | 164 s |

Gate F against both, each re-run from its pinned initialisation and compared
field by field:

| cell | verdict | traces | re-run |
|---|---|---:|---:|
| `…__ff-alif` | **1/1 bit-identical → PASS** | +3 | 68 s |
| `…__ff-fixed-attn__d32l1` | **1/1 bit-identical → PASS** | +3 | 239 s |

That is the first time in this repository's history that Gate F has regressed
anything other than `ff+fixed`, and the first time the attention kernel has had
any gate at all against a recorded cell.

### An incidental finding, not fixed

`results/shd_instrument_v4/initialization/*.orders` is matched by `.gitignore:90`
(`*.orders`), so **Gate F cannot run from a clean checkout** — it raises
`missing initialization artifact` before comparing anything. The artifacts do
regenerate byte-identically from seed, which was verified here by regenerating
`n8156-e100-s5170001.orders` through a different arm's `init` and comparing with
`cmp`. Nothing in the repository says so, and nothing regenerates them
automatically.

---

## 4. Clipping now acts where the failure happens

`AMENDMENT_2026-08-05_SURROGATE_GAIN_FOR_RECURRENT.md` §1 records batch gradient
clipping as **"never reached — abort fires on a per-sample gradient, upstream"**.
The finding was recorded; the code was never changed. `--clip-grad-norm` sat at
the optimiser step, and the recurrent explosion compounds inside a single
sample's backward, so the run returned an error before a batch gradient existed.

`--clip-sample-grad-norm` clips each sample before accumulation. The rule itself
moved to `binn-lab/src/gradient_clip.rs`, shared by both sites so a second
implementation could not drift from the first, reporting `Untouched` / `Bound` /
`NormOverflowed` and leaving policy to the caller:

* the **batch** site counts an overflow and continues, as it did before;
* the **per-sample** site refuses, because `threshold / inf` is zero and scaling
  would silently delete a sample from its batch while leaving a cell that looks
  trained.

**Off is bit-identical to before this existed** — with no flag the branch is not
entered and no arithmetic touches the gradient. Cells gain
`clip_sample_grad_norm` and `clipped_samples`; Gate F compares an explicit field
list, so recorded cells are unaffected.

A first draft of this incremented an `unclippable_samples` counter and then
returned past it, so the field could only ever have been zero in an emitted
cell. It was removed: a constant-zero field reads as evidence.

This makes clipping *reachable*. Whether it makes `rec+alif` *usable* is an
empirical question this change does not answer.

---

## 5. One owner for the validity gate, and two checks nobody had

Three copies existed and had already drifted:

| copy | temporal audit | `mechanical_status` | type validation |
|---|---|---|---|
| `aws/analyse_campaign.py` | yes | no | no |
| `aws/analyse_wave8.py` | **no** | no | no |
| `azure/analyse.py` | no | yes | yes |

Wave 9's entire result is a bin-shuffled arm, and it was scored through the copy
with no temporal check at all — as was wave 10, which imports the same name. The
claim "every `w9shf` cell passes the temporal audit" is true, but only because
the Rust instrument hard-errors upstream; nothing in the analysis path verified
it.

`scripts/cell_validity.py` is now the single owner, imported by all three. A test
fails if any of them grows its own copy again. Two checks are new:

* **The cell ran the condition the plan asked for.** A cell whose
  `temporal_condition` disagrees with its spec was previously scored as the arm
  it claims to be.
* **Gradient magnitude, reported at two tiers.** Every previous gate was blind to
  it, so a cell peaking at 3.93e33 passed them all.

### Magnitude warns and never voids, because the test said so

The first draft voided cells within five orders of f32 overflow, taking the
2026-08-05 amendment's own "~5 orders from f32 overflow" as a bar. Its test
caught the problem: that amendment's run peaked at **3.93e33** and was reported
as a *result* — expectation MET, `non_finite_events` 0, loss falling — described
as marginal, not discarded. A bar at 3.4e33 would have retroactively voided a
published run, which is a re-scoring rather than a hardening and needs its own
registration.

The warning tiers are empirical. Across the 624 campaign cells carrying per-epoch
norms the largest observed maximum is **1.13e8** (`ff+fixed+attn` at h1024) and
the median is 7.70:

| arm | width | n | max | median |
|---|---|---:|---:|---:|
| `ff+fixed+attn` | h1024 | 24 | 1.133e+08 | 1.226e+03 |
| `ff+fixed+attn` | h128 | 324 | 7.354e+02 | 1.001e+01 |
| `ff+fixed+attn` | h512 | 24 | 3.424e+02 | 3.679e+01 |
| `ff+fixed+attn` | h256 | 12 | 1.485e+02 | 8.776e+01 |
| `ff+fixed` | h128 | 192 | 1.676e+00 | 5.943e-01 |

**Verified against the record: 624 cells, 0 voided, 0 warnings.** Wave 8's
verdicts reproduce exactly under the new gate — S-1 ✗, S-2 ✓, S-3 ✗, S-4 ✓,
S-5 ✗, S-6 ✓.

---

## 6. A gate that failed on where the repository was, not what it contained

`check_gc2.sh` bans ML-framework dependencies by matching `torch|tch|candle|burn|dfdx`
against raw `cargo tree` output — which prints each workspace crate's **absolute
path** in parentheses. The gate therefore depended on the checkout location:
`scratchpad` contains `tch`, so running it from a worktree under
`.../scratchpad/...` reported "banned ML framework dependency" for every local
crate and listed paths rather than dependencies.

It now extracts crate names before matching, and matches hyphen-delimited
components — so `tch`, `tch-sys`, `torch-sys`, `candle-core` and `burn-tensor`
are still caught while `patch`, `matcher` and `scratchpad` are not. The matcher
is calibrated against a fixture of names it must catch and names it must not,
and **refuses to report a pass if that calibration fails**, which is the rule
`find_weak_checks.py` already applies to itself. Negative-tested: replacing the
pattern with one that matches nothing makes the gate exit 1 with
`GC2 SELF-CHECK FAILED: matcher caught 0 of 5 known banned crates` rather than
printing a pass.

This one was found by running the gate, not by reading it. It had never fired
because the repository had never been checked out under such a path.

---

## 7. Stress

| what | coverage | result |
|---|---|---|
| kernel vs independent derivation | 10 shapes × 8 arms = **80 comparisons** | all bit-exact |
| thread-count independence | 8 arms × {1, 2, 3, 5, 8, 16} threads | all bit-identical |
| `ds_attn` vs central differences | every index, 96/96 | worst resolvable relative 7.2e-3 |
| clipping semantics | 6 unit tests | boundary, uniformity, overflow, every block |

The shape sweep is where a kernel that is exactly right on one fixture can be
wrong: single timestep, single hidden unit, frames with no events, `d_model`
equal to hidden width, more attention layers than hidden units, `alpha` near
zero and near one. Prime thread counts are included because a chunking defect
that divides evenly into the batch is invisible at 1, 2, 4 and 8 — and
`PARALLEL_CHUNK` is 64.

`scripts/find_weak_checks.py` flagged three tests whose assertions a degenerate
result would satisfy. One,
`shd_attention::a_fully_saturated_trace_stays_finite_through_both_passes`, is
fixed: it now also asserts the attention rows still sum to 1, that neither
gradient is identically zero, and that timesteps receive *different* credit — a
saturated trace is the case where a lost positional code would otherwise pass
silently. The other two are untouched and named in §7.

---

## 8. What is **not** verified

* ~~**Source-versus-campaign-binary equivalence on the attention path.**~~
  **CLOSED 2026-08-22, same day.** Measured on one aarch64 host running both
  binaries in the same boot: identical on `ff+fixed` and on `ff+fixed+attn` at
  d32/L1 and d32/L4, e5 through e400, across 12 scientific fields and 1,200
  trace values on the e400 cell. The archived controls remain licensed against
  cells produced by today's source. See
  [`RESULT_2026-08-22_SOURCE_REPRODUCES_THE_PINNED_BINARY.md`](RESULT_2026-08-22_SOURCE_REPRODUCES_THE_PINNED_BINARY.md).
* **Whether per-sample clipping makes `rec+alif` usable.** The mechanism is
  reachable and tested; the empirical question is open.
* **Two weak checks remain.** `matched_deep_gradient::trains_at_every_depth_without_panicking`
  was being edited by another session during this work and was deliberately not
  touched; `temperature_ablation::suite_covers_at_least_two_ablation_axes` is in
  a different subsystem and out of scope here.
* Nothing here changes the instrument's `Uncalibrated` state, and
  `shd-arch-ablation` / `shd-frozen-attention` remain refused at the
  authorization gate.

---

## 9. Verification

```bash
cargo fmt --check                                   # clean
cargo clippy --workspace --all-targets -- -D warnings   # clean
cargo test --release --workspace
python3 scripts/test_campaign_tooling.py            # 31 tests
python3 scripts/find_weak_checks.py
python3 scripts/gate_f_rust.py --cell rust__fixed-t100__adjacent-sum-5__h128__e20__s5170001__ff-alif
```
