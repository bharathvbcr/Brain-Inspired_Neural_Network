# BINN-Hybrid diagnostic robustness study

- protocol: `binn-hybrid-diagnostic-v2-50c8899f9bc536f0`
- schedule: **FULL DIAGNOSTIC**
- seeds: 8
- budgets: [120, 480, 1920]
- learning rates: [0.005, 0.015, 0.035, 0.07]
- all test weights unchanged: **true**
- scientific gate effect: **none**

> Development-only diagnostics. Frozen H0 protocol v3 remains `HYBRID_NO_GO`; these sweeps cannot reverse it and use no fresh held-out seeds. The privileged arm receives true intermediate states and one supervised correction per composition step, so it is a harness ceiling with up to `depth` times the supervision and update magnitude, not an admissible or budget-matched learner.

## Best observed development D*

| arm | D* at lower-95 accuracy ≥ 0.65 |
|---|---:|
| existing-post-synaptic | none |
| least-squares-post-synaptic | 3 |
| direct-terminal | 4 |
| privileged-intermediate-target | 7 |
| shuffled-label | none |

## Best configuration by depth

| depth | arm | budget | learning rate | mean | lower 95% |
|---:|---|---:|---:|---:|---:|
| 1 | existing-post-synaptic | 1920 | 0.0700 | 0.5225 | 0.3940 |
| 1 | least-squares-post-synaptic | 1920 | 0.0700 | 1.0000 | 1.0000 |
| 1 | direct-terminal | 1920 | 0.0700 | 1.0000 | 1.0000 |
| 2 | existing-post-synaptic | 1920 | 0.0700 | 0.4035 | 0.2694 |
| 2 | least-squares-post-synaptic | 1920 | 0.0700 | 1.0000 | 1.0000 |
| 2 | direct-terminal | 1920 | 0.0700 | 1.0000 | 1.0000 |
| 3 | existing-post-synaptic | 480 | 0.0150 | 0.3728 | 0.3592 |
| 3 | least-squares-post-synaptic | 1920 | 0.0350 | 0.8150 | 0.7719 |
| 3 | direct-terminal | 1920 | 0.0350 | 1.0000 | 1.0000 |
| 4 | existing-post-synaptic | 1920 | 0.0150 | 0.3322 | 0.2810 |
| 4 | least-squares-post-synaptic | 1920 | 0.0350 | 0.5685 | 0.4990 |
| 4 | direct-terminal | 1920 | 0.0350 | 0.7435 | 0.6835 |
| 5 | existing-post-synaptic | 1920 | 0.0350 | 0.2262 | 0.2016 |
| 5 | least-squares-post-synaptic | 1920 | 0.0700 | 0.3620 | 0.3070 |
| 5 | direct-terminal | 1920 | 0.0350 | 0.6110 | 0.5390 |
| 6 | existing-post-synaptic | 480 | 0.0050 | 0.2882 | 0.2755 |
| 6 | least-squares-post-synaptic | 1920 | 0.0700 | 0.3575 | 0.3122 |
| 6 | direct-terminal | 1920 | 0.0700 | 0.7635 | 0.5268 |
| 7 | existing-post-synaptic | 1920 | 0.0050 | 0.2760 | 0.2607 |
| 7 | least-squares-post-synaptic | 1920 | 0.0350 | 0.2980 | 0.2900 |
| 7 | direct-terminal | 1920 | 0.0350 | 0.3340 | 0.3043 |
| 8 | existing-post-synaptic | 1920 | 0.0700 | 0.2375 | 0.2290 |
| 8 | least-squares-post-synaptic | 1920 | 0.0700 | 0.2617 | 0.2511 |
| 8 | direct-terminal | 1920 | 0.0350 | 0.2917 | 0.2701 |

## Controls at maximum budget

| depth | control | mean | lower 95% |
|---:|---|---:|---:|
| 1 | privileged-intermediate-target | 1.0000 | 1.0000 |
| 1 | shuffled-label | 0.0000 | 0.0000 |
| 2 | privileged-intermediate-target | 1.0000 | 1.0000 |
| 2 | shuffled-label | 0.0000 | 0.0000 |
| 3 | privileged-intermediate-target | 1.0000 | 1.0000 |
| 3 | shuffled-label | 0.0795 | 0.0543 |
| 4 | privileged-intermediate-target | 1.0000 | 1.0000 |
| 4 | shuffled-label | 0.0688 | 0.0421 |
| 5 | privileged-intermediate-target | 0.8762 | 0.8693 |
| 5 | shuffled-label | 0.1318 | 0.0990 |
| 6 | privileged-intermediate-target | 0.7340 | 0.7227 |
| 6 | shuffled-label | 0.2163 | 0.1835 |
| 7 | privileged-intermediate-target | 0.7040 | 0.6850 |
| 7 | shuffled-label | 0.2235 | 0.2054 |
| 8 | privileged-intermediate-target | 0.6477 | 0.6327 |
| 8 | shuffled-label | 0.2497 | 0.2379 |

## Mechanistic diagnostics at initialization

| depth | gradient norm | direct loss drop | shuffled loss drop | existing cosine | oracle cosine | existing sign | oracle sign |
|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 0.983606 | 0.000967 | -0.000278 | 0.4829 | 1.0000 | 0.2980 | 0.4536 |
| 2 | 1.580277 | 0.002661 | -0.000537 | 0.5387 | 0.9473 | 0.4594 | 0.7130 |
| 3 | 2.017585 | 0.004464 | -0.001214 | 0.6138 | 0.9230 | 0.5547 | 0.8453 |
| 4 | 2.739523 | 0.007702 | -0.001988 | 0.5545 | 0.8825 | 0.5978 | 0.9036 |
| 5 | 3.618954 | 0.013611 | -0.003371 | 0.5130 | 0.9031 | 0.5895 | 0.9216 |
| 6 | 4.072537 | 0.017434 | -0.004195 | 0.5579 | 0.9038 | 0.5994 | 0.9382 |
| 7 | 4.707309 | 0.023061 | -0.006130 | 0.5530 | 0.9028 | 0.6125 | 0.9483 |
| 8 | 5.497567 | 0.031350 | -0.007905 | 0.5429 | 0.9138 | 0.6360 | 0.9503 |

Raw seed-level sweep and mechanism rows are emitted beside this report as CSV.
