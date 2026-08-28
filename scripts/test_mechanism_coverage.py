#!/usr/bin/env python3
"""Tests for `scripts/mechanism_coverage.py`.

This report exists to state a scope limit the paper would otherwise overstate.
A coverage checker that counts a point as covered when it is not is worse than
no checker: it converts "nobody derived this" into "we derived it and we are
fine".
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

spec = importlib.util.spec_from_file_location(
    "mechanism_coverage", ROOT / "scripts/mechanism_coverage.py")
MC = importlib.util.module_from_spec(spec)
spec.loader.exec_module(MC)


def cell(arm="ff+fixed", temporal="intact", hidden=128, **over):
    out = {
        "arm": arm, "hidden": hidden, "contract": "published-2ms",
        "geometry": "adjacent-sum-5", "epochs": 400, "surrogate_scale": 1.0,
        "clip_grad_norm": None, "temporal_condition": temporal,
        "accuracy": 0.5,
    }
    out.update(over)
    return out


class CoverageTest(unittest.TestCase):
    def setUp(self):
        self.dir = Path(tempfile.mkdtemp())

    def write(self, name, body):
        (self.dir / f"{name}.json").write_text(json.dumps(body))

    def rows(self):
        return MC.coverage(MC.read([self.dir]))

    def four_arms(self, seeds=(1, 2), depth="d32l4", **kw):
        for s in seeds:
            self.write(f"w1__ff-fixed__h128__s{s}", cell(**kw))
            self.write(f"w9__ff-fixed__h128__bin-shuffled__s{s}",
                       cell(temporal="bin-shuffled", **kw))
            self.write(f"w1__ff-fixed-attn__h128__{depth}__s{s}",
                       cell(arm="ff+fixed+attn", **kw))
            self.write(f"w9__ff-fixed-attn__h128__{depth}__bin-shuffled__s{s}",
                       cell(arm="ff+fixed+attn", temporal="bin-shuffled", **kw))

    def test_all_four_arms_on_shared_seeds_is_covered(self):
        self.four_arms()
        self.assertEqual([r[6] for r in self.rows()], [2])

    def test_a_missing_shuffled_RATE_arm_is_not_covered(self):
        """The easy bug: check the attention arms and forget the contrast needs
        the rate read-out's own shuffle cost too. Without it there is a drop,
        not a difference of differences."""
        self.four_arms()
        for s in (1, 2):
            (self.dir / f"w9__ff-fixed__h128__bin-shuffled__s{s}.json").unlink()
        self.assertEqual([r[6] for r in self.rows()], [0])

    def test_a_missing_shuffled_attention_arm_is_not_covered(self):
        self.four_arms()
        for s in (1, 2):
            (self.dir /
             f"w9__ff-fixed-attn__h128__d32l4__bin-shuffled__s{s}.json").unlink()
        self.assertEqual([r[6] for r in self.rows()], [0])

    def test_seeds_must_be_shared_not_merely_counted(self):
        """Four arms of two cells each, no seed present in all four."""
        self.write("w1__ff-fixed__h128__s1", cell())
        self.write("w9__ff-fixed__h128__bin-shuffled__s2",
                   cell(temporal="bin-shuffled"))
        self.write("w1__ff-fixed-attn__h128__d32l4__s3", cell(arm="ff+fixed+attn"))
        self.write("w9__ff-fixed-attn__h128__d32l4__bin-shuffled__s4",
                   cell(arm="ff+fixed+attn", temporal="bin-shuffled"))
        self.assertEqual([r[6] for r in self.rows()], [0])

    def test_channel_shuffled_does_not_count_as_bin_shuffled(self):
        """A different destruction operator answers a different question."""
        self.four_arms()
        for s in (1, 2):
            path = self.dir / f"w9__ff-fixed__h128__bin-shuffled__s{s}.json"
            path.write_text(json.dumps(cell(temporal="channel-shuffled")))
        self.assertEqual([r[6] for r in self.rows()], [0])

    def test_the_rate_arm_pairs_across_every_readout_depth(self):
        """The rate read-out has no depth. One rate arm serves d32l1 and d32l4,
        and counting it for only one of them would understate coverage."""
        self.four_arms(depth="d32l4")
        for s in (1, 2):
            self.write(f"w1__ff-fixed-attn__h128__d32l1__s{s}",
                       cell(arm="ff+fixed+attn"))
            self.write(f"w9__ff-fixed-attn__h128__d32l1__bin-shuffled__s{s}",
                       cell(arm="ff+fixed+attn", temporal="bin-shuffled"))
        self.assertEqual(sorted(r[6] for r in self.rows()), [2, 2])

    def test_a_clipped_cell_does_not_join_an_unclipped_point(self):
        """`w15col` ran clip1000.0 arms. They are a different configuration."""
        self.four_arms()
        for s in (1, 2):
            path = self.dir / f"w9__ff-fixed__h128__bin-shuffled__s{s}.json"
            path.write_text(json.dumps(
                cell(temporal="bin-shuffled", clip_grad_norm=1000.0)))
        self.assertEqual([r[6] for r in self.rows()], [0])

    def test_a_different_width_is_a_different_operating_point(self):
        self.four_arms()
        for s in (1, 2):
            self.write(f"w1__ff-fixed-attn__h256__d32l4__s{s}",
                       cell(arm="ff+fixed+attn", hidden=256))
        rows = self.rows()
        self.assertEqual(len(rows), 2)
        self.assertEqual(sorted(r[6] for r in rows), [0, 2])

    def test_a_missing_root_is_fatal_not_empty(self):
        """A report that could not run must not read like one that found
        nothing."""
        with self.assertRaises(SystemExit):
            MC.read([self.dir / "does-not-exist"])

    def test_zero_coverage_exits_one(self):
        self.write("w1__ff-fixed-attn__h128__d32l4__s1", cell(arm="ff+fixed+attn"))
        self.write("w1__ff-fixed__h128__s1", cell())
        buf = io.StringIO()
        old, sys.argv = sys.argv, ["x", "--results", str(self.dir)]
        try:
            with contextlib.redirect_stdout(buf):
                # DEFAULT_ROOTS are real, so point them at the fixture only.
                real, MC.DEFAULT_ROOTS = MC.DEFAULT_ROOTS, ()
                try:
                    code = MC.main()
                finally:
                    MC.DEFAULT_ROOTS = real
        finally:
            sys.argv = old
        self.assertEqual(code, 1, buf.getvalue())
        self.assertIn("not a pass", buf.getvalue())


def flat(arm="ff+fixed", temporal="intact", hidden=128, depth="d32l4", seed=1):
    """A cell in the shape `read()` returns, which is what `coverage` consumes."""
    return {"seed": seed, "hidden": hidden, "contract": "published-2ms",
            "geometry": "adjacent-sum-5", "epochs": 400, "depth": depth,
            "arm": arm, "surrogate_scale": 1.0, "clip_grad_norm": None,
            "temporal": temporal}


def entry(arm="ff+fixed", temporal="intact", hidden=128, attn=(32, 4), seed=1):
    """A cell in the shape a published `cells.json` carries."""
    return {"seed": seed, "hidden": hidden, "contract": "published-2ms",
            "geometry": "adjacent-sum-5", "epochs": 400, "arm": arm,
            "attn_dim": attn[0] if attn else None,
            "attn_layers": attn[1] if attn else None,
            "surrogate_scale": None, "clip_grad_norm": None,
            "temporal": temporal}


class ProjectionTest(unittest.TestCase):
    """`--plan`: will the compute already committed buy what it was registered
    to buy?

    Measured coverage answers what the paper can say today. It cannot answer
    whether a queued wave will close the gap it was registered to close, and
    that question is answerable the moment the wave is queued rather than after
    its last cell lands. Wave 21 is 168 cells and roughly 300 slot-hours; a wave
    whose geometry token or read-out depth did not line up with the intact arms
    it must pair against would produce nothing, and would say so only at the end.
    """

    def project(self, cells, entries):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "cells.json"
            path.write_text(json.dumps(entries))
            return "\n".join(MC.planned(path, cells))

    def test_a_wave_that_completes_a_point_is_projected(self):
        """Intact arms exist at h256; the queued cells supply the shuffled
        halves, so the point becomes computable."""
        have = [flat(a, "intact", 256, d, s)
                for a, d in (("ff+fixed", None), ("ff+fixed+attn", "d32l4"))
                for s in range(3)]
        want = [entry(a, "bin-shuffled", 256, t, s)
                for a, t in (("ff+fixed", None), ("ff+fixed+attn", (32, 4)))
                for s in range(3)]
        out = self.project(have, want)
        self.assertIn("0 → 1 operating point", out)
        self.assertIn("+ h256", out)

    def test_a_wave_that_buys_nothing_raises_the_alarm(self):
        """The case this exists for. Queued cells that cannot pair with the
        intact arms they were registered against add no operating point, and
        saying so before the compute is spent is the whole value."""
        have = [flat(a, "intact", 256, d, s)
                for a, d in (("ff+fixed", None), ("ff+fixed+attn", "d32l4"))
                for s in range(3)]
        # Right destruction, wrong geometry: nothing to pair against.
        want = [dict(entry(a, "bin-shuffled", 256, t, s), geometry="channels-700")
                for a, t in (("ff+fixed", None), ("ff+fixed+attn", (32, 4)))
                for s in range(3)]
        out = self.project(have, want)
        self.assertIn("NO new operating point", out)
        self.assertIn("BEFORE its compute is spent", out)

    def test_a_wave_covering_ground_already_covered_raises_the_alarm(self):
        """Not every no-op is a mis-specification; re-running a covered point
        is also worth knowing before paying for it."""
        have = ([flat(a, t, 128, d, s)
                 for a, d in (("ff+fixed", None), ("ff+fixed+attn", "d32l4"))
                 for t in ("intact", "bin-shuffled") for s in range(3)])
        want = [entry(a, "bin-shuffled", 128, t, s + 100)
                for a, t in (("ff+fixed", None), ("ff+fixed+attn", (32, 4)))
                for s in range(3)]
        self.assertIn("NO new operating point", self.project(have, want))

    def test_the_projection_uses_the_real_pairing_rule(self):
        """It runs through `coverage` itself. A second implementation of the
        pairing rule would drift, and the projection would then reassure about
        a pairing the analyser does not perform."""
        source = (ROOT / "scripts/mechanism_coverage.py").read_text()
        body = source[source.index("def planned("):source.index("def render(")]
        self.assertEqual(body.count("coverage("), 2)

    def test_an_unreadable_plan_is_fatal(self):
        """"I could not read the plan" must not look like "the plan adds
        nothing"."""
        with self.assertRaises(SystemExit):
            MC.planned(Path("/nonexistent/cells.json"), [])

    def test_the_rate_arm_pairs_across_depths(self):
        """The rate read-out has no read-out depth, so its queued cells have to
        serve every depth at the same width — the same rule `coverage` applies
        to measured cells."""
        have = [flat(a, "intact", 512, d, s)
                for a, d in (("ff+fixed", None), ("ff+fixed+attn", "d32l1"),
                             ("ff+fixed+attn", "d32l4"))
                for s in range(3)]
        want = ([entry("ff+fixed", "bin-shuffled", 512, None, s) for s in range(3)]
                + [entry("ff+fixed+attn", "bin-shuffled", 512, (32, d), s)
                   for d in (1, 4) for s in range(3)])
        out = self.project(have, want)
        self.assertIn("0 → 2 operating point", out)


class LiveCorpusTest(unittest.TestCase):
    """What the real corpus says, pinned so a silent narrowing is visible."""

    def test_the_contrast_exists_at_exactly_one_width(self):
        rows = MC.coverage(MC.read(MC.DEFAULT_ROOTS))
        widths = sorted({r[0][0] for r in rows if r[6]})
        self.assertEqual(widths, [128],
                         "coverage moved; PAPER_DRAFT's scope limit and "
                         "PREREG ...THE_MECHANISM_CONTROL... need rereading")

    def test_most_operating_points_have_no_control_at_all(self):
        rows = MC.coverage(MC.read(MC.DEFAULT_ROOTS))
        covered = [r for r in rows if r[6]]
        self.assertLess(len(covered), len(rows) / 2)


if __name__ == "__main__":
    unittest.main(verbosity=2)
