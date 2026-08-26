# Result — the lead negative survives both graphs, and three of the four contrasts that gave it meaning do not

**Prereg:** [`PREREG_2026-08-25_MATCHED_ARCH_RERUN_ON_BOTH_FORWARDS.md`](PREREG_2026-08-25_MATCHED_ARCH_RERUN_ON_BOTH_FORWARDS.md),
committed in `c57a749` **before** the suites were run.
**Reports:** `results/matched_rerun_2026-08-25/`, eight of them, one per
(suite × forward graph). Every report names the graph it ran on.
**Registered outcomes fired: O-2 and, for EventProp, a reversal that O-2 covers
and that the abstract does not survive.**

---

## 1. Everything, in one table

n = 20 seeds, 80-epoch canonical budget, frozen splits, Gate G2 thresholds
unchanged. "Archived" is the 2026-07-23/24 record at `MATCHED_INPUT_SCALE = 0.5`,
on a forward pass that emitted **zero spikes at every seed**. The two right-hand
columns are the same arms at `2.0`, under the ceiling guard.

| arm | archived | recurrent | feed-forward | graphs differ by |
|---|---:|---:|---:|---:|
| **broadcast ±1 three-factor** (lead) | **0.5000** FAIL | **0.5100** FAIL | **0.5000** FAIL | 0.0100 |
| graded DFA | 0.9387 PASS | 0.9875 PASS | 0.9925 PASS | 0.0050 |
| broadcast-graded (honesty contrast) | 0.9863 | 0.9975 | 0.9975 | 0.0000 |
| REINFORCE × frozen `B_i` (primary) | 0.9200 PASS | 0.9812 PASS | 0.9950 PASS | 0.0138 |
| RL graded-reward broadcast | 0.5250 | **0.9100** | **0.8787** | 0.0313 |
| RL ±1 broadcast | 0.5113 | **0.7962** | **0.7775** | 0.0187 |
| **discrete EventProp spike-adjoint** | **0.5000 FAIL** | **0.8900 PASS** | **0.9450 PASS** | 0.0550 |
| SuperSpike BPTT ceiling | 0.8887–0.9150 | **1.0000** | **1.0000** | 0.0000 |

## 2. The lead claim survives, and it is now the only thing in §3.1 that does

**Broadcast ±1 three-factor stays at chance on both graphs** — 0.5000
feed-forward, 0.5100 recurrent — against a reference at 1.0000, with the
verdict `FAIL` on both (gap LCB 0.0000 and −0.0192 against a 0.5 bar; accuracy
0.51 against a 0.65 floor). The 0.0100 on the recurrent graph is four extra
correct answers spread over twenty 40-sample test splits.

That is the paper's headline, and the repair it survives is the one that
mattered most: `RESULT_2026-08-23_MATCHED_ARCH_REPAIR.md` established that the
archived 0.5000 came from a layer that could not spike, so a silent path and a
failing rule were indistinguishable. **Now only one explanation is left**, and
this is the 20-seed confirmation on the published protocol that the repair
document's 12-seed balanced-fixture spot check could not provide.

## 3. EventProp is not a negative result. It never was.

The abstract states:

> A discrete EventProp-style spike-adjoint head-to-head on the same matched
> forward also fails (`c1-eventprop-5bb083d5e88d0ad2`: 0.5000 against SuperSpike
> 0.9150).

**On a forward pass that can spike, it reaches 0.8900 on the recurrent graph and
0.9450 on the feed-forward one, and PASSes on both** (gap LCB 0.6494 and 0.7911
against a 0.5 bar).

The reason is not subtle and is worth stating plainly, because it should have
been predicted rather than measured: **a spike-adjoint method differentiates
through spike times, and the archived forward produced no spikes.** The 0.5000
was the arm having nothing to work with. Every other arm on that forward could
still separate the classes by sub-threshold membrane rate; the one method whose
entire mechanism is the spike could not, and it is the one that read as a clean
negative.

**This claim is withdrawn from the paper**, not restated. §4.3's framing —
"discrete ≠ continuous Wunderlich–Pehle" — was an explanation offered for a
number that had a different cause.

## 4. Three more contrasts moved, and two of them were FAILs

- **RL graded-reward broadcast: 0.5250 → 0.9100 / 0.8787.**
- **RL ±1 broadcast: 0.5113 → 0.7962 / 0.7775.**
- **Broadcast-graded on the DFA schedule: 0.9863 → 0.9975.**

The first two are the contrasts the paper leans on for *"continuous magnitude
without spatial directionality is information-theoretically insufficient on this
gate"*. Neither is at chance any more. The abstract's *"continuous
reward-prediction-error scalar broadcast … remains at chance (0.5120, LCB
−0.0230)"* was separately superseded on 2026-08-23, when the same arm under both
repairs read 0.5715 with a clean `FAIL` rather than `INVALID_HARNESS`
([`RESULT_2026-08-23_TRACK_B_REREAD.md`](RESULT_2026-08-23_TRACK_B_REREAD.md)).

## 5. And the task has stopped discriminating

**Every ceiling is now exactly 1.0000, on both graphs, in all four suites.**

`gap_closed = (arm − 0.5) / (ceiling − 0.5)`, so with the ceiling pinned at 1.0
every PASS in the table above reduces to *"the arm scored above 0.75"*. Five of
the seven arms sit between 0.88 and 1.00. The spread that made this comparison
informative — DFA 0.9387 against a ceiling of 0.8963, REINFORCE 0.9200 against
0.8887 — is gone, and what replaced it is a task every rule but one solves.

This is `RESULT_2026-08-19_A6_CEILING_HEALTH.md`'s saturation finding arriving in
full: it showed the reference climbing to 1.0000 by e640 and warned that *"any
future matched-architecture claim needs a task with headroom at convergence"*.
At `in_scale = 2.0` the reference reaches 1.0000 at the **canonical 80-epoch
budget**, so there is no budget at which this task separates the arms.

**The one thing it still does is separate broadcast ±1 three-factor from
everything else**, and it does that with the whole rest of the field at ceiling.
That is a weaker instrument than the paper describes and a sharper result than
the paper claims: not "richness and addressability are material factors on a
graded scale", but "one rule fails a task that every other rule tested now
saturates".

## 6. The ceiling guard did not fire, and that is informative

`guards::decide_matched_verdict` consults `CeilingHealth` first, and on the
archived numbers it would have returned `INVALID_HARNESS` for `c1_dfa` (0.9387
over a ceiling of 0.8963) and `c1_rl` (0.9200 over 0.8887).

**Neither fires now**, because the repaired ceiling reaches 1.0000 and nothing
can exceed it. The inversions were real and were an artefact of the same silent
forward: the reference was as crippled as the arms, and being a *gradient*
method it recovered less from the crippling than the local rules did.

So the guard's value here was diagnostic rather than protective. It would have
refused two published PASSes; the repair removed the condition that made them
refusable. Both facts belong in the record, and the second does not retire the
first — those two PASSes were, as published, comparisons against references
their treatments had beaten.

## 7. Which registered outcome fired

**O-2: the graphs disagree by more than 0.02 on at least one arm.** Two:
EventProp by 0.0550 and RL graded-reward by 0.0313. DFA (0.0050),
broadcast-graded (0.0000), REINFORCE (0.0138), RL flat (0.0187) and the lead arm
(0.0100) are all inside the bar.

So the paper's *"On that same forward family"* is **false as written** and cannot
be repaired by picking a graph: the failing arm and the passing contrasts were
measured on different graphs, and on the two arms where the graph matters it
matters by more than the registered threshold. The honest statement is that
these are **two comparisons**, and both are now reported.

**O-3 did not fire.** The lead FAIL did not move: same verdict, both graphs.

## 8. A defect this rerun exposed — the config hash did not cover the input scale

`MATCHED_INPUT_SCALE` was **not** mixed into any matched config hash. The
silent-initialisation repair moved it from 0.5 to 2.0, so re-running the
archived configuration afterwards produced `c1-eventprop-5bb083d5e88d0ad2`
reporting **0.8900** where the July record carries that same hash at **0.5000**.

One hash, two experiments, no way to tell from the label. The paper cites four
such hashes.

Fixed: the scale and the forward graph are now mixed into all four hash
families. **The archived hashes no longer resolve through `from_hash`**, which
is the correct outcome rather than a regression — this binary genuinely cannot
reproduce those numbers, and being told "unknown hash" is better than being
handed different ones under the name you asked for. Each config's frozen-hash
test now records the retired value beside the current one, so the break is
visible rather than inferred:

| suite | retired (scale absent from hash) | current |
|---|---|---|
| match | `c1-match-5dc6822e71229e9e` | `c1-match-6f6000f148f7d30c` |
| dfa | `c1-dfa-c8c4fe0899908b84` | `c1-dfa-f79c01ea36fe27d7` |
| rl | `c1-rl-42eddc9c801308e9` | `c1-rl-d35e13c758e522f8` |
| eventprop | `c1-eventprop-5bb083d5e88d0ad2` | `c1-eventprop-f1e841c29755b1c8` |

## 9. What this does not claim

- **It does not establish that any of the moved arms would pass a *good* gate.**
  They pass this one, and §5 is why that is worth little. A task on which five
  of seven arms sit above 0.88 cannot rank them.
- **It does not settle the local rule.** Whether broadcast ±1 three-factor can
  learn this task at some other operating point remains untested, and the
  stopping rule in `PREREG_2026-08-23_MATCHED_ARCH_REPAIR.md` §1 forbids
  searching for one.
- **It says nothing about a task with headroom**, which every conclusion above
  needs and none of them has.
- **It does not re-open Gate G2** or remassage `c1-118207fbc3eaba53`.
- **The archived numbers are not retracted as measurements.** They are what that
  instrument produced. What is retracted is the reading of them as measurements
  of the *rules*, on the arms where the silent forward was doing the work.
