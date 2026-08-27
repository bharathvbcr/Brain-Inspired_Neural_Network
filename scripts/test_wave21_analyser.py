#!/usr/bin/env python3
"""Tests for `scripts/aws/analyse_wave21.py`, frozen with its preregistration.

The statistic is a difference of differences over four arms. Every failure mode
worth testing is a way of computing something that looks like one and is not:
arms that are not seed-paired, a read-out depth that does not belong, a rank
correlation over whichever rungs survived.
"""

from __future__ import annotations

import contextlib
import importlib.util
import io
import json
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def load(name, relative):
    spec = importlib.util.spec_from_file_location(name, ROOT / relative)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    return mod


W21 = load("analyse_wave21", "scripts/aws/analyse_wave21.py")
W20 = load("analyse_wave20", "scripts/aws/analyse_wave20.py")
PREREG = ROOT / "results/PREREG_2026-08-27_THE_MECHANISM_ACROSS_THE_DESIGN_SPACE.md"


def cell(accuracy, arm, temporal="intact", hidden=128,
         contract="published-2ms", geometry="adjacent-sum-5"):
    return {
        "accuracy": accuracy, "arm": arm, "hidden": hidden,
        "contract": contract, "geometry": geometry, "epochs": 400,
        "surrogate_scale": 1.0, "clip_grad_norm": None,
        "temporal_condition": temporal,
        # enough for `validity_problems` to pass a healthy cell
        "classes_predicted": 20, "majority_prediction": 0.1,
        "silent_fraction": 0.02, "saturated_fraction": 0.0,
        "mechanical_status": "COMPLETE", "non_finite_events": 0,
        "epoch_max_gradient_norm": [1.0, 1.0],
        "temporal_audit": {"samples": 1, "counts_preserved": True,
                           "relocated_fraction": 0.9, "mean_bin_distance": 3.0},
    }


class RegisteredBarsTest(unittest.TestCase):
    """The analyser's constants and the document must say the same thing."""

    def test_the_prereg_carries_every_bar_the_analyser_uses(self):
        text = PREREG.read_text()
        for needle in ("+0.03", "9/12", "+0.02", "0.829", "9 seed-paired"):
            with self.subTest(needle):
                self.assertIn(needle, text)

    def test_the_analyser_constants_are_the_registered_ones(self):
        self.assertEqual(W21.MIN_PAIRS, 9)
        self.assertEqual(W21.H21_1_MIN_DID, 0.03)
        self.assertEqual(W21.H21_1_MIN_POSITIVE, 9)
        self.assertEqual(W21.H21_2_MAX_DID, 0.02)
        self.assertEqual(W21.H21_3_MIN_RHO, 0.829)
        self.assertEqual(W21.LADDER, (128, 256, 384, 512, 768, 1024))


class DiDTest(unittest.TestCase):
    def setUp(self):
        self.dir = Path(tempfile.mkdtemp())

    def write(self, wave, arm, seed, accuracy, temporal="intact", depth=None, **kw):
        parts = [wave, arm.replace("+", "-"), f"h{kw.get('hidden', 128)}"]
        if depth:
            parts.append(depth)
        if temporal != "intact":
            parts.append(temporal)
        parts.append(f"s{seed}")
        (self.dir / ("__".join(parts) + ".json")).write_text(
            json.dumps(cell(accuracy, arm, temporal, **kw)))

    def quad(self, seeds, ai, as_, ri, rs, **kw):
        for s in seeds:
            self.write("w1", "ff+fixed+attn", s, ai, depth="d32l4", **kw)
            self.write("w21mec", "ff+fixed+attn", s, as_, "bin-shuffled",
                       depth="d32l4", **kw)
            self.write("w1", "ff+fixed", s, ri, **kw)
            self.write("w21mec", "ff+fixed", s, rs, "bin-shuffled", **kw)

    def did(self, **kw):
        cells, _ = W21.index([self.dir])
        return W21.did(cells, kw.get("hidden", 128), "published-2ms",
                       kw.get("geometry", "adjacent-sum-5"))

    def test_the_difference_of_differences_is_the_attention_drop_minus_the_rate_drop(self):
        # attention drops 0.20, rate drops 0.05 -> DiD +0.15
        self.quad(range(1, 13), ai=0.85, as_=0.65, ri=0.70, rs=0.65)
        value, positive, pairs = self.did()
        self.assertEqual(pairs, 12)
        self.assertAlmostEqual(value, 0.15, places=6)
        self.assertEqual(positive, 12)

    def test_a_seed_missing_from_one_arm_leaves_all_four(self):
        """Four differently-populated arms are not a difference of differences."""
        self.quad(range(1, 13), ai=0.85, as_=0.65, ri=0.70, rs=0.65)
        (self.dir / "w21mec__ff-fixed__h128__bin-shuffled__s5.json").unlink()
        self.assertEqual(self.did()[2], 11)

    def test_below_the_floor_no_number_is_printed(self):
        self.quad(range(1, 6), ai=0.85, as_=0.65, ri=0.70, rs=0.65)
        out = self.run_tool()
        self.assertIn("NOT EVALUABLE", out)
        self.assertNotIn("+0.1500", out)

    def test_a_d32l1_cell_is_never_merged_into_the_d32l4_comparison(self):
        """The H17-2 defect: an archived control at the wrong read-out depth."""
        self.quad(range(1, 13), ai=0.85, as_=0.65, ri=0.70, rs=0.65)
        for s in range(1, 13):
            self.write("w9", "ff+fixed+attn", s, 0.99, "bin-shuffled",
                       depth="d32l1")
        self.assertAlmostEqual(self.did()[0], 0.15, places=6)

    def test_a_voided_cell_removes_its_seed_from_the_quadruple(self):
        self.quad(range(1, 13), ai=0.85, as_=0.65, ri=0.70, rs=0.65)
        sick = cell(0.65, "ff+fixed+attn", "bin-shuffled")
        sick["classes_predicted"] = 3          # collapsed read-out
        (self.dir / "w21mec__ff-fixed-attn__h128__d32l4__bin-shuffled__s3.json"
         ).write_text(json.dumps(sick))
        self.assertEqual(self.did()[2], 11)

    def test_a_different_geometry_is_a_different_operating_point(self):
        self.quad(range(1, 13), ai=0.85, as_=0.65, ri=0.70, rs=0.65)
        self.assertEqual(self.did(geometry="channels-700")[2], 0)

    def run_tool(self):
        buf = io.StringIO()
        old, sys.argv = sys.argv, ["x", "--results", str(self.dir),
                                   "--archive", str(self.dir)]
        try:
            with contextlib.redirect_stdout(buf):
                W21.main()
        finally:
            sys.argv = old
        return buf.getvalue()

    def test_h21_2_accepts_a_negative_did(self):
        """The prediction is the absence of an order-dependent benefit, not its
        sign. At h1024 attention is worse than no read-out, so shuffling could
        help it."""
        self.quad(range(1, 13), ai=0.58, as_=0.66, ri=0.74, rs=0.70, hidden=1024)
        out = self.run_tool()
        self.assertIn("H21-2: MET", out)

    def test_h21_2_rejects_a_large_positive_did(self):
        self.quad(range(1, 13), ai=0.85, as_=0.65, ri=0.70, rs=0.65, hidden=1024)
        self.assertIn("H21-2: NOT MET", self.run_tool())

    def test_h21_1_requires_all_three_widths(self):
        for h in (256, 384):
            self.quad(range(1, 13), ai=0.85, as_=0.65, ri=0.70, rs=0.65, hidden=h)
        self.quad(range(1, 13), ai=0.85, as_=0.85, ri=0.70, rs=0.70, hidden=512)
        out = self.run_tool()
        self.assertIn("H21-1: NOT MET", out)
        self.assertIn("2/3 widths", out)

    def test_h21_3_is_not_evaluable_when_a_rung_is_missing(self):
        for h in (128, 256, 384, 512, 768):
            self.quad(range(1, 13), ai=0.85, as_=0.65, ri=0.70, rs=0.65, hidden=h)
        out = self.run_tool()
        self.assertIn("H21-3: NOT EVALUABLE", out)
        self.assertIn("h1024", out)


class SpearmanAgreementTest(unittest.TestCase):
    """Two frozen analysers carry the same implementation and must agree.

    Neither imports the other: both are registered artefacts. This is the same
    trade `AWS_TIMEOUT_S` makes across the `scripts/aws` helpers.
    """

    CASES = [
        ([1, 2, 3, 4, 5, 6], [1, 2, 3, 4, 5, 6]),
        ([1, 2, 3, 4, 5, 6], [6, 5, 4, 3, 2, 1]),
        ([1, 1, 1, 2, 3, 4], [4, 3, 2, 1, 1, 1]),
        ([0.1275, 0.0966, 0.076, 0.0876, 0.056, -0.1618],
         [0.12, 0.09, 0.07, 0.08, 0.05, -0.01]),
    ]

    def test_the_two_copies_agree(self):
        for xs, ys in self.CASES:
            with self.subTest(xs=xs):
                self.assertEqual(W21.spearman(xs, ys), W20.spearman(xs, ys))

    def test_too_few_points_is_none_not_a_number(self):
        self.assertIsNone(W21.spearman([1, 2, 3], [1, 2, 3]))


class LiveCorpusTest(unittest.TestCase):
    """The analyser must reproduce the paper's own number without being told it."""

    def test_it_derives_the_published_h128_contrast(self):
        cells, _ = W21.index([W21.ARCHIVE_V2, W21.ARCHIVE_V1])
        value, positive, pairs = W21.did(cells, 128, "published-2ms",
                                         "adjacent-sum-5")
        self.assertEqual(pairs, 32)
        self.assertEqual(positive, 32)
        self.assertAlmostEqual(value, 0.1205, places=4)

    def test_no_other_operating_point_has_a_contrast_yet(self):
        """Pinned so that the day wave 21 lands, this test is what changes."""
        cells, _ = W21.index([W21.ARCHIVE_V2, W21.ARCHIVE_V1])
        for hidden in W21.LADDER[1:]:
            with self.subTest(hidden):
                self.assertEqual(
                    W21.did(cells, hidden, "published-2ms", "adjacent-sum-5")[2],
                    0)


if __name__ == "__main__":
    unittest.main(verbosity=2)
