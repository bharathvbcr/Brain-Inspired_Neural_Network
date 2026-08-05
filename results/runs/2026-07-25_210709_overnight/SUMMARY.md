# Overnight run summary — 2026-07-25_211707

Run dir: `results/runs/2026-07-25_210709_overnight`

| Job | Status | Wall (s) |
|---|---|---:|
| gate_check | ok | 0 |
| gate_check_gpu | ok | 1 |
| gate_test | ok | 143 |
| gate_clippy | ok | 0 |
| build_release | ok | 0 |
| smoke_arch_ablation | ok | 29 |
| smoke_arch_lr_sweep | ok | 107 |
| smoke_c1_enhanced | ok | 0 |
| smoke_multi_area | ok | 0 |
| smoke_deep_snn | ok | 22 |
| smoke_ei_sweep | ok | 0 |
| smoke_neuromod | ok | 1 |
| smoke_shd_cal | ok | 0 |
| shd_arch_lr_pilot | ok | 1044 |
| shd_arch_ablation_h128 | ok | 2115 |
| shd_cal_h128 | ok | 3933 |
| track_b_rescue | ok | 393 |
| live_transfer_rescue | ok | 296 |
| deep_snn_scaling | ok | 2266 |
| multi_area_scaling | fail:101 | 0 |
| c1_enhanced | ok | 1 |
| ei_inhibition_sweep | ok | 0 |
| multi_channel_neuromod | ok | 0 |

## Read these first

1. `shd_arch_ablation_h128.md` — **the decisive result.** Check the
   preregistered H1/H2 verdicts and the shuffled-label control.
2. Any job with status `fail` or `timeout` — see `logs/<job>.log`.
3. Grep every report for harness flags:

   ```bash
   grep -rn 'INVALID_HARNESS\|DEGENERATE\|INVERTED\|LEAK DETECTED\|MISMATCH' results/runs/2026-07-25_210709_overnight/*.md
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
