# BINN — camera-ready results table

> **CITATION WARNING (added 2026-08-07).** This document cites the
> `track-b-rescue` **v130** row (`1.0000`, gap LCB `0.9988`, PASS matched) as a
> matched-substrate result. That report is stale: the source is **v131**, and the
> 130→131 bump is precisely the clamp-and-separation-gate fix for the defect the
> row exhibits. Under current code the arm **cannot be reported as PASS**. Do not
> cite it until `track-b-rescue` has been re-run. The DFA
> (`c1-dfa-c8c4fe0899908b84`) and RL (`c1-rl-42eddc9c801308e9`) matched PASSes are
> **not** affected by this defect; they ran through the clamped `runner.rs` path.
> See `AUDIT_2026-08-07_JULY_CAMPAIGN_SCORING_PATH.md` and
> `TODO_2026-08-07_OPEN_WORK.md` §1.
> **RESOLVED 2026-08-19.** The re-run landed. At v131 the arm reports
> **INVALID_HARNESS**, not PASS: the ceiling-inverted warning fires on 3 of 20
> learned-FB seeds and the code refuses to emit a PASS while it is present.
> The v130 PASS is **withdrawn**. See
> `RESULT_2026-08-19_TRACK_B_V130_PASS_WITHDRAWN.md` and
> `track_b_results_v131.md`.



**Authority:** on-disk notes only. Every row cites a file under `binn/results/`.  
**Do not invent.** Quick/PILOT hashes are excluded from this sheet.

Companion: [`PUBLISHABLE_CLAIMS.md`](PUBLISHABLE_CLAIMS.md) · [`PAPER_SKELETON.md`](PAPER_SKELETON.md) · [`CAMPAIGN_2026-07-23_CLAIM_FREEZE.md`](CAMPAIGN_2026-07-23_CLAIM_FREEZE.md) · **hash-replay verify:** [`runs/2026-07-23-paper-hard-both/VERIFY_SUMMARY.md`](runs/2026-07-23-paper-hard-both/VERIFY_SUMMARY.md) (bit-stable) · **full metrics:** [`PAPER_METRICS_FULL.md`](PAPER_METRICS_FULL.md) · **closure:** [`DIFF_CLOSURE.md`](DIFF_CLOSURE.md)

---

## Table A — Matched dense-LIF (primary)

| Arm | Hash | Verdict | Primary mean | Contrast / ceiling | Gap LCB | Source |
|---|---|---|---:|---:|---:|---|
| Broadcast ±1 three-factor (v4) | `c1-match-5dc6822e71229e9e` | **FAIL** | 0.5000 | grad 0.8963 | **0.0000** | [`c1_match.md`](c1_match.md) |
| DFA graded×fixed-B (v5) | `c1-dfa-c8c4fe0899908b84` | **PASS** | 0.9387 | grad 0.8963; broadcast-graded **0.9863** | **0.6894** | [`c1_dfa.md`](c1_dfa.md) |
| RL `rl_reinforce_fb` (v12) | `c1-rl-42eddc9c801308e9` | **PASS** | 0.9200 | graded 0.5250; flat 0.5113; grad 0.8887 | **0.6846** | [`c1_rl.md`](c1_rl.md) |
| **RL Online Learned `B_i` (v130)** | `track-b-rescue` | **WITHDRAWN** | — | — | — | [`RESULT_2026-08-19_TRACK_B_V130_PASS_WITHDRAWN.md`](RESULT_2026-08-19_TRACK_B_V130_PASS_WITHDRAWN.md) · v131 `INVALID_HARNESS` |
| Discrete EventProp-style (v28) | `c1-eventprop-5bb083d5e88d0ad2` | **FAIL** | 0.5000 | SuperSpike **0.9150** | **0.0000** | [`c1_eventprop.md`](c1_eventprop.md) |
| RL graded primary (v11, archived) | `c1-rl-ef504db58916720d` | **FAIL** | 0.5900 | — | 0.0182 | [`c1_rl_v11_graded_primary.md`](c1_rl_v11_graded_primary.md) |

**Paper language (required MUST):** Lead negative is **broadcast ±1 three-factor** (±1 × surrogate eligibility), not “any broadcast rule” and not bare “broadcast credit topology.” On the DFA schedule, **broadcast-graded** also learns coincidence (**0.9863**) — disclose in text and in Figure M; do **not** use coincidence alone to claim “locality is required.” Locality flip evidence is **XOR** (Table D). Falsifier: matched ±1 clearing gap LCB under the same forward overturns the lead claim. Discrete EventProp H2H is **FAIL** (`c1-eventprop-5bb083d5e88d0ad2`); disclose discrete ≠ continuous Wunderlich–Pehle. SuperSpike is the matched ceiling. v131 is matched-only (not live transfer). **A6 ceiling health caveat:** The canonical 80-epoch schedule undertrains the gradient reference (0.8963 / 0.9013 at e80, climbing to 1.0000 at e640; `RESULT_2026-08-19_A6_CEILING_HEALTH.md`); gap closed at e80 is a statement of *learning speed*.

**Gate:** gap LCB > 0.5 and primary mean ≥ 0.65; gradient mean ≥ 0.65 for harness validity.

---

## Table B — Engine C1 / Gate G2 (secondary, caveated)

| Arm | Hash | Verdict | Local | Gap LCB | Source |
|---|---|---|---:|---:|---|
| Canonical C1 (v2) | `c1-118207fbc3eaba53` | **FAIL** | 0.4912 | −0.0048 | [`c1_g2.md`](c1_g2.md), [`U-NEG_protocol_v2.md`](U-NEG_protocol_v2.md) |
| Trial isolation (v5) | `c1-8ec031907a3426d0` | **FAIL** | 0.5188 | (gap mean 0.2109) | [`c1_iso.md`](c1_iso.md) |
| Capacity sensitivity (v3) | `c1-d38d7644d8afc84b` | **FAIL** | **0.6775** (floor ✓) | **0.0000** | [`c1_sens_capacity_full.md`](c1_sens_capacity_full.md) |
| Temporal-PC (v3) | `c1-a49deeaedb495a09` | **FAIL** | 0.5263 | (gap mean 0.0947) | [`c1_sens_temporal_pc_full.md`](c1_sens_temporal_pc_full.md) |

**Must disclose in appendix:** H1 sticky `last_spike`, H2 partial membrane reset, θ=∞ mute, `project` unused on v2.

---

## Table C — Live REINFORCE transfer + gap-close

| Protocol | Hash | Verdict | Local | Gap LCB | Source |
|---|---|---|---:|---:|---|
| v13 live RFB | `c1-660401d74db3c88d` | **FAIL** | 0.4900 | 0.0737 | [`c1_rfb.md`](c1_rfb.md) |
| v14 epoch | `c1-714c115e14a3eeed` | **FAIL** | 0.4838 | −0.0100 | [`c1_rfb_em.md`](c1_rfb_em.md) |
| v15 structured B | `c1-493ddd56f8714fb6` | **FAIL** | **0.7262** | 0.2567 | [`c1_sfb.md`](c1_sfb.md) |
| v16 structured×epoch | `c1-677df7f7cbe4f8ec` | **FAIL** | 0.5200 | 0.0844 | [`c1_sfb_em.md`](c1_sfb_em.md) |
| v17 structured×capacity | `c1-983ee5303c00b147` | **FAIL** | **0.6825** | **0.3127** | [`c1_sfb_cap.md`](c1_sfb_cap.md) |
| v18 elig×REINFORCE | `c1-c7d2c86a2b1927f6` | **FAIL** | **0.7125** | 0.2351 | [`c1_elig_rfb.md`](c1_elig_rfb.md) |
| v19 structured×teach | `c1-dfab4a7ec19f17c2` | **FAIL** | **0.6700** | 0.2238 | [`c1_sfb_teach.md`](c1_sfb_teach.md) |
| **v20** live DFA | `c1-4db53e645405fae0` | **FAIL** | **0.7325** | 0.2601 | [`c1_dfa_live.md`](c1_dfa_live.md) · chance LCB 0.3321 |
| **v21** soft-WTA×SFB | `c1-f975db8fb3e5d569` | **FAIL** | 0.5025 | 0.0406 | [`c1_sfb_soft.md`](c1_sfb_soft.md) |
| **v22** match 4×ep | `c1-match-b46b23549b37d90a` | **FAIL** | 0.5000 | 0.0000 | [`c1_match_ep4.md`](c1_match_ep4.md) |
| **v23** finite-θ SFB | `c1-4bbaf4b24c2d1da2` | **FAIL** | **0.6638** | 0.2370 | [`c1_sfb_finth.md`](c1_sfb_finth.md) |
| **v24** continuous B | `c1-840f820b7c07b512` | **FAIL** | 0.6437 | 0.1380 | [`c1_sfb_cont.md`](c1_sfb_cont.md) |
| P4 spiking true-DFA | `c1x-dfa-spike-true-dfa-a911e793e590b0ed` | **FAIL** | 0.6513 | 0.0733 | [`credit_dfa_spike.md`](credit_dfa_spike.md) |

**Reading:** matched RL/DFA PASS do **not** transfer to live k-WTA. Structured `B` clears accuracy floor; best prior gap LCB is v17 (0.3127) still < 0.5. Break-it v20–v24 all FAIL under fixed G2; dual-gap harvest in [`PAPER_METRICS_FULL.md`](PAPER_METRICS_FULL.md). Floor cleared ≠ gate cleared. Closure: [`DIFF_CLOSURE.md`](DIFF_CLOSURE.md) (zero empty cells).

Packaging note: [`GAP_CLOSE_RFB_TRANSFER.md`](GAP_CLOSE_RFB_TRANSFER.md), [`MATCHED_ARCH_LIVE_REINFORCE.md`](MATCHED_ARCH_LIVE_REINFORCE.md), camp [`runs/2026-07-23-paper-hard-both/`](runs/2026-07-23-paper-hard-both/).

---

## Table D — NumPy task evidence (supporting)

| Exp | Init | broadcast / err_broadcast | DFA | gradient | rl_fb | Source |
|---|---|---:|---:|---:|---:|---|
| `xor_thresh` (1-layer) | strong | **0.5008** | **0.8267** | 0.7733 | — | [`deep_xor_thresh.json`](deep_xor_thresh.json) |
| `depth_locality` (2-layer) | mid | **0.8158** | 0.8250 | 0.8308 | 0.8033 | [`deep_depth_locality_mid.json`](deep_depth_locality_mid.json) |

**Cite XOR as locality flip.** Do **not** claim depth locality flip (broadcast also solves mid-init depth).

---

## Table E — Methods footnotes (optional)

| Item | Hash / ID | Number | Source |
|---|---|---|---|
| True e-prop (σ′×pre) | `c1x-eprop-true-…0e2aeb90d68ac5f9` | true-surrogate 0.7125 | [`credit_eprop_true.md`](credit_eprop_true.md) |
| AC `project` on C1 | `c1-project*` | G2 **FAIL** | [`c1_project.md`](c1_project.md) |
| Natural spike / spike-s | `c1-spike*` / `c1-spike-s*` | **INVALID_HARNESS** (PC) | [`c1_spike.md`](c1_spike.md), [`c1_spike_s.md`](c1_spike_s.md) |

---

## Table F — SHD Attention Read-out & Mechanism (Waves 1–9)

| Configuration | Arm | Budget | Intact accuracy | Bin-shuffled | Mechanism effect (intact − shf) | Source |
|---|---|---|---:|---:|---:|---|
| **d32/L4 (headline)** | `ff+fixed+attn` | e400 | **0.8320** (12/12 ≥ 0.80) | **0.6983** | **+0.1337** (12/12 seeds) | [`RESULT_2026-08-21_W8_HEADLINE_SCOPE_IS_MEASURED.md`](RESULT_2026-08-21_W8_HEADLINE_SCOPE_IS_MEASURED.md) · [`RESULT_2026-08-21_W9_THE_MECHANISM_HOLDS_AT_THE_HEADLINE.md`](RESULT_2026-08-21_W9_THE_MECHANISM_HOLDS_AT_THE_HEADLINE.md) |
| d32/L4 Control | `ff+fixed` | e400 | **0.7062** (0/12 ≥ 0.80) | **0.6934** | **+0.0128** | same |
| d32/L1 (Wave 1) | `ff+fixed+attn` | e400 | 0.7483 | 0.6442 | +0.1041 | [`RESULT_2026-08-19_W1_ATTENTION_AT_CONVERGENCE.md`](RESULT_2026-08-19_W1_ATTENTION_AT_CONVERGENCE.md) |
| d32/L4 @ e100 | `ff+fixed+attn` | e100 | 0.8209 (11/12 ≥ 0.80) | — | — | [`RESULT_2026-08-20_D32L4_CLEARS_THE_080_GATE.md`](RESULT_2026-08-20_D32L4_CLEARS_THE_080_GATE.md) |
| d32/L4 @ e10 (sample eff) | `ff+fixed+attn` | e10 | 0.7337 (vs e400: 98.1%) | — | — | [`SUMMARY_2026-08-20_ATTENTION_CAMPAIGN.md`](SUMMARY_2026-08-20_ATTENTION_CAMPAIGN.md) |
| `channels-700` geometry | `ff+fixed+attn` | e400 | 0.7864 (gain +0.1090) | — | — | Wave 8 |
| `published-10ms` geometry | `ff+fixed+attn` | e400 | **0.8225** (gain +0.1491) | — | — | Wave 8 |
| h1024 width scaling | `ff+fixed+attn` | e400 | **0.5768** (gain **−0.1618**) | — | — | Wave 8 (width inversion) |

**Mechanism summary:** Attention readout advantage collapses from **+0.1258** (intact) to **+0.0050** (shuffled); **96% of the readout benefit is contingent on temporal order**.

---

## Non-claims (print in paper)

1. Not biology / cortex / digital brain.  
2. Not Assembly Calculus PASS.  
3. Not impossibility of local learning in principle.  
4. Not live-engine rescue from matched DFA / RL PASS (v13–v24). Not v131 as live PASS (matched-only).  
5. Not “structured B / capacity / eligibility / soft-WTA / continuous-B PASS G2” (floor ≠ gate).  
6. Not coincidence-only proof that credit locality is required (broadcast-graded **0.9863** also learns coincidence; use XOR for locality).  
7. Not reopening `c1-118207fbc3eaba53` by threshold massage.  
8. Not undertraining as the matched ±1 three-factor FAIL cause (v22).  
9. Not EventProp “absent”; discrete H2H `c1-eventprop-5bb083d5e88d0ad2` is **FAIL** and ≠ continuous Wunderlich–Pehle.  
10. Not equating hybrid T=2.0 collapse with live v21 (T=1).  
11. Not treating appendix G3 / G4 / H0 as reopening G2.  
12. Not mixing overnight SHD p27 (20-way capped e-prop) with proto-135 SHD sweep (5-class) or protocol-29 full-corpus SuperSpike (`c1-shd-full-*`).
13. Not claiming online learned FB v130 PASS (withdrawn under v131; `INVALID_HARNESS`).
14. Not claiming depth collapse / deep SNN scaling (withdrawn under v134; all ceilings at chance).
15. Not claiming anything from `shd-scientific-sweep` (withdrawn; synthetic data).
16. Not claiming temporal-resolution mechanism for attention (refuted by S-5).
17. Not claiming SHD attention calibration (criterion 5 Python mirror unmet).

Hardened package: [`HARD_AUDIT.md`](HARD_AUDIT.md) · [`CLAIM_AXIS.md`](CLAIM_AXIS.md) · [`DIFF_CLOSURE.md`](DIFF_CLOSURE.md) · [`PAPER_METRICS_FULL.md`](PAPER_METRICS_FULL.md) · [`APPENDIX_POST_G2.md`](APPENDIX_POST_G2.md) · mechanism: [`PAPER_FIGURE_SPEC.md`](PAPER_FIGURE_SPEC.md) Figure M.
