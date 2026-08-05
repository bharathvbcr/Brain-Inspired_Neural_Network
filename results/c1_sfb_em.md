# C1 / Gate G2 results note

**Config hash:** `c1-677df7f7cbe4f8ec`

**Scientific protocol version:** `16`

**Structured B × epoch-matched protocol:** `16` — v15 structured hidden `B` plus **20** local/dense epochs over the frozen train split (isolates single-pass handicap under aligned feedback); **positive control stays on broadcast ±1**; does **not** remassage v14/v15 hashes or reopen protocol-v2 `c1-118207fbc3eaba53`.

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
Config { experiment: "c1-sfb-em", master_seed: 212618061021185, n_seeds: 20, sequence_len: 8, max_lag: 1, n_hidden: 128, k_wta: 2, p_sparse: 0.35, init_w: 0.15, eta: 0.35, lambda: 0.002, tau_e: 40.0, n_train: 80, n_test: 40, bptt_epochs: 80, bptt_lr: 0.05, g2_min_gap_closed: 0.5, g2_min_accuracy: 0.65, g2_confidence_z: 1.96, g2_min_positive_control: 0.9, g2_min_reference_gap: 0.15, activity_sparsity_min: 0.005, activity_sparsity_max: 0.03, scientific_n_seeds: 20, power_sigma_prior: 0.15, power_effect_size: 0.1, use_surrogate_lif_reference: true, surrogate_beta: 5.0, matched_budget_repeat: true, quick: false }
```

Power analysis: required scientific n_seeds ≥ 20 (preregistered σ=0.150, effect=0.100; formula n=⌈(1.96+0.8416)²σ²/δ²⌉).

## Per-seed accuracies

| seed | local-assembly | dense-local | gradient reference | eligibility reference | activity_sparsity (local) | activity_sparsity (dense) | dense_matched |
|---|---:|---:|---:|---:|---:|---:|---:|
| 11400784225994701844 | 0.5000 | 0.5750 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.5750 |
| 4354472946875824171 | 0.0750 | 0.8250 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 15755469790931547198 | 0.2750 | 0.5000 | 0.7250 | 1.0000 | 0.0156 | 0.0156 | 0.3750 |
| 8709160710835925077 | 0.5000 | 0.3000 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.4250 |
| 1663413756060003432 | 0.8500 | 0.4750 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.4250 |
| 13063846550650677375 | 0.7250 | 0.2250 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.6250 |
| 6018099320996848786 | 0.5000 | 0.6250 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.7750 |
| 17418529916564267177 | 0.5000 | 0.7750 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.5250 |
| 10372782686910438588 | 0.5000 | 0.8000 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.4500 |
| 3326471682669467859 | 0.7750 | 0.5500 | 0.7250 | 1.0000 | 0.0156 | 0.0156 | 0.7500 |
| 14727610363725173990 | 0.5000 | 0.4250 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.3750 |
| 7681300184117924093 | 0.7250 | 0.4500 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.8250 |
| 635551854952467728 | 0.7250 | 0.7250 | 0.5000 | 1.0000 | 0.0156 | 0.0156 | 0.5750 |
| 12035985749054769447 | 0.5000 | 0.4750 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.3500 |
| 4990235495743964474 | 0.3000 | 0.3750 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.4750 |
| 16390669389846266193 | 0.5000 | 0.7500 | 0.5000 | 1.0000 | 0.0156 | 0.0156 | 0.4000 |
| 9344921060680809828 | 0.5500 | 0.4000 | 0.7250 | 1.0000 | 0.0156 | 0.0156 | 0.5250 |
| 2298610881073559931 | 0.5000 | 0.0250 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.7500 |
| 13699608824640910734 | 0.5000 | 0.4750 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.7250 |
| 6653297820399940005 | 0.4000 | 0.4750 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.2250 |

## Summary (paired normalized-gap analysis)

- mean ± var local-assembly: 0.5200 ± 0.032934
- mean ± var dense-local:    0.5113 ± 0.041215
- mean ± var gradient reference: 0.8938 ± 0.027163
- mean ± var eligibility reference: 1.0000 ± 0.000000
- mean normalized gap closed: 0.2231 (variance 0.100205)
- lower confidence bound (z=1.960, n=20): 0.0844
- mean |local − dense| (descriptive): 0.2238

## U-NEG

Negative result: local-assembly did **not** clear the preregistered normalized-gap confidence and absolute-accuracy gates. Program stops at G2; do not schedule P3+.

## Positive / sanity control

Mean local-pipeline accuracy on a trivially separable spatial feature-presence task: **0.9488** (threshold 0.900).

## Activity sparsity

Mean local-assembly activity_sparsity: **0.0156** (valid band [0.0050, 0.0300]; nominal k/N=0.0156).

## Parameter / compute budgets

| condition | n_cells | n_params | wall_secs | peak_rss_bytes | work_per_accuracy | spikes | deliveries | cell_updates | plasticity |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| local-assembly | 132 | 5777 | 0.0554 | 4161536 | 19244428.0000 | 10244 | 180010 | 188760 | 9243200 |
| dense-local | 132 | 16768 | 0.1440 | 4931584 | 49628258.4202 | 12550 | 842960 | 851938 | 26828800 |
| gradient-reference | 130 | 16769 | 0.6856 | 3031040 | 113926400.0000 | 0 | 51200 | 6553600 | 107321600 |
| eligibility-reference | 130 | 385 | 0.0238 | 2621440 | 9068800.0000 | 0 | 51200 | 6553600 | 2464000 |
| dense-matched | 132 | 5777 | 0.0694 | 4325376 | 18064974.2876 | 12501 | 561336 | 570323 | 9243200 |

Matched-budget dense mean accuracy: **0.5325** (n=20; primary G2 gap still uses unmatched dense-local).

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
config_hash=c1-677df7f7cbe4f8ec seed=11400784225994701844 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=19244428.000000 note=wall_secs=0.0554_peak_rss=4161536_spikes=10244_deliveries=180010_cells=188760_plasticity=9243200
config_hash=c1-677df7f7cbe4f8ec seed=11400784225994701844 condition=dense-local accuracy=0.575000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=49628258.420200 note=wall_secs=0.1440_peak_rss=4931584_spikes=12550_deliveries=842960_cells=851938_plasticity=26828800
config_hash=c1-677df7f7cbe4f8ec seed=11400784225994701844 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6856_peak_rss=3031040_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-677df7f7cbe4f8ec seed=11400784225994701844 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0238_peak_rss=2621440_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-677df7f7cbe4f8ec seed=11400784225994701844 condition=dense-matched accuracy=0.575000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=18064974.287567 note=wall_secs=0.0694_peak_rss=4325376_spikes=12501_deliveries=561336_cells=570323_plasticity=9243200
config_hash=c1-677df7f7cbe4f8ec seed=4354472946875824171 condition=local-assembly accuracy=0.075000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=128250114.903798 note=wall_secs=0.0613_peak_rss=4112384_spikes=9460_deliveries=180121_cells=189178_plasticity=9240000
config_hash=c1-677df7f7cbe4f8ec seed=4354472946875824171 condition=dense-local accuracy=0.825000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=34589159.287679 note=wall_secs=0.1466_peak_rss=4849664_spikes=12379_deliveries=842960_cells=851917_plasticity=26828800
config_hash=c1-677df7f7cbe4f8ec seed=4354472946875824171 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6720_peak_rss=2932736_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-677df7f7cbe4f8ec seed=4354472946875824171 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0178_peak_rss=2621440_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-677df7f7cbe4f8ec seed=4354472946875824171 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=20768594.000000 note=wall_secs=0.0694_peak_rss=4308992_spikes=12431_deliveries=561419_cells=570447_plasticity=9240000
config_hash=c1-677df7f7cbe4f8ec seed=15755469790931547198 condition=local-assembly accuracy=0.275000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=34972737.423805 note=wall_secs=0.0538_peak_rss=4276224_spikes=9296_deliveries=180294_cells=189513_plasticity=9238400
config_hash=c1-677df7f7cbe4f8ec seed=15755469790931547198 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=57072460.000000 note=wall_secs=0.1439_peak_rss=4866048_spikes=12521_deliveries=842960_cells=851949_plasticity=26828800
config_hash=c1-677df7f7cbe4f8ec seed=15755469790931547198 condition=gradient-reference accuracy=0.725000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=157139856.901371 note=wall_secs=0.6829_peak_rss=2998272_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-677df7f7cbe4f8ec seed=15755469790931547198 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0159_peak_rss=2637824_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-677df7f7cbe4f8ec seed=15755469790931547198 condition=dense-matched accuracy=0.375000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=27686258.666667 note=wall_secs=0.0698_peak_rss=4538368_spikes=12513_deliveries=561196_cells=570238_plasticity=9238400
config_hash=c1-677df7f7cbe4f8ec seed=8709160710835925077 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=19300326.000000 note=wall_secs=0.0542_peak_rss=4341760_spikes=11409_deliveries=182084_cells=191070_plasticity=9265600
config_hash=c1-677df7f7cbe4f8ec seed=8709160710835925077 condition=dense-local accuracy=0.300000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=95121639.553539 note=wall_secs=0.1443_peak_rss=4964352_spikes=12782_deliveries=842960_cells=851951_plasticity=26828800
config_hash=c1-677df7f7cbe4f8ec seed=8709160710835925077 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6763_peak_rss=3014656_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-677df7f7cbe4f8ec seed=8709160710835925077 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0407_peak_rss=2621440_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-677df7f7cbe4f8ec seed=8709160710835925077 condition=dense-matched accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=24496606.371712 note=wall_secs=0.0699_peak_rss=4390912_spikes=12663_deliveries=561894_cells=570901_plasticity=9265600
config_hash=c1-677df7f7cbe4f8ec seed=1663413756060003432 condition=local-assembly accuracy=0.850000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=11353822.034475 note=wall_secs=0.0542_peak_rss=4194304_spikes=10874_deliveries=182021_cells=190654_plasticity=9267200
config_hash=c1-677df7f7cbe4f8ec seed=1663413756060003432 condition=dense-local accuracy=0.475000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=60076348.122280 note=wall_secs=0.1446_peak_rss=5160960_spikes=12575_deliveries=842960_cells=851930_plasticity=26828800
config_hash=c1-677df7f7cbe4f8ec seed=1663413756060003432 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6689_peak_rss=2916352_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-677df7f7cbe4f8ec seed=1663413756060003432 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0151_peak_rss=2605056_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-677df7f7cbe4f8ec seed=1663413756060003432 condition=dense-matched accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=24498020.489320 note=wall_secs=0.0692_peak_rss=4292608_spikes=12397_deliveries=561565_cells=570497_plasticity=9267200
config_hash=c1-677df7f7cbe4f8ec seed=13063846550650677375 condition=local-assembly accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=13272483.701461 note=wall_secs=0.0535_peak_rss=4177920_spikes=11540_deliveries=182042_cells=190569_plasticity=9238400
config_hash=c1-677df7f7cbe4f8ec seed=13063846550650677375 condition=dense-local accuracy=0.225000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=126828607.804255 note=wall_secs=0.1451_peak_rss=4915200_spikes=12741_deliveries=842960_cells=851935_plasticity=26828800
config_hash=c1-677df7f7cbe4f8ec seed=13063846550650677375 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6632_peak_rss=2932736_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-677df7f7cbe4f8ec seed=13063846550650677375 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0150_peak_rss=2605056_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-677df7f7cbe4f8ec seed=13063846550650677375 condition=dense-matched accuracy=0.625000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=16612350.400000 note=wall_secs=0.0688_peak_rss=4341760_spikes=12550_deliveries=561421_cells=570348_plasticity=9238400
config_hash=c1-677df7f7cbe4f8ec seed=6018099320996848786 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=19246378.000000 note=wall_secs=0.0537_peak_rss=4145152_spikes=9709_deliveries=183971_cells=192709_plasticity=9236800
config_hash=c1-677df7f7cbe4f8ec seed=6018099320996848786 condition=dense-local accuracy=0.625000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=45658025.600000 note=wall_secs=0.1466_peak_rss=5193728_spikes=12588_deliveries=842960_cells=851918_plasticity=26828800
config_hash=c1-677df7f7cbe4f8ec seed=6018099320996848786 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6741_peak_rss=2883584_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-677df7f7cbe4f8ec seed=6018099320996848786 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0154_peak_rss=2637824_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-677df7f7cbe4f8ec seed=6018099320996848786 condition=dense-matched accuracy=0.775000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=13394161.702376 note=wall_secs=0.0695_peak_rss=4325376_spikes=12469_deliveries=561154_cells=570052_plasticity=9236800
config_hash=c1-677df7f7cbe4f8ec seed=17418529916564267177 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=19271202.000000 note=wall_secs=0.0544_peak_rss=4308992_spikes=11113_deliveries=183705_cells=192783_plasticity=9248000
config_hash=c1-677df7f7cbe4f8ec seed=17418529916564267177 condition=dense-local accuracy=0.775000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=36820607.584351 note=wall_secs=0.1451_peak_rss=5013504_spikes=12234_deliveries=842960_cells=851976_plasticity=26828800
config_hash=c1-677df7f7cbe4f8ec seed=17418529916564267177 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6687_peak_rss=2883584_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-677df7f7cbe4f8ec seed=17418529916564267177 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0399_peak_rss=2637824_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-677df7f7cbe4f8ec seed=17418529916564267177 condition=dense-matched accuracy=0.525000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=19794793.279895 note=wall_secs=0.0707_peak_rss=4489216_spikes=12448_deliveries=561398_cells=570420_plasticity=9248000
config_hash=c1-677df7f7cbe4f8ec seed=10372782686910438588 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=19268528.000000 note=wall_secs=0.0543_peak_rss=4177920_spikes=10677_deliveries=182554_cells=191433_plasticity=9249600
config_hash=c1-677df7f7cbe4f8ec seed=10372782686910438588 condition=dense-local accuracy=0.800000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=35670239.468472 note=wall_secs=0.1458_peak_rss=4915200_spikes=12469_deliveries=842960_cells=851963_plasticity=26828800
config_hash=c1-677df7f7cbe4f8ec seed=10372782686910438588 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6701_peak_rss=2883584_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-677df7f7cbe4f8ec seed=10372782686910438588 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0401_peak_rss=2621440_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-677df7f7cbe4f8ec seed=10372782686910438588 condition=dense-matched accuracy=0.450000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=23098062.834112 note=wall_secs=0.0713_peak_rss=4358144_spikes=12537_deliveries=561475_cells=570516_plasticity=9249600
config_hash=c1-677df7f7cbe4f8ec seed=3326471682669467859 condition=local-assembly accuracy=0.775000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=12426689.414549 note=wall_secs=0.0528_peak_rss=4128768_spikes=9917_deliveries=183662_cells=192305_plasticity=9244800
config_hash=c1-677df7f7cbe4f8ec seed=3326471682669467859 condition=dense-local accuracy=0.550000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=51884082.511806 note=wall_secs=0.1440_peak_rss=4898816_spikes=12527_deliveries=842960_cells=851959_plasticity=26828800
config_hash=c1-677df7f7cbe4f8ec seed=3326471682669467859 condition=gradient-reference accuracy=0.725000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=157139856.901371 note=wall_secs=0.6704_peak_rss=2932736_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-677df7f7cbe4f8ec seed=3326471682669467859 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0397_peak_rss=2621440_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-677df7f7cbe4f8ec seed=3326471682669467859 condition=dense-matched accuracy=0.750000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=13851920.000000 note=wall_secs=0.0700_peak_rss=4341760_spikes=12420_deliveries=561380_cells=570340_plasticity=9244800
config_hash=c1-677df7f7cbe4f8ec seed=14727610363725173990 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=19267608.000000 note=wall_secs=0.0536_peak_rss=4358144_spikes=11350_deliveries=181920_cells=190934_plasticity=9249600
config_hash=c1-677df7f7cbe4f8ec seed=14727610363725173990 condition=dense-local accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=67143993.410779 note=wall_secs=0.1433_peak_rss=5079040_spikes=12498_deliveries=842960_cells=851940_plasticity=26828800
config_hash=c1-677df7f7cbe4f8ec seed=14727610363725173990 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6763_peak_rss=2867200_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-677df7f7cbe4f8ec seed=14727610363725173990 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0404_peak_rss=2637824_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-677df7f7cbe4f8ec seed=14727610363725173990 condition=dense-matched accuracy=0.375000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=27717448.000000 note=wall_secs=0.0701_peak_rss=4325376_spikes=12511_deliveries=561467_cells=570465_plasticity=9249600
config_hash=c1-677df7f7cbe4f8ec seed=7681300184117924093 condition=local-assembly accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=13238014.737077 note=wall_secs=0.0523_peak_rss=4177920_spikes=11173_deliveries=180133_cells=188655_plasticity=9217600
config_hash=c1-677df7f7cbe4f8ec seed=7681300184117924093 condition=dense-local accuracy=0.450000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=63414006.124342 note=wall_secs=0.1432_peak_rss=4849664_spikes=12626_deliveries=842960_cells=851916_plasticity=26828800
config_hash=c1-677df7f7cbe4f8ec seed=7681300184117924093 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.7039_peak_rss=2998272_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-677df7f7cbe4f8ec seed=7681300184117924093 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0401_peak_rss=2637824_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-677df7f7cbe4f8ec seed=7681300184117924093 condition=dense-matched accuracy=0.825000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=12558872.908744 note=wall_secs=0.0685_peak_rss=4292608_spikes=12537_deliveries=561011_cells=569922_plasticity=9217600
config_hash=c1-677df7f7cbe4f8ec seed=635551854952467728 condition=local-assembly accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=13269951.287751 note=wall_secs=0.0533_peak_rss=4161536_spikes=11297_deliveries=183625_cells=192193_plasticity=9233600
config_hash=c1-677df7f7cbe4f8ec seed=635551854952467728 condition=dense-local accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=39360357.326312 note=wall_secs=0.1438_peak_rss=4997120_spikes=12558_deliveries=842960_cells=851942_plasticity=26828800
config_hash=c1-677df7f7cbe4f8ec seed=635551854952467728 condition=gradient-reference accuracy=0.500000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=227852800.000000 note=wall_secs=0.6754_peak_rss=2998272_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-677df7f7cbe4f8ec seed=635551854952467728 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0401_peak_rss=2621440_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-677df7f7cbe4f8ec seed=635551854952467728 condition=dense-matched accuracy=0.575000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=18047501.243727 note=wall_secs=0.0692_peak_rss=4440064_spikes=12416_deliveries=561188_cells=570109_plasticity=9233600
config_hash=c1-677df7f7cbe4f8ec seed=12035985749054769447 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=19321356.000000 note=wall_secs=0.0542_peak_rss=4145152_spikes=9974_deliveries=184097_cells=193007_plasticity=9273600
config_hash=c1-677df7f7cbe4f8ec seed=12035985749054769447 condition=dense-local accuracy=0.475000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=60076318.648595 note=wall_secs=0.1444_peak_rss=5062656_spikes=12520_deliveries=842960_cells=851971_plasticity=26828800
config_hash=c1-677df7f7cbe4f8ec seed=12035985749054769447 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6724_peak_rss=2899968_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-677df7f7cbe4f8ec seed=12035985749054769447 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0177_peak_rss=2621440_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-677df7f7cbe4f8ec seed=12035985749054769447 condition=dense-matched accuracy=0.350000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=29767046.221215 note=wall_secs=0.0695_peak_rss=4538368_spikes=12591_deliveries=561626_cells=570649_plasticity=9273600
config_hash=c1-677df7f7cbe4f8ec seed=4990235495743964474 condition=local-assembly accuracy=0.300000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=32137802.056292 note=wall_secs=0.0559_peak_rss=4210688_spikes=10043_deliveries=183796_cells=193102_plasticity=9254400
config_hash=c1-677df7f7cbe4f8ec seed=4990235495743964474 condition=dense-local accuracy=0.375000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=76096672.000000 note=wall_secs=0.1440_peak_rss=4915200_spikes=12504_deliveries=842960_cells=851988_plasticity=26828800
config_hash=c1-677df7f7cbe4f8ec seed=4990235495743964474 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6686_peak_rss=2883584_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-677df7f7cbe4f8ec seed=4990235495743964474 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0178_peak_rss=2637824_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-677df7f7cbe4f8ec seed=4990235495743964474 condition=dense-matched accuracy=0.475000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=21892583.432610 note=wall_secs=0.0690_peak_rss=4423680_spikes=12487_deliveries=561524_cells=570566_plasticity=9254400
config_hash=c1-677df7f7cbe4f8ec seed=16390669389846266193 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=19278126.000000 note=wall_secs=0.0558_peak_rss=4358144_spikes=11065_deliveries=182263_cells=191335_plasticity=9254400
config_hash=c1-677df7f7cbe4f8ec seed=16390669389846266193 condition=dense-local accuracy=0.750000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=38048261.333333 note=wall_secs=0.1437_peak_rss=4931584_spikes=12487_deliveries=842960_cells=851949_plasticity=26828800
config_hash=c1-677df7f7cbe4f8ec seed=16390669389846266193 condition=gradient-reference accuracy=0.500000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=227852800.000000 note=wall_secs=0.6628_peak_rss=2850816_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-677df7f7cbe4f8ec seed=16390669389846266193 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0176_peak_rss=2621440_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-677df7f7cbe4f8ec seed=16390669389846266193 condition=dense-matched accuracy=0.400000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=25997569.612606 note=wall_secs=0.0696_peak_rss=4390912_spikes=12461_deliveries=561602_cells=570565_plasticity=9254400
config_hash=c1-677df7f7cbe4f8ec seed=9344921060680809828 condition=local-assembly accuracy=0.550000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=17491850.529965 note=wall_secs=0.0539_peak_rss=4210688_spikes=10156_deliveries=181594_cells=190368_plasticity=9238400
config_hash=c1-677df7f7cbe4f8ec seed=9344921060680809828 condition=dense-local accuracy=0.400000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=71341003.936936 note=wall_secs=0.1443_peak_rss=4898816_spikes=12675_deliveries=842960_cells=851967_plasticity=26828800
config_hash=c1-677df7f7cbe4f8ec seed=9344921060680809828 condition=gradient-reference accuracy=0.725000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=157139856.901371 note=wall_secs=0.6702_peak_rss=2850816_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-677df7f7cbe4f8ec seed=9344921060680809828 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0400_peak_rss=2637824_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-677df7f7cbe4f8ec seed=9344921060680809828 condition=dense-matched accuracy=0.525000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=19775126.612335 note=wall_secs=0.0692_peak_rss=4292608_spikes=12554_deliveries=561022_cells=569965_plasticity=9238400
config_hash=c1-677df7f7cbe4f8ec seed=2298610881073559931 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=19250888.000000 note=wall_secs=0.0535_peak_rss=4177920_spikes=10791_deliveries=180434_cells=189419_plasticity=9244800
config_hash=c1-677df7f7cbe4f8ec seed=2298610881073559931 condition=dense-local accuracy=0.025000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1141448942.991085 note=wall_secs=0.1453_peak_rss=5177344_spikes=12529_deliveries=842960_cells=851935_plasticity=26828800
config_hash=c1-677df7f7cbe4f8ec seed=2298610881073559931 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6719_peak_rss=2916352_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-677df7f7cbe4f8ec seed=2298610881073559931 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0179_peak_rss=2621440_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-677df7f7cbe4f8ec seed=2298610881073559931 condition=dense-matched accuracy=0.750000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=13851406.666667 note=wall_secs=0.0695_peak_rss=4374528_spikes=12306_deliveries=561273_cells=570176_plasticity=9244800
config_hash=c1-677df7f7cbe4f8ec seed=13699608824640910734 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=19249884.000000 note=wall_secs=0.0538_peak_rss=4161536_spikes=10097_deliveries=178997_cells=187848_plasticity=9248000
config_hash=c1-677df7f7cbe4f8ec seed=13699608824640910734 condition=dense-local accuracy=0.475000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=60076272.332805 note=wall_secs=0.1443_peak_rss=5193728_spikes=12566_deliveries=842960_cells=851903_plasticity=26828800
config_hash=c1-677df7f7cbe4f8ec seed=13699608824640910734 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6649_peak_rss=2916352_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-677df7f7cbe4f8ec seed=13699608824640910734 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0400_peak_rss=2621440_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-677df7f7cbe4f8ec seed=13699608824640910734 condition=dense-matched accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=14333958.149313 note=wall_secs=0.0695_peak_rss=4554752_spikes=12407_deliveries=561394_cells=570319_plasticity=9248000
config_hash=c1-677df7f7cbe4f8ec seed=6653297820399940005 condition=local-assembly accuracy=0.400000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=24074404.641263 note=wall_secs=0.0557_peak_rss=4210688_spikes=9660_deliveries=180689_cells=189813_plasticity=9249600
config_hash=c1-677df7f7cbe4f8ec seed=6653297820399940005 condition=dense-local accuracy=0.475000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=60076341.806490 note=wall_secs=0.1462_peak_rss=4997120_spikes=12489_deliveries=842960_cells=852013_plasticity=26828800
config_hash=c1-677df7f7cbe4f8ec seed=6653297820399940005 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6658_peak_rss=2998272_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-677df7f7cbe4f8ec seed=6653297820399940005 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0177_peak_rss=2621440_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-677df7f7cbe4f8ec seed=6653297820399940005 condition=dense-matched accuracy=0.225000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=46195961.223775 note=wall_secs=0.0711_peak_rss=4440064_spikes=12525_deliveries=561445_cells=570521_plasticity=9249600
```
