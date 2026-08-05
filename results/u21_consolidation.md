# U21 — offline consolidation / replay

**Exploratory post-G2 override.** `c1-118207fbc3eaba53` remains a FAIL.

- schedule: scientific
- seeds: 10
- exact and generated arms use the same replay/update budget
- test examples are rejected by replay storage and generation

| arm | mean forgetting | final mean accuracy |
|---|---:|---:|
| no-sleep | 0.8000 | 0.2000 |
| exact-replay | 0.8000 | 0.2000 |
| generative-replay | 0.0000 | 1.0000 |
| offline-local-consolidation | 0.8000 | 0.2000 |

`offline-local-consolidation` drops replay labels and reinforces the locally predicted assembly; the other replay arms disclose supervised labels. The comparison identifies whether offline local consolidation adds value beyond matched replay without changing the C1 decision.
