# C3 BPTT — real surrogate backprop on production credit-depth

**Exploratory post-G2 preregistration.** Canonical C1 protocol-v2 hash `c1-118207fbc3eaba53` remains a FAIL. Frozen `c3v2-*` [`matched-forward-oracle-gradient-reference`] is **not** BPTT — it injects oracle target pulses after forward. This family (`c3-bptt-*`) tests true SuperSpike surrogate BPTT through layer transitions.

## Mechanism disclosure

| arm | label | learning path |
|---|---|---|
| `C3_SUPERSPIKE_SURROGATE_BPTT_REFERENCE` | superspike-bptt | backward unroll through layer scores; `Δw` on CSR edges; **no** oracle correction pulses |
| `C3_ORACLE_TARGET_PULSES_NOT_BPTT` | oracle-pulses | oracle `force_spike` target pulses + STDP credit (same idea as c3v2 matched reference; **not BPTT**) |

- protocol version: 1
- verdict: **MEASURED**
- seeds: 20
- depth: 1..=8
- train/test per depth×seed: 2000/500
- surrogate β: 5.0
- D* accuracy floor: 0.650

## Arm hashes and D*

| arm | hash | D* |
|---|---|---:|
| `superspike-bptt` | `c3-bptt-superspike-bptt-a1efec9cf8a24968` | 4 |
| `oracle-pulses` | `c3-bptt-oracle-pulses-fc574f1d7c8c8d4f` | 8 |

## Accuracy by depth

| depth | arm | mean | variance |
|---:|---|---:|---:|
| 1 | `superspike-bptt` | 1.0000 | 0.000000 |
| 1 | `oracle-pulses` | 0.9761 | 0.003951 |
| 2 | `superspike-bptt` | 0.9877 | 0.003026 |
| 2 | `oracle-pulses` | 1.0000 | 0.000000 |
| 3 | `superspike-bptt` | 0.8666 | 0.021708 |
| 3 | `oracle-pulses` | 1.0000 | 0.000000 |
| 4 | `superspike-bptt` | 0.7031 | 0.030049 |
| 4 | `oracle-pulses` | 1.0000 | 0.000000 |
| 5 | `superspike-bptt` | 0.5915 | 0.015977 |
| 5 | `oracle-pulses` | 1.0000 | 0.000000 |
| 6 | `superspike-bptt` | 0.4300 | 0.019303 |
| 6 | `oracle-pulses` | 1.0000 | 0.000000 |
| 7 | `superspike-bptt` | 0.3186 | 0.009332 |
| 7 | `oracle-pulses` | 1.0000 | 0.000000 |
| 8 | `superspike-bptt` | 0.2597 | 0.000603 |
| 8 | `oracle-pulses` | 1.0000 | 0.000000 |

## Interpretation contract

- SuperSpike BPTT is the scientifically meaningful gradient reference on this graph; oracle pulses are a labeled contrast only.
- Outcomes do not reopen frozen `c3v2-*` or canonical C1 G2.
