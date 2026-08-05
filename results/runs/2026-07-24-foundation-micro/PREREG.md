# Preregister — Foundation Microcircuit ~10⁶ synapses

Camp: `results/runs/2026-07-24-foundation-micro/`  
Hash family: `c1-micro-*` experiment prefix `c1-micro-foundation-*`  
Date preregistered: 2026-07-24

**Not Gate G2. Not overnight syn-matched-1e5 @ N=1e4. Not activity-scaled micro isolate. Not biology. Not Foundation R0 unlock.**

---

## Geometry (fixed; not αN)

| Knob | Value |
|---|---|
| `n_hidden` (N) | **10 000** |
| `max_fan_out` (fan) | **100** |
| `k_wta` (k) | **8 FIXED** (not αN / not `scaled_k_wta`) |
| Target measured nnz | **≈ 10⁶** (`FOUNDATION_MICRO_TARGET_NNZ`) |
| Wiring | capped (`p·N ≫ fan`) |
| `matched_budget_repeat` | `false` |
| SurrogateLif / dense C1 | **refused** (isolate `local-assembly` only) |
| Init rescale | `init_w_eff = init_w · √(45 / mean_hidden_fan_in)` |
| Readout gain | boost so `boost · mean_ro ≈ (1.15/0.15)·64`, clamped |

Distinct from:

- Overnight H1 Micro OP: N=10k **fan=10** → ~1.1×10⁵ nnz (**syn-matched-1e5**)
- Activity-scaled `c1-micro` isolate: N=10k fan=256 **k=100 (αN)** → ~2.57×10⁶ nnz

---

## Schedule

| Schedule | `n_seeds` | `n_train` / `n_test` | CLI |
|---|---:|---|---|
| Quick smoke | 2 | 16 / 12 | `--foundation-micro --quick --isolate-condition local-assembly` |
| Scientific | 20 (`c1_default`) | 80 / 40 | `--foundation-micro --isolate-condition local-assembly --seed S` |

Isolate CLI runs **one seed per invocation**; scientific = loop seeds from config seed list.

---

## Pass / Fail floors (engineering; not G2)

**Pass** iff all of:

1. Measured nnz ∈ **[800 000, 1 200 000]** (±20% of 10⁶)
2. Peak RSS **< 48 GB**
3. Wall **< 1200 s / seed**
4. `empty_winner_rate = 0`
5. Activity sparsity inside disclosed fixed-k band `[0.25·k/N, 4·k/N]`
6. No OOM / non-zero exit

**Fail** if any floor is violated.

Accuracy / work are **reported** but are **not** G2 PASS/FAIL signals and do **not** reopen `c1-118207fbc3eaba53`.

---

## Non-claims banner

1. Does **not** reopen Gate G2 (or remassage frozen G2 / v13–v24 / P4/P5/P9 hashes).
2. Does **not** claim biology or Foundation R0 unlock.
3. Does **not** equate cells with synapses — cite **measured nnz + fan + regime**.
4. Overnight syn-matched Micro OP is **not** this campaign.
