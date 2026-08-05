# C1 / Gate G2 results note

**Config hash:** `c1-c7d2c86a2b1927f6`

**Scientific protocol version:** `18`

**Eligibility × REINFORCE protocol:** `18` — v15 structured hidden `B` plus eligibility timing co-designed with sampled REINFORCE (`τ_e = 160`; mid-trial eligibility absorb after winners/readout before the REINFORCE action); **positive control stays on broadcast ±1**; does **not** remassage v13–v17 hashes or reopen protocol-v2 `c1-118207fbc3eaba53`.

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
Config { experiment: "c1-elig-rfb", master_seed: 212618061021185, n_seeds: 20, sequence_len: 8, max_lag: 1, n_hidden: 128, k_wta: 2, p_sparse: 0.35, init_w: 0.15, eta: 0.35, lambda: 0.002, tau_e: 160.0, n_train: 80, n_test: 40, bptt_epochs: 80, bptt_lr: 0.05, g2_min_gap_closed: 0.5, g2_min_accuracy: 0.65, g2_confidence_z: 1.96, g2_min_positive_control: 0.9, g2_min_reference_gap: 0.15, activity_sparsity_min: 0.005, activity_sparsity_max: 0.03, scientific_n_seeds: 20, power_sigma_prior: 0.15, power_effect_size: 0.1, use_surrogate_lif_reference: true, surrogate_beta: 5.0, matched_budget_repeat: true, quick: false }
```

Power analysis: required scientific n_seeds ≥ 20 (preregistered σ=0.150, effect=0.100; formula n=⌈(1.96+0.8416)²σ²/δ²⌉).

## Per-seed accuracies

| seed | local-assembly | dense-local | gradient reference | eligibility reference | activity_sparsity (local) | activity_sparsity (dense) | dense_matched |
|---|---:|---:|---:|---:|---:|---:|---:|
| 11400784225994701844 | 1.0000 | 0.5500 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.4500 |
| 4354472946875824171 | 0.7500 | 0.5750 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.2500 |
| 15755469790931547198 | 0.5000 | 0.4500 | 0.7250 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 8709160710835925077 | 0.9000 | 0.4500 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.2500 |
| 1663413756060003432 | 0.9000 | 0.3750 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.6000 |
| 13063846550650677375 | 1.0000 | 0.5750 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.5500 |
| 6018099320996848786 | 1.0000 | 0.4250 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.6250 |
| 17418529916564267177 | 0.5500 | 0.5000 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.6250 |
| 10372782686910438588 | 0.8250 | 0.4500 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.5250 |
| 3326471682669467859 | 0.5000 | 0.5000 | 0.7250 | 1.0000 | 0.0156 | 0.0156 | 0.4750 |
| 14727610363725173990 | 0.3750 | 0.4750 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 7681300184117924093 | 0.5000 | 0.5000 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.3500 |
| 635551854952467728 | 1.0000 | 0.4500 | 0.5000 | 1.0000 | 0.0156 | 0.0156 | 0.5500 |
| 12035985749054769447 | 0.5000 | 0.5750 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.5500 |
| 4990235495743964474 | 1.0000 | 0.4250 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 16390669389846266193 | 0.5000 | 0.4750 | 0.5000 | 1.0000 | 0.0156 | 0.0156 | 0.4500 |
| 9344921060680809828 | 0.7250 | 0.5000 | 0.7250 | 1.0000 | 0.0156 | 0.0156 | 0.6750 |
| 2298610881073559931 | 0.5000 | 0.4500 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.4500 |
| 13699608824640910734 | 0.7250 | 0.6250 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.5500 |
| 6653297820399940005 | 0.5000 | 0.5750 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.6000 |

## Summary (paired normalized-gap analysis)

- mean ± var local-assembly: 0.7125 ± 0.050230
- mean ± var dense-local:    0.4950 ± 0.004250
- mean ± var gradient reference: 0.8938 ± 0.027163
- mean ± var eligibility reference: 1.0000 ± 0.000000
- mean normalized gap closed: 0.4287 (variance 0.195050)
- lower confidence bound (z=1.960, n=20): 0.2351
- mean |local − dense| (descriptive): 0.2425

## U-NEG

Negative result: local-assembly did **not** clear the preregistered normalized-gap confidence and absolute-accuracy gates. Program stops at G2; do not schedule P3+.

## Positive / sanity control

Mean local-pipeline accuracy on a trivially separable spatial feature-presence task: **0.9600** (threshold 0.900).

## Activity sparsity

Mean local-assembly activity_sparsity: **0.0156** (valid band [0.0050, 0.0300]; nominal k/N=0.0156).

## Parameter / compute budgets

| condition | n_cells | n_params | wall_secs | peak_rss_bytes | work_per_accuracy | spikes | deliveries | cell_updates | plasticity |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| local-assembly | 132 | 5777 | 0.0074 | 3719168 | 490124.0000 | 808 | 13265 | 13891 | 462160 |
| dense-local | 132 | 16768 | 0.0115 | 4407296 | 2666119.9422 | 930 | 61680 | 62316 | 1341440 |
| gradient-reference | 130 | 16769 | 0.6551 | 2916352 | 113926400.0000 | 0 | 51200 | 6553600 | 107321600 |
| eligibility-reference | 130 | 385 | 0.0398 | 2654208 | 9068800.0000 | 0 | 51200 | 6553600 | 2464000 |
| dense-matched | 132 | 5777 | 0.0061 | 4030464 | 1213375.5877 | 898 | 41162 | 41799 | 462160 |

Matched-budget dense mean accuracy: **0.5013** (n=20; primary G2 gap still uses unmatched dense-local).

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
config_hash=c1-c7d2c86a2b1927f6 seed=11400784225994701844 condition=local-assembly accuracy=1.000000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=490124.000000 note=wall_secs=0.0074_peak_rss=3719168_spikes=808_deliveries=13265_cells=13891_plasticity=462160
config_hash=c1-c7d2c86a2b1927f6 seed=11400784225994701844 condition=dense-local accuracy=0.550000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2666119.942213 note=wall_secs=0.0115_peak_rss=4407296_spikes=930_deliveries=61680_cells=62316_plasticity=1341440
config_hash=c1-c7d2c86a2b1927f6 seed=11400784225994701844 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6551_peak_rss=2916352_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-c7d2c86a2b1927f6 seed=11400784225994701844 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0398_peak_rss=2654208_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-c7d2c86a2b1927f6 seed=11400784225994701844 condition=dense-matched accuracy=0.450000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1213375.587699 note=wall_secs=0.0061_peak_rss=4030464_spikes=898_deliveries=41162_cells=41799_plasticity=462160
config_hash=c1-c7d2c86a2b1927f6 seed=4354472946875824171 condition=local-assembly accuracy=0.750000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=652796.000000 note=wall_secs=0.0050_peak_rss=3702784_spikes=703_deliveries=13131_cells=13763_plasticity=462000
config_hash=c1-c7d2c86a2b1927f6 seed=4354472946875824171 condition=dense-local accuracy=0.575000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2550140.922435 note=wall_secs=0.0095_peak_rss=4472832_spikes=897_deliveries=61680_cells=62314_plasticity=1341440
config_hash=c1-c7d2c86a2b1927f6 seed=4354472946875824171 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6588_peak_rss=2916352_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-c7d2c86a2b1927f6 seed=4354472946875824171 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0398_peak_rss=2621440_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-c7d2c86a2b1927f6 seed=4354472946875824171 condition=dense-matched accuracy=0.250000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2182976.000000 note=wall_secs=0.0057_peak_rss=4030464_spikes=889_deliveries=41111_cells=41744_plasticity=462000
config_hash=c1-c7d2c86a2b1927f6 seed=15755469790931547198 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=979064.000000 note=wall_secs=0.0048_peak_rss=3719168_spikes=669_deliveries=13155_cells=13788_plasticity=461920
config_hash=c1-c7d2c86a2b1927f6 seed=15755469790931547198 condition=dense-local accuracy=0.450000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=3258504.530765 note=wall_secs=0.0100_peak_rss=4472832_spikes=893_deliveries=61680_cells=62314_plasticity=1341440
config_hash=c1-c7d2c86a2b1927f6 seed=15755469790931547198 condition=gradient-reference accuracy=0.725000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=157139856.901371 note=wall_secs=0.6557_peak_rss=2932736_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-c7d2c86a2b1927f6 seed=15755469790931547198 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0400_peak_rss=2621440_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-c7d2c86a2b1927f6 seed=15755469790931547198 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1091290.000000 note=wall_secs=0.0061_peak_rss=4046848_spikes=895_deliveries=41098_cells=41732_plasticity=461920
config_hash=c1-c7d2c86a2b1927f6 seed=8709160710835925077 condition=local-assembly accuracy=0.900000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=545962.236685 note=wall_secs=0.0055_peak_rss=3620864_spikes=760_deliveries=13346_cells=13980_plasticity=463280
config_hash=c1-c7d2c86a2b1927f6 seed=8709160710835925077 condition=dense-local accuracy=0.450000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=3258597.864101 note=wall_secs=0.0095_peak_rss=4505600_spikes=929_deliveries=61680_cells=62320_plasticity=1341440
config_hash=c1-c7d2c86a2b1927f6 seed=8709160710835925077 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6591_peak_rss=2916352_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-c7d2c86a2b1927f6 seed=8709160710835925077 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0175_peak_rss=2621440_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-c7d2c86a2b1927f6 seed=8709160710835925077 condition=dense-matched accuracy=0.250000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2188916.000000 note=wall_secs=0.0055_peak_rss=4014080_spikes=919_deliveries=41195_cells=41835_plasticity=463280
config_hash=c1-c7d2c86a2b1927f6 seed=1663413756060003432 condition=local-assembly accuracy=0.900000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=545976.681130 note=wall_secs=0.0048_peak_rss=3686400_spikes=786_deliveries=13302_cells=13931_plasticity=463360
config_hash=c1-c7d2c86a2b1927f6 seed=1663413756060003432 condition=dense-local accuracy=0.375000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=3910266.666667 note=wall_secs=0.0096_peak_rss=4341760_spikes=915_deliveries=61680_cells=62315_plasticity=1341440
config_hash=c1-c7d2c86a2b1927f6 seed=1663413756060003432 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6559_peak_rss=2850816_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-c7d2c86a2b1927f6 seed=1663413756060003432 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0151_peak_rss=2621440_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-c7d2c86a2b1927f6 seed=1663413756060003432 condition=dense-matched accuracy=0.600000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=911748.297104 note=wall_secs=0.0055_peak_rss=4046848_spikes=896_deliveries=41079_cells=41714_plasticity=463360
config_hash=c1-c7d2c86a2b1927f6 seed=13063846550650677375 condition=local-assembly accuracy=1.000000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=489982.000000 note=wall_secs=0.0050_peak_rss=3702784_spikes=786_deliveries=13322_cells=13954_plasticity=461920
config_hash=c1-c7d2c86a2b1927f6 seed=13063846550650677375 condition=dense-local accuracy=0.575000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2550219.183306 note=wall_secs=0.0096_peak_rss=4390912_spikes=931_deliveries=61680_cells=62325_plasticity=1341440
config_hash=c1-c7d2c86a2b1927f6 seed=13063846550650677375 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6579_peak_rss=2850816_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-c7d2c86a2b1927f6 seed=13063846550650677375 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0151_peak_rss=2605056_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-c7d2c86a2b1927f6 seed=13063846550650677375 condition=dense-matched accuracy=0.550000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=992189.069404 note=wall_secs=0.0056_peak_rss=4079616_spikes=924_deliveries=41107_cells=41753_plasticity=461920
config_hash=c1-c7d2c86a2b1927f6 seed=6018099320996848786 condition=local-assembly accuracy=1.000000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=490275.000000 note=wall_secs=0.0049_peak_rss=3702784_spikes=799_deliveries=13501_cells=14135_plasticity=461840
config_hash=c1-c7d2c86a2b1927f6 seed=6018099320996848786 condition=dense-local accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=3450237.550282 note=wall_secs=0.0100_peak_rss=4472832_spikes=907_deliveries=61680_cells=62324_plasticity=1341440
config_hash=c1-c7d2c86a2b1927f6 seed=6018099320996848786 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6554_peak_rss=2834432_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-c7d2c86a2b1927f6 seed=6018099320996848786 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0401_peak_rss=2621440_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-c7d2c86a2b1927f6 seed=6018099320996848786 condition=dense-matched accuracy=0.625000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=872936.000000 note=wall_secs=0.0057_peak_rss=3981312_spikes=898_deliveries=41101_cells=41746_plasticity=461840
config_hash=c1-c7d2c86a2b1927f6 seed=17418529916564267177 condition=local-assembly accuracy=0.550000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=892341.798841 note=wall_secs=0.0049_peak_rss=3653632_spikes=779_deliveries=13484_cells=14125_plasticity=462400
config_hash=c1-c7d2c86a2b1927f6 seed=17418529916564267177 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2932774.000000 note=wall_secs=0.0097_peak_rss=4472832_spikes=941_deliveries=61680_cells=62326_plasticity=1341440
config_hash=c1-c7d2c86a2b1927f6 seed=17418529916564267177 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6585_peak_rss=2850816_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-c7d2c86a2b1927f6 seed=17418529916564267177 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0153_peak_rss=2654208_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-c7d2c86a2b1927f6 seed=17418529916564267177 condition=dense-matched accuracy=0.625000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=873691.200000 note=wall_secs=0.0057_peak_rss=4063232_spikes=921_deliveries=41044_cells=41692_plasticity=462400
config_hash=c1-c7d2c86a2b1927f6 seed=10372782686910438588 condition=local-assembly accuracy=0.825000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=594850.917686 note=wall_secs=0.0048_peak_rss=3719168_spikes=809_deliveries=13415_cells=14048_plasticity=462480
config_hash=c1-c7d2c86a2b1927f6 seed=10372782686910438588 condition=dense-local accuracy=0.450000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=3258535.641877 note=wall_secs=0.0096_peak_rss=4407296_spikes=905_deliveries=61680_cells=62316_plasticity=1341440
config_hash=c1-c7d2c86a2b1927f6 seed=10372782686910438588 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6548_peak_rss=2850816_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-c7d2c86a2b1927f6 seed=10372782686910438588 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0395_peak_rss=2637824_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-c7d2c86a2b1927f6 seed=10372782686910438588 condition=dense-matched accuracy=0.525000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1040163.856761 note=wall_secs=0.0056_peak_rss=4046848_spikes=900_deliveries=41035_cells=41671_plasticity=462480
config_hash=c1-c7d2c86a2b1927f6 seed=3326471682669467859 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=981216.000000 note=wall_secs=0.0050_peak_rss=3702784_spikes=843_deliveries=13440_cells=14085_plasticity=462240
config_hash=c1-c7d2c86a2b1927f6 seed=3326471682669467859 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2932654.000000 note=wall_secs=0.0098_peak_rss=4374528_spikes=884_deliveries=61680_cells=62323_plasticity=1341440
config_hash=c1-c7d2c86a2b1927f6 seed=3326471682669467859 condition=gradient-reference accuracy=0.725000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=157139856.901371 note=wall_secs=0.6548_peak_rss=2834432_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-c7d2c86a2b1927f6 seed=3326471682669467859 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0159_peak_rss=2621440_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-c7d2c86a2b1927f6 seed=3326471682669467859 condition=dense-matched accuracy=0.475000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1149275.803895 note=wall_secs=0.0058_peak_rss=3997696_spikes=897_deliveries=41063_cells=41706_plasticity=462240
config_hash=c1-c7d2c86a2b1927f6 seed=14727610363725173990 condition=local-assembly accuracy=0.375000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1307554.666667 note=wall_secs=0.0051_peak_rss=3719168_spikes=715_deliveries=13248_cells=13890_plasticity=462480
config_hash=c1-c7d2c86a2b1927f6 seed=14727610363725173990 condition=dense-local accuracy=0.475000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=3087128.459791 note=wall_secs=0.0097_peak_rss=4472832_spikes=941_deliveries=61680_cells=62325_plasticity=1341440
config_hash=c1-c7d2c86a2b1927f6 seed=14727610363725173990 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6887_peak_rss=3014656_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-c7d2c86a2b1927f6 seed=14727610363725173990 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0435_peak_rss=2621440_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-c7d2c86a2b1927f6 seed=14727610363725173990 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1091830.000000 note=wall_secs=0.0059_peak_rss=4096000_spikes=920_deliveries=40935_cells=41580_plasticity=462480
config_hash=c1-c7d2c86a2b1927f6 seed=7681300184117924093 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=977522.000000 note=wall_secs=0.0049_peak_rss=3768320_spikes=839_deliveries=13199_cells=13843_plasticity=460880
config_hash=c1-c7d2c86a2b1927f6 seed=7681300184117924093 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2932712.000000 note=wall_secs=0.0107_peak_rss=4505600_spikes=915_deliveries=61680_cells=62321_plasticity=1341440
config_hash=c1-c7d2c86a2b1927f6 seed=7681300184117924093 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6580_peak_rss=2916352_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-c7d2c86a2b1927f6 seed=7681300184117924093 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0160_peak_rss=2621440_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-c7d2c86a2b1927f6 seed=7681300184117924093 condition=dense-matched accuracy=0.350000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1556360.026505 note=wall_secs=0.0059_peak_rss=4063232_spikes=915_deliveries=41145_cells=41786_plasticity=460880
config_hash=c1-c7d2c86a2b1927f6 seed=635551854952467728 condition=local-assembly accuracy=1.000000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=489847.000000 note=wall_secs=0.0050_peak_rss=3719168_spikes=771_deliveries=13386_cells=14010_plasticity=461680
config_hash=c1-c7d2c86a2b1927f6 seed=635551854952467728 condition=dense-local accuracy=0.450000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=3258553.419656 note=wall_secs=0.0096_peak_rss=4489216_spikes=910_deliveries=61680_cells=62319_plasticity=1341440
config_hash=c1-c7d2c86a2b1927f6 seed=635551854952467728 condition=gradient-reference accuracy=0.500000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=227852800.000000 note=wall_secs=0.6593_peak_rss=2932736_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-c7d2c86a2b1927f6 seed=635551854952467728 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0150_peak_rss=2621440_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-c7d2c86a2b1927f6 seed=635551854952467728 condition=dense-matched accuracy=0.550000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=991559.978509 note=wall_secs=0.0057_peak_rss=3997696_spikes=895_deliveries=41073_cells=41710_plasticity=461680
config_hash=c1-c7d2c86a2b1927f6 seed=12035985749054769447 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=984138.000000 note=wall_secs=0.0049_peak_rss=3735552_spikes=761_deliveries=13499_cells=14129_plasticity=463680
config_hash=c1-c7d2c86a2b1927f6 seed=12035985749054769447 condition=dense-local accuracy=0.575000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2550139.183304 note=wall_secs=0.0098_peak_rss=4472832_spikes=893_deliveries=61680_cells=62317_plasticity=1341440
config_hash=c1-c7d2c86a2b1927f6 seed=12035985749054769447 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6547_peak_rss=2834432_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-c7d2c86a2b1927f6 seed=12035985749054769447 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0150_peak_rss=2621440_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-c7d2c86a2b1927f6 seed=12035985749054769447 condition=dense-matched accuracy=0.550000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=995603.614785 note=wall_secs=0.0056_peak_rss=4063232_spikes=893_deliveries=41187_cells=41822_plasticity=463680
config_hash=c1-c7d2c86a2b1927f6 seed=4990235495743964474 condition=local-assembly accuracy=1.000000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=490972.000000 note=wall_secs=0.0052_peak_rss=3719168_spikes=779_deliveries=13414_cells=14059_plasticity=462720
config_hash=c1-c7d2c86a2b1927f6 seed=4990235495743964474 condition=dense-local accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=3450249.314988 note=wall_secs=0.0097_peak_rss=4489216_spikes=914_deliveries=61680_cells=62322_plasticity=1341440
config_hash=c1-c7d2c86a2b1927f6 seed=4990235495743964474 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6558_peak_rss=2818048_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-c7d2c86a2b1927f6 seed=4990235495743964474 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0397_peak_rss=2605056_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-c7d2c86a2b1927f6 seed=4990235495743964474 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1092962.000000 note=wall_secs=0.0059_peak_rss=3981312_spikes=907_deliveries=41106_cells=41748_plasticity=462720
config_hash=c1-c7d2c86a2b1927f6 seed=16390669389846266193 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=981868.000000 note=wall_secs=0.0049_peak_rss=3735552_spikes=809_deliveries=13382_cells=14023_plasticity=462720
config_hash=c1-c7d2c86a2b1927f6 seed=16390669389846266193 condition=dense-local accuracy=0.475000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=3087132.670317 note=wall_secs=0.0095_peak_rss=4456448_spikes=946_deliveries=61680_cells=62322_plasticity=1341440
config_hash=c1-c7d2c86a2b1927f6 seed=16390669389846266193 condition=gradient-reference accuracy=0.500000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=227852800.000000 note=wall_secs=0.6547_peak_rss=2834432_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-c7d2c86a2b1927f6 seed=16390669389846266193 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0176_peak_rss=2621440_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-c7d2c86a2b1927f6 seed=16390669389846266193 condition=dense-matched accuracy=0.450000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1214326.698835 note=wall_secs=0.0055_peak_rss=4063232_spikes=937_deliveries=41074_cells=41716_plasticity=462720
config_hash=c1-c7d2c86a2b1927f6 seed=9344921060680809828 condition=local-assembly accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=675788.943294 note=wall_secs=0.0050_peak_rss=3702784_spikes=776_deliveries=13307_cells=13944_plasticity=461920
config_hash=c1-c7d2c86a2b1927f6 seed=9344921060680809828 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2932692.000000 note=wall_secs=0.0099_peak_rss=4390912_spikes=910_deliveries=61680_cells=62316_plasticity=1341440
config_hash=c1-c7d2c86a2b1927f6 seed=9344921060680809828 condition=gradient-reference accuracy=0.725000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=157139856.901371 note=wall_secs=0.6631_peak_rss=2834432_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-c7d2c86a2b1927f6 seed=9344921060680809828 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0178_peak_rss=2621440_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-c7d2c86a2b1927f6 seed=9344921060680809828 condition=dense-matched accuracy=0.675000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=808117.022765 note=wall_secs=0.0057_peak_rss=4046848_spikes=907_deliveries=41009_cells=41643_plasticity=461920
config_hash=c1-c7d2c86a2b1927f6 seed=2298610881073559931 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=980236.000000 note=wall_secs=0.0049_peak_rss=3735552_spikes=808_deliveries=13218_cells=13852_plasticity=462240
config_hash=c1-c7d2c86a2b1927f6 seed=2298610881073559931 condition=dense-local accuracy=0.450000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=3258540.086322 note=wall_secs=0.0097_peak_rss=4456448_spikes=902_deliveries=61680_cells=62321_plasticity=1341440
config_hash=c1-c7d2c86a2b1927f6 seed=2298610881073559931 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6583_peak_rss=2850816_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-c7d2c86a2b1927f6 seed=2298610881073559931 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0401_peak_rss=2637824_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-c7d2c86a2b1927f6 seed=2298610881073559931 condition=dense-matched accuracy=0.450000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1213248.921029 note=wall_secs=0.0059_peak_rss=4030464_spikes=895_deliveries=41093_cells=41734_plasticity=462240
config_hash=c1-c7d2c86a2b1927f6 seed=13699608824640910734 condition=local-assembly accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=676009.632942 note=wall_secs=0.0051_peak_rss=3735552_spikes=812_deliveries=13133_cells=13762_plasticity=462400
config_hash=c1-c7d2c86a2b1927f6 seed=13699608824640910734 condition=dense-local accuracy=0.625000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2346152.000000 note=wall_secs=0.0095_peak_rss=4489216_spikes=906_deliveries=61680_cells=62319_plasticity=1341440
config_hash=c1-c7d2c86a2b1927f6 seed=13699608824640910734 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6598_peak_rss=2932736_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-c7d2c86a2b1927f6 seed=13699608824640910734 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0403_peak_rss=2654208_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-c7d2c86a2b1927f6 seed=13699608824640910734 condition=dense-matched accuracy=0.550000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=993369.069378 note=wall_secs=0.0060_peak_rss=4063232_spikes=894_deliveries=41210_cells=41849_plasticity=462400
config_hash=c1-c7d2c86a2b1927f6 seed=6653297820399940005 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=980604.000000 note=wall_secs=0.0053_peak_rss=3719168_spikes=708_deliveries=13234_cells=13880_plasticity=462480
config_hash=c1-c7d2c86a2b1927f6 seed=6653297820399940005 condition=dense-local accuracy=0.575000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2550165.270261 note=wall_secs=0.0101_peak_rss=4407296_spikes=900_deliveries=61680_cells=62325_plasticity=1341440
config_hash=c1-c7d2c86a2b1927f6 seed=6653297820399940005 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6583_peak_rss=2850816_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-c7d2c86a2b1927f6 seed=6653297820399940005 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0398_peak_rss=2621440_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-c7d2c86a2b1927f6 seed=6653297820399940005 condition=dense-matched accuracy=0.600000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=910346.630493 note=wall_secs=0.0059_peak_rss=4063232_spikes=903_deliveries=41090_cells=41735_plasticity=462480
```
