# Waves 18-19 verdicts

Coverage: **240 valid / 360 planned**, 0 invalid, 0 failures, 120 missing.

## H18-4 — does the fleet reproduce itself?

**H18-4: MET** — 12/12 duplicated cells byte-identical to `w15col` across every scientific field.

## H18-1 — is the optimum in read-out depth interior?

| depth | pairs | rate | attention | gain | positive | median epoch-mean norm |
|---|---:|---:|---:|---:|---:|---:|
| **L1** | 20 | 0.7392 | 0.7228 | -0.0159 | 3/20 | 0.025 |
| **L2** | 20 | 0.7392 | 0.7767 | +0.0405 | 20/20 | 0.658 |
| **L3** | 20 | 0.7392 | 0.7838 | +0.0371 | 18/20 | 1.347 |
| **L4** | 20 | 0.7392 | 0.6093 | -0.1318 | 3/20 | 34.469 |

**H18-1: MET** — the largest gain is at **L2** (+0.0405), clearing the nearer endpoint by +0.0563 (bar: interior depth, margin >= 0.02).

## H18-2 — is the collapse numerical?

**H18-2: NOT MET** — sick arms (norm >= 1.0) must have gain <= -0.1, healthy arms must not fall below -0.05.

- L3: norm 1.347 >= 1.0 but gain +0.0371 > -0.1

## H18-3 — is L2's advantage a seed artefact?

**H18-3: MET** — gain +0.0405, positive in 20/20 (bar: >= +0.02, >= 15/20).

## H19-1 — does the optimal depth fall as width rises?

| width | L2 gain | L4 gain | deeper wins? |
|---|---:|---:|---|
| h768 | +0.0419 | +0.0560 | yes |
| h1024 | +0.0405 | -0.1318 | no |

**H19-1: MET** — the ordering of L2 and L4 reverses between h768 and h1024.

## Stability warnings

Reported, and **never voiding** — gradient magnitude is the quantity under study in H18-2 and voiding on it would decide the question by definition.

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
