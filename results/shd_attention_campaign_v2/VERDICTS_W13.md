# Wave 13 — recurrent stability at the anchor budget

Prereg: `PREREG_2026-08-23_RECURRENT_STABILITY.md` §3. 48 cells, h128 / `published-2ms` / `adjacent-sum-5` / e400, same pinned binary.

A cell **completes** iff it was emitted *and* passes the validity gate, which includes `non_finite_events == 0`.

## Completion

| arm | surrogate scale | completed | voided | diverged |
|---|---:|---:|---:|---:|
| `rec+alif` | 0.4 | **11/12** | 0 | 1 |
| `rec+alif` | 1.0 | **8/12** | 0 | 4 |
| `rec+fixed` | 0.4 | **7/12** | 5 | 0 |
| `rec+fixed` | 1.0 | **5/12** | 5 | 2 |

## Registered verdicts

**R-1** *(primary)* some condition completes well enough to be measurable: best is `rec+alif` at scale 0.4 with **11/12**; bar 11/12 -> **SUPPORTED**
  - That condition is the operating point. The recurrent half of the factorial is registered separately and run there.

**R-2** adaptation is what destabilises: `rec+alif` 19/24 vs `rec+fixed` 12/24, difference **+7**; bar |Δ| ≥ 6 -> **SUPPORTED**

**R-3** the surrogate scale is a stability lever at this width: 0.4 18/24 vs 1.0 13/24, difference **+5**; bar |Δ| ≥ 6 -> **NOT SUPPORTED**

**R-4** *(diagnostic, no verdict)* how far from usable each condition is.

| arm | scale | peak ‖g‖ of completing cells | abort steps of diverged cells |
|---|---:|---|---|
| `rec+alif` | 0.4 | 1.30e+09 – 4.95e+32 | 8056 |
| `rec+alif` | 1.0 | 7.19e+07 – 1.02e+37 | 367, 3488, 3864, 7895 |
| `rec+fixed` | 0.4 | 2.24e+10 – 1.58e+23 | — |
| `rec+fixed` | 1.0 | 2.48e+14 – 3.80e+35 | 63, 428 |

## Accuracies of completing cells — **not a measurement**

Reported with each condition's completion count beside it. An arm that diverges more often can look better, because only its luckier trajectories survive to be scored; that is wave 11's recorded lesson and the reason no comparison between conditions with different completion rates is a result here.

| arm | scale | n completed | mean | min | max |
|---|---:|---:|---:|---:|---:|
| `rec+alif` | 0.4 | 11 | 0.5200 | 0.3913 | 0.6144 |
| `rec+alif` | 1.0 | 8 | 0.5288 | 0.4457 | 0.5910 |
| `rec+fixed` | 0.4 | 7 | 0.4972 | 0.3944 | 0.5994 |
| `rec+fixed` | 1.0 | 5 | 0.5448 | 0.4814 | 0.6179 |

**Stability notes: 30.** These are registered as expected and non-voiding — a recurrent arm above the 1e9 tier is the phenomenon under study, not a defect.

## Voided cells

- `w13rec__rec-fixed__ss0.4__s5170002`: saturated_fraction=0.055
- `w13rec__rec-fixed__ss0.4__s5170003`: saturated_fraction=0.258
- `w13rec__rec-fixed__ss0.4__s5170005`: saturated_fraction=0.141
- `w13rec__rec-fixed__ss0.4__s5170007`: saturated_fraction=0.469
- `w13rec__rec-fixed__ss0.4__s5170008`: saturated_fraction=0.117
- `w13rec__rec-fixed__ss1.0__s5170002`: saturated_fraction=0.250
- `w13rec__rec-fixed__ss1.0__s5170006`: saturated_fraction=0.320
- `w13rec__rec-fixed__ss1.0__s5170009`: saturated_fraction=0.523
- `w13rec__rec-fixed__ss1.0__s5170011`: saturated_fraction=0.133
- `w13rec__rec-fixed__ss1.0__s5170012`: saturated_fraction=0.141
