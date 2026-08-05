# U23 — resting-state dynamics

**Exploratory post-G2 override.** `c1-118207fbc3eaba53` remains a FAIL.

- schedule: PILOT
- ticks: 300
- stimulus-free background is unlabeled endogenous noise
- causal consolidation proxy, structured rest: 0.4400
- causal ablation, spectrum-matched rest: 0.4233

| condition | mean activity | metastability | reactivation | transitions | lag-1 autocorrelation |
|---|---:|---:|---:|---:|---:|
| observed | 0.0153 | 0.0141 | 0.0233 | 0.0000 | -0.0171 |
| RateMatched | 0.0152 | 0.0142 | 0.0200 | 0.0000 | -0.0411 |
| ActivityMatched | 0.0153 | 0.0141 | 0.0033 | 0.0000 | -0.0171 |
| SpectrumMatched | 0.0153 | 0.0160 | 0.0100 | 0.0000 | -0.0069 |

The spectrum null circularly shifts each cell train, preserving its temporal spectrum while ablating coordinated assembly timing. This is characterized as resting-state dynamics, not a biological Default Mode Network.
