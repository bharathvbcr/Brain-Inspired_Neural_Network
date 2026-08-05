# Dynamic E-I Interneuron Sweeps Report

**Protocol Version:** 135  
**Experiment:** ei-inhibition-sweep  
**Seeds per point:** 10; **steps per seed:** 50  
**Drive:** heterogeneous sparse activity (10% strongly driven, 90% weak background)

## Sweep

`Competition spread` is `(max − min) / mean` of the inhibitory current across excitatory cells. Uniform gain control gives 0; genuine competition requires ≥ 0.01.

| E:I ratio | W(I→E) | Mean inhibition | Competition spread | Temporal CV | Per-point verdict |
|---|---:|---:|---:|---:|---|
| 4:1 | 0.2 | 0.5592 | 0.162007 | 0.2848 | PASS |
| 4:1 | 0.5 | 1.3979 | 0.162007 | 0.2848 | PASS |
| 4:1 | 1.0 | 2.7959 | 0.162007 | 0.2848 | PASS |
| 8:1 | 0.2 | 1.1644 | 0.319417 | 0.2851 | PASS |
| 8:1 | 0.5 | 2.9109 | 0.319417 | 0.2851 | PASS |
| 8:1 | 1.0 | 5.8219 | 0.319417 | 0.2851 | PASS |
| 16:1 | 0.2 | 2.3243 | 0.640354 | 0.2854 | PASS |
| 16:1 | 0.5 | 5.8108 | 0.640354 | 0.2854 | PASS |
| 16:1 | 1.0 | 11.6215 | 0.640354 | 0.2854 | PASS |

## Criteria

| Criterion | Statement | Verdict |
|---|---|---|
| C1 | Mean inhibition increases with W(I→E) at fixed E:I | PASS |
| C2 | Mean inhibition increases with E:I ratio at fixed W(I→E) | PASS |
| C3 | Inhibition is differential across excitatory cells (soft-WTA competition) | PASS |

## Verdict

- Graded, monotone gain control: **supported**
- Soft-WTA *competition*: **supported**

## Mechanism disclosure

`InhibitoryInterneuronArea` uses deterministic heterogeneous E→I receptive fields and I→E projections. Projection normalization is population-level rather than per-cell, so individual excitatory cells receive distinct inhibitory currents. C3 still comes from the measured spread above; it is not assumed from the design.
