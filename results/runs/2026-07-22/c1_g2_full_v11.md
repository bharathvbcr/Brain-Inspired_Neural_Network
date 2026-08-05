# C1 / Gate G2 results note

**Config hash:** `c1-118207fbc3eaba53`

**Scientific protocol version:** `2`

**Verdict (Gate G2):** **FAIL**

PASS = lower confidence bound on normalized gradient gap closed > 0.500 and mean local accuracy >= 0.650.
FAIL = a full run missed at least one preregistered threshold.
PILOT = quick schedule or fewer seeds than the power-analysis requirement; not a scientific G2 decision.
INVALID_HARNESS = positive_control_mean < 0.900 or mean activity sparsity outside [0.0050, 0.0300]; prohibits PASS/FAIL and U-NEG language.

## Conditions

| Label | Meaning |
|---|---|
| `local-assembly` | Three-factor rule + sparse assembly wiring + k-WTA + dual readouts + two-sided ±1 reward |
| `dense-local` | Same three-factor rule + same k-winner budget on dense all-to-all connectivity, **no** assembly structure |
| `dense-matched` | Dense-local with nnz matched to local-assembly (parameter-matched; measured compute disclosed below) |
| `gradient-reference` | Same-architecture surrogate-LIF BPTT (primary); tanh RNN optional/secondary |
| `eligibility-reference` | E-prop-compatible eligibility local reference (rate-model approximation; feedforward-only) |

Plasticity uses hard ±1 reward by design (soft RPE deferred). Gap-closed is clamped to `[0, 1]` and seeds with `(reference − dense) < 0.150` contribute `closed = 0`.

## Config

```
Config { experiment: "c1", master_seed: 212618061021185, n_seeds: 20, sequence_len: 8, max_lag: 1, n_hidden: 128, k_wta: 2, p_sparse: 0.35, init_w: 0.15, eta: 0.35, lambda: 0.002, tau_e: 40.0, n_train: 80, n_test: 40, bptt_epochs: 80, bptt_lr: 0.05, g2_min_gap_closed: 0.5, g2_min_accuracy: 0.65, g2_confidence_z: 1.96, g2_min_positive_control: 0.9, g2_min_reference_gap: 0.15, activity_sparsity_min: 0.005, activity_sparsity_max: 0.03, scientific_n_seeds: 20, power_sigma_prior: 0.15, power_effect_size: 0.1, use_surrogate_lif_reference: true, surrogate_beta: 5.0, matched_budget_repeat: true, quick: false }
```

Power analysis: required scientific n_seeds ≥ 20 (preregistered σ=0.150, effect=0.100; formula n=⌈(1.96+0.8416)²σ²/δ²⌉).

## Per-seed accuracies

| seed | local-assembly | dense-local | gradient reference | eligibility reference | activity_sparsity (local) | activity_sparsity (dense) | dense_matched |
|---|---:|---:|---:|---:|---:|---:|---:|
| 11400784225994701844 | 0.4250 | 0.5000 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 4354472946875824171 | 0.4750 | 0.5000 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 15755469790931547198 | 0.5000 | 0.5000 | 0.7250 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 8709160710835925077 | 0.5000 | 0.5000 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 1663413756060003432 | 0.5000 | 0.5000 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 13063846550650677375 | 0.5000 | 0.5000 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 6018099320996848786 | 0.3500 | 0.5000 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 17418529916564267177 | 0.5000 | 0.5000 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 10372782686910438588 | 0.5000 | 0.5000 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 3326471682669467859 | 0.5000 | 0.5000 | 0.7250 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 14727610363725173990 | 0.6000 | 0.5000 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 7681300184117924093 | 0.5250 | 0.5000 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 635551854952467728 | 0.5000 | 0.5000 | 0.5000 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 12035985749054769447 | 0.5000 | 0.5000 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 4990235495743964474 | 0.5500 | 0.5000 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 16390669389846266193 | 0.4750 | 0.5000 | 0.5000 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 9344921060680809828 | 0.4250 | 0.5000 | 0.7250 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 2298610881073559931 | 0.5000 | 0.5000 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 13699608824640910734 | 0.5000 | 0.5000 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 6653297820399940005 | 0.5000 | 0.5000 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |

## Summary (paired normalized-gap analysis)

- mean ± var local-assembly: 0.4912 ± 0.002452
- mean ± var dense-local:    0.5000 ± 0.000000
- mean ± var gradient reference: 0.8938 ± 0.027163
- mean ± var eligibility reference: 1.0000 ± 0.000000
- mean normalized gap closed: 0.0189 (variance 0.002912)
- lower confidence bound (z=1.960, n=20): -0.0048
- mean |local − dense| (descriptive): 0.0262

## U-NEG

Negative result: local-assembly did **not** clear the preregistered normalized-gap confidence and absolute-accuracy gates. Program stops at G2; do not schedule P3+.

## Positive / sanity control

Mean local-pipeline accuracy on a trivially separable task: **0.9488** (threshold 0.900).

## Activity sparsity

Mean local-assembly activity_sparsity: **0.0156** (valid band [0.0050, 0.0300]; nominal k/N=0.0156).

## Parameter / compute budgets

| condition | n_cells | n_params | wall_secs | peak_rss_bytes | work_per_accuracy | spikes | deliveries | cell_updates | plasticity |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| local-assembly | 132 | 5777 | 0.0063 | 3407872 | 1886489.3589 | 816 | 13085 | 13739 | 774118 |
| dense-local | 132 | 16768 | 0.0126 | 4423680 | 4978492.0000 | 937 | 61680 | 62341 | 2364288 |
| gradient-reference | 130 | 16769 | 0.7136 | 2834432 | 113926400.0000 | 0 | 51200 | 6553600 | 107321600 |
| eligibility-reference | 130 | 385 | 0.0190 | 2457600 | 9068800.0000 | 0 | 51200 | 6553600 | 2464000 |
| dense-matched | 132 | 5777 | 0.0069 | 3735552 | 1796194.0000 | 937 | 40971 | 41632 | 814557 |

Matched-budget dense mean accuracy: **0.5000** (n=20; primary G2 gap still uses unmatched dense-local).

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
config_hash=c1-118207fbc3eaba53 seed=11400784225994701844 condition=local-assembly accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1886489.358850 note=wall_secs=0.0063_peak_rss=3407872_spikes=816_deliveries=13085_cells=13739_plasticity=774118
config_hash=c1-118207fbc3eaba53 seed=11400784225994701844 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=4978492.000000 note=wall_secs=0.0126_peak_rss=4423680_spikes=937_deliveries=61680_cells=62341_plasticity=2364288
config_hash=c1-118207fbc3eaba53 seed=11400784225994701844 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.7136_peak_rss=2834432_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-118207fbc3eaba53 seed=11400784225994701844 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0190_peak_rss=2457600_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-118207fbc3eaba53 seed=11400784225994701844 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1796194.000000 note=wall_secs=0.0069_peak_rss=3735552_spikes=937_deliveries=40971_cells=41632_plasticity=814557
config_hash=c1-118207fbc3eaba53 seed=4354472946875824171 condition=local-assembly accuracy=0.475000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1614658.967630 note=wall_secs=0.0055_peak_rss=3342336_spikes=797_deliveries=13159_cells=13807_plasticity=739200
config_hash=c1-118207fbc3eaba53 seed=4354472946875824171 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=4978492.000000 note=wall_secs=0.0125_peak_rss=4390912_spikes=937_deliveries=61680_cells=62341_plasticity=2364288
config_hash=c1-118207fbc3eaba53 seed=4354472946875824171 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.7074_peak_rss=2818048_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-118207fbc3eaba53 seed=4354472946875824171 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0163_peak_rss=2473984_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-118207fbc3eaba53 seed=4354472946875824171 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1796954.000000 note=wall_secs=0.0064_peak_rss=3686400_spikes=937_deliveries=41302_cells=41963_plasticity=814275
config_hash=c1-118207fbc3eaba53 seed=15755469790931547198 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1429328.000000 note=wall_secs=0.0058_peak_rss=3391488_spikes=759_deliveries=13080_cells=13719_plasticity=687106
config_hash=c1-118207fbc3eaba53 seed=15755469790931547198 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=4978492.000000 note=wall_secs=0.0128_peak_rss=4489216_spikes=937_deliveries=61680_cells=62341_plasticity=2364288
config_hash=c1-118207fbc3eaba53 seed=15755469790931547198 condition=gradient-reference accuracy=0.725000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=157139856.901371 note=wall_secs=0.7057_peak_rss=2818048_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-118207fbc3eaba53 seed=15755469790931547198 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0166_peak_rss=2473984_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-118207fbc3eaba53 seed=15755469790931547198 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1793108.000000 note=wall_secs=0.0071_peak_rss=3719168_spikes=937_deliveries=40411_cells=41072_plasticity=814134
config_hash=c1-118207fbc3eaba53 seed=8709160710835925077 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1457050.000000 note=wall_secs=0.0060_peak_rss=3309568_spikes=763_deliveries=13205_cells=13846_plasticity=700711
config_hash=c1-118207fbc3eaba53 seed=8709160710835925077 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=4978492.000000 note=wall_secs=0.0122_peak_rss=4407296_spikes=937_deliveries=61680_cells=62341_plasticity=2364288
config_hash=c1-118207fbc3eaba53 seed=8709160710835925077 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6999_peak_rss=2818048_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-118207fbc3eaba53 seed=8709160710835925077 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0171_peak_rss=2473984_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-118207fbc3eaba53 seed=8709160710835925077 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1802926.000000 note=wall_secs=0.0065_peak_rss=3735552_spikes=937_deliveries=41667_cells=42328_plasticity=816531
config_hash=c1-118207fbc3eaba53 seed=1663413756060003432 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1503506.000000 note=wall_secs=0.0060_peak_rss=3391488_spikes=674_deliveries=13217_cells=13862_plasticity=724000
config_hash=c1-118207fbc3eaba53 seed=1663413756060003432 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=4978492.000000 note=wall_secs=0.0118_peak_rss=4374528_spikes=937_deliveries=61680_cells=62341_plasticity=2364288
config_hash=c1-118207fbc3eaba53 seed=1663413756060003432 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.7018_peak_rss=2818048_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-118207fbc3eaba53 seed=1663413756060003432 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0163_peak_rss=2457600_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-118207fbc3eaba53 seed=1663413756060003432 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1800368.000000 note=wall_secs=0.0066_peak_rss=3670016_spikes=937_deliveries=40957_cells=41618_plasticity=816672
config_hash=c1-118207fbc3eaba53 seed=13063846550650677375 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1464450.000000 note=wall_secs=0.0060_peak_rss=3325952_spikes=757_deliveries=13199_cells=13841_plasticity=704428
config_hash=c1-118207fbc3eaba53 seed=13063846550650677375 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=4978492.000000 note=wall_secs=0.0125_peak_rss=4554752_spikes=937_deliveries=61680_cells=62341_plasticity=2364288
config_hash=c1-118207fbc3eaba53 seed=13063846550650677375 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.7047_peak_rss=2818048_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-118207fbc3eaba53 seed=13063846550650677375 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0188_peak_rss=2473984_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-118207fbc3eaba53 seed=13063846550650677375 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1796716.000000 note=wall_secs=0.0066_peak_rss=3702784_spikes=937_deliveries=41313_cells=41974_plasticity=814134
config_hash=c1-118207fbc3eaba53 seed=6018099320996848786 condition=local-assembly accuracy=0.350000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2126131.464779 note=wall_secs=0.0060_peak_rss=3391488_spikes=806_deliveries=13422_cells=14066_plasticity=715852
config_hash=c1-118207fbc3eaba53 seed=6018099320996848786 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=4978492.000000 note=wall_secs=0.0117_peak_rss=4440064_spikes=937_deliveries=61680_cells=62341_plasticity=2364288
config_hash=c1-118207fbc3eaba53 seed=6018099320996848786 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.7063_peak_rss=2818048_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-118207fbc3eaba53 seed=6018099320996848786 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0190_peak_rss=2473984_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-118207fbc3eaba53 seed=6018099320996848786 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1792974.000000 note=wall_secs=0.0067_peak_rss=3719168_spikes=937_deliveries=40448_cells=41109_plasticity=813993
config_hash=c1-118207fbc3eaba53 seed=17418529916564267177 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1477836.000000 note=wall_secs=0.0058_peak_rss=3342336_spikes=679_deliveries=13328_cells=13971_plasticity=710940
config_hash=c1-118207fbc3eaba53 seed=17418529916564267177 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=4978492.000000 note=wall_secs=0.0127_peak_rss=4390912_spikes=937_deliveries=61680_cells=62341_plasticity=2364288
config_hash=c1-118207fbc3eaba53 seed=17418529916564267177 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.7068_peak_rss=2818048_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-118207fbc3eaba53 seed=17418529916564267177 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0162_peak_rss=2473984_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-118207fbc3eaba53 seed=17418529916564267177 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1795876.000000 note=wall_secs=0.0069_peak_rss=3719168_spikes=937_deliveries=40680_cells=41341_plasticity=814980
config_hash=c1-118207fbc3eaba53 seed=10372782686910438588 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1489616.000000 note=wall_secs=0.0057_peak_rss=3325952_spikes=680_deliveries=13320_cells=13964_plasticity=716844
config_hash=c1-118207fbc3eaba53 seed=10372782686910438588 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=4978492.000000 note=wall_secs=0.0120_peak_rss=4358144_spikes=937_deliveries=61680_cells=62341_plasticity=2364288
config_hash=c1-118207fbc3eaba53 seed=10372782686910438588 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.7086_peak_rss=2818048_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-118207fbc3eaba53 seed=10372782686910438588 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0171_peak_rss=2506752_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-118207fbc3eaba53 seed=10372782686910438588 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1797626.000000 note=wall_secs=0.0064_peak_rss=3719168_spikes=937_deliveries=41047_cells=41708_plasticity=815121
config_hash=c1-118207fbc3eaba53 seed=3326471682669467859 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1488972.000000 note=wall_secs=0.0060_peak_rss=3309568_spikes=690_deliveries=13340_cells=13984_plasticity=716472
config_hash=c1-118207fbc3eaba53 seed=3326471682669467859 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=4978492.000000 note=wall_secs=0.0124_peak_rss=4374528_spikes=937_deliveries=61680_cells=62341_plasticity=2364288
config_hash=c1-118207fbc3eaba53 seed=3326471682669467859 condition=gradient-reference accuracy=0.725000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=157139856.901371 note=wall_secs=0.7047_peak_rss=2834432_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-118207fbc3eaba53 seed=3326471682669467859 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0195_peak_rss=2473984_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-118207fbc3eaba53 seed=3326471682669467859 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1796612.000000 note=wall_secs=0.0068_peak_rss=3768320_spikes=937_deliveries=41005_cells=41666_plasticity=814698
config_hash=c1-118207fbc3eaba53 seed=14727610363725173990 condition=local-assembly accuracy=0.600000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1260918.283229 note=wall_secs=0.0059_peak_rss=3407872_spikes=825_deliveries=13337_cells=13983_plasticity=728406
config_hash=c1-118207fbc3eaba53 seed=14727610363725173990 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=4978492.000000 note=wall_secs=0.0127_peak_rss=4472832_spikes=937_deliveries=61680_cells=62341_plasticity=2364288
config_hash=c1-118207fbc3eaba53 seed=14727610363725173990 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.7070_peak_rss=2818048_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-118207fbc3eaba53 seed=14727610363725173990 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0166_peak_rss=2473984_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-118207fbc3eaba53 seed=14727610363725173990 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1795850.000000 note=wall_secs=0.0067_peak_rss=3686400_spikes=937_deliveries=40603_cells=41264_plasticity=815121
config_hash=c1-118207fbc3eaba53 seed=7681300184117924093 condition=local-assembly accuracy=0.525000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1391683.872724 note=wall_secs=0.0061_peak_rss=3325952_spikes=806_deliveries=13172_cells=13814_plasticity=702842
config_hash=c1-118207fbc3eaba53 seed=7681300184117924093 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=4978492.000000 note=wall_secs=0.0118_peak_rss=4177920_spikes=937_deliveries=61680_cells=62341_plasticity=2364288
config_hash=c1-118207fbc3eaba53 seed=7681300184117924093 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.7108_peak_rss=2818048_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-118207fbc3eaba53 seed=7681300184117924093 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0190_peak_rss=2473984_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-118207fbc3eaba53 seed=7681300184117924093 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1791714.000000 note=wall_secs=0.0067_peak_rss=3702784_spikes=937_deliveries=40979_cells=41640_plasticity=812301
config_hash=c1-118207fbc3eaba53 seed=635551854952467728 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1498668.000000 note=wall_secs=0.0056_peak_rss=3375104_spikes=660_deliveries=13327_cells=13972_plasticity=721375
config_hash=c1-118207fbc3eaba53 seed=635551854952467728 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=4978492.000000 note=wall_secs=0.0117_peak_rss=4636672_spikes=937_deliveries=61680_cells=62341_plasticity=2364288
config_hash=c1-118207fbc3eaba53 seed=635551854952467728 condition=gradient-reference accuracy=0.500000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=227852800.000000 note=wall_secs=0.7222_peak_rss=2818048_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-118207fbc3eaba53 seed=635551854952467728 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0171_peak_rss=2457600_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-118207fbc3eaba53 seed=635551854952467728 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1794998.000000 note=wall_secs=0.0071_peak_rss=3686400_spikes=937_deliveries=41095_cells=41756_plasticity=813711
config_hash=c1-118207fbc3eaba53 seed=12035985749054769447 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1447154.000000 note=wall_secs=0.0054_peak_rss=3391488_spikes=763_deliveries=13327_cells=13967_plasticity=695520
config_hash=c1-118207fbc3eaba53 seed=12035985749054769447 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=4978492.000000 note=wall_secs=0.0118_peak_rss=4423680_spikes=937_deliveries=61680_cells=62341_plasticity=2364288
config_hash=c1-118207fbc3eaba53 seed=12035985749054769447 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.7043_peak_rss=2850816_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-118207fbc3eaba53 seed=12035985749054769447 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0167_peak_rss=2473984_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-118207fbc3eaba53 seed=12035985749054769447 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1802668.000000 note=wall_secs=0.0066_peak_rss=3686400_spikes=937_deliveries=41250_cells=41911_plasticity=817236
config_hash=c1-118207fbc3eaba53 seed=4990235495743964474 condition=local-assembly accuracy=0.550000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1376645.424707 note=wall_secs=0.0059_peak_rss=3293184_spikes=833_deliveries=13446_cells=14092_plasticity=728784
config_hash=c1-118207fbc3eaba53 seed=4990235495743964474 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=4978492.000000 note=wall_secs=0.0120_peak_rss=4456448_spikes=937_deliveries=61680_cells=62341_plasticity=2364288
config_hash=c1-118207fbc3eaba53 seed=4990235495743964474 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.7104_peak_rss=2850816_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-118207fbc3eaba53 seed=4990235495743964474 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0164_peak_rss=2473984_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-118207fbc3eaba53 seed=4990235495743964474 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1799224.000000 note=wall_secs=0.0070_peak_rss=3686400_spikes=937_deliveries=41235_cells=41896_plasticity=815544
config_hash=c1-118207fbc3eaba53 seed=16390669389846266193 condition=local-assembly accuracy=0.475000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1617844.230828 note=wall_secs=0.0063_peak_rss=3407872_spikes=810_deliveries=13333_cells=13981_plasticity=740352
config_hash=c1-118207fbc3eaba53 seed=16390669389846266193 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=4978492.000000 note=wall_secs=0.0119_peak_rss=4456448_spikes=937_deliveries=61680_cells=62341_plasticity=2364288
config_hash=c1-118207fbc3eaba53 seed=16390669389846266193 condition=gradient-reference accuracy=0.500000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=227852800.000000 note=wall_secs=0.7067_peak_rss=2818048_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-118207fbc3eaba53 seed=16390669389846266193 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0172_peak_rss=2457600_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-118207fbc3eaba53 seed=16390669389846266193 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1797200.000000 note=wall_secs=0.0064_peak_rss=3719168_spikes=937_deliveries=40729_cells=41390_plasticity=815544
config_hash=c1-118207fbc3eaba53 seed=9344921060680809828 condition=local-assembly accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1750705.833247 note=wall_secs=0.0055_peak_rss=3391488_spikes=806_deliveries=13312_cells=13956_plasticity=715976
config_hash=c1-118207fbc3eaba53 seed=9344921060680809828 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=4978492.000000 note=wall_secs=0.0117_peak_rss=4718592_spikes=937_deliveries=61680_cells=62341_plasticity=2364288
config_hash=c1-118207fbc3eaba53 seed=9344921060680809828 condition=gradient-reference accuracy=0.725000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=157139856.901371 note=wall_secs=0.7047_peak_rss=2818048_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-118207fbc3eaba53 seed=9344921060680809828 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0166_peak_rss=2441216_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-118207fbc3eaba53 seed=9344921060680809828 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1794560.000000 note=wall_secs=0.0066_peak_rss=3719168_spikes=937_deliveries=40774_cells=41435_plasticity=814134
config_hash=c1-118207fbc3eaba53 seed=2298610881073559931 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1453478.000000 note=wall_secs=0.0053_peak_rss=3293184_spikes=772_deliveries=13094_cells=13735_plasticity=699138
config_hash=c1-118207fbc3eaba53 seed=2298610881073559931 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=4978492.000000 note=wall_secs=0.0117_peak_rss=4096000_spikes=937_deliveries=61680_cells=62341_plasticity=2364288
config_hash=c1-118207fbc3eaba53 seed=2298610881073559931 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.7012_peak_rss=2818048_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-118207fbc3eaba53 seed=2298610881073559931 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0164_peak_rss=2457600_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-118207fbc3eaba53 seed=2298610881073559931 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1797764.000000 note=wall_secs=0.0068_peak_rss=3719168_spikes=937_deliveries=41293_cells=41954_plasticity=814698
config_hash=c1-118207fbc3eaba53 seed=13699608824640910734 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1488166.000000 note=wall_secs=0.0066_peak_rss=3325952_spikes=711_deliveries=13004_cells=13648_plasticity=716720
config_hash=c1-118207fbc3eaba53 seed=13699608824640910734 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=4978492.000000 note=wall_secs=0.0124_peak_rss=4456448_spikes=937_deliveries=61680_cells=62341_plasticity=2364288
config_hash=c1-118207fbc3eaba53 seed=13699608824640910734 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.7036_peak_rss=2818048_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-118207fbc3eaba53 seed=13699608824640910734 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0164_peak_rss=2457600_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-118207fbc3eaba53 seed=13699608824640910734 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1796900.000000 note=wall_secs=0.0070_peak_rss=3751936_spikes=937_deliveries=40936_cells=41597_plasticity=814980
config_hash=c1-118207fbc3eaba53 seed=6653297820399940005 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1443046.000000 note=wall_secs=0.0055_peak_rss=3358720_spikes=761_deliveries=13201_cells=13841_plasticity=693720
config_hash=c1-118207fbc3eaba53 seed=6653297820399940005 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=4978492.000000 note=wall_secs=0.0122_peak_rss=4374528_spikes=937_deliveries=61680_cells=62341_plasticity=2364288
config_hash=c1-118207fbc3eaba53 seed=6653297820399940005 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.7034_peak_rss=2818048_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-118207fbc3eaba53 seed=6653297820399940005 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0176_peak_rss=2473984_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-118207fbc3eaba53 seed=6653297820399940005 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1798278.000000 note=wall_secs=0.0066_peak_rss=3768320_spikes=937_deliveries=41210_cells=41871_plasticity=815121
```
