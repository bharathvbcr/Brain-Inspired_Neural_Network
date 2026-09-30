#!/usr/bin/env python3
"""Tests for `scripts/did_confidence_intervals.py`.

An interval table is only worth having if its point estimates are the published
ones and its row set is the coverage report's. The failures this guards against
are a lookup that resolves the wrong arm (and still prints a plausible number)
and a point that quietly drops out of the table.

Run: python3 scripts/test_did_confidence_intervals.py
"""

from __future__ import annotations

import importlib.util
import random
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location(
    "did_confidence_intervals", ROOT / "scripts/did_confidence_intervals.py")
DCI = importlib.util.module_from_spec(spec)
sys.modules["did_confidence_intervals"] = DCI
spec.loader.exec_module(DCI)

TOL = 5e-5
ANCHOR = ("published-2ms", "adjacent-sum-5")

#: Loading the corpus is the slow part; the bootstrap is run small here and at
#: full size only in the determinism test.
ROWS = DCI.compute(DCI.build_rows(), resamples=500)


def find(hidden, contract, geometry, depth, budget, condition="bin-shuffled"):
    hits = [r for r in ROWS if (r.hidden, r.contract, r.geometry, r.depth,
                                r.budget, r.condition)
            == (hidden, contract, geometry, depth, budget, condition)]
    if len(hits) != 1:
        raise AssertionError(f"{len(hits)} rows for h{hidden}/{contract}/"
                             f"{geometry}/{depth}/e{budget}/{condition}")
    return hits[0]


class PublishedValuesReproduce(unittest.TestCase):
    def check(self, row, n, gain, shuffled, did):
        s = row.stats
        self.assertEqual(s["n"], n)
        self.assertAlmostEqual(s["gain_intact"], gain, delta=TOL)
        if shuffled is not None:
            self.assertAlmostEqual(s["gain_shuffled"], shuffled, delta=TOL)
        self.assertAlmostEqual(s["did"], did, delta=TOL)

    def test_wave22_h512_l1(self):
        self.check(find(512, *ANCHOR, "d32l1", 400), 12, 0.0043, -0.0850, 0.0893)

    def test_wave22_h1024_l1(self):
        self.check(find(1024, *ANCHOR, "d32l1", 400), 12, -0.0159, -0.0834, 0.0675)

    def test_anchor_n32(self):
        self.check(find(128, *ANCHOR, "d32l4", 400), 32, 0.1275, None, 0.1205)


class AgreesWithTheOwningAnalyser(unittest.TestCase):
    """Each row's DiD, n and positive count equal the frozen analyser's own."""

    def test_every_row_matches_its_source(self):
        w21, _ = DCI.W21.index((DCI.W21.ARCHIVE_V2, DCI.W21.ARCHIVE_V1))
        w22, _ = DCI.W22.index([DCI.W22.ARCHIVE_V1, DCI.W22.ARCHIVE_V2])
        w24, _ = DCI.W24.index([DCI.W24.ARCHIVE_V2])
        w25, _ = DCI.W25.index([DCI.W25.ARCHIVE_V2])
        checked = 0
        for row in ROWS:
            point = (row.hidden, row.contract, row.geometry)
            if row.condition != "bin-shuffled" or row.source.startswith("wave 1"):
                continue
            if row.source.startswith("W21"):
                expected = DCI.W21.did(w21, *point)
            elif row.source.startswith("W22"):
                expected = DCI.W22.did(w22, *point, row.depth)
            elif row.source.startswith("W24"):
                expected = DCI.W24.did(w24, 100, *point, row.depth, "bin-shuffled")
            elif row.source.startswith("W25"):
                rate = (DCI.W25.W24 if point in DCI.W25.RATE_FROM_W24
                        else DCI.W25.THIS)
                expected = DCI.W25.did(
                    w25, "bin-shuffled", epochs=100, hidden=row.hidden,
                    contract=row.contract, geometry=row.geometry, depth=row.depth,
                    intact_wave=DCI.W25.THIS, treated_wave=DCI.W25.THIS,
                    rate_intact_wave=rate, rate_treated_wave=rate)
            else:
                self.fail(f"unrecognised source {row.source!r}")
            value, positive, pairs = expected
            with self.subTest(row=(point, row.depth, row.budget)):
                self.assertAlmostEqual(row.stats["did"], value, delta=1e-12)
                self.assertEqual(row.stats["positive"], positive)
                self.assertEqual(row.stats["n"], pairs)
            checked += 1
        self.assertEqual(checked, 41)  # 42 bin-shuffled rows less the one wave-1 row

    def test_hidden_shuffled_matches_wave27(self):
        cells, _, _ = DCI.W27.index(DCI.V3)
        value, positive, n = DCI.W27.did(cells, "w27hid", "hidden-shuffled")
        row = find(128, *ANCHOR, "d32l4", 400, "hidden-shuffled")
        self.assertAlmostEqual(row.stats["did"], value, delta=1e-12)
        self.assertEqual((row.stats["positive"], row.stats["n"]), (positive, n))

    def test_did_is_the_difference_of_the_gains(self):
        for row in ROWS:
            s = row.stats
            self.assertAlmostEqual(s["did"], s["gain_intact"] - s["gain_shuffled"],
                                   delta=1e-12)


class NoSilentDrops(unittest.TestCase):
    def test_every_coverage_point_is_in_the_output_at_both_budgets(self):
        coverage = DCI.MC.coverage(DCI.MC.read(DCI.MC.DEFAULT_ROOTS))
        self.assertEqual(len(coverage), 21)
        table, _ = DCI.render(ROWS)
        text = "\n".join(table)
        for (hidden, contract, geometry), depth, *_ in coverage:
            for budget in (400, 100):
                lead = (f"| {hidden} | `{contract}` | `{geometry}` | {depth} "
                        f"| e{budget} | bin-shuffled |")
                with self.subTest(point=(hidden, contract, geometry, depth, budget)):
                    self.assertEqual(text.count(lead), 1)

    def test_rows_follow_coverage_order(self):
        coverage = DCI.MC.coverage(DCI.MC.read(DCI.MC.DEFAULT_ROOTS))
        order = [(p, d) for p, d, *_ in coverage]
        seen = []
        for row in ROWS:
            key = ((row.hidden, row.contract, row.geometry), row.depth)
            if not seen or seen[-1] != key:
                seen.append(key)
        self.assertEqual(seen, order)

    def test_an_unpaired_point_is_printed_not_dropped(self):
        row = DCI.Row(64, "x", "y", "d1l1", 400, "", reason="nothing here")
        table, _ = DCI.render([row])
        self.assertIn("not paired: nothing here", table[-1])


class Bootstrap(unittest.TestCase):
    def test_deterministic(self):
        rows = [r for r in ROWS if r.hidden == 512 and r.depth == "d32l1"]
        first = [DCI.bootstrap(r.quads, random.Random(DCI.RNG_SEED)) for r in rows]
        second = [DCI.bootstrap(r.quads, random.Random(DCI.RNG_SEED)) for r in rows]
        self.assertEqual(first, second)
        a = DCI.render(DCI.compute(DCI.build_rows(), resamples=300))
        b = DCI.render(DCI.compute(DCI.build_rows(), resamples=300))
        self.assertEqual(a, b)

    def test_intervals_contain_the_point_estimate(self):
        for row in ROWS:
            for name in ("gain_intact", "gain_shuffled", "did"):
                lo, hi = row.stats[name + "_ci"]
                with self.subTest(row=(row.hidden, row.depth, row.budget, name)):
                    self.assertLessEqual(lo, row.stats[name])
                    self.assertLessEqual(row.stats[name], hi)

    def test_fabricated_quadruples_give_the_known_means(self):
        seeds = range(1, 11)
        # gain intact = 0.10 + 0.01 s, gain shuffled = 0.02 s - 0.05
        ai = {s: 0.80 + 0.01 * s for s in seeds}
        ri = {s: 0.70 for s in seeds}
        ax = {s: 0.60 + 0.02 * s for s in seeds}
        rx = {s: 0.65 for s in seeds}
        rx[99] = ai[99] = 0.5  # a seed missing from two arms must not enter
        quads = DCI.quadruples(ai, ax, ri, rx)
        self.assertEqual(sorted(quads), list(seeds))
        stats = DCI.bootstrap(quads, random.Random(0), resamples=2000)
        self.assertEqual(stats["n"], 10)
        self.assertAlmostEqual(stats["gain_intact"], 0.155, delta=1e-12)
        self.assertAlmostEqual(stats["gain_shuffled"], 0.06, delta=1e-12)
        self.assertAlmostEqual(stats["did"], 0.095, delta=1e-12)
        # DiD per seed = 0.15 - 0.01 s: positive for every s < 15.
        self.assertEqual(stats["positive"], 10)

    def test_constant_quadruples_give_a_zero_width_interval(self):
        quads = {s: (0.1, 0.02, 0.08) for s in range(12)}
        stats = DCI.bootstrap(quads, random.Random(1), resamples=200)
        for name, value in (("gain_intact", 0.1), ("gain_shuffled", 0.02), ("did", 0.08)):
            lo, hi = stats[name + "_ci"]
            self.assertAlmostEqual(lo, value, delta=1e-12)
            self.assertAlmostEqual(hi, value, delta=1e-12)

    def test_percentile_indices(self):
        values = list(range(10_000))
        self.assertEqual(DCI.percentile_interval(values), (250, 9749))

    def test_no_gain_flag(self):
        flagged = {(r.hidden, r.depth, r.budget) for r in ROWS
                   if r.stats["gain_intact_ci"][0] < DCI.NO_GAIN_BAR}
        self.assertIn((512, "d32l1", 400), flagged)
        self.assertNotIn((128, "d32l4", 400), flagged)
        table, _ = DCI.render([find(512, *ANCHOR, "d32l1", 400)])
        self.assertIn("no gain to lose", table[-1])


class TheManuscriptCarriesThisTable(unittest.TestCase):
    """Appendix K of the manuscript is this script's `--paper` output.

    The number sweep reads four-decimal values, and the appendix prints three,
    so without this test a hand edit to the table -- or a change to the cells
    -- would pass every other gate. Computed at the full resample count and the
    fixed RNG seed, exactly as `--paper` computes it."""

    def test_appendix_k_is_the_script_output_verbatim(self):
        table = DCI.paper_table(DCI.compute(DCI.build_rows()))
        draft = (ROOT / "results/PAPER_DRAFT.md").read_text()
        self.assertIn("## Appendix K — Every operating point, with intervals", draft)
        section = draft.split("## Appendix K — Every operating point, with intervals", 1)[1]
        section = section.split("\n## ", 1)[0]
        printed = "\n".join(line for line in section.splitlines() if line.startswith("|"))
        self.assertEqual(printed, table,
                         "Appendix K differs from `did_confidence_intervals.py "
                         "--paper`; regenerate the table rather than editing it")

    def test_the_paper_table_has_every_point_at_both_budgets(self):
        table = DCI.paper_table(ROWS)
        self.assertEqual(len(table.splitlines()) - 2, 42)

    def test_an_unpaired_point_stops_the_paper_table(self):
        broken = DCI.Row(hidden=128, contract="published-2ms",
                         geometry="adjacent-sum-5", depth="d32l4", budget=400,
                         source="test", reason="no cells")
        with self.assertRaises(ValueError):
            DCI.paper_table([broken])


if __name__ == "__main__":
    unittest.main()
