# Result — synchrony is an axis, depth is a scope limit, the budget limit is gone, and the recurrent question could not be asked

**Registered:** [`PREREG_2026-09-02_THE_MECHANISM_WHERE_IT_IS_UNMEASURED.md`](PREREG_2026-09-02_THE_MECHANISM_WHERE_IT_IS_UNMEASURED.md)
at `d91fe72`, **before any cell of this wave existed.**
**Analyser:** `scripts/aws/analyse_wave25.py`, frozen in that commit and the
authority on every verdict below.
**Cells:** 877 of 888 indexed. **10 failed, 1 voided — every one of them
recurrent.**

---

## 1. Verdicts

| hypothesis | verdict |
|---|---|
| **H25-1** — is the recurrent read-out's advantage order-dependent? | **NOT EVALUABLE** |
| **H25-2** — is the synchrony term a resolution effect? *(question)* | **answered: yes, at all three `fixed-tN` rungs** |
| **H25-3** — does the contrast depend on read-out depth? *(question)* | **NOT MET** — one point outside the bar |
| **H25-4** — does the contrast survive `e100` across the design space? | **MET** at all nineteen points |
| **H25-5** — cells | **NOT MET** — 877 of 888 |

## 2. H25-1 — the question the paper most needed answered could not be asked

This was the group that had to run. §3.7 reads its recurrent finding through
§3.5's shuffle result, and no recurrent cell in this campaign had ever been
manipulated. **It still has not been measured**, and the reason is not a
scheduling accident.

**Ten of the seventy-two recurrent cells failed**, every one with the
instrument's own guard firing:

```
shd-instrument: non-finite training value at optimizer step 1486
```

That is `rec+alif` at `surrogate_scale = 0.4` diverging numerically, aborting
the cell rather than writing a poisoned one. Landings, against 12 planned each:

| arm | `intact` | `bin-shuffled` | `reversed` |
|---|---:|---:|---:|
| `rec+alif` | 11 | 12 | **7** |
| `rec+alif+attn` | 11 | 12 (one voided at `saturated_fraction=0.305`) | **9** |

Seed-paired quadruples need all four arms on one seed. The `bin-shuffled`
contrast reaches **9** — exactly the registered floor — and the `reversed`
contrast reaches **3**.

| condition | DiD | positive | pairs | requirement | verdict |
|---|---:|---:|---:|---|---|
| `bin-shuffled` | +0.0291 | 6/9 | 9 | > +0.03, ≥ 9/12 | NOT MET |
| `reversed` | — | — | 3 | < +0.03 | not evaluable |

**H25-1 is NOT EVALUABLE, and it is reported as that and nothing else.** The
`bin-shuffled` row is printed because the analyser prints what it computed, and
it must not be read as a finding: it rests on nine of twelve quadruples, it
misses its bar by 0.0009, and the hypothesis required both halves.

**The temptation here is to rescue it** — raise the surrogate scale, add
clipping, drop `reversed` and quote the shuffle row. The registration forbids
all three, and it forbade them before the failure existed. **Nothing is
re-run.** What is now known, and was not before, is that
**`rec+alif` under time reversal is numerically unstable at the registered
operating point** — five of twelve rate cells diverged — and *that* is the
finding this group produced.

§3.7's cross-reference to §3.5 therefore remains **unsupported**. The manuscript
must say so rather than continue to assert it.

## 3. H25-2 — synchrony is a contract axis, not a single point

Wave 24 found the synchrony term at `fixed-t250` and nowhere else, which the
abstract hedged as "at at least one operating point". Both remaining rungs
answer, and both answer the same way.

| contract | bin width | DiD(`bin-shuffled`) | DiD(`channel-shuffled`) | difference |
|---|---:|---:|---:|---:|
| `fixed-t100` | 14.0 ms | +0.1323 | **+0.2631** | **+0.1308** |
| `fixed-t250` | 5.6 ms | +0.1119 | +0.2157 | +0.1038 *(wave 24)* |
| `fixed-t500` | 2.8 ms | +0.1161 | **+0.1736** | **+0.0575** |
| `published-2ms` / `d32L2` | 2 ms | +0.1145 | +0.1028 | −0.0117 *(wave 24)* |
| `published-2ms` / `d32L1` @ h1024 | 2 ms | +0.0675 | +0.0495 | −0.0181 *(wave 24)* |

**Destroying cross-channel synchrony on top of temporal order costs more at
every `fixed-tN` rung and at neither `published-2ms` point.** The read-out reads
synchrony, and how much depends on the contract.

**An observation, not a registered claim:** the synchrony term falls
monotonically as bins get finer — +0.1308, +0.1038, +0.0575 at 14.0, 5.6 and
2.8 ms — and is absent at 2 ms. That is the direction a bin-width account
predicts: a wider bin holds more coincident spikes, so there is more within-bin
synchrony for `channel-shuffled` to destroy and `bin-shuffled` to preserve.
**Three rungs are three points and no curve is fitted here**, and the two
`published-2ms` points differ from `fixed-t500` in sequence length as well as
bin width, so they are not a fourth rung of the same ladder.

## 4. H25-3 — read-out depth is a scope axis, at one width

H22-3 was NOT EVALUABLE because wave 22's plan held no `d32l4` anchor twin. The
twins ran here, and the question is answered.

| width | depth | DiD | `d32l4` twin | difference |
|---:|---|---:|---:|---:|
| 128 | `d32l2` | +0.1145 | +0.1208 | +0.0063 |
| 128 | `d64l4` | +0.1331 | +0.1208 | +0.0123 |
| 256 | `d32l1` | +0.0758 | +0.0862 | +0.0104 |
| 512 | `d32l1` | +0.0893 | +0.0968 | +0.0075 |
| **768** | **`d32l2`** | **+0.0610** | **+0.1881** | **+0.1271** |

Four of five sit inside 0.013 — the contrast is strikingly indifferent to depth
at h128, h256 and h512. **h768 is 0.1271 outside the registered 0.10 bar**, so
**H25-3 is NOT MET**: read-out depth is a scope axis for the mechanism claim,
which nothing in the paper currently suggests.

h768 is also where wave 21 recorded the campaign's **largest** DiD (+0.1881) and
its **smallest** positive gain (+0.0560). The two facts sit at the same
operating point, and this wave does not explain either. One point outside a bar
is a scope limit to disclose, not a depth effect to characterise.

## 5. H25-4 — the budget scope limit is retired

All nineteen remaining points clear the registered bar at `e100`, seventeen of
them at **12/12** seed quadruples and the weakest at 11/12. Range **+0.0469**
(h1024 `d32l2`) to **+0.1381** (h128 `d64l4`).

With wave 24's two points, **all twenty-one operating points now carry the
difference-in-differences at two budgets.** §4.6's budget sentence goes.

## 6. Provenance and cost

- **One binary.** All 25 gate reports in the campaign bucket name
  `3afd4434431a75a26cc9d5fa46831341fc2f1dd0ef08dc308e18ca139b576364`, checked by
  `collect.py --require-binary-sha256`, which exits non-zero rather than warning.
- **The fleet was reclaimed mid-wave.** Six `c7g.16xlarge` were terminated by
  AWS spot (`Service initiated`) inside four minutes at 525 of 888 cells. 96
  orphaned claims were released — every worker was gone, verified by
  `describe-instances`, so the 960-minute age guard had nothing left to protect
  — and the wave was relaunched across **two capacity pools** (3 ×
  `c7g.16xlarge` + 6 × `c7g.8xlarge`) so a single pool could not stall it again.
  It finished on that fleet, which self-terminated (`User initiated`).
- **The cost estimate was wrong, and by more than any before it.** The
  registration predicted **1,495 slot-hours**; the wave used **2,779** — a
  factor of **1.86**. The e100 cells were priced at a quarter of their e400
  medians, and the reclaim re-ran in-flight cells from the start. EC2 spot for
  2026-09-01 to 09-02 is **$97.50** with 09-03 not yet posted, against an
  estimate of $66 for this wave. **The estimator's three prior calibration hits
  do not survive this one**, and §5 of any future registration should quote it
  with this miss attached.

## 7. What this wave does not establish

- **The recurrent substrate is still unmeasured.** H25-1 is the wave's headline
  failure and no number from it may be quoted.
- **The synchrony ladder is three rungs**, all at h128 / `d32L4` /
  `adjacent-sum-5`. Nothing here separates bin width from sequence length.
- **H25-3 is one point outside a bar**, not a characterised depth effect.
- **Two budgets, still not a curve.**
- **Nothing here addresses the gain/DiD dissociation**, which remains the
  paper's leading open problem and still has no design.
