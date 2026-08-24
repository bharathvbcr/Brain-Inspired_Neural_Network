# Preregistration — does attention substitute for recurrence, at the operating point?

**Registered:** 2026-08-23, **before any wave-14 cell exists** and before the
fleet is launched.
**Campaign:** `shd_attention_campaign_v2`, wave 14, same bucket, **same pinned
binary** `22d97c51ab02`.

---

## 1. The last open half of the substitution question

Wave 12 refuted substitution on the **adaptation** axis: attention's gain is
+0.1258 on `ff+fixed` and +0.1285 on `ff+alif`, a difference of +0.0027 that is
positive in 6 of 12 seeds. The **recurrence** axis was deferred because no
recurrent arm could complete a 12-seed arm at the anchor budget.

Wave 13 found where it can: `rec+alif` at **surrogate scale 0.4** completes
**11 of 12** at h128 / `published-2ms` / `adjacent-sum-5` / e400. It also found
that adaptation is what makes that possible — `rec+fixed` fails by saturation,
ten cells voided with up to 52% of hidden units pinned at maximum firing — so
`rec+alif` is the only recurrent arm worth measuring.

## 2. Design — 36 new cells, 12 reused, n = 12

| arm | attention | surrogate scale |
|---|---|---:|
| `rec+alif` *(reused from wave 13)* | — | 0.4 |
| `rec+alif+attn` | d32/L4 | 0.4 |
| `ff+fixed` | — | 0.4 |
| `ff+fixed+attn` | d32/L4 | 0.4 |

h128, `published-2ms`, `adjacent-sum-5`, e400 throughout.

**Every arm runs at scale 0.4, including the feed-forward pair, and that is the
point.** The 24 archived anchor controls ran at the registered default of 1.0.
Comparing a gain measured at 0.4 against one measured at 1.0 would confound the
substrate with the scale — the exact confound this wave exists to avoid. So the
feed-forward pair is regenerated at 0.4 rather than reused, at a cost of 24
cells, and every comparison below is within one scale.

**`rec+alif` is not regenerated.** Wave 13 ran this configuration exactly — same
arm, width, budget, contract, geometry, scale, seeds and binary — and the
instrument is deterministic, so re-running would produce byte-identical cells.
The eleven completing cells are reused; seed 5170002 diverged and is carried as
diverged.

## 3. Completion gates the measurement

Wave 11's lesson, and this wave inherits the exposure: an arm that diverges more
often can look better, because only its luckier trajectories survive to be
scored.

**M-0 (gating).** For any comparison to be reportable:

* each arm in it completes **≥ 11 of 12**, and
* the comparison has **≥ 10 seed-pairs where both arms completed**.

Failing either, that comparison's hypotheses are **NOT EVALUABLE** — no mean is
reported for it and no verdict is issued. Nothing is retried and no threshold
moves.

**Every comparison is paired on seed**, and computed only over pairs where both
arms completed. The seed fixes the initial weights and the epoch order for both
arms, so a matched pair removes the variance the seed contributes and — more
importantly here — compares the *same* trajectories rather than two differently
filtered subsets. The surviving-pair count is printed beside every number.

## 4. Hypotheses and thresholds

Fixed here. Every verdict computed **once**, after all 36 cells settle. Write
`gain(S) = mean(S + attn) − mean(S)` over surviving pairs, all at scale 0.4.

| id | claim | threshold |
|---|---|---|
| **M-1** *(primary)* | attention helps on a recurrent, adaptive substrate | gain(`rec+alif`) ≥ **0.05**, positive in ≥ **10** surviving pairs |
| **M-2** *(primary, two-sided)* | the gain depends on whether the substrate is recurrent | \|gain(`rec+alif`) − gain(`ff+fixed`)\| ≥ **0.03**, both at scale 0.4, **sign reported** |
| **M-3** | recurrence plus adaptation alone reaches the gate | mean(`rec+alif`) ≥ **0.80** with ≥ **9** of its completing seeds ≥ 0.80; reported against mean(`ff+fixed`) at the same scale |
| **M-4** *(descriptive, no verdict)* | the scale is not quietly crippling the baseline | mean(`ff+fixed`) at 0.4 reported against the archived 0.7062 at 1.0 |

## 5. Named outcomes — all three directions this time

Wave 13's §4 named two of the three possible directions for its two-sided
hypothesis and the data landed on the third. That is not repeated here.

| id | outcome | means |
|---|---|---|
| **M-2, gain smaller on `rec+alif`** | attention was partly standing in for recurrence. Combined with wave 12, the mechanism claim would then be about *temporal state in general*, and the paper says the read-out substitutes for state the feed-forward substrate lacks. |
| **M-2, gain larger on `rec+alif`** | attention and recurrence are complementary rather than alternative — the read-out adds more where there is more temporal structure to index. A positive claim, and one that needs its own explanation rather than an assumption. |
| **M-2, flat** (< 0.03) | substitution is refuted on the recurrence axis as it was on adaptation. Together with wave 12 that is the strong form: the read-out's advantage is indifferent to what the spiking layer carries, and M-1's shuffle result stands as the whole mechanism. |
| **M-1 NOT SUPPORTED** | attention does not help a recurrent substrate, which would be evidence for substitution regardless of M-2's magnitude. |
| **M-3 SUPPORTED** | the biologically-motivated architecture reaches the registered gate without attention. That is a materially different paper, and wave 12's `ff+alif` result (0.7018, 0/12 over the gate) makes it the less likely of the two. |
| **M-4 shows a degraded baseline** | if `ff+fixed` at 0.4 falls materially below 0.7062, the cross-substrate comparison in M-2 is between a healthy recurrent arm and a weakened feed-forward one, and M-2 is reported as scale-limited rather than clean. |

## 6. What this wave may not claim

* **One scale, one width, one contract, one budget.** Nothing here generalises
  off the operating point, and the operating point is not the anchor: the anchor
  runs at scale 1.0.
* **Nothing about `rec+fixed`.** Wave 13 measured it and it does not complete.
* **No winner by inspection.** If an arm tops the table it is reported with its
  seeds and converted into no claim, for the reason wave 9 recorded for M-3.
* **Not calibration.** No comparison to macOS-recorded numbers.

## 7. Stopping rule and cost

Fixed at 36 new cells. No cell added, dropped, re-seeded or re-run on the basis
of its result. A cell that does not finish is reported unfinished.

`rec+alif+attn` at d32/L4 is the wall-clock floor. Estimated ~8 h on ~250 vCPU
of spot, roughly $20.
