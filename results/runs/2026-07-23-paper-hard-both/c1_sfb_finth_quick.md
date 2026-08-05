# C1 / Gate G2 results note

**Config hash:** `c1-e1dfffc003d8a616`

**Scientific protocol version:** `23`

**claim_axis:** Integrity
**object_under_test:** θ=∞ mute confounder under structured-B credit
**may_claim:** Turning mute off (finite θ) under SFB changes / does not change G2
**must_not_claim:** Spike-PC remassage; biology; remassage v15

**Finite-θ under SFB protocol:** `23` — v15 structured hidden `B` with **finite θ during integrate** (no θ=∞ mute) + trial-isolation resets; **positive control stays on broadcast ±1**; does **not** remassage v15 or reopen protocol-v2 `c1-118207fbc3eaba53`.

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
Config { experiment: "c1-sfb-finth", master_seed: 212618061021185, n_seeds: 5, sequence_len: 8, max_lag: 1, n_hidden: 64, k_wta: 1, p_sparse: 0.35, init_w: 0.15, eta: 0.35, lambda: 0.002, tau_e: 40.0, n_train: 24, n_test: 16, bptt_epochs: 40, bptt_lr: 0.05, g2_min_gap_closed: 0.5, g2_min_accuracy: 0.65, g2_confidence_z: 1.96, g2_min_positive_control: 0.9, g2_min_reference_gap: 0.15, activity_sparsity_min: 0.005, activity_sparsity_max: 0.03, scientific_n_seeds: 20, power_sigma_prior: 0.15, power_effect_size: 0.1, use_surrogate_lif_reference: true, surrogate_beta: 5.0, matched_budget_repeat: false, quick: true }
```

Power analysis: required scientific n_seeds ≥ 20 (preregistered σ=0.150, effect=0.100; formula n=⌈(1.96+0.8416)²σ²/δ²⌉).

## Per-seed accuracies

| seed | local-assembly | dense-local | gradient reference | eligibility reference | activity_sparsity (local) | activity_sparsity (dense) | dense_matched |
|---|---:|---:|---:|---:|---:|---:|---:|
| 11400784225994701844 | 0.7500 | 0.6875 | 0.7500 | 0.8750 | 0.0156 | 0.0156 | — |
| 4354472946875824171 | 0.7500 | 0.3125 | 0.5000 | 0.9375 | 0.0156 | 0.0156 | — |
| 15755469790931547198 | 0.7500 | 0.6875 | 0.5000 | 0.9375 | 0.0156 | 0.0156 | — |
| 8709160710835925077 | 0.5000 | 0.3125 | 1.0000 | 0.9375 | 0.0156 | 0.0156 | — |
| 1663413756060003432 | 0.5000 | 0.6875 | 0.5000 | 0.9375 | 0.0156 | 0.0156 | — |

## Summary (paired normalized-gap analysis)

- mean ± var local-assembly: 0.6500 ± 0.018750
- mean ± var dense-local:    0.5375 ± 0.042188
- mean ± var gradient reference: 0.6500 ± 0.050000
- mean ± var eligibility reference: 0.9250 ± 0.000781
- mean normalized gap closed: 0.2545 (variance 0.187603)
- lower confidence bound (z=1.960, n=5): -0.1251
- mean |local − dense| (descriptive): 0.1875
- descriptive chance-normalized gap mean / LCB: 0.2000 / -0.1920 (var 0.200000; **not a gate**)
- seed local min / max / frac≥0.65: 0.5000 / 0.7500 / 0.60

## Pilot limitation

This run uses a quick schedule or fewer seeds than the power-analysis requirement. It validates the harness only and is not evidence for passing or failing G2.

## Positive / sanity control

Mean local-pipeline accuracy on a trivially separable spatial feature-presence task: **1.0000** (threshold 0.900).

## Activity sparsity

Mean local-assembly activity_sparsity: **0.0156** (valid band [0.0050, 0.0300]; nominal k/N=0.0156).

## Parameter / compute budgets

| condition | n_cells | n_params | wall_secs | peak_rss_bytes | work_per_accuracy | spikes | deliveries | cell_updates | plasticity |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| local-assembly | 68 | 1480 | 0.0022 | 3129344 | 51380.0000 | 208 | 1319 | 1488 | 35520 |
| dense-local | 68 | 4288 | 0.0024 | 3407872 | 172762.1818 | 251 | 7720 | 7891 | 102912 |
| gradient-reference | 66 | 4289 | 0.0233 | 2637824 | 6155520.0000 | 0 | 7680 | 491520 | 4117440 |
| eligibility-reference | 66 | 193 | 0.0026 | 2621440 | 782262.8571 | 0 | 7680 | 491520 | 185280 |

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
config_hash=c1-e1dfffc003d8a616 seed=11400784225994701844 condition=local-assembly accuracy=0.750000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=51380.000000 note=wall_secs=0.0022_peak_rss=3129344_spikes=208_deliveries=1319_cells=1488_plasticity=35520
config_hash=c1-e1dfffc003d8a616 seed=11400784225994701844 condition=dense-local accuracy=0.687500 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=172762.181818 note=wall_secs=0.0024_peak_rss=3407872_spikes=251_deliveries=7720_cells=7891_plasticity=102912
config_hash=c1-e1dfffc003d8a616 seed=11400784225994701844 condition=gradient-reference accuracy=0.750000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=6155520.000000 note=wall_secs=0.0233_peak_rss=2637824_spikes=0_deliveries=7680_cells=491520_plasticity=4117440
config_hash=c1-e1dfffc003d8a616 seed=11400784225994701844 condition=eligibility-reference accuracy=0.875000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=782262.857143 note=wall_secs=0.0026_peak_rss=2621440_spikes=0_deliveries=7680_cells=491520_plasticity=185280
config_hash=c1-e1dfffc003d8a616 seed=4354472946875824171 condition=local-assembly accuracy=0.750000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=51393.333333 note=wall_secs=0.0018_peak_rss=3096576_spikes=215_deliveries=1295_cells=1467_plasticity=35568
config_hash=c1-e1dfffc003d8a616 seed=4354472946875824171 condition=dense-local accuracy=0.312500 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=380083.200000 note=wall_secs=0.0026_peak_rss=3391488_spikes=252_deliveries=7720_cells=7892_plasticity=102912
config_hash=c1-e1dfffc003d8a616 seed=4354472946875824171 condition=gradient-reference accuracy=0.500000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9233280.000000 note=wall_secs=0.0219_peak_rss=2670592_spikes=0_deliveries=7680_cells=491520_plasticity=4117440
config_hash=c1-e1dfffc003d8a616 seed=4354472946875824171 condition=eligibility-reference accuracy=0.937500 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=730112.000000 note=wall_secs=0.0024_peak_rss=2588672_spikes=0_deliveries=7680_cells=491520_plasticity=185280
config_hash=c1-e1dfffc003d8a616 seed=15755469790931547198 condition=local-assembly accuracy=0.750000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=51230.666667 note=wall_secs=0.0023_peak_rss=3129344_spikes=201_deliveries=1291_cells=1459_plasticity=35472
config_hash=c1-e1dfffc003d8a616 seed=15755469790931547198 condition=dense-local accuracy=0.687500 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=172762.181818 note=wall_secs=0.0023_peak_rss=3375104_spikes=251_deliveries=7720_cells=7891_plasticity=102912
config_hash=c1-e1dfffc003d8a616 seed=15755469790931547198 condition=gradient-reference accuracy=0.500000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9233280.000000 note=wall_secs=0.0221_peak_rss=2654208_spikes=0_deliveries=7680_cells=491520_plasticity=4117440
config_hash=c1-e1dfffc003d8a616 seed=15755469790931547198 condition=eligibility-reference accuracy=0.937500 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=730112.000000 note=wall_secs=0.0023_peak_rss=2605056_spikes=0_deliveries=7680_cells=491520_plasticity=185280
config_hash=c1-e1dfffc003d8a616 seed=8709160710835925077 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=76922.000000 note=wall_secs=0.0018_peak_rss=3129344_spikes=201_deliveries=1297_cells=1467_plasticity=35496
config_hash=c1-e1dfffc003d8a616 seed=8709160710835925077 condition=dense-local accuracy=0.312500 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=380083.200000 note=wall_secs=0.0025_peak_rss=3391488_spikes=252_deliveries=7720_cells=7892_plasticity=102912
config_hash=c1-e1dfffc003d8a616 seed=8709160710835925077 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=4616640.000000 note=wall_secs=0.0222_peak_rss=2637824_spikes=0_deliveries=7680_cells=491520_plasticity=4117440
config_hash=c1-e1dfffc003d8a616 seed=8709160710835925077 condition=eligibility-reference accuracy=0.937500 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=730112.000000 note=wall_secs=0.0025_peak_rss=2605056_spikes=0_deliveries=7680_cells=491520_plasticity=185280
config_hash=c1-e1dfffc003d8a616 seed=1663413756060003432 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=77094.000000 note=wall_secs=0.0021_peak_rss=3129344_spikes=202_deliveries=1291_cells=1462_plasticity=35592
config_hash=c1-e1dfffc003d8a616 seed=1663413756060003432 condition=dense-local accuracy=0.687500 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=172756.363636 note=wall_secs=0.0022_peak_rss=3391488_spikes=249_deliveries=7720_cells=7889_plasticity=102912
config_hash=c1-e1dfffc003d8a616 seed=1663413756060003432 condition=gradient-reference accuracy=0.500000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9233280.000000 note=wall_secs=0.0231_peak_rss=2654208_spikes=0_deliveries=7680_cells=491520_plasticity=4117440
config_hash=c1-e1dfffc003d8a616 seed=1663413756060003432 condition=eligibility-reference accuracy=0.937500 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=730112.000000 note=wall_secs=0.0024_peak_rss=2588672_spikes=0_deliveries=7680_cells=491520_plasticity=185280
```
