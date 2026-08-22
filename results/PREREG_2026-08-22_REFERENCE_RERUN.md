# Preregistration — re-run the six SHD reference cells

**Registered:** 2026-08-22, **before any reference cell is re-run and before any
post-run number exists.** Authorised by the maintainer, who chose this route over
discharging the gates by judgement.

**Closes, if it succeeds:** `clean_reference` and `historical_reference`, and
therefore `matrix_authorized`.

---

## 1. Why this is the chosen route

`FINDING_2026-08-21_CALIBRATION_GAP_IS_PROVENANCE_NOT_ACCURACY.md` established
that the two false gates are false for a **provenance** reason, not an accuracy
one. All six artifacts pass every content check — `result_sha256`, `log_sha256`,
`manifest_sha256`, mode, seed, `mechanical_status: COMPLETE` — and the recorded
accuracies already clear the floor:

| mode | seeds 5170001 / 5170002 / 5170003 | floor |
|---|---|---|
| clean | 0.9390 / 0.9368 / 0.9371 | 0.80 |
| historical | mean 0.9498 | within 0.05 of the published 0.951 |

Only `source_fingerprint` fails, because it was frozen 2026-07-27 over a path set
that includes the rust kernel — which the reference does not execute.

`AMENDMENT_2026-08-03_REFERENCE_FINGERPRINT_SCOPE.md` §6 names three routes.
Route 3 is unavailable. Route 1 is a human judgement call and remains open.
**Route 2 — re-running the six cells under the current source — is the only one
that closes the gates without a judgement call, and it is the one chosen.**

`AMENDMENT_2026-08-22_REFERENCE_FINGERPRINT_SCOPE_FORWARD.md` was registered and
implemented first, so that artifacts produced by this run are fingerprinted over
the reference pipeline alone and are not invalidated by the next unrelated kernel
edit. Without it this run would be a treadmill: the broad fingerprint moved again
today, on an edit to `binn-learn/src/lib.rs` that the PyTorch reference does not
import.

## 2. A correction to the record, made before relying on it

`FINDING_2026-08-21_CALIBRATION_GAP_IS_PROVENANCE_NOT_ACCURACY.md` §7 states that
`dcls` is **not installed** in `.venv-shd-cal` and that "the run has not been
attempted", and this was carried forward as the blocker on route 2.

**That is wrong.** The package is `dcls` on PyPI and imports as `DCLS`. The
repository's own dependency check —
`runner.setup_reference`'s `import torch, h5py, DCLS, spikingjelly` — passes today
against the existing venv:

```
$ .venv-shd-cal/bin/python -c "import torch, h5py, DCLS, spikingjelly; \
    from spikingjelly.datasets.shd import SpikingHeidelbergDigits; print(torch.__version__)"
2.13.0
```

No dependency needs adding. Route 2 has been available the whole time and was
recorded as blocked on a case-sensitivity error.

## 3. What is being run, and what is not being touched

> **Note, same day:** `runner.py` has no `__main__` block, so
> `python -m scripts.shd_calibration.runner` imports it and silently does
> nothing, exiting 0. The entry point is `scripts/run_shd_instrument.py`. This
> was caught before any cell ran, by noticing that a 150-epoch training run
> returned instantly with empty output.

`.venv-shd-cal/bin/python scripts/run_shd_instrument.py reference
--mode {historical,clean} --seed {5170001,5170002,5170003}`, six cells, using the
repository's own
orchestration exactly as written. Each clones `Thvnvtos/SNN-delays` at the pinned
`REFERENCE_COMMIT` into a fresh detached worktree and runs it under PyTorch for
150 epochs.

**Nothing in the pipeline is modified.** No threshold, no seed, no epoch count, no
`REFERENCE_COMMIT`, no `CALIBRATION_FLOOR`, no `PUBLISHED_ACCURACY`, and no gate is
written by hand. `SHD_INSTRUMENT_STATE` is untouched and stays `Uncalibrated`;
this run does not move it and is not a step toward moving it by any route other
than the registered criteria.

The four prerequisite gates `data_parity`, `forward_parity`, `gradient_parity` and
`update_parity` are already true; `reference()` refuses to run otherwise, and that
refusal is left in place.

## 4. Registered acceptance criteria, fixed before the run

| id | criterion | bar |
|---|---|---|
| **R-1** | every cell completes | 6/6 `mechanical_status: COMPLETE`, no non-zero exit |
| **R-2** | the historical log is fully parsed | 150 epochs, contiguous 0..149, by the existing parser — not relaxed |
| **R-3** | the clean arm reads the official test set exactly once | `verify_clean_source` passes, unmodified |
| **R-4** | clean accuracy clears the floor | all three seeds ≥ `CALIBRATION_FLOOR` (0.80) |
| **R-5** | historical mean matches the published figure | within 0.05 of `PUBLISHED_ACCURACY` (0.951) |
| **R-6** | the gates flip **by the code**, not by hand | `update_reference_gates()` sets `clean_reference` and `historical_reference` true from the artifacts alone |
| **R-7** | the new artifacts declare the narrow scope | `fingerprint_scope: "reference"` present in all six manifests |

## 5. Named outcomes

- **R-1…R-7 all hold** → `matrix_authorized` becomes true if its other conjuncts
  do. The calibration gap closes on evidence, and the pending human judgement in
  `AMENDMENT_2026-08-03_PROVENANCE_DISCHARGE_BY_BIT_IDENTITY.md` is **no longer
  needed** for these two gates and is not taken.
- **R-1 or R-2 fails** → the run is incomplete. **No partial credit**: gates stay
  false and the failure is reported with the log. A reference that did not finish
  is not a reference.
- **R-4 or R-5 fails** → this is the interesting outcome. The recorded 2026-07
  accuracies would then **not reproduce** under the current environment, which
  would mean the artifacts are not merely stale in provenance but wrong in
  substance — and would retroactively justify the gates having been false. It is
  reported as a finding whatever it costs, and the gates stay false.
- **The two disagree** — new numbers reproduce but the gates do not flip → a
  defect in `valid_reference_payload` or in the manifest writer. Reported, not
  patched around.

## 6. What this may not claim

- **It does not calibrate the instrument.** `matrix_authorized` and
  `SHD_INSTRUMENT_STATE` are two gates in series. Flipping the first does not
  flip the second, which needs all five registered criteria including a matched
  Python/Rust configuration, and no Python mirror of the attention axis exists
  (`BLOCKER_2026-08-19_FROZEN_ATTENTION_LOCAL_ARM.md`).
- **It does not validate the old artifacts.** They stay invalid. New cells replace
  them; they are not retroactively accepted, and the forward-only narrowing was
  written specifically so that cannot happen by accident.
- **It changes no published number.** Nothing in the paper depends on these gates
  today, precisely because they have been false.
