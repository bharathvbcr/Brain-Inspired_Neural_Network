# What is left for the paper — 2026-08-20

Written after §1 closed. Everything below is either **done**, **blocked on one
named thing**, or **a decision for the author**. Nothing on this list is waiting
on compute.

---

## 1. §1 record repair is closed

`TODO_2026-08-07_OPEN_WORK.md` §1 existed to close one pattern: *a fix that is
not re-run is not a fix*. All six items are now settled, and **four of the six
settled by withdrawal rather than restatement**.

| item | outcome |
|---|---|
| re-run `track-b-rescue` at v131 | done — the v130 PASS is **withdrawn**; both arms `INVALID_HARNESS` |
| correct the six v130 citations | done — all six updated |
| re-run `deep-snn-scaling` at v134 | done 2026-08-20 — **every ceiling at chance**; suite `INVALID_HARNESS` |
| restate or withdraw the depth collapse | done 2026-08-20 — **withdrawn** |
| port the ceiling guard into `shd_scientific_sweep` | **superseded** 2026-08-20 — the binary never loaded SHD |
| re-run `ei-inhibition-sweep` at v135 | done — v135 report on disk |

`AUDIT_2026-08-07_JULY_CAMPAIGN_SCORING_PATH.md` is fully discharged.

### The pattern the closure exposed

Three independent suites were checked and **three gradient references turned out
to be at or near chance** on tasks their own treatments solve:

| suite | treatment | reference | reference vs chance |
|---|---:|---:|---|
| `deep-snn-scaling` v134 | 1.0000 | 0.4880 | chance = 0.5000 |
| `shd-scientific-sweep` v135 | 1.0000 (DFA) | 0.2140 | chance = 0.2000 |
| C1 matched arms (A6) | 0.9387 | 0.9013 at e80, **1.0000 at e640** | reference still climbing |

The first two are references that do not learn. The third is a reference that
learns but was stopped early. **In all three the treatment beat its own ceiling**,
and in all three the guard that would have caught it either did not exist, was not
ported, or was ported to a different binary.

This is a genuine methodological contribution and it is stronger than any single
arm result in the package. `BLOG_2026-08-03_THE_CHECK_THAT_CANNOT_FAIL.md` argued
it; this is three worked instances, one of them found inside the very module the
blog post's own suite depends on.

## 2. What the paper can still claim

| claim | status |
|---|---|
| broadcast ±1 three-factor fails the matched dense-LIF gate (0.5000, LCB 0.0000) | **stands**, with the A6 caveat below |
| graded DFA / REINFORCE×frozen-`B_i` clear that gate | **stands**, same caveat |
| live k-WTA transfer is a scoped negative (v13–v24) | **stands** |
| attention read-out on SHD: 0.8320 at d32/L4, 12/12 seeds ≥ 0.80, +0.1258 | **stands**, scoped to h128 / `published-2ms` / `adjacent-sum-5` |
| the read-out buys temporal order, not capacity (12/12 shuffle inversion) | **stands** |
| online learned FB alignment reaches 1.0000 (v130) | **withdrawn** |
| local learning fails with depth | **withdrawn** |
| anything from `shd-scientific-sweep` | **withdrawn** |

**The A6 caveat must be in the paper, not in a footnote.** `gap_closed` at the
canonical 80-epoch budget divides by a reference that is still climbing —
`RESULT_2026-08-19_A6_CEILING_HEALTH.md` shows it reaching 1.0000 by e640, and
values above 1 are an artefact of the denominator. The defensible statement is:
*the 80-epoch schedule undertrains every rule on it, and the ordering between
local rules and BPTT at that budget is a statement about learning speed.* The
coincidence task saturates and cannot support a ceiling comparison at high budget.

## 3. The one gate that blocks three separate lines

> **CORRECTED 2026-08-21 — this section's framing was wrong.** Criterion 4 does
> not refer to any BINN arm; it is three seeds of the external
> `Thvnvtos/SNN-delays` PyTorch baseline, and it is **already satisfied on the
> numbers** (0.9390 / 0.9368 / 0.9371 against a 0.80 floor), as is criterion 3.
> The gates are false for a **provenance** reason that
> `AMENDMENT_2026-08-03_REFERENCE_FINGERPRINT_SCOPE.md` already diagnosed,
> attempted, and withdrew. Criterion 5 — the attention Python mirror — is
> **downstream of those two and not currently reachable**, so it is not the
> binding constraint and the claim that "it was never a compute problem" is
> wrong: re-running the six reference cells is a GPU job.
> See [`FINDING_2026-08-21_CALIBRATION_GAP_IS_PROVENANCE_NOT_ACCURACY.md`](FINDING_2026-08-21_CALIBRATION_GAP_IS_PROVENANCE_NOT_ACCURACY.md).
> The original section is retained below.


`SHD_INSTRUMENT_STATE` is `Uncalibrated` (`binn-lab/src/instrument_status.rs:9`),
a compile-time constant with no flag or environment override. It refuses
`LocalLearning`, `Transfer` and `Optimizer` campaigns. Three things wait on it:

1. **the frozen-attention local arm** — the only G2-relevant attention variant
   (`BLOCKER_2026-08-19_FROZEN_ATTENTION_LOCAL_ARM.md`);
2. **`temporal-deep-campaign`** — which uses `shared_bptt`, the *validated*
   replacement for the broken `MatchedDeepGradient` ceiling, and is therefore the
   only instrument that could answer the depth question properly;
3. **any re-run of `shd-scientific-sweep`** — verified by running it: exit 3.

Calibration PASS needs five criteria (`SHD_INSTRUMENT_STATUS.md`). Four are met
or mechanical. **Criterion 5 — at least one matched Python/Rust configuration
passing every registered gate — needs a Python mirror of the attention axis in
`scripts/shd_calibration/arms.py` that does not exist.**

> **Open question for the author, and it decides how far away calibration is.**
> Criterion 4 is *"three clean reference seeds, each at least 0.80"*. The
> feedforward reference `ff+fixed` reaches **0.7062** at e400 — below the bar. The
> attention arm `ff+fixed+attn` reaches **0.8320** with 12/12 seeds ≥ 0.80, and it
> is also a surrogate-gradient BPTT model, not a local rule. **If the attention
> read-out counts as part of the reference architecture, criterion 4 is met and
> only the Python mirror remains. If it does not, criterion 4 is still open too.**
> This is a definitional call, not a measurement, and it has not been made.

The standing instruction defers the Python arm (`TODO` §8), so **this is a
decision, not a task.** No amount of compute changes it.

## 4. Author-only work

| item | why it cannot be delegated |
|---|---|
| **`PAPER_DRAFT.md` abstract** | rewritten to reflect withdrawal of v130 PASS, depth-collapse, and synthetic sweep; leads with ±1 three-factor insufficiency, DFA/RL contrasts, and SHD attention results. |
| **Reframing around the new evidence** | the depth result is gone, the sweep is gone, the matched-arm claims are re-scoped to learning speed, and the strongest positive result is now on SHD rather than on the coincidence task. The paper's centre of gravity moved. |
| **arXiv endorsement (cs.NE)** | externally controlled, multi-day, and the longest-lead item in the week plan. Still `[~]`. |
| **retire vs rename `shd-scientific-sweep`** | the bin target name reaches `Cargo.toml`, `overnight.sh`, `run_all_experiments.sh` and several documents. |

## 5. Engineering work that is real but not blocking

| item | state |
|---|---|
| **A1 — commit the dirty tree** | **not done, and it is a provenance risk.** 20+ modified files; `git remote -v` is empty. A paper that cites a repo URL needs a pushed repo. |
| **§3 transfer-gap decomposition** | designed (`DESIGN_TRANSFER_GAP_DECOMPOSITION.md`), and the design itself says *"a multi-week build, not an overnight job"*. Not this week. |
| **26 serial experiment binaries** | each is the same ~15-line parallelisation. Only worth doing per binary, when that binary is next needed. See `MEASUREMENT_2026-08-20_EXPERIMENT_PARALLELISM.md`. |
| **A8 — LaTeX + figures** | not started; blocked on §4 above, not on data. |
| **§6 audit debt** | ~8,000 unswept lines in `binn-engine` / `binn-areas` / `binn-core`. A scope call. |

## 6. Campaigns — waves 1–9 complete as of 2026-08-22; wave 10 in flight

| campaign | outcome |
|---|---|
| AWS wave 8 (72 cells) | **complete**, 0 failures. S-1 NOT SUPPORTED (`channels-700` 0.7864), S-2 SUPPORTED (+0.1090), S-3 NOT SUPPORTED (h1024 −0.1618), S-4 SUPPORTED (+0.1491), S-5 NOT SUPPORTED, S-6 SUPPORTED |
| AWS wave 9 (24 cells) | **complete**, 0 failures. **M-1 SUPPORTED** (+0.1337, 12/12), **M-2 SUPPORTED**, M-3 descriptive (+0.0121, no verdict) |
| AWS wave 10 (72 planned) | **in flight** as of 2026-08-22 — 8 cells landed, 24 claimed, four `c7g.16xlarge` still running. No wave-10 cell has been collected to local disk yet, so nothing here is evaluable ([`SUMMARY_2026-08-22_CAMPAIGN_AND_RECORD_REPAIR.md`](SUMMARY_2026-08-22_CAMPAIGN_AND_RECORD_REPAIR.md)) |
| Azure (252 planned) | **stopped at 95** — operator deallocated, Azure credit exhausted; **not relaunchable**. AZ8-1/3/4 no data, AZ8-5 not evaluable, AZ8-2 NOT SUPPORTED, AZ8-6 **VOIDED** (6/12 degenerate) |

Fleets torn down — the wave-1–9 AWS instances terminated, Azure nodes
deallocated; the four wave-10 `c7g.16xlarge` instances are still running. AWS
spend across all nine waves ≈ $65, **as of the end of wave 9**; with wave 10 the
current total is ≈ $77 across ten waves
([`SUMMARY_2026-08-22_CAMPAIGN_AND_RECORD_REPAIR.md`](SUMMARY_2026-08-22_CAMPAIGN_AND_RECORD_REPAIR.md)).

**The strongest reproducibility evidence in the project came out of the Azure
failure.** Its surviving cells were the expensive h1024/h512 arms — the ones AWS
had also run — giving a 36-cell, **57,960-value cross-architecture comparison with
zero differing values** (aarch64 vs x86-64, different binaries, full 400-epoch
trajectories). The 2026-08-19 "not reproducible across machines" reading is
superseded: the divergence is **Apple libm vs glibc, not the ISA**. See
[`FINDING_2026-08-22_REPRODUCIBLE_ACROSS_ISA_UNDER_GLIBC.md`](FINDING_2026-08-22_REPRODUCIBLE_ACROSS_ISA_UNDER_GLIBC.md).

### The mechanism, now measured at the headline configuration

| | intact | bin-shuffled |
|---|---:|---:|
| d32/L4 gain over `ff+fixed` | **+0.1258** | **+0.0049** |

96% of the read-out's advantage is contingent on temporal order, 12/12 seeds. The
paper may claim **order**; it may not claim **resolution** (S-5 failed).

### Scope limits, now measured rather than assumed

- **h128 only.** The gain inverts by h1024 (−0.1618) and **depth does not rescue
  it** — that was wave 8's question, and the answer is no.
- **`adjacent-sum-5` only.** On the standard 700-channel geometry the effect
  survives (+0.1090) but the **0.80 gate does not** (0.7864).
- **d32 is the tested configuration, not the chosen one** (M-3, no verdict).
