# Azure campaign — stopped by the operator at 95 of 252 cells, out of credit

**Prereg:** `PREREG_2026-08-20_AZURE_D32L4_SCOPE.md`.
**Ran:** 2026-08-21 06:45Z → 2026-08-22 01:44Z, 4 × `Standard_D64als_v7`, x86-64,
binary `666a73420a63…`.
**Outcome:** **95 / 252 cells.** Fleet deallocated by the **operator**, who ran out
of Azure credit.

> **CORRECTION 2026-08-22.** This document first attributed the truncation to the
> 19-hour watchdog. **That was wrong**, and the operator supplied the actual
> cause. The inference is worth recording as a cautionary case: the campaign
> booted ~06:44Z, the watchdog was calculated in advance to fire at ~01:44Z, the
> last result landed **01:43:25Z**, and all four nodes were found `deallocated`.
> The prediction and the observation matched to under a minute — and the
> prediction was still not the cause. **A story that fits the evidence this
> tightly is exactly the kind that should have been checked rather than
> concluded**, and from outside the subscription it was not distinguishable:
> an operator-initiated deallocation and a watchdog deallocation leave the same
> trace.
>
> **Nothing downstream changes.** Every result below concerns *which cells exist*,
> not why the fleet stopped.

---

## 1. What happened

The operator deallocated the scale set at ~01:44Z after exhausting the Azure
credit budget. The last result was written at **01:43:25Z**; **157 cells never
ran.**

### The watchdog defect is still real, and is still unfired

`RECONCILIATION_2026-08-21_TWO_PREREGS_ONE_QUESTION.md` §4 flagged that
`bootstrap.sh:19` sets a **19-hour fleet watchdog** while the runner passes a
**24-hour per-cell timeout**, and estimated the makespan at ~22.5 h against that
19 h deadline.

That mismatch remains a latent defect and **has never been observed to fire**: a
per-cell timeout longer than the fleet's own lifetime can never be reached, so a
cell exceeding 19 hours is guaranteed to be destroyed without a record rather
than to time out with one. The makespan straddle is likewise **unresolved** — the
campaign was stopped before the question could be answered.

Both should be fixed before any successor Azure run: raise the watchdog above the
registered makespan, or lower the cell timeout beneath the watchdog, so that the
two cannot disagree.

## 2. What the truncation cost — the primary hypothesis

Longest-processing-time-first scheduling runs the most expensive cells first. It
minimises makespan and it put the **cheapest, most important** arms last:

| arm | cells |
|---|---|
| `az8wid` `ff+fixed` h128/e400 — **the primary baseline** | **0 / 12** |
| `az8wid` `ff+fixed+attn` h128/e400 d32L4 — **the primary treatment** | **0 / 12** |
| `az8geo` both arms, `channels-700` | **0 / 12** each |
| `az8con` all four control arms | **0 / 12** each |

**AZ8-1, the campaign's headline question — does the result replicate on x86? —
has no data.** So do AZ8-3 (geometry) and AZ8-4 (budget stability). AZ8-5 has one
treatment arm (`fixed-t250`, 12/12) and **no control for it**, so it is not
evaluable either.

Five arms completed 12/12, and they are the expensive ones: h1024 d32L4, h1024
rate-only, h1024 d64L4, h512 d32L4, and `fixed-t250` d32L4.

## 3. Registered verdicts

| ID | status |
|---|---|
| AZ8-1 x86 replication | **NO DATA** — 0/24 cells |
| **AZ8-2 width scope** | **NOT SUPPORTED** — see below |
| AZ8-3 geometry scope | **NO DATA** — 0/24 cells |
| AZ8-4 budget stability | **NO DATA** |
| AZ8-5 timing scope | **NOT EVALUABLE** — one treatment arm, no control |
| **AZ8-6 d32 bottleneck at h1024** | **VOIDED** — see below |

### AZ8-2 — NOT SUPPORTED, and identical to AWS

| arm | mean |
|---|---:|
| h1024 rate-only | 0.73858952 |
| h1024 d32/L4 | 0.57678151 |
| **gain** | **−0.16180801**, positive in **1/12** seeds |

Bar was ≥ +0.05 and positive in ≥ 9/12. **NOT SUPPORTED.**

This is the same verdict AWS wave 8's S-3 reached — and, as
`FINDING_2026-08-22_REPRODUCIBLE_ACROSS_ISA_UNDER_GLIBC.md` shows, **not merely
the same verdict but the same numbers to every serialised digit**. It is therefore
a *reproducibility* check, not an independent replication, and must not be
reported as a second observation of the effect. **N remains 12, not 24.**

### AZ8-6 — VOIDED, not a verdict

The `d64/L4 at h1024` arm is **degenerate in 6 of 12 seeds**, failing the
preregistered gates on `classes_predicted` and `majority_prediction`:

| seed | accuracy | gate |
|---|---:|---|
| 5170001 | 0.0716 | classes=14, majority=0.373 |
| 5170003 | 0.0755 | classes=13 |
| 5170005 | 0.0534 | classes=15, majority=0.382 |
| 5170007 | 0.0570 | **classes=9, majority=0.826** |
| 5170008 | 0.0592 | classes=18, majority=0.369 |
| 5170010 | 0.0517 | classes=15, majority=0.568 |

The arm's raw mean is 0.1278 against a chance of 0.05, which would have computed
to a "gain" of −0.6108. **That number is not reported as a verdict**, because half
its cells are collapsed readouts. Per the prereg — *"missing, timed-out, or
invalid cells are reported as such and never silently dropped"* — AZ8-6 is
**VOIDED**.

The degeneracy is itself informative and is offered only as an observation: a
d64 read-out at h1024 does not merely fail to help, it **destabilises training**.
That is consistent with AZ8-2's −0.1618 and with wave 8's S-3, and it is not a
registered finding.

## 4. What was salvaged

The truncation produced one thing worth more than the plan it destroyed. The
surviving expensive arms are exactly the ones AWS had already run, which made a
**36-cell, 57,960-value cross-architecture comparison** possible —
aarch64 vs x86-64, different binaries, **zero differing values**. See
`FINDING_2026-08-22_REPRODUCIBLE_ACROSS_ISA_UNDER_GLIBC.md`.

That answers AZ8-1's underlying question more strongly than AZ8-1 would have, and
it happened by luck rather than design.

## 5. Not done, and deliberately

**The campaign cannot be relaunched: the Azure credit is exhausted.** This was
originally recorded as a judgement call — that restarting would cost credit and
re-open a registered matrix, while AZ8-3 was already settled by wave 8 on aarch64
and AZ8-1 by the reproducibility finding. Those reasons still hold, but they are
no longer the operative ones. **The missing 157 cells are not available at any
price on this subscription**, and `az8con`'s four unrun contracts at e400 are a
permanent gap in this campaign rather than a pending decision.

If those contracts matter to the paper they must be run somewhere else. AWS has
capacity and its binary is already pinned; the cross-ISA reproducibility finding
means an AWS run would be directly comparable to the Azure cells that did
complete.

## 6. Scope

- **Verified:** the 95/252 count, the arm-by-arm coverage, the deallocation state
  of all four nodes, and every number above, from the campaign's own blob
  container this session.
- **Verified:** the 6/12 degeneracy, per seed, against the preregistered gates.
- **Not claimed:** any AZ8-6 effect size. The arm is void.
- **Not claimed:** that AZ8-2 independently replicates AWS. It reproduces it.

### Re-derivable from the repository, 2026-08-25

When this document was written, the numbers above were checked against the
campaign's blob container, which is no longer reachable — the subscription's
credit is exhausted and the fleet is deallocated. The 95 per-cell results are
now committed under
[`azure-d32l4-scope-v1/results/`](azure-d32l4-scope-v1/results/), so every figure here is
re-derivable without it.

Re-derived from those files on 2026-08-25, all matching to the digits printed
above: the 95-cell count and per-arm coverage; AZ8-2's two arm means
(`0.73858952`, `0.57678151`), its `-0.16180801` gain and its 1/12 positive
seeds; AZ8-6's raw mean `0.1278` and would-be `-0.6108`; and the six degenerate
seeds, whose identities fall out of the prereg's validity gate — all 20 classes
predicted, majority prediction below 0.30 — applied verbatim rather than
restated. Those six are the only invalid cells among all 95. The `NO DATA`
verdicts check as absences: zero cells at h128/e400/`published-2ms` for either
AZ8-1 arm, and zero at `channels-700` for AZ8-3.

The deallocation state of the four nodes is **not** re-derivable this way. It
was observed against the live subscription and rests on `hosts/`.
