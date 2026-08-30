# Audit — `shd_alif.rs` and `shared_bptt.rs`

**Date:** 2026-08-30, while waves 22–23 ran.
**Scope:** the two files
[`AUDIT_2026-08-03_RUST_DEFECT_REGISTER.md`](AUDIT_2026-08-03_RUST_DEFECT_REGISTER.md)
§3 lists as unswept and **on the cell path**: `binn-learn/src/shd_alif.rs`
(1453 lines) and `binn-learn/src/shared_bptt.rs` (1254 lines).
**Method:** the same six defect classes as the 2026-08-03 register, plus its
§2b classes A–D.
**Result: two defects, both fixed. Both are the same class, and both destroy a
learning signal silently.**

---

## 1. Why these two and not the 8,000 lines

The open-work register's §6 names `binn-engine`, `binn-areas` and `binn-core` —
about 7,000 lines — as the largest audit debt. **They are not on the cell
path.** `shd_instrument.rs` imports `binn_data` and `binn_learn` and references
those three crates **zero** times, so no cell any wave runs passes through them.

`shd_alif.rs` and `shared_bptt.rs` are the opposite: unswept *and* live.
`shd_alif.rs` is the `ff+alif` / `rec+alif` forward, which is where **Figure S's
substrate rows** come from; `shared_bptt.rs` is the SuperSpike BPTT reference
that the entire matched-architecture programme measures against.

## 2. Defect 1 — an overflowing modulator RMS zeroed the DFA update

**Site:** `shd_alif.rs::normalize_hidden_modulator`.
**Class:** silent success — the register's dominant class.
**Named in advance:** §2b, verbatim: *"partially unguarded. If `actual_rms`
overflows while `target_rms` is finite, `scale` becomes 0 and `mods` is zeroed —
silent, and not covered by the existing guard. The `actual_rms` sub-case is the
one worth fixing first if anyone does."* Open 27 days.

`f32::MAX` is ~3.4e38, so a sum of squares overflows once entries reach ~1e19 —
while the RMS itself, ~1e19, is comfortably representable. The old guard was

```rust
if actual_rms > f32::EPSILON && target_rms.is_finite() { … }
```

and `inf > f32::EPSILON` is **true**. So `scale = target_rms / inf = 0.0`, and
every entry was multiplied by zero. **The whole DFA update for that example
became exactly zero, the step proceeded, and nothing recorded it.**

This is worse than the `l2_norm` overflow that
[`AMENDMENT_2026-08-03_L2_NORM_CONDITIONAL_WIDENING.md`](AMENDMENT_2026-08-03_L2_NORM_CONDITIONAL_WIDENING.md)
was written for. That one corrupted a *reported number*; this one corrupts the
**dynamics**.

## 3. Defect 2 — an overflowing gradient norm zeroed the BPTT step

**Site:** `shared_bptt.rs::SharedGradients::global_norm`.
**Class:** the same one.

§2b surveyed this file's f32 sum-of-squares sites and assessed the RMS helper at
`:878` as *diagnostic*. It is. **`global_norm` is not**, and the two were never
separated: `Adam::update` clips against it.

```text
inf > GRADIENT_CLIP_NORM  ->  scale(GRADIENT_CLIP_NORM / inf) == scale(0.0)
```

Every gradient entry multiplied by zero, and Adam then steps on an all-zero
gradient. That is not what clipping to a norm means. `StepDiagnostics` stores
both norms but **no caller gates on their finiteness**, so an `inf` in the
record is the only trace and nothing reads it.

## 4. The fix, and why it was safe to make here

The register declined to fix these, and its reasoning was sound: *"these files
have no bit-identity regression suite… so a change here cannot be shown harmless
the way the instrument change could. Fixing untested code to remove a latent
defect can trade a known-dormant bug for an unknown live one."*

**Conditional widening supplies the missing proof in the code itself.** The
amendment's pattern computes in f32 and falls back to f64 *only when the f32 sum
is non-finite*, so below the overflow threshold the fallback is never entered
and the result is the same bits. Both files now assert that by `to_bits()`,
not by tolerance.

Two changes at the `shd_alif` site, and the second matters even after the first:

1. Both sums are conditionally widened.
2. `actual_rms.is_finite()` joins the guard, so a non-finite RMS leaves the
   modulator **alone** rather than scaling it by zero — failing closed, exactly
   as the existing `target_rms.is_finite()` arm already did.

### Two mistakes made while fixing it, both caught by tests

**The first fix was wrong.** The helper returned the widened *sum* and cast that
back to f32 — for entries around 3e19 the sum is ~2.3e39, beyond `f32::MAX`, so
it overflowed a second time and the widening bought nothing. The root and the
division must happen in f64 **before** the cast. The amendment does this
correctly; the transcription did not.

**The test that should have caught it did not, at first.** It asserted only
"not zeroed", which the fail-closed guard alone satisfies by *skipping*
normalisation — leaving the modulator un-normalised, which is the one thing the
function exists to prevent. Negative-testing exposed this: reverting the
widening fired nothing. Strengthened to assert the target RMS, it fires.

That sequence is the argument for negative-testing every guard, and it is why
the four properties below are each pinned by a perturbation that fires exactly
one test.

## 5. Evidence

Seven tests added across the two files; all four properties negative-tested by
reverting the fix and confirming the intended test fails.

| property | test |
|---|---|
| an overflowing modulator RMS is normalised, not zeroed | `a_modulator_whose_rms_overflows_is_not_zeroed` |
| widening never moves a representable value | `widening_is_bit_identical_below_the_overflow_threshold` |
| a non-finite **entry** fails closed | `a_non_finite_entry_leaves_the_modulator_untouched` |
| the RMS of any finite vector is representable | `the_rms_of_any_finite_vector_is_representable` |
| an overflowing gradient norm is clipped, not zeroed | `a_gradient_whose_norm_overflows_is_clipped_not_zeroed` |
| `global_norm` is bit-identical below the threshold | `global_norm_is_bit_identical_below_the_overflow_threshold` |

`cargo test -p binn-learn --lib`: **211 passed, 0 failed.** Clippy: clean.

## 6. Classes swept clean

| class | result |
|---|---|
| **A — fields read but never written** | **zero.** Every field in both files is written; the one apparent hit, `DenseTemporalExample.frames`, is a `pub` field constructed by callers in other crates |
| **C — clamping** | no true instances. The `.max(1e-12)` sites guard a logarithm and a softmax denominator, which is defined semantics, not a silently shortened request |
| **D — panics on data paths** | `widths.last().unwrap()` in `shared_bptt` is guarded by `assert!(!widths.is_empty())` at construction; `set_parameter`'s `panic!` is a programming error. `shd_alif`'s two `expect`s call `attention_forward`, which fails only on a shape mismatch or zero timesteps — structural invariants, not data values, and failing loudly there is correct |
| **silent divergence (parallel implementations)** | not applicable: these are not mirrored pairs of each other, unlike the `shd_matched.rs` / `shd_matched_arms.rs` pair that produced defects #8 and #9 |

## 7. Two findings recorded and deliberately NOT fixed

**7.1 `evaluate_detailed` does not check the forward for finiteness.** This is
the same gap as
[`DEFECT_2026-08-29_THE_EVALUATION_FORWARD_WAS_NEVER_CHECKED.md`](DEFECT_2026-08-29_THE_EVALUATION_FORWARD_WAS_NEVER_CHECKED.md),
and here it is **materially weaker**, for a reason worth writing down: this
module's argmax is an explicit `pi > best` loop from `NEG_INFINITY`, so under
NaN the comparison is **false** and a NaN never wins. The instrument's `argmax`
orders by `total_cmp`, under which NaN outranks every real and *does* win.

So a poisoned forward here collapses predictions to class 0 rather than
scattering them, which inflates `majority_pred_frac` against its 0.95 check and
is partially caught by `is_degenerate()`. The partial case — a small fraction of
poisoned samples — is still not caught.

Not fixed because the fix means changing the public `AlifEval` shape and every
caller across the experiment binaries, which is a larger blast radius than this
sweep warrants and than the residual risk justifies. Recorded so the next person
does not have to rediscover it.

**7.2 Two argmax implementations with different tie-breaking.** `shd_alif`'s
keeps the **first** maximum and lets NaN lose; `shd_matched`'s `total_cmp` keeps
the **last** and lets NaN win. Nothing compares them and they belong to
different arms, so this is not defect #8 recurring. It is recorded because #8
was exactly two argmaxes that were *assumed* to agree, and the assumption is
what cost.

## 8. What this does not establish

- **It does not audit `binn-engine`, `binn-areas` or `binn-core`.** Those remain
  unswept and remain off the cell path.
- **It does not mean any published number is wrong.** Whether either overflow
  ever fired in an archived run is **unknown and unknowable from the artefacts**
  — no cell records a gradient norm for the alif arms, and `StepDiagnostics` is
  not persisted. What can be said is that from now on neither can fire unseen.
- **It changes no dynamics below the overflow threshold**, which is the point of
  conditional widening and is asserted bitwise rather than argued.
