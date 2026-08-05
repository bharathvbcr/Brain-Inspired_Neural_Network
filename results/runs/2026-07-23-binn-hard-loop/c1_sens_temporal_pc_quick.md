# C1 / Gate G2 results note

**Config hash:** `c1-097696ca34d8a34d`

**Scientific protocol version:** `3`

**Sensitivity protocol (Tier-B):** `3` — optional confound probe; does **not** reopen protocol-v2 kill-gate hash `c1-118207fbc3eaba53` (canonical version `2`).

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
Config { experiment: "c1-sens-temporal-pc", master_seed: 212746910040065, n_seeds: 5, sequence_len: 8, max_lag: 1, n_hidden: 128, k_wta: 2, p_sparse: 0.35, init_w: 0.15, eta: 0.45, lambda: 0.002, tau_e: 40.0, n_train: 128, n_test: 40, bptt_epochs: 40, bptt_lr: 0.05, g2_min_gap_closed: 0.5, g2_min_accuracy: 0.65, g2_confidence_z: 1.96, g2_min_positive_control: 0.9, g2_min_reference_gap: 0.15, activity_sparsity_min: 0.005, activity_sparsity_max: 0.03, scientific_n_seeds: 20, power_sigma_prior: 0.15, power_effect_size: 0.1, use_surrogate_lif_reference: true, surrogate_beta: 5.0, matched_budget_repeat: false, quick: true }
```

Power analysis: required scientific n_seeds ≥ 20 (preregistered σ=0.150, effect=0.100; formula n=⌈(1.96+0.8416)²σ²/δ²⌉).

## Per-seed accuracies

| seed | local-assembly | dense-local | gradient reference | eligibility reference | activity_sparsity (local) | activity_sparsity (dense) | dense_matched |
|---|---:|---:|---:|---:|---:|---:|---:|
| 11400784148685290516 | 0.5000 | 0.6500 | 1.0000 | 1.0000 | 0.0156 | 0.0035 | — |
| 4354472921106020395 | 0.7000 | 0.6500 | 0.9500 | 1.0000 | 0.0156 | 0.0035 | — |
| 15755469816701350974 | 0.6250 | 0.6500 | 0.9000 | 1.0000 | 0.0156 | 0.0035 | — |
| 8709160805325205589 | 0.5000 | 0.6500 | 0.8750 | 1.0000 | 0.0156 | 0.0035 | — |
| 1663413627210984552 | 0.5750 | 0.6500 | 1.0000 | 1.0000 | 0.0156 | 0.0035 | — |

## Summary (paired normalized-gap analysis)

- mean ± var local-assembly: 0.5800 ± 0.007312
- mean ± var dense-local:    0.6500 ± 0.000000
- mean ± var gradient reference: 0.9450 ± 0.003250
- mean ± var eligibility reference: 1.0000 ± 0.000000
- mean normalized gap closed: 0.0333 (variance 0.005556)
- lower confidence bound (z=1.960, n=5): -0.0320
- mean |local − dense| (descriptive): 0.0900

## Pilot limitation

This run uses a quick schedule or fewer seeds than the power-analysis requirement. It validates the harness only and is not evidence for passing or failing G2.

## Positive / sanity control

Mean local-pipeline accuracy on a temporal coincidence-lag positive-control task: **0.9350** (threshold 0.900).

## Activity sparsity

Mean local-assembly activity_sparsity: **0.0156** (valid band [0.0050, 0.0300]; nominal k/N=0.0156).

## Parameter / compute budgets

| condition | n_cells | n_params | wall_secs | peak_rss_bytes | work_per_accuracy | spikes | deliveries | cell_updates | plasticity |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| local-assembly | 132 | 5781 | 0.0075 | 3801088 | 2309354.0000 | 1075 | 18482 | 19387 | 1115733 |
| dense-local | 132 | 16768 | 0.0164 | 4620288 | 6021184.8362 | 1192 | 77838 | 78708 | 3756032 |
| gradient-reference | 130 | 16769 | 0.5722 | 3063808 | 91141120.0000 | 0 | 40960 | 5242880 | 85857280 |
| eligibility-reference | 130 | 385 | 0.0130 | 2719744 | 7255040.0000 | 0 | 40960 | 5242880 | 1971200 |

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
config_hash=c1-097696ca34d8a34d seed=11400784148685290516 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2309354.000000 note=wall_secs=0.0075_peak_rss=3801088_spikes=1075_deliveries=18482_cells=19387_plasticity=1115733
config_hash=c1-097696ca34d8a34d seed=11400784148685290516 condition=dense-local accuracy=0.650000 activity_sparsity=0.003516 activity-sparsity=0.003516 work_per_accuracy=6021184.836240 note=wall_secs=0.0164_peak_rss=4620288_spikes=1192_deliveries=77838_cells=78708_plasticity=3756032
config_hash=c1-097696ca34d8a34d seed=11400784148685290516 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=91141120.000000 note=wall_secs=0.5722_peak_rss=3063808_spikes=0_deliveries=40960_cells=5242880_plasticity=85857280
config_hash=c1-097696ca34d8a34d seed=11400784148685290516 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=7255040.000000 note=wall_secs=0.0130_peak_rss=2719744_spikes=0_deliveries=40960_cells=5242880_plasticity=1971200
config_hash=c1-097696ca34d8a34d seed=4354472921106020395 condition=local-assembly accuracy=0.700000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1653045.742437 note=wall_secs=0.0069_peak_rss=3833856_spikes=936_deliveries=18814_cells=19719_plasticity=1117663
config_hash=c1-097696ca34d8a34d seed=4354472921106020395 condition=dense-local accuracy=0.650000 activity_sparsity=0.003516 activity-sparsity=0.003516 work_per_accuracy=6021184.836240 note=wall_secs=0.0160_peak_rss=4620288_spikes=1192_deliveries=77838_cells=78708_plasticity=3756032
config_hash=c1-097696ca34d8a34d seed=4354472921106020395 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=95938022.256495 note=wall_secs=0.5355_peak_rss=2981888_spikes=0_deliveries=40960_cells=5242880_plasticity=85857280
config_hash=c1-097696ca34d8a34d seed=4354472921106020395 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=7255040.000000 note=wall_secs=0.0125_peak_rss=2719744_spikes=0_deliveries=40960_cells=5242880_plasticity=1971200
config_hash=c1-097696ca34d8a34d seed=15755469816701350974 condition=local-assembly accuracy=0.625000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1882516.800000 note=wall_secs=0.0071_peak_rss=3866624_spikes=1129_deliveries=18627_cells=19536_plasticity=1137281
config_hash=c1-097696ca34d8a34d seed=15755469816701350974 condition=dense-local accuracy=0.650000 activity_sparsity=0.003516 activity-sparsity=0.003516 work_per_accuracy=6021184.836240 note=wall_secs=0.0165_peak_rss=4603904_spikes=1192_deliveries=77838_cells=78708_plasticity=3756032
config_hash=c1-097696ca34d8a34d seed=15755469816701350974 condition=gradient-reference accuracy=0.900000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=101267913.793795 note=wall_secs=0.5366_peak_rss=2981888_spikes=0_deliveries=40960_cells=5242880_plasticity=85857280
config_hash=c1-097696ca34d8a34d seed=15755469816701350974 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=7255040.000000 note=wall_secs=0.0124_peak_rss=2736128_spikes=0_deliveries=40960_cells=5242880_plasticity=1971200
config_hash=c1-097696ca34d8a34d seed=8709160805325205589 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2296158.000000 note=wall_secs=0.0067_peak_rss=3801088_spikes=1073_deliveries=18651_cells=19555_plasticity=1108800
config_hash=c1-097696ca34d8a34d seed=8709160805325205589 condition=dense-local accuracy=0.650000 activity_sparsity=0.003516 activity-sparsity=0.003516 work_per_accuracy=6021184.836240 note=wall_secs=0.0164_peak_rss=4620288_spikes=1192_deliveries=77838_cells=78708_plasticity=3756032
config_hash=c1-097696ca34d8a34d seed=8709160805325205589 condition=gradient-reference accuracy=0.875000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=104161280.000000 note=wall_secs=0.5396_peak_rss=2899968_spikes=0_deliveries=40960_cells=5242880_plasticity=85857280
config_hash=c1-097696ca34d8a34d seed=8709160805325205589 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=7255040.000000 note=wall_secs=0.0127_peak_rss=2719744_spikes=0_deliveries=40960_cells=5242880_plasticity=1971200
config_hash=c1-097696ca34d8a34d seed=1663413627210984552 condition=local-assembly accuracy=0.575000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2043389.607581 note=wall_secs=0.0080_peak_rss=3850240_spikes=1135_deliveries=18600_cells=19509_plasticity=1135705
config_hash=c1-097696ca34d8a34d seed=1663413627210984552 condition=dense-local accuracy=0.650000 activity_sparsity=0.003516 activity-sparsity=0.003516 work_per_accuracy=6021184.836240 note=wall_secs=0.0164_peak_rss=4587520_spikes=1192_deliveries=77838_cells=78708_plasticity=3756032
config_hash=c1-097696ca34d8a34d seed=1663413627210984552 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=91141120.000000 note=wall_secs=0.5347_peak_rss=2899968_spikes=0_deliveries=40960_cells=5242880_plasticity=85857280
config_hash=c1-097696ca34d8a34d seed=1663413627210984552 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=7255040.000000 note=wall_secs=0.0127_peak_rss=2719744_spikes=0_deliveries=40960_cells=5242880_plasticity=1971200
```
