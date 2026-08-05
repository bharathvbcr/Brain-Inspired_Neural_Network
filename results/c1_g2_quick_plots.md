# C1 / Gate G2 results note

**Config hash:** `c1-e0dfdbf4e3d2936b`

**Scientific protocol version:** `2`

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
Config { experiment: "c1", master_seed: 212618061021185, n_seeds: 5, sequence_len: 8, max_lag: 1, n_hidden: 64, k_wta: 1, p_sparse: 0.35, init_w: 0.15, eta: 0.35, lambda: 0.002, tau_e: 40.0, n_train: 24, n_test: 16, bptt_epochs: 40, bptt_lr: 0.05, g2_min_gap_closed: 0.5, g2_min_accuracy: 0.65, g2_confidence_z: 1.96, g2_min_positive_control: 0.9, g2_min_reference_gap: 0.15, activity_sparsity_min: 0.005, activity_sparsity_max: 0.03, scientific_n_seeds: 20, power_sigma_prior: 0.15, power_effect_size: 0.1, use_surrogate_lif_reference: true, surrogate_beta: 5.0, matched_budget_repeat: false, quick: true }
```

Power analysis: required scientific n_seeds ≥ 20 (preregistered σ=0.150, effect=0.100; formula n=⌈(1.96+0.8416)²σ²/δ²⌉).

## Per-seed accuracies

| seed | local-assembly | dense-local | gradient reference | eligibility reference | activity_sparsity (local) | activity_sparsity (dense) | dense_matched |
|---|---:|---:|---:|---:|---:|---:|---:|
| 11400784225994701844 | 0.5000 | 0.5000 | 0.7500 | 0.8750 | 0.0156 | 0.0156 | — |
| 4354472946875824171 | 0.5625 | 0.5000 | 0.5000 | 0.9375 | 0.0156 | 0.0156 | — |
| 15755469790931547198 | 0.5000 | 0.5000 | 0.5000 | 0.9375 | 0.0156 | 0.0156 | — |
| 8709160710835925077 | 0.5000 | 0.5000 | 1.0000 | 0.9375 | 0.0156 | 0.0156 | — |
| 1663413756060003432 | 0.7500 | 0.5000 | 0.5000 | 0.9375 | 0.0156 | 0.0156 | — |

## Summary (paired normalized-gap analysis)

- mean ± var local-assembly: 0.5625 ± 0.011719
- mean ± var dense-local:    0.5000 ± 0.000000
- mean ± var gradient reference: 0.6500 ± 0.050000
- mean ± var eligibility reference: 0.9250 ± 0.000781
- mean normalized gap closed: 0.0000 (variance 0.000000)
- lower confidence bound (z=1.960, n=5): 0.0000
- mean |local − dense| (descriptive): 0.0625

## Pilot limitation

This run uses a quick schedule or fewer seeds than the power-analysis requirement. It validates the harness only and is not evidence for passing or failing G2.

## Positive / sanity control

Mean local-pipeline accuracy on a trivially separable spatial feature-presence task: **1.0000** (threshold 0.900).

## Activity sparsity

Mean local-assembly activity_sparsity: **0.0156** (valid band [0.0050, 0.0300]; nominal k/N=0.0156).

## Parameter / compute budgets

| condition | n_cells | n_params | wall_secs | peak_rss_bytes | work_per_accuracy | spikes | deliveries | cell_updates | plasticity |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| local-assembly | 68 | 1480 | 0.0033 | 8830976 | 118530.0000 | 213 | 1319 | 1493 | 56240 |
| dense-local | 68 | 4288 | 0.0026 | 9256960 | 391944.0000 | 258 | 7720 | 7898 | 180096 |
| gradient-reference | 66 | 4289 | 0.0233 | 9338880 | 6155520.0000 | 0 | 7680 | 491520 | 4117440 |
| eligibility-reference | 66 | 193 | 0.0028 | 9338880 | 782262.8571 | 0 | 7680 | 491520 | 185280 |

## Plots

- raster: Written
- weights: Written
- raster: Written
- weights: Written
- raster: Written
- weights: Written
- raster: Written
- weights: Written
- raster: Written
- weights: Written

## Structured log (GC7)

```
config_hash=c1-e0dfdbf4e3d2936b seed=11400784225994701844 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=118530.000000 note=wall_secs=0.0033_peak_rss=8830976_spikes=213_deliveries=1319_cells=1493_plasticity=56240
config_hash=c1-e0dfdbf4e3d2936b seed=11400784225994701844 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=391944.000000 note=wall_secs=0.0026_peak_rss=9256960_spikes=258_deliveries=7720_cells=7898_plasticity=180096
config_hash=c1-e0dfdbf4e3d2936b seed=11400784225994701844 condition=gradient-reference accuracy=0.750000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=6155520.000000 note=wall_secs=0.0233_peak_rss=9338880_spikes=0_deliveries=7680_cells=491520_plasticity=4117440
config_hash=c1-e0dfdbf4e3d2936b seed=11400784225994701844 condition=eligibility-reference accuracy=0.875000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=782262.857143 note=wall_secs=0.0028_peak_rss=9338880_spikes=0_deliveries=7680_cells=491520_plasticity=185280
config_hash=c1-e0dfdbf4e3d2936b seed=4354472946875824171 condition=local-assembly accuracy=0.562500 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=100106.666667 note=wall_secs=0.0026_peak_rss=73646080_spikes=216_deliveries=1285_cells=1457_plasticity=53352
config_hash=c1-e0dfdbf4e3d2936b seed=4354472946875824171 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=391944.000000 note=wall_secs=0.0036_peak_rss=73728000_spikes=258_deliveries=7720_cells=7898_plasticity=180096
config_hash=c1-e0dfdbf4e3d2936b seed=4354472946875824171 condition=gradient-reference accuracy=0.500000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9233280.000000 note=wall_secs=0.0239_peak_rss=73760768_spikes=0_deliveries=7680_cells=491520_plasticity=4117440
config_hash=c1-e0dfdbf4e3d2936b seed=4354472946875824171 condition=eligibility-reference accuracy=0.937500 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=730112.000000 note=wall_secs=0.0028_peak_rss=73760768_spikes=0_deliveries=7680_cells=491520_plasticity=185280
config_hash=c1-e0dfdbf4e3d2936b seed=15755469790931547198 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=127052.000000 note=wall_secs=0.0028_peak_rss=75743232_spikes=189_deliveries=1281_cells=1458_plasticity=60598
config_hash=c1-e0dfdbf4e3d2936b seed=15755469790931547198 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=391944.000000 note=wall_secs=0.0024_peak_rss=75759616_spikes=258_deliveries=7720_cells=7898_plasticity=180096
config_hash=c1-e0dfdbf4e3d2936b seed=15755469790931547198 condition=gradient-reference accuracy=0.500000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9233280.000000 note=wall_secs=0.0236_peak_rss=75759616_spikes=0_deliveries=7680_cells=491520_plasticity=4117440
config_hash=c1-e0dfdbf4e3d2936b seed=15755469790931547198 condition=eligibility-reference accuracy=0.937500 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=730112.000000 note=wall_secs=0.0027_peak_rss=75759616_spikes=0_deliveries=7680_cells=491520_plasticity=185280
config_hash=c1-e0dfdbf4e3d2936b seed=8709160710835925077 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=121272.000000 note=wall_secs=0.0029_peak_rss=77889536_spikes=216_deliveries=1282_cells=1457_plasticity=57681
config_hash=c1-e0dfdbf4e3d2936b seed=8709160710835925077 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=391944.000000 note=wall_secs=0.0026_peak_rss=77889536_spikes=258_deliveries=7720_cells=7898_plasticity=180096
config_hash=c1-e0dfdbf4e3d2936b seed=8709160710835925077 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=4616640.000000 note=wall_secs=0.0231_peak_rss=77889536_spikes=0_deliveries=7680_cells=491520_plasticity=4117440
config_hash=c1-e0dfdbf4e3d2936b seed=8709160710835925077 condition=eligibility-reference accuracy=0.937500 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=730112.000000 note=wall_secs=0.0029_peak_rss=77889536_spikes=0_deliveries=7680_cells=491520_plasticity=185280
config_hash=c1-e0dfdbf4e3d2936b seed=1663413756060003432 condition=local-assembly accuracy=0.750000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=71153.333333 note=wall_secs=0.0032_peak_rss=79364096_spikes=211_deliveries=1281_cells=1451_plasticity=50422
config_hash=c1-e0dfdbf4e3d2936b seed=1663413756060003432 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=391944.000000 note=wall_secs=0.0025_peak_rss=79364096_spikes=258_deliveries=7720_cells=7898_plasticity=180096
config_hash=c1-e0dfdbf4e3d2936b seed=1663413756060003432 condition=gradient-reference accuracy=0.500000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9233280.000000 note=wall_secs=0.0234_peak_rss=79364096_spikes=0_deliveries=7680_cells=491520_plasticity=4117440
config_hash=c1-e0dfdbf4e3d2936b seed=1663413756060003432 condition=eligibility-reference accuracy=0.937500 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=730112.000000 note=wall_secs=0.0030_peak_rss=79364096_spikes=0_deliveries=7680_cells=491520_plasticity=185280
```
