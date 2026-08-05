# BINN exact-forward credit-assignment repreregistration

**Canonical C1 is immutable:** protocol-v2 hash `c1-118207fbc3eaba53` and every G2 threshold remain unchanged. These are separate protocols with fresh held-out seeds.

**Trial-isolation protocol (`c1x-iso-*`):** clears `ThreeFactor.last_spike` and applies C3-style full dynamic membrane reset at trial boundaries (arm protocol versions = base + 10). Does **not** reopen frozen non-isolated `c1x-*` hashes or protocol-v2 G2.

- schedule: **SCIENTIFIC**
- seeds: 20
- train/test: 80/40
- matched epochs: 80
- trial isolation: **yes**
- positive control: 0.9488 (minimum 0.9000)
- activity sparsity: 0.0031 (valid [0.0050, 0.0300])
- exact-forward parity: **PASS**

## Arm hashes and results

| arm | protocol | hash | mean accuracy | gap LCB | verdict |
|---|---:|---|---:|---:|---|
| `broadcast-one-pass` | 14 | `c1x-iso-broadcast-one-pass-4265d41a3ecad902` | 0.4925 | 0.0098 | **INVALID_HARNESS** |
| `broadcast-epoch-matched` | 14 | `c1x-iso-broadcast-epoch-matched-7becb435b63868c6` | 0.4250 | -0.0277 | **INVALID_HARNESS** |
| `rpe-three-factor` | 15 | `c1x-iso-rpe-three-factor-7be2092f12f5a653` | 0.6187 | 0.0246 | **INVALID_HARNESS** |
| `eprop-exact-forward` | 16 | `c1x-iso-eprop-exact-forward-1c2d2e8835df30ca` | 0.6637 | 0.0569 | **INVALID_HARNESS** |
| `dfa-exact-forward` | 17 | `c1x-iso-dfa-exact-forward-3fd0919313abbe04` | 0.6938 | 0.0998 | **INVALID_HARNESS** |
| `surrogate-gradient-exact-forward` | 14 | `c1x-iso-surrogate-gradient-exact-forward-28adc822a3a572d3` | 0.6000 | 0.1797 | **INVALID_HARNESS** |
| `dense-epoch-matched` | 14 | `c1x-iso-dense-epoch-matched-376a3ba027ad5bba` | 0.3500 | 0.0000 | **INVALID_HARNESS** |

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
