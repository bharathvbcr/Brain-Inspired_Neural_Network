# BINN-Hybrid research findings

## Evidence hierarchy

1. Canonical BINN protocol v2 remains a valid negative result.
2. BINN-Hybrid H0 protocol `binn-hybrid-h0-v3-caedeec1a47475a5` remains
   `HYBRID_NO_GO` with terminal-teacher D*=2 under its frozen development
   budget.
3. Diagnostic protocol `binn-hybrid-diagnostic-v3-fe72201b01e57cfe` is the
   authoritative robustness study. It is development-only and cannot reverse
   H0.
4. Production protocol
   `binn-hybrid-production-diagnostic-v3-f72033fbf6906b99` is the
   authoritative hard-event reproduction. It is also development-only.
5. Winner-temperature ladder
   `binn-hybrid-winner-temp-v1-fa7710de68ad7bfe` is the authoritative
   soft-to-hard transfer localization. It is separately preregistered,
   development-only, and cannot reverse H0.
6. Earlier diagnostic and production-pilot versions are retained only for
   provenance.

## Full diagnostic design

- 20 deterministic seeds disjoint from every H0 seed family.
- Depth 1 through 8.
- Training budgets: 120, 480, 1,920, and 7,680 examples.
- Learning rates: 0.002, 0.005, 0.015, 0.035, and 0.070.
- 1,000 frozen test examples per seed and condition.
- 11,200 seed-level sweep rows and 160 seed-level mechanism rows.
- No test-time weight changes.

The grid is exploratory. Best configurations and D* values are selected on the
same development seeds, so they are descriptive upper observations rather than
unbiased confirmatory estimates.

## Main results

| arm | best observed D* | interpretation |
|---|---:|---|
| existing postsynaptic credit | none | the current credit rule never clears the depth floor |
| least-squares postsynaptic credit | 3 | even the optimal postsynaptic factorization loses important edge structure |
| direct terminal gradient | 5 | richer per-edge credit helps materially but remains unreliable at depth 6–8 |
| privileged intermediate targets | 8 | the task/model can solve every depth when given inadmissible stepwise supervision |
| shuffled labels | none | success is not explained by leakage or generic weight motion |

At the direct-terminal optimum, the paired direct-minus-postsynaptic advantage
has a positive lower 95% bound at depths 3, 4, 5, 7, and 8. At depth 6 its mean
is positive but the lower bound crosses zero.

## Mechanistic result

The failure is not explained by a broken or vanishing teacher:

- direct gradient steps reduce same-example loss at every depth;
- rotating the same deltas increases loss at every depth;
- mean gradient norm rises from approximately 0.98 at depth 1 to 5.50 at depth
  8;
- the least-squares postsynaptic reconstruction retains high cosine similarity
  but still loses enough structured information to reduce trained accuracy.

Together with the D*=8 privileged ceiling, the strongest supported explanation
is deep compositional identifiability/optimization plus loss of edge-specific
credit structure. This is an inference from convergent controls, not proof of a
unique cause.

## Production event-engine result

The production study repeated the same 20-seed, depth 1-8, four-budget,
five-learning-rate, 1,000-test-example design, producing 11,200 seed-level
sweep rows and 160 mechanism rows. Forward execution used the real timing
wheel, synaptic delivery, membrane state, identity-residual event, hard k-WTA,
and production STDP eligibility. All arms shared topology, initialization,
examples, and target-independent forward traces; no test weights changed.

| arm | smooth D* | production D* |
|---|---:|---:|
| existing postsynaptic credit | none | 1 |
| least-squares postsynaptic credit | 3 | 1 |
| direct terminal gradient | 5 | 1 |
| privileged intermediate targets | 8 | 8 |
| shuffled labels | none | none |

The full hierarchy therefore did **not** reproduce. Direct terminal credit
retained a real relative advantage at depth 2 (mean 0.6230, lower 95% 0.6172)
and had positive paired lower bounds over least-squares credit at depths 2, 3,
5, and 7, but it never cleared the 0.65 gate beyond depth 1.

This is not explained by a broken terminal teacher:

- mean gradient norm increases from 0.9838 at depth 1 to 4.7351 at depth 8;
- direct steps reduce the smooth terminal loss at every depth;
- the same steps also reduce event loss after a fresh hard-forward rerun;
- rotated directions increase both losses on average;
- privileged intermediate supervision solves every depth;
- shuffled terminal labels never pass.

Production eligibility is extremely selective: nonzero support rises only from
about 3.1% of edges at depth 1 to 6.2% at depth 8. Depending on depth parity,
eligible edges contain only about 23%-50% of terminal-gradient energy. The
least-squares postsynaptic oracle lowers reconstruction MSE but cannot recover
updates on edges with zero eligibility.

The strongest updated interpretation is two bottlenecks:

1. eligibility times postsynaptic credit discards edge-specific terminal
   information;
2. even direct gradients of the matched smooth relaxation transfer poorly
   across hard per-step winner discretization.

The task has an exact parity baseline that prevents naive chance
interpretation. Zero transition weights yield an identity predictor with
expected accuracy 0 at odd depths and 0.5 at even depths. Production values
near 0.5 at even depths are baseline behavior.

## Winner-temperature transfer ladder

Protocol `binn-hybrid-winner-temp-v1-fa7710de68ad7bfe` holds the soft residual
terminal teacher fixed and evaluates the same trained weights under a
soft→hard winner ladder. Direct-terminal updates are temperature-independent;
only evaluation winners change.

| temperature | direct-terminal D* |
|---|---:|
| soft | 5 |
| 2.0 | 2 |
| 1.0 | 2 |
| 0.5 | 2 |
| 0.25 | 2 |
| 0.1 | 2 |
| hard | 1 |

Transfer collapse temperature is **2.0**: the softest ladder point at which
direct-terminal D* falls strictly below the soft endpoint. Soft recovers the
smooth-diagnostic D*=5; hard recovers the production-style D*=1. Intermediate
softmax winners retain only D*=2. Privileged intermediate targets remain
solvable through hard (D*=8); shuffled labels never pass.

Mechanistically, soft-teacher direct steps still reduce soft loss at every
depth, while the same deltas yield much smaller tempered-forward loss drops
once winners are softmax/hard. The failure is therefore localized to winner
discretization transfer, not a vanishing or sign-flipped teacher.

## Scientific limits

- The first diagnostic uses a smooth matched sparse surrogate. The production
  reproduction uses hard event forward execution, but its teacher remains a
  disclosed differentiable relaxation because hard k-WTA has no ordinary
  derivative.
- The temperature ladder uses development seeds and hyperparameter selection
  on a residual-score substrate; finite temperatures are softmax winners, not
  production event dynamics.
- It uses development seeds and hyperparameter selection.
- The privileged ceiling has up to `depth` times the supervision and update
  magnitude and is not budget matched.
- No H1 artifact was trained under the frozen H0 protocol.
- Fresh held-out seeds remain unused.
- H2 and H3 remain stopped.

## Artifacts

- Full report:
  `diagnostics/binn-hybrid-diagnostic-v3-fe72201b01e57cfe.md`
- Seed-level sweep:
  `diagnostics/binn-hybrid-diagnostic-v3-fe72201b01e57cfe-sweep.csv`
- Mechanism rows:
  `diagnostics/binn-hybrid-diagnostic-v3-fe72201b01e57cfe-mechanisms.csv`
- Production report:
  `production-diagnostics/binn-hybrid-production-diagnostic-v3-f72033fbf6906b99.md`
- Production sweep:
  `production-diagnostics/binn-hybrid-production-diagnostic-v3-f72033fbf6906b99-sweep.csv`
- Production mechanisms:
  `production-diagnostics/binn-hybrid-production-diagnostic-v3-f72033fbf6906b99-mechanisms.csv`
- Temperature-ladder report:
  `temperature-ladder/binn-hybrid-winner-temp-v1-fa7710de68ad7bfe.md`
- Temperature-ladder sweep:
  `temperature-ladder/binn-hybrid-winner-temp-v1-fa7710de68ad7bfe-sweep.csv`
- Temperature-ladder mechanisms:
  `temperature-ladder/binn-hybrid-winner-temp-v1-fa7710de68ad7bfe-mechanisms.csv`

SHA-256:

- report: `d40f9b2c240db39cdaf5ff3dfc84459ff22e227a7f84f182bff35c86b13ae1ea`
- sweep: `8119d5de787df6b3027be1c65b6812a6835145905c3b3e0a998c6c61286c6f24`
- mechanisms: `6f693ef4c75c0fd1ba6a8d57711b43c4dd3e9f83e594023d27e4a4692d17918d`
- production report:
  `ad6583d16000b5f533bb0cff15a6c4c256c1e80191bfff7719b64c449f56f084`
- production sweep:
  `0a19cbcddb3484139a10a08e19f991051a14c10247c7a39dbdf9597d218c3df5`
- production mechanisms:
  `52af2aea51271113f6e4b8cfb38a474fce0c19f8328ef18fba4c53b917f719e7`
- temperature-ladder report:
  `19bde1628cd82b3ce7c50a209db9208d335b623fcda952f4e5d6a9b1057eca57`
- temperature-ladder sweep:
  `b167949474138050e4f2772aa4d0cf4e401c289c2b35363b374e9f2f16d78d1d`
- temperature-ladder mechanisms:
  `738924208eaed352b7a7c76a00fe11c3f02859b48d548c737e3be0919a37478f`
