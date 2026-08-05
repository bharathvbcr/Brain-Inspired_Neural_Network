# Winner-temperature ladder provenance

## Authoritative protocol

`binn-hybrid-winner-temp-v1-fa7710de68ad7bfe`

- 20 development seeds from master `0x4842_5445_4d50_0001`, disjoint from H0,
  smooth diagnostic, production diagnostic, and unused held-out seeds;
- depths 1 through 8;
- budgets 480, 1,920, and 7,680;
- learning rates 0.015, 0.035, and 0.070;
- temperatures soft, 2.0, 1.0, 0.5, 0.25, 0.1, hard;
- 1,000 frozen test examples per seed and condition;
- 16,800 seed-level sweep rows and 1,120 mechanism rows;
- no test-time weight changes;
- wall time about 8.45 seconds for the full release run.

SHA-256:

- report: `19bde1628cd82b3ce7c50a209db9208d335b623fcda952f4e5d6a9b1057eca57`
- sweep: `b167949474138050e4f2772aa4d0cf4e401c289c2b35363b374e9f2f16d78d1d`
- mechanisms: `738924208eaed352b7a7c76a00fe11c3f02859b48d548c737e3be0919a37478f`

## Result summary

- soft direct-terminal D*=5
- T∈{2.0,1.0,0.5,0.25,0.1} direct-terminal D*=2
- hard direct-terminal D*=1
- transfer collapse temperature = 2.0
- H0 remains `HYBRID_NO_GO`; held-out unused; H1-H3 stopped

## Pilot

`binn-hybrid-winner-temp-v1-a403740b49361591` is the quick mechanics pilot.
Preserve it for provenance only; do not pool with the authoritative full run.
