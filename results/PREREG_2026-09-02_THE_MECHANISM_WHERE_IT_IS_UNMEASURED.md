# Preregistration — every place the mechanism claim is still asserted without a control

**Registered:** 2026-09-02, **before any cell of wave 25 exists and before any
instance for it is launched.** Attested by git history: this file,
`scripts/aws/analyse_wave25.py` and the `w25mec` plan are committed together,
and the first cell is produced afterwards.

**Analyser:** `scripts/aws/analyse_wave25.py`, frozen in the same commit and the
authority on every verdict below.

**Pinned binary:** `3afd4434431a75a26cc9d5fa46831341fc2f1dd0ef08dc308e18ca139b576364`,
already published at `s3://binn-campaign-v2-511192439661-us-east-1/input/binary.sha256`
and the binary every wave-22 and wave-24 cell was produced by. This wave
launches into that bucket under `launch.py --require-binary-sha256`, which
refuses to provision on a mismatch.

---

## 1. Four groups, one wave

They share a binary, a fleet and a question — *where does the
difference-in-differences actually hold* — and each carries its own hypothesis
and its own bar. Nothing is pooled across groups.

### 1.1 The recurrent substrate has never been manipulated. Not once.

This is the group that must run.

§3.7 of the draft establishes that the read-out's advantage is **larger** where
the substrate is recurrent, and then reads that finding through §3.5:

> *Read with §3.5's shuffle result — 94.5% of the advantage contingent on
> temporal order at n=32 — the claim the paper supports is about what the
> read-out consumes, not about a deficiency of one substrate.*

Every shuffled, reversed or channel-shuffled cell in this corpus is `ff+fixed`
or `ff+fixed+attn` — 784 of them, and **zero** on any recurrent arm. That
sentence therefore transports an order-dependence claim across substrates on no
evidence. A reviewer who checks will find the paper's own coverage script says
so: `mechanism_coverage.py` restricts `SUBSTRATE` to the feed-forward pair and
documents why.

`reversed` is in this group for the reason wave 24 exists. Running only
`bin-shuffled` here would rebuild, in the one arm that has never been
controlled, exactly the confound wave 24 removed everywhere else.

`surrogate_scale = 0.4` is the registered recurrent operating point
([`AMENDMENT_2026-08-05_SURROGATE_GAIN_FOR_RECURRENT.md`](AMENDMENT_2026-08-05_SURROGATE_GAIN_FOR_RECURRENT.md)),
not a value chosen here.

### 1.2 The synchrony term is one point, and the abstract says so

Wave 24 found that at `fixed-t250`, destroying cross-channel synchrony as well
as temporal order costs **+0.1038 more** than destroying order alone (+0.2157
against +0.1119), while both `published-2ms` points sat inside a ±0.03 band.
That is now in the abstract as *"at at least one operating point"* — the
weakest sentence in the manuscript.

The other two rungs of the same ladder decide whether it is a property of
resolution or of a single point. `intact` and `bin-shuffled` exist at both from
wave 22 on this binary, so only the `channel-shuffled` halves run.

### 1.3 The cells H22-3 needed and did not have

`analyse_wave22.py` compares each depth point against a `d32l4` twin **on the
anchor contract, drawn from the wave's own cells**, and wave 22's plan contained
none — so its registered depth question came back NOT EVALUABLE
([`DEFECT_2026-08-31_H22_3_CANNOT_BE_EVALUATED.md`](DEFECT_2026-08-31_H22_3_CANNOT_BE_EVALUATED.md)).
These are those twins. The rate arms already exist in `w22cov` at all four
widths, on this binary.

### 1.4 Nineteen points are still single-budget

Wave 24 measured two of the twenty-one at `e100`. This is the rest.

## 2. Design

| group | cells | what runs |
|---|---:|---|
| H25-1 recurrent | 72 | `rec+alif` and `rec+alif+attn` `d32l4`, ss 0.4, × `intact`/`bin-shuffled`/`reversed` × 12 seeds |
| H25-2 synchrony | 48 | `channel-shuffled` only, at `fixed-t100` and `fixed-t500`, both arms × 12 seeds |
| H25-3 depth twins | 96 | `d32l4` at the anchor, `intact` + `bin-shuffled`, at h128/h256/h512/h768 × 12 seeds |
| H25-4 budget | 672 | 19 points + 9 rate geometries at `e100`, `intact` + `bin-shuffled` × 12 seeds |
| **total** | **888** | |

Reuse is explicit and the analyser prints it: H25-2 pairs against `w22cov`
`intact`/`bin-shuffled`, H25-3 against `w22cov` rate arms, and H25-4 borrows the
two `e100` rate geometries wave 24 already produced. Every reused arm is on the
pinned binary above; §5 of
[`PREREG_2026-09-01_ORDER_SYNCHRONY_AND_BUDGET.md`](PREREG_2026-09-01_ORDER_SYNCHRONY_AND_BUDGET.md)
states the condition that voids such reuse and it is unchanged.

## 3. Hypotheses

Throughout, **DiD(X)** at a point is `(attn_intact − attn_X) − (rate_intact −
rate_X)` on seed-paired quadruples — the paper's estimator, unchanged.

**H25-1 — the recurrent read-out's advantage is order-dependent too.**
At h128 / `published-2ms` / `adjacent-sum-5` / `d32L4` / ss 0.4:
DiD(`bin-shuffled`) **> +0.03**, positive in **≥ 9 of 12**, *and*
DiD(`reversed`) **< +0.03**.
Both halves are required. The wave-21/22 bar and the wave-24 bar, unchanged and
used here for the first time on a recurrent arm.

**H25-2 — is the synchrony term a resolution effect?** *Registered as a
question, not a prediction.* At `fixed-t100` and `fixed-t500`, is
**DiD(`channel-shuffled`) − DiD(`bin-shuffled`) > +0.03**, as it is at
`fixed-t250` (+0.1038)? The campaign has one point and no basis for a
directional prediction at the others.

**H25-3 — does the contrast depend on read-out depth?** The wave-22 bar,
unchanged: across the points that vary only depth against their `d32L4` twin at
the same width on the anchor, the DiD range is **within 0.10**. *A question, not
a prediction* — this is H22-3 re-asked with the cells it needed.

**H25-4 — the contrast survives the second budget across the design space.**
At each of the nineteen points, at `e100`: DiD **> +0.03**, positive in **≥ 9 of
12**. The H22-1 bar unchanged.

**H25-5 — coverage is mechanical.** All 888 cells land with `mechanical_status`
clean and `non_finite_forward: 0`.

## 4. Named outcomes, in every direction

| outcome | reading |
|---|---|
| **H25-1 MET** | The mechanism claim covers both substrates and §3.7's cross-reference becomes supported rather than assumed. This is the outcome that closes a hole a reviewer would otherwise open. |
| **H25-1 NOT MET on the shuffle half** | The recurrent read-out's advantage is **not** order-dependent, §3.7's interpretive sentence is **withdrawn**, and the paper's mechanism claim is explicitly feed-forward. That is a narrower paper and an honest one. |
| **H25-1 NOT MET on the reversal half** | The recurrent arm's shuffle cost *is* displacement brittleness even though the feed-forward arm's is not. That would be the most interesting result in the wave and it gets its own subsection. |
| **H25-2 > +0.03 at both rungs** | Synchrony is read across the `fixed-tN` ladder and not at `published-2ms`. The mechanism has a resolution-dependent second term, and the abstract says "order and synchrony" with the axis named. |
| **H25-2 inside the band at both** | `fixed-t250` is a single point. It is reported as such, and the abstract's hedge stands rather than widening. |
| **H25-2 split** | One rung is not an axis either. Reported as unresolved, with both numbers. |
| **H25-3 within 0.10** | The contrast is a property of the read-out and not of its depth, and H22-3 is answered rather than abandoned. |
| **H25-3 outside 0.10** | Depth is a scope axis for the mechanism claim, which nothing in the paper currently suggests. |
| **H25-4 MET at all nineteen** | The budget scope limit is retired, not narrowed, and §4.6 loses a sentence. |
| **H25-4 NOT MET at some** | The points where it fails are named in the abstract, and the mechanism acquires a budget boundary. |

## 5. Cost, and the basis for it

From measured median `wall_secs` of `w22cov` / `w24ord` cells at the same
configurations, on the same fleet and binary — not from `estimate_cost.py`.

| group | cells | slot-hours |
|---|---:|---:|
| H25-1 recurrent | 72 | 259 |
| H25-2 synchrony | 48 | 241 |
| H25-3 depth twins | 96 | 334 |
| H25-4 budget | 672 | 661 |
| **total** | **888** | **1,495** |

- **≈ $66** of EC2 spot at the calibrated $0.0444/slot-hour — the rate that
  predicted $108 against wave 22's actual $107.66, and $26 against wave 24's.
- **Makespan ~18.5 h**, and it is set by one cell rather than by the fleet: the
  `fixed-t500` attention cell runs **18.3 h** on its own. Six boxes give 15.6 h
  of ideal packing against that 18.3 h floor, so a seventh buys minutes. Six is
  the choice, and the reason is written here rather than in a commit message.

## 6. What this wave will not answer

- **H25-1 is one operating point on one recurrent arm** — `rec+alif` at h128,
  ss 0.4. It does not cover `rec+fixed`, other widths, or other depths.
- **`channel-shuffled` is run at three of twenty-one points** even after this
  wave, all of them h128.
- **Two budgets, still not a curve.**
- **H25-3 re-asks H22-3; it does not re-run wave 22.** The depth comparison is
  between this wave's twins and wave 22's depth points, both on one binary.
- **Nothing here addresses the gain/DiD dissociation**, which remains the
  paper's leading open problem and has no design.
