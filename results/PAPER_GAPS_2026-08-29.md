# The paper's gaps, consolidated — 2026-08-29

**What this is.** Every open item standing between the package and a
submission, gathered from the four documents that each held part of the list,
with what closed today and what did not. It is a register, not a plan: nothing
here schedules work.

**Why it was needed.** The list existed in **five** places and no two agreed —
the fifth, [`PAPER_STATUS_2026-08-20.md`](PAPER_STATUS_2026-08-20.md), is an
earlier version of this same register and was found only by asking the question
a second time, after the first four had been reconciled.
[`VENUE_FORMATTING.md`](VENUE_FORMATTING.md) had a formatting checklist,
[`PAPER_FIGURE_SPEC.md`](PAPER_FIGURE_SPEC.md) had per-figure `Artwork target:`
statuses, [`PAPER_SKELETON.md`](PAPER_SKELETON.md) had a figure ↔ binary map,
and `scripts/test_paper_figures_match_the_spec.py` had a docstring naming a
"live authoring gap" none of the three mentioned. Three of the four were
**stale in a direction that hides work**: the skeleton said four drawn figures
had no artwork, the spec said no lead-program artwork existed at all, and
`VENUE_FORMATTING.md` required a **withdrawn** result be printed in the main
text. A gap register that reports work as outstanding when it is done, and done
when it is outstanding, is worse than none.

---

## 1. Closed today

Each was closed in code, with a test that fails against the pre-fix state.

| # | Gap | Where it lived | What closed it |
|---|---|---|---|
| 1 | **Figure 6 (`fig2_matched_means`) plotted the superseded value block** and **no generator produced it**, so the 2026-08-27 re-run that brought the rest of the package current could not reach it | spec §Figure 6 · venue inventory | `draw_fig6_matched_means` in `binn-lab/src/paper_figures.rs`; four bans specified and pinned |
| 2 | **Figure 8 (`fig4_transfer_ladder`) was stale at rung 1** — matched RL at 0.9200 / LCB 0.6846 — and had **no generator** | spec §Figure 8 · venue inventory | `draw_fig8_transfer_ladder`; three bans specified and pinned |
| 3 | **The manuscript cited no secondary-program figure.** Figures 5–9 existed and `PAPER_DRAFT.md` referred to none of them | the test file's own docstring, which scoped itself out of it | callouts in §3.1–§3.4; the test now covers all nine numbered figures |
| 4 | **`PAPER_SKELETON.md`'s figure map said the four lead figures had no spec entry and no artwork**, two days after they were specified and drawn — and called Figure 4 the *substrate panel*, where the spec and the draft both call it the resolution ladder | skeleton §3 note, §Figure ↔ binary map | map rewritten; `TheSkeletonsFigureMapIsCurrentTest` pins it |
| 5 | **Three published reproduction commands could not run.** `REPRO_ARTIFACT_CHECKLIST.md` §A passed three hashes retired on 2026-08-25, and §E a fourth; `from_hash` returns `None` for all four | the document a reviewer runs | commands moved to the frozen current hashes; `scripts/test_published_hashes_resolve.py` fails on any retired `--config-hash` anywhere in the record |
| 6 | **`PAPER_METRICS_FULL.md` Table A published the superseded block as current** — DFA 0.9387 / 0.6894, RL 0.9200 / 0.6846 — under retired hashes, with no banner | a `PAPER_SIDE` document, which `check_every_number.py` deliberately does not sweep | banner; Table A rewritten to the re-run; archived block kept and marked |
| 7 | **`PAPER_VERIFY.md` said "ALL MATCH freeze"** over three retired hashes and pre-repair numbers, with a Reproduce block that could not run | same | banner; per-row status; Reproduce block corrected |
| 8 | **`VENUE_FORMATTING.md` required a withdrawn result in the main text.** Disclosure 6 read "Discrete EventProp H2H **FAIL** `c1-eventprop-5bb083d5e88d0ad2`" — a FAIL withdrawn on 2026-08-25, under a hash retired the same day — inside the section headed *"do not drop"*, and it survived that file's 2026-08-27 rewrite | venue §Required main-text disclosures | rewritten: what must now be disclosed is the **withdrawal** |
| 9 | **The spec flagged a naming collision that does not exist** and instructed that one of two sheets be renamed before submission. Both sheets use the same two names and `PUBLISHABLE_CLAIMS.md` defines no table labels at all | spec §Where numbers come from | withdrawn, with the **real** ambiguity — `A`–`E` against `PAPER_METRICS_FULL.md`'s own `A`–`F` — left standing |
| 10 | **The lead-program graphical abstract was `TODO(source needed)`** — the last unspecified piece of figure work | spec · venue checklist · skeleton map | specified in the spec, drawn by `draw_lead_graphical_abstract`, four bans pinned. See §3: this is the one item where a judgement was made rather than a fact recorded |
| 12 | **`PAPER_STATUS_2026-08-20.md` was live in the index and stale in four places** — "A8 — LaTeX + figures: not started" (figures are complete), "`git remote -v` is empty" (`origin` is set and the record has been committed since 2026-08-23), "waves 1–9 complete, wave 10 in flight" (1–21 have landed), and a `shd-scientific-sweep` rename that now reaches one file. It is an earlier version of **this** register | found by asking the same question a second time | superseded banner pointing here; the index now marks it retired |
| 11 | **The substrate panel (Figure S) had no figure in any sheet.** §3.7 is a lead-program section with three waves behind it, and the omission was hidden behind a wrong label — the skeleton's map called it "Fig. 4", which is the resolution ladder's number | surfaced by closing item 4 | specified in the spec and drawn by `draw_fig_s_substrate`; **lettered** beside Figure M rather than renumbering the secondary program 5–9 → 6–10 one day after the last renumber. Four bans pinned, nine negative tests. **Every figure the package specifies is now drawn** |

### What the code now refuses

Three new guards, every assertion negative-tested by perturbing the source and
confirming exactly the intended test fails:

* `scripts/test_published_hashes_resolve.py` — the freeze comments must name the
  hash their test asserts; no published `--config-hash` may name a retired
  hash; every retired hash in a paper-side document must sit in a paragraph
  that marks it retired.
* `Figure6Test`, `Figure8Test`, `FigureSTest`, `LeadGraphicalAbstractTest` —
  every value against the **sheet** it is cited from (`Table A`, `Table C`,
  `Table SHD-1`, `Table SHD-2`, `Table SHD-7`), and every ban its spec section
  names.
* `TheSkeletonsFigureMapIsCurrentTest` — a drawn figure cannot be recorded as
  undrawn, and Figure 4 cannot go back to being the substrate panel.

`scripts/test_paper_figures_match_the_spec.py` went 60 → 113 tests.

---

## 2. Still open, and why

Nothing below is code-closeable today. Each says what it is blocked on.

### 2.1 Blocked on a venue decision (one decision unblocks four)

`VENUE_FORMATTING.md` records the working default as an ML-methods track with
an anonymous submission, and the template as TBD. Until that is a choice rather
than a default:

- **Venue template applied** (`.sty` / Overleaf).
- **Anonymous PDF build.**
- **Page budget and appendix split.** G3 / G4 / H0 are appendix-only; where the
  split falls depends on the page limit.
- **Caption pass.** Figures 1 and 6 carry **required wording** in the spec;
  the other seven do not, and writing them is prose work that a template's
  caption style constrains.

### 2.2 Authoring work, unblocked but not started

**Figure work is complete.** Every figure and both graphical abstracts the
package specifies are drawn, by one generator, each checked against the sheet it
cites and against every ban its spec section names. What remains here is prose.

- **Terminology copy-edit.** "broadcast ±1 three-factor" versus bare "broadcast
  credit topology". The requirement is recorded in four documents and is not
  mechanically checked.

### 2.3 Verification the repository cannot do for itself

- **§0's literature citations are unchecked against their primary sources**, and
  §0 says so on its face. Every frontier number in Figure 2 Panel B and on the
  lead graphical abstract came from a 2026-08-27 search pass;
  `check_every_number.py` does not sweep §0, and both figures state
  "NOT MACHINE-CHECKED" where they are read. Closing this means reading the
  papers.
- **The 0.087 residual attribution — CLOSED 2026-09-02, and it closed against
  the draft.** The ablation was run: `max_delay = 1` makes all three `Dcls1d`
  constructions pointwise, and the reference falls from 0.9387 to 0.6276. The
  kernel is worth **K = +0.3111**, which is **3.6× the residual it was invoked
  to explain**, and removing it drops the reference **0.2044 below** the
  instrument rather than down to it. So the kernel is real and measured — the
  weakest load-bearing inference in the paper is now the best-measured
  structural fact about the reference — but the *arithmetic* attribution is
  withdrawn: 0.087 is the net of at least two large terms of opposite sign, and
  §3.8 now says so. **The compensating ≈−0.20 term is not identified**, and
  naming it is the next ablation. Arm A reproduced the pinned 0.9376 at 0.9387
  on new hardware, which is what makes any of this readable
  ([`RESULT_2026-09-02_THE_KERNEL_ABLATION.md`](RESULT_2026-09-02_THE_KERNEL_ABLATION.md)).

### 2.4 Science, not paperwork

- **Synchrony is a second term in the mechanism — RESOLUTION LADDER CLOSED
  2026-09-03, second geometry still open.** `bin-shuffled` destroys temporal
  order and preserves within-bin synchrony; `channel-shuffled` destroys both.
  Wave 24 found the difference at `fixed-t250` (**+0.1038**) and nowhere else,
  and wave 25 measured what this entry asked for: **+0.1308** at `fixed-t100`
  and **+0.0575** at `fixed-t500`, against nothing at either `published-2ms`
  point. **Synchrony is an axis, not one point**, and the term falls
  monotonically as bins get finer (14.0, 5.6, 2.8 ms) — an observation carrying
  no registered claim, three points and no fitted curve. What this entry asked
  for and did not get is the **second geometry**: every synchrony cell is on
  `adjacent-sum-5`, so nothing separates the contract from the encoder
  ([`RESULT_2026-09-03_W25_THE_MECHANISM_WHERE_IT_WAS_UNMEASURED.md`](RESULT_2026-09-03_W25_THE_MECHANISM_WHERE_IT_WAS_UNMEASURED.md)).
- **NEW 2026-09-03: the recurrent substrate's mechanism is unmeasured, and the
  attempt to measure it failed.** Every manipulated cell in this corpus is
  feed-forward, so §3.7's claim about "what the read-out consumes" was resting
  on the feed-forward shuffle result. Wave 25 registered 72 recurrent cells to
  fix that and returned **NOT EVALUABLE**: ten failed on the instrument's
  non-finite-training guard, five of them the rate arm under time reversal,
  leaving three seed-paired quadruples against a floor of nine. `bin-shuffled`
  did reach nine (**+0.0291**, 6 of 9), which is a number and not a verdict. The
  registration forbade rescuing failed cells and none was rescued. **Closing
  this means a surrogate scale at which `rec+alif` is stable under reversal, and
  finding one is itself a wave**
  ([`RESULT_2026-09-03_W25_THE_MECHANISM_WHERE_IT_WAS_UNMEASURED.md`](RESULT_2026-09-03_W25_THE_MECHANISM_WHERE_IT_WAS_UNMEASURED.md)).
- **The reference's compensating −0.20 term — NAMED and 81.5% CLOSED the same
  day it was opened.** The kernel ablation showed the reference is not "the
  instrument plus a temporal kernel": the kernel is worth +0.3111 against an
  observed gap of 0.087, so a term worth roughly −0.20 was unaccounted for. It
  is the **membrane**. The reference's neuron retains 0.0050 per step against
  the instrument's 0.82, so with the kernel gone it has no temporal integration
  at all; matching the retention lifts it from 0.6276 to 0.7942, **M =
  +0.1666**. The decomposition closes: the reference's 0.1067 lead is
  +0.3111 − 0.1666 − 0.0378. **What remains unexplained is 0.0378**, below the
  0.05 this programme calls material, and the untouched candidates are the
  synaptic filter, the optimiser, the schedule and the read-out. Closing the
  last third is a further series and is no longer load-bearing for §3.8
  ([`RESULT_2026-09-02_THE_MEMBRANE_ABLATION.md`](RESULT_2026-09-02_THE_MEMBRANE_ABLATION.md)).

- **The gain/DiD dissociation is the leading open problem — NARROWED TWICE
  since this register was written, and still open.** At h1024 the read-out
  consumes temporal order (DiD **+0.1122** in 10 of 12 seeds against a ceiling
  of +0.02) while **harming** accuracy (**−0.1618**), and the paper's own
  registered prediction failed. Two waves have since moved it:
  wave 23 **excluded overfitting** and bounded the h1024 collapse in time — it
  is a late-training phenomenon and e100 avoids it
  ([`RESULT_2026-08-30_W23_THE_COLLAPSE_IS_LATE.md`](RESULT_2026-08-30_W23_THE_COLLAPSE_IS_LATE.md));
  wave 22 showed the dissociation is **not an h1024 pathology**, recurring at
  h512 (gain +0.0043, DiD +0.0893) and `channels-700` (gain +0.0243, DiD
  +0.1369) where nothing collapses. **What remains unexplained is why a
  read-out consumes temporal order while buying no accuracy**, and that is now
  a wider question than when it was written, not a narrower one.
- **Coverage is 21 of 21 operating points — CLOSED 2026-09-01 by wave 22.**
  It was 9 of 21 when this register was written, with twelve points carrying
  intact arms and no `bin-shuffled` twin. Widening it was compute, and the
  compute was spent: 504 self-contained cells, every one of the twelve clearing
  the registered bar
  ([`RESULT_2026-09-01_W22_THE_MECHANISM_AT_EVERY_OPERATING_POINT.md`](RESULT_2026-09-01_W22_THE_MECHANISM_AT_EVERY_OPERATING_POINT.md)).
  What replaced it as the scope limit was the **budget — CLOSED 2026-09-03.**
  Wave 24 measured two of the twenty-one at `e100` (+0.1183 and +0.0633, 12/12
  each) and wave 25 measured the other nineteen, all clearing the +0.03 bar
  (+0.0469 to +0.1381, seventeen at 12/12 and the weakest at 11/12). **Every
  operating point now carries the contrast at two budgets.** Wave 25 also
  answered wave 22's stranded depth question, H22-3, by running the `d32l4`
  anchor twins it lacked — and answered it **NOT MET**: four widths land within
  0.013 of their twin and **h768 is 0.1271 away**, outside the 0.10 bar, so
  **read-out depth replaces the budget as the scope limit** at one width of five
  ([`RESULT_2026-09-03_W25_THE_MECHANISM_WHERE_IT_WAS_UNMEASURED.md`](RESULT_2026-09-03_W25_THE_MECHANISM_WHERE_IT_WAS_UNMEASURED.md)).
  Wave 24 retired a limit this register never recorded: every DiD in the paper
  rested on ONE destruction operator, and `reversed` — which displaces further
  than bin-shuffling while destroying no information — now shows the cost is not
  displacement brittleness
  ([`RESULT_2026-09-02_W24_ORDER_SYNCHRONY_AND_BUDGET.md`](RESULT_2026-09-02_W24_ORDER_SYNCHRONY_AND_BUDGET.md)).
- **§6's audit debt is smaller than it reads.** The ~8,000 unswept lines in
  `binn-engine` / `binn-areas` / `binn-core` are **not on the cell path** — the
  instrument references those crates zero times. The two unswept files that
  *were* live, `shd_alif.rs` and `shared_bptt.rs`, were swept on 2026-08-30 and
  yielded two defects, both fixed
  ([`AUDIT_2026-08-30_SHD_ALIF_AND_SHARED_BPTT.md`](AUDIT_2026-08-30_SHD_ALIF_AND_SHARED_BPTT.md)).
- The standing register in
  [`TODO_2026-08-07_OPEN_WORK.md`](TODO_2026-08-07_OPEN_WORK.md) — the transfer-gap
  decomposition preregistration (§3), the SHD scope qualifiers (§4), the ~8,000
  unswept lines of BINN proper (§6). None of it blocks submission of the paper
  as scoped; §6 bounds what a future Gate 2 claim could be worth.

---

## 3. One judgement, recorded as a judgement

The lead-program graphical abstract (item 10) is the only entry above where
something was **decided** rather than **recorded**. The record said
`TODO(source needed)` and called it an open *authoring* task, which was exactly
right: every quantity it draws was already published, and what was missing was a
decision about what the paper's front image says.

That decision is now written into
[`PAPER_FIGURE_SPEC.md`](PAPER_FIGURE_SPEC.md) — message, layout, four bans,
and two disclosures an abstract cannot travel without — rather than only into
the generator, so it can be disagreed with in the place that owns it. The
substance is a compression of Figures 1–3 and asserts nothing they do not: the
pair of costs at equal weight, coverage at 21 of 21, prior art named, and the
ρ = −0.1430 strip that stops the image reading as a decomposition of the gain.

If the front image should say something else, the spec section is the edit.

---

## Companions

[`VENUE_FORMATTING.md`](VENUE_FORMATTING.md) ·
[`PAPER_FIGURE_SPEC.md`](PAPER_FIGURE_SPEC.md) ·
[`PAPER_SKELETON.md`](PAPER_SKELETON.md) ·
[`PAPER_DRAFT.md`](PAPER_DRAFT.md) ·
[`PAPER_RESULTS_TABLE.md`](PAPER_RESULTS_TABLE.md) ·
[`REPRO_ARTIFACT_CHECKLIST.md`](REPRO_ARTIFACT_CHECKLIST.md) ·
[`TODO_2026-08-07_OPEN_WORK.md`](TODO_2026-08-07_OPEN_WORK.md)
