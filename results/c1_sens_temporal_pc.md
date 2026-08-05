# C1 / Gate G2 results note

**Config hash:** `c1-a49deeaedb495a09`

**Scientific protocol version:** `3`

**Sensitivity protocol (Tier-B):** `3` — optional confound probe; does **not** reopen protocol-v2 kill-gate hash `c1-118207fbc3eaba53` (canonical version `2`).

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
Config { experiment: "c1-sens-temporal-pc", master_seed: 212746910040065, n_seeds: 20, sequence_len: 8, max_lag: 1, n_hidden: 128, k_wta: 2, p_sparse: 0.35, init_w: 0.15, eta: 0.35, lambda: 0.002, tau_e: 40.0, n_train: 80, n_test: 40, bptt_epochs: 80, bptt_lr: 0.05, g2_min_gap_closed: 0.5, g2_min_accuracy: 0.65, g2_confidence_z: 1.96, g2_min_positive_control: 0.9, g2_min_reference_gap: 0.15, activity_sparsity_min: 0.005, activity_sparsity_max: 0.03, scientific_n_seeds: 20, power_sigma_prior: 0.15, power_effect_size: 0.1, use_surrogate_lif_reference: true, surrogate_beta: 5.0, matched_budget_repeat: true, quick: false }
```

Power analysis: required scientific n_seeds ≥ 20 (preregistered σ=0.150, effect=0.100; formula n=⌈(1.96+0.8416)²σ²/δ²⌉).

## Per-seed accuracies

| seed | local-assembly | dense-local | gradient reference | eligibility reference | activity_sparsity (local) | activity_sparsity (dense) | dense_matched |
|---|---:|---:|---:|---:|---:|---:|---:|
| 11400784148685290516 | 0.5000 | 0.5000 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 4354472921106020395 | 0.7250 | 0.5000 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 15755469816701350974 | 0.5000 | 0.5000 | 0.5000 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 8709160805325205589 | 0.5000 | 0.5000 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 1663413627210984552 | 0.5000 | 0.5000 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 13063846473341266047 | 0.5500 | 0.5000 | 0.7250 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 6018099295227045010 | 0.5000 | 0.5000 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 17418529959513940137 | 0.5000 | 0.5000 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 10372782781399719100 | 0.5000 | 0.5000 | 0.7250 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 3326471553820448979 | 0.5000 | 0.5000 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 14727610286415762662 | 0.5000 | 0.5000 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 7681300175527989501 | 0.5000 | 0.5000 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 635551897902140688 | 0.5000 | 0.5000 | 0.7250 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 12035985843544049959 | 0.7250 | 0.5000 | 0.7250 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 4990235366894945594 | 0.5000 | 0.5000 | 0.7250 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 16390669329716724049 | 0.5500 | 0.5000 | 0.7250 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 9344921052090875236 | 0.4750 | 0.5000 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 2298610924023232891 | 0.5000 | 0.5000 | 0.7250 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 13699608919130191246 | 0.5000 | 0.5000 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 6653297708730790309 | 0.5000 | 0.5000 | 0.7250 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |

## Summary (paired normalized-gap analysis)

- mean ± var local-assembly: 0.5263 ± 0.004900
- mean ± var dense-local:    0.5000 ± 0.000000
- mean ± var gradient reference: 0.8575 ± 0.024020
- mean ± var eligibility reference: 1.0000 ± 0.000000
- mean normalized gap closed: 0.0947 (variance 0.059043)
- lower confidence bound (z=1.960, n=20): -0.0118
- mean |local − dense| (descriptive): 0.0288

## U-NEG

Negative result: local-assembly did **not** clear the preregistered normalized-gap confidence and absolute-accuracy gates. Program stops at G2; do not schedule P3+.

## Positive / sanity control

Mean local-pipeline accuracy on a temporal coincidence-lag positive-control task: **0.9675** (threshold 0.900).

## Activity sparsity

Mean local-assembly activity_sparsity: **0.0156** (valid band [0.0050, 0.0300]; nominal k/N=0.0156).

## Parameter / compute budgets

| condition | n_cells | n_params | wall_secs | peak_rss_bytes | work_per_accuracy | spikes | deliveries | cell_updates | plasticity |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| local-assembly | 132 | 5781 | 0.0088 | 3391488 | 1454618.0000 | 763 | 13202 | 13843 | 699501 |
| dense-local | 132 | 16768 | 0.0130 | 4292608 | 4978492.0000 | 937 | 61680 | 62341 | 2364288 |
| gradient-reference | 130 | 16769 | 0.6403 | 2719744 | 113926400.0000 | 0 | 51200 | 6553600 | 107321600 |
| eligibility-reference | 130 | 385 | 0.0382 | 2523136 | 9068800.0000 | 0 | 51200 | 6553600 | 2464000 |
| dense-matched | 132 | 5781 | 0.0069 | 3915776 | 1796758.0000 | 937 | 40830 | 41491 | 815121 |

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
config_hash=c1-a49deeaedb495a09 seed=11400784148685290516 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1454618.000000 note=wall_secs=0.0088_peak_rss=3391488_spikes=763_deliveries=13202_cells=13843_plasticity=699501
config_hash=c1-a49deeaedb495a09 seed=11400784148685290516 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=4978492.000000 note=wall_secs=0.0130_peak_rss=4292608_spikes=937_deliveries=61680_cells=62341_plasticity=2364288
config_hash=c1-a49deeaedb495a09 seed=11400784148685290516 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6403_peak_rss=2719744_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-a49deeaedb495a09 seed=11400784148685290516 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0382_peak_rss=2523136_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-a49deeaedb495a09 seed=11400784148685290516 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1796758.000000 note=wall_secs=0.0069_peak_rss=3915776_spikes=937_deliveries=40830_cells=41491_plasticity=815121
config_hash=c1-a49deeaedb495a09 seed=4354472921106020395 condition=local-assembly accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=997529.622368 note=wall_secs=0.0050_peak_rss=3489792_spikes=705_deliveries=13472_cells=14112_plasticity=694920
config_hash=c1-a49deeaedb495a09 seed=4354472921106020395 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=4978492.000000 note=wall_secs=0.0104_peak_rss=4194304_spikes=937_deliveries=61680_cells=62341_plasticity=2364288
config_hash=c1-a49deeaedb495a09 seed=4354472921106020395 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6382_peak_rss=2801664_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-a49deeaedb495a09 seed=4354472921106020395 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0152_peak_rss=2506752_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-a49deeaedb495a09 seed=4354472921106020395 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1799446.000000 note=wall_secs=0.0058_peak_rss=3801088_spikes=937_deliveries=40797_cells=41458_plasticity=816531
config_hash=c1-a49deeaedb495a09 seed=15755469816701350974 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1441414.000000 note=wall_secs=0.0054_peak_rss=3620864_spikes=667_deliveries=13320_cells=13960_plasticity=692760
config_hash=c1-a49deeaedb495a09 seed=15755469816701350974 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=4978492.000000 note=wall_secs=0.0119_peak_rss=4358144_spikes=937_deliveries=61680_cells=62341_plasticity=2364288
config_hash=c1-a49deeaedb495a09 seed=15755469816701350974 condition=gradient-reference accuracy=0.500000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=227852800.000000 note=wall_secs=0.6426_peak_rss=2736128_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-a49deeaedb495a09 seed=15755469816701350974 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0149_peak_rss=2490368_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-a49deeaedb495a09 seed=15755469816701350974 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1794150.000000 note=wall_secs=0.0063_peak_rss=3784704_spikes=937_deliveries=40742_cells=41403_plasticity=813993
config_hash=c1-a49deeaedb495a09 seed=8709160805325205589 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1442094.000000 note=wall_secs=0.0050_peak_rss=3473408_spikes=761_deliveries=13323_cells=13963_plasticity=693000
config_hash=c1-a49deeaedb495a09 seed=8709160805325205589 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=4978492.000000 note=wall_secs=0.0111_peak_rss=4292608_spikes=937_deliveries=61680_cells=62341_plasticity=2364288
config_hash=c1-a49deeaedb495a09 seed=8709160805325205589 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6549_peak_rss=2818048_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-a49deeaedb495a09 seed=8709160805325205589 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0146_peak_rss=2506752_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-a49deeaedb495a09 seed=8709160805325205589 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1795094.000000 note=wall_secs=0.0070_peak_rss=3850240_spikes=937_deliveries=40837_cells=41498_plasticity=814275
config_hash=c1-a49deeaedb495a09 seed=1663413627210984552 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1474068.000000 note=wall_secs=0.0051_peak_rss=3506176_spikes=772_deliveries=13262_cells=13905_plasticity=709095
config_hash=c1-a49deeaedb495a09 seed=1663413627210984552 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=4978492.000000 note=wall_secs=0.0106_peak_rss=4341760_spikes=937_deliveries=61680_cells=62341_plasticity=2364288
config_hash=c1-a49deeaedb495a09 seed=1663413627210984552 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6431_peak_rss=2818048_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-a49deeaedb495a09 seed=1663413627210984552 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0154_peak_rss=2523136_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-a49deeaedb495a09 seed=1663413627210984552 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1793722.000000 note=wall_secs=0.0058_peak_rss=3899392_spikes=937_deliveries=41199_cells=41860_plasticity=812865
config_hash=c1-a49deeaedb495a09 seed=13063846473341266047 condition=local-assembly accuracy=0.550000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1311852.698839 note=wall_secs=0.0052_peak_rss=3588096_spikes=701_deliveries=13229_cells=13869_plasticity=693720
config_hash=c1-a49deeaedb495a09 seed=13063846473341266047 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=4978492.000000 note=wall_secs=0.0112_peak_rss=4276224_spikes=937_deliveries=61680_cells=62341_plasticity=2364288
config_hash=c1-a49deeaedb495a09 seed=13063846473341266047 condition=gradient-reference accuracy=0.725000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=157139856.901371 note=wall_secs=0.6405_peak_rss=2719744_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-a49deeaedb495a09 seed=13063846473341266047 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0174_peak_rss=2523136_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-a49deeaedb495a09 seed=13063846473341266047 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1798958.000000 note=wall_secs=0.0061_peak_rss=3784704_spikes=937_deliveries=41380_cells=42041_plasticity=815121
config_hash=c1-a49deeaedb495a09 seed=6018099295227045010 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1467190.000000 note=wall_secs=0.0050_peak_rss=3440640_spikes=671_deliveries=13439_cells=14081_plasticity=705404
config_hash=c1-a49deeaedb495a09 seed=6018099295227045010 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=4978492.000000 note=wall_secs=0.0105_peak_rss=4259840_spikes=937_deliveries=61680_cells=62341_plasticity=2364288
config_hash=c1-a49deeaedb495a09 seed=6018099295227045010 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6543_peak_rss=2768896_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-a49deeaedb495a09 seed=6018099295227045010 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0156_peak_rss=2506752_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-a49deeaedb495a09 seed=6018099295227045010 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1797812.000000 note=wall_secs=0.0064_peak_rss=3784704_spikes=937_deliveries=41023_cells=41684_plasticity=815262
config_hash=c1-a49deeaedb495a09 seed=17418529959513940137 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1454382.000000 note=wall_secs=0.0057_peak_rss=3473408_spikes=764_deliveries=13203_cells=13844_plasticity=699380
config_hash=c1-a49deeaedb495a09 seed=17418529959513940137 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=4978492.000000 note=wall_secs=0.0116_peak_rss=4325376_spikes=937_deliveries=61680_cells=62341_plasticity=2364288
config_hash=c1-a49deeaedb495a09 seed=17418529959513940137 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6450_peak_rss=2736128_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-a49deeaedb495a09 seed=17418529959513940137 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0147_peak_rss=2490368_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-a49deeaedb495a09 seed=17418529959513940137 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1795716.000000 note=wall_secs=0.0065_peak_rss=3801088_spikes=937_deliveries=40640_cells=41301_plasticity=814980
config_hash=c1-a49deeaedb495a09 seed=10372782781399719100 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1453442.000000 note=wall_secs=0.0050_peak_rss=3489792_spikes=772_deliveries=13206_cells=13847_plasticity=698896
config_hash=c1-a49deeaedb495a09 seed=10372782781399719100 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=4978492.000000 note=wall_secs=0.0104_peak_rss=4227072_spikes=937_deliveries=61680_cells=62341_plasticity=2364288
config_hash=c1-a49deeaedb495a09 seed=10372782781399719100 condition=gradient-reference accuracy=0.725000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=157139856.901371 note=wall_secs=0.6726_peak_rss=2883584_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-a49deeaedb495a09 seed=10372782781399719100 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0153_peak_rss=2523136_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-a49deeaedb495a09 seed=10372782781399719100 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1795432.000000 note=wall_secs=0.0059_peak_rss=3915776_spikes=937_deliveries=40851_cells=41512_plasticity=814416
config_hash=c1-a49deeaedb495a09 seed=3326471553820448979 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1488094.000000 note=wall_secs=0.0056_peak_rss=3506176_spikes=701_deliveries=13115_cells=13759_plasticity=716472
config_hash=c1-a49deeaedb495a09 seed=3326471553820448979 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=4978492.000000 note=wall_secs=0.0111_peak_rss=4227072_spikes=937_deliveries=61680_cells=62341_plasticity=2364288
config_hash=c1-a49deeaedb495a09 seed=3326471553820448979 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.7178_peak_rss=2883584_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-a49deeaedb495a09 seed=3326471553820448979 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0149_peak_rss=2506752_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-a49deeaedb495a09 seed=3326471553820448979 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1796876.000000 note=wall_secs=0.0061_peak_rss=3915776_spikes=937_deliveries=41071_cells=41732_plasticity=814698
config_hash=c1-a49deeaedb495a09 seed=14727610286415762662 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1501264.000000 note=wall_secs=0.0049_peak_rss=3588096_spikes=676_deliveries=13218_cells=13863_plasticity=722875
config_hash=c1-a49deeaedb495a09 seed=14727610286415762662 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=4978492.000000 note=wall_secs=0.0114_peak_rss=4243456_spikes=937_deliveries=61680_cells=62341_plasticity=2364288
config_hash=c1-a49deeaedb495a09 seed=14727610286415762662 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6684_peak_rss=2883584_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-a49deeaedb495a09 seed=14727610286415762662 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0153_peak_rss=2523136_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-a49deeaedb495a09 seed=14727610286415762662 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1800758.000000 note=wall_secs=0.0064_peak_rss=3866624_spikes=937_deliveries=41689_cells=42350_plasticity=815403
config_hash=c1-a49deeaedb495a09 seed=7681300175527989501 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1450752.000000 note=wall_secs=0.0057_peak_rss=3457024_spikes=764_deliveries=13203_cells=13844_plasticity=697565
config_hash=c1-a49deeaedb495a09 seed=7681300175527989501 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=4978492.000000 note=wall_secs=0.0115_peak_rss=4292608_spikes=937_deliveries=61680_cells=62341_plasticity=2364288
config_hash=c1-a49deeaedb495a09 seed=7681300175527989501 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6716_peak_rss=2883584_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-a49deeaedb495a09 seed=7681300175527989501 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0154_peak_rss=2523136_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-a49deeaedb495a09 seed=7681300175527989501 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1794826.000000 note=wall_secs=0.0065_peak_rss=3883008_spikes=937_deliveries=41475_cells=42136_plasticity=812865
config_hash=c1-a49deeaedb495a09 seed=635551897902140688 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1443044.000000 note=wall_secs=0.0054_peak_rss=3489792_spikes=760_deliveries=13321_cells=13961_plasticity=693480
config_hash=c1-a49deeaedb495a09 seed=635551897902140688 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=4978492.000000 note=wall_secs=0.0113_peak_rss=4210688_spikes=937_deliveries=61680_cells=62341_plasticity=2364288
config_hash=c1-a49deeaedb495a09 seed=635551897902140688 condition=gradient-reference accuracy=0.725000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=157139856.901371 note=wall_secs=0.6921_peak_rss=2899968_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-a49deeaedb495a09 seed=635551897902140688 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0168_peak_rss=2523136_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-a49deeaedb495a09 seed=635551897902140688 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1796614.000000 note=wall_secs=0.0061_peak_rss=3801088_spikes=937_deliveries=40935_cells=41596_plasticity=814839
config_hash=c1-a49deeaedb495a09 seed=12035985843544049959 condition=local-assembly accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=873679.971269 note=wall_secs=0.0053_peak_rss=3440640_spikes=747_deliveries=13203_cells=13828_plasticity=605640
config_hash=c1-a49deeaedb495a09 seed=12035985843544049959 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=4978492.000000 note=wall_secs=0.0117_peak_rss=4571136_spikes=937_deliveries=61680_cells=62341_plasticity=2364288
config_hash=c1-a49deeaedb495a09 seed=12035985843544049959 condition=gradient-reference accuracy=0.725000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=157139856.901371 note=wall_secs=0.6700_peak_rss=2801664_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-a49deeaedb495a09 seed=12035985843544049959 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0154_peak_rss=2506752_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-a49deeaedb495a09 seed=12035985843544049959 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1795236.000000 note=wall_secs=0.0087_peak_rss=3784704_spikes=937_deliveries=41366_cells=42027_plasticity=813288
config_hash=c1-a49deeaedb495a09 seed=4990235366894945594 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1482122.000000 note=wall_secs=0.0055_peak_rss=3604480_spikes=689_deliveries=13226_cells=13869_plasticity=713277
config_hash=c1-a49deeaedb495a09 seed=4990235366894945594 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=4978492.000000 note=wall_secs=0.0114_peak_rss=4571136_spikes=937_deliveries=61680_cells=62341_plasticity=2364288
config_hash=c1-a49deeaedb495a09 seed=4990235366894945594 condition=gradient-reference accuracy=0.725000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=157139856.901371 note=wall_secs=0.6651_peak_rss=2899968_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-a49deeaedb495a09 seed=4990235366894945594 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0175_peak_rss=8388608_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-a49deeaedb495a09 seed=4990235366894945594 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1802882.000000 note=wall_secs=0.0064_peak_rss=3784704_spikes=937_deliveries=41092_cells=41753_plasticity=817659
config_hash=c1-a49deeaedb495a09 seed=16390669329716724049 condition=local-assembly accuracy=0.550000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1419374.514690 note=wall_secs=0.0058_peak_rss=9437184_spikes=828_deliveries=13434_cells=14084_plasticity=752310
config_hash=c1-a49deeaedb495a09 seed=16390669329716724049 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=4978492.000000 note=wall_secs=0.0107_peak_rss=10223616_spikes=937_deliveries=61680_cells=62341_plasticity=2364288
config_hash=c1-a49deeaedb495a09 seed=16390669329716724049 condition=gradient-reference accuracy=0.725000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=157139856.901371 note=wall_secs=0.6421_peak_rss=8667136_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-a49deeaedb495a09 seed=16390669329716724049 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0153_peak_rss=8421376_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-a49deeaedb495a09 seed=16390669329716724049 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1799514.000000 note=wall_secs=0.0060_peak_rss=9764864_spikes=937_deliveries=41096_cells=41757_plasticity=815967
config_hash=c1-a49deeaedb495a09 seed=9344921052090875236 condition=local-assembly accuracy=0.475000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1595789.493709 note=wall_secs=0.0057_peak_rss=9453568_spikes=844_deliveries=13359_cells=14005_plasticity=729792
config_hash=c1-a49deeaedb495a09 seed=9344921052090875236 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=4978492.000000 note=wall_secs=0.0107_peak_rss=10223616_spikes=937_deliveries=61680_cells=62341_plasticity=2364288
config_hash=c1-a49deeaedb495a09 seed=9344921052090875236 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6424_peak_rss=8683520_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-a49deeaedb495a09 seed=9344921052090875236 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0153_peak_rss=8388608_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-a49deeaedb495a09 seed=9344921052090875236 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1799168.000000 note=wall_secs=0.0059_peak_rss=9748480_spikes=937_deliveries=40657_cells=41318_plasticity=816672
config_hash=c1-a49deeaedb495a09 seed=2298610924023232891 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1481472.000000 note=wall_secs=0.0053_peak_rss=9437184_spikes=769_deliveries=13331_cells=13974_plasticity=712662
config_hash=c1-a49deeaedb495a09 seed=2298610924023232891 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=4978492.000000 note=wall_secs=0.0111_peak_rss=10240000_spikes=937_deliveries=61680_cells=62341_plasticity=2364288
config_hash=c1-a49deeaedb495a09 seed=2298610924023232891 condition=gradient-reference accuracy=0.725000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=157139856.901371 note=wall_secs=0.6467_peak_rss=8601600_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-a49deeaedb495a09 seed=2298610924023232891 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0148_peak_rss=8388608_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-a49deeaedb495a09 seed=2298610924023232891 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1802228.000000 note=wall_secs=0.0065_peak_rss=9748480_spikes=937_deliveries=41281_cells=41942_plasticity=816954
config_hash=c1-a49deeaedb495a09 seed=13699608919130191246 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1490262.000000 note=wall_secs=0.0055_peak_rss=9437184_spikes=683_deliveries=13232_cells=13876_plasticity=717340
config_hash=c1-a49deeaedb495a09 seed=13699608919130191246 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=4978492.000000 note=wall_secs=0.0107_peak_rss=10092544_spikes=937_deliveries=61680_cells=62341_plasticity=2364288
config_hash=c1-a49deeaedb495a09 seed=13699608919130191246 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6576_peak_rss=8765440_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-a49deeaedb495a09 seed=13699608919130191246 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0149_peak_rss=8388608_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-a49deeaedb495a09 seed=13699608919130191246 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1798882.000000 note=wall_secs=0.0060_peak_rss=9764864_spikes=937_deliveries=41079_cells=41740_plasticity=815685
config_hash=c1-a49deeaedb495a09 seed=6653297708730790309 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1505990.000000 note=wall_secs=0.0053_peak_rss=9437184_spikes=683_deliveries=13331_cells=13977_plasticity=725004
config_hash=c1-a49deeaedb495a09 seed=6653297708730790309 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=4978492.000000 note=wall_secs=0.0107_peak_rss=10190848_spikes=937_deliveries=61680_cells=62341_plasticity=2364288
config_hash=c1-a49deeaedb495a09 seed=6653297708730790309 condition=gradient-reference accuracy=0.725000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=157139856.901371 note=wall_secs=0.6714_peak_rss=8749056_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-a49deeaedb495a09 seed=6653297708730790309 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0158_peak_rss=8404992_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-a49deeaedb495a09 seed=6653297708730790309 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1790456.000000 note=wall_secs=0.0061_peak_rss=9650176_spikes=937_deliveries=41158_cells=41819_plasticity=811314
```
