# Two preregistrations cover the same questions — which governs, decided before the second lands

**Written:** 2026-08-21T16:45Z, **while the Azure campaign is still running and
before any Azure cell result exists** (0 of 252 complete, verified against the
campaign's own blob container). That timing is the entire point of this document:
once both result sets exist, choosing between two sets of bars is unfalsifiable.

---

## 1. What happened

Two campaigns were registered within ninety minutes of each other, by different
sessions, covering overlapping questions with **different thresholds**:

| | AWS wave 8 | Azure |
|---|---|---|
| prereg | `PREREG_2026-08-20_SHD_ATTENTION_HEADLINE_SCOPE.md` | `PREREG_2026-08-20_AZURE_D32L4_SCOPE.md` |
| registered | 2026-08-20, before any cell | 2026-08-20T19:20:33Z, before any VM |
| launched | 2026-08-20T20:49Z | 2026-08-21T06:45Z (gate quorum) |
| fleet | 4 × `c7g.16xlarge`, **aarch64** | 4 × `Standard_D64als_v7`, **x86-64-v4** |
| cells | 72 (+24 wave 9) | 252 |
| status | **complete**, 72/72, 0 failures | **running**, 0/252 |

Neither is at fault: they were registered in parallel and each is internally
disciplined. But two live preregistrations over one question is a hazard
regardless of intent, because whichever bar is cited can be chosen after the fact.

## 2. Where they overlap, and whether the bars differ materially

| question | Azure bar | wave 8 bar | wave 8 measured | same verdict? |
|---|---|---|---:|---|
| `channels-700` gain | AZ8-3: ≥ 0.05, ≥ **9**/12 | S-2: ≥ 0.05, ≥ **10**/12 | +0.1090, **12/12** | **yes** — SUPPORTED either way |
| h1024 gain | AZ8-2: ≥ 0.05, ≥ **9**/12 | S-3: ≥ 0.05, ≥ **10**/12 | −0.1618, **1/12** | **yes** — NOT SUPPORTED either way |
| contract gain (`published-10ms`) | AZ8-5: ≥ 0.05, ≥ **9**/12 | S-4: ≥ 0.05, ≥ **10**/12 | +0.1491, **12/12** | **yes** — SUPPORTED either way |
| budget stability | AZ8-4: \|Δ\| < 0.02 (e200→e400) | R-2: \|Δ\| < 0.02 | 0.0002 | **yes** |

**Every overlapping wave-8 measurement is far enough from both bars that the
9/12-vs-10/12 difference changes no verdict.** So this is a hazard that did not
fire. It is recorded anyway, because "it happened not to matter" is a fact about
the data, not about the process.

## 3. Ruling, fixed now

1. **On the four overlapping questions, `PREREG_..._HEADLINE_SCOPE.md` governs**
   and the wave-8 results stand as reported. Azure's AZ8-2/3/4/5 are a
   **replication on a different architecture**, reported as agreeing or
   disagreeing with wave 8 — never as a substitute verdict, and never as the
   citable one if they happen to be more favourable.
2. **Azure's genuinely new questions keep their own registration**, because wave 8
   does not cover them:
   - **AZ8-1** — does the headline replicate on **x86**? Wave 8 and 9 are entirely
     aarch64. This is the only cross-architecture evidence in the package.
   - **AZ8-6** — is fixed `d32` the bottleneck at **h1024**? Wave 9's `w9dim`
     tests d64 at **h128**; these are different configurations and neither
     substitutes for the other.
   - **AZ8-5's other contracts** — `fixed-t100 / t250 / t500`, unrun at e400 anywhere else.
3. **Cross-machine numbers are not compared absolutely.** The Azure binary is an
   x86-64-v4 build; the AWS binary is `22d97c51ab02` on aarch64. Gate F fails
   across architectures by construction
   (`MEASUREMENT_2026-08-19_CROSS_MACHINE_BIT_EXACTNESS.md`). Both campaigns are
   read only as **within-fleet paired contrasts**, which both preregs already
   require.

   > **REFINED 2026-08-22.** The premise of this clause turned out to be wrong in
   > one direction and right in the other. The two fleets' overlapping cells agree
   > on **57,960 of 57,960 serialised float values**, including full 400-epoch
   > trajectories — so aarch64 and x86-64 **do** reproduce each other under glibc,
   > and this restriction was unnecessary for glibc-to-glibc comparisons. It still
   > binds for anything compared against a **macOS-recorded** reference, which is
   > where the original divergence actually came from. See
   > `FINDING_2026-08-22_REPRODUCIBLE_ACROSS_ISA_UNDER_GLIBC.md`.
   >
   > **This changes no verdict**, because every registered contrast in both
   > campaigns was already paired within its own fleet. It does mean AZ8-2 is a
   > *reproduction* of wave 8's S-3 rather than an independent replication, so
   > **n stays 12, not 24** — recorded in
   > `RESULT_2026-08-22_AZURE_TRUNCATED_AT_95_OF_252.md` §3.
4. **No third campaign on these questions** without first superseding one of these
   two in writing.

## 4. An unrelated defect found while checking: the cell timeout exceeds the fleet watchdog

`scripts/azure/bootstrap.sh:19` sets `MAX_RUNTIME_SECONDS=68400` — a hard
deallocation of the **whole scale set at 19 hours**. The runner passes
`--cell-timeout-secs 86400`, a **24-hour** per-cell limit (verified on node 0's
live command line).

**A cell is therefore permitted to run five hours past the point at which its
fleet is destroyed.** Any cell needing more than 19 hours cannot report, ever;
it will always be killed rather than time out, and a killed cell leaves no record
of why.

### Whether this bites, measured rather than guessed

Anchoring on a real measurement — the AWS `h1024/d32l4/e400` cells took a median
**3.4 h at 16 threads** for an estimated 41 mac-core-hours:

| | |
|---|---:|
| total Azure work | ~3,644 mac-core-h |
| fleet throughput (256 vCPU, 4 threads/cell, 84% eff) | ~215 thread-h per wall h |
| **ideal makespan** | **~22.5 wall h** |
| **watchdog** | **19.0 wall h** (≈ 01:44Z, 2026-08-22) |
| longest single cell (`az8dim` h1024/d64l4/e400, ~74 mac-core-h @ 8 threads) | ~17.1 wall h |
| elapsed at time of writing | 10.0 wall h, **0/252 complete** |

The zero completions are **not** a stall — verified directly on node 0: load
average 58.4 on 64 vCPU, 13 live `shd-instrument` processes, and the cells in
flight are the most expensive in the matrix (`h1024/e400` at d32l4 and d64l4).
Longest-processing-time-first scheduling means the first ~17 hours produce
nothing by design.

> **NOT RESOLVED — the campaign was stopped for an unrelated reason.** At ~01:44Z
> the **operator** deallocated the fleet, having run out of Azure credit; 157
> cells never ran, including **every h128 cell**, so AZ8-1 (the primary
> hypothesis) has no data at all.
>
> An earlier revision of this line said the watchdog fired. **That was a wrong
> inference**, drawn because the predicted watchdog time and the observed stop
> matched to under a minute. The straddle below is therefore **still open**: the
> campaign never reached 19 hours, so whether the makespan would have overrun was
> never tested. The 24 h-cell-timeout-under-a-19 h-watchdog mismatch remains a
> real, unfired defect. Full outcome in
> `RESULT_2026-08-22_AZURE_TRUNCATED_AT_95_OF_252.md`.

**The estimate straddles the deadline.** If EPYC 9005 with AVX-512 is ~1.2×
Graviton3 per core the campaign lands around 18.8 h and finishes; if it is at
parity it overruns by ~3.5 h and the tail is destroyed. This is genuinely
marginal and cannot be resolved from outside — the first completed cell's
`wall_secs` will settle it.

**Not acted on.** Extending `MAX_RUNTIME_SECONDS` on a live scale set spends
credit and is the operator's call, not an agent's. The prereg's own envelope has
headroom — it registers a "37-hour cap costs at most $190.33" — so raising the
watchdog is within what was registered, but it is still a spend decision. Flagged
for a human.

## 5. Scope

- **Verified:** both preregs' thresholds, quoted from the files; wave 8's measured
  values from `analyse_wave8.py` over hash-verified cells; the Azure fleet's live
  state, blob inventory, plan composition, and node-0 process list, this session.
- **Verified:** `MAX_RUNTIME_SECONDS=68400` at `scripts/azure/bootstrap.sh:19` and
  `--cell-timeout-secs 86400` on the running command line.
- **Inferred, not verified:** the makespan estimate. It rests on an
  aarch64-to-x86 throughput assumption that has not been measured, which is
  exactly why it is presented as a straddle rather than a prediction.
- **Not claimed:** any Azure result. There are none yet, and this document was
  written so that there could be none when the ruling in §3 was made.
