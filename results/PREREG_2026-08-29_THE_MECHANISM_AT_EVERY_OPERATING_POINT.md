# Preregistration — the mechanism control at the twelve operating points that lack it

**Registered:** 2026-08-29, **before any cell of wave 22 exists and before any
instance for it is launched.** Attested by git history: this file and
`scripts/aws/analyse_wave22.py` are committed together, and the first cell is
produced afterwards.

**Analyser:** `scripts/aws/analyse_wave22.py`, frozen in the same commit as this
file, and the authority on every verdict below.

---

## 1. Why this wave exists

The paper's lead claim is a difference-in-differences: attention's bin-shuffle
cost against the rate read-out's own cost, on shared seeds.
`scripts/mechanism_coverage.py` recomputes on every gate run where that contrast
can actually be formed, and the answer is **9 of 21 operating points**. Wave 21
took it from 2 to 9 and is why the claim generalises at all.

**Twelve points still carry intact arms with no `bin-shuffled` twin.** The
manuscript, Figure 1 Panel D, and the lead graphical abstract all state "9 of
21" and disclose that twelve claim nothing. This wave is the attempt to remove
that sentence by measurement rather than by rewording.

## 2. Design

> **Amended before any cell existed, and the amendment is the whole design.**
> The first draft planned the **shuffled arms only** — 180 cells — and paired
> them against intact halves already in the corpus. That was correct as long as
> every cell in a contrast came from one binary, and it stopped being correct
> on 2026-08-29, when `shd_instrument.rs` gained the forward-finiteness guard
> ([`DEFECT_2026-08-29_THE_EVALUATION_FORWARD_WAS_NEVER_CHECKED.md`](DEFECT_2026-08-29_THE_EVALUATION_FORWARD_WAS_NEVER_CHECKED.md)).
>
> The corpus was produced by pinned binary `22d97c51ab02…`, which predates that
> guard. Running the shuffled halves on a new binary and pairing them against
> archived intact halves would build every difference-in-differences out of
> **two binaries**, and `scripts/aws/bootstrap.sh` states the rule: *"a campaign
> whose cells came from more than one binary is not one experiment."*
>
> **So this wave runs all four arms and answers for itself: 504 cells.** It
> costs 2.8× what it needed to and, as a side effect, re-measures twelve
> archived intact points on a second binary — an independent reproduction the
> corpus has never had. `analyse_wave22.py` admits only `w22cov` cells, so an
> archived cell at the same operating point and seed cannot be substituted by
> filename order.

Twelve points, `e400`, 12 seeds, on a new pinned binary carrying the guard. The
rate arm carries no read-out depth, so one `ff+fixed` pair serves every depth at
a given `(width, contract, geometry)`; there are **nine** distinct geometries
across the twelve points, which is why the rate arms number 216 and not 288.

The Group A / Group B split the first draft used is gone with it: it existed to
say which points needed one arm and which needed two, and now every point needs
all four. What remains is the nine geometries and the depths measured at each.

| geometry | read-out depths | attention cells | rate cells |
|---|---|---:|---:|
| `h128` / `fixed-t100` / `adjacent-sum-5` | `d32/L4` | 24 | 24 |
| `h128` / `fixed-t250` / `adjacent-sum-5` | `d32/L4` | 24 | 24 |
| `h128` / `fixed-t500` / `adjacent-sum-5` | `d32/L4` | 24 | 24 |
| `h128` / `published-2ms` / `adjacent-sum-5` | `d32/L2`, `d64/L4` | 48 | 24 |
| `h128` / `published-2ms` / `channels-700` | `d32/L1` | 24 | 24 |
| `h256` / `published-2ms` / `adjacent-sum-5` | `d32/L1` | 24 | 24 |
| `h512` / `published-2ms` / `adjacent-sum-5` | `d32/L1` | 24 | 24 |
| `h768` / `published-2ms` / `adjacent-sum-5` | `d32/L2` | 24 | 24 |
| `h1024` / `published-2ms` / `adjacent-sum-5` | `d32/L1`, `d32/L2`, `d32/L3` | 72 | 24 |

Each cell count is 12 seeds × 2 temporal conditions per arm.

**504 cells**: 288 attention (12 points × 2 conditions × 12 seeds) and 216 rate
(9 geometries × 2 conditions × 12 seeds). Every arm of every contrast is
produced by this wave on one binary.

The archived intact halves are **not** used for any verdict. They are compared
against the new ones descriptively, as a reproduction check, and that comparison
carries no registered hypothesis — it crosses two binaries by construction and
is reported as an observation, not a result.

### The `fixed-tN` group is the one that can surprise

Figure 4 reports that the *gain* falls as bins get finer — `−0.0453` from t100
to t500 — with the rate arm rising inside a 0.05 confound bar. **Nothing is
known about the shuffle cost on that axis.** If the read-out's contribution is
order-dependent, the difference-in-differences should survive at every rung;
if it collapses as resolution rises, the resolution finding and the mechanism
finding are the same finding, which the paper currently treats as separate.

## 3. Hypotheses

**H22-1 — the contrast clears its bar at every new point.**
Per point, DiD **> +0.03**, positive in **≥ 9 of 12** seed quadruples. This is
the wave-21 bar unchanged, deliberately: a bar moved between waves measuring the
same quantity is a bar chosen after seeing data.

**H22-2 — coverage reaches 21 of 21.**
`scripts/mechanism_coverage.py` reports 21 covered points after collection.
Mechanical, not scientific: it can fail only by cells failing to land.

**H22-3 — the contrast does not depend on read-out depth.**
Across the six points that vary only depth (`d32/L1`, `d32/L2`, `d32/L3`,
`d64/L4` against their `d32/L4` twins at the same width), the DiD range is
**within 0.10**. *This is registered as a question, not a prediction* — the
campaign has never varied depth with the shuffle control present, and wave 18
showed depth behaves unlike width at h1024.

**H22-4 — the contrast survives the resolution ladder.**
DiD > +0.03 at all three `fixed-tN` rungs, and Spearman ρ between the three
DiDs and their three gains is **not** required to be anything. The gain-versus-DiD
relationship was refuted at n=6 across width (ρ = −0.1430 against +0.829) and
three rungs cannot revisit it; any ρ computed here is descriptive and carries no
verdict.

## 4. Named outcomes, in every direction

| outcome | reading |
|---|---|
| H22-1 MET at all twelve | Coverage is complete. The manuscript's "9 of 21" and its "twelve claim nothing" disclosure are replaced by "21 of 21", and Figure 1 Panel D gains twelve points. |
| H22-1 MET at some, NOT MET at others | **The more useful outcome.** The mechanism has a boundary, and the boundary is where the paper's claim ends. The failing points are named in the abstract, not in a footnote. |
| H22-1 NOT MET on the `fixed-tN` rungs specifically | The read-out's contribution is order-dependent at coarse binning and not at fine, which would tie §3.6's resolution result to the mechanism and is a stronger paper than the one currently written. |
| H22-3 shows depth dependence beyond 0.10 | The lead claim is scoped to a read-out depth as well as to a width. Table SHD-2b gains a depth column and the abstract gains a qualifier. |
| Fewer than 9 of 12 quadruples at a point | That point is **NOT EVALUABLE**, reported as such, and coverage stays below 21. No partial-credit verdict. |

## 5. Stopping rule

504 cells, once. **No point is re-run to improve its verdict**, and no seed is
added beyond twelve at any point — the campaign's n=32 confirmations exist only
at the anchor and are not extended here. A point whose cells fail
`scripts/cell_validity.py` is reported with its exclusion count, not topped up.

## 6. What may not be claimed

1. **Coverage is not strength.** 21 of 21 would mean the contrast was measurable
   everywhere it was asked, not that the effect is larger or better established.
   The size of the effect is separately refuted as tracking the gain
   (ρ = −0.1430) and that stands.
2. **No new headline.** The headline stays the anchor at n=32. Nothing here
   moves it.
3. **Nothing about h1024's collapse.** Three of these points are at h1024 and
   all are intact-arm depths that do **not** collapse; they say nothing about
   `d32/L4`, which is wave 23's question.
4. **The `fixed-tN` rungs do not reopen S-5.** That hypothesis was refuted and
   withdrawn on the `published-Nms` family for confounding bin width with
   sequence length. `fixed-tN` holds the window fixed, which is why it is the
   axis used here, and no `published-Nms` number is compared against it.
