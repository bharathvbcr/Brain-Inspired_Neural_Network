# Mac/Micro isolate probe (`c1-micro-*`)

**Frame:** Engineering capacity stress after Gate **G2 FAIL** — **not** a Foundation R0 unlock, **not** Gate G2, **no** dense SurrogateLif / multi-condition G2 verdict.

**Mode:** `--isolate-condition local-assembly` only · `matched_budget_repeat=false` · `WiringPrior::max_fan_out=256` · activity-scaled `k_wta` with target `k/N≈0.01` clamped to `[0.005, 0.03]`.

## Protocol

| Knob | Value |
|---|---|
| Hash family | `c1-micro-*` (separate from overnight fixed-k `c1-mac-probe-*`) |
| `n_hidden` | `{1000, 10000}` primary; `{100000}` optional stress |
| `max_fan_out` | `256` |
| `k_wta` | `scaled_k_wta(N)` → `{10, 100, 1000}` |
| Surrogate LIF | off |
| Full multi-condition C1 | refused (isolate-only) |

CLI:

```bash
cargo run --locked --release -p binn-lab --bin c1 -- \
  --micro --n-hidden 1000 --isolate-condition local-assembly
cargo run --locked --release -p binn-lab --bin c1 -- \
  --micro --n-hidden 10000 --isolate-condition local-assembly
```

## Results (this machine, 2026-07-24)

MacProbeConfig hash is the preset fingerprint printed by CLI; `config_hash` in JSON is the runner `Config::hash_string()`.

| N | schedule | MacProbe hash | runner `config_hash` | k | k/N | measured nnz | peak RSS | wall (s) | acc |
|---:|---|---|---|---:|---:|---:|---:|---:|---:|
| 1e3 | quick | `c1-micro-4154f9fd4ffcface` | `c1-micro-40483f6d3a40efcb` | 10 | 0.010 | 257 184 | 20.9 MB | 0.033 | 0.75 |
| 1e4 | quick | `c1-micro-6369e6a32702efba` | `c1-micro-713322a01c532f01` | 100 | 0.010 | 2 570 837 | 185.9 MB | 0.370 | 0.50 |
| 1e3 | scientific | `c1-micro-cf7debd9470f9bf1` | `c1-micro-14ca592eee64167c` | 10 | 0.010 | 257 184 | 21.0 MB | 0.124 | 0.50 |
| 1e4 | scientific | `c1-micro-bf6e78039b04665d` | `c1-micro-2e51a51ef7702ce6` | 100 | 0.010 | 2 570 837 | 186.9 MB | 1.425 | 0.45 |
| 1e5 | quick stress | `c1-micro-05c262bd953671b8` | `c1-micro-f66a713f50c2d6d1` | 1000 | 0.010 | 25 704 209 | **1.86 GB** | 3.943 | 0.33 |

Raw JSON lines: `results/runs/2026-07-24-micro-isolate/`.

### Probe max N on this machine

- **Comfortable primary ladder:** **N=10 000** (~187 MB RSS, ~1.4 s scientific isolate).
- **Optional stress completed:** **N=100 000** (~1.86 GB RSS, ~4 s quick). Feasible here; do not treat as default CI.

Activity sparsity matched `k/N=0.01` on every run (inside `[0.005, 0.03]`). Empty-winner rate = 0.

## Non-claims

- Accuracy is **not** a G2 PASS/FAIL signal.
- Does **not** reopen frozen G2 hashes or Foundation R0.
- Overnight syn-matched `c1-mac-probe-*` (fixed `k=8`) remains a separate size-science family.
