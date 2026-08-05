# BINN-Hybrid terminal-credit hierarchy on the production event engine

- protocol: `binn-hybrid-production-diagnostic-v1-70406e787d80209d`
- schedule: **PILOT**
- seeds: 3
- depths: 1 through 8
- budgets: [60, 240]
- learning rates: [0.015, 0.035]
- frozen test examples per cell: 160
- event-forward contract: **PASS**
- all test weights unchanged: **true**
- scientific gate effect: **none**

> Development-only successor diagnostic. Canonical C1 protocol v2 remains a G2 failure and frozen H0 remains `HYBRID_NO_GO`. These seeds are disjoint from H0, the smooth diagnostic, and the unused held-out family. H1-H3 remain stopped.

## Mechanism contract

The evaluated forward path is the production event engine: forced source spikes, delayed CSR synaptic delivery, membrane charge, and hard k-WTA winners. The terminal teacher is an exact-gradient soft relaxation over the same layer topology and live production weights. It sees only the final label. Its gradient is checked by central finite differences. The two postsynaptic arms use actual production STDP eligibility captured from that hard event trace.

The privileged control receives true per-layer states and therefore has up to `depth` supervised corrections. It is an inadmissible solvability ceiling, not a matched learner.

## Best observed development D*

| arm | D* at lower-95 accuracy ≥ 0.65 |
|---|---:|
| production-existing-post-synaptic | 1 |
| production-least-squares-post-synaptic | 1 |
| production-direct-terminal | 1 |
| production-privileged-intermediate-target | 8 |
| production-shuffled-label | none |

## Best configuration by depth

| depth | arm | budget | learning rate | mean | lower 95% |
|---:|---|---:|---:|---:|---:|
| 1 | production-existing-post-synaptic | 240 | 0.0350 | 1.0000 | 1.0000 |
| 1 | production-least-squares-post-synaptic | 240 | 0.0350 | 1.0000 | 1.0000 |
| 1 | production-direct-terminal | 240 | 0.0350 | 1.0000 | 1.0000 |
| 2 | production-existing-post-synaptic | 240 | 0.0150 | 0.5917 | 0.5141 |
| 2 | production-least-squares-post-synaptic | 240 | 0.0350 | 0.6583 | 0.5391 |
| 2 | production-direct-terminal | 240 | 0.0150 | 0.3021 | 0.2793 |
| 3 | production-existing-post-synaptic | 240 | 0.0150 | 0.4542 | 0.3256 |
| 3 | production-least-squares-post-synaptic | 240 | 0.0150 | 0.4458 | 0.3185 |
| 3 | production-direct-terminal | 240 | 0.0150 | 0.2854 | 0.2707 |
| 4 | production-existing-post-synaptic | 240 | 0.0350 | 0.4188 | 0.3117 |
| 4 | production-least-squares-post-synaptic | 240 | 0.0150 | 0.3000 | 0.2646 |
| 4 | production-direct-terminal | 240 | 0.0350 | 0.2333 | 0.2106 |
| 5 | production-existing-post-synaptic | 240 | 0.0150 | 0.2792 | 0.2436 |
| 5 | production-least-squares-post-synaptic | 60 | 0.0150 | 0.2521 | 0.2305 |
| 5 | production-direct-terminal | 240 | 0.0350 | 0.2521 | 0.2439 |
| 6 | production-existing-post-synaptic | 60 | 0.0350 | 0.2875 | 0.2296 |
| 6 | production-least-squares-post-synaptic | 240 | 0.0350 | 0.3083 | 0.2629 |
| 6 | production-direct-terminal | 60 | 0.0350 | 0.2583 | 0.2335 |
| 7 | production-existing-post-synaptic | 240 | 0.0150 | 0.2583 | 0.2297 |
| 7 | production-least-squares-post-synaptic | 60 | 0.0150 | 0.2667 | 0.2117 |
| 7 | production-direct-terminal | 240 | 0.0350 | 0.2479 | 0.1948 |
| 8 | production-existing-post-synaptic | 60 | 0.0350 | 0.2542 | 0.2223 |
| 8 | production-least-squares-post-synaptic | 60 | 0.0150 | 0.2625 | 0.1828 |
| 8 | production-direct-terminal | 240 | 0.0350 | 0.2500 | 0.2429 |

## Paired effects at the direct-terminal optimum

| depth | direct - oracle mean | lower 95% | oracle - existing mean | lower 95% |
|---:|---:|---:|---:|---:|
| 1 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| 2 | -0.2625 | -0.3373 | -0.0271 | -0.1007 |
| 3 | -0.1604 | -0.2973 | -0.0083 | -0.0261 |
| 4 | -0.0792 | -0.1059 | -0.1062 | -0.1811 |
| 5 | 0.0021 | -0.0428 | 0.0083 | -0.0211 |
| 6 | -0.0375 | -0.0936 | 0.0083 | -0.0133 |
| 7 | 0.0229 | -0.0065 | -0.0021 | -0.0062 |
| 8 | -0.0042 | -0.0818 | 0.0292 | -0.0190 |

## Controls at maximum budget and best development rate

| depth | control | learning rate | mean | lower 95% |
|---:|---|---:|---:|---:|
| 1 | production-privileged-intermediate-target | 0.0350 | 1.0000 | 1.0000 |
| 1 | production-shuffled-label | 0.0350 | 0.0000 | 0.0000 |
| 2 | production-privileged-intermediate-target | 0.0350 | 1.0000 | 1.0000 |
| 2 | production-shuffled-label | 0.0350 | 0.2083 | 0.1516 |
| 3 | production-privileged-intermediate-target | 0.0350 | 1.0000 | 1.0000 |
| 3 | production-shuffled-label | 0.0350 | 0.2646 | 0.2483 |
| 4 | production-privileged-intermediate-target | 0.0350 | 1.0000 | 1.0000 |
| 4 | production-shuffled-label | 0.0350 | 0.2292 | 0.1472 |
| 5 | production-privileged-intermediate-target | 0.0350 | 1.0000 | 1.0000 |
| 5 | production-shuffled-label | 0.0350 | 0.2458 | 0.2191 |
| 6 | production-privileged-intermediate-target | 0.0350 | 1.0000 | 1.0000 |
| 6 | production-shuffled-label | 0.0350 | 0.2562 | 0.2492 |
| 7 | production-privileged-intermediate-target | 0.0350 | 1.0000 | 1.0000 |
| 7 | production-shuffled-label | 0.0350 | 0.2708 | 0.1929 |
| 8 | production-privileged-intermediate-target | 0.0350 | 1.0000 | 1.0000 |
| 8 | production-shuffled-label | 0.0150 | 0.2604 | 0.2426 |

## Mechanistic diagnostics at initialization

| depth | gradient norm | surrogate direct drop | surrogate rotated drop | event direct drop | event rotated drop | existing cosine | oracle cosine | existing MSE | oracle MSE | existing sign | oracle sign | eligibility support | target energy eligible |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 0.866267 | 0.000750 | -0.000250 | 0.000750 | -0.000250 | 0.4100 | 0.4100 | 0.00000002 | 0.00000002 | 0.6250 | 0.6250 | 0.0312 | 0.2228 |
| 2 | 0.433016 | 0.000187 | -0.000057 | 0.000189 | -0.000056 | 0.2483 | 0.2483 | 0.00000000 | 0.00000000 | 0.6927 | 0.6927 | 0.0312 | 0.0810 |
| 3 | 0.433020 | 0.000188 | -0.000057 | 0.000188 | -0.000057 | 0.1990 | 0.1990 | 0.00000000 | 0.00000000 | 0.7109 | 0.7109 | 0.0312 | 0.0524 |
| 4 | 0.433084 | 0.000188 | -0.000058 | 0.000188 | -0.000061 | 0.1929 | 0.1929 | 0.00000000 | 0.00000000 | 0.7259 | 0.7259 | 0.0312 | 0.0488 |
| 5 | 0.432999 | 0.000187 | -0.000060 | 0.000188 | -0.000059 | 0.2470 | 0.2470 | 0.00000000 | 0.00000000 | 0.7224 | 0.7224 | 0.0312 | 0.0801 |
| 6 | 0.432971 | 0.000187 | -0.000058 | 0.000188 | -0.000054 | 0.2050 | 0.2050 | 0.00000000 | 0.00000000 | 0.7279 | 0.7279 | 0.0312 | 0.0559 |
| 7 | 0.433096 | 0.000188 | -0.000056 | 0.000188 | -0.000056 | 0.1989 | 0.1989 | 0.00000000 | 0.00000000 | 0.7344 | 0.7344 | 0.0312 | 0.0522 |
| 8 | 0.433008 | 0.000187 | -0.000060 | 0.000188 | -0.000061 | 0.2231 | 0.2231 | 0.00000000 | 0.00000000 | 0.7396 | 0.7396 | 0.0312 | 0.0663 |

## Interpretation limits

- Best configurations are selected on these development seeds; D* values are descriptive, not confirmatory.
- Hard k-WTA is nondifferentiable. The terminal teacher is a disclosed differentiable relaxation, not a derivative of the discontinuous winner operation.
- The production hierarchy reproduces only if ordering and controls agree; exact D* equality with the smooth diagnostic is not required.
- Raw seed-level sweep and mechanism rows are emitted beside this report.
