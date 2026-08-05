# C1 / Gate G2 results note

**Config hash:** `c1-8cc19eccba9c70aa`

**Scientific protocol version:** `7`

**Assembly-Calculus `project` protocol:** `7` — hidden winners from `binn_areas::project` (charge k-WTA + Hebbian imprint) instead of inline membrane-score k-WTA; trial-isolation resets applied; does **not** reopen protocol-v2 kill-gate hash `c1-118207fbc3eaba53` (canonical version `2`).

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
Config { experiment: "c1-project", master_seed: 212686780497921, n_seeds: 20, sequence_len: 8, max_lag: 1, n_hidden: 128, k_wta: 2, p_sparse: 0.35, init_w: 0.15, eta: 0.35, lambda: 0.002, tau_e: 40.0, n_train: 80, n_test: 40, bptt_epochs: 80, bptt_lr: 0.05, g2_min_gap_closed: 0.5, g2_min_accuracy: 0.65, g2_confidence_z: 1.96, g2_min_positive_control: 0.9, g2_min_reference_gap: 0.15, activity_sparsity_min: 0.005, activity_sparsity_max: 0.03, scientific_n_seeds: 20, power_sigma_prior: 0.15, power_effect_size: 0.1, use_surrogate_lif_reference: true, surrogate_beta: 5.0, matched_budget_repeat: true, quick: false }
```

Power analysis: required scientific n_seeds ≥ 20 (preregistered σ=0.150, effect=0.100; formula n=⌈(1.96+0.8416)²σ²/δ²⌉).

## Per-seed accuracies

| seed | local-assembly | dense-local | gradient reference | eligibility reference | activity_sparsity (local) | activity_sparsity (dense) | dense_matched |
|---|---:|---:|---:|---:|---:|---:|---:|
| 11400784157275225108 | 0.5000 | 0.5000 | 0.5000 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 4354472878156347435 | 0.5000 | 0.5000 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 15755469859651023934 | 0.5000 | 0.5000 | 0.7250 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 8709160779555401813 | 0.5000 | 0.5000 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 1663413687340526696 | 0.5000 | 0.5000 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 13063846481931200639 | 0.5000 | 0.5000 | 0.7250 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 6018099252277372050 | 0.5000 | 0.5000 | 0.7250 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 17418529985283743913 | 0.5000 | 0.5000 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 10372782755629915324 | 0.5000 | 0.5000 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 3326471613949991123 | 0.5000 | 0.5000 | 0.7250 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 14727610295005697254 | 0.5000 | 0.5000 | 0.7250 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 7681300115398447357 | 0.5000 | 0.5000 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 635551923671944464 | 0.5000 | 0.5000 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 12035985817774246183 | 0.5000 | 0.5000 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 4990235427024487738 | 0.5000 | 0.5000 | 0.7250 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 16390669321126789457 | 0.5000 | 0.5000 | 0.7250 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 9344920991961333092 | 0.5000 | 0.5000 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 2298610949793036667 | 0.5000 | 0.5000 | 0.7250 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 13699608893360387470 | 0.5000 | 0.5000 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 6653297751680463269 | 0.5000 | 0.5000 | 0.7250 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |

## Summary (paired normalized-gap analysis)

- mean ± var local-assembly: 0.5000 ± 0.000000
- mean ± var dense-local:    0.5000 ± 0.000000
- mean ± var gradient reference: 0.8438 ± 0.023676
- mean ± var eligibility reference: 1.0000 ± 0.000000
- mean normalized gap closed: 0.0000 (variance 0.000000)
- lower confidence bound (z=1.960, n=20): 0.0000
- mean |local − dense| (descriptive): 0.0000

## U-NEG

Negative result: local-assembly did **not** clear the preregistered normalized-gap confidence and absolute-accuracy gates. Program stops at G2; do not schedule P3+.

## Positive / sanity control

Mean local-pipeline accuracy on a trivially separable spatial feature-presence task: **0.9163** (threshold 0.900).

## Activity sparsity

Mean local-assembly activity_sparsity: **0.0156** (valid band [0.0050, 0.0300]; nominal k/N=0.0156).

## Parameter / compute budgets

| condition | n_cells | n_params | wall_secs | peak_rss_bytes | work_per_accuracy | spikes | deliveries | cell_updates | plasticity |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| local-assembly | 132 | 5786 | 0.0073 | 3506176 | 1907996.0000 | 918 | 13320 | 14000 | 925760 |
| dense-local | 132 | 16768 | 0.0481 | 107347968 | 13678916.0000 | 34570 | 2060664 | 2061344 | 2682880 |
| gradient-reference | 130 | 16769 | 0.6898 | 2932736 | 227852800.0000 | 0 | 51200 | 6553600 | 107321600 |
| eligibility-reference | 130 | 385 | 0.0185 | 2555904 | 9068800.0000 | 0 | 51200 | 6553600 | 2464000 |
| dense-matched | 132 | 5786 | 0.0076 | 3817472 | 2020318.0000 | 1159 | 41280 | 41960 | 925760 |

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
config_hash=c1-8cc19eccba9c70aa seed=11400784157275225108 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1907996.000000 note=wall_secs=0.0073_peak_rss=3506176_spikes=918_deliveries=13320_cells=14000_plasticity=925760
config_hash=c1-8cc19eccba9c70aa seed=11400784157275225108 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=13678916.000000 note=wall_secs=0.0481_peak_rss=107347968_spikes=34570_deliveries=2060664_cells=2061344_plasticity=2682880
config_hash=c1-8cc19eccba9c70aa seed=11400784157275225108 condition=gradient-reference accuracy=0.500000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=227852800.000000 note=wall_secs=0.6898_peak_rss=2932736_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-8cc19eccba9c70aa seed=11400784157275225108 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0185_peak_rss=2555904_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-8cc19eccba9c70aa seed=11400784157275225108 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2020318.000000 note=wall_secs=0.0076_peak_rss=3817472_spikes=1159_deliveries=41280_cells=41960_plasticity=925760
config_hash=c1-8cc19eccba9c70aa seed=4354472878156347435 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1908956.000000 note=wall_secs=0.0064_peak_rss=3670016_spikes=918_deliveries=13320_cells=14000_plasticity=926240
config_hash=c1-8cc19eccba9c70aa seed=4354472878156347435 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=13678916.000000 note=wall_secs=0.0445_peak_rss=105103360_spikes=34570_deliveries=2060664_cells=2061344_plasticity=2682880
config_hash=c1-8cc19eccba9c70aa seed=4354472878156347435 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6744_peak_rss=2932736_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-8cc19eccba9c70aa seed=4354472878156347435 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0156_peak_rss=2539520_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-8cc19eccba9c70aa seed=4354472878156347435 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2024158.000000 note=wall_secs=0.0071_peak_rss=3866624_spikes=1159_deliveries=42000_cells=42680_plasticity=926240
config_hash=c1-8cc19eccba9c70aa seed=15755469859651023934 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1450520.000000 note=wall_secs=0.0059_peak_rss=3571712_spikes=1050_deliveries=15285_cells=15925_plasticity=693000
config_hash=c1-8cc19eccba9c70aa seed=15755469859651023934 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=13678916.000000 note=wall_secs=0.0444_peak_rss=105086976_spikes=34570_deliveries=2060664_cells=2061344_plasticity=2682880
config_hash=c1-8cc19eccba9c70aa seed=15755469859651023934 condition=gradient-reference accuracy=0.725000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=157139856.901371 note=wall_secs=0.6680_peak_rss=2768896_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-8cc19eccba9c70aa seed=15755469859651023934 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0160_peak_rss=2539520_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-8cc19eccba9c70aa seed=15755469859651023934 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2217756.000000 note=wall_secs=0.0081_peak_rss=4882432_spikes=3950_deliveries=90124_cells=90804_plasticity=924000
config_hash=c1-8cc19eccba9c70aa seed=8709160779555401813 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1855902.000000 note=wall_secs=0.0064_peak_rss=3555328_spikes=1027_deliveries=18450_cells=19124_plasticity=889350
config_hash=c1-8cc19eccba9c70aa seed=8709160779555401813 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=13678916.000000 note=wall_secs=0.0443_peak_rss=105103360_spikes=34570_deliveries=2060664_cells=2061344_plasticity=2682880
config_hash=c1-8cc19eccba9c70aa seed=8709160779555401813 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6733_peak_rss=2834432_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-8cc19eccba9c70aa seed=8709160779555401813 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0158_peak_rss=2539520_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-8cc19eccba9c70aa seed=8709160779555401813 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2034144.000000 note=wall_secs=0.0074_peak_rss=3899392_spikes=1272_deliveries=45560_cells=46240_plasticity=924000
config_hash=c1-8cc19eccba9c70aa seed=1663413687340526696 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1454502.000000 note=wall_secs=0.0058_peak_rss=3719168_spikes=1051_deliveries=15620_cells=16260_plasticity=694320
config_hash=c1-8cc19eccba9c70aa seed=1663413687340526696 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=13678916.000000 note=wall_secs=0.0447_peak_rss=105201664_spikes=34570_deliveries=2060664_cells=2061344_plasticity=2682880
config_hash=c1-8cc19eccba9c70aa seed=1663413687340526696 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6785_peak_rss=2850816_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-8cc19eccba9c70aa seed=1663413687340526696 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0158_peak_rss=2555904_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-8cc19eccba9c70aa seed=1663413687340526696 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2029438.000000 note=wall_secs=0.0072_peak_rss=3948544_spikes=1159_deliveries=43560_cells=44240_plasticity=925760
config_hash=c1-8cc19eccba9c70aa seed=13063846481931200639 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1441112.000000 note=wall_secs=0.0058_peak_rss=3522560_spikes=996_deliveries=13080_cells=13720_plasticity=692760
config_hash=c1-8cc19eccba9c70aa seed=13063846481931200639 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=13678916.000000 note=wall_secs=0.0431_peak_rss=105201664_spikes=34570_deliveries=2060664_cells=2061344_plasticity=2682880
config_hash=c1-8cc19eccba9c70aa seed=13063846481931200639 condition=gradient-reference accuracy=0.725000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=157139856.901371 note=wall_secs=0.7106_peak_rss=2932736_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-8cc19eccba9c70aa seed=13063846481931200639 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0187_peak_rss=2572288_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-8cc19eccba9c70aa seed=13063846481931200639 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2016158.000000 note=wall_secs=0.0070_peak_rss=3948544_spikes=1159_deliveries=41280_cells=41960_plasticity=923680
config_hash=c1-8cc19eccba9c70aa seed=6018099252277372050 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1909360.000000 note=wall_secs=0.0067_peak_rss=3555328_spikes=1040_deliveries=13680_cells=14360_plasticity=925600
config_hash=c1-8cc19eccba9c70aa seed=6018099252277372050 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=13678916.000000 note=wall_secs=0.0448_peak_rss=105103360_spikes=34570_deliveries=2060664_cells=2061344_plasticity=2682880
config_hash=c1-8cc19eccba9c70aa seed=6018099252277372050 condition=gradient-reference accuracy=0.725000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=157139856.901371 note=wall_secs=0.6865_peak_rss=2932736_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-8cc19eccba9c70aa seed=6018099252277372050 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0162_peak_rss=2555904_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-8cc19eccba9c70aa seed=6018099252277372050 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2040620.000000 note=wall_secs=0.0079_peak_rss=3915776_spikes=1272_deliveries=46379_cells=47059_plasticity=925600
config_hash=c1-8cc19eccba9c70aa seed=17418529985283743913 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1844190.000000 note=wall_secs=0.0066_peak_rss=3670016_spikes=1275_deliveries=18592_cells=19265_plasticity=882963
config_hash=c1-8cc19eccba9c70aa seed=17418529985283743913 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=13678916.000000 note=wall_secs=0.0467_peak_rss=107151360_spikes=34570_deliveries=2060664_cells=2061344_plasticity=2682880
config_hash=c1-8cc19eccba9c70aa seed=17418529985283743913 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.7261_peak_rss=2932736_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-8cc19eccba9c70aa seed=17418529985283743913 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0165_peak_rss=2539520_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-8cc19eccba9c70aa seed=17418529985283743913 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2017438.000000 note=wall_secs=0.0077_peak_rss=3899392_spikes=1159_deliveries=41760_cells=42440_plasticity=923360
config_hash=c1-8cc19eccba9c70aa seed=10372782755629915324 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1927068.000000 note=wall_secs=0.0068_peak_rss=3670016_spikes=1150_deliveries=18292_cells=18972_plasticity=925120
config_hash=c1-8cc19eccba9c70aa seed=10372782755629915324 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=13678916.000000 note=wall_secs=0.0445_peak_rss=105201664_spikes=34570_deliveries=2060664_cells=2061344_plasticity=2682880
config_hash=c1-8cc19eccba9c70aa seed=10372782755629915324 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.7050_peak_rss=2916352_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-8cc19eccba9c70aa seed=10372782755629915324 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0158_peak_rss=2539520_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-8cc19eccba9c70aa seed=10372782755629915324 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2019038.000000 note=wall_secs=0.0070_peak_rss=3964928_spikes=1159_deliveries=41280_cells=41960_plasticity=925120
config_hash=c1-8cc19eccba9c70aa seed=3326471613949991123 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1906236.000000 note=wall_secs=0.0064_peak_rss=3506176_spikes=918_deliveries=13440_cells=14120_plasticity=924640
config_hash=c1-8cc19eccba9c70aa seed=3326471613949991123 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=13678916.000000 note=wall_secs=0.0437_peak_rss=105185280_spikes=34570_deliveries=2060664_cells=2061344_plasticity=2682880
config_hash=c1-8cc19eccba9c70aa seed=3326471613949991123 condition=gradient-reference accuracy=0.725000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=157139856.901371 note=wall_secs=0.6817_peak_rss=2949120_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-8cc19eccba9c70aa seed=3326471613949991123 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0158_peak_rss=2539520_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-8cc19eccba9c70aa seed=3326471613949991123 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2223802.000000 note=wall_secs=0.0089_peak_rss=4964352_spikes=3971_deliveries=91305_cells=91985_plasticity=924640
config_hash=c1-8cc19eccba9c70aa seed=14727610295005697254 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1783192.000000 note=wall_secs=0.0064_peak_rss=3571712_spikes=1039_deliveries=13440_cells=14109_plasticity=863008
config_hash=c1-8cc19eccba9c70aa seed=14727610295005697254 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=13678916.000000 note=wall_secs=0.0434_peak_rss=105201664_spikes=34570_deliveries=2060664_cells=2061344_plasticity=2682880
config_hash=c1-8cc19eccba9c70aa seed=14727610295005697254 condition=gradient-reference accuracy=0.725000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=157139856.901371 note=wall_secs=0.6768_peak_rss=2752512_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-8cc19eccba9c70aa seed=14727610295005697254 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0159_peak_rss=2539520_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-8cc19eccba9c70aa seed=14727610295005697254 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2024158.000000 note=wall_secs=0.0072_peak_rss=3899392_spikes=1159_deliveries=41760_cells=42440_plasticity=926720
config_hash=c1-8cc19eccba9c70aa seed=7681300115398447357 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1816964.000000 note=wall_secs=0.0060_peak_rss=3506176_spikes=914_deliveries=13560_cells=14232_plasticity=879776
config_hash=c1-8cc19eccba9c70aa seed=7681300115398447357 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=13678916.000000 note=wall_secs=0.0436_peak_rss=105086976_spikes=34570_deliveries=2060664_cells=2061344_plasticity=2682880
config_hash=c1-8cc19eccba9c70aa seed=7681300115398447357 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6880_peak_rss=2899968_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-8cc19eccba9c70aa seed=7681300115398447357 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0172_peak_rss=2539520_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-8cc19eccba9c70aa seed=7681300115398447357 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2041128.000000 note=wall_secs=0.0075_peak_rss=3866624_spikes=1272_deliveries=46266_cells=46946_plasticity=926080
config_hash=c1-8cc19eccba9c70aa seed=635551923671944464 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1903352.000000 note=wall_secs=0.0073_peak_rss=3522560_spikes=916_deliveries=13200_cells=13880_plasticity=923680
config_hash=c1-8cc19eccba9c70aa seed=635551923671944464 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=13678916.000000 note=wall_secs=0.0471_peak_rss=105431040_spikes=34570_deliveries=2060664_cells=2061344_plasticity=2682880
config_hash=c1-8cc19eccba9c70aa seed=635551923671944464 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.7254_peak_rss=2916352_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-8cc19eccba9c70aa seed=635551923671944464 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0183_peak_rss=2555904_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-8cc19eccba9c70aa seed=635551923671944464 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2015198.000000 note=wall_secs=0.0076_peak_rss=3948544_spikes=1159_deliveries=41040_cells=41720_plasticity=923680
config_hash=c1-8cc19eccba9c70aa seed=12035985817774246183 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1452722.000000 note=wall_secs=0.0069_peak_rss=3571712_spikes=929_deliveries=15476_cells=16116_plasticity=693840
config_hash=c1-8cc19eccba9c70aa seed=12035985817774246183 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=13678916.000000 note=wall_secs=0.0473_peak_rss=105447424_spikes=34570_deliveries=2060664_cells=2061344_plasticity=2682880
config_hash=c1-8cc19eccba9c70aa seed=12035985817774246183 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.7133_peak_rss=2932736_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-8cc19eccba9c70aa seed=12035985817774246183 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0160_peak_rss=2539520_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-8cc19eccba9c70aa seed=12035985817774246183 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2018078.000000 note=wall_secs=0.0079_peak_rss=3964928_spikes=1159_deliveries=41040_cells=41720_plasticity=925120
config_hash=c1-8cc19eccba9c70aa seed=4990235427024487738 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1443034.000000 note=wall_secs=0.0060_peak_rss=3637248_spikes=877_deliveries=13080_cells=13720_plasticity=693840
config_hash=c1-8cc19eccba9c70aa seed=4990235427024487738 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=13678916.000000 note=wall_secs=0.0451_peak_rss=105201664_spikes=34570_deliveries=2060664_cells=2061344_plasticity=2682880
config_hash=c1-8cc19eccba9c70aa seed=4990235427024487738 condition=gradient-reference accuracy=0.725000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=157139856.901371 note=wall_secs=0.7258_peak_rss=2949120_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-8cc19eccba9c70aa seed=4990235427024487738 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0177_peak_rss=2539520_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-8cc19eccba9c70aa seed=4990235427024487738 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2042060.000000 note=wall_secs=0.0075_peak_rss=3948544_spikes=1272_deliveries=46979_cells=47659_plasticity=925120
config_hash=c1-8cc19eccba9c70aa seed=16390669321126789457 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1442798.000000 note=wall_secs=0.0071_peak_rss=3538944_spikes=999_deliveries=13320_cells=13960_plasticity=693120
config_hash=c1-8cc19eccba9c70aa seed=16390669321126789457 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=13678916.000000 note=wall_secs=0.0472_peak_rss=104579072_spikes=34570_deliveries=2060664_cells=2061344_plasticity=2682880
config_hash=c1-8cc19eccba9c70aa seed=16390669321126789457 condition=gradient-reference accuracy=0.725000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=157139856.901371 note=wall_secs=0.7079_peak_rss=2949120_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-8cc19eccba9c70aa seed=16390669321126789457 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0195_peak_rss=2539520_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-8cc19eccba9c70aa seed=16390669321126789457 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2039124.000000 note=wall_secs=0.0079_peak_rss=3997696_spikes=1272_deliveries=46725_cells=47405_plasticity=924160
config_hash=c1-8cc19eccba9c70aa seed=9344920991961333092 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1784864.000000 note=wall_secs=0.0073_peak_rss=3555328_spikes=1039_deliveries=13560_cells=14229_plasticity=863604
config_hash=c1-8cc19eccba9c70aa seed=9344920991961333092 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=13678916.000000 note=wall_secs=0.0465_peak_rss=105463808_spikes=34570_deliveries=2060664_cells=2061344_plasticity=2682880
config_hash=c1-8cc19eccba9c70aa seed=9344920991961333092 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6993_peak_rss=2965504_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-8cc19eccba9c70aa seed=9344920991961333092 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0166_peak_rss=2555904_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-8cc19eccba9c70aa seed=9344920991961333092 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2041656.000000 note=wall_secs=0.0080_peak_rss=3833856_spikes=1272_deliveries=45758_cells=46438_plasticity=927360
config_hash=c1-8cc19eccba9c70aa seed=2298610949793036667 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1856644.000000 note=wall_secs=0.0068_peak_rss=3588096_spikes=1268_deliveries=18592_cells=19266_plasticity=889196
config_hash=c1-8cc19eccba9c70aa seed=2298610949793036667 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=13678916.000000 note=wall_secs=0.0456_peak_rss=105201664_spikes=34570_deliveries=2060664_cells=2061344_plasticity=2682880
config_hash=c1-8cc19eccba9c70aa seed=2298610949793036667 condition=gradient-reference accuracy=0.725000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=157139856.901371 note=wall_secs=0.6819_peak_rss=2932736_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-8cc19eccba9c70aa seed=2298610949793036667 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0165_peak_rss=2523136_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-8cc19eccba9c70aa seed=2298610949793036667 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2013118.000000 note=wall_secs=0.0071_peak_rss=3948544_spikes=1159_deliveries=40440_cells=41120_plasticity=923840
config_hash=c1-8cc19eccba9c70aa seed=13699608893360387470 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1453050.000000 note=wall_secs=0.0063_peak_rss=3604480_spikes=1055_deliveries=15915_cells=16555_plasticity=693000
config_hash=c1-8cc19eccba9c70aa seed=13699608893360387470 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=13678916.000000 note=wall_secs=0.0475_peak_rss=105414656_spikes=34570_deliveries=2060664_cells=2061344_plasticity=2682880
config_hash=c1-8cc19eccba9c70aa seed=13699608893360387470 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.7126_peak_rss=2932736_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-8cc19eccba9c70aa seed=13699608893360387470 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0158_peak_rss=2539520_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-8cc19eccba9c70aa seed=13699608893360387470 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2012958.000000 note=wall_secs=0.0086_peak_rss=3932160_spikes=1159_deliveries=40320_cells=41000_plasticity=924000
config_hash=c1-8cc19eccba9c70aa seed=6653297751680463269 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1442318.000000 note=wall_secs=0.0058_peak_rss=3538944_spikes=999_deliveries=13320_cells=13960_plasticity=692880
config_hash=c1-8cc19eccba9c70aa seed=6653297751680463269 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=13678916.000000 note=wall_secs=0.0436_peak_rss=105168896_spikes=34570_deliveries=2060664_cells=2061344_plasticity=2682880
config_hash=c1-8cc19eccba9c70aa seed=6653297751680463269 condition=gradient-reference accuracy=0.725000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=157139856.901371 note=wall_secs=0.6784_peak_rss=2752512_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-8cc19eccba9c70aa seed=6653297751680463269 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0158_peak_rss=2555904_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-8cc19eccba9c70aa seed=6653297751680463269 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2014078.000000 note=wall_secs=0.0072_peak_rss=4014080_spikes=1159_deliveries=40680_cells=41360_plasticity=923840
```
