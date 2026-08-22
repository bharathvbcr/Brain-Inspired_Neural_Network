# Hardening — a dead reference can no longer be certified healthy

**Date:** 2026-08-21.
**Trigger:** `RESULT_2026-08-20_DEEP_SNN_V134_CEILING_IS_AT_CHANCE.md`, which found
a depth-matched gradient ceiling at chance. The re-run closed the record item; it
did not close the **defect class**, and this does.

---

## 1. The hole

Five experiments independently implemented a ceiling-health check. All five
implemented the same idea — some form of `ceiling_mean < treatment_mean` — and
that idea has a hole:

> It is silent when the reference never learned **and the treatment is below it**.

`deep-snn-scaling` v134 walked into it exactly:

| depth | ceiling | treatment | v134 printed | truth |
|---:|---:|---:|---|---|
| 1 | 0.4880 | 1.0000 | INVERTED | inverted **and** dead |
| 2 | 0.5000 | 0.5060 | INVERTED | inverted **and** dead |
| 3 | 0.5000 | 0.5810 | INVERTED | inverted **and** dead |
| **4** | **0.5000** | **0.4435** | **`ok`** | **dead reference** |

Chance is 0.5000 — the task is two-class. Depth 4's ceiling is a **constant
predictor**, and the guard written to catch dead references certified it healthy,
because 0.5000 is not less than 0.4435.

**Worse, the arm verdict did not depend on ceiling health at all.**
`Verdict::evaluate_mean(..., harness_valid: true)` was hardcoded, so the
1-hidden-layer arm reported **PASS** against a reference at 0.4880 on a two-class
task.

The same hole was present at every other site:

| site | check | blind to |
|---|---|---|
| `deep_snn_scaling` | `cm + 1e-6 < m` | reference at chance below treatment |
| `shd_arch_ablation` | `me + 1e-6 < md` | same |
| `temporal_deep_campaign` | `ceiling + 0.01 < treatment` | same |
| `track_b_rescue` | `exceeded_ceiling > 0` | a reference so weak every seed is *unidentifiable*: the gap series empties, `mean(&[])` returns `0.0`, and the arm is reported **FAIL** as though a gap had been measured |
| `live_transfer_rescue` | `gap_exceeded_ceiling == 0` | same |

This is not five careless implementations. It is **one idea with five owners**, so
a hole in the idea had to be found five times to be fixed once.

## 2. The fix — one canonical owner

`binn_lab::guards::CeilingHealth` (`guards.rs`) is now the single owner. It tests
the reference **against chance first** and against its treatment second:

```rust
pub enum CeilingHealth { Ok, DeadReference, Inverted, DeadAndInverted }
```

A reference that did not learn is unusable whatever the treatment did — and that
must be said *especially* when the treatment is also failing, which is the case
the old check could not see. `CEILING_ABOVE_CHANCE_MARGIN = 0.05` is deliberately
loose: this is a defect detector, not a quality bar. A reference inside that band
is not "weak", it is **not a reference**.

All five sites now call it. `deep-snn-scaling` additionally derives
`harness_valid` from it, so an arm whose reference did not learn reports
`INVALID_HARNESS` instead of PASS or FAIL, and the report leads with a defect
banner before any number.

### The NaN detail

`dead` is written `!(reference_mean > chance + margin)`, not
`reference_mean <= chance + margin`. Clippy asks for the rewrite; the rewrite is
wrong. `NaN <= x` is **false**, which would classify a non-finite reference as
*healthy*. `!(NaN > x)` is true, so it falls to the defect branch. Suppressed with
that reason and pinned by a test.

## 3. Regression tests that fail against the pre-fix code

In `guards.rs`:

- **`a_dead_reference_is_not_ok_just_because_the_treatment_is_worse`** — the exact
  v134 depth-4 row, plus an assertion that the superseded predicate really was
  silent on it.
- **`every_deep_snn_v134_ceiling_row_is_now_flagged`** — all four rows, none usable.
- `a_working_reference_above_its_treatment_is_ok`
- `a_live_reference_beaten_by_its_treatment_is_inverted_not_dead`
- `a_reference_at_chance_with_a_perfect_treatment_reports_both_defects` — the SHD
  sweep shape (0.2140 reference, 1.0000 treatment, chance 0.2000).
- `the_margin_boundary_is_exclusive_and_nan_is_not_healthy`.

## 4. A guard so it cannot be re-implemented

`binn-lab/tests/ceiling_health_guard.rs` fails the build if any experiment makes a
ceiling-health claim without going through `CeilingHealth`. It carries its own
falsifiability test, and refuses to pass vacuously: it asserts that **at least
four** experiments still match the markers, so a rename that quietly stops
covering anything is a failure rather than a green tick.

It immediately found a **sixth** site that grep had missed —
`a6_ceiling_health.rs`. That one is exempted with a reason, not a rubber stamp: it
reports the raw reference-vs-arm **ordering at each training budget**, and
collapsing that into a verdict would destroy the sensitivity curve that is its
entire finding. The exemption also records why the hole cannot bite there — its
swept references run 0.9013 to 1.0000 against a chance of 0.5 — and states the
condition under which the exemption expires.

## 5. What was re-run, and what could not be

| binary | gated? | status |
|---|---|---|
| `deep-snn-scaling` | no | **re-run at v135** — see §6 |
| `track-b-rescue` | no | migrated, protocol 131 → **132**; re-run pending |
| `live-transfer-rescue` | no | migrated; re-run pending |
| `shd-arch-ablation` | `LocalLearning` | migrated; **cannot run** while the instrument is `Uncalibrated` |
| `temporal-deep-campaign` | `LocalLearning` | migrated; **cannot run**, same reason |

The two gated binaries carry a correctness fix that is **inert until calibration**.
That is stated rather than hidden: their reports on disk were produced by the old
logic and remain so until they can be regenerated.

## 6. Verification

- **`guards` unit tests:** 16 passed, including all six new ones.
- **Class guard:** 2 passed; it detects the original defect and refuses to pass
  vacuously.
- **Workspace:** `cargo fmt --check` clean, `cargo clippy --workspace
  --all-targets -D warnings` clean, GC1–GC7 all executed and passed.

### Bit-identity stress

The parallelisation underpins every campaign cell, so it was stressed rather than
spot-checked.

`deep-snn-scaling --quick`, ten thread counts including primes:

```
threads   1  2  3  5  7  8  11  13  16  18
sha256    6907404ff0d7e340  (identical at every one)
```

`shd-instrument` — the binary that produced all 600+ campaign cells — regressed by
Gate F against **recorded** values, not merely against itself:

| `RAYON_NUM_THREADS` | result |
|---:|---|
| 1 | 3/3 bit-identical → PASS |
| 3 | 3/3 bit-identical → PASS |
| 8 | 3/3 bit-identical → PASS |
| 16 | 3/3 bit-identical → PASS |

spanning two geometries and two widths. A separate 10-cell Gate F run over two
geometries, two widths and two contracts also passed 10/10, which discharged the
manifest-freeze guard blocking `recover-references`.

### Attention read-out stress

The read-out is the paper's core artifact and runs 8,156 times per epoch per cell.
Seven adversarial tests added (15 total, all passing):

- all-silent trace — inside the instrument's passing validity gates, so it reaches
  this code on real data — stays finite, rows still sum to 1, gradient defined;
- fully saturated trace, both passes;
- **non-finite parameters propagate rather than being absorbed** — the row softmax
  seeds its maximum at `NEG_INFINITY` and compares with `>`, under which `NaN > x`
  is false, so it would have been easy for corruption to emit a plausible finite
  distribution. It does not;
- repeated evaluation **bitwise** identical, forward and backward;
- a length disagreeing with `t_steps` refused in **both** directions — a wrong
  `t_steps` would silently reshape a `t × t` matrix rather than crash;
- a 400-step sequence stays normalised.

Pre-existing coverage (finite differences on every parameter, position
load-bearingness, row normalisation, zero-init of `W_o`) was already sound;
softmax was already max-subtracted with a documented `total >= 1` invariant.

### Reused-control integrity

Wave 8 and wave 9 reuse 96 control cells from waves 1/3 instead of re-running
them, which is only legitimate if both campaigns ran the same binary and the
archived cells have not moved. That was asserted in the preregistration; it is now
**checked, and unskippable**.

`scripts/aws/analyse_wave8.py` refuses to report anything unless:

- the wave-1 and wave-8 manifests record the **same pinned binary** — otherwise
  every gain is a cross-instrument comparison;
- every cell read from the wave-1 archive **matches its recorded sha256**.

Result: pinned binary `22d97c51ab020470` on both sides, **96/96 reused cells
hash-verified, 0 drifted, 0 missing, 0 unrecorded**.

The guard was negative-tested rather than assumed: editing a single `accuracy`
field in one archived control made the analysis abort with
`REUSED CELL DRIFTED` and the two hashes, and restoring the file reproduced all
six wave-8 verdicts unchanged.

## 7. Scope

- **Verified:** every table entry above, this session, on this machine.
- **Not verified:** that `track-b-rescue` v132 and `live-transfer-rescue` produce
  the same verdicts as before the migration. Both arms are already
  `INVALID_HARNESS` on the record, so the expected outcome is no change — but
  expected is not measured, and the re-runs are named as pending rather than
  assumed.
- **Not claimed:** that `MatchedDeepGradient` or `ShdEpropCeiling` are now fixed.
  They are not. This work makes their failure **impossible to mistake for a
  result**; diagnosing them is separate.
