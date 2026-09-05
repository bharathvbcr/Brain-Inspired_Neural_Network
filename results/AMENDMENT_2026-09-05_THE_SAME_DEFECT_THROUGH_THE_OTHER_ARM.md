# Amendment — the same defect, arriving a second time through the other arm

**Filed:** 2026-09-05, after wave 27 completed (264/264) and **after H27-1,
H27-2, H27-3, H27-5 and H27-6 had been read**, along with the accuracy half of
H27-4. Disclosed rather than buried.

**What was still blind when it was made:** the qk-norm DiD. It printed
`NOT EVALUABLE (n=0)`; no value existed for it, and the `did()` call that
produces it returned nothing. The fix could not have been steered by it.

## What was wrong

`did()` addressed four arms with two waves. It assumed the rate arms live in
the same wave as the attention arms:

    bi = cells.get(arm_key(intact_wave, False, "intact", ...))
    bx = cells.get(arm_key(wave,        False, condition, ...))

`w27qk` carries **only attention cells**, by design: §5 compares a qk-norm
attention arm against the campaign's rate arms, which are wave 26's. So both
rate lookups landed in a wave that holds none, and the DiD half of a registered
two-part bar could not be computed with all 24 of its cells present and valid.

## This is the wave-26 defect, not a new one

[`AMENDMENT_2026-09-04_THE_LADDER_HAD_NO_BASELINE.md`](AMENDMENT_2026-09-04_THE_LADDER_HAD_NO_BASELINE.md)
fixed exactly this shape — an estimator looking for arms in a wave that does not
hold them — by adding `intact_wave`. **That fixed the instance and left the
shape.** The assumption "all four arms share a wave" merely weakened to "all
four arms share two waves", and the defect returned through the arm the first
fix did not touch.

The standing rule is *fix the class, not the case*. The first amendment did not,
and this is what that costs: the same bug, a wave later, found only because the
three-state verdict made it loud.

**The class is fixed here.** Every arm may name its own wave: `intact_wave` for
the unmanipulated arms, `baseline_wave` for both rate arms. `paired()` still
requires a common seed across all four, so a mismatched reuse cannot silently
produce a statistic over different seed sets.

## What did not change

- **No threshold.** `QK_BAND` is ±0.03, as registered in §5 and unchanged.
- **No estimator algebra.** The DiD is the same expression over the same four
  arms; only *which wave each arm is fetched from* changed, and it changed to
  what §5 names.
- **No existing call site moved.** `test_every_existing_call_site_keeps_its_baseline`
  pins the dropout DiD to its exact value and passes against the pre-fix
  analyser too, deliberately: it exists to catch a fix that silently re-points
  an arm that was already right.

## The failure direction was the safe one

Before the fix, H27-4 read `NOT EVALUABLE`. The accuracy half was already
**−0.0692**, outside the ±0.03 band, so NOT MET was in fact determinable from
the half that had run. The analyser was over-conservative: it **withheld a
verdict it could have reached** rather than announcing one it could not.

That is the direction this discipline is built to fail in, and it is worth
recording as such — the same code that hid a reachable NOT MET here is the code
that, pointed the other way, would have published an unreachable one.

## Evidence

Two of three new tests fail against the pre-fix analyser and pass against the
amended one; the third passes both, by design:

    ERROR: test_a_group_with_no_rate_arm_pairs_against_the_wave_that_has_one
    FAIL:  test_the_qk_norm_bar_reads_both_halves_once_it_is_paired
    Ran 14 tests — FAILED (failures=1, errors=1)   # pre-fix
    Ran 14 tests — OK                              # amended

## Consequence

H27-4 is decided rather than withheld, and it is decided on **both** halves:
accuracy −0.0692 and DiD −0.0577, each outside ±0.03. qk-norm is a different
read-out. The verdict did not depend on this fix — the accuracy half alone
already established it — but the second half is now on the record instead of
missing from it.
