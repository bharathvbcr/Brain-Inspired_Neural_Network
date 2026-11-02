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
| sleep-provenance-consolidation | 0.0000 | 1.0000 |

`offline-local-consolidation` and `sleep-provenance-consolidation` drop replay labels and reinforce the locally predicted assembly (`sleep-provenance-consolidation` adds decoupled coordinate-wise erase/write gating, homeostatic dormancy renewal, and bipartite episodic provenance); the other replay arms disclose supervised labels. The comparison identifies whether offline local consolidation adds value beyond matched replay without changing the C1 decision.
