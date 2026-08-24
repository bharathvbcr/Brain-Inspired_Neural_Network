# Preregistration — batchnorm, the last untested item on the list

**Registered:** 2026-08-24, before the run.

**Why:** `FINDING_2026-08-24_THE_FORWARD_PASSES_DIFFER_IN_KIND.md` found two
differences that were never enumerated — a 25-tap temporal kernel on every
synapse, and `BatchNorm1d` on every layer. The kernel cannot be removed by
configuration; **batchnorm can.** That finding says explicitly that batchnorm
should be tested before the kernel is credited with the remainder, and this is
that test.

---

## 1. The ablation

| id | change from the delay-free baseline (0.9042) | isolates |
|---|---|---|
| **A-4** | `use_batchnorm: True → False` | normalisation |

`use_batchnorm` gates three `BatchNorm1d` insertions — first layer
(`snn_delays.py:37`), hidden blocks (`:65`), and the forward application
(`:218`). One config line, all three follow.

Everything else held: `model_type = 'snn'`, 2 hidden layers, dropout 0.4,
`loss = 'sum'`, seed 5170001, 150 epochs, clean protocol.

## 2. Thresholds

Unchanged, from the 0.0022 three-seed spread: **≥ 0.010 real, < 0.005 not
resolvable at n=1, between suggestive.**

## 3. Outcomes, with signs named

Four ablations in, three of my predictions have failed and one outcome set had no
branch for the sign that occurred. So every direction is enumerated.

- **A-4 much lower (≥ 0.03)** → batchnorm is a large part of the reference's
  advantage. It is also the one remaining difference the instrument could adopt
  without changing model class, which would make it the actionable finding of the
  whole series.
- **A-4 lower by 0.010–0.03** → real but partial; the temporal kernel keeps most
  of the remainder.
- **A-4 within 0.010 either way** → batchnorm is exonerated. Every configurable
  difference is then measured and none accounts for the gap, leaving the temporal
  kernel as the only identified candidate — supported by elimination and by
  reading the code, and still not by an ablation, because it cannot be one.
- **A-4 *higher*** → batchnorm is costing the reference accuracy, as the second
  hidden layer does. Two suboptimal choices would then be two too many for the
  "stronger reference" framing to survive unexamined.
- **the run fails to converge** → reported as a failed diagnostic. Removing
  normalisation from a spiking network can destabilise training, and a divergent
  run is evidence about stability, **not** a measurement of accuracy. It would be
  reported as such and no number inferred from it.

## 4. Prediction

I expect **lower by 0.01–0.04**. Recorded with low confidence: my predictions in
this series have been wrong on magnitude, on sign, and on magnitude again.

## 5. What this cannot claim

- **One seed.**
- **It varies the reference, not the instrument.**
- **It does not test the temporal kernel**, which is not a config change. Whatever
  A-4 returns, the kernel remains supported by elimination rather than by
  measurement, and that distinction must survive into any summary.
- Diagnostic only: scratch copy, nothing written to `references/`,
  `reference-manifests/`, `reference-states/`; `gates.json` not opened.
