# BINN-Hybrid T=2.0 winner-temperature ablation suite

- protocol: `binn-hybrid-winner-temp-ablate-v1-4bba5113f678ca16`
- schedule: **SCIENTIFIC-ISH DEVELOPMENT ABLATION**
- seeds: 8
- budgets: [240, 960]
- learning rates: [0.035, 0.07]
- temperatures: soft, 2.0000, 1.0000, hard
- variants: 5
- all test weights unchanged: **true**
- scientific gate effect: **none**

> Fresh ablation hash family. Canonical H0 remains `HYBRID_NO_GO`; held-out seeds unused; H1–H3 stopped. This suite asks whether the soft→hard transfer collapse near **T=2.0** is stable under depth window, residual width, and connectivity changes. It is **not** a G2 rescue and does not remassage the one-task ladder `binn-hybrid-winner-temp-v1-*`.

## Ablation axes

| variant | n_states | depths | connectivity |
|---|---:|---|---|
| `baseline-dense-d1-8-s4` | 4 | 1–8 | dense |
| `width-s8-dense-d1-8` | 8 | 1–8 | dense |
| `sparse-p0.50-d1-8-s4` | 4 | 1–8 | sparse-bernoulli-0.50 |
| `shallow-dense-d1-3-s4` | 4 | 1–3 | dense |
| `deep-dense-d5-8-s4` | 4 | 5–8 | dense |

## Collapse summary (direct-terminal)

| variant | soft D* | T=2.0 D* | collapse temperature |
|---|---:|---:|---|
| `baseline-dense-d1-8-s4` | 4 | 2 | 2.0000 |
| `width-s8-dense-d1-8` | 5 | 1 | 2.0000 |
| `sparse-p0.50-d1-8-s4` | 2 | none | 2.0000 |
| `shallow-dense-d1-3-s4` | 3 | 2 | 2.0000 |
| `deep-dense-d5-8-s4` | none | none | none |

## Direct-terminal transfer (best depth per temperature × variant)

| variant | temperature | D* | mean@D* | L95@D* | budget | lr | mean nnz |
|---|---|---:|---:|---:|---:|---:|---:|
| `baseline-dense-d1-8-s4` | soft | 4 | 0.8784 | 0.7225 | 960 | 0.0700 | 32 |
| `baseline-dense-d1-8-s4` | 2.0000 | 2 | 0.9172 | 0.8221 | 960 | 0.0350 | 32 |
| `baseline-dense-d1-8-s4` | 1.0000 | 2 | 0.9409 | 0.8427 | 960 | 0.0700 | 32 |
| `baseline-dense-d1-8-s4` | hard | 1 | 1.0000 | 1.0000 | 960 | 0.0700 | 32 |
| `width-s8-dense-d1-8` | soft | 5 | 1.0000 | 1.0000 | 960 | 0.0350 | 128 |
| `width-s8-dense-d1-8` | 2.0000 | 1 | 1.0000 | 1.0000 | 960 | 0.0700 | 128 |
| `width-s8-dense-d1-8` | 1.0000 | 1 | 1.0000 | 1.0000 | 960 | 0.0700 | 128 |
| `width-s8-dense-d1-8` | hard | 1 | 1.0000 | 1.0000 | 960 | 0.0700 | 128 |
| `sparse-p0.50-d1-8-s4` | soft | 2 | 0.7741 | 0.6553 | 960 | 0.0700 | 16 |
| `sparse-p0.50-d1-8-s4` | 2.0000 | none | — | — | — | — | 16 |
| `sparse-p0.50-d1-8-s4` | 1.0000 | none | — | — | — | — | 16 |
| `sparse-p0.50-d1-8-s4` | hard | none | — | — | — | — | 16 |
| `shallow-dense-d1-3-s4` | soft | 3 | 0.8862 | 0.7956 | 960 | 0.0700 | 32 |
| `shallow-dense-d1-3-s4` | 2.0000 | 2 | 0.9172 | 0.8221 | 960 | 0.0350 | 32 |
| `shallow-dense-d1-3-s4` | 1.0000 | 2 | 0.9409 | 0.8427 | 960 | 0.0700 | 32 |
| `shallow-dense-d1-3-s4` | hard | 1 | 1.0000 | 1.0000 | 960 | 0.0700 | 32 |
| `deep-dense-d5-8-s4` | soft | none | — | — | — | — | 32 |
| `deep-dense-d5-8-s4` | 2.0000 | none | — | — | — | — | 32 |
| `deep-dense-d5-8-s4` | 1.0000 | none | — | — | — | — | 32 |
| `deep-dense-d5-8-s4` | hard | none | — | — | — | — | 32 |

## Limits

- Development seeds only; disjoint from frozen H0 / diagnostic families.
- Soft teacher remains a disclosed residual relaxation.
- Sparse / banded masks are fixed per seed (not learned).
- Cannot reopen H0 or authorize H1–H3 / Gate G2.
