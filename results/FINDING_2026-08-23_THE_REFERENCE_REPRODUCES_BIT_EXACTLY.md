# Finding — the archived reference was genuine, and reproduces bit-exactly a month later

**Registered run:** `PREREG_2026-08-22_REFERENCE_RERUN.md`, criterion R-4.
**Status:** first of six cells complete; the other five are running.

---

## 1. The number

A 150-epoch PyTorch training run of the pinned third-party baseline
(`Thvnvtos/SNN-delays`), on CPU, seed 5170001, clean protocol:

| | accuracy |
|---|---|
| archived 2026-07-27 | `0.9389628343621399` |
| re-run 2026-08-23 | `0.9389628343621399` |
| identical | **yes, all 16 digits** |

Every other recorded field matches too — mode, seed, epochs, schema,
`test_reads_during_training: 0`, `final_test_reads: 1`. Only `wall_secs` differs
(19,910 s, 5.53 hours).

## 2. What it establishes

**The archived artifacts were genuine.** This is the load-bearing claim of
`FINDING_2026-08-21_CALIBRATION_GAP_IS_PROVENANCE_NOT_ACCURACY.md`, and it is now
demonstrated rather than argued: the gate was false for a **provenance** reason
and nothing else. The recorded accuracy was correct, and a fresh run under the
current source reproduces it exactly.

**And the run is deterministic across a month and a rebuilt environment.** Same
bits from a 150-epoch stochastic training run, a month apart. That is a stronger
reproducibility result than anything the campaign has recorded for the Rust
instrument, and it came from the *reference*, not from BINN.

**R-4 is met for this seed**: 0.9389628 against the 0.80 calibration floor.

## 3. What it does not establish

- **The gates have not moved.** `clean_reference`, `historical_reference` and
  `matrix_authorized` are all still `false`, and stay false until all six cells
  land and `update_reference_gates()` reads them. Two more clean seeds and three
  historical are running.
- **It does not retroactively validate the archived artifacts.** They remain
  archived and invalid — their training logs are gone
  (`FINDING_2026-08-22_THE_REFERENCE_LOGS_ARE_GONE.md`) and a recorded hash with
  no file behind it is not evidence. The re-run *replaces* them; it does not
  vouch for them.
- **The historical arm is the harder half.** Its accuracy is parsed out of the
  training log by `parse_historical_validation_curve`, which requires 150
  contiguous epoch records. R-2 is a real risk there in a way it was not here.

## 4. The cost, recorded because it was underestimated

**5.53 hours per cell on CPU**, against the 4.45 hours the code comment
predicted. Six cells is roughly 33 CPU-hours; three at a time on this host puts
the full set at 11–12 hours wall.

The `BINN_SHD_TORCH_DEVICE=mps` escape hatch would cut that substantially and was
deliberately not used: the historical arm has no such hatch, so taking it for the
clean arm would run the two halves of the calibration on different backends, and
`clean_main.py`'s own comment records that MPS and CPU do not produce bit-identical
float results. Given §1, that caution was worth it — this cell would not have
matched to 16 digits on a different backend.
