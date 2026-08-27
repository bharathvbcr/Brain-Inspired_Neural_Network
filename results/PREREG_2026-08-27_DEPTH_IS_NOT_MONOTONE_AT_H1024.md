# Preregistration — read-out depth at h1024, and what the collapse is actually made of

**Registered:** 2026-08-27, **before any cell of waves 18–19 exists and before
any instance for them is launched.** Authorised by the maintainer, who asked for
additional hypotheses and larger samples.

**Protocol:** `shd-attention-campaign-v2`, waves `w18dep`, `w19int`.
**Binary:** the campaign's existing pinned binary
(`22d97c51ab0204702ce44661683ff8c759c29d7f3379e2f6606b048f4f032104`). One binary
across all nineteen waves; reused controls stay bit-comparable.
**Analyser:** `scripts/aws/analyse_wave18.py`, frozen in the same commit as this
document and before the first cell lands.

---

## 1. What waves 15–17 turned up, and why it needs its own wave

[`PREREG_2026-08-25_THE_H1024_COLLAPSE.md`](PREREG_2026-08-25_THE_H1024_COLLAPSE.md)
registered **H15-3** as *"at h1024, the paired gain at L2 lies between L1's
−0.0159 and L4's −0.1618, exclusive of both by at least 0.01."* Twelve cells
landed. The paired gain at L2 is **+0.0392, positive in 12/12 seeds** — outside
that interval, above both endpoints, in the one direction the registration had
no branch for.

H15-3 is therefore **NOT MET as registered**, and the frozen analyser says so.
This document exists because the observation is more interesting than the
hypothesis it failed, and because "the registered interval was wrong" is not a
licence to reinterpret the same cells. The reading below has to be tested on
cells that do not exist yet.

The h1024 depth ladder, every number paired on seed against the h1024 rate-only
control that ran beside it, n=12 per arm:

| depth | accuracy (median) | sd | paired gain | positive | median epoch-mean norm | max norm | source |
|---|---:|---:|---:|---:|---:|---:|---|
| **L1** | 0.7228 | 0.0094 | −0.0159 | 1/12 | 0.025 | 3.25e2 | archived `w3wid` |
| **L2** | **0.7747** | 0.0157 | **+0.0392** | **12/12** | 0.721 | 2.80e1 | `w15col`, this campaign |
| **L3** | — | — | — | — | — | — | **does not exist** |
| **L4** | 0.5808 | 0.0866 | −0.1618 | 1/12 | 55.494 | 1.13e8 | archived `w8wid` |

Three points, one of them missing, and two of the three from archived waves. The
shape that reads off this table — depth helps, then destroys — rests on L3 being
measured. That is the wave.

## 2. Three statistics that were computed before they were registered, and died

This section is here because each of these would have been a defensible-looking
bar, each is contradicted by cells already in hand, and each would have produced
a verdict rather than a crash.

1. **"Spearman ρ(accuracy, log₁₀ norm) ≤ −0.8 across the depth arms."**
   Measured over the 36 h1024 attention cells in hand: **ρ = −0.424**. The
   −0.970 quoted in the wave-15 preregistration §1 was computed *within the L4
   arm across two attention dimensions* — entirely inside the collapsed regime,
   where the relationship is monotone. It does not extend across depths, because
   L1 has both the **smallest** norm of any arm and a **negative** gain. A rank
   correlation is the wrong instrument for a non-monotone relationship, and
   registering it would have failed the mechanism for the wrong reason.

2. **"Gain is positive iff median epoch-mean norm lies in a healthy band
   [0.1, 10]."** Contradicted immediately: h128/L1 has norm **0.023**, far below
   the band, and gain **+0.0421**.

3. **"Gain is negative iff median epoch-mean norm ≥ 1.0."** Contradicted by
   h1024/L1: norm **0.025**, gain **−0.0159**. Checked against all six depth
   arms that exist at any width:

   | arm | gain | norm | sign rule |
   |---|---:|---:|---|
   | h128/L1 | +0.0421 | 0.023 | consistent |
   | h512/L1 | +0.0043 | 0.023 | consistent |
   | h512/L4 | +0.0876 | 0.460 | consistent |
   | **h1024/L1** | **−0.0159** | **0.025** | **contradicted** |
   | h1024/L2 | +0.0392 | 0.721 | consistent |
   | h1024/L4 | −0.1618 | 55.494 | consistent |

**What survives is narrower than any of them.** Gradient scale explains the
**collapse**, not the whole ordering. A one-layer read-out is mildly unhelpful at
h1024 for reasons that have nothing to do with exploding gradients — it is mildly
unhelpful at h512 too (+0.0043). H18-2 below is registered on the collapse alone,
and is deliberately silent about L1's mediocrity.

## 3. Design

**Wave `w18dep` — the depth ladder at h1024, one wave, one fleet, 20 seeds.**

All four depths are regenerated rather than assembled from three campaigns. The
h256 rung of the wave-16 width ladder was rejected for resting on four Azure
cells while its neighbours rested on twelve; a depth ladder with two archived
rungs, one new rung and one missing rung has the same defect in a different axis.
Seeds 1–12 of the L2 arm duplicate cells `w15col` has already produced, which
makes them a free harness check (H18-4) rather than waste.

80 attention cells (L1, L2, L3, L4 × 20 seeds) + 20 h1024 rate-only controls.

**Wave `w19int` — does the optimum move with width?** At h512 the deeper
read-out wins (+0.0876 at L4 vs +0.0043 at L1); at h1024 it collapses. If the
optimal depth falls as width rises, h768 should sit between: L4 still winning,
but by less. 12 cells at h768/d32/**L2**; h768/L4 and the h768 rate control come
from `w16lad`, same fleet, same binary, same seeds.

**Cost:** 112 new cells, ≈256 slot-hours, ≈16 h on a 16-slot fleet. Calibrated
from measured `wall_secs` in this campaign's own cells, not from
`plan_cells.estimated_seconds()`, which is a single-core ordinal function and was
misread as a wall-clock predictor once already.

## 4. Hypotheses

| ID | statement | criterion |
|---|---|---|
| **H18-1** | The optimum in read-out depth at h1024 is interior | the largest paired gain over `ff+fixed` across {L1, L2, L3, L4} is attained at **L2 or L3**, and exceeds the gain at **both** L1 and L4 by **≥ 0.02** |
| **H18-2** | The collapse, specifically, is numerical | every depth arm with median epoch-mean gradient norm **≥ 1.0** has paired gain **≤ −0.10**, and no arm with norm **< 1.0** has paired gain below **−0.05** |
| **H18-3** | L2's advantage is not a seed artefact | at L2, paired gain **≥ +0.02**, positive in **≥ 15/20** seeds |
| **H18-4** | The fleet reproduces itself | the twelve h1024/d32/**L2** cells at seeds 1–12 are **byte-identical** to the `w15col` cells of the same id on every scientific field |
| **H19-1** | The optimal depth falls as width rises | gain(h768, L4) **> ** gain(h768, L2), and gain(h1024, L2) **>** gain(h1024, L4) — the ordering of L2 and L4 **reverses** between h768 and h1024 |

H18-1, H18-2 and H19-1 are independent. Any one passing rescues none of the
others.

**H18-4 is a check on the harness, not on the science, and it is destructive.**
The instrument is deterministic and the binary is pinned; twelve cells at the
same ids and seeds on the same ISA must reproduce to the byte. If they do not,
the fleet is not reproducing itself, **every cell in waves 18 and 19 is void**,
and so is the comparability of waves 15–17 to the archive. This is the first
registered check in the campaign that would catch a silent change in the
execution environment rather than in the code.

**H18-2 is registered on the collapse and nothing else.** It makes exactly one
prediction about the unmeasured rung: if L3's norm comes back ≥ 1.0 its gain must
be ≤ −0.10, and if its norm comes back < 1.0 its gain must not be below −0.05.
Either outcome is informative; a norm near 1.0 with a gain near −0.07 falsifies
it, and that is the point of stating both halves.

## 5. Named outcomes, in every direction

| observed | reading |
|---|---|
| max gain at **L2 or L3**, clear of both ends by ≥ 0.02 | **H18-1 MET.** The paper's scope limit is a property of *deep* read-outs at h1024, not of h1024. §3.5 must be rewritten. |
| max gain at **L1** | depth monotonically hurts at h1024. The L2 result was a twelve-seed accident; H18-3 will have caught it. |
| max gain at **L4** | the archived collapse does not reproduce on this fleet. **Everything downstream is void**, including the wave-15 verdicts, and the archive/fleet discrepancy becomes the finding. |
| interior max, margin **< 0.02** | shape **unresolved**. Reported as unresolved; no rewrite of §3.5. |
| **L3 ≥ L2** | the optimum is at L3 and the ladder needs a rung at L5 to bound it. Registered here so that outcome does not arrive as a surprise requiring a new document. |
| H19-1 reversal **absent** | optimal depth does not move with width in this range; the h1024 collapse is a threshold in width at fixed depth, which is what wave 16 was built to test. |

## 6. Stopping rule

Every cell of both waves runs. No arm is extended, truncated or re-seeded on the
basis of what it shows. **An arm returning fewer than 9 valid seeds is
NOT EVALUABLE and carries no numbers at all** — not a mean, not a direction.
A comparison below its pair floor prints its pair count and stops.

Validity is `scripts/cell_validity.py` as it stands, unmodified. Gradient
magnitude is the quantity under study in H18-2 and **never voids a cell**;
voiding on it would decide the question by definition.

## 7. What may not be claimed from these waves

- **Nothing against macOS-recorded numbers.** Cross-machine Gate F FAILs
  macOS-vs-Linux on every node of this campaign, by design and as recorded in
  `scripts/aws/bootstrap.sh`. Every claim rests on the control arm that ran
  beside its treatment on the same machine.
- **No claim about h1024/L1's mediocrity.** H18-2 is silent on it deliberately;
  explaining it needs cells this wave does not run.
- **No claim about widths above h1024 or depths above L4.** The ladder is bounded
  at both ends and an interior maximum at L3 would mean it is bounded too tightly.
- **No re-reading of wave 15's L2 cells.** They motivated this document; they are
  not evidence for it. H18-1 and H18-3 are decided on wave 18's own cells.
