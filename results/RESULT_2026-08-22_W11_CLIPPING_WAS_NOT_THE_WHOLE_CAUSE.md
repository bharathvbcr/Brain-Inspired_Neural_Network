# Result — wave 11: clipping was a large cause, and not the whole one

**Amendment:** `AMENDMENT_2026-08-22_WAVE4_WITHOUT_CLIPPING.md`, registered before
launch. **Analyser:** `scripts/aws/analyse_wave11.py`, frozen before the first
cell landed.

**Registered outcome: §3 completion expectation NOT MET.** 15 of 24 against a bar
of 18. **T4-1, T4-2 and T4-3 are NOT EVALUABLE and no verdict is issued.**

---

## 1. What the amendment said would happen if this happened

> If fewer than 18 complete, the diagnosis in the finding is incomplete —
> clipping was then not the whole cause — and the re-run is reported as such
> rather than patched with a third parameter. **No further lever is added without
> its own amendment.**

That is the outcome, and it is honoured. Nothing was retried, no threshold moved,
no parameter added.

## 2. What is nonetheless established

**Clipping was a real and large cause.** Two independent lines:

- **Paired local control**, same binary, same seed, byte-identical initial
  weights, same data order, differing only in `--clip-grad-norm`: overflow at
  optimizer step 244 clipped, 100 epochs completed unclipped.
- **0 of 24 became 15 of 24** on the same grid.

**And it was not the only cause.** Nine cells still diverged, with the same guard
signature but much later — optimizer steps **438, 496, 643, 1035** against wave
4's median of about 176. Removing the flag delayed the divergence rather than
eliminating it.

So `FINDING_2026-08-22_WAVE4_KILLED_ITS_OWN_CELLS.md` is **amended, not
withdrawn**: its paired control stands, its withdrawal of the wave-4 verdict
stands — that document claimed *zero* usable cells and there are fifteen — and
its §4 statement that the numerical marginality is real and independent of
clipping is now the operative half.

## 3. Descriptive numbers, which are not a verdict

Recorded because they will motivate the properly-powered re-run, and labelled so
they cannot be mistaken for one.

| arm | surrogate scale | completed | mean accuracy | range |
|---|---:|---:|---:|---|
| `rec+alif` | 1.0 | 4/6 | 0.4654 | 0.4541 – 0.4770 |
| `rec+alif` | 0.4 | 3/6 | 0.4841 | 0.4563 – 0.5004 |
| `rec+alif+attn` | 1.0 | 3/6 | 0.7730 | 0.7681 – 0.7783 |
| `rec+alif+attn` | 0.4 | 5/6 | 0.7135 | 0.6833 – 0.7473 |

Every completing cell predicts all 20 classes with a majority share ≤ 0.155, so
none is degenerate.

Seeds where **both** arms survived — 5 of 12 planned pairs:

| seed | scale | attention − plain |
|---|---|---:|
| 5170001 | 0.4 | +0.1829 |
| 5170002 | 1.0 | +0.3185 |
| 5170003 | 0.4 | +0.2911 |
| 5170005 | 0.4 | +0.2359 |
| 5170006 | 1.0 | +0.3092 |

**Why this is not a result, and the reason is not procedural.** Nine of 24 cells
diverged, so the completing set is **not a random sample of the planned one**. An
arm that diverges more often can look better precisely because only its luckier
trajectories survive to be scored. Survivorship is exactly what the completion
bar exists to protect against, and it is why T4-2 is gated behind §3 rather than
reported alongside it. Five pairs from a biased sample is a reason to run the
experiment properly, not a measurement.

## 4. A defect in this analyser, found after the wave closed

The analyser was frozen before the first cell landed. **It was also wrong in two
places**, and both would have fired had the completion bar passed:

1. It grouped on `surrogate_scale == 0.4`. The field is f32 and the cell records
   **0.400000006**, so that bucket would have been empty and T4-3 a NaN.
2. It keyed paired seeds on `cell["seed"]`. **The emitted cell has no `seed`
   field** — `HARDENING_2026-08-22_THE_EVIDENCE_LAYER_HAD_NO_TESTS.md` had
   already recorded that as open work, and this analyser was written against it
   anyway. It would have raised `KeyError`.

Neither ran, because the completion expectation failed first. **That is luck, not
process.** Both are fixed here, declared: they are bugs, not threshold changes,
no verdict was issued before or after, and every bar is exactly as registered.

The generalisable point is the one worth keeping: **freezing an analyser before
the data does not make it correct.** It has to be exercised against a synthetic
fixture before the real cells arrive. `scripts/test_campaign_tooling.py::Wave11AnalyserTest`
now does that — four tests over a synthetic 24-cell grid that reproduces both
traps rather than a tidied version of them, and both original bugs turn it red.

## 5. What is still open

- **`rec+alif` remains unmeasured.** It is not refuted, and it is now known to
  produce usable cells — but not reliably enough at this operating point for a
  registered verdict.
- **The next lever is not chosen here.** The marginality analysis points at the
  `du` recursion's per-timestep backward gain, but picking a remedy after seeing
  this wave fail is what the amendment forbids. It needs its own registration.
- **Whether attention helps the recurrent arm is still unanswered**, for the
  third campaign running. Wave 4 answered nothing because every cell died; wave
  11 answers nothing because too few survived.
