# Finding — the matched architecture cannot spike, and the guard that would say so is not wired to it

> ## AMENDED 2026-08-23 — §3's objection was right and its prediction was wrong.
>
> The forward is repaired (`RESULT_2026-08-23_MATCHED_ARCH_REPAIR.md`): every
> hidden layer now sits inside the activity band and the ceiling on the same
> graph reaches 1.0000. **The local arm still scores exactly 0.5000 with zero
> variance across seeds, and the published numbers do not move.**
>
> So `MATCHED_ARCH_DFA_CONTROL.md`'s conclusion — *"the FAIL is the rule, not the
> path"* — is correct. What §3 below identified correctly is that the evidence
> for it was missing: a silent path and a failing rule produced the same 0.5000
> and nothing distinguished them. That is now settled, and it settled the other
> way from what §3 implied.
>
> §1 and §2 stand as measured. §3's reading of what the number meant is
> superseded.


**Found:** 2026-08-22, sweeping the ~6,400 lines of `binn-learn` reference
implementations that `FINDING_2026-08-22_A_SWEEP_OF_BINN_PROPER.md` §6 recorded as
the largest unswept gap. It was the right place to look.

**Bears on:** `results/c1_match.md`, `results/c1_eventprop.md`,
`results/MATCHED_ARCH_DFA_CONTROL.md`, and §3.1–3.4 of `PAPER_DRAFT.md`.

---

## 1. The forward is silent, and it is silent by arithmetic

`MatchedArch::with_options` (`binn-learn/src/matched_local_baseline.rs:112`) sets
`in_scale = 0.5`, so `win ~ U[−0.5, 0.5]`. The membrane leak is
`α = exp(−1/DEFAULT_TAU_M) = exp(−0.1) = 0.904837` (`:132`), the threshold is
`THETA_REST = 1.0`, and a unit fires on `ui >= theta` (`:163`).

Inputs are unit impulses on two channels. The largest membrane two adjacent
impulses can produce is

```
α · 0.5 + 0.5 = 0.952419  <  1.0
```

**Measured, 50 seeds × 8 coincidence-style inputs × 64 units = 400 forwards:**

```
total spikes = 0        max membrane ever = 0.974568   (theta = 1.0)
```

Zero. Not "low activity" — the hidden layer of the matched architecture cannot
emit a spike at initialisation, at any seed. `rates ≡ 0`, so `logit = by`, a bias
with no input dependence.

## 2. What that does to the published local arm

With a rate readout reading zeros, the local three-factor arm has no eligibility
to climb out on. Measured on balanced and unbalanced fixtures, against the
gradient ceiling on the identical forward:

| fixture | positive frac | majority frac | `MatchedLocal` | ceiling |
|---|---:|---:|---:|---:|
| balanced | 0.5000 | 0.5000 | **0.5000** | 1.0000 |
| 25% positive | 0.2500 | 0.7500 | **0.7500** | 1.0000 |
| 34% positive | 0.3438 | 0.6562 | 1.0000 | 1.0000 |

In the first two the arm returns **exactly the majority-class rate** — the
signature of a constant predictor. The published configuration is the balanced
one, and `results/c1_match.md:18` reports
`MATCHED_ARCH_LOCAL_THREE_FACTOR | 0.5000 | variance 0.000000`, with every one of
20 seeds reading exactly 0.5000 while the ceiling reads 1.0000 or 0.7250.
**Zero variance across 20 seeds on a stochastic learning rule is the tell.**

**Scope, stated honestly:** the third fixture reached 1.0000, so the arm is *not*
a constant predictor in every configuration, and I am not claiming it is. What is
established is that the forward starts silent for every seed, and that in the
published balanced configuration the reported number equals the majority-class
rate exactly.

## 3. Why this changes what the record says

`MATCHED_ARCH_DFA_CONTROL.md` reads the 0.5000 as:

> a broadcast ±1 reward rule fails at chance on an identical dense-LIF forward …
> the FAIL is the **rule**, not the path.

The forward is identical between the arms — that part is true, and it is why the
ceiling reaching 1.0000 on the same graph is a real contrast. But "identical"
has been carrying an implication it cannot support: that the shared path is
*functional*. It is not. It starts unable to spike, and the contrast being
measured is **which rule can climb out of a silent initialisation**, not which
rule learns the task. Those are different claims, and only the first is
supported.

This is the same defect class repaired this morning in `MatchedDeepGradient` and
`ShdArch` (`PREREG_2026-08-22_SILENT_INITIALISATION_REPAIR.md`) — a hidden layer
initialised below threshold, a circular trap where no spikes means no
eligibility means no weight growth means no spikes. It was not looked for in the
matched-arch family, which is what §3.1–3.4 of the paper rests on.

## 4. The guard exists, is documented as mandatory, and is not wired here

`binn-lab/src/guards.rs` defines `Degeneracy::ConstantPrediction` and
`Degeneracy::EqualsMajorityClass`, with `CONSTANT_PREDICTOR_EPS = 1e-4` — which
would fire on every one of the 20 published seeds. Its header states that
building a `ReadoutAudit` is **mandatory** for any experiment reporting an
accuracy.

```
$ grep -c "guards::" binn-lab/src/runner_match.rs binn-lab/src/runner_eventprop_match.rs
binn-lab/src/runner_match.rs:0
binn-lab/src/runner_eventprop_match.rs:0
```

Neither imports it. And the test that enforces the requirement,
`binn-lab/tests/report_verdict_guard.rs:44`, resolves its search root as
`CARGO_MANIFEST_DIR/experiments` — so **`binn-lab/src/runner_*.rs` is outside
what it can see**. The rule is real, the enforcement is real, and the two do not
overlap where these numbers are produced.

That is the part worth fixing first, because it is the reason this went unseen
for months rather than being caught the first time the number was printed.

## 5. What is not established here

- **No published number is corrected by this document.** The 0.5000 is what the
  code produces; what changes is what it means.
- **The DFA and gradient arms are not implicated.** They escape the silent
  initialisation — the same way `ShdSuperSpikeCeiling` did — because a surrogate
  derivative is non-zero below threshold and their updates grow `win` until it
  crosses. `matched_dfa_baseline.rs`'s arms clear a genuine 0.65 floor.
- **Whether raising `in_scale` rescues the local arm is untested**, and must not
  be tested by trying values until one works. It needs a preregistration with the
  operating point chosen by the activity band — exactly as
  `PREREG_2026-08-22_DEEP_PATH_AND_TRANSPORT_SCALE.md` §4 did, including its
  outcome for "the layer spikes and still does not learn", which is what happened
  there.

## 6. Related findings from the same sweep, recorded not fixed

All `[V]` unless marked. Full detail in the sweep transcript; the load-bearing
ones:

- **`matched_rl_baseline.rs:469, :713, :1014`** — the deep arms' readout update is
  `eta * (a - p) * e_out`, with **no reward term**, against a module header
  (`:24`) stating "Readout always uses the REINFORCE term `r·(a−p)`". The shallow
  arms at `:1191` do include it. `loss: 0.0` is hardcoded at `:533`, `:798`,
  `:1120`. These arms are now dead — v136 replaced them — but they remain
  exported and certified by a test a constant predictor passes.
- **The surrogate is evaluated after the reset** in the same module (`:421`,
  `:645`, `:925`), so a unit that just spiked gets the surrogate floor
  `1/(1+β)² = 0.028` while a near-threshold silent unit gets ~0.91. The gating is
  inverted.
- **`eprop_baseline.rs:144-148`** — `win`'s update reads `wout` *after* `wout` was
  updated on line 144. Worst single-step relative error 233%; after 120 epochs
  `max|win_shipped − win_correct| = 3.12` on weights initialised in ±0.6.
  `shd_alif.rs:787` takes a snapshot first and gets this right.
- **Four "identical forward" guarantees compare `0.0` with `0.0`** —
  `matched_local_baseline.rs:487`, `matched_rl_baseline.rs:1471`,
  `matched_dfa_baseline.rs:318`, `matched_eventprop_baseline.rs:219`. They are the
  only structural checks that the one-variable contrast holds, and they hold
  across different seeds, widths, and feedforward-vs-recurrent — because the
  logit is the bias in every case.
- **`matched_eventprop_baseline.rs:271`** — the only learning test asserts
  `accuracy > 0.45` with the comment "should beat near-chance". The constant
  predictor scores 0.5391 on that fixture and the arm scores 0.4609: it passes
  while sitting *below* a constant.
- **Three experiments pair a recurrent ceiling with feedforward treatment arms,
  and it is what produces the shipped harness warning.** Verified by inspection:

  ```
  track_b_rescue.rs:174       MatchedGradient::new            (wrec != 0)
  live_transfer_rescue.rs:137 MatchedGradient::new            (wrec != 0)
  continual_learning.rs:40    MatchedGradient::new            (wrec != 0)

  runner_rl_match.rs:268      MatchedGradient::new_feedforward
  runner_dfa_match.rs:244     MatchedGradient::new_feedforward
  a6_ceiling_health.rs:311    MatchedGradient::new_feedforward
  MATCHED_ARCH_RL_CONTROL.md:37  "`MatchedGradient` (`new_feedforward`)"
  ```

  Every `MatchedRl*` arm uses `feedforward`, so the ceiling carries an extra
  `hidden × hidden` matrix no arm has. The preregistration names the correct
  constructor; both sibling runners use it; the three experiment binaries do not.

  **The consequence is not cosmetic.** `track_b_results_v132.md` carries a
  headline *"HARNESS WARNING — ceiling inverted … no PASS is permitted while this
  warning is present"*, triggered by 3 of 20 learned-FB seeds exceeding a
  gap-closed of 1.0. Against an architecture-matched feedforward ceiling that
  falls to **1 of 20**, and all three inverting seeds are ones where the matched
  ceiling reaches 1.0000 while the shipped recurrent ceiling does not (0.9600,
  0.9500, 0.9500). The inversion is the reference losing to an extra recurrent
  matrix it should not have, not the arm beating the reference.

  The report's own diagnosis — *"a saturated task or an undertrained ceiling"* —
  names neither cause, and neither hypothesis points at a constructor.

  **And the diagnostic built to settle this question could never have caught it.**
  `a6_ceiling_health.rs` exists to answer "were the SuperSpike references
  undertrained?", and it uses the *correct* constructor. It does not share the
  defective call site, so the binary designed to catch this class of problem is
  structurally blind to this instance of it.

  An earlier draft of this bullet said the shipped table was "not numerically
  affected", from a partial run. That was wrong, and wrong in the direction that
  mattered — the full n=20 run reproduces the shipped numbers bit-faithfully and
  shows the effect lands exactly on the point the verdict turns on.
- **`[R]` The backward reset term drops `θ`** in four modules
  (`matched_local_baseline.rs:271` and siblings): `ds = g_r − du_next` where the
  forward's reset is `−θ·s_prev`. A no-op only because `THETA_REST == 1.0`.

## 7. Found clean, and worth recording

`shd_alif.rs` is the best-hardened module in the sweep: a wired `defects()` audit,
the activity band, `MAJORITY_PRED_MAX`, divergence reported rather than panicked,
a shuffled-label control, bitwise assertions on the frozen-attention block, and
the `wout` snapshot taken before the readout update — the exact bug
`eprop_baseline` has. `bptt_baseline.rs` and `shd_temporal.rs` also held up;
`shd_temporal::rebuild` merges same-channel collisions by adding counts, which is
precisely the bug `resting.rs::RateMatched` had and this module does not.
