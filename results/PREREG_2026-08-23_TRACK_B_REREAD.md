# Preregistration — re-reading track-b under the corrected instrument

**Registered:** 2026-08-23.

**This registration is compromised, and says so up front.** I have already run the
repaired instrument on the original 20 seeds and seen the outcome: 0/20
inversions, the harness warning cleared, E1.1 FAIL, E1.3 PASS. A preregistration
written after seeing the number is not a preregistration of that number, and no
wording fixes that.

Two things are done about it rather than around it.

1. **No threshold is invented here.** Every rule below already exists in the
   record and was fixed before any of this — so knowing the outcome cannot have
   shaped them.
2. **The verdict is decided on a disjoint seed set whose outcome I do not know.**
   The seeds I have seen become an exploratory set and decide nothing.

---

## 1. The reading rule — existing, unchanged

Taken verbatim from the shipped protocol (`track_b_results_v132.md`, the C1
runner, and `MATCHED_ARCH_RL_CONTROL.md`). **Nothing is added and nothing moves.**

| rule | value |
|---|---|
| gap-closed | `(arm − 0.5) / (ceiling − 0.5)`, clamped to `[0, 1]` |
| seeds excluded | reference separation `< 0.15` |
| PASS requires | gap LCB `> 0.5` **and** mean arm `≥ 0.65` |
| INVALID_HARNESS | mean ceiling `< 0.65` |
| inversion rule | any seed with raw gap-closed `> 1.0` ⇒ **no PASS permitted** |
| seeds | `REQUIRED_SEEDS = 20` |

The instrument changed, under
`PREREG_2026-08-23_MATCHED_ARCH_REPAIR.md`. The rule for reading it did not.

## 2. Exploratory set — already seen, decides nothing

`s_idx 0..20`, `seed = master_seed ^ (s_idx · 0x1000_0005)`. Outcome known and
recorded in `MEASUREMENT_2026-08-23_TRACK_B_UNDER_BOTH_REPAIRS.md`:

```
0/20 inverted   E1.1 0.5415 LCB 0.0039 FAIL   E1.3 1.0000 LCB 1.0000 PASS
```

Reported for completeness. **It is not evidence for the verdict**, because I saw
it before writing this.

## 3. Confirmatory set — the verdict

`s_idx 20..40`, the same lineage continued, **disjoint from anything run under
the repaired instrument**. Reached with a new `--seed-offset 20` flag that shifts
`s_idx` and changes nothing else; the flag is added before the run and its only
effect is the offset.

**The verdict for E1.1 and E1.3 is whatever the confirmatory set says, read by §1.**

### Registered expectation, which can fail

If the exploratory result is real, the confirmatory set should show:

- **0 or 1 of 20 inverted seeds** (the constructor-only condition gave 1/20 at
  3/20 baseline, so a single inversion is within what the instrument does);
- **E1.1 FAIL** — mean well below the 0.65 bar;
- **E1.3 PASS** — gap LCB `> 0.5`.

**If ≥ 2 seeds invert, the warning does not clear and no PASS is permitted.** That
is the outcome that would say the exploratory 0/20 was a lucky draw, and it is
reported as such rather than explained away.

## 4. Named outcomes

- **Confirmatory matches exploratory** → the reading is recorded, and
  `track_b_results_v132.md` is superseded by a v133 that carries both sets and
  the provenance of both repairs.
- **Confirmatory inverts ≥ 2 seeds** → the harness warning stands, both arms stay
  `INVALID_HARNESS`, and the exploratory 0/20 is recorded as a draw that did not
  replicate. **No third condition is tried.**
- **Confirmatory clears but E1.3 misses the LCB bar** → PASS is not granted. The
  arms are reported at their measured values with the warning cleared.
- **The two sets disagree on E1.1's direction** → n = 20 is too coarse for this
  contrast and that is the finding, not a verdict either way.

## 5. Mandatory disclosure with any PASS

This is a **disclosure, not a gate** — it cannot change a verdict, only what the
verdict is allowed to be read as. It is registered because the concern is real
and independent of which way the numbers fall.

Any PASS must be reported together with: E1.3's mean and variance, the ceiling's
mean, and their difference. On the exploratory set that is `1.0000 ± 0.000000`
against a ceiling of `0.9975`. **An arm at the ceiling on a task with no headroom
left is a statement about the task**, and a PASS obtained there does not
establish that the credit rule closes a gap — there is barely a gap to close.
That is what the original warning was reaching for, even though it named the
wrong causes.

## 6. What this may not claim

- **It does not revive the v132 verdict.** v132 was produced by a recurrent
  ceiling over a silent forward. Nothing here re-validates it; it is superseded or
  it stands, and either way its numbers are not comparable.
- **It does not settle the saturation question**, which needs a task with
  headroom, not a re-read of this one.
- **No gate moves.** `SHD_INSTRUMENT_STATE` stays `Uncalibrated`, Gate F stays
  10/10.
