# Preregistration — repairing the silent initialisation shared by both broken references

**Registered:** 2026-08-22, **before the repair and before any post-repair number
exists.** Authorised by the maintainer.

---

## 1. What is being repaired, and the correction that made it one thing

Two references were diagnosed at chance:

- `MatchedDeepGradient` — `FINDING_2026-08-22_MATCHED_DEEP_GRADIENT_COLLAPSES_TO_SILENCE.md`
- `ShdEpropCeiling` — `FINDING_2026-08-22_SHD_EPROP_CEILING_IS_A_CONSTANT_PREDICTOR.md`

**That second document claims the two have *different* mechanisms. It is wrong,
and this preregistration corrects it before building on it.** The claim rested on
`ShdEpropCeiling`'s modulator being non-zero and data-dependent, which was read as
"credit flows". Measuring the hidden layer directly shows the mean firing rate is
**0.00000000 both untrained and after 60 epochs**: the modulator is non-zero
because the surrogate derivative is non-zero below threshold, not because anything
spiked. `wout` never moves (`sum|w_out|` 4.106466 before and after) and the logits
are pure bias, bit-identical across classes.

So both defects are **the same defect**: the hidden layer is initialised below
threshold and the learning rule cannot climb out.

## 2. The measurement that identifies the cause

`ShdSuperSpikeCeiling` shares `ShdArch::new` with the failing arms — same
initialisation, same threshold — and solves the task:

| | hidden mean rate | `sum\|win\|` | accuracy |
|---|---:|---:|---:|
| shared initialisation | 0.00000000 | 52.6444 | — |
| `ShdSuperSpikeCeiling`, 60 epochs | **0.01562500** | **115.5081** | **1.0000** |
| `ShdEpropCeiling`, 60 epochs | 0.00000000 | 56.9404 | 0.1000 |

Every arm starts silent. The BPTT reference grows its input weights 2.2× and
crosses threshold; the local arm grows them 1.08× and never does. The trap is
circular: **no spikes → vanishing eligibility → no weight growth → no spikes.**

## 3. The repair

Raise the initialisation so the hidden layer spikes at initialisation, in
`ShdArch::new` and in `MatchedDeepGradient::new`. **The learning rules, the
thresholds, the surrogate, the readout and the optimiser are untouched** — only
the operating point the network starts from.

This is deliberately the *least* invasive repair that can work. It does not
advantage any arm over another: every arm in these modules shares the
initialisation, so all of them start from the same new operating point.

## 4. Registered acceptance criteria, fixed before the repair

| id | criterion | bar |
|---|---|---|
| **F-1** | the hidden layer is no longer silent at initialisation | mean firing rate in `[ACTIVITY_MIN, ACTIVITY_MAX]` = `[0.001, 0.500]`, the band this workspace already uses |
| **F-2** | the previously-working reference is **not broken** by the change | `ShdSuperSpikeCeiling` ≥ 0.99 on the disjoint fixture at 20 epochs, as before |
| **F-3** | `MatchedDeepGradient` learns its own separable fixture | accuracy ≥ 0.90 at depth 1, against 0.5000 today |
| **F-4** | the readout is no longer constant | more than one distinct predicted class, for both `ShdEpropCeiling` and `ShdDfa` |
| **F-5** | no arm is silently advantaged | the initialisation change is applied in one shared place per module, verified by inspection |

## 5. Named outcomes

- **All of F-1…F-5 hold** → both defects are repaired, the characterization tests
  are inverted to assert the *working* behaviour, and the findings are amended
  with the post-repair numbers.
- **F-1 holds but F-3 or F-4 fails** → silence was necessary but not sufficient.
  The repair is kept only if F-2 still holds, and the residual defect is
  documented as still open rather than declared fixed.
- **F-2 fails** → the repair is **reverted**. Breaking a working reference to fix
  a broken one is not a repair, and no partial credit is taken.

## 6. What this may not claim

- **It does not revive any withdrawn result.** `deep-snn-scaling` and
  `shd-scientific-sweep` stay withdrawn; a repaired instrument does not
  retroactively validate a report produced by the broken one.
- **It does not affect any live claim.** `matrix_authorized` is `false`, so
  nothing in the calibration matrix is authorised, and the attention campaign
  does not use either type.
- **It is a provenance event.** Every recorded number from these modules came
  from the pre-repair initialisation and is not comparable to anything produced
  after it.
