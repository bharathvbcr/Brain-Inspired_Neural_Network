# BINN paper hard audit (A0)

**Campaign:** `results/runs/2026-07-23-paper-hard-both/`  
**Date:** 2026-07-23 · **Mode:** no-deferrals  
**Companion:** [`CLAIM_AXIS.md`](CLAIM_AXIS.md) · [`DIFF_CLOSURE.md`](DIFF_CLOSURE.md) · [`PUBLISHABLE_CLAIMS.md`](PUBLISHABLE_CLAIMS.md)

---

## Overclaim risks (must not ship)

| Risk | Why it bites | Mitigation |
|---|---|---|
| Biology / cortex / digital brain | Motifs exist in engine; nothing here proves cortex | Tag Motif arms; explicit non-claims |
| Impossibility of local learning | Scoped operationalized FAILs only | Language: “this coded pipeline / matched gate” |
| Matched PASS ⇒ live PASS | v13–v19 transfer FAIL | Table C + v20–v24 dual-gap reporting |
| Floor cleared = G2 PASS | v15/v17 clear ≥0.65 but gap LCB < 0.5 | Gate stays dual: acc **and** gap LCB |
| Coincidence proves locality | Broadcast-graded 0.9863 on DFA schedule | Disclose; cite XOR for locality flip |
| Remassage closed hashes | Reviewer trap | Never reopen `c1-118207fbc3eaba53`; no in-place P4/P5/P9 / v13–v19 knob massage |
| Fake multiplicity correction | Sequential exploratory family | Label v14–v24 exploratory; no family-wise theater |
| Appendix orphans reopen G2 | C2/C3/R/hybrid post-gate | Banner: does not reopen G2 |

---

## Dual-gap formulas (always report both on live arms)

**Gate gap (dense-local normalized; unchanged):**

\[
g_{\text{gate}} = \mathrm{clamp}_{[0,1]}\!\left(\frac{a_{\text{local}} - a_{\text{dense}}}{a_{\text{grad}} - a_{\text{dense}}}\right)
\quad\text{if } a_{\text{grad}}-a_{\text{dense}} \ge g_{\min\text{-ref}},\ \text{else } 0
\]

LCB: \(\bar g - z\sqrt{\mathrm{Var}(g)/n}\). **PASS requires** LCB > 0.5 **and** \(\bar a_{\text{local}} \ge 0.65\).

**Descriptive chance-normalized gap (reporting only; not a gate):**

\[
g_{\text{chance}} = \mathrm{clamp}_{[0,1]}\!\left(\frac{a_{\text{local}} - 0.5}{a_{\text{grad}} - 0.5}\right)
\quad\text{if } a_{\text{grad}} > 0.5,\ \text{else } 0
\]

Report mean + LCB of \(g_{\text{chance}}\) on live arms. **Do not** change G2 thresholds to use chance gap.

---

## Confounders (integrity ledger)

| ID | Confounder | Status |
|---|---|---|
| H1 | Sticky `ThreeFactor.last_spike` across trials | Still true on v2; fixed under `c1-iso*` (still FAIL) |
| H2 | Incomplete membrane reset vs C3 | Same as H1 |
| θ=∞ mute | Hidden θ → ∞ during integrate | Canonical / RFB / SFB; finite under spike* and **v23** |
| Force-fire / mute | Nominal sparsity = k/N under mute | **DISCLOSED** (D10) |
| Elig-ref ceiling | Eligibility reference ~1.0 | **DISCLOSED** (D11) — grad is ceiling |
| Gap formula | Gate uses dense-local denom | **DISCLOSED** (D9) + dual harvest |
| Seed bimodality | v15 LCB instability risk | Harvest min/max / frac≥0.65 (D22) |
| Soft→hard collapse | Hybrid temp ladder | Appendix D20; motivates **v21** |
| Undertraining | Matched 3F epochs/η | Closed by **v22** |
| Sign-truncated B | v15 `sign(Δw)` | Closed by **v24** continuous B |

---

## Guardrails (locked)

1. Never reopen hash `c1-118207fbc3eaba53` or remassage P4/P5/P9 / v13–v19 knobs in place.
2. New experiment ⇒ new protocol version + new hash.
3. G2 bars fixed: local ≥ 0.65, gap LCB > 0.5.
4. Do **not** claim biology / cortex / digital brain.
5. Tag every new note with `claim_axis` / `object_under_test` / `may_claim` / `must_not_claim`.
6. Live arms: report gate LCB **and** descriptive chance-normalized gap.
7. Camp dir: `binn/results/runs/2026-07-23-paper-hard-both/`.

---

## Prior scientific anchors (do not invent)

See [`PAPER_RESULTS_TABLE.md`](PAPER_RESULTS_TABLE.md) Tables A–C and [`CAMPAIGN_2026-07-23_CLAIM_FREEZE.md`](CAMPAIGN_2026-07-23_CLAIM_FREEZE.md).
