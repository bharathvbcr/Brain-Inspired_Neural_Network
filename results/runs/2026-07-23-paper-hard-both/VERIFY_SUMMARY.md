# VERIFY_SUMMARY — paper-hard-both (2026-07-23)

Camp: `binn/results/runs/2026-07-23-paper-hard-both/`

Exact `--config-hash` replays of primary camera-ready arms. **All bit-stable vs claim freeze.**

| Arm | Hash | Verdict | Primary | Gap LCB | Match freeze |
|---|---|---|---:|---:|---|
| Matched broadcast 3F | `c1-match-5dc6822e71229e9e` | **FAIL** | 0.5000 | 0.0000 | YES |
| Matched DFA | `c1-dfa-c8c4fe0899908b84` | **PASS** | 0.9387 | 0.6894 | YES |
| Matched RL reinforce_fb | `c1-rl-42eddc9c801308e9` | **PASS** | 0.9200 | 0.6846 | YES |
| Canonical C1 / G2 | `c1-118207fbc3eaba53` | **FAIL** | 0.4912 | −0.0048 | YES |
| Live RFB v13 | `c1-660401d74db3c88d` | **FAIL** | 0.4900 | 0.0737 | YES |
| Structured B v15 | `c1-493ddd56f8714fb6` | **FAIL** | 0.7262 | 0.2567 | YES |

Replay logs: `c1_*_replay.log` / `c1_*_replay.md` in this camp.

## Break-it wave (new hashes; not freeze remassage)

| Proto | Hash | Verdict | Local / primary | Gate gap LCB | Chance gap LCB |
|---|---|---|---:|---:|---:|
| v20 dfa-live | `c1-4db53e645405fae0` | **FAIL** | 0.7325 | 0.2601 | 0.3321 |
| v21 sfb-soft | `c1-f975db8fb3e5d569` | **FAIL** | 0.5025 | 0.0406 | 0.0122 |
| v22 match-ep4 | `c1-match-b46b23549b37d90a` | **FAIL** | 0.5000 | 0.0000 | (matched chance gap) |
| v23 sfb-finth | `c1-4bbaf4b24c2d1da2` | **FAIL** | 0.6638 | 0.2370 | 0.2370 |
| v24 sfb-cont | `c1-840f820b7c07b512` | **FAIL** | 0.6437 | 0.1380 | 0.1163 |

G2 bars unchanged. No remassage of closed hashes.
