"""Tests for the AWS campaign tooling — the code that produced every cell.

`scripts/aws/` planned, scheduled, ran, collected and analysed 720 cells, and
every number in the attention campaign came through it. Until 2026-08-22 it had
**no tests at all**: the layer that generates the evidence had no check on itself,
which is the same shape of gap as a gradient reference nobody verified learns.

The tests that matter most here are the negative ones. Most of this file is about
what the tooling must still **refuse**: a plan whose ids disagree with their own
specs, a duplicated cell, a reused control that drifted on disk, a campaign whose
binary changed mid-flight, and a degenerate cell presented as a measurement.

Run: python3 scripts/test_campaign_tooling.py
"""

from __future__ import annotations

import json
import re
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(ROOT / "scripts" / "aws"))

import plan_cells  # noqa: E402
from plan_cells import cell, estimated_seconds  # noqa: E402


class PlanIdentityTest(unittest.TestCase):
    """A cell's id is its only record of which seed produced it.

    The emitted cell JSON carries every parameter **except the seed** — the seed
    lives only in the filename. So every paired statistic in the campaign
    ("positive in 12 of 12 seeds", M-1's per-seed deltas) rests on the id
    encoding its own spec. Nothing inside a cell file can confirm that, which is
    why it is pinned here instead.
    """

    def all_planned(self):
        for name, generator in plan_cells.WAVES.items():
            for spec in generator():
                yield name, spec

    def test_every_id_encodes_its_own_seed(self):
        for name, spec in self.all_planned():
            found = re.search(r"__s(\d+)$", spec["id"])
            self.assertIsNotNone(found, f"{name}: id has no seed suffix: {spec['id']}")
            self.assertEqual(
                int(found.group(1)), spec["seed"],
                f"{name}: id says seed {found.group(1)}, spec says {spec['seed']}",
            )

    def test_every_id_encodes_its_own_width_and_budget(self):
        for name, spec in self.all_planned():
            for field, pattern in (("hidden", r"__h(\d+)__"), ("epochs", r"__e(\d+)__")):
                found = re.search(pattern, spec["id"])
                self.assertIsNotNone(found, f"{name}: id lacks {field}: {spec['id']}")
                self.assertEqual(int(found.group(1)), spec[field],
                                 f"{name}: id/{field} disagree in {spec['id']}")

    def test_every_id_encodes_its_contract_and_geometry(self):
        for name, spec in self.all_planned():
            self.assertIn(spec["contract"], spec["id"], f"{name}: {spec['id']}")
            self.assertIn(spec["geometry"], spec["id"], f"{name}: {spec['id']}")

    def test_ids_are_unique_within_every_wave(self):
        for name, generator in plan_cells.WAVES.items():
            ids = [c["id"] for c in generator()]
            self.assertEqual(len(ids), len(set(ids)), f"{name} plans a duplicate id")

    def test_attention_cells_say_so_in_their_id(self):
        """A d32/L4 cell and a rate-only cell must never share an id."""
        for name, spec in self.all_planned():
            if spec["attn_dim"] is not None:
                self.assertIn(f"d{spec['attn_dim']}l{spec['attn_layers']}", spec["id"])
            else:
                self.assertIsNone(re.search(r"__d\d+l\d+", spec["id"]),
                                  f"{name}: rate-only cell carries an attention tag")

    def test_a_shuffled_cell_is_never_confusable_with_an_intact_one(self):
        intact = cell("t", "ff+fixed+attn", 128, 400, 5170001, attn_dim=32, attn_layers=4)
        shuffled = cell("t", "ff+fixed+attn", 128, 400, 5170001,
                        attn_dim=32, attn_layers=4, temporal="bin-shuffled")
        self.assertNotEqual(intact["id"], shuffled["id"])
        self.assertIn("bin-shuffled", shuffled["id"])
        # The shuffle needs its own seed; the intact arm must not carry one, or
        # the two would differ in state the id does not describe.
        self.assertIsNotNone(shuffled["temporal_seed"])
        self.assertIsNone(intact["temporal_seed"])


class SchedulingCostTest(unittest.TestCase):
    """`estimated_seconds` only orders the queue, but a sign error there buries
    the decision-relevant wave behind the expensive one — which has happened once
    already (wave 6 at plan index 336 of 468)."""

    def base(self, **kw):
        return cell("t", "ff+fixed+attn", 128, 400, 5170001,
                    attn_dim=32, attn_layers=4, **kw)

    def test_cost_rises_with_every_cost_driver(self):
        ref = estimated_seconds(self.base())
        self.assertGreater(
            estimated_seconds(cell("t", "ff+fixed+attn", 128, 800, 5170001,
                                   attn_dim=32, attn_layers=4)), ref, "epochs")
        self.assertGreater(
            estimated_seconds(cell("t", "ff+fixed+attn", 1024, 400, 5170001,
                                   attn_dim=32, attn_layers=4)), ref, "hidden")
        self.assertGreater(
            estimated_seconds(cell("t", "ff+fixed+attn", 128, 400, 5170001,
                                   attn_dim=64, attn_layers=4)), ref, "attn_dim")
        self.assertGreater(
            estimated_seconds(cell("t", "ff+fixed+attn", 128, 400, 5170001,
                                   attn_dim=32, attn_layers=8)), ref, "attn_layers")

    def test_attention_costs_more_than_the_arm_it_extends(self):
        rate = estimated_seconds(cell("t", "ff+fixed", 128, 400, 5170001))
        self.assertGreater(estimated_seconds(self.base()), rate)

    def test_more_timesteps_cost_more(self):
        t100 = estimated_seconds(self.base(contract="fixed-t100"))
        t500 = estimated_seconds(self.base(contract="fixed-t500"))
        self.assertGreater(t500, t100, "a 5x finer contract must not look cheaper")


class ValidityGateTest(unittest.TestCase):
    """Each gate must catch its own defect class, and a healthy cell must pass."""

    def healthy(self):
        return {"non_finite_events": 0, "classes_predicted": 20,
                "majority_prediction": 0.11, "silent_fraction": 0.02,
                "saturated_fraction": 0.0}

    def problems(self, **override):
        import analyse_wave8
        payload = self.healthy()
        payload.update(override)
        return analyse_wave8.validity_problems(payload)

    def test_a_healthy_cell_reports_no_problem(self):
        self.assertEqual(self.problems(), [])

    def test_each_defect_class_is_caught(self):
        for field, bad in (("non_finite_events", 1),
                           ("classes_predicted", 19),
                           ("majority_prediction", 0.31),
                           ("silent_fraction", 0.96),
                           ("saturated_fraction", 0.06)):
            with self.subTest(field=field):
                self.assertTrue(self.problems(**{field: bad}),
                                f"{field}={bad} slipped through the gate")

    def test_a_collapsed_readout_cannot_pass(self):
        """The AZ8-6 shape: 9 classes predicted, 83% in one of them."""
        self.assertTrue(self.problems(classes_predicted=9, majority_prediction=0.826))


class ReusedControlTest(unittest.TestCase):
    """Waves 8-10 reuse 96 archived controls rather than re-running them. That is
    only legitimate while the binary is the same and the files have not moved."""

    def test_pinned_binary_must_match_across_campaigns(self):
        import analyse_wave8
        v1 = json.loads((ROOT / analyse_wave8.V1_MANIFEST).read_text())["binary_sha256"]
        v2 = json.loads((ROOT / analyse_wave8.V2_MANIFEST).read_text())["pinned_binary_sha256"]
        self.assertEqual(v1, v2, "reused controls came from a different instrument")

    def test_a_drifted_reused_cell_is_refused(self):
        """Negative test: the guard must reject a cell whose hash moved."""
        import analyse_wave8
        with tempfile.TemporaryDirectory() as tmp:
            d = Path(tmp) / "shd_attention_campaign_v1" / "cells"
            d.mkdir(parents=True)
            name = "fake__s5170001.json"
            (d / name).write_text(json.dumps({"accuracy": 0.5}))
            analyse_wave8._V1_HASHES = {name: "0" * 64}   # a hash it cannot match
            with self.assertRaises(SystemExit):
                analyse_wave8.load(str(d), "fake")
            analyse_wave8._V1_HASHES = None

    def test_every_archived_cell_matches_its_recorded_hash(self):
        import hashlib
        man = json.loads((ROOT / "results/shd_attention_campaign_v1/manifest.json").read_text())
        root = ROOT / "results/shd_attention_campaign_v1/cells"
        checked = 0
        for name, recorded in man["cells"].items():
            path = root / name
            if not path.is_file():
                continue
            actual = hashlib.sha256(path.read_bytes()).hexdigest()
            self.assertEqual(actual, recorded, f"{name} drifted on disk")
            checked += 1
        self.assertGreater(checked, 500, "the archive shrank - check before trusting this")


class ClaimProtocolTest(unittest.TestCase):
    """`claim_next.py` decides what every worker runs. Its failure modes are
    silent by nature: hand the same cell to two workers, or make a transient
    S3 error look like an empty queue and shut a healthy fleet down early.

    The real conditional PUT is S3's to guarantee. What is tested here is the
    decision logic around it, by substituting the subprocess boundary.
    """

    def setUp(self):
        import claim_next
        self.mod = claim_next
        self.plan = [{"id": f"cell-{i}"} for i in range(5)]

    def run_main(self, listings, put_results):
        """Drive main() with scripted `aws` responses. Returns (stdout, puts)."""
        import io
        import contextlib
        puts = []

        def fake_run(argv, **kwargs):
            class R:
                pass
            r = R()
            r.returncode = 0
            r.stderr = ""
            if argv[1] == "s3api" and argv[2] == "list-objects-v2":
                prefix = argv[argv.index("--prefix") + 1]
                payload = listings.get(prefix)
                if payload is None:
                    r.returncode = 1
                    r.stderr = "transient"
                    r.stdout = ""
                else:
                    r.stdout = json.dumps(
                        {"Contents": [{"Key": prefix + k} for k in payload]})
                return r
            if argv[1] == "s3api" and argv[2] == "put-object":
                key = argv[argv.index("--key") + 1]
                puts.append(key)
                r.returncode = 0 if put_results.pop(0) else 1
                r.stdout = ""
                return r
            raise AssertionError(f"unexpected call {argv}")

        with tempfile.NamedTemporaryFile("w", suffix=".json", delete=False) as fh:
            json.dump(self.plan, fh)
            plan_path = fh.name
        original, self.mod.subprocess.run = self.mod.subprocess.run, fake_run
        try:
            buf = io.StringIO()
            with contextlib.redirect_stdout(buf):
                self.mod.main.__wrapped__() if hasattr(self.mod.main, "__wrapped__") else \
                    self._invoke(plan_path)
            return buf.getvalue().strip(), puts
        finally:
            self.mod.subprocess.run = original
            Path(plan_path).unlink()

    def _invoke(self, plan_path):
        old = sys.argv
        sys.argv = ["claim_next.py", "bkt", "--plan", plan_path]
        try:
            self.mod.main()
        finally:
            sys.argv = old

    def test_a_transient_list_failure_raises_instead_of_claiming_everything(self):
        """The comment in `keys()` names this exact hazard. Pin it.

        If a failed listing returned an empty set, every cell would look
        unclaimed and every worker would race for cell 0."""
        with self.assertRaises(SystemExit):
            self.run_main(listings={}, put_results=[True])

    def test_already_finished_and_already_claimed_cells_are_skipped(self):
        out, puts = self.run_main(
            listings={"results/": ["cell-0.json", "cell-1.json"],
                      "claims/": ["cell-2"]},
            put_results=[True])
        self.assertEqual(out, "cell-3", "skipped past done and held to the first free cell")
        self.assertEqual(puts, ["claims/cell-3"], "must not PUT a cell it already knows is taken")

    def test_losing_a_race_moves_on_rather_than_giving_up(self):
        """A failed conditional PUT means another worker won, not that the
        queue is empty."""
        out, puts = self.run_main(
            listings={"results/": [], "claims/": []},
            put_results=[False, False, True])
        self.assertEqual(out, "cell-2")
        self.assertEqual(puts, ["claims/cell-0", "claims/cell-1", "claims/cell-2"])

    def test_a_drained_queue_prints_nothing(self):
        out, puts = self.run_main(
            listings={"results/": [f"cell-{i}.json" for i in range(5)], "claims/": []},
            put_results=[])
        self.assertEqual(out, "", "an empty queue must print nothing, not a stale id")
        self.assertEqual(puts, [], "nothing left to claim means no PUT at all")

    def test_a_claimed_cell_is_never_handed_out_twice_in_one_pass(self):
        out, _ = self.run_main(
            listings={"results/": [], "claims/": ["cell-0", "cell-1", "cell-2", "cell-3"]},
            put_results=[True])
        self.assertEqual(out, "cell-4")


class CollectAndTeardownTest(unittest.TestCase):
    """`collect.py` decides what "done" means and `teardown.py` decides what gets
    destroyed. Both are one substituted subprocess boundary away from testable,
    and both have failure modes that are silent rather than loud: a paginated
    listing that stops early undercounts a finished campaign, and a teardown
    whose filter is wrong either spares a burning fleet or terminates something
    that is not ours.
    """

    def scripted(self, module, responses):
        """Replace the subprocess boundary with a scripted transcript."""
        calls = []

        def fake_run(argv, **kwargs):
            class R:
                pass
            r = R()
            calls.append(argv)
            key = next((k for k in responses if k in " ".join(argv)), None)
            if key is None:
                r.returncode, r.stdout, r.stderr = 0, "", ""
            else:
                payload = responses[key]
                if payload is FAIL:
                    r.returncode, r.stdout, r.stderr = 1, "", "boom"
                else:
                    r.returncode, r.stdout, r.stderr = 0, json.dumps(payload), ""
            return r

        original, module.subprocess.run = module.subprocess.run, fake_run
        return original, calls

    def test_collect_follows_pagination_instead_of_stopping_at_page_one(self):
        """A campaign larger than one S3 page must not report itself finished."""
        import collect
        pages = iter([
            {"Contents": [{"Key": "results/a.json"}], "NextToken": "more"},
            {"Contents": [{"Key": "results/b.json"}]},
        ])

        def fake_run(argv, **kwargs):
            class R:
                pass
            r = R()
            r.returncode, r.stderr = 0, ""
            r.stdout = json.dumps(next(pages))
            return r

        original, collect.subprocess.run = collect.subprocess.run, fake_run
        try:
            found = collect.keys("bkt", "results/")
        finally:
            collect.subprocess.run = original
        self.assertEqual(found, {"a.json", "b.json"},
                         "the second page was dropped - a finished campaign would read as partial")

    def test_collect_raises_on_an_aws_failure_rather_than_reporting_zero(self):
        """An empty result set and a failed call must never look the same."""
        import collect
        original, _ = self.scripted(collect, {"list-objects-v2": FAIL})
        try:
            with self.assertRaises(SystemExit):
                collect.keys("bkt", "results/")
        finally:
            collect.subprocess.run = original

    def test_teardown_only_ever_targets_tagged_campaign_instances(self):
        """The blast radius is the whole point. The describe call must filter by
        the campaign tag, and terminate must receive exactly what it returned."""
        import teardown
        described = {"Reservations": [{"Instances": [
            {"InstanceId": "i-aaa"}, {"InstanceId": "i-bbb"}]}]}
        original, calls = self.scripted(teardown, {"describe-instances": described})
        try:
            old_argv, sys.argv = sys.argv, ["teardown.py", "--bucket", "bkt"]
            try:
                teardown.main()
            finally:
                sys.argv = old_argv
        finally:
            teardown.subprocess.run = original

        describe = next(c for c in calls if "describe-instances" in c)
        self.assertTrue(
            any(f"Values={teardown.TAG}" in part for part in describe),
            "describe-instances is not filtered by the campaign tag")
        terminate = next((c for c in calls if "terminate-instances" in c), None)
        self.assertIsNotNone(terminate, "instances were found but never terminated")
        self.assertEqual([p for p in terminate if p.startswith("i-")], ["i-aaa", "i-bbb"],
                         "terminate must receive exactly the described instances")

    def test_teardown_never_deletes_the_results_bucket(self):
        """The bucket holds the evidence. `--bucket` is documented as
        'printed for reference; never deleted' - hold it to that."""
        import teardown
        original, calls = self.scripted(teardown, {"describe-instances": {"Reservations": []}})
        try:
            old_argv, sys.argv = sys.argv, ["teardown.py", "--bucket", "bkt"]
            try:
                teardown.main()
            finally:
                sys.argv = old_argv
        finally:
            teardown.subprocess.run = original
        flat = [" ".join(c) for c in calls]
        self.assertFalse([c for c in flat if "rb" in c.split() or "delete-bucket" in c],
                         f"teardown attempted to remove a bucket: {flat}")


#: Sentinel for a scripted AWS call that fails.
FAIL = object()


if __name__ == "__main__":
    unittest.main(verbosity=2)
