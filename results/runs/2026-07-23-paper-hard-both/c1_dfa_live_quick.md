# C1 / Gate G2 results note

**Config hash:** `c1-c62511dff8a4f508`

**Scientific protocol version:** `20`

**claim_axis:** Novel-CS
**object_under_test:** Graded DFA credit on live muted-θ / k-WTA C1
**may_claim:** Whether matched DFA PASS transfers under one honest live map
**must_not_claim:** Remassage matched `c1-dfa-*` / P4 spike-DFA; biology; impossibility

**Live graded-DFA transfer protocol:** `20` — same muted-θ / k-WTA / single-pass C1 substrate as v2/v13; main-condition plasticity uses graded readout error × fixed-random DFA feedback (`FixedRandomFeedback`) through three-factor eligibility; observe-only on incorrect; **positive control stays on broadcast ±1**; does **not** remassage matched `c1-dfa-c8c4fe0899908b84`, P4 spiking-DFA, or reopen protocol-v2 `c1-118207fbc3eaba53`.

**Verdict (Gate G2):** **PILOT**

PASS = lower confidence bound on normalized gradient gap closed > 0.500 and mean local accuracy >= 0.650.
FAIL = a full run missed at least one preregistered threshold.
PILOT = quick schedule or fewer seeds than the power-analysis requirement; not a scientific G2 decision.
INVALID_HARNESS = positive_control_mean < 0.900 or mean activity sparsity outside [0.0050, 0.0300]; prohibits PASS/FAIL and U-NEG language.

## Conditions

| Label | Meaning |
|---|---|
| `local-assembly` | Three-factor rule + sparse assembly wiring + k-WTA + dual readouts + two-sided ±1 reward |
| `dense-local` | Same three-factor rule + same k-winner budget on dense all-to-all connectivity, **no** assembly structure |
| `gradient-reference` | Same-architecture surrogate-LIF BPTT (primary); tanh RNN optional/secondary |
| `eligibility-reference` | E-prop-compatible eligibility local reference (rate-model approximation; feedforward-only) |

Plasticity uses hard ±1 reward by design (soft RPE deferred). Gap-closed is clamped to `[0, 1]` and seeds with `(reference − dense) < 0.150` contribute `closed = 0`.

## Config

```
Config { experiment: "c1-dfa-live", master_seed: 212618061021185, n_seeds: 5, sequence_len: 8, max_lag: 1, n_hidden: 64, k_wta: 1, p_sparse: 0.35, init_w: 0.15, eta: 0.35, lambda: 0.002, tau_e: 40.0, n_train: 24, n_test: 16, bptt_epochs: 40, bptt_lr: 0.05, g2_min_gap_closed: 0.5, g2_min_accuracy: 0.65, g2_confidence_z: 1.96, g2_min_positive_control: 0.9, g2_min_reference_gap: 0.15, activity_sparsity_min: 0.005, activity_sparsity_max: 0.03, scientific_n_seeds: 20, power_sigma_prior: 0.15, power_effect_size: 0.1, use_surrogate_lif_reference: true, surrogate_beta: 5.0, matched_budget_repeat: false, quick: true }
```

Power analysis: required scientific n_seeds ≥ 20 (preregistered σ=0.150, effect=0.100; formula n=⌈(1.96+0.8416)²σ²/δ²⌉).

## Per-seed accuracies

| seed | local-assembly | dense-local | gradient reference | eligibility reference | activity_sparsity (local) | activity_sparsity (dense) | dense_matched |
|---|---:|---:|---:|---:|---:|---:|---:|
| 11400784225994701844 | 0.5000 | 0.1875 | 0.7500 | 0.8750 | 0.0156 | 0.0156 | — |
| 4354472946875824171 | 0.8125 | 0.1875 | 0.5000 | 0.9375 | 0.0156 | 0.0156 | — |
| 15755469790931547198 | 0.5000 | 0.4375 | 0.5000 | 0.9375 | 0.0156 | 0.0156 | — |
| 8709160710835925077 | 0.5000 | 0.2500 | 1.0000 | 0.9375 | 0.0156 | 0.0156 | — |
| 1663413756060003432 | 0.8125 | 0.2500 | 0.5000 | 0.9375 | 0.0156 | 0.0156 | — |

## Summary (paired normalized-gap analysis)

- mean ± var local-assembly: 0.6250 ± 0.029297
- mean ± var dense-local:    0.2625 ± 0.010547
- mean ± var gradient reference: 0.6500 ± 0.050000
- mean ± var eligibility reference: 0.9250 ± 0.000781
- mean normalized gap closed: 0.5778 (variance 0.187654)
- lower confidence bound (z=1.960, n=5): 0.1981
- mean |local − dense| (descriptive): 0.3625
- descriptive chance-normalized gap mean / LCB: 0.0000 / 0.0000 (var 0.000000; **not a gate**)
- seed local min / max / frac≥0.65: 0.5000 / 0.8125 / 0.40

## Pilot limitation

This run uses a quick schedule or fewer seeds than the power-analysis requirement. It validates the harness only and is not evidence for passing or failing G2.

## Positive / sanity control

Mean local-pipeline accuracy on a trivially separable spatial feature-presence task: **0.9417** (threshold 0.900).

## Activity sparsity

Mean local-assembly activity_sparsity: **0.0156** (valid band [0.0050, 0.0300]; nominal k/N=0.0156).

## Parameter / compute budgets

| condition | n_cells | n_params | wall_secs | peak_rss_bytes | work_per_accuracy | spikes | deliveries | cell_updates | plasticity |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| local-assembly | 68 | 1480 | 0.0045 | 3227648 | 77014.0000 | 175 | 1320 | 1492 | 35520 |
| dense-local | 68 | 4288 | 0.0036 | 3424256 | 633365.3333 | 228 | 7720 | 7896 | 102912 |
| gradient-reference | 66 | 4289 | 0.0274 | 2654208 | 6155520.0000 | 0 | 7680 | 491520 | 4117440 |
| eligibility-reference | 66 | 193 | 0.0033 | 2605056 | 782262.8571 | 0 | 7680 | 491520 | 185280 |

## Plots

- raster: Skipped("plots feature disabled (enable --features plots)")
- weights: Skipped("plots feature disabled (enable --features plots)")
- raster: Skipped("plots feature disabled (enable --features plots)")
- weights: Skipped("plots feature disabled (enable --features plots)")
- raster: Skipped("plots feature disabled (enable --features plots)")
- weights: Skipped("plots feature disabled (enable --features plots)")
- raster: Skipped("plots feature disabled (enable --features plots)")
- weights: Skipped("plots feature disabled (enable --features plots)")
- raster: Skipped("plots feature disabled (enable --features plots)")
- weights: Skipped("plots feature disabled (enable --features plots)")

## Structured log (GC7)

```
config_hash=c1-c62511dff8a4f508 seed=11400784225994701844 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=77014.000000 note=wall_secs=0.0045_peak_rss=3227648_spikes=175_deliveries=1320_cells=1492_plasticity=35520
config_hash=c1-c62511dff8a4f508 seed=11400784225994701844 condition=dense-local accuracy=0.187500 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=633365.333333 note=wall_secs=0.0036_peak_rss=3424256_spikes=228_deliveries=7720_cells=7896_plasticity=102912
config_hash=c1-c62511dff8a4f508 seed=11400784225994701844 condition=gradient-reference accuracy=0.750000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=6155520.000000 note=wall_secs=0.0274_peak_rss=2654208_spikes=0_deliveries=7680_cells=491520_plasticity=4117440
config_hash=c1-c62511dff8a4f508 seed=11400784225994701844 condition=eligibility-reference accuracy=0.875000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=782262.857143 note=wall_secs=0.0033_peak_rss=2605056_spikes=0_deliveries=7680_cells=491520_plasticity=185280
config_hash=c1-c62511dff8a4f508 seed=4354472946875824171 condition=local-assembly accuracy=0.812500 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=47388.307692 note=wall_secs=0.0021_peak_rss=3227648_spikes=187_deliveries=1291_cells=1457_plasticity=35568
config_hash=c1-c62511dff8a4f508 seed=4354472946875824171 condition=dense-local accuracy=0.187500 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=633413.333333 note=wall_secs=0.0025_peak_rss=3375104_spikes=235_deliveries=7720_cells=7898_plasticity=102912
config_hash=c1-c62511dff8a4f508 seed=4354472946875824171 condition=gradient-reference accuracy=0.500000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9233280.000000 note=wall_secs=0.0236_peak_rss=2637824_spikes=0_deliveries=7680_cells=491520_plasticity=4117440
config_hash=c1-c62511dff8a4f508 seed=4354472946875824171 condition=eligibility-reference accuracy=0.937500 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=730112.000000 note=wall_secs=0.0027_peak_rss=2605056_spikes=0_deliveries=7680_cells=491520_plasticity=185280
config_hash=c1-c62511dff8a4f508 seed=15755469790931547198 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=76754.000000 note=wall_secs=0.0021_peak_rss=3211264_spikes=181_deliveries=1277_cells=1447_plasticity=35472
config_hash=c1-c62511dff8a4f508 seed=15755469790931547198 condition=dense-local accuracy=0.437500 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=271490.285714 note=wall_secs=0.0024_peak_rss=3407872_spikes=247_deliveries=7720_cells=7898_plasticity=102912
config_hash=c1-c62511dff8a4f508 seed=15755469790931547198 condition=gradient-reference accuracy=0.500000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9233280.000000 note=wall_secs=0.0223_peak_rss=2654208_spikes=0_deliveries=7680_cells=491520_plasticity=4117440
config_hash=c1-c62511dff8a4f508 seed=15755469790931547198 condition=eligibility-reference accuracy=0.937500 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=730112.000000 note=wall_secs=0.0024_peak_rss=2621440_spikes=0_deliveries=7680_cells=491520_plasticity=185280
config_hash=c1-c62511dff8a4f508 seed=8709160710835925077 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=76802.000000 note=wall_secs=0.0019_peak_rss=3178496_spikes=173_deliveries=1280_cells=1452_plasticity=35496
config_hash=c1-c62511dff8a4f508 seed=8709160710835925077 condition=dense-local accuracy=0.250000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=474968.000000 note=wall_secs=0.0024_peak_rss=3407872_spikes=210_deliveries=7720_cells=7900_plasticity=102912
config_hash=c1-c62511dff8a4f508 seed=8709160710835925077 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=4616640.000000 note=wall_secs=0.0222_peak_rss=2654208_spikes=0_deliveries=7680_cells=491520_plasticity=4117440
config_hash=c1-c62511dff8a4f508 seed=8709160710835925077 condition=eligibility-reference accuracy=0.937500 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=730112.000000 note=wall_secs=0.0025_peak_rss=2588672_spikes=0_deliveries=7680_cells=491520_plasticity=185280
config_hash=c1-c62511dff8a4f508 seed=1663413756060003432 condition=local-assembly accuracy=0.812500 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=47488.000000 note=wall_secs=0.0022_peak_rss=3194880_spikes=207_deliveries=1309_cells=1476_plasticity=35592
config_hash=c1-c62511dff8a4f508 seed=1663413756060003432 condition=dense-local accuracy=0.250000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=474964.000000 note=wall_secs=0.0023_peak_rss=3424256_spikes=209_deliveries=7720_cells=7900_plasticity=102912
config_hash=c1-c62511dff8a4f508 seed=1663413756060003432 condition=gradient-reference accuracy=0.500000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9233280.000000 note=wall_secs=0.0229_peak_rss=2637824_spikes=0_deliveries=7680_cells=491520_plasticity=4117440
config_hash=c1-c62511dff8a4f508 seed=1663413756060003432 condition=eligibility-reference accuracy=0.937500 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=730112.000000 note=wall_secs=0.0024_peak_rss=2588672_spikes=0_deliveries=7680_cells=491520_plasticity=185280
```
