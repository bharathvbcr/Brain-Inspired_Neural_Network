# Wave 20 verdicts

Coverage: **270 valid / 360 planned**, 0 invalid, 23 failures, 90 missing.

## H20-2 — is the comparison still one loss from unreportable?

| arm | valid of 32 |
|---|---:|
| `rec+alif` | 27 |
| `rec+alif+attn` | 29 |
| **usable pairs** | **24** |

**H20-2: MET** — 24 pairs against a floor of 24/32.

## H20-1 — does the recurrent substrate's larger gain survive?

| substrate | pairs | rate read-out | + attention | gain |
|---|---:|---:|---:|---:|
| `rec+alif` | 24 | 0.5187 | 0.7944 | **+0.2757** |
| `ff+fixed` | 32 | 0.7086 | 0.8275 | **+0.1189** |

**H20-1: MET** — difference of gains +0.1551 over 24 seed-paired comparisons, positive in 24/24 (bars: >= +0.03, >= 24/32).

## H20-3 — is survivorship shaping the gain?

Registered bar **ρ >= -0.30**, against a pilot of **-0.648** over the ten pairs that existed when this was written. The bar predicts the pilot is small-sample noise.

**H20-3: MET** — ρ = **-0.274** over 24 completing pairs.

## H20-4 — does the advantage survive headroom normalisation?

| substrate | base | headroom | gain | gain / headroom |
|---|---:|---:|---:|---:|
| `rec+alif` | 0.5187 | 0.4813 | +0.2757 | 0.5728 |
| `ff+fixed` | 0.7086 | 0.2914 | +0.1189 | 0.4080 |

**H20-4: MET** — ratio **1.404x** (bar: > 1.0x). §3.7's limit 1 computed 1.34x post-hoc; this is the registered measurement.

## Stability warnings

Reported, and **never voiding**. Peak gradient norm is the covariate H20-3 is measured on; voiding on it would decide the survivorship question by construction.

- `w20rec__rec-alif-attn__h128__e400__published-2ms__adjacent-sum-5__d32l4__ss0.4__s5170013`: peak gradient norm 4.399e+09 exceeds every cell in the recorded campaign (max 1.13e8)
- `w20rec__rec-alif-attn__h128__e400__published-2ms__adjacent-sum-5__d32l4__ss0.4__s5170014`: peak gradient norm 1.579e+11 exceeds every cell in the recorded campaign (max 1.13e8)
- `w20rec__rec-alif-attn__h128__e400__published-2ms__adjacent-sum-5__d32l4__ss0.4__s5170017`: peak gradient norm 1.827e+16 exceeds every cell in the recorded campaign (max 1.13e8)
- `w20rec__rec-alif-attn__h128__e400__published-2ms__adjacent-sum-5__d32l4__ss0.4__s5170019`: peak gradient norm 1.844e+32 exceeds every cell in the recorded campaign (max 1.13e8)
- `w20rec__rec-alif-attn__h128__e400__published-2ms__adjacent-sum-5__d32l4__ss0.4__s5170020`: peak gradient norm 7.618e+13 exceeds every cell in the recorded campaign (max 1.13e8)
- `w20rec__rec-alif-attn__h128__e400__published-2ms__adjacent-sum-5__d32l4__ss0.4__s5170023`: peak gradient norm 3.710e+30 exceeds every cell in the recorded campaign (max 1.13e8)
- `w20rec__rec-alif-attn__h128__e400__published-2ms__adjacent-sum-5__d32l4__ss0.4__s5170024`: peak gradient norm 4.584e+35 is within five orders of f32 overflow — the numerically marginal regime of AMENDMENT_2026-08-05; an accuracy from this cell says as much about the arithmetic as about the arm
- `w20rec__rec-alif-attn__h128__e400__published-2ms__adjacent-sum-5__d32l4__ss0.4__s5170026`: peak gradient norm 4.118e+16 exceeds every cell in the recorded campaign (max 1.13e8)
- `w20rec__rec-alif-attn__h128__e400__published-2ms__adjacent-sum-5__d32l4__ss0.4__s5170028`: peak gradient norm 2.657e+18 exceeds every cell in the recorded campaign (max 1.13e8)
- `w20rec__rec-alif-attn__h128__e400__published-2ms__adjacent-sum-5__d32l4__ss0.4__s5170029`: peak gradient norm 3.024e+16 exceeds every cell in the recorded campaign (max 1.13e8)
- `w20rec__rec-alif-attn__h128__e400__published-2ms__adjacent-sum-5__d32l4__ss0.4__s5170030`: peak gradient norm 5.517e+13 exceeds every cell in the recorded campaign (max 1.13e8)
- `w20rec__rec-alif-attn__h128__e400__published-2ms__adjacent-sum-5__d32l4__ss0.4__s5170031`: peak gradient norm 2.750e+09 exceeds every cell in the recorded campaign (max 1.13e8)
- `w20rec__rec-alif-attn__h128__e400__published-2ms__adjacent-sum-5__d32l4__ss0.4__s5170032`: peak gradient norm 6.026e+10 exceeds every cell in the recorded campaign (max 1.13e8)
- `w20rec__rec-alif__h128__e400__published-2ms__adjacent-sum-5__ss0.4__s5170014`: peak gradient norm 3.880e+22 exceeds every cell in the recorded campaign (max 1.13e8)
- `w20rec__rec-alif__h128__e400__published-2ms__adjacent-sum-5__ss0.4__s5170015`: peak gradient norm 6.193e+09 exceeds every cell in the recorded campaign (max 1.13e8)
- `w20rec__rec-alif__h128__e400__published-2ms__adjacent-sum-5__ss0.4__s5170016`: peak gradient norm 1.709e+18 exceeds every cell in the recorded campaign (max 1.13e8)
- `w20rec__rec-alif__h128__e400__published-2ms__adjacent-sum-5__ss0.4__s5170018`: peak gradient norm 1.225e+16 exceeds every cell in the recorded campaign (max 1.13e8)
- `w20rec__rec-alif__h128__e400__published-2ms__adjacent-sum-5__ss0.4__s5170019`: peak gradient norm 6.352e+09 exceeds every cell in the recorded campaign (max 1.13e8)
- `w20rec__rec-alif__h128__e400__published-2ms__adjacent-sum-5__ss0.4__s5170021`: peak gradient norm 8.187e+16 exceeds every cell in the recorded campaign (max 1.13e8)
- `w20rec__rec-alif__h128__e400__published-2ms__adjacent-sum-5__ss0.4__s5170022`: peak gradient norm 1.827e+16 exceeds every cell in the recorded campaign (max 1.13e8)
- `w20rec__rec-alif__h128__e400__published-2ms__adjacent-sum-5__ss0.4__s5170023`: peak gradient norm 1.482e+15 exceeds every cell in the recorded campaign (max 1.13e8)
- `w20rec__rec-alif__h128__e400__published-2ms__adjacent-sum-5__ss0.4__s5170024`: peak gradient norm 2.990e+17 exceeds every cell in the recorded campaign (max 1.13e8)
- `w20rec__rec-alif__h128__e400__published-2ms__adjacent-sum-5__ss0.4__s5170025`: peak gradient norm 7.081e+13 exceeds every cell in the recorded campaign (max 1.13e8)
- `w20rec__rec-alif__h128__e400__published-2ms__adjacent-sum-5__ss0.4__s5170026`: peak gradient norm 1.660e+13 exceeds every cell in the recorded campaign (max 1.13e8)
- `w20rec__rec-alif__h128__e400__published-2ms__adjacent-sum-5__ss0.4__s5170027`: peak gradient norm 2.990e+18 exceeds every cell in the recorded campaign (max 1.13e8)
- `w20rec__rec-alif__h128__e400__published-2ms__adjacent-sum-5__ss0.4__s5170028`: peak gradient norm 4.747e+17 exceeds every cell in the recorded campaign (max 1.13e8)
- `w20rec__rec-alif__h128__e400__published-2ms__adjacent-sum-5__ss0.4__s5170030`: peak gradient norm 1.649e+18 exceeds every cell in the recorded campaign (max 1.13e8)
- `w20rec__rec-alif__h128__e400__published-2ms__adjacent-sum-5__ss0.4__s5170031`: peak gradient norm 1.700e+11 exceeds every cell in the recorded campaign (max 1.13e8)
- `w20rec__rec-alif__h128__e400__published-2ms__adjacent-sum-5__ss0.4__s5170032`: peak gradient norm 2.521e+22 exceeds every cell in the recorded campaign (max 1.13e8)
