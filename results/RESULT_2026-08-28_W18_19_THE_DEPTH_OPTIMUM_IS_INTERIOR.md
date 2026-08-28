# Result — waves 18 and 19: the h1024 optimum in read-out depth is interior, and the collapse is still not numerical

**Registered:** [`PREREG_2026-08-27_DEPTH_IS_NOT_MONOTONE_AT_H1024.md`](PREREG_2026-08-27_DEPTH_IS_NOT_MONOTONE_AT_H1024.md),
before either wave existed.
**Analyser:** `scripts/aws/analyse_wave18.py`, frozen with the preregistration.
Its output is [`VERDICTS_W18.md`](shd_attention_campaign_v2/VERDICTS_W18.md) and
every verdict below is cross-checked against it by
`scripts/check_verdicts_transcribed.py`.
**Binary:** the campaign's pinned `22d97c51ab02`.
**Status:** complete — **wave 18 at 100/100 and wave 19 at 12/12, zero
divergences and zero voided cells**, nothing retried and no threshold moved.

---

## 1. Verdicts

| ID | statement | verdict |
|---|---|---|
| **H18-1** | The optimum in read-out depth at h1024 is interior | **MET** |
| **H18-2** | The collapse, specifically, is numerical | **NOT MET** |
| **H18-3** | L2's advantage is not a seed artefact | **MET** |
| **H18-4** | The fleet reproduces itself | **MET** |
| **H19-1** | The optimal depth falls as width rises | **MET** |

## 2. H18-4 first, because it is destructive and everything else rests on it

**H18-4: MET** — the twelve h1024/`d32`/L2 cells at seeds 1–12 are
**byte-identical** to the `w15col` cells of the same id on every scientific
field.

The preregistration scheduled this check at plan index 140 of 192, so a failure
would have surfaced after most of the compute it invalidates had been spent.
It passed. Had it not, H18-1 and H18-3 were registered VOID and the
archive/fleet discrepancy would have become the finding instead.

## 3. The ladder

n = 20 pairs per rung, h1024 / `d32` / e400 / `published-2ms` /
`adjacent-sum-5`, seed-paired against one shared `ff+fixed` rate arm.

| depth | pairs | rate (median) | attention (median) | gain (mean, paired) | positive | median epoch-mean norm |
|---|---:|---:|---:|---:|---:|---:|
| L1 | 20 | 0.7392 | 0.7228 | **−0.0159** | 3/20 | 0.025 |
| **L2** | 20 | 0.7392 | 0.7767 | **+0.0405** | **20/20** | 0.658 |
| L3 | 20 | 0.7392 | 0.7838 | **+0.0371** | 18/20 | 1.347 |
| L4 | 20 | 0.7392 | 0.6093 | **−0.1318** | 3/20 | 34.469 |

**Do not subtract the accuracy columns to get the gain column.** The first two
are **medians** over the shared seeds and the third is the **mean** of the
seed-paired differences; the analyser uses medians for accuracy because the
quantity it reports alongside spans eight orders of magnitude. At L3 the
difference of medians is +0.0446 while the paired mean gain is +0.0371, and the
paired mean is the registered statistic. The medians are printed because they
say where each arm sits; only the gain column decides anything.

**H18-1: MET** — the largest paired gain is attained at an interior depth and
exceeds both endpoints by more than the registered 0.02: **+0.0563** clear of
L1 and **+0.1723** clear of L4.

**The optimum is interior. It is not established to be at L2 rather than L3.**
The two differ by **0.0034** in paired gain, which is far inside the separation
this campaign has already refused to read as an ordering — H16-1 was NOT MET on
a width gap of 0.0116 with sd 0.0253. The preregistration's criterion is
deliberately *"attained at L2 or L3"* for that reason, and this document claims
no more than that.

**H18-3: MET** — at L2, paired gain **+0.0405**, positive in **20/20** against
registered bars of ≥ +0.02 and ≥ 15/20. The rung the interior maximum sits on
is not a twelve-seed accident.

## 4. H18-2 — the collapse is still not explained

**H18-2: NOT MET.** The registered rule was that every arm with median
epoch-mean gradient norm ≥ 1.0 has paired gain ≤ −0.10, and no arm below that
norm falls below −0.05.

**L3 breaks it.** Its norm is **1.347**, over the sickness threshold, and its
gain is **+0.0371** — not merely above −0.10 but positive, and positive in 18 of
20 seeds. An arm can sit in the numerically unhealthy regime and still be one of
the two best rungs on the ladder.

So the gradient-norm story does not predict the collapse. It remains what wave
15 concluded and what [`PAPER_DRAFT.md`](PAPER_DRAFT.md) §3.5 already says:
**located but unexplained.** Three preregistered levers failed to rescue L4 and
the one correlate that looked promising does not survive being asked of L3.

## 5. H19-1 — the optimal depth moves with width

| width | L2 gain | L4 gain | deeper wins? |
|---|---:|---:|---|
| h768 | +0.0419 | +0.0560 | yes |
| h1024 | +0.0405 | −0.1318 | no |

**H19-1: MET** — the ordering of L2 and L4 reverses between h768 and h1024.

At h768 the deeper read-out is the better one. At h1024 it is the worse one by a
wide margin, while the shallower rung barely moves: **L2's gain is +0.0419 at
h768 and +0.0405 at h1024**, a change of 0.0014 across the width where L4 falls
by 0.1878.

## 6. What this requires of the manuscript

The preregistration names this outcome and its consequence:

> max gain at **L2 or L3**, clear of both ends by ≥ 0.02 → **H18-1 MET.** The
> paper's scope limit is a property of *deep* read-outs at h1024, not of h1024.
> §3.5 must be rewritten.

That rewrite is **outstanding**. §3.5 item 4 currently reads the h1024 result as
a width phenomenon — *"Gain inverts at width h1024 (−0.1618 at L4)"* — and
waves 18 and 19 show the inversion is a property of the **read-out depth at that
width**: at the same width and budget, `d32/L2` gains **+0.0405** in 20 of 20
seeds. The width ladder in §3.5 is measured at L4 throughout, so every rung of
it is a statement about deep read-outs specifically.

Recorded here rather than performed here, because it is a change to the
manuscript's scope claims and deserves its own pass.

## 7. What may not be claimed

Carried unchanged from the preregistration's §7.

- **Nothing against macOS-recorded numbers.** Cross-machine Gate F FAILs
  macOS-vs-Linux on every node, by design.
- **No claim about h1024/L1's mediocrity.** H18-2 is silent on it deliberately;
  explaining it needs cells these waves do not run.
- **No claim about widths above h1024 or depths above L4.** The ladder is
  bounded at both ends. The maximum is at L2, one rung in from the shallow end,
  so the bound is not binding — but nothing here speaks past L4.
- **No re-reading of wave 15's L2 cells.** They motivated the preregistration;
  they are not evidence for it. H18-1 and H18-3 are decided on wave 18's own
  cells, and H18-4 confirms the two agree to the byte.

## 8. Provenance

112 planned across both waves, 112 complete, 0 diverged, 0 voided. One pinned
binary. Every instance's Gate F verdict is FAIL cross-machine. The cells are
landed in `results/shd_attention_campaign_v2` and the archive baseline is
re-frozen in `scripts/test_campaign_tooling.py`.
