# Temporal information in SHD: order matters a little and reliably; cross-channel synchrony matters 6.6x more

**Date:** 2026-08-03
**Prereg:** `PREREG_2026-08-02_SHD_TEMPORAL_INFORMATION.md`
**Amendment:** `AMENDMENT_2026-08-03_H1_SEED_EXTENSION.md` (3 → 6 seeds, stopping rule binding)
**Deviation record:** prereg §5b, written before any result was read
**Backend:** rust only. Binary `8c169a659c3c`, Gate F 13/13 PASS
**Cells:** 24 (`ff+fixed`), `published-2ms / adjacent-sum-5 / h512 / e100`, seeds 5170001-6
**Verdict tool:** `scripts/temporal_campaign_verdict.py`, thresholds hardcoded before any manipulated cell completed

> **The 3-seed verdict was H1 SUPPORTED. The binding 6-seed verdict is H1 NOT
> SUPPORTED.** Both are reported; the 6-seed one governs. §4.1 explains why the
> flip happened and why the extension was registered before it was run.

---

## claim_axis

```
axis: task-structure
claim: For this instrument on SHD, destroying temporal ORDER costs a small but
  statistically reliable amount of accuracy. Additionally destroying
  CROSS-CHANNEL SYNCHRONY costs 6.6x more. The two are separable and were
  separated.
may_claim: That at published-2ms / adjacent-sum-5 / h512 / e100 over 6 seeds,
  ff+fixed trained and tested on bin-shuffled data loses 0.0189 accuracy with
  95% CIs disjoint by 0.0120; on channel-shuffled loses 0.1437; on reversed
  loses 0.0032. That the synchrony-specific increment is 0.1248, 6.2x the
  registered H3 bound and 6.6x the order effect. That H3 is confirmatory,
  because H1 failed and the prereg conditioned H3 on exactly that.
must_not_claim: That ff+fixed is a rate coder — H1 is NOT SUPPORTED. Nor that
  temporal order is unimportant in absolute terms: the effect is below the
  registered practical bound of 0.02 while being statistically reliable, and
  both halves of that sentence are needed. That this is converged: e100 is
  known undertrained (§6.1). That the 0.1248 synchrony increment is purely an
  information effect — channel-shuffling also shifts the network's operating
  regime, so it is an UPPER BOUND until measured at matched activity (§6.7).
  The 0.0189 order effect is not subject to that caveat. Anything about
  rec+alif — H2 was NOT RUN (§4.3). Anything about SHD generally, other
  contracts, geometries, widths, or architectures. Nothing here is about BINN.
```

## 1. Design

Train **and test** on manipulated data — which is what separates this from a
test-time perturbation probe, whose effects are confounded with distribution
shift.

| condition | destroys | preserves |
|---|---|---|
| `intact` | — | — |
| `bin-shuffled` | temporal order | per-channel counts, within-bin synchrony |
| `channel-shuffled` | order **and** cross-channel synchrony | per-channel counts |
| `reversed` | direction | order magnitude, synchrony, counts |

The `bin-shuffled` / `channel-shuffled` contrast is the novel part. Published
shuffle controls routinely conflate order with synchrony; this separates them by
construction, and the separation is where the result lives.

## 2. Result — 6 seeds

| condition | mean | sd | 95% CI (t=2.571, df=5) |
|---|---:|---:|---|
| `intact` | **0.7158** | 0.0033 | [0.7123, 0.7192] |
| `bin-shuffled` | **0.6968** | 0.0033 | [0.6934, 0.7003] |
| `channel-shuffled` | **0.5721** | 0.0051 | [0.5667, 0.5775] |
| `reversed` | **0.7126** | 0.0046 | [0.7078, 0.7175] |

| manipulation | cost vs intact |
|---|---:|
| reverse time | **0.0032** |
| destroy order | **0.0189** |
| destroy order **and** synchrony | **0.1437** |
| **synchrony-specific increment** | **0.1248** — **6.6x the order effect** |

Per-seed `intact − bin-shuffled`: 0.019876, 0.020318, 0.019435, 0.020318,
0.018551, 0.015018. **All six positive.**

## 3. Validity gates — all pass

| gate | requirement | measured |
|---|---|---|
| 5.0 pipeline sensitivity | membrane rel L2 ≥ 0.1 untrained | 0.9264 worst of 18 configurations |
| 5.1 manipulation | counts bit-identical, entries relocated | PASS, all 18 manipulated cells |
| 5.2 trained regime | every intact cell ≥ 0.65 | min 0.7107 |
| numerical | `non_finite_events` = 0 | 0 across all 24 |

The three original intact cells reproduce the recorded 216-cell campaign
**bit-identically** (0.716431095 / 0.710689046 / 0.718197880) while still passing
through `apply_temporal` as an identity — so the manipulation harness adds no
artifact to the control arm.

## 4. Registered hypotheses

### 4.1 H1 — NOT SUPPORTED

> **H1:** `|intact − bin-shuffled| ≤ 0.02` **and** overlapping 95% CIs.

| criterion | value | verdict |
|---|---|---|
| mean difference ≤ 0.02 | **0.018919** | **PASSES** (margin 0.0011) |
| 95% CIs overlap | intact [0.7123, 0.7192] vs bin [0.6934, 0.7003] | **FAILS — disjoint by 0.0120** |

The prereg required **both**. H1 is **NOT SUPPORTED**.

**The two criteria disagree, and that disagreement is the finding.** The effect
of destroying temporal order is *smaller than the registered bound for practical
negligibility* and *statistically reliable* at the same time. Both halves are
true and the result is neither "order matters" nor "order doesn't matter" — it is
**order matters a little, and consistently**: all six seeds positive, ranging
0.0150 to 0.0203.

**Why the verdict flipped from 3 seeds to 6.** At n=3 the mean difference was
0.019876 and the CIs overlapped by 0.0006 — H1 SUPPORTED, by 0.62% of its bound.
At n=6 the standard deviation is essentially unchanged (0.0033), but t falls from
4.303 to 2.571, so the CIs narrow from ±0.0097 to ±0.0035 and the same real gap
between the means becomes statistically visible. **Nothing about the effect
changed; the resolution did.**

This is exactly the direction the amendment predicted in advance: *"adding seeds
makes H1 harder, not easier, on the CI criterion."* It was registered with a
binding stopping rule — **exactly three seeds added, verdict computed once,
reported whichever way it falls, no seventh seed** — before seed 5170004 was
run. Had that rule not been fixed in advance, the honest reading of a 3-seed
SUPPORTED flipping to a 6-seed NOT SUPPORTED would be unavailable: it would look
like sampling until the preferred answer appeared. It is available precisely
because the count was committed first.

**The 3-seed verdict is superseded, not withdrawn.** Publishing "ff+fixed is a
rate coder" on n=3 would have been wrong.

### 4.2 H3 — SUPPORTED, and confirmatory

> **H3:** `channel-shuffled` worse than `bin-shuffled` by ≥ 0.02.
> Registered as *"confirmatory only if H1 fails."*

Measured: **0.124779**, 6.2x the bound, CIs disjoint by a wide margin.

**H1 failed, so H3's confirmatory status activates exactly as registered.** This
is a confirmatory result under the original analysis plan, with no post-hoc
reclassification. The condition the prereg set was met.

The prereg's *rationale* for the conditioning — *"if the solution is
order-invariant there is no synchrony effect to decompose"* — turned out to be
unnecessary rather than wrong, since the solution is not order-invariant. The
conditioning was still poorly reasoned: `bin-shuffled` vs `channel-shuffled`
isolates synchrony whether or not order matters, because both destroy order and
only one destroys synchrony. A future prereg should register it unconditionally.

### 4.3 H2 — NOT RUN

`rec+alif` is registered at h512, the configuration measured the same day to
produce **zero usable cells across three seeds** (two abort on non-finite
gradient entries; one reaches a gradient norm of 7.36e29). See
`MEASUREMENT_2026-08-03_RECURRENT_ARM_STABILITY.md` §3.6.2.

H2 is **unmeasured** — not failed, not refuted. The blocker is an instrument
defect, not evidence about recurrence.

**Retried at h256 and still NOT RUN.** `AMENDMENT_2026-08-03_H2_AT_H256.md`
moved the width to h256, which had measured clean at e20. At the campaign budget
e100 it is not: 2 of 12 `rec+alif` cells abort (seed 5170002, steps 374 and 727).
The amendment's stopping rule required reporting NOT RUN rather than evaluating
the 10 survivors, and that was honoured. The e20 evidence covered 640 of the
3200 optimizer steps a campaign cell runs — see §7 of that amendment.

## 5. Reversal, and a post-hoc prediction that held

`intact − reversed = 0.0032` — reversal is nearly free, and cheaper than
bin-shuffling despite also destroying global sequence.

§4b of `MEASUREMENT_2026-08-03_TEMPORAL_SENSITIVITY_POSITIVE_CONTROL.md`
predicted `reversed ≈ intact` from a *test-time* probe of trained weights, and
recorded that prediction as post-hoc and unregistered. It held. It is reported as
a successful post-hoc prediction — weaker evidence than a registered one — and is
not upgraded.

The three costs order consistently and interpretably:

| manipulation | local structure | cross-channel synchrony | global direction | cost |
|---|---|---|---|---:|
| `reversed` | preserved | preserved | destroyed | 0.0032 |
| `bin-shuffled` | destroyed | within-bin preserved | destroyed | 0.0189 |
| `channel-shuffled` | destroyed | destroyed | destroyed | 0.1437 |

Direction is worth nothing. Order is worth a little. Synchrony is worth most of
it.

## 6. Caveats

1. **e100 is undertrained for this configuration.** The registered budget rule
   returned UNDERTRAINED: 0.7164 → 0.7284 → 0.7345 at e100/200/400, and 0.7369 ±
   0.0021 at e400, still rising. All conditions share the budget, so the
   contrast is internally valid — but whether the order effect grows or shrinks
   with training is **untested**, and it is the most likely reviewer objection.
   It needs its own registered extension.
2. **One contract, one geometry, one width.** The resolution invariance in
   prereg §1 suggests the contract matters little; that is an inference, not a
   measurement here.
3. **`ff+fixed` only** — dense single-hidden-layer LIF, no recurrence, fixed
   threshold. Says nothing about architectures that could exploit timing.
4. **Effect sizes are small in absolute terms.** 0.0189 on a 0.7158 baseline.
   Reliable is not the same as large.
5. **Instrument, not BINN.** Every number is about the matched SHD BPTT
   instrument.
6. **Not a SOTA comparison.** 0.7158 against a 0.939 reference; the instrument
   is deliberately simple.
7. **`channel-shuffled` shifts the operating regime; the synchrony increment is
   an upper bound.** *Added 2026-08-04 during independent re-derivation of the
   cells. This was not caught by the registered gates, which are written to
   detect collapse, not a regime shift inside them.*

   Per-channel input counts are preserved bit-identically by construction — that
   is gate 5.1 and it passes. But the *hidden layer* does not see the same
   regime across conditions:

   | condition | mean firing rate | `saturated_fraction` | occupied bins/sample |
   |---|---:|---:|---:|
   | `intact` | 0.199 | **0.0000** (all 6 seeds) | 302.9 |
   | `reversed` | 0.196 | **0.0000** (all 6 seeds) | 302.9 |
   | `bin-shuffled` | 0.234 | **0.0000** (all 6 seeds) | 302.9 |
   | `channel-shuffled` | 0.260 | **0.023 – 0.035** (all 6 seeds) | 357.9 |

   Independent per-channel permutation spreads each channel's spikes across more
   distinct bins, which raises drive. All six `channel-shuffled` cells stay
   inside the registered `saturated_fraction <= 0.05` gate, but at 46–70% of it
   rather than at zero like every other condition.

   **Consequence, stated precisely.** Some unquantified share of the 0.1248
   synchrony-specific increment may be the network being pushed toward
   saturation rather than a loss of usable information. **0.1248 should be read
   as an upper bound on the synchrony effect** until it is re-measured at
   matched hidden activity — e.g. by re-normalising input scale per condition to
   equalise mean firing rate, which is a registered change, not a re-analysis.

   **H1 is untouched by this.** `bin-shuffled` — the condition that carries the
   entire order effect — has `saturated_fraction` of exactly 0.0000 in all six
   seeds, identical to `intact`. Its firing-rate shift (0.199 → 0.234) is real
   but stays in the same non-saturating regime. The 0.0189 order effect and the
   H1 NOT SUPPORTED verdict do not depend on this caveat.

   H3 remains SUPPORTED regardless of how the caveat resolves: the increment is
   6.2× its registered bound, and no plausible share of a sub-4% saturation
   shift accounts for a 0.1248 accuracy drop. What is at stake is the *size* of
   the synchrony effect, not its existence or its confirmatory status.

## 7. What this supports

> For a feed-forward LIF network trained by BPTT on SHD at this configuration,
> the information extracted from temporal structure is dominated by
> **cross-channel synchrony**, not **temporal order**. Destroying order costs
> 0.0189 — small, below the pre-registered bound for practical negligibility,
> but statistically reliable across six seeds. Destroying synchrony as well
> costs a further 0.1248 (**6.6x more**, and an upper bound per §6.7). Reversing
> time costs 0.0032.

The decomposition is the contribution. The claim that the network is a *rate
coder* is **not** supported: it is order-sensitive, just weakly.

The mechanism this points to is **within-bin coincidence detection**, which also
resolves an apparent conflict with the matrix result. The 216-cell matrix found
accuracy flat in temporal resolution and read that as evidence of rate coding;
coincidence detection is *equally* invariant to how finely time is sliced, while
being order-sensitive and synchrony-dominated exactly as measured here. The
reconciliation is written up in `SHD_BPTT_CEILING_NEGATIVE_RESULT.md` §4.3, and
that document's rate-coding inference has been withdrawn on the strength of this
one.
