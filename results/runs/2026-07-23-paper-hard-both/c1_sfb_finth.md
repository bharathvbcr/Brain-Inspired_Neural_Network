# C1 / Gate G2 results note

**Config hash:** `c1-4bbaf4b24c2d1da2`

**Scientific protocol version:** `23`

**claim_axis:** Integrity
**object_under_test:** θ=∞ mute confounder under structured-B credit
**may_claim:** Turning mute off (finite θ) under SFB changes / does not change G2
**must_not_claim:** Spike-PC remassage; biology; remassage v15

**Finite-θ under SFB protocol:** `23` — v15 structured hidden `B` with **finite θ during integrate** (no θ=∞ mute) + trial-isolation resets; **positive control stays on broadcast ±1**; does **not** remassage v15 or reopen protocol-v2 `c1-118207fbc3eaba53`.

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
Config { experiment: "c1-sfb-finth", master_seed: 212618061021185, n_seeds: 20, sequence_len: 8, max_lag: 1, n_hidden: 128, k_wta: 2, p_sparse: 0.35, init_w: 0.15, eta: 0.35, lambda: 0.002, tau_e: 40.0, n_train: 80, n_test: 40, bptt_epochs: 80, bptt_lr: 0.05, g2_min_gap_closed: 0.5, g2_min_accuracy: 0.65, g2_confidence_z: 1.96, g2_min_positive_control: 0.9, g2_min_reference_gap: 0.15, activity_sparsity_min: 0.005, activity_sparsity_max: 0.03, scientific_n_seeds: 20, power_sigma_prior: 0.15, power_effect_size: 0.1, use_surrogate_lif_reference: true, surrogate_beta: 5.0, matched_budget_repeat: true, quick: false }
```

Power analysis: required scientific n_seeds ≥ 20 (preregistered σ=0.150, effect=0.100; formula n=⌈(1.96+0.8416)²σ²/δ²⌉).

## Per-seed accuracies

| seed | local-assembly | dense-local | gradient reference | eligibility reference | activity_sparsity (local) | activity_sparsity (dense) | dense_matched |
|---|---:|---:|---:|---:|---:|---:|---:|
| 11400784225994701844 | 0.9000 | 0.5000 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 4354472946875824171 | 0.6000 | 0.5000 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 15755469790931547198 | 0.6500 | 0.5000 | 0.7250 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 8709160710835925077 | 0.7250 | 0.5000 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 1663413756060003432 | 0.7250 | 0.5000 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 13063846550650677375 | 0.7250 | 0.5000 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 6018099320996848786 | 0.7250 | 0.5000 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 17418529916564267177 | 0.7500 | 0.5000 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 10372782686910438588 | 0.6000 | 0.5000 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 3326471682669467859 | 0.7250 | 0.5000 | 0.7250 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 14727610363725173990 | 0.1250 | 0.5000 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 7681300184117924093 | 0.5000 | 0.5000 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 635551854952467728 | 0.8250 | 0.5000 | 0.5000 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 12035985749054769447 | 0.5000 | 0.5000 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 4990235495743964474 | 0.5000 | 0.5000 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 16390669389846266193 | 0.7250 | 0.5000 | 0.5000 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 9344921060680809828 | 0.7250 | 0.5000 | 0.7250 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 2298610881073559931 | 0.9000 | 0.5000 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 13699608824640910734 | 1.0000 | 0.5000 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 6653297820399940005 | 0.3500 | 0.5000 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |

## Summary (paired normalized-gap analysis)

- mean ± var local-assembly: 0.6638 ± 0.040097
- mean ± var dense-local:    0.5000 ± 0.000000
- mean ± var gradient reference: 0.8938 ± 0.027163
- mean ± var eligibility reference: 1.0000 ± 0.000000
- mean normalized gap closed: 0.4019 (variance 0.141586)
- lower confidence bound (z=1.960, n=20): 0.2370
- mean |local − dense| (descriptive): 0.2163
- descriptive chance-normalized gap mean / LCB: 0.4019 / 0.2370 (var 0.141586; **not a gate**)
- seed local min / max / frac≥0.65: 0.1250 / 1.0000 / 0.65

## U-NEG

Negative result: local-assembly did **not** clear the preregistered normalized-gap confidence and absolute-accuracy gates. Program stops at G2; do not schedule P3+.

## Positive / sanity control

Mean local-pipeline accuracy on a trivially separable spatial feature-presence task: **0.9488** (threshold 0.900).

## Activity sparsity

Mean local-assembly activity_sparsity: **0.0156** (valid band [0.0050, 0.0300]; nominal k/N=0.0156).

## Parameter / compute budgets

| condition | n_cells | n_params | wall_secs | peak_rss_bytes | work_per_accuracy | spikes | deliveries | cell_updates | plasticity |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| local-assembly | 132 | 5777 | 0.0049 | 3555328 | 544605.5700 | 852 | 13252 | 13881 | 462160 |
| dense-local | 132 | 16768 | 0.0094 | 4489216 | 2932802.0000 | 964 | 61680 | 62317 | 1341440 |
| gradient-reference | 130 | 16769 | 0.6783 | 2916352 | 113926400.0000 | 0 | 51200 | 6553600 | 107321600 |
| eligibility-reference | 130 | 385 | 0.0153 | 2605056 | 9068800.0000 | 0 | 51200 | 6553600 | 2464000 |
| dense-matched | 132 | 5777 | 0.0057 | 3899392 | 1090530.0000 | 964 | 40752 | 41389 | 462160 |

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
config_hash=c1-4bbaf4b24c2d1da2 seed=11400784225994701844 condition=local-assembly accuracy=0.900000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=544605.569983 note=wall_secs=0.0049_peak_rss=3555328_spikes=852_deliveries=13252_cells=13881_plasticity=462160
config_hash=c1-4bbaf4b24c2d1da2 seed=11400784225994701844 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2932802.000000 note=wall_secs=0.0094_peak_rss=4489216_spikes=964_deliveries=61680_cells=62317_plasticity=1341440
config_hash=c1-4bbaf4b24c2d1da2 seed=11400784225994701844 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6783_peak_rss=2916352_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-4bbaf4b24c2d1da2 seed=11400784225994701844 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0153_peak_rss=2605056_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-4bbaf4b24c2d1da2 seed=11400784225994701844 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1090530.000000 note=wall_secs=0.0057_peak_rss=3899392_spikes=964_deliveries=40752_cells=41389_plasticity=462160
config_hash=c1-4bbaf4b24c2d1da2 seed=4354472946875824171 condition=local-assembly accuracy=0.600000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=825618.300526 note=wall_secs=0.0049_peak_rss=3571712_spikes=842_deliveries=15949_cells=16580_plasticity=462000
config_hash=c1-4bbaf4b24c2d1da2 seed=4354472946875824171 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2932686.000000 note=wall_secs=0.0096_peak_rss=4538368_spikes=906_deliveries=61680_cells=62317_plasticity=1341440
config_hash=c1-4bbaf4b24c2d1da2 seed=4354472946875824171 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6670_peak_rss=2899968_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-4bbaf4b24c2d1da2 seed=4354472946875824171 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0176_peak_rss=2621440_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-4bbaf4b24c2d1da2 seed=4354472946875824171 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1093410.000000 note=wall_secs=0.0054_peak_rss=4128768_spikes=906_deliveries=41581_cells=42218_plasticity=462000
config_hash=c1-4bbaf4b24c2d1da2 seed=15755469790931547198 condition=local-assembly accuracy=0.650000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=753064.643007 note=wall_secs=0.0051_peak_rss=3571712_spikes=693_deliveries=13123_cells=13756_plasticity=461920
config_hash=c1-4bbaf4b24c2d1da2 seed=15755469790931547198 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2932718.000000 note=wall_secs=0.0100_peak_rss=4423680_spikes=923_deliveries=61680_cells=62316_plasticity=1341440
config_hash=c1-4bbaf4b24c2d1da2 seed=15755469790931547198 condition=gradient-reference accuracy=0.725000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=157139856.901371 note=wall_secs=0.6890_peak_rss=3014656_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-4bbaf4b24c2d1da2 seed=15755469790931547198 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0234_peak_rss=2670592_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-4bbaf4b24c2d1da2 seed=15755469790931547198 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1089094.000000 note=wall_secs=0.0054_peak_rss=4128768_spikes=923_deliveries=40534_cells=41170_plasticity=461920
config_hash=c1-4bbaf4b24c2d1da2 seed=8709160710835925077 condition=local-assembly accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=677696.529438 note=wall_secs=0.0072_peak_rss=3702784_spikes=751_deliveries=13332_cells=13967_plasticity=463280
config_hash=c1-4bbaf4b24c2d1da2 seed=8709160710835925077 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2932672.000000 note=wall_secs=0.0135_peak_rss=4734976_spikes=895_deliveries=61680_cells=62321_plasticity=1341440
config_hash=c1-4bbaf4b24c2d1da2 seed=8709160710835925077 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6823_peak_rss=2998272_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-4bbaf4b24c2d1da2 seed=8709160710835925077 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0159_peak_rss=2654208_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-4bbaf4b24c2d1da2 seed=8709160710835925077 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1096700.000000 note=wall_secs=0.0100_peak_rss=4161536_spikes=895_deliveries=41767_cells=42408_plasticity=463280
config_hash=c1-4bbaf4b24c2d1da2 seed=1663413756060003432 condition=local-assembly accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=678165.494940 note=wall_secs=0.0048_peak_rss=3702784_spikes=827_deliveries=13426_cells=14057_plasticity=463360
config_hash=c1-4bbaf4b24c2d1da2 seed=1663413756060003432 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2932662.000000 note=wall_secs=0.0094_peak_rss=4390912_spikes=898_deliveries=61680_cells=62313_plasticity=1341440
config_hash=c1-4bbaf4b24c2d1da2 seed=1663413756060003432 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6736_peak_rss=2916352_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-4bbaf4b24c2d1da2 seed=1663413756060003432 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0150_peak_rss=2637824_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-4bbaf4b24c2d1da2 seed=1663413756060003432 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1093594.000000 note=wall_secs=0.0057_peak_rss=3932160_spikes=898_deliveries=40953_cells=41586_plasticity=463360
config_hash=c1-4bbaf4b24c2d1da2 seed=13063846550650677375 condition=local-assembly accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=675837.219154 note=wall_secs=0.0048_peak_rss=3588096_spikes=786_deliveries=13320_cells=13956_plasticity=461920
config_hash=c1-4bbaf4b24c2d1da2 seed=13063846550650677375 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2932724.000000 note=wall_secs=0.0096_peak_rss=4489216_spikes=919_deliveries=61680_cells=62323_plasticity=1341440
config_hash=c1-4bbaf4b24c2d1da2 seed=13063846550650677375 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6712_peak_rss=2899968_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-4bbaf4b24c2d1da2 seed=13063846550650677375 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0158_peak_rss=2654208_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-4bbaf4b24c2d1da2 seed=13063846550650677375 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1091728.000000 note=wall_secs=0.0056_peak_rss=3981312_spikes=919_deliveries=41191_cells=41834_plasticity=461920
config_hash=c1-4bbaf4b24c2d1da2 seed=6018099320996848786 condition=local-assembly accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=684336.529220 note=wall_secs=0.0049_peak_rss=3555328_spikes=871_deliveries=16392_cells=17041_plasticity=461840
config_hash=c1-4bbaf4b24c2d1da2 seed=6018099320996848786 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2932770.000000 note=wall_secs=0.0095_peak_rss=4538368_spikes=941_deliveries=61680_cells=62324_plasticity=1341440
config_hash=c1-4bbaf4b24c2d1da2 seed=6018099320996848786 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6712_peak_rss=2801664_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-4bbaf4b24c2d1da2 seed=6018099320996848786 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0155_peak_rss=2654208_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-4bbaf4b24c2d1da2 seed=6018099320996848786 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1089142.000000 note=wall_secs=0.0057_peak_rss=3981312_spikes=941_deliveries=40573_cells=41217_plasticity=461840
config_hash=c1-4bbaf4b24c2d1da2 seed=17418529916564267177 condition=local-assembly accuracy=0.750000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=654176.000000 note=wall_secs=0.0051_peak_rss=3686400_spikes=724_deliveries=13435_cells=14073_plasticity=462400
config_hash=c1-4bbaf4b24c2d1da2 seed=17418529916564267177 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2932704.000000 note=wall_secs=0.0097_peak_rss=4390912_spikes=904_deliveries=61680_cells=62328_plasticity=1341440
config_hash=c1-4bbaf4b24c2d1da2 seed=17418529916564267177 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6723_peak_rss=2916352_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-4bbaf4b24c2d1da2 seed=17418529916564267177 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0151_peak_rss=2605056_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-4bbaf4b24c2d1da2 seed=17418529916564267177 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1092328.000000 note=wall_secs=0.0057_peak_rss=3899392_spikes=904_deliveries=41106_cells=41754_plasticity=462400
config_hash=c1-4bbaf4b24c2d1da2 seed=10372782686910438588 condition=local-assembly accuracy=0.600000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=829954.967021 note=wall_secs=0.0051_peak_rss=3751936_spikes=964_deliveries=16948_cells=17581_plasticity=462480
config_hash=c1-4bbaf4b24c2d1da2 seed=10372782686910438588 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2932790.000000 note=wall_secs=0.0098_peak_rss=4374528_spikes=959_deliveries=61680_cells=62316_plasticity=1341440
config_hash=c1-4bbaf4b24c2d1da2 seed=10372782686910438588 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6691_peak_rss=2899968_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-4bbaf4b24c2d1da2 seed=10372782686910438588 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0150_peak_rss=2605056_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-4bbaf4b24c2d1da2 seed=10372782686910438588 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1090962.000000 note=wall_secs=0.0055_peak_rss=3899392_spikes=959_deliveries=40703_cells=41339_plasticity=462480
config_hash=c1-4bbaf4b24c2d1da2 seed=3326471682669467859 condition=local-assembly accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=676623.426025 note=wall_secs=0.0047_peak_rss=3719168_spikes=800_deliveries=13438_cells=14074_plasticity=462240
config_hash=c1-4bbaf4b24c2d1da2 seed=3326471682669467859 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2932708.000000 note=wall_secs=0.0094_peak_rss=4554752_spikes=911_deliveries=61680_cells=62323_plasticity=1341440
config_hash=c1-4bbaf4b24c2d1da2 seed=3326471682669467859 condition=gradient-reference accuracy=0.725000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=157139856.901371 note=wall_secs=0.6667_peak_rss=2916352_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-4bbaf4b24c2d1da2 seed=3326471682669467859 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0403_peak_rss=2654208_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-4bbaf4b24c2d1da2 seed=3326471682669467859 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1091756.000000 note=wall_secs=0.0056_peak_rss=4079616_spikes=911_deliveries=41042_cells=41685_plasticity=462240
config_hash=c1-4bbaf4b24c2d1da2 seed=14727610363725173990 condition=local-assembly accuracy=0.125000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=3971792.000000 note=wall_secs=0.0051_peak_rss=3555328_spikes=822_deliveries=16264_cells=16908_plasticity=462480
config_hash=c1-4bbaf4b24c2d1da2 seed=14727610363725173990 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2932688.000000 note=wall_secs=0.0099_peak_rss=4358144_spikes=900_deliveries=61680_cells=62324_plasticity=1341440
config_hash=c1-4bbaf4b24c2d1da2 seed=14727610363725173990 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6703_peak_rss=2998272_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-4bbaf4b24c2d1da2 seed=14727610363725173990 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0155_peak_rss=2654208_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-4bbaf4b24c2d1da2 seed=14727610363725173990 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1092804.000000 note=wall_secs=0.0056_peak_rss=4014080_spikes=900_deliveries=41189_cells=41833_plasticity=462480
config_hash=c1-4bbaf4b24c2d1da2 seed=7681300184117924093 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=977504.000000 note=wall_secs=0.0050_peak_rss=3702784_spikes=829_deliveries=13200_cells=13843_plasticity=460880
config_hash=c1-4bbaf4b24c2d1da2 seed=7681300184117924093 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2932742.000000 note=wall_secs=0.0099_peak_rss=4751360_spikes=931_deliveries=61680_cells=62320_plasticity=1341440
config_hash=c1-4bbaf4b24c2d1da2 seed=7681300184117924093 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6726_peak_rss=2899968_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-4bbaf4b24c2d1da2 seed=7681300184117924093 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0178_peak_rss=2637824_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-4bbaf4b24c2d1da2 seed=7681300184117924093 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1088922.000000 note=wall_secs=0.0058_peak_rss=4128768_spikes=931_deliveries=41005_cells=41645_plasticity=460880
config_hash=c1-4bbaf4b24c2d1da2 seed=635551854952467728 condition=local-assembly accuracy=0.825000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=601143.038989 note=wall_secs=0.0050_peak_rss=3620864_spikes=897_deliveries=16368_cells=16998_plasticity=461680
config_hash=c1-4bbaf4b24c2d1da2 seed=635551854952467728 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2932636.000000 note=wall_secs=0.0097_peak_rss=4390912_spikes=883_deliveries=61680_cells=62315_plasticity=1341440
config_hash=c1-4bbaf4b24c2d1da2 seed=635551854952467728 condition=gradient-reference accuracy=0.500000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=227852800.000000 note=wall_secs=0.6693_peak_rss=2883584_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-4bbaf4b24c2d1da2 seed=635551854952467728 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0151_peak_rss=2637824_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-4bbaf4b24c2d1da2 seed=635551854952467728 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1091368.000000 note=wall_secs=0.0055_peak_rss=4128768_spikes=883_deliveries=41243_cells=41878_plasticity=461680
config_hash=c1-4bbaf4b24c2d1da2 seed=12035985749054769447 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=984530.000000 note=wall_secs=0.0051_peak_rss=3571712_spikes=850_deliveries=13550_cells=14185_plasticity=463680
config_hash=c1-4bbaf4b24c2d1da2 seed=12035985749054769447 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2932658.000000 note=wall_secs=0.0094_peak_rss=4374528_spikes=893_deliveries=61680_cells=62316_plasticity=1341440
config_hash=c1-4bbaf4b24c2d1da2 seed=12035985749054769447 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6668_peak_rss=2899968_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-4bbaf4b24c2d1da2 seed=12035985749054769447 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0160_peak_rss=2621440_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-4bbaf4b24c2d1da2 seed=12035985749054769447 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1093674.000000 note=wall_secs=0.0053_peak_rss=4014080_spikes=893_deliveries=40814_cells=41450_plasticity=463680
config_hash=c1-4bbaf4b24c2d1da2 seed=4990235495743964474 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=982260.000000 note=wall_secs=0.0049_peak_rss=3702784_spikes=838_deliveries=13463_cells=14109_plasticity=462720
config_hash=c1-4bbaf4b24c2d1da2 seed=4990235495743964474 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2932664.000000 note=wall_secs=0.0099_peak_rss=4734976_spikes=890_deliveries=61680_cells=62322_plasticity=1341440
config_hash=c1-4bbaf4b24c2d1da2 seed=4990235495743964474 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6774_peak_rss=2899968_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-4bbaf4b24c2d1da2 seed=4990235495743964474 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0156_peak_rss=2637824_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-4bbaf4b24c2d1da2 seed=4990235495743964474 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1092920.000000 note=wall_secs=0.0062_peak_rss=4128768_spikes=890_deliveries=41104_cells=41746_plasticity=462720
config_hash=c1-4bbaf4b24c2d1da2 seed=16390669389846266193 condition=local-assembly accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=683329.632701 note=wall_secs=0.0052_peak_rss=3588096_spikes=887_deliveries=15584_cells=16223_plasticity=462720
config_hash=c1-4bbaf4b24c2d1da2 seed=16390669389846266193 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2932684.000000 note=wall_secs=0.0096_peak_rss=4489216_spikes=899_deliveries=61680_cells=62323_plasticity=1341440
config_hash=c1-4bbaf4b24c2d1da2 seed=16390669389846266193 condition=gradient-reference accuracy=0.500000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=227852800.000000 note=wall_secs=0.6774_peak_rss=2883584_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-4bbaf4b24c2d1da2 seed=16390669389846266193 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0152_peak_rss=2637824_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-4bbaf4b24c2d1da2 seed=16390669389846266193 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1092824.000000 note=wall_secs=0.0057_peak_rss=4128768_spikes=899_deliveries=41075_cells=41718_plasticity=462720
config_hash=c1-4bbaf4b24c2d1da2 seed=9344921060680809828 condition=local-assembly accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=675759.977777 note=wall_secs=0.0047_peak_rss=3686400_spikes=770_deliveries=13301_cells=13935_plasticity=461920
config_hash=c1-4bbaf4b24c2d1da2 seed=9344921060680809828 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2932714.000000 note=wall_secs=0.0096_peak_rss=4538368_spikes=926_deliveries=61680_cells=62311_plasticity=1341440
config_hash=c1-4bbaf4b24c2d1da2 seed=9344921060680809828 condition=gradient-reference accuracy=0.725000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=157139856.901371 note=wall_secs=0.6694_peak_rss=2883584_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-4bbaf4b24c2d1da2 seed=9344921060680809828 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0401_peak_rss=2654208_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-4bbaf4b24c2d1da2 seed=9344921060680809828 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1091234.000000 note=wall_secs=0.0054_peak_rss=4128768_spikes=926_deliveries=41070_cells=41701_plasticity=461920
config_hash=c1-4bbaf4b24c2d1da2 seed=2298610881073559931 condition=local-assembly accuracy=0.900000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=551475.570165 note=wall_secs=0.0051_peak_rss=3735552_spikes=937_deliveries=16261_cells=16890_plasticity=462240
config_hash=c1-4bbaf4b24c2d1da2 seed=2298610881073559931 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2932738.000000 note=wall_secs=0.0099_peak_rss=4489216_spikes=928_deliveries=61680_cells=62321_plasticity=1341440
config_hash=c1-4bbaf4b24c2d1da2 seed=2298610881073559931 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6734_peak_rss=2998272_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-4bbaf4b24c2d1da2 seed=2298610881073559931 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0189_peak_rss=2621440_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-4bbaf4b24c2d1da2 seed=2298610881073559931 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1091234.000000 note=wall_secs=0.0054_peak_rss=4079616_spikes=928_deliveries=40904_cells=41545_plasticity=462240
config_hash=c1-4bbaf4b24c2d1da2 seed=13699608824640910734 condition=local-assembly accuracy=1.000000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=490086.000000 note=wall_secs=0.0053_peak_rss=3702784_spikes=797_deliveries=13130_cells=13759_plasticity=462400
config_hash=c1-4bbaf4b24c2d1da2 seed=13699608824640910734 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2932640.000000 note=wall_secs=0.0105_peak_rss=4653056_spikes=884_deliveries=61680_cells=62316_plasticity=1341440
config_hash=c1-4bbaf4b24c2d1da2 seed=13699608824640910734 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6761_peak_rss=2883584_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-4bbaf4b24c2d1da2 seed=13699608824640910734 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0152_peak_rss=2637824_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-4bbaf4b24c2d1da2 seed=13699608824640910734 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1093924.000000 note=wall_secs=0.0059_peak_rss=4046848_spikes=884_deliveries=41521_cells=42157_plasticity=462400
config_hash=c1-4bbaf4b24c2d1da2 seed=6653297820399940005 condition=local-assembly accuracy=0.350000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1400731.452426 note=wall_secs=0.0047_peak_rss=3555328_spikes=738_deliveries=13197_cells=13841_plasticity=462480
config_hash=c1-4bbaf4b24c2d1da2 seed=6653297820399940005 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2932722.000000 note=wall_secs=0.0097_peak_rss=4538368_spikes=914_deliveries=61680_cells=62327_plasticity=1341440
config_hash=c1-4bbaf4b24c2d1da2 seed=6653297820399940005 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6801_peak_rss=2883584_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-4bbaf4b24c2d1da2 seed=6653297820399940005 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0153_peak_rss=2637824_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-4bbaf4b24c2d1da2 seed=6653297820399940005 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1093670.000000 note=wall_secs=0.0056_peak_rss=4046848_spikes=914_deliveries=41397_cells=42044_plasticity=462480
```
