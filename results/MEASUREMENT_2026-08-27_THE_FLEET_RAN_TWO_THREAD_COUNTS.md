# Measurement — the fleet ran two thread counts, and the cells did not notice

**This is a measurement, not a verdict.** No hypothesis is evaluated and no
wave's status changes. It records an execution-environment split that I
introduced, and the evidence that it did not reach the numbers.

---

## 1. What happened

Scaling the fleet into the account's spare ceiling, I launched two instances
with `launch.py`'s default `--threads-per-cell 4`. The four instances already
running had been launched at **16**. Nothing said so: the thread count appears
only in a `bootstrap.sh` banner line inside a hostlog, which is not read during
a launch and is not part of a cell's provenance.

| instances | type | layout | launched (UTC) |
|---:|---|---|---|
| 4 | `c7g.16xlarge` | 4 cells × **16** threads | 09:06 |
| 1 | `c7g.8xlarge` | 8 cells × **4** threads | 14:58 |
| 1 | `c7g.2xlarge` | 2 cells × **4** threads | 14:58 |

**Why it mattered enough to check.** `RAYON_NUM_THREADS` is passed straight to
the instrument. If any reduction in the training loop were parallel and
order-dependent, thread count would move the low bits, and the first thing to
break would be **H18-4** — the registered reproduction check that compares wave
18's h1024/d32/L2 cells against `w15col` byte for byte, and whose failure voids
every cell in waves 18 and 19. A self-inflicted void is the worst way to lose a
campaign.

## 2. The measurement

Every instance runs Gate F at boot and publishes the log. The log prints, to
nine decimals, six scientific fields for each of three cells — **18 numbers per
instance**, produced on that instance, at that thread count.

Comparing a 16-thread instance against a 4-thread instance, the logs differ on
**three lines, all of them wall-clock**:

```
< [ FAIL] rust__fixed-t100__adjacent-sum-5__h128__e20__s5170003  (11s)
> [ FAIL] rust__fixed-t100__adjacent-sum-5__h128__e20__s5170003  (18s)
```

Ignoring the timing column, the two logs are **identical**, and so are the two
4-thread instances against each other. All 18 observed values agree to the last
recorded digit, across two instance sizes and a 4× difference in thread count.

The rest of the environment is held, not assumed: all six instances report the
same AMI `ami-0cded71ff6ab7f608`, the same kernel
`6.18.41-94.142.amzn2023.aarch64`, and the same pinned binary
`22d97c51…f4f032104`.

**This corroborates a claim the record already carried rather than resting on
it.** `HARDENING_2026-08-21_CEILING_HEALTH_HAS_ONE_OWNER.md` reports 3/3
bit-identical at `RAYON_NUM_THREADS` ∈ {1, 3, 8, 16} — measured on macOS, against
recorded values. The evidence above is Linux/Graviton, on the campaign's own
fleet, with the binary that produced the cells.

**Gate F's verdict is FAIL on all six.** That is the macOS-vs-Linux libm
divergence the campaign expects and designs around, and it is unrelated to
thread count: the *observed* columns match across instances while differing from
the *recorded* macOS values in the last few digits.

## 3. The cost that is real

Thread count does not move the numbers. It moves the clock, and `launch.py`'s
own measured table says how: speedup 3.38× at 4 threads against 7.90× at 16. A
cell on a 4-thread instance therefore takes about **2.3× as long** — a wave-20
attention cell measured at ~6.75 h becomes ~15.8 h — while total throughput per
vCPU rises by about 1.7×.

That trade is the one `launch.py` documents, and it was made here by default
rather than on purpose. It is left standing: throughput is what the maintainer
asked for, and the exposure it buys is **recoverable**. A spot reclaim consumes
a claim, but `release_dead_claims.py` returns the cell to the queue. The four
`rec+alif` divergences of the same day are not recoverable, and that asymmetry
is why a longer cell is an acceptable trade and a diverging one is not.

## 4. What changed

- **`bootstrap.sh` records the environment it runs in.** The gate JSON now
  carries `threads_per_cell`, `concurrent_cells` and `instance_type` beside the
  binary hash and `uname`. A reproduction check that fails should not have to
  guess at the environment it is comparing.
- **`launch.py` names a split before it creates one.** Scaling into a fleet on a
  different thread count prints what the fleet runs, what this launch requests,
  and which way the trade goes. It does not refuse: the split is legitimate, it
  just has to be deliberate.
- **Five tests in `test_campaign_tooling.py`**, pinning both directions — a
  matching fleet stays silent, a split fleet is named, an unreadable gate reads
  as *unrecorded* rather than as agreement, and the bootstrap still writes the
  field the notice reads. Each was negative-tested: dropping the match guard,
  silencing the notice, and removing the field from `bootstrap.sh` each fail the
  suite, and the restored code passes.

The six instances now running booted before the field existed, so their gate
JSONs report `threads_per_cell` as absent. The notice prints that as **"an
unrecorded count"**, which is the honest reading and not the same as a match.

## 5. What this does not say

- **It does not clear H18-4.** The check has not run: wave 18's h1024/d32/L2
  arm sits at plan indices 140–159 and no cell of it has landed. This removes
  one candidate explanation for a failure in advance; it does not predict the
  outcome.
- **It says nothing about thread counts other than 4 and 16** on this fleet.
  {1, 3, 8} are corroborated only by the macOS record.
- **Nothing against the macOS-recorded numbers.** Cross-machine Gate F FAILs
  macOS-vs-Linux on every node of this campaign, by design.
