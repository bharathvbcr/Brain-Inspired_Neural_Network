# Result — wave 10, the attention read-out across the resolution ladder

**Prereg:** `PREREG_2026-08-22_SHD_ATTENTION_RESOLUTION_LADDER.md`, registered
before a single cell ran. The analyser (`scripts/aws/analyse_wave10.py`) was
written and frozen before the first cell landed, so nothing below was shaped by
looking at the data.

**Replaces:** the refuted S-5 temporal-resolution claim
(`RESULT_2026-08-21_W8_HEADLINE_SCOPE_IS_MEASURED.md`). S-5 asked the question on
the `published-Nms` family, where bin width and sequence length move together;
this wave asks it on the `fixed-tN` family, where the 1400 ms window is fixed and
only the number of frames changes, so resolution varies on its own.

**Campaign:** 72 cells, 12 seeds x 3 contracts x 2 arms, one pinned binary,
0 failures, 0 voided. Fleet torn down after collection.

All 72 cells from one pinned binary `22d97c51ab02`.

## Measurements

| contract | bin ms | `ff+fixed` | d32/L4 | gain | gain > 0 | ≥ 0.80 |
|---|---:|---:|---:|---:|---:|---:|
| `fixed-t100` | 14.0 | 0.6672 | 0.8599 | **+0.1927** | 12/12 | 12/12 |
| `fixed-t250` | 5.6 | 0.6844 | 0.8594 | **+0.1751** | 12/12 | 12/12 |
| `fixed-t500` | 2.8 | 0.7069 | 0.8543 | **+0.1474** | 12/12 | 12/12 |

**Validity gates: all 72 cells pass.**

**Stability notes: none — no cell exceeded the recorded peak gradient norm, and no cell was clipped.**

## Registered verdicts

**C-1** the read-out helps at every resolution: fixed-t100 +0.1927/12of12; fixed-t250 +0.1751/12of12; fixed-t500 +0.1474/12of12; bar ≥ +0.05 and ≥ 10/12 each -> **SUPPORTED**

**C-2** *(two-sided)* gain depends on resolution: gain(t500) − gain(t100) = **-0.0453**, |·| bar 0.03 -> **SUPPORTED**
  - direction: **falls with t (finer resolution)**

**C-3** baseline drift across the ladder: `ff+fixed` t500 − t100 = **+0.0397** (confound bar 0.05) -> **not confounded** — the baseline is stable, so C-2 is about the read-out.

**C-4** rungs clearing the registered gate:
  - `fixed-t100`: mean 0.8599, 12/12 seeds ≥ 0.8 -> **SUPPORTED**
  - `fixed-t250`: mean 0.8594, 12/12 seeds ≥ 0.8 -> **SUPPORTED**
  - `fixed-t500`: mean 0.8543, 12/12 seeds ≥ 0.8 -> **SUPPORTED**

**C-5** stability: 0 non-finite events, 0 incomplete cells across 72 -> **SUPPORTED**

## Cross-cloud check (prereg §5)

aarch64 (this wave) vs x86-64 (Azure `az8con` fixed-t250): **19320 float values, 0 differing**.

The registered expectation held; the reproducibility finding survives a test it could have failed.

## What this may not claim

- **It is one geometry and one width.** `adjacent-sum-5` at h128, d32/L4. The
  ladder says nothing about `channels-700`, where the headline arm misses the
  gate at 0.7864 (S-1), and nothing about other widths, where S-3 is negative.
- **C-2's direction is a measurement, not a mechanism.** The gain falls as
  resolution gets finer. Why it falls is untested: a finer ladder gives the plain
  arm more to work with (its own accuracy rises +0.0397 across the ladder, which
  C-3 confirms is inside the confound bar), and disentangling that from a
  property of the read-out needs a separate design.
- **The cross-cloud check is a reproducibility result, not a scientific one.**
  19,320 float values agreeing across aarch64 and x86-64 says the instrument is
  deterministic across ISAs under glibc. It says nothing about whether the
  measurement is right.
