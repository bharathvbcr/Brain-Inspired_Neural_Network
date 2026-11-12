# DRAFT preregistration: is the `--numerics blas` build equivalent to the exact build at the SHD anchor?

**Status: DRAFT, 2026-11-11. Not attested.** It becomes a registration only when
the owner commits it to BINN `results/`, with its frozen analyser, before any
cell of the series exists. Git history is the attestation, and an agent session
may not attest it. Written in binn-wsp-study so that it does not touch the BINN
main checkout. Claims are marked **verified** (with a source), **inferred** or
**unverified**.

**Realises** section 4 of `PLAN_2026-11-11_FAST_NUMERICS_SERIES.md`, with two
changes the plan did not anticipate: a same-platform exact control, and 24 seeds
instead of 12 because the plan's bars have about even odds at n = 12. Section 0
records both, and the owner's choice on the second.

**Depends on:**
- the spike record `MEASUREMENT_2026-11-11_FAST_NUMERICS_SPIKE.md`, which cleared
  the plan's 2.0x bar for Accelerate on cycles (2.32x at 3 threads) and left
  stage 2 permitted;
- the `--numerics exact|blas` build (lane E1) and the gate and launcher work
  (lane E2), integrated on BINN main at 3a9e544, and
  `scripts/analyse_fast_numerics_bridge.py` (lane E3), on main at 5742329.

---

## 0. Two owner decisions, made 2026-11-11

**The owner chose option B of section 0.2: 24 seeds with the plan's margins.**
Option B adds seeds that have no fleet record, so it can only run against the
Mac-exact control of section 0.1. Choosing B therefore also settles 0.1: the
comparator is Mac-exact. The rest of this document is written for that design.

### 0.1 The comparator must be Mac-exact, not the fleet record (settled by option B)

The plan's section 4 compares blas cells with "the W25 e100 record". That record
was produced on the campaign fleet:
- `w25mec` ran on `c7g` spot instances, `aarch64-unknown-linux-gnu`, binary
  `3afd4434...` (**verified**, `RESULT_2026-09-03_W25_...` section 6 and
  `analyse_wave25.PINNED_BINARY`); its `w24ord` rate arms ran on the fleet two
  days earlier (**inferred** from the cell dates, 2026-09-01, and wall times of
  135-145 s per rate cell).
- `--numerics blas` runs on macOS only: Accelerate does not exist on Graviton.
- Apple libm and glibc disagree on `exp`, `ln`, `tanh` and `powf` across the whole
  domain (**verified**, `FINDING_2026-08-19_LIBM_PORTABILITY_OF_REPLAY.md`), and
  the record's standing rule is that a clause never subtracts a macOS number from
  a fleet number (**verified**, `AMENDMENT_2026-09-07_WAVE_29_RUNS_ON_THE_LOCAL_PLATFORM.md`
  section 4).
- No Mac-exact e100 anchor exists: the only local corpus is `wave29_local`, an
  e400 wave with different arms (**verified**, directory listing).

So blas against the fleet would measure numerics and libm together, and a failed
HF-2 could not be attributed. The rate arm's byte-identity, which the contract
promises and HF-0b tests, also holds on one platform only: the fleet's rate cells
cannot be reused as the blas wave's rate arms, and a fleet directory passed to
the analyser as "exact" fails HF-0b by construction (**inferred**: its rate
accuracies would differ by libm; the analyser's tests cover the shape, not a real
fleet cell).

**This registration therefore has a third corpus: Mac-exact cells, the same 24
seeds, the same four arms, the same host and the same binary build as the blas
cells.** The fleet record stays as a pinned descriptive line (section 4), read over
its own 12 seeds. The cheaper alternative, plan section 4 as written, costs about 29
CPU-h but cannot say why it failed if it fails, and a pass would be a statement
about the fleet record, not about the numerics.

### 0.2 At n = 12 the plan's bars certify a truly equivalent mode only about half the time (decided: option B)

Simulated from the fleet record's own seed spreads (attention intact sd 0.0144,
shuffled 0.0129), with the blas-minus-exact run noise taken as two independent
runs per arm (the plan's own premise: "a one-ulp divergence compounds like a new
seed"), the analyser's own TOST procedure, 400 simulations per row (**inferred**,
not measured on any blas cell):

| seeds n | margins G / DiD | P(both pass), true delta 0 | P(both pass), true intact delta +0.005 |
|---:|---|---:|---:|
| 12 | +-0.015 / +-0.02 (the plan's) | 0.51 | 0.37 |
| 12 | +-0.02 / +-0.03 | 0.92 | 0.83 |
| 24 | +-0.015 / +-0.02 | 0.91 | 0.72 |
| 24 | +-0.02 / +-0.03 | 1.00 | 0.98 |

The plan's CI half-width figure (0.0105 at n = 12, for one arm set) is the
within-mode one; a cross-mode contrast carries both modes' noise. Options:
- **A.** Keep n = 12 and the plan's margins. About a coin flip on a truly
  equivalent mode; a failure then says "not shown equivalent", never "different".
- **B.** Keep the plan's margins and double the seeds (5170013-5170024 added to
  both new corpora; the Mac-exact control makes new seeds possible, since no fleet
  record is needed for them). Costs about 147 CPU-h, 200 cells.
- **C.** Keep n = 12 and widen the margins to +-0.02 (G) / +-0.03 (DiD). Same cost,
  a weaker licensed sentence.

**Decision: option B.** The margins stay the plan's (G +-0.015, DiD +-0.02); the
seeds are 5170001-5170024. On the simulation above, a truly equivalent mode passes
both bars about 91% of the time, and one with a true intact delta of +0.005 about
72% (**inferred**, not measured on any blas cell). The analyser's `SEEDS` is
5170001-5170024 on BINN main at 5742329, whose constants test pins it (**verified**, 50 tests pass).

**The terminal-seed rule does not bar the new seeds.**
`PREREG_2026-08-20_AZURE_D32L4_SCOPE.md` makes twelve the terminal seed count for
the hypotheses registered under it, so that no seed is added to rescue a marginal
verdict. This is a new registration that fixes n = 24 before any cell exists,
which `scripts/aws/plan_cells.py` (the note above `SEEDS_EXTENDED`) records as
outside that rule. Seeds 5170013-5170024 continue the same arithmetic sequence.
No fleet cell exists at this operating point for them (**verified**: no
`e100__published-2ms__adjacent-sum-5` file at those seeds in
`shd_attention_campaign_v2`), so none can be reused or confused with a new one.

Size, same simulation at a true delta exactly on the margin: the percentile
bootstrap at n = 12 passes 6.5% (G) and 8.7% (DiD) of 600 trials, against a nominal
5% (**inferred**, simulation error about 1 point). It is slightly anticonservative
at that n, as the delay series' prereg warned at n = 3-5. The size at n = 24 was
not simulated; it is expected to be closer to nominal (**inferred**). The repo's
own interval tables use the same procedure; this series keeps it for consistency
and reports the size.

---

## 1. The question

`--numerics blas` replaces the attention read-out's matmuls with Accelerate
`cblas_sgemm`, single-threaded per rayon worker. It was 2.3x cheaper in CPU on the
2-epoch d32/L4 cell (**verified**, spike, one seed). It changes arithmetic, so no
exact bar may be applied to its cells without a bridge (plan section 4,
ft-3966 precedent). The question: **at the SHD anchor, are blas results
statistically equivalent to exact results, on one platform, within margins fixed
now?**

If yes, later Mac-only waves may run fast and cite one sentence. If no, the mode
is usable only inside self-contained waves with their own controls.

## 2. What is under test

- **Exact** is the default and is byte-identical to main 02c6e59 (contract).
- **blas**: Accelerate sgemm in the read-out's forward (q, k, v, scores, context,
  w_o) and backward (d_attention, d_v, d_q, d_k, the embedding dot), BLAS
  single-threaded on every rayon worker.
- **Base arms never touch BLAS**, so rate cells must be byte-identical across
  modes on one platform. HF-0b tests this in-wave.
- Every blas cell carries `"numerics":"blas"` and
  `"numerics_platform":"macOS <ProductVersion> (<BuildVersion>)"`; every exact cell
  carries `"numerics":"exact"` or, if old, nothing.

## 3. Design

Operating point (the plan's, the W25 e100 anchor): **h128, published-2ms,
adjacent-sum-5, e100, d32/L4 attention and the rate arm, intact and bin-shuffled,
seeds 5170001-5170024.** All new cells come from **one binary** built with the
`accelerate` feature, run on **one Mac** (one OS build), exact arms with the
default `--numerics exact` and blas arms with `--numerics blas`.

| corpus | cells | contents |
|---|---:|---|
| **exact** (Mac) | 96 | {rate, attention} x {intact, bin-shuffled} x 24 seeds |
| **blas** (Mac) | 96 | the same 96 keys |
| **determinism** | 8 | attention intact, seeds 5170001-2 x threads {1, 16} x reruns {1, 2} |

200 new cells. Each directory holds exactly its registered cells and nothing else
(`results/<wave>/` for exact, `results/<wave>_blas/` for blas and determinism,
per lane E2's corpus rule).

- **Hidden-shuffled is omitted.** It has no e100 exact record: the only
  hidden-shuffled cells are W27's e400 (**verified**: no `hidden-shuffled` e100
  file in `shd_attention_campaign_v2`; DID_INTERVALS has it at e400 only).
- **Rate cells are re-run in both modes, not reused.** Reusing the fleet rate cells
  would put fleet numbers into a Mac contrast (section 0.1); reusing Mac-exact rate
  cells as blas rate cells would skip the in-wave check of the contract's
  byte-identity, which is the cheapest strong test in the series (HF-0b). They cost
  about 0.1 CPU-h each.
- **Thread layout.** Every main-wave cell runs at the production layout, 3 rayon
  threads. The determinism cells run at 1 and 16. The two main-wave blas
  attention-intact cells of seeds 5170001-2 are a fifth witness each, so HF-0a
  covers threads {1, 3, 16}.
- **Neither the thread count nor the rerun index is in a cell**, so the
  determinism cells are named `...__t<threads>__r<rerun>.json` (a convention lane
  E2's launcher must write; the analyser refuses a file without it).
- **Staging, cheapest first.** Stage 0 runs the 96 rate cells, the 8 determinism
  cells and the two main-wave attention-intact blas cells of seeds 5170001-2:
  106 cells, about 18 CPU-h. The analyser reads HF-0 from those alone. Stage 1 is
  the other 94 cells (48 exact attention, 46 blas attention) and is read only if
  HF-0 passed. Launching stage 1 before HF-0 is read is allowed (as in the delay
  series); the analyser still reads HF-0 first.

## 4. The fleet record, pinned

For the descriptive line only (verified by recomputing from the cells and against
`DID_INTERVALS_2026-09-29.md`): attention cells
`w25mec__ff-fixed-attn__h128__e100__published-2ms__adjacent-sum-5__d32l4[__bin-shuffled]__s<seed>.json`,
rate cells `w24ord__ff-fixed__h128__e100__published-2ms__adjacent-sum-5[__bin-shuffled]__s<seed>.json`,
both in `results/shd_attention_campaign_v2/` of the main checkout, seeds
5170001-5170012, binary `3afd4434...`:

| quantity | fleet mean (n = 12) |
|---|---:|
| attention intact | 0.8209 |
| attention bin-shuffled | 0.6831 |
| rate intact | 0.6710 |
| rate bin-shuffled | 0.6592 |
| **G** (attention intact - rate intact) | **+0.1499** [+0.1419, +0.1580] |
| **DiD** | **+0.1260** [+0.1152, +0.1363] |

The analyser pins G, DiD and the four means as constants and prints the Mac-exact
corpus's difference from them as the **platform move**. It reads the Mac-exact
corpus over the fleet record's seeds 5170001-5170012 only (`FLEET_SEEDS`), so both
means cover the same seeds. The line is reported with the Wave 29
amendment's 0.03 as a reference. It gates nothing, voids nothing and changes no
verdict (the amendment's own rule for such a line).

## 5. Hypotheses, with the bars fixed now

Definitions, per seed s and per mode: G(s) = attention intact - rate intact;
DiD(s) = G(s) - (attention bin-shuffled - rate bin-shuffled). Deltas are
blas minus exact, paired by seed: dG(s), dDiD(s). Pairing does not promise a
variance reduction; it is the correct unit regardless.

Read **in order, stopping at the first failure**; nothing after a failure is
computed.

**HF-0, determinism and base-arm identity. Both parts must hold.**
- **(a)** For each of seeds 5170001 and 5170002, the four determinism cells and the
  main-wave blas attention-intact cell are bit-identical: every field equal except
  `wall_secs`, `emitted_unix_s`, `emitted_utc`.
- **(b)** All 48 blas rate cells equal their Mac-exact twin in every field except
  the clock fields and `numerics` / `numerics_platform`.
- Fail: the mode is withdrawn. (a) failing is a nondeterministic kernel; (b) failing
  means BLAS leaked into the base arms, or the two corpora are not one platform.

**HF-1, levels.** |mean(blas) - mean(exact)| <= 0.03 on the attention-intact arm
(the campaign's level bar). The rate-intact arm is held to the same bar, but HF-0b
already implies it: it is printed, and is not independent evidence. The
bin-shuffled levels are printed, ungated.

**HF-2, equivalence (TOST, alpha = 0.05).** One 90% paired percentile bootstrap
interval for each of mean dG and mean dDiD: 10,000 resamples of the 24 seeds, ONE
index vector per resample applied to both statistics, RNG seed 20261111, interval
elements 500 and 9499 of the sorted draws (the repo's `percentile_interval` at
alpha 0.10, which a test pins). **Equivalent iff the interval lies strictly inside
the margin**: lower > -margin and upper < +margin, so an interval touching a margin
fails. Margins: **G +-0.015, DiD +-0.02**. Both must hold. The per-seed deltas and
their sign counts are reported.

**Ties and bars.** Every bar comparison allows a tolerance of 1e-12, which can
only absorb float rounding of a value on a bar (accuracies are multiples of
1/2264).

## 6. What may be claimed, and what may NOT

**May claim, if HF-0 to HF-2 hold:** "`--numerics blas` is equivalent to exact
within +-0.015 (G) / +-0.02 (DiD) at the SHD anchor (h128, published-2ms,
adjacent-sum-5, e100, d32/L4, 24 seeds), on `<numerics_platform>`." That is the
only cross-mode sentence. It says nothing about SSC, other budgets, other widths
or other platforms.

**May NOT:**
1. **Pool** exact and blas cells into any statistic, mean or interval.
2. **Hold a blas number to an exact bar.** No later fast wave reuses an exact
   control or an HE-0 style gate against an exact anchor; every fast wave carries
   its own in-wave controls.
3. **Compare across platforms.** The fleet record is never a comparator in a clause.
4. **Re-run to improve a verdict**, add seeds beyond those registered, or change a
   margin after the first cell exists. A failed cell is re-run only for an
   infrastructure fault named in the run log, with the same seed, and every such
   re-run is listed. (The choice of option B in section 0.2 was made before the
   commit and before any cell, not after.)
5. **Carry a pass across a macOS update.** Accelerate's kernels may change with the
   OS; the claim is for the recorded `numerics_platform` only.
6. **Read HF-2 after a failed HF-0 or HF-1**, or report a partial read as complete.

## 7. If it fails

| failed | consequence |
|---|---|
| HF-0 | the mode is withdrawn (nondeterminism or base-arm leakage is a build defect, not a result); fix and re-register |
| HF-1 | usable only for self-contained waves with their own controls, cited with the caveat |
| HF-2 | the same. "Not shown equivalent", never "different": at n = 24 a truly equivalent mode still fails about 9% of the time (section 0.2, inferred); SSC results are never compared with SHD across modes |

In every failure case SSC runs exact on Graviton, as plan section 5 already
recommends when no bridge exists.

## 8. Analyser

`scripts/analyse_fast_numerics_bridge.py`, frozen at BINN main 5742329 (git blob
1dbf698e243ff2a40a68e2c64f13691f8f5edb81, sha256
11c48a27c93a4d99a8b651a1ee1f2e302dc94a4b61c2cf20c736c77e59ad4d86) before this
registration was committed, is the authority on every verdict. 50 tests on synthetic corpora in
`scripts/test_analyse_fast_numerics_bridge.py`.
- **Usage:** `analyse_fast_numerics_bridge.py <exact_dir> <blas_dir> --determinism <det_dir> [--json out.json] [--markdown out.md]`.
- **It reads only cell records.** Cells are keyed by their own fields (arm,
  condition, seed, shape), never by file name, except the determinism cells'
  `__t<threads>__r<rerun>` tokens.
- **Refuses, exit 1 with a named reason:** a missing directory, an unreadable
  record, a duplicate cell, an unregistered seed or operating point, a hidden-shuffled
  or any other condition, a `numerics` value other than the corpus's (a blas cell in
  the exact corpus, an exact cell in the blas one, any other value), a blas cell
  without a well-formed `numerics_platform`, an exact cell carrying one, more than
  one `numerics_platform` across the blas and determinism corpora, and any cell
  failing `cell_validity.validity_problems`.
- **Incomplete, exit 2:** a registered cell is missing. No verdict is computed from
  fewer cells than registered; HF-0 alone is printed if it alone is evaluable, and
  the output says INCOMPLETE.
- **Exit status:** 0 all passed, 1 invalid, 2 incomplete, 3 HF-0 failed, 4 HF-1
  failed, 5 HF-2 failed.
- **Output:** a verdict JSON (per-seed deltas, intervals, `complete`, `stopped_at`,
  the platform) and a markdown table for the RESULT.
- **Machinery:** the bootstrap is the repo's, re-implemented in the analyser and
  pinned by tests against `did_confidence_intervals.percentile_interval` and
  `.bootstrap` on the same random stream, so a frozen analyser cannot drift with
  that script.

## 9. Cost (all inferred unless marked)

Unit costs: exact attention cell at e100, 1.9 CPU-h (138.7 s of user CPU per 2
epochs at 68a32a3, **verified**, scaled linearly); blas 2.3x cheaper, 0.83 CPU-h
(spike, 2-epoch, **verified** on one seed); rate cell about 0.1 CPU-h (the fleet's
135-145 s wall, the Mac about 1.7x faster per cell per the Wave 29 amendment).

| block | cells | CPU-h |
|---|---:|---:|
| exact attention (Mac) | 48 | 91.2 |
| blas attention | 48 | 39.8 |
| determinism (blas attention) | 8 | 6.6 |
| rate, both modes | 96 | 9.6 |
| **total** | **200** | **about 147** |

Stage 0 (HF-0): 106 cells, about 18 CPU-h. On the shared Mac (load 35-205 during
the spike) wall time is a multiple of the CPU time divided by the cores one is
allowed; not estimated. For comparison, option A was about 77 CPU-h and 104 cells,
and the plan's section 4 design (no Mac-exact control) about 29 CPU-h and 56 cells.
The plan's own "about 50 CPU-h" priced the blas cells at exact speed. Already
done, not counted: the 218-cell Gate F of the new binary in exact mode (218/218
bit-identical, 2026-11-11, **verified**) and the blas fixtures (23/23).

## 10. Checklist

Before the commit:
- [x] The owner decides sections 0.1 and 0.2: option B, which fixes the Mac-exact
      comparator (2026-11-11).
- [x] The analyser's `SEEDS` (5170001-5170024) and `FLEET_SEEDS` are on BINN main
      at 5742329 (2026-11-11; 50 tests pass on main).
- [x] The `--numerics` build is integrated on main at 3a9e544 (2026-11-11): exact
      byte-identical (kernel pins in both profiles, 23/23 fixtures'
      `weights_sha256`, Gate F 218/218 with the Accelerate-linked binary in exact
      mode), blas fixtures recorded and re-compared 23/23 at a different job
      count, `find_weak_checks --strict` 39 findings, all in the baseline. No
      blas-mode Gate F was run: Gate F replays exact-mode records, which a blas
      cell is not meant to match; the blas fixtures stand in for it.
- [x] The analyser is frozen on main at 5742329, a commit before this one (the
      blob and sha256 are in section 8). This document is committed by the
      owner's instruction (2026-11-11) before any bridge cell exists.
- [ ] Lane E2's launcher writes the determinism cells' `__t<threads>__r<rerun>`
      names, writes exact cells to a directory of their own, and logs the OS build
      (`sw_vers`) and the binary sha256 before and after each stage: the analyser
      cannot see either for the exact corpus, and the Mac-exact and blas cells must
      come from one host, one OS build and one binary.
- [ ] `numerics_platform` is recorded in every blas cell (read from the cell, not
      assumed) and is one value for the series.
- [ ] The novelty of "equivalence bridge for an Accelerate-backed training mode" is
      not a claim here; no search was run.

After the series: [ ] a `RESULT_` record of HF-0 to HF-2, including any refutation,
with the analyser's markdown table and the platform-move line.

## 11. Unresolved, stated rather than waved past

- **HF-0 compares cell fields, not weights hashes.** The cell reports 9-decimal
  measurements and per-epoch traces; two runs could in principle share them and
  still differ in a weight. The fixtures' `weights_sha256` carries the stronger
  check, but only for the 23 fixture cells. (**unverified** how often the two
  would disagree.)
- **The production thread count (3) and the rerun index are launcher conventions,
  not recorded in cells.** The analyser can enforce neither for the main wave.
- **Same host, OS build and binary across the exact and blas corpora** are checklist
  items, not analyser checks, for the same reason.
- **The "independent runs" noise premise in section 0.2** is the plan's, not
  measured. If blas-minus-exact deltas turn out smaller (shared init and data order
  may help), the power is better than tabled; the draft does not rely on it.
- **A fleet corpus is refused as "exact" only by its extra cells and by HF-0b.** The
  tests construct the failing shapes; no real fleet cell was run through the
  analyser.
