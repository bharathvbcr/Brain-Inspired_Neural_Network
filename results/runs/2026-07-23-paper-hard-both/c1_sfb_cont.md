# C1 / Gate G2 results note

**Config hash:** `c1-840f820b7c07b512`

**Scientific protocol version:** `24`

**claim_axis:** Novel-CS
**object_under_test:** Continuous/normalized structured B under muted-θ/k-WTA C1
**may_claim:** Whether continuous B∝(w1−w0) beats sign-truncated v15 on gap LCB
**must_not_claim:** Hypersearch over B constructions; remassage v15; biology

**Continuous structured B protocol:** `24` — same live RFB path as v15, but hidden `B_i` is L2-normalized `(w→r1 − w→r0)` (not sign-truncated); single-pass; **positive control stays on broadcast ±1**; does **not** remassage v15 hash `c1-493ddd56f8714fb6` or reopen protocol-v2 `c1-118207fbc3eaba53`.

**Verdict (Gate G2):** **FAIL**

PASS = lower confidence bound on normalized gradient gap closed > 0.500 and mean local accuracy >= 0.650.
FAIL = a full run missed at least one preregistered threshold.
PILOT = quick schedule or fewer seeds than the power-analysis requirement; not a scientific G2 decision.
INVALID_HARNESS = positive_control_mean < 0.900 or mean activity sparsity outside [0.0050, 0.0300]; prohibits PASS/FAIL and U-NEG language.

## Conditions

| Label | Meaning |
|---|---|
| `local-assembly` | Three-factor rule + sparse assembly + k-WTA + dual readouts + **`ReinforceFeedback` × `reinforce_term`** (opt-in; not broadcast ±1) |
| `dense-local` | Same three-factor + k-WTA budget on dense all-to-all, **no** assembly; same `ReinforceFeedback` neuromodulator |
| `dense-matched` | Dense-local with nnz matched to local-assembly (parameter-matched; measured compute disclosed below) |
| `gradient-reference` | Same-architecture surrogate-LIF BPTT (primary); tanh RNN optional/secondary |
| `eligibility-reference` | E-prop-compatible eligibility local reference (rate-model approximation; feedforward-only) |

Plasticity uses directional REINFORCE × frozen per-neuron feedback (`ReinforceFeedback`) by design; broadcast ±1 remains the default C1 path. Gap-closed is clamped to `[0, 1]` and seeds with `(reference − dense) < 0.150` contribute `closed = 0`.

## Config

```
Config { experiment: "c1-sfb-cont", master_seed: 212618061021185, n_seeds: 20, sequence_len: 8, max_lag: 1, n_hidden: 128, k_wta: 2, p_sparse: 0.35, init_w: 0.15, eta: 0.35, lambda: 0.002, tau_e: 40.0, n_train: 80, n_test: 40, bptt_epochs: 80, bptt_lr: 0.05, g2_min_gap_closed: 0.5, g2_min_accuracy: 0.65, g2_confidence_z: 1.96, g2_min_positive_control: 0.9, g2_min_reference_gap: 0.15, activity_sparsity_min: 0.005, activity_sparsity_max: 0.03, scientific_n_seeds: 20, power_sigma_prior: 0.15, power_effect_size: 0.1, use_surrogate_lif_reference: true, surrogate_beta: 5.0, matched_budget_repeat: true, quick: false }
```

Power analysis: required scientific n_seeds ≥ 20 (preregistered σ=0.150, effect=0.100; formula n=⌈(1.96+0.8416)²σ²/δ²⌉).

## Per-seed accuracies

| seed | local-assembly | dense-local | gradient reference | eligibility reference | activity_sparsity (local) | activity_sparsity (dense) | dense_matched |
|---|---:|---:|---:|---:|---:|---:|---:|
| 11400784225994701844 | 1.0000 | 0.4250 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.4750 |
| 4354472946875824171 | 0.5000 | 0.5750 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 15755469790931547198 | 0.5000 | 0.4500 | 0.7250 | 1.0000 | 0.0156 | 0.0156 | 0.5250 |
| 8709160710835925077 | 0.7250 | 0.5500 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.4500 |
| 1663413756060003432 | 0.7250 | 0.5500 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.3750 |
| 13063846550650677375 | 0.7250 | 0.5500 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.4750 |
| 6018099320996848786 | 0.7250 | 0.5750 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.5250 |
| 17418529916564267177 | 0.5000 | 0.5000 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.4750 |
| 10372782686910438588 | 0.5000 | 0.5000 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 3326471682669467859 | 0.7250 | 0.4500 | 0.7250 | 1.0000 | 0.0156 | 0.0156 | 0.3250 |
| 14727610363725173990 | 0.6000 | 0.4000 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.4500 |
| 7681300184117924093 | 0.7250 | 0.5000 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.3750 |
| 635551854952467728 | 0.7250 | 0.5750 | 0.5000 | 1.0000 | 0.0156 | 0.0156 | 0.6250 |
| 12035985749054769447 | 0.5000 | 0.5750 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 4990235495743964474 | 0.5000 | 0.4250 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 16390669389846266193 | 0.7000 | 0.5000 | 0.5000 | 1.0000 | 0.0156 | 0.0156 | 0.4750 |
| 9344921060680809828 | 0.5000 | 0.5000 | 0.7250 | 1.0000 | 0.0156 | 0.0156 | 0.6500 |
| 2298610881073559931 | 0.5000 | 0.5500 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.5750 |
| 13699608824640910734 | 1.0000 | 0.4500 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.5750 |
| 6653297820399940005 | 0.5000 | 0.4500 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.5500 |

## Summary (paired normalized-gap analysis)

- mean ± var local-assembly: 0.6437 ± 0.025847
- mean ± var dense-local:    0.5025 ± 0.003349
- mean ± var gradient reference: 0.8938 ± 0.027163
- mean ± var eligibility reference: 1.0000 ± 0.000000
- mean normalized gap closed: 0.2922 (variance 0.123846)
- lower confidence bound (z=1.960, n=20): 0.1380
- mean |local − dense| (descriptive): 0.1613
- descriptive chance-normalized gap mean / LCB: 0.2786 / 0.1163 (var 0.137074; **not a gate**)
- seed local min / max / frac≥0.65: 0.5000 / 1.0000 / 0.50

## U-NEG

Negative result: local-assembly did **not** clear the preregistered normalized-gap confidence and absolute-accuracy gates. Program stops at G2; do not schedule P3+.

## Positive / sanity control

Mean local-pipeline accuracy on a trivially separable spatial feature-presence task: **0.9488** (threshold 0.900).

## Activity sparsity

Mean local-assembly activity_sparsity: **0.0156** (valid band [0.0050, 0.0300]; nominal k/N=0.0156).

## Parameter / compute budgets

| condition | n_cells | n_params | wall_secs | peak_rss_bytes | work_per_accuracy | spikes | deliveries | cell_updates | plasticity |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| local-assembly | 132 | 5777 | 0.0053 | 3637248 | 490004.0000 | 807 | 13206 | 13831 | 462160 |
| dense-local | 132 | 16768 | 0.0100 | 4538368 | 3450261.0797 | 924 | 61680 | 62317 | 1341440 |
| gradient-reference | 130 | 16769 | 0.6748 | 2916352 | 113926400.0000 | 0 | 51200 | 6553600 | 107321600 |
| eligibility-reference | 130 | 385 | 0.0151 | 2637824 | 9068800.0000 | 0 | 51200 | 6553600 | 2464000 |
| dense-matched | 132 | 5777 | 0.0059 | 4079616 | 1149490.5407 | 901 | 41155 | 41792 | 462160 |

Matched-budget dense mean accuracy: **0.4950** (n=20; primary G2 gap still uses unmatched dense-local).

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
config_hash=c1-840f820b7c07b512 seed=11400784225994701844 condition=local-assembly accuracy=1.000000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=490004.000000 note=wall_secs=0.0053_peak_rss=3637248_spikes=807_deliveries=13206_cells=13831_plasticity=462160
config_hash=c1-840f820b7c07b512 seed=11400784225994701844 condition=dense-local accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=3450261.079693 note=wall_secs=0.0100_peak_rss=4538368_spikes=924_deliveries=61680_cells=62317_plasticity=1341440
config_hash=c1-840f820b7c07b512 seed=11400784225994701844 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6748_peak_rss=2916352_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-840f820b7c07b512 seed=11400784225994701844 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0151_peak_rss=2637824_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-840f820b7c07b512 seed=11400784225994701844 condition=dense-matched accuracy=0.475000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1149490.540740 note=wall_secs=0.0059_peak_rss=4079616_spikes=901_deliveries=41155_cells=41792_plasticity=462160
config_hash=c1-840f820b7c07b512 seed=4354472946875824171 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=979288.000000 note=wall_secs=0.0047_peak_rss=3604480_spikes=700_deliveries=13155_cells=13789_plasticity=462000
config_hash=c1-840f820b7c07b512 seed=4354472946875824171 condition=dense-local accuracy=0.575000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2550140.922435 note=wall_secs=0.0095_peak_rss=4374528_spikes=897_deliveries=61680_cells=62314_plasticity=1341440
config_hash=c1-840f820b7c07b512 seed=4354472946875824171 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6670_peak_rss=2899968_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-840f820b7c07b512 seed=4354472946875824171 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0157_peak_rss=2654208_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-840f820b7c07b512 seed=4354472946875824171 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1091696.000000 note=wall_secs=0.0059_peak_rss=3932160_spikes=892_deliveries=41161_cells=41795_plasticity=462000
config_hash=c1-840f820b7c07b512 seed=15755469790931547198 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=979168.000000 note=wall_secs=0.0051_peak_rss=3768320_spikes=711_deliveries=13160_cells=13793_plasticity=461920
config_hash=c1-840f820b7c07b512 seed=15755469790931547198 condition=dense-local accuracy=0.450000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=3258522.308544 note=wall_secs=0.0103_peak_rss=4521984_spikes=903_deliveries=61680_cells=62312_plasticity=1341440
config_hash=c1-840f820b7c07b512 seed=15755469790931547198 condition=gradient-reference accuracy=0.725000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=157139856.901371 note=wall_secs=0.6778_peak_rss=2916352_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-840f820b7c07b512 seed=15755469790931547198 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0155_peak_rss=2605056_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-840f820b7c07b512 seed=15755469790931547198 condition=dense-matched accuracy=0.525000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1039219.094813 note=wall_secs=0.0061_peak_rss=4145152_spikes=893_deliveries=41072_cells=41705_plasticity=461920
config_hash=c1-840f820b7c07b512 seed=8709160710835925077 condition=local-assembly accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=677799.977710 note=wall_secs=0.0054_peak_rss=3768320_spikes=767_deliveries=13359_cells=13999_plasticity=463280
config_hash=c1-840f820b7c07b512 seed=8709160710835925077 condition=dense-local accuracy=0.550000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2666103.578577 note=wall_secs=0.0101_peak_rss=4669440_spikes=916_deliveries=61680_cells=62321_plasticity=1341440
config_hash=c1-840f820b7c07b512 seed=8709160710835925077 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6758_peak_rss=2899968_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-840f820b7c07b512 seed=8709160710835925077 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0153_peak_rss=2605056_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-840f820b7c07b512 seed=8709160710835925077 condition=dense-matched accuracy=0.450000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1215635.587759 note=wall_secs=0.0058_peak_rss=4128768_spikes=906_deliveries=41105_cells=41745_plasticity=463280
config_hash=c1-840f820b7c07b512 seed=1663413756060003432 condition=local-assembly accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=677904.805293 note=wall_secs=0.0049_peak_rss=3784704_spikes=817_deliveries=13338_cells=13966_plasticity=463360
config_hash=c1-840f820b7c07b512 seed=1663413756060003432 condition=dense-local accuracy=0.550000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2666072.669487 note=wall_secs=0.0096_peak_rss=4489216_spikes=907_deliveries=61680_cells=62313_plasticity=1341440
config_hash=c1-840f820b7c07b512 seed=1663413756060003432 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6703_peak_rss=2965504_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-840f820b7c07b512 seed=1663413756060003432 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0156_peak_rss=2605056_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-840f820b7c07b512 seed=1663413756060003432 condition=dense-matched accuracy=0.375000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1459184.000000 note=wall_secs=0.0058_peak_rss=4079616_spikes=893_deliveries=41154_cells=41787_plasticity=463360
config_hash=c1-840f820b7c07b512 seed=13063846550650677375 condition=local-assembly accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=675848.253637 note=wall_secs=0.0050_peak_rss=3719168_spikes=810_deliveries=13313_cells=13947_plasticity=461920
config_hash=c1-840f820b7c07b512 seed=13063846550650677375 condition=dense-local accuracy=0.550000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2666118.124032 note=wall_secs=0.0100_peak_rss=4538368_spikes=920_deliveries=61680_cells=62325_plasticity=1341440
config_hash=c1-840f820b7c07b512 seed=13063846550650677375 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6683_peak_rss=2916352_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-840f820b7c07b512 seed=13063846550650677375 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0165_peak_rss=2621440_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-840f820b7c07b512 seed=13063846550650677375 condition=dense-matched accuracy=0.475000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1148625.277571 note=wall_secs=0.0057_peak_rss=3915776_spikes=917_deliveries=41057_cells=41703_plasticity=461920
config_hash=c1-840f820b7c07b512 seed=6018099320996848786 condition=local-assembly accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=676342.046724 note=wall_secs=0.0054_peak_rss=3751936_spikes=873_deliveries=13502_cells=14133_plasticity=461840
config_hash=c1-840f820b7c07b512 seed=6018099320996848786 condition=dense-local accuracy=0.575000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2550187.878958 note=wall_secs=0.0102_peak_rss=4734976_spikes=914_deliveries=61680_cells=62324_plasticity=1341440
config_hash=c1-840f820b7c07b512 seed=6018099320996848786 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6667_peak_rss=2834432_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-840f820b7c07b512 seed=6018099320996848786 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0179_peak_rss=2637824_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-840f820b7c07b512 seed=6018099320996848786 condition=dense-matched accuracy=0.525000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1039638.142451 note=wall_secs=0.0064_peak_rss=3899392_spikes=901_deliveries=41212_cells=41857_plasticity=461840
config_hash=c1-840f820b7c07b512 seed=17418529916564267177 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=981672.000000 note=wall_secs=0.0047_peak_rss=3719168_spikes=849_deliveries=13473_cells=14114_plasticity=462400
config_hash=c1-840f820b7c07b512 seed=17418529916564267177 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2932796.000000 note=wall_secs=0.0097_peak_rss=4505600_spikes=950_deliveries=61680_cells=62328_plasticity=1341440
config_hash=c1-840f820b7c07b512 seed=17418529916564267177 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6764_peak_rss=2965504_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-840f820b7c07b512 seed=17418529916564267177 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0159_peak_rss=2654208_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-840f820b7c07b512 seed=17418529916564267177 condition=dense-matched accuracy=0.475000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1149905.277587 note=wall_secs=0.0067_peak_rss=3981312_spikes=924_deliveries=41117_cells=41764_plasticity=462400
config_hash=c1-840f820b7c07b512 seed=10372782686910438588 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=981532.000000 note=wall_secs=0.0050_peak_rss=3555328_spikes=796_deliveries=13428_cells=14062_plasticity=462480
config_hash=c1-840f820b7c07b512 seed=10372782686910438588 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2932668.000000 note=wall_secs=0.0098_peak_rss=4489216_spikes=899_deliveries=61680_cells=62315_plasticity=1341440
config_hash=c1-840f820b7c07b512 seed=10372782686910438588 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6708_peak_rss=2998272_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-840f820b7c07b512 seed=10372782686910438588 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0154_peak_rss=2605056_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-840f820b7c07b512 seed=10372782686910438588 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1092272.000000 note=wall_secs=0.0056_peak_rss=4145152_spikes=902_deliveries=41059_cells=41695_plasticity=462480
config_hash=c1-840f820b7c07b512 seed=3326471682669467859 condition=local-assembly accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=676540.667407 note=wall_secs=0.0051_peak_rss=3604480_spikes=780_deliveries=13416_cells=14056_plasticity=462240
config_hash=c1-840f820b7c07b512 seed=3326471682669467859 condition=dense-local accuracy=0.450000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=3258504.530765 note=wall_secs=0.0098_peak_rss=4472832_spikes=884_deliveries=61680_cells=62323_plasticity=1341440
config_hash=c1-840f820b7c07b512 seed=3326471682669467859 condition=gradient-reference accuracy=0.725000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=157139856.901371 note=wall_secs=0.6684_peak_rss=2916352_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-840f820b7c07b512 seed=3326471682669467859 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0150_peak_rss=2605056_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-840f820b7c07b512 seed=3326471682669467859 condition=dense-matched accuracy=0.325000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1679889.292387 note=wall_secs=0.0058_peak_rss=4128768_spikes=895_deliveries=41093_cells=41736_plasticity=462240
config_hash=c1-840f820b7c07b512 seed=14727610363725173990 condition=local-assembly accuracy=0.600000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=817713.300840 note=wall_secs=0.0048_peak_rss=3571712_spikes=841_deliveries=13332_cells=13975_plasticity=462480
config_hash=c1-840f820b7c07b512 seed=14727610363725173990 condition=dense-local accuracy=0.400000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=3665984.945373 note=wall_secs=0.0095_peak_rss=4341760_spikes=950_deliveries=61680_cells=62324_plasticity=1341440
config_hash=c1-840f820b7c07b512 seed=14727610363725173990 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6719_peak_rss=2916352_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-840f820b7c07b512 seed=14727610363725173990 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0151_peak_rss=2637824_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-840f820b7c07b512 seed=14727610363725173990 condition=dense-matched accuracy=0.450000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1212800.032128 note=wall_secs=0.0060_peak_rss=4096000_spikes=924_deliveries=40856_cells=41500_plasticity=462480
config_hash=c1-840f820b7c07b512 seed=7681300184117924093 condition=local-assembly accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=673961.357147 note=wall_secs=0.0049_peak_rss=3735552_spikes=791_deliveries=13157_cells=13794_plasticity=460880
config_hash=c1-840f820b7c07b512 seed=7681300184117924093 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2932720.000000 note=wall_secs=0.0095_peak_rss=4423680_spikes=918_deliveries=61680_cells=62322_plasticity=1341440
config_hash=c1-840f820b7c07b512 seed=7681300184117924093 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.7071_peak_rss=2998272_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-840f820b7c07b512 seed=7681300184117924093 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0179_peak_rss=2637824_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-840f820b7c07b512 seed=7681300184117924093 condition=dense-matched accuracy=0.375000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1452392.000000 note=wall_secs=0.0056_peak_rss=4112384_spikes=918_deliveries=41104_cells=41745_plasticity=460880
config_hash=c1-840f820b7c07b512 seed=635551854952467728 condition=local-assembly accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=675713.081227 note=wall_secs=0.0049_peak_rss=3588096_spikes=794_deliveries=13390_cells=14028_plasticity=461680
config_hash=c1-840f820b7c07b512 seed=635551854952467728 condition=dense-local accuracy=0.575000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2550151.357218 note=wall_secs=0.0096_peak_rss=4538368_spikes=900_deliveries=61680_cells=62317_plasticity=1341440
config_hash=c1-840f820b7c07b512 seed=635551854952467728 condition=gradient-reference accuracy=0.500000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=227852800.000000 note=wall_secs=0.6745_peak_rss=2916352_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-840f820b7c07b512 seed=635551854952467728 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0401_peak_rss=2654208_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-840f820b7c07b512 seed=635551854952467728 condition=dense-matched accuracy=0.625000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=872185.600000 note=wall_secs=0.0060_peak_rss=4063232_spikes=898_deliveries=40950_cells=41588_plasticity=461680
config_hash=c1-840f820b7c07b512 seed=12035985749054769447 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=984482.000000 note=wall_secs=0.0050_peak_rss=3604480_spikes=837_deliveries=13542_cells=14182_plasticity=463680
config_hash=c1-840f820b7c07b512 seed=12035985749054769447 condition=dense-local accuracy=0.575000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2550137.444174 note=wall_secs=0.0099_peak_rss=4440064_spikes=892_deliveries=61680_cells=62317_plasticity=1341440
config_hash=c1-840f820b7c07b512 seed=12035985749054769447 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6663_peak_rss=2916352_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-840f820b7c07b512 seed=12035985749054769447 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0155_peak_rss=2605056_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-840f820b7c07b512 seed=12035985749054769447 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1094654.000000 note=wall_secs=0.0059_peak_rss=3948544_spikes=887_deliveries=41062_cells=41698_plasticity=463680
config_hash=c1-840f820b7c07b512 seed=4990235495743964474 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=981924.000000 note=wall_secs=0.0050_peak_rss=3604480_spikes=786_deliveries=13407_cells=14049_plasticity=462720
config_hash=c1-840f820b7c07b512 seed=4990235495743964474 condition=dense-local accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=3450218.726753 note=wall_secs=0.0097_peak_rss=4538368_spikes=901_deliveries=61680_cells=62322_plasticity=1341440
config_hash=c1-840f820b7c07b512 seed=4990235495743964474 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6689_peak_rss=2998272_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-840f820b7c07b512 seed=4990235495743964474 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0401_peak_rss=2637824_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-840f820b7c07b512 seed=4990235495743964474 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1093022.000000 note=wall_secs=0.0059_peak_rss=4063232_spikes=899_deliveries=41125_cells=41767_plasticity=462720
config_hash=c1-840f820b7c07b512 seed=16390669389846266193 condition=local-assembly accuracy=0.700000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=701082.869082 note=wall_secs=0.0049_peak_rss=3768320_spikes=769_deliveries=13314_cells=13955_plasticity=462720
config_hash=c1-840f820b7c07b512 seed=16390669389846266193 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2932744.000000 note=wall_secs=0.0097_peak_rss=4358144_spikes=930_deliveries=61680_cells=62322_plasticity=1341440
config_hash=c1-840f820b7c07b512 seed=16390669389846266193 condition=gradient-reference accuracy=0.500000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=227852800.000000 note=wall_secs=0.6685_peak_rss=2883584_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-840f820b7c07b512 seed=16390669389846266193 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0151_peak_rss=2637824_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-840f820b7c07b512 seed=16390669389846266193 condition=dense-matched accuracy=0.475000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1150604.224964 note=wall_secs=0.0055_peak_rss=3915776_spikes=913_deliveries=41131_cells=41773_plasticity=462720
config_hash=c1-840f820b7c07b512 seed=9344921060680809828 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=979736.000000 note=wall_secs=0.0049_peak_rss=3768320_spikes=752_deliveries=13280_cells=13916_plasticity=461920
config_hash=c1-840f820b7c07b512 seed=9344921060680809828 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2932720.000000 note=wall_secs=0.0094_peak_rss=4571136_spikes=924_deliveries=61680_cells=62316_plasticity=1341440
config_hash=c1-840f820b7c07b512 seed=9344921060680809828 condition=gradient-reference accuracy=0.725000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=157139856.901371 note=wall_secs=0.6881_peak_rss=2998272_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-840f820b7c07b512 seed=9344921060680809828 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0160_peak_rss=2637824_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-840f820b7c07b512 seed=9344921060680809828 condition=dense-matched accuracy=0.650000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=839560.030795 note=wall_secs=0.0059_peak_rss=3948544_spikes=915_deliveries=41122_cells=41757_plasticity=461920
config_hash=c1-840f820b7c07b512 seed=2298610881073559931 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=980220.000000 note=wall_secs=0.0060_peak_rss=3588096_spikes=795_deliveries=13218_cells=13857_plasticity=462240
config_hash=c1-840f820b7c07b512 seed=2298610881073559931 condition=dense-local accuracy=0.550000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2666078.124032 note=wall_secs=0.0107_peak_rss=4636672_spikes=902_deliveries=61680_cells=62321_plasticity=1341440
config_hash=c1-840f820b7c07b512 seed=2298610881073559931 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6770_peak_rss=2916352_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-840f820b7c07b512 seed=2298610881073559931 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0152_peak_rss=2654208_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-840f820b7c07b512 seed=2298610881073559931 condition=dense-matched accuracy=0.575000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=949450.454467 note=wall_secs=0.0065_peak_rss=3997696_spikes=901_deliveries=41076_cells=41717_plasticity=462240
config_hash=c1-840f820b7c07b512 seed=13699608824640910734 condition=local-assembly accuracy=1.000000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=490053.000000 note=wall_secs=0.0052_peak_rss=3571712_spikes=771_deliveries=13127_cells=13755_plasticity=462400
config_hash=c1-840f820b7c07b512 seed=13699608824640910734 condition=dense-local accuracy=0.450000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=3258546.752989 note=wall_secs=0.0097_peak_rss=4505600_spikes=908_deliveries=61680_cells=62318_plasticity=1341440
config_hash=c1-840f820b7c07b512 seed=13699608824640910734 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6806_peak_rss=2916352_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-840f820b7c07b512 seed=13699608824640910734 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0156_peak_rss=2637824_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-840f820b7c07b512 seed=13699608824640910734 condition=dense-matched accuracy=0.575000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=949794.802300 note=wall_secs=0.0059_peak_rss=4096000_spikes=898_deliveries=41098_cells=41736_plasticity=462400
config_hash=c1-840f820b7c07b512 seed=6653297820399940005 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=980752.000000 note=wall_secs=0.0057_peak_rss=3751936_spikes=734_deliveries=13257_cells=13905_plasticity=462480
config_hash=c1-840f820b7c07b512 seed=6653297820399940005 condition=dense-local accuracy=0.450000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=3258535.641877 note=wall_secs=0.0098_peak_rss=4489216_spikes=895_deliveries=61680_cells=62326_plasticity=1341440
config_hash=c1-840f820b7c07b512 seed=6653297820399940005 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6794_peak_rss=2916352_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-840f820b7c07b512 seed=6653297820399940005 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0170_peak_rss=2703360_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-840f820b7c07b512 seed=6653297820399940005 condition=dense-matched accuracy=0.550000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=993194.523928 note=wall_secs=0.0062_peak_rss=3915776_spikes=908_deliveries=41112_cells=41757_plasticity=462480
```
