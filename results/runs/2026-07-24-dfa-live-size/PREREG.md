# Preregister — H2 dfa-live width-transfer size protocol

Camp: `results/runs/2026-07-24-dfa-live-size/`  
Hash family: `c1-mac-probe-*` with `size_protocol=true` (experiment tag `-size-`)  
Date preregistered: 2026-07-24

**Not Gate G2. Not overnight quick H2 (n_test=12 inconclusive). Not frozen v20 remassage (`c1-4db53e645405fae0`). Not biology.**

---

## Geometry

| Knob | Value |
|---|---|
| N | **2000** |
| fan | **50** (`syn_matched_fan_out(2000)` → ~1e5 nnz) |
| k | **8** fixed |
| Init / readout rescale | on (same mac-probe formulas) |
| Isolate | `local-assembly` only (full C1 refused at N≥2k) |

---

## Arms (new hashes only)

| Arm | MacProbeMode | Role |
|---|---|---|
| pm1 | `Pm1` | Reference (±1 broadcast) |
| structured-fb | `StructuredFb` | Reference / transfer foil |
| dfa-live | `DfaLive` | Primary graded-DFA live transfer |

Scientific MacProbeConfig hashes (printed by CLI; do not remassage overnight quick H2 smokes):

- Resolve via `MacProbeConfig::dfa_live_size(mode, false)` / `--dfa-live-size --mac-mode …`

---

## Scientific schedule

| Knob | Value |
|---|---|
| `n_seeds` | **8** (`DFA_LIVE_SIZE_N_SEEDS`) |
| `n_train` / `n_test` | 80 / 40 (`c1_default`) |
| Confidence | z = **1.96** |
| Accuracy floor | **0.60** |
| Gap | per-seed `g_i = acc_arm(i) − acc_pm1(i)`; mean ± LCB |
| Gap clear | LCB **> 0** (strictly beats pm1) |

Quick (`--quick`) remains smoke-only and is **not** a verdict under this protocol.

---

## Verdict rule (preregistered)

For primary arm **dfa-live** (structured-fb reported the same way):

1. **Reject-floor** if mean accuracy **< 0.60**
2. Else **Reject-gap** if gap LCB vs pm1 **≤ 0**
3. Else **Accept**

No knob remassage after seeing data. If Reject-gap with floor cleared, disclose — do **not** claim PASS on overnight quick 0.75.

---

## Non-claims banner

1. Does **not** reopen Gate G2 or remassage frozen v20 / mac H2 overnight smokes.
2. Does **not** reinterpret G2 FAIL.
3. Size protocol is disclosed geometry — not a claim that DFA “works at scale” beyond this harness.
4. Not biology.
