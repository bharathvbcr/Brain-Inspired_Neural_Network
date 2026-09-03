# Amendment — the wave-26 analyser could not say "this did not run"

**Filed:** 2026-09-03, **20 minutes into wave 26**, with 72 of 264 cells landed,
**zero probe files written**, and no hypothesis carrying a single real number.
Every H26-1 statistic was `None` at the moment this was found and fixed. That
timing is the whole integrity argument and it is checkable: the wave's first
`w26sat` probe cannot exist until an h1024/e400 attention cell finishes, whose
archived median is 3.46 h.

**Amends:** `scripts/aws/analyse_wave26.py`, frozen with
[`PREREG_2026-09-03_W26_THE_SATURATION_AND_THE_TIMESCALE.md`](PREREG_2026-09-03_W26_THE_SATURATION_AND_THE_TIMESCALE.md).

## What was wrong

The analyser had two outcomes, `MET` and `NOT MET`, and computed each clause as
a Python boolean. Three different situations collapsed into `False`:

1. the bar was tested and the data fell short — a **refutation**;
2. the inputs were missing — the clause **did not run**;
3. the registered **void rule fired**.

Run against the partial corpus it printed, for H26-2a:

> `NOT EVALUABLE: 0 seed pairs, floor 9`
> `H26-2a ... : NOT MET`
> `removing the positional code does not cost the read-out its advantage. The
> paper's account of WHAT the read-out consumes is wrong`

It detected that it could not evaluate the clause, printed that it could not,
and then published the refutation anyway. H26-1a and H26-1b did the same with
no evaluability line at all.

This is the standing rule in this repository, violated by the tool that exists
to enforce it: *a check that could not run must never report the same result as
a check that ran.* Wave 26 is unusually exposed to it, because its primary
hypothesis is read from **probe files that land last** — only when the longest
cells in the wave finish. A failed probe upload would have produced a confident
printed refutation of the wave's own primary hypothesis, and the result document
would have transcribed it.

Case 3 additionally contradicted the prereg in its own words. §4 registers the
void as *"No verdict is taken from it"*; the analyser took one, and called it
NOT MET.

## What changed

A third state, `NOT EVALUABLE`, carrying the reason it could not run. Applied to
H26-1a, H26-1b, H26-2a, the secondary H26-1c reading, the H26-3 ladder rows, and
the `tau_half` bracket — where `full is None` (not computable) had been sharing a
message with `full <= BAR` (computed, below the bar).

The analyser now exits **3** when any primary clause did not run, so an
incomplete analysis cannot be mistaken for a completed one by a script or by me.

## What did not change, and how that is checked

- **No threshold moved.** `BAR`, `MIN_PAIRS`, `ENTROPY_DROP_MIN`,
  `ENTROPY_DROP_MARGIN`, `QK_GROWTH_MIN`, `QK_GROWTH_CONTROL_MAX`,
  `ENTROPY_ABSOLUTE`, `MONOTONE_TOLERANCE` are byte-identical to the frozen
  file. The only added constant is the string `NOT_EVALUABLE`.
- **No MET predicate changed.** Every comparison that can yield MET is
  unchanged. The sole edit to a predicate is the removal of `not void_h26_1`
  from H26-1a and H26-1b, which now routes through the blocked path — a fired
  void still cannot produce MET, and `test_a_fired_void_rule_takes_no_verdict`
  pins that.
- **NOT MET is still reachable.** `test_a_refutation_is_still_reachable` feeds
  complete probe data that misses the bar and requires NOT MET and exit 0. It
  passes against the pre-fix analyser too, deliberately: it exists so that a
  later attempt to silence refutations fails.

Only the `False` branch was split. Nothing that would have read MET reads
anything else, and nothing that would have read NOT MET on complete data reads
anything else.

## Evidence

Three of the four new tests fail against the pre-fix analyser and pass against
the amended one:

    FAIL: test_a_fired_void_rule_takes_no_verdict
    FAIL: test_a_missing_probe_reads_unevaluated_not_refuted
    FAIL: test_tau_half_separates_an_absent_did_from_one_below_the_bar
    Ran 16 tests — FAILED (failures=3)      # pre-fix
    Ran 16 tests — OK                       # amended

## Consequence for the result document

H26-1a, H26-1b and H26-2a may now be reported as **NOT EVALUABLE**. That is not
a refutation and must not be written as one. If the probes do not arrive, the
wave's primary hypothesis is **unevaluated**, the analyser exits 3, and the
result document says the instrument failed — not that the mechanism claim did.
