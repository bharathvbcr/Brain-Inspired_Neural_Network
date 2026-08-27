# Venue formatting notes (skeleton)

Status: **skeleton + bibliography stubs** (REPRO §G partially checked). Final
camera-ready style pass still open once venue is chosen.

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
   **one width (h128)** — a scope limit that has to reach the main text, not a
   footnote.
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

**Working default:** ML methods track, anonymous submission, venue template TBD.
The lead is a positive, preregistered contrast with a destroyed-structure
control; a negative-results venue is the wrong home for it and was the right
home for the paper this file used to describe.

---

## Formatting checklist

- [x] Claim ladder frozen ([`PUBLISHABLE_CLAIMS.md`](PUBLISHABLE_CLAIMS.md))
- [x] Cite-every-number table ([`PAPER_RESULTS_TABLE.md`](PAPER_RESULTS_TABLE.md))
- [x] Prose draft with §3.1 ↔ §4.2 honesty ([`PAPER_DRAFT.md`](PAPER_DRAFT.md))
- [x] Bibliography stubs ([`references.bib`](references.bib))
- [x] Every number in the manuscript checked — 72 derived from cells, 7 named in
      `ELSEWHERE`, 40 traced to a named primary record
      (`scripts/check_every_number.py`)
- [ ] **Lead-program figures: one of four drawn.** Figure 1, the
      difference-in-differences, was drawn on 2026-08-27
      (`leadfig1_the_conditional`). The headline accuracy, the width ladder and
      the resolution ladder are still marked **"none — this figure has no
      artwork"** in [`PAPER_FIGURE_SPEC.md`](PAPER_FIGURE_SPEC.md), and the spec
      records `TODO(source needed)` for a lead-program graphical abstract. All
      three are fully specified and drawable now — none waits on a wave.
- [ ] Secondary-program artwork: four of nine files current, two stale — see the
      table below
- [ ] Venue template applied (NeurIPS/ICML/TMLR/… `.sty` / Overleaf)
- [ ] Anonymous PDF build
- [ ] Caption pass against [`PAPER_FIGURE_SPEC.md`](PAPER_FIGURE_SPEC.md)
- [ ] Page budget / appendix split (G3/G4/H0 appendix-only)
- [ ] Final copy-edit for "broadcast ±1 three-factor" terminology consistency

---

## Required main-text disclosures (do not drop)

**Lead program**

1. The mechanism control exists at **one width (h128)**. Every statement about
   the difference-in-differences generalising beyond it is unsupported by the
   corpus, and `scripts/mechanism_coverage.py` recomputes this on every gate run.
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
6. Discrete EventProp H2H **FAIL** `c1-eventprop-5bb083d5e88d0ad2` ≠ continuous
   Wunderlich–Pehle.
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
| fig2 (Figure 6) | `…/fig2_matched_means.*` | **stale** — plots the superseded block; no generator |
| fig4 (Figure 8) | `…/fig4_transfer_ladder.*` | **stale at rung 1**; no generator |
| fig0 | `…/fig0_claim_axis_legend.*` | present, unaffected by the re-run |
| figD | `…/figD_diff_closure.*` | present, unaffected |
| fig5 (Figure 9) | `…/fig5_xor_locality.*` | present, unaffected |
| **lead Figure 1** | `…/leadfig1_the_conditional.*` | **drawn 2026-08-27** — the difference-in-differences |
| lead Figures 2–4 | — | **no artwork**; specified and drawable, waiting on nobody |
| lead graphical abstract | — | `TODO(source needed)` |

The four current files have one owner, `binn-lab/src/paper_figures.rs`:

```
cargo run --locked --release -p binn-lab --features plots --bin paper-figures -- \
  --out results/runs/2026-07-23-paper-hard-both/figures
```

`scripts/test_paper_figures_match_the_spec.py` fails if the generator's numbers
and the spec sheet disagree. The two stale files have **no** generator, so they
are authoring work rather than a re-run.

Orphans under `results/fig1_ladder.png` etc. are **not** camp MUST artwork.

---

## Bibliography ownership

Primary stubs live in [`references.bib`](references.bib). Expand DOIs / venue
pages when locking the template. Prefer citing on-disk hashes in Methods over
inventing external "BINN" papers.
