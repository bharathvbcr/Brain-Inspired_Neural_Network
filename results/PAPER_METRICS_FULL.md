# PAPER_METRICS_FULL — hardened harvest

> **SUPERSEDED IN PART — 2026-08-25 matched-architecture re-run, banner added
> 2026-08-29.** **Table A was the superseded pre-repair block**, published as
> current: DFA `0.9387` / gap LCB `0.6894` and RL `0.9200` / `0.6846`, under
> three hashes that were **retired** on 2026-08-25 and that `from_hash` no
> longer resolves. Those are the exact values
> [`PAPER_FIGURE_SPEC.md`](PAPER_FIGURE_SPEC.md) §"Figure 6" names as
> "superseded and not for drawing" — from a forward pass that emitted **zero
> spikes at any seed** — and the figure generator is machine-checked against
> drawing them while this sheet went on printing them.
>
> Table A below is now the re-run. The archived block is kept beneath it,
> marked, because a withdrawal that leaves no trace is indistinguishable from a
> claim that was never made.
>
> **Tables B–F are unaffected.** None of those arms runs on the matched
> dense-LIF forward; the C1, live-transfer, break-it and XOR rows are the same
> measurements they were.
>
> This document sits in `PAPER_SIDE` — the paper's own downstream artefacts,
> which `check_every_number.py` deliberately does not sweep — so no number check
> reached it. `scripts/test_published_hashes_resolve.py`, written 2026-08-29,
> is what found it.

**Authority:** on-disk notes only. Dual gap on live arms; gate unchanged.  
**Companions:** [`HARD_AUDIT.md`](HARD_AUDIT.md) · [`CLAIM_AXIS.md`](CLAIM_AXIS.md) · [`DIFF_CLOSURE.md`](DIFF_CLOSURE.md) · camp [`runs/2026-07-23-paper-hard-both/`](runs/2026-07-23-paper-hard-both/)

Gate: local ≥ 0.65 **and** dense-local gap LCB > 0.5. Chance-normalized gap is **descriptive only**.

---

## Table A — Matched dense-LIF (2026-08-25 re-run, `MATCHED_INPUT_SCALE = 2.0`, n = 20)

Feed-forward / recurrent, both graphs. Row-by-row provenance is **Table A** of
[`PAPER_RESULTS_TABLE.md`](PAPER_RESULTS_TABLE.md), which carries the per-graph
hashes; this sheet lists the hash whose frozen-hash test asserts it.

| Arm | Hash | Axis | Verdict | Primary (ff / rec) | Gap LCB (ff / rec) | Source |
|---|---|---|---|---:|---:|---|
| Broadcast ±1 3F (v4) | `c1-match-6f6000f148f7d30c` | Novel-CS | **FAIL** both | 0.5000 / 0.5100 | 0.0000 / −0.0192 | [`matched_rerun_2026-08-25/`](matched_rerun_2026-08-25/) |
| DFA (v5) | `c1-dfa-f79c01ea36fe27d7` | Novel-CS | **PASS** both | 0.9925 / 0.9875 | 0.9689 / 0.9509 | [`matched_rerun_2026-08-25/`](matched_rerun_2026-08-25/) |
| RL×B (v12) | `c1-rl-d35e13c758e522f8` | Novel-CS | **PASS** both | 0.9950 / 0.9812 | 0.9765 / 0.9079 | [`matched_rerun_2026-08-25/`](matched_rerun_2026-08-25/) |
| Discrete EventProp (v28) | `c1-eventprop-f1e841c29755b1c8` | Novel-CS | **PASS** both | 0.9450 / 0.8900 | 0.7911 / 0.6494 | [`matched_rerun_2026-08-25/`](matched_rerun_2026-08-25/) |
| Undertrain 4×ep (v22) | `c1-match-b46b23549b37d90a` | Integrity | **FAIL** | 0.5000 | 0.0000 | [`c1_match_ep4.md`](c1_match_ep4.md) |

**The gate no longer ranks this field.** The SuperSpike reference saturates at
**1.0000** on both graphs, so every PASS above reduces to "the arm scored above
0.75" and no ordering among them may be claimed — see
[`PAPER_FIGURE_SPEC.md`](PAPER_FIGURE_SPEC.md) Figure 6 ban 1, which the figure
generator is tested against. Contrasts, measured and **not gated**:
broadcast-graded 0.9975 / 0.9975, RL graded-reward 0.8787 / 0.9100, RL ±1
broadcast 0.7775 / 0.7962.

### Table A (archived) — the pre-repair block, retired and not citable as current

Kept because a withdrawal that leaves no trace is indistinguishable from a claim
that was never made. Every hash here is **retired** and does not resolve; every
number is from a forward that emitted no spikes.

| Arm | Retired hash | Verdict as recorded | Primary | Gap LCB | Source |
|---|---|---|---:|---:|---|
| Broadcast 3F (v4) | `c1-match-5dc6822e71229e9e` | FAIL | 0.5000 | 0.0000 | [`c1_match.md`](c1_match.md) |
| DFA (v5) | `c1-dfa-c8c4fe0899908b84` | PASS | 0.9387 | 0.6894 | [`c1_dfa.md`](c1_dfa.md) |
| RL×B (v12) | `c1-rl-42eddc9c801308e9` | PASS | 0.9200 | 0.6846 | [`c1_rl.md`](c1_rl.md) |

---

## Table B — Engine C1 / G2

| Arm | Hash | Axis | Verdict | Local | Gate LCB | Chance LCB | min/max/frac≥0.65 | Source |
|---|---|---|---|---:|---:|---:|---|---|
| Canonical v2 | `c1-118207fbc3eaba53` | Novel-CS (caveated) | **FAIL** | 0.4912 | −0.0048 | (harvest) | — | [`c1_g2.md`](c1_g2.md) |
| Isolation | `c1-8ec031907a3426d0` | Integrity | **FAIL** | 0.5188 | — | — | — | [`c1_iso.md`](c1_iso.md) |

---

## Table C — Live transfer + gap-close + break-it

| Proto | Hash | Axis | Verdict | Local | Gate LCB | Chance mean/LCB | frac≥0.65 | Source |
|---|---|---|---|---:|---:|---|---:|---|
| v13 RFB | `c1-660401d74db3c88d` | Novel-CS | **FAIL** | 0.4900 | 0.0737 | — | — | [`c1_rfb.md`](c1_rfb.md) |
| v15 SFB | `c1-493ddd56f8714fb6` | Novel-CS | **FAIL** | 0.7262 | 0.2567 | — | — | [`c1_sfb.md`](c1_sfb.md) |
| v17 SFB×cap | `c1-983ee5303c00b147` | Novel-CS | **FAIL** | 0.6825 | 0.3127 | — | — | [`c1_sfb_cap.md`](c1_sfb_cap.md) |
| **v20** DFA-live | `c1-4db53e645405fae0` | Novel-CS | **FAIL** | **0.7325** | 0.2601 | 0.5417 / 0.3321 | 0.60 | [`c1_dfa_live.md`](c1_dfa_live.md) |
| **v21** soft-WTA | `c1-f975db8fb3e5d569` | Novel-CS | **FAIL** | 0.5025 | 0.0406 | 0.0444 / 0.0122 | 0.00 | [`c1_sfb_soft.md`](c1_sfb_soft.md) |
| **v23** finth | `c1-4bbaf4b24c2d1da2` | Integrity | **FAIL** | **0.6638** | 0.2370 | 0.4019 / 0.2370 | 0.65 | [`c1_sfb_finth.md`](c1_sfb_finth.md) |
| **v24** cont-B | `c1-840f820b7c07b512` | Novel-CS | **FAIL** | 0.6437 | 0.1380 | 0.2786 / 0.1163 | 0.50 | [`c1_sfb_cont.md`](c1_sfb_cont.md) |

**Reading:** Live DFA (v20) clears the accuracy floor (best local among break-it) but misses gap LCB > 0.5 — matched DFA does **not** transfer. Soft-WTA under SFB (v21) regresses to chance. Finite-θ under SFB (v23) clears floor, still gap-short. Continuous B (v24) does **not** beat sign-truncated v15. Undertrain (v22) leaves matched three-factor at chance.

Full v14–v19: [`GAP_CLOSE_RFB_TRANSFER.md`](GAP_CLOSE_RFB_TRANSFER.md).

---

## Table D — XOR / depth (supporting)

| Exp | broadcast | DFA | grad | Axis | Source |
|---|---:|---:|---:|---|---|
| xor_thresh | 0.5008 | 0.8267 | 0.7733 | Novel-CS supporting | [`deep_xor_thresh.json`](deep_xor_thresh.json) |
| depth_locality mid | 0.8158 | 0.8250 | 0.8308 | Integrity | [`deep_depth_locality_mid.json`](deep_depth_locality_mid.json) |

---

## Table E — Motif / methods footnotes

See [`APPENDIX_POST_G2.md`](APPENDIX_POST_G2.md) motif honesty + D13–D16.

---

## Table F — Work-per-accuracy (descriptive; D21)

Per-seed `work_per_accuracy` rows live in each C1 markdown GC7 / budget section (e.g. [`c1_dfa_live.md`](c1_dfa_live.md)). No new G5 efficiency claim in this campaign.

---

## Statistical paragraph (print-ready)

Decisions use the preregistered z-LCB on dense-local normalized gap with n=20 scientific seeds and fixed G2 floors (acc ≥ 0.65, LCB > 0.5). Arms v14–v24 are a **sequential exploratory** family: each new hypothesis minted a new protocol version and hash; there is no multiplicity-corrected family-wise claim. Chance-normalized gaps are reported for live arms as descriptive dual-gap harvest and do not alter the gate.
