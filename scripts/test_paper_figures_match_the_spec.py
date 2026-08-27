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


class TheGeneratorOwnsTheArtworkTest(unittest.TestCase):
    """The committed files and the generator's stem list must not drift apart."""

    STEMS = ("figM_mechanism_richness_addressability", "fig1_matched_rule_swap",
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
