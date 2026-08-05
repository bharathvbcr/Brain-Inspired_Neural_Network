# BINN exact-forward credit-assignment repreregistration

**Canonical C1 is immutable:** protocol-v2 hash `c1-118207fbc3eaba53` and every G2 threshold remain unchanged. These are separate protocols with fresh held-out seeds.

- schedule: **SCIENTIFIC**
- seeds: 20
- train/test: 80/40
- matched epochs: 80
- positive control: 1.0000 (minimum 0.9000)
- activity sparsity: 0.0156 (valid [0.0050, 0.0300])
- exact-forward parity: **PASS**

## Arm hashes and results

| arm | protocol | hash | mean accuracy | gap LCB | verdict |
|---|---:|---|---:|---:|---|
| `broadcast-one-pass` | 4 | `c1x-broadcast-one-pass-ec3c5a4d19ccd57e` | 0.4912 | 0.0730 | **FAIL** |
| `broadcast-epoch-matched` | 4 | `c1x-broadcast-epoch-matched-911a03a2a45feaf2` | 0.6875 | 0.1327 | **FAIL** |
| `rpe-three-factor` | 5 | `c1x-rpe-three-factor-872e9eda9303f5df` | 0.5688 | 0.0907 | **FAIL** |
| `eprop-exact-forward` | 6 | `c1x-eprop-exact-forward-fcedc76a80ff0f0e` | 0.6650 | 0.1502 | **FAIL** |
| `dfa-exact-forward` | 7 | `c1x-dfa-exact-forward-4a1601e725edbc80` | 0.5775 | 0.0909 | **FAIL** |
| `surrogate-gradient-exact-forward` | 4 | `c1x-surrogate-gradient-exact-forward-cfe9a2c8d3e22257` | 0.6963 | 0.3263 | **FAIL** |
| `dense-epoch-matched` | 4 | `c1x-dense-epoch-matched-1387104803fe7e0a` | 0.3500 | 0.0000 | **FAIL** |

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
