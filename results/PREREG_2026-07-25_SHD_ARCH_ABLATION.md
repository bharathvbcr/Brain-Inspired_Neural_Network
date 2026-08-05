# Preregistration — SHD architecture ablation

**Registered:** 2026-07-25, before any run of `shd-arch-ablation`.
**Protocol version:** 141
**Binary:** `cargo run --release -p binn-lab --bin shd-arch-ablation`
**Claim axis:** architecture confound in the `c1-shd-cal-*` result.

This document fixes the hypotheses, thresholds and stopping rules **in advance**.
Nothing below may be edited after the first full run; amendments go in a new file
with a new timestamp.

---

## 1. Motivation

`c1-shd-cal-*` reports DFA ≈ 0.234 on SHD (20 classes, chance 0.05) and treats it
as a statement about local credit assignment. The forward model that produced it
is **feed-forward with no `W_rec` and a fixed threshold**.

Published results on the same dataset:

| Method | Locality | SHD accuracy |
|---|---|---:|
| BPTT + learned delays (Hammouamri et al. 2023) | none | 0.951 |
| e-prop | local in time, non-local in space | 0.808 |
| ETLP (Quintana et al. 2024) | fully local three-factor | 0.746 |
| this project (feed-forward, fixed θ) | fully local three-factor | **0.234** |

ETLP's stated conclusion: *"when using local plasticity, threshold adaptation in
spiking neurons and a recurrent topology are necessary to learn spatio-temporal
patterns with a rich temporal structure."*

Both ingredients are absent here. **The 0.234 figure is therefore confounded**:
it may describe the ceiling of a feed-forward rate readout on a temporal task
rather than a property of local credit assignment.

## 2. Design

Full crossing, frozen data split, common seed lineage:

```
{feed-forward, recurrent} × {fixed θ, adaptive θ} × {DFA, e-prop ceiling}
+ shuffled-label control on the best DFA architecture
```

- Recurrence: `W_rec`, zero diagonal (a self-loop is a threshold change in
  disguise and would confound the adaptation axis).
- Adaptation: `θ_i(t) = THETA_REST + β_a · a_i(t)`, `a_i(t+1) = ρ·a_i(t) + s_i(t)`,
  `τ_a = 20`, `β_a = 0.18`.
- Both credit arms build their hidden modulator at `shd_out_scale(hidden)`, so
  effective step size is set by `lr` alone.

**Schedule (default):** `hidden = 128`, `seeds = 3`, `epochs = 15`, `lr = 0.02`,
capped splits 2000/500. `--full` switches to official uncapped 8156/2264.

### Two-stage learning-rate design

`lr = 0.02` is inherited from `c1-shd-cal-*` and was never tuned for a recurrent
or adaptive forward. Declared in advance:

- **Stage 1 (pilot).** `--lr-sweep`: DFA only, all four architectures, at
  `{0.005, 0.02, 0.08}`, 2 seeds, 8 epochs. This is a **pilot**. It may inform
  which learning rate a later confirmatory run uses; it may **not** be reported
  as the confirmatory result.
- **Stage 2 (confirmatory).** The default run at the fixed `lr = 0.02`.

Where several learning rates are present in one report, H1/H2 use the **best
non-degenerate cell per architecture**. Selecting the best LR per architecture is
declared here in advance and applies symmetrically to baseline and treatment, so
it is not a forking path.

### Execution-order guarantee

Cells run in H1-critical order (`ff+fixed`, then `rec+alif`, then the two
interaction cells), and the report is rewritten after every cell. A truncated run
therefore still answers the preregistered contrast rather than losing everything.
Partial reports hold all verdicts at `UNDERPOWERED`.

## 3. Hypotheses and thresholds

| ID | Statement | Threshold |
|---|---|---|
| **H1** | Architecture is the binding constraint | `rec+alif` DFA − `ff+fixed` DFA ≥ **0.10** absolute, **and** their 95% CIs across seeds are disjoint |
| **H2** | Architecture closes most of the gap | best-architecture DFA ≥ **0.50** |
| **H0** | Neither holds | no cell clears `ff+fixed` by 0.10 |

## 4. Validity gates (dominate H1/H2)

A run is `INVALID_HARNESS` — **no H1/H2 claim permitted** — if any fails:

1. **Negative control.** Shuffled-label DFA on the best architecture must score
   ≤ `chance + 0.05`. Above that, the pipeline leaks and every number is void.
2. **Modulator parity.** The DFA/e-prop realised modulator RMS ratio must stay
   ≤ **3.5** in every architecture cell. This is the defect that made the
   original e-prop ceiling score *below* its own DFA treatment (≈56× effective
   learning-rate deficit at `h = 128`).
3. **Both H1 cells non-degenerate.** See below.
4. **Real data.** A run outside `--quick` that loads the smoke fixture aborts
   with exit code 3. A fixture run is indistinguishable from a real one in the
   report, so it is a hard error rather than a warning.

### Per-cell degeneracy (the failure mode most likely to invert the conclusion)

A recurrent network can **collapse** (predicts one class), go **silent** (barely
spikes) or **saturate** (runaway recurrent activity). All three produce
chance-level accuracy. Read as a bare number, that is indistinguishable from
*"recurrence does not help"* — which would flip H1 from PASS to FAIL for a purely
mechanical reason.

Every cell is therefore checked and flagged:

| Flag | Condition |
|---|---|
| `COLLAPSED` | predicts a single class across the whole test set |
| `NEAR-COLLAPSED` | > 95% of predictions in one class |
| `SILENT` | mean hidden activity < 0.001 spikes/neuron/step |
| `SATURATED` | mean hidden activity > 0.5 spikes/neuron/step |

Degenerate cells are **excluded** from best-cell selection, and their accuracies
may not be cited as statements about the credit rule. Training that produces
non-finite weights or activity panics rather than reporting a NaN accuracy.

Additionally, any cell where the ceiling scores below its DFA treatment is
flagged `INVERTED` and that cell's comparison is not interpretable.

**If `rec+alif` comes back `SATURATED`, H1 has not been tested.** The fix is
`rec_scale` in `ShdAlifArch::new`, not the hypothesis.

## 5. Decision rules — what each outcome obliges

**H1 PASS, H2 PASS.**
The 0.234 result was an architecture artifact. Obligations: restate the SHD claim
axis; mark `c1_shd_h128/256/512.md` superseded on architecture grounds as well as
ceiling grounds; re-run width and depth sweeps on `rec+alif` before any scaling
claim; do not cite any pre-2026-07-25 SHD number.

**H1 PASS, H2 FAIL.**
Architecture matters but does not close the gap to ETLP's 0.746. Obligations,
in order: (a) implement the exact ALIF e-prop eligibility term `ε_a` — the
current trace omits it, see §7; (b) per-arm learning-rate sweep; (c) only then
attribute the residual to locality.

**H1 FAIL (H0).**
Architecture is not the constraint. This is a legitimate and more interesting
negative result. But it may **not** be written up until the two confounds in §7
are eliminated, because either could mask a real architecture effect.

**INVALID_HARNESS.**
No claim. Fix the flagged gate, re-run. Per the U-NEG protocol, an
`INVALID_HARNESS` run may not be cited as positive evidence anywhere downstream —
this rule was violated by the 2026-07-24 summary and is now enforced in code
(`Verdict::is_citable_as_positive`).

## 6. What this may not claim

- Not SOTA.
- Not Gate G2.
- Not a like-for-like ETLP comparison: different eligibility formulation,
  different training schedule, capped splits unless `--full`. The reference table
  is context, not a benchmark result.
- Not a neuromorphic hardware claim — no hardware was involved, and no energy or
  latency was measured.

## 7. Known confounds, disclosed in advance

1. **Eligibility approximation.** The trace is the LIF form
   `e ← α·e + σ'(u−θ)·pre`, extended to `W_rec`. Exact ALIF e-prop carries an
   additional per-synapse adaptation term
   `ε_a ← σ'·ε_v + (ρ − σ'·β_a)·ε_a`, with `e = σ'·(ε_v − β_a·ε_a)`. It is not
   implemented (a second `h × n_in` array per example). Consequence: the adaptive
   arm's eligibility ignores how a spike suppresses its own synapse's future
   eligibility through the threshold. **This biases against H1.**
2. **No learning-rate sweep.** `lr = 0.02` is inherited from `c1-shd-cal-*` and
   was never tuned for a recurrent or adaptive forward. **Direction of bias
   unknown.**
3. **Capped splits by default.** 2000/500 rather than 8156/2264. Under-training
   relative to published numbers. **Biases all arms downward.**

## 8. Analysis plan

- Per-cell mean, SE and 95% CI across seeds (normal approximation on seed means).
- H1 tested on the `rec+alif` vs `ff+fixed` DFA contrast only. No other pairwise
  comparison is a preregistered test; all others are descriptive.
- No post-hoc cell selection: `rec+alif` is named in advance as the H1 arm.
- The shuffled-label control uses the best DFA architecture by mean accuracy;
  this is a validity gate, not a hypothesis test.

## 9. Stopping rule

One run at the stated schedule. If `INVALID_HARNESS`, fix and re-run once, and
record both runs. No selective reporting of seeds, cells or schedules.
