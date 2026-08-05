# C1 / Gate G2 results note

**Config hash:** `c1-dfab4a7ec19f17c2`

**Scientific protocol version:** `19`

**Structured B × target teach protocol:** `19` — same structured frozen hidden `B` as v15, but incorrect trials restore a secondary target update through `ReinforceFeedback::credit(+1)` (not observe-only); **positive control stays on broadcast ±1**; does **not** remassage v15 hash `c1-493ddd56f8714fb6` or reopen protocol-v2 `c1-118207fbc3eaba53`.

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
Config { experiment: "c1-sfb-teach", master_seed: 212618061021185, n_seeds: 20, sequence_len: 8, max_lag: 1, n_hidden: 128, k_wta: 2, p_sparse: 0.35, init_w: 0.15, eta: 0.35, lambda: 0.002, tau_e: 40.0, n_train: 80, n_test: 40, bptt_epochs: 80, bptt_lr: 0.05, g2_min_gap_closed: 0.5, g2_min_accuracy: 0.65, g2_confidence_z: 1.96, g2_min_positive_control: 0.9, g2_min_reference_gap: 0.15, activity_sparsity_min: 0.005, activity_sparsity_max: 0.03, scientific_n_seeds: 20, power_sigma_prior: 0.15, power_effect_size: 0.1, use_surrogate_lif_reference: true, surrogate_beta: 5.0, matched_budget_repeat: true, quick: false }
```

Power analysis: required scientific n_seeds ≥ 20 (preregistered σ=0.150, effect=0.100; formula n=⌈(1.96+0.8416)²σ²/δ²⌉).

## Per-seed accuracies

| seed | local-assembly | dense-local | gradient reference | eligibility reference | activity_sparsity (local) | activity_sparsity (dense) | dense_matched |
|---|---:|---:|---:|---:|---:|---:|---:|
| 11400784225994701844 | 1.0000 | 0.5000 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.4500 |
| 4354472946875824171 | 0.6250 | 0.5500 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.4500 |
| 15755469790931547198 | 0.6000 | 0.4500 | 0.7250 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 8709160710835925077 | 0.5000 | 0.5500 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.3500 |
| 1663413756060003432 | 0.6500 | 0.5500 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.4250 |
| 13063846550650677375 | 0.5250 | 0.5500 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.4750 |
| 6018099320996848786 | 1.0000 | 0.5750 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.5500 |
| 17418529916564267177 | 0.5000 | 0.5500 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.6000 |
| 10372782686910438588 | 0.6000 | 0.5000 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.5750 |
| 3326471682669467859 | 1.0000 | 0.4500 | 0.7250 | 1.0000 | 0.0156 | 0.0156 | 0.4000 |
| 14727610363725173990 | 0.1500 | 0.4000 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.4250 |
| 7681300184117924093 | 0.7250 | 0.5000 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.4000 |
| 635551854952467728 | 0.5250 | 0.5750 | 0.5000 | 1.0000 | 0.0156 | 0.0156 | 0.5750 |
| 12035985749054769447 | 0.5000 | 0.5500 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.4750 |
| 4990235495743964474 | 1.0000 | 0.4250 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.5750 |
| 16390669389846266193 | 0.5000 | 0.5250 | 0.5000 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 9344921060680809828 | 0.6750 | 0.5000 | 0.7250 | 1.0000 | 0.0156 | 0.0156 | 0.5750 |
| 2298610881073559931 | 0.9000 | 0.5500 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 13699608824640910734 | 1.0000 | 0.3750 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.5750 |
| 6653297820399940005 | 0.4250 | 0.5500 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.6000 |

## Summary (paired normalized-gap analysis)

- mean ± var local-assembly: 0.6700 ± 0.057934
- mean ± var dense-local:    0.5088 ± 0.003505
- mean ± var gradient reference: 0.8938 ± 0.027163
- mean ± var eligibility reference: 1.0000 ± 0.000000
- mean normalized gap closed: 0.4120 (variance 0.184329)
- lower confidence bound (z=1.960, n=20): 0.2238
- mean |local − dense| (descriptive): 0.2237

## U-NEG

Negative result: local-assembly did **not** clear the preregistered normalized-gap confidence and absolute-accuracy gates. Program stops at G2; do not schedule P3+.

## Positive / sanity control

Mean local-pipeline accuracy on a trivially separable spatial feature-presence task: **0.9488** (threshold 0.900).

## Activity sparsity

Mean local-assembly activity_sparsity: **0.0156** (valid band [0.0050, 0.0300]; nominal k/N=0.0156).

## Parameter / compute budgets

| condition | n_cells | n_params | wall_secs | peak_rss_bytes | work_per_accuracy | spikes | deliveries | cell_updates | plasticity |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| local-assembly | 132 | 5777 | 0.0052 | 3670016 | 623011.0000 | 831 | 13263 | 13886 | 595031 |
| dense-local | 132 | 16768 | 0.0105 | 4505600 | 4173556.0000 | 925 | 61680 | 62317 | 1961856 |
| gradient-reference | 130 | 16769 | 0.6554 | 2834432 | 113926400.0000 | 0 | 51200 | 6553600 | 107321600 |
| eligibility-reference | 130 | 385 | 0.0149 | 2621440 | 9068800.0000 | 0 | 51200 | 6553600 | 2464000 |
| dense-matched | 132 | 5777 | 0.0061 | 4030464 | 1688348.9336 | 901 | 41155 | 41792 | 675909 |

Matched-budget dense mean accuracy: **0.4987** (n=20; primary G2 gap still uses unmatched dense-local).

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
config_hash=c1-dfab4a7ec19f17c2 seed=11400784225994701844 condition=local-assembly accuracy=1.000000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=623011.000000 note=wall_secs=0.0052_peak_rss=3670016_spikes=831_deliveries=13263_cells=13886_plasticity=595031
config_hash=c1-dfab4a7ec19f17c2 seed=11400784225994701844 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=4173556.000000 note=wall_secs=0.0105_peak_rss=4505600_spikes=925_deliveries=61680_cells=62317_plasticity=1961856
config_hash=c1-dfab4a7ec19f17c2 seed=11400784225994701844 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6554_peak_rss=2834432_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-dfab4a7ec19f17c2 seed=11400784225994701844 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0149_peak_rss=2621440_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-dfab4a7ec19f17c2 seed=11400784225994701844 condition=dense-matched accuracy=0.450000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1688348.933615 note=wall_secs=0.0061_peak_rss=4030464_spikes=901_deliveries=41155_cells=41792_plasticity=675909
config_hash=c1-dfab4a7ec19f17c2 seed=4354472946875824171 condition=local-assembly accuracy=0.625000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1088704.000000 note=wall_secs=0.0051_peak_rss=3735552_spikes=810_deliveries=13211_cells=13844_plasticity=652575
config_hash=c1-dfab4a7ec19f17c2 seed=4354472946875824171 condition=dense-local accuracy=0.550000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=3733112.646360 note=wall_secs=0.0103_peak_rss=4505600_spikes=897_deliveries=61680_cells=62315_plasticity=1928320
config_hash=c1-dfab4a7ec19f17c2 seed=4354472946875824171 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6629_peak_rss=2932736_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-dfab4a7ec19f17c2 seed=4354472946875824171 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0154_peak_rss=2621440_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-dfab4a7ec19f17c2 seed=4354472946875824171 condition=dense-matched accuracy=0.450000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1662164.488477 note=wall_secs=0.0062_peak_rss=4046848_spikes=892_deliveries=41161_cells=41796_plasticity=664125
config_hash=c1-dfab4a7ec19f17c2 seed=15755469790931547198 condition=local-assembly accuracy=0.600000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1143116.621243 note=wall_secs=0.0050_peak_rss=3604480_spikes=714_deliveries=13143_cells=13777_plasticity=658236
config_hash=c1-dfab4a7ec19f17c2 seed=15755469790931547198 condition=dense-local accuracy=0.450000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=4450913.451242 note=wall_secs=0.0106_peak_rss=4505600_spikes=903_deliveries=61680_cells=62312_plasticity=1878016
config_hash=c1-dfab4a7ec19f17c2 seed=15755469790931547198 condition=gradient-reference accuracy=0.725000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=157139856.901371 note=wall_secs=0.6578_peak_rss=2850816_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-dfab4a7ec19f17c2 seed=15755469790931547198 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0149_peak_rss=2621440_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-dfab4a7ec19f17c2 seed=15755469790931547198 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1460712.000000 note=wall_secs=0.0060_peak_rss=3997696_spikes=892_deliveries=41072_cells=41704_plasticity=646688
config_hash=c1-dfab4a7ec19f17c2 seed=8709160710835925077 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1469260.000000 note=wall_secs=0.0051_peak_rss=3735552_spikes=784_deliveries=13351_cells=13993_plasticity=706502
config_hash=c1-dfab4a7ec19f17c2 seed=8709160710835925077 condition=dense-local accuracy=0.550000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=3916081.733303 note=wall_secs=0.0105_peak_rss=4489216_spikes=916_deliveries=61680_cells=62321_plasticity=2028928
config_hash=c1-dfab4a7ec19f17c2 seed=8709160710835925077 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6635_peak_rss=3014656_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-dfab4a7ec19f17c2 seed=8709160710835925077 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0160_peak_rss=2621440_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-dfab4a7ec19f17c2 seed=8709160710835925077 condition=dense-matched accuracy=0.350000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2224774.323602 note=wall_secs=0.0058_peak_rss=4030464_spikes=901_deliveries=41105_cells=41745_plasticity=694920
config_hash=c1-dfab4a7ec19f17c2 seed=1663413756060003432 condition=local-assembly accuracy=0.650000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=996404.651933 note=wall_secs=0.0053_peak_rss=3768320_spikes=768_deliveries=13262_cells=13889_plasticity=619744
config_hash=c1-dfab4a7ec19f17c2 seed=1663413756060003432 condition=dense-local accuracy=0.550000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=3702612.647021 note=wall_secs=0.0110_peak_rss=4505600_spikes=891_deliveries=61680_cells=62314_plasticity=1911552
config_hash=c1-dfab4a7ec19f17c2 seed=1663413756060003432 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6595_peak_rss=2932736_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-dfab4a7ec19f17c2 seed=1663413756060003432 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0393_peak_rss=2654208_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-dfab4a7ec19f17c2 seed=1663413756060003432 condition=dense-matched accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1737239.951272 note=wall_secs=0.0063_peak_rss=4030464_spikes=890_deliveries=41154_cells=41787_plasticity=654496
config_hash=c1-dfab4a7ec19f17c2 seed=13063846550650677375 condition=local-assembly accuracy=0.525000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1329150.536551 note=wall_secs=0.0052_peak_rss=3735552_spikes=788_deliveries=13298_cells=13934_plasticity=669784
config_hash=c1-dfab4a7ec19f17c2 seed=13063846550650677375 condition=dense-local accuracy=0.550000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=4007558.094957 note=wall_secs=0.0108_peak_rss=4489216_spikes=921_deliveries=61680_cells=62324_plasticity=2079232
config_hash=c1-dfab4a7ec19f17c2 seed=13063846550650677375 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6543_peak_rss=2850816_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-dfab4a7ec19f17c2 seed=13063846550650677375 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0394_peak_rss=2621440_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-dfab4a7ec19f17c2 seed=13063846550650677375 condition=dense-matched accuracy=0.475000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1695631.600225 note=wall_secs=0.0059_peak_rss=4096000_spikes=916_deliveries=41057_cells=41702_plasticity=721750
config_hash=c1-dfab4a7ec19f17c2 seed=6018099320996848786 condition=local-assembly accuracy=1.000000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=657778.000000 note=wall_secs=0.0051_peak_rss=3670016_spikes=820_deliveries=13536_cells=14165_plasticity=629257
config_hash=c1-dfab4a7ec19f17c2 seed=6018099320996848786 condition=dense-local accuracy=0.575000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=3804139.209302 note=wall_secs=0.0112_peak_rss=4489216_spikes=913_deliveries=61680_cells=62323_plasticity=2062464
config_hash=c1-dfab4a7ec19f17c2 seed=6018099320996848786 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6589_peak_rss=2932736_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-dfab4a7ec19f17c2 seed=6018099320996848786 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0150_peak_rss=2621440_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-dfab4a7ec19f17c2 seed=6018099320996848786 condition=dense-matched accuracy=0.550000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1454219.968481 note=wall_secs=0.0065_peak_rss=4079616_spikes=901_deliveries=41212_cells=41856_plasticity=715852
config_hash=c1-dfab4a7ec19f17c2 seed=17418529916564267177 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1501648.000000 note=wall_secs=0.0050_peak_rss=3686400_spikes=767_deliveries=13456_cells=14101_plasticity=722500
config_hash=c1-dfab4a7ec19f17c2 seed=17418529916564267177 condition=dense-local accuracy=0.550000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=4129527.183222 note=wall_secs=0.0107_peak_rss=4489216_spikes=928_deliveries=61680_cells=62328_plasticity=2146304
config_hash=c1-dfab4a7ec19f17c2 seed=17418529916564267177 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6607_peak_rss=2932736_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-dfab4a7ec19f17c2 seed=17418529916564267177 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0149_peak_rss=2621440_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-dfab4a7ec19f17c2 seed=17418529916564267177 condition=dense-matched accuracy=0.600000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1372679.945455 note=wall_secs=0.0063_peak_rss=4046848_spikes=906_deliveries=41107_cells=41755_plasticity=739840
config_hash=c1-dfab4a7ec19f17c2 seed=10372782686910438588 condition=local-assembly accuracy=0.600000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1116338.288974 note=wall_secs=0.0051_peak_rss=3719168_spikes=789_deliveries=13346_cells=13977_plasticity=641691
config_hash=c1-dfab4a7ec19f17c2 seed=10372782686910438588 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=4106460.000000 note=wall_secs=0.0105_peak_rss=4489216_spikes=915_deliveries=61680_cells=62315_plasticity=1928320
config_hash=c1-dfab4a7ec19f17c2 seed=10372782686910438588 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6557_peak_rss=2932736_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-dfab4a7ec19f17c2 seed=10372782686910438588 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0151_peak_rss=2621440_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-dfab4a7ec19f17c2 seed=10372782686910438588 condition=dense-matched accuracy=0.575000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1311761.766326 note=wall_secs=0.0061_peak_rss=4079616_spikes=913_deliveries=41059_cells=41695_plasticity=670596
config_hash=c1-dfab4a7ec19f17c2 seed=3326471682669467859 condition=local-assembly accuracy=1.000000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=698621.000000 note=wall_secs=0.0051_peak_rss=3735552_spikes=845_deliveries=13446_cells=14082_plasticity=670248
config_hash=c1-dfab4a7ec19f17c2 seed=3326471682669467859 condition=dense-local accuracy=0.450000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=4860842.350991 note=wall_secs=0.0107_peak_rss=4505600_spikes=912_deliveries=61680_cells=62323_plasticity=2062464
config_hash=c1-dfab4a7ec19f17c2 seed=3326471682669467859 condition=gradient-reference accuracy=0.725000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=157139856.901371 note=wall_secs=0.6522_peak_rss=2834432_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-dfab4a7ec19f17c2 seed=3326471682669467859 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0150_peak_rss=2621440_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-dfab4a7ec19f17c2 seed=3326471682669467859 condition=dense-matched accuracy=0.400000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1986082.470405 note=wall_secs=0.0060_peak_rss=4079616_spikes=910_deliveries=41093_cells=41736_plasticity=710694
config_hash=c1-dfab4a7ec19f17c2 seed=14727610363725173990 condition=local-assembly accuracy=0.150000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5234386.458671 note=wall_secs=0.0053_peak_rss=3637248_spikes=682_deliveries=13257_cells=13908_plasticity=757311
config_hash=c1-dfab4a7ec19f17c2 seed=14727610363725173990 condition=dense-local accuracy=0.400000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5510417.417888 note=wall_secs=0.0106_peak_rss=4407296_spikes=931_deliveries=61680_cells=62324_plasticity=2079232
config_hash=c1-dfab4a7ec19f17c2 seed=14727610363725173990 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6568_peak_rss=2850816_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-dfab4a7ec19f17c2 seed=14727610363725173990 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0400_peak_rss=2621440_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-dfab4a7ec19f17c2 seed=14727610363725173990 condition=dense-matched accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1882597.594253 note=wall_secs=0.0058_peak_rss=4063232_spikes=904_deliveries=40856_cells=41500_plasticity=716844
config_hash=c1-dfab4a7ec19f17c2 seed=7681300184117924093 condition=local-assembly accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=991922.726001 note=wall_secs=0.0053_peak_rss=3735552_spikes=778_deliveries=13203_cells=13843_plasticity=691320
config_hash=c1-dfab4a7ec19f17c2 seed=7681300184117924093 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=4341234.000000 note=wall_secs=0.0107_peak_rss=4407296_spikes=919_deliveries=61680_cells=62322_plasticity=2045696
config_hash=c1-dfab4a7ec19f17c2 seed=7681300184117924093 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6560_peak_rss=2916352_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-dfab4a7ec19f17c2 seed=7681300184117924093 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0155_peak_rss=2621440_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-dfab4a7ec19f17c2 seed=7681300184117924093 condition=dense-matched accuracy=0.400000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1952124.970911 note=wall_secs=0.0059_peak_rss=4030464_spikes=920_deliveries=41104_cells=41745_plasticity=697081
config_hash=c1-dfab4a7ec19f17c2 seed=635551854952467728 condition=local-assembly accuracy=0.525000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1328676.250815 note=wall_secs=0.0051_peak_rss=3719168_spikes=765_deliveries=13359_cells=13995_plasticity=669436
config_hash=c1-dfab4a7ec19f17c2 seed=635551854952467728 condition=dense-local accuracy=0.575000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=3599958.335504 note=wall_secs=0.0106_peak_rss=4489216_spikes=892_deliveries=61680_cells=62316_plasticity=1945088
config_hash=c1-dfab4a7ec19f17c2 seed=635551854952467728 condition=gradient-reference accuracy=0.500000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=227852800.000000 note=wall_secs=0.6626_peak_rss=2932736_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-dfab4a7ec19f17c2 seed=635551854952467728 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0153_peak_rss=2621440_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-dfab4a7ec19f17c2 seed=635551854952467728 condition=dense-matched accuracy=0.575000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1309316.548884 note=wall_secs=0.0063_peak_rss=3997696_spikes=885_deliveries=40950_cells=41586_plasticity=669436
config_hash=c1-dfab4a7ec19f17c2 seed=12035985749054769447 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1297408.000000 note=wall_secs=0.0050_peak_rss=3768320_spikes=865_deliveries=13520_cells=14147_plasticity=620172
config_hash=c1-dfab4a7ec19f17c2 seed=12035985749054769447 condition=dense-local accuracy=0.550000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=3794099.917765 note=wall_secs=0.0105_peak_rss=4489216_spikes=902_deliveries=61680_cells=62317_plasticity=1961856
config_hash=c1-dfab4a7ec19f17c2 seed=12035985749054769447 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6556_peak_rss=2916352_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-dfab4a7ec19f17c2 seed=12035985749054769447 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0178_peak_rss=2637824_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-dfab4a7ec19f17c2 seed=12035985749054769447 condition=dense-matched accuracy=0.475000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1579347.388239 note=wall_secs=0.0062_peak_rss=4014080_spikes=891_deliveries=41062_cells=41697_plasticity=666540
config_hash=c1-dfab4a7ec19f17c2 seed=4990235495743964474 condition=local-assembly accuracy=1.000000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=757037.000000 note=wall_secs=0.0052_peak_rss=3735552_spikes=777_deliveries=13415_cells=14061_plasticity=728784
config_hash=c1-dfab4a7ec19f17c2 seed=4990235495743964474 condition=dense-local accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5067830.446086 note=wall_secs=0.0108_peak_rss=4407296_spikes=899_deliveries=61680_cells=62321_plasticity=2028928
config_hash=c1-dfab4a7ec19f17c2 seed=4990235495743964474 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6544_peak_rss=2850816_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-dfab4a7ec19f17c2 seed=4990235495743964474 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0157_peak_rss=2654208_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-dfab4a7ec19f17c2 seed=4990235495743964474 condition=dense-matched accuracy=0.575000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1362867.854342 note=wall_secs=0.0059_peak_rss=4096000_spikes=894_deliveries=41125_cells=41766_plasticity=699864
config_hash=c1-dfab4a7ec19f17c2 seed=16390669389846266193 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1456112.000000 note=wall_secs=0.0052_peak_rss=3751936_spikes=765_deliveries=13393_cells=14034_plasticity=699864
config_hash=c1-dfab4a7ec19f17c2 seed=16390669389846266193 condition=dense-local accuracy=0.525000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=4134489.711569 note=wall_secs=0.0109_peak_rss=4489216_spikes=909_deliveries=61680_cells=62322_plasticity=2045696
config_hash=c1-dfab4a7ec19f17c2 seed=16390669389846266193 condition=gradient-reference accuracy=0.500000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=227852800.000000 note=wall_secs=0.6592_peak_rss=2916352_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-dfab4a7ec19f17c2 seed=16390669389846266193 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0395_peak_rss=2621440_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-dfab4a7ec19f17c2 seed=16390669389846266193 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1578902.000000 note=wall_secs=0.0060_peak_rss=4096000_spikes=899_deliveries=41131_cells=41773_plasticity=705648
config_hash=c1-dfab4a7ec19f17c2 seed=9344921060680809828 condition=local-assembly accuracy=0.675000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1016752.574636 note=wall_secs=0.0051_peak_rss=3751936_spikes=812_deliveries=13313_cells=13947_plasticity=658236
config_hash=c1-dfab4a7ec19f17c2 seed=9344921060680809828 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=4106478.000000 note=wall_secs=0.0105_peak_rss=4489216_spikes=924_deliveries=61680_cells=62315_plasticity=1928320
config_hash=c1-dfab4a7ec19f17c2 seed=9344921060680809828 condition=gradient-reference accuracy=0.725000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=157139856.901371 note=wall_secs=0.6578_peak_rss=2916352_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-dfab4a7ec19f17c2 seed=9344921060680809828 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0150_peak_rss=2621440_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-dfab4a7ec19f17c2 seed=9344921060680809828 condition=dense-matched accuracy=0.575000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1300526.983484 note=wall_secs=0.0059_peak_rss=4063232_spikes=914_deliveries=41122_cells=41757_plasticity=664010
config_hash=c1-dfab4a7ec19f17c2 seed=2298610881073559931 condition=local-assembly accuracy=0.900000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=769362.242603 note=wall_secs=0.0052_peak_rss=3702784_spikes=839_deliveries=13241_cells=13876_plasticity=664470
config_hash=c1-dfab4a7ec19f17c2 seed=2298610881073559931 condition=dense-local accuracy=0.550000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=3916056.278758 note=wall_secs=0.0107_peak_rss=4489216_spikes=902_deliveries=61680_cells=62321_plasticity=2028928
config_hash=c1-dfab4a7ec19f17c2 seed=2298610881073559931 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6616_peak_rss=2916352_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-dfab4a7ec19f17c2 seed=2298610881073559931 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0163_peak_rss=2621440_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-dfab4a7ec19f17c2 seed=2298610881073559931 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1565666.000000 note=wall_secs=0.0059_peak_rss=4046848_spikes=902_deliveries=41076_cells=41717_plasticity=699138
config_hash=c1-dfab4a7ec19f17c2 seed=13699608824640910734 condition=local-assembly accuracy=1.000000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=628812.000000 note=wall_secs=0.0049_peak_rss=3735552_spikes=784_deliveries=13142_cells=13766_plasticity=601120
config_hash=c1-dfab4a7ec19f17c2 seed=13699608824640910734 condition=dense-local accuracy=0.375000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5609384.000000 note=wall_secs=0.0105_peak_rss=4489216_spikes=897_deliveries=61680_cells=62318_plasticity=1978624
config_hash=c1-dfab4a7ec19f17c2 seed=13699608824640910734 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6542_peak_rss=2850816_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-dfab4a7ec19f17c2 seed=13699608824640910734 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0158_peak_rss=2621440_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-dfab4a7ec19f17c2 seed=13699608824640910734 condition=dense-matched accuracy=0.575000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1331768.723262 note=wall_secs=0.0059_peak_rss=4063232_spikes=893_deliveries=41098_cells=41736_plasticity=682040
config_hash=c1-dfab4a7ec19f17c2 seed=6653297820399940005 condition=local-assembly accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1765644.656357 note=wall_secs=0.0082_peak_rss=3670016_spikes=745_deliveries=13192_cells=13837_plasticity=722625
config_hash=c1-dfab4a7ec19f17c2 seed=6653297820399940005 condition=dense-local accuracy=0.550000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=4068483.548182 note=wall_secs=0.0173_peak_rss=4833280_spikes=892_deliveries=61680_cells=62326_plasticity=2112768
config_hash=c1-dfab4a7ec19f17c2 seed=6653297820399940005 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6909_peak_rss=3014656_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-dfab4a7ec19f17c2 seed=6653297820399940005 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0160_peak_rss=2654208_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-dfab4a7ec19f17c2 seed=6653297820399940005 condition=dense-matched accuracy=0.600000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1343998.279928 note=wall_secs=0.0091_peak_rss=4096000_spikes=905_deliveries=41112_cells=41757_plasticity=722625
```
