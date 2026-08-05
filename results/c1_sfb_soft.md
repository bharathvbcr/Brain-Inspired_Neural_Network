# C1 / Gate G2 results note

**Config hash:** `c1-f975db8fb3e5d569`

**Scientific protocol version:** `21`

**claim_axis:** Novel-CS
**object_under_test:** Soft/relaxed k-WTA winners under structured frozen B
**may_claim:** Whether soft winners under SFB close the live transfer gap
**must_not_claim:** Temperature grid search; remassage v15; biology

**Soft-WTA × structured B protocol:** `21` — v15 structured hidden `B` with soft/relaxed k-WTA at disclosed temperature `T=1` (one temp; no grid); **positive control stays on broadcast ±1**; does **not** remassage v15 hash `c1-493ddd56f8714fb6` or reopen protocol-v2 `c1-118207fbc3eaba53`.

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
Config { experiment: "c1-sfb-soft", master_seed: 212618061021185, n_seeds: 20, sequence_len: 8, max_lag: 1, n_hidden: 128, k_wta: 2, p_sparse: 0.35, init_w: 0.15, eta: 0.35, lambda: 0.002, tau_e: 40.0, n_train: 80, n_test: 40, bptt_epochs: 80, bptt_lr: 0.05, g2_min_gap_closed: 0.5, g2_min_accuracy: 0.65, g2_confidence_z: 1.96, g2_min_positive_control: 0.9, g2_min_reference_gap: 0.15, activity_sparsity_min: 0.005, activity_sparsity_max: 0.03, scientific_n_seeds: 20, power_sigma_prior: 0.15, power_effect_size: 0.1, use_surrogate_lif_reference: true, surrogate_beta: 5.0, matched_budget_repeat: true, quick: false }
```

Power analysis: required scientific n_seeds ≥ 20 (preregistered σ=0.150, effect=0.100; formula n=⌈(1.96+0.8416)²σ²/δ²⌉).

## Per-seed accuracies

| seed | local-assembly | dense-local | gradient reference | eligibility reference | activity_sparsity (local) | activity_sparsity (dense) | dense_matched |
|---|---:|---:|---:|---:|---:|---:|---:|
| 11400784225994701844 | 0.5750 | 0.3500 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.3500 |
| 4354472946875824171 | 0.5000 | 0.5250 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.5250 |
| 15755469790931547198 | 0.5000 | 0.3500 | 0.7250 | 1.0000 | 0.0156 | 0.0156 | 0.3500 |
| 8709160710835925077 | 0.5250 | 0.5500 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.5500 |
| 1663413756060003432 | 0.5000 | 0.4250 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.4250 |
| 13063846550650677375 | 0.4750 | 0.3250 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.3250 |
| 6018099320996848786 | 0.5000 | 0.5000 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 17418529916564267177 | 0.4250 | 0.4750 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.4750 |
| 10372782686910438588 | 0.5000 | 0.4750 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.4750 |
| 3326471682669467859 | 0.5250 | 0.3250 | 0.7250 | 1.0000 | 0.0156 | 0.0156 | 0.3250 |
| 14727610363725173990 | 0.5750 | 0.6000 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.6000 |
| 7681300184117924093 | 0.5500 | 0.4500 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.4500 |
| 635551854952467728 | 0.4250 | 0.4000 | 0.5000 | 1.0000 | 0.0156 | 0.0156 | 0.4000 |
| 12035985749054769447 | 0.5250 | 0.6000 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.6000 |
| 4990235495743964474 | 0.6250 | 0.4500 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.4500 |
| 16390669389846266193 | 0.5500 | 0.5000 | 0.5000 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 9344921060680809828 | 0.4250 | 0.4750 | 0.7250 | 1.0000 | 0.0156 | 0.0156 | 0.4750 |
| 2298610881073559931 | 0.4500 | 0.6000 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.6000 |
| 13699608824640910734 | 0.4500 | 0.5000 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 6653297820399940005 | 0.4500 | 0.4250 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.4250 |

## Summary (paired normalized-gap analysis)

- mean ± var local-assembly: 0.5025 ± 0.003086
- mean ± var dense-local:    0.4650 ± 0.007526
- mean ± var gradient reference: 0.8938 ± 0.027163
- mean ± var eligibility reference: 1.0000 ± 0.000000
- mean normalized gap closed: 0.1115 (variance 0.026175)
- lower confidence bound (z=1.960, n=20): 0.0406
- mean |local − dense| (descriptive): 0.0825
- descriptive chance-normalized gap mean / LCB: 0.0444 / 0.0122 (var 0.005419; **not a gate**)
- seed local min / max / frac≥0.65: 0.4250 / 0.6250 / 0.00

## U-NEG

Negative result: local-assembly did **not** clear the preregistered normalized-gap confidence and absolute-accuracy gates. Program stops at G2; do not schedule P3+.

## Positive / sanity control

Mean local-pipeline accuracy on a trivially separable spatial feature-presence task: **0.9488** (threshold 0.900).

## Activity sparsity

Mean local-assembly activity_sparsity: **0.0156** (valid band [0.0050, 0.0300]; nominal k/N=0.0156).

## Parameter / compute budgets

| condition | n_cells | n_params | wall_secs | peak_rss_bytes | work_per_accuracy | spikes | deliveries | cell_updates | plasticity |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| local-assembly | 132 | 5777 | 0.0050 | 3571712 | 851817.4090 | 812 | 13097 | 13726 | 462160 |
| dense-local | 132 | 16768 | 0.0097 | 4571136 | 4189622.9285 | 930 | 61680 | 62318 | 1341440 |
| gradient-reference | 130 | 16769 | 0.6634 | 2801664 | 113926400.0000 | 0 | 51200 | 6553600 | 107321600 |
| eligibility-reference | 130 | 385 | 0.0154 | 2654208 | 9068800.0000 | 0 | 51200 | 6553600 | 2464000 |
| dense-matched | 132 | 5777 | 0.0059 | 4096000 | 1559902.8837 | 930 | 41119 | 41757 | 462160 |

Matched-budget dense mean accuracy: **0.4650** (n=20; primary G2 gap still uses unmatched dense-local).

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
config_hash=c1-f975db8fb3e5d569 seed=11400784225994701844 condition=local-assembly accuracy=0.575000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=851817.408964 note=wall_secs=0.0050_peak_rss=3571712_spikes=812_deliveries=13097_cells=13726_plasticity=462160
config_hash=c1-f975db8fb3e5d569 seed=11400784225994701844 condition=dense-local accuracy=0.350000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=4189622.928492 note=wall_secs=0.0097_peak_rss=4571136_spikes=930_deliveries=61680_cells=62318_plasticity=1341440
config_hash=c1-f975db8fb3e5d569 seed=11400784225994701844 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6634_peak_rss=2801664_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-f975db8fb3e5d569 seed=11400784225994701844 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0154_peak_rss=2654208_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-f975db8fb3e5d569 seed=11400784225994701844 condition=dense-matched accuracy=0.350000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1559902.883708 note=wall_secs=0.0059_peak_rss=4096000_spikes=930_deliveries=41119_cells=41757_plasticity=462160
config_hash=c1-f975db8fb3e5d569 seed=4354472946875824171 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=979654.000000 note=wall_secs=0.0050_peak_rss=3784704_spikes=807_deliveries=13192_cells=13828_plasticity=462000
config_hash=c1-f975db8fb3e5d569 seed=4354472946875824171 condition=dense-local accuracy=0.525000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2792990.603029 note=wall_secs=0.0097_peak_rss=4538368_spikes=889_deliveries=61680_cells=62311_plasticity=1341440
config_hash=c1-f975db8fb3e5d569 seed=4354472946875824171 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6655_peak_rss=2883584_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-f975db8fb3e5d569 seed=4354472946875824171 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0150_peak_rss=2637824_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-f975db8fb3e5d569 seed=4354472946875824171 condition=dense-matched accuracy=0.525000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1039462.904348 note=wall_secs=0.0056_peak_rss=3915776_spikes=889_deliveries=41099_cells=41730_plasticity=462000
config_hash=c1-f975db8fb3e5d569 seed=15755469790931547198 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=979490.000000 note=wall_secs=0.0050_peak_rss=3735552_spikes=805_deliveries=13192_cells=13828_plasticity=461920
config_hash=c1-f975db8fb3e5d569 seed=15755469790931547198 condition=dense-local accuracy=0.350000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=4189497.214204 note=wall_secs=0.0098_peak_rss=4423680_spikes=891_deliveries=61680_cells=62313_plasticity=1341440
config_hash=c1-f975db8fb3e5d569 seed=15755469790931547198 condition=gradient-reference accuracy=0.725000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=157139856.901371 note=wall_secs=0.6654_peak_rss=2834432_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-f975db8fb3e5d569 seed=15755469790931547198 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0178_peak_rss=2621440_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-f975db8fb3e5d569 seed=15755469790931547198 condition=dense-matched accuracy=0.350000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1557948.597960 note=wall_secs=0.0056_peak_rss=4046848_spikes=891_deliveries=40919_cells=41552_plasticity=461920
config_hash=c1-f975db8fb3e5d569 seed=8709160710835925077 condition=local-assembly accuracy=0.525000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=935960.042505 note=wall_secs=0.0050_peak_rss=3604480_spikes=819_deliveries=13318_cells=13962_plasticity=463280
config_hash=c1-f975db8fb3e5d569 seed=8709160710835925077 condition=dense-local accuracy=0.550000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2666087.214941 note=wall_secs=0.0105_peak_rss=4833280_spikes=908_deliveries=61680_cells=62320_plasticity=1341440
config_hash=c1-f975db8fb3e5d569 seed=8709160710835925077 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6901_peak_rss=2998272_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-f975db8fb3e5d569 seed=8709160710835925077 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0192_peak_rss=2621440_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-f975db8fb3e5d569 seed=8709160710835925077 condition=dense-matched accuracy=0.550000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=994727.251167 note=wall_secs=0.0064_peak_rss=4096000_spikes=908_deliveries=41136_cells=41776_plasticity=463280
config_hash=c1-f975db8fb3e5d569 seed=1663413756060003432 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=983014.000000 note=wall_secs=0.0055_peak_rss=3751936_spikes=819_deliveries=13349_cells=13979_plasticity=463360
config_hash=c1-f975db8fb3e5d569 seed=1663413756060003432 condition=dense-local accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=3450211.667930 note=wall_secs=0.0099_peak_rss=4374528_spikes=906_deliveries=61680_cells=62314_plasticity=1341440
config_hash=c1-f975db8fb3e5d569 seed=1663413756060003432 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6731_peak_rss=2981888_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-f975db8fb3e5d569 seed=1663413756060003432 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0176_peak_rss=2637824_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-f975db8fb3e5d569 seed=1663413756060003432 condition=dense-matched accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1287265.846246 note=wall_secs=0.0059_peak_rss=3915776_spikes=906_deliveries=41094_cells=41728_plasticity=463360
config_hash=c1-f975db8fb3e5d569 seed=13063846550650677375 condition=local-assembly accuracy=0.475000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1031509.486628 note=wall_secs=0.0050_peak_rss=3588096_spikes=802_deliveries=13301_cells=13944_plasticity=461920
config_hash=c1-f975db8fb3e5d569 seed=13063846550650677375 condition=dense-local accuracy=0.325000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=4511870.934725 note=wall_secs=0.0097_peak_rss=4374528_spikes=916_deliveries=61680_cells=62322_plasticity=1341440
config_hash=c1-f975db8fb3e5d569 seed=13063846550650677375 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6702_peak_rss=2801664_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-f975db8fb3e5d569 seed=13063846550650677375 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0156_peak_rss=2605056_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-f975db8fb3e5d569 seed=13063846550650677375 condition=dense-matched accuracy=0.325000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1678123.138476 note=wall_secs=0.0057_peak_rss=4112384_spikes=916_deliveries=40956_cells=41598_plasticity=461920
config_hash=c1-f975db8fb3e5d569 seed=6018099320996848786 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=980344.000000 note=wall_secs=0.0050_peak_rss=3653632_spikes=816_deliveries=13435_cells=14081_plasticity=461840
config_hash=c1-f975db8fb3e5d569 seed=6018099320996848786 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2932716.000000 note=wall_secs=0.0099_peak_rss=4341760_spikes=911_deliveries=61680_cells=62327_plasticity=1341440
config_hash=c1-f975db8fb3e5d569 seed=6018099320996848786 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6679_peak_rss=2899968_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-f975db8fb3e5d569 seed=6018099320996848786 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0187_peak_rss=2621440_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-f975db8fb3e5d569 seed=6018099320996848786 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1090844.000000 note=wall_secs=0.0060_peak_rss=3932160_spikes=911_deliveries=41012_cells=41659_plasticity=461840
config_hash=c1-f975db8fb3e5d569 seed=17418529916564267177 condition=local-assembly accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1154717.614670 note=wall_secs=0.0051_peak_rss=3702784_spikes=824_deliveries=13443_cells=14088_plasticity=462400
config_hash=c1-f975db8fb3e5d569 seed=17418529916564267177 condition=dense-local accuracy=0.475000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=3087107.407159 note=wall_secs=0.0100_peak_rss=4554752_spikes=929_deliveries=61680_cells=62327_plasticity=1341440
config_hash=c1-f975db8fb3e5d569 seed=17418529916564267177 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6776_peak_rss=2998272_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-f975db8fb3e5d569 seed=17418529916564267177 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0167_peak_rss=2654208_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-f975db8fb3e5d569 seed=17418529916564267177 condition=dense-matched accuracy=0.475000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1149814.751270 note=wall_secs=0.0063_peak_rss=3932160_spikes=929_deliveries=41093_cells=41740_plasticity=462400
config_hash=c1-f975db8fb3e5d569 seed=10372782686910438588 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=981120.000000 note=wall_secs=0.0056_peak_rss=3801088_spikes=813_deliveries=13313_cells=13954_plasticity=462480
config_hash=c1-f975db8fb3e5d569 seed=10372782686910438588 condition=dense-local accuracy=0.475000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=3087027.407158 note=wall_secs=0.0106_peak_rss=4571136_spikes=902_deliveries=61680_cells=62316_plasticity=1341440
config_hash=c1-f975db8fb3e5d569 seed=10372782686910438588 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6724_peak_rss=2998272_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-f975db8fb3e5d569 seed=10372782686910438588 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0153_peak_rss=2605056_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-f975db8fb3e5d569 seed=10372782686910438588 condition=dense-matched accuracy=0.475000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1149465.277582 note=wall_secs=0.0064_peak_rss=4079616_spikes=902_deliveries=40989_cells=41625_plasticity=462480
config_hash=c1-f975db8fb3e5d569 seed=3326471682669467859 condition=local-assembly accuracy=0.525000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=934462.899580 note=wall_secs=0.0051_peak_rss=3571712_spikes=822_deliveries=13444_cells=14087_plasticity=462240
config_hash=c1-f975db8fb3e5d569 seed=3326471682669467859 condition=dense-local accuracy=0.325000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=4511784.780876 note=wall_secs=0.0096_peak_rss=4554752_spikes=888_deliveries=61680_cells=62322_plasticity=1341440
config_hash=c1-f975db8fb3e5d569 seed=3326471682669467859 condition=gradient-reference accuracy=0.725000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=157139856.901371 note=wall_secs=0.7084_peak_rss=2965504_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-f975db8fb3e5d569 seed=3326471682669467859 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0153_peak_rss=2605056_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-f975db8fb3e5d569 seed=3326471682669467859 condition=dense-matched accuracy=0.325000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1680338.523173 note=wall_secs=0.0058_peak_rss=4063232_spikes=888_deliveries=41170_cells=41812_plasticity=462240
config_hash=c1-f975db8fb3e5d569 seed=14727610363725173990 condition=local-assembly accuracy=0.575000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=853125.235078 note=wall_secs=0.0048_peak_rss=3588096_spikes=803_deliveries=13314_cells=13950_plasticity=462480
config_hash=c1-f975db8fb3e5d569 seed=14727610363725173990 condition=dense-local accuracy=0.600000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2443953.236219 note=wall_secs=0.0098_peak_rss=4358144_spikes=928_deliveries=61680_cells=62324_plasticity=1341440
config_hash=c1-f975db8fb3e5d569 seed=14727610363725173990 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6686_peak_rss=2916352_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-f975db8fb3e5d569 seed=14727610363725173990 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0154_peak_rss=2605056_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-f975db8fb3e5d569 seed=14727610363725173990 condition=dense-matched accuracy=0.600000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=910503.297153 note=wall_secs=0.0057_peak_rss=4161536_spikes=928_deliveries=41125_cells=41769_plasticity=462480
config_hash=c1-f975db8fb3e5d569 seed=7681300184117924093 condition=local-assembly accuracy=0.550000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=888485.435288 note=wall_secs=0.0051_peak_rss=3768320_spikes=804_deliveries=13172_cells=13811_plasticity=460880
config_hash=c1-f975db8fb3e5d569 seed=7681300184117924093 condition=dense-local accuracy=0.450000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=3258584.530767 note=wall_secs=0.0100_peak_rss=4390912_spikes=920_deliveries=61680_cells=62323_plasticity=1341440
config_hash=c1-f975db8fb3e5d569 seed=7681300184117924093 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6726_peak_rss=2916352_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-f975db8fb3e5d569 seed=7681300184117924093 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0409_peak_rss=2654208_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-f975db8fb3e5d569 seed=7681300184117924093 condition=dense-matched accuracy=0.450000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1209824.476494 note=wall_secs=0.0058_peak_rss=4079616_spikes=920_deliveries=40989_cells=41632_plasticity=460880
config_hash=c1-f975db8fb3e5d569 seed=635551854952467728 condition=local-assembly accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1152898.791191 note=wall_secs=0.0062_peak_rss=3784704_spikes=808_deliveries=13428_cells=14066_plasticity=461680
config_hash=c1-f975db8fb3e5d569 seed=635551854952467728 condition=dense-local accuracy=0.400000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=3665819.945375 note=wall_secs=0.0127_peak_rss=4784128_spikes=893_deliveries=61680_cells=62315_plasticity=1341440
config_hash=c1-f975db8fb3e5d569 seed=635551854952467728 condition=gradient-reference accuracy=0.500000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=227852800.000000 note=wall_secs=0.6820_peak_rss=2981888_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-f975db8fb3e5d569 seed=635551854952467728 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0164_peak_rss=2605056_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-f975db8fb3e5d569 seed=635551854952467728 condition=dense-matched accuracy=0.400000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1363044.979689 note=wall_secs=0.0061_peak_rss=4030464_spikes=893_deliveries=41005_cells=41640_plasticity=461680
config_hash=c1-f975db8fb3e5d569 seed=12035985749054769447 condition=local-assembly accuracy=0.525000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=937386.709236 note=wall_secs=0.0058_peak_rss=3653632_spikes=848_deliveries=13481_cells=14119_plasticity=463680
config_hash=c1-f975db8fb3e5d569 seed=12035985749054769447 condition=dense-local accuracy=0.600000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2443871.569556 note=wall_secs=0.0103_peak_rss=4571136_spikes=889_deliveries=61680_cells=62314_plasticity=1341440
config_hash=c1-f975db8fb3e5d569 seed=12035985749054769447 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6737_peak_rss=2916352_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-f975db8fb3e5d569 seed=12035985749054769447 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0153_peak_rss=2654208_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-f975db8fb3e5d569 seed=12035985749054769447 condition=dense-matched accuracy=0.600000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=912364.963746 note=wall_secs=0.0062_peak_rss=3915776_spikes=889_deliveries=41108_cells=41742_plasticity=463680
config_hash=c1-f975db8fb3e5d569 seed=4990235495743964474 condition=local-assembly accuracy=0.625000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=785790.400000 note=wall_secs=0.0051_peak_rss=3768320_spikes=833_deliveries=13462_cells=14104_plasticity=462720
config_hash=c1-f975db8fb3e5d569 seed=4990235495743964474 condition=dense-local accuracy=0.450000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=3258537.864100 note=wall_secs=0.0099_peak_rss=4390912_spikes=902_deliveries=61680_cells=62320_plasticity=1341440
config_hash=c1-f975db8fb3e5d569 seed=4990235495743964474 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6653_peak_rss=2916352_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-f975db8fb3e5d569 seed=4990235495743964474 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0157_peak_rss=2605056_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-f975db8fb3e5d569 seed=4990235495743964474 condition=dense-matched accuracy=0.450000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1214191.143276 note=wall_secs=0.0060_peak_rss=3932160_spikes=902_deliveries=41062_cells=41702_plasticity=462720
config_hash=c1-f975db8fb3e5d569 seed=16390669389846266193 condition=local-assembly accuracy=0.550000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=892456.344293 note=wall_secs=0.0052_peak_rss=3784704_spikes=827_deliveries=13333_cells=13971_plasticity=462720
config_hash=c1-f975db8fb3e5d569 seed=16390669389846266193 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2932722.000000 note=wall_secs=0.0103_peak_rss=4816896_spikes=919_deliveries=61680_cells=62322_plasticity=1341440
config_hash=c1-f975db8fb3e5d569 seed=16390669389846266193 condition=gradient-reference accuracy=0.500000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=227852800.000000 note=wall_secs=0.6705_peak_rss=2899968_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-f975db8fb3e5d569 seed=16390669389846266193 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0157_peak_rss=2654208_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-f975db8fb3e5d569 seed=16390669389846266193 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1093174.000000 note=wall_secs=0.0060_peak_rss=4161536_spikes=919_deliveries=41153_cells=41795_plasticity=462720
config_hash=c1-f975db8fb3e5d569 seed=9344921060680809828 condition=local-assembly accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1152934.085308 note=wall_secs=0.0054_peak_rss=3801088_spikes=810_deliveries=13316_cells=13951_plasticity=461920
config_hash=c1-f975db8fb3e5d569 seed=9344921060680809828 condition=dense-local accuracy=0.475000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=3087042.144000 note=wall_secs=0.0101_peak_rss=4374528_spikes=913_deliveries=61680_cells=62312_plasticity=1341440
config_hash=c1-f975db8fb3e5d569 seed=9344921060680809828 condition=gradient-reference accuracy=0.725000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=157139856.901371 note=wall_secs=0.6719_peak_rss=2883584_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-f975db8fb3e5d569 seed=9344921060680809828 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0157_peak_rss=2605056_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-f975db8fb3e5d569 seed=9344921060680809828 condition=dense-matched accuracy=0.475000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1148494.751254 note=wall_secs=0.0058_peak_rss=4145152_spikes=913_deliveries=41035_cells=41667_plasticity=461920
config_hash=c1-f975db8fb3e5d569 seed=2298610881073559931 condition=local-assembly accuracy=0.450000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1089108.917740 note=wall_secs=0.0050_peak_rss=3768320_spikes=812_deliveries=13205_cells=13842_plasticity=462240
config_hash=c1-f975db8fb3e5d569 seed=2298610881073559931 condition=dense-local accuracy=0.600000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2443913.236221 note=wall_secs=0.0099_peak_rss=4505600_spikes=906_deliveries=61680_cells=62322_plasticity=1341440
config_hash=c1-f975db8fb3e5d569 seed=2298610881073559931 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6718_peak_rss=2883584_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-f975db8fb3e5d569 seed=2298610881073559931 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0404_peak_rss=2654208_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-f975db8fb3e5d569 seed=2298610881073559931 condition=dense-matched accuracy=0.600000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=909993.297173 note=wall_secs=0.0059_peak_rss=4046848_spikes=906_deliveries=41104_cells=41746_plasticity=462240
config_hash=c1-f975db8fb3e5d569 seed=13699608824640910734 condition=local-assembly accuracy=0.450000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1088973.362181 note=wall_secs=0.0052_peak_rss=3571712_spikes=818_deliveries=13088_cells=13732_plasticity=462400
config_hash=c1-f975db8fb3e5d569 seed=13699608824640910734 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2932678.000000 note=wall_secs=0.0096_peak_rss=4374528_spikes=900_deliveries=61680_cells=62319_plasticity=1341440
config_hash=c1-f975db8fb3e5d569 seed=13699608824640910734 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6721_peak_rss=2899968_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-f975db8fb3e5d569 seed=13699608824640910734 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0160_peak_rss=2654208_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-f975db8fb3e5d569 seed=13699608824640910734 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1092802.000000 note=wall_secs=0.0061_peak_rss=4063232_spikes=900_deliveries=41231_cells=41870_plasticity=462400
config_hash=c1-f975db8fb3e5d569 seed=6653297820399940005 condition=local-assembly accuracy=0.450000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1090186.695547 note=wall_secs=0.0057_peak_rss=3588096_spikes=823_deliveries=13319_cells=13962_plasticity=462480
config_hash=c1-f975db8fb3e5d569 seed=6653297820399940005 condition=dense-local accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=3450258.726752 note=wall_secs=0.0111_peak_rss=4505600_spikes=912_deliveries=61680_cells=62328_plasticity=1341440
config_hash=c1-f975db8fb3e5d569 seed=6653297820399940005 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6717_peak_rss=2916352_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-f975db8fb3e5d569 seed=6653297820399940005 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0182_peak_rss=2621440_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-f975db8fb3e5d569 seed=6653297820399940005 condition=dense-matched accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1285294.081596 note=wall_secs=0.0066_peak_rss=3981312_spikes=912_deliveries=41105_cells=41753_plasticity=462480
```
