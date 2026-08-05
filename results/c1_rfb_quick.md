# C1 / Gate G2 results note

**Config hash:** `c1-a57975f13b73a599`

**Scientific protocol version:** `13`

**Live `ReinforceFeedback` protocol:** `13` — same k-WTA / single-pass C1 substrate as v2; main-condition plasticity uses production `ReinforceFeedback` × sampled `reinforce_term` (Bernoulli action from soft readout policy); **positive control stays on broadcast ±1** with a disclosed longer easy-PC schedule (substrate/encoding check; G2 floors unchanged); does **not** reopen protocol-v2 kill-gate hash `c1-118207fbc3eaba53` (canonical version `2`), remassage P4 spiking-DFA, or retune P5 `rl_graded`.

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
Config { experiment: "c1-rfb", master_seed: 212618061021185, n_seeds: 5, sequence_len: 8, max_lag: 1, n_hidden: 64, k_wta: 1, p_sparse: 0.35, init_w: 0.15, eta: 0.35, lambda: 0.002, tau_e: 40.0, n_train: 24, n_test: 16, bptt_epochs: 40, bptt_lr: 0.05, g2_min_gap_closed: 0.5, g2_min_accuracy: 0.65, g2_confidence_z: 1.96, g2_min_positive_control: 0.9, g2_min_reference_gap: 0.15, activity_sparsity_min: 0.005, activity_sparsity_max: 0.03, scientific_n_seeds: 20, power_sigma_prior: 0.15, power_effect_size: 0.1, use_surrogate_lif_reference: true, surrogate_beta: 5.0, matched_budget_repeat: false, quick: true }
```

Power analysis: required scientific n_seeds ≥ 20 (preregistered σ=0.150, effect=0.100; formula n=⌈(1.96+0.8416)²σ²/δ²⌉).

## Per-seed accuracies

| seed | local-assembly | dense-local | gradient reference | eligibility reference | activity_sparsity (local) | activity_sparsity (dense) | dense_matched |
|---|---:|---:|---:|---:|---:|---:|---:|
| 11400784225994701844 | 0.7500 | 0.5000 | 0.7500 | 0.8750 | 0.0156 | 0.0156 | — |
| 4354472946875824171 | 0.5000 | 0.3125 | 0.5000 | 0.9375 | 0.0156 | 0.0156 | — |
| 15755469790931547198 | 0.6875 | 0.5625 | 0.5000 | 0.9375 | 0.0156 | 0.0156 | — |
| 8709160710835925077 | 0.3125 | 0.5625 | 1.0000 | 0.9375 | 0.0156 | 0.0156 | — |
| 1663413756060003432 | 0.7500 | 0.3750 | 0.5000 | 0.9375 | 0.0156 | 0.0156 | — |

## Summary (paired normalized-gap analysis)

- mean ± var local-assembly: 0.6000 ± 0.036328
- mean ± var dense-local:    0.4625 ± 0.012891
- mean ± var gradient reference: 0.6500 ± 0.050000
- mean ± var eligibility reference: 0.9250 ± 0.000781
- mean normalized gap closed: 0.4000 (variance 0.300000)
- lower confidence bound (z=1.960, n=5): -0.0801
- mean |local − dense| (descriptive): 0.2375

## Pilot limitation

This run uses a quick schedule or fewer seeds than the power-analysis requirement. It validates the harness only and is not evidence for passing or failing G2.

## Positive / sanity control

Mean local-pipeline accuracy on a trivially separable spatial feature-presence task: **1.0000** (threshold 0.900).

## Activity sparsity

Mean local-assembly activity_sparsity: **0.0156** (valid band [0.0050, 0.0300]; nominal k/N=0.0156).

## Parameter / compute budgets

| condition | n_cells | n_params | wall_secs | peak_rss_bytes | work_per_accuracy | spikes | deliveries | cell_updates | plasticity |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| local-assembly | 68 | 1480 | 0.0029 | 3162112 | 51378.6667 | 208 | 1318 | 1488 | 35520 |
| dense-local | 68 | 4288 | 0.0029 | 3424256 | 237548.0000 | 251 | 7720 | 7891 | 102912 |
| gradient-reference | 66 | 4289 | 0.0250 | 2621440 | 6155520.0000 | 0 | 7680 | 491520 | 4117440 |
| eligibility-reference | 66 | 193 | 0.0028 | 2572288 | 782262.8571 | 0 | 7680 | 491520 | 185280 |

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
config_hash=c1-a57975f13b73a599 seed=11400784225994701844 condition=local-assembly accuracy=0.750000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=51378.666667 note=wall_secs=0.0029_peak_rss=3162112_spikes=208_deliveries=1318_cells=1488_plasticity=35520
config_hash=c1-a57975f13b73a599 seed=11400784225994701844 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=237548.000000 note=wall_secs=0.0029_peak_rss=3424256_spikes=251_deliveries=7720_cells=7891_plasticity=102912
config_hash=c1-a57975f13b73a599 seed=11400784225994701844 condition=gradient-reference accuracy=0.750000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=6155520.000000 note=wall_secs=0.0250_peak_rss=2621440_spikes=0_deliveries=7680_cells=491520_plasticity=4117440
config_hash=c1-a57975f13b73a599 seed=11400784225994701844 condition=eligibility-reference accuracy=0.875000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=782262.857143 note=wall_secs=0.0028_peak_rss=2572288_spikes=0_deliveries=7680_cells=491520_plasticity=185280
config_hash=c1-a57975f13b73a599 seed=4354472946875824171 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=77090.000000 note=wall_secs=0.0020_peak_rss=3276800_spikes=213_deliveries=1296_cells=1468_plasticity=35568
config_hash=c1-a57975f13b73a599 seed=4354472946875824171 condition=dense-local accuracy=0.312500 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=380044.800000 note=wall_secs=0.0024_peak_rss=3473408_spikes=240_deliveries=7720_cells=7892_plasticity=102912
config_hash=c1-a57975f13b73a599 seed=4354472946875824171 condition=gradient-reference accuracy=0.500000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9233280.000000 note=wall_secs=0.0225_peak_rss=2654208_spikes=0_deliveries=7680_cells=491520_plasticity=4117440
config_hash=c1-a57975f13b73a599 seed=4354472946875824171 condition=eligibility-reference accuracy=0.937500 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=730112.000000 note=wall_secs=0.0027_peak_rss=2605056_spikes=0_deliveries=7680_cells=491520_plasticity=185280
config_hash=c1-a57975f13b73a599 seed=15755469790931547198 condition=local-assembly accuracy=0.687500 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=55700.363636 note=wall_secs=0.0026_peak_rss=3178496_spikes=171_deliveries=1240_cells=1411_plasticity=35472
config_hash=c1-a57975f13b73a599 seed=15755469790931547198 condition=dense-local accuracy=0.562500 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=211125.333333 note=wall_secs=0.0028_peak_rss=3457024_spikes=236_deliveries=7720_cells=7890_plasticity=102912
config_hash=c1-a57975f13b73a599 seed=15755469790931547198 condition=gradient-reference accuracy=0.500000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9233280.000000 note=wall_secs=0.0220_peak_rss=2654208_spikes=0_deliveries=7680_cells=491520_plasticity=4117440
config_hash=c1-a57975f13b73a599 seed=15755469790931547198 condition=eligibility-reference accuracy=0.937500 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=730112.000000 note=wall_secs=0.0030_peak_rss=2588672_spikes=0_deliveries=7680_cells=491520_plasticity=185280
config_hash=c1-a57975f13b73a599 seed=8709160710835925077 condition=local-assembly accuracy=0.312500 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=122633.600000 note=wall_secs=0.0031_peak_rss=3162112_spikes=173_deliveries=1241_cells=1413_plasticity=35496
config_hash=c1-a57975f13b73a599 seed=8709160710835925077 condition=dense-local accuracy=0.562500 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=211146.666667 note=wall_secs=0.0027_peak_rss=3424256_spikes=246_deliveries=7720_cells=7892_plasticity=102912
config_hash=c1-a57975f13b73a599 seed=8709160710835925077 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=4616640.000000 note=wall_secs=0.0230_peak_rss=2637824_spikes=0_deliveries=7680_cells=491520_plasticity=4117440
config_hash=c1-a57975f13b73a599 seed=8709160710835925077 condition=eligibility-reference accuracy=0.937500 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=730112.000000 note=wall_secs=0.0027_peak_rss=2588672_spikes=0_deliveries=7680_cells=491520_plasticity=185280
config_hash=c1-a57975f13b73a599 seed=1663413756060003432 condition=local-assembly accuracy=0.750000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=51449.333333 note=wall_secs=0.0024_peak_rss=3244032_spikes=218_deliveries=1305_cells=1472_plasticity=35592
config_hash=c1-a57975f13b73a599 seed=1663413756060003432 condition=dense-local accuracy=0.375000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=316640.000000 note=wall_secs=0.0027_peak_rss=3473408_spikes=221_deliveries=7720_cells=7887_plasticity=102912
config_hash=c1-a57975f13b73a599 seed=1663413756060003432 condition=gradient-reference accuracy=0.500000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9233280.000000 note=wall_secs=0.0220_peak_rss=2654208_spikes=0_deliveries=7680_cells=491520_plasticity=4117440
config_hash=c1-a57975f13b73a599 seed=1663413756060003432 condition=eligibility-reference accuracy=0.937500 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=730112.000000 note=wall_secs=0.0025_peak_rss=2572288_spikes=0_deliveries=7680_cells=491520_plasticity=185280
```
