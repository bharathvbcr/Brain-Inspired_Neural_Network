# Measurement — track-b under the two matched-arch repairs, attributed

**This is a measurement, not a verdict.** `track_b_results_v132.md`'s
`INVALID_HARNESS` stands. §3 says why, and it is not a formality.

**Repairs:** `RESULT_2026-08-23_MATCHED_ARCH_REPAIR.md` — the input scale
(`0.5 → 2.0`) and the ceiling constructor (`new` → `new_feedforward`).

---

## 1. The three conditions

Full schedule, n = 20, identical in every other respect.

| condition | inverted seeds | E1.1 mean | E1.1 LCB | E1.3 | harness warning |
|---|---:|---:|---:|---|---|
| **shipped v132** — recurrent ceiling, silent forward | **3 / 20** | 0.5120 | −0.0230 | INVALID_HARNESS | present |
| **constructor only** — feedforward ceiling, silent forward | **1 / 20** | 0.5120 | −0.0230 | INVALID_HARNESS | **still present** |
| **both repairs** | **0 / 20** | 0.5415 | 0.0039 | PASS | **cleared** |

## 2. What each repair did, separately

**Neither repair alone clears the warning.** That is the useful result, and it is
why the isolation run was worth ten minutes.

- **The constructor accounts for two of the three inversions.** Removing the
  `hidden × hidden` matrix the ceiling carried and the arms did not takes 3/20 to
  1/20. It does **not** move a single arm number: E1.1 is 0.5120 with LCB −0.0230
  in both conditions, bit-for-bit. That is exactly what a pure ceiling swap should
  do, and it confirms the change touched the reference and nothing else.
- **The initialisation accounts for the last inversion**, and it is the only one
  of the two that moves the arms: E1.1 goes 0.5120 → 0.5415. The arms were reading
  a forward that could not spike; now they are not.

Both were real, they are independent, and the earlier finding's estimate for the
constructor alone (3/20 → 1/20) reproduces exactly.

## 3. Why no verdict is recorded here

Two reasons, and the second is the one that binds.

**The confound was resolved, but late.** The first re-run moved both variables at
once. Reading a PASS off it would have attributed to the constructor an effect
that needed the initialisation too. The isolation above fixes that — but the
attribution came after the headline number was already known.

**The reading was not registered before it was read.** I ran the re-run and saw
`E1.3 = PASS` before writing down how a re-run should be read.
`RESULT_2026-08-23_MATCHED_ARCH_REPAIR.md` §6 had said this warning "stands until
a re-run under the corrected constructor is **registered** and read", and then I
read one that was not registered. Re-running after a blocked verdict and reading
a PASS is the precise shape that a preregistration exists to guard, and the guard
does not work retroactively.

What is defensible, and is the reason this is worth recording at all: **the
constructor change was mandated by the pre-existing preregistration.**
`MATCHED_ARCH_RL_CONTROL.md:37` names `MatchedGradient (new_feedforward)`. The
repair restored a registered design; it was not selected to produce an outcome.
That separates it from tuning. It does not substitute for registering the
reading.

**So `track_b_results_v132.md` is unchanged, and E1.1 and E1.3 remain
`INVALID_HARNESS` in the record.**

## 4. What would close it

A preregistration that fixes, before anything is re-read: the schedule, the
inversion tolerance, and what each of PASS / FAIL / INVALID_HARNESS means under a
ceiling that is now architecture-matched — including the possibility that a
saturated E1.3 at 1.0000 against a ceiling at 0.9975 is *still* not a
credit-assignment result, which is what the original warning was reaching for
even though it named the wrong causes.

That last point deserves stating plainly: E1.3 sits at **1.0000 with variance
0.000000**. A PASS there is a statement about a task with no headroom left, and
the honest reading may be that the task is saturated regardless of which ceiling
it is measured against.

## 5. Process note

The isolation run was done by editing `MATCHED_INPUT_SCALE` **in the shared
working tree** while two other sessions commit to this branch every few minutes.
For roughly ten minutes the tree held a value that was not the committed one; a
`git add -A` from either session would have swept it up. It was restored as soon
as it was noticed, and the running binary was already compiled so the measurement
itself is unaffected.

The right pattern was available and was not used: the helper-consolidation work
earlier today did exactly this kind of experiment in a throwaway git worktree,
for exactly this reason. Recorded so the next isolation uses a worktree.
