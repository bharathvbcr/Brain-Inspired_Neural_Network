# Preregistration — decomposing the matched-to-live transfer gap

**Date:** 2026-09-07
**Status:** registered before the instrument exists, which is stricter than
before the first cell exists. §3 names the code that must be written, and §3 is
part of the registration.
**Design it makes concrete:** [`DESIGN_TRANSFER_GAP_DECOMPOSITION.md`](DESIGN_TRANSFER_GAP_DECOMPOSITION.md)
**Discharges:** `TODO_2026-08-07_OPEN_WORK.md` §3, "Write the decomposition preregistration"

---

## 1. The observation, on numbers that survived the repair

The design this registers was written on 2026-07-24 and its §1 table cites
`live_transfer_rescue.md` for a 1.0000-vs-0.5188 contrast. **That source is
withdrawn** — the `live-transfer-rescue` arms report `INVALID_HARNESS` and the
protocol was misnamed: it is matched-only, not live-engine
([`PAPER_DRAFT.md`](PAPER_DRAFT.md) Appendix C, episode 1). Nothing below rests
on it.

The observation survives in repaired form, and this is the version registered:

| | current status | source |
|---|---|---|
| matched dense-LIF forward | every rule but broadcast ±1 clears the gate — DFA 0.9925, REINFORCE 0.9950, broadcast-graded 0.9975, spike-adjoint 0.9450 | `RESULT_2026-08-25_MATCHED_ARCH_RERUN.md` |
| live event-driven k-WTA | **no** variant clears it: twelve gap-close arms v13–v24, best gap LCB **0.3127** against a 0.5 threshold | `PAPER_DRAFT.md` §3.3, §4.2 |

**Rules that pass on a dense forward buy nothing on a hard k-WTA substrate.**
That is a scoped negative the paper already reports. What no experiment in this
repository has done is say **which difference between the two substrates is
responsible**, and that is what this wave is for.

---

## 2. The four factors

The paper's Appendix A lists the canonical C1 path's integrity limitations.
Those are the factors, because they are the concrete, named ways the live
substrate differs from the matched one:

| # | factor | live pole (canonical v2) | dense pole | code site |
|---|---|---|---|---|
| **F1** | STDP pairing residency | `ThreeFactor.last_spike` is retained across trials | cleared at each trial boundary | `binn-lab/src/runner.rs:2422` |
| **F2** | membrane reset | soma `v` and `theta` only — H2-incomplete | C3-style full dynamic reset | `binn-lab/src/runner.rs:2427` |
| **F3** | hidden threshold | θ = ∞ during integrate, suppressing natural spiking | finite θ | `runner.rs:1508` (`natural_spiking`) |
| **F4** | selection | hard membrane-score k-WTA, k winners update | all units update | `runner.rs:1500` (`Area::new(.., config.k_wta)`) |

**One factor per arm is the whole design.** A wave that moves two at once
reproduces the confound it exists to remove.

---

## 3. The instrument does not exist yet, and this is what is missing

Read from the source on 2026-09-07 rather than assumed:

**F1 and F2 are welded.** `runner.rs:2422` is a single branch on one boolean:

```rust
if trial_isolation {
    reset_c1_dynamic_state(eng, &hidden_cells, &saved_thresholds);  // F2
    learner.reset_pairing_state();                                  // F1
} else { /* canonical v2: soma v + theta only */ }
```

and `trial_isolation` is set by four unrelated protocol predicates at
`runner.rs:1504`. **No configuration clears `last_spike` without also applying
the full membrane reset, or the reverse.** This is exactly the "never tested
individually" the open-work register names, and it is why the existing
`c1-iso-*` family cannot answer this question however many seeds it is given.

**F4 has no off switch.** `k_wta` is a width, not a mode; there is no
all-units-update path on the live engine.

**F3 is already separable** — `natural_spiking` at `runner.rs:1508` is its own
boolean.

### Registered preconditions

No cell of this wave may run until all three hold, and an arm that cannot be
built is reported as **NOT EVALUABLE**, never as a null:

1. `trial_isolation` is split into two independent opt-in switches, one per
   factor, such that all four combinations are reachable.
2. A selection mode exists in which every hidden unit receives credit, so F4 can
   be flipped rather than only rescaled.
3. **The canonical path is bit-identical with every new switch off.** Gate F
   over the existing recorded cells is the evidence, and a diff in any
   scientific field fails this precondition. Adding an experimental knob may not
   perturb `c1-118207fbc3eaba53`; if it does, the knob is wrong, not the gate.

---

## 4. Design

**Substrate:** the live event-driven k-WTA engine, canonical C1.

**Task — and this is non-negotiable, from the design's §3.3.** Not
`CoincidenceTask` at `N_IN = 2`. The matched gate saturates at exactly 1.0000
with zero variance, which is why the paper states it can name one failure and
cannot rank the survivors; a decomposition run against a ceiling measures task
triviality and calls it substrate. **The dense pole must score in 0.7–0.9.** If
no available task puts it there, this wave does not run and the registration
says so rather than lowering the requirement.

**Lattice:** all 16 rungs of `{F1,F2,F3,F4} × {live, dense}`, at **n ≥ 10
seeds**. Both readings the design requires:

1. **Knockouts** — one factor to `dense`, three held `live`. The factor's
   individual effect.
2. **Restorations** — one factor to `live`, three held `dense`. Its marginal
   effect in context.

Where (1) and (2) disagree the factors interact, which is a finding rather than
a nuisance, and is why all 16 rungs are run instead of the 8 single flips.

---

## 5. Clauses

**T-1 (primary).** At least one single-factor knockout moves accuracy by more
than **0.05** from the live pole, in at least **8 of 10** seeds.
*Falsifier:* every single flip is inside ±0.05. Then the gap is not attributable
to any one of these four, and the honest reading is that it is interactive or
lies outside the factors Appendix A names — a result about the decomposition's
own adequacy, and reported as one.

**T-2 (primary).** The four knockout effects **sum** to within **0.10** of the
full live-to-dense difference.
*Falsifier:* they do not, which localises the gap in interaction rather than in
any factor and makes the Shapley attribution over all 16 rungs the reportable
object.

**T-3 (secondary, directional).** The design's stated hypothesis — that trace
residency dominates — predicts **F1 is the largest single-factor effect**.
Registered as a prediction, and reported as failed if another factor is larger.

**T-4 (gate, not a hypothesis).** The dense pole scores in **0.7–0.9**. Outside
that band every clause above reads **NOT EVALUABLE**, and no number from this
wave is cited. A saturated pole is the failure mode §4 exists to prevent.

---

## 6. Stopping rule

Registered here because a lattice invites extension until something clears.

- **16 rungs × 10 seeds = 160 cells. That is the wave.** Rungs are not added,
  seeds are not extended, and no arm is re-run at a different budget, without a
  new registration governing the *next* wave.
- A rung whose cells void below **8 of 10** is reported as incomplete with its
  void count. It is **not** re-run under a loosened validity gate.
- If T-4 fails, the wave stops at T-4. The remaining clauses are not evaluated
  on a saturated pole "to see".

---

## 7. Named outcomes, in every direction

| outcome | reading |
|---|---|
| **T-1 MET, T-2 MET, T-3 MET** | The gap decomposes, and trace residency dominates as hypothesised. This is the result the design was written for and the paper's transfer negative gains a mechanism. |
| **T-1 MET, T-2 MET, T-3 NOT MET** | The gap decomposes and the hypothesis was wrong about which factor. The larger factor is named and the residency hypothesis is retired. Equally publishable, and the registered prediction is reported as failed. |
| **T-1 MET, T-2 NOT MET** | Single factors move accuracy but do not sum to the whole: the substrates differ **interactively**, and the Shapley attribution over 16 rungs is the object rather than any per-factor number. |
| **T-1 NOT MET** | No single flip moves anything. The four factors Appendix A names do not contain the gap, which is a finding about the decomposition rather than about the substrate, and points at what Appendix A does not list. |
| **T-4 fails** | **NOT EVALUABLE**, wave stopped. A gap measured against a ceiling is task triviality wearing a substrate's name. |
| **A precondition in §3 unbuildable** | Reported as NOT EVALUABLE with the reason. In particular, if the canonical path cannot stay bit-identical with the new switches off, the knob is withdrawn and `c1-118207fbc3eaba53` is not touched. |

---

## 8. What this wave cannot settle

- **Not the SHD read-out programme.** Nothing here bears on §3.5's
  difference-in-differences, and no number from this wave may be cited there.
- **Not a rescue of G2.** G2 FAIL is terminal. This decomposes a *transfer*
  negative and runs under the post-G2 opt-in flags like every other wave since.
- **Not hardware.** "Event-driven" here is this engine, not silicon, and no
  neuromorphic-deployment claim follows from any outcome.
- **Not generality.** One task, one engine, four named factors. A gap that
  decomposes here is decomposed here.
