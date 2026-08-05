# BINN-Hybrid terminal-credit hierarchy on the production event engine

- protocol: `binn-hybrid-production-diagnostic-v3-f72033fbf6906b99`
- schedule: **FULL DEVELOPMENT DIAGNOSTIC**
- seeds: 20
- depths: 1 through 8
- budgets: [120, 480, 1920, 7680]
- learning rates: [0.002, 0.005, 0.015, 0.035, 0.07]
- frozen test examples per cell: 1000
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
| production-existing-post-synaptic | 1 |
| production-least-squares-post-synaptic | 1 |
| production-direct-terminal | 1 |
| production-privileged-intermediate-target | 8 |
| production-shuffled-label | none |

## Best configuration by depth

| depth | arm | budget | learning rate | mean | lower 95% |
|---:|---|---:|---:|---:|---:|
| 1 | production-existing-post-synaptic | 7680 | 0.0700 | 1.0000 | 1.0000 |
| 1 | production-least-squares-post-synaptic | 7680 | 0.0700 | 1.0000 | 1.0000 |
| 1 | production-direct-terminal | 7680 | 0.0700 | 1.0000 | 1.0000 |
| 2 | production-existing-post-synaptic | 7680 | 0.0050 | 0.4998 | 0.4934 |
| 2 | production-least-squares-post-synaptic | 480 | 0.0700 | 0.5217 | 0.5065 |
| 2 | production-direct-terminal | 1920 | 0.0700 | 0.6230 | 0.6172 |
| 3 | production-existing-post-synaptic | 7680 | 0.0350 | 0.5597 | 0.4314 |
| 3 | production-least-squares-post-synaptic | 7680 | 0.0700 | 0.0843 | 0.0552 |
| 3 | production-direct-terminal | 7680 | 0.0350 | 0.3509 | 0.3068 |
| 4 | production-existing-post-synaptic | 7680 | 0.0020 | 0.5021 | 0.4938 |
| 4 | production-least-squares-post-synaptic | 7680 | 0.0150 | 0.5115 | 0.5032 |
| 4 | production-direct-terminal | 1920 | 0.0150 | 0.5074 | 0.4992 |
| 5 | production-existing-post-synaptic | 7680 | 0.0020 | 0.3320 | 0.2810 |
| 5 | production-least-squares-post-synaptic | 7680 | 0.0700 | 0.0000 | 0.0000 |
| 5 | production-direct-terminal | 7680 | 0.0700 | 0.2073 | 0.1600 |
| 6 | production-existing-post-synaptic | 7680 | 0.0700 | 0.4998 | 0.4933 |
| 6 | production-least-squares-post-synaptic | 120 | 0.0700 | 0.5009 | 0.4954 |
| 6 | production-direct-terminal | 7680 | 0.0150 | 0.5007 | 0.4951 |
| 7 | production-existing-post-synaptic | 480 | 0.0350 | 0.2994 | 0.2635 |
| 7 | production-least-squares-post-synaptic | 7680 | 0.0700 | 0.0000 | 0.0000 |
| 7 | production-direct-terminal | 1920 | 0.0700 | 0.2938 | 0.2050 |
| 8 | production-existing-post-synaptic | 7680 | 0.0020 | 0.4951 | 0.4887 |
| 8 | production-least-squares-post-synaptic | 7680 | 0.0050 | 0.4927 | 0.4860 |
| 8 | production-direct-terminal | 480 | 0.0350 | 0.4971 | 0.4887 |

## Paired effects at the direct-terminal optimum

| depth | direct - oracle mean | lower 95% | oracle - existing mean | lower 95% |
|---:|---:|---:|---:|---:|
| 1 | 0.0000 | 0.0000 | 0.0000 | 0.0000 |
| 2 | 0.1189 | 0.1075 | 0.0006 | -0.0108 |
| 3 | 0.3508 | 0.3068 | -0.5597 | -0.6879 |
| 4 | 0.0052 | -0.0025 | 0.0082 | -0.0063 |
| 5 | 0.2074 | 0.1600 | 0.0000 | 0.0000 |
| 6 | 0.0005 | -0.0067 | 0.0725 | 0.0214 |
| 7 | 0.2938 | 0.2050 | 0.0000 | 0.0000 |
| 8 | 0.0044 | -0.0039 | 0.1086 | 0.0568 |

## Controls at maximum budget and best development rate

| depth | control | learning rate | mean | lower 95% |
|---:|---|---:|---:|---:|
| 1 | production-privileged-intermediate-target | 0.0700 | 1.0000 | 1.0000 |
| 1 | production-shuffled-label | 0.0700 | 0.0000 | 0.0000 |
| 2 | production-privileged-intermediate-target | 0.0700 | 1.0000 | 1.0000 |
| 2 | production-shuffled-label | 0.0350 | 0.2923 | 0.2480 |
| 3 | production-privileged-intermediate-target | 0.0700 | 1.0000 | 1.0000 |
| 3 | production-shuffled-label | 0.0350 | 0.0000 | 0.0000 |
| 4 | production-privileged-intermediate-target | 0.0700 | 1.0000 | 1.0000 |
| 4 | production-shuffled-label | 0.0020 | 0.4906 | 0.4739 |
| 5 | production-privileged-intermediate-target | 0.0700 | 1.0000 | 1.0000 |
| 5 | production-shuffled-label | 0.0700 | 0.2262 | 0.1859 |
| 6 | production-privileged-intermediate-target | 0.0700 | 1.0000 | 1.0000 |
| 6 | production-shuffled-label | 0.0150 | 0.5023 | 0.4965 |
| 7 | production-privileged-intermediate-target | 0.0700 | 1.0000 | 1.0000 |
| 7 | production-shuffled-label | 0.0700 | 0.1365 | 0.0860 |
| 8 | production-privileged-intermediate-target | 0.0700 | 1.0000 | 1.0000 |
| 8 | production-shuffled-label | 0.0350 | 0.4927 | 0.4860 |

## Mechanistic diagnostics at initialization

| depth | gradient norm | surrogate direct drop | surrogate rotated drop | event direct drop | event rotated drop | existing cosine | oracle cosine | existing MSE | oracle MSE | existing sign | oracle sign | eligibility support | target energy eligible |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 0.983838 | 0.000968 | -0.000423 | 0.000968 | -0.000423 | 0.4831 | 0.4831 | 0.00000002 | 0.00000002 | 0.3438 | 0.7188 | 0.0312 | 0.2334 |
| 2 | 1.350328 | 0.001984 | -0.000297 | 0.000992 | -0.000148 | 0.5870 | 0.5870 | 0.00000006 | 0.00000004 | 0.4861 | 0.6408 | 0.0473 | 0.4924 |
| 3 | 2.393650 | 0.005829 | -0.002135 | 0.001951 | -0.000717 | 0.4187 | 0.4187 | 0.00000020 | 0.00000015 | 0.5563 | 0.5545 | 0.0544 | 0.2334 |
| 4 | 2.486943 | 0.006449 | -0.000994 | 0.001620 | -0.000250 | 0.5768 | 0.5768 | 0.00000030 | 0.00000014 | 0.5890 | 0.5231 | 0.0588 | 0.4823 |
| 5 | 3.798982 | 0.014508 | -0.005357 | 0.002891 | -0.001053 | 0.4199 | 0.4199 | 0.00000068 | 0.00000037 | 0.6036 | 0.5145 | 0.0606 | 0.2334 |
| 6 | 3.622921 | 0.013908 | -0.002119 | 0.002337 | -0.000348 | 0.5944 | 0.5944 | 0.00000097 | 0.00000031 | 0.6148 | 0.4793 | 0.0615 | 0.4915 |
| 7 | 5.200757 | 0.026976 | -0.010242 | 0.003872 | -0.001442 | 0.4302 | 0.4302 | 0.00000169 | 0.00000069 | 0.6202 | 0.5026 | 0.0621 | 0.2335 |
| 8 | 4.735127 | 0.023642 | -0.003699 | 0.002983 | -0.000460 | 0.6070 | 0.6070 | 0.00000227 | 0.00000052 | 0.6283 | 0.4696 | 0.0623 | 0.4952 |

## Interpretation limits

- Best configurations are selected on these development seeds; D* values are descriptive, not confirmatory.
- Hard k-WTA is nondifferentiable. The terminal teacher is a disclosed differentiable relaxation, not a derivative of the discontinuous winner operation.
- The production hierarchy reproduces only if ordering and controls agree; exact D* equality with the smooth diagnostic is not required.
- Raw seed-level sweep and mechanism rows are emitted beside this report.
