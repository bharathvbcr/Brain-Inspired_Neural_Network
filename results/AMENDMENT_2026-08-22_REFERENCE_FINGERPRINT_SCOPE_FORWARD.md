# Amendment: scope the reference fingerprint, for artifacts frozen from now on

**Registered:** 2026-08-22, before the change.
**Amends:** `valid_reference_payload` and the reference-manifest writer in
`scripts/shd_calibration/runner.py`.
**Unblocks:** nothing. That is the point — see §3.

This is the separate amendment that
`AMENDMENT_2026-08-03_REFERENCE_FINGERPRINT_SCOPE.md` §6 asked for and that was
never written:

> *"The narrowing itself is still worth doing for artifacts created from now on,
> so that a future kernel edit does not re-create this situation. That is a
> separate amendment with a correct diagnosis, and is deliberately not being
> smuggled in here."*

---

## 1. What the withdrawn amendment got right, and what it got wrong

**Right (§2, and it still stands):** the reference is a third-party PyTorch
baseline, `Thvnvtos/SNN-delays` at a pinned commit. It never executes a line of
the rust instrument — that independence is the entire reason it is a useful
calibration target. Fingerprinting it against `SOURCE_PATHS`, which includes
`shd_matched.rs`, `shd_instrument.rs` and `Cargo.lock`, ties a PyTorch model's
recorded accuracy to a kernel it does not touch.

**Wrong (§3):** it assumed narrowing the scope would retroactively validate the
six artifacts already frozen. It cannot. A fingerprint is the output of a
function over a set of paths; a value computed over `SOURCE_PATHS` and a value
computed over `REFERENCE_SOURCE_PATHS` are **not a stale and a fresh reading of
the same quantity**. The expectation registered in that document's §5 failed, and
the change was reverted rather than patched. That was correct.

## 2. The treadmill is not hypothetical, and it moved today

The stored fingerprint on all six artifacts is `64923d64655d86ee…`, frozen
2026-07-27. Measured today:

| scope | fingerprint |
|---|---|
| broad (`SOURCE_PATHS`) | `a4e808e36fb9bfef…` |
| narrow (`REFERENCE_SOURCE_PATHS`) | `50c0fe76a66eda56…` |

The narrow value is the same `50c0fe76a66eda56…` the withdrawn amendment
recorded on 2026-08-03. **The broad value is not** — it moved again today,
because `RESULT_2026-08-22_DEEP_PATH_AND_TRANSPORT_SCALE.md` edited
`binn-learn/src/lib.rs`, which is in `SOURCE_PATHS` and which the PyTorch
reference does not import.

That matters for the only code-closeable route out of the calibration gap.
Withdrawn-amendment §6 route 2 is *"re-run the three clean and three historical
reference cells under the current source"*. Without this narrowing, a re-run
today is invalidated by the next unrelated kernel edit — and this repository
edits the kernel most days. Route 2 would be a treadmill.

## 3. The change, and its deliberate limits

The narrow scope applies **only to artifacts that declare it.** Each reference
manifest now records `fingerprint_scope`, and `reference_fingerprint_matches`
reads it back and checks against the matching function. Artifacts that declare
nothing — every artifact frozen before today — default to `source` and are
checked against the broad fingerprint exactly as before.

An unrecognised scope is **refused**, not defaulted. A value this code does not
understand is a reason to reject an artifact, never a reason to fall back to
whichever check it happens to pass.

The manifest also records `instrument_source_fingerprint_at_freeze` for
provenance. **Nothing validates against it.** It is what the broad scope happened
to be at freeze time and it is expected to drift.

## 4. Falsifiable expectation, recorded before running

Unlike the withdrawn amendment, this one expects **no gate to move**:

1. `clean_reference`, `historical_reference` and `matrix_authorized` all stay
   **false**.
2. All six archived reference manifests remain invalid, failing on fingerprint
   and only on fingerprint.
3. A manifest declaring `fingerprint_scope: "reference"` with the current narrow
   fingerprint validates; the same manifest with the broad fingerprint does not.

If any gate moves, the change has done something it was not registered to do and
must be reverted rather than patched — the same rule that fired on 2026-08-03.

### Outcome

All three hold, measured rather than argued:

```
broad : a4e808e36fb9bfef    narrow: 50c0fe76a66eda56
clean      5170001..3  scope=source  stored=64923d64655d86ee  accepted=False
historical 5170001..3  scope=source  stored=64923d64655d86ee  accepted=False
```

`gates.json` is byte-unchanged: `clean_reference` false, `historical_reference`
false, `matrix_authorized` false. Five tests pin all of it, including one that
fails if the two scopes ever produce the same fingerprint — without which every
other test in the group would pass vacuously. The guard was verified falsifiable
by relaxing it to accept either fingerprint; three of the five went red.

## 5. What is still not closed

This changes nothing about the calibration gap today. Restoring
`clean_reference` and `historical_reference` still requires one of the two routes
in the withdrawn amendment's §6:

1. **Accepting the six artifacts on content evidence.** All six match on
   `result_sha256`, `log_sha256`, `manifest_sha256`, mode, seed and
   `mechanical_status: COMPLETE`; only the fingerprint fails. This is the same
   human decision already pending on
   `AMENDMENT_2026-08-03_PROVENANCE_DISCHARGE_BY_BIT_IDENTITY.md`, and it is not
   a code fix.
2. **Re-running the six reference cells.** Needs the pinned upstream checkout and
   `dcls`, which is **not installed** in `.venv-shd-cal`
   (`FINDING_2026-08-21_CALIBRATION_GAP_IS_PROVENANCE_NOT_ACCURACY.md` §7), plus
   compute. Adding that dependency is the maintainer's call.

What this amendment buys is that route 2, once taken, **stays** taken.

`SHD_INSTRUMENT_STATE` is untouched and remains `Uncalibrated`. Nothing here
moves it, and nothing here is a step toward moving it by any route other than the
registered criteria.
