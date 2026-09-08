# Measurement — the transfer-gap decomposition has no task to run on

**Date:** 2026-09-07
**This is a measurement, not a verdict.** No hypothesis is evaluated, no wave's
status changes, and no number here is a result about BINN. It answers one
precondition question with data.
**Bears on:** [`PREREG_2026-09-07_THE_TRANSFER_GAP_DECOMPOSITION.md`](PREREG_2026-09-07_THE_TRANSFER_GAP_DECOMPOSITION.md) §4 and clause T-4
**Instrument:** `c1 --matched-arch / --matched-dfa / --matched-rl --max-lag N`,
added in the same change as this document

---

## 1. The question, and why it was worth measuring rather than assuming

The decomposition registration makes one thing non-negotiable before any of its
160 cells may run:

> **The dense pole must score in 0.7–0.9.** If no available task puts it there,
> this wave does not run and the registration says so rather than lowering the
> requirement.

and clause **T-4** turns that into a gate: outside the band, every other clause
reads NOT EVALUABLE.

The registration named the problem — `CoincidenceTask` at `N_IN = 2` saturates
the matched gate at exactly 1.0000 with zero variance — but it did not establish
that *no* configuration of that task escapes the ceiling. Nobody had looked.
Until somebody did, "the wave is blocked on a task" was an assumption, and the
decomposition sat open behind it.

## 2. `max_lag` is the only knob, and that is a fact about the source

Read from `binn-data/src/datasets.rs` on 2026-09-07 rather than assumed:

- **`n_features: 2` is hard-coded** in `CoincidenceTask::new`. `N_IN = 2` is not
  a setting.
- **`sequence_len` is pinned to 8.** It is `REFERENCE_SEQUENCE_LEN`, a
  compile-time `const` with `T` baked into **30 fixed-size arrays** across
  `bptt_baseline.rs` and `matched_local_baseline.rs`, and it is asserted at
  `runner_match.rs:53` and `runner_dfa_match.rs:61`. A `--sequence-len` flag was
  written for this measurement and **removed**: every value but 8 printed a
  config hash and then panicked the runner, which is worse than no flag.
- **`difficulty: 0.05` and `depth: 1` are inert for this task.**
  `CoincidenceTask::new` hard-codes both, and `next_trial` then overwrites
  `s.values` and `s.label` for **every** frame it draws — and `Sample` has
  exactly those two fields (`binn-data/src/encoder.rs:22`). The synthetic stream
  contributes a length and a deterministic index; not one output value survives
  it. Raising `difficulty` could not change a single number below.

That leaves **`max_lag`**, and `sequence_len = 8` caps it at 6 (the task needs
`len ≥ max_lag + 2` to construct a negative, and silently raises the sequence
otherwise — the new flag refuses instead). Positives carry lag `0..=max_lag`,
negatives `max_lag+1..=7`, so raising `max_lag` moves the decision boundary out
of the tail and into the bulk of the lag distribution. **The whole reachable
knob space is six values.** It was swept exhaustively.

## 3. The sweep

All at the scientific schedule (n = 20 seeds), on `aarch64-apple-darwin`. Each
row is its own config hash; `max_lag 1` reproduces the archived hashes
(`c1-match-6f6000f148f7d30c`, `c1-dfa-f79c01ea36fe27d7`,
`c1-rl-d35e13c758e522f8`) and the archived numbers, which is what says the
instrument is the one the record was made with.

Each suite runs its own gradient reference on its own seeds, so the three are
tabulated separately rather than pooled into one "ceiling" column.

**`--matched-arch`** — the gate whose saturation the registration names:

| `max_lag` | gradient (ceiling) | broadcast ±1 (`matched-local`) |
|---:|---:|---:|
| 1 | 1.0000 | 0.5100 |
| 2 | 1.0000 | 0.5037 |
| 3 | 1.0000 | 0.5000 |
| 4 | 1.0000 | 0.5000 |
| 5 | **0.9750** | 0.5000 |
| 6 | 1.0000 | 0.5000 |

**`--matched-dfa`** and **`--matched-rl`** — the arms that pass G2:

| `max_lag` | DFA | broadcast-graded | its gradient | REINFORCE×frozen-`B` | its gradient |
|---:|---:|---:|---:|---:|---:|
| 1 | 0.9925 | 0.9975 | 1.0000 | 0.9950 | 1.0000 |
| 3 | 0.9975 | 1.0000 | 1.0000 | 0.9887 | 1.0000 |
| 5 | 0.9825 | 0.9625 | 0.9788 | 0.9312 | 0.9788 |
| 6 | 0.9875 | 0.9800 | 0.9988 | 0.9750 | 1.0000 |

**The ceiling never leaves saturation.** Its minimum across all three suites and
all six rungs is **0.9750** (`--matched-arch`, lag 5), and it is **not
monotone** — lag 6 returns to 1.0000 in that same suite — so the dip is seed
noise, not a difficulty trend. Every arm that passes Gate G2 stays at **0.93 or
above** everywhere. The broadcast ±1 arm is at chance everywhere, which is the
G2 FAIL and is below the band rather than in it: a pole at chance has no room to
be moved *down* by a factor flip either, so it is no more usable than a ceiling.

### The one thing that did land in the band, and why it does not count

The RL suite's two **control** arms do reach 0.7–0.9:

| `max_lag` | `rl_graded` | `rl_flat` |
|---:|---:|---:|
| 1 | **0.8787** | **0.7775** |
| 3 | 0.9150 | **0.8062** |
| 5 | 0.9350 | **0.7975** |
| 6 | 0.9525 | **0.8438** |

They are in the band **by construction, not by task difficulty**: they are the
degraded-feedback controls the RL suite runs precisely so that the primary arm
has something to beat, and their scores barely move across the whole lag ladder
(`rl_flat` spans 0.0663 over six rungs). Building the decomposition's dense pole
out of one of them would mean decomposing the transfer gap of a rule that is
neither C1's canonical rule nor a G2 survivor — measuring a control's
sensitivity to four factors and reporting it as the substrate's. That is the
same substitution T-4 exists to prevent, arriving from the other side.

## 4. What this establishes

**T-4 cannot be satisfied on any task this repository has.** There are two
tasks in the workspace — `CoincidenceTask` and `CreditDepthTask` — and the
matched suites are wired to the first; its only reachable knob has been swept
end to end and does not move the pole into the band.

Per the registration's own §7, that is a **named outcome**:

> **A precondition in §3 unbuildable** — Reported as NOT EVALUABLE with the
> reason.

So the decomposition wave is **NOT EVALUABLE on the task precondition**, for a
reason that is now measured rather than suspected, and **its 160 cells are not
scheduled**. This costs nothing that was going to be spent: the wave was already
blocked on two instrument preconditions (`trial_isolation` split into two
switches, and an all-units-update selection mode) that were never built. Neither
should be built for this wave until a task exists — an instrument built for a
wave that cannot run is the more expensive mistake.

## 5. What it does not establish

- **Not that the decomposition is a bad design.** The four factors and the
  16-rung lattice are unaffected. What is missing is a task, and §3.3 of the
  design said so before this measurement existed.
- **Not that no task could exist.** A task with more than two input channels, or
  a sequence longer than 8, would very likely put the pole in band. Both are
  code, not configuration: `n_features` is hard-coded and `T` is compile-time
  across 30 arrays, and moving `REFERENCE_SEQUENCE_LEN` would change the
  gradient reference for **every** archived G2 number. That is a decision about
  the whole matched programme, not a knob turn, and nothing here authorises it.
- **Not a G2 result.** G2 stays closed. `max_lag 1` reproduces the archived
  numbers and no archived hash moved; the five other rungs are new hashes that
  no claim rests on.
- **Not platform-corrected.** These were produced on Apple libm, where the
  archived matched numbers were also produced
  ([`FINDING_2026-08-19_LIBM_PORTABILITY_OF_REPLAY.md`](FINDING_2026-08-19_LIBM_PORTABILITY_OF_REPLAY.md)),
  so the lag-1 agreement with the record is a same-platform comparison and is
  meaningful as one.

## 6. Reproducing it

```
cargo build --release --locked -p binn-lab --bin c1
for lag in 1 2 3 4 5 6; do ./target/release/c1 --matched-arch --max-lag $lag; done
for lag in 1 3 5 6; do ./target/release/c1 --matched-dfa --max-lag $lag; done
for lag in 1 3 5 6; do ./target/release/c1 --matched-rl  --max-lag $lag; done
```

Omitting `--max-lag` reproduces the archived hash exactly, on all three suites.
That is asserted rather than hoped: the flag is applied after the preset
resolves, at the same seam `--matched-forward` uses.
