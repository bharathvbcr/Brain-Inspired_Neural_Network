# C1 / Gate G2 results note

**Config hash:** `c1-a5a9220f9df9d915`

**Scientific protocol version:** `21`

**claim_axis:** Novel-CS
**object_under_test:** Soft/relaxed k-WTA winners under structured frozen B
**may_claim:** Whether soft winners under SFB close the live transfer gap
**must_not_claim:** Temperature grid search; remassage v15; biology

**Soft-WTA × structured B protocol:** `21` — v15 structured hidden `B` with soft/relaxed k-WTA at disclosed temperature `T=1` (one temp; no grid); **positive control stays on broadcast ±1**; does **not** remassage v15 hash `c1-493ddd56f8714fb6` or reopen protocol-v2 `c1-118207fbc3eaba53`.

**Verdict (Gate G2):** **PILOT**

PASS = lower confidence bound on normalized gradient gap closed > 0.500 and mean local accuracy >= 0.650.
FAIL = a full run missed at least one preregistered threshold.
PILOT = quick schedule or fewer seeds than the power-analysis requirement; not a scientific G2 decision.
INVALID_HARNESS = positive_control_mean < 0.900 or mean activity sparsity outside [0.0050, 0.0300]; prohibits PASS/FAIL and U-NEG language.

## Conditions

| Label | Meaning |
|---|---|
| `local-assembly` | Three-factor rule + sparse assembly + k-WTA + dual readouts + **`ReinforceFeedback` × `reinforce_term`** (opt-in; not broadcast ±1) |
| `dense-local` | Same three-factor + k-WTA budget on dense all-to-all, **no** assembly; same `ReinforceFeedback` neuromodulator |
| `gradient-reference` | Same-architecture surrogate-LIF BPTT (primary); tanh RNN optional/secondary |
| `eligibility-reference` | E-prop-compatible eligibility local reference (rate-model approximation; feedforward-only) |

Plasticity uses directional REINFORCE × frozen per-neuron feedback (`ReinforceFeedback`) by design; broadcast ±1 remains the default C1 path. Gap-closed is clamped to `[0, 1]` and seeds with `(reference − dense) < 0.150` contribute `closed = 0`.

## Config

```
Config { experiment: "c1-sfb-soft", master_seed: 212618061021185, n_seeds: 5, sequence_len: 8, max_lag: 1, n_hidden: 64, k_wta: 1, p_sparse: 0.35, init_w: 0.15, eta: 0.35, lambda: 0.002, tau_e: 40.0, n_train: 24, n_test: 16, bptt_epochs: 40, bptt_lr: 0.05, g2_min_gap_closed: 0.5, g2_min_accuracy: 0.65, g2_confidence_z: 1.96, g2_min_positive_control: 0.9, g2_min_reference_gap: 0.15, activity_sparsity_min: 0.005, activity_sparsity_max: 0.03, scientific_n_seeds: 20, power_sigma_prior: 0.15, power_effect_size: 0.1, use_surrogate_lif_reference: true, surrogate_beta: 5.0, matched_budget_repeat: false, quick: true }
```

Power analysis: required scientific n_seeds ≥ 20 (preregistered σ=0.150, effect=0.100; formula n=⌈(1.96+0.8416)²σ²/δ²⌉).

## Per-seed accuracies

| seed | local-assembly | dense-local | gradient reference | eligibility reference | activity_sparsity (local) | activity_sparsity (dense) | dense_matched |
|---|---:|---:|---:|---:|---:|---:|---:|
| 11400784225994701844 | 0.6875 | 0.6875 | 0.7500 | 0.8750 | 0.0156 | 0.0156 | — |
| 4354472946875824171 | 0.5625 | 0.5000 | 0.5000 | 0.9375 | 0.0156 | 0.0156 | — |
| 15755469790931547198 | 0.4375 | 0.6250 | 0.5000 | 0.9375 | 0.0156 | 0.0156 | — |
| 8709160710835925077 | 0.6875 | 0.6875 | 1.0000 | 0.9375 | 0.0156 | 0.0156 | — |
| 1663413756060003432 | 0.6875 | 0.6875 | 0.5000 | 0.9375 | 0.0156 | 0.0156 | — |

## Summary (paired normalized-gap analysis)

- mean ± var local-assembly: 0.6125 ± 0.012500
- mean ± var dense-local:    0.6375 ± 0.006641
- mean ± var gradient reference: 0.6500 ± 0.050000
- mean ± var eligibility reference: 0.9250 ± 0.000781
- mean normalized gap closed: 0.0000 (variance 0.000000)
- lower confidence bound (z=1.960, n=5): 0.0000
- mean |local − dense| (descriptive): 0.0500
- descriptive chance-normalized gap mean / LCB: 0.2250 / -0.0690 (var 0.112500; **not a gate**)
- seed local min / max / frac≥0.65: 0.4375 / 0.6875 / 0.60

## Pilot limitation

This run uses a quick schedule or fewer seeds than the power-analysis requirement. It validates the harness only and is not evidence for passing or failing G2.

## Positive / sanity control

Mean local-pipeline accuracy on a trivially separable spatial feature-presence task: **1.0000** (threshold 0.900).

## Activity sparsity

Mean local-assembly activity_sparsity: **0.0156** (valid band [0.0050, 0.0300]; nominal k/N=0.0156).

## Parameter / compute budgets

| condition | n_cells | n_params | wall_secs | peak_rss_bytes | work_per_accuracy | spikes | deliveries | cell_updates | plasticity |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| local-assembly | 68 | 1480 | 0.0022 | 3194880 | 56071.2727 | 212 | 1324 | 1493 | 35520 |
| dense-local | 68 | 4288 | 0.0022 | 3342336 | 172762.1818 | 251 | 7720 | 7891 | 102912 |
| gradient-reference | 66 | 4289 | 0.0229 | 2654208 | 6155520.0000 | 0 | 7680 | 491520 | 4117440 |
| eligibility-reference | 66 | 193 | 0.0023 | 2605056 | 782262.8571 | 0 | 7680 | 491520 | 185280 |

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
config_hash=c1-a5a9220f9df9d915 seed=11400784225994701844 condition=local-assembly accuracy=0.687500 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=56071.272727 note=wall_secs=0.0022_peak_rss=3194880_spikes=212_deliveries=1324_cells=1493_plasticity=35520
config_hash=c1-a5a9220f9df9d915 seed=11400784225994701844 condition=dense-local accuracy=0.687500 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=172762.181818 note=wall_secs=0.0022_peak_rss=3342336_spikes=251_deliveries=7720_cells=7891_plasticity=102912
config_hash=c1-a5a9220f9df9d915 seed=11400784225994701844 condition=gradient-reference accuracy=0.750000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=6155520.000000 note=wall_secs=0.0229_peak_rss=2654208_spikes=0_deliveries=7680_cells=491520_plasticity=4117440
config_hash=c1-a5a9220f9df9d915 seed=11400784225994701844 condition=eligibility-reference accuracy=0.875000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=782262.857143 note=wall_secs=0.0023_peak_rss=2605056_spikes=0_deliveries=7680_cells=491520_plasticity=185280
config_hash=c1-a5a9220f9df9d915 seed=4354472946875824171 condition=local-assembly accuracy=0.562500 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=68428.444444 note=wall_secs=0.0018_peak_rss=3178496_spikes=204_deliveries=1274_cells=1445_plasticity=35568
config_hash=c1-a5a9220f9df9d915 seed=4354472946875824171 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=237542.000000 note=wall_secs=0.0024_peak_rss=3342336_spikes=247_deliveries=7720_cells=7892_plasticity=102912
config_hash=c1-a5a9220f9df9d915 seed=4354472946875824171 condition=gradient-reference accuracy=0.500000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9233280.000000 note=wall_secs=0.0223_peak_rss=2654208_spikes=0_deliveries=7680_cells=491520_plasticity=4117440
config_hash=c1-a5a9220f9df9d915 seed=4354472946875824171 condition=eligibility-reference accuracy=0.937500 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=730112.000000 note=wall_secs=0.0027_peak_rss=2605056_spikes=0_deliveries=7680_cells=491520_plasticity=185280
config_hash=c1-a5a9220f9df9d915 seed=15755469790931547198 condition=local-assembly accuracy=0.437500 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=87856.000000 note=wall_secs=0.0020_peak_rss=3178496_spikes=218_deliveries=1288_cells=1459_plasticity=35472
config_hash=c1-a5a9220f9df9d915 seed=15755469790931547198 condition=dense-local accuracy=0.625000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=190033.600000 note=wall_secs=0.0024_peak_rss=3342336_spikes=248_deliveries=7720_cells=7891_plasticity=102912
config_hash=c1-a5a9220f9df9d915 seed=15755469790931547198 condition=gradient-reference accuracy=0.500000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9233280.000000 note=wall_secs=0.0231_peak_rss=2670592_spikes=0_deliveries=7680_cells=491520_plasticity=4117440
config_hash=c1-a5a9220f9df9d915 seed=15755469790931547198 condition=eligibility-reference accuracy=0.937500 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=730112.000000 note=wall_secs=0.0024_peak_rss=2605056_spikes=0_deliveries=7680_cells=491520_plasticity=185280
config_hash=c1-a5a9220f9df9d915 seed=8709160710835925077 condition=local-assembly accuracy=0.687500 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=55883.636364 note=wall_secs=0.0018_peak_rss=3162112_spikes=205_deliveries=1274_cells=1445_plasticity=35496
config_hash=c1-a5a9220f9df9d915 seed=8709160710835925077 condition=dense-local accuracy=0.687500 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=172756.363636 note=wall_secs=0.0023_peak_rss=3391488_spikes=247_deliveries=7720_cells=7891_plasticity=102912
config_hash=c1-a5a9220f9df9d915 seed=8709160710835925077 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=4616640.000000 note=wall_secs=0.0234_peak_rss=2654208_spikes=0_deliveries=7680_cells=491520_plasticity=4117440
config_hash=c1-a5a9220f9df9d915 seed=8709160710835925077 condition=eligibility-reference accuracy=0.937500 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=730112.000000 note=wall_secs=0.0026_peak_rss=2621440_spikes=0_deliveries=7680_cells=491520_plasticity=185280
config_hash=c1-a5a9220f9df9d915 seed=1663413756060003432 condition=local-assembly accuracy=0.687500 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=56010.181818 note=wall_secs=0.0018_peak_rss=3211264_spikes=200_deliveries=1274_cells=1441_plasticity=35592
config_hash=c1-a5a9220f9df9d915 seed=1663413756060003432 condition=dense-local accuracy=0.687500 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=172752.000000 note=wall_secs=0.0027_peak_rss=3407872_spikes=246_deliveries=7720_cells=7889_plasticity=102912
config_hash=c1-a5a9220f9df9d915 seed=1663413756060003432 condition=gradient-reference accuracy=0.500000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9233280.000000 note=wall_secs=0.0221_peak_rss=2670592_spikes=0_deliveries=7680_cells=491520_plasticity=4117440
config_hash=c1-a5a9220f9df9d915 seed=1663413756060003432 condition=eligibility-reference accuracy=0.937500 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=730112.000000 note=wall_secs=0.0025_peak_rss=2621440_spikes=0_deliveries=7680_cells=491520_plasticity=185280
```
