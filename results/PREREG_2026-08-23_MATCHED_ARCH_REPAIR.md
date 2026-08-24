# Preregistration — the matched architecture's silent initialisation, and the ceiling constructor

**Registered:** 2026-08-23, **before either repair and before any post-repair
number exists.** Authorised by the maintainer, who asked for both.

**Diagnosis:** `FINDING_2026-08-22_THE_MATCHED_ARCHITECTURE_CANNOT_SPIKE.md`.

---

## 1. Repair A — the forward cannot spike

`MatchedArch::with_options` (`binn-learn/src/matched_local_baseline.rs:112`) draws
`win ~ U[−0.5, 0.5]` against `THETA_REST = 1.0` with `α = exp(−0.1)`. The largest
membrane two adjacent unit impulses can reach is `α·0.5 + 0.5 = 0.952419`.
Measured: **0 spikes in 400 forwards, max membrane 0.974568.**

The intent is visible in the numbers and is worth stating, because the repair must
not quietly discard it: at `in_scale = 0.5` a single channel contributes at most
0.5 and two coincident channels at most 1.0, which is a **coincidence detector** —
exactly right for `CoincidenceTask`. The defect is that the threshold is not
merely selective but unreachable, since it needs both weights at their maximum
simultaneously.

### The rule for choosing the new scale, fixed before the sweep

The smallest rung of a doubling ladder from `0.5` whose **initial mean hidden
firing rate** lies inside `[ACTIVITY_MIN, ACTIVITY_MAX] = [0.001, 0.500]`, at
every width in {16, 64, 256} and across 50 seeds. Rate is
`spikes / (hidden × T)`.

**Accuracy is not an input to this choice**, exactly as in
`PREREG_2026-08-22_DEEP_PATH_AND_TRANSPORT_SCALE.md` §4. The ladder and the
realised rates are reported whatever they show.

**Coincidence selectivity is reported, not optimised.** At the chosen scale the
report states the firing rate on coincident versus non-coincident inputs, so a
reader can see whether the architecture's original intent survived. It is not a
criterion, because making it one would mean choosing the scale on a second
objective and then having no rule to break the tie.

Every arm in this family shares `MatchedArch`, so all of them move together and
none is advantaged.

## 2. Repair B — a recurrent ceiling for feedforward arms

Three binaries build `MatchedGradient::new` (`wrec ≠ 0`) while every treatment arm
uses `MatchedArch::feedforward` (`wrec = 0`):

```
track_b_rescue.rs:174        live_transfer_rescue.rs:137     continual_learning.rs:40
```

Against: `MATCHED_ARCH_RL_CONTROL.md:37` naming `MatchedGradient (new_feedforward)`,
`runner_rl_match.rs:268` and `runner_dfa_match.rs:244` using it, and
`a6_ceiling_health.rs:311` — the diagnostic built for this very question — using
it.

**This is not a parameter choice.** It restores the constructor the
preregistration names. One line per file, no threshold, no tuning.

## 3. Registered acceptance criteria

| id | criterion | bar |
|---|---|---|
| **M-1** | no hidden layer silent at initialisation | mean rate in `[0.001, 0.500]` at widths 16/64/256 across 50 seeds |
| **M-2** | the working reference is not broken | `MatchedGradient` ≥ 0.99 on its own fixture, as it is today |
| **M-3** | the local arm is no longer a constant predictor | on an **unbalanced** fixture, accuracy differs from the majority-class rate by more than `CONSTANT_PREDICTOR_EPS = 1e-4` |
| **M-4** | one shared place | the scale is a single named constant, pinned by a test that re-runs the ladder selection |
| **C-1** | the three binaries use the registered constructor | `new_feedforward`, verified by inspection and by a test that fails if any of them regresses |
| **C-2** | Gate F is untouched | 10/10 bit-identical; `MatchedArch` is not on the instrument path and this checks rather than assumes |

## 4. Named outcomes

- **M-1…M-4 and C-1…C-2 hold** → both repaired. Affected reports are regenerated
  and every changed number carries its provenance.
- **M-1 holds, M-3 fails** → the layer spikes and the local rule still does not
  learn. The repair is kept only if M-2 holds, and the residual is **re-scoped to
  the rule** rather than the initialisation. This is the outcome that fired on the
  deep path yesterday, and naming it again is not pessimism — it is the most
  likely result, because breaking silence was necessary and not sufficient there.
- **M-2 fails** → Repair A is **reverted**. Breaking a working reference to fix a
  broken arm is not a repair.
- **C-2 fails** → stop. `MatchedArch` was believed off the instrument path; if it
  is not, the blast radius is much larger than this document assumes and the
  change needs re-scoping before anything else.

## 5. What this may not claim

- **It does not vindicate any prior number.** Every matched-arch figure on record
  — `c1_match.md`, `c1_eventprop.md`, `c1_dfa.md`, `track_b_results_v132.md` and
  their siblings — came from a forward that could not spike, or from a ceiling of
  the wrong architecture, or both. They are **not comparable** with anything
  produced after this.
- **It does not settle whether the local rule works.** If M-3 passes, that shows
  the arm is no longer degenerate; whether it *learns* is a separate question at a
  separate operating point.
- **It does not re-open a verdict.** `track_b_results_v132.md`'s harness warning
  stands until a re-run under the corrected constructor is registered and run. A
  corrected ceiling does not retroactively grant the PASS its own warning
  withheld.
- **It touches no gate.** `SHD_INSTRUMENT_STATE` stays `Uncalibrated`;
  `matrix_authorized` stays false.
