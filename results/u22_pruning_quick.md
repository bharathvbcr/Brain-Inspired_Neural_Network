# U22 — active forgetting / synaptic pruning

**Exploratory post-G2 override.** `c1-118207fbc3eaba53` remains a FAIL.

- schedule: PILOT
- matched target sparsity: 0.50
- recovery uses local reward-modulated regrowth on the unchanged CSR topology

| rule | accuracy before | after prune | after recovery | old-class retention | realized sparsity |
|---|---:|---:|---:|---:|---:|
| magnitude | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0.500 |
| age | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0.500 |
| eligibility | 1.0000 | 1.0000 | 1.0000 | 1.0000 | 0.500 |
| random | 1.0000 | 0.6700 | 1.0000 | 1.0000 | 0.500 |

Magnitude, age, eligibility, and random pruning receive the exact same edge budget. The table exposes recovery, interference on old classes, and retained active capacity; no rule is selected post hoc.
