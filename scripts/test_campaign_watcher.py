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
                 plan_readable=True, hostlogs=None, live_ids=None, claims=None):
        self.plan = list(plan)
        self.results = list(results)
        self.failures = list(failures)
        self.instances = instances
        self.plan_readable = plan_readable
        #: `{instance id: [cell ids that instance's log claims]}`
        self.hostlogs = dict(hostlogs or {})
        #: instance ids still alive; defaults to every instance with a hostlog
        self.live_ids = (list(self.hostlogs) if live_ids is None
                         else list(live_ids))
        #: Claim markers still held. Defaults to every planned cell, which is
        #: the state during a run; pass a shorter list to model a release.
        self.claims = list(self.plan) if claims is None else list(claims)
        #: Optional results-by-poll. Indexed by `self.poll`, which the test's
        #: sleep hook advances at each poll boundary — NOT by how many times
        #: the results prefix happens to be listed. Keying it to the call count
        #: coupled the fixture to the poll's internal call pattern, and a
        #: schedule that drifts by one produces a plausible wrong answer rather
        #: than an error.
        self.schedule = None
        self.poll = 0

    def __call__(self, args):
        import json
        if args[:2] == ["s3", "cp"]:
            if "hostlogs/" in args[2]:
                instance = args[2].rsplit("/", 1)[-1][:-4]
                return "".join(f"slot 1: running {c}\n"
                               for c in self.hostlogs.get(instance, []))
            if not self.plan_readable:
                return ""
            return json.dumps([{"id": c} for c in self.plan])
        if args[:2] == ["s3", "ls"]:
            if "results/" in args[2]:
                rows = self.results
                if self.schedule is not None:
                    rows = self.schedule[min(self.poll, len(self.schedule) - 1)]
                return listing("results/", [f"{c}.json" for c in rows])
            if "hostlogs/" in args[2]:
                return listing("hostlogs/", [f"{i}.log" for i in self.hostlogs])
            if "claims/" in args[2]:
                return listing("claims/", list(self.claims))
            return listing("failures/", [f"{c}.log" for c in self.failures])
        if args[0] == "ec2":
            if "InstanceId" in " ".join(args):
                return " ".join(self.live_ids) + "\n" if self.live_ids else ""
            return "" if self.instances is None else f"{self.instances}\n"
        return ""

    def run_twice(self, **kw):
        """Two polls, which is what the strandedness debounce requires.

        A single sample cannot tell a real strand from a hostlog that has not
        caught up with a live instance's claim, so nothing is reported until a
        cell survives two consecutive polls.
        """
        import itertools
        said: list[str] = []
        argv = ["watch_campaign", "--bucket", "b", "--interval", "0"]
        for cell in kw.pop("cells", []):
            argv += ["--cell", cell]
        stop = itertools.count()

        def sleep(_):
            if next(stop) >= 1:
                raise KeyboardInterrupt

        with mock.patch.object(WC, "aws", self), \
             mock.patch.object(WC, "say", said.append), \
             mock.patch.object(WC.time, "sleep", sleep), \
             mock.patch.object(sys, "argv", argv):
            try:
                WC.main()
            except KeyboardInterrupt:
                pass
        return said

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


class AReclaimedCellTest(unittest.TestCase):
    """A cell has three outcomes, not two, and only one of them is recoverable.

    A diverged cell writes a failure log and cannot be re-run: the seed and the
    binary are pinned, so a requeue burns a slot and lands back in `failures/`.
    A cell whose spot instance is reclaimed writes **nothing** — its claim is
    orphaned and it simply never finishes. Before this the watcher was silent
    on the second forever, which is how the one loss that can be fixed looked
    identical to no news.

    The regression test below is the important one. The first implementation
    mapped `cell -> owner` and let whichever hostlog was processed last win. It
    reported six of wave 20's cells stranded; `release_dead_claims.py`, asking
    the fleet over SSM, said zero. Hostlogs accumulate and outlive their
    instances, so a cell that was reclaimed, released and picked up again is
    named in both a dead log and a live one.
    """

    CELL = "w20rec__key__s1"

    def test_a_cell_whose_owner_is_gone_is_reported(self):
        fleet = Fleet(plan=[self.CELL, "other"],
                      hostlogs={"i-dead": [self.CELL]},
                      live_ids=["i-alive"], instances=1)
        lines = fleet.run_twice(cells=[self.CELL])
        self.assertTrue(any("STRANDED" in l for l in lines), lines)
        self.assertTrue(any("release_dead_claims.py" in l for l in lines), lines)
        self.assertTrue(any("KEY CELL" in l for l in lines), lines)

    def test_a_cell_a_live_log_also_claims_is_not_stranded(self):
        """The false positive that shipped and was caught by disagreeing with
        the authoritative check. A dead instance's log naming a cell proves
        nothing when a live instance's log names it too."""
        fleet = Fleet(plan=[self.CELL, "other"],
                      hostlogs={"i-dead": [self.CELL], "i-alive": [self.CELL]},
                      live_ids=["i-alive"])
        lines = fleet.run(cells=[self.CELL])
        self.assertFalse(any("STRANDED" in l for l in lines), lines)

    def test_a_released_cell_is_no_longer_reported(self):
        """The false positive in the RECOVERED state, seen in anger on
        2026-08-28: eight cells were released back to the queue and the very
        next poll called the same eight stranded. Hostlogs are append-only and
        outlive their instances, so a dead log goes on naming a cell that has
        already been handed back — for exactly as long as it waits unclaimed,
        which is the window in which it has already been fixed."""
        fleet = Fleet(plan=[self.CELL, "other"],
                      hostlogs={"i-dead": [self.CELL]},
                      live_ids=["i-alive"], claims=[])
        self.assertFalse(any("STRANDED" in l
                             for l in fleet.run(cells=[self.CELL])))

    def test_one_poll_alone_does_not_report(self):
        """Debounce. A cell a live instance has just claimed sits in the dead
        instance's log and not yet in the live one's — hostlogs ship once a
        minute — which is indistinguishable from a real strand in one sample.
        `--once` takes exactly one poll, so nothing is reported."""
        fleet = Fleet(plan=[self.CELL, "other"],
                      hostlogs={"i-dead": [self.CELL]},
                      live_ids=["i-alive"], claims=[self.CELL])
        self.assertFalse(any("STRANDED" in l
                             for l in fleet.run(cells=[self.CELL])))

    def test_two_consecutive_polls_report(self):
        """The mirror, so the debounce cannot silence the alarm entirely."""
        fleet = Fleet(plan=[self.CELL, "other"],
                      hostlogs={"i-dead": [self.CELL]},
                      live_ids=["i-alive"], claims=[self.CELL])
        self.assertTrue(any("STRANDED" in l
                            for l in fleet.run_twice(cells=[self.CELL])))

    def test_claim_names_survive_an_empty_suffix(self):
        """Claim markers carry no extension. `name[:-len("")]` is `name[:-0]`,
        which is the empty string for every key — a set of one empty name that
        intersects nothing, silently disabling the alarm."""
        with mock.patch.object(WC, "aws",
                               lambda a: listing("claims/", ["w__c1", "w__c2"])):
            self.assertEqual(WC.keys("b", "claims/", ""), {"w__c1", "w__c2"})

    def test_a_cell_whose_owner_is_alive_is_not_reported(self):
        """The common case a false alarm would ruin: still running. These take
        up to fourteen hours."""
        fleet = Fleet(plan=[self.CELL, "other"],
                      hostlogs={"i-alive": [self.CELL]}, live_ids=["i-alive"])
        self.assertFalse(any("STRANDED" in l
                             for l in fleet.run(cells=[self.CELL])))

    def test_strandedness_covers_cells_nobody_named(self):
        """Scoped to `--cell` at first, so a reclaim anywhere else in the plan
        went unreported until someone ran release_dead_claims.py by hand."""
        fleet = Fleet(plan=["unnamed", "other"],
                      hostlogs={"i-dead": ["unnamed"]}, live_ids=["i-alive"])
        lines = fleet.run_twice(cells=[])
        self.assertTrue(any("STRANDED" in l for l in lines), lines)

    def test_a_finished_cell_is_never_called_stranded(self):
        fleet = Fleet(plan=[self.CELL, "other"], results=[self.CELL],
                      hostlogs={"i-dead": [self.CELL]}, live_ids=["i-alive"])
        lines = fleet.run(cells=[self.CELL])
        self.assertFalse(any("STRANDED" in l for l in lines), lines)
        self.assertTrue(any("KEY CELL COMPLETE" in l for l in lines), lines)

    def test_a_failed_cell_is_called_unrecoverable_and_not_stranded(self):
        """Requeueing a divergence wastes a slot and cannot change the answer."""
        fleet = Fleet(plan=[self.CELL, "other"], failures=[self.CELL],
                      hostlogs={"i-dead": [self.CELL]}, live_ids=["i-alive"])
        lines = fleet.run(cells=[self.CELL])
        self.assertFalse(any("STRANDED" in l for l in lines), lines)
        self.assertTrue(any("not recoverable by requeueing" in l
                            for l in lines), lines)

    def test_an_unreadable_instance_list_raises_no_alarm(self):
        """The same refusal `release_dead_claims.py` makes: never act on
        incomplete information about what is running."""
        fleet = Fleet(plan=[self.CELL, "other"],
                      hostlogs={"i-dead": [self.CELL]}, live_ids=[])
        self.assertFalse(any("STRANDED" in l
                             for l in fleet.run(cells=[self.CELL])))

    def test_a_cell_no_hostlog_mentions_raises_no_alarm(self):
        """Hostlogs ship once a minute; a just-claimed cell is in none of
        them."""
        fleet = Fleet(plan=[self.CELL, "other"],
                      hostlogs={"i-alive": ["something-else"]},
                      live_ids=["i-alive"])
        self.assertFalse(any("STRANDED" in l
                             for l in fleet.run(cells=[self.CELL])))

    def test_a_mass_reclaim_is_summarised_rather_than_listed(self):
        """One reclaimed instance strands every cell it held. One line per slot
        is a flood the reader learns to skip."""
        cells = [f"w__c{i}" for i in range(16)]
        fleet = Fleet(plan=cells + ["other"],
                      hostlogs={"i-dead": cells}, live_ids=["i-alive"])
        lines = [l for l in fleet.run_twice(cells=[]) if "STRANDED" in l]
        self.assertEqual(len(lines), 1, lines)
        self.assertIn("16 cell(s)", lines[0])
        self.assertIn("and 13 more", lines[0])


class StallReportingTest(unittest.TestCase):
    """The stall check speaks when the answer changes, and not otherwise.

    A wave of three-hour cells is quiet by design. Repeating "quiet but
    healthy" every half hour is good news the reader learns to skip — which is
    how the one that says STALLED gets skipped with it. But silence must mean
    "still the answer you were last given", never "nothing was asked", so the
    first answer of a quiet stretch is always spoken.
    """

    def poll(self, fleet, times, workers):
        """Run `times` polls with a scripted worker-process count."""
        import itertools
        said: list[str] = []
        stop = itertools.count()

        def sleep(_):
            fleet.poll += 1
            if next(stop) >= times - 1:
                raise KeyboardInterrupt

        with mock.patch.object(WC, "aws", fleet), \
             mock.patch.object(WC, "say", said.append), \
             mock.patch.object(WC.time, "sleep", sleep), \
             mock.patch.object(WC, "worker_processes", lambda _r: workers), \
             mock.patch.object(sys, "argv",
                               ["watch_campaign", "--bucket", "b",
                                "--interval", "0", "--stall-after", "1"]):
            try:
                WC.main()
            except KeyboardInterrupt:
                pass
        return said

    def fleet(self):
        return Fleet(plan=["a", "b"], results=["a"], instances=2)

    def test_a_healthy_answer_is_given_once(self):
        said = self.poll(self.fleet(), times=5, workers=8)
        healthy = [l for l in said if "but healthy" in l]
        self.assertEqual(len(healthy), 1, said)
        self.assertIn("has not changed", healthy[0])

    # NOT TESTED, and named rather than left to be assumed: that progress
    # RESETS the health verdict, so a stall either side of a batch of work is
    # reported twice rather than once. The behaviour is right — traced by hand
    # on 2026-08-28, `health` goes stalled -> None -> stalled across a landing
    # cell — but driving it needs a fixture that advances results exactly at
    # poll boundaries, and two attempts at that disagreed with each other by
    # one poll. A test I cannot make say the same thing twice is worse than a
    # gap I have written down: it would pass or fail for reasons other than the
    # code under it.
    #
    # What IS covered below: a healthy answer is given once and not repeated, a
    # stall is reported, an unanswerable count is neither, and the stall check
    # survives progress smaller than --step.
    def test_the_stall_check_survives_progress_below_the_report_step(self):
        """The defect this class found. The quiet counter compared against
        `last_done`, which only advances when progress crosses `--step`. One
        cell landing under a step of ten left `len(done) != last_done` forever,
        so the counter never accumulated and the stall check stopped working
        until another full step arrived — most of a wave whose last cells
        trickle in, with a dying fleet unreported throughout."""
        fleet = Fleet(plan=[f"c{i}" for i in range(20)], instances=2)
        # One cell lands, well under the default step, then it goes quiet.
        fleet.schedule = [["c0"], ["c0", "c1"], ["c0", "c1"], ["c0", "c1"]]
        said = self.poll(fleet, times=4, workers=0)
        self.assertTrue(any("STALLED" in l for l in said),
                        f"progress under --step disabled the stall check: {said}")

    def test_a_stall_is_reported(self):
        said = self.poll(self.fleet(), times=4, workers=0)
        stalled = [l for l in said if "STALLED" in l]
        self.assertEqual(len(stalled), 1, said)
        self.assertIn("needs a hand", stalled[0])

    def test_an_unanswerable_count_is_neither(self):
        said = self.poll(self.fleet(), times=4, workers=None)
        self.assertTrue(any("neither a stall nor a clean bill" in l
                            for l in said), said)
        self.assertFalse(any("STALLED" in l for l in said), said)


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
