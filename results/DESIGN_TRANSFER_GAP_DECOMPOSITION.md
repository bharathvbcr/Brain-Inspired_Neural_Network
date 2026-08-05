# Design — matched-to-live transfer gap decomposition

**Status:** design only, not implemented. Build this after the SHD architecture
ablation reports.
**Why it matters:** this is the one observation in the 2026-07-24 suite that does
not reproduce a known result. It is the project's most credible path to a novel
contribution.

---

## 1. The observation

Same learning rule, same task, two substrates:

| Substrate | Accuracy | Source |
|---|---:|---|
| Matched dense-LIF schedule | 1.0000 ± 0.0000 (n=20) | `live_transfer_rescue.md` |
| Live event-driven k-WTA engine | 0.5188 | `c1_rfb_learned.md` |

A rule that is perfect in the simulation schedule everyone publishes in scores at
chance-plus-two-points on the event-driven substrate that neuromorphic hardware
actually implements.

A literature search on 2026-07-25 found no paper naming or characterising this
specific gap. The adjacent work (ETLP, TESS, S-TLLR, Traces Propagation,
EchoSpike) all targets hardware-compatible locality, but each evaluates its rule
in *one* substrate. Nobody appears to have measured what breaks in the crossing.

**If that survives a proper search, it is a paper.** The framing writes itself:
*local rules validated in dense simulation do not survive event-driven
execution — here is the decomposition of why.*

## 2. Why the current evidence is not yet a result

- It is an observation at two points, not a decomposition.
- The matched arm scores 1.0000 ± 0.0000 on `CoincidenceTask` with `N_IN = 2` and
  `difficulty = 0.05`. A saturated toy task inflates the apparent gap: some of
  the 0.48 drop is "the task was trivial in one substrate", not "the substrate
  broke the rule".
- The `c1-rfb-learned` run's own verdict is FAIL / U-NEG, and no mechanism is
  established. The stated hypothesis — "live k-WTA execution requires
  pre-absorbed eligibility traces" — has never been tested.

## 3. What must be built

### 3.1 A substrate ladder

The two endpoints differ in at least four ways at once. Make each a separately
switchable axis so the gap can be attributed:

| Axis | Dense pole | Live pole | Hypothesised contribution |
|---|---|---|---|
| **A. Selection** | all units update | k-WTA: only k winners update | large — most synapses never receive credit |
| **B. Timing** | synchronous frame steps | event-driven with synaptic delays | medium — eligibility decays between spike and reward |
| **C. Threshold** | fixed θ, soft reset | adaptive θ, hard reset | medium — reset cuts the eligibility path |
| **D. Trace residency** | eligibility held to end of trial | eligibility decays in real time | large — this is the stated hypothesis |

Each axis independently `{dense, live}` gives 16 rungs. Run the full lattice at
n ≥ 10 seeds. Report accuracy per rung.

### 3.2 The decomposition

Two complementary readings, both required:

1. **Single-axis knockouts from the dense pole** — flip one axis to `live`, hold
   the rest dense. Gives each axis's *individual* effect.
2. **Single-axis restorations from the live pole** — flip one axis back to
   `dense`, hold the rest live. Gives each axis's *marginal* effect in context.

Where (1) and (2) disagree, the axes interact — which is itself the interesting
finding, and the reason a Shapley-style attribution over all 16 rungs is worth
computing rather than just the 8 single flips.

### 3.3 A non-saturated task

`CoincidenceTask` (`N_IN = 2`) must not be the substrate for this. Use SHD, or at
minimum a task where the dense arm scores in the 0.7–0.9 band rather than 1.0000.
A ceiling at 1.0000 ± 0.0000 makes every gap measurement an artifact of task
triviality. **This is non-negotiable for publication.**

### 3.4 Mechanism instrumentation

Accuracy alone will not explain anything. Log per rung:

- **Credit coverage** — fraction of synapses that receive any nonzero update per
  trial. Axis A should crush this.
- **Eligibility survival** — mean `|e|` at reward time divided by mean peak `|e|`
  during the trial. Axis D's direct observable, and the test of the
  "pre-absorbed traces" hypothesis.
- **Credit-sign agreement** — per-synapse sign agreement between the live update
  and the dense update on the same trial and seed. This is the cleanest measure
  of whether the substrate corrupts the *direction* of credit or only its
  magnitude.
- **Effective step size** — realised modulator RMS per rung
  (`ModulatorScale`). If it varies across rungs, part of the "gap" is a learning
  rate difference and must be normalised out before any mechanistic claim. This
  is exactly the defect that invalidated the SHD ceiling comparison; do not
  repeat it.

### 3.5 Rescue arms

For whichever axis dominates, implement and test the obvious fix. If D dominates:

- eligibility with a longer `τ_e` matched to the reward delay
- a two-timescale trace (fast + slow), as in `DualEligibility`
- explicit trace freezing at spike time until the reward arrives
  (the "pre-absorbed" hypothesis, made concrete)

A paper that identifies the mechanism **and** shows a fix that recovers a
material fraction of the gap is substantially stronger than one that only
diagnoses.

## 4. Preregistration sketch

- **H1** — a single axis accounts for ≥ 50% of the dense→live accuracy drop.
- **H2** — that axis is D (trace residency), the stated hypothesis.
- **H3** — the corresponding rescue arm recovers ≥ 50% of the gap.
- **Validity gates** — dense-pole accuracy in `[0.70, 0.95]` (not saturated);
  modulator RMS ratio across rungs ≤ 3.0; shuffled-label control at chance.
- **n ≥ 10 seeds**, all 16 rungs, no post-hoc rung selection.

## 5. Implementation notes

- The live path already exists: `runner.rs::run_local_assembly` plus
  `binn-engine`. The dense path is `binn-learn::matched_*`. The work is putting
  **one** configurable harness over both so the axes are switchable rather than
  reimplemented per substrate — otherwise the comparison acquires exactly the
  kind of hidden asymmetry the 2026-07-25 audit found in the SHD ceiling.
- Reuse `binn_lab::guards::{ReadoutAudit, StimulusProbe, Verdict}` for every
  rung. Sixteen rungs is sixteen chances to ship a constant predictor.
- Reuse `ModulatorScale` for the step-size normalisation.
- Budget: this is a multi-week build, not an overnight job. It is the main line
  of work once the SHD architecture question is settled.

## 6. Before writing a line of code

Do the literature search properly. Search terms to cover:

- "simulation to hardware gap" spiking local learning
- "deployment gap" neuromorphic plasticity
- e-prop / ETLP / DECOLLE **on-chip vs off-chip** accuracy discrepancy
- Loihi / SpiNNaker / BrainScaleS on-chip learning accuracy drop
- sparse winner-take-all credit assignment coverage

If someone has already characterised this, the contribution collapses to a
replication and the effort is better spent elsewhere. Fifteen minutes of
searching now is worth more than three weeks of building.
