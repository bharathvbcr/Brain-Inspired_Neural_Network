# The paper's central claim has artwork

**2026-08-27.** `leadfig1_the_conditional` is drawn. It is the first artwork
the lead program has ever had.

## What was missing

[`PAPER_FIGURE_SPEC.md`](PAPER_FIGURE_SPEC.md) specifies four figures for the
lead program — the difference-in-differences, the headline accuracy, the width
ladder, the resolution ladder — and every one of them read:

> **Artwork target:** **none — this figure has no artwork.** Nothing has been
> drawn for it…

The manuscript's title is *"What a time-axis read-out buys is temporal order: a
difference-in-differences on SHD"*. Its central claim had no figure, while the
secondary program had nine files. That imbalance was invisible until
[`VENUE_FORMATTING.md`](VENUE_FORMATTING.md) was rewritten and its
`[x] Figure artwork complete` tick came off.

## What the figure had to avoid

The spec names **four things this figure must not be allowed to say**, and each
is now structural rather than a matter of care.

**1. It must not read as "SHD is temporal."** That is prior art and the figure
is not entitled to it — Cramer et al. 2022 could not exceed 60% on
spike-count-only SHD; the Neuromorphic Sequential Arena reports 86.48 → 68.51
with temporal processing removed model-side; Yu et al. randomise spike times at
fixed counts. Three independent destruction operators, one conclusion, all of
it prior. A banner across the top names all three under **"NOT SHOWN HERE, AND
NOT CLAIMED"**, and the centre of the figure is the *pair* of costs, both drawn
by the same helper at the same bar width with the same label sizes. The rate arm
is captioned "the control, and half the measurement".

**2. It must not be drawn as an ablation.** Bin-shuffling is applied to the
**data** — independently per sample, in both the training and test splits, so
the task itself becomes rate-solvable — and nothing is removed from the model.
The operation is spelled out where it is named; no panel carries a
model-component axis or an "attention off" label.

**3. It must not quote +0.1577.** The wave-17 analyser merged a `d32l1` archived
shuffled control into the `d32l4` comparison for twelve pairs and inflated the
cost from **+0.1347** to **+0.1577** — MET either way, effect size 17% high
([`AMENDMENT_2026-08-27_H17_2_MERGED_TWO_READOUT_DEPTHS.md`](AMENDMENT_2026-08-27_H17_2_MERGED_TWO_READOUT_DEPTHS.md)).
A test asserts that string never appears in drawable code.

**4. It must not imply n = 32 rescued anything.** Panel B draws n = 12 beside
n = 32 so the near-identity is the visible message: twenty further seeds move
the gain by +0.0017 and the shuffle cost by +0.0010.

## One deliberate departure from the spec's layout line

The layout section asks for each arm "drawn as an intact → bin-shuffled pair".
**Table SHD-2 prints `—` for the absolute bin-shuffled means at n = 32.** At that
sample size the two costs are the only quantities that exist, so an
intact → shuffled pair is not drawable there without inventing a number.

Panel A therefore plots the costs. The intact → shuffled pair exists at n = 12
alone, and that is Panel C — small, and labelled `n = 12` in the panel title,
because an accuracy-under-shuffle encoding given the centre of the figure would
reproduce prior art and lose the new result. The departure is recorded at the
spec's own artwork target so the next reader meets it there rather than
inferring it from the drawing.

## Bound to the sheet, not to the spec's restatement

`scripts/test_paper_figures_match_the_spec.py` grew from 13 tests to 22. The
lead-figure tests check every drawn constant against **Table SHD-2 of
[`PAPER_RESULTS_TABLE.md`](PAPER_RESULTS_TABLE.md)** — the sheet itself, not the
spec's copy of it — so the two cannot drift together.

Six breaks, each confirmed to fire:

| break | caught by |
|---|---|
| the inflated +0.1577 returns | `test_the_inflated_cost_is_never_drawn` |
| the rate arm is drawn narrower than the attention arm | `test_both_arms_are_drawn_at_equal_weight` |
| the prior-art banner is dropped | `test_the_prior_art_is_named_in_the_figure` |
| the figure is relabelled as an ablation | `test_the_shuffle_is_described_as_done_to_the_data` |
| n = 12 is dropped so n = 32 reads as a rescue | `test_the_smaller_sample_is_drawn_beside_the_larger` |
| a constant drifts from Table SHD-2 | `test_every_lead_value_is_in_table_shd_2` |

One test has no counterpart anywhere else: `test_no_absolute_shuffled_mean_is_drawn_at_n_32`
asserts the source defines no `ABS_*_SHUFFLED_32` constant at all. There is no
value to check it against — the point is that a figure quoting one would be
quoting a number nobody published.

A rounding check came out of drawing it: Table SHD-2 prints **96%**, and
`{:.1}` rendered it as 96.0%. A figure that adds a decimal the sheet does not
carry is quoting a number nobody computed, so the formatter now drops a
trailing zero.

## What was still undrawn — superseded the same day

> This section read: *"Lead Figures 2, 3 and 4 … All three figures are fully
> specified with their numbers already in hand; none waits on wave 18, 20 or
> 21."* That was true when written and stopped being true a few hours later.
> **All three were drawn on 2026-08-27**
> ([`HARDENING_2026-08-27_THE_LEAD_PROGRAM_IS_DRAWN.md`](HARDENING_2026-08-27_THE_LEAD_PROGRAM_IS_DRAWN.md)),
> and the manuscript now calls out all four.

What remains is a **lead-program graphical abstract**, which
[`PAPER_FIGURE_SPEC.md`](PAPER_FIGURE_SPEC.md) records as
`TODO(source needed)` — the only piece of figure work in the package with no
specification at all — and callouts for the secondary program's Figures 5–9.

The correction is left visible rather than rewritten away, because a
forward-looking section is the part of a record most likely to rot: it is a
claim about a state that is *expected* to change, and nothing in this repository
checks prose against the state it describes. `check_record_links.py` verifies
that links resolve, not that sentences are still true.

## Files

- `binn-lab/src/paper_figures.rs` — `draw_lead_fig1`, `cost_bar`, the Table
  SHD-2 constants
- `scripts/test_paper_figures_match_the_spec.py` — 22 tests
- `results/PAPER_FIGURE_SPEC.md` — Figure 1's artwork target and the recorded
  departure
- `results/VENUE_FORMATTING.md` — one of four lead figures drawn
- `results/runs/2026-07-23-paper-hard-both/figures/leadfig1_the_conditional.{png,pdf}`
