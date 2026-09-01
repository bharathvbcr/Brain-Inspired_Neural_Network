# Wave 22 — the mechanism control at every operating point

Registered: `PREREG_2026-08-29_THE_MECHANISM_AT_EVERY_OPERATING_POINT.md`.
This analyser is the authority on every verdict below.

## H22-1 — the contrast clears its bar at each new point

| point | read-out | DiD | positive | pairs | verdict |
|---|---|---:|---:|---:|---|
| h128 / fixed-t100 / adjacent-sum-5 | d32l4 | +0.1323 | 12/12 | 12 | **MET** |
| h128 / fixed-t250 / adjacent-sum-5 | d32l4 | +0.1119 | 12/12 | 12 | **MET** |
| h128 / fixed-t500 / adjacent-sum-5 | d32l4 | +0.1161 | 12/12 | 12 | **MET** |
| h128 / published-2ms / adjacent-sum-5 | d32l2 | +0.1145 | 12/12 | 12 | **MET** |
| h128 / published-2ms / adjacent-sum-5 | d64l4 | +0.1331 | 12/12 | 12 | **MET** |
| h128 / published-2ms / channels-700 | d32l1 | +0.1369 | 12/12 | 12 | **MET** |
| h256 / published-2ms / adjacent-sum-5 | d32l1 | +0.0758 | 12/12 | 12 | **MET** |
| h512 / published-2ms / adjacent-sum-5 | d32l1 | +0.0893 | 12/12 | 12 | **MET** |
| h768 / published-2ms / adjacent-sum-5 | d32l2 | +0.0610 | 12/12 | 12 | **MET** |
| h1024 / published-2ms / adjacent-sum-5 | d32l1 | +0.0675 | 12/12 | 12 | **MET** |
| h1024 / published-2ms / adjacent-sum-5 | d32l2 | +0.0658 | 12/12 | 12 | **MET** |
| h1024 / published-2ms / adjacent-sum-5 | d32l3 | +0.0675 | 11/12 | 12 | **MET** |

12 of 12 points evaluable (floor 9 seed-paired quadruples each).

## H22-3 — does the contrast depend on read-out depth?

Registered as a QUESTION, not a prediction. Each point against its `d32l4` twin at the same width, on the anchor; range bar 0.10.

| width | depth | DiD | d32l4 DiD | difference |
|---:|---|---:|---:|---:|
| h128 | d32l2 | — | — | not evaluable |
| h128 | d64l4 | — | — | not evaluable |
| h256 | d32l1 | — | — | not evaluable |
| h512 | d32l1 | — | — | not evaluable |
| h768 | d32l2 | — | — | not evaluable |

**H22-3: NOT EVALUABLE** — no depth point cleared the pair floor.
