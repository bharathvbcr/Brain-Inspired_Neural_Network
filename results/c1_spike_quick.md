# C1 / Gate G2 results note

**Config hash:** `c1-d6b811cec7feed26`

**Scientific protocol version:** `6`

**Natural-hidden-spiking protocol:** `6` — finite hidden θ during integrate (no θ=∞ mute); applies trial-isolation membrane + STDP pairing resets; does **not** reopen protocol-v2 kill-gate hash `c1-118207fbc3eaba53` (canonical version `2`).

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
| `gradient-reference` | Same-architecture surrogate-LIF BPTT (primary); tanh RNN optional/secondary |
| `eligibility-reference` | E-prop-compatible eligibility local reference (rate-model approximation; feedforward-only) |

Plasticity uses hard ±1 reward by design (soft RPE deferred). Gap-closed is clamped to `[0, 1]` and seeds with `(reference − dense) < 0.150` contribute `closed = 0`.

## Config

```
Config { experiment: "c1-spike", master_seed: 212592291217409, n_seeds: 5, sequence_len: 8, max_lag: 1, n_hidden: 64, k_wta: 1, p_sparse: 0.35, init_w: 0.15, eta: 0.35, lambda: 0.002, tau_e: 40.0, n_train: 24, n_test: 16, bptt_epochs: 40, bptt_lr: 0.05, g2_min_gap_closed: 0.5, g2_min_accuracy: 0.65, g2_confidence_z: 1.96, g2_min_positive_control: 0.9, g2_min_reference_gap: 0.15, activity_sparsity_min: 0.005, activity_sparsity_max: 0.03, scientific_n_seeds: 20, power_sigma_prior: 0.15, power_effect_size: 0.1, use_surrogate_lif_reference: true, surrogate_beta: 5.0, matched_budget_repeat: false, quick: true }
```

Power analysis: required scientific n_seeds ≥ 20 (preregistered σ=0.150, effect=0.100; formula n=⌈(1.96+0.8416)²σ²/δ²⌉).

## Per-seed accuracies

| seed | local-assembly | dense-local | gradient reference | eligibility reference | activity_sparsity (local) | activity_sparsity (dense) | dense_matched |
|---|---:|---:|---:|---:|---:|---:|---:|
| 11400784268944374804 | 0.3125 | 0.5000 | 0.7500 | 0.9375 | 0.0000 | 0.0156 | — |
| 4354473041365104683 | 0.5000 | 0.5000 | 1.0000 | 0.9375 | 0.0156 | 0.0156 | — |
| 15755469971320173630 | 0.3125 | 0.5000 | 0.5000 | 0.9375 | 0.0156 | 0.0156 | — |
| 8709160959944028245 | 0.5000 | 0.5000 | 1.0000 | 0.8750 | 0.0156 | 0.0156 | — |
| 1663413506951900264 | 0.5000 | 0.5000 | 0.5000 | 0.9375 | 0.0156 | 0.0156 | — |

## Summary (paired normalized-gap analysis)

- mean ± var local-assembly: 0.4250 ± 0.010547
- mean ± var dense-local:    0.5000 ± 0.000000
- mean ± var gradient reference: 0.7500 ± 0.062500
- mean ± var eligibility reference: 0.9250 ± 0.000781
- mean normalized gap closed: 0.0000 (variance 0.000000)
- lower confidence bound (z=1.960, n=5): 0.0000
- mean |local − dense| (descriptive): 0.0750

## Invalid harness

Positive control and/or activity sparsity failed the preregistered validity gates. No scientific PASS/FAIL or U-NEG claim is permitted from this run.

## Positive / sanity control

Mean local-pipeline accuracy on a trivially separable spatial feature-presence task: **0.7833** (threshold 0.900).

## Activity sparsity

Mean local-assembly activity_sparsity: **0.0125** (valid band [0.0050, 0.0300]; nominal k/N=0.0156).

## Parameter / compute budgets

| condition | n_cells | n_params | wall_secs | peak_rss_bytes | work_per_accuracy | spikes | deliveries | cell_updates | plasticity |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| local-assembly | 68 | 1476 | 0.0034 | 3014656 | 195859.2000 | 172 | 919 | 1075 | 59040 |
| dense-local | 68 | 4288 | 0.0041 | 3260416 | 391944.0000 | 258 | 7720 | 7898 | 180096 |
| gradient-reference | 66 | 4289 | 0.0268 | 2588672 | 6155520.0000 | 0 | 7680 | 491520 | 4117440 |
| eligibility-reference | 66 | 193 | 0.0032 | 2506752 | 730112.0000 | 0 | 7680 | 491520 | 185280 |

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
config_hash=c1-d6b811cec7feed26 seed=11400784268944374804 condition=local-assembly accuracy=0.312500 activity_sparsity=0.000000 activity-sparsity=0.000000 work_per_accuracy=195859.200000 note=wall_secs=0.0034_peak_rss=3014656_spikes=172_deliveries=919_cells=1075_plasticity=59040
config_hash=c1-d6b811cec7feed26 seed=11400784268944374804 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=391944.000000 note=wall_secs=0.0041_peak_rss=3260416_spikes=258_deliveries=7720_cells=7898_plasticity=180096
config_hash=c1-d6b811cec7feed26 seed=11400784268944374804 condition=gradient-reference accuracy=0.750000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=6155520.000000 note=wall_secs=0.0268_peak_rss=2588672_spikes=0_deliveries=7680_cells=491520_plasticity=4117440
config_hash=c1-d6b811cec7feed26 seed=11400784268944374804 condition=eligibility-reference accuracy=0.937500 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=730112.000000 note=wall_secs=0.0032_peak_rss=2506752_spikes=0_deliveries=7680_cells=491520_plasticity=185280
config_hash=c1-d6b811cec7feed26 seed=4354473041365104683 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=103488.000000 note=wall_secs=0.0026_peak_rss=3047424_spikes=280_deliveries=1856_cells=2024_plasticity=47584
config_hash=c1-d6b811cec7feed26 seed=4354473041365104683 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=391944.000000 note=wall_secs=0.0033_peak_rss=3293184_spikes=258_deliveries=7720_cells=7898_plasticity=180096
config_hash=c1-d6b811cec7feed26 seed=4354473041365104683 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=4616640.000000 note=wall_secs=0.0246_peak_rss=2572288_spikes=0_deliveries=7680_cells=491520_plasticity=4117440
config_hash=c1-d6b811cec7feed26 seed=4354473041365104683 condition=eligibility-reference accuracy=0.937500 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=730112.000000 note=wall_secs=0.0029_peak_rss=2506752_spikes=0_deliveries=7680_cells=491520_plasticity=185280
config_hash=c1-d6b811cec7feed26 seed=15755469971320173630 condition=local-assembly accuracy=0.312500 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=199027.200000 note=wall_secs=0.0031_peak_rss=3031040_spikes=188_deliveries=1356_cells=1532_plasticity=59120
config_hash=c1-d6b811cec7feed26 seed=15755469971320173630 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=391944.000000 note=wall_secs=0.0033_peak_rss=3325952_spikes=258_deliveries=7720_cells=7898_plasticity=180096
config_hash=c1-d6b811cec7feed26 seed=15755469971320173630 condition=gradient-reference accuracy=0.500000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9233280.000000 note=wall_secs=0.0252_peak_rss=2621440_spikes=0_deliveries=7680_cells=491520_plasticity=4117440
config_hash=c1-d6b811cec7feed26 seed=15755469971320173630 condition=eligibility-reference accuracy=0.937500 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=730112.000000 note=wall_secs=0.0030_peak_rss=2523136_spikes=0_deliveries=7680_cells=491520_plasticity=185280
config_hash=c1-d6b811cec7feed26 seed=8709160959944028245 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=115778.000000 note=wall_secs=0.0031_peak_rss=3014656_spikes=227_deliveries=1420_cells=1593_plasticity=54649
config_hash=c1-d6b811cec7feed26 seed=8709160959944028245 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=391944.000000 note=wall_secs=0.0029_peak_rss=3293184_spikes=258_deliveries=7720_cells=7898_plasticity=180096
config_hash=c1-d6b811cec7feed26 seed=8709160959944028245 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=4616640.000000 note=wall_secs=0.0241_peak_rss=2588672_spikes=0_deliveries=7680_cells=491520_plasticity=4117440
config_hash=c1-d6b811cec7feed26 seed=8709160959944028245 condition=eligibility-reference accuracy=0.875000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=782262.857143 note=wall_secs=0.0037_peak_rss=2506752_spikes=0_deliveries=7680_cells=491520_plasticity=185280
config_hash=c1-d6b811cec7feed26 seed=1663413506951900264 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=128874.000000 note=wall_secs=0.0024_peak_rss=3014656_spikes=194_deliveries=1324_cells=1501_plasticity=61418
config_hash=c1-d6b811cec7feed26 seed=1663413506951900264 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=391944.000000 note=wall_secs=0.0032_peak_rss=3293184_spikes=258_deliveries=7720_cells=7898_plasticity=180096
config_hash=c1-d6b811cec7feed26 seed=1663413506951900264 condition=gradient-reference accuracy=0.500000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9233280.000000 note=wall_secs=0.0245_peak_rss=2605056_spikes=0_deliveries=7680_cells=491520_plasticity=4117440
config_hash=c1-d6b811cec7feed26 seed=1663413506951900264 condition=eligibility-reference accuracy=0.937500 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=730112.000000 note=wall_secs=0.0039_peak_rss=2523136_spikes=0_deliveries=7680_cells=491520_plasticity=185280
```
