# U23 — resting-state dynamics

**Exploratory post-G2 override.** `c1-118207fbc3eaba53` remains a FAIL.

- schedule: scientific
- ticks: 3000
- stimulus-free background is unlabeled endogenous noise
- causal consolidation proxy, structured rest: 0.3913
- causal ablation, spectrum-matched rest: 0.3847

| condition | mean activity | metastability | reactivation | transitions | lag-1 autocorrelation |
|---|---:|---:|---:|---:|---:|
| observed | 0.0141 | 0.0146 | 0.0120 | 0.0003 | -0.0169 |
| RateMatched | 0.0140 | 0.0145 | 0.0090 | 0.0000 | -0.0188 |
| ActivityMatched | 0.0141 | 0.0146 | 0.0087 | 0.0007 | -0.0169 |
| SpectrumMatched | 0.0141 | 0.0148 | 0.0087 | 0.0000 | -0.0315 |

The spectrum null circularly shifts each cell train, preserving its temporal spectrum while ablating coordinated assembly timing. This is characterized as resting-state dynamics, not a biological Default Mode Network.
