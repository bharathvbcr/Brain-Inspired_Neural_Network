#!/usr/bin/env python3
"""Tests for `scripts/fit_retention.py`.

The script separates two failure modes that look identical in the accuracy
column — overfitting and losing a fit — so every test here is a way of getting
that separation wrong: a trace that never fit, a trace that fit and held, one
that went non-finite, and a summary computed over so few cells that a slice is
reported as the set.

Run: python3 scripts/test_fit_retention.py
"""

from __future__ import annotations

import importlib.util
import io
import json
import sys
import tempfile
import unittest
from contextlib import redirect_stdout
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
spec = importlib.util.spec_from_file_location("fit_retention", ROOT / "scripts/fit_retention.py")
fit_retention = importlib.util.module_from_spec(spec)
sys.modules["fit_retention"] = fit_retention
spec.loader.exec_module(fit_retention)


def cell(**over):
    base = {
        "schema": "shd-cal-cell-v1", "arm": "ff+fixed+attn", "hidden": 1024,
        "epochs": 400, "attn_dim": 32, "attn_layers": 4, "accuracy": 0.6,
        "contract": "published-2ms", "geometry": "adjacent-sum-5",
        "temporal_condition": "intact", "epoch_mean_loss": [1.0] * 40,
    }
    base.update(over)
    return base


class RetentionTest(unittest.TestCase):
    """The discriminator itself."""

    def test_a_trace_that_fits_and_holds_is_not_lost(self):
        """The overfitting shape: the arm reaches a low loss and stays there.
        Nothing about this should be flagged."""
        trace = [2.0] * 5 + [0.001] * 35
        measured = fit_retention.retention(cell(epoch_mean_loss=trace))
        self.assertFalse(measured["lost"])
        self.assertAlmostEqual(measured["best"], 0.001)
        self.assertAlmostEqual(measured["final"], 0.001)

    def test_a_trace_that_fits_and_loses_it_is_lost(self):
        """The h1024/d32l4 shape: a fit is reached early and not held."""
        trace = [2.0] * 5 + [0.03] * 10 + [0.6] * 25
        measured = fit_retention.retention(cell(epoch_mean_loss=trace))
        self.assertTrue(measured["lost"])
        self.assertAlmostEqual(measured["best"], 0.03)
        self.assertEqual(measured["best_epoch"], 5)

    def test_a_trace_that_never_fits_is_not_called_lost(self):
        """An arm that never descends has not LOST a fit — it never had one,
        which is a third failure and must not be merged into this one."""
        measured = fit_retention.retention(cell(epoch_mean_loss=[2.0] * 40))
        self.assertFalse(measured["lost"])

    def test_the_ratio_is_against_the_cells_own_best_not_a_constant(self):
        """Losses span four orders of magnitude between arms, so an absolute
        threshold would flag whichever arm happens to train to a small number."""
        tiny = [1.0] * 5 + [1e-6] * 5 + [1e-3] * 30
        self.assertTrue(fit_retention.retention(cell(epoch_mean_loss=tiny))["lost"])
        large = [9.0] * 5 + [1.0] * 35
        self.assertFalse(fit_retention.retention(cell(epoch_mean_loss=large))["lost"])

    def test_a_non_finite_trace_is_separated_rather_than_averaged(self):
        """A diverged run is a different failure. Averaging it in would move
        every summary it lands in and would look like a retention result."""
        trace = [1.0] * 39 + [float("nan")]
        self.assertTrue(fit_retention.retention(cell(epoch_mean_loss=trace))["non_finite_trace"])
        infinite = [1.0] * 39 + [float("inf")]
        self.assertTrue(
            fit_retention.retention(cell(epoch_mean_loss=infinite))["non_finite_trace"])

    def test_a_cell_with_no_usable_trace_is_skipped_not_guessed(self):
        self.assertIsNone(fit_retention.retention(cell(epoch_mean_loss=None)))
        self.assertIsNone(fit_retention.retention(cell(epoch_mean_loss=[1.0, 2.0])))


class ConfigurationTest(unittest.TestCase):

    def test_seeds_group_together_and_read_out_depths_do_not(self):
        a = fit_retention.configuration(cell())
        b = fit_retention.configuration(cell(accuracy=0.9))
        self.assertEqual(a, b, "two seeds of one arm must group")
        c = fit_retention.configuration(cell(attn_layers=2))
        self.assertNotEqual(a, c, "L2 and L4 are different experiments")

    def test_the_rate_arm_is_not_labelled_with_an_attention_depth(self):
        label = fit_retention.configuration(cell(arm="ff+fixed"))
        self.assertIn("rate", label)
        self.assertNotIn("d32", label)

    def test_the_temporal_condition_separates_arms(self):
        intact = fit_retention.configuration(cell())
        shuffled = fit_retention.configuration(cell(temporal_condition="bin-shuffled"))
        self.assertNotEqual(intact, shuffled,
                            "an intact arm and its shuffled twin are not one group")


class CorpusTest(unittest.TestCase):
    """Reading the corpus, and refusing to read too little of it."""

    def _run(self, cells, argv):
        with tempfile.TemporaryDirectory() as tmp:
            corpus = Path(tmp)
            for i, c in enumerate(cells):
                (corpus / f"c{i}.json").write_text(json.dumps(c))
            # A list, which the corpus directory really does contain: analyser
            # output and manifests live beside the cells.
            (corpus / "manifest.json").write_text(json.dumps([1, 2, 3]))
            original, fit_retention.CORPUS = fit_retention.CORPUS, corpus
            saved, sys.argv = sys.argv, argv
            try:
                out = io.StringIO()
                with redirect_stdout(out):
                    code = fit_retention.main()
                return code, out.getvalue()
            finally:
                fit_retention.CORPUS = original
                sys.argv = saved

    def test_too_few_cells_is_refused_rather_than_summarised(self):
        """The check that makes the rest trustworthy: a confident table over
        three cells is worse than no table."""
        code, out = self._run([cell()] * 3, ["fit_retention.py", "--all-widths"])
        self.assertEqual(code, 1)
        self.assertIn("below the floor", out)

    def test_a_non_dict_json_file_is_skipped_not_crashed_on(self):
        code, out = self._run([cell()] * 25, ["fit_retention.py", "--all-widths"])
        self.assertEqual(code, 0, out)

    def test_the_output_says_it_is_post_hoc(self):
        """It is analysis of cells that already existed. Transcribing it as a
        registered verdict is the specific misuse to prevent."""
        _, out = self._run([cell()] * 25, ["fit_retention.py", "--all-widths"])
        self.assertIn("POST-HOC", out)
        self.assertIn("not a registered verdict", out)

    def test_the_width_filter_selects(self):
        cells = [cell()] * 25 + [cell(hidden=128)] * 25
        _, out = self._run(cells, ["fit_retention.py", "--width", "128"])
        self.assertIn("h128", out)
        self.assertIn(" 25 ", out)


if __name__ == "__main__":
    unittest.main(verbosity=2)
