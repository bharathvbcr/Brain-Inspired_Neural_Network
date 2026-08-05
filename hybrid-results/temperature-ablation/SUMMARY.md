# T=2.0 winner-temperature ablation — summary

**Scientific-ish hash:** `binn-hybrid-winner-temp-ablate-v1-4bba5113f678ca16`  
**Smoke hash:** `binn-hybrid-winner-temp-ablate-v1-88ec197d34fd71e4`

Fresh family — does **not** remassage `binn-hybrid-winner-temp-v1-*` or reopen H0 / G2.

## Axes (all three)

1. **Depth schedule:** full 1–8, shallow 1–3, deep 5–8  
2. **Width / area size:** `n_states ∈ {4, 8}`  
3. **Connectivity:** dense vs Bernoulli `p=0.50`

## Collapse at T=2.0 (direct-terminal, scientific-ish)

| variant | soft D* | T=2.0 D* | collapse |
|---|---:|---:|---|
| baseline dense s4 d1–8 | 4 | 2 | **2.0** |
| width s8 dense d1–8 | 5 | 1 | **2.0** |
| sparse p=0.50 s4 d1–8 | 2 | none | **2.0** |
| shallow dense d1–3 | 3 | 2 | **2.0** |
| deep dense d5–8 | none | none | none (no soft D* on this grid) |

**Takeaway:** On variants that achieve a soft-endpoint D*, transfer collapse remains at **T=2.0** under width and connectivity changes; sparse connectivity loses tempered D* entirely; deep-only windows fail to clear the soft floor under this development budget. Not a G2 rescue.

Artifacts: `*-4bba5113f678ca16.md` / `-sweep.csv` (scientific), `*-88ec197d34fd71e4.*` (smoke).

```bash
cargo run --locked --release -p binn-hybrid-lab --bin hybrid-temperature-ablation -- \
  --quick --out-dir hybrid-results/temperature-ablation
cargo run --locked --release -p binn-hybrid-lab --bin hybrid-temperature-ablation -- \
  --out-dir hybrid-results/temperature-ablation
```
