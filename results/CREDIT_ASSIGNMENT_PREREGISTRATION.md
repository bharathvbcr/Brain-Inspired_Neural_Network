# BINN credit-assignment repreregistration

**Frozen:** 2026-07-23, before held-out scientific execution  
**Canonical record:** C1 protocol v2 `c1-118207fbc3eaba53` remains **FAIL**  
**Policy:** new hashes and fresh seeds; no threshold relaxation, rescue, or reinterpretation

## Questions

1. Does matching the exact production forward graph shrink the apparent
   gradient gap?
2. Does repeated exposure improve broadcast three-factor learning?
3. Does a causal reward-prediction-error baseline help without changing the
   credit topology?
4. Do neuron-specific e-prop or DFA signals outperform one broadcast scalar?
5. Does the production event engine reproduce the depth collapse previously
   observed only in the C3 v1 tabular proxy?

Development/PILOT results select no scientific conclusion. Hyperparameters are
fixed from the existing C1/C3 configs; held-out scientific seeds are disjoint
from quick seeds.

## Exact-forward C1 protocols

Shared substrate for every assembly arm:

- production `LatencyEncoder`
- event-driven dendritic-LIF `Engine`
- identical seeded sparse topology and initialized weights
- membrane-score hard k-WTA and forced winner spikes
- dual readout decision
- byte-identical frozen train/test split and deterministic exposure order
- no weight update during held-out evaluation

The dense arm is the declared topology control. The one-pass arm is the
declared exposure diagnostic; all other assembly arms receive 80 identical-order
epochs in the scientific schedule.

Scientific schedule: 20 fresh seeds, 80 train and 40 test examples per seed,
`N=128`, `k=2`. Existing G2 thresholds are unchanged: accuracy ≥0.65,
normalized-gap LCB >0.5, positive control ≥0.9, and activity sparsity in
[0.005, 0.03].

| Arm | Protocol | Held-out hash |
|---|---:|---|
| broadcast one pass | 4 | `c1x-broadcast-one-pass-ec3c5a4d19ccd57e` |
| broadcast epoch matched | 4 | `c1x-broadcast-epoch-matched-911a03a2a45feaf2` |
| RPE three-factor | 5 | `c1x-rpe-three-factor-872e9eda9303f5df` |
| exact-forward e-prop | 6 | `c1x-eprop-exact-forward-fcedc76a80ff0f0e` |
| exact-forward DFA | 7 | `c1x-dfa-exact-forward-4a1601e725edbc80` |
| exact-forward surrogate gradient | 4 | `c1x-surrogate-gradient-exact-forward-cfe9a2c8d3e22257` |
| dense epoch-matched control | 4 | `c1x-dense-epoch-matched-1387104803fe7e0a` |

Mechanisms:

- **RPE:** selected-action signal is `reward − mean(previous rewards)`. The
  current reward enters the running mean only afterward. The explicit wrong-arm
  corrective target pulse remains +1.
- **E-prop:** hidden-cell credit is output error projected through current
  hidden-to-readout weights; synaptic differentiation remains production STDP
  eligibility.
- **DFA:** output error is projected through one deterministic fixed-random
  matrix. Feedback weights are never transported, trained, or mutated.
- **Surrogate gradient:** the hard production forward result is retained;
  readout-error gradients and a straight-through membrane/WTA surrogate update
  the same topology. It is a disclosed reference, not a production learner.

Run:

```bash
cargo run --release -p binn-lab --bin credit-assignment -- \
  --out results/credit_assignment.md

# Development smoke only
cargo run -p binn-lab --bin credit-assignment -- --quick \
  --out results/credit_assignment_quick.md

# Print all frozen hashes without training
cargo run -p binn-lab --bin credit-assignment -- --list-hashes
```

Any arm hash replays the complete paired preset so comparisons cannot silently
lose their matched conditions.

## Production C3 v2

C3 v1 is preserved as a **tabular terminal-reward proxy**. Its former
“gradient” condition is an oracle teacher-forced control, not a production
gradient learner. C3 v2 is the production-faithful depth experiment.

At each layer, a real `(state, operation)` source cell spikes through CSR
synapses into an event-engine state area. Membrane charge selects one hard
k-WTA state; its forced spike creates production STDP eligibility. Broadcast
and RPE arms receive terminal reward only. E-prop transports current downstream
weight signals; DFA uses fixed random terminal-error feedback. The matched
reference receives per-layer oracle target pulses only after the common forward
rollout. Every training arm executes those post-forward pulses; only the matched
reference learns from them.

Scientific schedule: 20 fresh seeds, depths 1–8, 2,000 train and 500 test
examples per depth×seed, D* floor 0.65.

| Arm | Held-out hash |
|---|---|
| broadcast three-factor | `c3v2-broadcast-three-factor-c97b35589d7791c1` |
| RPE three-factor | `c3v2-rpe-three-factor-c0469d6f35801d37` |
| e-prop postsynaptic | `c3v2-eprop-postsynaptic-d2b46d8fae0b8981` |
| DFA fixed feedback | `c3v2-dfa-fixed-feedback-9ad49817f517bead` |
| matched-forward oracle-gradient reference | `c3v2-matched-forward-oracle-gradient-reference-5582ce8e7c3d56a9` |

Run:

```bash
cargo run --release -p binn-lab --bin c3-production -- \
  --enable-c3-v2 --out results/c3_v2_production.md

# Development smoke only
cargo run -p binn-lab --bin c3-production -- \
  --enable-c3-v2 --quick --out results/c3_v2_production_quick.md
```

## Interpretation contract

- Epoch-matched broadcast > one-pass: exposure was material.
- Exact-forward gradient collapse: the earlier reference/front-end mismatch
  inflated the gap.
- RPE > matched broadcast: reward centering/scaling was material.
- E-prop/DFA > matched broadcast: neuron-specific credit is supported.
- C3 v2 broadcast reproduces a depth collapse while its matched reference
  remains capable: only then may the depth limit be attributed to the production
  learner under this protocol.
- None of these outcomes changes canonical C1 protocol-v2.

## Held-out results after freeze

The 20-seed schedules were executed after this protocol and its hashes were
frozen. Both paired suites passed topology, initialization, forward,
target-independence, exposure, and test non-update checks. A representative arm
hash for each paired suite reproduced its complete Markdown report byte for
byte; unit tests round-trip every arm hash.

Exact-forward C1:

| Arm | Mean accuracy | Normalized-gap LCB | G2 contract |
|---|---:|---:|---|
| broadcast one pass | 0.4912 | 0.0730 | FAIL |
| broadcast epoch matched | 0.6875 | 0.1327 | FAIL |
| RPE three-factor | 0.5688 | 0.0907 | FAIL |
| exact-forward e-prop | 0.6650 | 0.1502 | FAIL |
| exact-forward DFA | 0.5775 | 0.0909 | FAIL |
| exact-forward surrogate gradient | 0.6963 | 0.3263 | FAIL |
| dense epoch-matched control | 0.3500 | 0.0000 | FAIL |

Exposure was material: epoch matching raised broadcast accuracy by 0.1963.
RPE, e-prop, and DFA did not beat epoch-matched broadcast, so this experiment
does not support reward centering or neuron-specific credit as the missing
mechanism. The exact-forward surrogate reference was substantially below the
canonical continuous-front-end reference (0.6963 versus 0.8938), supporting a
material reference/forward-graph contribution, but it still did not close the
preregistered gap.

Production C3 v2:

| Arm | D* |
|---|---:|
| broadcast three-factor | 3 |
| RPE three-factor | 3 |
| e-prop postsynaptic | none |
| DFA fixed feedback | none |
| matched-forward oracle-gradient reference | 8 |

The production event engine therefore reproduces the broadcast learner's
depth collapse under C3 v2 while the matched oracle remains capable through
depth 8. This is evidence about `ThreeFactor` under this protocol; C3 v1 remains
only the preserved tabular proxy.

Artifacts: [`credit_assignment.md`](credit_assignment.md),
[`c3_v2_production.md`](c3_v2_production.md), and replay records under
[`runs/2026-07-23-credit-rereg/`](runs/2026-07-23-credit-rereg/).
