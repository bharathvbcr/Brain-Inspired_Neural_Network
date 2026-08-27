# Result — waves 15–17: the collapse is a threshold in width, and the headline holds at n=32

**Preregistered:** [`PREREG_2026-08-25_THE_H1024_COLLAPSE.md`](PREREG_2026-08-25_THE_H1024_COLLAPSE.md),
frozen with its analyser before the first cell existed.
**Analyser:** `scripts/aws/analyse_wave15.py`, amended once during the run and
recorded in [`AMENDMENT_2026-08-27_H17_2_MERGED_TWO_READOUT_DEPTHS.md`](AMENDMENT_2026-08-27_H17_2_MERGED_TWO_READOUT_DEPTHS.md).
**Fleet:** 4 × `c7g.16xlarge` (aarch64/Graviton3), one pinned binary
`22d97c51ab0204702ce44661683ff8c759c29d7f3379e2f6606b048f4f032104`.
**Coverage:** 224 planned, **224 valid, 0 invalid, 0 failures**.

---

## 1. Verdicts

| ID | statement | verdict |
|---|---|---|
| **H15-1** | The h1024 collapse is an optimisation failure a lever can undo | **NOT MET** |
| **H15-2** | Recovery, if it happens, is numerical | **NOT MET** (no arm met H15-1) |
| **H15-3** | At h1024, L2's gain lies between L1's and L4's | **NOT MET** — it lies *above both* |
| **H15-4** | Clipping is inert where it cannot bind | **MET** |
| **H16-1** | The gain decays monotonically with width up to the collapse | **NOT MET** |
| **H16-2** | The collapse is a threshold, not the continuation of the slope | **MET** |
| **H17-1** | The headline holds at n=32 | **MET** |
| **H17-2** | The mechanism holds at n=32 | **MET** |

Four met, four not. The two that carry the paper both hold at more than double
their published sample size; the two registered explanations of the h1024
collapse both fail.

## 2. No lever recovers h1024/d32/L4

| lever | pairs | gain | positive | median epoch-mean norm |
|---|---:|---:|---:|---:|
| surrogate scale 0.5 | 12 | **−0.2106** | 0/12 | 142.009 |
| surrogate scale 0.25 | 12 | **−0.2565** | 0/12 | 151.391 |
| clip-grad-norm 1000.0 | 12 | **−0.0904** | 1/12 | 11.660 |

Every lever is negative and every one is worse than the unclipped arm it was
meant to rescue. Clipping moves the median norm from 55.494 to 11.660 — a real
numerical effect, in the intended direction — and the accuracy does not follow.
**Reducing the gradient magnitude at h1024/L4 does not restore the gain**, so
whatever the collapse is, it is not a gradient scale that can be turned down.

H15-2 is reported NOT MET because no arm met H15-1, which is the condition it
was registered under. It is not evidence that the numerics are healthy.

**How hard the clip actually bound**, now that its warning carries a
denominator: a median of 96 of 12,800 optimiser steps per cell (**0.75%**,
range 2–192), touching a median 37 of 400 epochs (**9.2%**). The
preregistration predicted "roughly a tenth of epochs" and that is what happened.
`unclippable_steps` is 0 in every cell: no gradient norm was ever
unrepresentable, so clipping was never unable to act.

## 3. H15-3 failed in the direction nobody registered

| depth | gain | positive | median epoch-mean norm | max norm |
|---|---:|---:|---:|---:|
| L1 | −0.0159 | 1/12 | 0.025 | 3.25e2 |
| **L2** | **+0.0392** | **12/12** | 0.721 | 2.80e1 |
| L4 | −0.1618 | 1/12 | 55.494 | 1.13e8 |

H15-3 registered L2 as lying *between* L1 and L4. It came back above both, in a
direction the registration had no branch for — the same error class this
workspace recorded on 2026-08-23 and repeated here.

L2 is also the only h1024 attention arm that is numerically healthy: norm 0.721
against L4's 55.494, max norm 2.80e1 against 1.13e8, and the tightest accuracy
sd of the three.

**This result is not claimed here.** It rests on three points, one of them
missing (L3) and two archived, and re-reading the cells that produced it is not
a licence to reinterpret it. It is registered as its own wave in
[`PREREG_2026-08-27_DEPTH_IS_NOT_MONOTONE_AT_H1024.md`](PREREG_2026-08-27_DEPTH_IS_NOT_MONOTONE_AT_H1024.md).

## 4. The clip is inert where it cannot bind — H15-4 MET

Twelve h512/d32/L4 cells at `clip=1000.0`, **12/12 byte-identical** to the
archived `w8wid` cells across every scientific field. The h512 arm's gradients
never reach 1000.0, so the flag touches nothing, and the twelve clipped cells at
h1024 measure clipping rather than the flag.

The clipped **rate** control is likewise 12/12 byte-identical, so the h1024
clipped treatment's comparison is not confounded by its control moving
underneath it.

Wave 4 had shown the clipping flag can be destructive when it binds; nothing had
shown it was inert when it does not. It is.

## 5. The width ladder — H16-1 NOT MET, H16-2 MET

| width | pairs | `ff+fixed` | d32/L4 | gain | positive |
|---|---:|---:|---:|---:|---:|
| h128 | 12 | 0.7062 | 0.8320 | **+0.1258** | 12/12 |
| h256 | 12 | 0.7240 | 0.8206 | **+0.0966** | 12/12 |
| h384 | 12 | 0.7336 | 0.8096 | **+0.0760** | 12/12 |
| h512 | 12 | 0.7357 | 0.8233 | **+0.0876** | 12/12 |
| h768 | 12 | 0.7386 | 0.7946 | **+0.0560** | 11/12 |
| h1024 | 12 | 0.7386 | 0.5768 | **−0.1618** | 1/12 |

Adjacent gaps: +0.0292, +0.0206, **−0.0116**, +0.0316. One is negative, so the
registered chain fails.

**H16-1 fails for a reason worth stating precisely.** Seed-paired,
gain(h384) − gain(h512) is **−0.0116, negative in only 7 of 12 seeds, sd
0.0253**. The inversion breaks a criterion that demanded strict ordering with
0.005 separations, but h384 and h512 are **not distinguishable at n=12**. The
honest reading is that the criterion registered an ordering over quantities too
close together to order — not that there is a real dip at h384. A ladder with
finer separations than its noise floor cannot be monotone by measurement, and
that is a defect in the registration, not a finding about width.

**H16-2 is the substantive result and it holds.** The drop into h1024 is
**0.2178**, against 3× the largest gap below it (0.0947) — a factor of 6.9.
The collapse is a **threshold**, not the slope continuing.

**Two published statements are superseded.** [`RESULT_2026-08-22_AZURE_TRUNCATED_AT_95_OF_252.md`](RESULT_2026-08-22_AZURE_TRUNCATED_AT_95_OF_252.md)
read the ladder off four rungs and concluded the gain "decays gently across
h128 → h256 → h512 and then collapses between h512 and h1024". With six rungs,
(a) the decay is not strictly ordered, and (b) the collapse is between **h768
and h1024** — h768 is still +0.0560. The four-rung reading was true of four
rungs.

**The Azure rung is confirmed, not replaced.** Its four h256 attention cells are
**byte-identical to their AWS twins** across thirteen scientific fields — x86-64
against aarch64 — and restricted to those same four seeds the AWS gain is
**+0.0962**, Azure's number to the digit. At n=12 it is +0.0966. The four cells
become a cross-ISA check on the twelve rather than the evidence itself, which is
what the preregistration asked for.

## 6. The headline and its mechanism, at n=32 — both MET

| | n | `ff+fixed` | d32/L4 | gain | positive | ≥ 0.80 |
|---|---:|---:|---:|---:|---:|---:|
| published | 12 | 0.7062 | 0.8320 | +0.1258 | 12/12 | 12/12 |
| **this wave** | **32** | **0.7057** | **0.8332** | **+0.1275** | **32/32** | **32/32** |

| arm | pairs | intact − bin-shuffled | positive |
|---|---:|---:|---:|
| attention d32/L4 | 32 | **+0.1347** | 32/32 |
| rate | 32 | +0.0142 | — |

The paper's two load-bearing numbers were measured on twelve seeds. Twenty more
move the headline gain by **+0.0017** and the shuffle cost by **+0.0010**, with
every added seed positive and every one at or above 0.80. Nothing was extended
to rescue anything: both numbers cleared their bars at n=12 and clear them at
n=32 by the same margin.

**H17-2 required an amendment mid-run.** The analyser was merging a `d32l1`
archived shuffled control into a `d32l4` comparison for twelve of its pairs,
inflating the shuffle cost from +0.1347 to +0.1577. The verdict was MET either
way; the effect size was 17% high. `PAPER_DRAFT.md` §3.5 was checked and is
unaffected — its +0.1337, +0.0128 and 10× all reproduce exactly from the
`d32l4` arms. Full account in the amendment.

## 7. What this campaign does not license

- **Nothing against macOS-recorded numbers.** Cross-machine Gate F FAILs
  macOS-vs-Linux on all four nodes, as designed and as recorded on every node in
  `gates/`. Every number here rests on the control arm that ran beside its
  treatment on the same machine.
- **No claim that L2 is the optimal read-out depth at h1024.** §3 is the
  motivation for wave 18, not its result.
- **No claim about a dip at h384.** §5 says why: the quantity is inside its own
  noise.
- **No explanation of the h1024 collapse.** H15-1 refuted the optimisation
  reading and nothing replaced it. The collapse is located (§5) and its
  correlate is known (gradient norms leaving O(1)), and neither is a mechanism.

## 8. Provenance

224 cells, 0 invalid, 0 failures, all on the pinned binary across 4 nodes.
The 17 failures in the bucket are wave 13/14 cells and predate this campaign.

**On cell timings, and what they do not support.** Attention cells on this
fleet ran 3.30 h (h128) to 3.75 h (h768), with h1024/L4 at 3.40 h; rate controls
ran 0.09–0.39 h. An earlier draft of this section read that as attention cost
being *"nearly independent of hidden width"*. **It does not support that**, and
the claim is withdrawn.

`wall_secs` is wall time under four-way co-scheduling on a shared
memory-bandwidth budget, and across the corpus it is not a function of
configuration at all. Two impossibilities make that plain: `d32l1` at h1024
records **5.21 h** against `d32l4`'s **3.40 h**, though `attn_layers` multiplies
the cost; and the same h1024 rate-only arm records **1.156 h** in one wave and
**0.390 h** in another. Different waves ran on differently sized fleets, and
cells are comparable only against cells scheduled beside them.

What the timings **do** support is one negative claim, and it is robust because
contention can only make measured time *longer*:
`scripts/aws/estimate_cost.py` predicts **9.5 h** for h1024/d32/L4 at
`--parallel-efficiency 0.49`. Reaching the measured 3.40 h would require an
efficiency above 100%. The model over-predicts, and it did so in the direction
that made this campaign look longer than it was — see §9.

## 9. A note on the cost model, which misled this campaign three times

`plan_cells.estimated_seconds()` states in its own docstring that it is a
single-core ordinal function for scheduling and that "precision does not
matter". It was read as a wall-clock predictor anyway, producing an ETA of
"~6 h" at the halfway mark against an actual ~14 h remaining, and
`estimate_cost.py` shares its calibration and predicts 46 h for waves 18–19
where the measured arms imply roughly 16 h.

The error is not a single wrong constant. Checked against the anchor the
calibration came from — `ff+fixed` at h128, 9.6 s/epoch — the model needs an
implied parallel efficiency of 143% to reach the measured 0.085 h, so it
over-predicts even where it was calibrated. Recalibrating it against
`wall_secs` is not available either, for the reasons in §8.

No new coefficients are invented here. What changes is that the estimate no
longer travels without its bias: `estimate_cost.py` now measures itself against
the cells on disk and prints the ratio, so a prediction can never again be
quoted as if it had been checked.
