# Amendment — the H26-3 ladder was paired against a wave that holds no `intact`

**Filed:** 2026-09-04, after the wave completed (264/264) and **after H26-1a,
H26-1b and H26-2a had been read**. This is disclosed rather than hidden: the
change was made post-unblinding and that is the weakest position from which to
touch an analyser.

**What was still blind when it was made:** the ladder itself. Every one of the
seven rungs printed `NOT EVALUABLE` and no numeric DiD existed for any window.
The fix could not have been steered by a ladder value because no ladder value
had been computed. τ½ was seen only after the change was written and tested.

## What was wrong

`did()` drew the `intact` arms from the same wave as the manipulated arms:

    ai = cells.get((wave, "ff+fixed+attn", "d32l4", readout, "intact"), {})

`w26win` carries no `intact` cells. That is not an omission — §2 of the
registration says so:

> H26-3 shares H26-2's `intact` and `bin-shuffled` rungs — the same cells, in
> the same wave, on the same binary — which is why they are not restated.

So the analyser did not implement what was registered. All 168 `w26win` cells
landed, were validated, and were then read as an absent effect at every rung.

This was visible only because of
[`AMENDMENT_2026-09-03_A_CLAUSE_THAT_DID_NOT_RUN.md`](AMENDMENT_2026-09-03_A_CLAUSE_THAT_DID_NOT_RUN.md).
Before it, the rungs printed `DiD None` beside real numbers; a reader — me —
could have transcribed seven absent rungs as seven measured nulls. The earlier
fix is what turned a silent wrong answer into a loud one.

## What changed

One optional parameter, `intact_wave`, and one call site passing
`intact_wave="w26pos"` for the ladder. No threshold, no bar, no estimator
algebra. `paired()` still requires a common seed across all four arms.

## Why the shared baseline is valid

Checked before the change, not asserted:

- `w26pos` attn and `w26win` attn are at **one** operating point and it is the
  same one: `hidden=128, epochs=400, published-2ms, adjacent-sum-5,
  attn_dim=32, attn_layers=4, n_train=8156, tau_m=10.050000191`.
- The rate arms likewise match on every field.
- `w26pos` seeds are a superset of `w26win` seeds, so every rung pairs on a
  real seed quadruple; all seven report n=12.
- One binary across the campaign, `434d38c2904e`.

## Evidence

Both new tests fail against the pre-fix analyser and pass against the amended
one:

    ERROR: test_the_ladder_is_paired_against_the_wave_that_holds_intact
    FAIL:  test_the_ladder_prints_numbers_once_it_is_paired
    Ran 18 tests — FAILED (failures=1, errors=1)   # pre-fix
    Ran 18 tests — OK                              # amended

## Consequence

The ladder is evaluable and τ½ is **WITHHELD** by a registered rule: half of
`DiD(bin-shuffled)` is 0.0604 and the ladder's top rung reaches 0.0596. The
half-maximum is not reached on the registered ladder, by 0.0008. That margin is
reported exactly as it fell. The rule was fixed in §1 of the governing
registration before any cell existed and is not revisited here.
