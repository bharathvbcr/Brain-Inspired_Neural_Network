# Amendment: scope the reference fingerprint to the reference pipeline

> ## WITHDRAWN 2026-08-03 — the diagnosis in §1 was incomplete and the
> ## registered expectation in §5 failed. Code reverted, gates unchanged.
> See §6. Kept in full because the reasoning is still needed by whoever
> takes this on, and because §5 firing is the mechanism working.

**Registered:** 2026-08-03, before the change.
**Amends:** `valid_reference_payload` / `relevant_source_fingerprint` in
`scripts/shd_calibration/runner.py`.
**Unblocks:** `historical_reference`, `clean_reference`, and therefore
`matrix_authorized`.

---

## 1. The defect

`historical_reference` and `clean_reference` are both **false**, which forces
`matrix_authorized` false. The handoff attributed this to "a fingerprint that is
scoped too broadly". That is now measured rather than assumed.

Every one of the six reference artifacts was checked field by field:

| artifact | accuracy | failing check |
|---|---|---|
| historical s5170001 | 0.9447 | `source_fingerprint` |
| historical s5170002 | 0.9573 | `source_fingerprint` |
| historical s5170003 | 0.9475 | `source_fingerprint` |
| clean s5170001 | 0.9389628 | `source_fingerprint` |
| clean s5170002 | 0.9367766 | `source_fingerprint` |
| clean s5170003 | 0.9371463 | `source_fingerprint` |

**All six pass every content check** — `mechanical_status`, mode, seed,
`result_sha256`, `log_sha256`, `manifest_sha256`. The artifacts are byte-for-byte
what they were. **Only the source fingerprint moved**, and it moved because the
rust kernel was edited.

## 2. Why that is wrong by construction

`SOURCE_PATHS` covers the rust instrument — `binn-learn/src/shd_matched.rs`,
`binn-lab/experiments/shd_instrument.rs`, `binn-data/src/shd_contract.rs`,
`Cargo.lock` — alongside the reference pipeline.

But the reference runs **do not execute a single line of that code**. They clone
`Thvnvtos/SNN-delays` at a pinned commit and run it under PyTorch:

```
$ grep -E "shd-instrument|RUST_BINARY|target/release|binn-" \
      scripts/shd_calibration/reference.py scripts/shd_calibration/reference_clean_main.py
  (no matches)
```

The reference is a **third-party baseline**, deliberately independent of the
instrument — that independence is the entire reason it is a useful calibration
target. Tying its validity to the instrument's source destroys the property it
exists to provide.

So the current rule says: *change the rust kernel, and a PyTorch model's recorded
accuracy stops counting.* That cannot be right, and it is not a conservative
error — it is an error that **blocks calibration permanently**, because the
kernel will keep changing.

## 3. The change

Introduce a **separate, narrower fingerprint for reference artifacts**, covering
only what can affect a reference run:

```
REFERENCE_SOURCE_PATHS = (
    scripts/shd_calibration/reference.py,           # harness + pinned commit
    scripts/shd_calibration/reference_clean_main.py,# clean-mode training entry
    scripts/shd_calibration/data.py,                # SHD loader
)
```

`relevant_source_fingerprint()` is **unchanged** and still governs instrument
cells. Only `valid_reference_payload` switches to the narrower one.

The pinned upstream commit (`REFERENCE_COMMIT` in `reference.py`) is inside the
fingerprint because that file is, so bumping the upstream revision still
invalidates the references — correctly.

## 4. What this does not do

- **It does not weaken the data checks.** `result_sha256`, `log_sha256`,
  `manifest_sha256`, `mechanical_status`, mode and seed are all still required
  and all still pass. If a reference artifact were edited, it would still be
  rejected.
- **It does not touch instrument-cell provenance.** `ensure_manifest`, the
  `core_manifest` inputs, and the freeze on `train_h5` / `test_h5` /
  `train_events` / `test_events` are untouched. This is a strictly narrower
  claim than `AMENDMENT_2026-08-03_PROVENANCE_DISCHARGE_BY_BIT_IDENTITY.md`,
  which remains default-off and still awaits a human decision.
- **It does not by itself make the matrix verdict PASS.**
  `SHD_BPTT_CEILING_NEGATIVE_RESULT.md` establishes that the verdict is `FAIL`
  by construction, because all 216 rust cells fail the 0.80 accuracy gate.
  Restoring these two gates removes a *harness* blocker, not a scientific one.

## 5. Falsifiable expectation, recorded before running

After the change: `historical_reference` and `clean_reference` both become
**true**, with `historical_mean_accuracy` ≈ 0.9498 (mean of 0.9447 / 0.9573 /
0.9475) and `clean_accuracies` = [0.9390, 0.9368, 0.9371], all three above the
calibration floor. `matrix_authorized` becomes true only if its other conjuncts
already hold.

If any of those does **not** happen, the diagnosis in §1 is incomplete and the
change should be reverted rather than patched.


## 6. WITHDRAWN — what §5 caught

The change was implemented, the expectation in §5 was checked, and **it failed**:
all six artifacts remained invalid. Per §5 — *"the change should be reverted
rather than patched"* — the code was reverted. `valid_reference_payload` is back
to the registered behaviour and the gates are exactly as they were.

**Why it failed.** The stored `source_fingerprint` is
`64923d64655d86ee…`, computed over the **broad** `SOURCE_PATHS` set at freeze
time. A fingerprint computed over the **narrow** `REFERENCE_SOURCE_PATHS` set is
`50c0fe76a66eda56…`. These are not a stale value and a fresh value of the same
quantity — **they are outputs of different functions over different inputs**, and
no amount of narrowing makes a historical broad fingerprint comparable to a
narrow one.

§2's argument still stands: fingerprinting a third-party PyTorch baseline against
the rust kernel is wrong, and the reference pipeline provably does not touch that
kernel. The error was in §3's assumption that narrowing the scope would
*retroactively* validate artifacts frozen under the old scope. It cannot.

**What this means for the gap.** Restoring `historical_reference` /
`clean_reference` requires one of:

1. **Accepting the artifacts on content evidence** — all six have matching
   `result_sha256`, `log_sha256`, `manifest_sha256`, correct mode/seed, and
   `mechanical_status: COMPLETE`. This is the *same* judgement call as
   `AMENDMENT_2026-08-03_PROVENANCE_DISCHARGE_BY_BIT_IDENTITY.md` and is a
   **human decision**, not a code fix.
2. **Re-running the three clean and three historical reference cells** under the
   current source, which regenerates manifests with a current fingerprint. This
   is honest but expensive, and needs the pinned upstream checkout plus GPU time.
3. **A migration** that recomputes each historical manifest's fingerprint under a
   narrowed scope *from the source state at freeze time* — which is not
   recoverable, so this is not actually available.

Option 1 is the same decision already pending on the provenance flag; option 2 is
the only one that closes it without a judgement call. **The gap is therefore not
closeable by code alone**, and saying so is more useful than a change that looks
like it fixed something.

The narrowing itself is still worth doing **for artifacts created from now on**,
so that a future kernel edit does not re-create this situation. That is a
separate amendment with a correct diagnosis, and is deliberately not being
smuggled in here.
