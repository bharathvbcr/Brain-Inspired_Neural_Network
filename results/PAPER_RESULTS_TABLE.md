# BINN — camera-ready results table

> ### SUPERSEDED IN PART — 2026-08-25 matched-architecture re-run
>
> Every matched-architecture number in this document was produced before the
> 2026-08-22 silent-initialisation repair, on a forward pass that emitted **zero
> spikes at any seed**, and none of them has been regenerated here. The re-run
> is [`RESULT_2026-08-25_MATCHED_ARCH_RERUN.md`](RESULT_2026-08-25_MATCHED_ARCH_RERUN.md)
> and [`PAPER_DRAFT.md`](PAPER_DRAFT.md) §3.1 carries the current figures.
>
> **The lead negative survives** — broadcast ±1 three-factor is at chance on both
> forward graphs. **Three contrasts do not**: the discrete EventProp-style
> spike-adjoint goes 0.5000 FAIL to 0.9450 / 0.8900 PASS, and the two RL
> broadcast contrasts go 0.5250 to 0.9100 and 0.5113 to 0.7962. The gradient
> ceiling goes 0.8887-0.9150 to **1.0000**, so every `gap_closed` here divides by
> a different denominator than the instrument now produces.
>
> The `c1-*` config hashes cited below are **retired**: `MATCHED_INPUT_SCALE` was
> not part of them, so each named two experiments. They no longer resolve, by
> design. The retirement table is in the re-run document section 8.
>
> Rows concerning the SHD attention campaign, the live-transfer package and the
> XOR task are unaffected: none of them runs on the matched dense-LIF forward.

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



**Authority:** on-disk notes only. Every row cites a file under `results/`.  
**Do not invent.** Quick/PILOT hashes are excluded from this sheet.

**Ordering, changed 2026-08-27.** The **SHD attention read-out** program is the
paper's lead and is now first in this sheet (Tables SHD-1 … SHD-7). The
**matched-architecture kill gate** is the secondary program and follows it
(Tables A–E). The reframe is prose-level only: no number moved because of it.

**Label collision — read before citing a bare letter.**
[`PAPER_METRICS_FULL.md`](PAPER_METRICS_FULL.md) defines its *own* Tables A–F,
and its **Table F is "Work-per-accuracy"**, which is what
[`DIFF_CLOSURE.md`](DIFF_CLOSURE.md) D21 and
[`APPENDIX_POST_G2.md`](APPENDIX_POST_G2.md) mean when they cite "Table F". This
sheet therefore **no longer uses the letter F at all**: its SHD tables are
labelled `SHD-1` … `SHD-7`, which collide with nothing in either file. Letters
A–E in *this* document keep their meaning, because
[`PAPER_FIGURE_SPEC.md`](PAPER_FIGURE_SPEC.md) §"Table 1…S2" and
[`PAPER_SKELETON.md`](PAPER_SKELETON.md) cite them by letter; renumbering them
would break those references. A bare "Table A" is still ambiguous between the two
files and should be written `PAPER_RESULTS_TABLE` Table A or
`PAPER_METRICS_FULL` Table A.

Companion: [`PUBLISHABLE_CLAIMS.md`](PUBLISHABLE_CLAIMS.md) · [`PAPER_SKELETON.md`](PAPER_SKELETON.md) · [`CAMPAIGN_2026-07-23_CLAIM_FREEZE.md`](CAMPAIGN_2026-07-23_CLAIM_FREEZE.md) · **hash-replay verify:** [`runs/2026-07-23-paper-hard-both/VERIFY_SUMMARY.md`](runs/2026-07-23-paper-hard-both/VERIFY_SUMMARY.md) (bit-stable) · **full metrics:** [`PAPER_METRICS_FULL.md`](PAPER_METRICS_FULL.md) · **closure:** [`DIFF_CLOSURE.md`](DIFF_CLOSURE.md)

---

# The read-out program (lead) — SHD attention read-out and mechanism (Waves 1–17)

Anchor configuration unless a row says otherwise: `ff+fixed+attn`, read-out
`d32/L4`, hidden `h128`, contract `published-2ms`, geometry `adjacent-sum-5`,
budget `e400`, rust backend on Linux/aarch64, pinned binary `22d97c51ab02`.
The rate read-out control is `ff+fixed` on the same seeds, same splits, same
binary. Cells: [`shd_attention_campaign_v1/`](shd_attention_campaign_v1/) (waves
1–7 plus `r1cal`) and [`shd_attention_campaign_v2/`](shd_attention_campaign_v2/)
(waves 8–17).

**Gate:** a cell carries `scientific_status: CELL_PASS` only if test accuracy is
**≥ 0.80** *and* all 20 classes are predicted, `majority_prediction < 0.30`,
`silent_fraction ≤ 0.95`, `saturated_fraction ≤ 0.05`, and `non_finite_events ==
0` — the conjunction is `binn-lab/experiments/shd_instrument.rs:963` and it is
written into every archived cell, so gate status is a property of the artefact
rather than of an analyser. The **0.80 floor is preregistered**, not chosen after
the fact: it was derived from one configuration of the pinned third-party PyTorch
reference — a *different model class* — and
[`PAPER_DRAFT.md`](PAPER_DRAFT.md) §4.6 states why it must not be read as a
standard this instrument is failing to meet. An **arm** clears the gate when the
registered rule holds: **mean ≥ 0.80 and ≥ 9 of 12 seeds individually ≥ 0.80**
(R-1), with **budget stability** `|acc(e400) − acc(e200)| < 0.01` (R-2) and
**gain ≥ 0.05** (R-3) — all three registered in
[`PREREG_2026-08-20_SHD_ATTENTION_D32L4_AT_E400.md`](PREREG_2026-08-20_SHD_ATTENTION_D32L4_AT_E400.md)
before any qualifying cell existed. 140 of 776 campaign cells clear the floor
([`FINDING_2026-08-23_THE_MATRIX_GRID_EXCLUDES_ITS_OWN_GATE.md`](FINDING_2026-08-23_THE_MATRIX_GRID_EXCLUDES_ITS_OWN_GATE.md)).

**There is no gap-LCB column below, and none is available.** Unlike the matched
program (Tables A–E), which decides on a z-LCB over a dense-local normalized gap,
this program registers **effect size plus a per-seed sign count** (e.g. "gain
≥ 0.05 and ≥ 10 of 12 seeds positive"). No LCB was computed for any SHD arm, so
none is quoted. Seed counts are per row.

**n=12 is the registered measurement; n=32 is the confirmation.** Both are
correct at their own sample sizes and neither supersedes the other. Twenty seeds
beyond the registered twelve move the headline gain by **+0.0017** and the
shuffle cost by **+0.0010**; both numbers cleared their bars at n=12 and clear
them at n=32 by the same margin
([`RESULT_2026-08-27_W15_17_THE_COLLAPSE_IS_A_THRESHOLD.md`](RESULT_2026-08-27_W15_17_THE_COLLAPSE_IS_A_THRESHOLD.md) §6).

---

## Table SHD-1 — Headline accuracy against the 0.80 gate

| Configuration | Arm | Budget | n | Accuracy | Gain over `ff+fixed` | Gain positive | ≥ 0.80 | Source |
|---|---|---|---:|---:|---:|---:|---:|---|
| **d32/L4 anchor — registered headline** | `ff+fixed+attn` | e400 | **12** | **0.8320** | **+0.1258** | 12/12 | **12/12** | [`RESULT_2026-08-20_D32L4_CLEARS_THE_080_GATE.md`](RESULT_2026-08-20_D32L4_CLEARS_THE_080_GATE.md) §1–2 |
| **d32/L4 anchor — n=32 confirmation** | `ff+fixed+attn` | e400 | **32** | **0.8332** | **+0.1275** | **32/32** | **32/32** | [`RESULT_2026-08-27_W15_17_THE_COLLAPSE_IS_A_THRESHOLD.md`](RESULT_2026-08-27_W15_17_THE_COLLAPSE_IS_A_THRESHOLD.md) §6 |
| d32/L4 anchor rate control | `ff+fixed` | e400 | 12 | 0.7062 | — | — | 0/12 | [`RESULT_2026-08-21_W8_HEADLINE_SCOPE_IS_MEASURED.md`](RESULT_2026-08-21_W8_HEADLINE_SCOPE_IS_MEASURED.md) §1 |
| d32/L4 anchor rate control (n=32) | `ff+fixed` | e400 | 32 | 0.7057 | — | — | — | [`RESULT_2026-08-27_W15_17_THE_COLLAPSE_IS_A_THRESHOLD.md`](RESULT_2026-08-27_W15_17_THE_COLLAPSE_IS_A_THRESHOLD.md) §6 |
| d32/L4 @ e200 (budget stability R-2) | `ff+fixed+attn` | e200 | 12 | 0.8322 | +0.1454 | — | 12/12 | [`RESULT_2026-08-20_D32L4_CLEARS_THE_080_GATE.md`](RESULT_2026-08-20_D32L4_CLEARS_THE_080_GATE.md) §1 |
| d32/L4 @ e100 | `ff+fixed+attn` | e100 | 12 | 0.8209 | +0.1550 | — | 11/12 | [`RESULT_2026-08-20_D32L4_CLEARS_THE_080_GATE.md`](RESULT_2026-08-20_D32L4_CLEARS_THE_080_GATE.md) §1 |
| d32/L2 anchor | `ff+fixed+attn` | e400 | 12 | 0.7897 | — | — | 4/12 | [`RESULT_2026-08-21_W8_HEADLINE_SCOPE_IS_MEASURED.md`](RESULT_2026-08-21_W8_HEADLINE_SCOPE_IS_MEASURED.md) §1 |
| d32/L1 anchor *(wave 1, the arm the campaign registered)* | `ff+fixed+attn` | e400 | 12 | 0.7483 | +0.0421 (bar 0.05 → **NOT SUPPORTED**) | 12/12 | 0/12 | [`RESULT_2026-08-19_W1_ATTENTION_AT_CONVERGENCE.md`](RESULT_2026-08-19_W1_ATTENTION_AT_CONVERGENCE.md) |
| d32/L1 @ e10 (sample efficiency) | `ff+fixed+attn` | e10 | 12 | 0.7337 | +0.2002 (rate 0.5336) | — | — | [`RESULT_2026-08-20_W7_CONVERGENCE_IS_BRACKETED.md`](RESULT_2026-08-20_W7_CONVERGENCE_IS_BRACKETED.md) §1 |
| d32/L1 @ e5 | `ff+fixed+attn` | e5 | 12 | 0.6756 | +0.2227 (rate 0.4529) | — | — | [`RESULT_2026-08-20_W7_CONVERGENCE_IS_BRACKETED.md`](RESULT_2026-08-20_W7_CONVERGENCE_IS_BRACKETED.md) §1 |
| d64/L4 *(descriptive, **no verdict** — never a registered hypothesis)* | `ff+fixed+attn` | e400 | 12 | 0.8441 | +0.1379 | — | 12/12 | [`RESULT_2026-08-21_W9_THE_MECHANISM_HOLDS_AT_THE_HEADLINE.md`](RESULT_2026-08-21_W9_THE_MECHANISM_HOLDS_AT_THE_HEADLINE.md) §5b |

**Reading.** The registered headline is **0.8320 / 12 of 12 ≥ 0.80** at n=12,
budget-stable at `|e400 − e200| = 0.0002` against a 0.01 bar, confirmed at
**0.8332 / 32 of 32** at n=32. The **e10 row is on the `d32/L1` ladder**: its
"98.1% of converged" denominator is that ladder's own e400 value (0.7483), **not**
the 0.8320 headline — against the headline the same cell is 88.2%. The two
operating points are not interchangeable and the L1 e400 gain is +0.0421, which
**failed** its registered 0.05 bar. **d64/L4 may not be promoted**: +0.0121 over
d32/L4 across an untested axis is an estimate, and W9 registered M-3 as
descriptive precisely to stop it being read as a finding.

## Table SHD-2 — The mechanism: bin-shuffle difference-in-differences

Bin shuffling permutes time bins independently per sample in **both the training
and test splits**, so the task itself becomes rate-solvable. Every `w9shf` cell
passes the temporal audit (counts preserved, relocated fraction ≥ 0.5), so a
shuffle that failed to shuffle would have been voided rather than scored.

| Arm | n | Intact | Bin-shuffled | Shuffle cost (intact − shuffled) | Cost positive | Source |
|---|---:|---:|---:|---:|---:|---|
| **d32/L4 (headline) — registered** | **12** | **0.8320** | **0.6983** | **+0.1337** | **12/12** | [`RESULT_2026-08-21_W9_THE_MECHANISM_HOLDS_AT_THE_HEADLINE.md`](RESULT_2026-08-21_W9_THE_MECHANISM_HOLDS_AT_THE_HEADLINE.md) §2–3 |
| d32/L4 rate control — registered | 12 | 0.7062 | 0.6934 | **+0.0128** | — | [`RESULT_2026-08-21_W9_THE_MECHANISM_HOLDS_AT_THE_HEADLINE.md`](RESULT_2026-08-21_W9_THE_MECHANISM_HOLDS_AT_THE_HEADLINE.md) §2 |
| **d32/L4 (headline) — n=32 confirmation** | **32** | **0.8332** | — | **+0.1347** | **32/32** | [`RESULT_2026-08-27_W15_17_THE_COLLAPSE_IS_A_THRESHOLD.md`](RESULT_2026-08-27_W15_17_THE_COLLAPSE_IS_A_THRESHOLD.md) §6 |
| d32/L4 rate control — n=32 confirmation | 32 | 0.7057 | — | **+0.0142** | — | [`RESULT_2026-08-27_W15_17_THE_COLLAPSE_IS_A_THRESHOLD.md`](RESULT_2026-08-27_W15_17_THE_COLLAPSE_IS_A_THRESHOLD.md) §6 |
| d32/L1 *(wave 1)* | 12 | 0.7483 | 0.6442 | +0.1041 | — | [`RESULT_2026-08-19_W1_ATTENTION_AT_CONVERGENCE.md`](RESULT_2026-08-19_W1_ATTENTION_AT_CONVERGENCE.md) |

| Derived quantity | n=12 (registered) | n=32 (confirmation) | Source |
|---|---:|---:|---|
| Ratio of shuffle costs (attention ÷ rate) | **10×** | **9.5×** | W9 §3 · W15/17 §6 |
| Read-out advantage, intact | +0.1258 | +0.1275 | W9 §4 · W15/17 §6 |
| Read-out advantage, shuffled | **+0.0050** | **+0.0070** | W9 §4 · [`PAPER_DRAFT.md`](PAPER_DRAFT.md) §3.5 |
| Fraction of the advantage contingent on temporal order | **96%** | **94.5%** | W9 §4 · [`PAPER_DRAFT.md`](PAPER_DRAFT.md) §3.5 |

**Reading.** This is a difference-in-differences on the **gain**, not on accuracy.
Per seed at n=12 the effect falls between **+0.0967 and +0.1568** — no seed in
which it is absent. The +0.0050 shuffled advantage is **recomputed from cells**,
not obtained by subtracting the two rounded means above it (that arithmetic gave
+0.0049); the correction is recorded in W9 §4 and the conclusion is unchanged.
**H17-2 required a mid-run amendment**: the analyser was merging a `d32l1`
archived shuffled control into a `d32l4` comparison for twelve pairs, inflating
the cost from +0.1347 to +0.1577. The verdict was MET either way and
[`PAPER_DRAFT.md`](PAPER_DRAFT.md) §3.5 is unaffected
([`AMENDMENT_2026-08-27_H17_2_MERGED_TWO_READOUT_DEPTHS.md`](AMENDMENT_2026-08-27_H17_2_MERGED_TWO_READOUT_DEPTHS.md)).

### Table SHD-2b — The same contrast at seven further operating points (wave 21)

Registered in [`PREREG_2026-08-27_THE_MECHANISM_ACROSS_THE_DESIGN_SPACE.md`](PREREG_2026-08-27_THE_MECHANISM_ACROSS_THE_DESIGN_SPACE.md)
before any cell existed; analysed by the frozen `scripts/aws/analyse_wave21.py`.
**168 cells, zero divergences, zero voided**, every point above the registered
floor of nine seed-paired quadruples. Intact halves are reused from the corpus at
the same seeds and the same pinned binary; only the `bin-shuffled` halves are new.

| Operating point | Quadruples | Gain | **DiD** | DiD positive |
|---|---:|---:|---:|---:|
| h128 / `published-2ms` / `adjacent-sum-5` *(anchor)* | 32 | +0.1275 | **+0.1205** | 32/32 |
| h256 | 12 | +0.0966 | **+0.0862** | 12/12 |
| h384 | 12 | +0.0760 | **+0.0767** | 12/12 |
| h512 | 12 | +0.0876 | **+0.0968** | 12/12 |
| h768 | 12 | +0.0560 | **+0.1881** | 12/12 |
| h1024 | 12 | −0.1318 | **+0.1122** | 10/12 |
| h128 / `channels-700` | 12 | +0.1090 | **+0.1122** | 12/12 |
| h128 / `published-10ms` | 12 | +0.1491 | **+0.0959** | 12/12 |

| Registered hypothesis | Bar | Verdict |
|---|---|---|
| **H21-1** the mechanism is not unique to h128 | DiD ≥ +0.03 and ≥ 9/12 positive at **each** of h256, h384, h512 | **MET** |
| **H21-2** where the gain inverts, the shuffle cost collapses | DiD(h1024) ≤ +0.02 | **NOT MET** |
| **H21-3** the shuffle cost tracks the gain across width | Spearman ρ ≥ **+0.829** over six rungs | **NOT MET** (ρ = **−0.1430**) |
| **H21-4** the mechanism survives a change of binning | DiD ≥ +0.03 and ≥ 9/12 at **both** points | **MET** |

**Reading, and the second row is the one that constrains the paper.** The
mechanism generalises: coverage goes from **2 to 9 of 21** operating points and
the DiD clears its bar at every one. Its **size does not track the gain** — h768
carries the smallest positive gain on the ladder and the largest DiD in the wave
— so the contrast is a property of the read-out and **not** a decomposition of
the gain. At h1024 the read-out consumes temporal order while *harming* accuracy,
which no account in this package permits.

**The `Gain` and `DiD` columns are over the same seeds everywhere except h1024.**
There the DiD is over the 12 quadruples and the gain over the 20 intact pairs
waves 18–19 extended that width to; over the twelve quadruple seeds the h1024
gain is **−0.1618**. The rank is unchanged, so H21-3's ρ is unaffected. This is
`analyse_wave21.py`'s documented `gain()` contract, not a defect.
([`RESULT_2026-08-29_W21_THE_MECHANISM_TRAVELS_BUT_ITS_SIZE_DOES_NOT.md`](RESULT_2026-08-29_W21_THE_MECHANISM_TRAVELS_BUT_ITS_SIZE_DOES_NOT.md);
[`VERDICTS_W21.md`](shd_attention_campaign_v2/VERDICTS_W21.md))

## Table SHD-3 — Width ladder and the h1024 threshold

Six rungs, `published-2ms` / `adjacent-sum-5` / e400 / d32/L4, seed-paired,
**n=12 pairs at every rung**.

| Width | n (pairs) | `ff+fixed` | d32/L4 | Gain | Gain positive | Source |
|---|---:|---:|---:|---:|---:|---|
| h128 *(anchor)* | 12 | 0.7062 | 0.8320 | **+0.1258** | 12/12 | [`RESULT_2026-08-27_W15_17_THE_COLLAPSE_IS_A_THRESHOLD.md`](RESULT_2026-08-27_W15_17_THE_COLLAPSE_IS_A_THRESHOLD.md) §5 |
| h256 | 12 | 0.7240 | 0.8206 | **+0.0966** | 12/12 | same §5 |
| h384 | 12 | 0.7336 | 0.8096 | **+0.0760** | 12/12 | same §5 |
| h512 | 12 | 0.7357 | 0.8233 | **+0.0876** | 12/12 | same §5 |
| h768 | 12 | 0.7386 | 0.7946 | **+0.0560** | 11/12 | same §5 |
| **h1024** | 12 | 0.7386 | **0.5768** | **−0.1618** | **1/12** | same §5 · [`RESULT_2026-08-21_W8_HEADLINE_SCOPE_IS_MEASURED.md`](RESULT_2026-08-21_W8_HEADLINE_SCOPE_IS_MEASURED.md) §1 (range 0.3746–0.7412) |

| Derived quantity | Value | Registered bar | Verdict | Source |
|---|---:|---|---|---|
| **Drop into h1024** | **0.2178** | ≥ 3× the largest gap below it (0.0947) | **H16-2 MET** — a **threshold**, not the slope continuing | W15/17 §5 |
| Ratio to the largest gap below it (0.0316) | **6.9×** | — | — | W15/17 §5 |
| Location of the collapse | between **h768 and h1024** | — | supersedes the four-rung reading in [`RESULT_2026-08-22_AZURE_TRUNCATED_AT_95_OF_252.md`](RESULT_2026-08-22_AZURE_TRUNCATED_AT_95_OF_252.md) | W15/17 §5 |
| gain(h384) − gain(h512), seed-paired | −0.0116 (sd 0.0253, negative in 7/12) | strict ordering, 0.005 separations | **H16-1 NOT MET** — the rungs are **not distinguishable at n=12**; no monotonicity is claimed and **no dip at h384 is claimed** | W15/17 §5 |
| Cross-ISA check, h256 | Azure x86-64 gain **+0.0962** on its four seeds; AWS aarch64 **+0.0962** restricted to the same four, **+0.0966** at n=12; four cells byte-identical across thirteen scientific fields | — | Azure rung **confirmed, not replaced** | W15/17 §5 |

## Table SHD-4 — The h1024 collapse: three registered rescue levers, all failed

Preregistered in
[`PREREG_2026-08-25_THE_H1024_COLLAPSE.md`](PREREG_2026-08-25_THE_H1024_COLLAPSE.md),
frozen with its analyser before the first cell existed. All at h1024 / d32/L4 /
e400, seed-paired, **n=12 pairs each**.

| Lever | n (pairs) | Gain | Gain positive | Median epoch-mean gradient norm | Source |
|---|---:|---:|---:|---:|---|
| surrogate scale 0.5 | 12 | **−0.2106** | 0/12 | 142.009 | [`RESULT_2026-08-27_W15_17_THE_COLLAPSE_IS_A_THRESHOLD.md`](RESULT_2026-08-27_W15_17_THE_COLLAPSE_IS_A_THRESHOLD.md) §2 |
| surrogate scale 0.25 | 12 | **−0.2565** | 0/12 | 151.391 | same §2 |
| clip-grad-norm 1000.0 | 12 | **−0.0904** | 1/12 | **11.660** (unclipped arm: 55.494) | same §2 |
| *(control)* clip 1000.0 at h512/d32/L4 | 12 | — | — | — | **12/12 byte-identical** to the archived `w8wid` cells → **H15-4 MET**, the flag is inert where it cannot bind; same §4 |

| Verdict | Statement | Result | Source |
|---|---|---|---|
| **H15-1** | the collapse is an optimisation failure a lever can undo | **NOT MET** — every lever is negative and **worse than the arm it was meant to rescue** | W15/17 §1–2 |
| **H15-2** | recovery, if it happens, is numerical | **NOT MET** (no arm met H15-1) — *not* evidence that the numerics are healthy | W15/17 §2 |

**Reading.** Clipping moves the median norm from 55.494 to 11.660 — a real
numerical effect in the intended direction — and **accuracy does not follow**, so
the collapse is not a gradient scale that can be turned down. The clip bound on a
median of 96 of 12,800 optimiser steps per cell (0.75%, range 2–192) across a
median 37 of 400 epochs (9.2%), and `unclippable_steps` is 0 in every cell. **The
collapse is located but unexplained**; nothing in this package offers a mechanism
for it, and its only known correlate (gradient norms leaving O(1)) is not one.
The h1024 depth observation (L1 −0.0159, L2 **+0.0392** 12/12, L4 −0.1618 — L2
above *both*, which H15-3 had no branch for) is **explicitly not claimed here**
and is registered as its own wave in
[`PREREG_2026-08-27_DEPTH_IS_NOT_MONOTONE_AT_H1024.md`](PREREG_2026-08-27_DEPTH_IS_NOT_MONOTONE_AT_H1024.md).

## Table SHD-5 — Geometry and input-contract scope

| Configuration | n | `ff+fixed` | d32/L4 | Gain | Gain positive | ≥ 0.80 | Verdict | Source |
|---|---:|---:|---:|---:|---:|---:|---|---|
| `adjacent-sum-5` / `published-2ms` *(anchor)* | 12 | 0.7062 | **0.8320** | **+0.1258** | 12/12 | **12/12** | clears the gate | [`RESULT_2026-08-20_D32L4_CLEARS_THE_080_GATE.md`](RESULT_2026-08-20_D32L4_CLEARS_THE_080_GATE.md) |
| `channels-700` *(standard 700-channel input)* | 12 | 0.6774 | 0.7864 | **+0.1090** | **12/12** | 6/12 | **S-1 NOT SUPPORTED** (bars 0.80 and 9/12); **S-2 SUPPORTED** — the gain survives | [`RESULT_2026-08-21_W8_HEADLINE_SCOPE_IS_MEASURED.md`](RESULT_2026-08-21_W8_HEADLINE_SCOPE_IS_MEASURED.md) §1–3 |
| `published-10ms` | 12 | 0.6734 | **0.8225** | **+0.1491** | **12/12** | 10/12 | **S-4 SUPPORTED** | same §1–3 |

**Reading.** *Attention buys roughly the same amount everywhere tested; the anchor
geometry is the one where that is enough to clear the bar.* The **0.80 clearance
is geometry-specific and must be stated as part of the headline, not as a
footnote**; the **gain is not**. S-5, the registered temporal-*resolution*
prediction, was **NOT SUPPORTED** on this family — and is withdrawn as a design,
because `published-Nms` moves bin width and sequence length together so no single
number can be attributed to either. It is re-asked on `fixed-tN` in Table SHD-6.

## Table SHD-6 — Temporal-resolution ladder (`fixed-tN`)

`fixed-tN` holds a 1400 ms window fixed and varies only the number of frames, so
bin width moves without sequence length moving with it. **n=12 at every rung.**

| Contract | Bin | n | `ff+fixed` | d32/L4 | Gain | Gain positive | ≥ 0.80 | Source |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| `fixed-t100` | 14.0 ms | 12 | 0.6672 | 0.8599 | **+0.1927** | 12/12 | 12/12 | [`RESULT_2026-08-22_W10_RESOLUTION_LADDER.md`](RESULT_2026-08-22_W10_RESOLUTION_LADDER.md) |
| `fixed-t250` | 5.6 ms | 12 | 0.6844 | 0.8594 | **+0.1751** | 12/12 | 12/12 | same |
| `fixed-t500` | 2.8 ms | 12 | 0.7069 | 0.8543 | **+0.1474** | 12/12 | 12/12 | same |

| Derived quantity | Value | Bar | Verdict | Source |
|---|---:|---|---|---|
| gain(t500) − gain(t100) | **−0.0453** | two-sided 0.03 | **C-2 SUPPORTED** — the advantage **shrinks with finer resolution**, the opposite of S-5's direction | W10 |
| baseline drift, `ff+fixed` t500 − t100 | +0.0397 | confound bar 0.05 | **C-3 not confounded** — a property of the read-out, not of the substrate | W10 |
| 0.80 clearance | 12/12 at all three rungs | ≥ 0.80 | **C-4 SUPPORTED** at every rung, the coarsest most comfortably | W10 |

## Table SHD-7 — Substrate comparison: the read-out does not substitute for temporal state

The anchor campaign's 720 cells all sat on one substrate, `ff+fixed`, leaving two
readings the campaign could not separate: the read-out **adds** temporal structure
no such substrate represents, or it **substitutes** for the adaptation and
recurrence `ff+fixed` happens not to have. Three waves settle it.

| Substrate | Surrogate scale | n (pairs) | Rate read-out | + attention d32/L4 | Gain | Gain positive | ≥ 0.80 (rate arm) | Source |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| `ff+fixed` *(anchor)* | 1.0 | 12 | 0.7062 | 0.8320 | **+0.1258** | 12/12 | 0/12 | [`RESULT_2026-08-23_W12_ATTENTION_DOES_NOT_SUBSTITUTE_FOR_ADAPTATION.md`](RESULT_2026-08-23_W12_ATTENTION_DOES_NOT_SUBSTITUTE_FOR_ADAPTATION.md) |
| `ff+alif` *(threshold adaptation)* | 1.0 | 12 | 0.7018 | 0.8303 | **+0.1285** | — | **0/12** | same |
| `rec+alif` *(recurrent + adaptation)* | 0.4 | **10** | 0.5262 | 0.7874 | **+0.2612** | **10/10** | — | [`RESULT_2026-08-23_W14_ATTENTION_AND_RECURRENCE_ARE_COMPLEMENTARY.md`](RESULT_2026-08-23_W14_ATTENTION_AND_RECURRENCE_ARE_COMPLEMENTARY.md) |
| `ff+fixed` *(scale-matched control)* | 0.4 | 12 | 0.7088 | 0.8289 | **+0.1201** | — | — | same |

| Derived quantity | Value | Bar | Verdict | Source |
|---|---:|---|---|---|
| gain(`ff+alif`) − gain(`ff+fixed`) | **+0.0027**, positive in **6 of 12** | two-sided 0.03 | **A-1** — adaptation makes no difference to the gain; a coin flip | W12 |
| `ff+alif` − `ff+fixed`, rate arms | **−0.0044**, better in **3 of 12** | — | adaptation alone does not help either; at this operating point it is **inert** | W12 |
| gain(`rec+alif`) − gain(`ff+fixed`) at scale 0.4 | **+0.1411**, positive in **10 of 10** | two-sided 0.03 | **M-2 SUPPORTED** — the gain roughly **doubles** on the recurrent substrate | W14 |
| scale confound | `ff+fixed` 0.7088 at scale 0.4 vs **0.7062** archived at 1.0 → **+0.0026** | — | **M-4** — the scale is not doing the work | W14 |
| headroom-normalised ratio *(post-hoc, **not registered**)* | 0.551 vs 0.412 → **1.34×**, down from 2.2× | — | the **ordering survives; most of its apparent size does not** | W14 |

**Substrate usability, which had to be measured before any of the above could be:**

| Arm | Surrogate scale | Completed | Voided | Diverged | Source |
|---|---:|---:|---:|---:|---|
| `rec+alif` | **0.4** | **11/12** (bar 11/12) | 0 | 1 | [`RESULT_2026-08-23_W13_RECURRENT_STABILITY.md`](RESULT_2026-08-23_W13_RECURRENT_STABILITY.md) §1 |
| `rec+alif` | 1.0 | 8/12 | 0 | 4 | same |
| `rec+fixed` | 0.4 | 7/12 | 5 | 0 | same |
| `rec+fixed` | 1.0 | 5/12 | 5 | 2 | same |

**Reading.** **Substitution is refuted on both axes.** The read-out's advantage is
indifferent to adaptation and *larger* where the substrate is recurrent, which is
a claim about what the read-out **consumes**, not about a deficiency of one
substrate. `rec+fixed`'s ten voided cells all failed by **saturation**
(`saturated_fraction` 0.055 to **0.523**), none by divergence at scale 0.4, so on
the recurrent substrate adaptation is **stabilising** — the opposite of the sign
that wave's own hypothesis name asserted (R-2, +7 on a two-sided bar of 6).
Four limits are load-bearing: (1) the recurrent gain is measured from a base 0.18
lower, with 0.4738 of headroom against 0.2912; (2) **the recurrent substrate does
not win** — `rec+alif+attn` 0.7874 against `ff+fixed+attn` 0.8289 at the same
scale, and **no verdict is issued on that ordering**; (3) the recurrent arms are
numerically extreme (peak gradient norms to 4.9e32 against 1.13e8 for the largest
cell anywhere else) and rest on **ten pairs, the registered minimum** — one
further loss on either arm would have made the comparison unreportable;
(4) survivorship is **reduced, not removed** — the surviving recurrent pairs are
those that did not diverge, and divergence is not random.

---

**Wave coverage note.** Waves 1–17 are on disk. **Wave 11 contributes no row**:
its completion expectation was NOT MET (15 of 24 against a bar of 18) and T4-1,
T4-2 and T4-3 are **NOT EVALUABLE with no verdict issued**
([`RESULT_2026-08-22_W11_CLIPPING_WAS_NOT_THE_WHOLE_CAUSE.md`](RESULT_2026-08-22_W11_CLIPPING_WAS_NOT_THE_WHOLE_CAUSE.md)).
Waves 18–19 are registered and unrun. Campaign narrative:
[`SUMMARY_2026-08-20_ATTENTION_CAMPAIGN.md`](SUMMARY_2026-08-20_ATTENTION_CAMPAIGN.md) ·
[`SUMMARY_2026-08-22_CAMPAIGN_AND_RECORD_REPAIR.md`](SUMMARY_2026-08-22_CAMPAIGN_AND_RECORD_REPAIR.md).

**Machine check.** `scripts/record_checks.sh` recomputes the SHD attention-campaign
numbers from the archived cells via `scripts/verify_published_numbers.py`
(**66 assertions**, 13 of them prose checks on [`PAPER_DRAFT.md`](PAPER_DRAFT.md)).
The matched-architecture numbers in Tables A–E are **not** among them: they are
attested by the on-disk hashed run records cited row-by-row.

**Not claimed by this program:** SHD attention calibration (criterion 5, the
Python mirror of the attention axis, does not exist, and `SHD_INSTRUMENT_STATE`
is a compile-time `Uncalibrated`); a temporal-*resolution* mechanism (S-5
refuted); competitive accuracy (the SHD frontier is 95–96.4%); any explanation of
the h1024 collapse; any ordering of read-out depth at h1024; anything from
`shd-scientific-sweep` (withdrawn, synthetic data).

---

# The matched-gate program (secondary) — matched-architecture kill gate

## Table A — Matched dense-LIF kill gate

| Arm | Hash | Verdict | Primary mean | Contrast / ceiling | Gap LCB | Source |
|---|---|---|---:|---:|---:|---|
| Broadcast ±1 three-factor (v4) | `c1-match-6f6366f148fab635` ff · `c1-match-6f6000f148f7d30c` rec | **FAIL** both | 0.5000 ff · 0.5100 rec | grad **1.0000** | 0.0000 ff · −0.0192 rec | [`matched_rerun_2026-08-25/`](matched_rerun_2026-08-25/) |
| DFA graded×fixed-B (v5) | `c1-dfa-f79c01ea36fe27d7` ff · `c1-dfa-f7989bea36fb44ae` rec | **PASS** both | 0.9925 ff · 0.9875 rec | grad **1.0000**; broadcast-graded **0.9975** | 0.9689 ff · 0.9509 rec | [`matched_rerun_2026-08-25/`](matched_rerun_2026-08-25/) |
| RL `rl_reinforce_fb` (v12) | `c1-rl-d35e13c758e522f8` ff · `c1-rl-d36179c758e80621` rec | **PASS** both | 0.9950 ff · 0.9812 rec | graded 0.8787/0.9100; flat 0.7775/0.7962; grad **1.0000** | 0.9765 ff · 0.9079 rec | [`matched_rerun_2026-08-25/`](matched_rerun_2026-08-25/) |
| **RL Online Learned `B_i` (v130)** | `track-b-rescue` | **WITHDRAWN** | — | — | — | [`RESULT_2026-08-19_TRACK_B_V130_PASS_WITHDRAWN.md`](RESULT_2026-08-19_TRACK_B_V130_PASS_WITHDRAWN.md) · v131 `INVALID_HARNESS` |
| Discrete EventProp-style (v28) | `c1-eventprop-f1e841c29755b1c8` rec · `c1-eventprop-f1eba7c2975894f1` ff | **PASS** both *(was FAIL)* | 0.8900 rec · 0.9450 ff | SuperSpike **1.0000** | 0.6494 rec · 0.7911 ff | [`matched_rerun_2026-08-25/`](matched_rerun_2026-08-25/) |
| RL graded primary (v11, archived) | `c1-rl-ef504db58916720d` | **FAIL** | 0.5900 | — | 0.0182 | [`c1_rl_v11_graded_primary.md`](c1_rl_v11_graded_primary.md) |

**Paper language (required MUST):** Lead negative is **±1 × surrogate eligibility** specifically — not “any broadcast rule”, not bare “broadcast credit topology”, and not “any ±1 rule”: `MatchedRlFlat` (±1 broadcast REINFORCE) reaches 0.7775/0.7962 while `MatchedLocal` sits at chance, so the two must be named by rule and never collapsed. Broadcast-**graded** reaches **0.9975** — disclose in text and in Figure M. **Every other rule tested now clears this gate against a reference at 1.0000**, so each PASS above reduces to “above 0.75” and **no ordering among the passing arms may be claimed**. Do not use coincidence alone to claim “locality is required”; locality flip evidence is **XOR** (Table D). Falsifier: matched ±1 clearing gap LCB under the same forward overturns the lead claim. **The discrete EventProp H2H FAIL is WITHDRAWN** — it PASSes on a forward that can spike (0.9450/0.8900), and the archived 0.5000 was a spike-adjoint method with no spikes to differentiate through; it remains discrete and no comparison to continuous Wunderlich–Pehle is claimed in either direction. SuperSpike is the matched ceiling. v131 is matched-only (not live transfer). **A6 ceiling health, now the binding limitation:** the reference reaches **1.0000 at the canonical 80-epoch budget itself**, so no budget separates the arms and no ceiling comparison on this task survives.

**Gate:** gap LCB > 0.5 and primary mean ≥ 0.65; gradient mean ≥ 0.65 for harness validity.

---

## Table B — Engine C1 / Gate G2 (supporting, caveated)

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

## Non-claims (print in paper)

1. Not biology / cortex / digital brain.  
2. Not Assembly Calculus PASS.  
3. Not impossibility of local learning in principle.  
4. Not live-engine rescue from matched DFA / RL PASS (v13–v24). Not v131 as live PASS (matched-only).  
5. Not “structured B / capacity / eligibility / soft-WTA / continuous-B PASS G2” (floor ≠ gate).  
6. Not coincidence-only proof that credit locality is required (broadcast-graded **0.9975** also learns coincidence; use XOR for locality). *(This row read 0.9863 until 2026-08-27. That was the pre-repair value and it contradicted Table A and the required-MUST paragraph, both of which carry 0.9975 from the 2026-08-25 re-run. 0.9975 is current; do not cite 0.9863.)*  
7. Not reopening `c1-118207fbc3eaba53` by threshold massage.  
8. Not undertraining as the matched ±1 three-factor FAIL cause (v22).  
9. Not EventProp “absent”, and no longer EventProp “fails”: the discrete H2H **PASSes** on the repaired forward (0.9450 ff / 0.8900 rec). The FAIL is withdrawn; it is still ≠ continuous Wunderlich–Pehle and no comparison to it is claimed.  
10. Not equating hybrid T=2.0 collapse with live v21 (T=1).  
11. Not treating appendix G3 / G4 / H0 as reopening G2.  
12. Not mixing overnight SHD p27 (20-way capped e-prop) with proto-135 SHD sweep (5-class) or protocol-29 full-corpus SuperSpike (`c1-shd-full-*`).
13. Not claiming online learned FB v130 PASS (withdrawn under v131; `INVALID_HARNESS`).
14. Not claiming depth collapse / deep SNN scaling (withdrawn under v134; all ceilings at chance).
15. Not claiming anything from `shd-scientific-sweep` (withdrawn; synthetic data).
16. Not claiming temporal-resolution *mechanism* for attention (S-5 refuted, and its `published-Nms` design withdrawn — it moved bin width and sequence length together). Table SHD-6 reports the measured *dependence* of the gain on resolution on `fixed-tN`, which is a description of the read-out and not a mechanism claim.
17. Not claiming SHD attention calibration (criterion 5 Python mirror unmet).

Hardened package: [`HARD_AUDIT.md`](HARD_AUDIT.md) · [`CLAIM_AXIS.md`](CLAIM_AXIS.md) · [`DIFF_CLOSURE.md`](DIFF_CLOSURE.md) · [`PAPER_METRICS_FULL.md`](PAPER_METRICS_FULL.md) · [`APPENDIX_POST_G2.md`](APPENDIX_POST_G2.md) · mechanism: [`PAPER_FIGURE_SPEC.md`](PAPER_FIGURE_SPEC.md) Figure M.
