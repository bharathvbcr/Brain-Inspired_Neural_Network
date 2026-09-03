# Venue formatting notes

Status: **venue chosen, template applied, anonymous PDF building and gated**
(2026-09-03). The submission is **NeurIPS 2026, main track, double-blind**, and
it builds from the manuscript rather than from a parallel `.tex`:

```
python3 scripts/build_paper.py        # generate, compile, and check
python3 scripts/build_paper.py --check   # check a build already on disk
```

`scripts/build_paper.py --check` runs inside `record_checks.sh` and fails on
any of: a modified style file, a section in neither half of the split, a paper
over nine content pages, or an identity leaking into the body text or the PDF
metadata. Each of those failures otherwise looks exactly like a finished
paper.

Companion: [`references.bib`](references.bib) · [`PAPER_DRAFT.md`](PAPER_DRAFT.md) · [`PAPER_SKELETON.md`](PAPER_SKELETON.md) · [`PAPER_FIGURE_SPEC.md`](PAPER_FIGURE_SPEC.md)

> **Rewritten 2026-08-27, because it described a paper that no longer exists.**
> Every judgment below used to be about the matched-architecture kill gate: it
> called the ±1 FAIL plus the transfer FAIL the "best fit", and picked a venue
> class to suit them. The manuscript now leads with the SHD read-out — its title
> is *"What a time-axis read-out buys is temporal order: a difference-in-differences
> on SHD"* — and the matched program is §3.3–3.4 supporting material. The old
> text scored **zero** mentions of attention, SHD, waves, or the
> difference-in-differences, which is how a venue note goes stale without ever
> looking wrong.

---

## What the paper is now

Two programs, in this order:

1. **Lead — the SHD read-out.** A difference-in-differences on the gain: what a
   time-axis read-out buys is *temporal order*, established by pairing an
   attention arm against its rate control on intact input and again on
   `bin-shuffled` input, seed-paired. Evidence is the `shd-attention-campaign-v2`
   wave series; the mechanism control's coverage is recomputed on every run of
   the evidence gate by `scripts/mechanism_coverage.py`, and is currently
   **21 of 21 operating points**, spanning every width from 128 to 1024 and both
   contracts and geometries. Until 2026-08-29 it was **one width (h128)**, and
   the disclosure below has been rewritten accordingly rather than deleted.
2. **Secondary — the matched-architecture kill gate.** Broadcast ±1 three-factor
   fails a task every other rule tested saturates; live muted-θ / k-WTA transfer
   fails. This is the program the old version of this file was written about.

## Target venue classes (pick one)

| Class | Fit | Style notes |
|---|---|---|
| ML methods / representation learning | **Best fit** for a seed-paired difference-of-differences with a destroyed-structure control | NeurIPS/ICML/ICLR main or TMLR; anonymous PDF; ~8–10 pp + appendix |
| Neuromorphic / SNN methods | Possible; SHD is the field's own benchmark, and the secondary program lands cleanly here | Emphasise the read-out contrast and the SuperSpike ceiling; refuse biology |
| Negative-results track | Fits the **secondary** program alone, and would bury the lead | Only if the lead is withdrawn |
| "Brain-like AI" venues | **Avoid** unless claims rewritten down | Would overclaim Assembly Calculus / cortex |

**Chosen 2026-09-03: NeurIPS 2026 main track, double-blind.** The lead is a
positive, preregistered contrast with a destroyed-structure control; a
negative-results venue is the wrong home for it and was the right home for the
paper this file used to describe. Nine content pages, with references, the
checklist and technical appendices excess to that.

**The style file is pinned by hash and was not taken from the first search
result.** `media.neurips.cc` 404s for the 2026 styles, so the file was recovered
from two unrelated arXiv submissions that ship it, which agree byte-for-byte.
The most findable third-party "unified" template is **27 lines longer** than
that — added comments and an extra `\AtBeginDocument` block — which is a
modified style file, and NeurIPS says modifying it may be grounds for desk
rejection. See [`../paper/STYLE_PROVENANCE.md`](../paper/STYLE_PROVENANCE.md).

**Where the page limit falls** is written down, not implicit:
[`../paper/split_manifest.toml`](../paper/split_manifest.toml) lists every
section as main text or appendix, and the build refuses a section that is in
neither. The main text is the SHD read-out programme end to end; the appendix is
the matched-architecture programme, which this manuscript already calls
supporting material. Fitting nine pages moved three sections out — §3.6, §3.8
and the whole secondary programme — and rewrote two: the abstract had grown to
795 words and the introduction was still about the paper this one replaced.

---

## Formatting checklist

- [x] Claim ladder frozen ([`PUBLISHABLE_CLAIMS.md`](PUBLISHABLE_CLAIMS.md))
- [x] Cite-every-number table ([`PAPER_RESULTS_TABLE.md`](PAPER_RESULTS_TABLE.md))
- [x] Prose draft with §3.1 ↔ §4.2 honesty ([`PAPER_DRAFT.md`](PAPER_DRAFT.md))
- [x] Bibliography stubs ([`references.bib`](references.bib))
- [x] Every number in the manuscript checked — **76** derived from cells, **14**
      named in `ELSEWHERE`, **40** traced to a named primary record
      (`scripts/check_every_number.py`); **125/125** published numbers
      independently recomputed (`scripts/verify_published_numbers.py`)
- [x] **Lead-program figures: all four drawn** on 2026-08-27, by the same
      generator as the secondary program's, each checked against the Table SHD-N
      it cites and against every ban its spec section names
- [x] **Lead-program graphical abstract specified and drawn** 2026-08-29. It was
      the last *unspecified* piece of figure work in the package, and it was an
      **authoring** task rather than a missing number: every value it draws was
      already published. The decision about what the paper's front image says
      is written into [`PAPER_FIGURE_SPEC.md`](PAPER_FIGURE_SPEC.md) so it can
      be argued with, and ten tests pin its four bans, each negative-tested
- [x] **Figure S drawn** 2026-08-29 — the substrate panel behind §3.7. It was
      the last piece of unspecified figure work in the package, and it had been
      hidden behind a wrong label: the skeleton's map called it "Fig. 4", which
      is the resolution ladder. **Every figure this package specifies is now
      drawn, and every one by the same generator**
- [x] **Figure 1 Panel D and the Figure 3 annotation drawn** 2026-08-29 for
      wave 21. Panel D is the eight-point difference-in-differences ladder;
      the Figure 3 annotation stops the width ladder implying the mechanism
      tracks the gain, which ρ = −0.1430 says it does not. Nine new tests pin
      the bans on both, each negative-tested
- [x] **Secondary-program artwork: all nine files current** as of 2026-08-29.
      `fig2_matched_means` (Figure 6) and `fig4_transfer_ladder` (Figure 8) were
      the last two, and they were the hard case: unlike the rest they had **no
      generator**, so the 2026-08-27 re-run could not reach them and one of them
      had been drawing the superseded value block at camera-ready quality since
      24 July. Both are now owned by `binn-lab/src/paper_figures.rs`, and
      `scripts/test_paper_figures_match_the_spec.py` checks each against the
      Table it cites and asserts every ban its spec section names
- [x] **Venue template applied** 2026-09-03 — NeurIPS 2026, `neurips_2026.sty`
      pinned at `0c1ad369…`, verified by two independent copies agreeing
- [x] **Anonymous PDF build** 2026-09-03 — `scripts/build_paper.py`, checked
      for identity in the rendered text *and* in the PDF metadata, which fail
      separately: pdflatex writes `/Author` from the environment unasked
- [x] **Caption pass** 2026-09-03 — and it turned out not to be writing work.
      This file said "Figures 1 and 6 carry required wording and the rest do
      not"; the spec in fact carries a `**Caption (required wording):**` block
      for **all seven**. So the build now READS them from
      [`PAPER_FIGURE_SPEC.md`](PAPER_FIGURE_SPEC.md) instead of holding a
      second copy, and a figure the spec does not caption is not typeset
- [x] **Page budget / appendix split** 2026-09-03 — nine content pages of nine,
      split written down in `paper/split_manifest.toml`
- [x] **Terminology check** 2026-09-03 — `scripts/check_terminology.py`. The
      requirement was recorded in four documents and checked in none, and one
      sentence in §3.4 had already drifted to a bare `broadcast` for an arm the
      figure spec calls `err_broadcast`. 102 uses now checked
- [x] **§0's citations read against their primaries** 2026-09-03 — three were
      wrong ([`CITATIONS_2026-09-03_SECTION_0_AGAINST_PRIMARIES.md`](CITATIONS_2026-09-03_SECTION_0_AGAINST_PRIMARIES.md))

---

## Required main-text disclosures (do not drop)

**Lead program**

1. **What the difference-in-differences does and does not generalise over.**
   The control exists at **21 of 21 operating points** (widths 128–1024, both
   contracts, both geometries) and clears its registered bar at every one of
   them, so the mechanism is no longer a single-configuration result.
   **Twelve operating points still carry intact arms with no `bin-shuffled`
   twin** and no mechanism claim is made at those.
   `scripts/mechanism_coverage.py` recomputes this on every gate run.
1b. **The effect's SIZE is not the gain, and this must not be softened.**
   Spearman ρ between the six per-width gains and their DiDs is **−0.1430**
   against a registered bar of **+0.829**. The claim is that the read-out's
   contribution *is* order-dependent, **not** that the gain decomposes into an
   order-dependent share and a remainder. Any sentence implying the latter is
   unsupported.
1c. **At h1024 the read-out consumes temporal order while harming accuracy** —
   DiD **+0.1122** in 10 of 12 seeds against a registered ceiling of +0.02,
   with a gain of **−0.1618** over those same seeds. The paper's own registered
   prediction failed here, and per the preregistration this is the **leading
   open problem**, not a caveat.
2. **Cross-machine Gate F FAILs macOS-vs-Linux by design.** No claim rests on a
   comparison against a macOS-recorded number; every contrast is against a
   control that ran beside its treatment on the same machine.
3. Scope limits on the recurrent contrast, including how many seed pairs survive
   and that divergence is not random.

**Secondary program**

4. Lead FAIL = **broadcast ±1 three-factor**, not "any broadcast": on the DFA
   schedule a broadcast-*graded* contrast reaches **0.9975**, and ±1 broadcast
   REINFORCE reaches **0.7775**. Neither is a PASS and neither erases the FAIL.
5. Live transfer = **v13–v24 FAIL**; v131 is matched-only.
6. **The discrete EventProp H2H FAIL is WITHDRAWN, and this instruction used to
   require printing it.** It read "Discrete EventProp H2H **FAIL**
   `c1-eventprop-5bb083d5e88d0ad2` ≠ continuous Wunderlich–Pehle" — a
   *required main-text disclosure* of a result the 2026-08-25 re-run withdrew,
   under a hash that was **retired** the same day and no longer resolves. It
   survived the 2026-08-27 rewrite of this file, inside the section headed
   "do not drop". What must be disclosed now is the **withdrawal**: the arm
   PASSes at **0.9450 ff / 0.8900 rec**, the archived 0.5000 was a spike-adjoint
   method with no spikes to differentiate through, it remains **discrete**, and
   **no comparison to continuous Wunderlich–Pehle is claimed in either
   direction** ([`RESULT_2026-08-25_MATCHED_ARCH_RERUN.md`](RESULT_2026-08-25_MATCHED_ARCH_RERUN.md)).
7. Integrity appendix for canonical C1 (H1/H2/θ=∞/`project`).
8. F1 / F2 / F5 efficiency honesty.
9. Non-claims: biology, AC PASS, impossibility, neuromorphic HW.

---

## Figure inventory vs camp artwork

Statuses are [`PAPER_FIGURE_SPEC.md`](PAPER_FIGURE_SPEC.md)'s, not this file's —
that sheet is the owner, and this table used to contradict it by ticking
"figure artwork complete" while the spec said figM was stale.

| Spec ID | Camp path | Status |
|---|---|---|
| **figM** | `…/figM_mechanism_*` | **redrawn 2026-08-27** to the re-run |
| fig1 (Figure 5) | `…/fig1_matched_rule_swap.*` | **current 2026-08-27** |
| fig3 (Figure 7) | `…/fig3_engine_c1_means.*` | **current 2026-08-27** |
| **graphical abstract** | `…/graphical_abstract.*` | **current 2026-08-27** (secondary program only) |
| **fig2 (Figure 6)** | `…/fig2_matched_means.*` | **authored and drawn 2026-08-29** — grouped by verdict, with the gap-LCB half of the gate beneath. Had no generator until then |
| **fig4 (Figure 8)** | `…/fig4_transfer_ladder.*` | **authored and drawn 2026-08-29** — rung 1 current, two gate axes, and a substrate break above the live arms. Had no generator until then |
| fig0 | `…/fig0_claim_axis_legend.*` | present, unaffected by the re-run |
| figD | `…/figD_diff_closure.*` | present, unaffected |
| fig5 (Figure 9) | `…/fig5_xor_locality.*` | present, unaffected |
| **lead Figure 1** | `…/leadfig1_the_conditional.*` | **Panels A–C drawn 2026-08-27, Panel D 2026-08-29** — the DiD across eight operating points |
| **lead Figure 2** | `…/leadfig2_headline_accuracy.*` | **drawn 2026-08-27** — headline, on one axis with the field |
| **lead Figure 3** | `…/leadfig3_width_ladder.*` | **drawn 2026-08-27**, annotated 2026-08-29 — the mechanism does not track this curve |
| **lead Figure 4** | `…/leadfig4_resolution_ladder.*` | **drawn 2026-08-27** — the resolution ladder |
| **lead Figure S** | `…/figS_substrate.*` | **specified and drawn 2026-08-29** — §3.7 had three waves behind it and no figure in any sheet. Lettered beside Figure M rather than renumbering the secondary program |
| **lead graphical abstract** | `…/lead_graphical_abstract.*` | **specified and drawn 2026-08-29** — the pair of costs at equal weight, coverage at 21 of 21, and the ρ = −0.1430 strip that stops it reading as a decomposition of the gain. A separate file from `graphical_abstract`, which is the secondary program's |

The four current files have one owner, `binn-lab/src/paper_figures.rs`:

```
cargo run --locked --release -p binn-lab --features plots --bin paper-figures -- \
  --out results/runs/2026-07-23-paper-hard-both/figures
```

`scripts/test_paper_figures_match_the_spec.py` fails if the generator's numbers
and the spec sheet disagree. Until 2026-08-29 two files had **no** generator and
so could not be brought current by a re-run at all; both now have one, and every
`.png`/`.pdf` in the camp is written by that single binary.

Orphans under `results/fig1_ladder.png` etc. are **not** camp MUST artwork.

---

## Bibliography ownership

[`references.bib`](references.bib) carries **ten SHD-literature entries whose
every field — volume, issue, pages, DOI, venue — came from the arXiv API or
Crossref on 2026-09-03**, not from recollection, alongside the secondary
programme's older stubs. `scripts/check_section0_citations.py` fails if that
file and the verification record disagree about which sources exist.

Prefer citing on-disk hashes in Methods over inventing external "BINN" papers.
