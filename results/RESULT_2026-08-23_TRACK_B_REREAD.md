# Result — track-b re-read: the prediction held, and the PASS says less than it looks like

**Prereg:** `PREREG_2026-08-23_TRACK_B_REREAD.md`, committed in `65aa592`
**before** the confirmatory block ran.
**Supersedes:** `track_b_results_v132.md`.
**Artifact:** `results/track_b_results_v133_confirmatory.md`.

---

## 1. The registered expectation held out of sample

The prereg named, on a seed block whose outcome was not known: **0 or 1 of 20
inverted, E1.1 FAIL, E1.3 PASS.**

Confirmatory block, `s_idx 20..40`, full schedule n = 20:

```
Ceiling health: no seed exceeded the gradient reference; gap-closed is identifiable.
Seeds excluded for insufficient reference separation (< 0.15): RPE 0/20, learned-FB 0/20.
```

| arm | mean | SE | gap closed | LCB (95%) | verdict |
|---|---:|---:|---:|---:|---|
| **E1.1 Graded RPE Critic** | 0.5715 | 0.0352 | 0.1430 | 0.0051 | **FAIL** |
| **E1.3 Online Learned FB** | 1.0000 | 0.0000 | 1.0000 | 1.0000 | **PASS** |

**0 of 20 inverted. E1.1 FAIL. E1.3 PASS.** All three as registered, on seeds
whose outcome could have contradicted them and did not. §4's first named outcome
fires.

This is the part of the re-read that carries evidential weight. The exploratory
block (`s_idx 0..20`) was seen before the reading rule was written and decides
nothing; it is reported in
`MEASUREMENT_2026-08-23_TRACK_B_UNDER_BOTH_REPAIRS.md` and agrees.

## 2. The mandatory disclosure, and it is the headline

§5 registered — before the run, and independently of which way it fell — that any
PASS must be reported with the arm's mean and variance beside the ceiling's.
Here it is:

| | mean | variance |
|---|---:|---:|
| E1.3 Online Learned FB | **1.0000** | 0.000000 |
| Gradient Ceiling | **1.0000** | 0.000000 |
| difference | **0.0000** | — |
| Frozen REINFORCE×B_i | **1.0000** | 0.000000 |

**The arm, its reference, and a third arm all sit at exactly 1.0000 with zero
variance across 20 seeds.** There is no headroom at all. `gap_closed = (arm − 0.5)
/ (ceiling − 0.5) = 1.0000` is arithmetically correct and scientifically empty:
it says the arm matched a reference that had nothing left to bound.

**So the PASS is real by the registered rule and establishes nothing about credit
assignment.** Both statements are true and neither may be dropped. A rule that
solves a saturated task perfectly has demonstrated that the task is saturated.

The original v132 warning was reaching for exactly this — *"this indicates a
saturated task or an undertrained ceiling, not a credit-assignment result"* — and
it named **saturation first**. It was right about that and wrong about the
mechanism it blamed: the inversions came from a ceiling of the wrong architecture
over a forward that could not spike, not from undertraining.

## 3. What changed, and what it was worth

Two repairs, attributed by isolation in
`MEASUREMENT_2026-08-23_TRACK_B_UNDER_BOTH_REPAIRS.md`:

| condition | inverted | warning |
|---|---:|---|
| v132 as shipped | 3/20 | present |
| constructor only | 1/20 | present |
| both repairs (exploratory) | 0/20 | cleared |
| **both repairs (confirmatory)** | **0/20** | **cleared** |

The instrument is now correct: the ceiling shares its architecture with the arms
it bounds, and the shared forward can spike. What that bought is a **clean
negative for E1.1** (FAIL at 0.5715, LCB 0.0051, no longer masked by
`INVALID_HARNESS`) and an **honest view of E1.3** — not a credit-assignment
result, but a saturation result that can now be stated as one.

## 4. Verdicts

- **E1.1 Graded RPE Critic — FAIL.** Mean 0.5715 against a 0.65 floor; gap LCB
  0.0051 against a 0.5 bar. It fails on both criteria, and this is the first time
  it has failed *on its own merits* rather than being voided by the harness.
- **E1.3 Online Learned FB — PASS by the registered rule, uninformative in
  substance.** See §2. It may be cited as "matches its gradient ceiling on the
  matched dense-LIF schedule"; it may **not** be cited as evidence that learned
  feedback closes a credit-assignment gap.

## 5. What this may not claim

- **It does not revive v132.** Those numbers came from a recurrent ceiling over a
  silent forward. v132 is superseded, not corrected.
- **It says nothing about a task with headroom.** Every conclusion here is
  bounded by a ceiling at 1.0000. Whether learned feedback closes a gap is
  untested and needs a task that leaves one — which is the same open item
  `RESULT_2026-08-22_DEEP_PATH_AND_TRANSPORT_SCALE.md` §6 records for the depth
  suite, and for the same reason.
- **The exploratory block is not evidence.** It is reported for completeness and
  because concealing it would be worse; it was seen before the rule was written.
- **No gate moved.** `SHD_INSTRUMENT_STATE` stays `Uncalibrated`, Gate F 10/10.
