# C1 / Gate G2 results note

**Config hash:** `c1-714c115e14a3eeed`

**Scientific protocol version:** `14`

**Live RFB × epoch-matched protocol:** `14` — same neuromodulator as v13 (`ReinforceFeedback` × `reinforce_term`), but local/dense arms train for **20** epochs over the frozen train split (isolates single-pass handicap); **positive control stays on broadcast ±1**; does **not** remassage v13 hash `c1-660401d74db3c88d` or reopen protocol-v2 `c1-118207fbc3eaba53`.

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
Config { experiment: "c1-rfb-em", master_seed: 212618061021185, n_seeds: 20, sequence_len: 8, max_lag: 1, n_hidden: 128, k_wta: 2, p_sparse: 0.35, init_w: 0.15, eta: 0.35, lambda: 0.002, tau_e: 40.0, n_train: 80, n_test: 40, bptt_epochs: 80, bptt_lr: 0.05, g2_min_gap_closed: 0.5, g2_min_accuracy: 0.65, g2_confidence_z: 1.96, g2_min_positive_control: 0.9, g2_min_reference_gap: 0.15, activity_sparsity_min: 0.005, activity_sparsity_max: 0.03, scientific_n_seeds: 20, power_sigma_prior: 0.15, power_effect_size: 0.1, use_surrogate_lif_reference: true, surrogate_beta: 5.0, matched_budget_repeat: true, quick: false }
```

Power analysis: required scientific n_seeds ≥ 20 (preregistered σ=0.150, effect=0.100; formula n=⌈(1.96+0.8416)²σ²/δ²⌉).

## Per-seed accuracies

| seed | local-assembly | dense-local | gradient reference | eligibility reference | activity_sparsity (local) | activity_sparsity (dense) | dense_matched |
|---|---:|---:|---:|---:|---:|---:|---:|
| 11400784225994701844 | 0.5000 | 0.7250 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.9000 |
| 4354472946875824171 | 0.5000 | 0.5000 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.2750 |
| 15755469790931547198 | 0.4250 | 0.1750 | 0.7250 | 1.0000 | 0.0156 | 0.0156 | 0.2750 |
| 8709160710835925077 | 0.2750 | 0.5000 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.8250 |
| 1663413756060003432 | 0.4250 | 0.5000 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 13063846550650677375 | 0.3500 | 0.3000 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 6018099320996848786 | 0.2750 | 0.2750 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 17418529916564267177 | 0.5000 | 0.5000 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 10372782686910438588 | 0.5000 | 0.8250 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.7250 |
| 3326471682669467859 | 0.2750 | 0.2750 | 0.7250 | 1.0000 | 0.0156 | 0.0156 | 0.2750 |
| 14727610363725173990 | 0.5000 | 0.5000 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.7250 |
| 7681300184117924093 | 0.7500 | 0.9000 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.7250 |
| 635551854952467728 | 0.4250 | 0.1750 | 0.5000 | 1.0000 | 0.0156 | 0.0156 | 0.3000 |
| 12035985749054769447 | 0.5000 | 0.9000 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.7250 |
| 4990235495743964474 | 0.4250 | 0.5000 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.2750 |
| 16390669389846266193 | 0.6500 | 0.8250 | 0.5000 | 1.0000 | 0.0156 | 0.0156 | 0.7250 |
| 9344921060680809828 | 1.0000 | 0.5000 | 0.7250 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 2298610881073559931 | 0.2500 | 0.2750 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 13699608824640910734 | 0.7250 | 0.7250 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.8250 |
| 6653297820399940005 | 0.4250 | 0.8250 | 0.9500 | 1.0000 | 0.0156 | 0.0156 | 0.8250 |

## Summary (paired normalized-gap analysis)

- mean ± var local-assembly: 0.4838 ± 0.033965
- mean ± var dense-local:    0.5350 ± 0.058776
- mean ± var gradient reference: 0.8938 ± 0.027163
- mean ± var eligibility reference: 1.0000 ± 0.000000
- mean normalized gap closed: 0.1148 (variance 0.081054)
- lower confidence bound (z=1.960, n=20): -0.0100
- mean |local − dense| (descriptive): 0.1562

## U-NEG

Negative result: local-assembly did **not** clear the preregistered normalized-gap confidence and absolute-accuracy gates. Program stops at G2; do not schedule P3+.

## Positive / sanity control

Mean local-pipeline accuracy on a trivially separable spatial feature-presence task: **0.9488** (threshold 0.900).

## Activity sparsity

Mean local-assembly activity_sparsity: **0.0156** (valid band [0.0050, 0.0300]; nominal k/N=0.0156).

## Parameter / compute budgets

| condition | n_cells | n_params | wall_secs | peak_rss_bytes | work_per_accuracy | spikes | deliveries | cell_updates | plasticity |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| local-assembly | 132 | 5777 | 0.0528 | 4063232 | 19240668.0000 | 9274 | 179541 | 188319 | 9243200 |
| dense-local | 132 | 16768 | 0.1394 | 5439488 | 39356642.8437 | 10164 | 842960 | 851643 | 26828800 |
| gradient-reference | 130 | 16769 | 0.6620 | 2916352 | 113926400.0000 | 0 | 51200 | 6553600 | 107321600 |
| eligibility-reference | 130 | 385 | 0.0154 | 2605056 | 9068800.0000 | 0 | 51200 | 6553600 | 2464000 |
| dense-matched | 132 | 5777 | 0.0624 | 4505600 | 11530098.0832 | 10692 | 557255 | 565941 | 9243200 |

Matched-budget dense mean accuracy: **0.5700** (n=20; primary G2 gap still uses unmatched dense-local).

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
config_hash=c1-714c115e14a3eeed seed=11400784225994701844 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=19240668.000000 note=wall_secs=0.0528_peak_rss=4063232_spikes=9274_deliveries=179541_cells=188319_plasticity=9243200
config_hash=c1-714c115e14a3eeed seed=11400784225994701844 condition=dense-local accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=39356642.843675 note=wall_secs=0.1394_peak_rss=5439488_spikes=10164_deliveries=842960_cells=851643_plasticity=26828800
config_hash=c1-714c115e14a3eeed seed=11400784225994701844 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6620_peak_rss=2916352_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-714c115e14a3eeed seed=11400784225994701844 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0154_peak_rss=2605056_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-714c115e14a3eeed seed=11400784225994701844 condition=dense-matched accuracy=0.900000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=11530098.083221 note=wall_secs=0.0624_peak_rss=4505600_spikes=10692_deliveries=557255_cells=565941_plasticity=9243200
config_hash=c1-714c115e14a3eeed seed=4354472946875824171 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=19233118.000000 note=wall_secs=0.0535_peak_rss=4341760_spikes=9427_deliveries=179032_cells=188100_plasticity=9240000
config_hash=c1-714c115e14a3eeed seed=4354472946875824171 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=57071646.000000 note=wall_secs=0.1401_peak_rss=5029888_spikes=11819_deliveries=842960_cells=852244_plasticity=26828800
config_hash=c1-714c115e14a3eeed seed=4354472946875824171 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6567_peak_rss=2834432_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-714c115e14a3eeed seed=4354472946875824171 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0150_peak_rss=2588672_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-714c115e14a3eeed seed=4354472946875824171 condition=dense-matched accuracy=0.275000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=37864853.724756 note=wall_secs=0.0645_peak_rss=4505600_spikes=10918_deliveries=576380_cells=585537_plasticity=9240000
config_hash=c1-714c115e14a3eeed seed=15755469790931547198 condition=local-assembly accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=22615545.248004 note=wall_secs=0.0518_peak_rss=4259840_spikes=9052_deliveries=177599_cells=186556_plasticity=9238400
config_hash=c1-714c115e14a3eeed seed=15755469790931547198 condition=dense-local accuracy=0.175000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=163059065.634022 note=wall_secs=0.1388_peak_rss=4882432_spikes=11452_deliveries=842960_cells=852124_plasticity=26828800
config_hash=c1-714c115e14a3eeed seed=15755469790931547198 condition=gradient-reference accuracy=0.725000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=157139856.901371 note=wall_secs=0.6868_peak_rss=3047424_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-714c115e14a3eeed seed=15755469790931547198 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0161_peak_rss=2588672_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-714c115e14a3eeed seed=15755469790931547198 condition=dense-matched accuracy=0.275000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=37693901.001189 note=wall_secs=0.0639_peak_rss=4358144_spikes=11188_deliveries=553492_cells=562743_plasticity=9238400
config_hash=c1-714c115e14a3eeed seed=8709160710835925077 condition=local-assembly accuracy=0.275000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=35074526.512507 note=wall_secs=0.0526_peak_rss=4030464_spikes=9192_deliveries=180819_cells=189884_plasticity=9265600
config_hash=c1-714c115e14a3eeed seed=8709160710835925077 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=57067242.000000 note=wall_secs=0.1351_peak_rss=5079040_spikes=10215_deliveries=842960_cells=851646_plasticity=26828800
config_hash=c1-714c115e14a3eeed seed=8709160710835925077 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6557_peak_rss=2850816_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-714c115e14a3eeed seed=8709160710835925077 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0154_peak_rss=2588672_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-714c115e14a3eeed seed=8709160710835925077 condition=dense-matched accuracy=0.825000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=12633991.697708 note=wall_secs=0.0630_peak_rss=4325376_spikes=9632_deliveries=569459_cells=578352_plasticity=9265600
config_hash=c1-714c115e14a3eeed seed=1663413756060003432 condition=local-assembly accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=22688884.069476 note=wall_secs=0.0525_peak_rss=4210688_spikes=9020_deliveries=178773_cells=187783_plasticity=9267200
config_hash=c1-714c115e14a3eeed seed=1663413756060003432 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=57066946.000000 note=wall_secs=0.1367_peak_rss=4866048_spikes=9875_deliveries=842960_cells=851838_plasticity=26828800
config_hash=c1-714c115e14a3eeed seed=1663413756060003432 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6599_peak_rss=2850816_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-714c115e14a3eeed seed=1663413756060003432 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0154_peak_rss=2588672_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-714c115e14a3eeed seed=1663413756060003432 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=20839246.000000 note=wall_secs=0.0625_peak_rss=4521984_spikes=9773_deliveries=566896_cells=575754_plasticity=9267200
config_hash=c1-714c115e14a3eeed seed=13063846550650677375 condition=local-assembly accuracy=0.350000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=27494063.325364 note=wall_secs=0.0533_peak_rss=4292608_spikes=12002_deliveries=181752_cells=190768_plasticity=9238400
config_hash=c1-714c115e14a3eeed seed=13063846550650677375 condition=dense-local accuracy=0.300000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=95114009.553842 note=wall_secs=0.1395_peak_rss=5062656_spikes=10348_deliveries=842960_cells=852096_plasticity=26828800
config_hash=c1-714c115e14a3eeed seed=13063846550650677375 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6546_peak_rss=2850816_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-714c115e14a3eeed seed=13063846550650677375 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0175_peak_rss=2637824_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-714c115e14a3eeed seed=13063846550650677375 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=20747810.000000 note=wall_secs=0.0635_peak_rss=4489216_spikes=10777_deliveries=557743_cells=566985_plasticity=9238400
config_hash=c1-714c115e14a3eeed seed=6018099320996848786 condition=local-assembly accuracy=0.275000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=34991210.150677 note=wall_secs=0.0534_peak_rss=4194304_spikes=9758_deliveries=183361_cells=192664_plasticity=9236800
config_hash=c1-714c115e14a3eeed seed=6018099320996848786 condition=dense-local accuracy=0.275000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=103761339.569215 note=wall_secs=0.1408_peak_rss=5029888_spikes=10313_deliveries=842960_cells=852296_plasticity=26828800
config_hash=c1-714c115e14a3eeed seed=6018099320996848786 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6571_peak_rss=2850816_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-714c115e14a3eeed seed=6018099320996848786 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0394_peak_rss=2637824_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-714c115e14a3eeed seed=6018099320996848786 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=20708314.000000 note=wall_secs=0.0645_peak_rss=4390912_spikes=11674_deliveries=548188_cells=557495_plasticity=9236800
config_hash=c1-714c115e14a3eeed seed=17418529916564267177 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=19267862.000000 note=wall_secs=0.0533_peak_rss=4063232_spikes=10326_deliveries=183256_cells=192349_plasticity=9248000
config_hash=c1-714c115e14a3eeed seed=17418529916564267177 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=57066636.000000 note=wall_secs=0.1354_peak_rss=4898816_spikes=9806_deliveries=842960_cells=851752_plasticity=26828800
config_hash=c1-714c115e14a3eeed seed=17418529916564267177 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6566_peak_rss=2818048_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-714c115e14a3eeed seed=17418529916564267177 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0155_peak_rss=2621440_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-714c115e14a3eeed seed=17418529916564267177 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=20754548.000000 note=wall_secs=0.0625_peak_rss=4325376_spikes=9890_deliveries=555210_cells=564174_plasticity=9248000
config_hash=c1-714c115e14a3eeed seed=10372782686910438588 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=19276012.000000 note=wall_secs=0.0531_peak_rss=4308992_spikes=9629_deliveries=184877_cells=193900_plasticity=9249600
config_hash=c1-714c115e14a3eeed seed=10372782686910438588 condition=dense-local accuracy=0.825000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=34586070.802786 note=wall_secs=0.1361_peak_rss=5095424_spikes=9998_deliveries=842960_cells=851750_plasticity=26828800
config_hash=c1-714c115e14a3eeed seed=10372782686910438588 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6563_peak_rss=2850816_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-714c115e14a3eeed seed=10372782686910438588 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0149_peak_rss=2621440_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-714c115e14a3eeed seed=10372782686910438588 condition=dense-matched accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=14311166.425924 note=wall_secs=0.0624_peak_rss=4358144_spikes=9865_deliveries=553687_cells=562444_plasticity=9249600
config_hash=c1-714c115e14a3eeed seed=3326471682669467859 condition=local-assembly accuracy=0.275000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=35025639.240840 note=wall_secs=0.0544_peak_rss=4210688_spikes=10414_deliveries=183721_cells=193116_plasticity=9244800
config_hash=c1-714c115e14a3eeed seed=3326471682669467859 condition=dense-local accuracy=0.275000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=103769645.023581 note=wall_secs=0.1400_peak_rss=5062656_spikes=12647_deliveries=842960_cells=852246_plasticity=26828800
config_hash=c1-714c115e14a3eeed seed=3326471682669467859 condition=gradient-reference accuracy=0.725000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=157139856.901371 note=wall_secs=0.6974_peak_rss=2981888_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-714c115e14a3eeed seed=3326471682669467859 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0154_peak_rss=2605056_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-714c115e14a3eeed seed=3326471682669467859 condition=dense-matched accuracy=0.275000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=37790253.726373 note=wall_secs=0.0650_peak_rss=4374528_spikes=12687_deliveries=562746_cells=572087_plasticity=9244800
config_hash=c1-714c115e14a3eeed seed=14727610363725173990 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=19263216.000000 note=wall_secs=0.0522_peak_rss=4276224_spikes=9214_deliveries=181942_cells=190852_plasticity=9249600
config_hash=c1-714c115e14a3eeed seed=14727610363725173990 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=57070426.000000 note=wall_secs=0.1477_peak_rss=5586944_spikes=11718_deliveries=842960_cells=851735_plasticity=26828800
config_hash=c1-714c115e14a3eeed seed=14727610363725173990 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6548_peak_rss=2818048_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-714c115e14a3eeed seed=14727610363725173990 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0151_peak_rss=2605056_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-714c115e14a3eeed seed=14727610363725173990 condition=dense-matched accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=14348217.459189 note=wall_secs=0.0633_peak_rss=4521984_spikes=11519_deliveries=566285_cells=575054_plasticity=9249600
config_hash=c1-714c115e14a3eeed seed=7681300184117924093 condition=local-assembly accuracy=0.750000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=12787592.000000 note=wall_secs=0.0515_peak_rss=4243456_spikes=8813_deliveries=177744_cells=186537_plasticity=9217600
config_hash=c1-714c115e14a3eeed seed=7681300184117924093 condition=dense-local accuracy=0.900000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=31706026.395478 note=wall_secs=0.1371_peak_rss=4882432_spikes=11960_deliveries=842960_cells=851703_plasticity=26828800
config_hash=c1-714c115e14a3eeed seed=7681300184117924093 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6630_peak_rss=2932736_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-714c115e14a3eeed seed=7681300184117924093 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0176_peak_rss=2621440_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-714c115e14a3eeed seed=7681300184117924093 condition=dense-matched accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=14279014.702844 note=wall_secs=0.0623_peak_rss=4259840_spikes=11576_deliveries=557205_cells=565905_plasticity=9217600
config_hash=c1-714c115e14a3eeed seed=635551854952467728 condition=local-assembly accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=22617378.189129 note=wall_secs=0.0522_peak_rss=4276224_spikes=8988_deliveries=180407_cells=189391_plasticity=9233600
config_hash=c1-714c115e14a3eeed seed=635551854952467728 condition=dense-local accuracy=0.175000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=163067939.919888 note=wall_secs=0.1408_peak_rss=5079040_spikes=13058_deliveries=842960_cells=852071_plasticity=26828800
config_hash=c1-714c115e14a3eeed seed=635551854952467728 condition=gradient-reference accuracy=0.500000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=227852800.000000 note=wall_secs=0.6582_peak_rss=2850816_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-714c115e14a3eeed seed=635551854952467728 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0153_peak_rss=2637824_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-714c115e14a3eeed seed=635551854952467728 condition=dense-matched accuracy=0.300000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=34508898.628740 note=wall_secs=0.0633_peak_rss=4243456_spikes=10385_deliveries=549784_cells=558901_plasticity=9233600
config_hash=c1-714c115e14a3eeed seed=12035985749054769447 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=19326260.000000 note=wall_secs=0.0533_peak_rss=4358144_spikes=11559_deliveries=184467_cells=193504_plasticity=9273600
config_hash=c1-714c115e14a3eeed seed=12035985749054769447 condition=dense-local accuracy=0.900000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=31703985.284313 note=wall_secs=0.1364_peak_rss=5079040_spikes=10081_deliveries=842960_cells=851745_plasticity=26828800
config_hash=c1-714c115e14a3eeed seed=12035985749054769447 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6559_peak_rss=2850816_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-714c115e14a3eeed seed=12035985749054769447 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0147_peak_rss=2588672_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-714c115e14a3eeed seed=12035985749054769447 condition=dense-matched accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=14383505.733890 note=wall_secs=0.0634_peak_rss=4472832_spikes=10775_deliveries=567435_cells=576232_plasticity=9273600
config_hash=c1-714c115e14a3eeed seed=4990235495743964474 condition=local-assembly accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=22678128.775660 note=wall_secs=0.0530_peak_rss=4177920_spikes=9256_deliveries=182727_cells=191822_plasticity=9254400
config_hash=c1-714c115e14a3eeed seed=4990235495743964474 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=57070824.000000 note=wall_secs=0.1395_peak_rss=4882432_spikes=11427_deliveries=842960_cells=852225_plasticity=26828800
config_hash=c1-714c115e14a3eeed seed=4990235495743964474 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6969_peak_rss=2998272_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-714c115e14a3eeed seed=4990235495743964474 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0151_peak_rss=2621440_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-714c115e14a3eeed seed=4990235495743964474 condition=dense-matched accuracy=0.275000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=37849915.543262 note=wall_secs=0.0647_peak_rss=4358144_spikes=10560_deliveries=567220_cells=576547_plasticity=9254400
config_hash=c1-714c115e14a3eeed seed=16390669389846266193 condition=local-assembly accuracy=0.650000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=14816739.005013 note=wall_secs=0.0524_peak_rss=4046848_spikes=9239_deliveries=179215_cells=188026_plasticity=9254400
config_hash=c1-714c115e14a3eeed seed=16390669389846266193 condition=dense-local accuracy=0.825000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=34588211.408877 note=wall_secs=0.1359_peak_rss=4997120_spikes=11871_deliveries=842960_cells=851643_plasticity=26828800
config_hash=c1-714c115e14a3eeed seed=16390669389846266193 condition=gradient-reference accuracy=0.500000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=227852800.000000 note=wall_secs=0.6584_peak_rss=2932736_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-714c115e14a3eeed seed=16390669389846266193 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0151_peak_rss=2605056_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-714c115e14a3eeed seed=16390669389846266193 condition=dense-matched accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=14319213.322212 note=wall_secs=0.0622_peak_rss=4505600_spikes=10630_deliveries=553775_cells=562625_plasticity=9254400
config_hash=c1-714c115e14a3eeed seed=9344921060680809828 condition=local-assembly accuracy=1.000000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=9622525.000000 note=wall_secs=0.0510_peak_rss=4308992_spikes=9547_deliveries=183038_cells=191540_plasticity=9238400
config_hash=c1-714c115e14a3eeed seed=9344921060680809828 condition=dense-local accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=57067114.000000 note=wall_secs=0.1353_peak_rss=5095424_spikes=10084_deliveries=842960_cells=851713_plasticity=26828800
config_hash=c1-714c115e14a3eeed seed=9344921060680809828 condition=gradient-reference accuracy=0.725000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=157139856.901371 note=wall_secs=0.6584_peak_rss=2916352_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-714c115e14a3eeed seed=9344921060680809828 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0151_peak_rss=2621440_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-714c115e14a3eeed seed=9344921060680809828 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=20850280.000000 note=wall_secs=0.0624_peak_rss=4341760_spikes=9592_deliveries=584122_cells=593026_plasticity=9238400
config_hash=c1-714c115e14a3eeed seed=2298610881073559931 condition=local-assembly accuracy=0.250000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=38481636.000000 note=wall_secs=0.0531_peak_rss=4243456_spikes=9118_deliveries=178740_cells=187751_plasticity=9244800
config_hash=c1-714c115e14a3eeed seed=2298610881073559931 condition=dense-local accuracy=0.275000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=103769775.932669 note=wall_secs=0.1401_peak_rss=5062656_spikes=12627_deliveries=842960_cells=852302_plasticity=26828800
config_hash=c1-714c115e14a3eeed seed=2298610881073559931 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=113926400.000000 note=wall_secs=0.6583_peak_rss=2834432_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-714c115e14a3eeed seed=2298610881073559931 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0154_peak_rss=2637824_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-714c115e14a3eeed seed=2298610881073559931 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=20825516.000000 note=wall_secs=0.0643_peak_rss=4505600_spikes=11657_deliveries=573511_cells=582790_plasticity=9244800
config_hash=c1-714c115e14a3eeed seed=13699608824640910734 condition=local-assembly accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=13277344.390956 note=wall_secs=0.0522_peak_rss=4063232_spikes=9684_deliveries=179858_cells=188533_plasticity=9248000
config_hash=c1-714c115e14a3eeed seed=13699608824640910734 condition=dense-local accuracy=0.725000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=39359219.395315 note=wall_secs=0.1366_peak_rss=5046272_spikes=12087_deliveries=842960_cells=851588_plasticity=26828800
config_hash=c1-714c115e14a3eeed seed=13699608824640910734 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6592_peak_rss=2850816_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-714c115e14a3eeed seed=13699608824640910734 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0156_peak_rss=2605056_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-714c115e14a3eeed seed=13699608824640910734 condition=dense-matched accuracy=0.825000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=12578292.302964 note=wall_secs=0.0629_peak_rss=4390912_spikes=11882_deliveries=554245_cells=562964_plasticity=9248000
config_hash=c1-714c115e14a3eeed seed=6653297820399940005 condition=local-assembly accuracy=0.425000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=22647397.011816 note=wall_secs=0.0519_peak_rss=4014080_spikes=9010_deliveries=178764_cells=187770_plasticity=9249600
config_hash=c1-714c115e14a3eeed seed=6653297820399940005 condition=dense-local accuracy=0.825000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=34585765.348236 note=wall_secs=0.1363_peak_rss=4997120_spikes=9667_deliveries=842960_cells=851829_plasticity=26828800
config_hash=c1-714c115e14a3eeed seed=6653297820399940005 condition=gradient-reference accuracy=0.950000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=119922527.820619 note=wall_secs=0.6575_peak_rss=2818048_spikes=0_deliveries=51200_cells=6553600_plasticity=107321600
config_hash=c1-714c115e14a3eeed seed=6653297820399940005 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=9068800.000000 note=wall_secs=0.0150_peak_rss=2605056_spikes=0_deliveries=51200_cells=6553600_plasticity=2464000
config_hash=c1-714c115e14a3eeed seed=6653297820399940005 condition=dense-matched accuracy=0.825000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=12620950.485398 note=wall_secs=0.0631_peak_rss=4521984_spikes=9758_deliveries=572062_cells=580864_plasticity=9249600
```
