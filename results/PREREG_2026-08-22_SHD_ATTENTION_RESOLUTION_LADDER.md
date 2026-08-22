# Preregistration — the temporal-resolution ladder, on a contract family that isolates it

**Registered:** 2026-08-22, **before any wave-10 cell exists** and before the fleet
is launched.
**Campaign:** `shd_attention_campaign_v2`, wave 10, same bucket, **same pinned
binary** `22d97c51ab0204702ce44661683ff8c759c29d7f3379e2f6606b048f4f032104` as
waves 1–9.
**Extends:** `PREREG_2026-08-20_SHD_ATTENTION_HEADLINE_SCOPE.md` (S-4, S-5).

---

## 1. Why this wave exists

Wave 8's **S-5 was the campaign's one mechanistic prediction, and it failed**: the
gain was expected to shrink with fewer timesteps and instead grew
(`published-10ms`, +0.1491, against `published-2ms`, +0.1258).

Wave 9 then showed the mechanism itself is intact and strong — shuffling temporal
order destroys **96%** of the read-out's advantage (M-1, +0.1337, 12/12 seeds). So
S-5 did not refute the mechanism. It was **a badly constructed test of it**, and
this wave is the properly constructed one.

### What was wrong with S-5

`published-Xms` fixes the **bin width** and lets the number of timesteps float
with each utterance's duration. Comparing `published-10ms` to `published-2ms`
therefore varies, simultaneously: the timestep count, the bin width, *and the
per-sample variability of `t`*. Three things moved; one conclusion was drawn.

`fixed-tN` fixes the **window** (`SHD_FIXED_WINDOW_MS`) and divides it into exactly
`N` frames, so **every sample has the same `t`**. The ladder
`fixed-t100 / t250 / t500` therefore varies temporal resolution **and nothing
else** — same window, same corpus, same geometry, same width, same budget.

That is the axis S-5 meant to test and could not.

## 2. Design — 72 cells, n = 12, the standing seed lineage

| label | arm | hidden | epochs | geometry | attention | contracts |
|---|---|---:|---:|---|---|---|
| `w10con` | `ff+fixed+attn` | 128 | 400 | `adjacent-sum-5` | d32/L4 | `fixed-t100`, `fixed-t250`, `fixed-t500` |
| `w10con` | `ff+fixed` | 128 | 400 | `adjacent-sum-5` | — | same three |

Both arms are generated here; **no control is reused**, because no `fixed-t*` cell
exists at e400 for either arm anywhere in the record.

`published-10ms` at e400 is **not** re-run — wave 8 already has it (both arms,
n=12, same binary) and it enters §3 as a reference point only, never as a rung of
the `fixed-t*` ladder.

### Why this is being run on AWS

The Azure campaign that registered these contracts (`AZ8-5`) stopped at 95/252
when its credit was exhausted, and **none of its four `az8con` control arms ran**.
That subscription has no remaining budget, so the cells are not obtainable there.

AWS is a legitimate venue rather than a substitute: waves 1–9 ran on the same
pinned binary on aarch64, and
`FINDING_2026-08-22_REPRODUCIBLE_ACROSS_ISA_UNDER_GLIBC.md` establishes that
aarch64 and x86-64 agree on 57,960 of 57,960 serialised float values under glibc.
The one completed Azure `fixed-t250` treatment arm is therefore directly
comparable to what this wave produces — and §5 registers that comparison as a
falsifiable check rather than an assumption.

## 3. Hypotheses and thresholds

Fixed here. Every verdict computed **once**, after all 72 cells settle.

| id | claim | threshold |
|---|---|---|
| **C-1** *(primary)* | the read-out helps at every temporal resolution | at **each** of t100 / t250 / t500 **independently**: gain ≥ **0.05** and positive in ≥ **10 of 12** seeds |
| **C-2** *(mechanistic, two-sided)* | the gain depends on temporal resolution | \|gain(t500) − gain(t100)\| ≥ **0.03**, with the **sign reported** |
| **C-3** | the *baseline* is not what moves with resolution | \|mean(`ff+fixed`, t500) − mean(`ff+fixed`, t100)\| reported; if it exceeds **0.05** then C-2 is confounded and is reported as such |
| **C-4** | any rung clears the registered gate | mean ≥ **0.80** and ≥ 9/12 seeds ≥ 0.80, reported per rung |
| **C-5** | stability | zero non-finite events and zero diverged cells across all 72 |

### C-2 is registered two-sided, and here is why that is not a hedge

I do not have a theory that predicts the sign, and I am registering **after**
seeing S-5 go the opposite way to its prediction. Registering a one-sided
directional bar now would be choosing a direction with knowledge of related data —
the exact move preregistration exists to prevent.

So the bar is on **magnitude**, the sign is **reported**, and both directions have
named consequences in §4. What C-2 can do is fail: if the gain is flat across a
5× change in temporal resolution, then whatever the read-out consumes, it is not
resolution — and the shuffle result (M-1) would then be the *whole* mechanism
rather than part of it.

## 4. Named outcomes

| id | outcome | means |
|---|---|---|
| C-1 | SUPPORTED at all three | the effect is resolution-general; the paper drops the contract caveat |
| C-1 | SUPPORTED at some | the effect is scoped to the resolutions where it holds, stated as measurement |
| C-1 | NOT SUPPORTED anywhere | the anchor result is specific to `published-*` framing — a serious scope limit, and a genuine surprise given S-4 |
| **C-2 gain rises with t** | more timesteps to order ⇒ more to exploit. The reading S-5 predicted, now on a clean axis; S-5's failure is attributed to its confound |
| **C-2 gain falls with t** | S-5's *direction* replicates on a clean axis. The mechanism is order, but finer resolution dilutes rather than enriches it — a real finding needing its own explanation, and the paper says so without one |
| **C-2 flat** | resolution is not the axis. M-1's shuffle result stands alone as the mechanism, and no resolution story is told at all |
| C-3 > 0.05 | C-2 is confounded by the substrate, not the read-out; C-2 is reported as uninterpretable |

## 5. A registered cross-cloud check

The Azure run completed `az8con__ff-fixed-attn__h128__e400__fixed-t250__…__d32l4`
at 12/12 on **x86-64** with a different binary. This wave regenerates that exact
configuration on **aarch64**.

**Registered expectation:** the two arms agree on every serialised float value,
as the 57,960-value comparison predicts.

**If they disagree**, `FINDING_2026-08-22_REPRODUCIBLE_ACROSS_ISA_UNDER_GLIBC.md`
is wrong or incomplete and must be amended before any verdict in this wave is
reported. This is registered as a **falsification opportunity for a finding I
already published**, not as a confirmation exercise.

## 6. Validity gates

Per cell, unchanged and enforced by `analyse_campaign.py::validity_problems`:
`non_finite_events == 0`, `classes_predicted == 20`,
`majority_prediction < 0.30`, `silent_fraction ≤ 0.95`,
`saturated_fraction ≤ 0.05`.

Wave-level: every cell settles or is reported `DIVERGED`; an arm with any diverged
cell reports **0 usable**, never a mean over survivors; every instance must report
the pinned binary hash or the campaign is void; no gate verdict, nothing
reportable.

## 7. Stopping rule

Fixed at 72 cells. No cell added, dropped, re-seeded, or re-run on the basis of
its result. `fixed-t500` cells are ~40 core-hours each and are expected to be the
wall-clock floor; **a cell that does not finish is reported as unfinished**, not
replaced by a cheaper substitute.

## 8. What this wave may not claim

- **Not calibration.** The instrument is `Uncalibrated`; nothing here changes that.
- **No comparison to macOS-recorded numbers.** Gate F fails against Apple libm.
- **Not optimality**, and nothing about `rec+alif`, h1024, or `channels-700`.
- **`published-*` and `fixed-t*` are different contract families.** The ladder's
  three rungs may be compared to each other; comparing a `fixed-t*` rung to a
  `published-*` result is a cross-family comparison and is reported as one.
