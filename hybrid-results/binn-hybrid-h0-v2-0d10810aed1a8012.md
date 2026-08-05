# BINN-Hybrid H0/H1 feasibility

- protocol: `binn-hybrid-h0-v2-0d10810aed1a8012`
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
| c1-terminal-surrogate | — | existing-post-synaptic | 0.5333 | 0.043529 | 0.2972 |
| c1-terminal-surrogate | — | least-squares-post-synaptic | 0.9750 | 0.000469 | 0.9505 |
| c1-terminal-surrogate | — | direct-per-synapse | 0.9750 | 0.000469 | 0.9505 |
| c3-terminal-composition | 1 | existing-post-synaptic | 0.0000 | 0.000000 | 0.0000 |
| c3-terminal-composition | 1 | least-squares-post-synaptic | 0.9479 | 0.008138 | 0.8458 |
| c3-terminal-composition | 1 | direct-per-synapse | 0.9479 | 0.008138 | 0.8458 |
| c3-terminal-composition | 2 | existing-post-synaptic | 0.1250 | 0.001211 | 0.0856 |
| c3-terminal-composition | 2 | least-squares-post-synaptic | 0.9167 | 0.000833 | 0.8840 |
| c3-terminal-composition | 2 | direct-per-synapse | 1.0000 | 0.000000 | 1.0000 |
| c3-terminal-composition | 3 | existing-post-synaptic | 0.3812 | 0.001211 | 0.3419 |
| c3-terminal-composition | 3 | least-squares-post-synaptic | 0.6521 | 0.003138 | 0.5887 |
| c3-terminal-composition | 3 | direct-per-synapse | 0.6687 | 0.006133 | 0.5801 |
| c3-terminal-composition | 4 | existing-post-synaptic | 0.3000 | 0.001406 | 0.2576 |
| c3-terminal-composition | 4 | least-squares-post-synaptic | 0.3292 | 0.000247 | 0.3114 |
| c3-terminal-composition | 4 | direct-per-synapse | 0.3250 | 0.000742 | 0.2942 |
| c3-terminal-composition | 5 | existing-post-synaptic | 0.1813 | 0.000977 | 0.1459 |
| c3-terminal-composition | 5 | least-squares-post-synaptic | 0.2479 | 0.002826 | 0.1878 |
| c3-terminal-composition | 5 | direct-per-synapse | 0.2875 | 0.001211 | 0.2481 |
| c3-terminal-composition | 6 | existing-post-synaptic | 0.2562 | 0.002617 | 0.1984 |
| c3-terminal-composition | 6 | least-squares-post-synaptic | 0.2375 | 0.001094 | 0.2001 |
| c3-terminal-composition | 6 | direct-per-synapse | 0.2792 | 0.003177 | 0.2154 |
| c3-terminal-composition | 7 | existing-post-synaptic | 0.2875 | 0.001523 | 0.2433 |
| c3-terminal-composition | 7 | least-squares-post-synaptic | 0.2979 | 0.001185 | 0.2590 |
| c3-terminal-composition | 7 | direct-per-synapse | 0.2833 | 0.001185 | 0.2444 |
| c3-terminal-composition | 8 | existing-post-synaptic | 0.2375 | 0.003281 | 0.1727 |
| c3-terminal-composition | 8 | least-squares-post-synaptic | 0.2583 | 0.000247 | 0.2405 |
| c3-terminal-composition | 8 | direct-per-synapse | 0.2708 | 0.000677 | 0.2414 |

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
