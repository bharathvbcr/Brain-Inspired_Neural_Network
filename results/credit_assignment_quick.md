# BINN exact-forward credit-assignment repreregistration

**Canonical C1 is immutable:** protocol-v2 hash `c1-118207fbc3eaba53` and every G2 threshold remain unchanged. These are separate protocols with fresh held-out seeds.

- schedule: **PILOT (development only)**
- seeds: 5
- train/test: 24/16
- matched epochs: 4
- positive control: 1.0000 (minimum 0.9000)
- activity sparsity: 0.0156 (valid [0.0050, 0.0300])
- exact-forward parity: **PASS**

## Arm hashes and results

| arm | protocol | hash | mean accuracy | gap LCB | verdict |
|---|---:|---|---:|---:|---|
| `broadcast-one-pass` | 4 | `c1x-broadcast-one-pass-a885b80b16dc6c30` | 0.5375 | 0.0000 | **PILOT** |
| `broadcast-epoch-matched` | 4 | `c1x-broadcast-epoch-matched-bfd1dc58bb6e6db4` | 0.5875 | 0.0000 | **PILOT** |
| `rpe-three-factor` | 5 | `c1x-rpe-three-factor-0d778c10682b1015` | 0.4625 | 0.0000 | **PILOT** |
| `eprop-exact-forward` | 6 | `c1x-eprop-exact-forward-024b057e78c3c020` | 0.5750 | -0.0114 | **PILOT** |
| `dfa-exact-forward` | 7 | `c1x-dfa-exact-forward-ecbf1b1bc2e1d9c6` | 0.7500 | -0.0801 | **PILOT** |
| `surrogate-gradient-exact-forward` | 4 | `c1x-surrogate-gradient-exact-forward-450d24e97542751d` | 0.6875 | 0.1199 | **PILOT** |
| `dense-epoch-matched` | 4 | `c1x-dense-epoch-matched-471607fb6639e8fc` | 0.5000 | 0.0000 | **PILOT** |

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
| no test updates | true |

## Interpretation contract

- Epoch-matched broadcast improves over one-pass: exposure is material.
- Exact-forward gradient collapses: the old front-end/reference mismatch inflated the gap.
- RPE alone improves: reward centering/scaling is material.
- E-prop/DFA improve while broadcast does not: neuron-specific credit is supported.
- No outcome changes or rescues canonical protocol-v2 G2.
