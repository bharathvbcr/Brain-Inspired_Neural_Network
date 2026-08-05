# BINN-Hybrid terminal-credit hierarchy on the production event engine

- protocol: `binn-hybrid-production-diagnostic-v3-2998fa4d999aa39d`
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

The evaluated forward path is the production event engine: forced source spikes, delayed CSR synaptic delivery, membrane charge, and hard k-WTA winners. One recurrent transition graph is reused at every composition step, matching the shared transition parameters of the smooth diagnostic. Its identity residual is delivered as a real weighted external event. The terminal teacher is the exact gradient of the original residual relaxation over that same topology and the live production weights. It sees only the final label and its gradient is checked by central finite differences. The two postsynaptic arms use actual production STDP eligibility captured from that hard event trace.

The privileged control receives true per-layer states and therefore has up to `depth` supervised corrections. It is an inadmissible solvability ceiling, not a matched learner.

## Best observed development D*

| arm | D* at lower-95 accuracy ≥ 0.65 |
|---|---:|
| production-existing-post-synaptic | none |
| production-least-squares-post-synaptic | none |
| production-direct-terminal | 1 |
| production-privileged-intermediate-target | 8 |
| production-shuffled-label | none |

## Best configuration by depth

| depth | arm | budget | learning rate | mean | lower 95% |
|---:|---|---:|---:|---:|---:|
| 1 | production-existing-post-synaptic | 240 | 0.0350 | 0.0000 | 0.0000 |
| 1 | production-least-squares-post-synaptic | 240 | 0.0350 | 0.0000 | 0.0000 |
| 1 | production-direct-terminal | 240 | 0.0350 | 0.9688 | 0.9075 |
| 2 | production-existing-post-synaptic | 240 | 0.0350 | 0.5854 | 0.5261 |
| 2 | production-least-squares-post-synaptic | 240 | 0.0350 | 0.4979 | 0.4571 |
| 2 | production-direct-terminal | 240 | 0.0350 | 0.5250 | 0.5179 |
| 3 | production-existing-post-synaptic | 240 | 0.0150 | 0.0000 | 0.0000 |
| 3 | production-least-squares-post-synaptic | 240 | 0.0350 | 0.0000 | 0.0000 |
| 3 | production-direct-terminal | 240 | 0.0350 | 0.0729 | 0.0007 |
| 4 | production-existing-post-synaptic | 240 | 0.0350 | 0.4854 | 0.4560 |
| 4 | production-least-squares-post-synaptic | 240 | 0.0350 | 0.4854 | 0.4560 |
| 4 | production-direct-terminal | 240 | 0.0350 | 0.4854 | 0.4560 |
| 5 | production-existing-post-synaptic | 240 | 0.0350 | 0.3125 | 0.1628 |
| 5 | production-least-squares-post-synaptic | 240 | 0.0350 | 0.0000 | 0.0000 |
| 5 | production-direct-terminal | 240 | 0.0350 | 0.0000 | 0.0000 |
| 6 | production-existing-post-synaptic | 240 | 0.0150 | 0.4771 | 0.4503 |
| 6 | production-least-squares-post-synaptic | 240 | 0.0350 | 0.4771 | 0.4503 |
| 6 | production-direct-terminal | 240 | 0.0350 | 0.4771 | 0.4503 |
| 7 | production-existing-post-synaptic | 240 | 0.0350 | 0.2333 | 0.2292 |
| 7 | production-least-squares-post-synaptic | 240 | 0.0350 | 0.0000 | 0.0000 |
| 7 | production-direct-terminal | 240 | 0.0350 | 0.0000 | 0.0000 |
| 8 | production-existing-post-synaptic | 240 | 0.0150 | 0.5167 | 0.4811 |
| 8 | production-least-squares-post-synaptic | 240 | 0.0350 | 0.5062 | 0.4632 |
| 8 | production-direct-terminal | 240 | 0.0350 | 0.5062 | 0.4632 |

## Paired effects at the direct-terminal optimum

| depth | direct - oracle mean | lower 95% | oracle - existing mean | lower 95% |
|---:|---:|---:|---:|---:|
| 1 | 0.9688 | 0.9075 | 0.0000 | 0.0000 |
| 2 | 0.0271 | -0.0144 | -0.0875 | -0.1062 |
| 3 | 0.0729 | 0.0007 | -0.0729 | -0.1485 |
| 4 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| 5 | 0.0000 | 0.0000 | -0.3125 | -0.4622 |
| 6 | 0.0000 | 0.0000 | -0.0104 | -0.0714 |
| 7 | 0.0000 | 0.0000 | -0.2333 | -0.2374 |
| 8 | 0.0000 | 0.0000 | 0.1500 | -0.0812 |

## Controls at maximum budget and best development rate

| depth | control | learning rate | mean | lower 95% |
|---:|---|---:|---:|---:|
| 1 | production-privileged-intermediate-target | 0.0350 | 0.9688 | 0.9075 |
| 1 | production-shuffled-label | 0.0350 | 0.0000 | 0.0000 |
| 2 | production-privileged-intermediate-target | 0.0350 | 1.0000 | 1.0000 |
| 2 | production-shuffled-label | 0.0150 | 0.4979 | 0.4571 |
| 3 | production-privileged-intermediate-target | 0.0350 | 1.0000 | 1.0000 |
| 3 | production-shuffled-label | 0.0350 | 0.0000 | 0.0000 |
| 4 | production-privileged-intermediate-target | 0.0350 | 1.0000 | 1.0000 |
| 4 | production-shuffled-label | 0.0350 | 0.4854 | 0.4560 |
| 5 | production-privileged-intermediate-target | 0.0350 | 1.0000 | 1.0000 |
| 5 | production-shuffled-label | 0.0350 | 0.0000 | 0.0000 |
| 6 | production-privileged-intermediate-target | 0.0350 | 1.0000 | 1.0000 |
| 6 | production-shuffled-label | 0.0350 | 0.4771 | 0.4503 |
| 7 | production-privileged-intermediate-target | 0.0350 | 1.0000 | 1.0000 |
| 7 | production-shuffled-label | 0.0350 | 0.0000 | 0.0000 |
| 8 | production-privileged-intermediate-target | 0.0350 | 1.0000 | 1.0000 |
| 8 | production-shuffled-label | 0.0350 | 0.5062 | 0.4632 |

## Mechanistic diagnostics at initialization

| depth | gradient norm | surrogate direct drop | surrogate rotated drop | event direct drop | event rotated drop | existing cosine | oracle cosine | existing MSE | oracle MSE | existing sign | oracle sign | eligibility support | target energy eligible |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 0.984206 | 0.000969 | -0.000423 | 0.000969 | -0.000423 | 0.4832 | 0.4832 | 0.00000002 | 0.00000002 | 0.3438 | 0.7188 | 0.0312 | 0.2335 |
| 2 | 1.380605 | 0.002085 | -0.000217 | 0.001043 | -0.000105 | 0.5686 | 0.5686 | 0.00000006 | 0.00000005 | 0.4421 | 0.6426 | 0.0475 | 0.4698 |
| 3 | 2.422172 | 0.005977 | -0.002021 | 0.002059 | -0.000672 | 0.4192 | 0.4192 | 0.00000020 | 0.00000015 | 0.5443 | 0.5599 | 0.0534 | 0.2338 |
| 4 | 2.549469 | 0.006738 | -0.000929 | 0.001621 | -0.000256 | 0.5533 | 0.5533 | 0.00000031 | 0.00000015 | 0.5807 | 0.5117 | 0.0599 | 0.4274 |
| 5 | 3.795864 | 0.014482 | -0.005752 | 0.002956 | -0.001131 | 0.4101 | 0.4101 | 0.00000071 | 0.00000037 | 0.5091 | 0.5195 | 0.0605 | 0.2365 |
| 6 | 3.612751 | 0.013810 | -0.002061 | 0.002392 | -0.000352 | 0.5925 | 0.5925 | 0.00000096 | 0.00000031 | 0.5697 | 0.4928 | 0.0618 | 0.4911 |
| 7 | 5.357408 | 0.028706 | -0.009373 | 0.003917 | -0.001299 | 0.4152 | 0.4152 | 0.00000164 | 0.00000075 | 0.5501 | 0.5111 | 0.0612 | 0.2324 |
| 8 | 4.813274 | 0.024525 | -0.003962 | 0.002953 | -0.000508 | 0.6047 | 0.6047 | 0.00000222 | 0.00000054 | 0.5716 | 0.4909 | 0.0618 | 0.4798 |

## Interpretation limits

- Best configurations are selected on these development seeds; D* values are descriptive, not confirmatory.
- Hard k-WTA is nondifferentiable. The terminal teacher is a disclosed differentiable relaxation, not a derivative of the discontinuous winner operation.
- The production hierarchy reproduces only if ordering and controls agree; exact D* equality with the smooth diagnostic is not required.
- Raw seed-level sweep and mechanism rows are emitted beside this report.
