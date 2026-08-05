# BINN-Hybrid terminal-credit hierarchy on the production event engine

- protocol: `binn-hybrid-production-diagnostic-v2-8cd6fea27aa912c2`
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

The evaluated forward path is the production event engine: forced source spikes, delayed CSR synaptic delivery, membrane charge, and hard k-WTA winners. One recurrent transition graph is reused at every composition step, matching the shared transition parameters of the smooth diagnostic. The terminal teacher is an exact-gradient soft relaxation over that same topology and the live production weights. It sees only the final label. Its gradient is checked by central finite differences. The two postsynaptic arms use actual production STDP eligibility captured from that hard event trace.

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
| 2 | production-existing-post-synaptic | 240 | 0.0150 | 0.5437 | 0.4622 |
| 2 | production-least-squares-post-synaptic | 60 | 0.0150 | 0.4938 | 0.4815 |
| 2 | production-direct-terminal | 240 | 0.0150 | 0.2500 | 0.2429 |
| 3 | production-existing-post-synaptic | 240 | 0.0350 | 0.3667 | 0.1806 |
| 3 | production-least-squares-post-synaptic | 240 | 0.0350 | 0.2979 | 0.2519 |
| 3 | production-direct-terminal | 240 | 0.0350 | 0.2896 | 0.2749 |
| 4 | production-existing-post-synaptic | 60 | 0.0350 | 0.5021 | 0.4858 |
| 4 | production-least-squares-post-synaptic | 240 | 0.0350 | 0.4813 | 0.4164 |
| 4 | production-direct-terminal | 240 | 0.0350 | 0.2708 | 0.2197 |
| 5 | production-existing-post-synaptic | 240 | 0.0150 | 0.2854 | 0.2560 |
| 5 | production-least-squares-post-synaptic | 240 | 0.0350 | 0.3896 | 0.2289 |
| 5 | production-direct-terminal | 240 | 0.0350 | 0.2396 | 0.2314 |
| 6 | production-existing-post-synaptic | 240 | 0.0350 | 0.4938 | 0.4682 |
| 6 | production-least-squares-post-synaptic | 240 | 0.0350 | 0.3958 | 0.2894 |
| 6 | production-direct-terminal | 60 | 0.0350 | 0.2604 | 0.2318 |
| 7 | production-existing-post-synaptic | 60 | 0.0150 | 0.3229 | 0.3051 |
| 7 | production-least-squares-post-synaptic | 240 | 0.0350 | 0.2958 | 0.2009 |
| 7 | production-direct-terminal | 240 | 0.0350 | 0.2479 | 0.1948 |
| 8 | production-existing-post-synaptic | 240 | 0.0150 | 0.4792 | 0.4497 |
| 8 | production-least-squares-post-synaptic | 240 | 0.0350 | 0.4375 | 0.3506 |
| 8 | production-direct-terminal | 240 | 0.0350 | 0.2500 | 0.2429 |

## Paired effects at the direct-terminal optimum

| depth | direct - oracle mean | lower 95% | oracle - existing mean | lower 95% |
|---:|---:|---:|---:|---:|
| 1 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| 2 | -0.2479 | -0.3825 | -0.0458 | -0.2267 |
| 3 | -0.0083 | -0.0685 | -0.0687 | -0.2416 |
| 4 | -0.2104 | -0.3178 | -0.0188 | -0.1021 |
| 5 | -0.1500 | -0.3121 | 0.1146 | -0.0237 |
| 6 | -0.0938 | -0.2373 | -0.1000 | -0.3158 |
| 7 | -0.0479 | -0.0911 | -0.0521 | -0.2143 |
| 8 | -0.1875 | -0.2735 | -0.0521 | -0.1551 |

## Controls at maximum budget and best development rate

| depth | control | learning rate | mean | lower 95% |
|---:|---|---:|---:|---:|
| 1 | production-privileged-intermediate-target | 0.0350 | 1.0000 | 1.0000 |
| 1 | production-shuffled-label | 0.0350 | 0.0000 | 0.0000 |
| 2 | production-privileged-intermediate-target | 0.0350 | 1.0000 | 1.0000 |
| 2 | production-shuffled-label | 0.0350 | 0.2146 | 0.1474 |
| 3 | production-privileged-intermediate-target | 0.0350 | 1.0000 | 1.0000 |
| 3 | production-shuffled-label | 0.0350 | 0.2479 | 0.2047 |
| 4 | production-privileged-intermediate-target | 0.0350 | 1.0000 | 1.0000 |
| 4 | production-shuffled-label | 0.0350 | 0.2562 | 0.1984 |
| 5 | production-privileged-intermediate-target | 0.0350 | 1.0000 | 1.0000 |
| 5 | production-shuffled-label | 0.0150 | 0.2167 | 0.2019 |
| 6 | production-privileged-intermediate-target | 0.0350 | 1.0000 | 1.0000 |
| 6 | production-shuffled-label | 0.0350 | 0.2708 | 0.2481 |
| 7 | production-privileged-intermediate-target | 0.0350 | 1.0000 | 1.0000 |
| 7 | production-shuffled-label | 0.0350 | 0.2708 | 0.1929 |
| 8 | production-privileged-intermediate-target | 0.0350 | 1.0000 | 1.0000 |
| 8 | production-shuffled-label | 0.0350 | 0.2667 | 0.2372 |

## Mechanistic diagnostics at initialization

| depth | gradient norm | surrogate direct drop | surrogate rotated drop | event direct drop | event rotated drop | existing cosine | oracle cosine | existing MSE | oracle MSE | existing sign | oracle sign | eligibility support | target energy eligible |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 0.866441 | 0.000751 | -0.000250 | 0.000751 | -0.000250 | 0.3621 | 0.3621 | 0.00000002 | 0.00000002 | 0.3438 | 0.7422 | 0.0312 | 0.1674 |
| 2 | 0.433137 | 0.000188 | -0.000060 | 0.000255 | -0.000061 | 0.1067 | 0.2648 | 0.00000002 | 0.00000001 | 0.6237 | 0.5781 | 0.1022 | 0.1573 |
| 3 | 0.433001 | 0.000187 | -0.000053 | 0.000188 | 0.000039 | 0.1241 | 0.2248 | 0.00000002 | 0.00000001 | 0.6491 | 0.5671 | 0.1641 | 0.2269 |
| 4 | 0.433081 | 0.000188 | -0.000048 | 0.000188 | -0.000108 | 0.0865 | 0.2735 | 0.00000003 | 0.00000001 | 0.7038 | 0.5267 | 0.2552 | 0.2842 |
| 5 | 0.433040 | 0.000187 | -0.000054 | 0.000188 | -0.000176 | 0.1070 | 0.2140 | 0.00000003 | 0.00000001 | 0.6699 | 0.5710 | 0.2669 | 0.2661 |
| 6 | 0.433040 | 0.000188 | -0.000049 | 0.000188 | -0.000050 | 0.1106 | 0.2967 | 0.00000004 | 0.00000001 | 0.6999 | 0.5384 | 0.3379 | 0.4066 |
| 7 | 0.432956 | 0.000187 | -0.000043 | 0.000221 | -0.000332 | 0.1236 | 0.2655 | 0.00000005 | 0.00000001 | 0.6634 | 0.5514 | 0.3906 | 0.4220 |
| 8 | 0.432829 | 0.000187 | -0.000051 | 0.000188 | -0.000060 | 0.1734 | 0.3054 | 0.00000006 | 0.00000001 | 0.6582 | 0.5254 | 0.4453 | 0.5156 |

## Interpretation limits

- Best configurations are selected on these development seeds; D* values are descriptive, not confirmatory.
- Hard k-WTA is nondifferentiable. The terminal teacher is a disclosed differentiable relaxation, not a derivative of the discontinuous winner operation.
- The production hierarchy reproduces only if ordering and controls agree; exact D* equality with the smooth diagnostic is not required.
- Raw seed-level sweep and mechanism rows are emitted beside this report.
