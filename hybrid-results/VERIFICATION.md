# BINN-Hybrid verification snapshot

## Passed

- `cargo test --locked -p binn-hybrid-learn -p binn-hybrid-lab`
  - 23 lab tests
  - 6 runtime tests
  - 1 deployment-boundary integration test
- `cargo clippy --locked -p binn-hybrid-learn -p binn-hybrid-lab --all-targets -- -D warnings`
- package-scoped Rust formatting
- `git diff --check`
- canonical `./scripts/gc_checks.sh` (GC1 through GC7)

The hybrid tests cover:

- C1 and C3 finite-difference gradient checks;
- gradient-versus-rotated-direction loss causality;
- least-squares factorization optimality;
- artifact round-trip, tamper, duplicate-field, unknown-field, schema, checksum,
  and topology rejection;
- topology invariance to weight values and sensitivity to edge changes;
- deterministic data, protocol hashing, exact experiment replay, and CSV replay;
- disjoint diagnostic and frozen H0 seed families;
- no test-time weight updates;
- no label influence on forward traces;
- no H1 artifact or student arm after H0 `HYBRID_NO_GO`;
- no training-lab dependency in the deployment runtime.

Production-event additions cover:

- event scores equal the shared residual transition calculation;
- the residual is delivered through the production event queue;
- captured eligibility exactly equals sparse production STDP state;
- the existing arm applies eligibility times postsynaptic credit through
  `ThreeFactor`;
- production terminal gradients match central finite differences;
- direct gradients beat rotated directions in both teacher and event reruns;
- production forward is deterministic and terminal-label independent;
- production seeds are disjoint from H0, smooth-diagnostic, and unused
  held-out families;
- exact quick report/CSV replay and no test-time updates.

Winner-temperature ladder additions cover:

- soft endpoint matches `C3CompositionModel` predictions;
- hard endpoint is the one-hot argmax winner chain on residual scores;
- ladder seeds are disjoint from H0, smooth-diagnostic, production-diagnostic,
  and unused held-out families;
- exact quick report/CSV replay and no test-time updates.

Full production artifacts contain 11,200 sweep rows and 160 mechanism rows.
An independent full replay under `production-diagnostics-replay/` completed in
528.87 seconds wall time and matched all three authoritative artifacts
byte-for-byte:

- report:
  `ad6583d16000b5f533bb0cff15a6c4c256c1e80191bfff7719b64c449f56f084`
- sweep:
  `0a19cbcddb3484139a10a08e19f991051a14c10247c7a39dbdf9597d218c3df5`
- mechanisms:
  `52af2aea51271113f6e4b8cfb38a474fce0c19f8328ef18fba4c53b917f719e7`

Full winner-temperature ladder artifacts contain 16,800 sweep rows and 1,120
mechanism rows under protocol
`binn-hybrid-winner-temp-v1-fa7710de68ad7bfe` (release wall time ≈ 8.45 s):

- report:
  `19bde1628cd82b3ce7c50a209db9208d335b623fcda952f4e5d6a9b1057eca57`
- sweep:
  `b167949474138050e4f2772aa4d0cf4e401c289c2b35363b374e9f2f16d78d1d`
- mechanisms:
  `738924208eaed352b7a7c76a00fe11c3f02859b48d548c737e3be0919a37478f`

## Concurrent workspace findings

The broader workspace was changing concurrently in canonical C3-BPTT/e-prop
work. Read-only verification found:

- full workspace tests: one deterministic failure in
  `matched_local_baseline::tests::matched_local_actually_updates_weights`
  because the tested readout weights did not move;
- full strict Clippy: unrelated `needless_range_loop` findings in
  `binn-lab/src/runner_c3_bptt.rs` and two locations in
  `binn-lab/src/runner.rs`;
- workspace-wide format check: unrelated formatting differences in the same
  concurrent canonical work.

Those files were not modified by the BINN-Hybrid work. The failures are
recorded rather than hidden or automatically reformatted over concurrent edits.
The repository-map regeneration command was unavailable in this environment;
source and the existing map were not rewritten to conceal that limitation.
