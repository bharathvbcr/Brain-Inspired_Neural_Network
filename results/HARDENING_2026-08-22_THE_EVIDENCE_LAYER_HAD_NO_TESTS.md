# Hardening round 2 — the layer that produced every number had no tests

**Date:** 2026-08-22.
**Companion to** [`HARDENING_2026-08-21_CEILING_HEALTH_HAS_ONE_OWNER.md`](HARDENING_2026-08-21_CEILING_HEALTH_HAS_ONE_OWNER.md),
which hardened the *source*. This round hardens the **evidence**.

---

## 1. The gap

`scripts/aws/` planned, scheduled, ran, collected and analysed **720 cells**, and
every number in the attention campaign came through it. Until today it had
**no tests at all**.

That is the same shape as the defect found yesterday — a gradient reference
nobody verified could learn — moved one layer out. The code that *generates* the
evidence was itself unexamined.

## 2. Issue 1 — a cell does not record which seed produced it

The emitted cell JSON carries **every** parameter except one:

```
arm, contract, geometry, hidden, epochs, n_train, n_test, attn_dim,
attn_layers, surrogate_scale, clip_grad_norm, temporal_condition, …
```

**There is no `seed` field.** The seed lives only in the filename.

This matters because the campaign's headline statistics are **paired per seed** —
"positive in 12 of 12 seeds", M-1's per-seed deltas, S-2's seed-consistency bar.
Those pairings assume `…_s5170003.json` for the treatment and for the control came
from the same seed, and **nothing inside either file can confirm it**. A
mislabelled result would leave the means unchanged and every paired statistic
silently wrong, with no gate firing.

### Why it cannot simply be added

`run_cell.py` passes `--seed` to the `init` subcommand only. `train-cell` — the
step that emits the cell — receives `--weights` and `--orders`, and the weights
format (`SHDWGT2/3`) carries no seed. **The training step genuinely cannot label
its own output.** Fixing it properly means either a `--seed` provenance
passthrough (a label that could itself be wrong) or a weights-format change
(a magic bump that invalidates existing weights). Neither is a change to make
while wave 10 is in flight on the pinned binary.

### What was done instead: verify the record empirically

| check | result |
|---|---|
| every planned id encodes its own seed, width, epochs, contract, geometry | **420 / 420 cells, 0 mismatches** (waves 8, 9, 10 and the Azure plan) |
| any two archived cells with identical content under different names | **624 cells scanned, 0 collisions** |
| the id threads through claim → plan lookup → work dir → upload as one variable | verified in `bootstrap.sh` and `run_cell.py` |

Zero content collisions is the load-bearing one: it means the seed genuinely
determines the output and no result was duplicated across names. Combined with
Gate F — which re-runs recorded cells *from their spec* and gets bit-identity —
the spec→content mapping is confirmed for the existing record.

**So the record is clean. The property is simply not self-evident from a cell
alone**, and that is now stated rather than assumed. Adding the seed to the
emitted JSON is the correct fix and is **left as named future work**, to be done
between campaigns rather than during one. Gate F compares named fields and
tolerates absent ones, so the addition will not break it.

## 3. Issue 2 — no test could distinguish a good plan from a broken one

`scripts/test_campaign_tooling.py`, **15 tests**, mostly negative. It pins the
invariants that the paired statistics rest on:

- **every id encodes its own seed / width / budget / contract / geometry** — the
  property §2 could not otherwise check;
- **ids are unique within every wave**, and an attention cell can never share an
  id with a rate-only cell, or an intact cell with a shuffled one;
- **`estimated_seconds` rises with every cost driver** — a sign error there only
  mis-orders the queue, but that has already happened once, burying the cheapest
  decision-relevant wave at index 336 of 468;
- **each validity gate catches its own defect class**, including the exact AZ8-6
  shape (9 classes predicted, 83% in one of them);
- **a drifted reused control is refused** — negative test against a hash that
  cannot match;
- **all 528 archived wave-1 cells still match their recorded hashes.**

## 4. Issue 3 — nothing checked the numbers actually printed in the papers

The analyser computes verdicts; a human transcribes them into markdown.
**Neither step checks the other.** A bug in the analyser would be faithfully
transcribed, and a transcription slip would never be caught by re-running the
analyser — it would agree with itself forever.

`scripts/verify_published_numbers.py` closes that loop. It reads numbers **out of
the published result documents** and recomputes them **from the cell JSON with an
implementation that shares no code with the analyser**.

```
[ok] S-1 channels-700 mean          computed +0.7864  published +0.7864
[ok] S-2 channels-700 gain          computed +0.1090  published +0.1090
[ok] S-3 h1024 gain                 computed -0.1618  published -0.1618
[ok] S-4 published-10ms gain        computed +0.1491  published +0.1491
[ok] S-6 L2 mean                    computed +0.7897  published +0.7897
[ok] M-1 intact - shuffled          computed +0.1337  published +0.1337
[ok] M-2 shuffle cost, plain arm    computed +0.0128  published +0.0128
[ok] headline d32/L4 mean           computed +0.8320  published +0.8320
[ok] M-3 d64 - d32                  computed +0.0121  published +0.0121
[ok] headline gain                  computed +0.1258  published +0.1258
[ok] M-1 seeds intact > shuffled    computed 12/12    published 12/12
[ok] headline seeds >= 0.80         computed 12/12    published 12/12

12/12 published numbers reproduce from the cells.
```

**Negative-tested, not assumed.** Changing `+0.0121` to `+0.0221` in one document
produced `[FAIL] M-3 … computed +0.0121 published +0.0221`, `11/12`, exit 1;
restoring it returned 12/12 and exit 0. A checker that has never failed is not
evidence that anything passed.

## 5. One command, so none of it depends on being remembered

`scripts/record_checks.sh` — the counterpart to `gc_checks.sh`. That one proves
things about the **source**; this proves things about the **evidence**:

```bash
bash scripts/record_checks.sh
```

It runs the tooling invariants, the published-number reproduction, and the
weak-check scan, and exits non-zero on any failure with:
*"do not cite a number until this is green."*

## 6. Verification

`fmt` clean · workspace clippy clean · GC1–GC7 pass · Rust suite green ·
record checks green (15 tests, 12/12 numbers, weak-check scanner calibrated).

The weak-check scanner still reports 3 hits, all triaged and none defects:
`trains_at_every_depth_without_panicking` (documented as deliberately weak, with
the finding it belongs to), `a_fully_saturated_trace_stays_finite_through_both_passes`
(a robustness test, where finiteness *is* the property), and one in
`binn-hybrid-lab`, off the paper's path.

## 7. Round 2b — the four remaining gaps, closed

### The seed is now recorded, opt-in so nothing recorded breaks

`train-cell` accepts `--seed` as a **provenance label that does not touch the
computation**, and emits `"seed": N`. It is deliberately optional: omitting it
produces **byte-identical** output to before the flag existed, verified by
**Gate F 12/12 bit-identical** on the new binary. `run_cell.py` now passes it, and
`analyse_wave8.load()` refuses any cell whose recorded seed disagrees with its
filename — older cells have no seed field and their absence is not a failure,
only a missing witness.

### A near-miss worth recording: strict flags nearly killed every attention cell

The parser found flags by searching, so an unrecognised one was **silently
ignored** — a typo in `--attn-dim` or `--surrogate-scale` would run the cell at
the default and emit a wrong-but-plausible result. `reject_unknown_flags` now
refuses it by name.

**The first version of that guard omitted `--arm`**, which is parsed in a shared
helper and so never appeared in a per-function grep of the flag literals. It would
have refused **every attention cell in the next campaign**. Gate F passed anyway,
because no recorded regression cell uses an attention arm — the guard's own blind
spot was invisible to the strongest check in the workspace.

It was caught by running the real campaign invocation by hand. That is now a test:
`every_real_campaign_invocation_is_accepted` pins the exact shapes `run_cell.py`
emits, and both allow-lists were extracted into `INIT_FLAGS` / `TRAIN_CELL_FLAGS`
so the subcommand and the test cannot drift apart.

### The claim protocol is tested, and the tests are mutation-checked

Five tests on `claim_next.py`'s decision logic, driven by substituting the
subprocess boundary: a transient list failure **raises rather than looking like an
empty queue**, finished and held cells are skipped, losing a race moves on rather
than giving up, a drained queue prints nothing, and a claimed cell is never handed
out twice.

**Mutation-tested, not assumed.** Making a list failure return empty broke
exactly one test; removing the `held` check broke exactly two. Both mutations
reverted.

### `MatchedDeepGradient` is diagnosed

Root cause found and two plausible hypotheses killed by measurement — see
[`FINDING_2026-08-22_MATCHED_DEEP_GRADIENT_COLLAPSES_TO_SILENCE.md`](FINDING_2026-08-22_MATCHED_DEEP_GRADIENT_COLLAPSES_TO_SILENCE.md).
Three characterization tests now **assert the broken behaviour**, so a repair
fails the suite and must be registered rather than slipped in.

## 8. Round 2c — the last named gaps

### `ShdEpropCeiling` diagnosed, and it is **not** the other mechanism

It predicts **one class for every sample** — `distinct preds 1`, `majority 60/60`,
in every configuration — so its reported "accuracy" is only that class's frequency
(0.1000 / 0.2000 / 0.2667, tracking class balance exactly). `ShdSuperSpikeCeiling`
reaches **1.0000** on the identical fixtures, so the data and the forward are fine.

Crucially the mechanism **differs** from `MatchedDeepGradient`: there the network
fell silent; here the modulator is non-zero and varies with the data, so credit
flows and the readout collapses anyway. Generalising the first diagnosis would
have produced a wrong answer. Three characterization tests pin it, one of which
fails if the mechanism ever becomes the silence one — so the two cannot be
conflated later.
See [`FINDING_2026-08-22_SHD_EPROP_CEILING_IS_A_CONSTANT_PREDICTOR.md`](FINDING_2026-08-22_SHD_EPROP_CEILING_IS_A_CONSTANT_PREDICTOR.md).

### `collect.py` and `teardown.py` tested, and mutation-checked

Four tests on the two failure modes that are silent rather than loud: a paginated
listing that stops at page one **undercounts a finished campaign**, and a teardown
filter that is wrong either spares a burning fleet or destroys something that is
not ours.

Pinned: pagination is followed; an AWS failure raises rather than reporting zero;
`describe-instances` is filtered by the campaign tag and `terminate` receives
exactly what it returned; and **the results bucket is never deleted**.

**Mutation-tested.** Removing the tag filter broke exactly one test; making
`collect` stop after page one broke exactly one test. Both reverted.

### The paper draft no longer carries a withdrawn claim

The abstract was rewritten around what survived: the withdrawn v130 PASS is gone,
the `live-transfer-rescue` numbers are gone, and **the A6 caveat is now a section
of the body** (§3.5) rather than an omission — the 80-epoch schedule undertrains
every rule on it, the reference rises 0.9013 → 1.0000 by e640, `gap_closed` is not
ceiling-normalised, and the ordering at the canonical budget is a statement about
**learning speed**.

`verify_published_numbers.py` now guards the draft itself: it fails if the draft
resurrects a withdrawn claim, or if the A6 caveat disappears. **Negative-tested** —
removing the caveat heading drops the run to 15/16 with exit 1.

## 9. Round 2d — the mechanism claim's own dependency, audited

**M-1 — the paper's mechanism claim — rests entirely on the bin-shuffle actually
destroying temporal order while preserving per-channel counts.** If the shuffle
were a no-op, or destroyed counts, M-1 would be measuring something else and every
validity gate would still pass, because the gates read the audit rather than the
data.

So the audit was checked rather than trusted. **It is already sound**, and this is
recorded as a gap that does not exist:

- `binn-learn/src/shd_temporal.rs` carries **8 tests**, including
  `every_condition_preserves_channel_counts`,
  `non_identity_conditions_actually_move_spikes` (asserting
  `relocated_fraction > 0.5` and `mean_bin_displacement > 1.0`),
  `bin_shuffle_preserves_within_bin_synchrony_channel_shuffle_does_not`, and
  `reversed_is_its_own_inverse`.
- The audit is **negative-tested**: `intact_is_a_no_op` asserts
  `relocated_fraction == 0.0`. So a shuffle that silently did nothing would report
  0.0 and fail the campaign's `>= 0.5` gate rather than scoring as a valid
  control.

The doc comment at `shd_temporal.rs:95` also records a subtlety worth keeping:
`counts_preserved` defaults to **true**, because a `false` default on an empty
merge fold would report a violation no matter what the samples did.

### `ShdDfa` — the caution I recorded is now discharged

`FINDING_2026-08-22_SHD_EPROP_CEILING_IS_A_CONSTANT_PREDICTOR.md` §3 declined to
call DFA's collapse a defect, because the sweep reported 1.0000 under a different
configuration. Both candidate explanations are now ruled out by measurement:
**width** (16–128, including the sweep's 64) and **budget** (1–60, spanning the
sweep's 30). Every arm starts at the same constant predictor; SuperSpike escapes
it by epoch 20 and neither local arm ever does.

What remains unexplained is the sweep's own number, and it **cannot** be
explained: `shd-scientific-sweep` is refused by `authorize_campaign` while the
instrument is `Uncalibrated`, so it cannot be re-run. That is recorded as an
honest terminus rather than a resolution.

## 10. Scope

- **Verified:** every count and result above, this session, on this machine.
- **Verified falsifiable:** the published-number checker, by perturbing a digit.
- **Not fixed:** the missing `seed` field in the emitted cell. The record is
  verified clean by other means; the field itself is named future work.
- **Not claimed:** that `scripts/aws/` is now well tested. Fifteen tests cover the
  plan, the cost model, the gates and the reuse guards. `claim_next.py`'s S3
  conditional-PUT path, `collect.py` and `teardown.py` are **still untested** —
  the claim protocol was read and judged sound (a list failure raises rather than
  looking like an empty queue) but that is a reading, not a test.
