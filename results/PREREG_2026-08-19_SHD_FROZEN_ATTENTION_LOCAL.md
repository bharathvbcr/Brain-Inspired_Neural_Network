# Preregistration — frozen-attention local arm (protocol v150)

**Registered:** 2026-08-19, **before the arm has ever run on real SHD data.**
The only executions to date are unit tests on toy fixtures, whose purpose is to
establish that the attention block is frozen and the plumbing correct, not to
measure anything.

**Binary:** `cargo run --locked --release -p binn-lab --bin shd-frozen-attention`
**Status:** **BLOCKED at the authorization gate.** See
`BLOCKER_2026-08-19_FROZEN_ATTENTION_LOCAL_ARM.md`. This document is registered
now precisely because the arm cannot yet run: thresholds fixed while no data
exists cannot be shaped by data.

---

## 1. The question, and why it is not the one `shd-arch-ablation` asks

`shd_alif`'s own opening question is whether DFA ≈ 0.234 on SHD is *"a limit of
local credit assignment, or a limit of a feed-forward fixed-threshold forward
model"*. The registered v141/v142 ablation answers it with the recurrence ×
adaptation grid — but both axes change the **dynamics** of the hidden layer,
which changes what the layer can represent *and* how credit moves through it.
The two are not separable there.

This arm separates them. It attaches a **frozen** time-axis attention read-out to
the unchanged feed-forward forward model:

- the block is drawn once and never updated;
- nothing is backpropagated through it;
- transported feedback reads only the hidden columns of `w_out`, so it carries
  no credit to the hidden layer;
- the read-out remains one layer with a local error signal.

The arm is therefore **exactly as local as the arm it extends**, and the only
thing that changes is how much temporal structure the read-out can see.

**This is the only attention arm in the project that bears on Gate G2.** The
`+attn` arms of the matched instrument are BPTT references and say nothing about
local learning.

## 2. Registered schedule

| axis | value |
|---|---|
| forward | `ff+fixed` (feed-forward, fixed threshold) — unchanged |
| rules | `DFA`, `BroadcastPm1`, and `EpropCeiling` as a reference |
| attention | frozen, `d_model = 32`, `layers = 1`, vs none |
| hidden | 128 · **epochs** 15 · **lr** 0.02 |
| splits | capped 2000 train / 500 test, matching the sibling ablation |
| seeds | **exactly 12**, shared between the two arms of each pair |

Six cells per seed. Baseline runs before treatment within each rule, so a
timeout leaves a comparable pair rather than a lone treatment number.

## 3. Hypotheses

| ID | statement | threshold |
|---|---|---|
| **F-1** (primary) | Frozen temporal mixing moves a local rule | mean(DFA + frozen) − mean(DFA) **≥ 0.05**, all-seed paired |
| **F-2** | The effect is not confined to one rule | `BroadcastPm1` moves in the **same direction** as DFA |
| **F-3** (reference) | The ceiling moves too, or the task saturates | report `EpropCeiling` with and without; **descriptive, no threshold** |
| **F-4** (validity) | Nothing degenerate is being read as a null | every reported cell non-degenerate; degenerate cells counted separately |

**F-1 is the G2-relevant one.** If frozen mixing — with the credit machinery
untouched — moves the local arm by 0.05 or more, then the binding constraint on
0.234 was the forward model's memory, not the locality of the rule. If it does
not move, locality survives a test it has not previously been given, and **that
is the more informative outcome for this project**, because it removes the
leading excuse for the local arms' performance.

## 4. What must not be claimed

- That this says anything about the *matched instrument's* attention arms. Those
  are BPTT; this is not.
- That a positive F-1 makes local learning competitive. It would move 0.234
  toward ETLP's 0.746, not to it, and the gap would still need explaining.
- That a negative F-1 refutes attention. It would refute *frozen* attention as a
  local feature extractor at this width and depth, on this forward model.
- Anything at all until the arm has run on real SHD data. It has not.

## 5. Validity gates

Per cell, from `AlifEval`: not diverged; more than one distinct class predicted;
`majority_pred_frac` within the registered band; activity within the band. A
degenerate arm scores near chance, which reads as "attention does not help"
unless counted separately — so the report counts and lists them, with defects.

## 6. Stopping rule

**Twelve seeds. The verdict is computed once and reported whichever way it
falls.** No thirteenth seed, no learning-rate sweep, no attention-dimension
sweep. `d_model` and `layers` are fixed at 32 and 1 in advance; if they turn out
to matter, that is a *separately registered* follow-up, not an extension of this.

## 7. Precondition, stated so it cannot be quietly skipped

This protocol may only run once `SHD_INSTRUMENT_STATE` is `Calibrated` through
the documented criteria in `SHD_INSTRUMENT_STATUS.md` — **not** by editing the
constant. If the arm is ever reported from a build with a hand-flipped state,
that report is void.
