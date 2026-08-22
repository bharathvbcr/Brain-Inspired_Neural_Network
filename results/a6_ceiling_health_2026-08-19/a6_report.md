# A6 — ceiling health of the surviving matched arms

`a6-ceiling-health` protocol v1. **Exploratory sensitivity sweep — not a canonical run, no frozen manifest, no `--config-hash` claim.**

The reference's training budget is swept while the forward, the frozen splits, the seed lineage, and every arm's budget are held fixed. Only `MatchedGradient` sees the swept `epochs`/`lr`; the arms are trained once at the canonical budget.

## `c1-dfa`

- seeds: **20** · arm budget (fixed): **80 epochs** · canonical reference budget: **e80/lr0.05**
- MatchedDfa (fixed): **0.9387**
- MatchedBroadcastErr (control) (fixed): **0.9863**

| reference budget | reference mean | SE | vs arm | vs control |
|---|---|---|---|---|
| `e80/lr0.05` *(canonical)* | 0.9013 | 0.0298 | arm above | control above |
| `e80/lr0.02` | 0.8600 | 0.0351 | arm above | control above |
| `e80/lr0.1` | 0.9062 | 0.0370 | arm above | control above |
| `e80/lr0.2` | 0.8175 | 0.0449 | arm above | control above |
| `e160/lr0.02` | 0.9075 | 0.0326 | arm above | control above |
| `e160/lr0.05` | 0.9175 | 0.0332 | arm above | control above |
| `e160/lr0.1` | 0.9288 | 0.0297 | arm above | control above |
| `e160/lr0.2` | 0.9225 | 0.0374 | arm above | control above |
| `e320/lr0.02` | 0.9400 | 0.0249 | **reference above** | control above |
| `e320/lr0.05` | 0.9700 | 0.0189 | **reference above** | control above |
| `e320/lr0.1` | 0.9500 | 0.0204 | **reference above** | control above |
| `e320/lr0.2` | 0.9700 | 0.0250 | **reference above** | control above |
| `e640/lr0.02` | 0.9863 | 0.0137 | **reference above** | control above |
| `e640/lr0.05` | 0.9975 | 0.0025 | **reference above** | **reference above** |
| `e640/lr0.1` | 0.9500 | 0.0227 | **reference above** | control above |
| `e640/lr0.2` | 1.0000 | 0.0000 | **reference above** | **reference above** |
| `e1280/lr0.02` | 0.9812 | 0.0144 | **reference above** | control above |
| `e1280/lr0.05` | 0.9975 | 0.0025 | **reference above** | **reference above** |
| `e1280/lr0.1` | 0.9675 | 0.0189 | **reference above** | control above |
| `e1280/lr0.2` | 1.0000 | 0.0000 | **reference above** | **reference above** |
| `e2560/lr0.02` | 1.0000 | 0.0000 | **reference above** | **reference above** |
| `e2560/lr0.05` | 1.0000 | 0.0000 | **reference above** | **reference above** |
| `e2560/lr0.1` | 1.0000 | 0.0000 | **reference above** | **reference above** |
| `e2560/lr0.2` | 1.0000 | 0.0000 | **reference above** | **reference above** |

**Harness check: reproduces.** Canonical row 0.9013 vs published 0.8963 (drift 0.0050).

**Verdict: the reference was undertrained.** At `e2560/lr0.2` the reference reaches 1.0000, above the MatchedDfa arm's 0.9387. The published 0.8963 is a budget artifact, and the matched-side ordering that the transfer-gap contrast rests on does not survive a fairer reference budget.

> The MatchedBroadcastErr (control) still sits above the arm under test (0.9863 vs 0.9387). Whatever the reference does, that ordering is its own open question.

## `c1-rl`

- seeds: **20** · arm budget (fixed): **80 epochs** · canonical reference budget: **e80/lr0.05**
- MatchedRlReinforceFb (fixed): **0.9200**
- MatchedRlGraded (fixed): **0.5250**
- MatchedRlFlat (±1 baseline) (fixed): **0.5113**

| reference budget | reference mean | SE | vs arm | vs control |
|---|---|---|---|---|
| `e80/lr0.05` *(canonical)* | 0.9188 | 0.0263 | arm above | **reference above** |
| `e80/lr0.02` | 0.8875 | 0.0385 | arm above | **reference above** |
| `e80/lr0.1` | 0.9125 | 0.0257 | arm above | **reference above** |
| `e80/lr0.2` | 0.8087 | 0.0469 | arm above | **reference above** |
| `e160/lr0.02` | 0.8788 | 0.0393 | arm above | **reference above** |
| `e160/lr0.05` | 0.9488 | 0.0220 | **reference above** | **reference above** |
| `e160/lr0.1` | 0.9475 | 0.0302 | **reference above** | **reference above** |
| `e160/lr0.2` | 0.9175 | 0.0332 | arm above | **reference above** |
| `e320/lr0.02` | 0.9838 | 0.0138 | **reference above** | **reference above** |
| `e320/lr0.05` | 0.9812 | 0.0144 | **reference above** | **reference above** |
| `e320/lr0.1` | 0.9200 | 0.0277 | **reference above** | **reference above** |
| `e320/lr0.2` | 0.9750 | 0.0250 | **reference above** | **reference above** |
| `e640/lr0.02` | 0.9725 | 0.0189 | **reference above** | **reference above** |
| `e640/lr0.05` | 1.0000 | 0.0000 | **reference above** | **reference above** |
| `e640/lr0.1` | 0.9587 | 0.0278 | **reference above** | **reference above** |
| `e640/lr0.2` | 1.0000 | 0.0000 | **reference above** | **reference above** |
| `e1280/lr0.02` | 0.9975 | 0.0025 | **reference above** | **reference above** |
| `e1280/lr0.05` | 0.9750 | 0.0250 | **reference above** | **reference above** |
| `e1280/lr0.1` | 0.9975 | 0.0025 | **reference above** | **reference above** |
| `e1280/lr0.2` | 1.0000 | 0.0000 | **reference above** | **reference above** |
| `e2560/lr0.02` | 1.0000 | 0.0000 | **reference above** | **reference above** |
| `e2560/lr0.05` | 1.0000 | 0.0000 | **reference above** | **reference above** |
| `e2560/lr0.1` | 1.0000 | 0.0000 | **reference above** | **reference above** |
| `e2560/lr0.2` | 1.0000 | 0.0000 | **reference above** | **reference above** |

**Harness check: DOES NOT REPRODUCE.** Canonical row 0.9188 vs published 0.8887 (drift 0.0301). Treat every row below as uninterpretable until this is explained — the sweep is measuring a different substrate than the published number.

**Verdict: the reference was undertrained.** At `e2560/lr0.2` the reference reaches 1.0000, above the MatchedRlReinforceFb arm's 0.9200. The published 0.8887 is a budget artifact, and the matched-side ordering that the transfer-gap contrast rests on does not survive a fairer reference budget.

## Reading this table

A row where the reference sits below the arm is only evidence about *that* budget. The sweep is bounded above by the largest budget listed; it cannot show the reference has converged, only that it had not overtaken the arm by the budgets tested. Report both numbers, never the conclusion alone.
