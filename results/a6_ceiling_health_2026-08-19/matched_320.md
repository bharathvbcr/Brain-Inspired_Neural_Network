# A6 — ceiling health of the surviving matched arms

`a6-ceiling-health` protocol v1. **Exploratory sensitivity sweep — not a canonical run, no frozen manifest, no `--config-hash` claim.**

The reference's training budget is swept while the forward, the frozen splits, the seed lineage, and every arm's budget are held fixed. Only `MatchedGradient` sees the swept `epochs`/`lr`; the arms are trained once at the canonical budget.

## `c1-dfa`

- seeds: **20** · arm budget (fixed): **320 epochs** · canonical reference budget: **e80/lr0.05**
- MatchedDfa (fixed): **1.0000**
- MatchedBroadcastErr (control) (fixed): **1.0000**

| reference budget | reference mean | SE | vs arm | vs control |
|---|---|---|---|---|
| `e80/lr0.05` *(canonical)* | 0.9013 | 0.0298 | arm above | control above |
| `e320/lr0.05` | 0.9700 | 0.0189 | arm above | control above |

**Harness check: reproduces.** Canonical row 0.9013 vs published 0.8963 (drift 0.0050).

**Verdict: no budget tested lifts the reference above the arm.** Best is `e320/lr0.05` at 0.9700, still below the MatchedDfa arm's 1.0000. On this evidence the published 0.8963 is not simply undertrained — but note this is a bounded sweep, not a proof of convergence.

## `c1-rl`

- seeds: **20** · arm budget (fixed): **320 epochs** · canonical reference budget: **e80/lr0.05**
- MatchedRlReinforceFb (fixed): **0.9863**
- MatchedRlGraded (fixed): **0.5500**
- MatchedRlFlat (±1 baseline) (fixed): **0.5250**

| reference budget | reference mean | SE | vs arm | vs control |
|---|---|---|---|---|
| `e80/lr0.05` *(canonical)* | 0.9188 | 0.0263 | arm above | **reference above** |
| `e320/lr0.05` | 0.9812 | 0.0144 | arm above | **reference above** |

**Harness check: DOES NOT REPRODUCE.** Canonical row 0.9188 vs published 0.8887 (drift 0.0301). Treat every row below as uninterpretable until this is explained — the sweep is measuring a different substrate than the published number.

**Verdict: no budget tested lifts the reference above the arm.** Best is `e320/lr0.05` at 0.9812, still below the MatchedRlReinforceFb arm's 0.9863. On this evidence the published 0.8887 is not simply undertrained — but note this is a bounded sweep, not a proof of convergence.

## Reading this table

A row where the reference sits below the arm is only evidence about *that* budget. The sweep is bounded above by the largest budget listed; it cannot show the reference has converged, only that it had not overtaken the arm by the budgets tested. Report both numbers, never the conclusion alone.
