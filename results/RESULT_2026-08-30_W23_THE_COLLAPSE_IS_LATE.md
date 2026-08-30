# Result — wave 23: the h1024 collapse is late, and stopping early avoids it

**Registered:** [`PREREG_2026-08-29_THE_COLLAPSE_IS_LATE.md`](PREREG_2026-08-29_THE_COLLAPSE_IS_LATE.md),
committed in `7fb7a70` **before any cell of this wave existed and before any
instance for it was launched**. Attested by git history, not by mtimes.
**Analyser:** `scripts/aws/analyse_wave23.py`, frozen in the same commit. Its
output is the authority on every verdict below.
**Ran:** 2026-08-30, on the `binn-campaign-v2` fleet, pinned binary
`3afd4434431a75a2…` — the first campaign binary carrying the forward-finiteness
guard.
**Status:** complete — **72/72 cells settled, 0 invalid, 0 failed**, nothing
retried, no threshold moved, no arm extended.

**All four registered hypotheses MET.**

---

## 1. What was asked

`PAPER_DRAFT.md` §3.5 calls the h1024 gain inversion **"located but
unexplained"**, and the wave-21 preregistration registers it as the paper's
**leading open problem**. Three registered rescue levers had already failed and
every one was worse than the arm it was meant to rescue.

[`FINDING_2026-08-29_THE_H1024_COLLAPSE_IS_A_LOST_FIT.md`](FINDING_2026-08-29_THE_H1024_COLLAPSE_IS_A_LOST_FIT.md)
— **post-hoc, on cells that already existed, at zero compute** — observed that
63 of 68 intact `d32l4` cells at h1024 reach a training-loss minimum around
epoch 39–99 and end **56× above it**, while `d32l1` (0/20), `d32l2` (4/32) and
the rate arm (0/32) hold theirs. That excluded overfitting and produced a
falsifiable prediction: **if the collapse is late, truncating the budget should
avoid it.** No cell at `h1024/d32l4` had ever run at any budget but e400.

## 2. What the cells say

Every gain below is **within-budget** — attention minus rate at the same
epochs, seed-paired. n = 12 per cell.

| budget | read-out | rate | attention | gain | positive |
|---:|---|---:|---:|---:|---:|
| **e100** | **`d32/L4`** | 0.7326 | **0.8153** | **+0.0827** | **12/12** |
| e100 | `d32/L2` | 0.7326 | 0.7880 | +0.0554 | 12/12 |
| e200 | `d32/L4` | 0.7390 | 0.7955 | +0.0564 | 11/12 |
| e200 | `d32/L2` | 0.7390 | 0.7931 | +0.0541 | 12/12 |

Against the archived **e400** figure for the same arm: **−0.1318**, positive in
**3 of 20**.

Every number above was recomputed by a second, independent implementation that
imports nothing from the analyser, and agrees to the digit.

## 3. Verdicts

**H23-1 — MET.** Gain at e100/`d32l4` is **+0.0827** against a bar of +0.03,
positive in **12/12** against a bar of 9.

**H23-2 — MET.** Improvement over the archived e400 gain is **+0.2145**
(+0.0827 against −0.1318) against a bar of +0.10.

**H23-3 — MET, and this is the one that makes the others mean anything.**
The `d32l2` control moves only **+0.0149** between e400 and e100, inside its
±0.03 bar. **The effect is specific to the collapsing arm.** Had L2 improved by
more than its bar, e400 would simply have been past the optimum for every deep
read-out at this width and nothing about the collapse would have been shown —
the preregistration names that outcome explicitly and it did not occur.

**H23-4 — MET.** **0 of 12** e100/`d32l4` cells end above 3× their own best
training loss. At e400 it is **63 of 68**. The fit is retained.

## 4. What this establishes

**The h1024 `d32/L4` collapse is a late-training phenomenon at a configuration
that fits perfectly well early, and truncating the budget avoids it.** The
paper's leading open problem has an account, and the account was predicted in
advance from data already on disk rather than discovered by searching.

Two further observations, **descriptive and not registered**:

- **The rate arm is budget-insensitive** — 0.7326 at e100, 0.7390 at e200,
  0.7386 archived at e400. So the change is in the attention arm, not the
  baseline it is measured against.
- **The interior depth optimum at h1024 is itself a budget artefact.** At e400
  the ordering is L2 > L3 > L4 with L4 collapsed
  ([`RESULT_2026-08-28_W18_19_THE_DEPTH_OPTIMUM_IS_INTERIOR.md`](RESULT_2026-08-28_W18_19_THE_DEPTH_OPTIMUM_IS_INTERIOR.md)).
  At e100 it inverts: L4 (0.8153) is **above** L2 (0.7880). This carries **no
  verdict** — no hypothesis registered it and L1/L3 were not run at these
  budgets — but it is the obvious next question and is recorded so it is not
  rediscovered as a surprise.

## 5. What this does NOT establish

1. **It does not say why the fit is lost.** Nothing here revisits that, and
   H15-1 already **refused** the gradient-norm account: `d32l3` sits above the
   registered sickness threshold at 1.347 and gains anyway. "Located but
   unexplained" becomes "located, bounded in time, and still unexplained".
2. **It does not rescue h1024.** One width, one read-out depth, two budgets.
   The width ladder's other rungs are untouched and are not reopened.
3. **It is not a new headline.** The paper's headline is `h128` at e400 and
   stays there. A positive gain at e100/h1024 is a **scope note on the budget
   axis**, not the reported gain.
4. **It says nothing about the mechanism.** No `bin-shuffled` arm ran at these
   budgets, so the difference-in-differences and its h1024 row are untouched.
   What it does add is a caveat to reading that row: wave 21's h1024 DiD of
   +0.1122 was measured at e400, between two arms **both** in late collapse.
   Nothing here says what it would be at e100, and nothing should be inferred.
5. **The anchor budget is now known to be past the optimum at this width and
   depth.** That is a disclosure the paper owes a reader, not a result.

## 6. Provenance

This wave ran on the **first campaign binary that checks its evaluation forward
for finiteness**
([`DEFECT_2026-08-29_THE_EVALUATION_FORWARD_WAS_NEVER_CHECKED.md`](DEFECT_2026-08-29_THE_EVALUATION_FORWARD_WAS_NEVER_CHECKED.md)).
Every one of its 72 cells carries `non_finite_forward: 0`, which no archived
cell carries at all. h1024 is the configuration with peak gradient norms of
4.9e32 — precisely where that guard was expected to matter — and it did not
fire, so the accuracies above are not silently corrupted by a poisoned forward.

Cross-machine Gate F **FAILs** on every instance, by design and as disclosed:
the recorded cells are macOS/aarch64 and libm is not obliged to agree to the
last ulp. **No verdict above rests on it.** Every gain is a within-budget
contrast against a rate arm that ran on the same fleet, on the same binary,
beside its own treatment.

## 7. Reproduce

```bash
python3 scripts/aws/analyse_wave23.py
python3 scripts/fit_retention.py --width 1024
```
