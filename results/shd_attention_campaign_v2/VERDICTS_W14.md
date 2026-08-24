# Wave 14 — attention on a recurrent substrate, at the operating point

Prereg: `PREREG_2026-08-23_RECURRENT_MEASUREMENT.md` §4. h128 / `published-2ms` / `adjacent-sum-5` / e400, **surrogate scale 0.4 on every arm**, same pinned binary. 36 new cells; `rec+alif` reused from wave 13.

## Completion

| arm | completed | voided | diverged |
|---|---:|---:|---:|
| `ff+fixed` | **12/12** | 0 | 0 |
| `ff+fixed+attn` | **12/12** | 0 | 0 |
| `rec+alif` | **11/12** | 0 | 1 |
| `rec+alif+attn` | **11/12** | 0 | 1 |

## Paired gains, over seeds where both arms completed

| substrate | pairs | rate read-out | + attention d32/L4 | gain | per-pair range |
|---|---:|---:|---:|---:|---|
| `rec+alif` | 10 | 0.5262 | 0.7874 | **+0.2612** | +0.1886 to +0.4329 |
| `ff+fixed` | 12 | 0.7088 | 0.8289 | **+0.1201** | +0.0914 to +0.1484 |

## Registered verdicts

**M-1** *(primary)* attention helps a recurrent, adaptive substrate: gain **+0.2612** (bar +0.05), positive in **10/10** pairs (bar 10) -> **SUPPORTED**

**M-2** *(primary, two-sided)* the gain depends on whether the substrate is recurrent: gain(`rec+alif`) **+0.2612** vs gain(`ff+fixed`) **+0.1201**, difference **+0.1411**; bar |Δ| ≥ 0.03 -> **SUPPORTED**
  - Larger on the recurrent substrate: attention and recurrence are complementary, and that needs its own explanation rather than an assumption.

**M-3** recurrence plus adaptation alone reaches the gate: `rec+alif` mean **0.5200** (bar 0.8), **0/11** completing seeds ≥ 0.8 (bar 9); `ff+fixed` at the same scale is 0.7088 -> **NOT SUPPORTED**

**M-4** *(descriptive, no verdict)* the scale is not quietly crippling the baseline: `ff+fixed` at 0.4 is **0.7088** against the archived 0.7062 at 1.0, a difference of **+0.0026**.

**Stability notes: 20**, registered as non-voiding.
- `rec+alif` s5170001: peak gradient norm 7.758e+15 exceeds every cell in the recorded campaign (max 1.13e8)
- `rec+alif` s5170003: peak gradient norm 2.215e+17 exceeds every cell in the recorded campaign (max 1.13e8)
- `rec+alif` s5170004: peak gradient norm 3.990e+09 exceeds every cell in the recorded campaign (max 1.13e8)
- `rec+alif` s5170005: peak gradient norm 7.333e+10 exceeds every cell in the recorded campaign (max 1.13e8)
- `rec+alif` s5170006: peak gradient norm 1.511e+29 exceeds every cell in the recorded campaign (max 1.13e8)
- `rec+alif` s5170007: peak gradient norm 4.100e+10 exceeds every cell in the recorded campaign (max 1.13e8)
- `rec+alif` s5170008: peak gradient norm 4.946e+32 exceeds every cell in the recorded campaign (max 1.13e8)
- `rec+alif` s5170009: peak gradient norm 8.143e+31 exceeds every cell in the recorded campaign (max 1.13e8)
- … and 12 more

## Cells that did not complete

- `rec+alif` s5170002: no cell emitted
- `rec+alif+attn` s5170012: diverged at optimizer step 4482

## Scope

- One scale (0.4), one width, one contract, one budget. The anchor runs at scale 1.0, so this is **not** the anchor.
- Nothing about `rec+fixed`: wave 13 measured it and it does not complete.
- Not calibration. No comparison to macOS-recorded numbers.
