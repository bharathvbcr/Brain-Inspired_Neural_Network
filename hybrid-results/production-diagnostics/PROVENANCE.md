# Production terminal-credit diagnostic provenance

## Authoritative protocol

`binn-hybrid-production-diagnostic-v3-f72033fbf6906b99`

- 20 development seeds disjoint from H0, smooth diagnostics, and unused
  held-out seeds;
- depths 1 through 8;
- budgets 120, 480, 1,920, and 7,680;
- learning rates 0.002, 0.005, 0.015, 0.035, and 0.070;
- 1,000 frozen test examples per seed and condition;
- production event delivery, identity-residual event, hard k-WTA, and
  production STDP eligibility;
- 11,200 seed-level sweep rows and 160 mechanism rows;
- no test-time weight changes.

The independent replay under `../production-diagnostics-replay/` is
byte-for-byte identical. Replay wall time was 528.87 seconds.

## Superseded pilots

- v1 (`70406e787d80209d`) used layer-specific transition weights and was rejected
  before a full run because it did not match the shared smooth transition
  operator.
- v2 (`8cd6fea27aa912c2`) shared weights but used a softmax transition relaxation
  and was rejected before a full run because it changed gradient geometry.
- v3 quick (`2998fa4d999aa39d`) was the mechanics pilot for the final shared
  residual protocol.

Pilot artifacts are preserved but must not be pooled with or substituted for
the authoritative full v3 evidence.
