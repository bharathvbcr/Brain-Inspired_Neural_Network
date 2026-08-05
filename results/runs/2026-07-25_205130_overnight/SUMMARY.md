# Overnight run summary — 2026-07-25_205130

Run dir: `results/runs/2026-07-25_205130_overnight`

| Job | Status | Wall (s) |
|---|---|---:|
| gate_check | ok | 2 |
| gate_check_gpu | ok | 0 |
| gate_test | ok | 180 |
| gate_clippy | ok | 7 |
| build_release | ok | 2 |
| smoke_arch_ablation | ok | 29 |
| smoke_arch_lr_sweep | ok | 109 |
| smoke_c1_enhanced | ok | 0 |
| smoke_multi_area | ok | 0 |
| smoke_deep_snn | ok | 23 |
| smoke_ei_sweep | ok | 0 |
| smoke_neuromod | ok | 0 |
| smoke_shd_cal | ok | 0 |
| shd_arch_lr_pilot | pending | — |
| shd_arch_ablation_h128 | pending | — |
| shd_cal_h128 | pending | — |
| track_b_rescue | pending | — |
| live_transfer_rescue | pending | — |
| deep_snn_scaling | pending | — |
| multi_area_scaling | pending | — |
| c1_enhanced | pending | — |
| ei_inhibition_sweep | pending | — |
| multi_channel_neuromod | pending | — |

## Read these first

1. `smoke_shd_arch_ablation.md` — execution and validity guards only;
   quick H1/H2 verdicts must remain `UNDERPOWERED`.
2. `smoke_ei_sweep.md` and `smoke_neuromod.md` — repaired property tests.
3. Tier 2+ is `pending`; this smoke run is not the decisive result.

   ```bash
   grep -rn 'INVALID_HARNESS\|DEGENERATE\|INVERTED\|LEAK DETECTED\|MISMATCH' results/runs/2026-07-25_205130_overnight/*.md
   ```

## Interpretation cheat-sheet

- **H1 PASS** → the 0.234 figure was an architecture artifact. Restate the
  SHD claim axis; re-run width/depth sweeps on the winning architecture.
- **H1 FAIL** → architecture is not the constraint. That is a real negative
  result only on a confirmatory schedule. Protocol v142 includes the exact
  ALIF adaptation term; keep the learning-rate sweep pilot-only.
- **INVALID_HARNESS anywhere** → no claim from that run, full stop.
- A `readout arm ... is degenerate` panic is the guard working, not a flake.

Execution status only records whether a process completed. Scientific
PASS/FAIL/INVALID_HARNESS verdicts live in the generated reports.
