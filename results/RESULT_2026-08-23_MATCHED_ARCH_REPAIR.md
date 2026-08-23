# Result — the matched architecture spikes now, and the conclusion it supported survives

**Prereg:** `PREREG_2026-08-23_MATCHED_ARCH_REPAIR.md`, registered before either
repair and before any post-repair number existed.
**Diagnosis:** `FINDING_2026-08-22_THE_MATCHED_ARCHITECTURE_CANNOT_SPIKE.md`.

---

## 1. Criteria

| id | criterion | outcome |
|---|---|---|
| **M-1** | no hidden layer silent at initialisation | **MET** — rate 0.01172–0.11328 across widths 16/64/256 × 50 seeds; band is `[0.001, 0.500]` |
| **M-2** | the working reference is not broken | **MET** — `MatchedGradient` 1.0000, bar 0.99 |
| **M-3** | the local arm is not a constant predictor | **MET as registered** — on the unbalanced fixture, 1.0000 against a majority rate of 0.7500, Δ = 0.25. **But see §3**: on the *balanced* published fixture it is still exactly 0.5000. |
| **M-4** | one shared place, pinned | **MET** — `MATCHED_INPUT_SCALE`, with a test that re-runs the ladder selection |
| **C-1** | the registered constructor | **MET** — three binaries corrected, pinned by a test across all four call sites |
| **C-2** | Gate F untouched | **MET** — 10/10 bit-identical. `MatchedArch` being off the instrument path was checked, not assumed. |

## 2. Repair A — the forward

`MATCHED_INPUT_SCALE` goes from `0.5` to `2.0`: the smallest rung of a doubling
ladder whose initial firing rate is inside the activity band at every width and
seed. **Accuracy was not an input to the choice**, and a test asserts both that
2.0 qualifies and that 1.0 and 0.5 do not.

| in_scale | min rate | max rate | inside band |
|---:|---:|---:|---|
| 0.5 | 0.00000 | 0.00000 | no — **silent, at every seed** |
| 1.0 | 0.00000 | 0.03906 | no |
| **2.0** | **0.01172** | **0.11328** | **yes** |
| 4.0 | 0.03906 | 0.22656 | yes |

**What the repair costs, reported because it was registered as reported.** At
`0.5` a single channel contributed at most 0.5 and two coincident channels at
most 1.0 — the architecture was built as a **coincidence detector**, the right
shape for `CoincidenceTask`. That selectivity does not survive: at 2.0 the initial
rate is **0.050 on coincident input and 0.059 on split input**, so single channels
now cross threshold alone and the split case fires slightly *more*. The arms can
still separate the classes by rate, but they are no longer detecting coincidence,
and any reading of them as coincidence detection is now wrong.

## 3. The registered second outcome fired: necessary, not sufficient

§4's second named outcome — *"M-1 holds, M-3 fails → the layer spikes and the
local rule still does not learn"* — is what happened on the published
configuration, and it is why it was named in advance.

Balanced fixture, 12 seeds, after the repair:

```
mean = 0.5000   variance = 0.000000
accs = [0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5, 0.5]
initial hidden activity = 0.03125 .. 0.06641    (was 0.00000 at every seed)
```

The layer spikes. The arm still emits a constant prediction. Regenerating the
report confirms it end to end: `matched-local = 0.5000`, `matched-gradient =
1.0000`, `gap_closed_matched = 0.0000` — **the published numbers do not move.**

M-3 as registered was stated on an unbalanced fixture and passes there (1.0000
against a 0.7500 majority), so the arm is not degenerate in every configuration.
On the balanced published configuration it is. Per §4 the repair is kept — M-2
holds — and the residual is **re-scoped from the initialisation to the rule**.

## 4. This corrects my own finding, in the direction that matters

`FINDING_2026-08-22_THE_MATCHED_ARCHITECTURE_CANNOT_SPIKE.md` §3 said of
`MATCHED_ARCH_DFA_CONTROL.md`'s claim — *"the FAIL is the **rule**, not the
path"* — that "identical" was carrying an implication it could not support,
because the shared path was not functional.

**That was the right objection and the wrong prediction.** With the path
demonstrably functional — every layer inside the activity band, the ceiling on
the same graph reaching 1.0000 — the local rule still fails at exactly chance
with zero variance across seeds. The conclusion the record drew was correct. What
was missing was the evidence for it, and it is no longer missing: before the
repair a silent path and a failing rule produced the same 0.5000, and nothing
distinguished them. Now only one explanation is left standing.

The finding's §3 is amended accordingly. Its §1 and §2 — the arithmetic, the zero
spikes, the majority-rate signature — stand exactly as measured.

## 5. Repair B — the ceiling constructor

`track_b_rescue.rs:174`, `live_transfer_rescue.rs:137` and
`continual_learning.rs:40` now build `MatchedGradient::new_feedforward`, matching
`MATCHED_ARCH_RL_CONTROL.md:37`, both sibling runners, and
`a6_ceiling_health.rs`. The ceiling no longer carries a `hidden × hidden`
recurrent matrix that no treatment arm has.

`binn-lab/tests/matched_ceiling_constructor.rs` pins all four call sites and
refuses a vacuous pass: if the constructor is renamed or the call sites move,
finding nothing must not read as finding nothing wrong.

## 6. What this may not claim

- **It does not re-open a verdict.** `track_b_results_v132.md`'s *"ceiling
  inverted — no PASS is permitted while this warning is present"* stands until a
  re-run under the corrected constructor is registered and read. A corrected
  ceiling does not retroactively grant a PASS its own warning withheld.
- **It does not vindicate a prior number.** Every matched-arch figure on record
  came from a forward that could not spike, a ceiling of the wrong architecture,
  or both. That `c1_match.md`'s numbers happen to be unchanged is a *finding*,
  not a continuity — it is what tells us the initialisation was not the cause.
- **It does not settle the local rule.** Whether a broadcast three-factor rule can
  learn this task at some operating point is untested and needs its own
  registration. Trying scales until one works is exactly what §1's rule exists to
  prevent.
- **No gate moved.** `SHD_INSTRUMENT_STATE` stays `Uncalibrated`,
  `matrix_authorized` stays false, Gate F stays 10/10.
