# BINN exact-forward credit-assignment repreregistration

**Canonical C1 is immutable:** protocol-v2 hash `c1-118207fbc3eaba53` and every G2 threshold remain unchanged. These are separate protocols with fresh held-out seeds.

**Sparsity-calibrated trial-isolation (`c1x-iso-s-*`):** clears `ThreeFactor.last_spike`, applies C3-style full dynamic membrane reset, and selects k-WTA over all finite membranes (winner floor; arm protocol versions = base + 20). G2 / sparsity-band thresholds unchanged. Does **not** reopen frozen `c1x-*`, prior `c1x-iso-*`, or protocol-v2 G2.

- schedule: **PILOT (development only)**
- seeds: 5
- train/test: 24/16
- matched epochs: 4
- trial isolation: **yes**
- positive control: 1.0000 (minimum 0.9000)
- activity sparsity: 0.0156 (valid [0.0050, 0.0300])
- exact-forward parity: **PASS**

## Arm hashes and results

| arm | protocol | hash | mean accuracy | gap LCB | verdict |
|---|---:|---|---:|---:|---|
| `broadcast-one-pass` | 24 | `c1x-iso-s-broadcast-one-pass-001fbf8eb05d9e6a` | 0.6500 | -0.1097 | **PILOT** |
| `broadcast-epoch-matched` | 24 | `c1x-iso-s-broadcast-epoch-matched-0d245e821418da2e` | 0.6500 | -0.1097 | **PILOT** |
| `rpe-three-factor` | 25 | `c1x-iso-s-rpe-three-factor-b6c1a556427b50db` | 0.5500 | 0.0000 | **PILOT** |
| `eprop-exact-forward` | 26 | `c1x-iso-s-eprop-exact-forward-b745cfca8f0e51e2` | 0.5625 | -0.0274 | **PILOT** |
| `dfa-exact-forward` | 27 | `c1x-iso-s-dfa-exact-forward-208bedb6f50d0b9c` | 0.5750 | 0.0000 | **PILOT** |
| `surrogate-gradient-exact-forward` | 24 | `c1x-iso-s-surrogate-gradient-exact-forward-32f4a5742da3bde3` | 0.5375 | -0.1920 | **PILOT** |
| `dense-epoch-matched` | 24 | `c1x-iso-s-dense-epoch-matched-35c9663264312a4e` | 0.5000 | 0.0000 | **PILOT** |

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
