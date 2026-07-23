# C1 / Gate G2 results note

**Config hash:** `c1-d38d7644d8afc84b`

**Scientific protocol version:** `3`

**Sensitivity protocol (Tier-B):** `3` — optional confound probe; does **not** reopen protocol-v2 kill-gate hash `c1-118207fbc3eaba53` (canonical version `2`).

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
Config { experiment: "c1-sens-capacity", master_seed: 213073327554561, n_seeds: 20, sequence_len: 8, max_lag: 1, n_hidden: 256, k_wta: 4, p_sparse: 0.3, init_w: 0.15, eta: 0.2, lambda: 0.002, tau_e: 40.0, n_train: 200, n_test: 100, bptt_epochs: 150, bptt_lr: 0.02, g2_min_gap_closed: 0.5, g2_min_accuracy: 0.65, g2_confidence_z: 1.96, g2_min_positive_control: 0.9, g2_min_reference_gap: 0.15, activity_sparsity_min: 0.005, activity_sparsity_max: 0.03, scientific_n_seeds: 20, power_sigma_prior: 0.15, power_effect_size: 0.1, use_surrogate_lif_reference: true, surrogate_beta: 5.0, matched_budget_repeat: true, quick: false }
```

Power analysis: required scientific n_seeds ≥ 20 (preregistered σ=0.150, effect=0.100; formula n=⌈(1.96+0.8416)²σ²/δ²⌉).

## Per-seed accuracies

| seed | local-assembly | dense-local | gradient reference | eligibility reference | activity_sparsity (local) | activity_sparsity (dense) | dense_matched |
|---|---:|---:|---:|---:|---:|---:|---:|
| 11400783787908037652 | 0.7400 | 0.9400 | 0.9300 | 1.0000 | 0.0156 | 0.0156 | 0.8900 |
| 4354473659840395307 | 0.5000 | 0.9400 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.9400 |
| 15755470452356510782 | 0.7400 | 0.9400 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.9100 |
| 8709160341468737621 | 0.9300 | 0.9400 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.9400 |
| 1663413025915563112 | 0.5000 | 0.9400 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.9300 |
| 13063846937197734015 | 0.7400 | 0.9400 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.9400 |
| 6018099759083512978 | 0.7400 | 0.9400 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.9400 |
| 17418529186419826857 | 0.7400 | 0.9400 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.9300 |
| 10372782042665344188 | 0.5000 | 0.9400 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.9100 |
| 3326472326914562259 | 0.7400 | 0.9400 | 0.9300 | 1.0000 | 0.0156 | 0.0156 | 0.9400 |
| 14727609925638509798 | 0.7400 | 0.9400 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.9300 |
| 7681299814750736637 | 0.7400 | 0.9400 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.9400 |
| 635552499197562128 | 0.5000 | 0.9400 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.7400 |
| 12035986479199209767 | 0.7400 | 0.9400 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.9300 |
| 4990234765599524154 | 0.7400 | 0.9400 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.7400 |
| 16390668728421302609 | 0.7400 | 0.9400 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.9100 |
| 9344921515947343204 | 0.7400 | 0.9400 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.9000 |
| 2298611250440747387 | 0.5000 | 0.9400 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.7400 |
| 13699608180395816334 | 0.7400 | 0.9400 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.5000 |
| 6653298481824903589 | 0.5000 | 0.9400 | 1.0000 | 1.0000 | 0.0156 | 0.0156 | 0.9300 |

## Summary (paired normalized-gap analysis)

- mean ± var local-assembly: 0.6775 ± 0.015978
- mean ± var dense-local:    0.9400 ± 0.000000
- mean ± var gradient reference: 0.9930 ± 0.000464
- mean ± var eligibility reference: 1.0000 ± 0.000000
- mean normalized gap closed: 0.0000 (variance 0.000000)
- lower confidence bound (z=1.960, n=20): 0.0000
- mean |local − dense| (descriptive): 0.2625

## U-NEG

Negative result: local-assembly did **not** clear the preregistered normalized-gap confidence and absolute-accuracy gates. Program stops at G2; do not schedule P3+.

## Positive / sanity control

Mean local-pipeline accuracy on a trivially separable spatial feature-presence task: **1.0000** (threshold 0.900).

## Activity sparsity

Mean local-assembly activity_sparsity: **0.0156** (valid band [0.0050, 0.0300]; nominal k/N=0.0156).

## Parameter / compute budgets

| condition | n_cells | n_params | wall_secs | peak_rss_bytes | work_per_accuracy | spikes | deliveries | cell_updates | plasticity |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| local-assembly | 260 | 20012 | 0.0234 | 10616832 | 7485291.7954 | 3064 | 105347 | 107513 | 5323192 |
| dense-local | 260 | 66304 | 0.0788 | 13549568 | 25182999.0000 | 3504 | 462000 | 464243 | 22742272 |
| gradient-reference | 258 | 66305 | 14.9377 | 9666560 | 2205193531.4271 | 0 | 240000 | 61440000 | 1989150000 |
| eligibility-reference | 258 | 769 | 0.1240 | 8519680 | 84750000.0000 | 0 | 240000 | 61440000 | 23070000 |
| dense-matched | 260 | 20012 | 0.0299 | 11255808 | 8269318.1104 | 3494 | 244920 | 247163 | 6864116 |

Matched-budget dense mean accuracy: **0.8765** (n=20; primary G2 gap still uses unmatched dense-local).

## Plots

- raster: Skipped("pyo3/matplotlib unavailable or failed")
- weights: Skipped("pyo3/matplotlib unavailable or failed")
- raster: Skipped("pyo3/matplotlib unavailable or failed")
- weights: Skipped("pyo3/matplotlib unavailable or failed")
- raster: Skipped("pyo3/matplotlib unavailable or failed")
- weights: Skipped("pyo3/matplotlib unavailable or failed")
- raster: Skipped("pyo3/matplotlib unavailable or failed")
- weights: Skipped("pyo3/matplotlib unavailable or failed")
- raster: Skipped("pyo3/matplotlib unavailable or failed")
- weights: Skipped("pyo3/matplotlib unavailable or failed")
- raster: Skipped("pyo3/matplotlib unavailable or failed")
- weights: Skipped("pyo3/matplotlib unavailable or failed")
- raster: Skipped("pyo3/matplotlib unavailable or failed")
- weights: Skipped("pyo3/matplotlib unavailable or failed")
- raster: Skipped("pyo3/matplotlib unavailable or failed")
- weights: Skipped("pyo3/matplotlib unavailable or failed")
- raster: Skipped("pyo3/matplotlib unavailable or failed")
- weights: Skipped("pyo3/matplotlib unavailable or failed")
- raster: Skipped("pyo3/matplotlib unavailable or failed")
- weights: Skipped("pyo3/matplotlib unavailable or failed")
- raster: Skipped("pyo3/matplotlib unavailable or failed")
- weights: Skipped("pyo3/matplotlib unavailable or failed")
- raster: Skipped("pyo3/matplotlib unavailable or failed")
- weights: Skipped("pyo3/matplotlib unavailable or failed")
- raster: Skipped("pyo3/matplotlib unavailable or failed")
- weights: Skipped("pyo3/matplotlib unavailable or failed")
- raster: Skipped("pyo3/matplotlib unavailable or failed")
- weights: Skipped("pyo3/matplotlib unavailable or failed")
- raster: Skipped("pyo3/matplotlib unavailable or failed")
- weights: Skipped("pyo3/matplotlib unavailable or failed")
- raster: Skipped("pyo3/matplotlib unavailable or failed")
- weights: Skipped("pyo3/matplotlib unavailable or failed")
- raster: Skipped("pyo3/matplotlib unavailable or failed")
- weights: Skipped("pyo3/matplotlib unavailable or failed")
- raster: Skipped("pyo3/matplotlib unavailable or failed")
- weights: Skipped("pyo3/matplotlib unavailable or failed")
- raster: Skipped("pyo3/matplotlib unavailable or failed")
- weights: Skipped("pyo3/matplotlib unavailable or failed")
- raster: Skipped("pyo3/matplotlib unavailable or failed")
- weights: Skipped("pyo3/matplotlib unavailable or failed")

## Structured log (GC7)

```
config_hash=c1-d38d7644d8afc84b seed=11400783787908037652 condition=local-assembly accuracy=0.740000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=7485291.795425 note=wall_secs=0.0234_peak_rss=10616832_spikes=3064_deliveries=105347_cells=107513_plasticity=5323192
config_hash=c1-d38d7644d8afc84b seed=11400783787908037652 condition=dense-local accuracy=0.940000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=25182999.000044 note=wall_secs=0.0788_peak_rss=13549568_spikes=3504_deliveries=462000_cells=464243_plasticity=22742272
config_hash=c1-d38d7644d8afc84b seed=11400783787908037652 condition=gradient-reference accuracy=0.930000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=2205193531.427125 note=wall_secs=14.9377_peak_rss=9666560_spikes=0_deliveries=240000_cells=61440000_plasticity=1989150000
config_hash=c1-d38d7644d8afc84b seed=11400783787908037652 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=84750000.000000 note=wall_secs=0.1240_peak_rss=8519680_spikes=0_deliveries=240000_cells=61440000_plasticity=23070000
config_hash=c1-d38d7644d8afc84b seed=11400783787908037652 condition=dense-matched accuracy=0.890000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=8269318.110442 note=wall_secs=0.0299_peak_rss=11255808_spikes=3494_deliveries=244920_cells=247163_plasticity=6864116
config_hash=c1-d38d7644d8afc84b seed=4354473659840395307 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=12512756.000000 note=wall_secs=0.0241_peak_rss=10584064_spikes=2848_deliveries=105060_cells=107262_plasticity=6041208
config_hash=c1-d38d7644d8afc84b seed=4354473659840395307 condition=dense-local accuracy=0.940000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=25182999.000044 note=wall_secs=0.0784_peak_rss=13516800_spikes=3504_deliveries=462000_cells=464243_plasticity=22742272
config_hash=c1-d38d7644d8afc84b seed=4354473659840395307 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=2050830000.000000 note=wall_secs=14.8432_peak_rss=9682944_spikes=0_deliveries=240000_cells=61440000_plasticity=1989150000
config_hash=c1-d38d7644d8afc84b seed=4354473659840395307 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=84750000.000000 note=wall_secs=0.1224_peak_rss=8519680_spikes=0_deliveries=240000_cells=61440000_plasticity=23070000
config_hash=c1-d38d7644d8afc84b seed=4354473659840395307 condition=dense-matched accuracy=0.940000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=7827059.594320 note=wall_secs=0.0298_peak_rss=11239424_spikes=3483_deliveries=245169_cells=247412_plasticity=6861372
config_hash=c1-d38d7644d8afc84b seed=15755470452356510782 condition=local-assembly accuracy=0.740000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=7378728.283285 note=wall_secs=0.0242_peak_rss=10715136_spikes=3807_deliveries=106097_cells=108259_plasticity=5242096
config_hash=c1-d38d7644d8afc84b seed=15755470452356510782 condition=dense-local accuracy=0.940000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=25182999.000044 note=wall_secs=0.0824_peak_rss=13533184_spikes=3504_deliveries=462000_cells=464243_plasticity=22742272
config_hash=c1-d38d7644d8afc84b seed=15755470452356510782 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=2050830000.000000 note=wall_secs=15.5508_peak_rss=9650176_spikes=0_deliveries=240000_cells=61440000_plasticity=1989150000
config_hash=c1-d38d7644d8afc84b seed=15755470452356510782 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=84750000.000000 note=wall_secs=0.1268_peak_rss=8519680_spikes=0_deliveries=240000_cells=61440000_plasticity=23070000
config_hash=c1-d38d7644d8afc84b seed=15755470452356510782 condition=dense-matched accuracy=0.910000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=8084944.821938 note=wall_secs=0.0309_peak_rss=11239424_spikes=3485_deliveries=244414_cells=246657_plasticity=6862744
config_hash=c1-d38d7644d8afc84b seed=8709160341468737621 condition=local-assembly accuracy=0.930000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=6446008.552575 note=wall_secs=0.0365_peak_rss=11304960_spikes=3469_deliveries=105143_cells=107332_plasticity=5778844
config_hash=c1-d38d7644d8afc84b seed=8709160341468737621 condition=dense-local accuracy=0.940000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=25182999.000044 note=wall_secs=0.0883_peak_rss=15351808_spikes=3504_deliveries=462000_cells=464243_plasticity=22742272
config_hash=c1-d38d7644d8afc84b seed=8709160341468737621 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=2050830000.000000 note=wall_secs=15.8365_peak_rss=9682944_spikes=0_deliveries=240000_cells=61440000_plasticity=1989150000
config_hash=c1-d38d7644d8afc84b seed=8709160341468737621 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=84750000.000000 note=wall_secs=0.1309_peak_rss=8519680_spikes=0_deliveries=240000_cells=61440000_plasticity=23070000
config_hash=c1-d38d7644d8afc84b seed=8709160341468737621 condition=dense-matched accuracy=0.940000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=7821332.998561 note=wall_secs=0.0318_peak_rss=11223040_spikes=3504_deliveries=243839_cells=246082_plasticity=6858628
config_hash=c1-d38d7644d8afc84b seed=1663413025915563112 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=12876202.000000 note=wall_secs=0.0290_peak_rss=10928128_spikes=2476_deliveries=105152_cells=107363_plasticity=6223110
config_hash=c1-d38d7644d8afc84b seed=1663413025915563112 condition=dense-local accuracy=0.940000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=25182999.000044 note=wall_secs=0.0918_peak_rss=14794752_spikes=3504_deliveries=462000_cells=464243_plasticity=22742272
config_hash=c1-d38d7644d8afc84b seed=1663413025915563112 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=2050830000.000000 note=wall_secs=16.2130_peak_rss=9715712_spikes=0_deliveries=240000_cells=61440000_plasticity=1989150000
config_hash=c1-d38d7644d8afc84b seed=1663413025915563112 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=84750000.000000 note=wall_secs=0.1260_peak_rss=2605056_spikes=0_deliveries=240000_cells=61440000_plasticity=23070000
config_hash=c1-d38d7644d8afc84b seed=1663413025915563112 condition=dense-matched accuracy=0.930000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=7913983.810102 note=wall_secs=0.0334_peak_rss=11288576_spikes=3504_deliveries=245414_cells=247657_plasticity=6863430
config_hash=c1-d38d7644d8afc84b seed=13063846937197734015 condition=local-assembly accuracy=0.740000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=7506849.903256 note=wall_secs=0.0243_peak_rss=4718592_spikes=2954_deliveries=105241_cells=107408_plasticity=5339466
config_hash=c1-d38d7644d8afc84b seed=13063846937197734015 condition=dense-local accuracy=0.940000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=25182999.000044 note=wall_secs=0.0857_peak_rss=8585216_spikes=3504_deliveries=462000_cells=464243_plasticity=22742272
config_hash=c1-d38d7644d8afc84b seed=13063846937197734015 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=2050830000.000000 note=wall_secs=15.8521_peak_rss=3784704_spikes=0_deliveries=240000_cells=61440000_plasticity=1989150000
config_hash=c1-d38d7644d8afc84b seed=13063846937197734015 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=84750000.000000 note=wall_secs=0.1256_peak_rss=2605056_spikes=0_deliveries=240000_cells=61440000_plasticity=23070000
config_hash=c1-d38d7644d8afc84b seed=13063846937197734015 condition=dense-matched accuracy=0.940000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=7821598.956009 note=wall_secs=0.0322_peak_rss=5373952_spikes=3494_deliveries=243626_cells=245869_plasticity=6859314
config_hash=c1-d38d7644d8afc84b seed=6018099759083512978 condition=local-assembly accuracy=0.740000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=7351089.094452 note=wall_secs=0.0240_peak_rss=4784128_spikes=3327_deliveries=105593_cells=107754_plasticity=5223132
config_hash=c1-d38d7644d8afc84b seed=6018099759083512978 condition=dense-local accuracy=0.940000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=25182999.000044 note=wall_secs=0.0843_peak_rss=7634944_spikes=3504_deliveries=462000_cells=464243_plasticity=22742272
config_hash=c1-d38d7644d8afc84b seed=6018099759083512978 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=2050830000.000000 note=wall_secs=16.0030_peak_rss=3751936_spikes=0_deliveries=240000_cells=61440000_plasticity=1989150000
config_hash=c1-d38d7644d8afc84b seed=6018099759083512978 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=84750000.000000 note=wall_secs=0.1251_peak_rss=2605056_spikes=0_deliveries=240000_cells=61440000_plasticity=23070000
config_hash=c1-d38d7644d8afc84b seed=6018099759083512978 condition=dense-matched accuracy=0.940000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=7826926.615597 note=wall_secs=0.0318_peak_rss=5308416_spikes=3504_deliveries=243724_cells=245967_plasticity=6864116
config_hash=c1-d38d7644d8afc84b seed=17418529186419826857 condition=local-assembly accuracy=0.740000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=7378814.769770 note=wall_secs=0.0238_peak_rss=4784128_spikes=3823_deliveries=106121_cells=108283_plasticity=5242096
config_hash=c1-d38d7644d8afc84b seed=17418529186419826857 condition=dense-local accuracy=0.940000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=25182999.000044 note=wall_secs=0.0842_peak_rss=7634944_spikes=3504_deliveries=462000_cells=464243_plasticity=22742272
config_hash=c1-d38d7644d8afc84b seed=17418529186419826857 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=2050830000.000000 note=wall_secs=16.0044_peak_rss=3751936_spikes=0_deliveries=240000_cells=61440000_plasticity=1989150000
config_hash=c1-d38d7644d8afc84b seed=17418529186419826857 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=84750000.000000 note=wall_secs=0.1266_peak_rss=8519680_spikes=0_deliveries=240000_cells=61440000_plasticity=23070000
config_hash=c1-d38d7644d8afc84b seed=17418529186419826857 condition=dense-matched accuracy=0.930000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=7914064.455263 note=wall_secs=0.0321_peak_rss=5373952_spikes=3507_deliveries=245793_cells=248036_plasticity=6862744
config_hash=c1-d38d7644d8afc84b seed=10372782042665344188 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=12277538.000000 note=wall_secs=0.0261_peak_rss=10616832_spikes=3627_deliveries=105881_cells=108077_plasticity=5921184
config_hash=c1-d38d7644d8afc84b seed=10372782042665344188 condition=dense-local accuracy=0.940000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=25182999.000044 note=wall_secs=0.0848_peak_rss=13860864_spikes=3504_deliveries=462000_cells=464243_plasticity=22742272
config_hash=c1-d38d7644d8afc84b seed=10372782042665344188 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=2050830000.000000 note=wall_secs=16.3277_peak_rss=9666560_spikes=0_deliveries=240000_cells=61440000_plasticity=1989150000
config_hash=c1-d38d7644d8afc84b seed=10372782042665344188 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=84750000.000000 note=wall_secs=0.1276_peak_rss=8519680_spikes=0_deliveries=240000_cells=61440000_plasticity=23070000
config_hash=c1-d38d7644d8afc84b seed=10372782042665344188 condition=dense-matched accuracy=0.910000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=8081434.931930 note=wall_secs=0.0323_peak_rss=11223040_spikes=3491_deliveries=243500_cells=245743_plasticity=6861372
config_hash=c1-d38d7644d8afc84b seed=3326472326914562259 condition=local-assembly accuracy=0.740000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=7365872.878045 note=wall_secs=0.0241_peak_rss=10600448_spikes=3040_deliveries=104999_cells=107161_plasticity=5235546
config_hash=c1-d38d7644d8afc84b seed=3326472326914562259 condition=dense-local accuracy=0.940000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=25182999.000044 note=wall_secs=0.0862_peak_rss=13565952_spikes=3504_deliveries=462000_cells=464243_plasticity=22742272
config_hash=c1-d38d7644d8afc84b seed=3326472326914562259 condition=gradient-reference accuracy=0.930000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=2205193531.427125 note=wall_secs=16.0124_peak_rss=9666560_spikes=0_deliveries=240000_cells=61440000_plasticity=1989150000
config_hash=c1-d38d7644d8afc84b seed=3326472326914562259 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=84750000.000000 note=wall_secs=0.1290_peak_rss=8519680_spikes=0_deliveries=240000_cells=61440000_plasticity=23070000
config_hash=c1-d38d7644d8afc84b seed=3326472326914562259 condition=dense-matched accuracy=0.940000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=7820539.381538 note=wall_secs=0.0334_peak_rss=11550720_spikes=3507_deliveries=245694_cells=247937_plasticity=6854169
config_hash=c1-d38d7644d8afc84b seed=14727609925638509798 condition=local-assembly accuracy=0.740000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=7379349.904899 note=wall_secs=0.0243_peak_rss=10649600_spikes=3887_deliveries=105894_cells=108056_plasticity=5242882
config_hash=c1-d38d7644d8afc84b seed=14727609925638509798 condition=dense-local accuracy=0.940000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=25182999.000044 note=wall_secs=0.0853_peak_rss=13565952_spikes=3504_deliveries=462000_cells=464243_plasticity=22742272
config_hash=c1-d38d7644d8afc84b seed=14727609925638509798 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=2050830000.000000 note=wall_secs=15.9202_peak_rss=9666560_spikes=0_deliveries=240000_cells=61440000_plasticity=1989150000
config_hash=c1-d38d7644d8afc84b seed=14727609925638509798 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=84750000.000000 note=wall_secs=0.1257_peak_rss=8519680_spikes=0_deliveries=240000_cells=61440000_plasticity=23070000
config_hash=c1-d38d7644d8afc84b seed=14727609925638509798 condition=dense-matched accuracy=0.930000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=7910174.132712 note=wall_secs=0.0324_peak_rss=11255808_spikes=3504_deliveries=243471_cells=245714_plasticity=6863773
config_hash=c1-d38d7644d8afc84b seed=7681299814750736637 condition=local-assembly accuracy=0.740000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=7478776.930644 note=wall_secs=0.0242_peak_rss=10584064_spikes=2587_deliveries=104239_cells=106405_plasticity=5321064
config_hash=c1-d38d7644d8afc84b seed=7681299814750736637 condition=dense-local accuracy=0.940000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=25182999.000044 note=wall_secs=0.0850_peak_rss=13500416_spikes=3504_deliveries=462000_cells=464243_plasticity=22742272
config_hash=c1-d38d7644d8afc84b seed=7681299814750736637 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=2050830000.000000 note=wall_secs=15.9356_peak_rss=9666560_spikes=0_deliveries=240000_cells=61440000_plasticity=1989150000
config_hash=c1-d38d7644d8afc84b seed=7681299814750736637 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=84750000.000000 note=wall_secs=0.1258_peak_rss=8503296_spikes=0_deliveries=240000_cells=61440000_plasticity=23070000
config_hash=c1-d38d7644d8afc84b seed=7681299814750736637 condition=dense-matched accuracy=0.940000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=7829352.147518 note=wall_secs=0.0325_peak_rss=11206656_spikes=3504_deliveries=246236_cells=248479_plasticity=6861372
config_hash=c1-d38d7644d8afc84b seed=635552499197562128 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=12438334.000000 note=wall_secs=0.0259_peak_rss=10797056_spikes=3863_deliveries=105502_cells=107702_plasticity=6002100
config_hash=c1-d38d7644d8afc84b seed=635552499197562128 condition=dense-local accuracy=0.940000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=25182999.000044 note=wall_secs=0.0872_peak_rss=13746176_spikes=3504_deliveries=462000_cells=464243_plasticity=22742272
config_hash=c1-d38d7644d8afc84b seed=635552499197562128 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=2050830000.000000 note=wall_secs=15.9895_peak_rss=9666560_spikes=0_deliveries=240000_cells=61440000_plasticity=1989150000
config_hash=c1-d38d7644d8afc84b seed=635552499197562128 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=84750000.000000 note=wall_secs=0.1252_peak_rss=8519680_spikes=0_deliveries=240000_cells=61440000_plasticity=23070000
config_hash=c1-d38d7644d8afc84b seed=635552499197562128 condition=dense-matched accuracy=0.740000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=9730982.307024 note=wall_secs=0.0331_peak_rss=11534336_spikes=3785_deliveries=246281_cells=248516_plasticity=6702345
config_hash=c1-d38d7644d8afc84b seed=12035986479199209767 condition=local-assembly accuracy=0.740000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=7485970.173795 note=wall_secs=0.0249_peak_rss=10649600_spikes=3604_deliveries=105594_cells=107760_plasticity=5322660
config_hash=c1-d38d7644d8afc84b seed=12035986479199209767 condition=dense-local accuracy=0.940000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=25182999.000044 note=wall_secs=0.0848_peak_rss=13484032_spikes=3504_deliveries=462000_cells=464243_plasticity=22742272
config_hash=c1-d38d7644d8afc84b seed=12035986479199209767 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=2050830000.000000 note=wall_secs=15.7524_peak_rss=9666560_spikes=0_deliveries=240000_cells=61440000_plasticity=1989150000
config_hash=c1-d38d7644d8afc84b seed=12035986479199209767 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=84750000.000000 note=wall_secs=0.1484_peak_rss=8519680_spikes=0_deliveries=240000_cells=61440000_plasticity=23070000
config_hash=c1-d38d7644d8afc84b seed=12035986479199209767 condition=dense-matched accuracy=0.930000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=7912856.928390 note=wall_secs=0.0320_peak_rss=11255808_spikes=3504_deliveries=244890_cells=247133_plasticity=6863430
config_hash=c1-d38d7644d8afc84b seed=4990234765599524154 condition=local-assembly accuracy=0.740000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=7402140.445146 note=wall_secs=0.0271_peak_rss=11173888_spikes=3345_deliveries=104723_cells=106886_plasticity=5262630
config_hash=c1-d38d7644d8afc84b seed=4990234765599524154 condition=dense-local accuracy=0.940000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=25182999.000044 note=wall_secs=0.0959_peak_rss=15417344_spikes=3504_deliveries=462000_cells=464243_plasticity=22742272
config_hash=c1-d38d7644d8afc84b seed=4990234765599524154 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=2050830000.000000 note=wall_secs=15.8075_peak_rss=9682944_spikes=0_deliveries=240000_cells=61440000_plasticity=1989150000
config_hash=c1-d38d7644d8afc84b seed=4990234765599524154 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=84750000.000000 note=wall_secs=0.1251_peak_rss=8536064_spikes=0_deliveries=240000_cells=61440000_plasticity=23070000
config_hash=c1-d38d7644d8afc84b seed=4990234765599524154 condition=dense-matched accuracy=0.740000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=9740741.766358 note=wall_secs=0.0314_peak_rss=11239424_spikes=3784_deliveries=249390_cells=251625_plasticity=6703350
config_hash=c1-d38d7644d8afc84b seed=16390668728421302609 condition=local-assembly accuracy=0.740000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=7558508.010698 note=wall_secs=0.0243_peak_rss=10649600_spikes=3172_deliveries=105457_cells=107626_plasticity=5377041
config_hash=c1-d38d7644d8afc84b seed=16390668728421302609 condition=dense-local accuracy=0.940000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=25182999.000044 note=wall_secs=0.0835_peak_rss=13434880_spikes=3504_deliveries=462000_cells=464243_plasticity=22742272
config_hash=c1-d38d7644d8afc84b seed=16390668728421302609 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=2050830000.000000 note=wall_secs=15.7265_peak_rss=9666560_spikes=0_deliveries=240000_cells=61440000_plasticity=1989150000
config_hash=c1-d38d7644d8afc84b seed=16390668728421302609 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=84750000.000000 note=wall_secs=0.1243_peak_rss=8503296_spikes=0_deliveries=240000_cells=61440000_plasticity=23070000
config_hash=c1-d38d7644d8afc84b seed=16390668728421302609 condition=dense-matched accuracy=0.910000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=8079490.975942 note=wall_secs=0.0318_peak_rss=11206656_spikes=3491_deliveries=245188_cells=247431_plasticity=6856227
config_hash=c1-d38d7644d8afc84b seed=9344921515947343204 condition=local-assembly accuracy=0.740000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=7594452.604829 note=wall_secs=0.0242_peak_rss=10584064_spikes=3041_deliveries=105317_cells=107487_plasticity=5404050
config_hash=c1-d38d7644d8afc84b seed=9344921515947343204 condition=dense-local accuracy=0.940000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=25182999.000044 note=wall_secs=0.0843_peak_rss=13467648_spikes=3504_deliveries=462000_cells=464243_plasticity=22742272
config_hash=c1-d38d7644d8afc84b seed=9344921515947343204 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=2050830000.000000 note=wall_secs=15.5273_peak_rss=9650176_spikes=0_deliveries=240000_cells=61440000_plasticity=1989150000
config_hash=c1-d38d7644d8afc84b seed=9344921515947343204 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=84750000.000000 note=wall_secs=0.1238_peak_rss=8503296_spikes=0_deliveries=240000_cells=61440000_plasticity=23070000
config_hash=c1-d38d7644d8afc84b seed=9344921515947343204 condition=dense-matched accuracy=0.900000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=8179597.994463 note=wall_secs=0.0320_peak_rss=11223040_spikes=3492_deliveries=245379_cells=247622_plasticity=6865145
config_hash=c1-d38d7644d8afc84b seed=2298611250440747387 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=13128998.000000 note=wall_secs=0.0265_peak_rss=10403840_spikes=2977_deliveries=105690_cells=107907_plasticity=6347925
config_hash=c1-d38d7644d8afc84b seed=2298611250440747387 condition=dense-local accuracy=0.940000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=25182999.000044 note=wall_secs=0.0829_peak_rss=13451264_spikes=3504_deliveries=462000_cells=464243_plasticity=22742272
config_hash=c1-d38d7644d8afc84b seed=2298611250440747387 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=2050830000.000000 note=wall_secs=15.5354_peak_rss=9666560_spikes=0_deliveries=240000_cells=61440000_plasticity=1989150000
config_hash=c1-d38d7644d8afc84b seed=2298611250440747387 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=84750000.000000 note=wall_secs=0.1251_peak_rss=8519680_spikes=0_deliveries=240000_cells=61440000_plasticity=23070000
config_hash=c1-d38d7644d8afc84b seed=2298611250440747387 condition=dense-matched accuracy=0.740000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=9737143.117756 note=wall_secs=0.0311_peak_rss=11157504_spikes=3784_deliveries=245546_cells=247781_plasticity=6708375
config_hash=c1-d38d7644d8afc84b seed=13699608180395816334 condition=local-assembly accuracy=0.740000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=7430641.796130 note=wall_secs=0.0247_peak_rss=10665984_spikes=3743_deliveries=105724_cells=107888_plasticity=5281320
config_hash=c1-d38d7644d8afc84b seed=13699608180395816334 condition=dense-local accuracy=0.940000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=25182999.000044 note=wall_secs=0.0830_peak_rss=13467648_spikes=3504_deliveries=462000_cells=464243_plasticity=22742272
config_hash=c1-d38d7644d8afc84b seed=13699608180395816334 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=2050830000.000000 note=wall_secs=15.5667_peak_rss=9666560_spikes=0_deliveries=240000_cells=61440000_plasticity=1989150000
config_hash=c1-d38d7644d8afc84b seed=13699608180395816334 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=84750000.000000 note=wall_secs=0.1252_peak_rss=8519680_spikes=0_deliveries=240000_cells=61440000_plasticity=23070000
config_hash=c1-d38d7644d8afc84b seed=13699608180395816334 condition=dense-matched accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=14224276.000000 note=wall_secs=0.0305_peak_rss=11206656_spikes=3846_deliveries=242203_cells=244434_plasticity=6621655
config_hash=c1-d38d7644d8afc84b seed=6653298481824903589 condition=local-assembly accuracy=0.500000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=12508300.000000 note=wall_secs=0.0259_peak_rss=10682368_spikes=3098_deliveries=105029_cells=107231_plasticity=6038792
config_hash=c1-d38d7644d8afc84b seed=6653298481824903589 condition=dense-local accuracy=0.940000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=25182999.000044 note=wall_secs=0.0868_peak_rss=14417920_spikes=3504_deliveries=462000_cells=464243_plasticity=22742272
config_hash=c1-d38d7644d8afc84b seed=6653298481824903589 condition=gradient-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=2050830000.000000 note=wall_secs=15.6992_peak_rss=9666560_spikes=0_deliveries=240000_cells=61440000_plasticity=1989150000
config_hash=c1-d38d7644d8afc84b seed=6653298481824903589 condition=eligibility-reference accuracy=1.000000 activity_sparsity=1.000000 activity-sparsity=1.000000 work_per_accuracy=84750000.000000 note=wall_secs=0.1236_peak_rss=8503296_spikes=0_deliveries=240000_cells=61440000_plasticity=23070000
config_hash=c1-d38d7644d8afc84b seed=6653298481824903589 condition=dense-matched accuracy=0.930000 activity_sparsity=0.015625 activity-sparsity=0.015625 work_per_accuracy=7903358.003732 note=wall_secs=0.0332_peak_rss=11468800_spikes=3504_deliveries=242874_cells=245117_plasticity=6858628
```
