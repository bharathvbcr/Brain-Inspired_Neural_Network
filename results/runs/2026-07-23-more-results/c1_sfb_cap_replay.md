# C1 / Gate G2 results note

**Config hash:** `c1-983ee5303c00b147`

**Scientific protocol version:** `17`

**Structured B × capacity protocol:** `17` — v15 structured hidden `B` on the Tier-B capacity substrate (richer `k_wta` / `n_hidden` / `n_train`); single-pass; **positive control stays on broadcast ±1**; does **not** remassage v15 or capacity-only `c1-d38d7644d8afc84b` or reopen protocol-v2 `c1-118207fbc3eaba53`.

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
Config { experiment: "c1-sfb-cap", master_seed: 212618061021185, n_seeds: 20, sequence_len: 8, max_lag: 1, n_hidden: 256, k_wta: 4, p_sparse: 0.3, init_w: 0.15, eta: 0.2, lambda: 0.002, tau_e: 40.0, n_train: 200, n_test: 100, bptt_epochs: 150, bptt_lr: 0.02, g2_min_gap_closed: 0.5, g2_min_accuracy: 0.65, g2_confidence_z: 1.96, g2_min_positive_control: 0.9, g2_min_reference_gap: 0.15, activity_sparsity_min: 0.005, activity_sparsity_max: 0.03, scientific_n_seeds: 20, power_sigma_prior: 0.15, power_effect_size: 0.1, use_surrogate_lif_reference: true, surrogate_beta: 5.0, matched_budget_repeat: true, quick: false }
```

Power analysis: required scientific n_seeds ≥ 20 (preregistered σ=0.150, effect=0.100; formula n=⌈(1.96+0.8416)²σ²/δ²⌉).

## Per-seed accuracies

| seed | local-assembly | dense-local | gradient reference | eligibility reference | activity_sparsity (local) | activity_sparsity (dense) | dense_matched |
|---|---:|---:|---:|---:|---:|---:|---:|
| 11400784225994701844 | 0.5000 | 0.6400 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.3800 |
| 4354472946875824171 | 0.7700 | 0.4700 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.4600 |
| 15755469790931547198 | 0.9400 | 0.0800 | 0.9300 | 1.0000 | 0.0156 | 0.0156 | 0.5200 |
| 8709160710835925077 | 0.7400 | 0.5000 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.3200 |
| 1663413756060003432 | 0.7400 | 0.5000 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.5200 |
| 13063846550650677375 | 0.7400 | 0.6300 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 6018099320996848786 | 0.5000 | 0.4900 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.4400 |
| 17418529916564267177 | 0.5000 | 0.5000 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.5200 |
| 10372782686910438588 | 1.0000 | 0.4000 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.3700 |
| 3326471682669467859 | 0.5000 | 0.0000 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.3700 |
| 14727610363725173990 | 0.7400 | 0.2400 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.6000 |
| 7681300184117924093 | 1.0000 | 0.2800 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.5300 |
| 635551854952467728 | 0.7400 | 0.2900 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.4300 |
| 12035985749054769447 | 0.7400 | 0.5000 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.5600 |
| 4990235495743964474 | 0.5000 | 0.0000 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.5200 |
| 16390669389846266193 | 0.5000 | 1.0000 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.4900 |
| 9344921060680809828 | 0.7400 | 0.6300 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.4800 |
| 2298610881073559931 | 0.2600 | 0.6200 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.3600 |
| 13699608824640910734 | 1.0000 | 0.8300 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.4100 |
| 6653297820399940005 | 0.5000 | 0.2000 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.2800 |

## Summary (paired normalized-gap analysis)

- mean ± var local-assembly: 0.6825 ± 0.042693
- mean ± var dense-local:    0.4400 ± 0.068011
- mean ± var gradient reference: 0.9940 ± 0.000352
- mean ± var eligibility reference: 1.0000 ± 0.000000
- mean normalized gap closed: 0.4670 (variance 0.123996)
- lower confidence bound (z=1.960, n=20): 0.3127
- mean |local − dense| (descriptive): 0.3425

## U-NEG

Negative result: local-assembly did **not** clear the preregistered normalized-gap confidence and absolute-accuracy gates. Program stops at G2; do not schedule P3+.

## Positive / sanity control

Mean local-pipeline accuracy on a trivially separable spatial feature-presence task: **1.0000** (threshold 0.900).

## Activity sparsity

Mean local-assembly activity_sparsity: **0.0156** (valid band [0.0050, 0.0300]; nominal k/N=0.0156).

## Parameter / compute budgets

| condition | n_cells | n_params | wall_secs | peak_rss_bytes | work_per_accuracy | spikes | deliveries | cell_updates | plasticity |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| local-assembly | 260 | 20014 | 0.0352 | 5390336 | 8437910.0000 | 2925 | 105513 | 107717 | 4002800 |
| dense-local | 260 | 66304 | 0.0895 | 9682944 | 22172670.8081 | 3520 | 462000 | 464189 | 13260800 |
| gradient-reference | 258 | 66305 | 16.0728 | 3899392 | 2050830000.0000 | 0 | 240000 | 61440000 | 1989150000 |
| eligibility-reference | 258 | 769 | 0.1324 | 2785280 | 84750000.0000 | 0 | 240000 | 61440000 | 23070000 |
| dense-matched | 260 | 20014 | 0.0333 | 5799936 | 11840318.5696 | 3485 | 245422 | 247614 | 4002800 |

Matched-budget dense mean accuracy: **0.4530** (n=20; primary G2 gap still uses unmatched dense-local).

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
config_hash=c1-983ee5303c00b147 seed=11400784225994701844 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=8437910.000000 note=wall_secs=0.0352_peak_rss=5390336_spikes=2925_deliveries=105513_cells=107717_plasticity=4002800
config_hash=c1-983ee5303c00b147 seed=11400784225994701844 condition=dense-local accuracy=0.640000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=22172670.808098 note=wall_secs=0.0895_peak_rss=9682944_spikes=3520_deliveries=462000_cells=464189_plasticity=13260800
config_hash=c1-983ee5303c00b147 seed=11400784225994701844 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=2050830000.000000 note=wall_secs=16.0728_peak_rss=3899392_spikes=0_deliveries=240000_cells=61440000_plasticity=1989150000
config_hash=c1-983ee5303c00b147 seed=11400784225994701844 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=84750000.000000 note=wall_secs=0.1324_peak_rss=2785280_spikes=0_deliveries=240000_cells=61440000_plasticity=23070000
config_hash=c1-983ee5303c00b147 seed=11400784225994701844 condition=dense-matched accuracy=0.380000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=11840318.569629 note=wall_secs=0.0333_peak_rss=5799936_spikes=3485_deliveries=245422_cells=247614_plasticity=4002800
config_hash=c1-983ee5303c00b147 seed=4354472946875824171 condition=local-assembly accuracy=0.770000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5477939.096732 note=wall_secs=0.0230_peak_rss=4685824_spikes=2582_deliveries=104825_cells=107006_plasticity=4003600
config_hash=c1-983ee5303c00b147 seed=4354472946875824171 condition=dense-local accuracy=0.470000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=30192340.502111 note=wall_secs=0.0709_peak_rss=7684096_spikes=3400_deliveries=462000_cells=464200_plasticity=13260800
config_hash=c1-983ee5303c00b147 seed=4354472946875824171 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=2050830000.000000 note=wall_secs=16.2050_peak_rss=3915776_spikes=0_deliveries=240000_cells=61440000_plasticity=1989150000
config_hash=c1-983ee5303c00b147 seed=4354472946875824171 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=84750000.000000 note=wall_secs=0.1312_peak_rss=2768896_spikes=0_deliveries=240000_cells=61440000_plasticity=23070000
config_hash=c1-983ee5303c00b147 seed=4354472946875824171 condition=dense-matched accuracy=0.460000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=9776199.822654 note=wall_secs=0.0297_peak_rss=5570560_spikes=3399_deliveries=243927_cells=246126_plasticity=4003600
config_hash=c1-983ee5303c00b147 seed=15755469790931547198 condition=local-assembly accuracy=0.940000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=4488556.394363 note=wall_secs=0.0240_peak_rss=4767744_spikes=3008_deliveries=105528_cells=107707_plasticity=4003000
config_hash=c1-983ee5303c00b147 seed=15755469790931547198 condition=dense-local accuracy=0.080000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=177380441.464762 note=wall_secs=0.0744_peak_rss=8749056_spikes=3432_deliveries=462000_cells=464203_plasticity=13260800
config_hash=c1-983ee5303c00b147 seed=15755469790931547198 condition=gradient-reference accuracy=0.930000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=2205193531.427125 note=wall_secs=16.0564_peak_rss=3932160_spikes=0_deliveries=240000_cells=61440000_plasticity=1989150000
config_hash=c1-983ee5303c00b147 seed=15755469790931547198 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=84750000.000000 note=wall_secs=0.1263_peak_rss=2768896_spikes=0_deliveries=240000_cells=61440000_plasticity=23070000
config_hash=c1-983ee5303c00b147 seed=15755469790931547198 condition=dense-matched accuracy=0.520000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=8652921.471233 note=wall_secs=0.0353_peak_rss=5652480_spikes=3459_deliveries=245427_cells=247633_plasticity=4003000
config_hash=c1-983ee5303c00b147 seed=8709160710835925077 condition=local-assembly accuracy=0.740000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5700744.521126 note=wall_secs=0.0232_peak_rss=4816896_spikes=2837_deliveries=105673_cells=107841_plasticity=4002200
config_hash=c1-983ee5303c00b147 seed=8709160710835925077 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=28380888.000000 note=wall_secs=0.0721_peak_rss=7700480_spikes=3446_deliveries=462000_cells=464198_plasticity=13260800
config_hash=c1-983ee5303c00b147 seed=8709160710835925077 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=2050830000.000000 note=wall_secs=16.0347_peak_rss=3932160_spikes=0_deliveries=240000_cells=61440000_plasticity=1989150000
config_hash=c1-983ee5303c00b147 seed=8709160710835925077 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=84750000.000000 note=wall_secs=0.1328_peak_rss=2785280_spikes=0_deliveries=240000_cells=61440000_plasticity=23070000
config_hash=c1-983ee5303c00b147 seed=8709160710835925077 condition=dense-matched accuracy=0.320000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=14050140.939045 note=wall_secs=0.0289_peak_rss=5488640_spikes=3474_deliveries=244086_cells=246285_plasticity=4002200
config_hash=c1-983ee5303c00b147 seed=1663413756060003432 condition=local-assembly accuracy=0.740000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5697997.223864 note=wall_secs=0.0232_peak_rss=4833280_spikes=2974_deliveries=104687_cells=106857_plasticity=4002000
config_hash=c1-983ee5303c00b147 seed=1663413756060003432 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=28381042.000000 note=wall_secs=0.0711_peak_rss=7880704_spikes=3525_deliveries=462000_cells=464196_plasticity=13260800
config_hash=c1-983ee5303c00b147 seed=1663413756060003432 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=2158768448.141605 note=wall_secs=16.1008_peak_rss=3899392_spikes=0_deliveries=240000_cells=61440000_plasticity=1989150000
config_hash=c1-983ee5303c00b147 seed=1663413756060003432 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=84750000.000000 note=wall_secs=0.1280_peak_rss=2768896_spikes=0_deliveries=240000_cells=61440000_plasticity=23070000
config_hash=c1-983ee5303c00b147 seed=1663413756060003432 condition=dense-matched accuracy=0.520000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=8651686.855804 note=wall_secs=0.0307_peak_rss=6471680_spikes=3496_deliveries=245593_cells=247788_plasticity=4002000
config_hash=c1-983ee5303c00b147 seed=13063846550650677375 condition=local-assembly accuracy=0.740000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5698410.737373 note=wall_secs=0.0233_peak_rss=4833280_spikes=2875_deliveries=105681_cells=107868_plasticity=4000400
config_hash=c1-983ee5303c00b147 seed=13063846550650677375 condition=dense-local accuracy=0.630000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=22524460.487944 note=wall_secs=0.0706_peak_rss=7749632_spikes=3405_deliveries=462000_cells=464205_plasticity=13260800
config_hash=c1-983ee5303c00b147 seed=13063846550650677375 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=2050830000.000000 note=wall_secs=16.0127_peak_rss=3932160_spikes=0_deliveries=240000_cells=61440000_plasticity=1989150000
config_hash=c1-983ee5303c00b147 seed=13063846550650677375 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=84750000.000000 note=wall_secs=0.1313_peak_rss=2785280_spikes=0_deliveries=240000_cells=61440000_plasticity=23070000
config_hash=c1-983ee5303c00b147 seed=13063846550650677375 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=8992284.000000 note=wall_secs=0.0288_peak_rss=5488640_spikes=3408_deliveries=245063_cells=247271_plasticity=4000400
config_hash=c1-983ee5303c00b147 seed=6018099320996848786 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=8433988.000000 note=wall_secs=0.0227_peak_rss=4702208_spikes=2404_deliveries=105006_cells=107184_plasticity=4002400
config_hash=c1-983ee5303c00b147 seed=6018099320996848786 condition=dense-local accuracy=0.490000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=28960179.028192 note=wall_secs=0.0704_peak_rss=7716864_spikes=3490_deliveries=462000_cells=464198_plasticity=13260800
config_hash=c1-983ee5303c00b147 seed=6018099320996848786 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=2050830000.000000 note=wall_secs=15.8745_peak_rss=3915776_spikes=0_deliveries=240000_cells=61440000_plasticity=1989150000
config_hash=c1-983ee5303c00b147 seed=6018099320996848786 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=84750000.000000 note=wall_secs=0.1264_peak_rss=2768896_spikes=0_deliveries=240000_cells=61440000_plasticity=23070000
config_hash=c1-983ee5303c00b147 seed=6018099320996848786 condition=dense-matched accuracy=0.440000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=10226161.419048 note=wall_secs=0.0295_peak_rss=5439488_spikes=3527_deliveries=245695_cells=247889_plasticity=4002400
config_hash=c1-983ee5303c00b147 seed=17418529916564267177 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=8438108.000000 note=wall_secs=0.0232_peak_rss=4882432_spikes=3018_deliveries=105021_cells=107215_plasticity=4003800
config_hash=c1-983ee5303c00b147 seed=17418529916564267177 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=28381034.000000 note=wall_secs=0.0711_peak_rss=7864320_spikes=3506_deliveries=462000_cells=464211_plasticity=13260800
config_hash=c1-983ee5303c00b147 seed=17418529916564267177 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=2050830000.000000 note=wall_secs=15.9280_peak_rss=3932160_spikes=0_deliveries=240000_cells=61440000_plasticity=1989150000
config_hash=c1-983ee5303c00b147 seed=17418529916564267177 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=84750000.000000 note=wall_secs=0.1332_peak_rss=2785280_spikes=0_deliveries=240000_cells=61440000_plasticity=23070000
config_hash=c1-983ee5303c00b147 seed=17418529916564267177 condition=dense-matched accuracy=0.520000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=8650669.548074 note=wall_secs=0.0287_peak_rss=5406720_spikes=3522_deliveries=244407_cells=246619_plasticity=4003800
config_hash=c1-983ee5303c00b147 seed=10372782686910438588 condition=local-assembly accuracy=1.000000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=4221925.000000 note=wall_secs=0.0257_peak_rss=4849664_spikes=2882_deliveries=106028_cells=108215_plasticity=4004800
config_hash=c1-983ee5303c00b147 seed=10372782686910438588 condition=dense-local accuracy=0.400000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=35476059.471366 note=wall_secs=0.0754_peak_rss=7995392_spikes=3429_deliveries=462000_cells=464195_plasticity=13260800
config_hash=c1-983ee5303c00b147 seed=10372782686910438588 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=2050830000.000000 note=wall_secs=15.9913_peak_rss=3915776_spikes=0_deliveries=240000_cells=61440000_plasticity=1989150000
config_hash=c1-983ee5303c00b147 seed=10372782686910438588 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=84750000.000000 note=wall_secs=0.1392_peak_rss=2785280_spikes=0_deliveries=240000_cells=61440000_plasticity=23070000
config_hash=c1-983ee5303c00b147 seed=10372782686910438588 condition=dense-matched accuracy=0.370000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=12169526.870192 note=wall_secs=0.0304_peak_rss=5750784_spikes=3459_deliveries=246136_cells=248330_plasticity=4004800
config_hash=c1-983ee5303c00b147 seed=3326471682669467859 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=8432742.000000 note=wall_secs=0.0243_peak_rss=4702208_spikes=2545_deliveries=105314_cells=107512_plasticity=4001000
config_hash=c1-983ee5303c00b147 seed=3326471682669467859 condition=dense-local accuracy=0.000000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=14190446035827.431641 note=wall_secs=0.0708_peak_rss=7897088_spikes=3447_deliveries=462000_cells=464199_plasticity=13260800
config_hash=c1-983ee5303c00b147 seed=3326471682669467859 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=2050830000.000000 note=wall_secs=16.0626_peak_rss=3915776_spikes=0_deliveries=240000_cells=61440000_plasticity=1989150000
config_hash=c1-983ee5303c00b147 seed=3326471682669467859 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=84750000.000000 note=wall_secs=0.1275_peak_rss=2785280_spikes=0_deliveries=240000_cells=61440000_plasticity=23070000
config_hash=c1-983ee5303c00b147 seed=3326471682669467859 condition=dense-matched accuracy=0.370000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=12153878.221745 note=wall_secs=0.0287_peak_rss=5406720_spikes=3475_deliveries=245131_cells=247329_plasticity=4001000
config_hash=c1-983ee5303c00b147 seed=14727610363725173990 condition=local-assembly accuracy=0.740000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5697032.359012 note=wall_secs=0.0234_peak_rss=4669440_spikes=2844_deliveries=105288_cells=107472_plasticity=4000200
config_hash=c1-983ee5303c00b147 seed=14727610363725173990 condition=dense-local accuracy=0.240000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=59126901.321589 note=wall_secs=0.0708_peak_rss=7684096_spikes=3448_deliveries=462000_cells=464208_plasticity=13260800
config_hash=c1-983ee5303c00b147 seed=14727610363725173990 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=2050830000.000000 note=wall_secs=16.0362_peak_rss=3915776_spikes=0_deliveries=240000_cells=61440000_plasticity=1989150000
config_hash=c1-983ee5303c00b147 seed=14727610363725173990 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=84750000.000000 note=wall_secs=0.1260_peak_rss=2768896_spikes=0_deliveries=240000_cells=61440000_plasticity=23070000
config_hash=c1-983ee5303c00b147 seed=14727610363725173990 condition=dense-matched accuracy=0.600000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=7494758.035518 note=wall_secs=0.0290_peak_rss=5537792_spikes=3475_deliveries=245487_cells=247693_plasticity=4000200
config_hash=c1-983ee5303c00b147 seed=7681300184117924093 condition=local-assembly accuracy=1.000000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=4214553.000000 note=wall_secs=0.0227_peak_rss=4947968_spikes=3089_deliveries=105362_cells=107502_plasticity=3998600
config_hash=c1-983ee5303c00b147 seed=7681300184117924093 condition=dense-local accuracy=0.280000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=50680674.784228 note=wall_secs=0.0713_peak_rss=7684096_spikes=3580_deliveries=462000_cells=464209_plasticity=13260800
config_hash=c1-983ee5303c00b147 seed=7681300184117924093 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=2050830000.000000 note=wall_secs=15.9461_peak_rss=3915776_spikes=0_deliveries=240000_cells=61440000_plasticity=1989150000
config_hash=c1-983ee5303c00b147 seed=7681300184117924093 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=84750000.000000 note=wall_secs=0.1266_peak_rss=2768896_spikes=0_deliveries=240000_cells=61440000_plasticity=23070000
config_hash=c1-983ee5303c00b147 seed=7681300184117924093 condition=dense-matched accuracy=0.530000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=8478223.099177 note=wall_secs=0.0295_peak_rss=5373952_spikes=3512_deliveries=244568_cells=246778_plasticity=3998600
config_hash=c1-983ee5303c00b147 seed=635551854952467728 condition=local-assembly accuracy=0.740000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5691055.332062 note=wall_secs=0.0233_peak_rss=4849664_spikes=2721_deliveries=104839_cells=107021_plasticity=3996800
config_hash=c1-983ee5303c00b147 seed=635551854952467728 condition=dense-local accuracy=0.290000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=48932373.821805 note=wall_secs=0.0703_peak_rss=7782400_spikes=3394_deliveries=462000_cells=464194_plasticity=13260800
config_hash=c1-983ee5303c00b147 seed=635551854952467728 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=2050830000.000000 note=wall_secs=15.9587_peak_rss=3915776_spikes=0_deliveries=240000_cells=61440000_plasticity=1989150000
config_hash=c1-983ee5303c00b147 seed=635551854952467728 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=84750000.000000 note=wall_secs=0.1276_peak_rss=2785280_spikes=0_deliveries=240000_cells=61440000_plasticity=23070000
config_hash=c1-983ee5303c00b147 seed=635551854952467728 condition=dense-matched accuracy=0.430000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=10445878.896012 note=wall_secs=0.0284_peak_rss=5472256_spikes=3395_deliveries=244669_cells=246864_plasticity=3996800
config_hash=c1-983ee5303c00b147 seed=12035985749054769447 condition=local-assembly accuracy=0.740000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5702886.412991 note=wall_secs=0.0227_peak_rss=4849664_spikes=2684_deliveries=105144_cells=107308_plasticity=4005000
config_hash=c1-983ee5303c00b147 seed=12035985749054769447 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=28380782.000000 note=wall_secs=0.0694_peak_rss=7667712_spikes=3407_deliveries=462000_cells=464184_plasticity=13260800
config_hash=c1-983ee5303c00b147 seed=12035985749054769447 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=2050830000.000000 note=wall_secs=15.9854_peak_rss=3915776_spikes=0_deliveries=240000_cells=61440000_plasticity=1989150000
config_hash=c1-983ee5303c00b147 seed=12035985749054769447 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=84750000.000000 note=wall_secs=0.1284_peak_rss=2768896_spikes=0_deliveries=240000_cells=61440000_plasticity=23070000
config_hash=c1-983ee5303c00b147 seed=12035985749054769447 condition=dense-matched accuracy=0.560000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=8034957.108649 note=wall_secs=0.0284_peak_rss=5439488_spikes=3409_deliveries=244492_cells=246675_plasticity=4005000
config_hash=c1-983ee5303c00b147 seed=4990235495743964474 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=8432526.000000 note=wall_secs=0.0231_peak_rss=4718592_spikes=2432_deliveries=105025_cells=107206_plasticity=4001600
config_hash=c1-983ee5303c00b147 seed=4990235495743964474 condition=dense-local accuracy=0.000000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=14190564035827.730469 note=wall_secs=0.0705_peak_rss=7831552_spikes=3564_deliveries=462000_cells=464200_plasticity=13260800
config_hash=c1-983ee5303c00b147 seed=4990235495743964474 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=2050830000.000000 note=wall_secs=15.9075_peak_rss=3948544_spikes=0_deliveries=240000_cells=61440000_plasticity=1989150000
config_hash=c1-983ee5303c00b147 seed=4990235495743964474 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=84750000.000000 note=wall_secs=0.1264_peak_rss=2768896_spikes=0_deliveries=240000_cells=61440000_plasticity=23070000
config_hash=c1-983ee5303c00b147 seed=4990235495743964474 condition=dense-matched accuracy=0.520000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=8648219.547984 note=wall_secs=0.0288_peak_rss=5570560_spikes=3600_deliveries=244839_cells=247035_plasticity=4001600
config_hash=c1-983ee5303c00b147 seed=16390669389846266193 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=8445656.000000 note=wall_secs=0.0239_peak_rss=4653056_spikes=2904_deliveries=106067_cells=108257_plasticity=4005600
config_hash=c1-983ee5303c00b147 seed=16390669389846266193 condition=dense-local accuracy=1.000000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=14190451.000000 note=wall_secs=0.0716_peak_rss=7815168_spikes=3448_deliveries=462000_cells=464203_plasticity=13260800
config_hash=c1-983ee5303c00b147 seed=16390669389846266193 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=2050830000.000000 note=wall_secs=16.3689_peak_rss=3915776_spikes=0_deliveries=240000_cells=61440000_plasticity=1989150000
config_hash=c1-983ee5303c00b147 seed=16390669389846266193 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=84750000.000000 note=wall_secs=0.1364_peak_rss=2768896_spikes=0_deliveries=240000_cells=61440000_plasticity=23070000
config_hash=c1-983ee5303c00b147 seed=16390669389846266193 condition=dense-matched accuracy=0.490000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=9188483.494636 note=wall_secs=0.0288_peak_rss=5423104_spikes=3518_deliveries=245515_cells=247724_plasticity=4005600
config_hash=c1-983ee5303c00b147 seed=9344921060680809828 condition=local-assembly accuracy=0.740000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5690462.088826 note=wall_secs=0.0237_peak_rss=4816896_spikes=2797_deliveries=104389_cells=106556_plasticity=3997200
config_hash=c1-983ee5303c00b147 seed=9344921060680809828 condition=dense-local accuracy=0.630000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=22524563.662548 note=wall_secs=0.0725_peak_rss=7864320_spikes=3470_deliveries=462000_cells=464205_plasticity=13260800
config_hash=c1-983ee5303c00b147 seed=9344921060680809828 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=2050830000.000000 note=wall_secs=16.2969_peak_rss=3915776_spikes=0_deliveries=240000_cells=61440000_plasticity=1989150000
config_hash=c1-983ee5303c00b147 seed=9344921060680809828 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=84750000.000000 note=wall_secs=0.1260_peak_rss=2752512_spikes=0_deliveries=240000_cells=61440000_plasticity=23070000
config_hash=c1-983ee5303c00b147 seed=9344921060680809828 condition=dense-matched accuracy=0.480000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=9357264.792484 note=wall_secs=0.0307_peak_rss=5554176_spikes=3512_deliveries=244285_cells=246490_plasticity=3997200
config_hash=c1-983ee5303c00b147 seed=2298610881073559931 condition=local-assembly accuracy=0.260000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=16193465.978588 note=wall_secs=0.0237_peak_rss=4734976_spikes=2637_deliveries=104215_cells=106449_plasticity=3997000
config_hash=c1-983ee5303c00b147 seed=2298610881073559931 condition=dense-local accuracy=0.620000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=22887896.598165 note=wall_secs=0.0710_peak_rss=7847936_spikes=3492_deliveries=462000_cells=464204_plasticity=13260800
config_hash=c1-983ee5303c00b147 seed=2298610881073559931 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=2050830000.000000 note=wall_secs=16.0504_peak_rss=3915776_spikes=0_deliveries=240000_cells=61440000_plasticity=1989150000
config_hash=c1-983ee5303c00b147 seed=2298610881073559931 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=84750000.000000 note=wall_secs=0.1261_peak_rss=2752512_spikes=0_deliveries=240000_cells=61440000_plasticity=23070000
config_hash=c1-983ee5303c00b147 seed=2298610881073559931 condition=dense-matched accuracy=0.360000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=12477830.059731 note=wall_secs=0.0288_peak_rss=5570560_spikes=3490_deliveries=244664_cells=246865_plasticity=3997000
config_hash=c1-983ee5303c00b147 seed=13699608824640910734 condition=local-assembly accuracy=1.000000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=4217442.000000 note=wall_secs=0.0232_peak_rss=4734976_spikes=2949_deliveries=105658_cells=107835_plasticity=4001000
config_hash=c1-983ee5303c00b147 seed=13699608824640910734 condition=dense-local accuracy=0.830000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=17096872.632934 note=wall_secs=0.0729_peak_rss=8667136_spikes=3402_deliveries=462000_cells=464202_plasticity=13260800
config_hash=c1-983ee5303c00b147 seed=13699608824640910734 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=2050830000.000000 note=wall_secs=15.9267_peak_rss=3915776_spikes=0_deliveries=240000_cells=61440000_plasticity=1989150000
config_hash=c1-983ee5303c00b147 seed=13699608824640910734 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=84750000.000000 note=wall_secs=0.1270_peak_rss=2785280_spikes=0_deliveries=240000_cells=61440000_plasticity=23070000
config_hash=c1-983ee5303c00b147 seed=13699608824640910734 condition=dense-matched accuracy=0.410000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=10967624.485910 note=wall_secs=0.0296_peak_rss=5521408_spikes=3402_deliveries=245061_cells=247263_plasticity=4001000
config_hash=c1-983ee5303c00b147 seed=6653297820399940005 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=8428808.000000 note=wall_secs=0.0231_peak_rss=4947968_spikes=2393_deliveries=104914_cells=107097_plasticity=4000000
config_hash=c1-983ee5303c00b147 seed=6653297820399940005 condition=dense-local accuracy=0.200000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=70952953.942719 note=wall_secs=0.0715_peak_rss=7880704_spikes=3590_deliveries=462000_cells=464201_plasticity=13260800
config_hash=c1-983ee5303c00b147 seed=6653297820399940005 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=2050830000.000000 note=wall_secs=15.8597_peak_rss=3915776_spikes=0_deliveries=240000_cells=61440000_plasticity=1989150000
config_hash=c1-983ee5303c00b147 seed=6653297820399940005 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=84750000.000000 note=wall_secs=0.1240_peak_rss=2752512_spikes=0_deliveries=240000_cells=61440000_plasticity=23070000
config_hash=c1-983ee5303c00b147 seed=6653297820399940005 condition=dense-matched accuracy=0.280000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=16052785.645941 note=wall_secs=0.0296_peak_rss=5505024_spikes=3538_deliveries=244521_cells=246721_plasticity=4000000
```
