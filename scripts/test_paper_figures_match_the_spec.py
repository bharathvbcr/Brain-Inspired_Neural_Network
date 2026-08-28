"""The figure generator's numbers must be the ones the spec sheet carries.

`binn-lab/src/paper_figures.rs` hardcodes every value it draws, with a header
saying they come from `PAPER_FIGURE_SPEC.md` and `PAPER_RESULTS_TABLE.md` and
are "never remassaged". Nothing checked that. On 2026-08-27 the generator was
still drawing DFA 0.9387, RL 0.9200 and a gradient ceiling of 0.8963 — the exact
block `PAPER_FIGURE_SPEC.md` names as **"superseded and not for drawing"**,
pre-repair figures from a forward pass that emitted zero spikes at any seed.

It had reproduced the committed artwork byte-for-byte the whole time, so the
files in `runs/2026-07-23-paper-hard-both/figures/` were the superseded block
rendered at camera-ready quality, with nothing anywhere to say so.

This file is the missing link. It is Python rather than a Rust `#[test]`
because the generator lives behind `--features plots`: a test inside it runs
only when someone builds with that feature, and `scripts/run_python_tests.sh`
discovers this one on every run of the evidence gate.

Run: python3 scripts/test_paper_figures_match_the_spec.py
"""

from __future__ import annotations

import re
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SOURCE = ROOT / "binn-lab/src/paper_figures.rs"
SPEC = ROOT / "results/PAPER_FIGURE_SPEC.md"

#: The block `PAPER_FIGURE_SPEC.md` records as superseded. None of these may
#: appear in the generator again. `0.5000` is deliberately absent: it is the
#: superseded broadcast value AND the current one, so banning it would ban a
#: number the figure must draw.
SUPERSEDED = ("0.9387", "0.9200", "0.8963", "0.8887", "0.6894", "0.6846")

#: Rust constant -> the series name in the spec's Figure 6 table.
MATCHED = {
    "BROADCAST_PM1": "Broadcast ±1 three-factor",
    "RL_FLAT": "RL ±1 broadcast",
    "RL_FB": "REINFORCE × frozen `B_i`",
    "DFA": "Graded DFA",
    "BROADCAST_GRADED": "Broadcast graded error",
    "CEILING": "SuperSpike BPTT ceiling",
}

def shd_table(name: str, next_name: str) -> str:
    """One `## Table SHD-N` block of `PAPER_RESULTS_TABLE.md`."""
    text = (ROOT / "results/PAPER_RESULTS_TABLE.md").read_text()
    start = text.index(f"## Table {name}")
    return text[start:text.index(f"## Table {next_name}", start)]


def figure_body(name: str, ends_before: str) -> str:
    """One `draw_*` function's source."""
    text = SOURCE.read_text()
    start = text.index(f"fn {name}(")
    return text[start:text.index(ends_before, start)]


def drawable(text: str) -> str:
    """The source with `//` and `///` comment lines removed."""
    return "\n".join(line for line in text.splitlines()
                      if not line.lstrip().startswith("//"))


ROW = re.compile(
    r"^\|\s*(?:\*\*)?([^|*]+?)(?:\*\*)?\s*\|\s*\*{0,2}([01]\.\d{4})\*{0,2}"
    r"\s*\|\s*\*{0,2}([01]\.\d{4})\*{0,2}\s*\|",
    re.M,
)


def spec_table() -> dict[str, tuple[str, str]]:
    """Figure 6's current value block, series -> (feed-forward, recurrent)."""
    text = SPEC.read_text()
    start = text.index("## Figure 6 — Matched means")
    block = text[start:text.index("## Figure 7 —", start)]
    # The superseded block is quoted above the current one in a blockquote so
    # the reader can see what was replaced. Skip it, or this test would accept
    # the very values it exists to refuse.
    current = block[block.index("Current values"):]
    return {name: (ff, rec) for name, ff, rec in ROW.findall(current)}


def source_pairs() -> dict[str, tuple[str, str]]:
    text = SOURCE.read_text()
    nums = text[text.index("mod nums {"):text.index("/// How a cell is encoded")]
    return {n: (a, b) for n, a, b in
            re.findall(r"pub const (\w+): Both = \(([\d.]+), ([\d.]+)\);", nums)}


def source_scalars() -> dict[str, str]:
    text = SOURCE.read_text()
    nums = text[text.index("mod nums {"):text.index("/// How a cell is encoded")]
    return dict(re.findall(r"pub const (\w+): f64 = ([\d.]+);", nums))


class MatchedValuesTest(unittest.TestCase):

    @classmethod
    def setUpClass(cls):
        cls.spec = spec_table()
        cls.src = source_pairs()

    def test_the_spec_table_was_found(self):
        """A parser that silently matches nothing would pass every test below."""
        self.assertGreaterEqual(len(self.spec), 8, self.spec)
        self.assertIn("Graded DFA", self.spec)

    def test_the_source_constants_were_found(self):
        self.assertGreaterEqual(len(self.src), 6, self.src)

    def test_every_drawn_matched_value_is_the_spec_value(self):
        for const, series in MATCHED.items():
            with self.subTest(const=const):
                self.assertIn(const, self.src, f"{const} is gone from the source")
                self.assertIn(series, self.spec, f"{series} is gone from the spec")
                self.assertEqual(self.src[const], self.spec[series],
                                 f"{const} draws {self.src[const]} but the spec "
                                 f"sheet says {series} is {self.spec[series]}")

    def test_the_source_draws_no_matched_value_the_spec_does_not_carry(self):
        """A constant nobody mapped is a number with no sheet behind it."""
        self.assertEqual(set(self.src), set(MATCHED),
                         "a Both constant was added or removed without being "
                         "mapped to a series in the spec's Figure 6 table")

    def test_the_superseded_block_is_gone(self):
        # Comments are stripped first: the ban is on DRAWING those values, not
        # on naming them in the note that records why they were replaced. A
        # scan over the raw file would forbid the explanation and leave no way
        # to write down what happened.
        text = drawable(SOURCE.read_text())
        for value in SUPERSEDED:
            with self.subTest(value=value):
                self.assertNotIn(value, text,
                                 f"{value} is part of the value block the spec "
                                 f"records as superseded and not for drawing")


class ScalarValuesTest(unittest.TestCase):
    """The XOR flip, which Figure M's Panel B and Figure 9 both draw."""

    def test_the_xor_values_are_the_spec_values(self):
        src = source_scalars()
        block = SPEC.read_text()
        block = block[block.index("### Panel B — XOR locality flip"):]
        block = block[:block.index("**Caption")]
        for const, expected in (("XOR_BCAST", "0.5008"), ("XOR_DFA", "0.8267"),
                                ("XOR_GRAD", "0.7733")):
            with self.subTest(const=const):
                self.assertEqual(src[const], expected)
                self.assertIn(expected, block,
                              f"{expected} is no longer in the spec's Panel B")


class LeadFigureTest(unittest.TestCase):
    """Figure 1 of the lead program, against Table SHD-2 and its four bans.

    The manuscript leads with this figure and nothing was drawn for it until
    2026-08-27, so there is no prior artwork to have drifted — what this class
    pins is that it cannot start drifting.
    """

    @classmethod
    def setUpClass(cls):
        text = SOURCE.read_text()
        cls.text = text
        cls.fig = text[text.index("fn draw_lead_fig1("):
                       text.index("/// One rule, drawn by rule")]
        cls.table = shd_table("SHD-2", "SHD-3")

    def test_the_table_was_found(self):
        self.assertIn("bin-shuffle difference-in-differences", self.table)

    def test_every_lead_value_is_in_table_shd_2(self):
        """Each constant the figure draws must appear in the sheet the spec
        cites for it, not merely in the spec's own restatement."""
        expected = {
            "SHUFFLE_COST_ATTN_32": "0.1347", "SHUFFLE_COST_RATE_32": "0.0142",
            "ADVANTAGE_INTACT_32": "0.1275", "ADVANTAGE_SHUFFLED_32": "0.0070",
            "ADVANTAGE_INTACT_12": "0.1258", "ADVANTAGE_SHUFFLED_12": "0.0050",
            "ABS_ATTN_INTACT_12": "0.8320", "ABS_ATTN_SHUFFLED_12": "0.6983",
            "ABS_RATE_INTACT_12": "0.7062", "ABS_RATE_SHUFFLED_12": "0.6934",
            "SHUFFLE_COST_ATTN_12": "0.1337", "SHUFFLE_COST_RATE_12": "0.0128",
            "PER_SEED_MIN_12": "0.0967", "PER_SEED_MAX_12": "0.1568",
        }
        scalars = source_scalars()
        for const, value in expected.items():
            with self.subTest(const=const):
                self.assertEqual(scalars.get(const), value,
                                 f"{const} is not {value}")
                self.assertIn(value, self.table,
                              f"{value} is no longer in Table SHD-2")

    def test_the_inflated_cost_is_never_drawn(self):
        """Ban 3. The wave-17 analyser merged a d32l1 archived shuffled control
        into the d32l4 comparison for twelve pairs and inflated the cost from
        +0.1347 to +0.1577 — MET either way, 17% high."""
        self.assertNotIn("0.1577", drawable(self.text))

    def test_the_prior_art_is_named_in_the_figure(self):
        """Ban 1. Without it the figure invites the reading it is least
        entitled to: that it shows SHD depends on temporal order."""
        for marker in ("NOT SHOWN HERE, AND NOT CLAIMED", "Cramer",
                       "Neuromorphic Sequential Arena", "Yu et al"):
            self.assertIn(marker, self.fig)

    def test_both_arms_are_drawn_at_equal_weight(self):
        """Ban 1, continued: the rate arm is half the measurement, not a faint
        control. Same helper, same bar width, same label sizes."""
        self.assertEqual(self.fig.count("cost_bar("), 2)
        widths = re.findall(r"\(\d+, base, (\d+), height\)", self.fig)
        self.assertEqual(len(set(widths)), 1, f"bar widths differ: {widths}")

    def test_the_shuffle_is_described_as_done_to_the_data(self):
        """Ban 2. Nothing is removed from the model, so no ablation framing."""
        self.assertIn("BOTH the training and test splits", self.fig)
        self.assertIn("Nothing is removed from the model", self.fig)
        for banned in ("attention off", "ablation of", "component axis"):
            self.assertNotIn(banned, self.fig)

    def test_the_smaller_sample_is_drawn_beside_the_larger(self):
        """Ban 4. n = 32 must not read as a rescue: the two are near-identical
        and that near-identity is the message."""
        self.assertIn("ADVANTAGE_INTACT_12", self.fig)
        self.assertIn("ADVANTAGE_INTACT_32", self.fig)
        self.assertIn("did not rescue it", self.fig)

    def test_no_absolute_shuffled_mean_is_drawn_at_n_32(self):
        """Table SHD-2 prints `—` for them: at n = 32 only the costs exist, so
        a figure quoting an absolute shuffled accuracy there would be quoting a
        number nobody published."""
        self.assertNotIn("ABS_ATTN_SHUFFLED_32", self.text)
        self.assertNotIn("ABS_RATE_SHUFFLED_32", self.text)

    def test_the_figure_is_a_new_file(self):
        """The spec: "A new file must be produced; do not repoint an existing
        fig* file at this spec"."""
        self.assertIn('"leadfig1_the_conditional"', self.text)
        self.assertIn("leadfig1_the_conditional", SPEC.read_text())


class LeadFigure2Test(unittest.TestCase):
    """Headline accuracy, against Table SHD-1 / SHD-5 and its four bans."""

    @classmethod
    def setUpClass(cls):
        cls.text = SOURCE.read_text()
        cls.fig = figure_body("draw_lead_fig2", "/// Figure 3 of the lead program")
        cls.shd1 = shd_table("SHD-1", "SHD-2")
        cls.shd5 = shd_table("SHD-5", "SHD-6")

    def test_the_headline_values_are_table_shd_1(self):
        scalars = source_scalars()
        for const, value in (("HEAD_RATE_32", "0.7057"), ("HEAD_ATTN_32", "0.8332"),
                             ("HEAD_GAIN_32", "0.1275"), ("HEAD_RATE_12", "0.7062"),
                             ("HEAD_ATTN_12", "0.8320"), ("HEAD_GAIN_12", "0.1258")):
            with self.subTest(const=const):
                self.assertEqual(scalars.get(const), value)
                self.assertIn(value, self.shd1)

    def test_the_geometry_ladder_is_table_shd_5(self):
        geom = re.search(r"pub const GEOM: \[.*?\] = \[(.*?)\];",
                         self.text, re.S).group(1)
        drawn = re.findall(r"(0\.\d{4})", geom)
        self.assertEqual(len(drawn), 9, drawn)
        for value in drawn:
            with self.subTest(value=value):
                self.assertIn(value, self.shd5,
                              f"{value} is drawn but is not in Table SHD-5")

    def test_the_axis_shows_where_the_frontier_is(self):
        """Ban 1: an axis starting at 0.65, or one without the frontier marker,
        makes 0.8332 read as a win. One axis, 0.50 to 1.00, both on it."""
        self.assertIn("let (lo, hi) = (0.50, 1.00);", self.fig)
        self.assertIn("FIELD_FRONTIER_LO", self.fig)
        self.assertIn("FIELD_FRONTIER_HI", self.fig)
        self.assertIn("NOT COMPETITIVE", self.fig)

    def test_the_unresolvable_band_is_drawn_to_scale(self):
        """Ban 2: differences below ~1.5 points between published SHD numbers
        are not reliably meaningful, this paper's own included."""
        self.assertIn("FIELD_UNRESOLVABLE", self.fig)
        self.assertIn("not reliably meaningful", self.fig)
        self.assertEqual(source_scalars().get("FIELD_UNRESOLVABLE"), "0.015")

    def test_the_excluded_comparison_numbers_have_no_constants(self):
        """Ban 3, enforced by absence rather than by care: Pfa-SNN 96.26,
        Event-SSMA 95.90, SpikeSCR 95.60 and d-cAdLIF 94.85 came from a
        secondary comparison table and are excluded from the paper's claims.
        With no constant for them they cannot be plotted by accident."""
        nums_block = self.text[self.text.index("mod nums {"):
                               self.text.index("/// How a cell is encoded")]
        for value in ("0.9626", "0.9590", "0.9560", "0.9485"):
            self.assertNotIn(value, nums_block)

    def test_the_literature_strip_says_it_is_unverified(self):
        """Ban 4: every value in Panel B came from a search pass, is not
        machine-checked against cells, and check_every_number.py does not sweep
        the section it lives in."""
        self.assertIn("NOT MACHINE-CHECKED", self.fig)
        self.assertIn("before submission", self.fig)


class LeadFigure3Test(unittest.TestCase):
    """The width ladder, against Table SHD-3 / SHD-4 and its five bans."""

    @classmethod
    def setUpClass(cls):
        cls.text = SOURCE.read_text()
        cls.fig = figure_body("draw_lead_fig3", "/// Figure 4 of the lead program")
        cls.shd3 = shd_table("SHD-3", "SHD-4")
        cls.shd4 = shd_table("SHD-4", "SHD-5")

    def test_every_rung_is_in_table_shd_3(self):
        rungs = re.search(r"pub const LADDER: \[.*?\] = \[(.*?)\];",
                          self.text, re.S).group(1)
        for value in re.findall(r"(-?0\.\d{4})", rungs):
            with self.subTest(value=value):
                self.assertIn(value.lstrip("-"), self.shd3)

    def test_every_lever_is_in_table_shd_4(self):
        levers = re.search(r"pub const LEVERS: \[.*?\] = \[(.*?)\];",
                           self.text, re.S).group(1)
        for value in re.findall(r"(-?\d+\.\d+)", levers):
            with self.subTest(value=value):
                self.assertIn(value.lstrip("-"), self.shd4 + self.shd3)

    def test_no_curve_is_fitted_through_the_rungs(self):
        """Ban 1. H16-1 is NOT MET; a connector through the first five rungs
        asserts an ordering the measurement cannot support."""
        rung_loop = self.fig[self.fig.index("for (i, (name, _rate"):]
        self.assertNotIn("PathElement", rung_loop.split("// h384")[0])

    def test_the_indistinguishable_rungs_are_marked_as_such(self):
        """Ban 1 and 2 together: h384 and h512 are not distinguishable at n=12,
        and the figure must not manufacture a dip at h384."""
        self.assertIn("not distinguishable at n = 12", self.fig)
        self.assertIn("LADDER_H384_H512", self.fig)
        self.assertIn("LADDER_H384_H512_SD", self.fig)

    def test_the_step_is_placed_between_h768_and_h1024(self):
        """Ban 3: the four-rung reading placing it below h512 is superseded."""
        self.assertIn("between h768 and h1024", self.fig)
        self.assertIn("(at_x(4) + at_x(5)) / 2", self.fig)

    def test_no_mechanism_is_offered(self):
        """Ban 4: the levers all failed and the gradient-norm correlate is a
        correlate. "Gradient pathology" would claim what H15-1 refuted."""
        self.assertIn("LOCATED BUT UNEXPLAINED", self.fig)
        self.assertIn("correlate, not a cause", self.fig)
        self.assertIn("overfitting on 8,156 training samples is not excluded",
                      self.fig)
        for banned in ("gradient pathology", "gradient explosion", "scaling law"):
            self.assertNotIn(banned, self.fig.lower())

    def test_the_h1024_depth_result_is_absent(self):
        """Ban 5: d32/L2 at h1024 reaching +0.0392 rests on three points with
        L3 missing and is registered as its own wave. Keep it off entirely."""
        # Comments stripped: the ban is on DRAWING it, and the reason it is
        # banned has to be writable down beside the code that obeys the ban.
        self.assertNotIn("0.0392", drawable(self.text))

    def test_the_width_axis_is_evenly_spaced(self):
        """A log-width axis visually compresses the step, which is the one
        feature this figure exists for."""
        self.assertIn("x0 + (i as i32) * step_x", self.fig)
        self.assertNotIn("ln()", self.fig)
        self.assertNotIn("log2", self.fig)


class LeadFigure4Test(unittest.TestCase):
    """The resolution ladder, against Table SHD-6 and its three bans."""

    @classmethod
    def setUpClass(cls):
        cls.text = SOURCE.read_text()
        cls.fig = figure_body("draw_lead_fig4", "/// One rule, drawn by rule")
        cls.shd6 = shd_table("SHD-6", "SHD-7")

    def test_every_rung_is_in_table_shd_6(self):
        rungs = re.search(r"pub const RESOLUTION: \[.*?\] = \[(.*?)\];",
                          self.text, re.S).group(1)
        for value in re.findall(r"(0\.\d{4})", rungs):
            with self.subTest(value=value):
                self.assertIn(value, self.shd6)

    def test_only_the_fixed_family_is_plotted(self):
        """Ban 1: published-Nms moves bin width and sequence length together,
        so no number from it can be attributed to either."""
        self.assertIn("fixed-t100", self.text)
        self.assertNotIn("published-2ms", self.fig)
        self.assertNotIn("published-10ms", self.fig)

    def test_the_fixed_window_is_stated_on_the_figure(self):
        """It is the whole reason the axis means anything."""
        self.assertIn("1400 ms analysis window is HELD FIXED", self.fig)

    def test_both_series_are_drawn(self):
        """Ban 2: without the baseline the falling gain reads as the attention
        arm degrading rather than as the rate arm catching up."""
        self.assertIn('("attention", attn, 1usize)', self.fig)
        self.assertIn('("rate", rate, 0usize)', self.fig)
        self.assertIn("RESOLUTION_BASELINE_DRIFT", self.fig)
        self.assertIn("confound bar", self.fig)

    def test_no_mechanism_or_preference_is_offered(self):
        """Ban 3. "Attention prefers coarse bins" is an interpretation the
        evidence does not carry."""
        self.assertIn("NO MECHANISM AND NO PREFERENCE", self.fig)
        self.assertNotIn("optimal resolution", self.fig.lower())


class PanelAEncodingTest(unittest.TestCase):
    """The two things the spec says Panel A must not be allowed to say."""

    @classmethod
    def setUpClass(cls):
        text = SOURCE.read_text()
        cls.fig_m = text[text.index("fn draw_fig_m("):text.index("fn draw_fig1(")]
        cls.text = text

    def test_the_low_low_cell_draws_both_rules(self):
        """`MatchedLocal` is at chance and `MatchedRlFlat` reaches 0.78. The
        spec: collapsing them into one "broadcast ±1" cell is a stronger version
        of the overreach the lead claim's wording exists to avoid."""
        self.assertIn("nums::BROADCAST_PM1", self.fig_m)
        self.assertIn("nums::RL_FLAT", self.fig_m)

    def test_panel_a_encodes_a_verdict_rather_than_a_magnitude(self):
        """With the reference at 1.0000 every pass reduces to "above 0.75", so
        any accuracy-to-size mapping manufactures an ordering the task cannot
        support. Panel A draws cards with verdict chips; the one bar row in this
        figure is Panel B, which is a different task."""
        self.assertEqual(self.fig_m.count("draw_bar_row("), 1,
                         "Panel A must not gain a bar row")
        self.assertGreaterEqual(self.fig_m.count("rule_card("), 4)

    def test_the_verdicts_are_labelled_and_not_colour_alone(self):
        """A categorical encoding carried only by hue does not survive a
        greyscale print, which is how a reviewer will often see it."""
        self.assertIn('Verdict::Pass => "PASS"', self.text)
        self.assertIn('Verdict::Fail => "FAIL"', self.text)
        self.assertIn("verdict.tag()", self.text)

    def test_the_saturated_reference_is_stated_in_the_figure(self):
        """The spec makes the ceiling the point of the panel rather than an
        aside: it is why the passing arms cannot be ranked."""
        self.assertIn("nums::CEILING", self.fig_m)
        self.assertIn("does not rank", self.fig_m)

    def test_the_bar_row_can_draw_a_chance_line(self):
        """A bar at 0.5008 on a two-class task reads as "half as good" without
        one — the same manufactured ordering Panel A is forbidden to draw."""
        self.assertIn("chance: Option<f64>", self.text)
        self.assertIn("Some(0.5)", self.fig_m)


class TheManuscriptReachesTheFiguresTest(unittest.TestCase):
    """A figure nothing cites does not appear in a submission.

    On 2026-08-27, after all four lead figures were drawn,
    `PAPER_DRAFT.md` referenced **Figure M four times and no other figure at
    all**. The artwork existed, the spec named where each figure belonged, and
    the manuscript pointed at none of them — which is the same failure as
    artwork that does not exist, arriving one step later.

    Scoped to the lead program and Figure M. The secondary program's figures
    are numbered 5–9 in the spec and the draft does not call them out; that is
    a live authoring gap, not something this file should assert away.
    """

    DRAFT = ROOT / "results/PAPER_DRAFT.md"

    @classmethod
    def setUpClass(cls):
        cls.text = cls.DRAFT.read_text()

    def test_each_lead_figure_is_cited(self):
        for n in (1, 2, 3, 4):
            with self.subTest(figure=n):
                self.assertRegex(
                    self.text, rf"\(Figure {n}\)",
                    f"Figure {n} is drawn but the manuscript never refers to it")

    def test_figure_m_is_cited(self):
        self.assertIn("Figure M", self.text)

    def test_no_lead_figure_is_cited_more_than_its_home(self):
        """Each is placed once, at the claim a reader meets it on. Repeating a
        callout in the abstract and again in the discussion is a layout
        decision, not something to accrue by accident."""
        for n in (1, 2, 3, 4):
            with self.subTest(figure=n):
                self.assertEqual(len(re.findall(rf"\(Figure {n}\)", self.text)), 1)

    def test_every_lead_figure_the_generator_writes_has_a_number(self):
        """The map from stem to manuscript number, asserted rather than assumed:
        a fifth lead stem added without a callout would otherwise pass."""
        source = SOURCE.read_text()
        stems = set(re.findall(r'"(leadfig\d+_\w+)"', source))
        self.assertEqual(
            stems,
            {"leadfig1_the_conditional", "leadfig2_headline_accuracy",
             "leadfig3_width_ladder", "leadfig4_resolution_ladder"},
            "a lead figure was added or removed; give it a manuscript callout "
            "and list it here")


class TheGeneratorOwnsTheArtworkTest(unittest.TestCase):
    """The committed files and the generator's stem list must not drift apart."""

    STEMS = ("leadfig1_the_conditional", "leadfig2_headline_accuracy",
             "leadfig3_width_ladder", "leadfig4_resolution_ladder",
             "figM_mechanism_richness_addressability", "fig1_matched_rule_swap",
             "fig3_engine_c1_means", "graphical_abstract")
    FIGURES = ROOT / "results/runs/2026-07-23-paper-hard-both/figures"

    def test_each_generated_stem_exists_on_disk(self):
        text = SOURCE.read_text()
        for stem in self.STEMS:
            with self.subTest(stem=stem):
                self.assertIn(f'"{stem}"', text)
                for ext in ("png", "pdf"):
                    self.assertTrue((self.FIGURES / f"{stem}.{ext}").is_file(),
                                    f"{stem}.{ext} is missing")

    def test_the_spec_names_the_same_artwork_paths(self):
        spec = SPEC.read_text()
        for stem in self.STEMS:
            with self.subTest(stem=stem):
                self.assertIn(stem, spec,
                              f"{stem} is generated but the spec names no "
                              f"artwork target for it")


if __name__ == "__main__":
    unittest.main(verbosity=2)
