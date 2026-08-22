# Preregistration — does the d32/L4 headline survive off the anchor?

**Registered:** 2026-08-20, before any wave-8 cell exists.
**Campaign:** `shd_attention_campaign_v2` (wave 8), same bucket, **same pinned binary**
`22d97c51ab0204702ce44661683ff8c759c29d7f3379e2f6606b048f4f032104` as waves 1–7.
**Supersedes nothing.** Extends `PREREG_2026-08-19_SHD_ATTENTION_CAMPAIGN.md` and
`PREREG_2026-08-20_SHD_ATTENTION_D32L4_AT_E400.md`.

---

## 1. Why this wave exists

`RESULT_2026-08-20_D32L4_CLEARS_THE_080_GATE.md` reports the campaign's headline:
d32/L4 at e400 reaches **0.8320** on the anchor (h128, `published-2ms`,
`adjacent-sum-5`), with 12/12 seeds ≥ 0.80 and a gain of **+0.1258** over
`ff+fixed`.

Every scope limit attached to that result was **measured at d32/L1**, not at
d32/L4:

| limit | source | measured at |
|---|---|---|
| gain inverts by h1024 (−0.0159) | `RESULT_2026-08-20_W3_SCOPE_LIMITS.md` W3-1 | d32/**L1** |
| `channels-700` gain not seed-consistent | same, W3-2 | d32/**L1** |
| contract axis | wave 3 `w3con` | d32/**L1**, and only at **e100** |

Wave 2 established that depth is the axis with the largest effect
(L1→L2 +0.0357, L2→L4 +0.0299). Carrying an L1 scope limit onto an L4 headline
is therefore an extrapolation across the one axis known to matter. This wave
measures those limits at the configuration the paper actually leads with,
rather than inheriting them.

**Either outcome is publishable and neither is preferred.** A confirmed limit is
a scope statement backed by measurement instead of by assumption; a refuted one
widens the claim. What is not acceptable is shipping the headline with an
untested scope paragraph.

## 2. Design

72 cells, n = 12 seeds (`5170001…5170012`), the campaign's standing lineage.

| label | arm | hidden | epochs | contract | geometry | attention |
|---|---|---:|---:|---|---|---|
| `w8geo` | `ff+fixed+attn` | 128 | 400 | `published-2ms` | `channels-700` | d32/L4 |
| `w8wid` | `ff+fixed+attn` | 512 | 400 | `published-2ms` | `adjacent-sum-5` | d32/L4 |
| `w8wid` | `ff+fixed+attn` | 1024 | 400 | `published-2ms` | `adjacent-sum-5` | d32/L4 |
| `w8con` | `ff+fixed+attn` | 128 | 400 | `published-10ms` | `adjacent-sum-5` | d32/L4 |
| `w8con` | `ff+fixed` | 128 | 400 | `published-10ms` | `adjacent-sum-5` | — |
| `w8lyr` | `ff+fixed+attn` | 128 | 400 | `published-2ms` | `adjacent-sum-5` | d32/**L2** |

### 2.1 Controls that are reused rather than re-run — and why that is legitimate

Three of the five comparisons need a matched `ff+fixed` control that **already
exists on disk**:

| comparison | control cells | from |
|---|---|---|
| `w8geo` | `ff+fixed` h128 e400 `channels-700` | wave 3 `w3geo` |
| `w8wid` h512 / h1024 | `ff+fixed` h512 / h1024 e400 | wave 3 `w3wid` |
| `w8lyr` L2 vs L1 vs L4 | wave 1 (L1) and the registered run (L4) | w1 / R |

They are reused because they were produced by **the same pinned binary on the
same fleet architecture with the same twelve seeds**. Re-running them would cost
~350 core-hours and could not produce a different number; the only thing it could
produce is a second copy to disagree with. `w8con`'s control has no counterpart
on disk at e400, so it is generated here.

This reuse is the reason the binary pin matters, and it is registered now rather
than justified later.

## 3. Hypotheses and thresholds

All thresholds are fixed here. Every verdict is computed **once**, after all 72
cells settle.

| id | claim | threshold |
|---|---|---|
| **S-1** | d32/L4 clears the gate on the standard 700-channel geometry | mean ≥ **0.80** *and* ≥ **9 of 12** seeds individually ≥ 0.80 |
| **S-2** | the attention gain survives that geometry | mean(`w8geo`) − mean(`w3geo ff+fixed`) ≥ **0.05**, positive in ≥ **10 of 12** seeds |
| **S-3** | depth rescues the width inversion wave 3 found at L1 | gain at h1024 ≥ **0.05**, positive in ≥ **10 of 12** seeds |
| **S-3b** | the width trend is monotone in the gain | gain(h128) ≥ gain(h512) ≥ gain(h1024) is **not** required; reported either way |
| **S-4** | the gain survives the coarser literature contract | gain on `published-10ms` ≥ **0.05**, positive in ≥ **10 of 12** seeds |
| **S-5** | *(mechanistic)* the gain shrinks when there are fewer timesteps to order | gain(`published-10ms`, t=72) ≤ gain(`published-2ms`, t=358) − **0.02** |
| **S-6** | the depth ladder is monotone at convergence, not only at e100 | mean(L1) ≤ mean(L2) ≤ mean(L4), both steps ≥ **0.0** |

### 3.1 S-5 is the one that can move the mechanism claim

`published-2ms` frames an utterance into 358 timesteps; `published-10ms` frames
the same utterance into 72. The read-out's attention matrix is `t × t`, so the
coarser contract gives it roughly a fifth of the temporal positions to order.

The campaign's mechanism claim (`RESULT_2026-08-20_W6…`, and the 12/12 shuffle
inversion in W1-3) is that the read-out buys **temporal order**, not capacity. If
that is right, the gain must fall when temporal resolution falls. **S-5 is a
directional prediction the mechanism claim makes and could fail.** If the gain is
flat or larger at t=72, the temporal-order reading is wrong or incomplete and the
paper must say so — the shuffle control establishes that order *matters*, not
that order is the *whole* mechanism.

## 4. Named outcomes

| id | SUPPORTED means | NOT SUPPORTED means |
|---|---|---|
| S-1 | the headline is not an artefact of the 140-input downsample; the paper drops the geometry caveat | the headline is scoped to `adjacent-sum-5`, stated as a measurement |
| S-2 | the effect, not just the accuracy, transfers to the standard input | the effect is geometry-specific — a genuine and reportable limit |
| S-3 | W3-1's inversion is a property of **L1**, not of width; the scope line changes from "scoped to h128" to "wide models need depth in the read-out" | width is a real limit and survives depth |
| S-4 | the effect is contract-general across both literature contracts | the effect needs fine temporal framing, which is itself mechanistic evidence |
| S-5 | the temporal-order mechanism makes a correct quantitative prediction | the mechanism claim is incomplete; the paper reports the shuffle result **without** the resolution story |
| S-6 | depth helps at convergence, not only at a truncated budget | the e100 depth ladder does not survive convergence; the depth claim is budget-scoped |

## 5. Validity gates

Per cell, unchanged from the campaign prereg §5 and enforced by
`analyse_campaign.py::validity_problems`:

- `non_finite_events == 0`
- `classes_predicted == 20`
- `majority_prediction < 0.30`
- `silent_fraction ≤ 0.95`
- `saturated_fraction ≤ 0.05`

Wave-level:

- **Every cell settles.** A cell that diverges is reported as `DIVERGED`, never
  as missing. An arm with any diverged cell is reported as `0 usable`, not as a
  mean over the survivors.
- **No gate verdict, nothing reportable.** Each instance runs cross-machine Gate F
  at boot and publishes its verdict; per campaign prereg §5.7 no number below is
  reportable without them.
- **Binary identity.** Every instance must report the pinned hash
  `22d97c51…f032104`. A mismatch aborts the instance; a campaign with mixed
  binaries is void, because the reused controls in §2.1 depend on it.

## 6. Stopping rule

Fixed at 72 cells. No cell is added, dropped, re-seeded or re-run on the basis of
its result. If a cell diverges it is reported as diverged; it is not replaced with
a different seed. Verdicts are computed once, after settlement, and reported
together.

## 7. What this wave may not claim

- **Not calibration.** The instrument is `Uncalibrated`; criterion 5 (a matched
  Python mirror of the attention axis) is unmet and no compute changes that.
- **No comparison to macOS-recorded numbers.** Cross-machine Gate F FAILs
  (`MEASUREMENT_2026-08-19_CROSS_MACHINE_BIT_EXACTNESS.md`); every comparison here
  is within-fleet against cells from the same pinned binary.
- **Not optimality.** Nothing here shows d32/L4 is the best configuration, only
  where the measured one does and does not hold.
- **No claim about `rec+alif`.** Wave 4 diverged 24/24 and that remains unmeasured.

---

# Addendum — wave 9, the mechanism control for the headline configuration

**Registered:** 2026-08-20, the same day and before any wave-9 cell exists.
**Runs:** after wave 8 drains, on the same bucket and the same pinned binary.

## 8. Why wave 9 exists

Wave 8 exists because wave 3's **scope** limits were measured at d32/L1 and then
carried onto a d32/L4 headline. The same error was made with the **mechanism**
claim, and it is the more serious of the two.

The temporal-order claim rests on W1-3: the bin-shuffled arm was worse than the
intact arm in **12 of 12** seeds at e400. That control was run at **d32/L1**. The
result the paper leads with is **d32/L4**. There is no shuffle control at the
headline configuration, at any budget.

`w9dim` closes a second gap: wave 2 found the gain monotone in `d_model` at e100,
and d32 is the only dimension ever run at convergence. d32 is therefore the
**tested** configuration, not a chosen one, and the paper should not imply
otherwise.

## 9. Design — 24 cells, n = 12, same seed lineage

| label | arm | hidden | epochs | contract / geometry | attention | temporal |
|---|---|---:|---:|---|---|---|
| `w9shf` | `ff+fixed+attn` | 128 | 400 | anchor | d32/L4 | **bin-shuffled** |
| `w9dim` | `ff+fixed+attn` | 128 | 400 | anchor | **d64**/L4 | intact |

Reused control: `ff+fixed` bin-shuffled at h128/e400 on the anchor, from wave 1.
Reused treatment: d32/L4 intact at e400, from the registered run.

## 10. Hypotheses

| id | claim | threshold |
|---|---|---|
| **M-1** *(primary)* | the temporal-order mechanism holds at the headline configuration | intact d32/L4 − shuffled d32/L4 ≥ **0.05**, and intact > shuffled in ≥ **10 of 12** seeds |
| **M-2** | shuffling costs the attention arm more than it costs the plain arm | (intact−shuffled) for d32/L4 ≥ (intact−shuffled) for `ff+fixed`, both at e400 |
| **M-3** | d32 is not merely adequate at depth | mean(d64/L4) − mean(d32/L4) reported with its sign; **no threshold, no verdict** — this is an estimate, registered as descriptive so it cannot be converted into a claim afterwards |

## 11. Named outcomes

| id | SUPPORTED means | NOT SUPPORTED means |
|---|---|---|
| M-1 | the mechanism claim is measured at the configuration the paper reports, not extrapolated from L1 | **the headline's mechanism is unexplained.** The paper reports the accuracy and drops the temporal-order reading for d32/L4 |
| M-2 | the read-out is what consumes temporal order, not the spiking layer | the order sensitivity is not specific to the read-out, and the attribution in W1-3 needs restating |
| M-3 | — | — |

**M-1 is the one that can cost the paper its mechanism.** It is registered as
primary precisely because there is no fallback reading if it fails: without a
shuffle control at d32/L4 the paper would be asserting a mechanism for a
configuration it never tested, which is the error this addendum exists to avoid
repeating.

## 12. Gates and stopping rule

Identical to §5 and §6 above, with the wave-1 temporal audit gates applying to
every `w9shf` cell: `counts_preserved` true and `relocated_fraction ≥ 0.5`, so a
"shuffle" that did not shuffle is caught rather than scored.

Fixed at 24 cells. No cell added, dropped, re-seeded, or re-run on the basis of
its result.
