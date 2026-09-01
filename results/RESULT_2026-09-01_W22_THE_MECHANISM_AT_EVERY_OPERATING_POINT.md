# Result — wave 22: the mechanism is present at every operating point, and it is not the gain

**Registered:** [`PREREG_2026-08-29_THE_MECHANISM_AT_EVERY_OPERATING_POINT.md`](PREREG_2026-08-29_THE_MECHANISM_AT_EVERY_OPERATING_POINT.md),
committed in `7fb7a70` **before any cell of this wave existed and before any
instance for it was launched**. Attested by git history, not by mtimes.
**Analyser:** `scripts/aws/analyse_wave22.py`, frozen in the same commit. Its
output is the authority on H22-1 and H22-3. H22-2 is
`scripts/mechanism_coverage.py`, as the preregistration specifies.
**Ran:** 2026-08-30 to 2026-09-01, on the `binn-campaign-v2` fleet, pinned
binary `3afd4434431a75a2…` — the first campaign binary carrying the
forward-finiteness guard.
**Status:** complete — **504/504 cells settled, 0 invalid, 0 failed**, nothing
retried, no threshold moved, no arm extended.

**H22-1 MET at all twelve points. H22-2 MET. H22-4 MET. H22-3 is NOT
EVALUABLE, structurally and permanently** —
[`DEFECT_2026-08-31_H22_3_CANNOT_BE_EVALUATED.md`](DEFECT_2026-08-31_H22_3_CANNOT_BE_EVALUATED.md).

---

## 1. What was asked

`PAPER_DRAFT.md` disclosed that the difference-in-differences — the
`bin-shuffled` destruction control that carries the paper's mechanism claim —
had been measured at only **9 of 21** covered operating points, and that twelve
points "claim nothing". This wave measures the remaining twelve.

It ran **all four arms itself**. Pairing new shuffled halves against archived
intact halves would have built every DiD out of two binaries, one predating the
forward-finiteness guard
([`DEFECT_2026-08-29_THE_EVALUATION_FORWARD_WAS_NEVER_CHECKED.md`](DEFECT_2026-08-29_THE_EVALUATION_FORWARD_WAS_NEVER_CHECKED.md)),
and `bootstrap.sh` states the rule: *"a campaign whose cells came from more than
one binary is not one experiment."* Self-containment cost 2.8x.

## 2. What the cells say

Every DiD is seed-paired: `(attention intact − attention shuffled) − (rate
intact − rate shuffled)`, per seed, then averaged. n = 12 per point.

| point | read-out | gain | **DiD** | positive |
|---|---|---:|---:|---:|
| h128 / `published-2ms` / `channels-700` | `d32/L1` | +0.0243 | **+0.1369** | 12/12 |
| h128 / `published-2ms` / `adjacent-sum-5` | `d64/L4` | +0.1379 | **+0.1331** | 12/12 |
| h128 / `fixed-t100` / `adjacent-sum-5` | `d32/L4` | +0.1927 | **+0.1323** | 12/12 |
| h128 / `fixed-t500` / `adjacent-sum-5` | `d32/L4` | +0.1474 | **+0.1161** | 12/12 |
| h128 / `published-2ms` / `adjacent-sum-5` | `d32/L2` | +0.0835 | **+0.1145** | 12/12 |
| h128 / `fixed-t250` / `adjacent-sum-5` | `d32/L4` | +0.1751 | **+0.1119** | 12/12 |
| h512 / `published-2ms` / `adjacent-sum-5` | `d32/L1` | +0.0043 | **+0.0893** | 12/12 |
| h256 / `published-2ms` / `adjacent-sum-5` | `d32/L1` | +0.0150 | **+0.0758** | 12/12 |
| h1024 / `published-2ms` / `adjacent-sum-5` | `d32/L1` | **−0.0159** | **+0.0675** | 12/12 |
| h1024 / `published-2ms` / `adjacent-sum-5` | `d32/L3` | +0.0324 | **+0.0675** | 11/12 |
| h1024 / `published-2ms` / `adjacent-sum-5` | `d32/L2` | +0.0392 | **+0.0658** | 12/12 |
| h768 / `published-2ms` / `adjacent-sum-5` | `d32/L2` | +0.0419 | **+0.0610** | 12/12 |

Every DiD was recomputed by a second implementation that imports nothing from
the analyser — separate parsing, separate pairing, separate arithmetic — and
agrees **to the digit on all twelve, including the 11/12 positive count**.

## 3. Verdicts

**H22-1 — MET at all twelve.** Every point clears the registered +0.03 bar and
the 9-of-12 positive floor. The smallest DiD is **+0.0610**, twice the bar; the
weakest positive count is **11 of 12**. Twelve of twelve points evaluable
against a floor of 9 seed-paired quadruples each.

**H22-2 — MET.** `mechanism_coverage.py` reports **21 of 21** operating points
able to support the difference-in-differences, and **0** carrying intact arms
with no `bin-shuffled` control. Covered widths 128–1024, five contracts, two
geometries.

**H22-4 — MET.** DiD exceeds +0.03 at all three `fixed-tN` rungs: **+0.1323**
(t100), **+0.1119** (t250), **+0.1161** (t500). *The frozen analyser prints no
H22-4 verdict* — it prints those three numbers inside H22-1's table, and the
verdict follows from them mechanically. Spearman ρ between the three DiDs and
their three gains is **+0.5 exactly** — at n = 3 the statistic can only take
values in {−1, −½, 0, +½, +1}, so it is quoted at the precision it has rather
than padded to four places. The preregistration declares it **descriptive and
carrying no verdict**.

**H22-3 — NOT EVALUABLE, and no cell could have changed it.** The analyser
compares each depth point against a `d32l4` twin on the anchor contract, drawn
from this wave's own cells; the wave contains **no** `d32l4` cell at
`published-2ms`/`adjacent-sum-5` at any width. The preregistration's H22-3
assumed archived twins and its §3 then forbade archived cells outright — **the
isolation that makes every other verdict sound is what starves this one.** The
analyser was not edited, no cells were added, and no archived twin was
substituted; after twelve MET results each of those is indistinguishable from
tuning to the data. It was registered as *a question, not a prediction*, so
nothing is refuted.

## 4. What this establishes

**The mechanism claim now rests on 21 of 21 operating points, not 9.** The
manuscript's "9 of 21" disclosure and its "twelve claim nothing" caveat are
retired. Destroying temporal order inside the bin costs the attention arm more
than it costs the rate arm at **every width, every temporal contract and both
geometries tested** — the read-out consumes temporal order wherever it was
measured.

**And the mechanism is not the gain.** Wave 21 registered that the DiD and the
gain are unrelated (Spearman ρ = −0.1430 at n = 6 against a bar of +0.829).
Twelve points sharpen it past correlation into dissociation:

- **h1024 / `d32/L1` gains −0.0159 — the read-out makes accuracy *worse* — and
  its DiD is +0.0675 in 12 of 12 seeds.**
- h512 / `d32/L1` gains **+0.0043**, indistinguishable from nothing, and carries
  a DiD of **+0.0893**.
- h128 / `channels-700` gains **+0.0243** and carries the wave's **largest** DiD,
  **+0.1369**.

The DiD range across the twelve is **+0.0610 to +0.1369**, a spread of
**0.0759**, while the gains over the same points run from −0.0159 to +0.1927.
**A read-out can consume temporal order while buying no accuracy at all.**
Wave 21 found this at h1024 and the manuscript records it as the paper's
leading open problem; wave 22 shows it is **not an h1024 pathology** — it
recurs at h512 and at `channels-700`, at widths and geometries where nothing
collapses.

## 5. What this does NOT establish

1. **It does not explain the dissociation.** That the read-out destroys order
   without buying accuracy is now measured at more points, not accounted for.
   It remains the paper's leading open problem and this wave widens it.
2. **It does not say the contrast is depth-independent.** That is H22-3 and it
   is unevaluable. Nothing here licenses a depth claim in either direction.
3. **It is not a new headline.** The paper's headline stays `h128` `d32/L4` at
   e400. These twelve points are **coverage**, not a competing result.
4. **It does not revisit the gain-versus-DiD correlation.** Wave 21 refuted that
   at n = 6 with a registered bar; the ρ = +0.5 here is over three rungs and
   the preregistration forbids reading a verdict into it.
5. **It says nothing at budgets other than e400.** Wave 23 showed the h1024
   collapse is a late-training phenomenon, and **no `bin-shuffled` arm has ever
   run at e100**. Every DiD above is an e400 measurement.

## 6. Provenance

**One binary, `3afd4434431a75a2…`, across all 504 cells**, and it is the first
campaign binary that checks its evaluation forward for finiteness. **Every cell
carries `non_finite_forward: 0` and `non_finite_events: 0`** — the guard was
present throughout and never fired, so no accuracy above is silently corrupted
by a poisoned forward.

**56 of 504 cells are `CELL_PASS`.** That is expected and is not a defect: the
predicate requires `accuracy >= 0.80`, which most `bin-shuffled` arms and most
h1024 cells do not reach by construction — the shuffled arm is *designed* to be
degraded and h1024 is in late collapse. Validity is gated on
`mechanical_status`, which is `COMPLETE` for all 504, and
`scripts/cell_validity.py` reports **576 usable, 0 INVALID** across the campaign.

**The fleet self-terminated.** All four instances ended `User initiated` — their
own `shutdown -h now` on finding the queue drained — so no teardown was needed
and nothing accrued after the last cell. `bootstrap.sh:162` records that on
every earlier campaign *"the instances never self-terminated once"*; on this one
they all did.

Cross-machine Gate F **FAILs** on every instance, by design and as disclosed:
the recorded cells are macOS/aarch64 and libm is not obliged to agree to the
last ulp. **No verdict above rests on it.** Every DiD is a within-wave contrast
among four arms that ran on the same fleet, on the same binary, beside each
other.

Total compute: **2,423 slot-hours** over 504 cells.

## 7. Reproduce

```bash
python3 scripts/aws/analyse_wave22.py
python3 scripts/mechanism_coverage.py
```
