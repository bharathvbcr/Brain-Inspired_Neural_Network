# Amendment — the wave-28 analyser measured against the null arm

**Filed:** 2026-09-06, after 73 of 84 cells landed and **after H28-1 was read**.
Disclosed. H28-2 was not decided at the time of the fix: it had 7 and 6 seed
pairs against a floor of 9 and now reads NOT EVALUABLE.

## What was wrong, and how it was caught

`analyse_wave28.py` indexed cells on `(wave, arm, depth, temporal)` — **without
`attn_readout`**. `w26pos` carries a `no-position` read-out beside the default
one at the same anchor, so both landed on one key. The loop used `setdefault`,
so the first file in sort order won, and `…__d32l4__no-position__s5170001.json`
sorts before `…__d32l4__s5170001.json`.

**Every DiD in this wave was therefore measured against the null arm** — the
positionless read-out wave 26 built as a control — instead of the default one:

| `w26pos` attention intact | mean |
|---|---:|
| what wave 27's analyser reads (default) | 0.8320 |
| what wave 28's analyser read (no-position) | **0.7568** |

0.7568 is wave 26's `intact` rate mean plus its no-position gain
(0.7062 + 0.0506), which identifies the wrong arm exactly.

**It was caught by cross-check, not by inspection.** Both analysers compute a
DiD for the same p30 cells. Wave 27 reported **+0.0015**; wave 28 reported
**−0.0737**. Two frozen analysers disagreeing by 0.075 on one number is not
something either could report about itself, and the disagreement was the only
symptom — the wrong value was plausible, correctly signed for a story, and had
n=12.

## The third instance of one family

- Wave 26: the ladder had no `intact` baseline in its own wave.
- Wave 27: `did()` looked for the rate arms in a wave that held none.
- Here: the index key omitted a dimension the corpus varies.

All three are *an estimator reading arms that are not the ones the registration
names*. The first two were addressed by making waves addressable per arm. This
one is upstream of that: no per-arm addressing helps if two different arms
collapse into one key.

**So the fix is not another parameter.** `attn_readout` joins the key, and
`setdefault` is gone: **a second cell landing on an existing `(key, seed)` is
now fatal**, and the analyser refuses to produce any verdict. A collision means
the key is missing a dimension the corpus varies, and resolving it by sort order
is a wrong number with no symptom. That guard is general — it catches the next
missing dimension without my having to predict which one it is.

## A second defect, found while fixing the first

H28-1 applied the campaign's `MIN_PAIRS` floor of 9; **H28-2 did not**. With 11
attention cells lost to a spot reclaim, H28-2 was deciding its verdict on 7 and
6 seed quadruples. It now reads NOT EVALUABLE below the floor.

This matters beyond tidiness: without it, **a wave that loses cells silently
decides its primary clause on the survivors**, which is the survivorship
statistic §7 of the instrument registration forbids for τ rungs, arriving by a
different route.

## What did not change

- **No threshold.** `BAR` 0.03, `DROPOUT_MIN_COST` 0.03, `MIN_PAIRS` 9 — all as
  registered. `MIN_PAIRS` was already declared and applied to H28-1; it is now
  applied consistently rather than newly introduced.
- **No estimator algebra.** The DiD is the same expression; it now reads the arm
  §4 names.
- **H28-1 is unaffected.** It is a rate-arm comparison and never touched the
  attention key. Its numbers are identical before and after.

## Evidence

Four tests fail against the pre-fix analyser and pass against the amended one:

    FAIL: test_an_index_collision_refuses_rather_than_picking_one
    FAIL: test_the_did_uses_the_default_arm_as_its_baseline
    FAIL: test_the_no_position_arm_is_not_mistaken_for_the_default_arm
    FAIL: test_a_sensitive_rung_short_of_the_floor_does_not_decide_the_clause
    Ran 12 tests — FAILED (failures=4)   # pre-fix
    Ran 12 tests — OK                    # amended

The corrected p30 DiD is **+0.0015**, agreeing with wave 27's analyser on the
same cells to the last digit printed.
