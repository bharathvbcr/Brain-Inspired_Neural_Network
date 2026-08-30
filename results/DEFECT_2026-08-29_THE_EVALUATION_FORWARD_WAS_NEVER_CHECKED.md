# Defect — a cell's accuracy was never checked for a non-finite forward

**Found:** 2026-08-29, sweeping the cell path before launching waves 22–24.
**Class:** silent success — the register's dominant class, 5 of its 10 defects.
**Site:** `binn-lab/experiments/shd_instrument.rs::evaluate`.
**Status:** FIXED, with a test that fails against the pre-fix code.
**Named in advance:** [`AUDIT_2026-08-03_RUST_DEFECT_REGISTER.md`](AUDIT_2026-08-03_RUST_DEFECT_REGISTER.md)
§3, item 2 — verbatim, and open for 26 days.

---

## 1. What it was

`evaluate()` computes the accuracy every published number in the read-out
program is built from. It took `forward.prediction` for each test sample and
counted it, and **nothing anywhere checked that the forward's logits were
finite.**

Three facts compose into the defect:

1. **`argmax` orders by `total_cmp`**, under which NaN sorts **above** every
   real. A poisoned sample therefore does not crash and does not abstain — it
   predicts, deterministically, and its class is counted.
2. **`non_finite_events` does not see it.** That counter is incremented in the
   training loop, on `gradient_norm` and `update_rms`. It is a *training*
   diagnostic. The evaluation pass ran unguarded.
3. **The pass predicate could only catch the total case.** It requires
   `classes_predicted == n_classes` and `majority_prediction < 0.30`, so a cell
   where *every* sample is poisoned collapses to one class and is caught.

**The partial case was caught by nothing.** A minority of poisoned samples
leaves the class histogram healthy, keeps the majority share low, and silently
moves `accuracy` — beside `"non_finite_events": 0`, `"mechanical_status":
"COMPLETE"`, and a well-formed number in `[0, 1]`.

## 2. Why it mattered now

The waves registered for this week include **h1024**, which is the most
numerically extreme configuration in the campaign: peak gradient norms to
**4.9e32** against 1.13e8 for the largest cell anywhere else
([`RESULT_2026-08-23_W13_RECURRENT_STABILITY.md`](RESULT_2026-08-23_W13_RECURRENT_STABILITY.md)).
It is precisely where a forward would leave f32's range, and it is the operating
point the paper's leading open problem is about.

Running 250+ cells through this would have produced evidence with no way to
tell a corrupted accuracy from a real one.

## 3. The fix

`Evaluation` gains `non_finite_forward`: the count of test samples whose logits
were not all finite. It is counted inside the existing parallel closure — from
`forward.logits`, which the forward already computed and returned, so nothing
extra is calculated — and returned as a flag rather than by carrying the vector
out. The count is an integer, so unlike `unit_rate` it does not depend on
accumulation order.

It reaches three places:

* the **pass predicate**, which now requires `non_finite_forward == 0`;
* the **cell JSON**, appended after `non_finite_events` — never inserted — on
  the `seed` provenance precedent, so archived cells and Gate F's explicit field
  list are unaffected;
* [`scripts/cell_validity.py`](../scripts/cell_validity.py), which is the live
  validity owner and voids a cell carrying a non-zero count.

### One asymmetry, deliberate

In `cell_validity.py` every other field is read through `_number`, which records
a **missing** field as a problem. `non_finite_forward` is not, because the 861
archived cells predate the guard and `_number` would void the entire corpus at a
stroke. Absence means "produced before the forward was checked", which is a
provenance fact rather than a defect.

That is exactly the shape that becomes a silent hole, so it does not stay
implicit: `pre_guard_cells()` counts them, so a check that *could not run* stays
distinguishable from one that ran and passed.

### What was deliberately *not* done

`non_finite_forward` is **not** added to `gate_f_rust.py::COMPARED_FIELDS`, and
the reason is recorded at the tuple so the omission is not read as an oversight
and quietly corrected. That tuple is coupled to the **frozen** per-wave
analysers by
`test_reproduction_check.py::test_no_analyser_drops_a_field_gate_f_compares`,
whose invariant is that an analyser may check more than Gate F and never less.
`analyse_wave15` and `analyse_wave18` were registered with their
preregistrations before their first cell existed and are not editable after the
fact. Nothing is lost — no archived cell carries the field, so comparing it
would compare nothing today. Wave analysers written from now on carry it.

## 4. Evidence

`a_forward_that_overflows_is_counted_rather_than_scored`, in the instrument's
own test module. Two properties, and the fixture took two attempts:

* **Inflating `w_in` does not work**, and the reason is worth keeping. The
  membrane goes to `inf` on the first frame; the next step computes
  `alpha * inf * (1.0 - 1.0)` and gets NaN; `NaN >= threshold` is **false**, so
  the neuron stops spiking and the read-out stays finite. **The spiking
  nonlinearity absorbs an input-side overflow.** It cannot absorb an
  output-side one.
* The fixture therefore inflates **`w_out` only**, with `w_in` raised just
  enough that every unit fires and the rates are ~1.0. At the deterministic
  init the rates are low enough that the eight-term read-out sum stays inside
  f32 and the fixture silently stops demonstrating anything — which it did on
  the first attempt — so `mean_firing_rate > 0.5` is asserted as a precondition.

The test also asserts the accuracy is still a well-formed number in `[0, 1]`
under poisoning. That is the defect stated as an assertion: nothing about the
value reveals how it was computed.

### Bit-identity, and its limit

The six other fields `evaluate` returns are **pinned to values captured by
running this fixture through the pre-fix code**, not read back off the new
implementation:

    accuracy 0.125 · classes_predicted 3 · majority_prediction 0.5
    mean_firing_rate 0.04817708465270698 · silent_fraction 0.375
    saturated_fraction 0.0

**Gate F would normally establish this over archived cells and it could not
run.** The initialization artefacts are gitignored (`*.orders`,
`results/**/orders/`) and absent from a fresh worktree, and the real SHD `.h5`
data is gitignored too — `data/shd/fixture` is a CI stub the instrument rejects
with "bad SHD event magic". So end-to-end bit-identity over real cells is
**unverified here** and is stated as unverified rather than argued from the
diff. The pinned unit-level values are what stands in its place. Running
`python3 scripts/gate_f_rust.py --cheapest 6` in the main checkout, where the
artefacts live, is what would close it.

## 5. What this does not establish

- **It does not mean any published number is wrong.** No archived cell can be
  checked retroactively — the logits were not recorded — so whether the defect
  ever fired is **unknown and unknowable from the artefacts**. What can be said
  is that from now on it cannot fire unseen.
- It does not audit the other f32 accumulations in the register's §3. The
  `evaluate()` rate statistics named there turn out to be **already clean**:
  `unit_rate` accumulates in `f64` and the ordering discipline is documented at
  the loop.
- It does not touch `binn-engine`, `binn-areas` or `binn-core`. Those are
  **not on the cell path** — the instrument imports `binn_data` and
  `binn_learn` and references none of them — so no cell this campaign runs
  passes through them.

## 6. Companions

[`AUDIT_2026-08-03_RUST_DEFECT_REGISTER.md`](AUDIT_2026-08-03_RUST_DEFECT_REGISTER.md) ·
[`PAPER_GAPS_2026-08-29.md`](PAPER_GAPS_2026-08-29.md) ·
[`MEASUREMENT_2026-08-03_RECURRENT_ARM_STABILITY.md`](MEASUREMENT_2026-08-03_RECURRENT_ARM_STABILITY.md)
