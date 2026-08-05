# BINN-Hybrid diagnostic robustness study

- protocol: `binn-hybrid-diagnostic-v3-54b751e38e8212f6`
- schedule: **PILOT**
- seeds: 3
- budgets: [60, 240]
- learning rates: [0.015, 0.035]
- all test weights unchanged: **true**
- scientific gate effect: **none**

> Development-only diagnostics. Frozen H0 protocol v3 remains `HYBRID_NO_GO`; these sweeps cannot reverse it and use no fresh held-out seeds. The privileged arm receives true intermediate states and one supervised correction per composition step, so it is a harness ceiling with up to `depth` times the supervision and update magnitude, not an admissible or budget-matched learner.

## Best observed development D*

| arm | D* at lower-95 accuracy ≥ 0.65 |
|---|---:|
| existing-post-synaptic | none |
| least-squares-post-synaptic | 2 |
| direct-terminal | 3 |
| privileged-intermediate-target | 8 |
| shuffled-label | none |

## Best configuration by depth

| depth | arm | budget | learning rate | mean | lower 95% |
|---:|---|---:|---:|---:|---:|
| 1 | existing-post-synaptic | 240 | 0.0350 | 0.0000 | 0.0000 |
| 1 | least-squares-post-synaptic | 240 | 0.0350 | 0.8063 | 0.6940 |
| 1 | direct-terminal | 240 | 0.0350 | 0.8063 | 0.6940 |
| 2 | existing-post-synaptic | 240 | 0.0350 | 0.1292 | 0.1210 |
| 2 | least-squares-post-synaptic | 240 | 0.0350 | 0.9458 | 0.8857 |
| 2 | direct-terminal | 240 | 0.0350 | 1.0000 | 1.0000 |
| 3 | existing-post-synaptic | 240 | 0.0350 | 0.3667 | 0.3155 |
| 3 | least-squares-post-synaptic | 240 | 0.0350 | 0.6104 | 0.5633 |
| 3 | direct-terminal | 240 | 0.0350 | 0.7021 | 0.6980 |
| 4 | existing-post-synaptic | 240 | 0.0350 | 0.3021 | 0.2595 |
| 4 | least-squares-post-synaptic | 240 | 0.0350 | 0.4167 | 0.2795 |
| 4 | direct-terminal | 240 | 0.0350 | 0.4562 | 0.4099 |
| 5 | existing-post-synaptic | 240 | 0.0350 | 0.1729 | 0.1566 |
| 5 | least-squares-post-synaptic | 240 | 0.0350 | 0.2438 | 0.2193 |
| 5 | direct-terminal | 240 | 0.0350 | 0.2500 | 0.2192 |
| 6 | existing-post-synaptic | 60 | 0.0150 | 0.3042 | 0.2470 |
| 6 | least-squares-post-synaptic | 60 | 0.0150 | 0.2833 | 0.2262 |
| 6 | direct-terminal | 60 | 0.0350 | 0.2854 | 0.2422 |
| 7 | existing-post-synaptic | 60 | 0.0350 | 0.2917 | 0.2568 |
| 7 | least-squares-post-synaptic | 240 | 0.0150 | 0.2750 | 0.2609 |
| 7 | direct-terminal | 240 | 0.0150 | 0.2604 | 0.2496 |
| 8 | existing-post-synaptic | 60 | 0.0150 | 0.2250 | 0.2128 |
| 8 | least-squares-post-synaptic | 60 | 0.0350 | 0.2562 | 0.2238 |
| 8 | direct-terminal | 60 | 0.0350 | 0.2875 | 0.2688 |

## Paired effects at the direct-terminal optimum

| depth | direct - oracle mean | lower 95% | oracle - existing mean | lower 95% |
|---:|---:|---:|---:|---:|
| 1 | 0.0000 | 0.0000 | 0.8063 | 0.6940 |
| 2 | 0.0542 | -0.0060 | 0.8167 | 0.7636 |
| 3 | 0.0917 | 0.0462 | 0.2438 | 0.1885 |
| 4 | 0.0396 | -0.0585 | 0.1146 | 0.0115 |
| 5 | 0.0062 | -0.0246 | 0.0708 | 0.0300 |
| 6 | 0.0187 | 0.0065 | -0.0125 | -0.0370 |
| 7 | -0.0146 | -0.0350 | -0.0000 | -0.0368 |
| 8 | 0.0313 | 0.0125 | 0.0187 | -0.0365 |

## Controls at maximum budget and best development rate

| depth | control | learning rate | mean | lower 95% |
|---:|---|---:|---:|---:|
| 1 | privileged-intermediate-target | 0.0350 | 0.8063 | 0.6940 |
| 1 | shuffled-label | 0.0350 | 0.0000 | 0.0000 |
| 2 | privileged-intermediate-target | 0.0350 | 1.0000 | 1.0000 |
| 2 | shuffled-label | 0.0150 | 0.1292 | 0.1210 |
| 3 | privileged-intermediate-target | 0.0350 | 1.0000 | 1.0000 |
| 3 | shuffled-label | 0.0350 | 0.1396 | 0.1218 |
| 4 | privileged-intermediate-target | 0.0350 | 1.0000 | 1.0000 |
| 4 | shuffled-label | 0.0150 | 0.2062 | 0.1599 |
| 5 | privileged-intermediate-target | 0.0350 | 1.0000 | 1.0000 |
| 5 | shuffled-label | 0.0350 | 0.2021 | 0.1687 |
| 6 | privileged-intermediate-target | 0.0350 | 1.0000 | 1.0000 |
| 6 | shuffled-label | 0.0150 | 0.2521 | 0.1928 |
| 7 | privileged-intermediate-target | 0.0350 | 0.9854 | 0.9676 |
| 7 | shuffled-label | 0.0350 | 0.2896 | 0.2628 |
| 8 | privileged-intermediate-target | 0.0350 | 0.9729 | 0.9688 |
| 8 | shuffled-label | 0.0350 | 0.2521 | 0.2050 |

## Mechanistic diagnostics at initialization

| depth | gradient norm | direct loss drop | shuffled loss drop | existing cosine | oracle cosine | existing sign | oracle sign |
|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 0.983936 | 0.000968 | -0.000245 | 0.4869 | 1.0000 | 0.3027 | 0.4303 |
| 2 | 1.550336 | 0.002567 | -0.000448 | 0.5442 | 0.9515 | 0.4746 | 0.7012 |
| 3 | 2.104631 | 0.004904 | -0.001227 | 0.6174 | 0.9398 | 0.5547 | 0.8255 |
| 4 | 2.765627 | 0.007867 | -0.002116 | 0.5386 | 0.8526 | 0.5859 | 0.8880 |
| 5 | 3.563123 | 0.013298 | -0.003221 | 0.5059 | 0.8817 | 0.5885 | 0.9076 |
| 6 | 3.978870 | 0.016860 | -0.003070 | 0.5991 | 0.9319 | 0.5827 | 0.9707 |
| 7 | 4.696499 | 0.023164 | -0.006803 | 0.5680 | 0.8946 | 0.6113 | 0.9421 |
| 8 | 5.529582 | 0.032356 | -0.009550 | 0.5567 | 0.9337 | 0.5768 | 0.9609 |

Raw seed-level sweep and mechanism rows are emitted beside this report as CSV.
