# Campaign plan — the matched-to-live transfer gap

**Registered:** 2026-07-25
**Lead paper:** local rules that clear a matched simulation gate do not survive
event-driven execution — mechanism and decomposition.
**Generalization bar:** software substrates (NumPy reference → Brian2 →
Lava simulation backend). **No neuromorphic hardware**, by decision.
**Timeline:** open-ended; optimise for the strongest paper, not the earliest.

---

## 0. Why this is the lead, and not the frozen matched-arch claim

`PUBLISHABLE_CLAIMS.md` rank 1 is: broadcast ±1 three-factor FAILs the matched
gate while DFA and REINFORCE×frozen-`B` PASS. That claim is clean, but the
freeze's own honesty note limits it:

- Broadcast-**graded** reaches **0.9863** on the DFA schedule, so the claim
  reduces to "±1 quantisation of the reward hurts", not "locality is required".
- The locality-flip evidence is a **1-layer XOR** in a NumPy script.
- The task is `CoincidenceTask`, `N_IN = 2`, `difficulty = 0.05`.

That is a solid workshop paper. It is not a top-venue lead, and further
investment in it has low marginal return. **Package it once, cheaply, in
parallel** (§7).

The transfer story is under-sold in the opposite direction. §2b/§2c already
contain **twelve preregistered live protocols** that vary the substrate one axis
at a time — and one of them, **v23 (finite-θ, mute off, FAIL, gap LCB 0.2370)**,
already survives the single most obvious reviewer objection: *"your gap is just
the θ=∞ bug."* That is a survived falsification and it is currently filed as one
FAIL among many.

### What already exists

| Axis varied | Protocol | Live acc | Gap LCB |
|---|---|---:|---:|
| baseline live RFB | v13 `c1-660401d74db3c88d` | 0.4900 | 0.0737 |
| epochs | v14 `c1-714c115e14a3eeed` | 0.4838 | −0.0100 |
| **feedback structure** | v15 `c1-493ddd56f8714fb6` | **0.7262** | 0.2567 |
| structure × epochs | v16 `c1-677df7f7cbe4f8ec` | 0.5200 | 0.0844 |
| **structure × capacity** | v17 `c1-983ee5303c00b147` | 0.6825 | **0.3127** |
| eligibility timing | v18 `c1-c7d2c86a2b1927f6` | 0.7125 | 0.2351 |
| teaching signal | v19 `c1-dfab4a7ec19f17c2` | 0.6700 | 0.2238 |
| **rule identity (DFA)** | v20 `c1-4db53e645405fae0` | **0.7325** | 0.2601 |
| WTA softness | v21 `c1-f975db8fb3e5d569` | 0.5025 | 0.0406 |
| **threshold dynamics** | v23 `c1-4bbaf4b24c2d1da2` | 0.6638 | 0.2370 |
| feedback continuity | v24 `c1-840f820b7c07b512` | 0.6437 | 0.1380 |

### What is missing

1. **Attribution arithmetic.** Twelve FAILs, no statement of the form "axis *X*
   accounts for *Y*% of the gap." Mostly re-analysis of on-disk results.
2. **A non-saturated task.** The matched pole sits at 1.0000 ± 0.0000 on a
   2-input task, so the measured gap is partly task triviality.
3. **An external substrate.** Nothing rules out "this is a BINN engine defect."
4. **A mechanism**, as opposed to a list of things that did not help.
5. **A rescue arm.** Diagnosis plus a fix is a materially stronger paper.

---

## 0.5 What the 2026-07-25 overnight run established

Run: `results/runs/2026-07-25_210709_overnight`. All validity gates passed on the
ablation (control 0.0513 vs chance 0.0500; modulator parity 1.02; no degenerate
cells). The verdicts are H1 **FAIL**, H2 **FAIL**. Four findings matter more than
those verdicts, and two of them change this plan.

### F1 — H1 FAIL is real but narrowly scoped: recurrence was inert

| Architecture | DFA acc | Hidden activity |
|---|---:|---:|
| ff+fixed | 0.2313 | 0.0034 |
| rec+alif | 0.2213 | 0.0028 |
| rec+fixed | 0.2393 | 0.0035 |
| ff+alif | 0.2313 | 0.0027 |

Activity ≈ **0.003** means ~43 spikes per example across 128 neurons over 100
timesteps — roughly 0.34 spikes per neuron per example. **This is below the
project's own preregistered activity floor** (`activity_sparsity_min = 0.005` in
the C1 config). The C1 harness would mark this `INVALID_HARNESS` on sparsity
grounds.

With `s_prev` almost always all-zero, the recurrent current `Σ wrec[i,k]·s_prev[k]`
is zero on most timesteps. **Recurrence cannot act on a network that does not
spike.** The four architectures landing within 0.018 of each other is what an
inert recurrent path predicts.

The supportable claim is therefore *"at 0.003 activity, recurrence and adaptation
are inert"* — **not** *"architecture is not the binding constraint."* The
`ACTIVITY_MIN = 0.001` guard was set too permissively; it catches silence but not
"technically spiking, too sparse to matter."

The LR pilot does not rescue it: at each architecture's own best rate,
`rec+alif` = 0.2350 (lr 0.02) vs `ff+fixed` = 0.2310 (lr 0.005). Gain +0.004. So
the null survives LR correction — but on an inert substrate, which is not a test.

### F2 — the 0.234 DFA figure was itself a step-size artifact

The old harness re-run with the scale-matched feedback matrix:

| Arm | Before (2026-07-24) | After parity fix |
|---|---:|---:|
| `SHD_DFA` | 0.2336 | **0.0848** |
| `SHD_EPROP_CEILING` | 0.0920 | **0.3020** |

**The treatment and the ceiling swapped places.** Scaling `B` down to match
`wout`'s initialisation collapses DFA to near chance (0.05) and lifts the ceiling
to 0.30. The original DFA competence on SHD was an over-stepping effect, not
credit quality.

This retires the question the ablation was built to answer. There is no 0.234
result to explain.

### F3 — init-time parity is not enough; `wout` grows

That same report flags a residual **11.78×** mismatch (DFA 1.32e-2, e-prop
1.56e-1) at `lr = 0.02`. `B` is frozen; `wout` grows ~10× during training. Parity
enforced at initialisation decays away.

Parity *held* (1.00–1.02) in the ALIF ablation only because activity 0.003 makes
`Δwout ∝ rates` negligible — i.e. **parity held because nothing was learning.**

Any future ceiling comparison must equalise realised step size **throughout
training**, not at `t = 0`.

### F4 — two harness defects, one guard win

- **`multi-area-scaling` panicked at M=4**: `distinct states = 1` across 100
  samples. The stimulus propagates through one inter-area hop and dies. M=2 gave
  1.0000 with only 5 distinct states out of 100 — barely non-degenerate. The
  multi-area path does not carry stimulus information past the first projection.
  The guard fired exactly as designed; this is the machinery working.
- **The depth-matched gradient ceiling is broken**: 0.488 / 0.500 / 0.500 / 0.500
  across depths 1–4 on a binary task — chance at *every* depth, including depth 1
  where it should reproduce `MatchedGradient`'s 0.9895. The per-layer RMS
  normalisation in `MatchedDeepGradient` destroys the signal. That suite is
  unusable until fixed.

### Consequence for this plan

Phase 2a assumed SHD could supply a non-saturated matched pole. It currently
cannot: every SHD number is either confounded by modulator scale (F2, F3) or
measured on a sub-floor-activity network (F1). **There is presently no validated
SHD result in this project.**

A new blocking phase (0c) is inserted below. Do not build a 16-rung transfer
ladder on a substrate whose forward model has not been shown to work.

---

## 0.6 The 2026-07-26 pivot — v143 stopped the SHD line, v144 replaced the lead

Run: `results/runs/2026-07-26_112642_code_transfer`. The runner stopped itself at
the `INVALID_TASK` gate. That is the machinery working; the two reports it
produced first are the most consequential results the project has.

### G1 — SHD: the hidden layer is worse than no hidden layer

The 0c-1 input-only control, paired, 10 seeds, hierarchical bootstrap:

| Comparison | Schedule | Input-only | Hidden | Hidden − input | 95% CI |
|---|---|---:|---:|---:|---|
| capped ALIF ff+fixed | 2000/500, 15 ep | 0.2618 | 0.2224 | **−0.0394** | [−0.0572, −0.0208] |
| **full SuperSpike BPTT** | **8156/2264, 20 ep** | **0.4428** | **0.4157** | **−0.0270** | [−0.0432, −0.0087] |

Shuffled-label input controls clean (0.0568, 0.0468). Neither arm degenerate. No
test-time updates; deterministic replay confirmed.

**The hidden spiking layer does not merely fail to help — it hurts, with CIs
excluding zero, on both schedules.** And the second row is the one that ends the
line: this is **full SuperSpike BPTT on the full official splits**. The gradient
reference. Published BPTT on SHD reaches 0.951; e-prop 0.808; ETLP 0.746. This
stack gets **0.4157**, and a linear readout on raw input rates beats it.

That gap has nothing to do with locality, credit assignment, or substrate. The
SHD forward model, encoding, or readout is mis-specified. Every SHD number this
project has produced — DFA, e-prop, broadcast, and the SuperSpike ceiling alike —
measures that defect.

**The §8 stop rule fires as written:** *"0c-1: input-only readout matches the full
network → Stop. The hidden layer is not contributing."* It did not merely match;
it won.

**Obligations.** Retract the entire SHD claim axis, not just the DFA rows.
`c1_shd_h128/256/512.md`, `c1_shd_full_smoke.md` and the v143 ALIF ablation are
all superseded. Do not cite 0.234, 0.0848, 0.2213, or the 0.30 ceiling. Do not
re-run SHD until the forward model is fixed and the input-only control is beaten
by a stated margin.

### G2 — v144 built a working substrate, and the local rule fails on it

Shortcut-resistant temporal calibration, 4 classes (chance 0.25):

| (jitter, distractors) | Matched RFB | BPTT | Raw rate | Time-shuffled |
|---|---:|---:|---:|---:|
| (0, 4) | 0.2533 | **1.0000** | 0.2500 | 0.2600 |
| (1, 8) | 0.2600 | **1.0000** | 0.2500 | 0.2400 |
| (2, 12) | 0.2467 | 0.9733 | 0.2500 | 0.2533 |
| (3, 16) | 0.2733 | 0.9433 | 0.2500 | 0.2767 |

Read the columns:

- **BPTT 0.94–1.00** — the task is learnable and the harness is sound. This is
  the within-run positive control that SHD never had.
- **Raw rate exactly 0.2500** — no rate shortcut exists. Structural, by
  construction: every quartet has byte-identical channel counts.
- **Time-shuffled at chance** — temporal structure is required, not decorative.
- **Matched RFB at chance on every setting**, including (0, 4) where BPTT is
  perfect.

The gate called this `INVALID_TASK` because no difficulty put the matched arm
in-band. Mechanically correct — you cannot measure a matched→live transfer gap
when the matched pole is already on the floor. **Scientifically it is the
result, not a failure.**

### G3 — what this does to the frozen claim sheet

`PUBLISHABLE_CLAIMS.md` §1b records matched DFA at 0.9387, matched RFB at 0.9200,
and broadcast-*graded* at 0.9863 — all on `CoincidenceTask`. The same rule family
is at chance on a task built to be immune to rate shortcuts.

The most economical explanation is that **CoincidenceTask is solvable by a
rate-accessible strategy**, and the §1b PASSes measure that rather than credit
assignment. The 0.9863 broadcast-graded number — already flagged in the freeze's
own honesty note as awkward — is exactly what a rate shortcut predicts.

This is a hypothesis, not yet a finding. It is testable directly: run the v144
raw-rate control on `CoincidenceTask`. If raw rate solves coincidence, §1b needs
restating.

### Consequence: the lead changes again

The transfer-gap lead assumed a rule that works in matched simulation and breaks
on the live substrate. G2 says the rule does not work in matched simulation
either, once the task cannot be solved by rate shortcuts. **The interesting gap
moved upstream** — from *matched → live* to *easy-task → shortcut-resistant
task*, and it needs no substrate machinery at all to demonstrate.

The transfer campaign is **deferred, not cancelled**: it becomes meaningful again
only if a local rule is found that clears a shortcut-resistant task. See §11.

---

## 1. Claim structure

**C1 (phenomenon).** A three-factor local rule that clears a preregistered gate
on a matched dense-LIF forward fails the same gate when executed on an
event-driven, k-WTA substrate, on a task where the matched arm is **not**
saturated.

**C2 (mechanism).** The gap is dominated by a specific, nameable substrate
property — not by the rule, the task, or the training schedule.

**C3 (generality).** C1 and C2 reproduce in at least one independently
implemented event-driven simulator.

**C4 (remedy).** A targeted modification addressing the dominant axis recovers a
material fraction of the gap.

A paper with C1+C2+C3 is publishable. C4 makes it strong. **C1 alone is not a
paper** — that is the current state.

---

## 2. Phase 0 — falsification first (target: 2–3 weeks)

Two cheap experiments that can kill the entire campaign. Run them before
anything else.

### 0a. Literature search (1 day, blocking)

Search terms, all of them:

- "simulation to hardware gap" spiking local learning
- "deployment gap" neuromorphic plasticity
- on-chip vs off-chip accuracy discrepancy — e-prop / ETLP / DECOLLE
- Loihi / SpiNNaker / BrainScaleS on-chip learning accuracy drop
- sparse winner-take-all credit assignment coverage
- surrogate gradient "sim-to-real" spiking

**Kill rule:** if the phenomenon is already characterised with a mechanism, stop.
Write the workshop paper (§7) and pick a different problem. One day now beats
three months later.

### 0b. NumPy event-driven reference (1 week)

Reimplement the live substrate — event queue, k-WTA selection, eligibility decay,
three-factor update — in a few hundred lines of NumPy, from the *specification*,
not by porting the Rust.

**Kill rule:** if the matched→live gap does **not** reproduce within ±0.10 of the
BINN result, the gap is a BINN engine defect. Stop the campaign, fix the engine,
and revisit. This is the highest value-per-hour experiment in the plan.

### 0c. Forward-model validation (NEW — blocking, 2–3 weeks)

Added 2026-07-26 in response to F1–F3. The transfer campaign needs a substrate
whose *matched* pole demonstrably learns. Right now none has been shown to.

Four deliverables, in order:

**0c-1. Input-only readout control (1 day, do this first).**
Train the linear softmax readout directly on raw input rates, no hidden layer, on
the same SHD split. If it reaches ≈ 0.22, the hidden layer contributes nothing
and every SHD number in this project — old harness and ALIF alike — is a readout
measurement. This is a one-afternoon experiment that could invalidate months of
work; run it before anything else.

**0c-2. Activity calibration.**
Tune `in_scale` / `THETA_REST` / input gain until hidden activity lands in
**[0.02, 0.10]** spikes/neuron/timestep — well above the project's own 0.005
floor, and high enough that recurrent current is comparable to input current.
Report the achieved band. Raise `ACTIVITY_MIN` from 0.001 to 0.005 to match the
C1 preregistered floor, so a sub-floor run is flagged rather than passed.

**0c-3. Running-scale parity.**
Replace init-time matching with sustained matching. Options, in preference order:
(a) renormalise `B` to `‖wout‖` at each epoch boundary; (b) per-arm learning rate
chosen so realised modulator RMS is equal; (c) report realised RMS per epoch and
restrict claims to the window where parity holds. Add a test asserting parity at
the *end* of training, not just at construction.

**0c-4. Re-run the architecture ablation.**
Only after 0c-1..3. H1 is currently untested, not falsified.

**Gate to Phase 1:** 0a and 0b pass, **and** 0c yields a matched arm that is
(i) above chance by a margin that survives its CI, (ii) inside the activity band,
(iii) parity-stable to end of training, and (iv) beats the 0c-1 input-only
control by a stated margin.

If 0c-1 shows the hidden layer adds nothing on SHD, stop and reconsider the task
before spending anything on 0c-2..4.

---

## 3. Phase 1 — attribution from existing results (2–3 weeks, little new compute)

Turn twelve FAILs into a mechanism claim.

### 1a. Common-scale re-analysis

Recompute every v13–v24 arm onto a single comparable scale:

- gap-closed via `guards::gap_closed_clamped` (uniform clamping — the rescue
  harnesses did not clamp, which is how 1.0244 shipped)
- realised modulator RMS per arm via `ModulatorScale`; **if arms differ in
  effective step size, their accuracy differences are not attributable to the
  axis being varied.** This is the defect that inverted the SHD ceiling.
- per-arm degeneracy audit via `guards::ReadoutAudit`

**Expected outcome:** some of the twelve become uninterpretable. That is fine and
should be reported.

### 1b. Attribution

For each axis, from the dense pole:

```
contribution(axis) = acc(dense) − acc(dense with only that axis set to live)
```

and from the live pole:

```
marginal(axis) = acc(live with only that axis restored to dense) − acc(live)
```

Disagreement between the two ⇒ interaction, which is itself a finding.

### 1c. Sharpen the hypothesis

The current stated hypothesis — "live k-WTA execution requires pre-absorbed
eligibility traces" — is not yet operational. Restate it as a measurable
prediction, e.g.:

> **H-mech:** at reward time, mean |eligibility| on the live substrate is a
> fraction *f* < 0.3 of its within-trial peak, whereas on the matched substrate
> *f* ≈ 1. The accuracy gap scales monotonically with (1 − *f*).

**Gate to Phase 2:** one axis accounts for ≥ 40% of the gap under 1b, **or** the
attribution shows the gap is irreducibly distributed — which is a different but
still publishable claim, and changes the paper's framing.

---

## 4. Phase 2 — non-saturated substrate ladder (6–10 weeks)

The main new compute, and the part that makes C1 defensible.

### 4a. Task replacement (blocking) — revised 2026-07-26

Move the matched↔live contrast off `CoincidenceTask` (`N_IN = 2`), whose matched
arm sits at 1.0000 ± 0.0000.

**Revised target band: the matched arm must land in 0.40–0.85**, not the 0.70–0.95
originally written here. On 20-way SHD, ETLP — the best published fully-local
rule — reaches 0.746, so demanding ≥ 0.70 from this project's rule was not a
realistic floor. What matters is that the pole is (a) clearly above chance
relative to its CI and (b) clearly below ceiling, so a gap is measurable in both
directions.

**SHD's status is downgraded from "the natural target" to "a candidate pending
0c."** The overnight run showed SHD accuracy here is either scale-confounded
(F2, F3) or measured below the project's own activity floor (F1). SHD becomes the
Phase 2 substrate only if 0c delivers a matched arm meeting the gate.

**Fallback if 0c fails on SHD:** an intermediate-complexity temporal task —
N-MNIST, SHD with fewer classes, or a synthetic multi-channel temporal task with
tunable difficulty. The requirement is a *measurable* gap, not a hard dataset.
Record the choice and its justification in the Phase 2 preregistration.

### 4b. The 16-rung lattice

Four independently switchable axes, each `{dense, live}`:

| Axis | Dense pole | Live pole |
|---|---|---|
| **A. Selection** | all units update | k-WTA: only winners update |
| **B. Timing** | synchronous frames | event-driven with synaptic delays |
| **C. Threshold** | fixed θ, soft reset | adaptive θ, hard reset |
| **D. Trace residency** | eligibility held to trial end | eligibility decays in real time |

All 16 rungs, n ≥ 10 seeds. Single-axis knockouts and restorations per §3.1b,
plus a Shapley-style attribution over the full lattice to expose interactions.

**Implementation constraint:** one configurable harness spanning both poles. Two
separate implementations would reintroduce exactly the hidden asymmetry the
2026-07-25 audit found in the SHD ceiling.

### 4c. Mechanism instrumentation

Accuracy alone explains nothing. Per rung, log:

| Metric | What it tests |
|---|---|
| **Credit coverage** — fraction of synapses receiving nonzero update per trial | axis A |
| **Eligibility survival** — mean \|e\| at reward ÷ within-trial peak \|e\| | axis D; the direct test of H-mech |
| **Credit-sign agreement** — per-synapse sign match between live and dense update on the same trial and seed | whether the substrate corrupts credit *direction* or only magnitude |
| **Effective step size** — realised modulator RMS | must be normalised out before any mechanistic claim |

Sign agreement is the most informative and the least likely to be confounded.

**Gate to Phase 3:** C1 holds on the non-saturated task, and C2 has a named
dominant axis with a supporting mechanism metric.

---

## 5. Phase 3 — external replication (4–8 weeks)

### 3a. Brian2 (primary)

Same rule, same task, same lattice — implemented independently by a third-party
simulator. This is what converts "our engine does this" into "event-driven
execution does this."

### 3b. Lava simulation backend (secondary)

Loihi execution semantics without hardware access. Gives the paper its
Loihi-facing framing while respecting the freeze's standing non-claim that this
project makes **no neuromorphic hardware claims**.

**Gate to Phase 4:** the dominant axis from Phase 2 reproduces in Brian2 with the
same sign and within a stated tolerance.

---

## 6. Phase 4 — rescue arm (4–6 weeks)

Implement the fix implied by the dominant axis. If D (trace residency) dominates:

- eligibility `τ_e` matched to the reward delay
- two-timescale trace (`DualEligibility` already exists)
- explicit trace freezing at spike time until reward — the "pre-absorbed"
  hypothesis made concrete

**Success:** recovers ≥ 50% of the gap, and the recovery is predicted by the
mechanism metric rather than found by search.

---

## 7. Parallel track — package the frozen claim once

Independent of the above, and time-boxed to **2 weeks total**:

- Workshop paper from `PUBLISHABLE_CLAIMS.md` rank 1 / 1b.
- Lead: broadcast ±1 three-factor fails a matched gate that graded DFA and
  REINFORCE×frozen-`B` clear.
- **Must disclose** the 0.9863 broadcast-graded contrast in the abstract, not a
  footnote. A reviewer who finds it later will not be generous.
- Locality-flip claim cites the 1-layer XOR explicitly and does not generalise.

Do not extend this track. Its purpose is to bank the result and stop.

---

## 8. Stop rules (binding)

| Trigger | Action |
|---|---|
| **0c-1: input-only readout matches the full network** | **Stop. The hidden layer is not contributing; the task or the forward is wrong, and no transfer result on it would mean anything.** |
| **0c: no configuration gives a matched arm in-band, in-parity, above the input-only control** | **Stop the SHD line. Switch task per §4a fallback, or stop the campaign.** |
| Phase 0a finds prior mechanistic characterisation | Stop campaign. Ship §7. |
| Phase 0b gap does not reproduce in NumPy | Stop campaign. It is an engine bug. |
| Matched pole cannot be brought inside 0.40–0.85 on any task | Stop. The gap is not measurable. |
| No axis exceeds 40% and no coherent interaction story | Reframe as "distributed, irreducible" — reduced-scope paper. |
| Brian2 contradicts the dominant axis | Stop C3. Publish C1+C2 as engine-scoped, explicitly. |
| Any phase exceeds 2× its budget | Re-plan; do not silently extend. |

### A standing rule, earned on 2026-07-25

**Before any accuracy comparison, check that the substrate is in a regime where
the manipulated variable can act.** Recurrence cannot be tested on a network that
does not spike; a ceiling cannot be compared to a treatment stepping at 12× its
rate. Both failures produce numbers that look like results. Neither is caught by
an accuracy threshold — only by an activity band and a realised step-size check.

Every phase gets a preregistration in `results/PREREG_*.md` **before** it runs,
with thresholds fixed in advance — same discipline as protocol 141.

---

## 9. Manuscript skeleton

**Title:** *Local learning rules that pass in simulation fail under event-driven
execution: a substrate decomposition*

1. **Introduction.** Local rules are motivated by neuromorphic deployment, but
   are validated in dense synchronous simulation. Nobody measures the crossing.
2. **Related work.** e-prop, ETLP, TESS, S-TLLR, DECOLLE, Traces Propagation —
   each validated in *one* substrate. Position: orthogonal to "which rule is
   best", asking "does any of it survive execution."
3. **Setup.** Matched forward, the rule, the two poles, the four axes.
4. **The gap (C1).** Non-saturated task, matched vs live, n ≥ 10.
5. **Decomposition (C2).** Lattice, knockouts, restorations, Shapley,
   interactions.
6. **Mechanism (C2).** Credit coverage, eligibility survival, sign agreement.
   Figure: accuracy gap vs eligibility survival.
7. **Generality (C3).** Brian2, Lava-sim.
8. **Remedy (C4).** Rescue arm; recovery predicted by the mechanism metric.
9. **Limitations.** Software only, no hardware, no energy claims. One rule
   family. Specific task set.
10. **Appendix — negative-result integrity.** The twelve v13–v24 protocols, the
    preregistrations, the harness defects found and fixed in the 2026-07-25
    audit. This appendix is a genuine asset; most papers in this area cannot
    write it.

**Figure 1** must be the gap on the non-saturated task.
**Figure 2** must be the attribution bar chart.
**Figure 3** must be accuracy gap vs eligibility survival across all 16 rungs.
If Figure 3 is not a clean monotone relationship, the mechanism claim is not
ready.

---

## 11. The new lead — "shortcut-resistant temporal failure" (2026-07-26)

**Candidate claim.** On a temporal task constructed to be immune to rate
shortcuts — verified by a raw-rate control at exact chance and a time-shuffle
control at chance — a matched local three-factor rule performs at chance across
all difficulty settings, while SuperSpike BPTT on the identical forward reaches
1.0000.

This is stronger than both previously-considered leads:

| | Old lead (frozen §1b) | Transfer lead | **This** |
|---|---|---|---|
| Controls | none for shortcuts | none for shortcuts | raw-rate + time-shuffle, both at chance |
| Reference | BPTT 0.8963 | matched 1.0000 | BPTT **1.0000** |
| Undercut by | graded broadcast 0.9863 | task saturation | — |
| Substrate machinery needed | no | yes, 16 rungs | **no** |

It also explains the awkward 0.9863: a rate-accessible task should be solvable by
almost any credit signal, including a graded broadcast scalar.

### The single next experiment: v147 — the shortcut-accessibility contrast

Supersedes running V1–V5 separately. **One run does the positive control, tests
G3, and produces the paper's headline figure.**

Same binary, same 4-class multiclass local arm, same BPTT reference, same seeds.
Only one thing varies: whether the task is solvable from rate.

| Variant | Channel counts | Expected raw-rate | Expected BPTT | Local arm |
|---|---|---|---|---|
| **A. rate-accessible** | differ by class | high | ~1.00 | **must be high** |
| **B. rate-immune** (current v144) | byte-identical | 0.2500 | ~1.00 | 0.2533 observed |

Read the outcomes:

- **Local high on A, chance on B** → the finding is real, the positive control
  passed, and G3 is supported in one stroke. Write the paper.
- **Local chance on both** → the multiclass local path is broken. Everything
  above is void; debug before claiming anything. *This is the outcome the last
  three rounds should make you expect.*
- **Local high on both** → the v144 result was a difficulty artifact, not a
  shortcut artifact. Re-scope.

Why this beats reproducing the frozen `CoincidenceTask` PASS as the control: the
frozen number is **binary**, and v144's local arm is **multiclass** — different
code path. A binary positive control would not exercise the code that produced
the chance result. Variant A does.

Extend the same design to a continuum (partial rate-accessibility between A and
B) and it becomes Figure 2 directly.

### Remaining verification, folded into v147



The last three rounds each produced a chance-level arm that turned out to be an
artifact (56× step-size, 0.003 activity, hidden layer worse than no layer). Do
**not** accept a fourth at face value. `temporal_calibration_v144.md` currently
discloses **no activity and no modulator-scale figures at all** — grep count zero.

| # | Check | Rationale |
|---|---|---|
| V1 | **Within-harness positive control**: the same binary, same local arm, must reproduce the frozen `CoincidenceTask` PASS (RFB ≈ 0.92) | Proves the local arm is wired and learning *in this code path*. Without it, chance could be a plumbing bug. **Highest priority.** |
| V2 | **Hidden activity** per arm, against the C1 floor of 0.005 | The v143 ablation failed silently at 0.0034. Repeat unacceptable. |
| V3 | **Realised modulator RMS**, local arm vs BPTT's effective gradient scale, end of training | Step-size mismatch has already inverted one conclusion. |
| V4 | **LR sweep on the local arm at (0, 4)** | Chance at one learning rate is not chance at all rates. |
| V5 | **Learning curve**, not just final accuracy | Distinguishes "never learned" from "learned then collapsed". |

**V1 is the load-bearing one.** If the same binary solves coincidence at 0.92 and
this task at 0.25, with V2–V4 clean, the finding is real and the paper is
straightforward. If it cannot reproduce coincidence, the harness is broken and
nothing above stands.

### Then: raw-rate control on CoincidenceTask

One run, directly tests G3. If raw rate solves coincidence, `PUBLISHABLE_CLAIMS.md`
§1b must be restated — and that restatement is itself part of the paper's
contribution, since it explains the 0.9863 anomaly the freeze could not.

### Re-scoping v145 / v146

The runner correctly blocked both. They should be **re-scoped rather than
unblocked**:

- **v145 (temporal depth)** was to sweep depth on a frozen difficulty. With the
  local arm at chance everywhere, depth has nothing to act on. Re-scope to: at
  which point on a *shortcut-accessibility* continuum does the local rule fail?
  Interpolate from CoincidenceTask (rate-accessible) to v144 (rate-immune) and
  find the transition. That curve is Figure 2 of the new paper.
- **v146 (transfer falsifier)** presumed a matched pole to transfer *from*.
  Deferred until a local rule clears a shortcut-resistant task.

---

## 10. Immediate next actions — revised 2026-07-26

Superseded by the v143/v144 pivot. Steps 2, 5, 6, 7 are **cancelled** — 0c-1 has
run and its stop rule fired, so there is nothing to calibrate on SHD.

Ordered by information-per-hour:

1. **V1 within-harness positive control** (§11). The same binary, same local arm,
   must reproduce the frozen `CoincidenceTask` RFB ≈ 0.92. Everything else waits
   on this. Half a day.
2. **V2–V3 disclosure**: add hidden activity and end-of-training modulator RMS to
   the v144 report. It currently discloses neither. Half a day.
3. **Raw-rate control on `CoincidenceTask`** (§11). One run. Tests G3 directly and
   would explain the 0.9863 anomaly.
4. **Retract the SHD axis in full.** Not just DFA — the SuperSpike ceiling too.
   Mark `c1_shd_h128/256/512.md`, `c1_shd_full_smoke.md` and the v143 ALIF
   ablation superseded. State the reason as *input-only equivalence under the
   gradient reference*, which is stronger and simpler than the scale-artifact
   reason recorded on 2026-07-25.
5. **V4 LR sweep** on the local arm at (0, 4).
6. **Phase 0a literature search**, re-scoped to the new lead: search for prior
   work on rate-shortcut confounds in spiking benchmarks and on local rules
   failing shortcut-controlled temporal tasks. Still one day, still blocking.
7. **Fix `MatchedDeepGradient`** — chance at every depth including depth 1, where
   it should reproduce 0.9895. Needed before v145's re-scoped depth sweep.
8. **Re-scope v145** to the shortcut-accessibility continuum (§11).
9. Only if a local rule ever clears a shortcut-resistant task: revisit Phase 0b
   and the transfer ladder.

**§7 is unaffected and its value has risen.** The frozen matched-arch claim rests
on `c1-match-*` / `c1-dfa-*` / `c1-rl-*`, none of which are SHD. But note G3: if
the raw-rate control solves coincidence, §7's framing must change before
submission — from "broadcast ±1 fails where DFA succeeds" to something that
accounts for the task being rate-accessible. **Do not submit §7 before step 3
returns.**
