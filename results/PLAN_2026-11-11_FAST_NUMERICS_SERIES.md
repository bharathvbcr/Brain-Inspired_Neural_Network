# PLAN: an opt-in fast-numerics training mode, and the series that would license it

**Status: PLAN, 2026-11-11. Nothing is registered or built.** The owner asked for
this plan ("Yes, plan it") after the bit-exact kernel work on ft-d94b. It has
two stages:

1. A measurement spike, scratch only and at most 3 days. It decides whether the
   mode is worth building at all.
2. Only if the spike clears the bar below: the engineering contract and a
   bridging series. Its section 4 is written to become a `PREREG_` once the
   owner commits it.

Claims are marked **verified** (with a source), **inferred** or **unverified**.
Design advice is from Fable (2026-11-11). The orchestrating session checked the
record citations.

## 0. The verdict up front

**Do the spike first. Register nothing until it reports.** A numerics-changing
mode pays only in a narrow case, and the record already has a zero-risk
alternative.

- **Exact mode is already fast.** As of main 68a32a3, the d32/L4 attention
  train-cell takes 2.56× fewer cycles than aa073b0, the h512 base cell 2.82×
  fewer than 8c00289, and Gate F replays 218/218 bit-identical (**verified**,
  card ft-d94b).
- **The SHD queue is small.**
  - ft-891's comparators come to about 29 cell-hours (**inferred**, ft-891
    rev 3).
  - ft-8de0's event-gated read-out comes to about 22–31 h (**inferred**).
  - Both must reproduce the *exact* anchor, through HE-0 and the same-binary
    arms, so both stay exact whatever this plan decides.
- **SSC is the only large job.** SSC has 75,466 / 9,981 / 20,382
  train / validation / test samples (**verified**, zenkelab.org and
  PMC9479696). That is 9.25× SHD's 8,156.
  - A 2-epoch d32/L4 attention cell costs 138.7 s of user CPU at 68a32a3
    (**verified**).
  - Scaled linearly (**inferred**), that is about 1.9 CPU-h for an SHD
    attention cell at e100 and about 18 CPU-h for an SSC one.
  - The SSC anchor has 72 cells: rate and d32/L4 × intact, bin-shuffled and
    hidden-shuffled × 12 seeds. 36 of them are attention cells, about
    **650–700 CPU-h exact at e100** and about 4× that at e400 (**inferred**).
- **Exact mode is portable.** Exact-mode cells already run on Graviton (the
  record's c7g corpus). On one c8g.16xlarge, the e100 SSC anchor is about 15 h
  of wall time, about $45 at spot prices (**inferred**).
- **Precedent.** ft-3966 refused numerics-changing speedups under **1.6×**,
  because re-running fresh controls cost more than they saved (**verified**,
  ft-3966 card).
- **Amdahl ceilings** (**inferred** from the shares in
  `results/train_cell_kernel_2026-11-10/` and the round-4 profile):

| path | end-to-end gain on the attention cell | runs on Graviton | deterministic | engineering |
|---|---|---|---|---|
| FMA in `neon::dot_block` / `axpy_blocks` / `dot_columns` | 1.15–1.3× | yes | yes, by construction | days |
| plus multi-accumulator reassociation | a further +5–15% | yes | yes, if the split is fixed | days |
| Accelerate sgemm on the four T²·d loops and the transposes | 1.5–1.7×; about 3× only if the whole forward and backward are rewritten as batched GEMMs (weeks) | **no** | must be pinned and probed | 1–2 weeks |
| Metal | forward 1.5–2×, backward unmeasured, GPU `exp` ≤ 2 ULP from libm, flushes subnormals | no | needs a subnormal guard | weeks |

- **The base arms gain nothing meaningful.** They are substrate-bound and
  already vectorised.

So a bridge (section 4) plus a second full Gate F costs more than it saves,
unless the spike shows **≥ 2.0×** *and* SSC must run on the Mac. The bigger lever
on SSC cost is the epoch budget, which moves cost 5–10×. The gain itself depends
on the budget: +0.1499 at e100 against +0.1275 at e400 (**verified**,
DID_INTERVALS_2026-09-29). That is a decision for the SSC preregistration
(ft-b219), not for this plan.

## 1. Stage 1: the spike (at most 3 days, scratch branch, nothing registered)

1. **Re-profile at 68a32a3** on a quiet-as-possible machine. Profile the
   2-epoch oracle cell and an e20 cell, so that one-off loading costs
   (`frame_events`) do not inflate the shares. Compute the Amdahl ceiling of
   each path from the measured shares.
2. **FMA variant** behind `--numerics fma`, in the three kernel bodies only
   (`vfmaq_f32` in `dot_block` / `axpy_blocks` / `dot_columns`, and `mul_add`
   in `axpy_into`). Run interleaved ABBA, min of 4, `/usr/bin/time -l` cycles,
   on the 2-epoch d32/L4 cell and the h512 cell, against 68a32a3.
3. **FMA plus fixed 2- or 4-way accumulator splits**, measured the same way.
4. **Accelerate, scratch only, never committed.** Replace the four
   `dot_columns` call sites with `cblas_sgemm` through `#[link(name =
   "Accelerate", kind = "framework")]`, with no crate. Measure cycles **and wall
   time**, because AMX work may not appear in CPU cycle counts. This needs the
   owner's go-ahead even in scratch, because it is the dependency question.
5. **A/A determinism** for every variant. Run the same seed twice at each of
   `RAYON_NUM_THREADS` ∈ {1, 3, 16}. The final weights must be bit-identical.
   For Accelerate, find the current single-thread setting in Apple's
   documentation first (**unverified**: do not code it from memory).

**Decision rule, stated now.** Proceed to stage 2 only if a variant reaches **≥
2.0× fewer cycles (or less wall time, for Accelerate) on the attention cell
than 68a32a3** *and* passes A/A. Otherwise write a `MEASUREMENT_` record of the
numbers and close the line. ft-d94b's own refutation bar was 1.5×. 2.0× is
where the bridge pays.

## 2. Determinism contract for any fast mode

- **Run-to-run and thread-count bit-identity, as in exact mode.** The existing
  tests run unchanged in fast mode:
  - `parallelism_is_bit_identical_to_serial_for_every_arm`, threads
    {1, 2, 3, 5, 8, 16};
  - `range_parallel_fold_is_bit_identical_to_serial_add_assign`.
- **Within-platform only.** That is the record's existing contract. Apple libm
  and glibc already differ on `exp`, `ln`, `tanh` and `powf`
  (**verified**, `results/FINDING_2026-08-19_LIBM_PORTABILITY_OF_REPLAY.md`),
  and platforms are already separate corpora (**verified**,
  `results/AMENDMENT_2026-09-07_WAVE_29_RUNS_ON_THE_LOCAL_PLATFORM.md`).
  - An FMA kernel would match between M5 and Graviton, but a whole cell would
    not, because of libm.
  - Accelerate adds a third axis: its kernel choice can change with a macOS
    update. Every cell records the `sw_vers` build, and replay across an OS
    update is not promised.
- **A fast-mode Gate F of its own.**
  - A second fixture set in `scripts/train_cell_fixtures.py`, with
    `--numerics fma` and its own oracle sha.
  - `gate_f_rust.py --numerics …`, which regresses recorded fast cells from
    their own root.

## 3. Engineering contract (stage 2 only)

- **Selection.**
  - A runtime flag `--numerics exact|fma|blas`, default `exact`.
  - A cargo feature alone is not enough, because a feature-built binary is
    invisible in the record.
  - Linking Accelerate needs a feature that is off by default, and `blas`
    without that feature is an error.
  - Exact mode stays byte-identical: pins, the 23 fixtures and Gate F 218/218
    are unchanged.
- **Recording.**
  - The cell JSON always carries a `"numerics"` field, even in exact mode. The
    schema's own rule, from the `attn_readout` precedent, is that an absent
    field must never mean two things.
  - A trap: `canonical_cell_sha256` hashes the JSON, so the new key changes all
    23 fixture `cell_sha256` values while `weights_sha256` stays the same.
    Re-record them in the same commit, with the unchanged weights hashes as the
    proof.
  - Do **not** reuse `backend`, because `gate_f_rust.py` keys on it.
- **Gate F.** `regress_cell` refuses a numerics mismatch as unrunnable
  (`NUMERICS_MISMATCH`), never as a comparison result. Do **not** add
  `numerics` to `COMPARED_FIELDS`: the frozen analysers are coupled to that
  tuple through `test_reproduction_check`.
- **Corpora.**
  - Fast cells live in `results/<wave>_fma/` (or `_blas`), with their own
    `CORPUS_BASELINE` keys.
  - `cell_validity` gains a homogeneity check: one numerics value per
    directory, and an exact corpus may never contain a fast cell. A missing
    field means exact.
- **Launchers.** The plan JSON and the `scripts/aws/run_cell.py` builders carry
  the flag.

## 4. Stage 2 science: the bridging series (draft for a `PREREG_`)

It reuses the HA-0 template of `PREREG_2026-11-08_ATTENTION_ON_THE_DELAY_SUBSTRATE`.

- **Arms.** 48 cells: SHD anchor (h128, published-2ms, adjacent-sum-5), e100,
  the fast mode the spike chose, and the same 12 seeds as the W25 e100 record.
  The arms are rate and d32/L4, each intact and bin-shuffled. Hidden-shuffled
  is left out because it has no e100 exact record (**inferred** from the DID
  table; check before registering).
- **Hypotheses, read in order, stopping at the first failure.**
  - **HF-0, determinism.** 2 seeds × 2 reruns × 2 thread counts give
    bit-identical cells.
  - **HF-1, levels.** |fast intact mean − exact intact mean| ≤ 0.03 on both
    arms (the campaign bar).
  - **HF-2, equivalence (TOST, α = 0.05).** G within **±0.015** of the exact
    record. DiD within **±0.02**: ±0.015 is underpowered for DiD, given a CI
    half-width of about 0.0105 at n = 12 (**inferred**). Report the paired
    per-seed deltas too, without promising any variance reduction: a one-ulp
    divergence compounds like a new seed.
- **What may be claimed afterwards.**
  - The only cross-mode sentence is "equivalent within ±0.015 (G) / ±0.02
    (DiD) at the SHD anchor".
  - Fast and exact cells are never pooled into one statistic.
  - Every later fast wave carries its own in-wave controls.
  - No fast number is ever held to an exact bar (HE-0 style).
- **If it fails.**
  - HF-0 fails: the mode is withdrawn.
  - HF-1 or HF-2 fails: the mode is usable only for self-contained waves with
    their own controls, cited with the caveat, and SSC results are never
    compared to SHD across modes.
- **Cost.** 48 cells × about 1.9 CPU-h (e100 attention, half the cells) plus
  the rate cells comes to about 50 CPU-h. On top of that come a second
  218-cell Gate F of the new binary in exact mode (about 8 h) and the fast
  fixtures (**inferred**).

## 5. Value gate: what would run in fast mode

| work | mode | why |
|---|---|---|
| ft-891, the §4.8 comparators | exact | compared against the exact anchor arms |
| ft-8de0, the event-gated read-out | exact | HE-0 reproduces the exact dense gain |
| ft-fd514, the delay series | n/a | PyTorch reference, its own platform rules |
| ft-b219, SSC | **fast only if** the spike is ≥ 2× **and** SSC must run on the Mac | otherwise run exact on Graviton |

## 6. Owner decisions this plan needs

1. **Run the spike** (section 1). Items 1–3 and 5 need no new dependency.
   Item 4, the Accelerate scratch probe, is the dependency question.
2. **The registration bar:** ≥ 2.0× fewer cycles or less wall time, as above,
   or a different bar.
3. **Where SSC runs:** exact on Graviton (recommended, no bridge needed) or
   fast on the Mac.
4. **SSC epoch budget** (ft-b219's preregistration). It is the largest cost
   lever, at 5–10×.

## 7. Checklist, if stage 2 is reached

- [ ] Spike `MEASUREMENT_` record committed, with the decision against the 2.0× bar.
- [ ] `--numerics` implemented. Exact mode unchanged: pins in both profiles, 23/23 fixtures, Gate F 218/218, `find_weak_checks --strict`.
- [ ] Fast fixtures and fast Gate F. A/A determinism across thread counts.
- [ ] `numerics` in the schema, with the homogeneity check in `cell_validity`, `NUMERICS_MISMATCH` in Gate F, and `CORPUS_BASELINE` keys.
- [ ] This section 4 committed by the owner as a `PREREG_`, with a frozen analyser, before any bridging cell exists.
- [ ] A `RESULT_` record of HF-0 to HF-2, including any refutation.
