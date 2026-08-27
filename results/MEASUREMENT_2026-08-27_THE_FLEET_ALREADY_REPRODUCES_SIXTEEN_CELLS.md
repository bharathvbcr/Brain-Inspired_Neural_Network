# Measurement — wave 18 already reproduces the archive on sixteen cells, at index 68 of 192

**This is a measurement, not a verdict.** It does not evaluate H18-4 and does not
discharge it. H18-4 is registered on twelve named h1024/d32/**L2** cells against
`w15col`, and it is answered by those cells or not at all.

---

## 1. The sequencing problem this is about

[`PREREG_2026-08-27_DEPTH_IS_NOT_MONOTONE_AT_H1024.md`](PREREG_2026-08-27_DEPTH_IS_NOT_MONOTONE_AT_H1024.md)
registers H18-4 and states its consequence plainly: if the twelve duplicated
cells are not byte-identical, **every cell in waves 18 and 19 is void**. It is
the campaign's first registered check on the execution environment rather than
on the code, and it is the right check to have.

The difficulty is where those cells sit. They are plan indices 140–151 of 192.
On 2026-08-27, with the fleet at 68 completed, **54 unclaimed cells were queued
ahead of them** and 26 more were in flight. A reproduction failure would have
been discovered after most of the compute it invalidates had already been spent.

Reordering a live queue is not the fix. Appending to a published queue
mid-campaign is what consumed eighty claims earlier the same day. The fix is to
stop relying on one scheduled pair and check **every** pair the corpus already
contains.

## 2. What the corpus already contained

Six configurations have been run by two different waves. Nobody planned them as
reproduction checks; they are duplicates that fell out of the campaign's own
history, and until now nothing compared them.

| waves | cells | configuration |
|---|---:|---|
| `w1` vs `w3wid` | 12 | `ff+fixed` / h128 / e400 / rate |
| `w1` vs `w3wid` | 12 | `ff+fixed+attn` / h128 / e400 / d32l1 |
| `w2dim` vs `w2lyr` | 12 | `ff+fixed+attn` / h128 / e100 / d32l1 |
| `w16lad` vs `w3wid` | 12 | `ff+fixed` / h256 / e400 / rate |
| **`w18dep` vs `w3wid`** | **4** | `ff+fixed` / h1024 / e400 / rate |
| **`w18dep` vs `w8wid`** | **12** | `ff+fixed+attn` / h1024 / e400 / d32l4 |
| **total** | **64** | |

**All 64 are byte-identical over 90,184 compared values** — every serialised
field including the full per-epoch trajectories, `wall_secs` excluded as a
timing rather than a measurement.

**Sixteen of them are wave 18's own cells**, reproducing two different earlier
waves, at a rung H18-4 does not cover. That evidence existed at 68 of 192 and
nothing was reading it.

## 3. What this licenses

It removes one explanation for a future H18-4 failure. If those twelve L2 cells
come back differing, the cause is not that this fleet has stopped reproducing
the record in general, because at h1024 on both the rate arm and the d32l4
attention arm it demonstrably has not.

**It does not license the converse.** A per-rung environment change — one that
moved L2 and left L4 alone — would pass everything above and still fail H18-4.
That is not a hypothetical hedge: the whole reason wave 18 exists is that h1024
behaves differently by read-out depth.

## 4. What changed

`scripts/aws/check_reproduction.py` runs the sweep, and
[`record_checks.sh`](../scripts/record_checks.sh) runs it on every invocation, so
a reproduction failure now surfaces at the next gate rather than at whichever
plan index a preregistration happened to place it.

It **composes over `cross_isa_reproduction.py`** rather than restating it,
importing that file's `configuration()` and `compare_pair()` so the definitions
of "the same experiment" and of "identical" cannot drift between the two checks.
The axis is what differs: the cross-ISA check asks whether two machines agree,
this asks whether one fleet still agrees with its own record. `load()` there
folds all roots into one map keyed by configuration and seed, so two cells of
one configuration inside one corpus overwrite each other — which is exactly the
pair this looks for, and a test pins that blind spot so the reasoning cannot
quietly become false.

Nineteen tests, negative-tested: folding the wave tag into the configuration key
(so no pair is ever found), returning success on an empty sweep, dropping the
per-epoch trajectories from the comparison, and re-defining the comparator
locally each fail the suite. Restored, all nineteen pass.

**An empty sweep exits 2, not 0.** "Nothing was compared" and "everything
checked out" must not be the same answer — the same conflation that had
`release_dead_claims.py` calling 22 finished failures orphans.

## 5. What this does not say

- **Nothing about H18-4's outcome.** Its cells have not run.
- **Nothing about the thread-count split.** All 64 cells here predate the
  4-thread instances; that question is answered separately in
  [`MEASUREMENT_2026-08-27_THE_FLEET_RAN_TWO_THREAD_COUNTS.md`](MEASUREMENT_2026-08-27_THE_FLEET_RAN_TWO_THREAD_COUNTS.md).
- **Nothing against the macOS-recorded numbers.** Cross-machine Gate F FAILs
  macOS-vs-Linux on every node of this campaign, by design.
