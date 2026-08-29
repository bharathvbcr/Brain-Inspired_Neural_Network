# Wave 21 — the temporal-order mechanism away from h128

Bars from `PREREG_2026-08-27_THE_MECHANISM_ACROSS_THE_DESIGN_SPACE.md`.

| operating point | quadruples | gain | DiD | positive |
|---|---:|---:|---:|---:|
| h128 / `published-2ms` / `adjacent-sum-5` | 32 | +0.1275 | +0.1205 | 32/32 |
| h256 / `published-2ms` / `adjacent-sum-5` | 12 | +0.0966 | +0.0862 | 12/12 |
| h384 / `published-2ms` / `adjacent-sum-5` | 12 | +0.0760 | +0.0767 | 12/12 |
| h512 / `published-2ms` / `adjacent-sum-5` | 12 | +0.0876 | +0.0968 | 12/12 |
| h768 / `published-2ms` / `adjacent-sum-5` | 12 | +0.0560 | +0.1881 | 12/12 |
| h1024 / `published-2ms` / `adjacent-sum-5` | 12 | -0.1318 | +0.1122 | 10/12 |
| h128 / `published-2ms` / `channels-700` | 12 | +0.1090 | +0.1122 | 12/12 |
| h128 / `published-10ms` / `adjacent-sum-5` | 12 | +0.1491 | +0.0959 | 12/12 |

## H21-1 — is the mechanism unique to h128?

- **h256: MET** — DiD +0.0862 against +0.03, positive in 12/12.
- **h384: MET** — DiD +0.0767 against +0.03, positive in 12/12.
- **h512: MET** — DiD +0.0968 against +0.03, positive in 12/12.

**H21-1: MET** — 3/3 widths clear both bars; the hypothesis requires all three.

## H21-2 — does the shuffle cost collapse where the gain inverts?

**H21-2: NOT MET** — DiD +0.1122 against a ceiling of +0.02. The bar is one-sided and a negative DiD satisfies it: the prediction is the absence of an order-dependent benefit, not its sign.

## H21-3 — does the shuffle cost track the gain?

**H21-3: NOT MET** — Spearman ρ -0.143 over 6 widths against a bar of +0.829, the n=6 one-tailed critical value at α=0.05. A ρ below it is not a trend and is not reported as one.

## H21-4 — does the mechanism survive a change of binning?

- **`published-2ms` / `channels-700`: MET** — DiD +0.1122 against +0.03, positive in 12/12.
- **`published-10ms` / `adjacent-sum-5`: MET** — DiD +0.0959 against +0.03, positive in 12/12.

**H21-4: MET** — the hypothesis requires both points.

---

Cross-machine Gate F FAILs macOS-vs-Linux on every node of this campaign by design. Every contrast above is between arms that ran on the same fleet from the same pinned binary.
