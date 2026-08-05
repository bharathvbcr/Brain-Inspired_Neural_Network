# Multi-Channel Neuromodulation Report

**Protocol Version:** 136  
**Experiment:** multi-channel-neuromod  
**Seeds:** 20; **cells:** 64; **θ:** 1; **β:** 5  
**Property floor:** 0.95 of seeds  

## Falsifiable properties

| Property | Statement | Seeds holding | Rate | Verdict |
|---|---|---:|---:|---|
| P1 | Larger \|RPE\| yields larger mean credit magnitude | 20/20 | 1.0000 | PASS |
| P2 | Reversing RPE sign reverses credit direction | 20/20 | 1.0000 | PASS |
| P3 | ACh gating concentrates credit near threshold | 20/20 | 1.0000 | PASS |
| P4 | Credit is per-cell addressed, not a broadcast scalar | 20/20 | 1.0000 | PASS |

## Descriptive means

- Mean absolute credit at RPE = 0.8: 0.516754
- Mean absolute credit at RPE = 0.1: 0.054378
- Mean signed credit at RPE = -0.8 (descriptive): -0.008925

## Verdict

- All four properties supported: **yes**

## Caveats

- This is a **property test of the modulator**, not a learning result. It says nothing about downstream task accuracy and must not be cited as evidence that multi-channel neuromodulation improves credit assignment.
- P2 holds feedback, voltages and channel weights fixed and tests vector anti-symmetry. P3 uses the explicit per-channel decomposition and holds feedback and RPE fixed; neither property is inferred by subtracting mixed signals.
