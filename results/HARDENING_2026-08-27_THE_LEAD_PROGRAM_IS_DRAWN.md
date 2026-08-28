# All four lead-program figures are drawn

**2026-08-27**, continuing
[the first lead figure](HARDENING_2026-08-27_THE_LEAD_CLAIM_HAS_A_FIGURE.md).
`leadfig2_headline_accuracy`, `leadfig3_width_ladder` and
`leadfig4_resolution_ladder` join it. The program the manuscript leads with went
from nine secondary-program files and no artwork of its own to a complete set.

## Figure 2 — headline accuracy, and that it is not competitive

The spec's first ban is the whole construction: *"Drawing the panel with an axis
that starts at 0.65, or omitting the frontier marker, would make the bar read as
a win."*

So there is exactly **one** accuracy axis in the figure, it runs 0.50 to 1.00,
and the instrument's markers and the published field sit on it together. 0.8332
cannot look like a frontier number on an axis that shows the frontier at
95–96.4%.

Two constructions came out of drawing it rather than out of the spec:

- **The no-delay recurrent baseline is a band, not a marker.** Cramer et al.'s
  83.2 ± 1.3% and this instrument's 0.8332 are the same number, so drawn as two
  markers they collided. Drawn as a band with the marker inside it, the
  collision *is* the reading: an architecture carrying no temporal kernel lands
  where the dataset authors' own no-delay baseline lands. The first render had
  the band painted over the marker, hiding the one point on the axis that must
  be visible; bands are now drawn before anything else.
- **The ~1.5-point unresolvable difference is drawn to scale on that axis**, not
  only asserted in words. SHD ships no validation set — Baronig et al. report one
  model at 95.81 validating on test and 93.79 on a held-out split — and the band
  makes visible how much of the published field's ordering that swallows,
  including this paper's own comparisons.

The four excluded comparison numbers (Pfa-SNN 96.26, Event-SSMA 95.90, SpikeSCR
95.60, d-cAdLIF 94.85) have **no constant in the generator at all**, so they
cannot be plotted by accident rather than merely being left out by care. A test
asserts the `nums` block contains none of them.

## Figure 3 — the width ladder

Five of this figure's bans are about lines that are not there. H16-1 — "the gain
decays monotonically with width up to the collapse" — is **NOT MET**:
seed-paired, gain(h384) − gain(h512) is **−0.0116, sd 0.0253, negative in only 7
of 12 seeds**. So:

- the rungs are **points with no connector of any kind** through them, and a
  test asserts the rung-drawing loop emits no `PathElement`;
- h384–h512 carries an explicit bracket reading *"not distinguishable at
  n = 12"*, because a reader who sees +0.0760 then +0.0876 will otherwise read a
  dip the registration manufactured by demanding 0.005 separations over
  quantities inside their own noise floor;
- the step glyph sits **between h768 and h1024** — the four-rung reading placing
  it lower is superseded, and h768 is still +0.0560;
- the width axis is **evenly spaced, not log**, because a log axis compresses the
  step this figure exists for;
- the h1024 `d32/L2` result has no constant in the generator.

The panel's own words are **"LOCATED BUT UNEXPLAINED"**, and the figure says in
its own text that gradient norms leaving O(1) are a correlate rather than a
cause and that overfitting on 8,156 training samples is not excluded by anything
in this paper. A test refuses the strings "gradient pathology", "gradient
explosion" and "scaling law".

## Figure 4 — the resolution ladder

Both series are drawn, and that is a ban rather than a preference: the rate arm
rises **+0.0397** across the same ladder, inside a preregistered 0.05 confound
bar, and without it the falling gain reads as the attention arm degrading rather
than as the rate arm catching up. The fixed **1400 ms** analysis window is in
the subtitle because it is the whole reason the axis means anything — the
`published-Nms` family moves bin width and sequence length together, so no
number from it can be attributed to either, and a test asserts no
`published-Nms` value appears in this figure. The figure carries **"NO MECHANISM
AND NO PREFERENCE"**.

## What binds them

`scripts/test_paper_figures_match_the_spec.py` is now **41 tests**. Each lead
figure's values are checked against the `Table SHD-N` of
[`PAPER_RESULTS_TABLE.md`](PAPER_RESULTS_TABLE.md) that its spec section cites —
the sheet itself, never the spec's restatement of it, so the two cannot drift
together — and each of the twelve bans across the three figures was broken and
confirmed to fire:

| break | figure |
|---|---|
| the axis starts at 0.65 | 2 |
| an excluded comparison number gains a constant | 2 |
| the unverified-literature warning is dropped | 2 |
| the indistinguishable bracket is removed | 3 |
| the step moves below h512 | 3 |
| a mechanism is offered | 3 |
| the `d32/L2` result is drawn | 3 |
| the baseline series is dropped | 4 |
| `published-Nms` is plotted | 4 |
| a rung drifts from Table SHD-3 | 3 |

Two of my own test bugs surfaced while writing them, both the shape I have been
fixing all day: a scan for a banned value ran over the raw source and flagged the
comment explaining *why* it is banned, and a table check asserted against an
expression that was true by construction. Both now scan drawable code and the
actual constant block.

## The figures existed and nothing pointed at them

Drawn, checked, committed — and `PAPER_DRAFT.md` referenced **Figure M four
times and no other figure at all.** All four lead figures were unreachable from
the manuscript, which is the same failure as artwork that does not exist,
arriving one step later.

Each is now called out once, at the claim a reader meets it on: §3.5 item 1 →
Figure 2, item 2 → Figure 1, item 4 → Figure 3, item 5 → Figure 4. That mapping
is the spec's own — its `Draft cite:` lines already name those sections — so the
placement was executed rather than invented, and one callout per figure rather
than one at every location those lines mention, because a `Draft cite` line is a
provenance map for the author, not an instruction to repeat the callout in the
abstract and the discussion too.

**No number was touched**, and the sweep confirms it: 72 derived from cells,
7 named in `ELSEWHERE`, 40 traced to a named record, unchanged either side of
the edit.

Four tests hold it, all negative-tested: a figure losing its callout, a callout
duplicating, and a fifth lead stem being added to the generator without one.
The invariant is deliberately scoped to the lead program and Figure M — the
secondary program's figures are numbered 5–9 and the draft calls out none of
them, which is a live authoring gap and not something a test should assert away.

## What is left

A **lead-program graphical abstract**, which the spec records as
`TODO(source needed)`, and callouts for the secondary program's Figures 5–9.
The graphical abstract is the only piece of figure work in the package with no
specification at all; everything else either exists or is named as authoring
work with its reason.

## Files

- `binn-lab/src/paper_figures.rs` — `draw_lead_fig2`, `draw_lead_fig3`,
  `draw_lead_fig4`, and their Table SHD-1/3/4/5/6 constants
- `scripts/test_paper_figures_match_the_spec.py` — 41 tests
- `results/PAPER_FIGURE_SPEC.md` — three artwork targets, and the status banner
- `results/VENUE_FORMATTING.md` — all four lead figures drawn
- `results/runs/2026-07-23-paper-hard-both/figures/leadfig{2,3,4}_*.{png,pdf}`
