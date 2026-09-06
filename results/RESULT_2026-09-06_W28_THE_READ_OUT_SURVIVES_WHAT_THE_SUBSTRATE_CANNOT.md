# Wave 28 — the read-out survives what the substrate cannot

**Registered:** [`PREREG_2026-09-05_W28_THE_RATE_AT_WHICH_DROPOUT_BITES.md`](PREREG_2026-09-05_W28_THE_RATE_AT_WHICH_DROPOUT_BITES.md),
committed before the first cell.
**Analyser:** `scripts/aws/analyse_wave28.py`, the authority on every verdict
below. One amendment to it is disclosed in §5, and it is consequential.
**Corpus:** 84/84 cells, **0 failures**, **0 INVALID** — after a recovery run,
disclosed in §6. The v3 corpus holds 612 cells across waves 26–28, all valid.
**Binary:** `434d38c2904e9ac5f75429fc1b2ef0a32ae17b26a9319bbf17c2aab14f0b160a`,
unchanged since wave 26 and verified at launch.

> **Reproduction gate, disclosed.** Cross-machine Gate F **FAILED on every
> instance**, as on every fleet in this campaign: divergence exactly **0.0049**,
> characterised in
> [`FINDING_2026-08-22_REPRODUCIBLE_ACROSS_ISA_UNDER_GLIBC.md`](FINDING_2026-08-22_REPRODUCIBLE_ACROSS_ISA_UNDER_GLIBC.md).

## 1. H27-2 is closed: dropout becomes sensitive at p70

**H28-1: MET.**

| p | 30 | 50 | 60 | **70** | **80** | **90** |
|---|---:|---:|---:|---:|---:|---:|
| rate-arm drop from intact | 0.0124 | 0.0219 | 0.0231 | **0.0388** | **0.0572** | **0.1080** |
| above the 0.03 bar | 0/12 | 1/12 | 2/12 | **11/12** | **12/12** | **12/12** |

**The smallest sensitive rate is p70.** Wave 27's H27-2 failure was a rate
problem, not a broken operator: the same instrument that could not feel its own
hand at p30 clears the bar comfortably from p70 up, and the ladder is orderly —
0.0124, 0.0219, 0.0231, 0.0388, 0.0572, 0.1080 — with the crossing between p60
and p70.

A local single-seed pilot predicted 0.0384 at p70 before the wave was planned;
twelve seeds give **0.0388**. The pilot is disclosed in §1 of the registration
and no verdict is taken from it.

**Nothing was destroyed to achieve this.** Not one cell voided at any rung. Even
at p90, with 90% of spikes deleted, cells predict all 20 classes and sit at
0.5982 against a chance level of 0.05. The band §3 of the registration required
— sensitive without destroying the substrate — exists and the ladder is inside it.

## 2. H28-2 — NOT MET, and the registered reading is the opposite of what happened

**H28-2: NOT MET.**

| | rate | attention | gain |
|---|---:|---:|---:|
| intact | 0.7062 | 0.8320 | 0.1258 |
| p30 (not sensitive) | 0.6938 | 0.8181 | 0.1243 |
| **p70** | 0.6674 (−0.0388) | 0.8064 (−0.0256) | **0.1390** |
| **p90** | 0.5982 (−0.1080) | 0.7622 (−0.0698) | **0.1640** |

DiD: p70 **−0.0132** (2/12 outside the band), p90 **−0.0382** (9/12 outside).

### The registered reading is not used, because the sign refutes it

§4 registered, for NOT MET: *"destroying counts DOES cost the read-out its
advantage."* **That is not what happened, and writing it would be false.** The
DiD is **negative at both sensitive rates**: the attention arm loses *less* than
the rate arm, and the read-out's advantage **grows** from 0.1258 to 0.1390 at
p70 and to **0.1640** at p90.

At p90 the substrate loses 0.1080 of accuracy and the read-out loses 0.0698. The
read-out is **more robust to spike deletion than the substrate it reads from**.

### The bar was two-sided, and that was a registration defect

The clause failed only because |−0.0382| exceeds 0.03 — that is, because the
advantage grew *too much*. A two-sided band was registered for a one-sided
question: §4 was written to detect *the read-out losing its advantage*, and it
fires identically when the advantage strengthens.

§9.3 of the instrument registration governs this exactly: *"A bar that turns out
to be wrong is reported as a wrong bar with the result it produced, and a new
registration changes it for the next wave."* So: **the bar was wrong, the
verdict it produced is NOT MET, and the verdict stands as recorded.** It is not
reinterpreted into a pass, and the threshold is not moved after the fact.

### What the data supports, stated on its own terms

A count-destroying manipulation **demonstrated sensitive by H28-1** does not
reduce the read-out's advantage at any rate tested; it increases it. Destroying
*order* costs the read-out **0.1208**. Destroying *counts* costs it **nothing**,
and buys it 0.0382.

That is the asymmetry §2 of the instrument registration existed to expose, and
it now rests on an instrument proven able to see rather than on a null of
unknown meaning. **But H28-2 as registered did not test it**, so this is
reported as the substance of a mis-specified clause, not as a MET verdict.
Turning it into a claim requires a new registration with a one-sided bar, and
this document does not grant one.

## 3. What that means for the paper's mechanism claim

Read with wave 27's τ½ = 261 ms and wave 26's structural null, the read-out's
advantage now has three measured properties, each from a separate manipulation:

- it **runs through position** — removing the positional code costs 0.0752, 12/12 (H26-2a);
- it **has a timescale** — half the effect below ~261 ms (H27-3);
- it **does not run through spike counts** — deleting 90% of spikes costs it nothing and gains it 0.0382 (here).

The third was the one the campaign could not previously assert, because every
other operator preserves counts and the only one that did not was insensitive.

## 4. Coverage

84/84 landed, 0 failures, 0 INVALID, no rung voided. 60 rate cells at
p{50,60,70,80,90} and 24 attention cells at p{70,90}; p30 is wave 27's rung and
`w26pos` supplies `intact`, neither re-run — same binary, so §9.1 is satisfied
by pairing.

## 5. One amendment, and it changed a number

[`AMENDMENT_2026-09-06_THE_BASELINE_WAS_THE_NULL_ARM.md`](AMENDMENT_2026-09-06_THE_BASELINE_WAS_THE_NULL_ARM.md)
— the index key omitted `attn_readout`. `w26pos` carries a `no-position`
read-out beside the default one; both collapsed onto one key and `setdefault`
kept whichever sorted first, which is `no-position`. **Every DiD in the wave was
being measured against the positionless null arm** (0.7568) instead of the
default one (0.8320).

**It was caught by cross-check, not inspection.** Wave 27 and wave 28 both
compute a DiD for the same p30 cells; they disagreed by 0.075. The wrong value
was plausible, cleanly signed and had n=12 — the disagreement was its only
symptom.

This is the **third instance of one family**: wave 26's ladder had no `intact`
baseline in its own wave; wave 27's `did()` looked for rate arms in a wave
holding none. Both earlier fixes made waves addressable per arm, which does not
help when two arms collapse into one key. So `setdefault` is gone and a second
cell landing on an existing `(key, seed)` is now **fatal** — a general guard
that catches the next missing dimension without predicting which it is.

A second defect surfaced while fixing the first: H28-1 applied the `MIN_PAIRS`
floor of 9 and **H28-2 did not**, so with 11 cells lost it was briefly deciding
its verdict on 7 and 6 quadruples. Both clauses now apply it. Without that, a
wave that loses cells silently decides its primary clause on the survivors,
which is the survivorship statistic §7 forbids for τ rungs arriving by another
route.

No threshold and no estimator algebra changed. **H28-1 is unaffected** — it is a
rate-arm comparison that never touched the attention key, and its numbers are
identical before and after.

## 6. A spot reclaim, and the claim leak it exposed

The wave first stopped at **73/84**. One of the two instances was reclaimed by
AWS (`instance-terminated-no-capacity`) at 00:40 UTC; the other finished its
queue and self-terminated. The survivor could not pick up the reclaimed work,
because **a claimed cell is never re-issued** — so 11 attention cells stranded
behind claims held by a machine that no longer existed.

They were recovered by releasing the orphaned claims and relaunching on one
instance. Releasing was safe to do below the 16-hour default guard for a
checkable reason: **zero instances were running**, so no claim could still be in
progress. The guard exists because that is normally unknowable; here it was
knowable and was checked before acting.

**This is a standing fragility, not a one-off.** Any spot reclaim strands its
in-flight cells until a human releases them, and the smaller the fleet the
larger the fraction lost. It cost this wave a second launch.

## 7. Cost

| | registered | actual |
|---|---:|---:|
| slot-hours | 268 | **261** |
| instance-hours | ~22 (2 × 11 h) | **31.7** |
| **spot cost** | $15 – 18 | **$22 – 23** |

Slot-hours came in 2.6% under. The cost overrun is entirely the recovery run:
the reclaim cost about 10 instance-hours of lost work and a third launch. Sizing
the fleet to the wave was still right — six instances would have idled 64 slots
— but a two-instance fleet has no spare capacity to absorb a reclaim, and that
trade is now measured rather than assumed.

## 8. What this wave does not answer

- **One deletion model.** Uniform random spike deletion. Nothing about
  structured, per-channel, or burst deletion, or about jitter.
- **Why the read-out is more robust than the substrate.** The mechanism behind
  the +0.0382 is unmeasured, and it is the obvious next question.
- **One anchor, feed-forward, h128.** No claim transfers to h1024 or recurrent.
- **The attention arm was measured at two rates**, not across the ladder.
- **The gain/DiD dissociation** remains the campaign's leading open problem and
  still has no design.
- **Nothing here is bit-reproducible off this fleet.**
