# SHD BPTT converges at 0.7378 and does not reach 0.80 — preregistered negative result (rust arm, n=216 + convergence extension)

```
claim_axis: Integrity
object_under_test: Whether a feed-forward, fixed-threshold spiking network can
  clear the registered 0.80 SHD accuracy gate under matched BPTT, at a budget
  and width demonstrated to be sufficient.
may_claim: Under protocol shd_instrument_v4, matched BPTT on the ff+fixed-theta
  forward failed the registered 0.80 accuracy gate in all 216 rust cells, with
  every non-accuracy validity gate passing. That at published-2ms /
  adjacent-sum-5 the forward converges to 0.7378 +/- 0.0007 with both scaling
  axes closed (final epoch doubling +0.000294, final width doubling +0.000883,
  both against the registered 0.01 bound), giving a shortfall of 0.0622.
must_not_claim: That the ceiling holds beyond this contract and geometry —
  channels-700 is unrun at convergence and is now the binding scope limit; SOTA;
  Gate G2; biology or cortex; neuromorphic hardware; like-for-like comparison to
  e-prop, ETLP, or DCLS-delays; any statement about locality in general rather
  than this forward under this harness. That the forward is a rate coder — the
  direct test refutes it, see §4.
```

**Status:** rust arm complete (216/216), convergence extension complete, and
every quantitative claim below re-verified against the cells on disk on
2026-08-04 by independent recomputation from the per-cell JSON.

**Backend of record is rust.** The python arm (80/216) was superseded by
`AMENDMENT_2026-08-02_INSTRUMENT_KERNEL_AND_FRAMING.md` and is not being
completed. `matrix_verdict` is reported as **`FAIL`**, which the 216 completed
rust cells establish on the accuracy gate alone; the cross-backend criterion for
`CALIBRATED` is **explicitly unmet and will remain so**, and no verdict here
depends on it. Fixture-level cross-backend parity does hold (§7.1).

> ## CURRENT POSITION (2026-08-04) — the ceiling holds at 0.7378, on this contract and geometry
>
> **Quote `0.7378 ± 0.0007`. Shortfall to the registered 0.80 gate: `0.0622`.**
>
> The figure is converged on **both** scaling axes, each tested against the same
> registered 0.01 bound:
>
> | axis | final doubling | gain | verdict |
> |---|---|---:|---|
> | budget | e400 → e800, 3 seeds | **+0.000294** | SUFFICIENT (34× below bound) |
> | width | h512 → h1024 at e400, 3 seeds | **+0.000883** | SUFFICIENT |
>
> Training loss keeps falling ~6.4% per final decile in all three e800 seeds
> while test accuracy is flat or declining. That is the registered rule's
> **OVERFITTING** branch — *"the budget is sufficient"* — not undertraining.
>
> **The word "ceiling" is therefore supportable, with no budget qualifier and no
> width qualifier.** The remaining scope limit is **contract and geometry**: this
> is `published-2ms / adjacent-sum-5`, and `channels-700` is unrun at
> convergence. Full record:
> `RESULT_2026-08-03_BUDGET_CONVERGENCE_CEILING_RESTORED.md`.
>
> **The matrix verdict is unchanged.** All 216 cells fail `accuracy >= 0.80` on
> accuracy alone, and so do all e400 and e800 cells.
>
> ### How this document got here — withdrawal history, kept deliberately
>
> The body below was written against `0.7151`, an **e100** figure, and the
> sections still discuss it because that is what the 216-cell matrix measured.
> `0.7151` is superseded by `0.7378`; it understates the converged value by
> `0.021`. Two corrections happened in sequence and both are load-bearing:
>
> 1. **The ceiling claim was withdrawn** when the budget probe returned
>    UNDERTRAINED on a +0.0181 gain (0.7164 → 0.7284 → 0.7345 at e100/200/400).
>    `0.7151` was measuring the budget, not the architecture. The prereg's own
>    instruction was to withdraw rather than soften the prose, and that was done.
> 2. **It was restored** once the convergence rule itself was fixed. The
>    registered rule compared the *endpoints* of the ladder, so extending the
>    ladder could never escape UNDERTRAINED — the first rung stays at e100 while
>    the last climbs, making the measured gain monotonically larger the more
>    evidence was collected.
>    `AMENDMENT_2026-08-03_CONVERGENCE_RULE_FINAL_DOUBLING.md` asks the question
>    of the final doubling instead, **with the 0.01 constant unchanged** —
>    registered before the e800 cells ran.
>
> The width axis closed the same way and is the sharper lesson: at e100 the
> curve read *+0.017 per doubling and still rising*, which is why width was named
> the binding limit. At the converged budget it flattens to +0.000883. Wider
> networks reach a given loss in fewer epochs, so at a fixed short budget they
> look better; trained to convergence the advantage vanishes. **The apparent
> width trend was substantially a budget artefact**, visible only once the budget
> axis was closed. Every §4 statement about width scaling must be read with that
> correction applied.
>
> Errata against the numbers below, plus the authorization-gate regression:
> **`MEASUREMENT_2026-08-03_SHD_BUDGET_AND_ERRATA.md`**. Read it before citing
> anything here.

---

## 1. Summary

The matched BPTT matrix was run to establish a **credit-assignment ceiling** for
the feed-forward, fixed-threshold forward used throughout the `c1-shd-cal-*`
family. BPTT is the strongest credit assignment available to this architecture;
if BPTT cannot clear a bar, no local rule on the same forward can.

**That ceiling is established, but not by this matrix alone.** The matrix budget
of 100 epochs turned out to be binding, so the 216 cells below measure
BPTT-at-100-epochs. The convergence extension (banner above) closes both scaling
axes and puts the ceiling at **0.7378**, shortfall **0.0622** — on this contract
and geometry. Read the matrix as the breadth evidence and the extension as the
depth evidence; neither is sufficient alone.

Across 216 cells — 6 data contracts × 2 geometries × 3 widths × 2 epoch budgets
× 3 seeds — **not one cell reached the registered `accuracy >= 0.80` gate**,
while **every other registered gate passed in all 216 cells**.

Best configuration (`published-2ms` / `adjacent-sum-5` / `h512` / `e100`):

| Quantity | Value |
|---|---:|
| Mean accuracy | **0.7151** |
| SD across seeds (population, ddof=0) | 0.0032 |
| SD across seeds (sample, ddof=1) | 0.0039 |
| 95% CI (normal approx., sample SD) | 0.7107 – 0.7195 |
| Shortfall vs registered 0.80 gate | **0.0849** |

Both SD conventions are given because the original draft quoted the population
SD without saying so; see erratum E2. Nothing turns on the choice — both
intervals sit far below 0.80.

The same anchor, carried to convergence:

| Budget / width | Accuracy | Shortfall |
|---|---:|---:|
| e100, h512 (the matrix) | 0.7151 ± 0.0039 | 0.0849 |
| e400, h512 | 0.7369 ± 0.0021 | 0.0631 |
| e800, h512 | 0.7372 ± 0.0038 | 0.0628 |
| **e400, h1024 — converged on both axes** | **0.7378 ± 0.0007** | **0.0622** |

This is a clean negative result, not a harness failure. It is the outcome the
`PREREG_2026-07-25_SHD_ARCH_ABLATION` confound analysis anticipated, obtained
from the opposite direction. The intended inference — *"the best possible rule
does not rescue this architecture"* — was **not** licensed by the matrix alone,
because the matrix did not run the rule to convergence. It **is** licensed by the
matrix plus the convergence extension, on `published-2ms / adjacent-sum-5`, and
remains unlicensed for `channels-700`.

## 2. The threshold is registered and was not moved

`accuracy >= 0.80` (`scripts/shd_calibration/model.py:284`) is traceable, not
arbitrary. `SHD_INSTRUMENT_STATUS.md` §Why item 4 sets 0.80 as the clean-reference
bar, and the prereg's published-results table places **e-prop at 0.808** — a
local-in-time credit rule. The gate encodes "match a local-learning method on
SHD."

The bar was **not** adjusted after seeing the results, and this document makes no
argument that it should be. Per `PREREG_2026-07-25_SHD_ARCH_ABLATION` §preamble,
amendments require a new file with a new timestamp; per `SHD_INSTRUMENT_STATUS`,
the status guard "does not rewrite any historical protocol hash, result,
threshold, or verdict." A post-hoc move from 0.80 to just below 0.7151 would be
the precise forking path this apparatus exists to prevent.

Note for context only, not as a re-scored verdict: Cramer et al. report their
**recurrent** SNN at 0.714 ± 0.019 on SHD, with feed-forward architectures worse.
The e100 measurement here (0.7151 ± 0.0039) sits at that recurrent baseline with
roughly a fifth of the variance, achieved without recurrence, and the e400 anchor
exceeds it at 0.7345. This is context for interpreting *how far short* 0.0849 is
— it is not a claim of parity, since split handling, training schedule, and
readout are not matched.

## 3. The failure is isolated to accuracy

Registered gate audit, all 216 rust cells:

| Gate | Pass |
|---|---:|
| `accuracy >= 0.80` | **0 / 216** |
| `classes_predicted == 20` | 216 / 216 |
| `majority_prediction < 0.30` | 216 / 216 |
| `silent_fraction <= 0.95` | 216 / 216 |
| `saturated_fraction <= 0.05` | 216 / 216 |
| `non_finite_events == 0` | 216 / 216 |

Prereg degeneracy flags: zero `COLLAPSED`, zero `NEAR-COLLAPSED`, max
`majority_prediction` 0.1250 against a 20-class chance rate of 0.05. Mean firing
rate spans 0.108–0.331 spikes/neuron/step; `silent_fraction` never exceeds 0.008
and `saturated_fraction` is identically zero.

The networks train, spike in a healthy regime, use all 20 output classes, and
never collapse. They simply do not get past ~0.72. The single-gate failure is
what licenses reading this as an architecture statement rather than a mechanical
artifact.

## 4. The forward is resolution-invariant and capacity-limited — but it is *not* a rate coder

*This section originally concluded "the forward is rate-coding". That conclusion
was an inference from resolution invariance, and it has since been tested
directly and **refuted**. The evidence below stands; the inference drawn from it
does not. See §4.3.*

### 4.1 Accuracy is flat in temporal resolution

Timestep count (rust, all cells at each T):

| Timesteps | Mean | SD (sample) | SD (pop.) | n |
|---:|---:|---:|---:|---:|
| 100 | 0.6557 | 0.0248 | 0.0246 | 72 |
| 250 | 0.6570 | 0.0285 | 0.0283 | 72 |
| 500 | 0.6536 | 0.0372 | 0.0370 | 72 |

*Erratum E2b (2026-08-04): this table previously quoted population SDs while the
width table in §4.2 quoted sample SDs, without saying so — the same convention
mismatch erratum E2 records for §1. Both are now given. Nothing turns on the
choice.*

A **5× increase in temporal resolution moves accuracy by 0.002** (the T=100 vs
T=500 endpoints), and **no two resolutions differ by more than 0.0034** — the
trend is non-monotone, so the endpoint delta understates the spread slightly
(erratum E3). Either figure is an order of magnitude below the T-level SD of
0.025–0.037. Matched-T comparison of the `published` (fixed bin width) against
`fixed-t` (fixed step count) contracts likewise differs by at most 0.017 and
changes sign across T.

### 4.2 Accuracy is monotone in width — at a short budget only

Width, at `e100` (sample SDs, ddof=1):

| Hidden | Mean | SD | n |
|---:|---:|---:|---:|
| 128 | 0.6588 | 0.0147 | 36 |
| 256 | 0.6751 | 0.0154 | 36 |
| 512 | 0.6928 | 0.0155 | 36 |

Roughly **+0.017 per doubling** (+0.0163 then +0.0177), still climbing at 512 and
in fact mildly accelerating.

*Erratum E1: earlier drafts of this document quoted +0.034 per doubling, which is
the total across both doublings mislabelled as a rate.*

**Erratum E4 (2026-08-03) — this trend does not survive convergence.** Measured
at the sufficient budget e400, the same axis gives 0.7032 / 0.7217 / 0.7369 /
0.7378 for h128/256/512/1024, so the final doubling buys **+0.000883** and width
**saturates at h512**. The earlier extrapolation in this section — *"reaching
0.80 from h512 by width alone would need roughly five more doublings, to
h16384"* — is **withdrawn**: width does not reach 0.80 at any tested size,
because the curve is flat, not slow. Wider networks reach a given loss in fewer
epochs, so a fixed short budget rewards width for a reason unrelated to capacity.
Full record: `RESULT_2026-08-03_BUDGET_CONVERGENCE_CEILING_RESTORED.md` §4.

### 4.3 Why "rate coder" is the wrong reading

Read together, §4.1 and §4.2 say the forward is insensitive to how finely the
input is sliced in time, and sensitive to capacity until capacity saturates. The
original inference was that it must therefore be reading **rate** — a per-channel
count readout over binned frames, on a task whose discriminative content is
spatio-temporal.

That inference was registered as hypothesis H1 and **tested directly**. It is
**NOT SUPPORTED** (`RESULT_2026-08-03_SHD_TEMPORAL_INFORMATION_H1.md`). Training
and testing on data whose temporal order is destroyed — with per-channel counts
held bit-identical — costs **0.0189** with 95% CIs disjoint by 0.0120 and all six
seeds positive. A rate coder would lose nothing. This one loses a little, and
loses it reliably.

The decomposition also identifies what it *is* using. Additionally destroying
**cross-channel synchrony** costs a further **0.1248 — 6.6× the order effect**.

That reconciles the two observations rather than leaving them in tension:

| observation | consistent with a rate code? | consistent with within-bin coincidence detection? |
|---|---|---|
| flat in bin resolution (§4.1) | yes | yes — coincidence is preserved at every resolution tested |
| order shuffle costs 0.0189, reliably | **no** | yes, weakly |
| synchrony shuffle costs a further 0.1248 | **no** | yes, dominantly |

The mechanism behind the shortfall is a network reading **coincidence within
bins**, largely ignoring the sequence those bins are in. That is invariant to
resolution — which is why §4.1 looked like a rate code — but it is not a rate
code, and the difference matters for what would fix it. It remains consistent
with ETLP's conclusion that threshold adaptation and recurrence are *necessary*
for spatio-temporal structure, both absent here.

## 5. Gap decomposition

Using the prereg reference table as **context, not benchmark**:

| Segment | From | To | Δ |
|---|---:|---:|---:|
| Cost of locality on this forward | 0.234 (DFA) | 0.7378 (BPTT, converged) | **0.504** |
| Cost of this architecture | 0.7378 (BPTT, converged) | 0.951 (DCLS-delays) | **0.213** |

*Both rows are restated against the converged 0.7378 rather than the superseded
e100 figure 0.7151, which gave 0.481 / 0.236.*

The headline consequence: the historical **0.234 DFA number is architecture-bound
as well as locality-bound**. It was previously read as a statement about local
credit assignment; with exact gradients on the same forward reaching 0.7378, most
of that gap is credit assignment, and the DFA number cannot be read as an
architecture statement on its own.

**The stronger version of this claim was withdrawn, and is now partially
restored.** An earlier draft argued the residual to the gate "cannot be
attributed to locality at all — it is unreachable even with exact gradients."
That requires the BPTT figure to be a ceiling, which the budget probe refuted at
the time. It has since been re-established at 0.7378, converged on budget and
width, so the argument holds **on this contract and geometry**: the residual
0.0622 is not reachable by this forward under exact gradients at any tested
budget or width, and therefore cannot be charged to locality.

The scope limit that remains is the one that now does all the work:
`channels-700` is unrun at convergence, so the second row is a measurement for
`adjacent-sum-5` and a lower bound on the architecture cost in general.

Caveat carried from the prereg: the 0.234 figure came from capped splits and a
different training schedule, and is not like-for-like with these uncapped
8156/2264 runs. The decomposition is indicative of magnitude, not a matched
contrast.

## 6. Exposure hygiene — secondary result

The instrument reproduces the pinned DCLS-delays reference
(`Thvnvtos/SNN-delays`, `reference.py:16`) under two disclosed exposure regimes:

| Regime | Selection | Test reads | Accuracy |
|---|---|---:|---:|
| Historical | `official-test-best-accuracy`, `max()` over curve | 150 | 0.9498 (mean) |
| Clean | final epoch, no checkpoint selection | 1 | 0.9390 / 0.9368 / 0.9371 |

The ~0.011 delta is the selection bias induced by choosing the best of 150 test
evaluations. The historical path is labeled `EXPOSURE_TAINTED_DESCRIPTIVE` in
code (`reference.py:50-62`) rather than in prose, which is the point: the taint
is machine-checkable. Quantifying this against a pinned published recipe is a
methods contribution independent of the BINN result.

## 7. Threats to validity

1. **Python arm incomplete (80/216), superseded, and not being completed.**
   End-to-end cross-backend agreement is therefore *not* established across the
   full matrix, and `matrix_verdict` **cannot and will not** return `CALIBRATED`.
   It is reported as `FAIL`, which the rust cells establish on accuracy alone.
   Fixture-level parity does hold — forward 1e-6, gradient 1e-4, update 1e-5,
   plus data parity and fresh-process replay — which is a tighter equivalence
   check than the 0.05 end-to-end accuracy band, but it is not the registered
   criterion. This writeup states which was met; any other must do the same.
2. **Two hyperparameters inherited, not swept.** The one-cycle LR schedule and
   `SURROGATE_ALPHA = 5.0` come from the calibration lineage and were not tuned
   for this forward. Direction of bias unknown, and this is now the **strongest
   remaining threat** to the ceiling reading, since budget and width are both
   closed. Note that the schedule scales with the total step count, so the e400
   and e800 runs are independent full runs with stretched schedules rather than
   continuations — the convergence test is therefore about the budget and not an
   artefact of a decayed learning rate.
3. **Single architecture.** ff + fixed θ only. The prereg's `rec+alif` arm is not
   covered by this matrix; nothing here speaks to it. It is also **unmeasured
   rather than refuted** — `rec+alif` at h512 produces zero usable cells and
   gradient clipping does not fix it
   (`MEASUREMENT_2026-08-03_GRADIENT_CLIPPING_DOES_NOT_FIX_H512.md`). The blocker
   is an instrument defect at that width, not evidence about recurrence.
4. **The ceiling argument assumes BPTT dominates.** BPTT is the strongest
   practical credit assignment for this forward, but "no local rule can beat it"
   is an empirical regularity, not a theorem.
5. **Epoch budget — THIS THREAT MATERIALISED, AND IS NOW CLOSED.**
   The concern was that 100 epochs, an untuned schedule, and width still climbing
   at h512 (0.6588 → 0.6751 → 0.6928, +0.017 per doubling, erratum E1) meant
   0.7151 might measure the budget rather than the architecture.

   **It did.** `scripts/probe.py budget` ran the anchor at 100/200/400 epochs and
   the registered rule returned **UNDERTRAINED** on a +0.0181 test-accuracy gain
   (0.7164 → 0.7284 → 0.7345). The central claim was withdrawn throughout this
   document at that point — per the prereg's instruction, withdraw and record
   rather than soften the prose.

   **It is now discharged.** The ladder was extended to e800 under an amended
   convergence rule (final doubling, 0.01 constant unchanged, registered before
   the cells ran): the final doubling buys **+0.000294** across three seeds with
   training loss still falling ~6.4% per final decile — the **OVERFITTING**
   branch. Width closed the same way at **+0.000883**. The ceiling reading is
   reinstated at **0.7378 ± 0.0007**, shortfall **0.0622**, with no budget and no
   width qualifier. Detail: `MEASUREMENT_2026-08-03_SHD_BUDGET_AND_ERRATA.md` §1
   and `RESULT_2026-08-03_BUDGET_CONVERGENCE_CEILING_RESTORED.md`.

   **What replaces it as the binding scope limit: contract and geometry.**
   `channels-700` is unrun at convergence.
6. **Overfitting was diagnosed, not measured against a validation split.** SHD
   holds out speakers and no separate validation set was used, so the
   OVERFITTING verdict rests on train-loss-falling with test-accuracy-flat. That
   is the registered criterion, but it is weaker than an explicit early-stopping
   curve.
7. **Three seeds at the convergence extension**, df=2. The e800 95% CI is
   [0.7277, 0.7467] — wide. The convergence claim rests on the point estimate of
   the *difference* being 34× below its bound plus the overfitting signature, not
   on a tight interval around the level.

8. **The harness authorization gate has since gone red.** `harness_status` is now
   `PENDING_PREREQUISITES`, with `historical_reference`, `clean_reference` and
   `matrix_authorized` all false. The reference runs themselves are intact — all
   six state hashes match and all accuracy and exposure checks pass — but
   `SOURCE_PATHS` folds the instrument kernel into the same fingerprint that
   guards the third-party reference, and the 2026-08-02 kernel edit changed it.
   The Provenance block below is therefore stale; see the errata file §3.

## 8. What this supports

**Defensible:** an isolated single-gate failure across 216 cells at the
registered 100-epoch budget, with every validity gate passing and zero degeneracy
— i.e. a healthy network that simply does not clear 0.80; that the same forward
**converges** to 0.7378 ± 0.0007 on `published-2ms / adjacent-sum-5` with both
scaling axes closed, giving a shortfall of 0.0622 that is not attributable to
budget or width; a mechanism — **within-bin coincidence detection**, evidenced by
resolution invariance *plus* the direct order/synchrony decomposition (§4.3); and
the exposure-bias methods result in §6, which is independent of all of the above.

**Not defensible:** that the ceiling extends beyond this contract and geometry
(`channels-700` is unrun at convergence — §7.5); that the forward is a **rate
coder** (directly refuted, §4.3); that width scaling continues past h512
(erratum E4); any SOTA framing; any claim of parity with Cramer et al., e-prop,
ETLP, or DCLS-delays; any statement about locality in general; anything about
`rec+alif`, which is unmeasured rather than refuted (§7.3); and any claim resting
on a currently-`VALID` harness (§7.8).

**Recommended next tests, in order:**

1. ~~Convergence probe~~ **DONE 2026-08-03.** Returned UNDERTRAINED; see the
   banner, §7.5, and `MEASUREMENT_2026-08-03_SHD_BUDGET_AND_ERRATA.md`.
2. ~~Re-measure the anchor at e400~~ **DONE 2026-08-03**, and
   ~~extend to e800 under the amended rule~~ **DONE 2026-08-03** —
   0.7378 ± 0.0007 converged, shortfall 0.0622.
   ~~Width axis~~ **CLOSED 2026-08-03** at +0.000883 on the final doubling.
3. ~~Temporal-information experiment~~ **DONE 2026-08-03**
   (`RESULT_2026-08-03_SHD_TEMPORAL_INFORMATION_H1.md`). The registered
   expectation stated here — *"if order-invariance holds, the shortfall is a
   rate-code limit"* — **did not hold**. Order-invariance was refuted (0.0189,
   CIs disjoint, six of six seeds), and the positive result is the
   order/synchrony decomposition instead: synchrony is worth 6.6× order. §4.3
   carries the reconciliation.
4. **`channels-700` at convergence** — now the binding scope limit on the ceiling
   claim, and the cheapest remaining test of it. Three seeds at e400, h512,
   `published-2ms / channels-700`, against the existing `adjacent-sum-5` anchor.
5. **H1 at a converged budget.** The temporal campaign ran at e100, now *known*
   undertrained by 0.021. All conditions share the budget so the contrast is
   internally valid, but whether the order effect grows or shrinks with training
   is untested and is the most likely reviewer objection. Re-running the 24 cells
   at e400 answers it, and needs its own registered extension.
6. **rec+alif BPTT arm** (`PREREG_2026-08-02_SHD_BPTT_CEILING_RECALIF`). Now
   **blocked on an instrument defect, not on compute**: h512 produces zero usable
   cells and clipping fires downstream of the failure. Needs a registered model
   change — truncated BPTT, lower surrogate gain, spectral-radius-normalised
   init, or a narrower width. With the ff+fixed ceiling restored at a converged
   budget, a rec+alif number measured at the *same* budget would now decide the
   architecture question, which it could not while the comparison point was
   budget-limited.

---

**Provenance.** `results/shd_instrument_v4/` — 216 rust cells,
`manifest_sha256` `7f612774972ccd61dd6af283fb9568c6eba88920185bddc212fed269bf41c52d`
(re-verified 2026-08-03).

Convergence extension, produced under binary `8c169a659c3c…` with Gate F 13/13
`BIT_IDENTICAL`: `probe/` (e100/e200/e400 ladder, e400 × 3 seeds),
`budget-e800/` (e800 × 3 seeds), `width-converged/` (h128/h256/h1024 at e400 × 3
seeds each). Gate F has now passed 13/13 across **9 distinct binary hashes** over
11 runs, so the determinism claim is not a single-build coincidence.

**Independent re-derivation, 2026-08-04.** Every quantitative claim in this
document was recomputed from the per-cell JSON rather than carried forward from
an earlier draft: the 216-cell gate audit (0/216 accuracy, 216/216 on the other
five), the anchor under both SD conventions, the T-level and width tables, the
budget and width final doublings, the overfitting diagnosis per seed, and the
temporal decomposition cited in §4.3. All reproduced. The instrument's 160 tests
were re-run and pass.

*At the time the matrix ran*, under source fingerprint `64923d64…`: `data_parity`,
`forward_parity`, `gradient_parity`, `update_parity`, `fresh_process_replay`,
`historical_reference`, `clean_reference`, `matrix_authorized` all true;
`harness_status: VALID`.

*As of 2026-08-03*, under fingerprint `4b85606d…`: the four parity gates and
`fresh_process_replay` remain true; `historical_reference`, `clean_reference` and
`matrix_authorized` are **false** and `harness_status` is
**`PENDING_PREREQUISITES`**. The reference artifacts are unchanged and all their
content checks pass — only the fingerprint equality fails, because the
2026-08-02 instrument-kernel edit is inside the fingerprint that also guards the
third-party reference. See `MEASUREMENT_2026-08-03_SHD_BUDGET_AND_ERRATA.md` §3.
This document must not be cited as resting on a currently-`VALID` harness.

Two disclosed interruptions: `2026-07-27-worker-session-loss`,
`2026-07-28-user-travel-stop`; 0 result files removed in both.

**References.**
Cramer et al., *The Heidelberg spiking datasets*, arXiv:1910.07407.
Hammouamri, Khalfaoui-Hassani & Masquelier, *Learning Delays in SNNs using
Dilated Convolutions with Learnable Spacings*, arXiv:2306.17670 (ICLR 2024).
Quintana et al., *ETLP*, 2024 — via prereg reference table.
