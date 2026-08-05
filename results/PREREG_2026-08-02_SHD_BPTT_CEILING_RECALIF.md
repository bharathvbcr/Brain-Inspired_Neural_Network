# Preregistration — rec+alif BPTT ceiling on SHD

**Registered:** 2026-08-02, before any run of `shd-bptt-ceiling`.
**Protocol version:** 144 (proposed)
**Instrument:** `scripts/shd_calibration/` matched BPTT matrix, extended per §2.
**Claim axis:** architecture ceiling under exact credit assignment.

```
claim_axis: Integrity
object_under_test: Whether adding recurrence (W_rec) and an adaptive threshold
  to the matched SHD instrument raises the BPTT accuracy ceiling past the
  registered 0.80 gate that ff+fixed failed in all 216 rust cells.
may_claim: Named arms cleared or failed the registered 0.80 gate under matched
  BPTT, with disclosed forward, backend parity, and degeneracy flags.
must_not_claim: SOTA; Gate G2; biology or cortex; neuromorphic hardware;
  like-for-like comparison to e-prop, ETLP, or DCLS-delays; anything about
  local credit assignment (no local rule is run here).
```

This document fixes hypotheses, thresholds and stopping rules **in advance**.
Nothing below may be edited after the first confirmatory run; amendments go in a
new file with a new timestamp.

---

> **2026-08-03 — a defect that would have invalidated this entire prereg was
> found and FIXED before any cell ran.** The `rec+*` forward read
> partially-updated spikes inside a timestep while the backward differentiated
> the clean-previous-step model, so the recurrent gradient was not the gradient
> of the recurrent forward. Since this prereg's entire object is the rec+alif
> arm, every one of its cells would have measured a broken gradient. It is fixed
> and pinned by a test; see
> `DEFECT_2026-08-03_RECURRENT_ARM_FORWARD_BACKWARD_MISMATCH.md`, and note §4b —
> **no gradient-tolerance check could have detected it**, so the Gate E
> precondition in §0 would not have protected this run.
>
> **2026-08-03 (later) — the arm has now been run. It learns, but it produces
> gradient excursions the instrument could not previously detect.** With the
> defect fixed and the kernel 6.3x faster, both recurrent arms were exercised on
> real SHD data for the first time. At h128/e20, `rec+fixed` reaches 0.2633 and
> `rec+alif` 0.3785, both with monotonically falling loss — so the arm is *not*
> broken. But `rec+fixed` hits a genuinely infinite gradient norm in epoch 3
> before settling, and `rec+alif` keeps excursing to 1e11 as late as epoch 8.
> Full detail in `MEASUREMENT_2026-08-03_RECURRENT_ARM_STABILITY.md`; note that
> its §3.5 corrects an earlier and more alarming reading taken from 3-epoch
> cells, which sit entirely inside the early-training transient.
>
> Three consequences for this prereg:
>
> - **§5's `W_rec`-freeze gate cannot be discharged as written.** The scale
>   pilot ran across three seeds and the scale axis does not survive replication:
>   at scale 1.0, `rec+fixed` gives 9.8e12, an infinite norm, and an outright
>   abort on the three seeds respectively. There is no scale to freeze.
> - **2 of 30 pilot cells aborted mid-training** with a non-finite value and
>   produced no cell file at all. For a 24-cell campaign that is an operational
>   failure mode to plan for, not merely a bad number.
> - **Measured at this prereg's own width, and it fails.** `rec+alif` at
>   **h512**/e20 reaches a peak gradient norm of **7.36e29** — some 30 orders of
>   magnitude above the healthy `ff+alif` value of ~0.15. h128 and h256 are
>   entirely clean on the same configuration, so the failure appears between
>   h256 and h512, consistent with the `O(hidden^2)` recurrent fan-in this
>   prereg's width maximises.
>
>   **The cell looks healthy.** Loss falls monotonically 2.926 → 2.241, the
>   lowest of any cell measured, and accuracy is 0.3507. Nothing a reader would
>   check flags it, and before 2026-08-03 `non_finite_events` would have read 0
>   in every such cell because that counter was never incremented at all.
>
>   **Replicated across three seeds, and this campaign cannot run as
>   registered.** At h512/e20, `rec+alif`: seed 5170001 completes with a peak
>   gradient norm of **7.36e29**; seed 5170002 **aborts** at optimizer step 220;
>   seed 5170003 **aborts** at step 50. **Zero of three seeds produce a usable
>   cell.** (An earlier version of this note said seed 5170001 had 420/640
>   non-finite steps. That was an f32 overflow in the norm computation, since
>   fixed — the gradients were finite. See §3.6.0 of the measurement.)
>   The aborts are the more severe failure — an individual gradient *entry* went
>   non-finite, not merely the f32 norm — so they stop rather than producing a
>   misleading cell.
>
>   Applied to this prereg's 24 cells: roughly two thirds would abort and yield
>   nothing, and the remainder would report accuracies produced under gradient
>   norms around 1e29. h128 and h256 are entirely clean on the same
>   configuration, so this is a property of the registered width.
>
>   This is a blocking result **about the instrument at h512**, not a finding
>   about recurrence, and must not be written up as one. What it blocks is
>   running the campaign. **Gradient clipping was tried on 2026-08-03 and
>   cannot work**: the abort fires on a non-finite *per-sample* gradient,
>   strictly before the batch gradient that clipping acts on is formed, so
>   no threshold could change it — and it made the one previously-completing
>   seed abort. See `MEASUREMENT_2026-08-03_GRADIENT_CLIPPING_DOES_NOT_FIX_H512.md`.
>   Remaining candidates: truncated BPTT, lower surrogate gain, a
>   spectral-radius-normalised init, or amending this prereg to a narrower
>   width. Each is a model change needing registration.
>
> Still argued rather than measured: that the rust and python recurrent forwards
> now agree. They should — both implement `sum_j w_rec[h,j] * s_j(t-1)` — but
> there is no cross-backend recurrent fixture yet (G7).
>
> Separately, this document's premise — that ff+fixed has a measured ceiling of
> 0.7151 to be raised past — is withdrawn. The registered budget rule returned
> UNDERTRAINED; see `MEASUREMENT_2026-08-03_SHD_BUDGET_AND_ERRATA.md`. The
> ff+fixed comparison point must be re-measured at a converged budget, or both
> arms run at the same longer budget, before a rec+alif number means anything.

## 0. Authorization precondition — this run is currently BLOCKED

`SHD_INSTRUMENT_STATUS.md` lists "new SHD local-learning or architecture-ablation
campaigns" under **Blocked work** while the instrument is `UNCALIBRATED`, and
states that the Rust entry points enforce that state in code.

**This campaign may not start until calibration closes**: all 432 matrix cells
complete and `matrix_verdict` returns a verdict. Registering the design now is
explicitly permitted — executing it now is not. If the calibration verdict is
`FAIL` (which `SHD_BPTT_CEILING_NEGATIVE_RESULT.md` shows is already determined
by the completed rust arm), a separate authorization amendment is required
before this campaign runs, and that amendment must state on what basis a
`FAIL`-verdict instrument is fit to measure a new architecture.

## 1. Motivation

The completed rust arm (216/216) establishes a BPTT ceiling of **0.7151 ± 0.0032**
for the feed-forward, fixed-threshold forward — 0.0849 short of the registered
`accuracy >= 0.80` gate, with every non-accuracy gate passing in all 216 cells.
Because BPTT is the strongest practical credit assignment for this forward, no
local rule on the same architecture can clear the gate either.

Two facts in the completed data point at the architecture rather than the rule:

- **Temporal resolution buys nothing.** T = 100 / 250 / 500 gives 0.6557 / 0.6570
  / 0.6536 — a 5× resolution change moves accuracy by 0.002, two orders of
  magnitude below seed spread.
- **Width buys a little, monotonically.** At `e100`, h128 / h256 / h512 gives
  0.6588 / 0.6751 / 0.6928, still rising at 512.

That is the signature of a rate readout on a spatio-temporal task. ETLP's stated
conclusion is that threshold adaptation *and* a recurrent topology are necessary
to learn such structure. Both are absent from the current forward. This campaign
tests whether adding them raises the ceiling.

**This is the ceiling question, not the locality question.** `binn-learn`'s
existing `ShdAlifArch` already crosses `recurrent × adaptive`, but `ShdAlifRule`
offers only `Dfa`, `EpropCeiling`, `BroadcastPm1` — no BPTT. `shd-arch-ablation`
therefore answers a different question and is not superseded by this document.

## 2. Instrument extension (must land and pass parity before any cell runs)

The matched instrument gains two disclosed terms in **both** backends:

- **Recurrence.** `W_rec`, `hidden × hidden`, **zero diagonal enforced** (a
  self-loop is a threshold change in disguise and would confound the adaptation
  axis). Backward pass carries the recurrent term across time.
- **Adaptive threshold.** `θ_i(t) = THETA_REST + β_a · a_i(t)`,
  `a_i(t+1) = ρ · a_i(t) + s_i(t)`, with `τ_a = 20`, `β_a = 0.18`, matching
  `binn-learn/src/shd_alif.rs` defaults. Backward pass carries the adaptation
  trace; the exact eligibility term is **not** approximated away — this is BPTT,
  so `∂a/∂s` is differentiated, not truncated.

Everything else is frozen at the calibration lineage: `SURROGATE_ALPHA = 5.0`,
Adam(0.9, 0.999, 1e-8), one-cycle LR, `weight_decay = 1e-5`, batch 256,
`PHYSICAL_TAU_MS = 10.05`, canonical 8156/2264 split.

**Gate E (instrument parity).** A regenerated parity fixture covering all four
arms must clear the existing registered tolerances — forward `<= 1e-6`, gradient
`<= 1e-4`, update `<= 1e-5` — before any matrix cell runs. Failing Gate E blocks
the campaign; it is not a result.

**Gate F (regression).** With `recurrent = false, adaptive = false`, the extended
instrument must reproduce the existing ff+fixed cells **bit-identically**. A
non-bit-identical regression invalidates comparison against the 216 completed
cells and blocks the campaign.

## 3. Design

Full 2×2 crossing, named in advance, at one anchor configuration:

```
{ff, rec} × {fixed θ, alif} × 3 seeds × {python, rust}
anchor: published-2ms · adjacent-sum-5 · hidden 512 · epochs 100
```

**24 cells** (4 arms × 3 seeds × 2 backends).

**Anchor selection is a declared forking path.** `published-2ms / adjacent-sum-5
/ h512 / e100` is the *best-performing ff+fixed configuration in the completed
matrix* (0.7151). Choosing it is post-hoc with respect to that matrix. It is
declared here in advance, applies **symmetrically to all four arms**, and is
therefore not a per-arm selection. It biases **in favour** of the ff baseline,
which makes H1 harder to pass, not easier. No other configuration may be
substituted after the run begins.

Seeds: `5170001, 5170002, 5170003` — the calibration lineage, unchanged.

## 4. Hypotheses and thresholds

| ID | Statement | Threshold |
|---|---|---|
| **H1** | Architecture lifts the ceiling past the registered gate | `rec+alif` mean accuracy **≥ 0.80** across 3 seeds, in **both** backends |
| **H2** | Architecture materially closes the gap even if the gate is missed | `rec+alif` − `ff+fixed` **≥ 0.10** absolute, with disjoint 95% CIs across seeds |
| **H3** (attribution, descriptive) | Which axis carries the effect | compare `rec+fixed` and `ff+alif` against `ff+fixed`; **not** a confirmatory test |
| **H0** | Neither H1 nor H2 holds | architecture is not the binding constraint on this instrument |

H1 and H2 are the only confirmatory tests. H3 is descriptive and may not be
reported as a hypothesis result.

## 5. Validity gates (dominate H1/H2/H3)

A run is `INVALID_HARNESS` — no claim permitted — if any fails:

1. **Gate E and Gate F** (§2) both pass.
2. **Backend parity.** Every arm's python and rust seed-means agree within
   **0.05** absolute, matching the existing `matrix_verdict` criterion. A
   backend disagreement voids that arm.
3. **Negative control.** Shuffled-label `rec+alif` must score ≤ chance + 0.05
   (i.e. ≤ 0.10 on 20 classes). Above that, the pipeline leaks and every number
   is void.
4. **Registered per-cell gates.** The five non-accuracy gates already in
   `model.py` — `classes_predicted == n_classes`, `majority_prediction < 0.30`,
   `silent_fraction <= 0.95`, `saturated_fraction <= 0.05`, `non_finite == 0` —
   must pass in every cell of both H1 arms.
5. **Real data.** Any confirmatory run that loads a capped or fixture split
   aborts. Capped runs are pilots and are labelled as such.

### Per-cell degeneracy

Recurrent networks collapse, go silent, or saturate; all three produce
chance-level accuracy that is indistinguishable from "recurrence does not help."
Flags, applied per cell:

| Flag | Condition |
|---|---|
| `COLLAPSED` | predicts a single class across the test set |
| `NEAR-COLLAPSED` | > 95% of predictions in one class |
| `SILENT` | mean firing rate < 0.001 spikes/neuron/step |
| `SATURATED` | mean firing rate > 0.5 spikes/neuron/step |

Degenerate cells are excluded from arm means and their accuracies may not be
cited. **If `rec+alif` returns `SATURATED`, H1 has not been tested** — the fix is
recurrent weight scaling at initialization, not the hypothesis.

For reference, the completed ff+fixed arm had zero degenerate cells: max
`majority_prediction` 0.1250, firing rates 0.108–0.331, `saturated_fraction`
identically zero. Any degeneracy in this campaign is therefore attributable to
the added terms, not to the harness.

## 6. Decision rules

**H1 PASS.** The 0.80 gate is reachable and the ff+fixed FAIL was architectural.
Obligations: restate the SHD claim axis; mark the ff+fixed ceiling result as
superseded on architecture grounds; the locality question becomes askable, and
`shd-arch-ablation` should be re-run on the cleared architecture before any
statement about local credit assignment.

**H1 FAIL, H2 PASS.** Architecture matters but does not reach the gate.
Obligations, in order: (a) width sweep on `rec+alif` — the ff+fixed width trend
was still rising at h512, so the anchor may be capacity-bound rather than
architecture-bound; (b) learning-rate sweep, since the one-cycle schedule was
inherited and never tuned for a recurrent forward; (c) only then treat the
residual as an architecture limit.

**H1 FAIL, H2 FAIL (H0).** Architecture is not the binding constraint on this
instrument. This is the more interesting negative result and materially
strengthens `SHD_BPTT_CEILING_NEGATIVE_RESULT.md` — but it may **not** be written
up until confound 7.2 is eliminated, because an untuned learning rate on a
recurrent forward could mask a real effect.

**INVALID_HARNESS.** No claim. Fix the flagged gate and re-run once. Per the
U-NEG protocol an `INVALID_HARNESS` run may not be cited as positive evidence
anywhere downstream.

## 7. Known confounds, disclosed in advance

1. **Anchor inherited from the ff+fixed winner.** Declared in §3. Biases toward
   the baseline; makes H1 harder.
2. **No learning-rate sweep.** The one-cycle schedule (`base 1e-3, max 5e-3`) was
   tuned for a feed-forward forward. **Direction of bias unknown** — this is the
   confound most likely to produce a false H0.
3. **Single width, single contract.** h512 / published-2ms only. Nothing here
   speaks to whether a wider recurrent net clears the gate.
4. **`W_rec` initialization scale is a free parameter** not fixed by the
   calibration lineage. It is set once, before the run, to the value that keeps
   the ff+fixed firing-rate band (0.108–0.331) in a **pilot**, and frozen. The
   pilot may not be reported as confirmatory.
5. **Ceiling argument is empirical.** "No local rule beats BPTT" is a regularity,
   not a theorem.

## 8. Analysis plan

- Per-arm mean, SE and 95% CI across the 3 seeds (normal approximation on seed
  means), reported separately per backend and pooled only after gate 5.2 passes.
- H1 tested on `rec+alif` against the fixed 0.80 constant. H2 tested on the
  `rec+alif` vs `ff+fixed` contrast only. No other pairwise comparison is a
  preregistered test.
- No post-hoc cell or arm selection: `rec+alif` is named in advance as the H1/H2
  arm.
- The shuffled-label control is a validity gate, not a hypothesis test.
- Report carries the four claim-axis fields and the degeneracy flag table for
  every cell, including passing ones.

## 9. Compute budget

Derived from the completed anchor cell (rust, ff+fixed, 1405 s mean across 3
seeds). Recurrence adds roughly `3 · H²` per timestep against the existing
`2 · n_in · H`; at `n_in = 140, H = 512` that is a **≈6.5× multiplier**. The
adaptive threshold adds `O(H)` per step and is negligible.

| Arm | Rust est. | Python est. (post-patch) |
|---|---:|---:|
| `ff+fixed` | 1,405 s | ~4,200 s |
| `ff+alif` | ~1,450 s | ~4,350 s |
| `rec+fixed` | ~9,100 s | ~27,000 s |
| `rec+alif` | ~9,150 s | ~27,500 s |
| **× 3 seeds** | **~63,000 s ≈ 17.5 h** | **~189,000 s ≈ 53 h** |

**Total ≈ 70 hours ≈ 3 days** if the backends run concurrently as before, plus
the shuffled-label control (3 cells, ~7.6 h rust) and the Gate E/F fixture work.

**These are estimates, not measurements.** The 6.5× recurrent multiplier is a
FLOP-count derivation, and the python figures assume the 2026-08-02 kernel patch
holds its measured 2.7× at `published-2ms` density. **A timing pilot is mandatory
before the confirmatory run** — one `rec+alif` cell at `--limit 1`, reported as a
pilot, to replace both estimates with measurements. If the measured total exceeds
**5 days**, the design drops to `hidden = 256` (multiplier ≈3.7×) and that
substitution must be recorded as an amendment before the confirmatory run starts.

## 10. Stopping rule

One confirmatory run at the stated schedule. If `INVALID_HARNESS`, fix the
flagged gate and re-run once. No third run under this registration. Cells run in
H1-critical order — `ff+fixed`, `rec+alif`, then `rec+fixed`, `ff+alif` — and the
report is rewritten after every cell, so a truncated run still answers the
preregistered contrast. Partial reports hold all verdicts at `UNDERPOWERED`.

---

**References.**
`results/SHD_BPTT_CEILING_NEGATIVE_RESULT.md` — the ff+fixed ceiling this tests against.
`results/PREREG_2026-07-25_SHD_ARCH_ABLATION.md` — the local-rule counterpart.
`results/SHD_INSTRUMENT_STATUS.md` — authorization state, §0 above.
`binn-learn/src/shd_alif.rs` — existing `recurrent × adaptive` forward and defaults.
