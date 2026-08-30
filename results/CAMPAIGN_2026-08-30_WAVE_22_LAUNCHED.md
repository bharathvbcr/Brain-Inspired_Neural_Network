# Campaign — wave 22 launched

**Launched:** 2026-08-30 05:44 UTC.
**Registered:** [`PREREG_2026-08-29_THE_MECHANISM_AT_EVERY_OPERATING_POINT.md`](PREREG_2026-08-29_THE_MECHANISM_AT_EVERY_OPERATING_POINT.md),
committed in `7fb7a70` and amended in `9d5bc5d` — **both before this launch**,
which is the ordering that carries the epistemic weight and is attested by git
history rather than by mtimes.
**Analyser:** `scripts/aws/analyse_wave22.py`, frozen in the same commits, and
verified to report `0 of 12 points evaluable` against an empty corpus — the
correct pre-launch state.

---

## What is running

| | |
|---|---|
| plan | 504 cells — 288 attention, 216 rate |
| fleet | 2 × `c7g.16xlarge`, `i-03fbc9c8bd557f748` and `i-09e51d77bb6fbebd5` |
| bucket | `binn-campaign-v2-511192439661-us-east-1` |
| estimated | $124 spot; the estimator over-predicts ~3×, so expect $35–45 |

## Why a new bucket

The campaign bucket `binn-campaign-511192439661-us-east-1` holds **765 results**
and pins binary `22d97c51ab02…`, which produced the entire archived corpus.
`scripts/aws/bootstrap.sh` makes a published pin **mandatory**, so running wave
22 on the guarded binary would have meant replacing that pin in place — and a
future re-run of any earlier wave from that bucket would then silently use a
different binary. That is the "one label, two experiments" failure the
matched-architecture hash retirement was about.

A second bucket leaves the old pin untouched. It starts with **no** pin, so the
first instance builds and publishes the guarded binary, and every later instance
downloads exactly that. The corpus was copied server-side from the old bucket,
so `train.events` and `test.events` are byte-identical to what every previous
wave ran against.

## The first launch failed, and the launcher was the reason

The first attempt (`i-01c9dc277aa06dc28`, `i-0a5a0ed4dbdfadccd`) came up unable
to fetch `bootstrap.sh`. S3 answered **403**, the instance wrote the XML error
body to `/tmp/bootstrap.sh`, and bash tried to execute it:

    /tmp/bootstrap.sh: line 1: `<?xml version="1.0" encoding="UTF-8"?>`

cloud-init finished in **7.6 seconds** and two `c7g.16xlarge` sat running and
idle until they were terminated by hand.

**The cause was `launch.py::ensure_role`.** It returned as soon as the instance
profile existed — correct for the profile, wrong for the IAM policy attached to
it. The policy names a bucket, and it went on naming
`binn-campaign-511192439661-us-east-1`, the bucket it was first created for.
Confirmed against the live policy before changing anything. So the deliberate
choice to use a second bucket was exactly what tripped it.

Nothing in the launch output said anything was wrong. It printed
`instance profile binn-campaign-worker exists` and then `2 instance(s)`, and
both were true. **That is the failure mode this repository exists to hunt: a
step that could not do its job reporting the same thing as one that did.**

Fixed — `put-role-policy` overwrites by name and is idempotent, so it is applied
on every launch and the policy stays scoped to exactly one bucket. Three tests
in `test_campaign_tooling.py::WorkerRolePolicyTest` pin it, negative-tested by
restoring the early return. The relaunch printed the line that had been missing:
`role policy re-scoped to binn-campaign-v2-...`.

## The binary this wave runs

Built from the source at `a5e671f`, which carries the forward-finiteness guard
([`DEFECT_2026-08-29_THE_EVALUATION_FORWARD_WAS_NEVER_CHECKED.md`](DEFECT_2026-08-29_THE_EVALUATION_FORWARD_WAS_NEVER_CHECKED.md)).
**Every cell of this wave is checked for a non-finite evaluation forward**, which
no archived cell was — and three of the twelve operating points are at h1024,
the configuration with peak gradient norms of 4.9e32.

Gate F passed on that source before launch: **21 distinct cells, bit-identical,
zero failures**, across six contracts, two widths and three arms including
`ff+fixed+attn` and `ff+alif`. That is measured on macOS/aarch64; the
Linux/Graviton build's equivalence to `22d97c51ab02…` is **inferred, not
measured**, and no verdict in this wave rests on it — the wave is self-contained
precisely so it does not have to.

## Boot outcome

Booted 06:10Z on the relaunch. The guarded binary built and published on
Graviton as **`3afd4434431a75a26cc9d5fa46831341fc2f1dd0ef08dc308e18ca139b576364`**,
and `collect.py` confirms **one binary across the campaign** — the pin is doing
its job.

**Cross-machine Gate F: FAIL on both instances. This is the documented
expectation, not a fault.** `bootstrap.sh` says why: the recorded cells were
produced on macOS/aarch64, `exp`/`sin`/`cos`/`powf`/`ln` come from libm, and
glibc's are not obliged to agree with Apple's to the last ulp — one ulp flips a
spike and compounds through Adam. `VENUE_FORMATTING.md` carries it as a required
main-text disclosure: *"Cross-machine Gate F FAILs macOS-vs-Linux by design. No
claim rests on a comparison against a macOS-recorded number."*

It is recorded either way, because "the check could not run" and "the check ran
and passed" must never look the same downstream. **No verdict in this wave rests
on it**: every arm of every contrast here runs on this fleet, on this binary,
beside its own control — which is what "self-contained" was for.

Note this is a *different* question from the local Gate F that passed before
launch. That one asked whether the guard changed the computation **on one
machine**, and answered no across 21 cells. This one asks whether two machines
agree, and has always answered no.

## Monitoring

```bash
python3 scripts/aws/watch_campaign.py --bucket binn-campaign-v2-511192439661-us-east-1
```

```bash
python3 scripts/aws/collect.py --bucket binn-campaign-v2-511192439661-us-east-1
```

Teardown, which must be run once the queue drains — spot instances do not stop
themselves if the drain logic is not reached:

```bash
python3 scripts/aws/teardown.py --bucket binn-campaign-v2-511192439661-us-east-1
```

## What must not happen to this wave

1. **No point is re-run to improve its verdict**, and no seed is added beyond
   twelve. The stopping rule is 504 cells, once.
2. **No verdict is transcribed except from the frozen analyser.**
   `scripts/check_verdicts_transcribed.py` cross-checks whatever is written up.
3. **The archived intact halves are not used for any verdict.** They may be
   compared against the new ones descriptively, as a reproduction observation
   that crosses two binaries by construction and carries no hypothesis.
4. A point whose cells fail `scripts/cell_validity.py` is **reported with its
   exclusion count, not topped up**.

## Amendment 2026-08-30 15:24 UTC — a scale-up that the spot quota refused

**Nothing about the experiment changed. No cell was added, withdrawn, reordered
or re-run.** This section exists because the audit trail contains a `plan
uploaded` line and a failed `run-instances` at this hour, and the next person to
read it should not have to reconstruct why.

### Why it was attempted

At 15:10 UTC the queue read **288/576**. That is 50% of the *cells* and much
less than 50% of the *compute*: every one of wave 22's 216 completed cells is a
**rate arm**, and not one attention arm had finished. Attention read-out
dominates wall time, so the cheap half ran first and the whole expensive half
was still ahead. **That much is directly observed and stands.**

An accompanying estimate — a remainder of ~1,560 slot-hours, ≈24 h at 64 slots,
≈$95 for the campaign — was also produced, and **it is withdrawn**. See below.
Doubling the fleet would have roughly halved whatever the true wall clock is, at
roughly the same dollar cost, since the work is fixed and spot bills per
instance-hour. That argument is a ratio and does not depend on the withdrawn
figure.

### Why it failed

`MaxSpotInstanceCountExceeded`. The account's spot quota is **300 vCPU** and
four `c7g.16xlarge` already hold **256**. A fifth needs 64 and only 44 were
free. The quota (`L-34B43A08`) spans **all** standard families A, C, D, H, I, M,
R, T and Z, so no instance family escapes it. A request to raise it to 512 has
been `CASE_OPENED` since **2026-08-19** and had not moved in eleven days.

**Decision: let it ride at four instances.** The 44 vCPU of headroom buys
**+10 slots, or +15.6% throughput**, which does not justify a mixed instance type
in the fleet. This is a ratio, so it survives the withdrawn estimate below: the
headroom removes 13.5% of the remaining wall clock whatever that remainder
actually is.

Re-slotting the existing boxes was also considered and rejected on the reasoning
already recorded at `scripts/aws/launch.py:205`: 2 threads x 32 cells throughputs
59.1 against 54.1 for the current 4 x 16, but pushes the slowest cell from ~14 h
to ~26 h, past the 16-hour reclaim threshold, so under spot interruption it is a
cell that may never finish. It would also have killed the 64 cells in flight.

### The republish was verified to be a no-op *before* it was made

`upload_plan` publishes the queue on **every** launch, deliberately — the
comment at `scripts/aws/launch.py:246` explains that "which cells run" is never
a thing an upload optimisation may decide. Under a union queue that behaviour is
also the hazard this record's rule 1 names: a single-wave plan here withdraws the
other wave's cells from under a running fleet.

So the plan was **regenerated from source** rather than taken from S3 —
`plan_cells.py --waves w22,w23 --priority w23` — and checked three ways against
the live published queue: same 576 ids, **same order** (order is the schedule,
since `claim_next.py` takes the first unclaimed cell in plan order), and
**byte-identical**. It published with no `REPLACING` line, which is the
observable proof it changed nothing.

Two further preconditions were checked, because a new instance that differs from
the running four is a second experiment wearing one label:

- `scripts/aws/bootstrap.sh` on disk is **identical** to the published copy that
  `--skip-inputs` would have re-uploaded.
- The campaign binary is pinned: `bootstrap.sh` downloads `input/binary.sha256`
  and **aborts on a hash mismatch**, so a new box could only ever have joined on
  `3afd4434431a` or died loudly.

`run-instances` failed before provisioning. **No instance was created and no
spend was incurred.**

### Withdrawn 17:59 UTC — the remaining-time estimate rested on a bad measure

The ~24 h / ~$95 figure above was built from archived `wall_secs` compared
**across waves**. That is not a valid measure of what a configuration costs, and
the archived corpus refutes it on its own terms. Split by contract and geometry
rather than collapsed, the e400 attention cells read:

| hidden | contract | read-out | n | median h |
|---:|---|---|---:|---:|
| 1024 | `published-2ms` | **`d32l1`** | 32 | **5.20** |
| 1024 | `published-2ms` | `d32l2` | 32 | 2.37 |
| 1024 | `published-2ms` | `d32l3` | 20 | 3.04 |
| 1024 | `published-2ms` | **`d32l4`** | 80 | **3.39** |

**A one-layer read-out cannot cost more than a four-layer one.** The ordering is
impossible as a cost model, so these numbers are dominated by the fleet
conditions of the wave that produced them — instance count, concurrent cells per
box, threads per cell — and not by the configuration named in the row. Using
them as configuration costs was the error.

Two consequences, both of which push the true remainder **up**:

1. **Contracts were conflated.** The `h128/d32l4` cells in flight are
   `fixed-t500`; the archive puts that at **5.06 h** against **3.34 h** for
   `published-2ms`. The 5.76 h figure quoted above came from a single 12-cell
   wave and is not the right baseline for either.
2. **This fleet is slower than the archive.** It is the one measurement that is
   safe to make, because it is within-fleet: wave 23 ran `h1024/d32l4` at e200
   in **6.39 h**, and epochs scale linearly here to within 2%, so e400 is
   **~12.8 h**. The archive puts the same configuration at e400 at **3.39 h** —
   roughly **3.8x faster**. The 16-cells-x-4-threads slotting is the difference.

So ~24 h is a **floor**, not an estimate, and the ~$95 follows it upward. No
replacement number is recorded here, because the only sound way to get one is a
within-fleet measurement of an attention arm at e400 and **no such cell has
finished yet** — 288/576 has not moved since 13:40 UTC, which at 17:59 is the
longest gap of the campaign and is fully explained by the in-flight attention
arms having been claimed between 10:20 and 13:40.

**The first attention completion measures this exactly.** The watcher on this
campaign reports the first six completions with their `wall_secs` for that
reason.

**Nothing scientific depends on any of this.** Wall time is not a reported
quantity, no verdict reads it, and the withdrawn figure never reached
`PAPER_DRAFT.md` or any result document. What it did affect is an operational
decision — whether to add capacity — and that decision was a ratio and is
unchanged.

### One defect in the watcher, found and fixed at 17:59 UTC

The first watcher anchored its stall clock to **its own start time**. It
restarted at 16:55 when the session resumed, which silently forgave the 4.3 h
gap already in progress: a queue that had been stuck for hours would have read
as freshly quiet. That is the failure mode where **silence is mistaken for
success**, and it defeats the purpose of watching.

The replacement anchors the stall clock to the newest object already in
`results/`, so restarting it cannot launder a stuck queue. Its startup line
prints the real gap as proof.

### 19:48 UTC — a replacement estimate, and why no cell can be cut to save it

The watcher's stall check fired at **6.1 h with no completion**. Investigated,
and **the fleet is healthy**: all four instances at **93–96% CPU**, hostlogs
being written every minute. What the hostlogs also show is that they are
**byte-identical to the 15:24 snapshot** — 22,922 bytes, 318 lines, unchanged —
so not one slot has finished a cell and claimed another in over four hours. The
boxes are computing hard on cells that are simply long. This is the slow-fleet
finding above, not a hang.

That supplies the within-fleet anchor the previous section said was missing.
Wave 23 ran `h1024/d32l4` at e200 in 6.39 h, so **12.78 h** at e400 against the
archive's **3.39 h** for the same configuration: a measured **3.77x**. Applying
it to the archived median of every remaining configuration — **all 288 have
one** — gives **3,332 slot-hours**:

| remaining | n | archive h | this fleet h | slot-h |
|---|---:|---:|---:|---:|
| `h128` `published-2ms` `d64/L4` | 24 | 7.66 | 28.9 | **693** |
| `h1024` `published-2ms` `d32/L1` | 24 | 5.20 | 19.6 | **470** |
| `h128` `fixed-t500` `d32/L4` | 24 | 5.06 | 19.1 | **457** |
| `h512` `published-2ms` `d32/L1` | 24 | 3.99 | 15.0 | **361** |
| *(eight further points)* | 168 | | | 1,351 |

**≈52 h at 64 slots — about 2.2 days — and ≈$150 more, ≈$188 for the campaign.**

Treat that as an upper end. It inherits the archive's own contamination: the
`d32/L1` rows rest on the impossible 5.20 h figure that discredited the earlier
estimate, and a one-layer read-out cannot really cost more than a two-layer one.
Holding L1 at or below its L2 twin gives **≈45 h and ≈$167 total**. The
defensible statement is **45–52 h and $167–188**, against $40–50 at launch.

**No cell can be cut to save this**, which is the operative point:

- Dropping `d64/L4` — the single largest item at 693 slot-hours — **destroys
  H22-3**, which is defined over exactly "`d64/L4` against their `d32/L4` twins".
- Dropping `fixed-t500` **destroys H22-4**, which requires all three `fixed-tN`
  rungs.
- Dropping **anything** fails **H22-2**, whose whole content is that coverage
  reaches 21 of 21, and which "can fail only by cells failing to land".

And abandoning the wave is worse than it looks: the **216 rate arms already
paid for are the shuffle controls**. They have no standalone value. Stopping now
discards them along with the money already spent.

**So the wave runs to completion.** The overrun is recorded here because the
launch estimate was wrong by ~4x and the next campaign should size from
within-fleet measurements, never from archived `wall_secs` across waves.
