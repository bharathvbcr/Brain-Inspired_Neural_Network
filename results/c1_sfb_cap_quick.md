# C1 / Gate G2 results note

**Config hash:** `c1-b72fb5d90427b358`

**Scientific protocol version:** `17`

**Structured B × capacity protocol:** `17` — v15 structured hidden `B` on the Tier-B capacity substrate (richer `k_wta` / `n_hidden` / `n_train`); single-pass; **positive control stays on broadcast ±1**; does **not** remassage v15 or capacity-only `c1-d38d7644d8afc84b`.

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
Config { experiment: "c1-sfb-cap", master_seed: 212618061021185, n_seeds: 5, sequence_len: 8, max_lag: 1, n_hidden: 128, k_wta: 2, p_sparse: 0.3, init_w: 0.15, eta: 0.2, lambda: 0.002, tau_e: 40.0, n_train: 48, n_test: 24, bptt_epochs: 40, bptt_lr: 0.02, g2_min_gap_closed: 0.5, g2_min_accuracy: 0.65, g2_confidence_z: 1.96, g2_min_positive_control: 0.9, g2_min_reference_gap: 0.15, activity_sparsity_min: 0.005, activity_sparsity_max: 0.03, scientific_n_seeds: 20, power_sigma_prior: 0.15, power_effect_size: 0.1, use_surrogate_lif_reference: true, surrogate_beta: 5.0, matched_budget_repeat: false, quick: true }
```

Power analysis: required scientific n_seeds ≥ 20 (preregistered σ=0.150, effect=0.100; formula n=⌈(1.96+0.8416)²σ²/δ²⌉).

## Per-seed accuracies

| seed | local-assembly | dense-local | gradient reference | eligibility reference | activity_sparsity (local) | activity_sparsity (dense) | dense_matched |
|---|---:|---:|---:|---:|---:|---:|---:|
| 11400784225994701844 | 0.5000 | 0.5833 | 1.0000 | 0.9583 | 0.0156 | 0.0156 | — |
| 4354472946875824171 | 0.5000 | 0.4167 | 1.0000 | 0.9583 | 0.0156 | 0.0156 | — |
| 15755469790931547198 | 0.5000 | 0.5833 | 0.5000 | 0.9583 | 0.0156 | 0.0156 | — |
| 8709160710835925077 | 0.5000 | 0.4167 | 1.0000 | 0.9583 | 0.0156 | 0.0156 | — |
| 1663413756060003432 | 1.0000 | 0.4167 | 1.0000 | 0.9583 | 0.0156 | 0.0156 | — |

## Summary (paired normalized-gap analysis)

- mean ± var local-assembly: 0.6000 ± 0.050000
- mean ± var dense-local:    0.4833 ± 0.008333
- mean ± var gradient reference: 0.9000 ± 0.050000
- mean ± var eligibility reference: 0.9583 ± 0.000000
- mean normalized gap closed: 0.2571 (variance 0.177551)
- lower confidence bound (z=1.960, n=5): -0.1122
- mean |local − dense| (descriptive): 0.1833

## Pilot limitation

This run uses a quick schedule or fewer seeds than the power-analysis requirement. It validates the harness only and is not evidence for passing or failing G2.

## Positive / sanity control

Mean local-pipeline accuracy on a trivially separable spatial feature-presence task: **1.0000** (threshold 0.900).

## Activity sparsity

Mean local-assembly activity_sparsity: **0.0156** (valid band [0.0050, 0.0300]; nominal k/N=0.0156).

## Parameter / compute budgets

| condition | n_cells | n_params | wall_secs | peak_rss_bytes | work_per_accuracy | spikes | deliveries | cell_updates | plasticity |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| local-assembly | 132 | 5009 | 0.0039 | 3604480 | 510808.0000 | 503 | 7043 | 7426 | 240432 |
| dense-local | 132 | 16768 | 0.0067 | 4407296 | 1508208.0514 | 526 | 37008 | 37390 | 804864 |
| gradient-reference | 130 | 16769 | 0.1988 | 2850816 | 34177920.0000 | 0 | 15360 | 1966080 | 32196480 |
| eligibility-reference | 130 | 385 | 0.0060 | 2621440 | 2838928.7545 | 0 | 15360 | 1966080 | 739200 |

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
config_hash=c1-b72fb5d90427b358 seed=11400784225994701844 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=510808.000000 note=wall_secs=0.0039_peak_rss=3604480_spikes=503_deliveries=7043_cells=7426_plasticity=240432
config_hash=c1-b72fb5d90427b358 seed=11400784225994701844 condition=dense-local accuracy=0.583333 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1508208.051369 note=wall_secs=0.0067_peak_rss=4407296_spikes=526_deliveries=37008_cells=37390_plasticity=804864
config_hash=c1-b72fb5d90427b358 seed=11400784225994701844 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=34177920.000000 note=wall_secs=0.1988_peak_rss=2850816_spikes=0_deliveries=15360_cells=1966080_plasticity=32196480
config_hash=c1-b72fb5d90427b358 seed=11400784225994701844 condition=eligibility-reference accuracy=0.958333 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=2838928.754509 note=wall_secs=0.0060_peak_rss=2621440_spikes=0_deliveries=15360_cells=1966080_plasticity=739200
config_hash=c1-b72fb5d90427b358 seed=4354472946875824171 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=509476.000000 note=wall_secs=0.0038_peak_rss=3588096_spikes=435_deliveries=6840_cells=7223_plasticity=240240
config_hash=c1-b72fb5d90427b358 seed=4354472946875824171 condition=dense-local accuracy=0.416667 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2111496.050342 note=wall_secs=0.0066_peak_rss=4489216_spikes=527_deliveries=37008_cells=37391_plasticity=804864
config_hash=c1-b72fb5d90427b358 seed=4354472946875824171 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=34177920.000000 note=wall_secs=0.2021_peak_rss=2883584_spikes=0_deliveries=15360_cells=1966080_plasticity=32196480
config_hash=c1-b72fb5d90427b358 seed=4354472946875824171 condition=eligibility-reference accuracy=0.958333 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=2838928.754509 note=wall_secs=0.0060_peak_rss=2637824_spikes=0_deliveries=15360_cells=1966080_plasticity=739200
config_hash=c1-b72fb5d90427b358 seed=15755469790931547198 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=509638.000000 note=wall_secs=0.0039_peak_rss=3620864_spikes=460_deliveries=6894_cells=7273_plasticity=240192
config_hash=c1-b72fb5d90427b358 seed=15755469790931547198 condition=dense-local accuracy=0.583333 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1508201.194226 note=wall_secs=0.0067_peak_rss=4374528_spikes=524_deliveries=37008_cells=37388_plasticity=804864
config_hash=c1-b72fb5d90427b358 seed=15755469790931547198 condition=gradient-reference accuracy=0.500000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=68355840.000000 note=wall_secs=0.2002_peak_rss=2801664_spikes=0_deliveries=15360_cells=1966080_plasticity=32196480
config_hash=c1-b72fb5d90427b358 seed=15755469790931547198 condition=eligibility-reference accuracy=0.958333 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=2838928.754509 note=wall_secs=0.0056_peak_rss=2621440_spikes=0_deliveries=15360_cells=1966080_plasticity=739200
config_hash=c1-b72fb5d90427b358 seed=8709160710835925077 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=511586.000000 note=wall_secs=0.0034_peak_rss=3604480_spikes=446_deliveries=6977_cells=7362_plasticity=241008
config_hash=c1-b72fb5d90427b358 seed=8709160710835925077 condition=dense-local accuracy=0.416667 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2111524.850343 note=wall_secs=0.0065_peak_rss=4374528_spikes=533_deliveries=37008_cells=37397_plasticity=804864
config_hash=c1-b72fb5d90427b358 seed=8709160710835925077 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=34177920.000000 note=wall_secs=0.2015_peak_rss=2801664_spikes=0_deliveries=15360_cells=1966080_plasticity=32196480
config_hash=c1-b72fb5d90427b358 seed=8709160710835925077 condition=eligibility-reference accuracy=0.958333 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=2838928.754509 note=wall_secs=0.0059_peak_rss=2621440_spikes=0_deliveries=15360_cells=1966080_plasticity=739200
config_hash=c1-b72fb5d90427b358 seed=1663413756060003432 condition=local-assembly accuracy=1.000000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=256211.000000 note=wall_secs=0.0038_peak_rss=3637248_spikes=512_deliveries=7110_cells=7485_plasticity=241104
config_hash=c1-b72fb5d90427b358 seed=1663413756060003432 condition=dense-local accuracy=0.416667 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2111467.250341 note=wall_secs=0.0068_peak_rss=4390912_spikes=521_deliveries=37008_cells=37385_plasticity=804864
config_hash=c1-b72fb5d90427b358 seed=1663413756060003432 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=34177920.000000 note=wall_secs=0.2003_peak_rss=2818048_spikes=0_deliveries=15360_cells=1966080_plasticity=32196480
config_hash=c1-b72fb5d90427b358 seed=1663413756060003432 condition=eligibility-reference accuracy=0.958333 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=2838928.754509 note=wall_secs=0.0057_peak_rss=2621440_spikes=0_deliveries=15360_cells=1966080_plasticity=739200
```
