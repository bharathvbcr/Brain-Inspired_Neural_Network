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
| `az8con` the four `ff+fixed` control arms | **0 / 12** each |

**AZ8-1, the campaign's headline question — does the result replicate on x86? —
has no data.** So do AZ8-3 (geometry) and AZ8-4 (budget stability). AZ8-5 has one
complete treatment arm (`fixed-t250`, 12/12), one partial one (`fixed-t500`,
9/12), and **no control for either**, so it is not evaluable.

Five arms completed 12/12, and they are the expensive ones: h1024 d32L4, h1024
rate-only, h1024 d64L4, h512 d32L4, and `fixed-t250` d32L4.

### CORRECTION 2026-08-25 — five arms were partially run, and this table hid them

The table above lists the complete arms and the empty ones. **It passed over 35
cells in five partially run arms**, and the sentence "`az8con` all four control
arms 0/12" was wrong on its face: 21 `az8con` cells exist, in two *treatment*
arms. The full coverage, arm by arm, is now generated rather than narrated —
[`azure-d32l4-scope-v1/VERDICT.md`](azure-d32l4-scope-v1/VERDICT.md) §"Coverage,
arm by arm", from `scripts/azure/analyse.py`:

| wave | arm | ran / planned | valid | mean |
|---|---|---:|---:|---:|
| `az8wid` | `ff+fixed` h512 | **10 / 12** | 10 | 0.735203 |
| `az8con` | `ff+fixed+attn` `fixed-t500` d32L4 | **9 / 12** | 9 | 0.852866 |
| `az8wid` | `ff+fixed+attn` h128 **e200** d32L4 | **8 / 12** | 8 | 0.831493 |
| `az8wid` | `ff+fixed` h256 | **4 / 12** | 4 | 0.722725 |
| `az8wid` | `ff+fixed+attn` h256 d32L4 | **4 / 12** | 4 | 0.818573 |

A partially run arm carries data and no registered verdict. Reporting it as
absent is the same error as reporting it as complete, and the verdict table's
single word "incomplete" cannot distinguish them — so the coverage table is now
emitted on every run and a test pins the five partial arms by name.

None of this moves a verdict. AZ8-1, AZ8-3 and AZ8-4 remain **NO DATA**: every
arm they need is at 0/12, which the table above confirms rather than assumes.

### What the partial arms are worth: the h256 rung

Four of the 35 cells were, when this section was written, the only ones of their
kind anywhere in the project.

> **SUPERSEDED 2026-08-26 — deliberately, and the rung's numbers stand.** Wave 16
> regenerates h256/d32-L4 at the full seed count, as
> [`PREREG_2026-08-25_THE_H1024_COLLAPSE.md`](PREREG_2026-08-25_THE_H1024_COLLAPSE.md)
> §5 registered it would: *"a rung at n=4 beside rungs at n=12 makes the
> ladder's shape depend on which rung you trust."* These four cells are no
> longer the evidence for the rung; they are now a cross-ISA check on it.
>
> That check has been run and is the strongest form of the reproduction claim so
> far, because it was **prospective**: the four Azure x86-64 cells were recorded
> 2026-08-21 and the aarch64 cells were produced 2026-08-26, and on the four
> shared seeds **6,520 leaf values agree with zero differences**, across all
> four complete 400-epoch trajectories. The 79-cell comparison in
> [`FINDING_2026-08-22_REPRODUCIBLE_ACROSS_ISA_UNDER_GLIBC.md`](FINDING_2026-08-22_REPRODUCIBLE_ACROSS_ISA_UNDER_GLIBC.md)
> derived its overlap from two archives that already existed; this one is a
> prediction the claim made about cells that did not.
>
> The rung's per-seed values are unchanged and the table below is not restated
> here — the wave-16 figures supersede it once that wave completes, and the
> ladder is rebuilt from twelve cells rather than four.

The d32/L4 width ladder was published in
[`RESULT_2026-08-21_W8_HEADLINE_SCOPE_IS_MEASURED.md`](RESULT_2026-08-21_W8_HEADLINE_SCOPE_IS_MEASURED.md)
at h128, h512 and h1024. **h256 at L4 had never been run on AWS** when this was
written, so these four cells were the whole of it. Paired by seed against the
`ff+fixed` h256 control (whose Azure and AWS copies are bit-identical, §4):

| width | pairs | `ff+fixed` | d32/L4 | gain | positive | source |
|---|---:|---:|---:|---:|---:|---|
| h128 | 12 | 0.706198 | 0.832008 | **+0.1258** | 12/12 | W8 |
| **h256** | **4** | **0.722394** | **0.818573** | **+0.0962** | **4/4** | **Azure only** |
| h512 | 12 | 0.735718 | 0.823322 | **+0.0876** | 12/12 | W8 |
| h1024 | 12 | 0.738590 | 0.576782 | **−0.1618** | 1/12 | W8 |

With the rung filled, the shape of the width scope is no longer a two-point
statement. **The gain decays gently across h128 → h256 → h512 and then
collapses between h512 and h1024** — it does not thin out steadily and cross
zero. The published reading, "the gain inverts by h1024", is true and is the
weaker of the two available.

> **SUPERSEDED 2026-08-27 by [`RESULT_2026-08-27_W15_17_THE_COLLAPSE_IS_A_THRESHOLD.md`](RESULT_2026-08-27_W15_17_THE_COLLAPSE_IS_A_THRESHOLD.md) §5.**
> Wave 16 added h384 and h768 at twelve seeds each. On the six-rung ladder the
> paragraph above is wrong in two ways, and both were invisible at four rungs:
>
> 1. **The decay is not strictly ordered.** h384 is +0.0760 and h512 is +0.0876,
>    so the sequence rises between them. The inversion is inside its own noise
>    (seed-paired difference −0.0116, sd 0.0253, negative in 7 of 12), so this is
>    a statement that h384 and h512 cannot be ordered at n=12 — not that there is
>    a dip.
> 2. **The collapse is between h768 and h1024**, not h512 and h1024. h768 is
>    still positive at **+0.0560**.
>
> The four numbers in the table above are unchanged and remain correct. The four
> Azure h256 cells are now **byte-identical to their AWS twins** across thirteen
> scientific fields, and the AWS gain over the same four seeds is **+0.0962** —
> this rung's number to the digit. At twelve seeds it is +0.0966. The rung is
> confirmed, not replaced, and it is now a cross-ISA check on the twelve rather
> than the evidence itself.

**This is descriptive and is not a verdict.** AZ8-2 was registered at h1024
only; no criterion was registered at h256 or h512, and **four pairs is a third
of the terminal seed count**, so the rung locates the collapse and does not
measure it. What it is good for is choosing where a successor campaign should
put its seeds, which is §5's business.

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
