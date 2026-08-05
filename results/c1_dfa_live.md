# C1 / Gate G2 results note

**Config hash:** `c1-4db53e645405fae0`

**Scientific protocol version:** `20`

**claim_axis:** Novel-CS
**object_under_test:** Graded DFA credit on live muted-θ / k-WTA C1
**may_claim:** Whether matched DFA PASS transfers under one honest live map
**must_not_claim:** Remassage matched `c1-dfa-*` / P4 spike-DFA; biology; impossibility

**Live graded-DFA transfer protocol:** `20` — same muted-θ / k-WTA / single-pass C1 substrate as v2/v13; main-condition plasticity uses graded readout error × fixed-random DFA feedback (`FixedRandomFeedback`) through three-factor eligibility; observe-only on incorrect; **positive control stays on broadcast ±1**; does **not** remassage matched `c1-dfa-c8c4fe0899908b84`, P4 spiking-DFA, or reopen protocol-v2 `c1-118207fbc3eaba53`.

**Verdict (Gate G2):** **FAIL**

PASS = lower confidence bound on normalized gradient gap closed > 0.500 and mean local accuracy >= 0.650.
FAIL = a full run missed at least one preregistered threshold.
PILOT = quick schedule or fewer seeds than the power-analysis requirement; not a scientific G2 decision.
INVALID_HARNESS = positive_control_mean < 0.900 or mean activity sparsity outside [0.0050, 0.0300]; prohibits PASS/FAIL and U-NEG language.

## Conditions

| Label | Meaning |
|---|---|
| `local-assembly` | Three-factor rule + sparse assembly wiring + k-WTA + dual readouts + two-sided ±1 reward |
| `dense-local` | Same three-factor rule + same k-winner budget on dense all-to-all connectivity, **no** assembly structure |
| `dense-matched` | Dense-local with nnz matched to local-assembly (parameter-matched; measured compute disclosed below) |
| `gradient-reference` | Same-architecture surrogate-LIF BPTT (primary); tanh RNN optional/secondary |
| `eligibility-reference` | E-prop-compatible eligibility local reference (rate-model approximation; feedforward-only) |

Plasticity uses hard ±1 reward by design (soft RPE deferred). Gap-closed is clamped to `[0, 1]` and seeds with `(reference − dense) < 0.150` contribute `closed = 0`.

## Config

```
Config { experiment: "c1-dfa-live", master_seed: 212618061021185, n_seeds: 20, sequence_len: 8, max_lag: 1, n_hidden: 128, k_wta: 2, p_sparse: 0.35, init_w: 0.15, eta: 0.35, lambda: 0.002, tau_e: 40.0, n_train: 80, n_test: 40, bptt_epochs: 80, bptt_lr: 0.05, g2_min_gap_closed: 0.5, g2_min_accuracy: 0.65, g2_confidence_z: 1.96, g2_min_positive_control: 0.9, g2_min_reference_gap: 0.15, activity_sparsity_min: 0.005, activity_sparsity_max: 0.03, scientific_n_seeds: 20, power_sigma_prior: 0.15, power_effect_size: 0.1, use_surrogate_lif_reference: true, surrogate_beta: 5.0, matched_budget_repeat: true, quick: false }
```

Power analysis: required scientific n_seeds ≥ 20 (preregistered σ=0.150, effect=0.100; formula n=⌈(1.96+0.8416)²σ²/δ²⌉).

## Per-seed accuracies

| seed | local-assembly | dense-local | gradient reference | eligibility reference | activity_sparsity (local) | activity_sparsity (dense) | dense_matched |
|---|---:|---:|---:|---:|---:|---:|---:|
| 11400784225994701844 | 1.0000 | 0.7250 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 4354472946875824171 | 1.0000 | 0.5000 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.7250 |
| 15755469790931547198 | 0.4250 | 0.5000 | 0.7250 | 1.0000 | 0.0156 | 0.0156 | 0.7250 |
| 8709160710835925077 | 1.0000 | 0.5000 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.7250 |
| 1663413756060003432 | 0.8250 | 0.5000 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.7250 |
| 13063846550650677375 | 0.5000 | 0.1000 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.7250 |
| 6018099320996848786 | 0.5000 | 0.5000 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 17418529916564267177 | 1.0000 | 0.7250 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.7250 |
| 10372782686910438588 | 0.9000 | 0.5000 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.7250 |
| 3326471682669467859 | 0.7250 | 0.7250 | 0.7250 | 1.0000 | 0.0156 | 0.0156 | 0.7250 |
| 14727610363725173990 | 0.9000 | 0.5000 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 7681300184117924093 | 0.5000 | 0.5000 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 635551854952467728 | 0.5000 | 0.5000 | 0.5000 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 12035985749054769447 | 1.0000 | 0.5000 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 4990235495743964474 | 1.0000 | 0.5000 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 16390669389846266193 | 0.5000 | 0.5000 | 0.5000 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 9344921060680809828 | 0.7250 | 0.7250 | 0.7250 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 2298610881073559931 | 0.5000 | 0.7250 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 13699608824640910734 | 0.5000 | 0.5000 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.7250 |
| 6653297820399940005 | 0.6500 | 0.5000 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |

## Summary (paired normalized-gap analysis)

- mean ± var local-assembly: 0.7325 ± 0.051257
- mean ± var dense-local:    0.5362 ± 0.020360
- mean ± var gradient reference: 0.8938 ± 0.027163
- mean ± var eligibility reference: 1.0000 ± 0.000000
- mean normalized gap closed: 0.4639 (variance 0.216139)
- lower confidence bound (z=1.960, n=20): 0.2601
- mean |local − dense| (descriptive): 0.2263
- descriptive chance-normalized gap mean / LCB: 0.5417 / 0.3321 (var 0.228679; **not a gate**)
- seed local min / max / frac≥0.65: 0.4250 / 1.0000 / 0.60

## U-NEG

Negative result: local-assembly did **not** clear the preregistered normalized-gap confidence and absolute-accuracy gates. Program stops at G2; do not schedule P3+.

## Positive / sanity control

Mean local-pipeline accuracy on a trivially separable spatial feature-presence task: **0.9500** (threshold 0.900).

## Activity sparsity

Mean local-assembly activity_sparsity: **0.0156** (valid band [0.0050, 0.0300]; nominal k/N=0.0156).

## Parameter / compute budgets

| condition | n_cells | n_params | wall_secs | peak_rss_bytes | work_per_accuracy | spikes | deliveries | cell_updates | plasticity |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| local-assembly | 132 | 5777 | 0.0052 | 3604480 | 490155.0000 | 843 | 13266 | 13886 | 462160 |
| dense-local | 132 | 16768 | 0.0094 | 4423680 | 2022281.3128 | 716 | 61680 | 62318 | 1341440 |
| gradient-reference | 130 | 16769 | 0.6617 | 2801664 | 113926400.0000 | 0 | 51200 | 6553600 | 107321600 |
| eligibility-reference | 130 | 385 | 0.0399 | 2637824 | 9068800.0000 | 0 | 51200 | 6553600 | 2464000 |
| dense-matched | 132 | 5777 | 0.0057 | 4063232 | 1091318.0000 | 773 | 41041 | 41685 | 462160 |

Matched-budget dense mean accuracy: **0.6012** (n=20; primary G2 gap still uses unmatched dense-local).

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
config_hash=c1-4db53e645405fae0 seed=11400784225994701844 condition=local-assembly accuracy=1.000000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=490155.000000 note=wall_secs=0.0052_peak_rss=3604480_spikes=843_deliveries=13266_cells=13886_plasticity=462160
config_hash=c1-4db53e645405fae0 seed=11400784225994701844 condition=dense-local accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2022281.312807 note=wall_secs=0.0094_peak_rss=4423680_spikes=716_deliveries=61680_cells=62318_plasticity=1341440
config_hash=c1-4db53e645405fae0 seed=11400784225994701844 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6617_peak_rss=2801664_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-4db53e645405fae0 seed=11400784225994701844 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0399_peak_rss=2637824_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-4db53e645405fae0 seed=11400784225994701844 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1091318.000000 note=wall_secs=0.0057_peak_rss=4063232_spikes=773_deliveries=41041_cells=41685_plasticity=462160
config_hash=c1-4db53e645405fae0 seed=4354472946875824171 condition=local-assembly accuracy=1.000000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=489841.000000 note=wall_secs=0.0048_peak_rss=3719168_spikes=721_deliveries=13244_cells=13876_plasticity=462000
config_hash=c1-4db53e645405fae0 seed=4354472946875824171 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2932332.000000 note=wall_secs=0.0094_peak_rss=4571136_spikes=693_deliveries=61680_cells=62353_plasticity=1341440
config_hash=c1-4db53e645405fae0 seed=4354472946875824171 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6654_peak_rss=2834432_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-4db53e645405fae0 seed=4354472946875824171 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0148_peak_rss=2637824_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-4db53e645405fae0 seed=4354472946875824171 condition=dense-matched accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=754587.561392 note=wall_secs=0.0057_peak_rss=4079616_spikes=712_deliveries=41859_cells=42505_plasticity=462000
config_hash=c1-4db53e645405fae0 seed=15755469790931547198 condition=local-assembly accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1150896.438306 note=wall_secs=0.0048_peak_rss=3801088_spikes=645_deliveries=12961_cells=13605_plasticity=461920
config_hash=c1-4db53e645405fae0 seed=15755469790931547198 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2932400.000000 note=wall_secs=0.0096_peak_rss=4587520_spikes=725_deliveries=61680_cells=62355_plasticity=1341440
config_hash=c1-4db53e645405fae0 seed=15755469790931547198 condition=gradient-reference accuracy=0.725000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=157139856.901371 note=wall_secs=0.6631_peak_rss=2899968_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-4db53e645405fae0 seed=15755469790931547198 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0151_peak_rss=2605056_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-4db53e645405fae0 seed=15755469790931547198 condition=dense-matched accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=751404.113221 note=wall_secs=0.0057_peak_rss=3964928_spikes=783_deliveries=40702_cells=41363_plasticity=461920
config_hash=c1-4db53e645405fae0 seed=8709160710835925077 condition=local-assembly accuracy=1.000000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=491480.000000 note=wall_secs=0.0047_peak_rss=3620864_spikes=686_deliveries=13440_cells=14074_plasticity=463280
config_hash=c1-4db53e645405fae0 seed=8709160710835925077 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2932380.000000 note=wall_secs=0.0097_peak_rss=4423680_spikes=717_deliveries=61680_cells=62353_plasticity=1341440
config_hash=c1-4db53e645405fae0 seed=8709160710835925077 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6665_peak_rss=2883584_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-4db53e645405fae0 seed=8709160710835925077 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0151_peak_rss=2654208_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-4db53e645405fae0 seed=8709160710835925077 condition=dense-matched accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=755151.699305 note=wall_secs=0.0059_peak_rss=3981312_spikes=748_deliveries=41400_cells=42057_plasticity=463280
config_hash=c1-4db53e645405fae0 seed=1663413756060003432 condition=local-assembly accuracy=0.825000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=595470.311635 note=wall_secs=0.0049_peak_rss=3686400_spikes=699_deliveries=13287_cells=13917_plasticity=463360
config_hash=c1-4db53e645405fae0 seed=1663413756060003432 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2932336.000000 note=wall_secs=0.0098_peak_rss=4505600_spikes=694_deliveries=61680_cells=62354_plasticity=1341440
config_hash=c1-4db53e645405fae0 seed=1663413756060003432 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6686_peak_rss=2883584_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-4db53e645405fae0 seed=1663413756060003432 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0154_peak_rss=2637824_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-4db53e645405fae0 seed=1663413756060003432 condition=dense-matched accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=753048.251098 note=wall_secs=0.0054_peak_rss=4046848_spikes=776_deliveries=40591_cells=41233_plasticity=463360
config_hash=c1-4db53e645405fae0 seed=13063846550650677375 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=979686.000000 note=wall_secs=0.0049_peak_rss=3801088_spikes=652_deliveries=13315_cells=13956_plasticity=461920
config_hash=c1-4db53e645405fae0 seed=13063846550650677375 condition=dense-local accuracy=0.100000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=14662239.781516 note=wall_secs=0.0098_peak_rss=4571136_spikes=747_deliveries=61680_cells=62357_plasticity=1341440
config_hash=c1-4db53e645405fae0 seed=13063846550650677375 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6786_peak_rss=2916352_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-4db53e645405fae0 seed=13063846550650677375 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0154_peak_rss=2605056_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-4db53e645405fae0 seed=13063846550650677375 condition=dense-matched accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=752987.561445 note=wall_secs=0.0057_peak_rss=4030464_spikes=796_deliveries=41279_cells=41921_plasticity=461920
config_hash=c1-4db53e645405fae0 seed=6018099320996848786 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=980050.000000 note=wall_secs=0.0049_peak_rss=3620864_spikes=658_deliveries=13440_cells=14087_plasticity=461840
config_hash=c1-4db53e645405fae0 seed=6018099320996848786 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2932394.000000 note=wall_secs=0.0097_peak_rss=4538368_spikes=729_deliveries=61680_cells=62348_plasticity=1341440
config_hash=c1-4db53e645405fae0 seed=6018099320996848786 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6774_peak_rss=2883584_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-4db53e645405fae0 seed=6018099320996848786 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0159_peak_rss=2637824_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-4db53e645405fae0 seed=6018099320996848786 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1088842.000000 note=wall_secs=0.0059_peak_rss=3948544_spikes=754_deliveries=40577_cells=41250_plasticity=461840
config_hash=c1-4db53e645405fae0 seed=17418529916564267177 condition=local-assembly accuracy=1.000000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=490722.000000 note=wall_secs=0.0052_peak_rss=3768320_spikes=661_deliveries=13509_cells=14152_plasticity=462400
config_hash=c1-4db53e645405fae0 seed=17418529916564267177 condition=dense-local accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2022281.312807 note=wall_secs=0.0097_peak_rss=4538368_spikes=715_deliveries=61680_cells=62319_plasticity=1341440
config_hash=c1-4db53e645405fae0 seed=17418529916564267177 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6683_peak_rss=2916352_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-4db53e645405fae0 seed=17418529916564267177 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0153_peak_rss=2605056_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-4db53e645405fae0 seed=17418529916564267177 condition=dense-matched accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=751942.044238 note=wall_secs=0.0056_peak_rss=3981312_spikes=715_deliveries=40702_cells=41341_plasticity=462400
config_hash=c1-4db53e645405fae0 seed=10372782686910438588 condition=local-assembly accuracy=0.900000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=544707.792208 note=wall_secs=0.0048_peak_rss=3637248_spikes=663_deliveries=13231_cells=13863_plasticity=462480
config_hash=c1-4db53e645405fae0 seed=10372782686910438588 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2932336.000000 note=wall_secs=0.0100_peak_rss=4587520_spikes=694_deliveries=61680_cells=62354_plasticity=1341440
config_hash=c1-4db53e645405fae0 seed=10372782686910438588 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6700_peak_rss=2801664_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-4db53e645405fae0 seed=10372782686910438588 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0150_peak_rss=2637824_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-4db53e645405fae0 seed=10372782686910438588 condition=dense-matched accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=752706.182144 note=wall_secs=0.0055_peak_rss=4030464_spikes=730_deliveries=40931_cells=41571_plasticity=462480
config_hash=c1-4db53e645405fae0 seed=3326471682669467859 condition=local-assembly accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=676201.357073 note=wall_secs=0.0047_peak_rss=3768320_spikes=675_deliveries=13350_cells=13981_plasticity=462240
config_hash=c1-4db53e645405fae0 seed=3326471682669467859 condition=dense-local accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2022347.519701 note=wall_secs=0.0093_peak_rss=4505600_spikes=759_deliveries=61680_cells=62323_plasticity=1341440
config_hash=c1-4db53e645405fae0 seed=3326471682669467859 condition=gradient-reference accuracy=0.725000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=157139856.901371 note=wall_secs=0.6632_peak_rss=2818048_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-4db53e645405fae0 seed=3326471682669467859 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0152_peak_rss=2605056_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-4db53e645405fae0 seed=3326471682669467859 condition=dense-matched accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=753525.492461 note=wall_secs=0.0057_peak_rss=4145152_spikes=835_deliveries=41294_cells=41937_plasticity=462240
config_hash=c1-4db53e645405fae0 seed=14727610363725173990 condition=local-assembly accuracy=0.900000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=544808.903321 note=wall_secs=0.0048_peak_rss=3637248_spikes=657_deliveries=13274_cells=13917_plasticity=462480
config_hash=c1-4db53e645405fae0 seed=14727610363725173990 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2932496.000000 note=wall_secs=0.0098_peak_rss=4571136_spikes=778_deliveries=61680_cells=62350_plasticity=1341440
config_hash=c1-4db53e645405fae0 seed=14727610363725173990 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6653_peak_rss=2818048_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-4db53e645405fae0 seed=14727610363725173990 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0150_peak_rss=2605056_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-4db53e645405fae0 seed=14727610363725173990 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1090038.000000 note=wall_secs=0.0059_peak_rss=4112384_spikes=740_deliveries=40562_cells=41237_plasticity=462480
config_hash=c1-4db53e645405fae0 seed=7681300184117924093 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=977122.000000 note=wall_secs=0.0047_peak_rss=3801088_spikes=670_deliveries=13190_cells=13821_plasticity=460880
config_hash=c1-4db53e645405fae0 seed=7681300184117924093 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2932440.000000 note=wall_secs=0.0096_peak_rss=4505600_spikes=751_deliveries=61680_cells=62349_plasticity=1341440
config_hash=c1-4db53e645405fae0 seed=7681300184117924093 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6619_peak_rss=2801664_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-4db53e645405fae0 seed=7681300184117924093 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0178_peak_rss=2637824_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-4db53e645405fae0 seed=7681300184117924093 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1089194.000000 note=wall_secs=0.0057_peak_rss=3964928_spikes=740_deliveries=41150_cells=41827_plasticity=460880
config_hash=c1-4db53e645405fae0 seed=635551854952467728 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=979232.000000 note=wall_secs=0.0047_peak_rss=3620864_spikes=648_deliveries=13323_cells=13965_plasticity=461680
config_hash=c1-4db53e645405fae0 seed=635551854952467728 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2932400.000000 note=wall_secs=0.0095_peak_rss=4521984_spikes=729_deliveries=61680_cells=62351_plasticity=1341440
config_hash=c1-4db53e645405fae0 seed=635551854952467728 condition=gradient-reference accuracy=0.500000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=227852800.000000 note=wall_secs=0.6634_peak_rss=2916352_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-4db53e645405fae0 seed=635551854952467728 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0154_peak_rss=2637824_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-4db53e645405fae0 seed=635551854952467728 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1094496.000000 note=wall_secs=0.0056_peak_rss=3948544_spikes=693_deliveries=42101_cells=42774_plasticity=461680
config_hash=c1-4db53e645405fae0 seed=12035985749054769447 condition=local-assembly accuracy=1.000000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=492192.000000 note=wall_secs=0.0048_peak_rss=3784704_spikes=787_deliveries=13552_cells=14173_plasticity=463680
config_hash=c1-4db53e645405fae0 seed=12035985749054769447 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2932328.000000 note=wall_secs=0.0095_peak_rss=4587520_spikes=690_deliveries=61680_cells=62354_plasticity=1341440
config_hash=c1-4db53e645405fae0 seed=12035985749054769447 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6647_peak_rss=2834432_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-4db53e645405fae0 seed=12035985749054769447 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0155_peak_rss=2654208_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-4db53e645405fae0 seed=12035985749054769447 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1092808.000000 note=wall_secs=0.0057_peak_rss=4046848_spikes=690_deliveries=40680_cells=41354_plasticity=463680
config_hash=c1-4db53e645405fae0 seed=4990235495743964474 condition=local-assembly accuracy=1.000000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=490817.000000 note=wall_secs=0.0047_peak_rss=3784704_spikes=674_deliveries=13394_cells=14029_plasticity=462720
config_hash=c1-4db53e645405fae0 seed=4990235495743964474 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2932416.000000 note=wall_secs=0.0095_peak_rss=4407296_spikes=740_deliveries=61680_cells=62348_plasticity=1341440
config_hash=c1-4db53e645405fae0 seed=4990235495743964474 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.7277_peak_rss=2965504_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-4db53e645405fae0 seed=4990235495743964474 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0400_peak_rss=2654208_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-4db53e645405fae0 seed=4990235495743964474 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1094620.000000 note=wall_secs=0.0057_peak_rss=4161536_spikes=727_deliveries=41595_cells=42268_plasticity=462720
config_hash=c1-4db53e645405fae0 seed=16390669389846266193 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=981158.000000 note=wall_secs=0.0050_peak_rss=3637248_spikes=694_deliveries=13267_cells=13898_plasticity=462720
config_hash=c1-4db53e645405fae0 seed=16390669389846266193 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2932392.000000 note=wall_secs=0.0099_peak_rss=4571136_spikes=728_deliveries=61680_cells=62348_plasticity=1341440
config_hash=c1-4db53e645405fae0 seed=16390669389846266193 condition=gradient-reference accuracy=0.500000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=227852800.000000 note=wall_secs=0.6697_peak_rss=2916352_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-4db53e645405fae0 seed=16390669389846266193 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0149_peak_rss=2654208_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-4db53e645405fae0 seed=16390669389846266193 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1091076.000000 note=wall_secs=0.0055_peak_rss=3964928_spikes=726_deliveries=40712_cells=41380_plasticity=462720
config_hash=c1-4db53e645405fae0 seed=9344921060680809828 condition=local-assembly accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=675684.115711 note=wall_secs=0.0048_peak_rss=3637248_spikes=697_deliveries=13296_cells=13958_plasticity=461920
config_hash=c1-4db53e645405fae0 seed=9344921060680809828 condition=dense-local accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2022281.312807 note=wall_secs=0.0094_peak_rss=4440064_spikes=716_deliveries=61680_cells=62318_plasticity=1341440
config_hash=c1-4db53e645405fae0 seed=9344921060680809828 condition=gradient-reference accuracy=0.725000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=157139856.901371 note=wall_secs=0.6627_peak_rss=2801664_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-4db53e645405fae0 seed=9344921060680809828 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0151_peak_rss=2637824_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-4db53e645405fae0 seed=9344921060680809828 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1094072.000000 note=wall_secs=0.0054_peak_rss=4161536_spikes=796_deliveries=41836_cells=42484_plasticity=461920
config_hash=c1-4db53e645405fae0 seed=2298610881073559931 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=980286.000000 note=wall_secs=0.0047_peak_rss=3784704_spikes=770_deliveries=13258_cells=13875_plasticity=462240
config_hash=c1-4db53e645405fae0 seed=2298610881073559931 condition=dense-local accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2022319.933495 note=wall_secs=0.0094_peak_rss=4571136_spikes=730_deliveries=61680_cells=62332_plasticity=1341440
config_hash=c1-4db53e645405fae0 seed=2298610881073559931 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6623_peak_rss=2834432_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-4db53e645405fae0 seed=2298610881073559931 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0155_peak_rss=2637824_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-4db53e645405fae0 seed=2298610881073559931 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1091084.000000 note=wall_secs=0.0057_peak_rss=4177920_spikes=721_deliveries=40953_cells=41628_plasticity=462240
config_hash=c1-4db53e645405fae0 seed=13699608824640910734 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=979486.000000 note=wall_secs=0.0047_peak_rss=3719168_spikes=686_deliveries=13017_cells=13640_plasticity=462400
config_hash=c1-4db53e645405fae0 seed=13699608824640910734 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2932432.000000 note=wall_secs=0.0096_peak_rss=4587520_spikes=750_deliveries=61680_cells=62346_plasticity=1341440
config_hash=c1-4db53e645405fae0 seed=13699608824640910734 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6628_peak_rss=2818048_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-4db53e645405fae0 seed=13699608824640910734 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0152_peak_rss=2637824_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-4db53e645405fae0 seed=13699608824640910734 condition=dense-matched accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=756559.975120 note=wall_secs=0.0057_peak_rss=4145152_spikes=838_deliveries=42313_cells=42955_plasticity=462400
config_hash=c1-4db53e645405fae0 seed=6653297820399940005 condition=local-assembly accuracy=0.650000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=754040.027658 note=wall_secs=0.0049_peak_rss=3735552_spikes=675_deliveries=13164_cells=13807_plasticity=462480
config_hash=c1-4db53e645405fae0 seed=6653297820399940005 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=2932410.000000 note=wall_secs=0.0097_peak_rss=4423680_spikes=762_deliveries=61680_cells=62323_plasticity=1341440
config_hash=c1-4db53e645405fae0 seed=6653297820399940005 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6647_peak_rss=2834432_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-4db53e645405fae0 seed=6653297820399940005 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0398_peak_rss=2654208_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-4db53e645405fae0 seed=6653297820399940005 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=1088066.000000 note=wall_secs=0.0053_peak_rss=4128768_spikes=745_deliveries=40084_cells=40724_plasticity=462480
```
