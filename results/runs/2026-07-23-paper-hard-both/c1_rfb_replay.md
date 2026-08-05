# C1 / Gate G2 results note

**Config hash:** `c1-660401d74db3c88d`

**Scientific protocol version:** `13`

**Live `ReinforceFeedback` protocol:** `13` — same k-WTA / single-pass C1 substrate as v2; main-condition plasticity uses production `ReinforceFeedback` × sampled `reinforce_term` (Bernoulli action from soft readout policy); **positive control stays on broadcast ±1** with a disclosed longer easy-PC schedule (substrate/encoding check; G2 floors unchanged); does **not** reopen protocol-v2 kill-gate hash `c1-118207fbc3eaba53` (canonical version `2`), remassage P4 spiking-DFA, or retune P5 `rl_graded`.

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
Config { experiment: "c1-rfb", master_seed: 212618061021185, n_seeds: 20, sequence_len: 8, max_lag: 1, n_hidden: 128, k_wta: 2, p_sparse: 0.35, init_w: 0.15, eta: 0.35, lambda: 0.002, tau_e: 40.0, n_train: 80, n_test: 40, bptt_epochs: 80, bptt_lr: 0.05, g2_min_gap_closed: 0.5, g2_min_accuracy: 0.65, g2_confidence_z: 1.96, g2_min_positive_control: 0.9, g2_min_reference_gap: 0.15, activity_sparsity_min: 0.005, activity_sparsity_max: 0.03, scientific_n_seeds: 20, power_sigma_prior: 0.15, power_effect_size: 0.1, use_surrogate_lif_reference: true, surrogate_beta: 5.0, matched_budget_repeat: true, quick: false }
```

Power analysis: required scientific n_seeds ≥ 20 (preregistered σ=0.150, effect=0.100; formula n=⌈(1.96+0.8416)²σ²/δ²⌉).

## Per-seed accuracies

| seed | local-assembly | dense-local | gradient reference | eligibility reference | activity_sparsity (local) | activity_sparsity (dense) | dense_matched |
|---|---:|---:|---:|---:|---:|---:|---:|
| 11400784225994701844 | 0.5000 | 0.7250 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.7250 |
| 4354472946875824171 | 0.3500 | 0.5000 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 15755469790931547198 | 0.5000 | 0.2750 | 0.7250 | 1.0000 | 0.0156 | 0.0156 | 0.2750 |
| 8709160710835925077 | 0.4000 | 0.5000 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.7250 |
| 1663413756060003432 | 0.5750 | 0.5000 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 13063846550650677375 | 0.5000 | 0.2750 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.2750 |
| 6018099320996848786 | 0.5000 | 0.2750 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.7750 |
| 17418529916564267177 | 0.5000 | 0.5000 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 10372782686910438588 | 0.5000 | 0.5250 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.2250 |
| 3326471682669467859 | 0.5000 | 0.2750 | 0.7250 | 1.0000 | 0.0156 | 0.0156 | 0.2750 |
| 14727610363725173990 | 0.5000 | 0.7000 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 7681300184117924093 | 0.4250 | 0.5000 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.7250 |
| 635551854952467728 | 0.5750 | 0.2750 | 0.5000 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 12035985749054769447 | 0.5000 | 0.7250 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.7250 |
| 4990235495743964474 | 0.4000 | 0.2750 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.2750 |
| 16390669389846266193 | 0.6500 | 0.3500 | 0.5000 | 1.0000 | 0.0156 | 0.0156 | 0.3500 |
| 9344921060680809828 | 0.5000 | 0.7250 | 0.7250 | 1.0000 | 0.0156 | 0.0156 | 0.7250 |
| 2298610881073559931 | 0.5000 | 0.2750 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.2750 |
| 13699608824640910734 | 0.5000 | 0.5000 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.7250 |
| 6653297820399940005 | 0.4250 | 0.7250 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |

## Summary (paired normalized-gap analysis)

- mean ± var local-assembly: 0.4900 ± 0.004500
- mean ± var dense-local:    0.4700 ± 0.031684
- mean ± var gradient reference: 0.8938 ± 0.027163
- mean ± var eligibility reference: 1.0000 ± 0.000000
- mean normalized gap closed: 0.2135 (variance 0.101829)
- lower confidence bound (z=1.960, n=20): 0.0737
- mean |local − dense| (descriptive): 0.1725
- descriptive chance-normalized gap mean / LCB: 0.0083 / -0.0080 (var 0.001389; **not a gate**)
- seed local min / max / frac≥0.65: 0.3500 / 0.6500 / 0.05

## U-NEG

Negative result: local-assembly did **not** clear the preregistered normalized-gap confidence and absolute-accuracy gates. Program stops at G2; do not schedule P3+.

## Positive / sanity control

Mean local-pipeline accuracy on a trivially separable spatial feature-presence task: **0.9488** (threshold 0.900).

## Activity sparsity

Mean local-assembly activity_sparsity: **0.0156** (valid band [0.0050, 0.0300]; nominal k/N=0.0156).

## Parameter / compute budgets

| condition | n_cells | n_params | wall_secs | peak_rss_bytes | work_per_accuracy | spikes | deliveries | cell_updates | plasticity |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| local-assembly | 132 | 5777 | 0.0049 | 3784704 | 979664.0000 | 782 | 13126 | 13764 | 462160 |
| dense-local | 132 | 16768 | 0.0094 | 4554752 | 2022510.2783 | 889 | 61680 | 62311 | 1341440 |
| gradient-reference | 130 | 16769 | 0.6747 | 2899968 | 113926400.0000 | 0 | 51200 | 6553600 | 107321600 |
| eligibility-reference | 130 | 385 | 0.0152 | 2605056 | 9068800.0000 | 0 | 51200 | 6553600 | 2464000 |
| dense-matched | 132 | 5777 | 0.0057 | 3948544 | 753799.9752 | 894 | 41409 | 42042 | 462160 |

Matched-budget dense mean accuracy: **0.5037** (n=20; primary G2 gap still uses unmatched dense-local).

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
config_hash=c1-660401d74db3c88d seed=11400784225994701844 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=979664.000000 note=wall_secs=0.0049_peak_rss=3784704_spikes=782_deliveries=13126_cells=13764_plasticity=462160
config_hash=c1-660401d74db3c88d seed=11400784225994701844 condition=dense-local accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2022510.278317 note=wall_secs=0.0094_peak_rss=4554752_spikes=889_deliveries=61680_cells=62311_plasticity=1341440
config_hash=c1-660401d74db3c88d seed=11400784225994701844 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6747_peak_rss=2899968_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-660401d74db3c88d seed=11400784225994701844 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0152_peak_rss=2605056_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-660401d74db3c88d seed=11400784225994701844 condition=dense-matched accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=753799.975211 note=wall_secs=0.0057_peak_rss=3948544_spikes=894_deliveries=41409_cells=42042_plasticity=462160
config_hash=c1-660401d74db3c88d seed=4354472946875824171 condition=local-assembly accuracy=0.350000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1399105.738112 note=wall_secs=0.0055_peak_rss=3686400_spikes=771_deliveries=13141_cells=13775_plasticity=462000
config_hash=c1-660401d74db3c88d seed=4354472946875824171 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2932556.000000 note=wall_secs=0.0101_peak_rss=4489216_spikes=845_deliveries=61680_cells=62313_plasticity=1341440
config_hash=c1-660401d74db3c88d seed=4354472946875824171 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6861_peak_rss=2998272_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-660401d74db3c88d seed=4354472946875824171 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0153_peak_rss=2637824_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-660401d74db3c88d seed=4354472946875824171 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1094048.000000 note=wall_secs=0.0056_peak_rss=4145152_spikes=860_deliveries=41765_cells=42399_plasticity=462000
config_hash=c1-660401d74db3c88d seed=15755469790931547198 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=978898.000000 note=wall_secs=0.0053_peak_rss=3604480_spikes=719_deliveries=13086_cells=13724_plasticity=461920
config_hash=c1-660401d74db3c88d seed=15755469790931547198 condition=dense-local accuracy=0.275000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5332105.338975 note=wall_secs=0.0111_peak_rss=4653056_spikes=892_deliveries=61680_cells=62317_plasticity=1341440
config_hash=c1-660401d74db3c88d seed=15755469790931547198 condition=gradient-reference accuracy=0.725000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=157139856.901371 note=wall_secs=0.6862_peak_rss=2998272_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-660401d74db3c88d seed=15755469790931547198 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0162_peak_rss=2637824_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-660401d74db3c88d seed=15755469790931547198 condition=dense-matched accuracy=0.275000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1979690.866182 note=wall_secs=0.0061_peak_rss=4145152_spikes=891_deliveries=40483_cells=41121_plasticity=461920
config_hash=c1-660401d74db3c88d seed=8709160710835925077 condition=local-assembly accuracy=0.400000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1228207.481698 note=wall_secs=0.0051_peak_rss=3768320_spikes=729_deliveries=13321_cells=13953_plasticity=463280
config_hash=c1-660401d74db3c88d seed=8709160710835925077 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2932634.000000 note=wall_secs=0.0096_peak_rss=4505600_spikes=878_deliveries=61680_cells=62319_plasticity=1341440
config_hash=c1-660401d74db3c88d seed=8709160710835925077 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6803_peak_rss=2981888_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-660401d74db3c88d seed=8709160710835925077 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0179_peak_rss=2637824_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-660401d74db3c88d seed=8709160710835925077 condition=dense-matched accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=755631.699289 note=wall_secs=0.0060_peak_rss=3915776_spikes=873_deliveries=41520_cells=42160_plasticity=463280
config_hash=c1-660401d74db3c88d seed=1663413756060003432 condition=local-assembly accuracy=0.575000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=853617.409002 note=wall_secs=0.0050_peak_rss=3702784_spikes=647_deliveries=13093_cells=13730_plasticity=463360
config_hash=c1-660401d74db3c88d seed=1663413756060003432 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2932534.000000 note=wall_secs=0.0106_peak_rss=4653056_spikes=836_deliveries=61680_cells=62311_plasticity=1341440
config_hash=c1-660401d74db3c88d seed=1663413756060003432 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6953_peak_rss=2998272_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-660401d74db3c88d seed=1663413756060003432 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0414_peak_rss=2637824_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-660401d74db3c88d seed=1663413756060003432 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1093074.000000 note=wall_secs=0.0060_peak_rss=4145152_spikes=839_deliveries=40852_cells=41486_plasticity=463360
config_hash=c1-660401d74db3c88d seed=13063846550650677375 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=980020.000000 note=wall_secs=0.0057_peak_rss=3751936_spikes=853_deliveries=13298_cells=13939_plasticity=461920
config_hash=c1-660401d74db3c88d seed=13063846550650677375 condition=dense-local accuracy=0.275000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5332083.520794 note=wall_secs=0.0112_peak_rss=4915200_spikes=882_deliveries=61680_cells=62321_plasticity=1341440
config_hash=c1-660401d74db3c88d seed=13063846550650677375 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6913_peak_rss=2998272_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-660401d74db3c88d seed=13063846550650677375 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0405_peak_rss=2637824_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-660401d74db3c88d seed=13063846550650677375 condition=dense-matched accuracy=0.275000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1982298.138853 note=wall_secs=0.0060_peak_rss=3981312_spikes=895_deliveries=40837_cells=41480_plasticity=461920
config_hash=c1-660401d74db3c88d seed=6018099320996848786 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=980404.000000 note=wall_secs=0.0053_peak_rss=3768320_spikes=746_deliveries=13482_cells=14134_plasticity=461840
config_hash=c1-660401d74db3c88d seed=6018099320996848786 condition=dense-local accuracy=0.275000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5331945.338979 note=wall_secs=0.0100_peak_rss=4407296_spikes=835_deliveries=61680_cells=62330_plasticity=1341440
config_hash=c1-660401d74db3c88d seed=6018099320996848786 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6774_peak_rss=2981888_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-660401d74db3c88d seed=6018099320996848786 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0160_peak_rss=2654208_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-660401d74db3c88d seed=6018099320996848786 condition=dense-matched accuracy=0.775000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=699598.731200 note=wall_secs=0.0055_peak_rss=3948544_spikes=904_deliveries=39399_cells=40046_plasticity=461840
config_hash=c1-660401d74db3c88d seed=17418529916564267177 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=981052.000000 note=wall_secs=0.0050_peak_rss=3784704_spikes=721_deliveries=13381_cells=14024_plasticity=462400
config_hash=c1-660401d74db3c88d seed=17418529916564267177 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2932530.000000 note=wall_secs=0.0102_peak_rss=4407296_spikes=816_deliveries=61680_cells=62329_plasticity=1341440
config_hash=c1-660401d74db3c88d seed=17418529916564267177 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6815_peak_rss=3014656_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-660401d74db3c88d seed=17418529916564267177 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0401_peak_rss=2654208_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-660401d74db3c88d seed=17418529916564267177 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1089970.000000 note=wall_secs=0.0060_peak_rss=3964928_spikes=816_deliveries=40560_cells=41209_plasticity=462400
config_hash=c1-660401d74db3c88d seed=10372782686910438588 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=981836.000000 note=wall_secs=0.0050_peak_rss=3801088_spikes=841_deliveries=13478_cells=14119_plasticity=462480
config_hash=c1-660401d74db3c88d seed=10372782686910438588 condition=dense-local accuracy=0.525000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2792979.174457 note=wall_secs=0.0098_peak_rss=4407296_spikes=878_deliveries=61680_cells=62316_plasticity=1341440
config_hash=c1-660401d74db3c88d seed=10372782686910438588 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6917_peak_rss=2998272_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-660401d74db3c88d seed=10372782686910438588 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0155_peak_rss=2605056_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-660401d74db3c88d seed=10372782686910438588 condition=dense-matched accuracy=0.225000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2424008.953103 note=wall_secs=0.0055_peak_rss=3932160_spikes=892_deliveries=40697_cells=41333_plasticity=462480
config_hash=c1-660401d74db3c88d seed=3326471682669467859 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=981258.000000 note=wall_secs=0.0050_peak_rss=3702784_spikes=787_deliveries=13480_cells=14122_plasticity=462240
config_hash=c1-660401d74db3c88d seed=3326471682669467859 condition=dense-local accuracy=0.275000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5332338.066243 note=wall_secs=0.0098_peak_rss=4554752_spikes=950_deliveries=61680_cells=62323_plasticity=1341440
config_hash=c1-660401d74db3c88d seed=3326471682669467859 condition=gradient-reference accuracy=0.725000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=157139856.901371 note=wall_secs=0.6751_peak_rss=2899968_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-660401d74db3c88d seed=3326471682669467859 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0153_peak_rss=2637824_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-660401d74db3c88d seed=3326471682669467859 condition=dense-matched accuracy=0.275000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1987101.775113 note=wall_secs=0.0056_peak_rss=4063232_spikes=914_deliveries=41327_cells=41972_plasticity=462240
config_hash=c1-660401d74db3c88d seed=14727610363725173990 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=980402.000000 note=wall_secs=0.0051_peak_rss=3588096_spikes=672_deliveries=13204_cells=13845_plasticity=462480
config_hash=c1-660401d74db3c88d seed=14727610363725173990 condition=dense-local accuracy=0.700000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2094758.607102 note=wall_secs=0.0099_peak_rss=4571136_spikes=889_deliveries=61680_cells=62322_plasticity=1341440
config_hash=c1-660401d74db3c88d seed=14727610363725173990 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6707_peak_rss=2916352_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-660401d74db3c88d seed=14727610363725173990 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0165_peak_rss=2605056_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-660401d74db3c88d seed=14727610363725173990 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1093634.000000 note=wall_secs=0.0057_peak_rss=3948544_spikes=863_deliveries=41417_cells=42057_plasticity=462480
config_hash=c1-660401d74db3c88d seed=7681300184117924093 condition=local-assembly accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1148710.556015 note=wall_secs=0.0054_peak_rss=3735552_spikes=665_deliveries=13006_cells=13651_plasticity=460880
config_hash=c1-660401d74db3c88d seed=7681300184117924093 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2932662.000000 note=wall_secs=0.0107_peak_rss=4751360_spikes=884_deliveries=61680_cells=62327_plasticity=1341440
config_hash=c1-660401d74db3c88d seed=7681300184117924093 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6917_peak_rss=2981888_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-660401d74db3c88d seed=7681300184117924093 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0183_peak_rss=2637824_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-660401d74db3c88d seed=7681300184117924093 condition=dense-matched accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=750380.664979 note=wall_secs=0.0063_peak_rss=3915776_spikes=899_deliveries=40803_cells=41444_plasticity=460880
config_hash=c1-660401d74db3c88d seed=635551854952467728 condition=local-assembly accuracy=0.575000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=851081.756775 note=wall_secs=0.0056_peak_rss=3784704_spikes=641_deliveries=13207_cells=13844_plasticity=461680
config_hash=c1-660401d74db3c88d seed=635551854952467728 condition=dense-local accuracy=0.275000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5331999.884432 note=wall_secs=0.0098_peak_rss=4390912_spikes=865_deliveries=61680_cells=62315_plasticity=1341440
config_hash=c1-660401d74db3c88d seed=635551854952467728 condition=gradient-reference accuracy=0.500000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=227852800.000000 note=wall_secs=0.6792_peak_rss=2981888_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-660401d74db3c88d seed=635551854952467728 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0155_peak_rss=2637824_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-660401d74db3c88d seed=635551854952467728 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1088622.000000 note=wall_secs=0.0062_peak_rss=4308992_spikes=851_deliveries=40573_cells=41207_plasticity=461680
config_hash=c1-660401d74db3c88d seed=12035985749054769447 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=984436.000000 note=wall_secs=0.0049_peak_rss=3620864_spikes=794_deliveries=13553_cells=14191_plasticity=463680
config_hash=c1-660401d74db3c88d seed=12035985749054769447 condition=dense-local accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2022515.795558 note=wall_secs=0.0095_peak_rss=4472832_spikes=888_deliveries=61680_cells=62316_plasticity=1341440
config_hash=c1-660401d74db3c88d seed=12035985749054769447 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6688_peak_rss=2899968_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-660401d74db3c88d seed=12035985749054769447 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0149_peak_rss=2654208_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-660401d74db3c88d seed=12035985749054769447 condition=dense-matched accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=756062.044102 note=wall_secs=0.0058_peak_rss=3932160_spikes=872_deliveries=41479_cells=42114_plasticity=463680
config_hash=c1-660401d74db3c88d seed=4990235495743964474 condition=local-assembly accuracy=0.400000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1226529.981723 note=wall_secs=0.0053_peak_rss=3588096_spikes=705_deliveries=13272_cells=13915_plasticity=462720
config_hash=c1-660401d74db3c88d seed=4990235495743964474 condition=dense-local accuracy=0.275000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5332203.520791 note=wall_secs=0.0099_peak_rss=4521984_spikes=913_deliveries=61680_cells=62323_plasticity=1341440
config_hash=c1-660401d74db3c88d seed=4990235495743964474 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6740_peak_rss=2998272_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-660401d74db3c88d seed=4990235495743964474 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0156_peak_rss=2637824_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-660401d74db3c88d seed=4990235495743964474 condition=dense-matched accuracy=0.275000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1986843.593300 note=wall_secs=0.0056_peak_rss=4063232_spikes=899_deliveries=41061_cells=41702_plasticity=462720
config_hash=c1-660401d74db3c88d seed=16390669389846266193 condition=local-assembly accuracy=0.650000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=754487.719982 note=wall_secs=0.0050_peak_rss=3801088_spikes=708_deliveries=13174_cells=13815_plasticity=462720
config_hash=c1-660401d74db3c88d seed=16390669389846266193 condition=dense-local accuracy=0.350000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=4189434.357060 note=wall_secs=0.0100_peak_rss=4407296_spikes=854_deliveries=61680_cells=62328_plasticity=1341440
config_hash=c1-660401d74db3c88d seed=16390669389846266193 condition=gradient-reference accuracy=0.500000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=227852800.000000 note=wall_secs=0.6742_peak_rss=2916352_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-660401d74db3c88d seed=16390669389846266193 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0153_peak_rss=2637824_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-660401d74db3c88d seed=16390669389846266193 condition=dense-matched accuracy=0.350000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1557537.169382 note=wall_secs=0.0057_peak_rss=3932160_spikes=854_deliveries=40458_cells=41106_plasticity=462720
config_hash=c1-660401d74db3c88d seed=9344921060680809828 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=980166.000000 note=wall_secs=0.0053_peak_rss=3588096_spikes=750_deliveries=13390_cells=14023_plasticity=461920
config_hash=c1-660401d74db3c88d seed=9344921060680809828 condition=dense-local accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2022453.726594 note=wall_secs=0.0098_peak_rss=4390912_spikes=849_deliveries=61680_cells=62310_plasticity=1341440
config_hash=c1-660401d74db3c88d seed=9344921060680809828 condition=gradient-reference accuracy=0.725000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=157139856.901371 note=wall_secs=0.6900_peak_rss=2981888_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-660401d74db3c88d seed=9344921060680809828 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0157_peak_rss=2637824_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-660401d74db3c88d seed=9344921060680809828 condition=dense-matched accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=755878.595832 note=wall_secs=0.0058_peak_rss=4079616_spikes=855_deliveries=42304_cells=42933_plasticity=461920
config_hash=c1-660401d74db3c88d seed=2298610881073559931 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=979468.000000 note=wall_secs=0.0051_peak_rss=3588096_spikes=676_deliveries=13090_cells=13728_plasticity=462240
config_hash=c1-660401d74db3c88d seed=2298610881073559931 condition=dense-local accuracy=0.275000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5331992.611705 note=wall_secs=0.0098_peak_rss=4358144_spikes=856_deliveries=61680_cells=62322_plasticity=1341440
config_hash=c1-660401d74db3c88d seed=2298610881073559931 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6911_peak_rss=2981888_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-660401d74db3c88d seed=2298610881073559931 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0157_peak_rss=2637824_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-660401d74db3c88d seed=2298610881073559931 condition=dense-matched accuracy=0.275000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1989861.775053 note=wall_secs=0.0058_peak_rss=4046848_spikes=841_deliveries=41746_cells=42385_plasticity=462240
config_hash=c1-660401d74db3c88d seed=13699608824640910734 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=980194.000000 note=wall_secs=0.0050_peak_rss=3620864_spikes=776_deliveries=13139_cells=13782_plasticity=462400
config_hash=c1-660401d74db3c88d seed=13699608824640910734 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2932672.000000 note=wall_secs=0.0097_peak_rss=4390912_spikes=893_deliveries=61680_cells=62323_plasticity=1341440
config_hash=c1-660401d74db3c88d seed=13699608824640910734 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.7184_peak_rss=2998272_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-660401d74db3c88d seed=13699608824640910734 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0152_peak_rss=2637824_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-660401d74db3c88d seed=13699608824640910734 condition=dense-matched accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=751056.527025 note=wall_secs=0.0056_peak_rss=4128768_spikes=900_deliveries=40290_cells=40926_plasticity=462400
config_hash=c1-660401d74db3c88d seed=6653297820399940005 condition=local-assembly accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1152790.555900 note=wall_secs=0.0049_peak_rss=3620864_spikes=646_deliveries=13084_cells=13726_plasticity=462480
config_hash=c1-660401d74db3c88d seed=6653297820399940005 condition=dense-local accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2022576.485211 note=wall_secs=0.0097_peak_rss=4505600_spikes=930_deliveries=61680_cells=62318_plasticity=1341440
config_hash=c1-660401d74db3c88d seed=6653297820399940005 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6780_peak_rss=2899968_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-660401d74db3c88d seed=6653297820399940005 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0407_peak_rss=2637824_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-660401d74db3c88d seed=6653297820399940005 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1096624.000000 note=wall_secs=0.0055_peak_rss=4063232_spikes=889_deliveries=42150_cells=42793_plasticity=462480
```
