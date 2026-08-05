# Dynamic E-I Interneuron Sweeps Report

**Protocol Version:** 134  
**Experiment:** ei-inhibition-sweep  
**Seeds per point:** 10; **steps per seed:** 50  
**Drive:** heterogeneous sparse activity (10% strongly driven, 90% weak background)

## Sweep

`Competition spread` is `(max − min) / mean` of the inhibitory current across excitatory cells. Uniform gain control gives 0; genuine competition requires ≥ 0.01.

| E:I ratio | W(I→E) | Mean inhibition | Competition spread | Temporal CV | Per-point verdict |
|---|---:|---:|---:|---:|---|
| 4:1 | 0.2 | 0.5592 | 0.000000 | 0.2849 | FAIL |
| 4:1 | 0.5 | 1.3981 | 0.000000 | 0.2849 | FAIL |
| 4:1 | 1.0 | 2.7962 | 0.000000 | 0.2849 | FAIL |
| 8:1 | 0.2 | 1.1651 | 0.000000 | 0.2849 | FAIL |
| 8:1 | 0.5 | 2.9127 | 0.000000 | 0.2849 | FAIL |
| 8:1 | 1.0 | 5.8255 | 0.000000 | 0.2849 | FAIL |
| 16:1 | 0.2 | 2.3302 | 0.000000 | 0.2849 | FAIL |
| 16:1 | 0.5 | 5.8255 | 0.000000 | 0.2849 | FAIL |
| 16:1 | 1.0 | 11.6510 | 0.000000 | 0.2849 | FAIL |

## Criteria

| Criterion | Statement | Verdict |
|---|---|---|
| C1 | Mean inhibition increases with W(I→E) at fixed E:I | PASS |
| C2 | Mean inhibition increases with E:I ratio at fixed W(I→E) | PASS |
| C3 | Inhibition is differential across excitatory cells (soft-WTA competition) | FAIL |

## Verdict

- Graded, monotone gain control: **supported**
- Soft-WTA *competition*: **NOT supported**

## Known architectural limitation

`InhibitoryInterneuronArea::compute_inhibition` returns `vec![inhibition_per_e; n_excitatory]` — one scalar broadcast to every excitatory cell. Uniform inhibition rescales all scores identically and therefore cannot change the k-WTA winner ordering. C3 cannot pass until the interneuron population projects with cell-specific weights. Until then this module provides **divisive gain control**, not competition, and reports must say so.
