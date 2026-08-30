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
    "EVENTPROP": "Discrete EventProp-style spike-adjoint",
    "RL_GRADED": "RL graded-reward broadcast",
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


def did_ladder() -> list[tuple[str, str, int, int, bool]]:
    """`nums::DID_LADDER` as (point, DiD, positive, quadruples, gain_negative)."""
    text = SOURCE.read_text()
    block = text[text.index("pub const DID_LADDER"):]
    block = block[:block.index("];")]
    return [(m[0], m[1], int(m[2]), int(m[3]), m[4] == "true")
            for m in re.findall(
                r'\("([^"]+)",\s*([\d.]+),\s*(\d+),\s*(\d+),\s*(true|false)\)', block)]


class LeadFigure1PanelDTest(unittest.TestCase):
    """Figure 1 Panel D — wave 21's eight-point difference-in-differences.

    Every ban the spec puts on this panel is a constraint on how the points are
    ARRANGED, not on which are drawn, because the arrangement is what can lie:
    the wave measured that the effect's size does NOT track the gain, and the
    obvious renderings (sorted bars, a connecting line, a shared axis) all
    assert the relationship the rank correlation rejects.
    """

    @classmethod
    def setUpClass(cls):
        text = SOURCE.read_text()
        cls.text = text
        cls.fig = text[text.index("fn draw_lead_fig1("):
                       text.index("/// Figure 2 of the lead program")]
        cls.panel = cls.fig[cls.fig.index("// --- Panel D ---"):]
        cls.ladder = did_ladder()
        cls.table = shd_table("SHD-2", "SHD-3")

    def test_the_ladder_was_parsed(self):
        """A parser matching nothing would pass every test below."""
        self.assertEqual(len(self.ladder), 8, self.ladder)

    def test_every_point_is_in_table_shd_2b(self):
        """Table SHD-2b is inside the SHD-2 slice, which runs to SHD-3."""
        self.assertIn("Table SHD-2b", self.table)
        for point, did, positive, of, _ in self.ladder:
            with self.subTest(point=point):
                self.assertIn(did, self.table, f"{point}: {did} is not in the sheet")
                self.assertIn(f"{positive}/{of}", self.table)

    def test_ban_1_the_points_are_in_ladder_order_not_sorted_by_effect(self):
        """Sorting by DiD manufactures the trend H21-3 refuted."""
        widths = [p for p, *_ in self.ladder if p.startswith("h") and "/" not in p]
        self.assertEqual(widths, ["h128", "h256", "h384", "h512", "h768", "h1024"])
        values = [float(d) for _, d, *_ in self.ladder]
        self.assertNotEqual(values, sorted(values), "points are sorted by DiD")
        self.assertNotEqual(values, sorted(values, reverse=True),
                            "points are reverse-sorted by DiD")

    def test_ban_2_no_connector_is_drawn_between_the_points(self):
        """rho = -0.1430 against +0.829. A line asserts a NOT MET result.

        The panel draws gridlines and the registered bar with PathElement, so
        the check is that no path is built from the ladder itself.
        """
        body = drawable(self.panel)
        for line in body.splitlines():
            if "PathElement" in line:
                self.assertNotIn("DID_LADDER", line)
        self.assertNotIn("did_points", body)

    def test_ban_2_the_gain_is_not_drawn_on_this_axis(self):
        """One quantity per panel. LADDER is Figure 3's series, not this one."""
        self.assertNotIn("nums::LADDER", drawable(self.panel))

    def test_ban_3_the_negative_gain_arm_is_marked(self):
        """A DiD-only column reads as 'healthy at every width'. h1024 is not."""
        marked = [p for p, _, _, _, neg in self.ladder if neg]
        self.assertEqual(marked, ["h1024"], self.ladder)
        self.assertIn("gain is NEGATIVE here", self.panel)

    def test_ban_4_coverage_is_stated_as_nine_of_twenty_one(self):
        """Twelve operating points still carry no bin-shuffled twin."""
        scalars = re.findall(r"pub const COVERAGE_(\w+): u32 = (\d+);", self.text)
        self.assertEqual(dict(scalars), {"COVERED": "9", "TOTAL": "21"})
        self.assertIn("NOT every width", self.panel)

    def test_the_refutation_is_on_the_panel_not_only_in_the_caption(self):
        """H21-3 is NOT MET and the panel has to say so where it is read."""
        self.assertIn("is not the gain", self.panel)
        self.assertIn("NOT connected", self.panel)


class LeadFigure3AnnotationTest(unittest.TestCase):
    """Figure 3's required 2026-08-29 annotation, and its sixth ban."""

    @classmethod
    def setUpClass(cls):
        text = SOURCE.read_text()
        cls.text = text
        cls.fig = text[text.index("fn draw_lead_fig3("):
                       text.index("fn draw_lead_fig4(")]

    def test_the_annotation_is_present(self):
        self.assertIn("THE MECHANISM DOES NOT TRACK THIS CURVE", self.fig)

    def test_it_names_the_statistic_and_that_it_is_not_met(self):
        self.assertIn("DID_RHO", self.fig)
        self.assertIn("DID_RHO_BAR", self.fig)
        self.assertIn("NOT MET", self.fig)

    def test_it_names_h768_as_the_clearest_case(self):
        """Smallest positive gain on the ladder, largest DiD in the campaign."""
        self.assertIn("GAIN_H768", self.fig)
        self.assertIn("DID_H768", self.fig)

    def test_ban_6_the_did_ladder_is_not_a_second_series_here(self):
        """Superimposing two uncorrelated quantities invites the eye to find
        the relationship the statistic rejects. One quantity per figure."""
        self.assertNotIn("DID_LADDER", self.fig)

    def test_it_says_where_the_did_ladder_actually_is(self):
        """Naming the ban without naming the alternative reads as an omission."""
        self.assertIn("Figure 1 Panel D", self.fig)


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

    def test_the_ladder_declares_its_read_out_depth(self):
        """Ban 5, as rewritten 2026-08-28. Every rung is d32/L4, and wave 18
        showed the inversion is a property of that depth rather than of the
        width: at h1024, d32/L2 gains +0.0405 in 20/20. A figure that says
        "gain inverts at h1024" without naming the depth states something
        broader than the measurement."""
        self.assertIn("EVERY RUNG IS d32/L4", self.fig)
        self.assertIn("not of the width", self.fig)

    def test_the_depth_ladder_itself_is_still_not_plotted(self):
        """The same ban's surviving half: one depth per figure. A width ladder
        with a second axis is a different figure and none is specified."""
        self.assertNotIn("LADDER_L2", self.text)
        # Word-boundary: LADDER_DROP and LADDER_LARGEST_GAP_BELOW are scalars
        # about the step, not a second series. Counting the bare prefix caught
        # them and made this assert 5 == 1.
        self.assertEqual(len(re.findall(r"nums::LADDER\b", self.fig)), 1)

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


def figure_s_table() -> str:
    """Table SHD-7, which is the last SHD table before the matched program."""
    text = (ROOT / "results/PAPER_RESULTS_TABLE.md").read_text()
    start = text.index("## Table SHD-7")
    return text[start:text.index("# The matched-gate program", start)]


def tuples(const: str, arity: int) -> list[tuple[str, ...]]:
    """A fixed-arity `[( ... ); N]` constant, as strings."""
    text = SOURCE.read_text()
    block = text[text.index(f"pub const {const}"):]
    block = block[:block.index("];")]
    field = r'"([^"]*)"|(-?[\d.]+)'
    rows = []
    for line in block.splitlines():
        line = line.strip()
        if not line.startswith("("):
            continue
        found = [a or b for a, b in re.findall(field, line)]
        if len(found) == arity:
            rows.append(tuple(found))
    return rows


class FigureSTest(unittest.TestCase):
    """The substrate panel, against Table SHD-7 and its four bans.

    §3.7 of the draft is a lead-program section with three waves behind it and
    **no figure was specified in any sheet**. The omission was hidden behind a
    wrong label: `PAPER_SKELETON.md`'s figure map called this the "Fig. 4
    substrate panel", and Figure 4 is the resolution ladder, so the map looked
    complete while naming something that did not exist.
    """

    @classmethod
    def setUpClass(cls):
        cls.text = SOURCE.read_text()
        cls.fig = figure_body("draw_fig_s_substrate",
                              "/// The lead program's graphical abstract")
        cls.table = figure_s_table()

    def test_the_table_was_found(self):
        self.assertIn("Substrate comparison", self.table)

    def test_the_rows_were_parsed(self):
        """A parser matching nothing would pass every test below."""
        self.assertEqual(len(tuples("SUBSTRATE", 7)), 4, tuples("SUBSTRATE", 7))
        self.assertEqual(len(tuples("USABILITY", 5)), 4, tuples("USABILITY", 5))

    def test_every_substrate_row_is_in_table_shd_7(self):
        for row in tuples("SUBSTRATE", 7):
            substrate, _note, _scale, pairs, rate, attn, gain = row
            with self.subTest(substrate=f"{substrate}/{_scale}"):
                for value in (rate, attn, gain):
                    self.assertIn(value, self.table,
                                  f"{substrate}: {value} is not in Table SHD-7")
                self.assertIn(f"| {pairs} |", self.table.replace("**", ""),
                              f"{substrate}: n = {pairs} is not in Table SHD-7")

    def test_every_contrast_and_headroom_value_is_in_table_shd_7(self):
        scalars = source_scalars()
        for const in ("SUBSTRATE_A1", "SUBSTRATE_M2", "SUBSTRATE_M4",
                      "HEADROOM_REC", "HEADROOM_FF", "HEADROOM_RATIO_REC",
                      "HEADROOM_RATIO_FF", "NORMALISED_RATIO",
                      "SATURATED_FRACTION_MAX"):
            with self.subTest(const=const):
                self.assertIn(scalars[const], self.table,
                              f"{const} = {scalars[const]} is not in Table SHD-7")

    def test_every_usability_row_is_in_table_shd_7(self):
        for arm, scale, completed, voided, diverged in tuples("USABILITY", 5):
            with self.subTest(arm=f"{arm}/{scale}"):
                self.assertIn(f"{completed}/12", self.table.replace("**", ""),
                              f"{arm} at {scale}: {completed}/12 is not in the sheet")
                self.assertIn(f"| {voided} | {diverged} |",
                              self.table.replace("**", ""),
                              f"{arm} at {scale}: {voided}/{diverged} is not in the sheet")

    def test_ban_1_the_recurrent_substrate_is_not_drawn_as_a_win(self):
        """rec+alif+attn 0.7874 against ff+fixed+attn 0.8289 at the same scale,
        and the paper issues NO VERDICT on that ordering."""
        self.assertIn("THE RECURRENT SUBSTRATE DOES NOT WIN", self.fig)
        self.assertIn("NO VERDICT IS ISSUED ON THAT ORDERING", self.fig)
        # The marker is placed at the feed-forward attention arm so the
        # recurrent one is visibly below it, and it comes from the array rather
        # than being written down.
        self.assertIn("let ff_attn = nums::SUBSTRATE[3].5;", self.fig)
        self.assertIn("let rec_attn = nums::SUBSTRATE[2].5;", self.fig)

    def test_ban_1_the_rows_are_in_sheet_order_not_sorted_by_gain(self):
        gains = [float(r[6]) for r in tuples("SUBSTRATE", 7)]
        self.assertNotEqual(gains, sorted(gains), "rows are sorted by gain")
        self.assertNotEqual(gains, sorted(gains, reverse=True),
                            "rows are reverse-sorted by gain")
        self.assertEqual([r[0] for r in tuples("SUBSTRATE", 7)],
                         ["ff+fixed", "ff+alif", "rec+alif", "ff+fixed"])
        self.assertIn("NOT sorted by gain", self.fig)

    def test_ban_1_nothing_is_joined_between_rows(self):
        """The comparison this figure makes is horizontal, within a row. A
        connector across rows would draw the substrate ordering."""
        for line in drawable(self.fig).splitlines():
            if "PathElement" in line:
                self.assertNotIn("SUBSTRATE", line)

    def test_ban_2_both_readings_of_the_doubling_are_drawn(self):
        """The recurrent gain is measured from a base 0.18 lower. Neither the
        raw ratio nor the normalised one may be quoted alone."""
        self.assertIn("BOTH READINGS", self.fig)
        for const in ("RAW_RATIO", "NORMALISED_RATIO", "HEADROOM_REC",
                      "HEADROOM_FF"):
            self.assertIn(f"nums::{const}", self.fig)

    def test_ban_2_the_normalisation_is_labelled_post_hoc(self):
        self.assertIn("POST-HOC AND NOT REGISTERED", self.fig)
        self.assertIn("not licence to prefer whichever", self.fig)

    def test_ban_3_the_ten_pairs_are_on_the_figure(self):
        """The registered minimum, and one further loss on either arm would
        have made M-2 unreportable."""
        self.assertIn("TEN PAIRS", self.fig)
        self.assertIn("THE REGISTERED MINIMUM", self.fig)
        # The count is interpolated from the row it describes, not written into
        # the sentence: "TEN" beside an array saying 9 would disagree silently.
        self.assertIn("nums::SUBSTRATE[2].3", self.fig)
        self.assertEqual(tuples("SUBSTRATE", 7)[2][3], "10")
        self.assertIn("unreportable", self.fig)
        self.assertIn("divergence is not random", self.fig)
        self.assertIn("n = {pairs} pairs", self.fig, "n is not printed per row")

    def test_ban_4_adaptation_is_inert_at_this_operating_point_only(self):
        """Panel C is why: on the recurrent substrate adaptation is what
        prevents saturation, so dropping it turns a scoped null into a general
        one."""
        self.assertIn("Panel C", self.fig)
        self.assertIn("nums::USABILITY", self.fig)
        self.assertIn("ADAPTATION IS STABILISING", self.fig)
        self.assertIn("inert AT THIS OPERATING POINT", self.fig)
        for banned in ("adaptation is unnecessary", "adaptation does nothing",
                       "recurrence is better"):
            self.assertNotIn(banned, self.fig.lower())

    def test_it_is_lettered_rather_than_numbered(self):
        """A fifth lead figure would renumber the secondary program 5-9 -> 6-10
        one day after the 2026-08-27 renumber, for one figure."""
        self.assertIn('"figS_substrate"', self.text)
        spec = SPEC.read_text()
        self.assertIn("## Figure S — Substrate", spec)
        self.assertNotIn("## Figure 10", spec)

    def test_the_manuscript_cites_it(self):
        draft = (ROOT / "results/PAPER_DRAFT.md").read_text()
        self.assertEqual(len(re.findall(r"\(Figure S\)", draft)), 1)


class LeadGraphicalAbstractTest(unittest.TestCase):
    """The lead program's graphical abstract, against its four bans.

    It was `TODO(source needed)` from 2026-08-27 to 2026-08-29 and correctly
    recorded as an **authoring** task, not a missing number: every quantity it
    draws was already published. What it needed was a decision about what the
    paper's front image says, and the point of this class is that the decision
    can now be broken by an edit rather than only disagreed with.

    Bans 1-3 are Figure 1's, unchanged. An abstract is a compression of the
    figure, not a licence to say something the figure may not.
    """

    @classmethod
    def setUpClass(cls):
        cls.text = SOURCE.read_text()
        cls.fig = figure_body("draw_lead_graphical_abstract",
                              "/// Figure 6 of the secondary program")
        cls.shd1 = shd_table("SHD-1", "SHD-2")
        cls.shd2 = shd_table("SHD-2", "SHD-3")

    def test_every_value_is_in_the_sheet(self):
        """Each against the Table the spec cites for it, not against Figure 1's
        restatement of the same numbers."""
        scalars = source_scalars()
        expected = {
            "SHUFFLE_COST_ATTN_32": self.shd2,
            "SHUFFLE_COST_RATE_32": self.shd2,
            "ADVANTAGE_INTACT_32": self.shd2,
            "ADVANTAGE_SHUFFLED_32": self.shd2,
            "HEAD_ATTN_32": self.shd1,
            "HEAD_RATE_32": self.shd1,
        }
        for const, table in expected.items():
            with self.subTest(const=const):
                self.assertIn(f"nums::{const}", self.fig,
                              f"{const} is no longer drawn")
                self.assertIn(scalars[const], table,
                              f"{const} = {scalars[const]} is not in its sheet")

    def test_ban_1_the_prior_art_is_named_on_the_image(self):
        """Without it an abstract that travels alone reads as the claim it is
        least entitled to: that this work shows SHD depends on temporal order."""
        for marker in ("NOT SHOWN HERE, AND NOT CLAIMED", "Cramer",
                       "Neuromorphic Sequential Arena", "Yu et al"):
            self.assertIn(marker, self.fig)

    def test_ban_1_both_arms_are_drawn_at_equal_weight(self):
        """The rate arm is half the measurement. Same helper, same bar width,
        same scale — a faint control would make this "shuffling hurts the
        attention model", which is prior art."""
        self.assertEqual(self.fig.count("cost_bar("), 2)
        boxes = re.findall(r"\((\d+), base, (\d+), height\)", self.fig)
        self.assertEqual(len(boxes), 2, boxes)
        self.assertEqual(boxes[0][1], boxes[1][1], f"bar widths differ: {boxes}")
        self.assertEqual(self.fig.count("scale,"), 2, "one shared scale")

    def test_ban_2_the_shuffle_is_described_as_done_to_the_data(self):
        self.assertIn("on the INPUT", self.fig)
        self.assertIn("BOTH the training and", self.fig)
        self.assertIn("Nothing is removed from the model", self.fig)
        for banned in ("attention off", "ablation of the read-out.",
                       "component axis"):
            self.assertNotIn(banned, self.fig)

    def test_ban_3_the_inflated_cost_is_not_here_either(self):
        self.assertNotIn("0.1577", drawable(self.fig))

    def test_ban_4_the_did_is_not_presented_as_the_gain(self):
        """rho = -0.1430 against +0.829. The abstract must not imply the gain
        decomposes into an order-dependent share and a remainder."""
        self.assertIn("THE EFFECT'S SIZE IS NOT THE GAIN", self.fig)
        self.assertIn("nums::DID_RHO", self.fig)
        self.assertIn("nums::DID_RHO_BAR", self.fig)
        self.assertIn("NOT that the gain decomposes", self.fig)

    def test_ban_4_the_headline_accuracy_is_not_the_largest_number(self):
        """The 0.8332 is on the image for scale and is drawn at body size; the
        costs and the ratio are the large type. Sizes are asserted rather than
        eyeballed, because "for scale and no more" is a layout promise."""
        head = re.search(r"Accuracy, for scale and no more.*?\n\s*(\d+),",
                         self.fig, re.S)
        self.assertIsNotNone(head, "the accuracy line is gone")
        ratio = re.search(r"SHUFFLE_COST_RATIO_32\),\n\s*(\d+),", self.fig)
        self.assertIsNotNone(ratio, "the ratio is gone")
        self.assertGreater(int(ratio.group(1)), int(head.group(1)))
        for banned in ("share of the gain", "% of the gain", "decomposes into"):
            self.assertNotIn(banned, self.fig.replace(
                "NOT that the gain decomposes into", ""))

    def test_the_two_disclosures_an_abstract_cannot_travel_without(self):
        """0.8332 is not competitive, and the gain inverts at h1024. Omitting
        either turns the image into a results claim about SHD."""
        self.assertIn("NOT COMPETITIVE", self.fig)
        self.assertIn("FIELD_FRONTIER_LO", self.fig)
        self.assertIn("INVERTS at width h1024", self.fig)
        self.assertIn("LOCATED BUT UNEXPLAINED", self.fig)

    def test_it_is_a_separate_file_from_the_secondary_abstract(self):
        """`graphical_abstract` exists and depicts the matched kill gate.
        Repointing it would have deleted the secondary program's front image."""
        self.assertIn('"lead_graphical_abstract"', self.text)
        self.assertIn('"graphical_abstract"', self.text)
        self.assertIn("lead_graphical_abstract", SPEC.read_text())

    def test_the_coverage_is_stated_as_measured(self):
        self.assertIn("nums::COVERAGE_COVERED", self.fig)
        self.assertIn("nums::COVERAGE_TOTAL", self.fig)
        self.assertIn("claim nothing", self.fig)


def matched_table(name: str, next_name: str) -> str:
    """One lettered `## Table X` block of `PAPER_RESULTS_TABLE.md`."""
    text = (ROOT / "results/PAPER_RESULTS_TABLE.md").read_text()
    start = text.index(f"## Table {name} —")
    return text[start:text.index(f"## Table {next_name} —", start)]


def triples(const: str) -> list[tuple[str, str, str]]:
    """A `[(&str, f64, f64); N]` constant, as (name, first, second) strings."""
    text = SOURCE.read_text()
    block = text[text.index(f"pub const {const}"):]
    block = block[:block.index("];")]
    return re.findall(r'\("([^"]+)",\s*(-?[\d.]+),\s*(-?[\d.]+)\)', block)


class Figure6Test(unittest.TestCase):
    """Figure 6, against Table A and its four bans.

    This figure had **no generator at all** until 2026-08-29. The file on disk
    was drawn on 24 July and plotted the superseded value block; the 2026-08-27
    pass that brought Figure M, Figure 5, Figure 7 and the graphical abstract
    current could not reach it, because a re-run cannot regenerate something
    nothing generates. Every assertion here is therefore new ground rather than
    a guard against drift.
    """

    @classmethod
    def setUpClass(cls):
        cls.text = SOURCE.read_text()
        cls.fig = figure_body("draw_fig6_matched_means", "fn draw_fig3(")
        cls.table_a = matched_table("A", "B")

    def test_the_table_was_found(self):
        self.assertIn("Matched dense-LIF kill gate", self.table_a)

    def test_every_drawn_mean_is_in_table_a(self):
        """The spec's Figure 6 block is this file's restatement of the sheet;
        the sheet is what the numbers have to survive."""
        for const in MATCHED:
            ff, rec = source_pairs()[const]
            with self.subTest(const=const):
                self.assertIn(ff, self.table_a, f"{const} ff {ff} is not in Table A")
                self.assertIn(rec, self.table_a, f"{const} rec {rec} is not in Table A")

    def test_every_gap_lcb_is_in_table_a(self):
        rows = triples("GAP_LCB")
        self.assertEqual(len(rows), 4, rows)
        for arm, ff, rec in rows:
            with self.subTest(arm=arm):
                self.assertIn(ff, self.table_a, f"{arm} ff {ff} is not in Table A")
                self.assertIn(rec.lstrip("-"), self.table_a,
                              f"{arm} rec {rec} is not in Table A")

    def test_ban_1_no_accuracy_is_encoded_as_a_length_or_a_position(self):
        """With the reference at 1.0000 every PASS reduces to "above 0.75". A
        bar row, a forest, or any sort over the means supplies an ordering the
        task cannot support."""
        self.assertNotIn("draw_bar_row(", self.fig)
        self.assertIn("does not rank them", self.fig)
        for banned in ("sort", "rank(", "best arm"):
            self.assertNotIn(banned, drawable(self.fig).lower())

    def test_ban_1_the_row_order_is_the_sheets_and_is_stated(self):
        """Grouping by verdict is the encoding the spec asks for; within a group
        the order is Table A's and the figure says which it is."""
        self.assertIn("rows are in the order of Table A", self.fig)
        self.assertIn("grouped by verdict", self.fig)

    def test_ban_2_the_contrasts_are_their_own_group(self):
        """Three ungated measurements coloured like passes would make six
        passing arms; coloured like failures they would become evidence for the
        FAIL the 0.9975 disclosure exists to qualify."""
        self.assertIn("Verdict::Contrast", self.fig)
        self.assertIn("Measured and NOT gated", self.fig)
        for const in ("BROADCAST_GRADED", "RL_GRADED", "RL_FLAT"):
            self.assertIn(f"nums::{const}", self.fig)

    def test_ban_3_the_recurrent_column_is_on_every_row(self):
        """The lead FAIL is a FAIL on BOTH graphs at n = 20, which is the
        claim's strength; a single-column figure drops it."""
        self.assertEqual(self.fig.count("rule_card("), 1,
                         "the cards are drawn by one loop over one array")
        self.assertIn("feed-forward / recurrent", self.fig)
        self.assertIn("● feed-forward   ○ recurrent", self.fig)

    def test_ban_3_the_negative_gap_lcb_keeps_its_sign(self):
        """−0.0192 is below zero, not merely short of the gate. A `Both` would
        have hidden it: the spec-parity parser only reads unsigned pairs."""
        self.assertEqual(triples("GAP_LCB")[0][2], "-0.0192")
        self.assertIn("BELOW ZERO", self.fig)
        self.assertIn("at_lcb(0.0)", self.fig)

    def test_ban_4_both_halves_of_the_gate_are_drawn(self):
        """acc ≥ 0.65 AND gap LCB > 0.5. Panel B exists because "cleared the
        floor" and "cleared the gate" are different sentences."""
        self.assertIn("Panel A", self.fig)
        self.assertIn("Panel B", self.fig)
        self.assertIn("nums::GATE_LCB", self.fig)
        self.assertIn("GATE_FLOOR: f64 = 0.65", self.text)
        self.assertIn("GATE_LCB: f64 = 0.5", self.text)

    def test_panel_b_is_positioned_from_the_loop_that_precedes_it(self):
        """The 2026-08-29 lead-figure pass lost a line to a hardcoded base left
        behind by a coordinate shift. Panel B derives its origin instead."""
        self.assertIn("let panel_b = y + 20;", self.fig)
        self.assertIn("let strip_top = panel_b + 48;", self.fig)


class Figure8Test(unittest.TestCase):
    """The transfer ladder, against Tables A and C and its three bans."""

    @classmethod
    def setUpClass(cls):
        cls.text = SOURCE.read_text()
        cls.fig = figure_body("draw_fig8_transfer_ladder",
                              "fn draw_graphical_abstract(")
        cls.table_a = matched_table("A", "B")
        cls.table_c = matched_table("C", "D")

    def test_the_tables_were_found(self):
        self.assertIn("Live REINFORCE transfer", self.table_c)

    def test_rung_1_is_the_current_matched_figures_from_table_a(self):
        """It was stale here, at 0.9200 / 0.6846 — the superseded block."""
        self.assertIn("nums::RL_FB", self.fig)
        self.assertIn("0.9765 ff / 0.9079 rec", self.fig)
        for value in ("0.9950", "0.9812", "0.9765", "0.9079"):
            self.assertIn(value, self.table_a, f"{value} is not in Table A")

    def test_every_rung_below_the_break_is_in_table_c(self):
        rows = triples("GAP_CLOSE") + triples("BREAK_IT")
        self.assertEqual(len(rows), 11, rows)
        for name, local, lcb in rows:
            with self.subTest(protocol=name):
                self.assertIn(local, self.table_c, f"{name}: {local} is not in Table C")
                self.assertIn(lcb.lstrip("-"), self.table_c,
                              f"{name}: {lcb} is not in Table C")

    def test_rung_2_is_in_table_c(self):
        scalars = source_scalars()
        self.assertEqual(scalars["LIVE_RFB"], "0.4900")
        self.assertEqual(scalars["LIVE_RFB_LCB"], "0.0737")
        for value in ("0.4900", "0.0737"):
            self.assertIn(value, self.table_c)

    def test_ban_1_the_substrate_change_is_drawn_and_named(self):
        """Rung 1 is the matched dense-LIF forward; rungs 2–4 are the live
        muted-θ / k-WTA engine. One connected descent would read as a single
        system degrading, which is the reading the transfer gap refuses."""
        self.assertIn("THE SUBSTRATE CHANGES HERE", self.fig)
        self.assertIn("NOT one system at twelve settings", self.fig)

    def test_ban_1_nothing_is_drawn_across_the_break(self):
        """Rung 1 is a card. The axes below it are built from `at_acc`/`at_lcb`,
        whose domains stop at 0.80 and 0.55 — rung 1's 0.9950 and 0.9765 are not
        on them and cannot be, which is the point."""
        self.assertIn("let (acc_lo, acc_hi) = (0.40, 0.80);", self.fig)
        self.assertIn("let (lcb_lo, lcb_hi) = (-0.05, 0.55);", self.fig)
        body = drawable(self.fig)
        for line in body.splitlines():
            if "PathElement" in line:
                self.assertNotIn("GAP_CLOSE", line)
                self.assertNotIn("BREAK_IT", line)

    def test_ban_2_both_gates_are_drawn(self):
        """On accuracy alone v15, v18 and v20 read as near-misses of one bar."""
        self.assertIn("nums::GATE_FLOOR", self.fig)
        self.assertIn("nums::GATE_LCB", self.fig)
        self.assertIn("local accuracy", self.fig)
        self.assertIn("gap LCB", self.fig)
        self.assertIn("Floor cleared is not gate cleared", self.fig)

    def test_ban_2_the_floor_count_is_counted_and_not_written_down(self):
        """A literal would go on reading "six" after an arm was added,
        corrected or withdrawn from the arrays the dots come from."""
        self.assertIn(".filter(|local| *local >= nums::GATE_FLOOR)", self.fig)
        self.assertIn(".count()", self.fig)
        for banned in ("Six clear", "six clear", "6 of 12"):
            self.assertNotIn(banned, self.fig)

    def test_ban_3_the_arms_are_in_protocol_order(self):
        """A sequential exploratory family with no family-wise claim. Sorting
        by either quantity asserts a ranking the statistics do not carry."""
        self.assertEqual([n.split()[0] for n, *_ in triples("GAP_CLOSE")],
                         ["v14", "v15", "v16", "v17", "v18", "v19"])
        self.assertEqual([n.split()[0] for n, *_ in triples("BREAK_IT")],
                         ["v20", "v21", "v22", "v23", "v24"])
        for series in ("GAP_CLOSE", "BREAK_IT"):
            values = [float(local) for _, local, _ in triples(series)]
            self.assertNotEqual(values, sorted(values), f"{series} is sorted")
            self.assertNotEqual(values, sorted(values, reverse=True),
                                f"{series} is reverse-sorted")
        self.assertIn("NOT A RANKING", self.fig)

    def test_ban_3_only_the_two_landmarks_the_spec_names_are_called_out(self):
        self.assertIn("named as landmarks because the specification names them",
                      self.fig)
        self.assertEqual(source_scalars()["BEST_LOCAL_GAP_CLOSE"], "0.7262")
        self.assertEqual(source_scalars()["BEST_LCB_ANYWHERE"], "0.3127")

    def test_no_mechanism_is_attributed(self):
        """The four suspects have never been tested individually
        (`DESIGN_TRANSFER_GAP_DECOMPOSITION.md`), so the figure names them as
        untested rather than as an explanation."""
        self.assertIn("NO MECHANISM", self.fig)
        self.assertIn("never been tested individually", self.fig)

    def test_the_legend_carries_colour_rather_than_a_text_bullet(self):
        """The first version wrote "●" twice in one grey label, so the only
        distinction the legend existed for was absent from it."""
        self.assertIn("cleared the 0.65 accuracy floor", self.fig)
        self.assertNotIn('"● cleared', self.fig)


class TheManuscriptReachesTheFiguresTest(unittest.TestCase):
    """A figure nothing cites does not appear in a submission.

    On 2026-08-27, after all four lead figures were drawn,
    `PAPER_DRAFT.md` referenced **Figure M four times and no other figure at
    all**. The artwork existed, the spec named where each figure belonged, and
    the manuscript pointed at none of them — which is the same failure as
    artwork that does not exist, arriving one step later.

    This class was scoped to the lead program and Figure M until 2026-08-29,
    with a note that the secondary program's Figures 5–9 were uncited and that
    that was "a live authoring gap, not something this file should assert away".
    Two of those five had no artwork to cite; both now do, so the gap is closed
    rather than scoped around and the assertion covers the whole package.
    """

    DRAFT = ROOT / "results/PAPER_DRAFT.md"
    #: Every numbered figure the spec carries. Lettered figures (0, D, M) are
    #: legend and appendix material and are checked separately.
    NUMBERED = tuple(range(1, 10))

    @classmethod
    def setUpClass(cls):
        cls.text = cls.DRAFT.read_text()

    def test_every_numbered_figure_is_cited(self):
        for n in self.NUMBERED:
            with self.subTest(figure=n):
                self.assertRegex(
                    self.text, rf"\(Figure {n}\)",
                    f"Figure {n} is drawn but the manuscript never refers to it")

    def test_figure_m_is_cited(self):
        self.assertIn("Figure M", self.text)

    def test_no_figure_is_cited_more_than_its_home(self):
        """Each is placed once, at the claim a reader meets it on. Repeating a
        callout in the abstract and again in the discussion is a layout
        decision, not something to accrue by accident."""
        for n in self.NUMBERED:
            with self.subTest(figure=n):
                self.assertEqual(len(re.findall(rf"\(Figure {n}\)", self.text)), 1)

    def test_the_spec_numbers_the_same_nine_figures(self):
        """The range above is asserted against the sheet that owns it, so a
        tenth figure cannot be specified and left uncited.

        Figure 0 is excluded because the spec groups it with the LETTERED
        figures — "Lettered; unaffected by the renumbering" — and D and M say
        the same. It carries a numeral and is not part of the sequence, which
        is exactly the kind of thing a bare `\\d` sweep gets wrong, so the
        grouping is asserted here rather than assumed.
        """
        spec = SPEC.read_text()
        headings = re.findall(r"^## Figure (\d)(?: \(optional\))? — (.+)$", spec, re.M)
        self.assertIn("Lettered; unaffected by the renumbering",
                      spec[spec.index("## Figure 0 — "):],
                      "Figure 0 is no longer grouped with the lettered figures")
        numbered = {int(n) for n, _ in headings} - {0}
        self.assertEqual(numbered, set(self.NUMBERED), sorted(numbered))

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


class TheSkeletonsFigureMapIsCurrentTest(unittest.TestCase):
    """`PAPER_SKELETON.md`'s figure ↔ binary map, against the artwork that exists.

    On 2026-08-29 that map still carried four `TODO(source needed): no spec
    entry, no artwork` rows for the **lead** figures — two days after all four
    were specified and drawn — and a numbering note saying the spec "contains no
    entry for any SHD figure". It also called Figure 4 the *substrate panel*,
    where the spec and the draft both call it the resolution ladder, so the
    package disagreed with itself about what Figure 4 is.

    None of that would have been caught by anything: the figure tests read the
    spec and the draft, and the skeleton is a third document that describes both.
    """

    SKELETON = ROOT / "results/PAPER_SKELETON.md"

    @classmethod
    def setUpClass(cls):
        text = cls.SKELETON.read_text()
        cls.text = text
        cls.map = text[text.index("## Figure ↔ binary / hash map"):]

    def test_the_map_was_found(self):
        self.assertIn("Fig. 1 difference-in-differences", self.map)

    def test_no_drawn_figure_is_recorded_as_having_no_artwork(self):
        """The failure this class exists for. A `TODO(source needed)` is a
        useful marker and a stale one is worse than none: it tells a reader to
        go and make something that already exists."""
        for stem in TheGeneratorOwnsTheArtworkTest.STEMS:
            with self.subTest(stem=stem):
                self.assertIn(stem, self.map,
                              f"{stem} is drawn but the skeleton's map does not "
                              f"name it")
        self.assertNotIn("no artwork", self.map)
        self.assertNotIn("no spec entry", self.map)

    def test_the_map_names_every_stem_the_generator_writes(self):
        """The generator's stem list and this class's STEMS must not drift.
        Asserted here rather than in the map test, so a stem added to the
        generator and to neither list cannot slip through both."""
        source = SOURCE.read_text()
        stems = set(re.findall(r'write_pair(?:_sized)?\(\s*out_dir,\s*"([\w]+)"',
                               source))
        self.assertEqual(stems, set(TheGeneratorOwnsTheArtworkTest.STEMS),
                         sorted(stems))

    def test_figure_4_is_the_resolution_ladder_here_too(self):
        """The spec and the draft both call it that; the skeleton called it the
        substrate panel, which is a different measurement on different waves."""
        self.assertIn("Fig. 4 resolution ladder", self.map)
        self.assertNotIn("Fig. 4 substrate panel", self.map)

    def test_the_substrate_panel_has_a_figure_and_the_map_names_it(self):
        """Correcting Figure 4's identity left the substrate comparison with no
        figure at all — §3.7 is a lead-program section with three waves behind
        it and nothing drawn. This assertion was the opposite of itself for
        part of 2026-08-29: it required the map to say "no figure is specified",
        which was true for as long as it took to draw one. What it pins now is
        that the row cannot go back to naming the resolution ladder's number.
        """
        self.assertIn("Fig. S substrate", self.map)
        self.assertIn("figS_substrate", self.map)
        self.assertIn("Table SHD-7", self.map)
        self.assertNotIn("no figure is specified", self.map.lower())

    def test_the_numbering_note_no_longer_claims_the_spec_has_no_shd_entry(self):
        note = self.text[self.text.index("> **Figure-numbering note"):]
        note = note[:note.index("\n\n---")]
        self.assertNotIn("no spec entry", note)
        self.assertIn("2026-08-27", note)
        self.assertIn("5–9", note, "the matched program is Figures 5-9, not 5-8")


class TheGeneratorOwnsTheArtworkTest(unittest.TestCase):
    """The committed files and the generator's stem list must not drift apart."""

    STEMS = ("leadfig1_the_conditional", "leadfig2_headline_accuracy",
             "leadfig3_width_ladder", "leadfig4_resolution_ladder",
             "figS_substrate", "lead_graphical_abstract",
             "figM_mechanism_richness_addressability", "fig1_matched_rule_swap",
             "fig2_matched_means", "fig3_engine_c1_means",
             "fig4_transfer_ladder", "graphical_abstract")
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
