# C1 / Gate G2 results note

**Config hash:** `c1-befbfe8f014bda18`

**Scientific protocol version:** `5`

**Trial-isolation protocol:** `5` — clears `ThreeFactor.last_spike` and applies C3-style full dynamic membrane reset at trial boundaries; does **not** reopen protocol-v2 kill-gate hash `c1-118207fbc3eaba53` (canonical version `2`).

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
Config { experiment: "c1-iso", master_seed: 212549341544449, n_seeds: 5, sequence_len: 8, max_lag: 1, n_hidden: 64, k_wta: 1, p_sparse: 0.35, init_w: 0.15, eta: 0.35, lambda: 0.002, tau_e: 40.0, n_train: 24, n_test: 16, bptt_epochs: 40, bptt_lr: 0.05, g2_min_gap_closed: 0.5, g2_min_accuracy: 0.65, g2_confidence_z: 1.96, g2_min_positive_control: 0.9, g2_min_reference_gap: 0.15, activity_sparsity_min: 0.005, activity_sparsity_max: 0.03, scientific_n_seeds: 20, power_sigma_prior: 0.15, power_effect_size: 0.1, use_surrogate_lif_reference: true, surrogate_beta: 5.0, matched_budget_repeat: false, quick: true }
```

Power analysis: required scientific n_seeds ≥ 20 (preregistered σ=0.150, effect=0.100; formula n=⌈(1.96+0.8416)²σ²/δ²⌉).

## Per-seed accuracies

| seed | local-assembly | dense-local | gradient reference | eligibility reference | activity_sparsity (local) | activity_sparsity (dense) | dense_matched |
|---|---:|---:|---:|---:|---:|---:|---:|
| 11400784294714178580 | 0.5000 | 0.5000 | 0.5000 | 0.9375 | 0.0156 | 0.0156 | — |
| 4354473015595300907 | 0.5000 | 0.5000 | 0.7500 | 0.9375 | 0.0156 | 0.0156 | — |
| 15755469997089977406 | 0.5000 | 0.5000 | 1.0000 | 0.9375 | 0.0156 | 0.0156 | — |
| 8709160916994355285 | 0.3125 | 0.5000 | 0.7500 | 0.9375 | 0.0000 | 0.0156 | — |
| 1663413549901573224 | 0.5000 | 0.5000 | 1.0000 | 0.9375 | 0.0156 | 0.0156 | — |

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

Mean local-pipeline accuracy on a trivially separable spatial feature-presence task: **0.9000** (threshold 0.900).

## Activity sparsity

Mean local-assembly activity_sparsity: **0.0125** (valid band [0.0050, 0.0300]; nominal k/N=0.0156).

## Parameter / compute budgets

| condition | n_cells | n_params | wall_secs | peak_rss_bytes | work_per_accuracy | spikes | deliveries | cell_updates | plasticity |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| local-assembly | 68 | 1480 | 0.0027 | 3145728 | 112608.0000 | 212 | 1320 | 1492 | 53280 |
| dense-local | 68 | 4288 | 0.0029 | 3375104 | 391944.0000 | 258 | 7720 | 7898 | 180096 |
| gradient-reference | 66 | 4289 | 0.0245 | 2670592 | 9233280.0000 | 0 | 7680 | 491520 | 4117440 |
| eligibility-reference | 66 | 193 | 0.0027 | 2605056 | 730112.0000 | 0 | 7680 | 491520 | 185280 |

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
config_hash=c1-befbfe8f014bda18 seed=11400784294714178580 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=112608.000000 note=wall_secs=0.0027_peak_rss=3145728_spikes=212_deliveries=1320_cells=1492_plasticity=53280
config_hash=c1-befbfe8f014bda18 seed=11400784294714178580 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=391944.000000 note=wall_secs=0.0029_peak_rss=3375104_spikes=258_deliveries=7720_cells=7898_plasticity=180096
config_hash=c1-befbfe8f014bda18 seed=11400784294714178580 condition=gradient-reference accuracy=0.500000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9233280.000000 note=wall_secs=0.0245_peak_rss=2670592_spikes=0_deliveries=7680_cells=491520_plasticity=4117440
config_hash=c1-befbfe8f014bda18 seed=11400784294714178580 condition=eligibility-reference accuracy=0.937500 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=730112.000000 note=wall_secs=0.0027_peak_rss=2605056_spikes=0_deliveries=7680_cells=491520_plasticity=185280
config_hash=c1-befbfe8f014bda18 seed=4354473015595300907 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=116036.000000 note=wall_secs=0.0021_peak_rss=3162112_spikes=206_deliveries=1273_cells=1446_plasticity=55093
config_hash=c1-befbfe8f014bda18 seed=4354473015595300907 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=391944.000000 note=wall_secs=0.0028_peak_rss=3375104_spikes=258_deliveries=7720_cells=7898_plasticity=180096
config_hash=c1-befbfe8f014bda18 seed=4354473015595300907 condition=gradient-reference accuracy=0.750000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=6155520.000000 note=wall_secs=0.0233_peak_rss=2654208_spikes=0_deliveries=7680_cells=491520_plasticity=4117440
config_hash=c1-befbfe8f014bda18 seed=4354473015595300907 condition=eligibility-reference accuracy=0.937500 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=730112.000000 note=wall_secs=0.0025_peak_rss=2605056_spikes=0_deliveries=7680_cells=491520_plasticity=185280
config_hash=c1-befbfe8f014bda18 seed=15755469997089977406 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=115370.000000 note=wall_secs=0.0024_peak_rss=3145728_spikes=204_deliveries=1311_cells=1484_plasticity=54686
config_hash=c1-befbfe8f014bda18 seed=15755469997089977406 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=391944.000000 note=wall_secs=0.0027_peak_rss=3489792_spikes=258_deliveries=7720_cells=7898_plasticity=180096
config_hash=c1-befbfe8f014bda18 seed=15755469997089977406 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=4616640.000000 note=wall_secs=0.0228_peak_rss=2654208_spikes=0_deliveries=7680_cells=491520_plasticity=4117440
config_hash=c1-befbfe8f014bda18 seed=15755469997089977406 condition=eligibility-reference accuracy=0.937500 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=730112.000000 note=wall_secs=0.0028_peak_rss=2605056_spikes=0_deliveries=7680_cells=491520_plasticity=185280
config_hash=c1-befbfe8f014bda18 seed=8709160916994355285 condition=local-assembly accuracy=0.312500 activity_sparsity=0.000000 activity-sparsity=0.000000 work_per_accuracy=204307.200000 note=wall_secs=0.0024_peak_rss=3112960_spikes=167_deliveries=787_cells=942_plasticity=61950
config_hash=c1-befbfe8f014bda18 seed=8709160916994355285 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=391944.000000 note=wall_secs=0.0026_peak_rss=3457024_spikes=258_deliveries=7720_cells=7898_plasticity=180096
config_hash=c1-befbfe8f014bda18 seed=8709160916994355285 condition=gradient-reference accuracy=0.750000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=6155520.000000 note=wall_secs=0.0227_peak_rss=2654208_spikes=0_deliveries=7680_cells=491520_plasticity=4117440
config_hash=c1-befbfe8f014bda18 seed=8709160916994355285 condition=eligibility-reference accuracy=0.937500 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=730112.000000 note=wall_secs=0.0025_peak_rss=2605056_spikes=0_deliveries=7680_cells=491520_plasticity=185280
config_hash=c1-befbfe8f014bda18 seed=1663413549901573224 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=128260.000000 note=wall_secs=0.0022_peak_rss=3162112_spikes=220_deliveries=1342_cells=1519_plasticity=61049
config_hash=c1-befbfe8f014bda18 seed=1663413549901573224 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=391944.000000 note=wall_secs=0.0027_peak_rss=3440640_spikes=258_deliveries=7720_cells=7898_plasticity=180096
config_hash=c1-befbfe8f014bda18 seed=1663413549901573224 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=4616640.000000 note=wall_secs=0.0228_peak_rss=2670592_spikes=0_deliveries=7680_cells=491520_plasticity=4117440
config_hash=c1-befbfe8f014bda18 seed=1663413549901573224 condition=eligibility-reference accuracy=0.937500 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=730112.000000 note=wall_secs=0.0026_peak_rss=2621440_spikes=0_deliveries=7680_cells=491520_plasticity=185280
```
