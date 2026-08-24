# Result — the calibration gates are open, and every cell reproduced bit-exactly

**Prereg:** `PREREG_2026-08-22_REFERENCE_RERUN.md`.
**Run:** 2026-08-22 21:41 → 2026-08-23 08:08 UTC, six cells, ~33 CPU-hours.

**All seven criteria met. `matrix_authorized` is `true`.**

---

## 1. Every cell reproduced its 2026-07 value exactly

| cell | archived 2026-07-27 | re-run 2026-08-23 | identical | epochs parsed |
|---|---|---|---|---|
| clean 5170001 | `0.9389628343621399` | `0.9389628343621399` | **yes** | n/a |
| clean 5170002 | `0.9367766203703705` | `0.9367766203703705` | **yes** | n/a |
| clean 5170003 | `0.9371463477366255` | `0.9371463477366255` | **yes** | n/a |
| historical 5170001 | `0.9447` | `0.9447` | **yes** | 150/150 |
| historical 5170002 | `0.9573` | `0.9573` | **yes** | 150/150 |
| historical 5170003 | `0.9475` | `0.9475` | **yes** | 150/150 |

**Six for six, to every recorded digit.** A 150-epoch stochastic PyTorch training
run, on CPU, a month apart, in a rebuilt environment.

This settles what `FINDING_2026-08-21_CALIBRATION_GAP_IS_PROVENANCE_NOT_ACCURACY.md`
argued: **the gate was false for a provenance reason and nothing else.** The
recorded accuracies were right the whole time. The artifacts were not wrong; their
fingerprint was stale and their logs had been deleted.

## 2. Criteria

| id | criterion | outcome |
|---|---|---|
| **R-1** | every cell completes | **MET** — 6/6, after two were re-run (§4) |
| **R-2** | historical log fully parsed, 150 contiguous epochs | **MET** — 150/150 on all three. This was the flagged risk and it did not bite. |
| **R-3** | clean arm reads the official test set exactly once | **MET** — `test_reads_during_training: 0`, `final_test_reads: 1` |
| **R-4** | clean accuracy clears the floor | **MET** — 0.9390 / 0.9368 / 0.9371 against 0.80 |
| **R-5** | historical mean matches the published figure | **MET** — 0.9498333 against 0.951, difference 0.0012, bar 0.05 |
| **R-6** | gates flip **by the code** | **MET** — `reference()` calls `update_reference_gates()` after each cell; the sixth landing flipped them. `gates.json` was never hand-edited. |
| **R-7** | new artifacts declare the narrow scope | **MET** — `fingerprint_scope: reference` on all six manifests |

## 3. The gates

```json
"clean_reference": true,
"historical_reference": true,
"matrix_authorized": true,
"clean_accuracies": [0.9389628343621399, 0.9367766203703705, 0.9371463477366255],
"historical_mean_accuracy": 0.9498333333333333
```

`harness_status` moved `PENDING_PREREQUISITES → VALID`.

**`scientific_status` is still `UNCALIBRATED`, and that is correct.**
`matrix_authorized` and `SHD_INSTRUMENT_STATE` are two gates in series. The first
is now open; the second is a compile-time constant in
`binn-lab/src/instrument_status.rs`, conditioned on all five registered criteria
including a matched Python/Rust configuration for which no Python mirror of the
attention axis exists. **Nothing here moves it, and nothing here should.**

## 4. What went wrong on the way

Three historical cells were launched simultaneously and two died in the same
second on a git index-lock race in `ensure_checkout` — recorded in
`DEFECT_2026-08-23_REFERENCE_SETUP_HAS_NO_LOCK.md`. They were recovered with a
180-second stagger, which is timing rather than synchronisation.

The failure went unnoticed for two hours because the watcher tracked artifact
count and process liveness, and with one cell still training those look exactly
like healthy progress. **A watcher that cannot distinguish "still working" from
"two of these are already dead" is the same defect shape this workspace keeps
finding, and I wrote it.**

The lock fix cannot be applied yet: `reference.py` is in `REFERENCE_SOURCE_PATHS`,
so editing it moves the narrow fingerprint and would invalidate all six cells that
just cost 33 CPU-hours. It must land as its own registered provenance event, with
the reference set produced entirely before or entirely after — never straddling.

## 5. What is now unblocked, and what is not

**Unblocked:** the 136 remaining cells of the 432-cell matrix, which
`matrix_authorized` gated at `runner.py:1038`.

**Still blocked:** everything behind `SHD_INSTRUMENT_STATE`. `shd-frozen-attention`,
`shd-arch-ablation`, `shd-scientific-sweep`, `temporal-deep-campaign`,
`shd-depth-scaling` and the rest still exit 2 at `authorize_campaign`. The SHD
depth suite — the thing `RESULT_2026-08-23_DEPTH_ON_A_TASK_WITH_HEADROOM.md` was
a substitute for — remains refused.

**Not claimed:** that the archived artifacts are valid. They stay archived and
invalid; their logs are gone and a recorded hash with no file behind it is not
evidence. These six replace them. The 2026-08-03 route-1 judgement call is now
**moot rather than resolved** — it was never taken.

## 6. Cost

33 CPU-hours, ~10.5 hours wall at three concurrent, 5.5 h/cell against the 4.45 h
the code comment predicted. CPU throughout: the `BINN_SHD_TORCH_DEVICE=mps` hatch
was declined because the historical arm has none, and `clean_main.py` records that
MPS and CPU do not produce bit-identical floats. Given §1, that was the decision
the whole exercise turned on.
