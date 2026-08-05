# Deep SNN Scaling Report

**Protocol Version:** 134  
**Experiment:** deep-snn-scaling  
**Schedule:** QUICK / PILOT (n=5, hidden=128, epochs=40)  
**Accuracy floor:** 0.65  

## Learned-feedback arms

| Arm | Hidden architecture | Mean accuracy | SE | Clears floor | Verdict |
|---|---|---:|---:|---|---|
| 1-Hidden-Layer Learned FB | 128 | 0.6000 | 0.1000 | no | UNDERPOWERED |
| 2-Hidden-Layer Learned FB | 128 × 2 | 0.4733 | 0.0125 | no | UNDERPOWERED |
| 3-Hidden-Layer Learned FB | 128 × 3 | 0.5000 | 0.0000 | no | UNDERPOWERED |
| 4-Hidden-Layer Learned FB | 128 × 4 | 0.5000 | 0.0000 | no | UNDERPOWERED |

## Depth-matched gradient ceilings

Each depth is compared against a ceiling of **the same depth**, not against a 1-hidden-layer reference. `Modulator RMS` is the realised scale of the credit signal reaching the input layer; if it differs by orders of magnitude across arms, the comparison is measuring effective learning rate rather than credit-assignment quality.

| Arm | Hidden architecture | Mean accuracy | SE | Modulator RMS | Ceiling health |
|---|---|---:|---:|---:|---|
| 1-Hidden-Layer Gradient Ceiling (depth-matched) | 128 | 0.5333 | 0.0333 | 5.052e-1 | INVERTED — ceiling below treatment; do not interpret |
| 2-Hidden-Layer Gradient Ceiling (depth-matched) | 128 × 2 | 0.5000 | 0.0000 | 5.046e-1 | ok |
| 3-Hidden-Layer Gradient Ceiling (depth-matched) | 128 × 3 | 0.5000 | 0.0000 | 5.049e-1 | ok |
| 4-Hidden-Layer Gradient Ceiling (depth-matched) | 128 × 4 | 0.5000 | 0.0000 | 5.048e-1 | ok |

## Verdict

- 4-Hidden-Layer learned feedback alignment: **UNDERPOWERED**
- Any depth clearing the floor: **no**

## Interpretation caveat

The matched-architecture task (`CoincidenceTask`) has **N_IN = 2** and `difficulty = 0.05`. A 128-wide, 4-deep stack on a 2-dimensional near-noiseless input has no depth structure to exploit, so neither a depth collapse nor a depth success on this task is strong evidence about deep credit assignment. Move this suite to an input-rich task (the SHD path already exists) before drawing a scaling conclusion.
