# SHD attention campaign — what 528 cells established, and what they killed

> **SUPERSEDED 2026-08-22 by
> [`SUMMARY_2026-08-22_CAMPAIGN_AND_RECORD_REPAIR.md`](SUMMARY_2026-08-22_CAMPAIGN_AND_RECORD_REPAIR.md).**
> This document covers waves 1–7 only. It predates the wave-8 scope measurements,
> the wave-9 mechanism result, the Azure run, the cross-architecture
> reproducibility finding, and the ceiling-health hardening. Its numbers are still
> correct for the waves it covers; its *conclusions about scope* were written when
> the scope was assumed rather than measured. Read the successor.

**Cells:** 552 planned, 528 completed, 24 diverged, **0 voided** — the
`shd_attention_campaign_v1/manifest.json` figures; this line previously read
516/492, which excluded the 36 `r1cal` recalibration cells.
**Backend:** rust on Linux/aarch64, **one pinned binary `22d97c51`** throughout,
6 spot instances, ~$30.
**Seeds:** n=12 on every confirmatory contrast.
**Protocols:** five preregistrations and one amendment, each registered before the
cells it judges existed.

> **Cross-machine gate FAILed on every instance.** Absolute comparison against the
> macOS-recorded record (0.7032, 0.7378) is **unlicensed**. Every number below is a
> same-machine paired contrast, which is what the campaign was designed to make it.

---

## 1. The claim that survived

**A time-axis attention read-out reaches its accuracy within 10 epochs and not
within 5, and in ten epochs exceeds what the rate-only read-out reaches in four
hundred.**

| epochs | `ff+fixed` | `ff+fixed+attn` | gain |
|---:|---:|---:|---:|
| 5 | 0.4529 | 0.6756 | +0.2227 |
| 10 | 0.5336 | 0.7337 | +0.2002 |
| 20 | 0.5851 | 0.7479 | +0.1627 |
| 50 | 0.6484 | **0.7539** | +0.1055 |
| 400 | 0.7062 | 0.7483 | +0.0421 |
| 800 | 0.7164 | 0.7546 | +0.0382 |

Convergence is **bracketed** at `(5, 10]`: e5 reaches 90.3% of the arm's e400
accuracy and fails the 0.95 criterion; e10 reaches 98.1% and passes. A failing
rung below a passing one is what makes this a measurement rather than a floor —
wave 6 first reported "20 epochs", which was only where the ladder stopped.

**The mechanism is temporal order.** On bin-shuffled data the attention arm is
*worse* than its own control by −0.0492 in **12 of 12 seeds**, against a base-arm
order sensitivity of 0.0128 — a **7x** larger order-derived component than the
architecture could previously express.

**It is not capacity.** The h192 control carries more parameters (30,740 vs
29,332) and buys a third as much.

## 2. What the campaign killed

**The ceiling claim, for the configuration tested.** The pilot's **+0.1702**
became **+0.0421** at convergence — W1-1 **NOT SUPPORTED** against a registered
0.05, despite t=8.21 and all twelve seeds positive. The pilot measured a budget
effect. *Anyone citing +0.1702 as an architecture result is citing an artefact.*

**The recurrent rescue.** `rec+alif` and `rec+alif+attn` both produced **0 usable
cells of 12**, at two surrogate scales, with gradient clipping active — at half
the width where the record already found zero. Attention neither causes nor cures
it. W4-1 **NOT SUPPORTED**.

**Generality across width and geometry.** The gain **inverts by h1024**
(−0.0159) and is **not seed-consistent** at `channels-700` (−0.0309 to +0.0729).
W3-1 and W3-2 both **NOT SUPPORTED**. The surviving claim was measured entirely
at **h128 / `adjacent-sum-5`** and inherits that scope.

## 3. Verdicts, all n=12

| ID | question | verdict |
|---|---|---|
| W1-1 | converged gain ≥ 0.05 at d32/L1 | **NOT SUPPORTED** (+0.0421) |
| W1-2 | not capacity | SUPPORTED (+0.0301 vs h192) |
| W1-3 | temporal order | **SUPPORTED** (+0.0912) |
| W1-4 | converged | UNDERTRAINED — *defective, reads backwards* |
| W2-1 | monotone in `d_model` | SUPPORTED (3/3 steps) |
| W2-2 | depth (descriptive) | L1→L2 +0.0357, L2→L4 +0.0299 |
| W3-1 | width saturation | **NOT SUPPORTED** (−0.0173, inverts) |
| W3-2 | `channels-700` | **NOT SUPPORTED** (+0.0243, not unanimous) |
| W3-3 | resolution invariance | SUPPORTED — *interpretation does not follow* |
| W4-1 | attention stabilises recurrence | **NOT SUPPORTED** (0/12 usable) |
| W5-1 | converged at e400 | SUPPORTED — *disqualified by W5-2* |
| W5-2 | control converged | **NOT SUPPORTED** (+0.0102) |
| W5-3 | contrast stable across budget | SUPPORTED (moved 0.0039) |
| W6-1..3 | sample efficiency | **ALL SUPPORTED** |
| W7-1,3 | convergence bracketed | **SUPPORTED** |
| W7-2 | still converged at e5 | NOT SUPPORTED — *which is what brackets it* |

## 4. Three defects in my own protocol, on the record

1. **W2-1 was unevaluable as written** — "three of four steps" over four values
   that give three transitions. Amended **before any wave-2 cell was claimed**.
2. **W1-4's window scaled with the budget** — 2 epochs at e20, 40 at e400. Its
   bound also rejects the *known-converged* control. Disclosed **before any
   attention cell existed**; reported as registered anyway; contradicted four
   times over by direct measurement.
3. **Four thresholds anchored to macOS values this campaign may not compare
   against** (W1-4, W5-2, W3-1, W3-3). The paired hypotheses have no such problem.
   **Rule for successors: the criterion must be computable from cells measured in
   the same campaign.**

None was repaired after seeing its data. Re-anchoring a threshold post hoc is the
one repair never available.

## 5. What a paper can and cannot say

**Can:** sample efficiency, bracketed at (5, 10] epochs, mechanism established as
temporal order by a 12/12 shuffle inversion, not explained by capacity — **at
h128 and `adjacent-sum-5`**, with that scope in the claim rather than in a
limitations paragraph.

**Cannot:** anything about a higher ceiling at d32/L1; anything about the
macOS-recorded 0.7378; anything about local learning (these are BPTT references —
the frozen-attention local arm is built, registered, and blocked on instrument
calibration); anything at h1024 or `channels-700`.

**Completed (Waves 8 & 9, 2026-08-21):**
- **Wave 8 (headline):** `d32/L4` at `e400` reaches **0.8320** with **12 of 12 seeds ≥ 0.80**, budget-stable (|e400−e200|=0.0002), providing a **+0.1258** gain over `ff+fixed` (0.7062). See [`RESULT_2026-08-21_W8_HEADLINE_SCOPE_IS_MEASURED.md`](RESULT_2026-08-21_W8_HEADLINE_SCOPE_IS_MEASURED.md).
- **Wave 9 (mechanism):** Bin-shuffling causes a **+0.1337** drop (0.8320 → 0.6983) in **12 of 12 seeds** on the attention arm, vs **+0.0128** on the plain arm (10× factor). **96% of the readout advantage is lost without temporal order**. See [`RESULT_2026-08-21_W9_THE_MECHANISM_HOLDS_AT_THE_HEADLINE.md`](RESULT_2026-08-21_W9_THE_MECHANISM_HOLDS_AT_THE_HEADLINE.md).
- **Scope:** Scoped to h128 / `adjacent-sum-5` / `published-2ms`; width inversion at h1024 (−0.1618 at L4); resolution mechanism (S-5) refuted; Python mirror unmet (criterion 5, not calibration).

## 6. Artifacts

- `results/shd_attention_campaign_v1/` — 528 cells (waves 1–7 plus the 36 `r1cal` recalibration cells), 24 divergence logs, gates, plan, manifest sha256.
- `results/shd_attention_campaign_v2/` — 96 cells in the manifest, of which 72 are the wave-8 scope measurements (`w8*__`).
- `results/shd_attention_campaign_v2/`, cells named `w9dim__` / `w9shf__` — 24 cells, mechanism validation runs (Wave 9).
- Pinned binary: `22d97c51ab02`.
