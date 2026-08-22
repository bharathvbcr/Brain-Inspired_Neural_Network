# Preregistration — does attention *add* temporal structure, or *substitute* for adaptation?

**Registered:** 2026-08-22, **before any wave-12 cell exists** and before the
fleet is launched.
**Campaign:** `shd_attention_campaign_v2`, wave 12, same bucket, **same pinned
binary** `22d97c51ab0204702ce44661683ff8c759c29d7f3379e2f6606b048f4f032104` as
waves 1–11, now known to be behaviourally reproduced by the current source
(`RESULT_2026-08-22_SOURCE_REPRODUCES_THE_PINNED_BINARY.md`).

---

## 1. The question the campaign cannot currently answer

Every cell in waves 1–10 that carries the anchor configuration is on `ff+fixed`.
Nothing in 720 cells varies the substrate. So the headline gain of **+0.1258**
has two readings and the record cannot separate them:

* **Addition** — attention supplies temporal structure no spiking substrate of
  this kind can represent, and would help on any of them.
* **Substitution** — attention stands in for the threshold adaptation `ff+fixed`
  happens not to have, and its advantage would shrink on a substrate that has it.

ETLP's own conclusion, quoted in `binn-lab/experiments/shd_arch_ablation.rs:16`,
is that threshold adaptation and a recurrent topology are what a spiking network
needs for rich temporal structure. Neither is present in `ff+fixed`. Attention
was added instead of either, and the factorial has never been run.

This matters beyond bookkeeping. Under **substitution**, the paper's mechanism
claim is about a deficiency of one substrate rather than about attention; under
**addition**, it is about attention.

## 2. Design — 24 new cells, 24 reused, n = 12

| label | arm | hidden | epochs | contract | geometry | attention |
|---|---|---:|---:|---|---|---|
| `w12ada` | `ff+alif` | 128 | 400 | `published-2ms` | `adjacent-sum-5` | — |
| `w12ada` | `ff+alif+attn` | 128 | 400 | `published-2ms` | `adjacent-sum-5` | d32/L4 |

The `ff+fixed` corners are **not** re-run. Twelve seeds of `ff+fixed` (`w1`) and
twelve of `ff+fixed+attn` at d32/L4 (`r1cal`) exist at exactly this
configuration, from the same pinned binary, and are reused under the manifest
hash check waves 8 and 9 use: the analysis refuses to report unless every reused
cell matches its recorded hash.

Seeds are the standing lineage `5170001–5170012`. No clipping, no surrogate
scaling: any deviation there would make these cells incomparable to the reused
controls, which ran at the registered defaults.

### Why the recurrent half is not in this wave

Deferred on measurement, not on preference. Wave 11 ran the recurrent arms
unclipped at h256/e100 and completed **15 of 24** — `rec+alif` 7 of 12,
`rec+alif+attn` 8 of 12. The campaign's validity rule is that an arm with any
diverged cell reports **zero** usable cells and never a mean over survivors, so a
12-seed recurrent arm at ~60% per-cell completion cannot carry a verdict at all.
Running it would predictably spend the expensive half of a wave to learn what
wave 11 already showed. Making those arms complete is a numerical-stability
question and needs its own registration.

**This is a scope limit and is stated as one.** A substitution result on the
adaptation axis does not settle the recurrence axis, and §6 says so.

## 3. Hypotheses and thresholds

Fixed here. Every verdict computed **once**, after all 24 cells settle.

Write `gain(S) = mean(S + attn) − mean(S)`. The reused controls give
`gain(ff+fixed) = +0.1258`.

| id | claim | threshold |
|---|---|---|
| **A-1** *(primary, two-sided)* | the attention gain depends on whether the substrate adapts | \|gain(`ff+alif`) − gain(`ff+fixed`)\| ≥ **0.03**, with the **sign reported** |
| **A-2** | attention still helps on an adaptive substrate | gain(`ff+alif`) ≥ **0.05** and positive in ≥ **10 of 12** seeds |
| **A-3** | adaptation alone is not the whole story | mean(`ff+alif`) reported against mean(`ff+fixed`) = 0.7062 and against the 0.80 gate; ≥ 9/12 seeds ≥ 0.80 required to claim the gate |
| **A-4** | the best arm | **reported, no verdict** — see §5 |
| **A-5** | stability | zero non-finite events and zero diverged cells across all 24 |

### Why A-1 is two-sided

I do not have a theory that predicts the sign, and I am registering **after**
seeing a related number: wave 11's `rec+alif+attn` scored 0.68–0.78 against
`rec+alif` at 0.45–0.50 at h256/e100, an apparent gain near **+0.28** on a
substrate that has both adaptation *and* recurrence. That points away from
substitution. It is also a different width, a different budget, a different
attention depth, unmatched seeds and no registered comparison — which is exactly
why it may not be used to pick a direction here. Registering one-sided now would
be choosing with knowledge of related data, the move preregistration exists to
prevent.

So the bar is on **magnitude**, the sign is reported, and both directions have
named consequences.

## 4. Named outcomes

| id | outcome | means |
|---|---|---|
| **A-1, gain shrinks** | attention was partly standing in for adaptation. The paper's mechanism claim narrows to *this* substrate and must say so; the honest framing becomes "a rate read-out over a non-adapting substrate cannot use order, and either adaptation or attention restores some of it". |
| **A-1, gain grows or holds** | attention supplies something adaptation does not. The mechanism claim generalises across the adaptation axis, and the read-out — not the substrate's deficiency — is what the result is about. |
| **A-1 flat** (< 0.03) | adaptation is irrelevant to the read-out's advantage. Substitution is refuted on this axis, and the M-1 shuffle result stands as the whole mechanism. |
| **A-2 NOT SUPPORTED** | attention does not help an adaptive substrate, which would be strong evidence for substitution regardless of A-1's magnitude. |
| **A-3 gate cleared by `ff+alif`** | the biologically-motivated fix reaches the registered gate without attention. That is a materially different paper. |
| **A-5 fails** | the adaptive feed-forward arm is not stable at e400, and no verdict above is reportable. |

## 5. What this wave may not claim

* **No winner by inspection.** If `ff+alif+attn` is the highest-scoring arm, that
  is reported with its seeds and converted into no claim. Wave 9's M-3 is the
  precedent: a difference on an untested axis is an estimate, and promoting it is
  the exact move registration prevents.
* **Nothing about recurrence.** See §2.
* **Nothing about `channels-700`, h512, h1024, or any contract but the anchor.**
* **Not calibration.** The instrument stays `Uncalibrated`.
* No comparison to macOS-recorded numbers.

## 6. Validity gates

Per cell, enforced by the single owner in `scripts/cell_validity.py`:
`mechanical_status == COMPLETE`, `non_finite_events == 0`,
`classes_predicted == 20`, `majority_prediction < 0.30`,
`silent_fraction ≤ 0.95`, `saturated_fraction ≤ 0.05`, and the cell's
`temporal_condition` must equal the plan's.

**Stability warnings are expected and do not void.** Wave 11's completing
recurrent cells carried peak gradient norms from 1e10 to 1e34. `ff+alif` has no
recurrent recursion and should stay far below that, but any cell above 1e9 will
be reported as a stability note beside its accuracy. A warned cell is a cell to
read carefully, not a defect — registered here so that it cannot later be
treated as one.

Wave-level: every cell settles or is reported `DIVERGED`; an arm with any
diverged cell reports **0 usable**, never a mean over survivors; every instance
must report the pinned binary hash or the campaign is void.

## 7. Stopping rule and cost

Fixed at 24 cells. No cell added, dropped, re-seeded, or re-run on the basis of
its result. A cell that does not finish is reported unfinished, not replaced.

The `ff+*+attn` d32/L4 cells at this contract are the wall-clock floor: the
recorded `r1cal` controls took **20,322–20,806 s** each. Expect ~6 hours with
every cell in flight at once, on ~96 vCPU of spot — roughly $10.
