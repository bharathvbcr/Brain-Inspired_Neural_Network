# BINN exact-forward credit-assignment repreregistration

**Canonical C1 is immutable:** protocol-v2 hash `c1-118207fbc3eaba53` and every G2 threshold remain unchanged. These are separate protocols with fresh held-out seeds.

**Sparsity-calibrated trial-isolation (`c1x-iso-s-*`):** clears `ThreeFactor.last_spike`, applies C3-style full dynamic membrane reset, and selects k-WTA over all finite membranes (winner floor; arm protocol versions = base + 20). G2 / sparsity-band thresholds unchanged. Does **not** reopen frozen `c1x-*`, prior `c1x-iso-*`, or protocol-v2 G2.

- schedule: **SCIENTIFIC**
- seeds: 20
- train/test: 80/40
- matched epochs: 80
- trial isolation: **yes**
- positive control: 0.9875 (minimum 0.9000)
- activity sparsity: 0.0156 (valid [0.0050, 0.0300])
- exact-forward parity: **PASS**

## Arm hashes and results

| arm | protocol | hash | mean accuracy | gap LCB | verdict |
|---|---:|---|---:|---:|---|
| `broadcast-one-pass` | 24 | `c1x-iso-s-broadcast-one-pass-6abe723b6700113c` | 0.6563 | 0.0000 | **FAIL** |
| `broadcast-epoch-matched` | 24 | `c1x-iso-s-broadcast-epoch-matched-4e3236f8f60433d0` | 0.6650 | 0.0000 | **FAIL** |
| `rpe-three-factor` | 25 | `c1x-iso-s-rpe-three-factor-e1fd914d40873269` | 0.6175 | 0.0000 | **FAIL** |
| `eprop-exact-forward` | 26 | `c1x-iso-s-eprop-exact-forward-552924e96f2dded4` | 0.7138 | 0.0000 | **FAIL** |
| `dfa-exact-forward` | 27 | `c1x-iso-s-dfa-exact-forward-d2c8d3c929a68bd2` | 0.6738 | 0.0000 | **FAIL** |
| `surrogate-gradient-exact-forward` | 24 | `c1x-iso-s-surrogate-gradient-exact-forward-75f280fac365d671` | 0.5700 | 0.0000 | **FAIL** |
| `dense-epoch-matched` | 24 | `c1x-iso-s-dense-epoch-matched-1f81769d0d7623b0` | 1.0000 | 0.0000 | **FAIL** |

## Exact-forward contract

Assembly arms share the production LatencyEncoder, event engine, sparse topology and initialized weights, membrane-score hard k-WTA, forced winners, dual readout decision, frozen split, and deterministic exposure order. The one-pass arm is the declared exposure diagnostic; all other assembly arms use the matched epoch count. The dense arm is the declared topology control.

| check | passed |
|---|---|
| topology | true |
| initial weights | true |
| frozen split | true |
| matched exposure order | true |
| initial prediction | true |
| initial winners | true |
| initial charges | true |
| forward target independence | true |
| no test updates | true |

## Preregistered interpretation contract

- If epoch-matched broadcast improves over one-pass, exposure was material.
- If the exact-forward gradient reference collapses, the old front-end/reference mismatch inflated the gap.
- If RPE alone improves, reward centering/scaling was material.
- If E-prop/DFA improve while matched broadcast does not, neuron-specific credit is supported.
- No outcome changes or rescues canonical protocol-v2 G2.
