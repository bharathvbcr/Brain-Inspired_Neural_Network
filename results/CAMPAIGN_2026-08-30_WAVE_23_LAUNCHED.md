# Campaign — wave 23 launched into wave 22's queue

**Launched:** 2026-08-30 07:15 UTC, while wave 22 was running.
**Registered:** [`PREREG_2026-08-29_THE_COLLAPSE_IS_LATE.md`](PREREG_2026-08-29_THE_COLLAPSE_IS_LATE.md),
committed in `7fb7a70` — before any cell of either wave existed.
**Analyser:** `scripts/aws/analyse_wave23.py`, frozen in the same commit, and
verified to report `NOT EVALUABLE` against an empty corpus before launch.

---

## Why it went into the same bucket and the same queue

Wave 22 was already consuming `input/cells.json`, and
`scripts/aws/claim_next.py` **re-reads the published queue on every claim** —
not the copy `bootstrap.sh` downloaded at boot. That is deliberate, and the
comment there records why: on 2026-08-26 a reordered republish was a silent
no-op against the running fleet, and "a queue change that appears to succeed and
does not is worse than one that is refused."

The consequence for this launch is direct: **publishing a wave-23-only plan
would have stopped the running instances claiming wave-22 cells**, stranding
roughly 489 of them mid-campaign along with the money already spent on the
fleet.

So the published queue is the **union**: 576 cells, `72 new, 0 withdrawn`, which
`upload_plan` printed and which was verified beforehand — every wave-22 id in
the old queue is present in the new one.

**Same bucket is also the better provenance.** Both waves now run on one binary,
`3afd4434431a75a2…`, the guarded build. A separate bucket for wave 23 would have
built a third binary and put wave 23 in its own provenance class for no gain.

## Ordering: wave 23 first

Plan order **is** the schedule — `claim_next.py` takes the first unclaimed cell
in plan order — so the queue was generated with `--priority w23`.

Wave 23 is 72 cells against wave 22's 504, and it answers the paper's registered
**leading open problem**. Putting it first costs wave 22 a few hours of delay and
finishes wave 23 in a fraction of the time. Total work is unchanged; only the
order is.

## Capacity

Two further `c7g.16xlarge` were added — `i-0cb911fc54e3f0ff6`,
`i-0866070ce833c6ad4` — bringing the fleet to **four**. Spot is billed by
instance-hour and the total work is fixed, so this is close to cost-neutral and
roughly halves wall-clock. Estimated total for both waves: **$137 spot**, and the
estimator over-predicts ~3×, so expect **$40–50**.

## What must not happen

1. **Do not publish a single-wave plan into this bucket while either wave is
   running.** It withdraws the other wave's cells from the queue. Publish the
   union or nothing.
2. Wave 23's stopping rule is **72 cells, once**. No budget between e100 and
   e400 is added after seeing these, and no third budget is tried to find one
   that works — the same rule the matched-architecture programme uses to forbid
   searching for a result.
3. **H23-3 is the control and can make H23-1 and H23-2 uninformative without
   refuting them.** If `d32l2` also improves when truncated, e400 is simply past
   the optimum for deep read-outs at h1024 and nothing specific to the collapse
   has been shown. The analyser prints that conclusion rather than leaving it to
   a reader.
4. Verdicts come from the frozen analyser only;
   `scripts/check_verdicts_transcribed.py` cross-checks the write-up.

## Monitoring and teardown

```bash
python3 scripts/aws/collect.py --bucket binn-campaign-v2-511192439661-us-east-1
```

Teardown once the queue drains — **both waves must be complete first**, since
they share a fleet:

```bash
python3 scripts/aws/teardown.py --bucket binn-campaign-v2-511192439661-us-east-1
```
