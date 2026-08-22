# Deep SNN Scaling Report

**Protocol Version:** 136  
**Experiment:** deep-snn-scaling  
**Schedule:** FULL SCIENTIFIC (n=20, hidden=128, epochs=60)  
**Accuracy floor:** 0.65  
**Preregistration:** `results/PREREG_2026-08-22_DEEP_PATH_AND_TRANSPORT_SCALE.md` section 7  

Treatment and ceiling share one forward graph, one initialisation, one optimiser and one step size. They differ in the credit pathway and in nothing else: the treatment projects the readout error through a learned feedback matrix, the ceiling uses exact reverse-mode gradients. The Adam ceiling is the best-achievable reference for the same architecture and does **not** decide ceiling health, because it differs from the treatment in two things at once.

## Headline, per depth

Both arms use Adam at the module's frozen settings. There is no step size to choose and nothing was tuned on either arm.

| Depth | Hidden | Treatment | SE | Ceiling | SE | Gap | Input modulator RMS | Ceiling health | Verdict |
|---:|---|---:|---:|---:|---:|---:|---:|---|---|
| 1 | 128 x 1 | 0.9920 | 0.0009 | 0.9945 | 0.0011 | -0.0025 | 2.648e-1 | ok | PASS |
| 2 | 128 x 2 | 1.0000 | 0.0000 | 1.0000 | 0.0000 | +0.0000 | 5.620e-3 | ok | PASS |
| 3 | 128 x 3 | 0.9740 | 0.0022 | 1.0000 | 0.0000 | -0.0260 | 2.976e-1 | ok | PASS |
| 4 | 128 x 4 | 0.9780 | 0.0052 | 1.0000 | 0.0000 | -0.0220 | 3.914e-1 | ok | PASS |

`Gap` is treatment minus ceiling; negative means the treatment is below its own reference. `Input modulator RMS` is the realised scale of the credit signal reaching the input layer - the deepest the credit has to travel. If it collapses with depth, the comparison is measuring effective step size rather than credit-assignment quality.

## Why Adam, and what plain SGD does

The optimiser is matched across the two arms either way. Adam is used because plain SGD cannot train this architecture at depth: the table below is the full registered step-size ladder, run for **both** arms at every depth. The selection that led here read the **ceiling** only - a reference that cannot learn bounds nothing - and never the treatment.

| Depth | SGD lr | Treatment | Ceiling | |
|---:|---:|---:|---:|---|
| 1 | 1e-3 | 0.5895 | 0.5895 |  |
| 1 | 3e-3 | 0.5555 | 0.5565 |  |
| 1 | 1e-2 | 0.5750 | 0.5780 |  |
| 1 | 3e-2 | 0.8630 | 0.8680 |  |
| 1 | 1e-1 | 1.0000 | 1.0000 | best rung |
| 2 | 1e-3 | 0.5000 | 0.5000 |  |
| 2 | 3e-3 | 0.5000 | 0.5000 |  |
| 2 | 1e-2 | 0.5000 | 0.5000 |  |
| 2 | 3e-2 | 0.5000 | 0.5000 |  |
| 2 | 1e-1 | 0.5000 | 0.5000 | best rung |
| 3 | 1e-3 | 0.5000 | 0.5000 |  |
| 3 | 3e-3 | 0.5000 | 0.5000 |  |
| 3 | 1e-2 | 0.5000 | 0.5000 |  |
| 3 | 3e-2 | 0.5000 | 0.5000 |  |
| 3 | 1e-1 | 0.5000 | 0.5000 | best rung |
| 4 | 1e-3 | 0.5000 | 0.5000 |  |
| 4 | 3e-3 | 0.5000 | 0.5000 |  |
| 4 | 1e-2 | 0.5000 | 0.5000 |  |
| 4 | 3e-2 | 0.5000 | 0.5000 |  |
| 4 | 1e-1 | 0.5000 | 0.5000 | best rung |

## Verdict

- Deepest (4-layer) learned feedback alignment: **PASS**
- Any depth clearing the floor: **yes**

## Interpretation caveat

The matched-architecture task (`CoincidenceTask`) has **N_IN = 2** and `difficulty = 0.05`. A 128-wide, 4-deep stack on a 2-dimensional near-noiseless input has no depth structure to exploit, so neither a depth collapse nor a depth success on this task is strong evidence about deep credit assignment. Move this suite to an input-rich task before drawing a scaling conclusion.

## Provenance

v134 and v135 of this suite are **withdrawn** and are not comparable with anything here. They compared a ceiling that was silent above layer 1 against a treatment that was silent above layer 1 **and** had no readout bias, so its decision boundary was pinned at the origin. See `results/PREREG_2026-08-22_DEEP_PATH_AND_TRANSPORT_SCALE.md` sections 1-2 for the measurements, and `results/RESULT_2026-08-20_DEEP_SNN_V134_CEILING_IS_AT_CHANCE.md` for the original withdrawal.
