# Campaign claim freeze

Numbers only from this campaign's on-disk notes. Do not invent.

| Claim object | Hash | Verdict | Local / primary | Gap LCB |
|---|---|---|---:|---:|
| Matched broadcast (v4) | `c1-match-5dc6822e71229e9e` | **FAIL** | 0.5000 | 0.0000 |
| Matched DFA (v5) | `c1-dfa-c8c4fe0899908b84` | **PASS** | 0.9387 | 0.6894 |
| Matched RL reinforce_fb (v12) | `c1-rl-42eddc9c801308e9` | **PASS** | 0.9200 | 0.6846 |
| Live ReinforceFeedback (v13) | `c1-660401d74db3c88d` | **FAIL** | 0.4900 | 0.0737 |
| Capacity sensitivity | `c1-d38d7644d8afc84b` | **FAIL** | 0.6775 (≥0.65 floor) | 0.0000 |
| Temporal-PC sensitivity | `c1-a49deeaedb495a09` | **FAIL** | 0.5263 | (gap mean 0.0947) |
| Trial isolation | `c1-8ec031907a3426d0` | **FAIL** | 0.5188 | (gap mean 0.2109) |
| Canonical C1 / G2 | `c1-118207fbc3eaba53` | **FAIL** | 0.4912 | −0.0048 |
| True e-prop (methods) | `c1x-eprop-true-true-surrogate-eprop-0e2aeb90d68ac5f9` | note | 0.7125 | — |

## NumPy deep (this campaign)

| Exp | Finding |
|---|---|
| `xor_thresh` | broadcast 0.501 / DFA 0.827 / grad 0.773 — **locality flip** |
| `depth_locality` mid | broadcast 0.816 / DFA 0.825 / rl_fb 0.803 — depth help ≠ locality flip |

## Locked reading

1. **Primary publishable:** broadcast three-factor fails matched dense-LIF gate.
2. DFA / REINFORCE×`B_i` clear matched gate; live k-WTA transfer fails (RFB).
3. Capacity raises local above 0.65 but gap vs dense/grad stays closed — descriptive footnote only.
4. Temporal-PC and isolation do not rescue G2.
5. Canonical v2 kill-gate replay FAIL — integrity intact.
6. Do **not** claim biology, Assembly Calculus PASS, or impossibility.

## Stop decision

Experimental campaign **closed** for this loop. Next work requires a *new* protocol+hash (e.g. structured/learned `B` under k-WTA) or paper packaging from `PAPER_SKELETON.md` / `PUBLISHABLE_CLAIMS.md`. No remassage of P4/P5/P9.
