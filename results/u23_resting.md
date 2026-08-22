# U23 — resting-state dynamics

> **Provenance, 2026-08-22 — the `RateMatched` row changed, and only that row.**
> `0.0140 | 0.0145 | 0.0090 | 0.0000 | -0.0188` → `0.0141 | 0.0146 | 0.0090 |
> 0.0000 | -0.0212`.
>
> The rate-matched null was not rate-matched. It drew `total` uniform
> `(tick, cell)` pairs and then deduplicated, so every within-tick collision
> destroyed a spike; the two other nulls already redrew on collision, which is
> why they matched the observed 0.0141 and this one read 0.0140. The shortfall
> was in this table all along.
>
> The null now redraws, like its siblings, and matches the observed rate exactly.
> Nothing else in the pipeline changed: the observed, ActivityMatched and
> SpectrumMatched rows are byte-identical, and the row's own reactivation and
> transition figures are unchanged.
>
> Any earlier comparison against the RateMatched row was against a null carrying
> slightly less activity than it claimed. See
> `results/FINDING_2026-08-22_A_SWEEP_OF_BINN_PROPER.md` section 1.1.


**Exploratory post-G2 override.** `c1-118207fbc3eaba53` remains a FAIL.

- schedule: scientific
- ticks: 3000
- stimulus-free background is unlabeled endogenous noise
- causal consolidation proxy, structured rest: 0.3913
- causal ablation, spectrum-matched rest: 0.3847

| condition | mean activity | metastability | reactivation | transitions | lag-1 autocorrelation |
|---|---:|---:|---:|---:|---:|
| observed | 0.0141 | 0.0146 | 0.0120 | 0.0003 | -0.0169 |
| RateMatched | 0.0141 | 0.0146 | 0.0090 | 0.0000 | -0.0212 |
| ActivityMatched | 0.0141 | 0.0146 | 0.0087 | 0.0007 | -0.0169 |
| SpectrumMatched | 0.0141 | 0.0148 | 0.0087 | 0.0000 | -0.0315 |

The spectrum null circularly shifts each cell train, preserving its temporal spectrum while ablating coordinated assembly timing. This is characterized as resting-state dynamics, not a biological Default Mode Network.
