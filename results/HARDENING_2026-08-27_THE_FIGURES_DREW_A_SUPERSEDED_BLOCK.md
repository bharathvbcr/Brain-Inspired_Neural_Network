# The camera-ready figures were drawing a value block the spec calls "not for drawing"

**2026-08-27.** Figure M is redrawn to
[`PAPER_FIGURE_SPEC.md`](PAPER_FIGURE_SPEC.md), three other figures are brought
to the 2026-08-25 re-run, and a test now binds the generator to the sheet.

## What was wrong

`binn-lab/src/paper_figures.rs` opens with *"Numbers are hardcoded from those
sheets — never remassaged."* It held:

```rust
pub const BROADCAST_GRADED: f64 = 0.9863;
pub const DFA: f64            = 0.9387;
pub const RL_FB: f64          = 0.9200;
pub const GRAD_MATCH: f64     = 0.8963;
```

with `"gap LCB 0.6894"` and `"gap LCB 0.6846"` written inline in the figure
bodies. [`PAPER_FIGURE_SPEC.md`](PAPER_FIGURE_SPEC.md) §"Figure 6" names that
exact set:

> **VALUE BLOCK REPLACED 2026-08-27 — the previous one was superseded.** It read
> DFA **0.9387**, RL **0.9200**, gradient ceiling **0.8963 / 0.8963 / 0.8887**,
> and gap LCBs **0.0000 / 0.6894 / 0.6846**. … they are pre-repair figures from
> a forward pass that emitted **zero spikes at any seed**. … recorded here as
> **superseded and not for drawing**.

Running the generator reproduced the four committed PNGs **byte-for-byte**. So
the artwork in `runs/2026-07-23-paper-hard-both/figures/` was the superseded
block rendered at camera-ready quality, and nothing anywhere connected the
generator's constants to the sheet they claimed to come from.

## Figure M, redrawn

The spec named two things Panel A must not be allowed to say. Both are now
structural rather than a matter of care.

**1. It is not a graded surface.** Six of seven arms sit between 0.78 and 1.00
against a ceiling of 1.0000; one sits at chance. Any accuracy-to-size mapping
manufactures an ordering the task cannot support. Panel A now draws labelled
verdict chips — `PASS` / `FAIL` / `contrast (not gated)` / `reference` — with
no magnitude encoding at all, and the chips carry text so the categories
survive a greyscale print rather than living in hue alone. The saturated
reference leads the panel instead of trailing it, because it is *why* the
passing arms cannot be ranked.

**2. The low/low cell holds two rules that disagree by 0.28.** `MatchedLocal`
(±1 × surrogate eligibility) is at chance, 0.5000 ff / 0.5100 rec.
`MatchedRlFlat` (±1 broadcast REINFORCE) reaches 0.7775 / 0.7962. They are now
drawn as two cards inside one shaded cell, labelled by **rule** rather than by
topology, with the distance between them printed under them. Collapsing them
is the overreach the lead claim's wording exists to avoid.

Panel B gained an explicit **chance line at 0.50**. Without one, a bar at
0.5008 on a two-class task reads as "half as good as 1.0" rather than "did not
learn" — the same manufactured ordering Panel A is forbidden to draw. Figure 7
gained the same line, and it changes how that figure reads: local-assembly
(0.4912) and dense-local (0.5000) are *at chance*, not at half the reference.

## The other three

`fig1_matched_rule_swap` (Figure 5) and `graphical_abstract` shared the same
constants and were silently drawing the superseded block too; both are now on
the re-run values. The graphical abstract states its own scope — it depicts the
**secondary** program, and the manuscript leads with the SHD read-out — and
stacks the two passes rather than ranking them.

`fig3_engine_c1_means` (Figure 7) draws C1 gate values unaffected by the matched
re-run; it changed only by the chance line and the renumbering.

**Two files remain stale and are not fixed here.** `fig2_matched_means`
(Figure 6) plots the superseded block, and `fig4_transfer_ladder` (Figure 8) is
stale at rung 1. **No generator produces either**, so they are authoring work
rather than a re-run, and the spec now says so at each artwork target instead of
leaving the reader to discover it.

## The check that was missing

`scripts/test_paper_figures_match_the_spec.py` — 13 tests — parses the spec's own
Figure 6 table and the generator's `mod nums`, and fails if they disagree. It is
Python rather than a Rust `#[test]` because the generator sits behind
`--features plots`: a test inside it runs only when someone builds with that
feature, while `scripts/run_python_tests.sh` discovers this one on every run of
the evidence gate.

Six breaks, each confirmed to fire:

| break | caught by |
|---|---|
| a drawn value drifts from the sheet | `test_every_drawn_matched_value_is_the_spec_value` |
| a superseded value returns to drawable code | `test_the_superseded_block_is_gone` |
| the low/low cell collapses to one rule | `test_the_low_low_cell_draws_both_rules` |
| Panel A gains a bar row | `test_panel_a_encodes_a_verdict_rather_than_a_magnitude` |
| a verdict chip loses its label | `test_the_verdicts_are_labelled_and_not_colour_alone` |
| the chance line is dropped from Panel B | `test_the_bar_row_can_draw_a_chance_line` |

The superseded-value scan runs over the source with comment lines stripped: the
ban is on **drawing** those numbers, not on naming them in the note that records
why they were replaced. A scan of the raw file would have forbidden the
explanation and left no way to write down what happened.

Two tests guard the parsers themselves — `test_the_spec_table_was_found` and
`test_the_source_constants_were_found` — because a regex that silently matches
nothing would make every other test in the file pass.

## And the venue note, which described a different paper

[`VENUE_FORMATTING.md`](VENUE_FORMATTING.md) is rewritten. It scored **zero**
mentions of attention, SHD, waves, or the difference-in-differences, while
[`PAPER_DRAFT.md`](PAPER_DRAFT.md)'s title is *"What a time-axis read-out buys
is temporal order: a difference-in-differences on SHD"*. It recommended a
negative-results track fitted to the matched ±1 FAIL plus the transfer FAIL —
the right home for the paper it was written about in July, and the wrong home
for a positive preregistered contrast with a destroyed-structure control.

It also ticked `[x] Figure artwork complete (figM, fig1, fig3, graphical
abstract)` against a spec that said figM was stale. The figure table now takes
its statuses from the spec sheet, names the sheet as the owner, and records the
item that box was hiding: **all four lead-program figures have no artwork at
all**, and no lead-program graphical abstract is specified. Four specified
figures with nothing drawn is the largest open authoring task in the package,
and it is on the program the manuscript leads with.

## Files

- `binn-lab/src/paper_figures.rs` — the re-run value block, `Verdict`,
  `rule_card`, the redrawn Figure M, the chance line
- `scripts/test_paper_figures_match_the_spec.py` — 13 tests
- `results/PAPER_FIGURE_SPEC.md` — artwork statuses, and which files have no
  generator
- `results/VENUE_FORMATTING.md` — rewritten around the current manuscript
- `results/runs/2026-07-23-paper-hard-both/figures/` — four regenerated files
