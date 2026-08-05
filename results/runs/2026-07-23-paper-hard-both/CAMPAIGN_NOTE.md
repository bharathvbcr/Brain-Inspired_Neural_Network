# BINN paper-hard-both campaign note

**Camp:** `results/runs/2026-07-23-paper-hard-both/`  
**Mode:** no-deferrals · audit + verify + break-it v20–v24 + package  
**Guardrails:** no reopen `c1-118207fbc3eaba53`; no remassage P4/P5/P9 / v13–v19; G2 bars fixed.

---

## A0 artifacts

- [`HARD_AUDIT.md`](../../HARD_AUDIT.md)
- [`CLAIM_AXIS.md`](../../CLAIM_AXIS.md)
- [`DIFF_CLOSURE.md`](../../DIFF_CLOSURE.md)

---

## Verify

Primary hash replays **bit-stable** — [`VERIFY_SUMMARY.md`](VERIFY_SUMMARY.md).

---

## Break-it scientific (n=20)

| Proto | Hash | Axis | Verdict | Local | Gate LCB | Chance LCB |
|---|---|---|---|---:|---:|---:|
| v20 | `c1-4db53e645405fae0` | Novel-CS | **DONE-FAIL** | 0.7325 | 0.2601 | 0.3321 |
| v21 | `c1-f975db8fb3e5d569` | Novel-CS | **DONE-FAIL** | 0.5025 | 0.0406 | 0.0122 |
| v22 | `c1-match-b46b23549b37d90a` | Integrity | **DONE-FAIL** | 0.5000 | 0.0000 | — |
| v23 | `c1-4bbaf4b24c2d1da2` | Integrity | **DONE-FAIL** | 0.6638 | 0.2370 | 0.2370 |
| v24 | `c1-840f820b7c07b512` | Novel-CS | **DONE-FAIL** | 0.6437 | 0.1380 | 0.1163 |

Stop rule honored: FAIL → no remassage.

---

## Closure

[`DIFF_CLOSURE.md`](../../DIFF_CLOSURE.md) — all D1–D22 cells `DONE-*` / `DISCLOSED` (no empty / deferred).

Metrics: [`PAPER_METRICS_FULL.md`](../../PAPER_METRICS_FULL.md) · Appendix: [`APPENDIX_POST_G2.md`](../../APPENDIX_POST_G2.md)
