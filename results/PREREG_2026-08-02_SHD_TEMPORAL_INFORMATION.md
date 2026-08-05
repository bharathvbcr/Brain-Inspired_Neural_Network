# Preregistration — does the matched SHD instrument use spike timing?

**Registered:** 2026-08-02, before any confirmatory run of `shd-temporal-information`.
**Instrument:** `shd_instrument_v4` matched BPTT, arms per `shd_matched_arms`.
**Claim axis:** what information the learned solution actually uses.

```
claim_axis: Novel-CS
object_under_test: Whether a surrogate-gradient BPTT solution on SHD depends on
  temporal order at all, or only on per-channel spike counts.
may_claim: Under this protocol, destroying temporal order changed accuracy by
  the measured amount, for the named arms, at the named configuration.
must_not_claim: SOTA; that SHD contains no temporal information; that surrogate
  gradients cannot learn timing in general; anything about biology.
```

---

> **2026-08-03 — a defect that would have invalidated the rec+alif half was
> found and FIXED before any cell ran.**
> The recurrent forward read partially-updated spikes within a timestep while the
> backward differentiated the clean-previous-step model, so the `rec+*` gradient
> was not the gradient of the `rec+*` forward (divergence order-1 against a unit
> threshold). It is fixed and pinned by a test that reimplements the forward
> independently; `ff+fixed` was never affected and is verified bit-identical
> across the change. See
> `DEFECT_2026-08-03_RECURRENT_ARM_FORWARD_BACKWARD_MISMATCH.md`.
>
> Had this run as written, 12 of the 24 cells, hypothesis **H2**, and the §5.6
> `W_rec`-scale pilot would all have been measured on a broken gradient. Note
> §4b of that report: **no gradient-tolerance check could have caught it**, so
> Gate E as currently specified would not have protected this campaign either.
>
> **This design is no longer blocked by the defect.** It remains blocked by
> authorization — `matrix_authorized` is false and `SHD_INSTRUMENT_STATUS.md`
> blocks new architecture campaigns while `UNCALIBRATED`
> (`MEASUREMENT_2026-08-03_SHD_BUDGET_AND_ERRATA.md` §3). The ff+fixed half is
> also now ~2.8× cheaper (`AMENDMENT_2026-08-03_RUST_KERNEL_TRANSPOSE.md`),
> putting it near 1.7 h.

> **ERRATUM 2026-08-03 — registered text below is unchanged; two of its stated
> facts are wrong.** Recorded here rather than edited in place, per the
> §preamble rule that registered documents are amended, not rewritten.
>
> 1. **§1 "width moves it by +0.034 per doubling" is a factor of two out.** The
>    measured rate is **+0.017 per doubling** (+0.0163 for 128→256, +0.0177 for
>    256→512); +0.034 is the total across both doublings. The motivating
>    contrast — resolution invariance versus width sensitivity — survives
>    unchanged, since 0.017 is still five times the T-spread of 0.0034.
> 2. **§2 and §5.2 call 0.7151 "the instrument's known ceiling". It is not.**
>    The budget probe returned **UNDERTRAINED**: the same anchor reaches 0.7284
>    at e200 and 0.7345 at e400. 0.7151 is the 100-epoch measurement.
>
> **No hypothesis, threshold, condition, or decision rule changes.** Gate 5.2's
> floor of 0.65 is registered as an absolute number and still passes — the e100
> anchor measures 0.7164. But its *rationale* ("within 0.07 of the measured
> ceiling") no longer holds, and if the confirmatory run is executed at a longer
> budget the floor should be re-registered in a new file first, not adjusted
> in place. Detail: `MEASUREMENT_2026-08-03_SHD_BUDGET_AND_ERRATA.md`.

## 1. Why this, instead of another ceiling number

The completed rust arm already contains an unexplained regularity. Accuracy is
**flat in temporal resolution** — T = 100 / 250 / 500 gives 0.6557 / 0.6570 /
0.6536, so a 5× change in resolution moves accuracy by 0.002, two orders of
magnitude below seed spread — while **width** moves it by +0.034 per doubling and
is still rising at h512.

That pattern says the solution may not be using spike timing at all. But
resolution invariance is indirect evidence: coarser bins could preserve whatever
timing the model uses. The direct test is to destroy temporal order while
holding spike counts exactly fixed.

This matters beyond the project. Recent work argues surrogate gradients *do*
enable spike-timing learning. If a well-controlled BPTT solution on the canonical
SHD benchmark turns out to be order-invariant, that is a substantive
qualification of that claim, and it is a positive finding rather than another
failed threshold.

## 2. Pilot — suggestive, weak, and explicitly not evidence

A capped pilot (1200/300 split, 12 epochs, h128, published-10ms/adjacent-sum-5,
2 seeds, python) gave:

| arm | condition | mean | sd | Δ vs intact |
|---|---|---:|---:|---:|
| ff+fixed | intact | 0.3100 | 0.0100 | — |
| ff+fixed | bin-shuffled | 0.3050 | 0.0083 | −0.0050 |
| ff+fixed | reversed | 0.3150 | 0.0083 | +0.0050 |
| rec+alif | intact | 0.2917 | 0.0083 | — |
| rec+alif | bin-shuffled | 0.3317 | 0.0217 | +0.0400 |
| rec+alif | reversed | 0.3300 | 0.0100 | +0.0383 |

Destroying temporal order cost nothing in either arm.

**This pilot cannot support the claim, for one dominating reason.** It reaches
~0.31 accuracy against the instrument's known 0.7151 ceiling. The models are
severely undertrained, and *an undertrained model has not yet learned timing, so
shuffling cannot take timing away from it.* The null result is therefore
uninformative about the trained regime. The rec+alif direction (+0.04) is
probably noise at n = 2, and its `W_rec` scale was untuned.

The pilot's only legitimate role is to establish that the manipulation runs and
the harness is wired. It is reported here so the confirmatory design cannot be
presented later as if it were exploratory-free.

## 3. Design

At the configuration where ff+fixed reaches its measured ceiling —
`published-2ms / adjacent-sum-5 / h512 / e100`, full 8156/2264 split.

```
{ff+fixed, rec+alif} × {intact, bin-shuffled, channel-shuffled, reversed} × 3 seeds
```

**24 cells, rust backend.** Python parity follows only after a rust verdict; per
the 2026-08-02 amendment the python arm is mid-rerun and is not a blocker here.

### Conditions

| Condition | Manipulation | What it destroys | What it preserves |
|---|---|---|---|
| `intact` | none | — | — |
| `bin-shuffled` | permute time bins per sample | temporal order | per-channel counts, within-bin synchrony |
| `channel-shuffled` | independently permute each channel's bin occupancies | order **and** cross-channel synchrony | per-channel counts |
| `reversed` | reverse bin order | direction | order magnitude, synchrony, counts |

The `bin-shuffled` / `channel-shuffled` contrast decomposes the effect into
**order** and **synchrony**, which is the part of this design that is actually
novel — most shuffle controls conflate them.

Manipulations are applied **after framing**, independently per sample, with the
same seed lineage for train and test, and are regenerated per seed.

## 4. Hypotheses and thresholds

| ID | Statement | Threshold |
|---|---|---|
| **H1** | ff+fixed is a rate coder | `|intact − bin-shuffled| ≤ 0.02` for ff+fixed, with overlapping 95% CIs |
| **H2** | recurrence makes timing usable | `rec+alif` degradation under bin-shuffled exceeds ff+fixed's by **≥ 0.05** absolute, CIs disjoint |
| **H3** | synchrony carries information beyond order | `channel-shuffled` is worse than `bin-shuffled` by ≥ 0.02 in either arm |
| **H0** | shuffling degrades both arms comparably | timing is used, and §1's resolution invariance needs another explanation |

H1 and H2 are confirmatory. H3 is confirmatory only if H1 fails; if the solution
is order-invariant there is no synchrony effect to decompose.

**H1 is an equivalence test, not a null result.** It passes on a bounded
difference, not on failure to reject. That is the point: "no effect" claims from
underpowered nulls are exactly what §2 disqualifies the pilot for.

## 5. Validity gates

0. **Pipeline sensitivity (blocking, pre-run).** Added 2026-08-03 —
   `MEASUREMENT_2026-08-03_TEMPORAL_SENSITIVITY_POSITIVE_CONTROL.md`. For this
   campaign's contract and geometry, `shd-instrument temporal-sensitivity` must
   show `mean_membrane_rel_l2 >= 0.1` and `mean_spike_hamming > 0` for every
   non-identity condition, against the untrained registered initialization.

   **Why it comes first.** H1 concludes from a null, and a null is interpretable
   only if the measurement could have detected an effect. Gate 1 proves the
   manipulation changed timing rather than rate; it does not prove the *pipeline*
   carries timing to the loss. Were framing to attenuate temporal structure
   before the network saw it, all four conditions would score alike, H1 would
   "pass" as an artefact, and that same artefact would independently explain the
   resolution invariance in §1. This gate costs ~2 s and can void the design, so
   it precedes the 35 h of §5.2.

   **Measured at the anchor:** `0.957 / 0.969 / 1.151` against a `0.1` bound, and
   between `0.96` and `1.41` across all three contracts and both geometries.
   **PASSES.** The campaign is valid to run.

   **Replicated 2026-08-03** across every registered initialization — 3 seeds x
   3 widths x 2 geometries, 18 configurations, 256 test samples each
   (`scripts/temporal_sensitivity_sweep.py`, §4c of the measurement). The
   **minimum** membrane rel L2 anywhere in the sweep is `0.9264`, and the sweep
   reports minima rather than means precisely so one weak configuration cannot
   hide behind an average. `mean_spike_hamming > 0` in all 18, and the identity
   condition returns exactly `0.000000` in all 18. **PASSES with an order of
   magnitude of margin.**

1. **Manipulation check (blocking).** For every manipulated sample, per-channel
   total spike counts must be **bit-identical** to intact. Any mismatch voids
   the run — the manipulation would then be changing rate, not just timing.
2. **Trained-regime gate (blocking).** Every `intact` cell must reach ≥ 0.65
   accuracy, i.e. within 0.07 of the measured 0.7151 ceiling. This is the gate
   the pilot fails, and it exists so an undertrained null can never again be
   mistaken for order-invariance.
3. **Degeneracy.** The prereg-2026-07-25 flags — `COLLAPSED`, `NEAR-COLLAPSED`,
   `SILENT`, `SATURATED` — applied per cell; degenerate cells excluded.
4. **Registered per-cell gates.** The five non-accuracy gates in `model.py`.
5. **Determinism.** Each condition reproducible across fresh processes
   (`gates_ef.py determinism`).
6. **`W_rec` scale frozen by the G8 pilot** before any confirmatory cell.

   **Status 2026-08-03: the pilot has been run and does not deliver what this
   gate assumes.** `init --w-rec-scale` was added (the scale had been hard-coded,
   so no pilot was possible), and `scripts/w_rec_scale_pilot.py` swept
   1.0 / 0.5 / 0.25 / 0.1 / 0.05 on both recurrent arms at h128. There is **no
   stable regime to freeze**: the response is non-monotonic over three orders of
   magnitude, 8 of 10 cells exceed a peak gradient norm of 1e4, and `rec+alif`
   is worst at the smallest scale tested. See
   `MEASUREMENT_2026-08-03_RECURRENT_ARM_STABILITY.md` §3.2.

   This gate therefore cannot be discharged as written. Any H2 cell — every
   recurrent cell in this campaign — is currently at risk of being a measurement
   of gradient explosion rather than of architecture. Resolving it needs a
   decision on gradient clipping, not another scale sweep.

## 5b. RUN RECORD — deviation from the registered design, logged before results

**Written 2026-08-03 while the cells were still executing, and before any
accuracy was read.** The deviation is recorded here rather than in the write-up
so it cannot be mistaken for a post-hoc choice.

**Registered:** `{ff+fixed, rec+alif} × 4 conditions × 3 seeds` = 24 cells.
**Being run:** the **`ff+fixed` half only — 12 cells.**

**Why the `rec+alif` half is not being run.** It is registered at h512, and that
is the exact configuration measured to fail on 2026-08-03: across three seeds,
**zero produce a usable cell** — two abort mid-training on non-finite gradient
entries and the third reaches a gradient norm of 7.36e29. See
`MEASUREMENT_2026-08-03_RECURRENT_ARM_STABILITY.md` §3.6.2. Running it would
produce either nothing or numbers computed under a diverging gradient, and
either way could not test H2.

**Consequences for the registered hypotheses:**

| ID | needs | status |
|---|---|---|
| **H1** — ff+fixed is a rate coder | ff+fixed only | **testable as registered** |
| **H3** — synchrony beyond order | "either arm" | **testable** on ff+fixed |
| **H2** — recurrence makes timing usable | rec+alif | **NOT TESTABLE.** Blocked on the h512 instability, not on evidence |
| **H0** | both arms | partially testable |

H2 is **not** being reported as failed, refuted, or unsupported. It is
unmeasured, and the reason is an instrument defect at that width rather than
anything about recurrence. Any write-up must say so in those terms.

**Run parameters:** binary `8c169a659c3c` (Gate F 13/13 PASS, 11 runs / 9
binaries all PASS), `published-2ms / adjacent-sum-5 / h512 / e100`, seeds
5170001-3, `--temporal-seed` equal to the cell seed. Manipulations are applied
to **both train and test** with separate seed lineages
(`0x1111…`/`0x2222…`), which is what distinguishes this from the test-time probe
in `MEASUREMENT_2026-08-03_TEMPORAL_SENSITIVITY_POSITIVE_CONTROL.md` §4b and its
distribution-shift confound.

**Verdict procedure fixed in advance:** `scripts/temporal_campaign_verdict.py`,
written before the first cell completed, with the §4 thresholds as hardcoded
constants. Changing a threshold is an amendment in a new file, not an edit to
that script.

## 6. Decision rules

**H1 PASS.** The instrument's SHD solution is order-invariant: it is a rate code
over binned channels. This explains the resolution invariance in §1, reframes the
0.7151 ceiling as a *rate-code ceiling* rather than an architecture ceiling, and
is the publishable result. Obligation: report as a property of *this* instrument
and configuration, never as a property of SNNs or of SHD.

**H1 PASS + H2 PASS.** The stronger and more interesting outcome: architecture
determines whether timing is usable at all. Obligation: this becomes the primary
claim, and the rec+alif ceiling campaign
(`PREREG_2026-08-02_SHD_BPTT_CEILING_RECALIF`) should be re-scoped to measure how
much of rec+alif's gain is timing-derived.

**H1 FAIL (H0).** Timing is used, and §1's resolution invariance is unexplained.
Do not write up until that tension is resolved.

## 7. Known confounds, disclosed in advance

1. **Anchor inherited** from the ff+fixed winner, applied symmetrically.
2. **`W_rec` scale untuned**, as in the ceiling prereg. Biases H2 unknown.
3. **Bin-shuffling preserves within-bin synchrony**, so H1 alone cannot separate
   "no timing" from "only sub-bin timing." That is what `channel-shuffled` is
   for; the decomposition is descriptive unless H1 fails.
4. **Rate-matched surrogates are not tested.** A stronger control — replacing
   each sample with a synthetic train of identical per-channel counts — would
   test rate-sufficiency directly. Deferred; noted so its absence is not read as
   an oversight.
5. **The invariance, if found, cannot mean timing never enters the network.**
   Gate 5.0 shows the hidden representation is strongly timing-sensitive at every
   resolution, and `MEASUREMENT_2026-08-03…` §4b shows training *amplifies* that
   sensitivity ~3.6× rather than damping it. An order-invariant accuracy result
   would therefore have to mean the task does not require order — not that the
   instrument cannot see it.

   That measurement also generates a directional expectation, **post-hoc and
   unregistered**, recorded here so the write-up cannot present it as predicted:
   on an intact-trained model, time-*reversal* is nearly harmless
   (`ΔLoss = 0.181`, 7.8 % of predictions change) while *shuffling* is severe
   (`2.451` bin, `10.764` channel). If that survives the train-on-condition
   design, expect `reversed ≈ intact` with any H1 failure concentrated in the
   shuffled conditions, and most of the effect in cross-channel synchrony rather
   than within-channel order. Note this is confounded with distribution shift in
   the source measurement and is precisely why this campaign is still needed.

## 8. Analysis plan

- Per-cell accuracy; per-condition mean, SE, 95% CI across 3 seeds.
- H1 as a two-one-sided-test equivalence check at bound 0.02.
- H2 on the arm × condition interaction only.
- No post-hoc condition or arm selection; all four conditions named here.
- Report every cell including degenerate ones, flagged.

## 9. Compute budget

Rust, from the measured anchor (ff+fixed 1405 s; rec+alif ≈ 6.5× ⇒ ≈ 9150 s):

| | per cell | × 4 conditions × 3 seeds |
|---|---:|---:|
| ff+fixed | 1,405 s | 4.7 h |
| rec+alif | ~9,150 s | 30.5 h |
| **total** | | **≈ 35 h** |

Manipulations are applied post-framing and cost nothing measurable. A timing
pilot on one rec+alif cell is mandatory before the confirmatory run, per the
ceiling prereg §9.

## 10. Stopping rule

One confirmatory run. If `INVALID_HARNESS`, fix the flagged gate and re-run once.
Cells run in H1-critical order — ff+fixed intact, ff+fixed bin-shuffled, then
rec+alif intact, rec+alif bin-shuffled, then the remaining conditions — so a
truncated run still answers H1. Partial reports hold all verdicts at
`UNDERPOWERED`.

---

**References.**
`results/SHD_BPTT_CEILING_NEGATIVE_RESULT.md` — the resolution invariance in §1.
`results/PREREG_2026-08-02_SHD_BPTT_CEILING_RECALIF.md` — the rec+alif arm.
`results/AMENDMENT_2026-08-02_INSTRUMENT_KERNEL_AND_FRAMING.md` — python arm state.
Cramer et al., arXiv:1910.07407 — SHD.
"Beyond Rate Coding: Surrogate Gradients Enable Spike Timing Learning in SNNs",
arXiv:2507.16043 — the claim this design would qualify.
