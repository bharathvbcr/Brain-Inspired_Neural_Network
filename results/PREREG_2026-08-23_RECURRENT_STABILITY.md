# Preregistration — can the recurrent arms complete at the anchor budget at all?

**Registered:** 2026-08-23, **before any wave-13 cell exists** and before the
fleet is launched.
**Campaign:** `shd_attention_campaign_v2`, wave 13, same bucket, **same pinned
binary** `22d97c51ab02`.

---

## 1. This wave is not the measurement, and that is deliberate

Wave 12 ran the adaptation half of the substrate factorial and deferred the
recurrent half. The obvious next step is to run it. The record says that would
be spending a wave to re-learn wave 11:

* Wave 11 removed `--clip-grad-norm 1.0`, the flag that diverged all 24 of
  wave 4, and **nine of 24 still diverged** — at optimizer steps 438–1035
  against wave 4's median of ~176. Removing the flag *delayed* the divergence
  rather than ending it. 15 of 24 against a registered bar of 18 made every
  wave-11 hypothesis **NOT EVALUABLE**.
* Under the campaign's validity rule an arm with any diverged cell reports
  **zero** usable cells. A 12-seed recurrent arm at wave 11's completion rate
  cannot carry a verdict at all.
* **No recurrent cell at h128 has ever run past e20.** The two that exist
  already show the trouble:

  | arm | epochs | accuracy | `non_finite_events` | peak ‖g‖ |
  |---|---:|---:|---:|---:|
  | `rec+alif` | 20 | 0.3785 | 0 | **1.166e11** |
  | `rec+fixed` | 20 | 0.2633 | **2** | 2.057e6 |

  `rec+fixed` — which no wave has ever stress-tested, wave 4 and wave 11 being
  `rec+alif` only — already trips the counter that voids a cell, twenty epochs
  in. It is not the stable alternative.

The anchor budget is e400: **12,800 optimizer steps** against e20's ~640, twenty
times deeper into the range where wave 11's cells died.

So the registered question here is the prior one: **is there an operating point
at the anchor budget where these arms complete?** The outcome is a completion
rate, not an accuracy.

## 2. Design — 48 cells, n = 12

| label | arms | hidden | epochs | contract | geometry | surrogate scale |
|---|---|---:|---:|---|---|---|
| `w13rec` | `rec+fixed`, `rec+alif` | 128 | 400 | `published-2ms` | `adjacent-sum-5` | 1.0 and 0.4 |

Anchor width, contract, geometry and budget throughout, so a condition that
clears the bar is immediately usable by a measurement wave without changing
anything else.

**No attention.** It roughly quadruples the cost, and wave 11 showed
`rec+alif+attn` diverges too, so it buys nothing until a substrate completes on
its own.

**No clipping, on either lever.** `--clip-grad-norm` is what diverged wave 4.
`--clip-sample-grad-norm` is untried at any threshold, and introducing an untried
parameter into the wave meant to characterise the baseline would confound the
thing being measured. It is §5's next lever, not this wave's.

## 3. Hypotheses and thresholds

Fixed here. Every verdict computed **once**, after all 48 cells settle.

A cell **completes** iff it is emitted *and* passes the validity gate in
`scripts/cell_validity.py` — which includes `non_finite_events == 0`. A cell that
is emitted while reporting non-finite events has not completed, and the `e20`
`rec+fixed` cell above is exactly that case.

| id | claim | threshold |
|---|---|---|
| **R-1** *(primary)* | some condition completes well enough to be measurable | **≥ 11 of 12** completions in at least one (arm, scale) condition |
| **R-2** | adaptation is what destabilises the recurrent arm | completion of `rec+fixed` vs `rec+alif`, pooled over scales: difference of **≥ 6 of 24** either way, sign reported |
| **R-3** | the surrogate scale is a stability lever at this width | completion at 0.4 vs 1.0, pooled over arms: difference of **≥ 6 of 24** either way, sign reported |
| **R-4** | *(diagnostic, no verdict)* how far from usable each condition is | peak ‖g‖ distribution and the optimizer step of each divergence, reported per condition |

## 4. Named outcomes

| outcome | means |
|---|---|
| **R-1 SUPPORTED** | that condition is the operating point. A measurement wave — the recurrent half of the factorial, `+attn` at d32/L4 — is registered separately and run there. |
| **R-1 NOT SUPPORTED** | no operating point at the anchor budget on these levers. The recurrent arms are then not measurable at the anchor without a new lever, and §5 names the candidates in the order the evidence supports. The recurrent axis stays out of the paper, stated as unmeasured rather than negative. |
| **R-2 favours `rec+fixed`** | adaptation is the destabiliser, and the recurrent-without-adaptation corner may be reachable while `rec+alif` is not. |
| **R-2 flat** | recurrence alone destabilises; adaptation is not the lever, and neither is the corner. |
| **R-3 favours 0.4** | per-timestep backward gain is the axis, as `AMENDMENT_2026-08-05` argued; a finer scale ladder becomes worth registering. |
| **R-3 flat** | gain is not the axis at this width, which contradicts the amendment's reasoning at h512 and is worth recording as such. |

## 5. What this wave may not claim, and what comes next

* **Accuracies of completing cells are reported and are not a measurement.**
  Wave 11's own record states the trap: an arm that diverges more often can look
  better, because only its luckier trajectories survive to be scored. Every
  accuracy printed here carries its condition's completion count beside it, and
  no comparison between conditions with different completion rates is a result.
* **Nothing about attention on a recurrent substrate.** That is the measurement
  wave, and it is not this one.
* Not calibration. No comparison to macOS-recorded numbers.

If R-1 fails, the levers in the order the evidence supports them:

1. **Per-sample gradient clipping** (`--clip-sample-grad-norm`), at a threshold
   taken from the recurrent arm's *own* per-sample distribution rather than from
   `ff+fixed`'s scale. Wave 4 failed because a 1.0 threshold, chosen from a
   healthy arm whose epoch-mean norm is 0.20–0.29, bound on essentially every
   step of an arm whose own norm exceeds 1.0 in 100 of 100 epochs — that is
   unconditional renormalisation, not outlier suppression, and it removes the
   second-moment damping that lets the unclipped arm recover. Per-sample
   clipping acts one level lower and leaves the batch gradient free to vary. It
   is a different intervention and it is untried.
2. **Narrower still** — h64. Narrowing h512 → h256 helped and did not suffice;
   h128 is untested at budget and is what this wave measures.
3. **Shorter budget with a stated scope limit** — measure the recurrent axis at
   the deepest budget that completes, and say plainly that it is not the anchor.

## 6. Stopping rule and cost

Fixed at 48 cells. No cell added, dropped, re-seeded, or re-run on the basis of
its result. **A diverged cell is the measurement here**, not a failure to be
retried, and its abort step is recorded.

Estimated 1.5 h wall with every cell in flight at once on ~192 vCPU of spot —
roughly $3.
