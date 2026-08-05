# Measurement — SHD epoch-budget probe, and errata against the ceiling document

**Recorded:** 2026-08-03.
**Discharges:** `SHD_BPTT_CEILING_NEGATIVE_RESULT.md` §7.5 obligation ("run it on
one anchor cell and record the verdict here before this document leaves the
repository").
**Backend:** rust only. No python cell was run, read, or cited for any number in
this file.

```
claim_axis: Integrity
object_under_test: Whether the 100-epoch budget, not the architecture, produced
  the 0.7151 figure; and whether the quantitative claims in the ceiling document
  reproduce from the cells on disk.
may_claim: The registered budget rule returns UNDERTRAINED at the anchor, and the
  anchor re-measured at e400 across three seeds gives 0.7369 +/- 0.0021. Three
  numerical claims in the ceiling document do not reproduce and are corrected
  here. The harness authorization gate has regressed since that document was
  written.
must_not_claim: That ff+fixed can or cannot reach 0.80; that 0.7369 is a ceiling
  (e400 is not converged either); that the corrected width slope changes any
  verdict; that the reference runs themselves regressed.
```

---

## 1. The budget probe returned, and the registered verdict is UNDERTRAINED

`scripts/probe.py budget` completed all three cells at the anchor
(`published-2ms / adjacent-sum-5 / h512 / s5170001`, full 8156/2264 split, rust):

| epochs | test accuracy | final-epoch train loss | wall |
|---:|---:|---:|---:|
| 100 | 0.7164 | 0.2146 | 35.5 m |
| 200 | 0.7284 | 0.0979 | 72.7 m |
| 400 | **0.7345** | 0.0456 | 138.5 m |

Registered decision rule (`probe.py:430`, written before the run): a
100→400 test-accuracy gain above 0.01 returns **UNDERTRAINED**. The measured gain
is **+0.0181**, and the rule fires:

> UNDERTRAINED. More budget buys real generalisation; the 0.7151 figure is a
> budget artefact and the ceiling claim must be withdrawn or re-measured at the
> longer budget.

**The word "ceiling" is therefore withdrawn, not softened.** The blocking caveat
that has sat at the top of `SHD_BPTT_CEILING_NEGATIVE_RESULT.md` since 2026-08-02
resolves against the ceiling reading.

### 1.1 Both failure modes are present; the rule's precedence picks one

The §3.1 dichotomy — "test accuracy climbs → undertrained" versus "test flat,
train loss falling → overfitting" — assumed the two were exclusive. They are not.
Between e100 and e400, test accuracy rose by 0.0181 **and** training loss fell
4.7× (0.2146 → 0.0456). The model is still generalising *and* overfitting hard.
The registered rule checks the gain first, so it returns UNDERTRAINED; that is
the verdict of record.

`tail_loss_improvement` also grows more negative with budget (−4.28% → −5.95% →
−7.30% over each run's final ten epochs). Under a one-cycle schedule defined over
`total_steps`, a longer run is annealing more steeply at its own tail, so this is
a property of the schedule, not evidence of continued generalisation. It should
not be cited as either.

### 1.2 What the longer budget does not do

All figures in this subsection are **seed 5170001 only**, since that is the seed
the 100/200/400 sequence was run on; the three-seed result is §1.2.1 below.

Per-doubling test gains are **+0.0119** (100→200) and **+0.0062** (200→400) — a
ratio of 0.52, i.e. halving with each doubling. The seed-5170001 gate shortfall
at e400 is **0.0655** (the three-seed mean shortfall is **0.0631**).

A geometric extrapolation of that sequence puts the asymptote near **0.741**.
**This extrapolation is post-hoc, unregistered, and may not be used to restore
the ceiling claim.** It is recorded only to size the follow-up: closing 0.0655 at
a halving rate is not something two or three more doublings reach, so an
epoch-budget sweep alone is unlikely to clear 0.80.

The registered rule offers two exits — withdraw, or re-measure at the longer
budget. **Both were taken. The re-measurement completed 2026-08-03.**

### 1.2.1 The e400 re-measurement, three seeds, rust

| seed | e100 (matrix) | e400 (this run) |
|---|---:|---:|
| 5170001 | 0.716431 | 0.734541 |
| 5170002 | 0.710689 | 0.737633 |
| 5170003 | 0.718198 | 0.738516 |
| **mean** | **0.7151** | **0.7369** |
| sample SD | 0.0039 | **0.0021** |
| 95% CI | 0.7107 – 0.7195 | **0.7345 – 0.7393** |
| shortfall to 0.80 | 0.0849 | **0.0631** |

The gain is **+0.0218**, which is 5.6× the e100 seed SD. The two seed ranges do
not overlap at all — the largest e100 cell (0.7182) is below the smallest e400
cell (0.7345). The single-seed verdict was not a seed artifact, and the
three-seed gain (+0.0218) triggers the registered rule exactly as the
single-seed gain (+0.0181) did.

Two things worth recording beyond the headline:

- **Seed variance nearly halves with budget** (0.0039 → 0.0021). Part of the
  e100 spread was itself a truncation effect: cells stopped at different points
  on their own descent.
- **e400 is not converged either.** Per-doubling gains at seed 5170001 were
  +0.0119 then +0.0062, and `tail_loss_improvement` at e400 is −7.30% / −7.44% /
  −7.36% across the three seeds. So **0.7369 is a better-characterised
  budget-limited measurement, not a ceiling.** It replaces 0.7151 as the number
  to quote for this configuration; it does not license the architectural claim
  that 0.7151 was withdrawn for. Nothing here should be described as a ceiling
  without a convergence criterion being met first.

The matrix verdict is untouched: all 216 cells still fail `accuracy >= 0.80`, and
so do all three e400 cells (`CELL_FAIL`, on accuracy alone — 20 classes
predicted, majority 0.084–0.089, firing 0.204–0.210, zero saturation).

### 1.3 A free determinism result

The e100 budget cell was re-run from scratch under the current binary, in a fresh
process, ~11 months of edits after the matrix cell it duplicates. It reproduced
that cell **bit-exactly**:

| | matrix cell | budget re-run |
|---|---:|---:|
| accuracy | 0.716431095 | 0.716431095 |
| mean_loss | 0.578342072 | 0.578342072 |
| mean_firing_rate | 0.198910019 | 0.198910019 |
| wall_secs | 1403.98 | 2127.04 |

Identical to nine decimal places on a different wall-clock, which is a
cross-binary confirmation of the amendment's claim that `ff+fixed` is bit-identical
through the new `shd_matched_arms` surface. The 216 rust cells stand.

## 2. Errata — claims in the ceiling document that do not reproduce

All 216 rust cells were re-read from disk and every quantitative claim in
`SHD_BPTT_CEILING_NEGATIVE_RESULT.md` recomputed. The gate audit (0/216 accuracy,
216/216 on all five other gates), the degeneracy figures (max majority 0.1250,
firing 0.108–0.331, max silent 0.0078, saturated identically 0), the best-group
mean 0.7151, the shortfall 0.0849, the timestep means, the exposure table, the
manifest hash, and the 72-group `matrix_verdict` structure **all reproduce
exactly**. Three claims do not.

### E1 — width scaling is +0.017 per doubling, not +0.034 *(material)*

Measured at e100, n=36 per width:

| hidden | mean | Δ vs previous |
|---:|---:|---:|
| 128 | 0.6588 | — |
| 256 | 0.6751 | **+0.0163** |
| 512 | 0.6928 | **+0.0177** |

The published figure **+0.034 per doubling** is the *total* across two doublings
(0.6928 − 0.6588 = 0.0340) mislabelled as a per-doubling rate. The true rate is
**≈ +0.017 per doubling**, and it is mildly *accelerating*, not flattening.

This error is a factor of two and it propagates. It appears in
`SHD_BPTT_CEILING_NEGATIVE_RESULT.md` §4 and §7.5, and in
`PREREG_2026-08-02_SHD_TEMPORAL_INFORMATION.md` §1.

Consequence: §7.5's downstream sentence — "the width trend alone would cover
roughly a third of [0.0849] over two more doublings" — is *correct as stated*
(2 × 0.017 = 0.034, which is 40% of 0.0849) and was evidently computed from the
right rate. Only the quoted rate is wrong. No verdict changes: at +0.017 per
doubling, reaching 0.80 from h512 needs roughly five more doublings, to h16384.

### E2 — the SD convention is unstated, and the CI is narrow *(minor)*

The best-group spread is reported as `0.0032`, which is the **population** SD
(ddof=0) of the three seed accuracies. The sample SD (ddof=1) is **0.0039**. For
a spread across three seeds the sample convention is the standard one, and the
published 95% CI of 0.7115–0.7187 is correspondingly narrow; recomputed from the
sample SD it is **0.7107–0.7195**.

Nothing turns on this — both intervals sit far below 0.80 — but the convention
should be stated wherever the number appears.

### E3 — "0.002" is an endpoint difference, not the spread *(minor)*

"A 5× increase in temporal resolution moves accuracy by 0.002" is the T=100 vs
T=500 endpoint delta (0.6557 − 0.6536 = 0.0021). The largest gap between any two
resolutions is **0.0034** (T=250 at 0.6570 against T=500 at 0.6536), because the
trend is non-monotone. The resolution-invariance argument is unaffected — 0.0034
is still an order of magnitude below the T-level SD of 0.025–0.037 — but the
honest statement is "no two resolutions differ by more than 0.0034."

## 3. The harness authorization gate has regressed

`SHD_BPTT_CEILING_NEGATIVE_RESULT.md` §Provenance states `harness_status: VALID`
with `historical_reference`, `clean_reference` and `matrix_authorized` all true.
**As of 2026-08-03 that is no longer the case:**

```
harness_status : PENDING_PREREQUISITES
historical_reference : false
clean_reference      : false
matrix_authorized    : false
```

### 3.1 Cause: an over-broad fingerprint, not a reference regression

`valid_reference_payload` (`runner.py:668`) requires the stored
`source_fingerprint` to equal the current one. `SOURCE_PATHS` (`runner.py:54`)
covers the rust instrument *and* `scripts/shd_calibration/model.py` and
`data.py`. The 2026-08-02 kernel and framing vectorisation edited both, so:

```
current fingerprint : 4b85606d11fb3d523bd421afcfe312327e5b739029c28d8958ee3678364af7d9
stored in manifests : 64923d64655d86eeba334c5fac03ba32627f06336a511cdccb3e195b48e53456
```

Every other condition still passes. Re-checked directly, bypassing only the
fingerprint equality:

| check | result |
|---|---|
| all six `reference-states` `mechanical_status` | COMPLETE |
| `result_sha256` / `log_sha256` / `manifest_sha256` | match, 6/6 |
| historical mean accuracy | 0.9498, within 0.05 of published 0.951 ✓ |
| historical exposure fields | 150 reads / 0 final / `EXPOSURE_TAINTED_DESCRIPTIVE` ✓ |
| clean accuracies | 0.9390 / 0.9368 / 0.9371, all ≥ 0.80 floor ✓ |
| clean exposure fields | 0 reads / 1 final ✓ |

**The reference runs did not regress.** The gate is false because a fingerprint
covering the instrument kernel also guards a third-party reference
(`Thvnvtos/SNN-delays`) whose result cannot depend on `model.py` at all.

### 3.2 What follows, and what does not

The 2026-08-02 amendment recorded that `source_fingerprint` would change (§3) but
did not record that this would also drop `historical_reference` and
`clean_reference`, and with them `matrix_authorized` and `harness_status`. That
consequence is recorded here.

Any document quoting `harness_status: VALID` for the 216-cell matrix must now say
instead: the matrix was authorized under fingerprint `64923d64…`, and the harness
has since dropped to `PENDING_PREREQUISITES` under `4b85606d…` for the reason
above.

**No fix is applied here.** Narrowing `SOURCE_PATHS` so the reference gates
depend only on `reference.py` / `reference_clean_main.py` is defensible on the
merits, but it is a change to the meaning of a registered gate, made after seeing
that the gate is red. Per `PREREG_2026-07-25_SHD_ARCH_ABLATION` §preamble it
requires its own registered amendment, written before the change. That amendment
is **decision 2** in §5 below and is not written by this file.

## 4. Net effect on the ceiling document

| § | claim | status after this file |
|---|---|---|
| header | "credit-assignment ceiling" | **withdrawn** — budget rule returns UNDERTRAINED |
| §1 | best 0.7151 ± 0.0032 over 216 cells | stands; SD convention per E2; not a ceiling |
| §1 | shortfall 0.0849 | stands at e100; **0.0631** at e400 across three seeds |
| §3 | single-gate failure, 0/216 vs 216/216 | **stands, verified** |
| §4 | resolution invariance | stands; wording per E3 |
| §4 | width +0.034/doubling | **corrected to +0.017** (E1) |
| §5 | gap decomposition | arithmetic stands; "ceiling" relabelled to "e100 measurement" |
| §6 | exposure hygiene | **stands, verified** |
| §7.5 | budget unvalidated | **discharged** — verdict UNDERTRAINED, recorded here |
| prov. | `harness_status: VALID` | **stale** — see §3 |

What survives intact is the strongest part: a single-gate failure across 216
cells with every validity gate passing, and the exposure-bias methods result.
What does not survive is the framing that made 0.7151 an architectural limit.

## 5. Open decisions

1. ~~**Re-measure or withdraw.**~~ **DONE** — both. The ceiling claim is
   withdrawn and the anchor is re-measured at e400 across three seeds:
   **0.7369 ± 0.0021**, shortfall **0.0631** (§1.2.1). What remains open is
   whether to re-run the *rest* of the matrix at e400. Note the e400 cells were
   produced by the shipped binary; the 2.7× faster bit-identical kernel
   (`AMENDMENT_2026-08-03_RUST_KERNEL_TRANSPOSE.md`) would bring a full anchor
   column down substantially.
2. **Fingerprint scope.** Whether reference gates should depend on the instrument
   kernel. Requires a registered amendment written before the change.
3. **Temporal campaign gate 5.2.** Unaffected in the arithmetic — its floor is
   0.65 and the anchor measures 0.7164 at e100 — but its §1 motivation carries the
   E1 error and its §5.2 text calls 0.7151 a "measured ceiling". See the erratum
   block added to that prereg.
4. **Python arm.** Untouched by this file. Per the standing lean, the rust arm is
   the scientific record; `matrix_verdict` cannot return `CALIBRATED` without a
   python arm, but it already returns `FAIL` from the rust half alone.
5. **Recurrent arms are defective** — the `rec+*` gradient is not the gradient of
   the `rec+*` forward, which blocks both outstanding preregs. See
   `DEFECT_2026-08-03_RECURRENT_ARM_FORWARD_BACKWARD_MISMATCH.md`. `ff+fixed`,
   and therefore everything in this file, is unaffected.
6. **The fingerprint is wrong in both directions.** §3 above shows it is too
   *broad* — it invalidates a third-party reference that cannot depend on the
   instrument kernel. `AMENDMENT_2026-08-03_RUST_KERNEL_TRANSPOSE.md` §5 shows it
   is simultaneously too *narrow* — `shd_matched_arms.rs`, the module every rust
   number actually flows through, is outside `SOURCE_PATHS`, so the compute
   kernel can be rewritten without the fingerprint moving. One registered
   amendment should fix both.

## 6. Companion documents written the same day

| file | what it records |
|---|---|
| `AMENDMENT_2026-08-03_RUST_KERNEL_TRANSPOSE.md` | a bit-identical 2.8× rust kernel optimisation, its Gate F evidence, and the fact that the rust arm previously had no runnable gate |
| `DEFECT_2026-08-03_RECURRENT_ARM_FORWARD_BACKWARD_MISMATCH.md` | the recurrent forward/backward mismatch blocking both preregs |

---

**Provenance.** `results/shd_instrument_v4/probe/budget__published-2ms__adjacent-sum-5__h512__e{100,200,400}__s5170001.json`;
216 cells under `results/shd_instrument_v4/cells/rust__*.json`;
`manifest_sha256` `7f612774972ccd61dd6af283fb9568c6eba88920185bddc212fed269bf41c52d`
(re-verified 2026-08-03). Longer-budget order files under `probe/orders/`, with
regenerated weights asserted byte-identical to `initialization/`; no registered
initialization artifact was modified.
