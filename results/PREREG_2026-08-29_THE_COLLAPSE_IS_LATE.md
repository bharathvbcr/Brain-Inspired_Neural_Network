# Preregistration — is the h1024 collapse late, and does stopping early avoid it?

**Registered:** 2026-08-29, **before any cell of wave 23 exists and before any
instance for it is launched.** The ordering is attested by git history, not by
mtimes: this file and `scripts/aws/analyse_wave23.py` are committed together,
and the first cell is produced afterwards.

**Analyser:** `scripts/aws/analyse_wave23.py`, frozen in the same commit as this
file. It is the authority on every verdict below;
`scripts/check_verdicts_transcribed.py` cross-checks whatever is written up
against what it printed.

**Motivated by:**
[`FINDING_2026-08-29_THE_H1024_COLLAPSE_IS_A_LOST_FIT.md`](FINDING_2026-08-29_THE_H1024_COLLAPSE_IS_A_LOST_FIT.md)
— **post-hoc** analysis of existing cells, which is exactly why it needs this
wave and cannot stand on its own.

---

## 1. Why this wave exists

`PAPER_DRAFT.md` §3.5 calls the h1024 gain inversion **"located but
unexplained"**, and the wave-21 preregistration registers it as the paper's
leading open problem. Three registered rescue levers have already failed and
every one is worse than the arm it was meant to rescue.

The post-hoc finding above changes the shape of the question. At
`h1024/d32l4`, **63 of 68** intact cells reach a training-loss minimum and then
end **56× above it**, while `d32l1` (0/20), the rate arm (0/32) and `d32l2`
(4/32) hold theirs. The fit is reached at **epoch 39–99** in most seeds, and the
three seeds that keep it carry the three highest test accuracies in the set.

So the arm is not one that cannot learn this task. It is one that learns it and
then loses it. **That is a falsifiable claim about the budget axis, and no cell
at `h1024/d32l4` has ever been run at any budget but e400.**

## 2. The prediction, and what would refute it

If the collapse is late, then **truncating the budget should avoid it**, and the
gain at `h1024/d32l4` should be positive at a budget near where the fit is
reached — where at e400 it is **−0.1318** in 3 of 20 seeds.

This is refutable in a way the rescue levers were not. Surrogate scale and
gradient clipping change the optimisation everywhere; the budget changes only
when it stops. If the gain stays negative at e100 and e200, the late-collapse
account is **wrong**, and the finding above is withdrawn rather than reworded.

## 3. Design

Every cell `h1024`, `published-2ms`, `adjacent-sum-5`, 12 seeds, on the pinned
binary. Two budgets, three arms.

| arm | e100 | e200 | why it is here |
|---|---:|---:|---|
| `ff+fixed` (rate) | 12 | 12 | the gain is against this arm at the same budget, never against an archived e400 number |
| `ff+fixed+attn` `d32/L4` | 12 | 12 | the collapsing arm — the whole question |
| `ff+fixed+attn` `d32/L2` | 12 | 12 | **the control that makes this interpretable** |

**72 cells.**

### The L2 control is not optional

`d32l2` gains **+0.0405** at e400 and does **not** lose its fit. If truncating
the budget improves L4 *and* leaves L2 unchanged, the effect is specific to the
collapsing arm and the account survives. If truncating improves **both**, then
e400 is simply past the optimum for every deep read-out at this width and
nothing has been learned about the collapse. Without L2 those two outcomes are
indistinguishable, and the wave would be unable to fail informatively.

### Budgets

e100 and e200 bracket the observed fit epoch (39–99) from above. e100 is the
first budget at which the fit exists in most seeds; e200 asks whether the loss
of fit has already begun by then. Neither is the anchor budget, so **no number
from this wave may be compared against an e400 result across arms** — every
contrast below is within-budget.

## 4. Hypotheses

**H23-1 — the gain at `d32/L4` is positive at e100.**
Paired over seeds, gain(`d32l4`) − gain(rate) at e100 **> +0.03**, positive in
**≥ 9 of 12** seeds. *Refuted if the gain is ≤ 0.*

**H23-2 — the gain at `d32/L4` is higher at e100 than at e400.**
Against the archived e400 gain of **−0.1318**, the e100 gain is greater by
**> +0.10**. This is the one cross-budget comparison and it is on the *gain*, a
within-budget difference at each end, never on raw accuracy.

**H23-3 — the improvement is specific to the collapsing arm.**
gain(`d32l2`, e100) − gain(`d32l2`, e400) is **within ±0.03**. *If L2 also
improves by more than that, H23-1 and H23-2 are uninformative about the
collapse and are reported as such regardless of their own verdicts.*

**H23-4 — the fit is retained at the shorter budget.**
At e100, `d32l4` cells with final training loss above 3× their own best number
**≤ 3 of 12**, against 63 of 68 at e400. Measured by
`scripts/fit_retention.py`, whose threshold is fixed here and not tuned after.

## 5. Named outcomes, in every direction

| outcome | reading |
|---|---|
| H23-1, H23-2, H23-4 MET, H23-3 MET | The collapse is late and stopping early avoids it. The paper gains an account of its leading open problem, and a scope note: the anchor budget is past the optimum at this width and depth. |
| H23-1/2 MET, **H23-3 NOT MET** | e400 is past the optimum for deep read-outs at h1024 generally. Nothing specific to the collapse is established, and it is reported as a budget finding, not an explanation. |
| H23-1 NOT MET | **The late-collapse account is refuted.** `FINDING_2026-08-29` is withdrawn, not reworded, and "located but unexplained" stands with one fewer candidate. |
| H23-4 MET but H23-1 NOT MET | The fit is retained and the gain still does not appear — so retention is not sufficient, and the accuracy loss is not the fit loss. The most interesting outcome, and the one that would need its own wave. |
| Fewer than 10 of 12 seeds complete on any arm | The wave is **NOT EVALUABLE** and no verdict is issued. |

## 6. Stopping rule

72 cells, once. **No budget between e100 and e400 will be added after seeing
these**, and no third budget will be tried to find one that works — that is the
search the matched-architecture stopping rule exists to forbid, and the same
rule applies here. If e100 and e200 disagree, that is the result.

Seeds are the campaign's standard twelve. No seed is dropped for its value; a
seed is excluded only if its cell fails `scripts/cell_validity.py`, and the
count of exclusions is reported with the verdict.

## 7. What may not be claimed

1. **Not a rescue of h1024.** Even if every hypothesis is met, this is a
   statement about the budget axis at one width and one read-out depth. The
   width ladder's other rungs are not re-opened.
2. **Not a mechanism.** Why a fit is lost is not asked here and is not
   answerable from these cells. H15-1 already refused the gradient-norm
   account, and nothing in this design revisits it.
3. **Not a new headline.** The paper's headline is e400 and stays e400. A
   positive gain at e100 does not become the reported gain; it becomes a
   scope note on the budget axis.
4. **No comparison against archived e400 accuracies across arms.** Only the
   within-budget gains are compared, and only H23-2 crosses budgets at all.
5. **Nothing about the difference-in-differences.** No shuffled arm is run at
   these budgets, so the mechanism control is untouched and its h1024 row is
   unaffected.
