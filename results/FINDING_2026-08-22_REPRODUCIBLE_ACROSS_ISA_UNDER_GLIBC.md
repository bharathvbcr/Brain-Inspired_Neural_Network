# The instrument reproduces exactly across CPU architectures — the divergence was libm, not the ISA

**Found:** 2026-08-22, from the truncated Azure campaign.
**Refines:** `MEASUREMENT_2026-08-19_CROSS_MACHINE_BIT_EXACTNESS.md`, which recorded
that cross-machine Gate F **FAILs** with accuracy divergence up to 0.0049 and
concluded that results are not portable across machines. That measurement stands.
Its **cause** was mis-attributed.

---

## 1. The measurement

The Azure campaign re-ran three configurations that the AWS campaign had already
run, on entirely different hardware and a separately compiled binary:

| | AWS | Azure |
|---|---|---|
| CPU | Graviton3, **aarch64** | EPYC 9005, **x86-64** (built `x86-64-v4`, AVX-512 available) |
| binary sha256 | `22d97c51ab02…` | `666a73420a63…` |
| OS | Amazon Linux 2023 (**glibc**) | Linux (**glibc**) |

Comparing every serialized floating-point value in the overlapping cells,
**excluding `wall_secs`**:

| configuration | cell pairs | float values | **differing** |
|---|---:|---:|---:|
| h1024 / d32-L4 / e400 | 12 | 19,320 | **0** |
| h1024 / rate-only / e400 | 12 | 19,320 | **0** |
| h512 / d32-L4 / e400 | 12 | 19,320 | **0** |
| **total** | **36** | **57,960** | **0** |

That includes the **complete 400-epoch training trajectories** — `epoch_mean_loss`,
`epoch_max_gradient_norm`, `epoch_mean_gradient_norm` — not merely end-point
summaries.

> ### AMENDED 2026-08-25 — the overlap is eight configurations, not three
>
> The three rows above are the overlap I **went looking for**, and I found the
> expensive width arms because those were the ones I had in mind. Deriving the
> overlap instead of choosing it — key every Azure cell by its scientific
> configuration, key every AWS cell the same way, compare every pair that
> matches on configuration *and* seed — gives **79 cell pairs across eight
> configurations, 122,713 values, 0 differing**:
>
> | configuration | cell pairs | values | differing |
> |---|---:|---:|---:|
> | h128 / d32-L4 / `fixed-t250` / e400 | 12 | 19,620 | **0** |
> | h128 / d32-L4 / `fixed-t500` / e400 | 9 | 14,715 | **0** |
> | h128 / d32-L4 / `published-2ms` / **e200** | 8 | 6,680 | **0** |
> | h512 / d32-L4 / e400 | 12 | 19,620 | **0** |
> | h1024 / d32-L4 / e400 | 12 | 19,620 | **0** |
> | h256 / rate-only / e400 | 4 | 6,532 | **0** |
> | h512 / rate-only / e400 | 10 | 16,330 | **0** |
> | h1024 / rate-only / e400 | 12 | 19,596 | **0** |
> | **total** | **79** | **122,713** | **0** |
>
> The count is larger than 57,960 for two reasons that must not be confused. It
> covers **43 more cell pairs**, and it counts **every serialised leaf** rather
> than ten scalars plus four trajectories, so even the original 36 pairs
> contribute more values here. Same agreement, more of it, measured with a
> wider denominator.
>
> **What the four missed configurations add is not volume.** Every row in the
> original table is h512 or h1024 — widths at which the read-out's gain is
> reduced or inverted, and none of them the configuration the paper leads with.
> Three of the new rows are at **h128 / d32-L4**, the headline width, across two
> timing contracts and a second budget. The reproducibility evidence now covers
> the configuration the paper actually claims, which it did not before.
>
> This is no longer narrated. `scripts/cross_isa_reproduction.py` derives the
> overlap on every run of `scripts/record_checks.sh`, refuses to report if the
> pair count falls below 79, and treats an absent field as a difference so that
> a cell with less to disagree about cannot pass. It is negative-tested against
> a one-digit perturbation deep inside a trajectory, a dropped field, and a
> shrunken overlap.
>
> **Sixteen Azure cells have no AWS twin** and are excluded from the claim: the
> four h256 / d32-L4 cells (the ladder rung AWS never ran) and the twelve
> h1024 / d64-L4 cells of the voided AZ8-6 arm.

These are genuinely separate runs on separate silicon: `wall_secs` for the same
cell is **12,220 s on AWS and 38,063 s on Azure**, a 3.1× difference.

## 2. Why accuracy alone would not have been enough

`accuracy` is `n_correct / 2264`, so it takes only 2,265 distinct values. Two runs
whose internals differ slightly can print the same accuracy whenever no test
sample flips its argmax. **Identical accuracy is therefore not evidence of
identical computation**, and the first comparison here — which showed identical
accuracy — was not treated as a result.

The claim rests on the continuous quantities instead: `mean_loss`,
`mean_gradient_norm`, `mean_update_rms`, `mean_firing_rate`,
`tail_loss_improvement`, and 1,200 per-epoch trajectory values per cell. Those
have no such quantisation, and they agree everywhere.

## 3. What this changes

The 2026-08-19 measurement compared **macOS against Linux**. This one compares
**Linux against Linux** across two instruction sets. The variable that moved in
the first comparison and not in this one is the **libm implementation** — Apple's
versus glibc's `exp`/`sin`/`cos`/`powf`/`ln`.

- **Superseded reading:** *"results are not reproducible across machines."*
- **Supported reading:** *"results are reproducible across CPU architectures under
  the same libm; the only observed divergence is Apple libm versus glibc."*

That is a materially stronger and more precise reproducibility claim, and it is
independently verified on two clouds.

### Consequence for the campaign record

`RECONCILIATION_2026-08-21_TWO_PREREGS_ONE_QUESTION.md` §3.4 required AWS and
Azure numbers to be read only as within-fleet paired contrasts. **For
glibc-to-glibc comparisons that restriction is now unnecessary** — though nothing
in the reported verdicts depended on it, because every registered contrast was
already paired within its own fleet. The restriction **still binds** for any
comparison against a macOS-recorded reference, including the historical 0.7378.

### Consequence for AZ8-1

AZ8-1 asked whether the headline replicates on x86, and the campaign stopped
before its h128 `published-2ms` **e400** arms ran (§4 below). But the question is
answered anyway, and more strongly than it was asked: **the instrument does not
merely replicate across ISA, it reproduces exactly.** A statistical replication
of h128 would have added nothing that these values do not already establish.

> **CORRECTED 2026-08-25 — twice.** This paragraph read *"the watchdog killed
> the campaign before a single h128 cell ran."* Both halves were wrong.
>
> The watchdog did not kill it — the operator deallocated the fleet on
> exhausted credit, corrected in
> [`RESULT_2026-08-22_AZURE_TRUNCATED_AT_95_OF_252.md`](RESULT_2026-08-22_AZURE_TRUNCATED_AT_95_OF_252.md)
> and not carried back here. And **29 h128 cells ran**: twelve at `fixed-t250`,
> nine at `fixed-t500`, eight at `published-2ms`/e200. The claim was written
> from the arm the hypothesis named rather than from the archive, and the
> archive was on disk at the time.
>
> It matters beyond tidiness, because those 29 cells are what extends the
> reproduction to the headline width — the amendment in §1. The sentence
> asserting no h128 data existed sat directly above a table that would have
> shown otherwise had it been derived rather than chosen.

## 4. How this evidence came to exist

Not by design. The Azure campaign stopped at **95 of 252 cells** when the
operator deallocated the fleet at ~01:44Z, having exhausted the Azure credit
budget; the last result was written at **01:43:25Z**.

Longest-processing-time-first scheduling meant the surviving cells were the most
expensive ones — the h1024 and h512 arms — which happened to be exactly the ones
the AWS campaign had also run. **The cross-architecture evidence exists because
the campaign failed in the particular way it did.** Had it completed in
registration order, the overlap would have been the cheap h128 cells and the
comparison would have been far weaker.

This is luck, and is recorded as luck.

## 5. Scope

- **Verified:** all 57,960 comparisons, this session, from the two campaigns'
  archived cell JSON; the differing binary hashes; the 3.1× wall-time difference
  confirming independent execution. **Re-verified and widened 2026-08-25** to
  122,713 values over 79 pairs, by `scripts/cross_isa_reproduction.py` under
  `scripts/record_checks.sh`.
- **Verified:** the 95/252 truncation, from the Azure blob container's own
  timestamps. **The timing was verified; the cause attributed to it was not** —
  the fleet was deallocated by the operator on exhausted credit, and a watchdog
  firing at the predicted minute would have left the same trace. Corrected
  2026-08-25; the original wording of this bullet claimed "the watchdog timing"
  as verified, which conflated a measurement with an inference drawn from it.
- **Not verified — and the limit of the claim:** cell JSON serialises floats at
  about nine decimal places. This establishes agreement **to that precision across
  57,960 values**, not bit-identity of the underlying `f32`. Proving bit-identity
  needs a raw-bit comparison, which the cell format does not currently carry.
- **Not claimed:** that macOS agrees with either. It does not — that is the
  original measurement, and it is unchanged.
- **Not claimed:** anything about Windows, musl, non-glibc libm, or other ISAs.
  Two architectures under one libm is the evidence.
