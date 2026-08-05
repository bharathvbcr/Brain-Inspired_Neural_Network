# Build Audit (v10) — second, independent pass

**Date:** 22 July 2026
**Auditor:** static source review (no Rust toolchain in the review sandbox — every Rust distribution channel is network-blocked here; host compile still required).
**Method:** adversarial re-read of `binn-engine`, `binn-learn`, `binn-data`, and `binn-lab`, with three independent reviewers fanned out over engine mechanics, plasticity/reference sign conventions, and task-balance/gate math. Every claim below is grounded in a file:line citation checked against the on-disk source.

**Verdict in one line:** the substrate is still **engineered well and the v9 correctness fixes hold**, but this pass finds **two substantive issues the v9 audit missed** — (1) the local learning path is structurally too weak and one-sided to be a fair test of the thesis, and (2) the G2 kill-gate contains a real **false-PASS route**. Neither a PASS nor a clean FAIL is trustworthy until both are addressed. A working-tree inconsistency (below) also needs reconciling first.

---

## 0. State of the working tree (read before anything else)

The tree is **self-consistent and should compile**, but it is in a *partially-fixed* state relative to the previous session's notes:

| Change | Status in source | Evidence |
|---|---|---|
| Unbiased readout tiebreak (replace `charge_1 >= charge_0` fixed-`>=` with larger-charge + seed-parity tiebreak) | **PRESENT** | `binn-lab/src/runner.rs:1085-1095` (`let diff = charge_1 - charge_0; … (t0 & 1) as u32`) |
| Reward-prediction-error modulator (`rpe = reward − baseline`) + baseline threading | **ABSENT / reverted** | `runner.rs:1110-1112` is still raw `let reward = if correct {1.0} else {-1.0}; … Modulators::reward(reward)`; zero `reward_baseline` references anywhere; `run_trial` is back to its 10-arg signature (`runner.rs:1007-1018`) |

So the modulator is currently **raw ±1 reward with an implicit zero baseline**. Any earlier claim that the RPE fix is in place is **not true of the current tree**. This does not break the build (the reverted state is the original, coherent code), but a re-run today would exercise the *un-improved* learner. Re-apply the RPE change (or decide against it) before regenerating a verdict.

---

## 1. What is confirmed solid ✅

**The v9 correctness fixes are real and verifiable in source.**

- **Task labels are exactly 50/50, not merely balanced in expectation.** `CoincidenceTask` sets the label by trial-ordinal parity (`datasets.rs:110,121`), and the ordinal is exact because `next_sequence` consumes exactly `sequence_len` samples per trial (`synth.rs:135,142-155`). Train (`n_train=80`) and test (`n_test=40`) are both even ⇒ exactly balanced. **Chance = 0.5** for a constant predictor.
- **The v9 wrap-around bug is fixed.** Peaks use linear, non-wrapping distance (`datasets.rs:123-135`); the `% len` path is gone; `debug_assert_eq!(label, positive)` enforces construction ≡ rule (`:137-141`); constructor guarantees `sequence_len ≥ max_lag+2` (`datasets.rs:89`); balance test `coincidence_task_is_label_balanced` exists (`datasets.rs:200-210`).
- **STDP sign convention is correct and internally consistent.** `stdp(dt>0)` returns `+A₊·exp(−dt/τ₊) > 0` (causal ⇒ potentiation, `eligibility.rs:31-40`). Both branches of `apply_spike_stdp` feed the same `t_post − t_pre` into `stdp` (`three_factor.rs:170-192`) — no sign mismatch. End-to-end, a **correct** action (reward `+1`) drives `dw = η·e·(+1) > 0` on the chosen readout's incoming edges; a wrong action drives `dw < 0`. Credit assignment points the right way.
- **The reference baselines are honest.** All conditions share one per-seed `FrozenSplit`, with the test set drawn from a *separately seeded* task instance (`runner.rs:562-568`); references consume the same examples (`:704-717,724-725,781-782`); test labels are used only for scoring (no leakage); each is explicitly labeled a reference, not an upper bound, and enforced by `header_forbids_production_use` tests. The surrogate-LIF ceiling shares the engine's `θ/V_reset/α` constants — a genuine same-architecture reference, not a rigged-easy one. `EpropReference` is a true forward-eligibility rule (no reverse-time loop, unlike `BpttBaseline`/`SurrogateLifReference`) — `eprop_baseline.rs:115-152`.
- **Determinism is intact.** Timing wheel with FIFO tie-break for equal ticks, checked against a heap reference (`queue.rs`); CSR fan-out in index order; k-WTA deterministic (`wta.rs:27-39`); no `HashMap`/`HashSet` iteration in any hot path; the learner sorts new spikes by tick before applying STDP (`three_factor.rs:99-103`); `seed_identical_spike_train` asserts bit-identical trains.
- **Encoder/metrics are correct.** `LatencyEncoder` is intensity→latency; the runner injects it per-frame at `t0 + frame_i·frame_stride + ev.t` (`runner.rs:1031-1042`), so the coincidence (t0 vs t1) timing is faithfully preserved. `sparsity` and `work_per_accuracy` compute as documented.
- **Gate precedence is correct.** `decide_g2_verdict` checks `InvalidHarness` (bad positive control **or** sparsity out of band) **first**, then Pilot (quick/insufficient seeds), then Pass/Fail (`runner.rs:1378-1387`). A broken harness can never emit PASS/FAIL. `gate_g2` requires **both** `gap_closed_lower > min` **and** `mean_local ≥ min_accuracy` (`:1363-1369`). Lower bound uses the correct standard error `mean − z·√(var/n)` with sample variance (n−1) (`:1300-1344`).

---

## 2. New finding A — the local learner is structurally too weak to be a fair test ❌

Three mechanisms, each cited, compound so that the local-assembly condition is unlikely to learn *even with* the tiebreak fix and *even if* the RPE fix is re-applied:

**A1. The readouts never fire — the spike-based decision is dead code.** Every cell (including both readouts) starts at `theta = 1.0` (`cell.rs:19,80`; `engine.rs:77`), and readouts are outside the hidden `Area`, so they keep `theta = 1.0` the whole trial. With `init_w = 0.15` and `k_wta = 2` winners, a readout receives at most ≈`2 × 0.15 = 0.30` of somatic drive (`cell.rs:208-210`, `g_c = 1.0`) — **0.30 < 1.0, so it cannot cross threshold.** `fired_0`/`fired_1` are therefore ~always false and `pred` always falls through to the `charge_1 − charge_0` comparison (`runner.rs:1077-1095`). The decision is *de facto* `0.15 × (winners wired to r)` per readout — a fixed-topology comparison, not a learned spiking readout. This isn't fatal on its own (the charge comparison is a valid linear readout), but "the readout learns to fire for its class" is illusory in the current numbers.

**A2. Credit assignment is one-sided.** Only the **selected** readout is force-spiked (`runner.rs:1100-1102`), so only the chosen action's incoming edges ever receive a postsynaptic-LTP eligibility event (`three_factor.rs:183-192`). The non-chosen readout accrues no eligibility. The learner is REINFORCE-on-the-chosen-arm: it can raise or lower the arm it *took*, but never directly credit the arm it *should* have taken. An RPE modulator does not fix this asymmetry — it only rescales the signal on the arm that was already selected.

**A3. Uniform decay bleeds the readout edges.** `apply_weights` applies `−λ·w` to **every** synapse on every update (`three_factor.rs:126-137`, `λ = 0.002`), while readout edges receive eligibility only intermittently and only for the selected arm (A2). Between the sparse LTP events, decay pulls those edges back toward zero, further weakening an already thin learning signal.

**Consequence.** A FAIL produced by this configuration would be ambiguous in exactly the way the v8 spec warns against — it could reflect a crippled learner rather than the thesis. **The saving grace is the positive-control gate:** a trivially separable task that this learner cannot solve trips `InvalidHarness` (positive_control_mean < 0.90) *before* any FAIL is emitted (`runner.rs:1381-1383`). So as long as that gate is enforced, the weak learner yields `INVALID_HARNESS`, not a fake negative — which is the correct outcome. The fix is to strengthen the learner (two-sided readout credit, and/or lower readout `theta` or raise drive so the spike path is live) until the positive control genuinely clears 0.90, then re-run.

Minor, same family: winners are force-spiked while their `theta = f32::INFINITY`, relying on `INF ≥ INF` to fire (`runner.rs:1027,1069-1071`); and the fallback that wires an unreachable readout from input cell `n_in` (`runner.rs:1191-1195`) delivers no charge in the decision window, so such a readout is a constant-zero-charge arm.

---

## 3. New finding B — a real false-PASS route in the kill gate ❌ (most important)

The normalized gap-closed statistic is `closed = (local − dense) / (gradient_reference − dense)` per seed, guarded only by a `1e-6` denominator floor (`runner.rs:1329-1334`). The sign pathology is handled (when `dense ≥ reference`, `closed = 0`), **but there is no upper clamp and no requirement that the reference beat dense by a meaningful margin.**

Spurious-PASS scenario: if across seeds `dense ≈ 0.60`, `gradient_reference ≈ 0.61`, `local ≈ 0.65`, then `closed = 0.05 / 0.01 = 5.0` on every seed → low variance → `gap_closed_lower_95 ≫ 0.5`, and `mean_local = 0.65 ≥ 0.65` → **PASS** — even though local beats dense by only 5 absolute points. The gate is rewarding a *weak reference*, not a strong local model. (A single noisy seed with tiny gap does not do this — it inflates the sample variance and drives the lower bound down; the risk is a *systematically* weak reference across all seeds.)

This matters more than Finding A because it is a defect in the **trustworthiness machinery itself** — the one thing the whole project exists to protect. Recommended: clamp `closed` to `[0, 1]`, and/or require a preregistered minimum `reference_gap` (reference must beat dense by, say, ≥ 0.15) before a seed counts. This is the single change I would prioritize.

---

## 4. Minor / disclosed

- **z vs. Student-t.** The lower bound uses the normal critical value `z = 1.96` rather than `t₁₉,.975 = 2.093` at n=20, so it is slightly anti-conservative (PASS marginally easier). It is disclosed as a "normal approximation" (`config.rs:44-49`), so this is a documented choice, not a hidden bug. Consider `t_{n−1}` at these sample sizes.
- **eprop reference scope.** `EpropReference` is feedforward-only (no recurrent eligibility) and uses a soft sigmoid rate rather than hard spikes — a rate-model approximation, honestly labeled "e-prop-*compatible*" (`eprop_baseline.rs:35`) rather than a faithful spiking e-prop. Fine as a reference; worth a one-line caveat in the results note.

---

## 5. Fix list (in priority order)

1. **Bound the gap-closed metric** (Finding B): clamp to `[0,1]` and/or require a minimum absolute `reference_gap`. This closes the false-PASS route.
2. **Reconcile the tree** (§0): decide on the RPE fix and either re-apply it cleanly (with the `reward_baseline` threading and all three call sites) or drop it deliberately. Don't leave the half-applied state.
3. **Strengthen the local learner** (Finding A) until the positive control clears 0.90 on its own: give the non-chosen readout a credit path (two-sided update), and make the spike readout live (lower readout `theta` toward the ≈0.30 drive, or raise `init_w`/fan-in) so the decision isn't purely the init-topology charge comparison.
4. **Run the host toolchain** — still not done in any audit sandbox: `cargo test --locked --workspace`, `cargo clippy --all-targets -- -D warnings`, `./scripts/gc_checks.sh`, then `cargo run -p binn-lab --bin c1`.
5. Optional: `t_{n−1}` critical value; one-line eprop caveat.

---

## 6. Bottom line

Engineering: **still solid**, and the v9 correctness work genuinely landed. But this second pass changes the risk picture in two ways the first missed. The G2 gate has a concrete route to a **false PASS** through a weak reference (Finding B) — fix this first. And the local learner is currently too weak and one-sided to be a fair test of the thesis (Finding A); the positive-control gate correctly converts that into `INVALID_HARNESS` rather than a fake FAIL, so the negative is *safe but not yet earned*. Plus the tree lost the RPE fix and needs reconciling. Address §5.1–§5.3, get the positive control over 0.90 honestly, re-run on a host, and *then* — most likely — a trustworthy FAIL. As the v8 prior says, that remains the expected and successful outcome.
