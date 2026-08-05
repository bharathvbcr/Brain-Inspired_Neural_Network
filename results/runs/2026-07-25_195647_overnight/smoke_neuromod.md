# Multi-Channel Neuromodulation Report

**Protocol Version:** 135  
**Experiment:** multi-channel-neuromod  
**Seeds:** 20; **cells:** 64; **θ:** 1; **β:** 5  
**Property floor:** 0.95 of seeds  

## Falsifiable properties

| Property | Statement | Seeds holding | Rate | Verdict |
|---|---|---:|---:|---|
| P1 | Larger \|RPE\| yields larger mean credit magnitude | 19/20 | 0.9500 | PASS |
| P2 | Reversing RPE sign reverses credit direction | 0/20 | 0.0000 | FAIL |
| P3 | ACh gating concentrates credit near threshold | 11/20 | 0.5500 | FAIL |
| P4 | Credit is per-cell addressed, not a broadcast scalar | 20/20 | 1.0000 | PASS |

## Descriptive means

- Mean credit at RPE = 0.8: 0.025862
- Mean credit at RPE = 0.1: 0.002666

## Verdict

- All four properties supported: **no**

## Caveats

- This is a **property test of the modulator**, not a learning result. It says nothing about downstream task accuracy and must not be cited as evidence that multi-channel neuromodulation improves credit assignment.
- P3 subtracts a scaled low-RPE signal as a crude proxy for the DA term. It is a directional check, not an exact channel decomposition; a clean test needs per-channel ablation hooks on `MultiChannelNeuromodulator`.
