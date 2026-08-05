# C3 BPTT — real surrogate backprop on production credit-depth

**Exploratory post-G2 preregistration.** Canonical C1 protocol-v2 hash `c1-118207fbc3eaba53` remains a FAIL. Frozen `c3v2-*` [`matched-forward-oracle-gradient-reference`] is **not** BPTT — it injects oracle target pulses after forward. This family (`c3-bptt-*`) tests true SuperSpike surrogate BPTT through layer transitions.

## Mechanism disclosure

| arm | label | learning path |
|---|---|---|
| `C3_SUPERSPIKE_SURROGATE_BPTT_REFERENCE` | superspike-bptt | backward unroll through layer scores; `Δw` on CSR edges; **no** oracle correction pulses |
| `C3_ORACLE_TARGET_PULSES_NOT_BPTT` | oracle-pulses | oracle `force_spike` target pulses + STDP credit (same idea as c3v2 matched reference; **not BPTT**) |

- protocol version: 1
- verdict: **PILOT**
- seeds: 3
- depth: 1..=4
- train/test per depth×seed: 120/80
- surrogate β: 5.0
- D* accuracy floor: 0.650

> PILOT only: development seeds validate mechanics and cannot support a scientific D* claim.

## Arm hashes and D*

| arm | hash | D* |
|---|---|---:|
| `superspike-bptt` | `c3-bptt-superspike-bptt-c5f2eecb9278df4e` | 2 |
| `oracle-pulses` | `c3-bptt-oracle-pulses-83799c3011267d05` | 4 |

## Accuracy by depth

| depth | arm | mean | variance |
|---:|---|---:|---:|
| 1 | `superspike-bptt` | 1.0000 | 0.000000 |
| 1 | `oracle-pulses` | 0.9625 | 0.004219 |
| 2 | `superspike-bptt` | 0.9333 | 0.003490 |
| 2 | `oracle-pulses` | 1.0000 | 0.000000 |
| 3 | `superspike-bptt` | 0.5625 | 0.030625 |
| 3 | `oracle-pulses` | 1.0000 | 0.000000 |
| 4 | `superspike-bptt` | 0.3833 | 0.027240 |
| 4 | `oracle-pulses` | 1.0000 | 0.000000 |

## Interpretation contract

- SuperSpike BPTT is the scientifically meaningful gradient reference on this graph; oracle pulses are a labeled contrast only.
- Outcomes do not reopen frozen `c3v2-*` or canonical C1 G2.
