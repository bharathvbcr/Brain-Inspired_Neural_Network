# Preregistration — does today's source still compute what the campaign binary computed?

**Registered:** 2026-08-22, **before the instance is launched** and before any
output exists.
**Source under test:** `93f3767`, which includes the attention re-pin in
`7f908c7`.
**Binary under test against:** the campaign's pinned
`22d97c51ab0204702ce44661683ff8c759c29d7f3379e2f6606b048f4f032104`, at
`s3://binn-campaign-511192439661-us-east-1/input/shd-instrument`.

---

## 1. The gap this closes

`HARDENING_2026-08-22_THE_ATTENTION_KERNEL_HAD_NO_GATE.md` §8 leaves one thing
unverified, and it is the one that bears on whether another wave can be run:

> Gate F now guards future changes against a macOS reference recorded today. It
> does not establish that today's source reproduces the pinned campaign binary
> `22d97c51ab02`.

Every cell in waves 1–10 came from that binary, hash-checked on every instance.
Nothing establishes that a binary built from today's tree agrees with it. If it
does not, a new wave is not comparable to the 96 reused controls, and the
attention re-pin — which changed what the kernel is *recorded* to produce —
would be the obvious suspect.

Hashing the two binaries answers nothing: identical source produces different
bytes across toolchains and build paths. What has to be compared is behaviour.

## 2. Design

One `c7g.8xlarge` spot instance, aarch64/AL2023 — the same ISA and libc family
as the fleet that produced the record. Both binaries run **on that one host, in
the same boot**, so the operating system, the libm and the CPU are held fixed
and the only variable is the binary.

Four cells, chosen to span the paths in question and to be cheap. Each is a
*recorded campaign cell*, replayed from its archived plan entry through
`scripts/aws/run_cell.py` — the same script that produced it:

| cell | arm | shape | epochs | recorded wall |
|---|---|---|---:|---:|
| `w7flr__ff-fixed__…__s5170002` | `ff+fixed` | — | 5 | 9 s |
| `w7flr__ff-fixed-attn__…__d32l1__s5170012` | `ff+fixed+attn` | d32/L1 | 5 | 81 s |
| `w7flr__ff-fixed-attn__…__d32l1__s5170009` | `ff+fixed+attn` | d32/L1 | 10 | 154 s |
| `w8con__ff-fixed-attn__…__d32l4__s5170005` | `ff+fixed+attn` | **d32/L4** | **400** | 1390 s |

The first is the control: it exercises the path Gate F already covers, so a
disagreement there would indict the method rather than the attention kernel.
The last is the paper's own depth and budget.

Both binaries run every cell, at `RAYON_NUM_THREADS=4`, the campaign's
`THREADS_PER_CELL` default. Eight cell runs, four threads each, on 32 vCPU.

## 3. Comparisons, and what each one can fail

**E-1 (primary) — source vs pinned, same host.** For each of the four cells,
every scientific field and every per-epoch trace of the cell produced by
today's build must equal the cell produced by the pinned binary, **bit-exactly
as serialised**. This is the registered question, and it is fully controlled:
same host, same boot, same corpus, same initialisation.

**E-2 — pinned vs archive.** The pinned binary's output must equal the archived
cell. This tests the *environment*, not the source: AL2023's glibc has moved
since 2026-08-19, and a difference here would mean the record is not replayable
today even with the original binary. E-2 failing while E-1 passes is a
different, and less alarming, finding than the reverse.

**E-3 — source vs archive.** The end-to-end claim, and it follows from E-1 and
E-2 rather than adding evidence of its own. Reported for completeness.

**E-4 — thread-count invariance, on this host.** One cell
(`…__d32l1__s5170012`) is additionally run by today's build at
`RAYON_NUM_THREADS=1`. Its output must equal the same build's 4-thread output.
`parallelism_is_bit_identical_to_serial_for_every_arm` asserts this in-process;
this checks it end to end, on the machine class the record came from.

## 4. Named outcomes

| outcome | means |
|---|---|
| **E-1 passes on all four** | today's source is behaviourally the campaign binary. A new wave may reuse the archived controls, and the re-pin is confirmed to have changed only what the *test* recorded, not what the kernel computes. |
| **E-1 fails on an attention cell only** | the source and the campaign binary disagree on the attention path. The archived controls may not be reused for any attention comparison, and the divergence must be bisected before another wave runs. |
| **E-1 fails on `ff+fixed` too** | the disagreement is not attention-specific and reaches the 216-cell calibration matrix. Gate F on macOS would then be passing while the Linux path diverges, which is a larger problem than this wave. |
| **E-1 passes, E-2 fails** | the source is fine and the *environment* has moved. Absolute replay of the archive on today's AL2023 is unlicensed; within-fleet comparison is unaffected. |
| **E-4 fails** | thread-count independence does not hold end to end, and every cell in the record whose thread count is unknown becomes suspect. This would supersede §5 of the hardening record. |

## 5. What this cannot show

* Nothing about macOS. Gate F FAILs macOS-vs-Linux by design (Apple libm), and
  no comparison here touches a macOS-recorded number.
* Nothing about `rec+*` or `ff+alif` — no recorded campaign cell exists on those
  arms to replay. Their coverage is the two Gate F reference cells added on
  macOS in `7f908c7`, which is a different claim.
* Not calibration. The instrument stays `Uncalibrated`.

## 6. Stopping rule and cost

Fixed at four cells plus the one thread-count replicate. No cell is added,
dropped or re-run on the basis of its result. A cell that does not finish is
reported unfinished.

One instance, spot, ~$0.40/hr in us-east-1, expected ≈ 45 minutes including the
build — under $0.50. `--instance-initiated-shutdown-behavior terminate` and a
watchdog bound the run; `teardown.py` is the backstop.

Artifacts land under `s3://binn-campaign-511192439661-us-east-1/equivalence/`,
which is a **new prefix**: nothing under `input/` is written, so the pinned
binary, the corpus and the campaign plan are untouched.
