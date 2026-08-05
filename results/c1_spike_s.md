# C1 / Gate G2 results note

**Config hash:** `c1-c3e47b1e5f564df6`

**Scientific protocol version:** `9`

**Calibrated natural-spiking protocol:** `9` — finite hidden θ during integrate (no θ=∞ mute); **spike-count k-WTA** (not residual membrane) for hidden selection; disclosed multi-frame easy PC; production knobs `init_w`/`eta`/`tau_e` calibrated; trial-isolation resets; does **not** reopen v2 `c1-118207fbc3eaba53` or reinterpret v6 `c1-09442acdbdc0c752` (G2 thresholds unchanged).

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
Config { experiment: "c1-spike-s", master_seed: 212596586184705, n_seeds: 20, sequence_len: 8, max_lag: 1, n_hidden: 128, k_wta: 2, p_sparse: 0.35, init_w: 0.22, eta: 0.45, lambda: 0.002, tau_e: 48.0, n_train: 80, n_test: 40, bptt_epochs: 80, bptt_lr: 0.05, g2_min_gap_closed: 0.5, g2_min_accuracy: 0.65, g2_confidence_z: 1.96, g2_min_positive_control: 0.9, g2_min_reference_gap: 0.15, activity_sparsity_min: 0.005, activity_sparsity_max: 0.03, scientific_n_seeds: 20, power_sigma_prior: 0.15, power_effect_size: 0.1, use_surrogate_lif_reference: true, surrogate_beta: 5.0, matched_budget_repeat: true, quick: false }
```

Power analysis: required scientific n_seeds ≥ 20 (preregistered σ=0.150, effect=0.100; formula n=⌈(1.96+0.8416)²σ²/δ²⌉).

## Per-seed accuracies

| seed | local-assembly | dense-local | gradient reference | eligibility reference | activity_sparsity (local) | activity_sparsity (dense) | dense_matched |
|---|---:|---:|---:|---:|---:|---:|---:|
| 11400784264649407508 | 0.5000 | 0.4250 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.4250 |
| 4354473045660071979 | 0.4250 | 0.4250 | 1.0000 | 1.0000 | 0.0000 | 0.0156 | 0.4250 |
| 15755469975615140926 | 0.4250 | 0.4250 | 1.0000 | 1.0000 | 0.0000 | 0.0156 | 0.4250 |
| 8709160955649060949 | 0.5000 | 0.4250 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.4250 |
| 1663413502656932968 | 0.5000 | 0.4250 | 0.7250 | 1.0000 | 0.0156 | 0.0156 | 0.4250 |
| 13063846323017410687 | 0.4250 | 0.4250 | 0.9500 | 1.0000 | 0.0000 | 0.0156 | 0.4250 |
| 6018099144903189650 | 0.4250 | 0.4250 | 0.7250 | 1.0000 | 0.0000 | 0.0156 | 0.4250 |
| 17418529800600150185 | 0.5000 | 0.4250 | 0.7250 | 1.0000 | 0.0156 | 0.0156 | 0.4250 |
| 10372782656845667516 | 0.5000 | 0.4250 | 1.0000 | 1.0000 | 0.0078 | 0.0156 | 0.4250 |
| 3326471712734238931 | 0.5000 | 0.4250 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.4250 |
| 14727610410969814246 | 0.4250 | 0.4250 | 1.0000 | 1.0000 | 0.0000 | 0.0156 | 0.4250 |
| 7681300291492106493 | 0.5000 | 0.4250 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.4250 |
| 635552013866257680 | 0.5750 | 0.4250 | 1.0000 | 1.0000 | 0.0000 | 0.0156 | 0.4250 |
| 12035986002457839911 | 0.5000 | 0.4250 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.4250 |
| 4990235250930828602 | 0.4250 | 0.4250 | 0.9500 | 1.0000 | 0.0000 | 0.0156 | 0.4250 |
| 16390669205162672465 | 0.5000 | 0.4250 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.4250 |
| 9344920893177085284 | 0.4250 | 0.4250 | 0.9500 | 1.0000 | 0.0000 | 0.0156 | 0.4250 |
| 2298610773699377531 | 0.4250 | 0.4250 | 0.9500 | 1.0000 | 0.0000 | 0.0156 | 0.4250 |
| 13699608803166074254 | 0.4250 | 0.4250 | 1.0000 | 1.0000 | 0.0000 | 0.0156 | 0.4250 |
| 6653297859054645669 | 0.5000 | 0.4250 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.4250 |

## Summary (paired normalized-gap analysis)

- mean ± var local-assembly: 0.4700 ± 0.002013
- mean ± var dense-local:    0.4250 ± 0.000000
- mean ± var gradient reference: 0.9387 ± 0.009044
- mean ± var eligibility reference: 1.0000 ± 0.000000
- mean normalized gap closed: 0.0927 (variance 0.008993)
- lower confidence bound (z=1.960, n=20): 0.0511
- mean |local − dense| (descriptive): 0.0450

## Invalid harness

Positive control and/or activity sparsity failed the preregistered validity gates. No scientific PASS/FAIL or U-NEG claim is permitted from this run.

## Positive / sanity control

Mean local-pipeline accuracy on a disclosed multi-frame spatial feature-presence (calibrated spike-s PC; main coincidence task unchanged) task: **0.8413** (threshold 0.900).

## Activity sparsity

Mean local-assembly activity_sparsity: **0.0074** (valid band [0.0050, 0.0300]; nominal k/N=0.0156).

## Parameter / compute budgets

| condition | n_cells | n_params | wall_secs | peak_rss_bytes | work_per_accuracy | spikes | deliveries | cell_updates | plasticity |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| local-assembly | 132 | 5783 | 0.0059 | 3506176 | 1468284.0000 | 836 | 13569 | 14211 | 705526 |
| dense-local | 132 | 16768 | 0.0125 | 4472832 | 5581315.1376 | 1133 | 61680 | 62334 | 2246912 |
| gradient-reference | 130 | 16769 | 0.6697 | 2850816 | 119922527.8206 | 0 | 51200 | 6553600 | 107321600 |
| eligibility-reference | 130 | 385 | 0.0393 | 2588672 | 9068800.0000 | 0 | 51200 | 6553600 | 2464000 |
| dense-matched | 132 | 5783 | 0.0073 | 3997696 | 2022548.1786 | 1133 | 41437 | 42091 | 774922 |

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
config_hash=c1-c3e47b1e5f564df6 seed=11400784264649407508 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1468284.000000 note=wall_secs=0.0059_peak_rss=3506176_spikes=836_deliveries=13569_cells=14211_plasticity=705526
config_hash=c1-c3e47b1e5f564df6 seed=11400784264649407508 condition=dense-local accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5581315.137566 note=wall_secs=0.0125_peak_rss=4472832_spikes=1133_deliveries=61680_cells=62334_plasticity=2246912
config_hash=c1-c3e47b1e5f564df6 seed=11400784264649407508 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6697_peak_rss=2850816_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-c3e47b1e5f564df6 seed=11400784264649407508 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0393_peak_rss=2588672_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-c3e47b1e5f564df6 seed=11400784264649407508 condition=dense-matched accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2022548.178563 note=wall_secs=0.0073_peak_rss=3997696_spikes=1133_deliveries=41437_cells=42091_plasticity=774922
config_hash=c1-c3e47b1e5f564df6 seed=4354473045660071979 condition=local-assembly accuracy=0.425000 activity_sparsity=0.000000 activity-sparsity=0.000000 work_per_accuracy=1806148.184633 note=wall_secs=0.0055_peak_rss=3604480_spikes=638_deliveries=6889_cells=7386_plasticity=752700
config_hash=c1-c3e47b1e5f564df6 seed=4354473045660071979 condition=dense-local accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5581315.137566 note=wall_secs=0.0118_peak_rss=4341760_spikes=1133_deliveries=61680_cells=62334_plasticity=2246912
config_hash=c1-c3e47b1e5f564df6 seed=4354473045660071979 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6691_peak_rss=2883584_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-c3e47b1e5f564df6 seed=4354473045660071979 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0185_peak_rss=2588672_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-c3e47b1e5f564df6 seed=4354473045660071979 condition=dense-matched accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2021748.178586 note=wall_secs=0.0065_peak_rss=3817472_spikes=1133_deliveries=40798_cells=41452_plasticity=775860
config_hash=c1-c3e47b1e5f564df6 seed=15755469975615140926 condition=local-assembly accuracy=0.425000 activity_sparsity=0.000000 activity-sparsity=0.000000 work_per_accuracy=1803414.067063 note=wall_secs=0.0055_peak_rss=3440640_spikes=569_deliveries=7054_cells=7558_plasticity=751270
config_hash=c1-c3e47b1e5f564df6 seed=15755469975615140926 condition=dense-local accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5581315.137566 note=wall_secs=0.0119_peak_rss=4390912_spikes=1133_deliveries=61680_cells=62334_plasticity=2246912
config_hash=c1-c3e47b1e5f564df6 seed=15755469975615140926 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6734_peak_rss=2801664_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-c3e47b1e5f564df6 seed=15755469975615140926 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0178_peak_rss=2588672_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-c3e47b1e5f564df6 seed=15755469975615140926 condition=dense-matched accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2019908.178637 note=wall_secs=0.0066_peak_rss=3850240_spikes=1133_deliveries=41144_cells=41798_plasticity=774386
config_hash=c1-c3e47b1e5f564df6 seed=8709160955649060949 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1476078.000000 note=wall_secs=0.0059_peak_rss=3719168_spikes=1552_deliveries=23419_cells=24058_plasticity=689010
config_hash=c1-c3e47b1e5f564df6 seed=8709160955649060949 condition=dense-local accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5581315.137566 note=wall_secs=0.0120_peak_rss=4308992_spikes=1133_deliveries=61680_cells=62334_plasticity=2246912
config_hash=c1-c3e47b1e5f564df6 seed=8709160955649060949 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6667_peak_rss=2867200_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-c3e47b1e5f564df6 seed=8709160955649060949 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0153_peak_rss=2605056_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-c3e47b1e5f564df6 seed=8709160955649060949 condition=dense-matched accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2023969.354994 note=wall_secs=0.0067_peak_rss=3997696_spikes=1133_deliveries=41270_cells=41924_plasticity=775860
config_hash=c1-c3e47b1e5f564df6 seed=1663413502656932968 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1473160.000000 note=wall_secs=0.0058_peak_rss=3719168_spikes=1541_deliveries=23528_cells=24167_plasticity=687344
config_hash=c1-c3e47b1e5f564df6 seed=1663413502656932968 condition=dense-local accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5581315.137566 note=wall_secs=0.0122_peak_rss=4292608_spikes=1133_deliveries=61680_cells=62334_plasticity=2246912
config_hash=c1-c3e47b1e5f564df6 seed=1663413502656932968 condition=gradient-reference accuracy=0.725000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=157139856.901371 note=wall_secs=0.6734_peak_rss=2883584_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-c3e47b1e5f564df6 seed=1663413502656932968 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0155_peak_rss=2588672_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-c3e47b1e5f564df6 seed=1663413502656932968 condition=dense-matched accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2017447.002236 note=wall_secs=0.0063_peak_rss=4046848_spikes=1133_deliveries=40822_cells=41476_plasticity=773984
config_hash=c1-c3e47b1e5f564df6 seed=13063846323017410687 condition=local-assembly accuracy=0.425000 activity_sparsity=0.000000 activity-sparsity=0.000000 work_per_accuracy=1725435.245720 note=wall_secs=0.0055_peak_rss=3440640_spikes=591_deliveries=7865_cells=8382_plasticity=716472
config_hash=c1-c3e47b1e5f564df6 seed=13063846323017410687 condition=dense-local accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5581315.137566 note=wall_secs=0.0126_peak_rss=4341760_spikes=1133_deliveries=61680_cells=62334_plasticity=2246912
config_hash=c1-c3e47b1e5f564df6 seed=13063846323017410687 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6736_peak_rss=2883584_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-c3e47b1e5f564df6 seed=13063846323017410687 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0152_peak_rss=2588672_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-c3e47b1e5f564df6 seed=13063846323017410687 condition=dense-matched accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2018962.296311 note=wall_secs=0.0065_peak_rss=3850240_spikes=1133_deliveries=41010_cells=41664_plasticity=774252
config_hash=c1-c3e47b1e5f564df6 seed=6018099144903189650 condition=local-assembly accuracy=0.425000 activity_sparsity=0.000000 activity-sparsity=0.000000 work_per_accuracy=1738628.186527 note=wall_secs=0.0051_peak_rss=3457024_spikes=491_deliveries=4534_cells=4982_plasticity=728910
config_hash=c1-c3e47b1e5f564df6 seed=6018099144903189650 condition=dense-local accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5581315.137566 note=wall_secs=0.0121_peak_rss=4325376_spikes=1133_deliveries=61680_cells=62334_plasticity=2246912
config_hash=c1-c3e47b1e5f564df6 seed=6018099144903189650 condition=gradient-reference accuracy=0.725000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=157139856.901371 note=wall_secs=0.6870_peak_rss=2965504_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-c3e47b1e5f564df6 seed=6018099144903189650 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0394_peak_rss=2588672_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-c3e47b1e5f564df6 seed=6018099144903189650 condition=dense-matched accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2020703.472733 note=wall_secs=0.0065_peak_rss=3850240_spikes=1133_deliveries=40911_cells=41565_plasticity=775190
config_hash=c1-c3e47b1e5f564df6 seed=17418529800600150185 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1466166.000000 note=wall_secs=0.0062_peak_rss=3473408_spikes=883_deliveries=13321_cells=13963_plasticity=704916
config_hash=c1-c3e47b1e5f564df6 seed=17418529800600150185 condition=dense-local accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5581315.137566 note=wall_secs=0.0124_peak_rss=4734976_spikes=1133_deliveries=61680_cells=62334_plasticity=2246912
config_hash=c1-c3e47b1e5f564df6 seed=17418529800600150185 condition=gradient-reference accuracy=0.725000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=157139856.901371 note=wall_secs=0.6711_peak_rss=2932736_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-c3e47b1e5f564df6 seed=17418529800600150185 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0178_peak_rss=2588672_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-c3e47b1e5f564df6 seed=17418529800600150185 condition=dense-matched accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2019898.766873 note=wall_secs=0.0065_peak_rss=4030464_spikes=1133_deliveries=41209_cells=41863_plasticity=774252
config_hash=c1-c3e47b1e5f564df6 seed=10372782656845667516 condition=local-assembly accuracy=0.500000 activity_sparsity=0.007812 activity-sparsity=0.007812 work_per_accuracy=1462816.000000 note=wall_secs=0.0050_peak_rss=3457024_spikes=726_deliveries=9714_cells=10274_plasticity=710694
config_hash=c1-c3e47b1e5f564df6 seed=10372782656845667516 condition=dense-local accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5581315.137566 note=wall_secs=0.0117_peak_rss=4423680_spikes=1133_deliveries=61680_cells=62334_plasticity=2246912
config_hash=c1-c3e47b1e5f564df6 seed=10372782656845667516 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6712_peak_rss=2883584_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-c3e47b1e5f564df6 seed=10372782656845667516 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0159_peak_rss=2588672_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-c3e47b1e5f564df6 seed=10372782656845667516 condition=dense-matched accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2019428.178651 note=wall_secs=0.0065_peak_rss=3817472_spikes=1133_deliveries=41109_cells=41763_plasticity=774252
config_hash=c1-c3e47b1e5f564df6 seed=3326471712734238931 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1566444.000000 note=wall_secs=0.0061_peak_rss=3702784_spikes=994_deliveries=26714_cells=27360_plasticity=728154
config_hash=c1-c3e47b1e5f564df6 seed=3326471712734238931 condition=dense-local accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5581315.137566 note=wall_secs=0.0122_peak_rss=4554752_spikes=1133_deliveries=61680_cells=62334_plasticity=2246912
config_hash=c1-c3e47b1e5f564df6 seed=3326471712734238931 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6818_peak_rss=2965504_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-c3e47b1e5f564df6 seed=3326471712734238931 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0184_peak_rss=2588672_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-c3e47b1e5f564df6 seed=3326471712734238931 condition=dense-matched accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2019588.178646 note=wall_secs=0.0069_peak_rss=4030464_spikes=1133_deliveries=41076_cells=41730_plasticity=774386
config_hash=c1-c3e47b1e5f564df6 seed=14727610410969814246 condition=local-assembly accuracy=0.425000 activity_sparsity=0.000000 activity-sparsity=0.000000 work_per_accuracy=1734049.363126 note=wall_secs=0.0055_peak_rss=3588096_spikes=682_deliveries=9890_cells=10423_plasticity=715976
config_hash=c1-c3e47b1e5f564df6 seed=14727610410969814246 condition=dense-local accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5581315.137566 note=wall_secs=0.0119_peak_rss=4423680_spikes=1133_deliveries=61680_cells=62334_plasticity=2246912
config_hash=c1-c3e47b1e5f564df6 seed=14727610410969814246 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6944_peak_rss=2965504_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-c3e47b1e5f564df6 seed=14727610410969814246 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0180_peak_rss=2588672_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-c3e47b1e5f564df6 seed=14727610410969814246 condition=dense-matched accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2015014.061127 note=wall_secs=0.0063_peak_rss=3997696_spikes=1133_deliveries=40439_cells=41093_plasticity=773716
config_hash=c1-c3e47b1e5f564df6 seed=7681300291492106493 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1499938.000000 note=wall_secs=0.0062_peak_rss=3702784_spikes=963_deliveries=26303_cells=26943_plasticity=695760
config_hash=c1-c3e47b1e5f564df6 seed=7681300291492106493 condition=dense-local accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5581315.137566 note=wall_secs=0.0121_peak_rss=4407296_spikes=1133_deliveries=61680_cells=62334_plasticity=2246912
config_hash=c1-c3e47b1e5f564df6 seed=7681300291492106493 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6736_peak_rss=2867200_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-c3e47b1e5f564df6 seed=7681300291492106493 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0175_peak_rss=2588672_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-c3e47b1e5f564df6 seed=7681300291492106493 condition=dense-matched accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2026035.237289 note=wall_secs=0.0068_peak_rss=4030464_spikes=1133_deliveries=41173_cells=41827_plasticity=776932
config_hash=c1-c3e47b1e5f564df6 seed=635552013866257680 condition=local-assembly accuracy=0.575000 activity_sparsity=0.000000 activity-sparsity=0.000000 work_per_accuracy=1345850.462685 note=wall_secs=0.0055_peak_rss=3702784_spikes=782_deliveries=12479_cells=13048_plasticity=747555
config_hash=c1-c3e47b1e5f564df6 seed=635552013866257680 condition=dense-local accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5581315.137566 note=wall_secs=0.0113_peak_rss=4341760_spikes=1133_deliveries=61680_cells=62334_plasticity=2246912
config_hash=c1-c3e47b1e5f564df6 seed=635552013866257680 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6804_peak_rss=2883584_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-c3e47b1e5f564df6 seed=635552013866257680 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0395_peak_rss=2588672_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-c3e47b1e5f564df6 seed=635552013866257680 condition=dense-matched accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2024863.472616 note=wall_secs=0.0065_peak_rss=3850240_spikes=1133_deliveries=41125_cells=41779_plasticity=776530
config_hash=c1-c3e47b1e5f564df6 seed=12035986002457839911 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1511122.000000 note=wall_secs=0.0062_peak_rss=3719168_spikes=1055_deliveries=30133_cells=30773_plasticity=693600
config_hash=c1-c3e47b1e5f564df6 seed=12035986002457839911 condition=dense-local accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5581315.137566 note=wall_secs=0.0120_peak_rss=4472832_spikes=1133_deliveries=61680_cells=62334_plasticity=2246912
config_hash=c1-c3e47b1e5f564df6 seed=12035986002457839911 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6675_peak_rss=2785280_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-c3e47b1e5f564df6 seed=12035986002457839911 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0180_peak_rss=2588672_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-c3e47b1e5f564df6 seed=12035986002457839911 condition=dense-matched accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2018576.413969 note=wall_secs=0.0065_peak_rss=3850240_spikes=1133_deliveries=40794_cells=41448_plasticity=774520
config_hash=c1-c3e47b1e5f564df6 seed=4990235250930828602 condition=local-assembly accuracy=0.425000 activity_sparsity=0.000000 activity-sparsity=0.000000 work_per_accuracy=1775303.479616 note=wall_secs=0.0056_peak_rss=3588096_spikes=602_deliveries=6719_cells=7215_plasticity=739968
config_hash=c1-c3e47b1e5f564df6 seed=4990235250930828602 condition=dense-local accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5581315.137566 note=wall_secs=0.0115_peak_rss=4489216_spikes=1133_deliveries=61680_cells=62334_plasticity=2246912
config_hash=c1-c3e47b1e5f564df6 seed=4990235250930828602 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6707_peak_rss=2867200_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-c3e47b1e5f564df6 seed=4990235250930828602 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0151_peak_rss=2588672_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-c3e47b1e5f564df6 seed=4990235250930828602 condition=dense-matched accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2018548.178675 note=wall_secs=0.0064_peak_rss=3817472_spikes=1133_deliveries=40721_cells=41375_plasticity=774654
config_hash=c1-c3e47b1e5f564df6 seed=16390669205162672465 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1507242.000000 note=wall_secs=0.0059_peak_rss=3571712_spikes=1223_deliveries=20961_cells=21604_plasticity=709833
config_hash=c1-c3e47b1e5f564df6 seed=16390669205162672465 condition=dense-local accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5581315.137566 note=wall_secs=0.0123_peak_rss=4472832_spikes=1133_deliveries=61680_cells=62334_plasticity=2246912
config_hash=c1-c3e47b1e5f564df6 seed=16390669205162672465 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6704_peak_rss=2785280_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-c3e47b1e5f564df6 seed=16390669205162672465 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0152_peak_rss=2588672_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-c3e47b1e5f564df6 seed=16390669205162672465 condition=dense-matched accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2018251.708095 note=wall_secs=0.0065_peak_rss=4030464_spikes=1133_deliveries=41328_cells=41982_plasticity=773314
config_hash=c1-c3e47b1e5f564df6 seed=9344920893177085284 condition=local-assembly accuracy=0.425000 activity_sparsity=0.000000 activity-sparsity=0.000000 work_per_accuracy=1772785.832628 note=wall_secs=0.0054_peak_rss=3588096_spikes=532_deliveries=5715_cells=6195_plasticity=740992
config_hash=c1-c3e47b1e5f564df6 seed=9344920893177085284 condition=dense-local accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5581315.137566 note=wall_secs=0.0119_peak_rss=4407296_spikes=1133_deliveries=61680_cells=62334_plasticity=2246912
config_hash=c1-c3e47b1e5f564df6 seed=9344920893177085284 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6696_peak_rss=2867200_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-c3e47b1e5f564df6 seed=9344920893177085284 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0397_peak_rss=2605056_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-c3e47b1e5f564df6 seed=9344920893177085284 condition=dense-matched accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2018661.119849 note=wall_secs=0.0063_peak_rss=4046848_spikes=1133_deliveries=40209_cells=40863_plasticity=775726
config_hash=c1-c3e47b1e5f564df6 seed=2298610773699377531 condition=local-assembly accuracy=0.425000 activity_sparsity=0.000000 activity-sparsity=0.000000 work_per_accuracy=1760211.715333 note=wall_secs=0.0061_peak_rss=3555328_spikes=480_deliveries=4176_cells=4618_plasticity=738816
config_hash=c1-c3e47b1e5f564df6 seed=2298610773699377531 condition=dense-local accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5581315.137566 note=wall_secs=0.0121_peak_rss=4325376_spikes=1133_deliveries=61680_cells=62334_plasticity=2246912
config_hash=c1-c3e47b1e5f564df6 seed=2298610773699377531 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6759_peak_rss=2883584_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-c3e47b1e5f564df6 seed=2298610773699377531 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0158_peak_rss=2588672_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-c3e47b1e5f564df6 seed=2298610773699377531 condition=dense-matched accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2017348.178709 note=wall_secs=0.0067_peak_rss=4030464_spikes=1133_deliveries=41069_cells=41723_plasticity=773448
config_hash=c1-c3e47b1e5f564df6 seed=13699608803166074254 condition=local-assembly accuracy=0.425000 activity_sparsity=0.000000 activity-sparsity=0.000000 work_per_accuracy=1807821.125763 note=wall_secs=0.0055_peak_rss=3473408_spikes=647_deliveries=7943_cells=8464_plasticity=751270
config_hash=c1-c3e47b1e5f564df6 seed=13699608803166074254 condition=dense-local accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5581315.137566 note=wall_secs=0.0118_peak_rss=4358144_spikes=1133_deliveries=61680_cells=62334_plasticity=2246912
config_hash=c1-c3e47b1e5f564df6 seed=13699608803166074254 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6887_peak_rss=2949120_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-c3e47b1e5f564df6 seed=13699608803166074254 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0195_peak_rss=2588672_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-c3e47b1e5f564df6 seed=13699608803166074254 condition=dense-matched accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2019075.237484 note=wall_secs=0.0065_peak_rss=3981312_spikes=1133_deliveries=40967_cells=41621_plasticity=774386
config_hash=c1-c3e47b1e5f564df6 seed=6653297859054645669 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1469546.000000 note=wall_secs=0.0065_peak_rss=3702784_spikes=1098_deliveries=22965_cells=23604_plasticity=687106
config_hash=c1-c3e47b1e5f564df6 seed=6653297859054645669 condition=dense-local accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=5581315.137566 note=wall_secs=0.0137_peak_rss=4620288_spikes=1133_deliveries=61680_cells=62334_plasticity=2246912
config_hash=c1-c3e47b1e5f564df6 seed=6653297859054645669 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.7233_peak_rss=2965504_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-c3e47b1e5f564df6 seed=6653297859054645669 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0196_peak_rss=2605056_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-c3e47b1e5f564df6 seed=6653297859054645669 condition=dense-matched accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2019701.119820 note=wall_secs=0.0077_peak_rss=4046848_spikes=1133_deliveries=41435_cells=42089_plasticity=773716
```
