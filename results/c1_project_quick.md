# C1 / Gate G2 results note

**Config hash:** `c1-41458c2941a9d96e`

**Scientific protocol version:** `7`

**Assembly-Calculus `project` protocol:** `7` — hidden winners from `binn_areas::project` (charge k-WTA + Hebbian imprint) instead of inline membrane-score k-WTA; trial-isolation resets applied; does **not** reopen protocol-v2 kill-gate hash `c1-118207fbc3eaba53` (canonical version `2`).

**Verdict (Gate G2):** **INVALID_HARNESS**

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
Config { experiment: "c1-project", master_seed: 212686780497921, n_seeds: 5, sequence_len: 8, max_lag: 1, n_hidden: 64, k_wta: 1, p_sparse: 0.35, init_w: 0.15, eta: 0.35, lambda: 0.002, tau_e: 40.0, n_train: 24, n_test: 16, bptt_epochs: 40, bptt_lr: 0.05, g2_min_gap_closed: 0.5, g2_min_accuracy: 0.65, g2_confidence_z: 1.96, g2_min_positive_control: 0.9, g2_min_reference_gap: 0.15, activity_sparsity_min: 0.005, activity_sparsity_max: 0.03, scientific_n_seeds: 20, power_sigma_prior: 0.15, power_effect_size: 0.1, use_surrogate_lif_reference: true, surrogate_beta: 5.0, matched_budget_repeat: false, quick: true }
```

Power analysis: required scientific n_seeds ≥ 20 (preregistered σ=0.150, effect=0.100; formula n=⌈(1.96+0.8416)²σ²/δ²⌉).

## Per-seed accuracies

| seed | local-assembly | dense-local | gradient reference | eligibility reference | activity_sparsity (local) | activity_sparsity (dense) | dense_matched |
|---|---:|---:|---:|---:|---:|---:|---:|
| 11400784157275225108 | 0.5000 | 0.5000 | 0.7500 | 0.9375 | 0.0156 | 0.0156 | — |
| 4354472878156347435 | 0.5000 | 0.5000 | 0.9375 | 0.9375 | 0.0156 | 0.0156 | — |
| 15755469859651023934 | 0.5000 | 0.5000 | 1.0000 | 0.9375 | 0.0156 | 0.0156 | — |
| 8709160779555401813 | 0.5000 | 0.5000 | 1.0000 | 0.9375 | 0.0156 | 0.0156 | — |
| 1663413687340526696 | 0.5000 | 0.5000 | 0.9375 | 0.9375 | 0.0156 | 0.0156 | — |

## Summary (paired normalized-gap analysis)

- mean ± var local-assembly: 0.5000 ± 0.000000
- mean ± var dense-local:    0.5000 ± 0.000000
- mean ± var gradient reference: 0.9250 ± 0.010547
- mean ± var eligibility reference: 0.9375 ± 0.000000
- mean normalized gap closed: 0.0000 (variance 0.000000)
- lower confidence bound (z=1.960, n=5): 0.0000
- mean |local − dense| (descriptive): 0.0000

## Invalid harness

Positive control and/or activity sparsity failed the preregistered validity gates. No scientific PASS/FAIL or U-NEG claim is permitted from this run.

## Positive / sanity control

Mean local-pipeline accuracy on a trivially separable spatial feature-presence task: **0.8000** (threshold 0.900).

## Activity sparsity

Mean local-assembly activity_sparsity: **0.0156** (valid band [0.0050, 0.0300]; nominal k/N=0.0156).

## Parameter / compute budgets

| condition | n_cells | n_params | wall_secs | peak_rss_bytes | work_per_accuracy | spikes | deliveries | cell_updates | plasticity |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| local-assembly | 68 | 1485 | 0.0040 | 3031040 | 112970.0000 | 213 | 1320 | 1492 | 53460 |
| dense-local | 68 | 4288 | 0.0042 | 3358720 | 443422.0000 | 263 | 7720 | 7904 | 205824 |
| gradient-reference | 66 | 4289 | 0.0265 | 2588672 | 6155520.0000 | 0 | 7680 | 491520 | 4117440 |
| eligibility-reference | 66 | 193 | 0.0033 | 2490368 | 730112.0000 | 0 | 7680 | 491520 | 185280 |

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
config_hash=c1-41458c2941a9d96e seed=11400784157275225108 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=112970.000000 note=wall_secs=0.0040_peak_rss=3031040_spikes=213_deliveries=1320_cells=1492_plasticity=53460
config_hash=c1-41458c2941a9d96e seed=11400784157275225108 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=443422.000000 note=wall_secs=0.0042_peak_rss=3358720_spikes=263_deliveries=7720_cells=7904_plasticity=205824
config_hash=c1-41458c2941a9d96e seed=11400784157275225108 condition=gradient-reference accuracy=0.750000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=6155520.000000 note=wall_secs=0.0265_peak_rss=2588672_spikes=0_deliveries=7680_cells=491520_plasticity=4117440
config_hash=c1-41458c2941a9d96e seed=11400784157275225108 condition=eligibility-reference accuracy=0.937500 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=730112.000000 note=wall_secs=0.0033_peak_rss=2490368_spikes=0_deliveries=7680_cells=491520_plasticity=185280
config_hash=c1-41458c2941a9d96e seed=4354472878156347435 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=148560.000000 note=wall_secs=0.0027_peak_rss=3080192_spikes=224_deliveries=1320_cells=1504_plasticity=71232
config_hash=c1-41458c2941a9d96e seed=4354472878156347435 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=443422.000000 note=wall_secs=0.0035_peak_rss=3440640_spikes=263_deliveries=7720_cells=7904_plasticity=205824
config_hash=c1-41458c2941a9d96e seed=4354472878156347435 condition=gradient-reference accuracy=0.937500 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=4924416.000000 note=wall_secs=0.0246_peak_rss=2588672_spikes=0_deliveries=7680_cells=491520_plasticity=4117440
config_hash=c1-41458c2941a9d96e seed=4354472878156347435 condition=eligibility-reference accuracy=0.937500 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=730112.000000 note=wall_secs=0.0029_peak_rss=2506752_spikes=0_deliveries=7680_cells=491520_plasticity=185280
config_hash=c1-41458c2941a9d96e seed=15755469859651023934 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=147610.000000 note=wall_secs=0.0032_peak_rss=3162112_spikes=261_deliveries=1280_cells=1464_plasticity=70800
config_hash=c1-41458c2941a9d96e seed=15755469859651023934 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=443422.000000 note=wall_secs=0.0037_peak_rss=3391488_spikes=263_deliveries=7720_cells=7904_plasticity=205824
config_hash=c1-41458c2941a9d96e seed=15755469859651023934 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=4616640.000000 note=wall_secs=0.0250_peak_rss=2621440_spikes=0_deliveries=7680_cells=491520_plasticity=4117440
config_hash=c1-41458c2941a9d96e seed=15755469859651023934 condition=eligibility-reference accuracy=0.937500 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=730112.000000 note=wall_secs=0.0030_peak_rss=2506752_spikes=0_deliveries=7680_cells=491520_plasticity=185280
config_hash=c1-41458c2941a9d96e seed=8709160779555401813 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=148144.000000 note=wall_secs=0.0030_peak_rss=3080192_spikes=224_deliveries=1360_cells=1544_plasticity=70944
config_hash=c1-41458c2941a9d96e seed=8709160779555401813 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=443422.000000 note=wall_secs=0.0030_peak_rss=3309568_spikes=263_deliveries=7720_cells=7904_plasticity=205824
config_hash=c1-41458c2941a9d96e seed=8709160779555401813 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=4616640.000000 note=wall_secs=0.0243_peak_rss=2572288_spikes=0_deliveries=7680_cells=491520_plasticity=4117440
config_hash=c1-41458c2941a9d96e seed=8709160779555401813 condition=eligibility-reference accuracy=0.937500 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=730112.000000 note=wall_secs=0.0037_peak_rss=2506752_spikes=0_deliveries=7680_cells=491520_plasticity=185280
config_hash=c1-41458c2941a9d96e seed=1663413687340526696 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=147852.000000 note=wall_secs=0.0025_peak_rss=3014656_spikes=222_deliveries=1240_cells=1424_plasticity=71040
config_hash=c1-41458c2941a9d96e seed=1663413687340526696 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=443422.000000 note=wall_secs=0.0034_peak_rss=3489792_spikes=263_deliveries=7720_cells=7904_plasticity=205824
config_hash=c1-41458c2941a9d96e seed=1663413687340526696 condition=gradient-reference accuracy=0.937500 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=4924416.000000 note=wall_secs=0.0246_peak_rss=2605056_spikes=0_deliveries=7680_cells=491520_plasticity=4117440
config_hash=c1-41458c2941a9d96e seed=1663413687340526696 condition=eligibility-reference accuracy=0.937500 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=730112.000000 note=wall_secs=0.0039_peak_rss=2523136_spikes=0_deliveries=7680_cells=491520_plasticity=185280
```
