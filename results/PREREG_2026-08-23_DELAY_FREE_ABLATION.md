# Preregistration — is the reference gap actually delays?

**Registered:** 2026-08-23, before the run and before any number exists.

**Question:** `HANDOFF_2026-08-02.md` §3 attributes the instrument-to-reference
residual to delays — *"0.7151 → delays 0.951 = architecture cost 0.236"*. That is
a subtraction with a label on it. This measures it.

---

## 1. The one variable

The pinned reference (`Thvnvtos/SNN-delays` at `REFERENCE_COMMIT`) ships a
delay-free mode. `config.py:16`:

```python
# model type could be set to : 'snn_delays' | 'snn_delays_lr0' | 'snn'
model_type = 'snn_delays'
```

**`model_type = 'snn'` is the only edit.** Everything else in `config.py` is
derived from it by the upstream author's own code — `lr_pos = 0`,
`scheduler_pos = 'none'`, `DCLSversion = 'max'`, `sigInit = 0`, `final_epoch = 0`
(lines 69–97). Nothing else is touched: two hidden layers, dropout 0.4,
augmentations, stateful synapses, 150 epochs, batch size, optimiser, schedule and
seed all stay exactly as the calibration reference ran them.

So the contrast is **delays, and only delays**, against a run I already have a
bit-exact number for: clean seed 5170001 at **0.9389628343621399**.

## 2. Isolation from the calibration artifacts

This is a **diagnostic, not a reference**. A modified config does not produce the
pinned baseline, and an artifact from it must never be mistaken for one.

- it runs from a **scratch copy** of the worktree, outside `results/`;
- its result is written to the scratch directory, never to `references/`,
  `reference-manifests/` or `reference-states/`;
- it does not go through `runner.reference()`, so it cannot touch `gates.json`
  or the manifest freeze.

The six calibration cells cost 33 CPU-hours and are not at risk from this run.

## 3. Registered outcomes

Comparison is against 0.9390, the same seed under the same protocol with delays.

- **`snn` lands ≤ 0.78** → delays account for most or all of the residual. The
  instrument at 0.8320 (attention, converged) would then sit **above** a
  delay-free reference, and "the instrument is far below the reference" would be
  a statement about delays rather than about the instrument. The record's
  attribution is vindicated.
- **`snn` lands ≥ 0.88** → delays account for little of it. The residual is then
  layers, dropout, augmentation and stateful synapses — four things the
  instrument could adopt **without implementing delays**, which makes them the
  cheaper next targets. The record's attribution is wrong.
- **`snn` lands between 0.78 and 0.88** → delays are one contributor among
  several and no single cause dominates. Reported as a partial attribution, with
  the remaining candidates still untested and named.
- **the run fails or does not converge** → reported as a failed diagnostic. No
  number is inferred from a partial curve, and the residual stays unattributed.

## 4. Prediction, so it can be wrong

I expect **0.75–0.85** — delays mattering substantially but not entirely, because
four other differences remain and each is independently known to matter on this
dataset. If it comes back at 0.93, the delay mechanism contributes almost nothing
and every conclusion drawn from the "architecture cost" framing needs revisiting.

## 5. What this cannot claim

- **It does not measure the instrument.** It varies one knob in the *reference*.
  Any statement about BINN's arms is inference, not measurement.
- **It does not validate the 0.80 gate**, which
  `FINDING_2026-08-23_THE_MATRIX_GRID_EXCLUDES_ITS_OWN_GATE.md` shows the matrix
  grid cannot reach for unrelated reasons.
- **One seed.** The clean reference spread across three seeds is 0.9368–0.9390, so
  a difference smaller than ~0.003 is not resolvable here and none is claimed.
- **No gate moves.** `SHD_INSTRUMENT_STATE` stays `Uncalibrated`; `gates.json` is
  untouched.
