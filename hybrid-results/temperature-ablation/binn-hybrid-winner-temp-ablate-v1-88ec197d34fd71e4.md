# BINN-Hybrid T=2.0 winner-temperature ablation suite

- protocol: `binn-hybrid-winner-temp-ablate-v1-88ec197d34fd71e4`
- schedule: **PILOT / SMOKE**
- seeds: 2
- budgets: [48]
- learning rates: [0.035]
- temperatures: soft, 2.0000, hard
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
| `baseline-dense-d1-8-s4` | none | none | none |
| `width-s8-dense-d1-8` | none | none | none |
| `sparse-p0.50-d1-8-s4` | none | none | none |
| `shallow-dense-d1-3-s4` | none | none | none |
| `deep-dense-d5-8-s4` | none | none | none |

## Direct-terminal transfer (best depth per temperature × variant)

| variant | temperature | D* | mean@D* | L95@D* | budget | lr | mean nnz |
|---|---|---:|---:|---:|---:|---:|---:|
| `baseline-dense-d1-8-s4` | soft | none | — | — | — | — | 32 |
| `baseline-dense-d1-8-s4` | 2.0000 | none | — | — | — | — | 32 |
| `baseline-dense-d1-8-s4` | hard | none | — | — | — | — | 32 |
| `width-s8-dense-d1-8` | soft | none | — | — | — | — | 128 |
| `width-s8-dense-d1-8` | 2.0000 | none | — | — | — | — | 128 |
| `width-s8-dense-d1-8` | hard | none | — | — | — | — | 128 |
| `sparse-p0.50-d1-8-s4` | soft | none | — | — | — | — | 15 |
| `sparse-p0.50-d1-8-s4` | 2.0000 | none | — | — | — | — | 15 |
| `sparse-p0.50-d1-8-s4` | hard | none | — | — | — | — | 15 |
| `shallow-dense-d1-3-s4` | soft | none | — | — | — | — | 32 |
| `shallow-dense-d1-3-s4` | 2.0000 | none | — | — | — | — | 32 |
| `shallow-dense-d1-3-s4` | hard | none | — | — | — | — | 32 |
| `deep-dense-d5-8-s4` | soft | none | — | — | — | — | 32 |
| `deep-dense-d5-8-s4` | 2.0000 | none | — | — | — | — | 32 |
| `deep-dense-d5-8-s4` | hard | none | — | — | — | — | 32 |

## Limits

- Development seeds only; disjoint from frozen H0 / diagnostic families.
- Soft teacher remains a disclosed residual relaxation.
- Sparse / banded masks are fixed per seed (not learned).
- Cannot reopen H0 or authorize H1–H3 / Gate G2.
