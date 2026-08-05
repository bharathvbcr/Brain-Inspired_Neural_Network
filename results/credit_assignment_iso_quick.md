# BINN exact-forward credit-assignment repreregistration

**Canonical C1 is immutable:** protocol-v2 hash `c1-118207fbc3eaba53` and every G2 threshold remain unchanged. These are separate protocols with fresh held-out seeds.

**Trial-isolation protocol (`c1x-iso-*`):** clears `ThreeFactor.last_spike` and applies C3-style full dynamic membrane reset at trial boundaries (arm protocol versions = base + 10). Does **not** reopen frozen non-isolated `c1x-*` hashes or protocol-v2 G2.

- schedule: **PILOT (development only)**
- seeds: 5
- train/test: 24/16
- matched epochs: 4
- trial isolation: **yes**
- positive control: 0.9417 (minimum 0.9000)
- activity sparsity: 0.0125 (valid [0.0050, 0.0300])
- exact-forward parity: **PASS**

## Arm hashes and results

| arm | protocol | hash | mean accuracy | gap LCB | verdict |
|---|---:|---|---:|---:|---|
| `broadcast-one-pass` | 14 | `c1x-iso-broadcast-one-pass-f7e33ebac3fb829c` | 0.5625 | -0.0510 | **PILOT** |
| `broadcast-epoch-matched` | 14 | `c1x-iso-broadcast-epoch-matched-d7da19199fb154e0` | 0.5625 | -0.0510 | **PILOT** |
| `rpe-three-factor` | 15 | `c1x-iso-rpe-three-factor-38157d2b3ed64021` | 0.4250 | 0.0000 | **PILOT** |
| `eprop-exact-forward` | 16 | `c1x-iso-eprop-exact-forward-5d5f984a1e9c4584` | 0.5125 | 0.0171 | **PILOT** |
| `dfa-exact-forward` | 17 | `c1x-iso-dfa-exact-forward-a6ff28d662f0ede2` | 0.7500 | 0.0671 | **PILOT** |
| `surrogate-gradient-exact-forward` | 14 | `c1x-iso-surrogate-gradient-exact-forward-0ebeab8647603ca1` | 0.8000 | 0.4080 | **PILOT** |
| `dense-epoch-matched` | 14 | `c1x-iso-dense-epoch-matched-e958ed981e959774` | 0.5000 | 0.0000 | **PILOT** |

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
