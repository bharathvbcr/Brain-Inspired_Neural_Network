# BINN-Hybrid

BINN-Hybrid is a separate successor experiment. It reuses BINN's sparse runtime
storage but does not change the interpretation of canonical BINN protocol v2:
`c1-118207fbc3eaba53` remains a valid G2 failure.

## Crate boundary

- `binn-hybrid-learn` is the teacher-free runtime. It owns the causal feature
  schema, checksummed frozen artifacts, topology validation, learned local
  updates, and reward-only online adaptation.
- `binn-hybrid-lab` is training-only. It owns terminal-loss teachers,
  finite-difference checks, factorization audits, distillation, seed schedules,
  and result rendering.
- A deployment crate may depend on `binn-hybrid-learn`. It must not depend on
  `binn-hybrid-lab`.

Successor evidence is written under `hybrid-results/` using independent
protocol hashes. Canonical files under `results/` are not successor evidence.

## H0 gate

H0 compares three upper bounds on identical sparse forward traces:

1. eligibility multiplied by the teacher's postsynaptic credit;
2. the least-squares postsynaptic credit that best reconstructs teacher edge
   deltas through the same eligibility;
3. direct per-synapse terminal-teacher deltas.

The terminal teacher receives only the final task label. It never receives true
intermediate states. Both C1 and C3 teacher gradients have finite-difference
tests.

The current H0 forward is explicitly a smooth matched sparse surrogate backed
by production `Engine` CSR and weight storage. It validates teacher
mathematics and interface expressivity, but cannot authorize a production
event-engine scientific claim.

Run the development pilot:

```text
cargo run --locked --release -p binn-hybrid-lab \
  --bin hybrid-feasibility -- --quick --out-dir hybrid-results
```

Protocol `binn-hybrid-h0-v3-caedeec1a47475a5` returned
`HYBRID_NO_GO`: direct per-synapse learning cleared the C1 surrogate but reached
only C3 D*=2 versus the preregistered D*>=6 gate. Consequently:

- no H1 student artifact was trained or frozen under protocol v3;
- the fresh held-out seed family was not evaluated;
- H2 continual-learning/consolidation work remains stopped;
- H3 composition/scaling/efficiency work remains stopped.

Changing the C3 task, teacher, thresholds, seed families, or update semantics
requires a new protocol version. A quick run remains development evidence only.

## Development-only diagnostic study

The robustness study is separately versioned and cannot change the frozen H0
decision. It uses a third seed family disjoint from H0 development, pilot, and
fresh held-out seeds.

Run the final full diagnostic:

```text
cargo run --locked --release -p binn-hybrid-lab \
  --bin hybrid-diagnostics -- --out-dir hybrid-results/diagnostics
```

Protocol `binn-hybrid-diagnostic-v3-fe72201b01e57cfe` swept:

- 20 deterministic development seeds;
- depths 1 through 8;
- 120, 480, 1,920, and 7,680 training examples;
- learning rates 0.002, 0.005, 0.015, 0.035, and 0.070;
- existing postsynaptic, least-squares postsynaptic, and direct terminal credit;
- privileged intermediate-target and shuffled-label controls;
- 1,000 frozen test examples per seed and condition.

Best observed development D* values were: existing postsynaptic `none`,
least-squares postsynaptic `3`, direct terminal `5`, privileged intermediate
target `8`, and shuffled label `none`. These are exploratory maxima selected
over the diagnostic grid, not confirmatory estimates.

Mechanistically, direct gradient steps reduced the same-example loss at every
depth while a rotated-gradient control increased it. Gradient norms grew rather
than vanished with depth. The evidence therefore points to deep compositional
identifiability/optimization and credit-structure loss, not an incorrectly
signed or numerically vanishing terminal teacher.

This study still uses the smooth matched sparse surrogate rather than hard
production event dynamics. It cannot authorize H1, H2, H3, or a scientific
BINN-Hybrid pass.

## Production event-engine diagnostic

Production protocol
`binn-hybrid-production-diagnostic-v3-f72033fbf6906b99` asks whether the
terminal-credit hierarchy survives real event dynamics. It preserves the
smooth diagnostic's shared residual transition parameters, but executes every
forward decision through:

- the production timing wheel and delayed CSR delivery;
- a weighted external event for the identity residual;
- membrane charge measurement;
- a hard k-WTA winner;
- production `ThreeFactor` STDP eligibility.

The terminal teacher is the central-finite-difference-checked gradient of the
same residual relaxation used by the smooth diagnostic. It receives only the
terminal label. Existing and least-squares postsynaptic credit are applied
through production eligibility; direct terminal updates act on CSR edges.

Run the full development diagnostic:

```text
cargo run --locked --release -p binn-hybrid-lab \
  --bin hybrid-production-diagnostics -- \
  --out-dir hybrid-results/production-diagnostics
```

The full 20-seed result did **not** reproduce the smooth D* hierarchy:
existing postsynaptic D*=1, least-squares postsynaptic D*=1, direct terminal
D*=1, privileged intermediate D*=8, and shuffled label `none`. At depth 2,
direct terminal reached mean 0.6230 with lower 95% bound 0.6172, above both
postsynaptic arms but below the 0.65 gate.

The negative is mechanically informative rather than a teacher failure.
Gradient norms increase with depth, direct steps reduce both the differentiable
teacher loss and rerun event loss, and rotated controls move loss in the wrong
direction. The production event discretization therefore introduces an
additional optimization mismatch beyond the edge-credit bottleneck observed in
the smooth diagnostic.

The odd/even raw accuracies require a disclosed task baseline. With zero
transition weights, the identity residual predicts the start state. Under the
two-operation C3 generator, that identity predictor has exact expected accuracy
0 at odd depths and 0.5 at even depths. Values near 0.5 at even depths are
therefore baseline behavior, not successful compositional learning.

Protocol v1 used layer-specific weights and protocol v2 used a different
softmax transition relaxation. Both were quick pilots, were rejected as
non-matched reproductions before the full run, and remain provenance only.

## Soft-to-hard winner-temperature ladder

Separately preregistered successor study. It cannot reverse H0, does not use
held-out seeds, and cannot authorize H1-H3.

Question: holding the matched soft residual terminal teacher fixed, at which
winner temperature do direct terminal gradients stop transferring into
trained accuracy under tempered / hard winner evaluation?

Contract:

- train with soft residual terminal gradients only (temperature-independent
  direct-terminal updates);
- evaluate the same weights on the ladder
  `soft`, `2.0`, `1.0`, `0.5`, `0.25`, `0.1`, `hard`;
- `soft` keeps linear residual states; finite `T` uses `softmax(scores / T)`;
  `hard` uses one-hot argmax;
- development seed family disjoint from H0, smooth diagnostic, production
  diagnostic, and unused held-out seeds;
- collapse temperature = softest ladder point whose direct-terminal D* falls
  strictly below the soft-endpoint D*.

Run:

```text
cargo run --locked --release -p binn-hybrid-lab \
  --bin hybrid-temperature-ladder -- \
  --out-dir hybrid-results/temperature-ladder
```

## T=2.0 ablation suite (successor)

Separately hashed ablation of the soft→hard collapse across **depth windows**,
**residual width (`n_states`)**, and **connectivity** (dense vs Bernoulli).
Hash family `binn-hybrid-winner-temp-ablate-v1-*`. Cannot reopen H0 or Gate G2.

```text
cargo run --locked --release -p binn-hybrid-lab \
  --bin hybrid-temperature-ablation -- \
  --out-dir hybrid-results/temperature-ablation
```

See `hybrid-results/temperature-ablation/SUMMARY.md`.

## Runtime artifact contract

`CreditHeadArtifact` records:

- feature schema version;
- topology signature;
- selected credit granularity;
- teacher protocol hash;
- training seed hash;
- head parameters, output scale, and checksum.

Artifact loading rejects schema, checksum, and topology mismatches. Runtime
prediction accepts only causal local features and an optional broadcast reward.
It has no label or teacher input.
