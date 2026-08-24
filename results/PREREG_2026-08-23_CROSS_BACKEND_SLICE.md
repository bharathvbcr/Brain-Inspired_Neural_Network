# Preregistration — the first evaluation of the cross-backend criterion

**Registered:** 2026-08-23, before any Python-arm cell of this slice runs.
**Authorised by the maintainer**, who chose the cheapest slice over the full 136
and over leaving the deferral in place.

---

## 1. What is being run and why it is a slice

The calibration gates opened
(`RESULT_2026-08-23_REFERENCE_RERUN.md`), unblocking the 136 remaining cells of
the 432-cell matrix. **All 136 are the Python arm**; the Rust arm is complete at
216/216.

The Python arm is deferred by standing instruction
(`TODO_2026-08-07_OPEN_WORK.md` §8). The deferral is **not lifted in general**.
What is authorised is the smallest slice that lets the cross-backend criterion be
evaluated at all: **21 cells at `epochs = 20`, `hidden = 128`**, which complete
**7 groups** where both backends have all three seeds.

Cost is roughly a tenth of the full 136 (≈2.8 days by the record's own estimate).

## 2. The criterion, which already exists

From `matrix_verdict` (`runner.py:1205-1232`), unchanged and not re-derived here:

- a group is `(contract, geometry, hidden, epochs)`;
- it is evaluable only when **both** backends have 3 cells;
- every one of those 6 cells must carry `scientific_status: CELL_PASS`;
- agreement is `|python_accuracy − rust_accuracy| ≤ 0.05` **per seed**, not on the
  mean.

**No threshold is introduced by this document.** The 0.05 has been in the code
since before the Python arm was deferred.

## 3. What this cannot do, stated so it is not misread later

`scientific_status` calls `matrix_verdict` **only when all 432 cells are
complete** (`write_ledger`). Twenty-one cells cannot move it, and it will still
read `UNCALIBRATED` afterwards. **That is expected and is not a failure of this
run.**

What the slice produces is the first direct measurement of per-seed agreement
between the two backends on matched configurations. The verdict machinery stays
where it is.

## 4. Registered outcomes

- **All 7 groups agree within 0.05 on every seed** → the cross-backend criterion
  holds where it has been measured. This is evidence *for* spending the remaining
  ~2.5 days, and the decision to do so is the maintainer's, not mine.
- **Any group disagrees on any seed** → that is the finding, and it is worth more
  than the other 115 cells. It would mean the two implementations of the same
  registered configuration do not compute the same thing, which bears directly on
  every Rust-arm number the paper cites. **Reported immediately and in full, with
  the disagreeing group named, before any further cell is run.**
- **Any cell fails to complete, or completes with `CELL_FAIL`** → the group is
  not evaluable and is reported as such. A group that cannot be scored must never
  be reported as a group that agreed.
- **Mixed: some groups agree, some disagree** → reported per group. No aggregate
  "mostly agrees" is issued; the criterion is per seed and per group, and an
  average over it would hide exactly the case it exists to catch.

## 5. Prediction, so that it can be wrong

The Rust and Python arms are independent implementations of the same registered
configuration, and the workspace has never measured whether they agree. I expect
agreement within 0.05 on all 21 pairs — but the honest note is that this
expectation rests on nothing measured, which is precisely why the criterion
exists and why it has been sitting unmet.

## 6. What this may not claim

- **It does not calibrate the instrument.** `SHD_INSTRUMENT_STATE` stays
  `Uncalibrated`; that is a separate constant behind separate criteria.
- **It does not lift the deferral.** 115 Python-arm cells remain unrun by
  instruction after this slice.
- **It does not validate the Rust arm.** Agreement between two implementations is
  agreement, not correctness. Both could be wrong in the same way — and on this
  task they share a data pipeline, so a data-side defect would not show up here
  at all.
