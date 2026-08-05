# Overnight run summary — 2026-07-25_202434

Run dir: `results/runs/2026-07-25_195647_overnight`

| Job | Status | Wall (s) |
|---|---|---:|
| gate_check | skipped | 0 |
| gate_check_gpu | skipped | 0 |
| gate_test | skipped | 0 |
| gate_clippy | skipped | 0 |
| build_release | skipped | 0 |
| smoke_arch_ablation | skipped | 0 |
| smoke_arch_lr_sweep | skipped | 0 |
| smoke_c1_enhanced | skipped | 0 |
| smoke_multi_area | skipped | 0 |
| smoke_deep_snn | skipped | 0 |
| smoke_ei_sweep | skipped | 0 |
| smoke_neuromod | skipped | 0 |
| smoke_shd_cal | skipped | 0 |

## Read these first

1. `shd_arch_ablation_h128.md` — **the decisive result.** Check the
   preregistered H1/H2 verdicts and the shuffled-label control.
2. Any job with status `fail` or `timeout` — see `logs/<job>.log`.
3. Grep every report for harness flags:

   ```bash
   grep -rn 'INVALID_HARNESS\|DEGENERATE\|INVERTED\|LEAK DETECTED\|MISMATCH' results/runs/2026-07-25_195647_overnight/*.md
   ```

## Interpretation cheat-sheet

- **H1 PASS** → the 0.234 figure was an architecture artifact. Restate the
  SHD claim axis; re-run width/depth sweeps on the winning architecture.
- **H1 FAIL** → architecture is not the constraint. That is a real negative
  result, but verify the learning-rate sweep and the exact ALIF eligibility
  term before writing it up.
- **INVALID_HARNESS anywhere** → no claim from that run, full stop.
- A `readout arm ... is degenerate` panic is the guard working, not a flake.
