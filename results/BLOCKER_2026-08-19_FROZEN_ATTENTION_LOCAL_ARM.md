# The frozen-attention local arm is built and gated — the dependency chain, exactly

**Date:** 2026-08-19
**Binary:** `cargo run --release -p binn-lab --bin shd-frozen-attention`
**Status:** implemented, unit-tested, wired, **refused at the authorization gate**.

---

## 1. What exists

`binn-learn/src/shd_alif.rs` gained a third ablation axis: a **frozen** time-axis
attention read-out. The block is drawn once and never updated; nothing is
backpropagated through it; the transported-feedback path reads only the hidden
columns of `w_out`, so no credit reaches the hidden layer through it. The
read-out stays a single layer with a local error signal, which is what keeps the
arm exactly as local as the arm it extends.

That locality is asserted **bitwise**, for all three rules, by
`shd_alif::tests::training_never_moves_the_frozen_attention_block`. Two further
tests pin that the block widens the read-out without perturbing the spiking
forward, and that `attention: None` reproduces the base arm exactly.

## 2. Why it will not run

```
shd-frozen-attention
  └── requests CampaignKind::LocalLearning
        └── authorize_campaign refuses while SHD_INSTRUMENT_STATE == Uncalibrated
              └── SHD_INSTRUMENT_STATUS.md blocks "new SHD local-learning or
                  architecture-ablation campaigns"
                    └── calibration criterion 4: three clean reference seeds,
                        each >= 0.80  — converged ff+fixed ceiling is 0.7378
                    └── calibration criterion 5: at least one matched
                        Python/Rust configuration — no Python mirror of the
                        attention axis exists
```

`SHD_INSTRUMENT_STATE` is a compile-time constant with no flag and no
environment override, and the status document says so explicitly: *"The
calibration runner has no flag that bypasses prerequisites."* Its sibling
`shd-arch-ablation` is blocked identically, so this is not a regression
introduced by the new binary — it is the gate doing its job on a new campaign of
a family that is currently blocked.

**Flipping the constant would not unblock the work, it would falsify it.** That
constant *is* the claim that the instrument measures what it says it measures.

## 3. What would unblock it, and what is already in flight

| criterion | status | what closes it |
|---|---|---|
| 4 — three clean seeds ≥ 0.80 | **unmet** (0.7378) | possibly wave 1 of the running attention campaign: the pilot reached **0.7509 at e20**, and wave 1 runs the same arm to e400 |
| 5 — matched Python/Rust config | **unmet** for the attention axis | the Python mirror in `scripts/shd_calibration/arms.py`, which is deferred by instruction (`TODO_2026-08-07_OPEN_WORK.md` §8) |

This is worth stating plainly because it inverts the usual reading of the
attention result. The campaign now running is not only a measurement about
temporal memory — **if the converged attention arm clears 0.80 it becomes a
prerequisite for calibrating the instrument**, which is the gate standing between
this project and any new local-learning campaign, including the one arm that
speaks to G2.

Criterion 5 is not closed by accuracy at any level. It needs the Python mirror,
and no amount of compute substitutes for writing it.

## 4. What must not be claimed from this

That the frozen-attention arm helps local learning, or does not. **It has never
been run on real SHD data.** The only executions so far are unit tests on toy
fixtures, whose purpose is to establish that the block is frozen and that the
plumbing is correct — not to measure anything.
