# C1 / Gate G2 results note

**Config hash:** `c1-493ddd56f8714fb6`

**Scientific protocol version:** `15`

**Structured frozen feedback protocol:** `15` — same live RFB plasticity path as v13, but hidden `B_i = sign(w→readout_1 − w→readout_0)` after readout boost (not Uniform[-1,1]); single-pass; **positive control stays on broadcast ±1**; does **not** remassage v13 hash `c1-660401d74db3c88d` or reopen protocol-v2 `c1-118207fbc3eaba53`.

**Verdict (Gate G2):** **FAIL**

PASS = lower confidence bound on normalized gradient gap closed > 0.500 and mean local accuracy >= 0.650.
FAIL = a full run missed at least one preregistered threshold.
PILOT = quick schedule or fewer seeds than the power-analysis requirement; not a scientific G2 decision.
INVALID_HARNESS = positive_control_mean < 0.900 or mean activity sparsity outside [0.0050, 0.0300]; prohibits PASS/FAIL and U-NEG language.

## Conditions

| Label | Meaning |
|---|---|
| `local-assembly` | Three-factor rule + sparse assembly + k-WTA + dual readouts + **`ReinforceFeedback` × `reinforce_term`** (opt-in; not broadcast ±1) |
| `dense-local` | Same three-factor + k-WTA budget on dense all-to-all, **no** assembly; same `ReinforceFeedback` neuromodulator |
| `dense-matched` | Dense-local with nnz matched to local-assembly (parameter-matched; measured compute disclosed below) |
| `gradient-reference` | Same-architecture surrogate-LIF BPTT (primary); tanh RNN optional/secondary |
| `eligibility-reference` | E-prop-compatible eligibility local reference (rate-model approximation; feedforward-only) |

Plasticity uses directional REINFORCE × frozen per-neuron feedback (`ReinforceFeedback`) by design; broadcast ±1 remains the default C1 path. Gap-closed is clamped to `[0, 1]` and seeds with `(reference − dense) < 0.150` contribute `closed = 0`.

## Config

```
Config { experiment: "c1-sfb", master_seed: 212618061021185, n_seeds: 20, sequence_len: 8, max_lag: 1, n_hidden: 128, k_wta: 2, p_sparse: 0.35, init_w: 0.15, eta: 0.35, lambda: 0.002, tau_e: 40.0, n_train: 80, n_test: 40, bptt_epochs: 80, bptt_lr: 0.05, g2_min_gap_closed: 0.5, g2_min_accuracy: 0.65, g2_confidence_z: 1.96, g2_min_positive_control: 0.9, g2_min_reference_gap: 0.15, activity_sparsity_min: 0.005, activity_sparsity_max: 0.03, scientific_n_seeds: 20, power_sigma_prior: 0.15, power_effect_size: 0.1, use_surrogate_lif_reference: true, surrogate_beta: 5.0, matched_budget_repeat: true, quick: false }
```

Power analysis: required scientific n_seeds ≥ 20 (preregistered σ=0.150, effect=0.100; formula n=⌈(1.96+0.8416)²σ²/δ²⌉).

## Per-seed accuracies

| seed | local-assembly | dense-local | gradient reference | eligibility reference | activity_sparsity (local) | activity_sparsity (dense) | dense_matched |
|---|---:|---:|---:|---:|---:|---:|---:|
| 11400784225994701844 | 1.0000 | 0.4250 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.4750 |
| 4354472946875824171 | 0.5000 | 0.5750 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 15755469790931547198 | 1.0000 | 0.4500 | 0.7250 | 1.0000 | 0.0156 | 0.0156 | 0.5250 |
| 8709160710835925077 | 0.5000 | 0.5500 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.4500 |
| 1663413756060003432 | 0.5750 | 0.5500 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.3750 |
| 13063846550650677375 | 0.6250 | 0.5500 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.4750 |
| 6018099320996848786 | 1.0000 | 0.5750 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.5250 |
| 17418529916564267177 | 0.5750 | 0.5000 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.4750 |
| 10372782686910438588 | 0.5500 | 0.5000 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 3326471682669467859 | 0.7250 | 0.4500 | 0.7250 | 1.0000 | 0.0156 | 0.0156 | 0.3250 |
| 14727610363725173990 | 0.3500 | 0.4000 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.4500 |
| 7681300184117924093 | 1.0000 | 0.5000 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.3750 |
| 635551854952467728 | 1.0000 | 0.5750 | 0.5000 | 1.0000 | 0.0156 | 0.0156 | 0.6250 |
| 12035985749054769447 | 1.0000 | 0.5750 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 4990235495743964474 | 1.0000 | 0.4250 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 16390669389846266193 | 0.5000 | 0.5000 | 0.5000 | 1.0000 | 0.0156 | 0.0156 | 0.4750 |
| 9344921060680809828 | 0.7000 | 0.5000 | 0.7250 | 1.0000 | 0.0156 | 0.0156 | 0.6500 |
| 2298610881073559931 | 0.5000 | 0.5500 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.5750 |
| 13699608824640910734 | 1.0000 | 0.4500 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.5750 |
| 6653297820399940005 | 0.4250 | 0.4500 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.5500 |

## Summary (paired normalized-gap analysis)

- mean ± var local-assembly: 0.7262 ± 0.059176
- mean ± var dense-local:    0.5025 ± 0.003349
- mean ± var gradient reference: 0.8938 ± 0.027163
- mean ± var eligibility reference: 1.0000 ± 0.000000
- mean normalized gap closed: 0.4690 (variance 0.234643)
- lower confidence bound (z=1.960, n=20): 0.2567
- mean |local − dense| (descriptive): 0.2488

## U-NEG

Negative result: local-assembly did **not** clear the preregistered normalized-gap confidence and absolute-accuracy gates. Program stops at G2; do not schedule P3+.

## Positive / sanity control

Mean local-pipeline accuracy on a trivially separable spatial feature-presence task: **0.9488** (threshold 0.900).

## Activity sparsity

Mean local-assembly activity_sparsity: **0.0156** (valid band [0.0050, 0.0300]; nominal k/N=0.0156).

## Parameter / compute budgets

| condition | n_cells | n_params | wall_secs | peak_rss_bytes | work_per_accuracy | spikes | deliveries | cell_updates | plasticity |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| local-assembly | 132 | 5777 | 0.0086 | 3850240 | 490129.0000 | 812 | 13265 | 13892 | 462160 |
| dense-local | 132 | 16768 | 0.0142 | 5177344 | 3450261.0797 | 924 | 61680 | 62317 | 1341440 |
| gradient-reference | 130 | 16769 | 0.6957 | 3063808 | 113926400.0000 | 0 | 51200 | 6553600 | 107321600 |
| eligibility-reference | 130 | 385 | 0.0157 | 2686976 | 9068800.0000 | 0 | 51200 | 6553600 | 2464000 |
| dense-matched | 132 | 5777 | 0.0083 | 3964928 | 1149490.5407 | 901 | 41155 | 41792 | 462160 |

Matched-budget dense mean accuracy: **0.4950** (n=20; primary G2 gap still uses unmatched dense-local).

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
config_hash=c1-493ddd56f8714fb6 seed=11400784225994701844 condition=local-assembly accuracy=1.000000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=490129.000000 note=wall_secs=0.0086_peak_rss=3850240_spikes=812_deliveries=13265_cells=13892_plasticity=462160
config_hash=c1-493ddd56f8714fb6 seed=11400784225994701844 condition=dense-local accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=3450261.079693 note=wall_secs=0.0142_peak_rss=5177344_spikes=924_deliveries=61680_cells=62317_plasticity=1341440
config_hash=c1-493ddd56f8714fb6 seed=11400784225994701844 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6957_peak_rss=3063808_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-493ddd56f8714fb6 seed=11400784225994701844 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0157_peak_rss=2686976_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-493ddd56f8714fb6 seed=11400784225994701844 condition=dense-matched accuracy=0.475000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1149490.540740 note=wall_secs=0.0083_peak_rss=3964928_spikes=901_deliveries=41155_cells=41792_plasticity=462160
config_hash=c1-493ddd56f8714fb6 seed=4354472946875824171 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=979388.000000 note=wall_secs=0.0050_peak_rss=3833856_spikes=711_deliveries=13174_cells=13809_plasticity=462000
config_hash=c1-493ddd56f8714fb6 seed=4354472946875824171 condition=dense-local accuracy=0.575000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2550140.922435 note=wall_secs=0.0096_peak_rss=4571136_spikes=897_deliveries=61680_cells=62314_plasticity=1341440
config_hash=c1-493ddd56f8714fb6 seed=4354472946875824171 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6674_peak_rss=3047424_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-493ddd56f8714fb6 seed=4354472946875824171 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0153_peak_rss=2703360_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-493ddd56f8714fb6 seed=4354472946875824171 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1091696.000000 note=wall_secs=0.0056_peak_rss=4014080_spikes=892_deliveries=41161_cells=41795_plasticity=462000
config_hash=c1-493ddd56f8714fb6 seed=15755469790931547198 condition=local-assembly accuracy=1.000000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=489533.000000 note=wall_secs=0.0051_peak_rss=3735552_spikes=672_deliveries=13154_cells=13787_plasticity=461920
config_hash=c1-493ddd56f8714fb6 seed=15755469790931547198 condition=dense-local accuracy=0.450000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=3258522.308544 note=wall_secs=0.0098_peak_rss=4456448_spikes=903_deliveries=61680_cells=62312_plasticity=1341440
config_hash=c1-493ddd56f8714fb6 seed=15755469790931547198 condition=gradient-reference accuracy=0.725000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=157139856.901371 note=wall_secs=0.6663_peak_rss=2965504_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-493ddd56f8714fb6 seed=15755469790931547198 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0157_peak_rss=2670592_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-493ddd56f8714fb6 seed=15755469790931547198 condition=dense-matched accuracy=0.525000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1039219.094813 note=wall_secs=0.0057_peak_rss=4014080_spikes=893_deliveries=41072_cells=41705_plasticity=461920
config_hash=c1-493ddd56f8714fb6 seed=8709160710835925077 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=982784.000000 note=wall_secs=0.0053_peak_rss=3686400_spikes=775_deliveries=13351_cells=13986_plasticity=463280
config_hash=c1-493ddd56f8714fb6 seed=8709160710835925077 condition=dense-local accuracy=0.550000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2666103.578577 note=wall_secs=0.0102_peak_rss=4554752_spikes=916_deliveries=61680_cells=62321_plasticity=1341440
config_hash=c1-493ddd56f8714fb6 seed=8709160710835925077 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6688_peak_rss=2965504_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-493ddd56f8714fb6 seed=8709160710835925077 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0152_peak_rss=2670592_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-493ddd56f8714fb6 seed=8709160710835925077 condition=dense-matched accuracy=0.450000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1215635.587759 note=wall_secs=0.0063_peak_rss=4145152_spikes=906_deliveries=41105_cells=41745_plasticity=463280
config_hash=c1-493ddd56f8714fb6 seed=1663413756060003432 condition=local-assembly accuracy=0.575000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=854396.539453 note=wall_secs=0.0051_peak_rss=3784704_spikes=766_deliveries=13262_cells=13890_plasticity=463360
config_hash=c1-493ddd56f8714fb6 seed=1663413756060003432 condition=dense-local accuracy=0.550000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2666072.669487 note=wall_secs=0.0099_peak_rss=4505600_spikes=907_deliveries=61680_cells=62313_plasticity=1341440
config_hash=c1-493ddd56f8714fb6 seed=1663413756060003432 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6938_peak_rss=3063808_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-493ddd56f8714fb6 seed=1663413756060003432 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0189_peak_rss=2686976_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-493ddd56f8714fb6 seed=1663413756060003432 condition=dense-matched accuracy=0.375000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1459184.000000 note=wall_secs=0.0059_peak_rss=4112384_spikes=893_deliveries=41154_cells=41787_plasticity=463360
config_hash=c1-493ddd56f8714fb6 seed=13063846550650677375 condition=local-assembly accuracy=0.625000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=783833.600000 note=wall_secs=0.0084_peak_rss=3784704_spikes=773_deliveries=13284_cells=13919_plasticity=461920
config_hash=c1-493ddd56f8714fb6 seed=13063846550650677375 condition=dense-local accuracy=0.550000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2666118.124032 note=wall_secs=0.0130_peak_rss=5160960_spikes=920_deliveries=61680_cells=62325_plasticity=1341440
config_hash=c1-493ddd56f8714fb6 seed=13063846550650677375 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.7010_peak_rss=3112960_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-493ddd56f8714fb6 seed=13063846550650677375 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0167_peak_rss=2703360_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-493ddd56f8714fb6 seed=13063846550650677375 condition=dense-matched accuracy=0.475000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1148625.277571 note=wall_secs=0.0090_peak_rss=4063232_spikes=917_deliveries=41057_cells=41703_plasticity=461920
config_hash=c1-493ddd56f8714fb6 seed=6018099320996848786 condition=local-assembly accuracy=1.000000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=490315.000000 note=wall_secs=0.0051_peak_rss=3670016_spikes=800_deliveries=13521_cells=14154_plasticity=461840
config_hash=c1-493ddd56f8714fb6 seed=6018099320996848786 condition=dense-local accuracy=0.575000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2550187.878958 note=wall_secs=0.0096_peak_rss=4456448_spikes=914_deliveries=61680_cells=62324_plasticity=1341440
config_hash=c1-493ddd56f8714fb6 seed=6018099320996848786 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6703_peak_rss=2965504_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-493ddd56f8714fb6 seed=6018099320996848786 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0150_peak_rss=2686976_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-493ddd56f8714fb6 seed=6018099320996848786 condition=dense-matched accuracy=0.525000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1039638.142451 note=wall_secs=0.0059_peak_rss=4030464_spikes=901_deliveries=41212_cells=41857_plasticity=461840
config_hash=c1-493ddd56f8714fb6 seed=17418529916564267177 condition=local-assembly accuracy=0.575000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=853542.626391 note=wall_secs=0.0050_peak_rss=3817472_spikes=777_deliveries=13485_cells=14125_plasticity=462400
config_hash=c1-493ddd56f8714fb6 seed=17418529916564267177 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2932796.000000 note=wall_secs=0.0097_peak_rss=4571136_spikes=950_deliveries=61680_cells=62328_plasticity=1341440
config_hash=c1-493ddd56f8714fb6 seed=17418529916564267177 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6662_peak_rss=2965504_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-493ddd56f8714fb6 seed=17418529916564267177 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0156_peak_rss=2654208_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-493ddd56f8714fb6 seed=17418529916564267177 condition=dense-matched accuracy=0.475000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1149905.277587 note=wall_secs=0.0058_peak_rss=4046848_spikes=924_deliveries=41117_cells=41764_plasticity=462400
config_hash=c1-493ddd56f8714fb6 seed=10372782686910438588 condition=local-assembly accuracy=0.550000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=892029.071575 note=wall_secs=0.0048_peak_rss=3686400_spikes=776_deliveries=13364_cells=13996_plasticity=462480
config_hash=c1-493ddd56f8714fb6 seed=10372782686910438588 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2932668.000000 note=wall_secs=0.0098_peak_rss=4472832_spikes=899_deliveries=61680_cells=62315_plasticity=1341440
config_hash=c1-493ddd56f8714fb6 seed=10372782686910438588 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6657_peak_rss=2981888_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-493ddd56f8714fb6 seed=10372782686910438588 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0409_peak_rss=2703360_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-493ddd56f8714fb6 seed=10372782686910438588 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1092272.000000 note=wall_secs=0.0059_peak_rss=4128768_spikes=902_deliveries=41059_cells=41695_plasticity=462480
config_hash=c1-493ddd56f8714fb6 seed=3326471682669467859 condition=local-assembly accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=676593.081198 note=wall_secs=0.0056_peak_rss=3686400_spikes=792_deliveries=13430_cells=14068_plasticity=462240
config_hash=c1-493ddd56f8714fb6 seed=3326471682669467859 condition=dense-local accuracy=0.450000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=3258504.530765 note=wall_secs=0.0104_peak_rss=4603904_spikes=884_deliveries=61680_cells=62323_plasticity=1341440
config_hash=c1-493ddd56f8714fb6 seed=3326471682669467859 condition=gradient-reference accuracy=0.725000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=157139856.901371 note=wall_secs=0.6669_peak_rss=3047424_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-493ddd56f8714fb6 seed=3326471682669467859 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0180_peak_rss=2670592_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-493ddd56f8714fb6 seed=3326471682669467859 condition=dense-matched accuracy=0.325000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1679889.292387 note=wall_secs=0.0064_peak_rss=4128768_spikes=895_deliveries=41093_cells=41736_plasticity=462240
config_hash=c1-493ddd56f8714fb6 seed=14727610363725173990 condition=local-assembly accuracy=0.350000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1401080.023860 note=wall_secs=0.0054_peak_rss=3784704_spikes=737_deliveries=13259_cells=13902_plasticity=462480
config_hash=c1-493ddd56f8714fb6 seed=14727610363725173990 condition=dense-local accuracy=0.400000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=3665984.945373 note=wall_secs=0.0097_peak_rss=4456448_spikes=950_deliveries=61680_cells=62324_plasticity=1341440
config_hash=c1-493ddd56f8714fb6 seed=14727610363725173990 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6666_peak_rss=2965504_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-493ddd56f8714fb6 seed=14727610363725173990 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0184_peak_rss=2670592_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-493ddd56f8714fb6 seed=14727610363725173990 condition=dense-matched accuracy=0.450000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1212800.032128 note=wall_secs=0.0060_peak_rss=4046848_spikes=924_deliveries=40856_cells=41500_plasticity=462480
config_hash=c1-493ddd56f8714fb6 seed=7681300184117924093 condition=local-assembly accuracy=1.000000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=488707.000000 note=wall_secs=0.0047_peak_rss=3817472_spikes=795_deliveries=13198_cells=13834_plasticity=460880
config_hash=c1-493ddd56f8714fb6 seed=7681300184117924093 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2932720.000000 note=wall_secs=0.0094_peak_rss=4440064_spikes=918_deliveries=61680_cells=62322_plasticity=1341440
config_hash=c1-493ddd56f8714fb6 seed=7681300184117924093 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6691_peak_rss=3063808_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-493ddd56f8714fb6 seed=7681300184117924093 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0402_peak_rss=2703360_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-493ddd56f8714fb6 seed=7681300184117924093 condition=dense-matched accuracy=0.375000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1452392.000000 note=wall_secs=0.0058_peak_rss=4128768_spikes=918_deliveries=41104_cells=41745_plasticity=460880
config_hash=c1-493ddd56f8714fb6 seed=635551854952467728 condition=local-assembly accuracy=1.000000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=489860.000000 note=wall_secs=0.0054_peak_rss=3768320_spikes=776_deliveries=13387_cells=14017_plasticity=461680
config_hash=c1-493ddd56f8714fb6 seed=635551854952467728 condition=dense-local accuracy=0.575000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2550151.357218 note=wall_secs=0.0098_peak_rss=4554752_spikes=900_deliveries=61680_cells=62317_plasticity=1341440
config_hash=c1-493ddd56f8714fb6 seed=635551854952467728 condition=gradient-reference accuracy=0.500000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=227852800.000000 note=wall_secs=0.7088_peak_rss=3063808_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-493ddd56f8714fb6 seed=635551854952467728 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0160_peak_rss=2670592_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-493ddd56f8714fb6 seed=635551854952467728 condition=dense-matched accuracy=0.625000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=872185.600000 note=wall_secs=0.0057_peak_rss=4030464_spikes=898_deliveries=40950_cells=41588_plasticity=461680
config_hash=c1-493ddd56f8714fb6 seed=12035985749054769447 condition=local-assembly accuracy=1.000000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=491995.000000 note=wall_secs=0.0051_peak_rss=3817472_spikes=761_deliveries=13463_cells=14091_plasticity=463680
config_hash=c1-493ddd56f8714fb6 seed=12035985749054769447 condition=dense-local accuracy=0.575000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2550137.444174 note=wall_secs=0.0103_peak_rss=4718592_spikes=892_deliveries=61680_cells=62317_plasticity=1341440
config_hash=c1-493ddd56f8714fb6 seed=12035985749054769447 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6754_peak_rss=3047424_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-493ddd56f8714fb6 seed=12035985749054769447 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0160_peak_rss=2670592_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-493ddd56f8714fb6 seed=12035985749054769447 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1094654.000000 note=wall_secs=0.0060_peak_rss=4128768_spikes=887_deliveries=41062_cells=41698_plasticity=463680
config_hash=c1-493ddd56f8714fb6 seed=4990235495743964474 condition=local-assembly accuracy=1.000000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=490999.000000 note=wall_secs=0.0056_peak_rss=3686400_spikes=803_deliveries=13415_cells=14061_plasticity=462720
config_hash=c1-493ddd56f8714fb6 seed=4990235495743964474 condition=dense-local accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=3450218.726753 note=wall_secs=0.0098_peak_rss=4538368_spikes=901_deliveries=61680_cells=62322_plasticity=1341440
config_hash=c1-493ddd56f8714fb6 seed=4990235495743964474 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6710_peak_rss=2965504_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-493ddd56f8714fb6 seed=4990235495743964474 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0180_peak_rss=2686976_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-493ddd56f8714fb6 seed=4990235495743964474 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1093022.000000 note=wall_secs=0.0062_peak_rss=4128768_spikes=899_deliveries=41125_cells=41767_plasticity=462720
config_hash=c1-493ddd56f8714fb6 seed=16390669389846266193 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=981942.000000 note=wall_secs=0.0054_peak_rss=3768320_spikes=828_deliveries=13391_cells=14032_plasticity=462720
config_hash=c1-493ddd56f8714fb6 seed=16390669389846266193 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2932744.000000 note=wall_secs=0.0104_peak_rss=4374528_spikes=930_deliveries=61680_cells=62322_plasticity=1341440
config_hash=c1-493ddd56f8714fb6 seed=16390669389846266193 condition=gradient-reference accuracy=0.500000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=227852800.000000 note=wall_secs=0.6637_peak_rss=2981888_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-493ddd56f8714fb6 seed=16390669389846266193 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0399_peak_rss=2703360_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-493ddd56f8714fb6 seed=16390669389846266193 condition=dense-matched accuracy=0.475000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1150604.224964 note=wall_secs=0.0065_peak_rss=4030464_spikes=913_deliveries=41131_cells=41773_plasticity=462720
config_hash=c1-493ddd56f8714fb6 seed=9344921060680809828 condition=local-assembly accuracy=0.700000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=699980.011921 note=wall_secs=0.0050_peak_rss=3686400_spikes=808_deliveries=13313_cells=13945_plasticity=461920
config_hash=c1-493ddd56f8714fb6 seed=9344921060680809828 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2932720.000000 note=wall_secs=0.0099_peak_rss=4472832_spikes=924_deliveries=61680_cells=62316_plasticity=1341440
config_hash=c1-493ddd56f8714fb6 seed=9344921060680809828 condition=gradient-reference accuracy=0.725000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=157139856.901371 note=wall_secs=0.6645_peak_rss=2965504_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-493ddd56f8714fb6 seed=9344921060680809828 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0406_peak_rss=2686976_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-493ddd56f8714fb6 seed=9344921060680809828 condition=dense-matched accuracy=0.650000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=839560.030795 note=wall_secs=0.0056_peak_rss=3948544_spikes=915_deliveries=41122_cells=41757_plasticity=461920
config_hash=c1-493ddd56f8714fb6 seed=2298610881073559931 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=980266.000000 note=wall_secs=0.0050_peak_rss=3686400_spikes=803_deliveries=13228_cells=13862_plasticity=462240
config_hash=c1-493ddd56f8714fb6 seed=2298610881073559931 condition=dense-local accuracy=0.550000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2666078.124032 note=wall_secs=0.0097_peak_rss=4554752_spikes=902_deliveries=61680_cells=62321_plasticity=1341440
config_hash=c1-493ddd56f8714fb6 seed=2298610881073559931 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6655_peak_rss=2965504_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-493ddd56f8714fb6 seed=2298610881073559931 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0153_peak_rss=2686976_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-493ddd56f8714fb6 seed=2298610881073559931 condition=dense-matched accuracy=0.575000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=949450.454467 note=wall_secs=0.0065_peak_rss=4128768_spikes=901_deliveries=41076_cells=41717_plasticity=462240
config_hash=c1-493ddd56f8714fb6 seed=13699608824640910734 condition=local-assembly accuracy=1.000000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=490236.000000 note=wall_secs=0.0055_peak_rss=3817472_spikes=829_deliveries=13190_cells=13817_plasticity=462400
config_hash=c1-493ddd56f8714fb6 seed=13699608824640910734 condition=dense-local accuracy=0.450000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=3258546.752989 note=wall_secs=0.0099_peak_rss=4456448_spikes=908_deliveries=61680_cells=62318_plasticity=1341440
config_hash=c1-493ddd56f8714fb6 seed=13699608824640910734 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6698_peak_rss=3047424_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-493ddd56f8714fb6 seed=13699608824640910734 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0161_peak_rss=2670592_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-493ddd56f8714fb6 seed=13699608824640910734 condition=dense-matched accuracy=0.575000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=949794.802300 note=wall_secs=0.0058_peak_rss=4030464_spikes=898_deliveries=41098_cells=41736_plasticity=462400
config_hash=c1-493ddd56f8714fb6 seed=6653297820399940005 condition=local-assembly accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1153524.673527 note=wall_secs=0.0052_peak_rss=3817472_spikes=739_deliveries=13192_cells=13837_plasticity=462480
config_hash=c1-493ddd56f8714fb6 seed=6653297820399940005 condition=dense-local accuracy=0.450000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=3258535.641877 note=wall_secs=0.0100_peak_rss=4456448_spikes=895_deliveries=61680_cells=62326_plasticity=1341440
config_hash=c1-493ddd56f8714fb6 seed=6653297820399940005 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6637_peak_rss=2981888_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-493ddd56f8714fb6 seed=6653297820399940005 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0154_peak_rss=2703360_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-493ddd56f8714fb6 seed=6653297820399940005 condition=dense-matched accuracy=0.550000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=993194.523928 note=wall_secs=0.0057_peak_rss=4030464_spikes=908_deliveries=41112_cells=41757_plasticity=462480
```
