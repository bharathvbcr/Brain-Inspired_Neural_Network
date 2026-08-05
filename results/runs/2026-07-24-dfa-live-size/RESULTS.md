# H2 dfa-live width-transfer — size protocol results

Camp: [`PREREG.md`](PREREG.md) · machine Apple M5 Pro (64 GB) · 2026-07-24

**Not Gate G2. Not overnight quick H2 inconclusive (0.75 @ n_test=12). Not frozen v20 remassage. Not biology.**

---

## MacProbe fingerprints (new size-protocol only)

| Mode | MacProbeConfig | Runner config (seed 1) |
|---|---|---|
| pm1 (reference) | `c1-mac-probe-e14b8f80a366d9c0` | `c1-mac-probe-36d5afd8c61c75b5` |
| structured-fb | `c1-mac-probe-a180f42193693700` | `c1-10be9b7180235e1d` (proto-15 lineage @ size geometry) |
| **dfa-live (primary)** | `c1-mac-probe-e13053acc4e40c18` | `c1-f490afb73ea9be37` (proto-20 lineage @ size geometry) |

Distinct from overnight quick H2 smokes (`c1-mac-probe-680952d91d51f28b` etc.) and frozen v20 `c1-4db53e645405fae0`.

CLI:

```bash
./scripts/run_dfa_live_size.sh
./target/release/c1 --dfa-live-size --mac-mode dfa-live --isolate-condition local-assembly --seed S
```

---

## Protocol (preregistered)

| Knob | Value |
|---|---|
| Geometry | N=2000, fan=50 (syn-matched ~1e5 nnz), k=8 |
| Seeds | **8** scientific |
| Train / test | 80 / 40 |
| Acc floor | **0.60** |
| Gap | per-seed vs pm1; z=1.96 LCB; clear if LCB **> 0** |

---

## Arm table

| Arm | Hash | Verdict | Primary mean | Gap LCB | Source |
|---|---|---|---:|---:|---|
| pm1 | `c1-mac-probe-e14b8f80a366d9c0` | reference | 0.4969 | | [`RESULTS.md`](RESULTS.md) |
| structured-fb | `c1-mac-probe-a180f42193693700` | **Reject-floor** | 0.5000 | -0.3146 | [`RESULTS.md`](RESULTS.md) |
| dfa-live | `c1-mac-probe-e13053acc4e40c18` | **Accept** | 0.7781 | 0.1010 | [`RESULTS.md`](RESULTS.md) |

Per-seed dfa-live accuracies: 0.725, 1.000, 0.500, 0.725, 0.725, 0.550, 1.000, 1.000  
Per-seed pm1 accuracies: 0.500, 0.500, 0.625, 0.300, 0.500, 0.600, 0.525, 0.425  
Per-seed gaps (dfa−pm1): 0.225, 0.500, −0.125, 0.425, 0.225, −0.050, 0.475, 0.575

Geometry integrity (dfa-live seed 1): measured nnz=102 166, peak RSS≈10.2 MB, empty_winner=0.

### Primary verdict: **Accept** (dfa-live under disclosed size protocol)

structured-fb: **Reject-floor** (mean 0.50 < 0.60) — structured-B does not transfer under this size protocol.

---

## Non-claims

1. Does **not** reopen Gate G2 or remassage frozen v20 / overnight mac H2 smokes.
2. Accept is scoped to this preregistered N=2k syn-matched isolate harness — not a claim that graded DFA clears G2 live k-WTA at default geometry.
3. Overnight quick dfa-live 0.75 remains inconclusive as a standalone smoke; this note supersedes it for size-protocol reading.
4. Not biology.
