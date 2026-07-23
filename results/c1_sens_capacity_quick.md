# C1 / Gate G2 results note

**Config hash:** `c1-e519403aff33b384`

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
Config { experiment: "c1-sens-capacity", master_seed: 213073327554561, n_seeds: 5, sequence_len: 8, max_lag: 1, n_hidden: 128, k_wta: 2, p_sparse: 0.3, init_w: 0.15, eta: 0.2, lambda: 0.002, tau_e: 40.0, n_train: 48, n_test: 24, bptt_epochs: 40, bptt_lr: 0.02, g2_min_gap_closed: 0.5, g2_min_accuracy: 0.65, g2_confidence_z: 1.96, g2_min_positive_control: 0.9, g2_min_reference_gap: 0.15, activity_sparsity_min: 0.005, activity_sparsity_max: 0.03, scientific_n_seeds: 20, power_sigma_prior: 0.15, power_effect_size: 0.1, use_surrogate_lif_reference: true, surrogate_beta: 5.0, matched_budget_repeat: false, quick: true }
```

Power analysis: required scientific n_seeds ≥ 20 (preregistered σ=0.150, effect=0.100; formula n=⌈(1.96+0.8416)²σ²/δ²⌉).

## Per-seed accuracies

| seed | local-assembly | dense-local | gradient reference | eligibility reference | activity_sparsity (local) | activity_sparsity (dense) | dense_matched |
|---|---:|---:|---:|---:|---:|---:|---:|
| 11400783787908037652 | 0.6667 | 0.4167 | 0.9167 | 0.9583 | 0.0156 | 0.0156 | — |
| 4354473659840395307 | 0.5000 | 0.4167 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | — |
| 15755470452356510782 | 0.5000 | 0.4167 | 1.0000 | 0.9583 | 0.0156 | 0.0156 | — |
| 8709160341468737621 | 0.5000 | 0.4167 | 1.0000 | 0.9583 | 0.0156 | 0.0156 | — |
| 1663413025915563112 | 0.6667 | 0.4167 | 0.9167 | 0.9583 | 0.0156 | 0.0156 | — |

## Summary (paired normalized-gap analysis)

- mean ± var local-assembly: 0.5667 ± 0.008333
- mean ± var dense-local:    0.4167 ± 0.000000
- mean ± var gradient reference: 0.9667 ± 0.002083
- mean ± var eligibility reference: 0.9667 ± 0.000347
- mean normalized gap closed: 0.2857 (variance 0.038265)
- lower confidence bound (z=1.960, n=5): 0.1142
- mean |local − dense| (descriptive): 0.1500

## Pilot limitation

This run uses a quick schedule or fewer seeds than the power-analysis requirement. It validates the harness only and is not evidence for passing or failing G2.

## Positive / sanity control

Mean local-pipeline accuracy on a trivially separable spatial feature-presence task: **1.0000** (threshold 0.900).

## Activity sparsity

Mean local-assembly activity_sparsity: **0.0156** (valid band [0.0050, 0.0300]; nominal k/N=0.0156).

## Parameter / compute budgets

| condition | n_cells | n_params | wall_secs | peak_rss_bytes | work_per_accuracy | spikes | deliveries | cell_updates | plasticity |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| local-assembly | 132 | 5014 | 0.0555 | 3686400 | 526166.9843 | 457 | 7002 | 7381 | 335938 |
| dense-local | 132 | 16768 | 0.0993 | 4521984 | 3600612.0858 | 562 | 37008 | 37405 | 1425280 |
| gradient-reference | 130 | 16769 | 7.8687 | 3129344 | 37285002.8282 | 0 | 15360 | 1966080 | 32196480 |
| eligibility-reference | 130 | 385 | 0.1303 | 2867200 | 2838928.7545 | 0 | 15360 | 1966080 | 739200 |

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
config_hash=c1-e519403aff33b384 seed=11400783787908037652 condition=local-assembly accuracy=0.666667 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=526166.984319 note=wall_secs=0.0555_peak_rss=3686400_spikes=457_deliveries=7002_cells=7381_plasticity=335938
config_hash=c1-e519403aff33b384 seed=11400783787908037652 condition=dense-local accuracy=0.416667 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=3600612.085845 note=wall_secs=0.0993_peak_rss=4521984_spikes=562_deliveries=37008_cells=37405_plasticity=1425280
config_hash=c1-e519403aff33b384 seed=11400783787908037652 condition=gradient-reference accuracy=0.916667 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=37285002.828233 note=wall_secs=7.8687_peak_rss=3129344_spikes=0_deliveries=15360_cells=1966080_plasticity=32196480
config_hash=c1-e519403aff33b384 seed=11400783787908037652 condition=eligibility-reference accuracy=0.958333 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=2838928.754509 note=wall_secs=0.1303_peak_rss=2867200_spikes=0_deliveries=15360_cells=1966080_plasticity=739200
config_hash=c1-e519403aff33b384 seed=4354473659840395307 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=760948.000000 note=wall_secs=0.0443_peak_rss=3702784_spikes=462_deliveries=6985_cells=7370_plasticity=365657
config_hash=c1-e519403aff33b384 seed=4354473659840395307 condition=dense-local accuracy=0.416667 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=3600612.085845 note=wall_secs=0.0995_peak_rss=4521984_spikes=562_deliveries=37008_cells=37405_plasticity=1425280
config_hash=c1-e519403aff33b384 seed=4354473659840395307 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=34177920.000000 note=wall_secs=7.8619_peak_rss=3031040_spikes=0_deliveries=15360_cells=1966080_plasticity=32196480
config_hash=c1-e519403aff33b384 seed=4354473659840395307 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=2720640.000000 note=wall_secs=0.1394_peak_rss=2867200_spikes=0_deliveries=15360_cells=1966080_plasticity=739200
config_hash=c1-e519403aff33b384 seed=15755470452356510782 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=793120.000000 note=wall_secs=0.0453_peak_rss=3702784_spikes=550_deliveries=7089_cells=7477_plasticity=381444
config_hash=c1-e519403aff33b384 seed=15755470452356510782 condition=dense-local accuracy=0.416667 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=3600612.085845 note=wall_secs=0.0986_peak_rss=4538368_spikes=562_deliveries=37008_cells=37405_plasticity=1425280
config_hash=c1-e519403aff33b384 seed=15755470452356510782 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=34177920.000000 note=wall_secs=7.8588_peak_rss=3047424_spikes=0_deliveries=15360_cells=1966080_plasticity=32196480
config_hash=c1-e519403aff33b384 seed=15755470452356510782 condition=eligibility-reference accuracy=0.958333 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=2838928.754509 note=wall_secs=0.1298_peak_rss=2867200_spikes=0_deliveries=15360_cells=1966080_plasticity=739200
config_hash=c1-e519403aff33b384 seed=8709160341468737621 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=801178.000000 note=wall_secs=0.0454_peak_rss=3719168_spikes=527_deliveries=7067_cells=7456_plasticity=385539
config_hash=c1-e519403aff33b384 seed=8709160341468737621 condition=dense-local accuracy=0.416667 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=3600612.085845 note=wall_secs=0.0984_peak_rss=4571136_spikes=562_deliveries=37008_cells=37405_plasticity=1425280
config_hash=c1-e519403aff33b384 seed=8709160341468737621 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=34177920.000000 note=wall_secs=7.8677_peak_rss=3129344_spikes=0_deliveries=15360_cells=1966080_plasticity=32196480
config_hash=c1-e519403aff33b384 seed=8709160341468737621 condition=eligibility-reference accuracy=0.958333 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=2838928.754509 note=wall_secs=0.1297_peak_rss=2867200_spikes=0_deliveries=15360_cells=1966080_plasticity=739200
config_hash=c1-e519403aff33b384 seed=1663413025915563112 condition=local-assembly accuracy=0.666667 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=586048.482534 note=wall_secs=0.0448_peak_rss=3702784_spikes=533_deliveries=7052_cells=7439_plasticity=375675
config_hash=c1-e519403aff33b384 seed=1663413025915563112 condition=dense-local accuracy=0.416667 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=3600612.085845 note=wall_secs=0.0992_peak_rss=4554752_spikes=562_deliveries=37008_cells=37405_plasticity=1425280
config_hash=c1-e519403aff33b384 seed=1663413025915563112 condition=gradient-reference accuracy=0.916667 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=37285002.828233 note=wall_secs=7.8689_peak_rss=3227648_spikes=0_deliveries=15360_cells=1966080_plasticity=32196480
config_hash=c1-e519403aff33b384 seed=1663413025915563112 condition=eligibility-reference accuracy=0.958333 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=2838928.754509 note=wall_secs=0.1294_peak_rss=2883584_spikes=0_deliveries=15360_cells=1966080_plasticity=739200
```
