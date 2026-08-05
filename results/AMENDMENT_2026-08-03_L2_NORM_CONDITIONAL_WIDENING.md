# Amendment: conditional f64 widening of the gradient L2 norm

**Date:** 2026-08-03
**Registered before the change.**
**Scope:** `MatchedGradient::l2_norm` (`shd_matched.rs:188`) and
`ArmGradient::l2_norm` (`shd_matched_arms.rs`).
**Affects:** `mean_gradient_norm`, `epoch_mean_gradient_norm`,
`epoch_max_gradient_norm` — the first of which is a **Gate F compared field**
across 216 recorded cells.

---

## claim_axis

```
axis: instrument-correctness
claim: The gradient L2 norm returned infinity for norms f32 can represent,
  because the sum of squares overflowed while the norm did not. The fix widens
  the accumulation to f64 only on that overflow path, leaving every
  representable value bit-identical.
may_claim: That values which were finite before are bit-identical after, and
  that norms between roughly 1e19 and f32::MAX are now returned instead of
  infinity.
must_not_claim: That this makes the recurrent arms trainable. It corrects the
  *record*, not the dynamics. The two h512 seeds that abort do so because an
  individual gradient entry is non-finite, which this does not touch (§4).
```

## 1. The defect

`l2_norm` accumulated squares in f32:

```rust
.map(|value| value * value).sum::<f32>().sqrt()
```

`f32::MAX` is ~3.4e38, so the **sum of squares** overflows once entries reach
~1e19 — while the norm itself, ~1e19, remains comfortably representable. The
function therefore returned `inf` for values f32 could hold.

Two existing guards both missed it:

- `all_finite()` checks individual **entries**, and every entry was finite.
- `non_finite_events` was never incremented at all
  (`MEASUREMENT_2026-08-03_RECURRENT_ARM_STABILITY.md` §4b).

Observed on `rec+alif` at h512: 420 of 640 optimizer steps produced an infinite
norm this way, in a cell that otherwise looked healthy.

## 2. Why not simply accumulate in f64

That is the tidier fix and it is **deliberately not what this does**.

Widening unconditionally changes the summation order and precision for *every*
call, which moves `mean_gradient_norm` in the last ulp for the 216 recorded
`ff+fixed` cells. `mean_gradient_norm` is in Gate F's `COMPARED_FIELDS`, so that
is a change to a registered result, not a bug fix — precisely the category the
2026-08-02 amendment exists to prevent.

## 3. The change

Compute in f32 as before; fall back to f64 **only when the f32 sum is
non-finite**:

```rust
let sum = ...sum::<f32>();
if sum.is_finite() { return sum.sqrt(); }
// f64 recomputation
```

Three properties, all pinned by `l2_norm_widens_only_when_f32_overflows`:

1. **Finite values are bit-identical** — asserted against the naive f32 fold by
   `to_bits()`, not by tolerance.
2. **Overflowed values are recovered** — a fixture with true norm ~1.4e20, whose
   sum of squares is 2e40, returns the correct finite value.
3. **Genuinely unrepresentable norms stay infinite** — `f32::MAX` entries still
   return infinity rather than wrapping.

`ArmGradient::l2_norm` needs the same treatment and cannot reuse the base norm on
the fallback path, since `base * base` can overflow even when `base` is finite.

## 4. What this does not fix

**It corrects the record, not the dynamics.** Of the three h512 `rec+alif` seeds,
only 5170001 is affected — its 420 infinite norms become finite numbers, so the
cell now reports what actually happened.

Seeds 5170002 and 5170003 **abort**, and this changes nothing for them. Their
guard is

```rust
if !forward.loss.is_finite() || !sample_gradient.all_finite() { return Err(...) }
```

an individual gradient *entry* going non-finite — a real numerical failure of
BPTT at that width, not a reporting artifact. `PREREG_2026-08-02_SHD_BPTT_CEILING_RECALIF`
remains blocked, and the open question is still gradient clipping.

## 5. Evidence

- `cargo test -p binn-learn` — 158 pass, including the new bit-identity test.
- **Gate F, 13 cells, against the rebuilt binary** — recorded below. This is the
  binding check: if any `mean_gradient_norm` moved, it fails.

*(Written before the Gate F run so the prediction is on the record: it should
pass 13/13, because no recorded `ff+fixed` cell has ever produced a non-finite
sum of squares — every one of them has a finite recorded `mean_gradient_norm`,
so none can take the new branch.)*

**Result: 13/13 bit-identical, PASS**, binary `26a83c1253e2`. The prediction
held, and the reasoning behind it is the reason the fix was shaped this way: the
branch is unreachable for every value that was ever recorded.

This is the ninth Gate F run in `gate-f-rust/runs.jsonl`, all PASS, spanning
seven distinct binary hashes across a day of kernel and instrument changes.

## 6. Why this was worth the shape it took

The straightforward version of this fix — accumulate in f64, always — was
explicitly authorised, including its cost of breaking Gate F against 216
recorded cells. It was not taken, because the cost turned out to be avoidable:
the overflow only ever occurs on values that were already wrong, so correcting
them cannot disturb anything that was right.

Stated as a rule worth reusing: **when a numerical fix appears to require
invalidating a recorded result, check whether the defect's domain and the
record's domain actually intersect.** Here they are disjoint by construction —
every recorded value is finite, and the new branch is reachable only from
infinity.
