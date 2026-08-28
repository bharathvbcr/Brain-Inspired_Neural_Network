# Waves 15-17 verdicts

Coverage: **165 valid / 360 planned**, 0 invalid, 0 failures, 195 missing.

## H15-1 — does any lever restore the gain at h1024/d32/L4?

| lever | pairs | gain | positive | median epoch-mean norm | H15-1 | H15-2 |
|---|---:|---:|---:|---:|---|---|
| surrogate scale 0.5 | 12 | -0.2106 | 0/12 | 142.009 | **NOT MET** | **NOT MET** |
| surrogate scale 0.25 | 12 | -0.2565 | 0/12 | 151.391 | **NOT MET** | **NOT MET** |
| clip-grad-norm 1000.0 | 12 | -0.0904 | 1/12 | 11.660 | **NOT MET** | **NOT MET** |

**H15-1: NOT MET** (bar: gain >= +0.05, positive in >= 9/12).

## H15-3 — is L2 between L1 and L4 at h1024?

| depth | gain | source |
|---|---:|---|
| L1 | -0.0159 | archived `w3wid` |
| **L2** | **+0.0392** | this wave, 12 pairs, 12/12 positive |
| L4 | -0.1618 | archived `w8wid` |

**H15-3: NOT MET** — L2 must lie strictly inside (-0.1618, -0.0159) by at least 0.01.

## H15-4 — is the clip inert where it cannot bind?

**H15-4: MET** — 12/12 cells byte-identical to the archive across every scientific field. The clip is inert below its threshold, so the h1024 clipped arm measures clipping and not the flag.

## Clipped rate control (reporting only)

| | clipped | archived unclipped | difference |
|---|---:|---:|---:|
| mean accuracy (12 pairs) | 0.738590 | 0.738590 | +0.000000 |

**Inert: 12/12 byte-identical to the archive.** The clip cannot bind on the rate arm, so the clipped treatment's gain over the unclipped archive is not confounded by the control moving underneath it.

## H16 — the width ladder at d32/L4

| width | pairs | rate | attention | gain | positive |
|---|---:|---:|---:|---:|---:|
| h128 | 12 | 0.7062 | 0.8320 | +0.1258 | 12/12 |
| h256 | 12 | 0.7240 | 0.8206 | +0.0966 | 12/12 |
| h384 | 12 | 0.7336 | 0.8096 | +0.0760 | 12/12 |
| h512 | 12 | 0.7357 | 0.8233 | +0.0876 | 12/12 |
| h768 | 12 | 0.7386 | 0.7946 | +0.0560 | 11/12 |
| h1024 | 12 | 0.7386 | 0.5768 | -0.1618 | 1/12 |

**H16-1: NOT MET** — each rung below h1024 exceeds the next by >= 0.005 (gaps: +0.0292, +0.0206, -0.0116, +0.0316).
**H16-2: MET** — the drop into h1024 is 0.2178, against 3.0x the largest gap below it (0.0947).

## H17 — the headline and its mechanism at n=32

| n | rate | attention | gain | positive | >= 0.8 |
|---:|---:|---:|---:|---:|---:|
| 32 | 0.7057 | 0.8332 | +0.1275 | 32/32 | 32/32 |

**H17-1: MET** (bars: gain >= +0.05, positive >= 24/32, >= 24/32 at or above 0.8).

| arm | pairs | intact − shuffled | positive |
|---|---:|---:|---:|
| attention d32/L4 | 32 | +0.1347 | 32/32 |
| rate | 32 | +0.0142 | — |

**H17-2: MET** — shuffle cost >= +0.05, positive >= 24/32, and at least 5.0x the rate arm's (9.5x measured).

## Stability warnings

Reported, and **never voiding** — gradient magnitude is the quantity under study in H15-2 and voiding on it would decide the question by definition.

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
