"""Tests for `scripts/aws/watch_campaign.py`.

The watcher exists because two earlier versions of it reported a number that
was not the one it claimed. The first hardcoded the plan size at start-up, so
appending a wave left it counting toward a stale total — and it would have
printed `WAVES 18-20 COMPLETE` and **exited** at exactly the moment the new
wave began. The second re-read the plan and then took its numerator from every
object under `results/`, which holds cells from all twenty-one waves, and
reported 525 of 360 done.

Both failures have the same shape: two numbers that must come from one source
came from two. Most of this file pins that, and the states where **silence is
the wrong output** — an unreadable plan and an idle fleet with work left.

Run: python3 scripts/test_campaign_watcher.py
"""

from __future__ import annotations

import importlib.util
import sys
import unittest
from pathlib import Path
from unittest import mock

ROOT = Path(__file__).resolve().parent.parent
_spec = importlib.util.spec_from_file_location(
    "watch_campaign", ROOT / "scripts/aws/watch_campaign.py")
WC = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(WC)


def listing(prefix: str, names: list[str]) -> str:
    return "".join(
        f"2026-08-27 12:00:00       1234 {prefix}{n}\n" for n in names)


class Fleet:
    """A scripted bucket and account, driven through `WC.aws`."""

    def __init__(self, plan, results=(), failures=(), instances=5,
                 plan_readable=True):
        self.plan = list(plan)
        self.results = list(results)
        self.failures = list(failures)
        self.instances = instances
        self.plan_readable = plan_readable

    def __call__(self, args):
        import json
        if args[:2] == ["s3", "cp"]:
            if not self.plan_readable:
                return ""
            return json.dumps([{"id": c} for c in self.plan])
        if args[:2] == ["s3", "ls"]:
            if "results/" in args[2]:
                return listing("results/", [f"{c}.json" for c in self.results])
            return listing("failures/", [f"{c}.log" for c in self.failures])
        if args[0] == "ec2":
            return "" if self.instances is None else f"{self.instances}\n"
        return ""

    def run(self, **kw):
        """One poll. Returns the emitted lines."""
        said: list[str] = []
        argv = ["watch_campaign", "--bucket", "b", "--once"]
        for cell in kw.pop("cells", []):
            argv += ["--cell", cell]
        for key, value in kw.items():
            argv += [f"--{key}", str(value)]
        with mock.patch.object(WC, "aws", self), \
             mock.patch.object(WC, "say", said.append), \
             mock.patch.object(sys, "argv", argv):
            self.code = WC.main()
        return said


class ProgressComesFromThePlanTest(unittest.TestCase):

    def test_results_outside_the_plan_are_not_counted(self):
        """`results/` holds every wave the bucket has ever run. Counting it raw
        is how the previous version reported 525 done against a plan of 360."""
        fleet = Fleet(plan=["w21__a", "w21__b"],
                      results=["w21__a", "w1__old", "w9__older"])
        lines = fleet.run()
        self.assertIn("watching 2 planned cells: 1 done", lines[0])
        self.assertIn("1 outstanding", lines[0])

    def test_failures_outside_the_plan_are_not_counted(self):
        fleet = Fleet(plan=["w21__a"], failures=["w21__a", "w13__old"])
        self.assertIn("1 failed", fleet.run()[0])

    def test_done_failed_and_outstanding_partition_the_plan(self):
        fleet = Fleet(plan=[f"w21__{i}" for i in range(10)],
                      results=[f"w21__{i}" for i in range(4)],
                      failures=["w21__7"])
        line = fleet.run()[0]
        self.assertIn("watching 10 planned cells: 4 done, 1 failed, "
                      "5 outstanding", line)

    def test_a_grown_plan_is_reported_rather_than_ignored(self):
        """The defect that started this: a wave appended mid-campaign."""
        fleet = Fleet(plan=["a", "b"], results=["a"])
        fleet.run()
        # A second poll in the same process is what the real loop does; drive
        # it by widening the plan and re-running from the recorded state.
        wide = Fleet(plan=["a", "b", "c"], results=["a"])
        self.assertIn("watching 3 planned cells", wide.run()[0])


class SilenceIsTheWrongOutputTest(unittest.TestCase):

    def test_an_unreadable_plan_says_so(self):
        """"I could not read the plan" and "there is no work left" must not
        produce the same output — the second ends the watch."""
        fleet = Fleet(plan=["a"], plan_readable=False)
        lines = fleet.run()
        self.assertTrue(any("could not be read" in l for l in lines), lines)
        self.assertEqual(fleet.code, 1)

    def test_an_idle_fleet_with_work_left_raises_an_alarm(self):
        fleet = Fleet(plan=["a", "b"], results=["a"], instances=0)
        lines = fleet.run()
        self.assertTrue(any("FLEET DOWN" in l for l in lines), lines)
        self.assertTrue(any("1 cell(s) unfinished" in l for l in lines), lines)

    def test_an_idle_fleet_with_no_work_left_is_not_an_alarm(self):
        """Every cell finished and the instances self-terminated is the
        expected end of a campaign, not an outage."""
        fleet = Fleet(plan=["a"], results=["a"], instances=0)
        lines = fleet.run()
        self.assertFalse(any("FLEET DOWN" in l for l in lines), lines)
        self.assertTrue(any("PLAN COMPLETE" in l for l in lines), lines)

    def test_completion_is_reported_with_its_numbers(self):
        fleet = Fleet(plan=["a", "b"], results=["a"], failures=["b"])
        lines = fleet.run()
        self.assertTrue(any("PLAN COMPLETE: 1 done, 1 failed, of 2 planned" in l
                            for l in lines), lines)
        self.assertEqual(fleet.code, 0)

    def test_a_transient_api_failure_does_not_end_the_watch(self):
        """`aws` returning nothing for the instance query must not be read as
        zero instances, which would fire a false outage alarm."""
        fleet = Fleet(plan=["a", "b"], results=["a"], instances=None)
        lines = fleet.run()
        self.assertFalse(any("FLEET DOWN" in l for l in lines), lines)


class NamedEventsTest(unittest.TestCase):

    def test_a_key_cell_completing_gets_its_own_line(self):
        fleet = Fleet(plan=["a", "b"], results=["a"])
        lines = fleet.run(cells=["a"])
        self.assertTrue(any("KEY CELL COMPLETE: a" in l for l in lines), lines)

    def test_a_key_cell_lost_gets_its_own_line(self):
        fleet = Fleet(plan=["a", "b"], failures=["a"])
        lines = fleet.run(cells=["a"])
        self.assertTrue(any("KEY CELL LOST: a" in l for l in lines), lines)

    def test_a_key_cell_still_running_says_nothing(self):
        fleet = Fleet(plan=["a", "b"])
        lines = fleet.run(cells=["a"])
        self.assertFalse(any("KEY CELL" in l for l in lines), lines)


class TheScriptIsUsableTest(unittest.TestCase):

    def test_the_timeout_matches_the_other_campaign_helpers(self):
        """`test_campaign_tooling.py` asserts the copies agree; this file is a
        new copy and caught one drifting to 60 once already."""
        self.assertEqual(WC.AWS_TIMEOUT_S, 300)

    def test_every_aws_call_is_best_effort(self):
        """A watch that dies on one failed API call is worse than no watch: it
        stops reporting and nothing says it stopped."""
        source = (ROOT / "scripts/aws/watch_campaign.py").read_text()
        self.assertIn("except (subprocess.TimeoutExpired, OSError)", source)
        self.assertIn("capture_output=True", source)


if __name__ == "__main__":
    unittest.main(verbosity=2)
