# Finding — a first sweep of BINN proper, and what it turned up

**Date:** 2026-08-22. `TODO_2026-08-07_OPEN_WORK.md` §6 records that roughly 8,000
lines of BINN proper had never been swept. This is the sweep.

**Verification key.** `[V]` verified by me, by running code, with the output
below. `[R]` read the cited lines myself and the mechanism follows from them.
`[U]` reported by a delegated sweep and **not independently verified** — listed so
it is not lost, not to be cited.

---

## 1. Two defects that reach a published number, both `[V]`

### 1.1 The `RateMatched` null is not rate-matched

`binn-engine/src/resting.rs:80-87` draws `total` uniform `(tick, cell)` pairs and
the shared `dedup()` at `:109-112` then deletes every within-tick collision. The
sibling `ActivityMatched` arm avoids exactly this by retrying — `while
out[tick].len() < …` with a `contains` check — and `RateMatched` does not. So the
null systematically emits **fewer** spikes than the raster it claims to match.

The test at `:219-247` asserts `sum(null) == sum(source)`, which is false in
general. It passes because it hardcodes seed 7. Measured on the module's own
6×6 fixture (total = 10):

```
RateMatched loses spikes for 52 of 64 seeds
  seed 7 (the hardcoded one): 10 of 10   <- passes
  seed 0: 8 of 10     seed 1: 8 of 10     seed 2: 9 of 10
```

**This is visible in the shipped table.** `results/u23_resting.md:13-16`:

| condition | mean activity |
|---|---:|
| observed | 0.0141 |
| **RateMatched** | **0.0140** |
| ActivityMatched | 0.0141 |
| SpectrumMatched | 0.0141 |

The two nulls that retry on collision match the observed rate exactly. The one
that does not is low, by the amount this bug predicts. **A published null does not
do what its name says, and the number that shows it was sitting in the table.**

### 1.2 The U08 locality gate cannot fail

`binn-areas/tests/determinism.rs:254-278` asserts `intra_area_edge_fraction >
0.90` at `p_inter = 0.001`. Measured, across all three roles at that setting:

```
p_inter=0.001  Sensory/Association/Hub:  edges=5040  inter=0  frac_intra=1.0000
```

**Zero long-range edges exist**, so the fraction is exactly 1.0 by construction
and the assertion has nothing to test. It would pass under a total inversion of
the role modulation.

The cause is `wiring.rs:155-179`: out-degree is `round(expected_degree)` applied
per cell, not a Bernoulli draw per pair, so any `p_inter` whose expected remote
degree is below 0.5 rounds to exactly zero. The step is stark:

```
p_inter=0.002   inter = 0 / 0 / 120     (role-dependent)
p_inter=0.0021  inter = 240 / 360 / 360
p_inter=0.01    inter = 600 / 720 / 960   frac_intra = 0.8936 / 0.8750 / 0.8400
```

`wiring.rs:43-45` calls these "edge probability". They are not probabilities in
the realized model.

And the claim itself is narrower than the gate implies: **at `p_inter = 0.01`
locality is 0.84–0.89, below the gate's own 0.90 bar.** The >0.90 result holds
only for `p_inter` at or below roughly 0.0021. That scope was never stated,
because the gate was never in a position to discover it.

## 2. Checks that cannot fail `[R]` unless marked

Each of these currently certifies something it cannot test.

- **`binn-learn/src/input_rate_control.rs:316-326`** `[U, mechanism confirmed by
  reading]` — the equivalence-gate test sets `labels == input_only == hidden`, so
  every resample difference is identically zero and `equivalent = true` is forced.
  This is the sole guard on the input-rate-shortcut equivalence claim.
- **`binn-learn/src/three_factor.rs:179-183`** — `without_eligibility` gates the
  decay term on `e.abs() > 1e-8`, so forcing `e ≡ 0` also kills `λ·w`. The
  ablation is "no update at all", not "no eligibility", and the test asserting
  `|w − w₀| < 1e-6` is satisfied by construction. No callers outside the module.
- **`binn-learn/src/scan_training.rs:159-161`** — three assertions in a row that
  cannot fail: `max >= min` over one collection, a fraction asserted to be in
  `[0,1]` when it is a ratio of a count to its own bound, and `scan_headroom`
  asserted against its own definition two lines up. `scan_headroom` is published
  in `results/u20_efficiency.md`.
- **`binn-data/src/metrics.rs:196-201`** — `honest_work` contains
  `synaptic_deliveries·delivery` plus non-negative terms and
  `naive_linear_activity_work(d/a, a)` is algebraically `d`, so `honest > naive`
  holds for every possible input.
- **`binn-engine/src/parallel.rs:153-177`** — the partitioned-vs-sequential parity
  test uses 5 cells against a threshold of 8, so the parallel branch never runs.
  A coverage gap, not a live bug.
- **`binn-areas/tests/determinism.rs:68-71`** — `u06_measured_activity_approx_k_over_n`
  compares `activity_sparsity` against `k/n`, but `ActivityLog::record` defines it
  as exactly that, and `k_wta` returns `min(k, #finite)` regardless of scores.

**One thing `scripts/find_weak_checks.py` structurally cannot see:** it classifies
`assert_eq!` as STRONG. An `assert_eq!` on a structurally-determined value —
`credit.rs:356` asserts `m.for_post(999) == m.for_post(0)` where `for_post` takes
`_post` and ignores it — is invisible to it. That is a gap in the tool, worth more
than any single instance it would find.

## 3. Silent failure `[R]`

- **`binn-learn/src/three_factor.rs:123-133`** — `absorb_spikes` resets its cursor
  only when `spike_cursor > spikes.len()`. After `Engine::reset_state()` clears
  the log, a new trial producing at least as many spikes leaves the guard unarmed
  and the learner silently skips that many, while `last_spike` still holds
  cross-trial pairing times. `reset_full_trial_state()` — the fix — has **zero
  callers in the workspace**. Latent: neither `reset_state()` call site uses
  `ThreeFactor`.
- **`binn-learn/src/credit.rs:96`** — `PostSynapticCredit::for_post` returns `0.0`
  for an out-of-range cell, while `set` panics and `FixedRandomFeedback::project`
  asserts. Only the read path fails open, so a credit vector sized for one
  population gives every edge outside it zero credit and the run reports success.
- **`binn-data/src/metrics.rs:110`** — `work_vs_activity_ratio` divides by
  `.max(1e-12)`, and `runner.rs:1636-1639` returns `activity_sparsity = 0.0` for a
  zero-event path. A dead arm therefore produces the largest activity≠compute
  ratio in the report. Adjacent, `runner.rs:1649`: `cell_updates.max(1)` makes a
  path that produced no events report one unit of work.
- **`binn-learn/src/input_rate_control.rs:251-253`** — `fold(NEG_INFINITY, f32::max)`
  skips NaN, so one NaN logit yields a NaN softmax sum that `.max(1e-12)` then
  discards, and the reported loss comes out a plausible finite number. The
  `.max(1e-12)` cannot bind for finite logits, so hiding NaN is its only effect.

## 4. Dead and unwired `[U]`

Reported for follow-up, not acted on — deleting needs two independent signals and
this list has one. Notable entries: the whole `binn-data::decoder` module (176
lines), `binn-areas::predictive`, `ContrastiveWakeSleepLearner`,
`batch_advance_euler` (exported, zero callers), and
`project::project_reference`, described as an oracle for a parity check that does
not exist.

The repo's own tooling found none of it: `dev map dead --confidence extracted`
reports no entries. That is worth knowing about the tooling.

## 5. What was checked and found clean

Recorded because a sweep that only lists problems misrepresents the codebase.
`binn-core/src/queue.rs` is the best-hardened code in the repository — the
occupancy-mask invariant is pinned by proptests after every operation against a
brute-force scan. `scan.rs` asserts bit-exact equality across six chunk sizes.
`sparse.rs`, `buffer.rs`, `rng.rs`, `consolidation.rs`, `transfer_bundle`,
`temporal_order` channel disjointness, `frame_events` count conservation and the
G1 gate in `binn-areas/tests/determinism.rs:92-161` all held up.

No long-sequence `f32` accumulation anywhere: the MSE loops and `total_work` are
`f64`.

## 6. Coverage, stated honestly

Read in full: all of `binn-core` (1,740 lines) and all of `binn-engine` (2,635),
plus about 2,400 of `binn-learn`. `binn-areas` and `binn-data` were read by
delegated sweeps with their top claims re-verified from source.

**Roughly 6,400 lines of `binn-learn` gradient and RL reference implementations
are still unswept** — `matched_rl_baseline.rs`, `shd_alif.rs`,
`matched_local_baseline.rs` and their siblings. That is the largest remaining
gap and it is the one that matters most, because those modules are the
*references* every `gap_closed` number is measured against. `binn-lab/src/`, where
most published numbers are actually assembled, is also unswept.

## 7. What is fixed in this change, and what is not

Fixed: §1.1 and §1.2 — the two that reach a published number, both verified by
measurement.

Not fixed, and deliberately: everything in §2 and §3. Each needs its own decision
about what the check should assert, several change published numbers, and doing
them in one sweep is how a repair becomes unreviewable. They are recorded here
with file and line so the next pass starts from evidence rather than from another
sweep.
