# C1 / Gate G2 results note

**Config hash:** `c1-09442acdbdc0c752`

**Scientific protocol version:** `6`

**Natural-hidden-spiking protocol:** `6` — finite hidden θ during integrate (no θ=∞ mute); applies trial-isolation membrane + STDP pairing resets; does **not** reopen protocol-v2 kill-gate hash `c1-118207fbc3eaba53` (canonical version `2`).

**Verdict (Gate G2):** **INVALID_HARNESS**

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
Config { experiment: "c1-spike", master_seed: 212592291217409, n_seeds: 20, sequence_len: 8, max_lag: 1, n_hidden: 128, k_wta: 2, p_sparse: 0.35, init_w: 0.15, eta: 0.35, lambda: 0.002, tau_e: 40.0, n_train: 80, n_test: 40, bptt_epochs: 80, bptt_lr: 0.05, g2_min_gap_closed: 0.5, g2_min_accuracy: 0.65, g2_confidence_z: 1.96, g2_min_positive_control: 0.9, g2_min_reference_gap: 0.15, activity_sparsity_min: 0.005, activity_sparsity_max: 0.03, scientific_n_seeds: 20, power_sigma_prior: 0.15, power_effect_size: 0.1, use_surrogate_lif_reference: true, surrogate_beta: 5.0, matched_budget_repeat: true, quick: false }
```

Power analysis: required scientific n_seeds ≥ 20 (preregistered σ=0.150, effect=0.100; formula n=⌈(1.96+0.8416)²σ²/δ²⌉).

## Per-seed accuracies

| seed | local-assembly | dense-local | gradient reference | eligibility reference | activity_sparsity (local) | activity_sparsity (dense) | dense_matched |
|---|---:|---:|---:|---:|---:|---:|---:|
| 11400784268944374804 | 0.5000 | 0.4250 | 0.7250 | 1.0000 | 0.0000 | 0.0156 | 0.4250 |
| 4354473041365104683 | 0.4250 | 0.4250 | 1.0000 | 1.0000 | 0.0000 | 0.0156 | 0.4250 |
| 15755469971320173630 | 0.5000 | 0.4250 | 0.7250 | 1.0000 | 0.0156 | 0.0156 | 0.4250 |
| 8709160959944028245 | 0.5000 | 0.4250 | 0.7250 | 1.0000 | 0.0121 | 0.0156 | 0.4250 |
| 1663413506951900264 | 0.5000 | 0.4250 | 1.0000 | 1.0000 | 0.0121 | 0.0156 | 0.4250 |
| 13063846318722443391 | 0.4250 | 0.4250 | 1.0000 | 1.0000 | 0.0000 | 0.0156 | 0.4250 |
| 6018099140608222354 | 0.5750 | 0.4250 | 0.9500 | 1.0000 | 0.0121 | 0.0156 | 0.4250 |
| 17418529804895117481 | 0.4250 | 0.4250 | 0.7250 | 1.0000 | 0.0000 | 0.0156 | 0.4250 |
| 10372782661140634812 | 0.5000 | 0.4250 | 1.0000 | 1.0000 | 0.0121 | 0.0156 | 0.4250 |
| 3326471708439271635 | 0.5000 | 0.4250 | 0.7250 | 1.0000 | 0.0156 | 0.0156 | 0.4250 |
| 14727610406674846950 | 0.5000 | 0.4250 | 1.0000 | 1.0000 | 0.0121 | 0.0156 | 0.4250 |
| 7681300295787073789 | 0.5000 | 0.4250 | 0.9500 | 1.0000 | 0.0121 | 0.0156 | 0.4250 |
| 635552018161224976 | 0.4250 | 0.4250 | 0.7250 | 1.0000 | 0.0000 | 0.0156 | 0.4250 |
| 12035985998162872615 | 0.5000 | 0.4250 | 0.7250 | 1.0000 | 0.0121 | 0.0156 | 0.4250 |
| 4990235246635861306 | 0.5000 | 0.4250 | 0.7250 | 1.0000 | 0.0156 | 0.0156 | 0.4250 |
| 16390669209457639761 | 0.5000 | 0.4250 | 1.0000 | 1.0000 | 0.0121 | 0.0156 | 0.4250 |
| 9344920897472052580 | 0.4250 | 0.4250 | 0.7250 | 1.0000 | 0.0000 | 0.0156 | 0.4250 |
| 2298610769404410235 | 0.5000 | 0.4250 | 0.9500 | 1.0000 | 0.0061 | 0.0156 | 0.4250 |
| 13699608798871106958 | 0.5000 | 0.4250 | 1.0000 | 1.0000 | 0.0121 | 0.0156 | 0.4250 |
| 6653297863349612965 | 0.5000 | 0.4250 | 0.5000 | 1.0000 | 0.0156 | 0.0156 | 0.4250 |

## Summary (paired normalized-gap analysis)

- mean ± var local-assembly: 0.4850 ± 0.001539
- mean ± var dense-local:    0.4250 ± 0.000000
- mean ± var gradient reference: 0.8438 ± 0.023676
- mean ± var eligibility reference: 1.0000 ± 0.000000
- mean normalized gap closed: 0.1362 (variance 0.011138)
- lower confidence bound (z=1.960, n=20): 0.0899
- mean |local − dense| (descriptive): 0.0600

## Invalid harness

Positive control and/or activity sparsity failed the preregistered validity gates. No scientific PASS/FAIL or U-NEG claim is permitted from this run.

## Positive / sanity control

Mean local-pipeline accuracy on a trivially separable spatial feature-presence task: **0.7738** (threshold 0.900).

## Activity sparsity

Mean local-assembly activity_sparsity: **0.0089** (valid band [0.0050, 0.0300]; nominal k/N=0.0156).

## Parameter / compute budgets

| condition | n_cells | n_params | wall_secs | peak_rss_bytes | work_per_accuracy | spikes | deliveries | cell_updates | plasticity |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| local-assembly | 132 | 5766 | 0.0329 | 37830656 | 8231684.0000 | 39262 | 1677638 | 1678192 | 720750 |
| dense-local | 132 | 16768 | 0.0132 | 4538368 | 5580875.1376 | 946 | 61680 | 62334 | 2246912 |
| gradient-reference | 130 | 16769 | 0.6916 | 2916352 | 157139856.9014 | 0 | 51200 | 6553600 | 107321600 |
| eligibility-reference | 130 | 385 | 0.0165 | 2539520 | 9068800.0000 | 0 | 51200 | 6553600 | 2464000 |
| dense-matched | 132 | 5766 | 0.0069 | 3866624 | 2015454.0611 | 946 | 41162 | 41816 | 772644 |

Matched-budget dense mean accuracy: **0.4250** (n=20; primary G2 gap still uses unmatched dense-local).

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
config_hash=c1-09442acdbdc0c752 seed=11400784268944374804 condition=local-assembly accuracy=0.500000 activity_sparsity=0.000000 activity-sparsity=0.000000 work_per_accuracy=8231684.000000 note=wall_secs=0.0329_peak_rss=37830656_spikes=39262_deliveries=1677638_cells=1678192_plasticity=720750
config_hash=c1-09442acdbdc0c752 seed=11400784268944374804 condition=dense-local accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5580875.137578 note=wall_secs=0.0132_peak_rss=4538368_spikes=946_deliveries=61680_cells=62334_plasticity=2246912
config_hash=c1-09442acdbdc0c752 seed=11400784268944374804 condition=gradient-reference accuracy=0.725000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=157139856.901371 note=wall_secs=0.6916_peak_rss=2916352_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-09442acdbdc0c752 seed=11400784268944374804 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0165_peak_rss=2539520_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-09442acdbdc0c752 seed=11400784268944374804 condition=dense-matched accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2015454.061115 note=wall_secs=0.0069_peak_rss=3866624_spikes=946_deliveries=41162_cells=41816_plasticity=772644
config_hash=c1-09442acdbdc0c752 seed=4354473041365104683 condition=local-assembly accuracy=0.425000 activity_sparsity=0.000000 activity-sparsity=0.000000 work_per_accuracy=1745981.127497 note=wall_secs=0.0057_peak_rss=3604480_spikes=556_deliveries=6235_cells=6719_plasticity=728532
config_hash=c1-09442acdbdc0c752 seed=4354473041365104683 condition=dense-local accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5580875.137578 note=wall_secs=0.0123_peak_rss=4423680_spikes=946_deliveries=61680_cells=62334_plasticity=2246912
config_hash=c1-09442acdbdc0c752 seed=4354473041365104683 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6767_peak_rss=2916352_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-09442acdbdc0c752 seed=4354473041365104683 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0159_peak_rss=2555904_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-09442acdbdc0c752 seed=4354473041365104683 condition=dense-matched accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2020225.825687 note=wall_secs=0.0067_peak_rss=3883008_spikes=946_deliveries=41104_cells=41758_plasticity=774788
config_hash=c1-09442acdbdc0c752 seed=15755469971320173630 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1464882.000000 note=wall_secs=0.0063_peak_rss=3620864_spikes=705_deliveries=13089_cells=13731_plasticity=704916
config_hash=c1-09442acdbdc0c752 seed=15755469971320173630 condition=dense-local accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5580875.137578 note=wall_secs=0.0134_peak_rss=4571136_spikes=946_deliveries=61680_cells=62334_plasticity=2246912
config_hash=c1-09442acdbdc0c752 seed=15755469971320173630 condition=gradient-reference accuracy=0.725000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=157139856.901371 note=wall_secs=0.6986_peak_rss=2932736_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-09442acdbdc0c752 seed=15755469971320173630 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0162_peak_rss=2555904_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-09442acdbdc0c752 seed=15755469971320173630 condition=dense-matched accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2019364.649241 note=wall_secs=0.0068_peak_rss=3899392_spikes=946_deliveries=41189_cells=41843_plasticity=774252
config_hash=c1-09442acdbdc0c752 seed=8709160959944028245 condition=local-assembly accuracy=0.500000 activity_sparsity=0.012109 activity-sparsity=0.012109 work_per_accuracy=26875762.000000 note=wall_secs=0.0987_peak_rss=59473920_spikes=148165_deliveries=6292888_cells=6293498_plasticity=703330
config_hash=c1-09442acdbdc0c752 seed=8709160959944028245 condition=dense-local accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5580875.137578 note=wall_secs=0.0122_peak_rss=4325376_spikes=946_deliveries=61680_cells=62334_plasticity=2246912
config_hash=c1-09442acdbdc0c752 seed=8709160959944028245 condition=gradient-reference accuracy=0.725000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=157139856.901371 note=wall_secs=0.6948_peak_rss=2899968_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-09442acdbdc0c752 seed=8709160959944028245 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0157_peak_rss=2555904_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-09442acdbdc0c752 seed=8709160959944028245 condition=dense-matched accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2012319.943556 note=wall_secs=0.0065_peak_rss=3883008_spikes=946_deliveries=40563_cells=41217_plasticity=772510
config_hash=c1-09442acdbdc0c752 seed=1663413506951900264 condition=local-assembly accuracy=0.500000 activity_sparsity=0.012109 activity-sparsity=0.012109 work_per_accuracy=43958338.000000 note=wall_secs=0.1288_peak_rss=26345472_spikes=236254_deliveries=10543907_cells=10544512_plasticity=654496
config_hash=c1-09442acdbdc0c752 seed=1663413506951900264 condition=dense-local accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5580875.137578 note=wall_secs=0.0122_peak_rss=4276224_spikes=946_deliveries=61680_cells=62334_plasticity=2246912
config_hash=c1-09442acdbdc0c752 seed=1663413506951900264 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.7404_peak_rss=2916352_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-09442acdbdc0c752 seed=1663413506951900264 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0191_peak_rss=2523136_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-09442acdbdc0c752 seed=1663413506951900264 condition=dense-matched accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2024202.296164 note=wall_secs=0.0066_peak_rss=3883008_spikes=946_deliveries=41279_cells=41933_plasticity=776128
config_hash=c1-09442acdbdc0c752 seed=13063846318722443391 condition=local-assembly accuracy=0.425000 activity_sparsity=0.000000 activity-sparsity=0.000000 work_per_accuracy=1737745.833610 note=wall_secs=0.0055_peak_rss=3424256_spikes=482_deliveries=4288_cells=4736_plasticity=729036
config_hash=c1-09442acdbdc0c752 seed=13063846318722443391 condition=dense-local accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5580875.137578 note=wall_secs=0.0125_peak_rss=4292608_spikes=946_deliveries=61680_cells=62334_plasticity=2246912
config_hash=c1-09442acdbdc0c752 seed=13063846318722443391 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.7082_peak_rss=2899968_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-09442acdbdc0c752 seed=13063846318722443391 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0159_peak_rss=2539520_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-09442acdbdc0c752 seed=13063846318722443391 condition=dense-matched accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2020070.531574 note=wall_secs=0.0073_peak_rss=3866624_spikes=946_deliveries=40803_cells=41457_plasticity=775324
config_hash=c1-09442acdbdc0c752 seed=6018099140608222354 condition=local-assembly accuracy=0.575000 activity_sparsity=0.012109 activity-sparsity=0.012109 work_per_accuracy=1343499.158288 note=wall_secs=0.0066_peak_rss=3702784_spikes=1543_deliveries=31984_cells=32605_plasticity=706380
config_hash=c1-09442acdbdc0c752 seed=6018099140608222354 condition=dense-local accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5580875.137578 note=wall_secs=0.0119_peak_rss=4292608_spikes=946_deliveries=61680_cells=62334_plasticity=2246912
config_hash=c1-09442acdbdc0c752 seed=6018099140608222354 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6784_peak_rss=2867200_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-09442acdbdc0c752 seed=6018099140608222354 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0158_peak_rss=2555904_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-09442acdbdc0c752 seed=6018099140608222354 condition=dense-matched accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2023882.296173 note=wall_secs=0.0061_peak_rss=3866624_spikes=946_deliveries=41345_cells=41999_plasticity=775860
config_hash=c1-09442acdbdc0c752 seed=17418529804895117481 condition=local-assembly accuracy=0.425000 activity_sparsity=0.000000 activity-sparsity=0.000000 work_per_accuracy=1723112.892844 note=wall_secs=0.0055_peak_rss=3571712_spikes=587_deliveries=6763_cells=7261_plasticity=717712
config_hash=c1-09442acdbdc0c752 seed=17418529804895117481 condition=dense-local accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5580875.137578 note=wall_secs=0.0119_peak_rss=4374528_spikes=946_deliveries=61680_cells=62334_plasticity=2246912
config_hash=c1-09442acdbdc0c752 seed=17418529804895117481 condition=gradient-reference accuracy=0.725000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=157139856.901371 note=wall_secs=0.6757_peak_rss=2850816_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-09442acdbdc0c752 seed=17418529804895117481 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0161_peak_rss=2539520_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-09442acdbdc0c752 seed=17418529804895117481 condition=dense-matched accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2023689.355002 note=wall_secs=0.0065_peak_rss=3997696_spikes=946_deliveries=41438_cells=42092_plasticity=775592
config_hash=c1-09442acdbdc0c752 seed=10372782661140634812 condition=local-assembly accuracy=0.500000 activity_sparsity=0.012109 activity-sparsity=0.012109 work_per_accuracy=12858198.000000 note=wall_secs=0.0401_peak_rss=23085056_spikes=71838_deliveries=2840075_cells=2840692_plasticity=676494
config_hash=c1-09442acdbdc0c752 seed=10372782661140634812 condition=dense-local accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5580875.137578 note=wall_secs=0.0119_peak_rss=4390912_spikes=946_deliveries=61680_cells=62334_plasticity=2246912
config_hash=c1-09442acdbdc0c752 seed=10372782661140634812 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6762_peak_rss=2867200_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-09442acdbdc0c752 seed=10372782661140634812 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0157_peak_rss=2555904_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-09442acdbdc0c752 seed=10372782661140634812 condition=dense-matched accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2018428.178679 note=wall_secs=0.0067_peak_rss=3964928_spikes=946_deliveries=40722_cells=41376_plasticity=774788
config_hash=c1-09442acdbdc0c752 seed=3326471708439271635 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1484366.000000 note=wall_secs=0.0058_peak_rss=3588096_spikes=921_deliveries=17791_cells=18433_plasticity=705038
config_hash=c1-09442acdbdc0c752 seed=3326471708439271635 condition=dense-local accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5580875.137578 note=wall_secs=0.0117_peak_rss=4325376_spikes=946_deliveries=61680_cells=62334_plasticity=2246912
config_hash=c1-09442acdbdc0c752 seed=3326471708439271635 condition=gradient-reference accuracy=0.725000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=157139856.901371 note=wall_secs=0.6945_peak_rss=2932736_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-09442acdbdc0c752 seed=3326471708439271635 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0164_peak_rss=2555904_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-09442acdbdc0c752 seed=3326471708439271635 condition=dense-matched accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2017849.355166 note=wall_secs=0.0068_peak_rss=3850240_spikes=946_deliveries=40800_cells=41454_plasticity=774386
config_hash=c1-09442acdbdc0c752 seed=14727610406674846950 condition=local-assembly accuracy=0.500000 activity_sparsity=0.012109 activity-sparsity=0.012109 work_per_accuracy=1536666.000000 note=wall_secs=0.0059_peak_rss=3604480_spikes=801_deliveries=19259_cells=19867_plasticity=728406
config_hash=c1-09442acdbdc0c752 seed=14727610406674846950 condition=dense-local accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5580875.137578 note=wall_secs=0.0117_peak_rss=4423680_spikes=946_deliveries=61680_cells=62334_plasticity=2246912
config_hash=c1-09442acdbdc0c752 seed=14727610406674846950 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6805_peak_rss=2850816_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-09442acdbdc0c752 seed=14727610406674846950 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0162_peak_rss=2539520_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-09442acdbdc0c752 seed=14727610406674846950 condition=dense-matched accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2019171.708070 note=wall_secs=0.0065_peak_rss=3866624_spikes=946_deliveries=40947_cells=41601_plasticity=774654
config_hash=c1-09442acdbdc0c752 seed=7681300295787073789 condition=local-assembly accuracy=0.500000 activity_sparsity=0.012109 activity-sparsity=0.012109 work_per_accuracy=34472072.000000 note=wall_secs=0.1042_peak_rss=26066944_spikes=185196_deliveries=8160468_cells=8161084_plasticity=729288
config_hash=c1-09442acdbdc0c752 seed=7681300295787073789 condition=dense-local accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5580875.137578 note=wall_secs=0.0126_peak_rss=4636672_spikes=946_deliveries=61680_cells=62334_plasticity=2246912
config_hash=c1-09442acdbdc0c752 seed=7681300295787073789 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.7022_peak_rss=2867200_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-09442acdbdc0c752 seed=7681300295787073789 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0161_peak_rss=2555904_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-09442acdbdc0c752 seed=7681300295787073789 condition=dense-matched accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2021007.002136 note=wall_secs=0.0070_peak_rss=3932160_spikes=946_deliveries=40868_cells=41522_plasticity=775592
config_hash=c1-09442acdbdc0c752 seed=635552018161224976 condition=local-assembly accuracy=0.425000 activity_sparsity=0.000000 activity-sparsity=0.000000 work_per_accuracy=4969315.154732 note=wall_secs=0.0164_peak_rss=22429696_spikes=16945_deliveries=706092_cells=706646_plasticity=682276
config_hash=c1-09442acdbdc0c752 seed=635552018161224976 condition=dense-local accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5580875.137578 note=wall_secs=0.0121_peak_rss=4325376_spikes=946_deliveries=61680_cells=62334_plasticity=2246912
config_hash=c1-09442acdbdc0c752 seed=635552018161224976 condition=gradient-reference accuracy=0.725000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=157139856.901371 note=wall_secs=0.6864_peak_rss=2916352_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-09442acdbdc0c752 seed=635552018161224976 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0168_peak_rss=2555904_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-09442acdbdc0c752 seed=635552018161224976 condition=dense-matched accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2019604.649234 note=wall_secs=0.0068_peak_rss=3899392_spikes=946_deliveries=40972_cells=41626_plasticity=774788
config_hash=c1-09442acdbdc0c752 seed=12035985998162872615 condition=local-assembly accuracy=0.500000 activity_sparsity=0.012109 activity-sparsity=0.012109 work_per_accuracy=1543716.000000 note=wall_secs=0.0059_peak_rss=3538944_spikes=848_deliveries=21630_cells=22234_plasticity=727146
config_hash=c1-09442acdbdc0c752 seed=12035985998162872615 condition=dense-local accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5580875.137578 note=wall_secs=0.0118_peak_rss=4325376_spikes=946_deliveries=61680_cells=62334_plasticity=2246912
config_hash=c1-09442acdbdc0c752 seed=12035985998162872615 condition=gradient-reference accuracy=0.725000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=157139856.901371 note=wall_secs=0.6784_peak_rss=2834432_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-09442acdbdc0c752 seed=12035985998162872615 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0157_peak_rss=2539520_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-09442acdbdc0c752 seed=12035985998162872615 condition=dense-matched accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2016945.825779 note=wall_secs=0.0065_peak_rss=3964928_spikes=946_deliveries=41144_cells=41798_plasticity=773314
config_hash=c1-09442acdbdc0c752 seed=4990235246635861306 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1572720.000000 note=wall_secs=0.0061_peak_rss=3571712_spikes=998_deliveries=22245_cells=22893_plasticity=740224
config_hash=c1-09442acdbdc0c752 seed=4990235246635861306 condition=dense-local accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5580875.137578 note=wall_secs=0.0121_peak_rss=4554752_spikes=946_deliveries=61680_cells=62334_plasticity=2246912
config_hash=c1-09442acdbdc0c752 seed=4990235246635861306 condition=gradient-reference accuracy=0.725000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=157139856.901371 note=wall_secs=0.6988_peak_rss=2899968_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-09442acdbdc0c752 seed=4990235246635861306 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0164_peak_rss=2539520_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-09442acdbdc0c752 seed=4990235246635861306 condition=dense-matched accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2019171.708070 note=wall_secs=0.0064_peak_rss=3964928_spikes=946_deliveries=40813_cells=41467_plasticity=774922
config_hash=c1-09442acdbdc0c752 seed=16390669209457639761 condition=local-assembly accuracy=0.500000 activity_sparsity=0.012109 activity-sparsity=0.012109 work_per_accuracy=1607294.000000 note=wall_secs=0.0065_peak_rss=3620864_spikes=1944_deliveries=42368_cells=42987_plasticity=716348
config_hash=c1-09442acdbdc0c752 seed=16390669209457639761 condition=dense-local accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5580875.137578 note=wall_secs=0.0125_peak_rss=4390912_spikes=946_deliveries=61680_cells=62334_plasticity=2246912
config_hash=c1-09442acdbdc0c752 seed=16390669209457639761 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.7030_peak_rss=2916352_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-09442acdbdc0c752 seed=16390669209457639761 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0175_peak_rss=2539520_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-09442acdbdc0c752 seed=16390669209457639761 condition=dense-matched accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2020301.119803 note=wall_secs=0.0069_peak_rss=3932160_spikes=946_deliveries=41455_cells=42109_plasticity=774118
config_hash=c1-09442acdbdc0c752 seed=9344920897472052580 condition=local-assembly accuracy=0.425000 activity_sparsity=0.000000 activity-sparsity=0.000000 work_per_accuracy=1799291.714237 note=wall_secs=0.0057_peak_rss=3620864_spikes=777_deliveries=10736_cells=11298_plasticity=741888
config_hash=c1-09442acdbdc0c752 seed=9344920897472052580 condition=dense-local accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5580875.137578 note=wall_secs=0.0128_peak_rss=4587520_spikes=946_deliveries=61680_cells=62334_plasticity=2246912
config_hash=c1-09442acdbdc0c752 seed=9344920897472052580 condition=gradient-reference accuracy=0.725000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=157139856.901371 note=wall_secs=0.7032_peak_rss=2899968_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-09442acdbdc0c752 seed=9344920897472052580 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0167_peak_rss=2539520_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-09442acdbdc0c752 seed=9344920897472052580 condition=dense-matched accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2026291.707870 note=wall_secs=0.0071_peak_rss=4030464_spikes=946_deliveries=41455_cells=42109_plasticity=776664
config_hash=c1-09442acdbdc0c752 seed=2298610769404410235 condition=local-assembly accuracy=0.500000 activity_sparsity=0.006055 activity-sparsity=0.006055 work_per_accuracy=6573586.000000 note=wall_secs=0.0281_peak_rss=34947072_spikes=29861_deliveries=1269631_cells=1270209_plasticity=717092
config_hash=c1-09442acdbdc0c752 seed=2298610769404410235 condition=dense-local accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5580875.137578 note=wall_secs=0.0132_peak_rss=4636672_spikes=946_deliveries=61680_cells=62334_plasticity=2246912
config_hash=c1-09442acdbdc0c752 seed=2298610769404410235 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.7126_peak_rss=2932736_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-09442acdbdc0c752 seed=2298610769404410235 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0169_peak_rss=2539520_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-09442acdbdc0c752 seed=2298610769404410235 condition=dense-matched accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2020047.002163 note=wall_secs=0.0073_peak_rss=3932160_spikes=946_deliveries=40999_cells=41653_plasticity=774922
config_hash=c1-09442acdbdc0c752 seed=13699608798871106958 condition=local-assembly accuracy=0.500000 activity_sparsity=0.012109 activity-sparsity=0.012109 work_per_accuracy=11220672.000000 note=wall_secs=0.0370_peak_rss=18317312_spikes=55821_deliveries=2421108_cells=2421729_plasticity=711678
config_hash=c1-09442acdbdc0c752 seed=13699608798871106958 condition=dense-local accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5580875.137578 note=wall_secs=0.0129_peak_rss=4603904_spikes=946_deliveries=61680_cells=62334_plasticity=2246912
config_hash=c1-09442acdbdc0c752 seed=13699608798871106958 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.7244_peak_rss=2932736_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-09442acdbdc0c752 seed=13699608798871106958 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0171_peak_rss=2523136_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-09442acdbdc0c752 seed=13699608798871106958 condition=dense-matched accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2024037.590286 note=wall_secs=0.0071_peak_rss=3948544_spikes=946_deliveries=41646_cells=42300_plasticity=775324
config_hash=c1-09442acdbdc0c752 seed=6653297863349612965 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1544650.000000 note=wall_secs=0.0061_peak_rss=3588096_spikes=947_deliveries=20911_cells=21557_plasticity=728910
config_hash=c1-09442acdbdc0c752 seed=6653297863349612965 condition=dense-local accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5580875.137578 note=wall_secs=0.0124_peak_rss=4390912_spikes=946_deliveries=61680_cells=62334_plasticity=2246912
config_hash=c1-09442acdbdc0c752 seed=6653297863349612965 condition=gradient-reference accuracy=0.500000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=227852800.000000 note=wall_secs=0.7030_peak_rss=2932736_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-09442acdbdc0c752 seed=6653297863349612965 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0172_peak_rss=2539520_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-09442acdbdc0c752 seed=6653297863349612965 condition=dense-matched accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2022155.237398 note=wall_secs=0.0069_peak_rss=3866624_spikes=946_deliveries=41313_cells=41967_plasticity=775190
```
