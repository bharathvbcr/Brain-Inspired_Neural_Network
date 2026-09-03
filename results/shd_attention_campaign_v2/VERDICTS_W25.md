# Wave 25 — the mechanism where it was unmeasured

Registered: `PREREG_2026-09-02_THE_MECHANISM_WHERE_IT_IS_UNMEASURED.md`.
This analyser is the authority on every verdict below.

Pinned binary this contrast requires: `3afd4434431a75a26cc9d5fa46831341fc2f1dd0ef08dc308e18ca139b576364`.
`w25mec` cells indexed: **877** of 888 planned.

## H25-1 — is the recurrent read-out's advantage order-dependent?

The first manipulated recurrent cells in this campaign. Both halves are required: the shuffle must clear +0.03 in ≥ 9 of 12, and the displacement-matched null must fail to.

| substrate | condition | DiD | positive | pairs | requirement | verdict |
|---|---|---:|---:|---:|---|---|
| `rec+alif` | `bin-shuffled` | +0.0291 | 6/9 | 9 | > +0.03, ≥ 9/12 | **NOT MET** |
| `rec+alif` | `reversed` | — | — | 3 | — | not evaluable |

**H25-1: NOT EVALUABLE** — at least one contrast did not reach 9 seed-paired quadruples. A hypothesis that could not be tested is not one that passed.

## H25-2 — is the synchrony term a resolution effect?

Registered as a QUESTION. At `fixed-t250` wave 24 measured +0.1038; the band is ±0.03.

| rung | DiD(bin) | DiD(channel) | difference | pairs | answer |
|---|---:|---:|---:|---:|---|
| `fixed-t100` | +0.1323 | +0.2631 | +0.1308 | 12 | **synchrony also read** |
| `fixed-t500` | +0.1161 | +0.1736 | +0.0575 | 12 | **synchrony also read** |

## H25-3 — does the contrast depend on read-out depth?

H22-3 re-asked with the twins its analyser needed. Registered as a QUESTION; range bar 0.10.

| width | depth | DiD | `d32l4` twin | difference | verdict |
|---:|---|---:|---:|---:|---|
| 128 | `d32l2` | +0.1145 | +0.1208 | +0.0063 | within |
| 128 | `d64l4` | +0.1331 | +0.1208 | +0.0123 | within |
| 256 | `d32l1` | +0.0758 | +0.0862 | +0.0104 | within |
| 512 | `d32l1` | +0.0893 | +0.0968 | +0.0075 | within |
| 768 | `d32l2` | +0.0610 | +0.1881 | +0.1271 | **outside** |

**H25-3: NOT MET** — read-out depth is a scope axis for the mechanism claim, which nothing in the paper currently suggests.

## H25-4 — does the contrast survive e100 across the design space?

The H22-1 bar unchanged: DiD > +0.03, positive in ≥ 9 of 12.

| point | read-out | DiD | positive | pairs | verdict |
|---|---|---:|---:|---:|---|
| h128 / `fixed-t100` / `adjacent-sum-5` | `d32l4` | +0.1144 | 12/12 | 12 | **MET** |
| h128 / `fixed-t250` / `adjacent-sum-5` | `d32l4` | +0.0992 | 12/12 | 12 | **MET** |
| h128 / `fixed-t500` / `adjacent-sum-5` | `d32l4` | +0.1112 | 12/12 | 12 | **MET** |
| h128 / `published-10ms` / `adjacent-sum-5` | `d32l4` | +0.0916 | 12/12 | 12 | **MET** |
| h128 / `published-2ms` / `adjacent-sum-5` | `d32l1` | +0.0995 | 12/12 | 12 | **MET** |
| h128 / `published-2ms` / `adjacent-sum-5` | `d32l4` | +0.1260 | 12/12 | 12 | **MET** |
| h128 / `published-2ms` / `adjacent-sum-5` | `d64l4` | +0.1381 | 12/12 | 12 | **MET** |
| h128 / `published-2ms` / `channels-700` | `d32l1` | +0.1140 | 12/12 | 12 | **MET** |
| h128 / `published-2ms` / `channels-700` | `d32l4` | +0.1255 | 12/12 | 12 | **MET** |
| h256 / `published-2ms` / `adjacent-sum-5` | `d32l1` | +0.0830 | 12/12 | 12 | **MET** |
| h256 / `published-2ms` / `adjacent-sum-5` | `d32l4` | +0.0998 | 12/12 | 12 | **MET** |
| h384 / `published-2ms` / `adjacent-sum-5` | `d32l4` | +0.0888 | 12/12 | 12 | **MET** |
| h512 / `published-2ms` / `adjacent-sum-5` | `d32l1` | +0.0733 | 12/12 | 12 | **MET** |
| h512 / `published-2ms` / `adjacent-sum-5` | `d32l4` | +0.0864 | 12/12 | 12 | **MET** |
| h768 / `published-2ms` / `adjacent-sum-5` | `d32l2` | +0.0592 | 12/12 | 12 | **MET** |
| h768 / `published-2ms` / `adjacent-sum-5` | `d32l4` | +0.0711 | 12/12 | 12 | **MET** |
| h1024 / `published-2ms` / `adjacent-sum-5` | `d32l2` | +0.0469 | 11/12 | 12 | **MET** |
| h1024 / `published-2ms` / `adjacent-sum-5` | `d32l3` | +0.0502 | 12/12 | 12 | **MET** |
| h1024 / `published-2ms` / `adjacent-sum-5` | `d32l4` | +0.0628 | 12/12 | 12 | **MET** |

**H25-4: MET** — the budget scope limit is retired rather than narrowed.

## Provenance — which wave supplied each arm

| group | arm | source wave | found |
|---|---|---|---:|
| H25-1 | every arm | `w25mec` | 6 |
| H25-2 | `channel-shuffled` | `w25mec` | 4 |
| H25-2 | `intact` / `bin-shuffled` | `w22cov` | 8 |
| H25-3 | `d32l4` attention twins | `w25mec` | 8 |
| H25-4 | attention | `w25mec` | 38 |
| H25-4 | rate (two geometries) | `w24ord` | 4 |

## H25-5 — cells

`w25mec` cells that passed validity: **877** of 888. Voided by `cell_validity`: **1**.
**H25-5: NOT MET** — mechanical; it can fail only by cells failing to run.
