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

AZ8-1 asked whether the headline replicates on x86, and the watchdog killed the
campaign before a single h128 cell ran (§4 below). But the question is answered
anyway, and more strongly than it was asked: **the instrument does not merely
replicate across ISA, it reproduces exactly.** A statistical replication of h128
would have added nothing that these 57,960 values do not already establish.

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
  confirming independent execution.
- **Verified:** the watchdog timing and the 95/252 truncation, from the Azure blob
  container's own timestamps.
- **Not verified — and the limit of the claim:** cell JSON serialises floats at
  about nine decimal places. This establishes agreement **to that precision across
  57,960 values**, not bit-identity of the underlying `f32`. Proving bit-identity
  needs a raw-bit comparison, which the cell format does not currently carry.
- **Not claimed:** that macOS agrees with either. It does not — that is the
  original measurement, and it is unchanged.
- **Not claimed:** anything about Windows, musl, non-glibc libm, or other ISAs.
  Two architectures under one libm is the evidence.
