# Geometry is not the ceiling's binding scope limit — but the registered budget-artefact prediction failed, and Stage 2 never ran

**Protocol:** [`PREREG_2026-08-04_SHD_GEOMETRY_AT_CONVERGENCE.md`](PREREG_2026-08-04_SHD_GEOMETRY_AT_CONVERGENCE.md),
committed `d2a8c17` at **2026-08-04 23:09 −0700**, before any cell of this stage
existed. The three Stage 1 cells were written **2026-08-05 05:28–05:29**. The
ordering is version-controlled, not attested by mtime alone.
**Cells:** Stage 1 **3 of 3**, 0 voided. Stage 2 **0 of 6** — see §5.
**Backend:** rust, macOS, `ff+fixed`, `published-2ms`, `--temporal intact`.
**Archive:** [`shd_instrument_v4/geometry-converged/`](shd_instrument_v4/geometry-converged/).

**Two of four hypotheses supported. The registered prediction of §2 — the one the
prereg wrote down specifically so that it could fail — failed, by 0.0024.**

This document is the first to issue a verdict for G1, G2, G3 or G0 by ID. The
numbers themselves were reported informally in
[`AMENDMENT_2026-08-03_H1_AT_CONVERGED_BUDGET.md`](AMENDMENT_2026-08-03_H1_AT_CONVERGED_BUDGET.md) §4B;
they reproduce exactly from the cells (§4), but that document's closing sentence
does not (§6).

---

## 1. The measurements

All statistics computed from the cells, using the repo's existing convention —
`mean`, `stdev` (ddof=1) and `ci95` (Student t, two-sided 95%, df = n−1 = 2, so
t = 4.303) copied verbatim from
[`scripts/temporal_campaign_verdict.py`](../scripts/temporal_campaign_verdict.py):38–57,
and the five per-cell validity gates copied verbatim from
[`scripts/aws/analyse_wave8.py`](../scripts/aws/analyse_wave8.py)`::validity_problems`.

| configuration | n | mean | sd | 95% CI (t, df=2) |
|---|---:|---:|---:|---|
| **`channels-700` / h512 / e400** *(Stage 1)* | 3 | **0.708628** | 0.004104 | [0.698432, 0.718824] |
| `adjacent-sum-5` / h512 / e400 *(matched comparator)* | 3 | 0.736896 | 0.002087 | [0.731711, 0.742082] |
| `channels-700` / h512 / e100 *(e100 anchor)* | 3 | 0.689193 | 0.007099 | [0.671556, 0.706830] |
| `adjacent-sum-5` / h512 / e100 *(e100 anchor)* | 3 | 0.715106 | 0.003926 | [0.705353, 0.724859] |

Per-cell, by filename:

| cell | accuracy |
|---|---:|
| `geometry-converged/ff-fixed__channels-700__h512__e400__s5170001.json` | 0.709806 |
| `geometry-converged/ff-fixed__channels-700__h512__e400__s5170002.json` | 0.704064 |
| `geometry-converged/ff-fixed__channels-700__h512__e400__s5170003.json` | 0.712014 |
| `probe/budget__published-2ms__adjacent-sum-5__h512__e400__s5170001.json` | 0.734541 |
| `probe/budget__published-2ms__adjacent-sum-5__h512__e400__s5170002.json` | 0.737633 |
| `probe/budget__published-2ms__adjacent-sum-5__h512__e400__s5170003.json` | 0.738516 |

The e100 anchor rows come from `shd_instrument_v4/cells/rust__published-2ms__{geometry}__h512__e100__s517000{1,2,3}.json`.
The comparison ceiling figure **0.737780** (`adjacent-sum-5` / h1024 / e400,
`width-converged/ff-fixed__h1024__e400__s517000{1,2,3}.json`) is recomputed here
and matches the recorded 0.7378.

**Gap, `adjacent-sum-5` − `channels-700`:** 0.025913 at e100, **0.028269** at e400.
The two e400 CIs are **disjoint**.

## 2. Verdicts

| ID | statement | registered threshold | measured | verdict |
|---|---|---|---:|---|
| **G1** *(primary)* | `channels-700` does not clear the registered gate | mean < 0.80 **and** 95% CI (t, df=2) entirely below 0.80 | mean **0.708628**, CI upper **0.718824** | **SUPPORTED** |
| **G2** | geometry does not rescue the shortfall | `channels-700` ≤ `adjacent-sum-5` + 0.02 | 0.708628 ≤ **0.756896**, margin **+0.048269** | **SUPPORTED** |
| **G3** | the e100 geometry gap is substantially a budget artefact | gap at e400 **< 0.025913** | gap **0.028269**, margin **−0.002356** | **NOT SUPPORTED** |
| **G0** | geometry was load-bearing | `channels-700` − `adjacent-sum-5` ≥ +0.02 | **−0.028269** | **NOT SUPPORTED** |

G1 and G2 are the confirmatory pair and both hold, which is the prereg §6 outcome
labelled *"the expected outcome"*. G0 is the complement of G2 and fails with it.

## 3. G3 failed, narrowly, and the threshold was not moved

G3 is the prereg's §2 registered prediction, written down with a mechanism —
`channels-700` presents 700 input channels against 140, so it has 5× the input
parameters to fit from the same 8156 training samples, and at a fixed short budget
that alone would make it look worse. The prediction was that the gap would
**narrow** by e400.

**It did not narrow. It grew, from 0.025913 to 0.028269.**

The margin is **0.002356** — about 9% of the threshold, and smaller than the sd of
either e400 arm. **This is a near-threshold result and is reported as one.** The
registered rule is a strict inequality on a single pre-existing number, the
measured value is on the wrong side of it, and the verdict follows the rule. No
equivalence band was added after the fact, and none was registered.

What that licenses and what it does not:

- **Licensed.** The prereg §2 states the contrapositive in advance: *"if it is
  information-bearing it should be stable or grow with training; if it is a
  fitting artefact it should shrink."* It grew. The geometry gap is **not**
  explained away as a short-budget fitting artefact at this budget and width.
- **Not licensed.** "The gap grows with training" as a trend claim. Two points
  (e100, e400) with a 0.0024 difference between them, at n=3 apiece, is not a
  trend. G3 is graded exploratory-confirmatory in the prereg §4 for exactly this
  reason and carries less weight than G1.

## 4. Prose versus cells: no discrepancy in the numbers

`AMENDMENT_2026-08-03_H1_AT_CONVERGED_BUDGET.md` §4B reports these same three
cells informally. Every figure it quotes reproduces from the cells:

| the amendment says | recomputed from cells | agrees |
|---|---|---|
| `channels-700` mean 0.708628 | 0.708628 | yes |
| `adjacent-sum-5` mean 0.736896 | 0.736896 | yes |
| `channels-700` CI [0.6984, 0.7188] | [0.6984, 0.7188] | yes |
| `adjacent-sum-5` CI [0.7317, 0.7421] | [0.7317, 0.7421] | yes |
| "0.0283 worse, CIs disjoint" | 0.028269, disjoint | yes |
| e100 anchor gap 0.025913 *(prereg §1)* | 0.025913 | yes |

**The arithmetic in the record is sound.** The problem is not a number. It is §6.

## 5. Stage 2 never ran — 0 of the 6 registered cells exist

Prereg §3.2 registers Stage 2 as `h512 / e800 / 3 seeds` (budget axis) and
`h1024 / e400 / 3 seeds` (width axis), each bounded by the same 0.01 final-doubling
constant. A repository-wide search for any `channels-700` cell at `h512/e800` or
`h1024/e400` returns **zero files**. The only `channels-700` cells above e100
anywhere in `results/` are the three Stage 1 cells and the h128 attention-campaign
cells, which are a different arm and a different registration.

**Skipping Stage 2 is a registered decision, not a budget excuse.** The prereg §3.2
escalation rule, with its 0.78 constant fixed before Stage 1 ran, says Stage 2 is
not required for the verdict when Stage 1's 95% CI upper bound is below 0.78. The
measured upper bound is **0.718824 < 0.78**. The rule is met and the skip is legal.

**The naming restriction is therefore in force, and it is absolute.** Prereg §6:

> If Stage 2 does not run, no figure from this campaign may be described as a
> **ceiling**, a **converged** value, or *"what `channels-700` reaches"*. The only
> permitted form is *"X at e400/h512"*.

So the only sentence this campaign supports is: **`channels-700` reaches 0.708628
± 0.010196 at `published-2ms` / h512 / e400 with `ff+fixed` on the rust backend.**

## 6. The one thing in the record that must be corrected

`AMENDMENT_2026-08-03_H1_AT_CONVERGED_BUDGET.md` §4B closes with:

> **Both geometries are now measured at convergence**, so the ceiling claim's
> scope qualifier narrows from "one contract, one geometry" to "one contract".

**That sentence is not supported by these cells, and it uses the exact word the
prereg §6 forbids.** `channels-700` is measured at **one** budget and **one**
width. Its budget axis and its width axis are both open — closing them is what
Stage 2 was for, and Stage 2 did not run. The amendment's §4 outcome text was
written after 2026-08-05 (the cells did not exist before then, and the file was not
committed until `a3dafd1`), so the restriction was already registered and committed
when that sentence was written.

The scope-narrowing conclusion also does not follow arithmetically, on the prereg's
own numbers. The prereg §3.2 fixes **+0.018551** as the largest per-doubling gain
ever recorded in this instrument. Two doublings therefore admit up to +0.037102,
and `0.708628 + 0.037102 = 0.745730`, which is **above** the `adjacent-sum-5`
figure of 0.736896. Nothing measured excludes `channels-700` converging *higher*
than `adjacent-sum-5`. That possibility is irrelevant to G1 — 0.745730 is still far
below 0.80, which is why the escalation rule can safely skip Stage 2 for the
*verdict* — but it is decisive for the *scope* claim, which is about relative
values, not about the 0.80 gate.

**Correct scoping, as of this document:** the ceiling claim's qualifier narrows
from *"one contract, one geometry, one budget, one width"* to *"one contract, with
the second geometry measured at one budget and width and found lower there."* It
does **not** narrow to "one contract".

## 7. Validity gates

**§5.1 registered per-cell gates — 3 of 3 PASS, 0 exclusions.** Every Stage 1 cell:
`classes_predicted = 20`, `majority_prediction` ∈ [0.0950, 0.0998] (bar < 0.30),
`silent_fraction = 0.0000` (bar ≤ 0.95), `saturated_fraction = 0.0000` (bar ≤ 0.05),
`non_finite_events = 0`. The three matched `adjacent-sum-5` comparator cells also
pass all five.

**§5.2 activity-regime disclosure (reporting, non-blocking).** Means over 3 seeds:

| | mean_firing_rate | silent_fraction | saturated_fraction |
|---|---:|---:|---:|
| `channels-700` / h512 / e400 | 0.178659 | 0.000000 | 0.000000 |
| `adjacent-sum-5` / h512 / e400 | 0.205765 | 0.000651 | 0.000000 |

`channels-700` runs **0.0271 lower in mean firing rate** — a 13% relative
difference — with both geometries far inside every gate and neither saturating.
This is the disclosure §5.2 exists to force, and it is a real regime difference,
not a gate failure. It is **not** offered as an explanation of the accuracy gap;
no such link is registered here, and testing one would need its own design.

Also disclosed, unregistered here and therefore carrying no verdict:
`tail_loss_improvement` is **−0.062187** mean for `channels-700` and **−0.073237**
for `adjacent-sum-5`. Under the W1-4 rule of a *different* prereg (worst > −0.02 =
CONVERGED) both would read UNDERTRAINED — including the geometry whose budget axis
is closed by the registered final-doubling rule at +0.000294. The two criteria
disagree at e400 for `adjacent-sum-5`, which is a known open question and not
something these three cells can settle.

**§5.3 determinism — UNDISCHARGED.** The prereg requires Gate F to return **13/13
BIT_IDENTICAL on the binary used, before the stage runs**, and names
`rust__published-2ms__channels-700__h512__e20__s5170002` as the fixture covering
this geometry. The current
`shd_instrument_v4/gate-f-rust/report.json` records **12 cells**, all
`adjacent-sum-5`, status PASS — and it **does not contain the `channels-700`
fixture**. `gate-f-rust/runs.jsonl` holds 23 historical runs, eleven of them
13-cell all-BIT_IDENTICAL, of which **two** (binaries `6f6dbbc9fd58…`,
`10df998c491c…`) do include that fixture. But the rows carry **no timestamps**, and
no stage log records which binary produced the Stage 1 cells, so **no 13/13 verdict
can be tied to the binary under test.** Per the project's own rule, this is
recorded as *the check did not run*, not as *the check passed*.

**§5.4 provenance — PARTIALLY DISCHARGED.** The cells are in the registered
location, `results/shd_instrument_v4/geometry-converged/`. **No stage log exists
anywhere in the repository** — the prereg is the only file that mentions
`geometry-converged` — so the required binary sha256 for this stage was never
recorded, and the cells carry no binary field of their own.

**§5.5 convergence rule — NOT APPLICABLE**, being Stage 2 only.

**§8 pairing / orders determinism — DISCHARGED.** The freshly written init orders
are **byte-identical** to the reused e400 orders, all three seeds:

| seed | sha256 (first 16) | bytes |
|---|---|---:|
| 5170001 | `d8ff262cb76d9b08` | 13049616 |
| 5170002 | `31babd900ece94b9` | 13049616 |
| 5170003 | `b43faa2d0d6f4b32` | 13049616 |

verified between `geometry-converged/init/orders-s{seed}.orders` and
`probe/orders/n8156-e400-s{seed}.orders`. The prereg §8 command writes the init
file as `n8156-e400-s$S.orders`; on disk it is `orders-s$S.orders`. **The
difference is in the filename only** — the content check the prereg actually
specifies passes, so the §3.1 pairing assumption holds and the design is not void.

**§0 authorization — the caveat stands, verified.** `shd_instrument_v4/gates.json`
records `matrix_authorized: false`, `clean_reference: false`,
`historical_reference: false`. This result **inherits threat §7.8 of
`SHD_BPTT_CEILING_NEGATIVE_RESULT.md`** and must not be cited as resting on a
currently-`VALID` harness.

## 8. What is NOT concluded

- **Not a ceiling.** No figure here is a ceiling, a converged value, or "what
  `channels-700` reaches". Stage 2 did not run; prereg §6 binds.
- **Not "both geometries are converged".** See §6. That is the sentence this
  document exists to correct.
- **Not a narrowing of the ceiling scope to "one contract".** See §6.
- **Not "the geometry gap grows with training".** G3 failing means the gap did not
  shrink below its e100 value. Two points 0.0024 apart is not a trend.
- **Not a statement about `channels-700` at any other width, budget, contract or
  arm.** One contract, one width, one budget, one arm, n=3.
- **Not generalisation of geometry beyond the two tested**, per `must_not_claim`.
- **Nothing about SOTA, Gate G2, biology, neuromorphic hardware, or BINN**, per
  `must_not_claim`.
- **No claim that `adjacent-sum-5` discards nothing.** G2 holding means the
  700-channel presentation does not *recover* enough to matter at this budget and
  width. It does not establish that the binning is lossless.

## 9. Shortfall against the registered design

| registered | delivered | status |
|---|---|---|
| Stage 1 — 3 cells, h512/e400, seeds 5170001–3 | 3 cells, exactly as specified | **complete** |
| Stage 2a — 3 cells, h512/e800 | **0 cells** | **not run** |
| Stage 2b — 3 cells, h1024/e400 | **0 cells** | **not run** |
| §5.1 per-cell gates | 3/3 pass, 0 exclusions | **complete** |
| §5.2 activity disclosure | reported §7 | **complete** |
| §5.3 Gate F 13/13 on the binary used | 12-cell report without the named fixture; no timestamped, attributable 13/13 | **undischarged** |
| §5.4 binary sha256 in a stage log | no stage log exists | **undischarged** |
| §8 orders byte-identity | verified, all 3 seeds | **complete** |

**n is not a shortfall.** Prereg §3.1 registers Stage 1 at exactly three cells and
three seeds; n=3 is the registered design, and the CI convention used here (t,
df=2, t = 4.303) is the one the prereg names and the one that widens honestly at
that n. The shortfall is **Stage 2's six cells**, and the two undischarged gates.

## 10. What would close the prereg completely

Three things, in order of what each buys:

1. **The six Stage 2 cells** — `channels-700` at h512/e800 and h1024/e400, 3 seeds
   each — ≈ 19 h by the prereg §7 estimate, but nearer **40 h** at the Stage 1
   rate measured in §11. Only these license a converged figure for
   `channels-700`, lift the §6 naming restriction, and settle whether the geometry
   scope qualifier may in fact be dropped. **They are what §6 of this document is
   waiting on.** Note that the escalation rule already licenses skipping them for
   the *verdict*; running them is now a question about *scope*, and needs its own
   registration to say so.
2. **A Gate F run of the full 13 fixtures, including
   `rust__published-2ms__channels-700__h512__e20__s5170002`, on a binary whose
   sha256 is recorded**, plus a stage log tying that sha256 to these three cells.
   Without it §5.3 and §5.4 stay undischarged and the determinism claim is
   unattributable rather than failed.
3. **The `matrix_authorized` decision** (`AMENDMENT_2026-08-03_REFERENCE_FINGERPRINT_SCOPE.md`),
   which is not closeable by code and needs a human decision or a reference re-run.
   Until then §0's caveat rides on every figure above.

None of the three changes the G1/G2/G3/G0 verdicts. Item 1 changes what may be
*said* about them; items 2 and 3 change how much weight the saying carries.

## 11. Cost, measured against the estimate

| | prereg §7 estimate | measured |
|---|---:|---:|
| `channels-700` / h512 / e400, per cell | ≈ 2925 s | **12890 s** |
| Stage 1 total | ≈ 5.3 h | **≈ 10.7 h** |
| geometry factor vs `adjacent-sum-5` at e400 | 2.19× *(pre-transpose)* | **1.446×** |

The prereg flagged the 2.19× factor as measured before the transposed kernel and
warned the true factor could move either way. It fell to 1.446×, while the
absolute per-cell time came in **4.4× the estimate** — the estimate's error was in
the `adjacent-sum-5` e400 baseline, not in the geometry factor. Recorded here
because §7 asked for actuals.
