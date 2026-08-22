# `deep-snn-scaling` at v134 — the depth-matched ceiling does not learn

**Run:** 2026-08-20, local M5 Pro, `cargo build --locked --release`.
**Report:** [`deep_snn_results_v134.md`](deep_snn_results_v134.md) ·
log `/tmp/deep_v134.log`.
**Closes:** `TODO_2026-08-07_OPEN_WORK.md` §1 item 3 ("Re-run `deep-snn-scaling`
at v134") and settles §1 item 4 (the depth-collapse restatement) and
`WEEK_PLAN_2026-08-19.md` A4 / A5.

---

## The finding

The re-run does not restate the depth result. It removes the basis for one.

**Every depth-matched gradient ceiling sits at chance**, and the v134 guard — the
whole reason for the re-run — fires at three of the four depths:

| depth | learned FB | depth-matched ceiling | modulator RMS | ceiling health |
|---:|---:|---:|---:|---|
| 1 | **1.0000** ± 0.0000 | 0.4880 ± 0.0120 | 5.044e-1 | **INVERTED** |
| 2 | 0.5060 ± 0.0768 | 0.5000 ± 0.0000 | 5.035e-1 | **INVERTED** |
| 3 | 0.5810 ± 0.0679 | 0.5000 ± 0.0000 | 5.035e-1 | **INVERTED** |
| 4 | 0.4435 ± 0.0664 | 0.5000 ± 0.0000 | 5.036e-1 | ok |

`CoincidenceTask` is a two-class task, so 0.5000 is chance. A ceiling of
`0.5000 ± 0.0000` across 20 seeds is not a weak ceiling; it is a **constant
predictor**.

**Verdict: `deep-snn-scaling` is `INVALID_HARNESS` at v134.** The depth axis
cannot be interpreted, in either direction, because there is nothing to interpret
it against.

## Why this is stronger evidence than the 2026-08-07 audit's version

`AUDIT_2026-08-07_JULY_CAMPAIGN_SCORING_PATH.md` §3 and
`CODE_FIRST_TRANSFER_STATUS.md:12` already treat this suite as `INVALID_HARNESS`,
and `TODO` §1 item 4 already argued the depth collapse was weak evidence because
`CoincidenceTask` has `N_IN = 2`. Both were arguments from the source. This is the
measurement, and it is worse than the argument predicted:

- The argument said *the task has no depth structure to exploit*.
- The measurement says *the reference does not learn the task at any depth,
  including depth 1, on splits the treatment solves perfectly in the same
  process*.

That distinction matters, because the `N_IN = 2` argument would still have
permitted a depth-1 comparison. The measurement does not.

## The control is internal, so there is no cross-run confound

The 1-hidden-layer learned-feedback arm reaches **1.0000 on every one of the 20
seeds** using **the same frozen splits, in the same process, on the same
timestep budget** as the ceiling that scores 0.4880.

So the data are perfectly learnable and the ceiling does not learn them. Nothing
about task difficulty, seed lineage, split construction, framing, or platform can
explain a gap that is measured inside one process on one set of examples.

The `--quick` schedule reproduces it at a different width, budget and training-set
size (h128 / e40 / 60 train): ceilings 0.5333 / 0.5000 / 0.5000 / 0.5000. So the
behaviour is not specific to the full schedule's h256.

## What it is *not*

**Not modulator collapse.** That was the hypothesised mechanism —
`deep_snn_scaling.rs:18-20` names it, and `matched_deep_gradient.rs` carries a
test for it (`raw_transport_collapses_deep_modulator_scale`). The realised
input-layer modulator RMS is **5.03e-1 to 5.04e-1 at every depth**, within 0.2% of
itself across the whole ladder. The credit signal reaching the input layer is
present and correctly scaled. **The instrumentation added in 2026-07-25 did its
job and ruled out the explanation it was built to test.**

## How this survived: the tests could not fail

`matched_deep_gradient.rs::trains_at_every_depth_without_panicking` is the only
test that exercises training, and it asserts:

```rust
assert!(r.accuracy.is_finite());
assert!((0.0..=1.0).contains(&r.accuracy));
```

A constant predictor at chance satisfies both. There is **no test anywhere that
asserts `MatchedDeepGradient` learns anything at all** — which is exactly the
failure mode `BLOG_2026-08-03_THE_CHECK_THAT_CANNOT_FAIL.md` is about, sitting in
this repository, unnoticed, in the module the blog post's own suite depends on.

The named next step is therefore not "re-run again" but: a test that a gradient
reference clears chance on a task a local rule solves, applied to **every**
reference arm in the workspace, not just this one.

## Consequences for the record

1. **Withdraw the depth-collapse result.** `1.0000 → 0.4525` (v132) and
   `1.0000 → 0.4435` (v134) may not be cited as local learning failing with
   depth, or as anything else. This discharges `TODO` §1 item 4 and A5 by
   **withdrawal**, not restatement.
2. **`deep-snn-scaling` is `INVALID_HARNESS` at v134**, on measured evidence.
   `CODE_FIRST_TRANSFER_STATUS.md:12` already says so; that row is now backed by
   a run rather than by a reading of the source.
3. **The 1L learned-feedback 1.0000 is not a PASS either.** It is a treatment
   with no reference. It stays out of the paper.
4. **`MatchedDeepGradient` is used only here.** Confirmed by grep across the
   workspace — no other experiment or claim depends on it, so the blast radius of
   this defect is this suite alone. *(Verified: the only non-test references are
   `deep_snn_scaling.rs` and the crate re-export.)*

## The replacement ceiling already exists, and is already validated

`binn-learn/src/shared_bptt.rs:3` states its own purpose: *"This replaces the
invalid historical `MatchedDeepGradient` ceiling without changing or deleting
that legacy type."* It is a runtime-sized shared-forward stack with exact reverse
mode, and unlike the legacy type it carries tests that can fail —
`depth_one_bptt_overfits_easy_fixture`, `finite_differences_agree_at_depths_one_and_two`,
`cloned_treatment_and_ceiling_have_identical_pretraining_forward`.

It is **wired**: `temporal_deep_campaign.rs` and `transfer_falsifier.rs` both use
it. So the depth question has a valid instrument.

**That instrument cannot run.** `temporal_deep_campaign.rs:32` calls
`authorize_campaign(CampaignKind::LocalLearning)`, which
`instrument_status.rs:34-46` refuses while `SHD_INSTRUMENT_STATE` is
`Uncalibrated`. There is no report for it on disk and there cannot be one until
the instrument calibrates.

So the depth result is not merely withdrawn — it is **blocked behind the same
calibration gate as the frozen-attention local arm**
(`BLOCKER_2026-08-19_FROZEN_ATTENTION_LOCAL_ARM.md`). Two independent lines of
work now wait on one gate. That gate is not a compute problem.

**Do not re-point `deep-snn-scaling` at `shared_bptt` as a workaround.** It would
route the depth question around the authorization gate rather than through it,
which is the same move as flipping the constant, done indirectly.

## Confirmed at v135, with the guard repaired

Re-run 2026-08-21 after `guards::CeilingHealth` replaced the local inversion test
(`HARDENING_2026-08-21_CEILING_HEALTH_HAS_ONE_OWNER.md`). **Every accuracy is
unchanged**; only the interpretation moved:

| row | v134 said | v135 says |
|---|---|---|
| depth-4 ceiling (0.5000) | `ok` | **DEAD REFERENCE — at chance** |
| depths 1–3 ceilings | INVERTED | **DEAD REFERENCE + INVERTED** |
| 1-layer arm (1.0000) | **PASS** | **INVALID_HARNESS** |
| overall verdict | FAIL | **INVALID_HARNESS** |

The report now leads with a harness-defect banner before any number. That the
accuracies did not move is the point: this was never a measurement error, it was
a reading error that the instrument was reporting for us.

## Scope of this document

- **Verified:** every number above, from one full run at v134, n=20, on the local
  M5 Pro, plus the `--quick` schedule at h128.
- **Verified:** the internal control (same process, same splits, treatment 1.0000
  vs ceiling 0.4880).
- **Not verified:** *why* `MatchedDeepGradient` fails to learn. Modulator scale is
  ruled out; the forward, the surrogate, and the output-layer update are not yet
  isolated. No fix is proposed here and none is implied.
- **Not claimed:** anything about deep credit assignment, in either direction.

## Reproducing

```bash
cargo build --locked --release -p binn-lab --bin deep-snn-scaling
./target/release/deep-snn-scaling --out results/deep_snn_results_v134.md
```

Runs in **7 minutes** on 18 cores after the seed loop was parallelised
(`deep_snn_scaling.rs`, 2026-08-20); it was ~51 minutes single-threaded. The
parallel report is **byte-identical** to the serial one — checked by SHA-256
against the pre-parallel binary at `RAYON_NUM_THREADS` of 1, 4, and unset. See
[`MEASUREMENT_2026-08-20_EXPERIMENT_PARALLELISM.md`](MEASUREMENT_2026-08-20_EXPERIMENT_PARALLELISM.md).
