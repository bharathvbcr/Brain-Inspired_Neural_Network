# BINN-Hybrid H0/H1 feasibility

- protocol: `binn-hybrid-h0-v1-b5e3d84c9d3abf47`
- schedule: PILOT
- decision: **HYBRID_NO_GO**
- selected granularity: none
- teacher D*: 2
- student D*: none
- no test updates: **true**
- scientific claim allowed: **false**

> This protocol uses production CSR/weight storage with a smooth matched sparse forward. It validates teacher math, factorization, artifact freezing, and teacher-free execution, but it does not replace a production event-engine C1/C3 result. H2/H3 remain stopped.

Thresholds: C1 accuracy 0.650; C3 accuracy 0.650; teacher D* ≥ 6; student D* ≥ 6; normalized gap target 0.500.

| benchmark | depth | arm | mean accuracy | variance | lower 95% |
|---|---:|---|---:|---:|---:|
| c1-terminal-surrogate | — | existing-post-synaptic | 0.6729 | 0.017279 | 0.5242 |
| c1-terminal-surrogate | — | least-squares-post-synaptic | 0.9979 | 0.000013 | 0.9938 |
| c1-terminal-surrogate | — | direct-per-synapse | 0.9979 | 0.000013 | 0.9938 |
| c1-terminal-surrogate | — | distilled-student | 0.6750 | 0.016094 | 0.5314 |
| c3-terminal-composition | 1 | existing-post-synaptic | 0.0000 | 0.000000 | 0.0000 |
| c3-terminal-composition | 1 | least-squares-post-synaptic | 0.7917 | 0.002591 | 0.7341 |
| c3-terminal-composition | 1 | direct-per-synapse | 0.7917 | 0.002591 | 0.7341 |
| c3-terminal-composition | 1 | distilled-student | 0.0000 | 0.000000 | 0.0000 |
| c3-terminal-composition | 2 | existing-post-synaptic | 0.1396 | 0.000872 | 0.1062 |
| c3-terminal-composition | 2 | least-squares-post-synaptic | 0.9333 | 0.001810 | 0.8852 |
| c3-terminal-composition | 2 | direct-per-synapse | 1.0000 | 0.000000 | 1.0000 |
| c3-terminal-composition | 2 | distilled-student | 0.1396 | 0.000872 | 0.1062 |
| c3-terminal-composition | 3 | existing-post-synaptic | 0.3625 | 0.001406 | 0.3201 |
| c3-terminal-composition | 3 | least-squares-post-synaptic | 0.6208 | 0.007279 | 0.5243 |
| c3-terminal-composition | 3 | direct-per-synapse | 0.6479 | 0.002552 | 0.5907 |
| c3-terminal-composition | 3 | distilled-student | 0.2667 | 0.002904 | 0.2057 |
| c3-terminal-composition | 4 | existing-post-synaptic | 0.2604 | 0.001966 | 0.2102 |
| c3-terminal-composition | 4 | least-squares-post-synaptic | 0.3563 | 0.002969 | 0.2946 |
| c3-terminal-composition | 4 | direct-per-synapse | 0.3854 | 0.008060 | 0.2838 |
| c3-terminal-composition | 4 | distilled-student | 0.2604 | 0.001966 | 0.2102 |
| c3-terminal-composition | 5 | existing-post-synaptic | 0.1854 | 0.000638 | 0.1568 |
| c3-terminal-composition | 5 | least-squares-post-synaptic | 0.2354 | 0.000404 | 0.2127 |
| c3-terminal-composition | 5 | direct-per-synapse | 0.2292 | 0.000638 | 0.2006 |
| c3-terminal-composition | 5 | distilled-student | 0.1854 | 0.000638 | 0.1568 |
| c3-terminal-composition | 6 | existing-post-synaptic | 0.2562 | 0.000820 | 0.2238 |
| c3-terminal-composition | 6 | least-squares-post-synaptic | 0.2542 | 0.001302 | 0.2133 |
| c3-terminal-composition | 6 | direct-per-synapse | 0.2729 | 0.000951 | 0.2380 |
| c3-terminal-composition | 6 | distilled-student | 0.2604 | 0.000951 | 0.2255 |
| c3-terminal-composition | 7 | existing-post-synaptic | 0.2333 | 0.002591 | 0.1757 |
| c3-terminal-composition | 7 | least-squares-post-synaptic | 0.2313 | 0.000156 | 0.2171 |
| c3-terminal-composition | 7 | direct-per-synapse | 0.2500 | 0.001914 | 0.2005 |
| c3-terminal-composition | 7 | distilled-student | 0.2292 | 0.000404 | 0.2064 |
| c3-terminal-composition | 8 | existing-post-synaptic | 0.2458 | 0.002552 | 0.1887 |
| c3-terminal-composition | 8 | least-squares-post-synaptic | 0.2812 | 0.001211 | 0.2419 |
| c3-terminal-composition | 8 | direct-per-synapse | 0.2583 | 0.001654 | 0.2123 |
| c3-terminal-composition | 8 | distilled-student | 0.2500 | 0.002852 | 0.1896 |

| benchmark | depth | existing cosine | oracle cosine | existing sign | oracle sign |
|---|---:|---:|---:|---:|---:|
| c1-terminal-surrogate | — | 0.0128 | 1.0000 | 0.5040 | 1.0000 |
| c3-terminal-composition | 1 | 0.8271 | 1.0000 | 0.2859 | 0.7418 |
| c3-terminal-composition | 2 | 0.6291 | 0.9372 | 0.5140 | 0.7934 |
| c3-terminal-composition | 3 | 0.6507 | 0.8765 | 0.5364 | 0.8211 |
| c3-terminal-composition | 4 | 0.6014 | 0.8674 | 0.6401 | 0.8413 |
| c3-terminal-composition | 5 | 0.5615 | 0.8360 | 0.7219 | 0.8439 |
| c3-terminal-composition | 6 | 0.5748 | 0.8695 | 0.6931 | 0.8682 |
| c3-terminal-composition | 7 | 0.5626 | 0.8427 | 0.6694 | 0.8565 |
| c3-terminal-composition | 8 | 0.3509 | 0.7646 | 0.6340 | 0.8084 |
