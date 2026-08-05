# C1 / Gate G2 results note

**Config hash:** `c1-1b68226b364a8973`

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
| `dense-matched` | Dense-local with nnz matched to local-assembly (compute-matched disclosure) |
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
| 11400784225994701844 | 0.5000 | 0.5000 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.7250 |
| 4354472946875824171 | 0.5000 | 0.5000 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.7250 |
| 15755469790931547198 | 0.5000 | 0.5000 | 0.7250 | 1.0000 | 0.0156 | 0.0156 | 0.7250 |
| 8709160710835925077 | 0.5000 | 0.5000 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.7250 |
| 1663413756060003432 | 0.5250 | 0.5000 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.7250 |
| 13063846550650677375 | 0.5000 | 0.5000 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.7250 |
| 6018099320996848786 | 0.5750 | 0.5000 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.7250 |
| 17418529916564267177 | 0.5000 | 0.5000 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.7250 |
| 10372782686910438588 | 0.5000 | 0.5000 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.7250 |
| 3326471682669467859 | 0.5500 | 0.5000 | 0.7250 | 1.0000 | 0.0156 | 0.0156 | 0.7250 |
| 14727610363725173990 | 0.5000 | 0.5000 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.7250 |
| 7681300184117924093 | 0.5000 | 0.5000 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.7250 |
| 635551854952467728 | 0.5000 | 0.5000 | 0.5000 | 1.0000 | 0.0156 | 0.0156 | 0.7250 |
| 12035985749054769447 | 0.5000 | 0.5000 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.7250 |
| 4990235495743964474 | 0.5000 | 0.5000 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.7250 |
| 16390669389846266193 | 0.6750 | 0.5000 | 0.5000 | 1.0000 | 0.0156 | 0.0156 | 0.7250 |
| 9344921060680809828 | 0.4500 | 0.5000 | 0.7250 | 1.0000 | 0.0156 | 0.0156 | 0.7250 |
| 2298610881073559931 | 1.0000 | 0.5000 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.7250 |
| 13699608824640910734 | 0.5000 | 0.5000 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.7250 |
| 6653297820399940005 | 0.5000 | 0.5000 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.7250 |

## Summary (paired normalized-gap analysis)

- mean ± var local-assembly: 0.5388 ± 0.013781
- mean ± var dense-local:    0.5000 ± 0.000000
- mean ± var gradient reference: 0.8938 ± 0.027163
- mean ± var eligibility reference: 1.0000 ± 0.000000
- mean normalized gap closed: 0.0714 (variance 0.051213)
- lower confidence bound (z=1.960, n=20): -0.0278
- mean |local − dense| (descriptive): 0.0437

## U-NEG

Negative result: local-assembly did **not** clear the preregistered normalized-gap confidence and absolute-accuracy gates. Program stops at G2; do not schedule P3+.

## Positive / sanity control

Mean local-pipeline accuracy on a trivially separable task: **0.9475** (threshold 0.900).

## Activity sparsity

Mean local-assembly activity_sparsity: **0.0156** (valid band [0.0050, 0.0300]; nominal k/N=0.0156).

## Parameter / compute budgets

| condition | n_cells | n_params | wall_secs | peak_rss_bytes | work_per_accuracy | spikes | deliveries | cell_updates | plasticity |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| local-assembly | 132 | 5777 | 0.0057 | 3375104 | 1522130.0000 | 681 | 13029 | 13676 | 733679 |
| dense-local | 132 | 17024 | 0.0126 | 4128768 | 5266164.0000 | 901 | 115568 | 116229 | 2400384 |
| gradient-reference | 130 | 16769 | 0.6960 | 2834432 | 113926400.0000 | 0 | 51200 | 6553600 | 107321600 |
| eligibility-reference | 130 | 385 | 0.0168 | 2457600 | 9068800.0000 | 0 | 51200 | 6553600 | 2464000 |
| dense-matched | 132 | 5777 | 0.0060 | 3719168 | 1033442.7246 | 640 | 27363 | 28003 | 693240 |

Matched-budget dense mean accuracy: **0.7250** (n=20; primary G2 gap still uses unmatched dense-local).

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
config_hash=c1-1b68226b364a8973 seed=11400784225994701844 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1522130.000000 note=wall_secs=0.0057_peak_rss=3375104_spikes=681_deliveries=13029_cells=13676_plasticity=733679
config_hash=c1-1b68226b364a8973 seed=11400784225994701844 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5266164.000000 note=wall_secs=0.0126_peak_rss=4128768_spikes=901_deliveries=115568_cells=116229_plasticity=2400384
config_hash=c1-1b68226b364a8973 seed=11400784225994701844 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6960_peak_rss=2834432_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-1b68226b364a8973 seed=11400784225994701844 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0168_peak_rss=2457600_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-1b68226b364a8973 seed=11400784225994701844 condition=dense-matched accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1033442.724636 note=wall_secs=0.0060_peak_rss=3719168_spikes=640_deliveries=27363_cells=28003_plasticity=693240
config_hash=c1-1b68226b364a8973 seed=4354472946875824171 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1499204.000000 note=wall_secs=0.0056_peak_rss=3375104_spikes=706_deliveries=13188_cells=13833_plasticity=721875
config_hash=c1-1b68226b364a8973 seed=4354472946875824171 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5266164.000000 note=wall_secs=0.0132_peak_rss=4407296_spikes=901_deliveries=115568_cells=116229_plasticity=2400384
config_hash=c1-1b68226b364a8973 seed=4354472946875824171 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6984_peak_rss=2850816_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-1b68226b364a8973 seed=4354472946875824171 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0187_peak_rss=2473984_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-1b68226b364a8973 seed=4354472946875824171 condition=dense-matched accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1033111.690164 note=wall_secs=0.0060_peak_rss=3735552_spikes=640_deliveries=27363_cells=28003_plasticity=693000
config_hash=c1-1b68226b364a8973 seed=15755469790931547198 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1429090.000000 note=wall_secs=0.0051_peak_rss=3506176_spikes=640_deliveries=13080_cells=13719_plasticity=687106
config_hash=c1-1b68226b364a8973 seed=15755469790931547198 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5266164.000000 note=wall_secs=0.0123_peak_rss=4096000_spikes=901_deliveries=115568_cells=116229_plasticity=2400384
config_hash=c1-1b68226b364a8973 seed=15755469790931547198 condition=gradient-reference accuracy=0.725000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=157139856.901371 note=wall_secs=0.6898_peak_rss=2818048_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-1b68226b364a8973 seed=15755469790931547198 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0194_peak_rss=2473984_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-1b68226b364a8973 seed=15755469790931547198 condition=dense-matched accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1032946.172928 note=wall_secs=0.0055_peak_rss=3735552_spikes=640_deliveries=27363_cells=28003_plasticity=692880
config_hash=c1-1b68226b364a8973 seed=8709160710835925077 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1456814.000000 note=wall_secs=0.0053_peak_rss=3375104_spikes=645_deliveries=13205_cells=13846_plasticity=700711
config_hash=c1-1b68226b364a8973 seed=8709160710835925077 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5266164.000000 note=wall_secs=0.0129_peak_rss=4308992_spikes=901_deliveries=115568_cells=116229_plasticity=2400384
config_hash=c1-1b68226b364a8973 seed=8709160710835925077 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.7100_peak_rss=2834432_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-1b68226b364a8973 seed=8709160710835925077 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0173_peak_rss=2457600_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-1b68226b364a8973 seed=8709160710835925077 condition=dense-matched accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1035804.103868 note=wall_secs=0.0066_peak_rss=3981312_spikes=640_deliveries=27379_cells=28019_plasticity=694920
config_hash=c1-1b68226b364a8973 seed=1663413756060003432 condition=local-assembly accuracy=0.525000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1432502.922197 note=wall_secs=0.0060_peak_rss=3489792_spikes=725_deliveries=13347_cells=13992_plasticity=724000
config_hash=c1-1b68226b364a8973 seed=1663413756060003432 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5266164.000000 note=wall_secs=0.0133_peak_rss=4128768_spikes=901_deliveries=115568_cells=116229_plasticity=2400384
config_hash=c1-1b68226b364a8973 seed=1663413756060003432 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6961_peak_rss=2834432_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-1b68226b364a8973 seed=1663413756060003432 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0163_peak_rss=2457600_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-1b68226b364a8973 seed=1663413756060003432 condition=dense-matched accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1035969.621104 note=wall_secs=0.0058_peak_rss=3735552_spikes=640_deliveries=27379_cells=28019_plasticity=695040
config_hash=c1-1b68226b364a8973 seed=13063846550650677375 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1464228.000000 note=wall_secs=0.0068_peak_rss=3391488_spikes=646_deliveries=13199_cells=13841_plasticity=704428
config_hash=c1-1b68226b364a8973 seed=13063846550650677375 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5266164.000000 note=wall_secs=0.0133_peak_rss=4128768_spikes=901_deliveries=115568_cells=116229_plasticity=2400384
config_hash=c1-1b68226b364a8973 seed=13063846550650677375 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6988_peak_rss=2834432_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-1b68226b364a8973 seed=13063846550650677375 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0192_peak_rss=2457600_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-1b68226b364a8973 seed=13063846550650677375 condition=dense-matched accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1032946.172928 note=wall_secs=0.0062_peak_rss=3751936_spikes=640_deliveries=27363_cells=28003_plasticity=692880
config_hash=c1-1b68226b364a8973 seed=6018099320996848786 condition=local-assembly accuracy=0.575000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1344224.375695 note=wall_secs=0.0058_peak_rss=3457024_spikes=703_deliveries=13430_cells=14079_plasticity=744717
config_hash=c1-1b68226b364a8973 seed=6018099320996848786 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5266164.000000 note=wall_secs=0.0128_peak_rss=4145152_spikes=901_deliveries=115568_cells=116229_plasticity=2400384
config_hash=c1-1b68226b364a8973 seed=6018099320996848786 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6906_peak_rss=2818048_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-1b68226b364a8973 seed=6018099320996848786 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0162_peak_rss=2490368_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-1b68226b364a8973 seed=6018099320996848786 condition=dense-matched accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1032780.655692 note=wall_secs=0.0057_peak_rss=3735552_spikes=640_deliveries=27363_cells=28003_plasticity=692760
config_hash=c1-1b68226b364a8973 seed=17418529916564267177 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1477770.000000 note=wall_secs=0.0054_peak_rss=3375104_spikes=646_deliveries=13328_cells=13971_plasticity=710940
config_hash=c1-1b68226b364a8973 seed=17418529916564267177 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5266164.000000 note=wall_secs=0.0133_peak_rss=4407296_spikes=901_deliveries=115568_cells=116229_plasticity=2400384
config_hash=c1-1b68226b364a8973 seed=17418529916564267177 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6843_peak_rss=2834432_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-1b68226b364a8973 seed=17418529916564267177 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0182_peak_rss=2457600_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-1b68226b364a8973 seed=17418529916564267177 condition=dense-matched accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1033983.414273 note=wall_secs=0.0061_peak_rss=3702784_spikes=640_deliveries=27379_cells=28019_plasticity=693600
config_hash=c1-1b68226b364a8973 seed=10372782686910438588 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1524258.000000 note=wall_secs=0.0052_peak_rss=3375104_spikes=655_deliveries=13320_cells=13967_plasticity=734187
config_hash=c1-1b68226b364a8973 seed=10372782686910438588 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5266164.000000 note=wall_secs=0.0126_peak_rss=4128768_spikes=901_deliveries=115568_cells=116229_plasticity=2400384
config_hash=c1-1b68226b364a8973 seed=10372782686910438588 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6849_peak_rss=2834432_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-1b68226b364a8973 seed=10372782686910438588 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0164_peak_rss=2457600_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-1b68226b364a8973 seed=10372782686910438588 condition=dense-matched accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1034148.931509 note=wall_secs=0.0056_peak_rss=3702784_spikes=640_deliveries=27379_cells=28019_plasticity=693720
config_hash=c1-1b68226b364a8973 seed=3326471682669467859 condition=local-assembly accuracy=0.550000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1395983.606107 note=wall_secs=0.0060_peak_rss=3440640_spikes=707_deliveries=13426_cells=14074_plasticity=739584
config_hash=c1-1b68226b364a8973 seed=3326471682669467859 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5266164.000000 note=wall_secs=0.0132_peak_rss=4440064_spikes=901_deliveries=115568_cells=116229_plasticity=2400384
config_hash=c1-1b68226b364a8973 seed=3326471682669467859 condition=gradient-reference accuracy=0.725000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=157139856.901371 note=wall_secs=0.7017_peak_rss=2834432_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-1b68226b364a8973 seed=3326471682669467859 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0187_peak_rss=2457600_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-1b68226b364a8973 seed=3326471682669467859 condition=dense-matched accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1033652.379801 note=wall_secs=0.0060_peak_rss=3735552_spikes=640_deliveries=27379_cells=28019_plasticity=693360
config_hash=c1-1b68226b364a8973 seed=14727610363725173990 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1466158.000000 note=wall_secs=0.0052_peak_rss=3391488_spikes=697_deliveries=13229_cells=13871_plasticity=705282
config_hash=c1-1b68226b364a8973 seed=14727610363725173990 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5266164.000000 note=wall_secs=0.0123_peak_rss=4112384_spikes=901_deliveries=115568_cells=116229_plasticity=2400384
config_hash=c1-1b68226b364a8973 seed=14727610363725173990 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6910_peak_rss=2834432_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-1b68226b364a8973 seed=14727610363725173990 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0160_peak_rss=2457600_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-1b68226b364a8973 seed=14727610363725173990 condition=dense-matched accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1034148.931509 note=wall_secs=0.0058_peak_rss=3702784_spikes=640_deliveries=27379_cells=28019_plasticity=693720
config_hash=c1-1b68226b364a8973 seed=7681300184117924093 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1507116.000000 note=wall_secs=0.0054_peak_rss=3424256_spikes=686_deliveries=13170_cells=13816_plasticity=725886
config_hash=c1-1b68226b364a8973 seed=7681300184117924093 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5266164.000000 note=wall_secs=0.0125_peak_rss=4145152_spikes=901_deliveries=115568_cells=116229_plasticity=2400384
config_hash=c1-1b68226b364a8973 seed=7681300184117924093 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6937_peak_rss=2834432_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-1b68226b364a8973 seed=7681300184117924093 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0165_peak_rss=2457600_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-1b68226b364a8973 seed=7681300184117924093 condition=dense-matched accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1030794.448861 note=wall_secs=0.0054_peak_rss=3702784_spikes=640_deliveries=27363_cells=28003_plasticity=691320
config_hash=c1-1b68226b364a8973 seed=635551854952467728 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1498642.000000 note=wall_secs=0.0060_peak_rss=3391488_spikes=647_deliveries=13327_cells=13972_plasticity=721375
config_hash=c1-1b68226b364a8973 seed=635551854952467728 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5266164.000000 note=wall_secs=0.0137_peak_rss=4423680_spikes=901_deliveries=115568_cells=116229_plasticity=2400384
config_hash=c1-1b68226b364a8973 seed=635551854952467728 condition=gradient-reference accuracy=0.500000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=227852800.000000 note=wall_secs=0.6877_peak_rss=2834432_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-1b68226b364a8973 seed=635551854952467728 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0186_peak_rss=2457600_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-1b68226b364a8973 seed=635551854952467728 condition=dense-matched accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1032449.621220 note=wall_secs=0.0060_peak_rss=3702784_spikes=640_deliveries=27363_cells=28003_plasticity=692520
config_hash=c1-1b68226b364a8973 seed=12035985749054769447 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1446918.000000 note=wall_secs=0.0049_peak_rss=3457024_spikes=645_deliveries=13327_cells=13967_plasticity=695520
config_hash=c1-1b68226b364a8973 seed=12035985749054769447 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5266164.000000 note=wall_secs=0.0122_peak_rss=4472832_spikes=901_deliveries=115568_cells=116229_plasticity=2400384
config_hash=c1-1b68226b364a8973 seed=12035985749054769447 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6856_peak_rss=2834432_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-1b68226b364a8973 seed=12035985749054769447 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0196_peak_rss=2457600_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-1b68226b364a8973 seed=12035985749054769447 condition=dense-matched accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1036609.621083 note=wall_secs=0.0058_peak_rss=3735552_spikes=640_deliveries=27371_cells=28011_plasticity=695520
config_hash=c1-1b68226b364a8973 seed=4990235495743964474 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1478986.000000 note=wall_secs=0.0058_peak_rss=3407872_spikes=666_deliveries=13376_cells=14019_plasticity=711432
config_hash=c1-1b68226b364a8973 seed=4990235495743964474 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5266164.000000 note=wall_secs=0.0123_peak_rss=4194304_spikes=901_deliveries=115568_cells=116229_plasticity=2400384
config_hash=c1-1b68226b364a8973 seed=4990235495743964474 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6784_peak_rss=2834432_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-1b68226b364a8973 seed=4990235495743964474 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0166_peak_rss=2457600_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-1b68226b364a8973 seed=4990235495743964474 condition=dense-matched accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1034645.483217 note=wall_secs=0.0060_peak_rss=3670016_spikes=640_deliveries=27379_cells=28019_plasticity=694080
config_hash=c1-1b68226b364a8973 seed=16390669389846266193 condition=local-assembly accuracy=0.675000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1138361.461377 note=wall_secs=0.0060_peak_rss=3375104_spikes=722_deliveries=13336_cells=13984_plasticity=740352
config_hash=c1-1b68226b364a8973 seed=16390669389846266193 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5266164.000000 note=wall_secs=0.0123_peak_rss=4194304_spikes=901_deliveries=115568_cells=116229_plasticity=2400384
config_hash=c1-1b68226b364a8973 seed=16390669389846266193 condition=gradient-reference accuracy=0.500000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=227852800.000000 note=wall_secs=0.6906_peak_rss=2834432_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-1b68226b364a8973 seed=16390669389846266193 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0165_peak_rss=2457600_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-1b68226b364a8973 seed=16390669389846266193 condition=dense-matched accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1034645.483217 note=wall_secs=0.0058_peak_rss=3719168_spikes=640_deliveries=27379_cells=28019_plasticity=694080
config_hash=c1-1b68226b364a8973 seed=9344921060680809828 condition=local-assembly accuracy=0.450000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1704535.600710 note=wall_secs=0.0058_peak_rss=3440640_spikes=711_deliveries=13305_cells=13953_plasticity=739072
config_hash=c1-1b68226b364a8973 seed=9344921060680809828 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5266164.000000 note=wall_secs=0.0134_peak_rss=4390912_spikes=901_deliveries=115568_cells=116229_plasticity=2400384
config_hash=c1-1b68226b364a8973 seed=9344921060680809828 condition=gradient-reference accuracy=0.725000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=157139856.901371 note=wall_secs=0.6926_peak_rss=2834432_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-1b68226b364a8973 seed=9344921060680809828 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0154_peak_rss=2473984_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-1b68226b364a8973 seed=9344921060680809828 condition=dense-matched accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1032946.172928 note=wall_secs=0.0066_peak_rss=3899392_spikes=640_deliveries=27363_cells=28003_plasticity=692880
config_hash=c1-1b68226b364a8973 seed=2298610881073559931 condition=local-assembly accuracy=1.000000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=634426.000000 note=wall_secs=0.0054_peak_rss=3473408_spikes=763_deliveries=13174_cells=13799_plasticity=606690
config_hash=c1-1b68226b364a8973 seed=2298610881073559931 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5266164.000000 note=wall_secs=0.0126_peak_rss=4161536_spikes=901_deliveries=115568_cells=116229_plasticity=2400384
config_hash=c1-1b68226b364a8973 seed=2298610881073559931 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6870_peak_rss=2834432_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-1b68226b364a8973 seed=2298610881073559931 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0174_peak_rss=2457600_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-1b68226b364a8973 seed=2298610881073559931 condition=dense-matched accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1033652.379801 note=wall_secs=0.0059_peak_rss=3801088_spikes=640_deliveries=27379_cells=28019_plasticity=693360
config_hash=c1-1b68226b364a8973 seed=13699608824640910734 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1488342.000000 note=wall_secs=0.0055_peak_rss=3440640_spikes=707_deliveries=13050_cells=13694_plasticity=716720
config_hash=c1-1b68226b364a8973 seed=13699608824640910734 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5266164.000000 note=wall_secs=0.0125_peak_rss=4194304_spikes=901_deliveries=115568_cells=116229_plasticity=2400384
config_hash=c1-1b68226b364a8973 seed=13699608824640910734 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6894_peak_rss=2834432_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-1b68226b364a8973 seed=13699608824640910734 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0158_peak_rss=2473984_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-1b68226b364a8973 seed=13699608824640910734 condition=dense-matched accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1033983.414273 note=wall_secs=0.0058_peak_rss=3637248_spikes=640_deliveries=27379_cells=28019_plasticity=693600
config_hash=c1-1b68226b364a8973 seed=6653297820399940005 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1442806.000000 note=wall_secs=0.0054_peak_rss=3473408_spikes=641_deliveries=13201_cells=13841_plasticity=693720
config_hash=c1-1b68226b364a8973 seed=6653297820399940005 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5266164.000000 note=wall_secs=0.0134_peak_rss=4423680_spikes=901_deliveries=115568_cells=116229_plasticity=2400384
config_hash=c1-1b68226b364a8973 seed=6653297820399940005 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6934_peak_rss=2834432_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-1b68226b364a8973 seed=6653297820399940005 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0159_peak_rss=2457600_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-1b68226b364a8973 seed=6653297820399940005 condition=dense-matched accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1034148.931509 note=wall_secs=0.0059_peak_rss=3719168_spikes=640_deliveries=27379_cells=28019_plasticity=693720
```
