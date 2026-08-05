# C1 / Gate G2 results note

**Config hash:** `c1-e8722ed71f60d5d4`

**Scientific protocol version:** `16`

**Structured B × epoch-matched protocol:** `16` — v15 structured hidden `B` plus **4** local/dense epochs over the frozen train split (isolates single-pass handicap under aligned feedback); **positive control stays on broadcast ±1**; does **not** remassage v14/v15 hashes or reopen protocol-v2 `c1-118207fbc3eaba53`.

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
Config { experiment: "c1-sfb-em", master_seed: 212618061021185, n_seeds: 5, sequence_len: 8, max_lag: 1, n_hidden: 64, k_wta: 1, p_sparse: 0.35, init_w: 0.15, eta: 0.35, lambda: 0.002, tau_e: 40.0, n_train: 24, n_test: 16, bptt_epochs: 40, bptt_lr: 0.05, g2_min_gap_closed: 0.5, g2_min_accuracy: 0.65, g2_confidence_z: 1.96, g2_min_positive_control: 0.9, g2_min_reference_gap: 0.15, activity_sparsity_min: 0.005, activity_sparsity_max: 0.03, scientific_n_seeds: 20, power_sigma_prior: 0.15, power_effect_size: 0.1, use_surrogate_lif_reference: true, surrogate_beta: 5.0, matched_budget_repeat: false, quick: true }
```

Power analysis: required scientific n_seeds ≥ 20 (preregistered σ=0.150, effect=0.100; formula n=⌈(1.96+0.8416)²σ²/δ²⌉).

## Per-seed accuracies

| seed | local-assembly | dense-local | gradient reference | eligibility reference | activity_sparsity (local) | activity_sparsity (dense) | dense_matched |
|---|---:|---:|---:|---:|---:|---:|---:|
| 11400784225994701844 | 0.9375 | 0.1250 | 0.7500 | 0.8750 | 0.0156 | 0.0156 | — |
| 4354472946875824171 | 0.5000 | 0.3750 | 0.5000 | 0.9375 | 0.0156 | 0.0156 | — |
| 15755469790931547198 | 1.0000 | 0.5625 | 0.5000 | 0.9375 | 0.0156 | 0.0156 | — |
| 8709160710835925077 | 0.5000 | 0.6250 | 1.0000 | 0.9375 | 0.0156 | 0.0156 | — |
| 1663413756060003432 | 0.9375 | 0.7500 | 0.5000 | 0.9375 | 0.0156 | 0.0156 | — |

## Summary (paired normalized-gap analysis)

- mean ± var local-assembly: 0.7750 ± 0.063672
- mean ± var dense-local:    0.4875 ± 0.059375
- mean ± var gradient reference: 0.6500 ± 0.050000
- mean ± var eligibility reference: 0.9250 ± 0.000781
- mean normalized gap closed: 0.2000 (variance 0.200000)
- lower confidence bound (z=1.960, n=5): -0.1920
- mean |local − dense| (descriptive): 0.3375

## Pilot limitation

This run uses a quick schedule or fewer seeds than the power-analysis requirement. It validates the harness only and is not evidence for passing or failing G2.

## Positive / sanity control

Mean local-pipeline accuracy on a trivially separable spatial feature-presence task: **1.0000** (threshold 0.900).

## Activity sparsity

Mean local-assembly activity_sparsity: **0.0156** (valid band [0.0050, 0.0300]; nominal k/N=0.0156).

## Parameter / compute budgets

| condition | n_cells | n_params | wall_secs | peak_rss_bytes | work_per_accuracy | spikes | deliveries | cell_updates | plasticity |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| local-assembly | 68 | 1480 | 0.0036 | 3244032 | 160346.6667 | 522 | 3617 | 4106 | 142080 |
| dense-local | 68 | 4288 | 0.0052 | 3489792 | 3648640.0000 | 711 | 21616 | 22105 | 411648 |
| gradient-reference | 66 | 4289 | 0.0257 | 2654208 | 6155520.0000 | 0 | 7680 | 491520 | 4117440 |
| eligibility-reference | 66 | 193 | 0.0026 | 2605056 | 782262.8571 | 0 | 7680 | 491520 | 185280 |

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
config_hash=c1-e8722ed71f60d5d4 seed=11400784225994701844 condition=local-assembly accuracy=0.937500 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=160346.666667 note=wall_secs=0.0036_peak_rss=3244032_spikes=522_deliveries=3617_cells=4106_plasticity=142080
config_hash=c1-e8722ed71f60d5d4 seed=11400784225994701844 condition=dense-local accuracy=0.125000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=3648640.000000 note=wall_secs=0.0052_peak_rss=3489792_spikes=711_deliveries=21616_cells=22105_plasticity=411648
config_hash=c1-e8722ed71f60d5d4 seed=11400784225994701844 condition=gradient-reference accuracy=0.750000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=6155520.000000 note=wall_secs=0.0257_peak_rss=2654208_spikes=0_deliveries=7680_cells=491520_plasticity=4117440
config_hash=c1-e8722ed71f60d5d4 seed=11400784225994701844 condition=eligibility-reference accuracy=0.875000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=782262.857143 note=wall_secs=0.0026_peak_rss=2605056_spikes=0_deliveries=7680_cells=491520_plasticity=185280
config_hash=c1-e8722ed71f60d5d4 seed=4354472946875824171 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=301138.000000 note=wall_secs=0.0034_peak_rss=3244032_spikes=611_deliveries=3597_cells=4089_plasticity=142272
config_hash=c1-e8722ed71f60d5d4 seed=4354472946875824171 condition=dense-local accuracy=0.375000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1216213.333333 note=wall_secs=0.0046_peak_rss=3489792_spikes=708_deliveries=21616_cells=22108_plasticity=411648
config_hash=c1-e8722ed71f60d5d4 seed=4354472946875824171 condition=gradient-reference accuracy=0.500000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9233280.000000 note=wall_secs=0.0230_peak_rss=2670592_spikes=0_deliveries=7680_cells=491520_plasticity=4117440
config_hash=c1-e8722ed71f60d5d4 seed=4354472946875824171 condition=eligibility-reference accuracy=0.937500 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=730112.000000 note=wall_secs=0.0027_peak_rss=2605056_spikes=0_deliveries=7680_cells=491520_plasticity=185280
config_hash=c1-e8722ed71f60d5d4 seed=15755469790931547198 condition=local-assembly accuracy=1.000000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=150130.000000 note=wall_secs=0.0030_peak_rss=3358720_spikes=577_deliveries=3588_cells=4077_plasticity=141888
config_hash=c1-e8722ed71f60d5d4 seed=15755469790931547198 condition=dense-local accuracy=0.562500 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=810787.555556 note=wall_secs=0.0047_peak_rss=3407872_spikes=695_deliveries=21616_cells=22109_plasticity=411648
config_hash=c1-e8722ed71f60d5d4 seed=15755469790931547198 condition=gradient-reference accuracy=0.500000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9233280.000000 note=wall_secs=0.0242_peak_rss=2637824_spikes=0_deliveries=7680_cells=491520_plasticity=4117440
config_hash=c1-e8722ed71f60d5d4 seed=15755469790931547198 condition=eligibility-reference accuracy=0.937500 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=730112.000000 note=wall_secs=0.0026_peak_rss=2605056_spikes=0_deliveries=7680_cells=491520_plasticity=185280
config_hash=c1-e8722ed71f60d5d4 seed=8709160710835925077 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=300554.000000 note=wall_secs=0.0030_peak_rss=3375104_spikes=574_deliveries=3612_cells=4107_plasticity=141984
config_hash=c1-e8722ed71f60d5d4 seed=8709160710835925077 condition=dense-local accuracy=0.625000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=729694.400000 note=wall_secs=0.0048_peak_rss=3407872_spikes=685_deliveries=21616_cells=22110_plasticity=411648
config_hash=c1-e8722ed71f60d5d4 seed=8709160710835925077 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=4616640.000000 note=wall_secs=0.0232_peak_rss=2670592_spikes=0_deliveries=7680_cells=491520_plasticity=4117440
config_hash=c1-e8722ed71f60d5d4 seed=8709160710835925077 condition=eligibility-reference accuracy=0.937500 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=730112.000000 note=wall_secs=0.0033_peak_rss=2605056_spikes=0_deliveries=7680_cells=491520_plasticity=185280
config_hash=c1-e8722ed71f60d5d4 seed=1663413756060003432 condition=local-assembly accuracy=0.937500 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=160683.733333 note=wall_secs=0.0030_peak_rss=3227648_spikes=568_deliveries=3610_cells=4095_plasticity=142368
config_hash=c1-e8722ed71f60d5d4 seed=1663413756060003432 condition=dense-local accuracy=0.750000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=608102.666667 note=wall_secs=0.0049_peak_rss=3571712_spikes=703_deliveries=21616_cells=22110_plasticity=411648
config_hash=c1-e8722ed71f60d5d4 seed=1663413756060003432 condition=gradient-reference accuracy=0.500000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9233280.000000 note=wall_secs=0.0235_peak_rss=2654208_spikes=0_deliveries=7680_cells=491520_plasticity=4117440
config_hash=c1-e8722ed71f60d5d4 seed=1663413756060003432 condition=eligibility-reference accuracy=0.937500 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=730112.000000 note=wall_secs=0.0026_peak_rss=2605056_spikes=0_deliveries=7680_cells=491520_plasticity=185280
```
