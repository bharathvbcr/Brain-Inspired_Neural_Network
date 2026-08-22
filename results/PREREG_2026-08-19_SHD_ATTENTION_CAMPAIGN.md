# Preregistration — SHD attention campaign, waves 1–4 (AWS, n=12 seeds)

**Registered:** 2026-08-19, **before any campaign cell was run.** The only prior
executions of the attention arm are the 15-cell pilot
(`RESULT_2026-08-19_SHD_ATTENTION_READOUT_PILOT.md`) and unit tests.
**Plan:** `scripts/aws/plan_cells.py` — 396 cells, enumerated deterministically.
**Supersedes nothing.** `PREREG_2026-08-19_SHD_ATTENTION_READOUT.md` is closed.

---

## 0. Why n=12, and why every wave carries its own control

**n=12.** This instrument has already demonstrated, on its own data, that n=3 is
not enough to publish on: H1 was SUPPORTED at three seeds and NOT SUPPORTED at
six, with the effect size and the standard deviation essentially unchanged — only
the resolution moved (`RESULT_2026-08-03_SHD_TEMPORAL_INFORMATION_H1.md` §4.1).
Every confirmatory contrast here runs at twelve.

**Every wave carries its own control arm, on the same machine.** The recorded
cells were produced on macOS/aarch64; the campaign runs on Linux/aarch64. `exp`,
`sin`, `cos`, `powf` and `ln` come from libm, and glibc is not obliged to agree
with Apple's to the last ulp — and this instrument amplifies one ulp into a
flipped spike and then through Adam. Whether they agree is **measured, not
assumed**: every instance runs Gate F against recorded cells before doing any
campaign work and writes PASS or FAIL to `gates/<instance>.json`.

- If Gate F **passes** on an instance, absolute comparison to the macOS record
  (0.7032 at h128/e400, 0.7378 converged) is licensed for that instance's cells.
- If it **fails**, those comparisons are unlicensed and **every claim below still
  holds**, because each is a difference between two arms that ran on the same
  machine from the same base weights and the same epoch orders.

No result in this campaign depends on the gate passing. That is a design
property, not a hope.

## 1. Wave 1 — does the pilot's +0.1702 survive convergence?

60 cells: `ff+fixed`, `ff+fixed+attn`, `ff+fixed` h192, and the bin-shuffled
control pair, all at **h128 / e400** — the budget at which the recorded width
axis is converged. 12 seeds.

| ID | statement | threshold |
|---|---|---|
| **W1-1** (primary) | The lift survives convergence | mean(attn) − mean(ff+fixed) **≥ 0.05**, and a paired two-sided t-test over 12 seeds at α=0.01 |
| **W1-2** | Not capacity | mean(attn) − mean(h192) **≥ 0.02** |
| **W1-3** | Temporal-order-derived | gain(intact) − gain(bin-shuffled) **≥ 0.02** |
| **W1-4** | Converged, not undertrained | `tail_loss_improvement` **> −0.02** in every attention cell — *threshold UNCHANGED, but see `DEFECT_2026-08-19_W1_4_THRESHOLD_IS_NOT_BUDGET_INVARIANT.md`: the statistic's window scales with the budget, so this bound also rejects the known-converged `ff+fixed` reference. Reported as registered, with the control's value beside it.*|

**W1-4 is the one that can invalidate the wave.** The pilot's attention arms sat
at −0.149 against the control's −0.011: they were still learning fast when the
budget ran out. If e400 does not close that, the accuracy is a budget artefact
and W1-1 is **reported as untested rather than as supported**, exactly as the
budget probe forced the ceiling claim to be withdrawn on 2026-08-03.

## 2. Wave 2 — the attention design space

96 cells at h128 / e100: `d_model ∈ {16, 32, 64, 128}` and
`layers ∈ {1, 2, 4}`, against a shared `ff+fixed` control, 12 seeds.

| ID | statement | threshold |
|---|---|---|
| **W2-1** | The effect is not an artefact of one width | accuracy is monotone non-decreasing in `d_model` across at least **2 of the 3** steps — *amended 2026-08-19 before any wave-2 cell was claimed; as first written this said "three of the four steps", which four values cannot supply. See `AMENDMENT_2026-08-19_W2_1_STEP_COUNT.md`* |
| **W2-2** | Depth is or is not worth it | report mean(layers=2) − mean(layers=1) and mean(layers=4) − mean(layers=2); **descriptive, no threshold** |

W2-2 is registered as descriptive on purpose. There is no prior that predicts a
direction, and inventing a threshold after seeing three numbers is how a sweep
becomes a story.

## 3. Wave 3 — the axes the 0.7378 ceiling is scoped on

216 cells. Width {128, 256, 512, 1024} at e400 with and without attention;
`channels-700` at e400 with and without; the four other timing contracts at e100
with and without. 12 seeds throughout.

| ID | statement | threshold |
|---|---|---|
| **W3-1** | Attention changes where width saturates | the attention arm's final width doubling (h512→h1024) gains **≥ 0.01**, against the recorded ff+fixed value of +0.000883 |
| **W3-2** | The lift is not a `adjacent-sum-5` artefact | the attention gain at `channels-700` is **≥ 0.05** |
| **W3-3** | Attention breaks resolution invariance | the spread of the attention arm's accuracy across the five contracts **exceeds 0.02**, against the recorded ff+fixed spread of 0.0034 |

W3-3 is the sharpest test in the campaign. Resolution invariance is what made
the forward look like a rate coder in the first place. A read-out that genuinely
uses cross-bin structure should *stop* being invariant to how finely time is
sliced. If it stays invariant, the mechanism story in the pilot is wrong even if
the accuracy is real, and that is what will be reported.

## 4. Wave 4 — is `rec+alif` unmeasured because of BPTT depth?

24 cells at h256 / e100, surrogate scale {1.0, 0.4}, clipping at 1.0, 6 seeds.

| ID | statement | threshold |
|---|---|---|
| **W4-1** | Attention does not inherit the recurrent instability | every `rec+alif+attn` cell completes with `non_finite_events == 0` |
| **W4-2** | Attention does not *rescue* the recurrent arm either | descriptive: report usable-cell counts for `rec+alif` and `rec+alif+attn` side by side |

W4-2 is descriptive because the honest prior is that it will not help: the
explosion is in the recurrent BPTT path, which attention sits beside rather than
inside. Registering it as a hypothesis would be inventing a prediction to be
right about.

## 5. Validity gates — a cell failing any of these is void, not a result

Unchanged from the pilot, and applied per cell: `non_finite_events == 0`;
`classes_predicted == 20`; `majority_prediction < 0.30`; `silent_fraction ≤ 0.95`;
`saturated_fraction ≤ 0.05`; for shuffled arms `counts_preserved` and
`relocated_fraction ≥ 0.5`. Plus, campaign-wide:

7. **Every reported wave must have its instance's Gate F verdict recorded**, pass
   or fail. A wave whose gate verdict is missing is not reportable — "the check
   did not run" must never read the same as "the check ran and passed".
8. **Cells are reconciled against the plan by id.** `collect.py` warns on any
   result not in the plan; a campaign with unplanned results is not analysed
   until the discrepancy is explained.

## 6. Stopping rule

**Twelve seeds. The verdicts are computed once, per wave, from the cells in the
plan, and reported whichever way they fall.** No thirteenth seed, no widening a
sweep because a trend looked promising, no dropping a cell that came out awkward.
A cell voided by §5 is reported as voided with its reason, not silently replaced.

Waves are independent: wave 1 failing does not cancel waves 2–4, and wave 1
succeeding does not license reading waves 2–4 more generously.

## 7. Named outcomes, before the run

| outcome | reading |
|---|---|
| W1-1 and W1-3 met, W1-4 met | The ceiling is a memory limit and attention reaches the missing structure at convergence. This is the paper. |
| W1-4 **not** met | Even at e400 the arm is undertrained; every accuracy in wave 1 is a budget artefact. Report as untested, extend the budget under a *new* registration, do not reinterpret these cells. |
| W1-1 met, W1-3 not | Attention helps, but not via temporal order. The pilot's mechanism claim is withdrawn; the accuracy result stands on its own and is described as pairwise structure, not memory. |
| W3-3 not met | Attention is accurate and still resolution-invariant. That is a genuinely confusing result and it is reported as one, not smoothed over. |
| W1-1 not met | The pilot's +0.1702 was a short-budget artefact. Report the negative; the attention axis does not enter any paper claim. |
