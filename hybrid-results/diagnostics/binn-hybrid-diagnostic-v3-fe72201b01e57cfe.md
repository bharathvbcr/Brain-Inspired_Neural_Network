# BINN-Hybrid diagnostic robustness study

- protocol: `binn-hybrid-diagnostic-v3-fe72201b01e57cfe`
- schedule: **FULL DIAGNOSTIC**
- seeds: 20
- budgets: [120, 480, 1920, 7680]
- learning rates: [0.002, 0.005, 0.015, 0.035, 0.07]
- all test weights unchanged: **true**
- scientific gate effect: **none**

> Development-only diagnostics. Frozen H0 protocol v3 remains `HYBRID_NO_GO`; these sweeps cannot reverse it and use no fresh held-out seeds. The privileged arm receives true intermediate states and one supervised correction per composition step, so it is a harness ceiling with up to `depth` times the supervision and update magnitude, not an admissible or budget-matched learner.

## Best observed development D*

| arm | D* at lower-95 accuracy ≥ 0.65 |
|---|---:|
| existing-post-synaptic | none |
| least-squares-post-synaptic | 3 |
| direct-terminal | 5 |
| privileged-intermediate-target | 8 |
| shuffled-label | none |

## Best configuration by depth

| depth | arm | budget | learning rate | mean | lower 95% |
|---:|---|---:|---:|---:|---:|
| 1 | existing-post-synaptic | 7680 | 0.0700 | 0.4847 | 0.4015 |
| 1 | least-squares-post-synaptic | 7680 | 0.0700 | 1.0000 | 1.0000 |
| 1 | direct-terminal | 7680 | 0.0700 | 1.0000 | 1.0000 |
| 2 | existing-post-synaptic | 7680 | 0.0050 | 0.3063 | 0.2759 |
| 2 | least-squares-post-synaptic | 7680 | 0.0700 | 1.0000 | 1.0000 |
| 2 | direct-terminal | 7680 | 0.0700 | 1.0000 | 1.0000 |
| 3 | existing-post-synaptic | 120 | 0.0350 | 0.3754 | 0.3670 |
| 3 | least-squares-post-synaptic | 1920 | 0.0350 | 0.8171 | 0.7904 |
| 3 | direct-terminal | 7680 | 0.0700 | 1.0000 | 1.0000 |
| 4 | existing-post-synaptic | 7680 | 0.0020 | 0.2979 | 0.2797 |
| 4 | least-squares-post-synaptic | 7680 | 0.0700 | 0.7326 | 0.6413 |
| 4 | direct-terminal | 7680 | 0.0150 | 0.8643 | 0.8391 |
| 5 | existing-post-synaptic | 7680 | 0.0150 | 0.2653 | 0.2374 |
| 5 | least-squares-post-synaptic | 7680 | 0.0700 | 0.4708 | 0.3751 |
| 5 | direct-terminal | 7680 | 0.0150 | 0.7344 | 0.7206 |
| 6 | existing-post-synaptic | 1920 | 0.0020 | 0.2859 | 0.2804 |
| 6 | least-squares-post-synaptic | 7680 | 0.0700 | 0.6984 | 0.5546 |
| 6 | direct-terminal | 7680 | 0.0350 | 0.6295 | 0.6142 |
| 7 | existing-post-synaptic | 7680 | 0.0020 | 0.2737 | 0.2663 |
| 7 | least-squares-post-synaptic | 7680 | 0.0150 | 0.3551 | 0.3364 |
| 7 | direct-terminal | 7680 | 0.0150 | 0.5538 | 0.5392 |
| 8 | existing-post-synaptic | 7680 | 0.0150 | 0.2464 | 0.2398 |
| 8 | least-squares-post-synaptic | 7680 | 0.0350 | 0.2892 | 0.2705 |
| 8 | direct-terminal | 7680 | 0.0350 | 0.4831 | 0.3803 |

## Paired effects at the direct-terminal optimum

| depth | direct - oracle mean | lower 95% | oracle - existing mean | lower 95% |
|---:|---:|---:|---:|---:|
| 1 | 0.0000 | 0.0000 | 0.5153 | 0.4321 |
| 2 | 0.0000 | 0.0000 | 0.7191 | 0.6526 |
| 3 | 0.2698 | 0.2151 | 0.4704 | 0.4117 |
| 4 | 0.2690 | 0.2321 | 0.3096 | 0.2710 |
| 5 | 0.3329 | 0.2913 | 0.1361 | 0.0892 |
| 6 | 0.0599 | -0.0353 | 0.2947 | 0.1970 |
| 7 | 0.1987 | 0.1783 | 0.0959 | 0.0739 |
| 8 | 0.1938 | 0.0870 | 0.0422 | 0.0220 |

## Controls at maximum budget and best development rate

| depth | control | learning rate | mean | lower 95% |
|---:|---|---:|---:|---:|
| 1 | privileged-intermediate-target | 0.0700 | 1.0000 | 1.0000 |
| 1 | shuffled-label | 0.0700 | 0.0000 | 0.0000 |
| 2 | privileged-intermediate-target | 0.0700 | 1.0000 | 1.0000 |
| 2 | shuffled-label | 0.0020 | 0.0548 | 0.0459 |
| 3 | privileged-intermediate-target | 0.0700 | 1.0000 | 1.0000 |
| 3 | shuffled-label | 0.0020 | 0.1386 | 0.1244 |
| 4 | privileged-intermediate-target | 0.0700 | 1.0000 | 1.0000 |
| 4 | shuffled-label | 0.0020 | 0.1834 | 0.1688 |
| 5 | privileged-intermediate-target | 0.0050 | 1.0000 | 1.0000 |
| 5 | shuffled-label | 0.0020 | 0.1669 | 0.1567 |
| 6 | privileged-intermediate-target | 0.0050 | 1.0000 | 1.0000 |
| 6 | shuffled-label | 0.0020 | 0.2143 | 0.2047 |
| 7 | privileged-intermediate-target | 0.0020 | 1.0000 | 1.0000 |
| 7 | shuffled-label | 0.0020 | 0.2544 | 0.2491 |
| 8 | privileged-intermediate-target | 0.0020 | 1.0000 | 1.0000 |
| 8 | shuffled-label | 0.0020 | 0.2447 | 0.2376 |

## Mechanistic diagnostics at initialization

| depth | gradient norm | direct loss drop | shuffled loss drop | existing cosine | oracle cosine | existing sign | oracle sign |
|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 0.983797 | 0.000968 | -0.000275 | 0.4831 | 1.0000 | 0.2976 | 0.4498 |
| 2 | 1.603881 | 0.002727 | -0.000544 | 0.5328 | 0.9471 | 0.4561 | 0.7136 |
| 3 | 2.061173 | 0.004642 | -0.001271 | 0.6060 | 0.9207 | 0.5497 | 0.8423 |
| 4 | 2.729814 | 0.007640 | -0.002015 | 0.5603 | 0.8893 | 0.5852 | 0.9006 |
| 5 | 3.630057 | 0.013686 | -0.003459 | 0.5128 | 0.8952 | 0.5952 | 0.9192 |
| 6 | 4.018364 | 0.017043 | -0.004152 | 0.5675 | 0.9023 | 0.6105 | 0.9375 |
| 7 | 4.689850 | 0.022948 | -0.005947 | 0.5604 | 0.9023 | 0.6139 | 0.9489 |
| 8 | 5.501851 | 0.031411 | -0.007796 | 0.5436 | 0.9139 | 0.6175 | 0.9539 |

Raw seed-level sweep and mechanism rows are emitted beside this report as CSV.
