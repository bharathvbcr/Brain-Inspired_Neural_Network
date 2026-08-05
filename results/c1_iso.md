# C1 / Gate G2 results note

**Config hash:** `c1-8ec031907a3426d0`

**Scientific protocol version:** `5`

**Trial-isolation protocol:** `5` — clears `ThreeFactor.last_spike` and applies C3-style full dynamic membrane reset at trial boundaries; does **not** reopen protocol-v2 kill-gate hash `c1-118207fbc3eaba53` (canonical version `2`).

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
| `dense-matched` | Dense-local with nnz matched to local-assembly (parameter-matched; measured compute disclosed below) |
| `gradient-reference` | Same-architecture surrogate-LIF BPTT (primary); tanh RNN optional/secondary |
| `eligibility-reference` | E-prop-compatible eligibility local reference (rate-model approximation; feedforward-only) |

Plasticity uses hard ±1 reward by design (soft RPE deferred). Gap-closed is clamped to `[0, 1]` and seeds with `(reference − dense) < 0.150` contribute `closed = 0`.

## Config

```
Config { experiment: "c1-iso", master_seed: 212549341544449, n_seeds: 20, sequence_len: 8, max_lag: 1, n_hidden: 128, k_wta: 2, p_sparse: 0.35, init_w: 0.15, eta: 0.35, lambda: 0.002, tau_e: 40.0, n_train: 80, n_test: 40, bptt_epochs: 80, bptt_lr: 0.05, g2_min_gap_closed: 0.5, g2_min_accuracy: 0.65, g2_confidence_z: 1.96, g2_min_positive_control: 0.9, g2_min_reference_gap: 0.15, activity_sparsity_min: 0.005, activity_sparsity_max: 0.03, scientific_n_seeds: 20, power_sigma_prior: 0.15, power_effect_size: 0.1, use_surrogate_lif_reference: true, surrogate_beta: 5.0, matched_budget_repeat: true, quick: false }
```

Power analysis: required scientific n_seeds ≥ 20 (preregistered σ=0.150, effect=0.100; formula n=⌈(1.96+0.8416)²σ²/δ²⌉).

## Per-seed accuracies

| seed | local-assembly | dense-local | gradient reference | eligibility reference | activity_sparsity (local) | activity_sparsity (dense) | dense_matched |
|---|---:|---:|---:|---:|---:|---:|---:|
| 11400784294714178580 | 0.5000 | 0.4250 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.4250 |
| 4354473015595300907 | 0.7250 | 0.4250 | 0.7250 | 1.0000 | 0.0156 | 0.0156 | 0.4250 |
| 15755469997089977406 | 0.5000 | 0.4250 | 0.7250 | 1.0000 | 0.0156 | 0.0156 | 0.4250 |
| 8709160916994355285 | 0.4250 | 0.4250 | 0.7250 | 1.0000 | 0.0000 | 0.0156 | 0.4250 |
| 1663413549901573224 | 0.5000 | 0.4250 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.4250 |
| 13063846344492247167 | 0.4250 | 0.4250 | 1.0000 | 1.0000 | 0.0000 | 0.0156 | 0.4250 |
| 6018099114838418578 | 0.7250 | 0.4250 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.4250 |
| 17418529847844790441 | 0.5000 | 0.4250 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.4250 |
| 10372782618190961852 | 0.4250 | 0.4250 | 0.7250 | 1.0000 | 0.0000 | 0.0156 | 0.4250 |
| 3326471751388944595 | 0.7250 | 0.4250 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.4250 |
| 14727610432444650726 | 0.5000 | 0.4250 | 0.5000 | 1.0000 | 0.0156 | 0.0156 | 0.4250 |
| 7681300252837400829 | 0.5000 | 0.4250 | 0.7250 | 1.0000 | 0.0156 | 0.0156 | 0.4250 |
| 635552061110897936 | 0.4250 | 0.4250 | 0.7250 | 1.0000 | 0.0000 | 0.0156 | 0.4250 |
| 12035985955213199655 | 0.5000 | 0.4250 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.4250 |
| 4990235289585534266 | 0.5000 | 0.4250 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.4250 |
| 16390669183687835985 | 0.5000 | 0.4250 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.4250 |
| 9344920854522379620 | 0.5000 | 0.4250 | 0.7250 | 1.0000 | 0.0156 | 0.0156 | 0.4250 |
| 2298610812354083195 | 0.5000 | 0.4250 | 0.7250 | 1.0000 | 0.0156 | 0.0156 | 0.4250 |
| 13699608755921433998 | 0.5000 | 0.4250 | 0.7250 | 1.0000 | 0.0156 | 0.0156 | 0.4250 |
| 6653297889119416741 | 0.5000 | 0.4250 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.4250 |

## Summary (paired normalized-gap analysis)

- mean ± var local-assembly: 0.5188 ± 0.008808
- mean ± var dense-local:    0.4250 ± 0.000000
- mean ± var gradient reference: 0.8488 ± 0.025031
- mean ± var eligibility reference: 1.0000 ± 0.000000
- mean normalized gap closed: 0.2109 (variance 0.057339)
- lower confidence bound (z=1.960, n=20): 0.1060
- mean |local − dense| (descriptive): 0.0938

## U-NEG

Negative result: local-assembly did **not** clear the preregistered normalized-gap confidence and absolute-accuracy gates. Program stops at G2; do not schedule P3+.

## Positive / sanity control

Mean local-pipeline accuracy on a trivially separable spatial feature-presence task: **1.0000** (threshold 0.900).

## Activity sparsity

Mean local-assembly activity_sparsity: **0.0125** (valid band [0.0050, 0.0300]; nominal k/N=0.0156).

## Parameter / compute budgets

| condition | n_cells | n_params | wall_secs | peak_rss_bytes | work_per_accuracy | spikes | deliveries | cell_updates | plasticity |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| local-assembly | 132 | 5768 | 0.0063 | 3686400 | 1544408.0000 | 775 | 13354 | 14003 | 744072 |
| dense-local | 132 | 16768 | 0.0123 | 4407296 | 5580875.1376 | 946 | 61680 | 62334 | 2246912 |
| gradient-reference | 130 | 16769 | 0.6546 | 2785280 | 113926400.0000 | 0 | 51200 | 6553600 | 107321600 |
| eligibility-reference | 130 | 385 | 0.0151 | 2588672 | 9068800.0000 | 0 | 51200 | 6553600 | 2464000 |
| dense-matched | 132 | 5768 | 0.0062 | 4030464 | 2015661.1199 | 946 | 41072 | 41726 | 772912 |

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
config_hash=c1-8ec031907a3426d0 seed=11400784294714178580 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1544408.000000 note=wall_secs=0.0063_peak_rss=3686400_spikes=775_deliveries=13354_cells=14003_plasticity=744072
config_hash=c1-8ec031907a3426d0 seed=11400784294714178580 condition=dense-local accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5580875.137578 note=wall_secs=0.0123_peak_rss=4407296_spikes=946_deliveries=61680_cells=62334_plasticity=2246912
config_hash=c1-8ec031907a3426d0 seed=11400784294714178580 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6546_peak_rss=2785280_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-8ec031907a3426d0 seed=11400784294714178580 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0151_peak_rss=2588672_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-8ec031907a3426d0 seed=11400784294714178580 condition=dense-matched accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2015661.119933 note=wall_secs=0.0062_peak_rss=4030464_spikes=946_deliveries=41072_cells=41726_plasticity=772912
config_hash=c1-8ec031907a3426d0 seed=4354473015595300907 condition=local-assembly accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=877284.109081 note=wall_secs=0.0046_peak_rss=3670016_spikes=768_deliveries=13344_cells=13969_plasticity=607950
config_hash=c1-8ec031907a3426d0 seed=4354473015595300907 condition=dense-local accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5580875.137578 note=wall_secs=0.0108_peak_rss=4538368_spikes=946_deliveries=61680_cells=62334_plasticity=2246912
config_hash=c1-8ec031907a3426d0 seed=4354473015595300907 condition=gradient-reference accuracy=0.725000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=157139856.901371 note=wall_secs=0.6629_peak_rss=2867200_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-8ec031907a3426d0 seed=4354473015595300907 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0153_peak_rss=2588672_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-8ec031907a3426d0 seed=4354473015595300907 condition=dense-matched accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2025952.884350 note=wall_secs=0.0060_peak_rss=3981312_spikes=946_deliveries=41785_cells=42439_plasticity=775860
config_hash=c1-8ec031907a3426d0 seed=15755469997089977406 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1474410.000000 note=wall_secs=0.0049_peak_rss=3620864_spikes=704_deliveries=13320_cells=13963_plasticity=709218
config_hash=c1-8ec031907a3426d0 seed=15755469997089977406 condition=dense-local accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5580875.137578 note=wall_secs=0.0108_peak_rss=4521984_spikes=946_deliveries=61680_cells=62334_plasticity=2246912
config_hash=c1-8ec031907a3426d0 seed=15755469997089977406 condition=gradient-reference accuracy=0.725000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=157139856.901371 note=wall_secs=0.6563_peak_rss=2801664_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-8ec031907a3426d0 seed=15755469997089977406 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0155_peak_rss=2588672_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-8ec031907a3426d0 seed=15755469997089977406 condition=dense-matched accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2013562.296462 note=wall_secs=0.0061_peak_rss=3964928_spikes=946_deliveries=40760_cells=41414_plasticity=772644
config_hash=c1-8ec031907a3426d0 seed=8709160916994355285 condition=local-assembly accuracy=0.425000 activity_sparsity=0.000000 activity-sparsity=0.000000 work_per_accuracy=1775032.891388 note=wall_secs=0.0050_peak_rss=3538944_spikes=618_deliveries=7223_cells=7732_plasticity=738816
config_hash=c1-8ec031907a3426d0 seed=8709160916994355285 condition=dense-local accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5580875.137578 note=wall_secs=0.0111_peak_rss=4472832_spikes=946_deliveries=61680_cells=62334_plasticity=2246912
config_hash=c1-8ec031907a3426d0 seed=8709160916994355285 condition=gradient-reference accuracy=0.725000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=157139856.901371 note=wall_secs=0.6588_peak_rss=2801664_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-8ec031907a3426d0 seed=8709160916994355285 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0153_peak_rss=2588672_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-8ec031907a3426d0 seed=8709160916994355285 condition=dense-matched accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2017195.237537 note=wall_secs=0.0061_peak_rss=4079616_spikes=946_deliveries=41130_cells=41784_plasticity=773448
config_hash=c1-8ec031907a3426d0 seed=1663413549901573224 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1492346.000000 note=wall_secs=0.0049_peak_rss=3637248_spikes=783_deliveries=13331_cells=13975_plasticity=718084
config_hash=c1-8ec031907a3426d0 seed=1663413549901573224 condition=dense-local accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5580875.137578 note=wall_secs=0.0107_peak_rss=4521984_spikes=946_deliveries=61680_cells=62334_plasticity=2246912
config_hash=c1-8ec031907a3426d0 seed=1663413549901573224 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6589_peak_rss=2785280_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-8ec031907a3426d0 seed=1663413549901573224 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0174_peak_rss=2588672_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-8ec031907a3426d0 seed=1663413549901573224 condition=dense-matched accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2023435.237362 note=wall_secs=0.0059_peak_rss=4046848_spikes=946_deliveries=41183_cells=41837_plasticity=775994
config_hash=c1-8ec031907a3426d0 seed=13063846344492247167 condition=local-assembly accuracy=0.425000 activity_sparsity=0.000000 activity-sparsity=0.000000 work_per_accuracy=1748752.892125 note=wall_secs=0.0050_peak_rss=3653632_spikes=528_deliveries=5709_cells=6183_plasticity=730800
config_hash=c1-8ec031907a3426d0 seed=13063846344492247167 condition=dense-local accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5580875.137578 note=wall_secs=0.0109_peak_rss=4472832_spikes=946_deliveries=61680_cells=62334_plasticity=2246912
config_hash=c1-8ec031907a3426d0 seed=13063846344492247167 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6596_peak_rss=2801664_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-8ec031907a3426d0 seed=13063846344492247167 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0157_peak_rss=2572288_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-8ec031907a3426d0 seed=13063846344492247167 condition=dense-matched accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2025929.354939 note=wall_secs=0.0060_peak_rss=4030464_spikes=946_deliveries=41110_cells=41764_plasticity=777200
config_hash=c1-8ec031907a3426d0 seed=6018099114838418578 condition=local-assembly accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=883754.453696 note=wall_secs=0.0052_peak_rss=3686400_spikes=796_deliveries=13257_cells=13883_plasticity=612786
config_hash=c1-8ec031907a3426d0 seed=6018099114838418578 condition=dense-local accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5580875.137578 note=wall_secs=0.0114_peak_rss=4472832_spikes=946_deliveries=61680_cells=62334_plasticity=2246912
config_hash=c1-8ec031907a3426d0 seed=6018099114838418578 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6590_peak_rss=2785280_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-8ec031907a3426d0 seed=6018099114838418578 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0151_peak_rss=2588672_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-8ec031907a3426d0 seed=6018099114838418578 condition=dense-matched accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2017171.708126 note=wall_secs=0.0060_peak_rss=3981312_spikes=946_deliveries=40522_cells=41176_plasticity=774654
config_hash=c1-8ec031907a3426d0 seed=17418529847844790441 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1467288.000000 note=wall_secs=0.0048_peak_rss=3620864_spikes=982_deliveries=13430_cells=14072_plasticity=705160
config_hash=c1-8ec031907a3426d0 seed=17418529847844790441 condition=dense-local accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5580875.137578 note=wall_secs=0.0107_peak_rss=4456448_spikes=946_deliveries=61680_cells=62334_plasticity=2246912
config_hash=c1-8ec031907a3426d0 seed=17418529847844790441 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6608_peak_rss=2801664_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-8ec031907a3426d0 seed=17418529847844790441 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0152_peak_rss=2588672_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-8ec031907a3426d0 seed=17418529847844790441 condition=dense-matched accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2021750.531527 note=wall_secs=0.0061_peak_rss=4063232_spikes=946_deliveries=41562_cells=42216_plasticity=774520
config_hash=c1-8ec031907a3426d0 seed=10372782618190961852 condition=local-assembly accuracy=0.425000 activity_sparsity=0.000000 activity-sparsity=0.000000 work_per_accuracy=1768863.479796 note=wall_secs=0.0048_peak_rss=3637248_spikes=515_deliveries=4963_cells=5425_plasticity=740864
config_hash=c1-8ec031907a3426d0 seed=10372782618190961852 condition=dense-local accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5580875.137578 note=wall_secs=0.0108_peak_rss=4538368_spikes=946_deliveries=61680_cells=62334_plasticity=2246912
config_hash=c1-8ec031907a3426d0 seed=10372782618190961852 condition=gradient-reference accuracy=0.725000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=157139856.901371 note=wall_secs=0.6619_peak_rss=2867200_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-8ec031907a3426d0 seed=10372782618190961852 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0154_peak_rss=2588672_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-8ec031907a3426d0 seed=10372782618190961852 condition=dense-matched accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2023392.884422 note=wall_secs=0.0061_peak_rss=4063232_spikes=946_deliveries=41375_cells=42029_plasticity=775592
config_hash=c1-8ec031907a3426d0 seed=3326471751388944595 condition=local-assembly accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=852004.109913 note=wall_secs=0.0048_peak_rss=3588096_spikes=923_deliveries=13503_cells=14125_plasticity=589152
config_hash=c1-8ec031907a3426d0 seed=3326471751388944595 condition=dense-local accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5580875.137578 note=wall_secs=0.0109_peak_rss=4456448_spikes=946_deliveries=61680_cells=62334_plasticity=2246912
config_hash=c1-8ec031907a3426d0 seed=3326471751388944595 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6552_peak_rss=2785280_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-8ec031907a3426d0 seed=3326471751388944595 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0152_peak_rss=2588672_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-8ec031907a3426d0 seed=3326471751388944595 condition=dense-matched accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2018912.884547 note=wall_secs=0.0061_peak_rss=4030464_spikes=946_deliveries=41227_cells=41881_plasticity=773984
config_hash=c1-8ec031907a3426d0 seed=14727610432444650726 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1461382.000000 note=wall_secs=0.0052_peak_rss=3653632_spikes=671_deliveries=13085_cells=13727_plasticity=703208
config_hash=c1-8ec031907a3426d0 seed=14727610432444650726 condition=dense-local accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5580875.137578 note=wall_secs=0.0109_peak_rss=4505600_spikes=946_deliveries=61680_cells=62334_plasticity=2246912
config_hash=c1-8ec031907a3426d0 seed=14727610432444650726 condition=gradient-reference accuracy=0.500000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=227852800.000000 note=wall_secs=0.6615_peak_rss=2883584_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-8ec031907a3426d0 seed=14727610432444650726 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0150_peak_rss=2588672_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-8ec031907a3426d0 seed=14727610432444650726 condition=dense-matched accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2015021.119951 note=wall_secs=0.0061_peak_rss=4030464_spikes=946_deliveries=41204_cells=41858_plasticity=772376
config_hash=c1-8ec031907a3426d0 seed=7681300252837400829 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1491438.000000 note=wall_secs=0.0049_peak_rss=3670016_spikes=673_deliveries=13221_cells=13865_plasticity=717960
config_hash=c1-8ec031907a3426d0 seed=7681300252837400829 condition=dense-local accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5580875.137578 note=wall_secs=0.0107_peak_rss=4505600_spikes=946_deliveries=61680_cells=62334_plasticity=2246912
config_hash=c1-8ec031907a3426d0 seed=7681300252837400829 condition=gradient-reference accuracy=0.725000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=157139856.901371 note=wall_secs=0.6571_peak_rss=2867200_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-8ec031907a3426d0 seed=7681300252837400829 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0175_peak_rss=2588672_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-8ec031907a3426d0 seed=7681300252837400829 condition=dense-matched accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2023656.413826 note=wall_secs=0.0059_peak_rss=4046848_spikes=946_deliveries=41297_cells=41951_plasticity=775860
config_hash=c1-8ec031907a3426d0 seed=635552061110897936 condition=local-assembly accuracy=0.425000 activity_sparsity=0.000000 activity-sparsity=0.000000 work_per_accuracy=1741512.892328 note=wall_secs=0.0051_peak_rss=3538944_spikes=531_deliveries=5680_cells=6156_plasticity=727776
config_hash=c1-8ec031907a3426d0 seed=635552061110897936 condition=dense-local accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5580875.137578 note=wall_secs=0.0115_peak_rss=4505600_spikes=946_deliveries=61680_cells=62334_plasticity=2246912
config_hash=c1-8ec031907a3426d0 seed=635552061110897936 condition=gradient-reference accuracy=0.725000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=157139856.901371 note=wall_secs=0.6613_peak_rss=2801664_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-8ec031907a3426d0 seed=635552061110897936 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0159_peak_rss=2588672_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-8ec031907a3426d0 seed=635552061110897936 condition=dense-matched accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2018061.119865 note=wall_secs=0.0059_peak_rss=3981312_spikes=946_deliveries=41046_cells=41700_plasticity=773984
config_hash=c1-8ec031907a3426d0 seed=12035985955213199655 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1532056.000000 note=wall_secs=0.0054_peak_rss=3653632_spikes=766_deliveries=13347_cells=13995_plasticity=737920
config_hash=c1-8ec031907a3426d0 seed=12035985955213199655 condition=dense-local accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5580875.137578 note=wall_secs=0.0113_peak_rss=4472832_spikes=946_deliveries=61680_cells=62334_plasticity=2246912
config_hash=c1-8ec031907a3426d0 seed=12035985955213199655 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6585_peak_rss=2801664_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-8ec031907a3426d0 seed=12035985955213199655 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0154_peak_rss=2588672_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-8ec031907a3426d0 seed=12035985955213199655 condition=dense-matched accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2016649.355199 note=wall_secs=0.0065_peak_rss=4014080_spikes=946_deliveries=41483_cells=42137_plasticity=772510
config_hash=c1-8ec031907a3426d0 seed=4990235289585534266 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1465590.000000 note=wall_secs=0.0051_peak_rss=3555328_spikes=865_deliveries=13186_cells=13828_plasticity=704916
config_hash=c1-8ec031907a3426d0 seed=4990235289585534266 condition=dense-local accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5580875.137578 note=wall_secs=0.0109_peak_rss=4505600_spikes=946_deliveries=61680_cells=62334_plasticity=2246912
config_hash=c1-8ec031907a3426d0 seed=4990235289585534266 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6663_peak_rss=2965504_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-8ec031907a3426d0 seed=4990235289585534266 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0152_peak_rss=2588672_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-8ec031907a3426d0 seed=4990235289585534266 condition=dense-matched accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2016475.237557 note=wall_secs=0.0063_peak_rss=4079616_spikes=946_deliveries=40575_cells=41229_plasticity=774252
config_hash=c1-8ec031907a3426d0 seed=16390669183687835985 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1443980.000000 note=wall_secs=0.0051_peak_rss=3653632_spikes=698_deliveries=13346_cells=13986_plasticity=693960
config_hash=c1-8ec031907a3426d0 seed=16390669183687835985 condition=dense-local accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5580875.137578 note=wall_secs=0.0111_peak_rss=4456448_spikes=946_deliveries=61680_cells=62334_plasticity=2246912
config_hash=c1-8ec031907a3426d0 seed=16390669183687835985 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6597_peak_rss=2883584_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-8ec031907a3426d0 seed=16390669183687835985 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0150_peak_rss=2588672_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-8ec031907a3426d0 seed=16390669183687835985 condition=dense-matched accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2021148.178602 note=wall_secs=0.0062_peak_rss=3997696_spikes=946_deliveries=41233_cells=41887_plasticity=774922
config_hash=c1-8ec031907a3426d0 seed=9344920854522379620 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1477332.000000 note=wall_secs=0.0050_peak_rss=3588096_spikes=699_deliveries=13069_cells=13712_plasticity=711186
config_hash=c1-8ec031907a3426d0 seed=9344920854522379620 condition=dense-local accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5580875.137578 note=wall_secs=0.0111_peak_rss=4521984_spikes=946_deliveries=61680_cells=62334_plasticity=2246912
config_hash=c1-8ec031907a3426d0 seed=9344920854522379620 condition=gradient-reference accuracy=0.725000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=157139856.901371 note=wall_secs=0.6732_peak_rss=2965504_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-8ec031907a3426d0 seed=9344920854522379620 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0188_peak_rss=2588672_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-8ec031907a3426d0 seed=9344920854522379620 condition=dense-matched accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2019901.119814 note=wall_secs=0.0058_peak_rss=4046848_spikes=946_deliveries=41035_cells=41689_plasticity=774788
config_hash=c1-8ec031907a3426d0 seed=2298610812354083195 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1460842.000000 note=wall_secs=0.0065_peak_rss=3588096_spikes=767_deliveries=13329_cells=13971_plasticity=702354
config_hash=c1-8ec031907a3426d0 seed=2298610812354083195 condition=dense-local accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5580875.137578 note=wall_secs=0.0116_peak_rss=4505600_spikes=946_deliveries=61680_cells=62334_plasticity=2246912
config_hash=c1-8ec031907a3426d0 seed=2298610812354083195 condition=gradient-reference accuracy=0.725000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=157139856.901371 note=wall_secs=0.6669_peak_rss=2883584_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-8ec031907a3426d0 seed=2298610812354083195 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0150_peak_rss=2572288_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-8ec031907a3426d0 seed=2298610812354083195 condition=dense-matched accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2011171.708294 note=wall_secs=0.0060_peak_rss=3997696_spikes=946_deliveries=40855_cells=41509_plasticity=771438
config_hash=c1-8ec031907a3426d0 seed=13699608755921433998 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1465792.000000 note=wall_secs=0.0048_peak_rss=3670016_spikes=770_deliveries=13101_cells=13743_plasticity=705282
config_hash=c1-8ec031907a3426d0 seed=13699608755921433998 condition=dense-local accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5580875.137578 note=wall_secs=0.0110_peak_rss=4521984_spikes=946_deliveries=61680_cells=62334_plasticity=2246912
config_hash=c1-8ec031907a3426d0 seed=13699608755921433998 condition=gradient-reference accuracy=0.725000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=157139856.901371 note=wall_secs=0.6636_peak_rss=2785280_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-8ec031907a3426d0 seed=13699608755921433998 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0149_peak_rss=2588672_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-8ec031907a3426d0 seed=13699608755921433998 condition=dense-matched accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2021138.766838 note=wall_secs=0.0059_peak_rss=4046848_spikes=946_deliveries=41365_cells=42019_plasticity=774654
config_hash=c1-8ec031907a3426d0 seed=6653297889119416741 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1498946.000000 note=wall_secs=0.0050_peak_rss=3588096_spikes=735_deliveries=13234_cells=13879_plasticity=721625
config_hash=c1-8ec031907a3426d0 seed=6653297889119416741 condition=dense-local accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5580875.137578 note=wall_secs=0.0108_peak_rss=4521984_spikes=946_deliveries=61680_cells=62334_plasticity=2246912
config_hash=c1-8ec031907a3426d0 seed=6653297889119416741 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6864_peak_rss=2981888_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-8ec031907a3426d0 seed=6653297889119416741 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0181_peak_rss=2588672_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-8ec031907a3426d0 seed=6653297889119416741 condition=dense-matched accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2016941.119897 note=wall_secs=0.0057_peak_rss=3997696_spikes=946_deliveries=41009_cells=41663_plasticity=773582
```
