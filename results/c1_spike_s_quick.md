# C1 / Gate G2 results note

**Config hash:** `c1-078cdbd91088c2f6`

**Scientific protocol version:** `9`

**Calibrated natural-spiking protocol:** `9` — finite hidden θ during integrate (no θ=∞ mute); **spike-count k-WTA** (not residual membrane) for hidden selection; disclosed multi-frame easy PC; production knobs `init_w`/`eta`/`tau_e` calibrated; trial-isolation resets; does **not** reopen v2 `c1-118207fbc3eaba53` or reinterpret v6 `c1-09442acdbdc0c752` (G2 thresholds unchanged).

**Verdict (Gate G2):** **PILOT**

PASS = lower confidence bound on normalized gradient gap closed > 0.500 and mean local accuracy >= 0.650.
FAIL = a full run missed at least one preregistered threshold.
PILOT = quick schedule or fewer seeds than the power-analysis requirement; not a scientific G2 decision.
INVALID_HARNESS = positive_control_mean < 0.900 or mean activity sparsity outside [0.0050, 0.0300]; prohibits PASS/FAIL and U-NEG language.

## Conditions

| Label | Meaning |
|---|---|
| `local-assembly` | Three-factor rule + sparse assembly wiring + k-WTA + dual readouts + two-sided ±1 reward |
| `dense-local` | Same three-factor rule + same k-winner budget on dense all-to-all connectivity, **no** assembly structure |
| `gradient-reference` | Same-architecture surrogate-LIF BPTT (primary); tanh RNN optional/secondary |
| `eligibility-reference` | E-prop-compatible eligibility local reference (rate-model approximation; feedforward-only) |

Plasticity uses hard ±1 reward by design (soft RPE deferred). Gap-closed is clamped to `[0, 1]` and seeds with `(reference − dense) < 0.150` contribute `closed = 0`.

## Config

```
Config { experiment: "c1-spike-s", master_seed: 212596586184705, n_seeds: 5, sequence_len: 8, max_lag: 1, n_hidden: 64, k_wta: 1, p_sparse: 0.35, init_w: 0.22, eta: 0.45, lambda: 0.002, tau_e: 48.0, n_train: 24, n_test: 16, bptt_epochs: 40, bptt_lr: 0.05, g2_min_gap_closed: 0.5, g2_min_accuracy: 0.65, g2_confidence_z: 1.96, g2_min_positive_control: 0.9, g2_min_reference_gap: 0.15, activity_sparsity_min: 0.005, activity_sparsity_max: 0.03, scientific_n_seeds: 20, power_sigma_prior: 0.15, power_effect_size: 0.1, use_surrogate_lif_reference: true, surrogate_beta: 5.0, matched_budget_repeat: false, quick: true }
```

Power analysis: required scientific n_seeds ≥ 20 (preregistered σ=0.150, effect=0.100; formula n=⌈(1.96+0.8416)²σ²/δ²⌉).

## Per-seed accuracies

| seed | local-assembly | dense-local | gradient reference | eligibility reference | activity_sparsity (local) | activity_sparsity (dense) | dense_matched |
|---|---:|---:|---:|---:|---:|---:|---:|
| 11400784264649407508 | 0.5000 | 0.5000 | 0.7500 | 0.9375 | 0.0156 | 0.0156 | — |
| 4354473045660071979 | 0.5000 | 0.5000 | 1.0000 | 0.9375 | 0.0156 | 0.0156 | — |
| 15755469975615140926 | 0.3125 | 0.5000 | 1.0000 | 0.9375 | 0.0000 | 0.0156 | — |
| 8709160955649060949 | 0.5000 | 0.5000 | 0.5000 | 0.9375 | 0.0156 | 0.0156 | — |
| 1663413502656932968 | 0.5000 | 0.5000 | 0.7500 | 0.9375 | 0.0156 | 0.0156 | — |

## Summary (paired normalized-gap analysis)

- mean ± var local-assembly: 0.4625 ± 0.007031
- mean ± var dense-local:    0.5000 ± 0.000000
- mean ± var gradient reference: 0.8000 ± 0.043750
- mean ± var eligibility reference: 0.9375 ± 0.000000
- mean normalized gap closed: 0.0000 (variance 0.000000)
- lower confidence bound (z=1.960, n=5): 0.0000
- mean |local − dense| (descriptive): 0.0375

## Pilot limitation

This run uses a quick schedule or fewer seeds than the power-analysis requirement. It validates the harness only and is not evidence for passing or failing G2.

## Positive / sanity control

Mean local-pipeline accuracy on a disclosed multi-frame spatial feature-presence (calibrated spike-s PC; main coincidence task unchanged) task: **1.0000** (threshold 0.900).

## Activity sparsity

Mean local-assembly activity_sparsity: **0.0125** (valid band [0.0050, 0.0300]; nominal k/N=0.0156).

## Parameter / compute budgets

| condition | n_cells | n_params | wall_secs | peak_rss_bytes | work_per_accuracy | spikes | deliveries | cell_updates | plasticity |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| local-assembly | 68 | 1483 | 0.0026 | 3063808 | 115628.0000 | 212 | 1279 | 1452 | 54871 |
| dense-local | 68 | 4288 | 0.0026 | 3358720 | 391944.0000 | 258 | 7720 | 7898 | 180096 |
| gradient-reference | 66 | 4289 | 0.0242 | 2621440 | 6155520.0000 | 0 | 7680 | 491520 | 4117440 |
| eligibility-reference | 66 | 193 | 0.0029 | 2572288 | 730112.0000 | 0 | 7680 | 491520 | 185280 |

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

## Structured log (GC7)

```
config_hash=c1-078cdbd91088c2f6 seed=11400784264649407508 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=115628.000000 note=wall_secs=0.0026_peak_rss=3063808_spikes=212_deliveries=1279_cells=1452_plasticity=54871
config_hash=c1-078cdbd91088c2f6 seed=11400784264649407508 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=391944.000000 note=wall_secs=0.0026_peak_rss=3358720_spikes=258_deliveries=7720_cells=7898_plasticity=180096
config_hash=c1-078cdbd91088c2f6 seed=11400784264649407508 condition=gradient-reference accuracy=0.750000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=6155520.000000 note=wall_secs=0.0242_peak_rss=2621440_spikes=0_deliveries=7680_cells=491520_plasticity=4117440
config_hash=c1-078cdbd91088c2f6 seed=11400784264649407508 condition=eligibility-reference accuracy=0.937500 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=730112.000000 note=wall_secs=0.0029_peak_rss=2572288_spikes=0_deliveries=7680_cells=491520_plasticity=185280
config_hash=c1-078cdbd91088c2f6 seed=4354473045660071979 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=125224.000000 note=wall_secs=0.0022_peak_rss=3014656_spikes=234_deliveries=1281_cells=1457_plasticity=59640
config_hash=c1-078cdbd91088c2f6 seed=4354473045660071979 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=391944.000000 note=wall_secs=0.0030_peak_rss=3276800_spikes=258_deliveries=7720_cells=7898_plasticity=180096
config_hash=c1-078cdbd91088c2f6 seed=4354473045660071979 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=4616640.000000 note=wall_secs=0.0232_peak_rss=2621440_spikes=0_deliveries=7680_cells=491520_plasticity=4117440
config_hash=c1-078cdbd91088c2f6 seed=4354473045660071979 condition=eligibility-reference accuracy=0.937500 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=730112.000000 note=wall_secs=0.0030_peak_rss=2555904_spikes=0_deliveries=7680_cells=491520_plasticity=185280
config_hash=c1-078cdbd91088c2f6 seed=15755469975615140926 condition=local-assembly accuracy=0.312500 activity_sparsity=0.000000 activity-sparsity=0.000000 work_per_accuracy=214403.200000 note=wall_secs=0.0021_peak_rss=3031040_spikes=176_deliveries=796_cells=953_plasticity=65076
config_hash=c1-078cdbd91088c2f6 seed=15755469975615140926 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=391944.000000 note=wall_secs=0.0027_peak_rss=3276800_spikes=258_deliveries=7720_cells=7898_plasticity=180096
config_hash=c1-078cdbd91088c2f6 seed=15755469975615140926 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=4616640.000000 note=wall_secs=0.0229_peak_rss=2605056_spikes=0_deliveries=7680_cells=491520_plasticity=4117440
config_hash=c1-078cdbd91088c2f6 seed=15755469975615140926 condition=eligibility-reference accuracy=0.937500 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=730112.000000 note=wall_secs=0.0047_peak_rss=2572288_spikes=0_deliveries=7680_cells=491520_plasticity=185280
config_hash=c1-078cdbd91088c2f6 seed=8709160955649060949 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=115658.000000 note=wall_secs=0.0031_peak_rss=3031040_spikes=215_deliveries=1322_cells=1495_plasticity=54797
config_hash=c1-078cdbd91088c2f6 seed=8709160955649060949 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=391944.000000 note=wall_secs=0.0028_peak_rss=3309568_spikes=258_deliveries=7720_cells=7898_plasticity=180096
config_hash=c1-078cdbd91088c2f6 seed=8709160955649060949 condition=gradient-reference accuracy=0.500000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9233280.000000 note=wall_secs=0.0242_peak_rss=2605056_spikes=0_deliveries=7680_cells=491520_plasticity=4117440
config_hash=c1-078cdbd91088c2f6 seed=8709160955649060949 condition=eligibility-reference accuracy=0.937500 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=730112.000000 note=wall_secs=0.0036_peak_rss=2539520_spikes=0_deliveries=7680_cells=491520_plasticity=185280
config_hash=c1-078cdbd91088c2f6 seed=1663413502656932968 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=115336.000000 note=wall_secs=0.0029_peak_rss=3063808_spikes=236_deliveries=2062_cells=2234_plasticity=53136
config_hash=c1-078cdbd91088c2f6 seed=1663413502656932968 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=391944.000000 note=wall_secs=0.0029_peak_rss=3358720_spikes=258_deliveries=7720_cells=7898_plasticity=180096
config_hash=c1-078cdbd91088c2f6 seed=1663413502656932968 condition=gradient-reference accuracy=0.750000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=6155520.000000 note=wall_secs=0.0227_peak_rss=2621440_spikes=0_deliveries=7680_cells=491520_plasticity=4117440
config_hash=c1-078cdbd91088c2f6 seed=1663413502656932968 condition=eligibility-reference accuracy=0.937500 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=730112.000000 note=wall_secs=0.0028_peak_rss=2572288_spikes=0_deliveries=7680_cells=491520_plasticity=185280
```
