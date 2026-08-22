# Finding — the reference training logs were never tracked, and are gone

**Found:** 2026-08-22, while starting the re-run registered in
`PREREG_2026-08-22_REFERENCE_RERUN.md`.

**Consequence:** route 1 of `AMENDMENT_2026-08-03_REFERENCE_FINGERPRINT_SCOPE.md`
§6 — accepting the six reference artifacts on content evidence — is **not
available**. Re-running them is now the only route, not merely the preferred one.

---

## 1. What the record says

Two documents state that the six reference artifacts are intact and fail on one
check only:

- `AMENDMENT_2026-08-03_REFERENCE_FINGERPRINT_SCOPE.md` §1 tabulates all six with
  **`source_fingerprint`** as the failing check, and says: *"All six pass every
  content check — `mechanical_status`, mode, seed, `result_sha256`,
  `log_sha256`, `manifest_sha256`. The artifacts are byte-for-byte what they
  were."*
- `FINDING_2026-08-21_CALIBRATION_GAP_IS_PROVENANCE_NOT_ACCURACY.md` carries that
  forward as the basis for route 1.

## 2. What is actually on disk

```
$ find results -path "*reference*" -name "*.log" | wc -l
0
```

**Zero.** Not in `shd_instrument_v4`, not in the `shd_instrument_v3` tree it was
recovered from, not anywhere.

`valid_reference_payload` opens with

```python
if not all(path.is_file() for path in (result_path, log_path, manifest, state_path)):
    return None
```

so for every one of the six it returns `None` **at that line**, before it reaches
the `source_fingerprint` comparison at all. The recorded failing check is not
wrong about the fingerprint — the fingerprint genuinely does not match — but it is
no longer the *first* failure, and it is no longer the *only* one.

## 3. Why they are gone

`.gitignore` line 43 is `*.log`. The reference training logs were **never
version-controlled**, in any tree, at any point:

```
$ git log --all --oneline -- "*references/*.log"
(no output)
```

They existed when `recover_references` ran — that function reads
`source_root/references/{mode}-seed-{seed}.log` and refuses to proceed without it,
and all six state files carry a `log_sha256`. They were lost afterwards, to an
untracked-file cleanup, and nothing noticed because no check re-reads them.

## 4. Why this is worse than a stale fingerprint

A recorded `log_sha256` with no file behind it is **not evidence**. It is a claim
about a file that cannot be checked.

For the historical arm this is not a bookkeeping detail. That arm's accuracy is
not written by the training script — it is **parsed out of the log** by
`parse_historical_validation_curve`, which requires exactly 150 contiguous epoch
records. The log *is* the measurement. Losing it means the historical accuracies
in `references/historical-seed-*.json` can no longer be re-derived from anything.

So the artifacts are not "byte-for-byte what they were, with a stale
fingerprint". They are incomplete, and the missing piece is the primary record
for three of the six.

## 5. What this changes

- **Route 1 is withdrawn as an option.** Accepting the artifacts on content
  evidence was already a human judgement call; it is now a judgement call that
  cannot be supported, because a third of the content is missing. The maintainer's
  choice of route 2 turns out to have been the only available one.
- **The two documents in §1 are corrected in place**, pointing here.
- **The six artifacts are archived, not deleted**, at
  `results/shd_instrument_v4/references-superseded-2026-07-27/`, with a SHA-256
  index and `training_logs_present: false` recorded in `INDEX.json`. They are
  still cited by the record and they remain the subject of the 2026-08-03
  amendment.

## 6. The gap this leaves open

**Nothing re-reads a recorded hash to check its file still exists.** That is the
class of defect, not the instance. `log_sha256`, `result_sha256` and
`manifest_sha256` are all recorded at freeze time and only ever compared when the
validator happens to run — and the validator returns early on a missing file
rather than reporting *which* file is missing, which is why this was invisible for
three weeks and was recorded as a fingerprint problem instead.

Two changes worth making, neither of which is in scope for the re-run and both of
which are named here so they are not lost:

1. `valid_reference_payload` should say **which** artifact is missing rather than
   returning a bare `None`. A check that cannot explain itself produced this
   misdiagnosis.
2. Reference logs should be tracked, or their absence should be a loud failure at
   freeze time rather than a silent one three weeks later.
