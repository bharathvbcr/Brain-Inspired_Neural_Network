# Deep SNN Scaling Report

**Protocol Version:** 134  
**Experiment:** deep-snn-scaling  
**Schedule:** FULL SCIENTIFIC (n=20, hidden=256, epochs=80)  
**Accuracy floor:** 0.65  

## Learned-feedback arms

| Arm | Hidden architecture | Mean accuracy | SE | Clears floor | Verdict |
|---|---|---:|---:|---|---|
| 1-Hidden-Layer Learned FB | 256 | 1.0000 | 0.0000 | yes | PASS |
| 2-Hidden-Layer Learned FB | 256 × 2 | 0.5060 | 0.0768 | no | FAIL |
| 3-Hidden-Layer Learned FB | 256 × 3 | 0.5810 | 0.0679 | no | FAIL |
| 4-Hidden-Layer Learned FB | 256 × 4 | 0.4435 | 0.0664 | no | FAIL |

## Depth-matched gradient ceilings

Each depth is compared against a ceiling of **the same depth**, not against a 1-hidden-layer reference. `Modulator RMS` is the realised scale of the credit signal reaching the input layer; if it differs by orders of magnitude across arms, the comparison is measuring effective learning rate rather than credit-assignment quality.

| Arm | Hidden architecture | Mean accuracy | SE | Modulator RMS | Ceiling health |
|---|---|---:|---:|---:|---|
| 1-Hidden-Layer Gradient Ceiling (depth-matched) | 256 | 0.4880 | 0.0120 | 5.044e-1 | INVERTED — ceiling below treatment; do not interpret |
| 2-Hidden-Layer Gradient Ceiling (depth-matched) | 256 × 2 | 0.5000 | 0.0000 | 5.035e-1 | INVERTED — ceiling below treatment; do not interpret |
| 3-Hidden-Layer Gradient Ceiling (depth-matched) | 256 × 3 | 0.5000 | 0.0000 | 5.035e-1 | INVERTED — ceiling below treatment; do not interpret |
| 4-Hidden-Layer Gradient Ceiling (depth-matched) | 256 × 4 | 0.5000 | 0.0000 | 5.036e-1 | ok |

## Verdict

- 4-Hidden-Layer learned feedback alignment: **FAIL**
- Any depth clearing the floor: **yes**

## Interpretation caveat

The matched-architecture task (`CoincidenceTask`) has **N_IN = 2** and `difficulty = 0.05`. A 256-wide, 4-deep stack on a 2-dimensional near-noiseless input has no depth structure to exploit, so neither a depth collapse nor a depth success on this task is strong evidence about deep credit assignment. Move this suite to an input-rich task (the SHD path already exists) before drawing a scaling conclusion.
