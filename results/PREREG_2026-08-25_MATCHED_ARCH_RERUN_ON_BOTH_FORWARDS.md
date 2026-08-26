# Preregistration — re-run every matched arm, on both forward graphs, under the ceiling guard

**Registered:** 2026-08-25, **before any post-repair matched-arch number
exists** and before the suite is run. Authorised by the maintainer, who asked
for the outstanding issues to be fixed before a larger campaign.

**Diagnoses this discharges:**
[`RESULT_2026-08-23_MATCHED_ARCH_REPAIR.md`](RESULT_2026-08-23_MATCHED_ARCH_REPAIR.md)
§6, which states that *"every matched-arch figure on record came from a forward
that could not spike, a ceiling of the wrong architecture, or both"*, and
[`TODO_2026-08-07_OPEN_WORK.md`](TODO_2026-08-07_OPEN_WORK.md) §1, whose whole
subject is the pattern **a fix that is not re-run is not a fix**.

---

## 1. Why this exists

Three repairs have landed since the matched-architecture reports were generated,
and **none of the reports has been regenerated.**

| repair | landed | what it changes |
|---|---|---|
| `MATCHED_INPUT_SCALE` `0.5 → 2.0` | 2026-08-22 (`db825aa`) | the forward could not spike at all; every arm and every ceiling ran on it |
| ceiling constructor in three binaries | 2026-08-22 (`db825aa`) | a recurrent ceiling bounded feed-forward arms |
| `guards::decide_matched_verdict` | 2026-08-25 | the verdict never checked whether the treatment **exceeded** its reference |

`results/c1_match.md`, `c1_dfa.md`, `c1_rl.md` and `c1_eventprop.md` are dated
**2026-07-23 and 07-24**. Every number the paper's §3.1, §3.2, §3.3 and §3.4 cite
comes from them. The repair document re-ran exactly one pair — matched-local
against matched-gradient on a 12-seed balanced fixture — and reported that those
two did not move. **The other arms were not re-run**, and the repair's own §6
forbids reading that as continuity: *"That `c1_match.md`'s numbers happen to be
unchanged is a finding, not a continuity."*

There is a second reason, and it is not a repair. The paper says the forward is
held fixed and only the rule changes. It is not:

| arm | report | graph |
|---|---|---|
| broadcast ±1 three-factor, and its ceiling | `c1_match.md` | **recurrent** |
| EventProp spike-adjoint, and its ceiling | `c1_eventprop.md` | **recurrent** |
| graded DFA, and its ceiling | `c1_dfa.md` | **feed-forward** |
| REINFORCE × frozen `B_i`, and its ceiling | `c1_rl.md` | **feed-forward** |

Within each pair the arm and its ceiling agree, so nothing inverted on that
account. Across pairs they do not, and the paper's central contrast — the ±1 rule
fails where DFA and REINFORCE pass — **is a comparison across pairs**. The
preregistration in `MATCHED_ARCH_RL_CONTROL.md:37` names the feed-forward
constructor; protocol v4 predates it and was never migrated.

So this campaign does not merely re-run. It turns the confound into an axis.

## 2. The matrix

Every arm × both forward graphs. `MatchedForward` is now an argument
(`binn-learn/src/matched_local_baseline.rs`), and each runner records the graph
it ran on in its own report.

| arm | binary | historical graph |
|---|---|---|
| broadcast ±1 three-factor vs SuperSpike BPTT | `c1 --matched-arch` | recurrent |
| graded DFA (+ broadcast-graded contrast) vs BPTT | `c1 --matched-dfa` | feed-forward |
| REINFORCE × frozen `B_i` (+ graded, + flat contrasts) vs BPTT | `c1 --matched-rl` | feed-forward |
| discrete EventProp spike-adjoint vs BPTT | `c1 --eventprop` | feed-forward *(new)* / recurrent |

**Held fixed:** `MATCHED_INPUT_SCALE = 2.0`, n = 20 seeds, the frozen splits, the
80-epoch canonical budget, every Gate G2 numeric threshold, and each suite's
existing seed lineage. **Varied:** the forward graph, and nothing else.

Each non-default graph gets its own config hash. The default-graph hashes are
unchanged **by construction** — `forward` is mixed into the hash only when it
differs from the suite's historical default — so `c1-match-5dc6822e71229e9e` and
its siblings still resolve through `from_hash` and every existing citation still
replays.

## 3. What the ceiling guard will do, stated before it runs

`decide_matched_verdict` now consults `CeilingHealth` **first**. On the archived
numbers it would return `INVALID_HARNESS` for two of the four suites:

| suite | treatment | ceiling | `CeilingHealth::evaluate` |
|---|---:|---:|---|
| `c1_dfa` | 0.9387 | 0.8963 | **Inverted** |
| `c1_rl` | 0.9200 | 0.8887 | **Inverted** |
| `c1_match` | 0.5000 | 0.8963 | Ok |
| `c1_eventprop` | 0.5000 | 0.9150 | Ok |

That classification is not new and is not a judgement made for this campaign:
`binn-lab/src/guards.rs` has asserted `evaluate(0.8963, 0.9387, 0.5) == Inverted`
in a unit test since 2026-08-21, using those exact two numbers as its worked
example. The binary that produced them never called it.

**So the expected outcome of this rerun is that two of the paper's four
matched-arch results stop being reportable as PASS.** Registering that in advance
is the point: if they come back PASS anyway, that is information, and if they come
back `INVALID_HARNESS`, nobody can say the guard was chosen after seeing them.

## 4. Named outcomes, every direction enumerated

Signs are named as well as magnitudes. The last five predictions in this
workspace's ablation series were wrong — twice on sign — and one outcome set had
no branch for what happened.

- **O-1. Both graphs agree, arm by arm, within 0.02.** The forward asymmetry was
  a bookkeeping defect with no scientific content. The paper may then state one
  forward and cite either. *This is the outcome I consider most likely and it is
  the one that would most flatter the existing record, which is why it is named
  first and why the bar is registered rather than judged afterwards.*
- **O-2. The graphs disagree by more than 0.02 on at least one arm.** The
  paper's "same forward family" sentence is then false as written and must be
  either re-scoped to one graph or re-stated as two comparisons. The lead claim
  survives only on the graph where both the failing arm and the passing arms
  were measured.
- **O-3. The lead FAIL moves.** Broadcast ±1 leaves 0.5000 on either graph at
  `in_scale = 2.0`. The lead claim is then **withdrawn pending re-registration**,
  not restated. It does not matter which direction it moves.
- **O-4. A ceiling is dead or inverted on a graph.** Reported as
  `INVALID_HARNESS` for that suite on that graph, with the arm and ceiling means
  printed beside it. No PASS/FAIL is issued. This is expected for `c1_dfa` and
  `c1_rl` on their historical graph (§3) and is a **result**, not a failure of
  the run.
- **O-5. An arm fails to converge or produces a degenerate readout** — constant
  prediction, silent layer, non-finite. Reported as a defect diagnostic with the
  initial activity band, and no accuracy is inferred from it.

**No arithmetic combines the two graphs.** They are separate measurements of the
same arm and there is no pooled number to report.

## 5. Stopping rule

The first complete n=20 run on each (arm, graph) is the result. Thresholds,
budgets, seeds and the input scale do not move after outcomes are visible. If an
arm comes back inverted, the response is to report it inverted — **not** to
search for a scale, budget or seed count at which it is not.

Trying input scales until one produces a usable ceiling is precisely what
`PREREG_2026-08-23_MATCHED_ARCH_REPAIR.md` §1's selection rule exists to
prevent, and that rule stands: `2.0` was chosen on initial firing rate with
accuracy explicitly not an input, and it is not reopened here.

## 6. What this cannot claim

- **It does not re-open Gate G2 or remassage `c1-118207fbc3eaba53`.** The G2
  numeric thresholds are reused unchanged under the existing `c1-*` hash
  families.
- **It does not settle whether a broadcast three-factor rule can learn this task
  at some other operating point.** That needs its own registration.
- **It says nothing about a task with headroom.** The coincidence task saturates
  — `RESULT_2026-08-19_A6_CEILING_HEALTH.md` shows every arm at 1.0000 by e640 —
  so nothing here bounds asymptotic capacity, only behaviour at the canonical
  budget. That limitation is unchanged by this campaign and is why
  `PREREG_2026-08-23_DEPTH_ON_A_TASK_WITH_HEADROOM.md` exists.
- **It does not vindicate any prior number.** If a figure comes back unchanged
  that is a finding about the repairs, not evidence that the old run was sound.
