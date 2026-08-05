# Foundation Microcircuit results — ~10⁶ synapses

Camp: [`PREREG.md`](PREREG.md) · machine Apple M5 Pro (64 GB) · 2026-07-24

**Not Gate G2. Not overnight syn-matched-1e5 @ N=1e4. Not biology. Not Foundation R0 unlock.**

---

## Hashes

| Schedule | MacProbeConfig hash | Runner `config_hash` (seed 1) |
|---|---|---|
| Quick | `c1-micro-a90bc3044c3703dd` | `c1-micro-8af906eae1287b28` |
| Scientific | `c1-micro-40ac0c7725a689a0` | `c1-micro-61713f700782adcf` |

CLI:

```bash
./scripts/run_foundation_micro.sh          # scientific 20 seeds
./scripts/run_foundation_micro.sh --quick
# or:
./target/release/c1 --foundation-micro --isolate-condition local-assembly --seed S
```

Full dense+SurrogateLif C1 **refused** without `--isolate-condition local-assembly`.

---

## Geometry (preregistered)

| Knob | Value |
|---|---|
| N | 10 000 |
| fan | 100 |
| k | **8 FIXED** (not αN) |
| Target nnz | ≈ 10⁶ |
| Pass band | [800 000, 1 200 000] |
| Regime | capped |

---

## Scientific results (n_seeds=20)

| Metric | Value |
|---|---|
| Measured nnz | **1 010 069 … 1 010 359** (all in Pass band; ≈ **1.010×10⁶**) |
| Mean accuracy | 0.5113 (min 0.375 / max 0.725) — **not** a G2 signal |
| Peak RSS (max) | 72.2 MB ≪ 48 GB |
| Wall / seed (max) | 0.565 s ≪ 1200 s |
| Wall sum | ≈ 10.0 s |
| Activity sparsity | 0.0008 (= k/N; in disclosed fixed-k band) |
| Empty-winner rate | 0 |
| Mean work/accuracy | ≈ 2.66×10⁸ |

Raw JSON: `scientific-seed{1..20}.json` · log: `run.log`

### Verdict: **Pass** (engineering floors)

All Pass/Fail floors from prereg cleared. Accuracy remains near chance under live ±1 isolate — capacity stress only.

---

## Contrast (do not conflate)

| Label | N | fan | k | nnz scale |
|---|---:|---:|---:|---|
| Overnight Micro OP | 10 000 | 10 | 8 | ~1.1×10⁵ (**not** this) |
| Activity-scaled micro isolate | 10 000 | 256 | 100 (αN) | ~2.57×10⁶ |
| **Foundation Micro (this)** | 10 000 | 100 | **8 fixed** | **~1.01×10⁶** |

---

## Non-claims

1. Not G2 PASS/FAIL reinterpretation; frozen `c1-118207fbc3eaba53` / v13–v24 / P4/P5/P9 untouched.
2. Not biology; not Foundation R0 unlock.
3. Cite **measured nnz + fan + regime**, never cells alone.
