# Today's source reproduces the campaign binary, on every path tested

**Registered:** `PREREG_2026-08-22_SOURCE_VERSUS_PINNED_BINARY.md`, before the
instance was launched (`6e953ab`).
**Ran:** 2026-08-22, one `c7g.8xlarge` spot instance, us-east-1d, self-terminated
after upload.
**Status:** complete — 9/9 runs exited 0, nothing unfinished, nothing voided.

---

## 1. The environment, which was held fixed

| | |
|---|---|
| instance | `i-0ae4798ff32a1c8a7`, `c7g.8xlarge`, 32 vCPU |
| kernel | `Linux 6.18.41-94.142.amzn2023.aarch64` |
| libc | `ldd (GNU libc) 2.34` |
| rustc | `1.98.0 (88d9e12ae 2026-08-18)` |
| threads per cell | 4, the campaign's `THREADS_PER_CELL` default |
| pinned binary | `22d97c51ab0204702ce44661683ff8c759c29d7f3379e2f6606b048f4f032104` |
| today's binary | `c88a07f840677b0729ec2f1fd162928512ba769d15cd6170a6c41b5a35a9c1d5` |

**The two binaries do not hash the same, and were never expected to.** Different
toolchain, different build path, different build time. That is precisely why the
question had to be asked about behaviour rather than bytes.

Both ran on the one host in the same boot, so the OS, the libm and the CPU were
constant and the binary was the only variable.

## 2. Verdicts

**E-1 — today's build vs the pinned binary, same host. IDENTICAL on all four
cells.**

| cell | arm | shape | epochs | verdict |
|---|---|---|---:|---|
| `w7flr__ff-fixed__…__s5170002` | `ff+fixed` | — | 5 | identical |
| `w7flr__ff-fixed-attn__…__d32l1__s5170012` | `ff+fixed+attn` | d32/L1 | 5 | identical |
| `w7flr__ff-fixed-attn__…__d32l1__s5170009` | `ff+fixed+attn` | d32/L1 | 10 | identical |
| `w8con__ff-fixed-attn__…__d32l4__s5170005` | `ff+fixed+attn` | **d32/L4** | **400** | identical |

**E-2 — the pinned binary vs the archive. IDENTICAL on all four.** The
environment has not drifted: the recorded cells replay today on AL2023 with the
original binary. This was the comparison that could have separated "the source
moved" from "the world moved", and the world did not move.

**E-3 — today's build vs the archive. IDENTICAL on all four.** The end-to-end
claim, following from E-1 and E-2.

**E-4 — thread-count invariance, end to end. IDENTICAL at 1 and 4 threads** on
`…__d32l1__s5170012`. `parallelism_is_bit_identical_to_serial_for_every_arm`
asserts this in-process; it now also holds across whole cells, on the machine
class the record came from.

## 3. What the comparison actually compared

Per cell: **12 scientific fields and 3 per-epoch traces**. For the e400 cell the
traces are 400 entries each, so that single cell contributes 12 scalars and
**1,200 trace values**, including the complete loss, mean-gradient-norm and
max-gradient-norm trajectories.

Comparison is `repr`-level on the serialised values — whether the two runs wrote
the same characters, not whether they round to the same float.

Field *presence* differs and is reported separately rather than as a
disagreement: today's cells carry `clip_sample_grad_norm`, `clipped_samples` and
`seed`, which the pinned binary predates. A whole-file diff would have called
that a failure and buried the question.

### The check can fail

Negative-tested by perturbing the downloaded cells and re-running the analyser:
adding `1e-9` to one accuracy, and `1e-9` to **one entry at epoch 307 of 400** in
one loss trace. Both were caught, named by field and by epoch, and the analyser
exited 1:

```
w7flr__ff-fixed__…  DIFFERS — accuracy: 0.44169611400000003 vs 0.441696113
w8con__ff-fixed-attn__…  DIFFERS — epoch_mean_loss: first differs at epoch 307
                                   (4.1838e-05 vs 4.1837e-05)
```

## 4. What this settles

`HARDENING_2026-08-22_THE_ATTENTION_KERNEL_HAD_NO_GATE.md` §8 listed one
unverified item, and it is now measured:

> ~~Gate F now guards future changes against a macOS reference recorded today.
> It does not establish that today's source reproduces the pinned campaign
> binary `22d97c51ab02`.~~

**Supported:** a binary built from `93f3767` computes what
`22d97c51ab02` computes, on `ff+fixed` and on `ff+fixed+attn` at both d32/L1 and
d32/L4, at budgets from e5 to e400, on aarch64 under glibc 2.34.

Two consequences:

* **A new wave may reuse the archived controls.** The 96 reused control cells
  remain licensed against cells produced by today's source.
* **The attention re-pin changed what the *test* recorded, not what the kernel
  computes.** The old pin constants disagreed with a kernel that reproduces the
  campaign binary exactly — which is the strongest available confirmation that
  the constants were stale and the kernel was right.

## 5. What it does not settle

* **Nothing about macOS.** Gate F FAILs macOS-vs-Linux by design, and no
  comparison here touches a macOS-recorded number.
* **Nothing about `rec+*` or `ff+alif`.** No recorded campaign cell exists on
  those arms to replay; their coverage is the two Gate F reference cells added
  on macOS in `7f908c7`, which is a different claim about a different machine.
* **Not calibration.** The instrument remains `Uncalibrated`.
* Four cells is not the whole record. What is licensed is the *class* of paths
  they exercise, not every cell in the campaign.

## 6. Cost and cleanup

One spot instance, ~48 minutes wall including the build, at ~$0.40/hr — under
$0.40. Instance self-terminated; confirmed `terminated`. The 239 MB source
tarball was deleted from `s3://…/equivalence/`; the cells, plan and logs remain
there and are archived in `results/equivalence_2026-08-22/`. Nothing under
`input/` was written at any point, so the pinned binary and corpus are as they
were.
