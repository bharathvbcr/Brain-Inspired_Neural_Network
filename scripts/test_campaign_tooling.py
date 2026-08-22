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

import contextlib
import io
import json
import re
import sys
import tempfile
import types
import unittest
from datetime import datetime, timezone
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
        """A complete cell, not a partial one.

        This fixture used to omit `mechanical_status` and `temporal_condition`,
        which every real cell carries. Omitting them made the gate look
        satisfied on a payload the instrument never emits, and hid that two of
        the three copies of the gate were not checking them at all.
        """
        return {"mechanical_status": "COMPLETE",
                "non_finite_events": 0, "classes_predicted": 20,
                "majority_prediction": 0.11, "silent_fraction": 0.02,
                "saturated_fraction": 0.0,
                "temporal_condition": "intact",
                "epoch_max_gradient_norm": [0.4, 1.2, 0.9]}

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
                won = put_results.pop(0)
                r.returncode = 0 if won else 1
                # A losing PUT must look like what S3 actually returns. A bare
                # non-zero exit is an *error*, and since 2026-08-22 claim_next
                # refuses to mistake one for the other.
                r.stderr = "" if won else (
                    "An error occurred (PreconditionFailed) when calling the "
                    "PutObject operation")
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

    def test_a_gate_report_that_names_no_binary_is_reported_not_dropped(self):
        """The multi-binary warning is the check that protects reused controls,
        and it used to be the one thing a malformed gate report could silence.

        `collect.py` read `payload["binary_sha256"]` unguarded, so a report from
        an older bootstrap raised KeyError *after* the Gate F verdicts had
        printed - taking the binary check with it, and leaving a run that looked
        clean and simply had nothing to warn about. Two binaries in one campaign
        would then go unreported.

        Skipping such a report is not good enough either: the caller would print
        "single binary across the campaign", which is the strongest claim this
        script makes and the one that needs the most evidence.
        """
        import collect
        reports = {
            "i-old.json": {"instance": "i-old"},               # pre-dates the field
            "i-blank.json": {"binary_sha256": ""},             # present but empty
            "i-bad.json": "not a dict",                        # unparsed payload
            "i-new.json": {"binary_sha256": "a" * 64},
            "i-other.json": {"binary_sha256": "b" * 64},
        }
        binaries, unattributed = collect.attribute_binaries(reports)
        self.assertEqual(binaries, {"a" * 64, "b" * 64},
                         "two distinct binaries must both be seen")
        self.assertEqual(sorted(unattributed),
                         ["i-bad.json", "i-blank.json", "i-old.json"],
                         "every report without a usable binary must be named")

    def test_the_binary_check_still_fires_when_a_report_is_malformed(self):
        """The regression that mattered: one bad report used to hide a real
        two-binary campaign. Here the bad report and the disagreement coexist,
        and the disagreement must still be visible."""
        import collect
        binaries, unattributed = collect.attribute_binaries({
            "i-old.json": {"instance": "i-old"},
            "i-a.json": {"binary_sha256": "a" * 64},
            "i-b.json": {"binary_sha256": "b" * 64},
        })
        self.assertGreater(len(binaries), 1,
                           "a malformed report suppressed the multi-binary warning")
        self.assertEqual(unattributed, ["i-old.json"])

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
        the campaign tag, and terminate must receive exactly what it returned.

        AMENDED 2026-08-22 with the client-side tag re-check: the two instances
        now carry the `Project` tag they always had in reality, because since
        that date teardown terminates only instances whose own tags confirm the
        campaign. Untagged ids are refused, which is what
        `TeardownBlastRadiusTest` below pins; the claim here is unchanged — a
        confirmed instance reaches terminate, unaltered and unabridged."""
        import teardown
        tags = [{"Key": "Project", "Value": "binn-campaign"}]
        described = {"Reservations": [{"Instances": [
            {"InstanceId": "i-aaa", "Tags": tags},
            {"InstanceId": "i-bbb", "Tags": tags}]}]}
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


# ---------------------------------------------------------------------------
# A shared `aws` CLI double for the three scripts below.
#
# All three reach AWS through `subprocess.run(["aws", ...])` and nothing else,
# so that single call is the entire seam; faking it needs no network, no
# credentials and no new dependency. The double records every argv it is handed
# and every test below asserts on that transcript, which is the thing that stops
# a test from going green because the script stopped calling AWS at all.
# ---------------------------------------------------------------------------


class FakeAws:
    """Record every `aws` invocation and reply from a scripted handler.

    `handler(argv)` returns `(returncode, stdout, stderr)`. A call the handler
    does not recognise raises rather than returning a benign empty reply: an
    unanticipated AWS call is a change in behaviour, not a detail to swallow.
    """

    def __init__(self, handler):
        self.handler = handler
        self.calls = []

    def __call__(self, argv, **kwargs):
        argv = list(argv)
        self.calls.append(argv)
        code, stdout, stderr = self.handler(argv)
        return types.SimpleNamespace(returncode=code, stdout=stdout, stderr=stderr)

    def matching(self, *words):
        """Every recorded call whose argv contains all of `words`."""
        return [c for c in self.calls if all(w in c for w in words)]


class AwsScriptedTest(unittest.TestCase):
    """Base class: patch one module's subprocess boundary, keep the transcript.

    The fake is stored on `self` rather than returned, so a test whose script is
    expected to raise can still inspect what it managed to call first.
    """

    def drive(self, module, handler, argv, entry):
        self.fake = FakeAws(handler)
        self.stdout = ""
        buf = io.StringIO()
        old_argv, sys.argv = sys.argv, argv
        original, module.subprocess.run = module.subprocess.run, self.fake
        try:
            with contextlib.redirect_stdout(buf):
                return entry()
        finally:
            module.subprocess.run = original
            sys.argv = old_argv
            self.stdout = buf.getvalue()


class ClaimConditionalPutTest(AwsScriptedTest):
    """The conditional PUT itself — the one line the whole protocol rests on.

    `ClaimProtocolTest` above covers which cell gets chosen. It never looks at
    *how* the claim is written, so dropping `--if-none-match` — which turns the
    atomic claim into a plain overwrite and lets every worker claim the same
    cell — leaves all of those tests green. That is what this class pins, along
    with the two ways the PUT can come back other than "won".
    """

    def setUp(self):
        import claim_next
        self.mod = claim_next
        self.plan = [{"id": f"cell-{i}"} for i in range(4)]

    def script(self, done=(), held=(), put_codes=(), put_stderr=""):
        """Two listings and a scripted run of PUT exit codes (0 = claim won).

        Once `put_codes` runs out every further PUT fails, which is the shape of
        a credentials or policy problem rather than a lost race.
        """
        listings = {"results/": list(done), "claims/": list(held)}
        codes = list(put_codes)

        def handler(argv):
            if argv[1:3] == ["s3api", "list-objects-v2"]:
                prefix = argv[argv.index("--prefix") + 1]
                body = {"Contents": [{"Key": prefix + k} for k in listings[prefix]]}
                return 0, json.dumps(body), ""
            if argv[1:3] == ["s3api", "put-object"]:
                return (codes.pop(0) if codes else 1), "", put_stderr
            raise AssertionError(f"unexpected aws call: {argv}")

        return handler

    def claim(self, handler, plan=None):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "cells.json"
            path.write_text(json.dumps(self.plan if plan is None else plan))
            return self.drive(self.mod, handler,
                              ["claim_next.py", "bkt", "--plan", str(path)],
                              self.mod.main)

    def test_the_claim_is_written_as_a_conditional_put(self):
        """Without `--if-none-match *` the PUT overwrites an existing claim, so
        two workers both 'win' the same cell and one cell's compute is spent
        twice while another is never run. Nothing downstream would notice."""
        self.claim(self.script(put_codes=[0]))
        puts = self.fake.matching("s3api", "put-object")
        self.assertEqual(len(puts), 1, "the claim was never written to S3")
        argv = puts[0]
        self.assertIn("--if-none-match", argv, "the claim PUT is not conditional")
        self.assertEqual(argv[argv.index("--if-none-match") + 1], "*",
                         "a conditional PUT must be conditional on *any* existing object")
        self.assertEqual(argv[argv.index("--key") + 1], "claims/cell-0")

    def test_a_precondition_failure_is_a_lost_race_not_a_crash(self):
        """What S3 actually returns when another worker got there first. The
        loser must walk on to the next cell, not abort and not re-PUT."""
        rc = self.claim(self.script(
            put_codes=[1, 0],
            put_stderr="An error occurred (PreconditionFailed) when calling the "
                       "PutObject operation: At least one of the pre-conditions "
                       "you specified did not hold"))
        self.assertEqual(rc, 0)
        self.assertEqual(self.stdout.strip(), "cell-1")
        self.assertEqual([c[c.index("--key") + 1] for c in
                          self.fake.matching("s3api", "put-object")],
                         ["claims/cell-0", "claims/cell-1"])

    def test_a_held_claim_is_never_written_over(self):
        """A cell that is already claimed must not even be PUT at. The claim
        object carries no body, so a second PUT would be invisible after the
        fact — the only evidence is that it was never attempted."""
        rc = self.claim(self.script(held=["cell-0", "cell-1"], put_codes=[0]))
        self.assertEqual(rc, 0)
        self.assertEqual(self.stdout.strip(), "cell-2")
        self.assertEqual([c[c.index("--key") + 1] for c in
                          self.fake.matching("s3api", "put-object")],
                         ["claims/cell-2"])

    def test_done_is_read_from_result_objects_and_nothing_else(self):
        """`done` is the set of `results/<id>.json`. A partial upload landing as
        `cell-0.json.part`, or a bare `cell-0` marker, must not retire the cell:
        the campaign would come up one result short with no error anywhere."""
        rc = self.claim(self.script(
            done=["cell-0.json.part", "cell-1", "cell-2.json"], put_codes=[0]))
        self.assertEqual(rc, 0)
        self.assertEqual(self.stdout.strip(), "cell-0",
                         "a partial upload was mistaken for a finished cell")

    def test_a_drained_queue_exits_zero_having_actually_asked_s3(self):
        """Both facts matter. Exit 0 with no output is how the worker learns the
        campaign is over; and the listings must really have happened, or this
        test would pass just as well against a script that does nothing."""
        rc = self.claim(self.script(done=[f"cell-{i}.json" for i in range(4)]))
        self.assertEqual(rc, 0)
        self.assertEqual(self.stdout.strip(), "")
        self.assertEqual(len(self.fake.matching("s3api", "list-objects-v2")), 2,
                         "claim_next must list both results/ and claims/")
        self.assertEqual(self.fake.matching("s3api", "put-object"), [])

    def test_an_authorisation_failure_is_never_mistaken_for_a_drained_queue(self):
        """Repaired 2026-08-22. This was how a whole fleet retired itself.

        A PUT can fail for a reason that is not a lost race — expired instance
        credentials, a revoked bucket policy, the wrong region — and when it
        does, it fails for *every* cell in the plan. The old code treated all of
        them as lost races, walked the entire plan, printed nothing and returned
        0. `bootstrap.sh:148` reads empty stdout as `no work left`; the worker
        returns, `wait` completes, and line 169 runs `shutdown -h now`. The
        campaign came back short with the only trace in a console log.

        Now it raises on the first such failure, which the worker loop already
        handles by sleeping and retrying. **It must also give up immediately** —
        walking the rest of the plan would issue one doomed PUT per cell and bury
        the real error under the last one."""
        with self.assertRaises(SystemExit) as caught:
            self.claim(self.script(put_stderr="An error occurred (AccessDenied)"))
        self.assertNotEqual(caught.exception.code, 0,
                            "a credentials failure must not exit like a clean drain")
        self.assertIn("AccessDenied", str(caught.exception.code),
                      "the operator needs the real reason, not a generic message")
        self.assertEqual(len(self.fake.matching("s3api", "put-object")), 1,
                         "it must stop at the first real error, not try all four")


class CollectDownloadTest(AwsScriptedTest):
    """`collect.py --out` is how cells leave S3 and become the local record.

    Everything about *which* bytes get written is delegated to `aws s3 sync`,
    so what can be tested here is the contract collect hands it and what collect
    claims afterwards. Both turn out to be weaker than the printed summary
    suggests, and the tests below pin that rather than assert it away.
    """

    #: The smallest payload `collect.cell_problem` accepts, with plausible values.
    #: Spelled out here rather than imported from the collector so that a test
    #: cell stops being valid the moment the collector's idea of a cell changes —
    #: importing the requirement would make every test below agree with whatever
    #: the code currently demands, including nothing.
    VALID_CELL = {"accuracy": 0.8198, "non_finite_events": 0, "classes_predicted": 20,
                  "majority_prediction": 0.0985, "silent_fraction": 0.0156,
                  "saturated_fraction": 0.0}

    def setUp(self):
        import collect
        self.mod = collect
        self.plan = [{"id": "cell-0", "wave": "w1"}, {"id": "cell-1", "wave": "w1"}]

    def write_cell(self, target, name, **override):
        """Write one cell that the collector's validation pass accepts."""
        target.mkdir(parents=True, exist_ok=True)
        (target / name).write_text(json.dumps(dict(self.VALID_CELL, **override)))

    def script(self, results=(), claims=(), failures=(), sync=0, on_sync=None):
        listings = {"results/": list(results), "claims/": list(claims),
                    "failures/": list(failures), "gates/": []}
        stamp = datetime.now(timezone.utc).isoformat().replace("+00:00", "Z")

        def handler(argv):
            joined = " ".join(argv)
            if argv[1:3] == ["s3", "cp"] and "input/cells.json" in joined:
                return 0, json.dumps(self.plan), ""
            if argv[1:3] == ["s3api", "list-objects-v2"]:
                prefix = argv[argv.index("--prefix") + 1]
                # Fresh timestamps: every claim is younger than the orphan cutoff.
                body = {"Contents": [{"Key": prefix + k, "LastModified": stamp}
                                     for k in listings[prefix]]}
                return 0, json.dumps(body), ""
            if argv[1:3] == ["s3", "sync"]:
                if on_sync is not None:
                    on_sync(Path(argv[4]))
                return sync, "", ("boom" if sync else "")
            raise AssertionError(f"unexpected aws call: {argv}")

        return handler

    def collect(self, handler, extra=()):
        return self.drive(self.mod, handler,
                          ["collect.py", "--bucket", "bkt", *extra], self.mod.main)

    def test_the_sync_asks_for_the_results_prefix_and_nothing_wider(self):
        """The whole download is one delegated call, so its argv is the contract.
        `--delete` in particular must never appear: the out directory is where
        the archived record lives, and sync would prune it to match S3."""
        with tempfile.TemporaryDirectory() as tmp:
            out = Path(tmp) / "cells"
            self.collect(self.script(results=["cell-0.json"]), ["--out", str(out)])
            syncs = self.fake.matching("s3", "sync")
            self.assertEqual(len(syncs), 1, "nothing was downloaded at all")
            self.assertEqual(syncs[0], ["aws", "s3", "sync", "s3://bkt/results/",
                                        str(out), "--quiet"])
            self.assertNotIn("--delete", syncs[0],
                             "sync would delete local cells that are no longer in S3")

    def test_an_existing_local_cell_is_left_to_the_cli_to_judge(self):
        """HAZARD, pinned as it stands.

        `aws s3 sync` compares size and mtime, never content. A local cell that
        was edited in place without changing its length is therefore never
        re-downloaded, and collect still adds no *content* check of its own — no
        hash, no comparison against S3. The manifest in
        `results/shd_attention_campaign_v1` is what catches that class of drift;
        the collector does not.

        AMENDED 2026-08-22: collect now re-reads every collected cell, so the
        drifted file here is a complete, valid cell rather than the fragment it
        used to be — otherwise this test would pass on the validation pass
        catching a truncation, which is a different claim from the one it makes.
        Drift that keeps the schema and the length intact is still invisible,
        and the assertion is deliberately about what collect does *not* do."""
        with tempfile.TemporaryDirectory() as tmp:
            out = Path(tmp) / "cells"
            self.write_cell(out, "cell-0.json", accuracy=0.99)
            drifted = out / "cell-0.json"
            before = drifted.read_text()
            rc = self.collect(self.script(results=["cell-0.json"]), ["--out", str(out)])
            self.assertEqual(rc, 0, "a well-formed cell that drifted still reads as clean")
            self.assertEqual(drifted.read_text(), before)
            argv = self.fake.matching("s3", "sync")[0]
            for flag in ("--exact-timestamps", "--size-only"):
                self.assertNotIn(flag, argv)

    def test_a_sync_that_downloads_nothing_reports_nothing_downloaded(self):
        """INVERTED 2026-08-22. It used to assert `synced 3 cells` here.

        `synced N cells` was `len(target.glob('*.json'))` — everything already in
        the directory, fetched by this run or not — so this exact scenario, a
        sync that downloads nothing onto three stale files, printed a line that
        read as three fresh cells. The count is now taken by comparing the
        directory across the sync, so the same scenario must report zero.

        The stale files are complete cells, because the count and the validation
        pass are separate claims and this test is only about the count."""
        with tempfile.TemporaryDirectory() as tmp:
            out = Path(tmp) / "cells"
            for i in range(3):
                self.write_cell(out, f"stale-{i}.json")
            rc = self.collect(self.script(results=[]), ["--out", str(out)])
            self.assertEqual(rc, 0, "three good cells and no download is not a failure")
            self.assertIn("downloaded 0 cells", self.stdout,
                          "the count still reflects the directory, not the download")
            self.assertNotIn("downloaded 3", self.stdout)
            # The directory total is still worth printing - it is how an operator
            # sees the collection growing - but it is labelled as what it is.
            self.assertIn("cells on disk: 3", self.stdout)

    def test_only_the_cells_the_sync_wrote_are_counted_as_downloaded(self):
        """The other half of the same claim: zero must not be the only number
        this can print, or a collector that reported `downloaded 0` forever
        would pass the test above.

        Two cells sit in the directory; the sync writes one of them and leaves
        the other untouched. A directory count says 3, a download count says 2
        (one rewritten, one new), and the two must not be confusable."""
        def sync(target):
            self.write_cell(target, "cell-0.json", accuracy=0.77)   # rewritten
            self.write_cell(target, "cell-2.json")                  # new

        with tempfile.TemporaryDirectory() as tmp:
            out = Path(tmp) / "cells"
            self.write_cell(out, "cell-0.json")
            self.write_cell(out, "cell-1.json")
            rc = self.collect(self.script(results=["cell-0.json", "cell-2.json"],
                                          on_sync=sync), ["--out", str(out)])
            self.assertEqual(rc, 0)
            self.assertIn("downloaded 2 cells", self.stdout,
                          "a cell the sync rewrote and a cell it created are both "
                          "downloads; a cell it never touched is not")
            self.assertIn("cells on disk: 3", self.stdout)

    def test_a_failed_download_aborts_loudly_and_reports_no_count(self):
        """A partial sync must not be summarised as a collection. The truncated
        file it leaves behind is still on disk afterwards — sync's size compare
        is what re-fetches it next run — so the guarantee tested here is the
        narrow one: collect exits non-zero and reports no collection at all,
        rather than handing a short cell set to an analyser."""""
        def truncate(target):
            target.mkdir(parents=True, exist_ok=True)
            (target / "cell-0.json").write_text('{"accur')

        with tempfile.TemporaryDirectory() as tmp:
            out = Path(tmp) / "cells"
            with self.assertRaises(SystemExit):
                self.collect(self.script(results=["cell-0.json"], sync=1,
                                         on_sync=truncate), ["--out", str(out)])
            # Checked against the current wording, not the retired one: asserting
            # the absence of a string the script can no longer print anywhere is
            # a check that cannot fail.
            self.assertNotIn("downloaded", self.stdout)
            self.assertNotIn("validated", self.stdout)
            self.assertTrue((out / "cell-0.json").exists(),
                            "the truncated file is left behind - the next sync's "
                            "size compare is what repairs it, not collect")

    def test_a_truncated_cell_is_reported_and_never_folded_into_the_total(self):
        """INVERTED 2026-08-22. It used to assert `synced 1 cells` for this.

        Nothing in collect parsed what it downloaded, so a half-written cell
        counted toward the total exactly like a good one and the only trace was
        a JSONDecodeError days later, inside an analyser, on a machine that no
        longer had the fleet. Collect now reads every cell in the target
        directory: the truncated one is named, counted apart from the usable
        ones, and the run does not exit clean."""
        def truncate(target):
            target.mkdir(parents=True, exist_ok=True)
            (target / "cell-0.json").write_text('{"accur')

        with tempfile.TemporaryDirectory() as tmp:
            out = Path(tmp) / "cells"
            rc = self.collect(self.script(results=["cell-0.json"], on_sync=truncate),
                              ["--out", str(out)])
            self.assertEqual(rc, 1, "a corrupt cell must not exit like a clean collection")
            self.assertIn("1 INVALID", self.stdout)
            self.assertIn("INVALID cell-0.json", self.stdout,
                          "the operator needs the filename, not just a count")
            self.assertIn("0 usable", self.stdout,
                          "a cell that does not parse was still counted as usable")
            # Unchanged, and deliberately so: collect does not repair or delete
            # what it found. The next sync's size compare is what re-fetches it.
            self.assertTrue((out / "cell-0.json").exists())

    def test_a_cell_that_parses_but_lacks_the_gate_fields_is_refused(self):
        """The subtler half of the same hazard. A cell can be valid JSON and
        still be unusable: `validity_problems` in both analysers reads
        `cell["classes_predicted"]` and the other prereg §5 gates with `[]`, not
        `.get`, so a cell missing one is a KeyError at analysis time — after the
        fleet is gone. Truncation is not the only way a cell arrives short."""
        def partial(target):
            target.mkdir(parents=True, exist_ok=True)
            payload = dict(self.VALID_CELL)
            del payload["classes_predicted"]
            (target / "cell-0.json").write_text(json.dumps(payload))

        with tempfile.TemporaryDirectory() as tmp:
            out = Path(tmp) / "cells"
            rc = self.collect(self.script(results=["cell-0.json"], on_sync=partial),
                              ["--out", str(out)])
            self.assertEqual(rc, 1)
            self.assertIn("classes_predicted", self.stdout,
                          "the report must name the missing field")

    def test_a_directory_of_good_cells_validates_clean(self):
        """The positive case, without which every assertion above is satisfied
        by a collector that calls everything invalid."""
        def sync(target):
            self.write_cell(target, "cell-0.json")
            self.write_cell(target, "cell-1.json")

        with tempfile.TemporaryDirectory() as tmp:
            out = Path(tmp) / "cells"
            rc = self.collect(self.script(results=["cell-0.json", "cell-1.json"],
                                          on_sync=sync), ["--out", str(out)])
            self.assertEqual(rc, 0)
            self.assertIn("validated 2 cells: 2 usable, 0 INVALID", self.stdout)
            self.assertNotIn("INVALID cell", self.stdout)

    def test_a_plan_file_beside_the_cells_is_not_reported_as_a_broken_cell(self):
        """The validation pass must not cry wolf.

        A collection directory holds more than cells — `results/shd_attention_
        campaign_v2` keeps `plan_w8.json` and `manifest.json` right beside its 96
        cells, and both would fail a cell schema check on their first line. An
        operator who sees `INVALID plan_w8.json` once stops reading the line, at
        which point a real truncated cell scrolls past unread. So the pass is
        scoped to files that S3 says are results, and everything else is
        reported as unchecked rather than as broken."""
        with tempfile.TemporaryDirectory() as tmp:
            out = Path(tmp) / "cells"
            self.write_cell(out, "cell-0.json")
            (out / "plan_w8.json").write_text(json.dumps([{"id": "cell-0"}]))
            rc = self.collect(self.script(results=["cell-0.json"]), ["--out", str(out)])
            self.assertEqual(rc, 0, "a plan file was treated as a corrupt cell")
            self.assertIn("validated 1 cells: 1 usable, 0 INVALID", self.stdout)
            self.assertNotIn("INVALID plan_w8.json", self.stdout)
            self.assertIn("were not checked", self.stdout,
                          "a file that was skipped must be declared, not just skipped")

    def test_a_young_claim_is_never_released_back_to_the_queue(self):
        """`--release-orphans` deletes claim objects, which is the one
        destructive thing collect can do. A claim younger than the cutoff is a
        cell that is running right now; releasing it duplicates the work."""
        self.collect(self.script(results=[], claims=["cell-0", "cell-1"]),
                     ["--release-orphans"])
        self.assertEqual(self.fake.matching("s3", "rm"), [],
                         "a claim in flight was handed back to the queue")
        self.assertIn("releasable: 0", self.stdout)


class TeardownBlastRadiusTest(AwsScriptedTest):
    """`teardown.py` is the only script here that destroys anything.

    `CollectAndTeardownTest` above covers the happy path: tagged instances in,
    the same ids out. These are the edges — nothing found, a reply that is not
    a reply, and the question of what teardown actually trusts.
    """

    def setUp(self):
        import teardown
        self.mod = teardown

    def script(self, described):
        def handler(argv):
            if "describe-instances" in argv:
                return 0, (described if isinstance(described, str)
                           else json.dumps(described)), ""
            if "terminate-instances" in argv:
                return 0, json.dumps({"TerminatingInstances": []}), ""
            if argv[1] == "iam":
                return 0, "", ""
            raise AssertionError(f"unexpected aws call: {argv}")

        return handler

    def tear(self, described, extra=()):
        return self.drive(self.mod, self.script(described),
                          ["teardown.py", "--region", "us-east-1", *extra],
                          self.mod.main)

    def test_an_empty_fleet_terminates_nothing(self):
        """The filter coming back empty means the fleet is already gone, and the
        only safe reading of that is to terminate nothing at all."""
        rc = self.tear({"Reservations": []})
        self.assertEqual(rc, 0)
        self.assertEqual(self.fake.matching("ec2", "terminate-instances"), [])
        self.assertIn("no campaign instances running", self.stdout)
        self.assertEqual(len(self.fake.matching("ec2", "describe-instances")), 1,
                         "it must still have asked EC2 - not asking is not the same "
                         "as being told there is nothing there")

    def test_a_malformed_describe_reply_terminates_nothing(self):
        """`aws()` only parses stdout that starts with `{` or `[`; anything else
        comes back as a string. A string has no `.get`, so a garbled or empty
        reply raises before the terminate call is built. Ugly, but closed."""
        with self.assertRaises(AttributeError):
            self.tear("Unable to locate credentials")
        self.assertEqual(self.fake.matching("ec2", "terminate-instances"), [],
                         "a reply that could not be parsed still reached terminate")

    def test_an_instance_whose_tags_say_it_is_someone_elses_is_refused(self):
        """INVERTED 2026-08-22. It used to assert that `i-not-ours` was
        terminated, and to say so in its own failure message.

        The tag filter was sent to EC2 and whatever came back was terminated:
        the reply's tags were never read, so a widened filter, another region's
        instances or a hand-edited call reached `terminate-instances` without a
        second look. The instance in this reply is explicitly tagged to another
        project and arrives through the filter anyway — which is exactly the
        shape of the failure the filter is supposed to make impossible. It must
        now survive, be named, and take the exit status with it."""
        rc = self.tear({"Reservations": [{"Instances": [
            {"InstanceId": "i-not-ours", "Tags": [{"Key": "Project",
                                                   "Value": "someone-else"}]}]}]})
        self.assertEqual(self.fake.matching("ec2", "terminate-instances"), [],
                         "an instance tagged to another project was terminated")
        self.assertNotEqual(rc, 0, "a refusal must not exit like a clean teardown")
        self.assertIn("REFUSED", self.stdout)
        self.assertIn("i-not-ours", self.stdout,
                      "the operator needs the id that was refused")

    def test_an_untagged_instance_is_refused(self):
        """The commoner shape of the same thing: a reply with no `Tags` at all.

        That is what a `--query` that projects only `InstanceId`, or an instance
        that genuinely carries no tags, looks like from here. There is nothing
        in it that says the instance is ours, and 'nothing that says no' is not
        confirmation — this is the script that destroys things."""
        rc = self.tear({"Reservations": [{"Instances": [{"InstanceId": "i-bare"}]}]})
        self.assertEqual(self.fake.matching("ec2", "terminate-instances"), [],
                         "an instance with no tags at all was terminated")
        self.assertNotEqual(rc, 0)
        self.assertIn("i-bare", self.stdout)

    def test_a_confirmed_instance_is_still_terminated_beside_a_refused_one(self):
        """Both halves in one reply, because a check that refuses everything
        would satisfy the two tests above and leave a burning fleet up.

        The confirmed instance is terminated and the unconfirmed one is not, in
        the same run, and terminate receives only the confirmed id."""
        rc = self.tear({"Reservations": [{"Instances": [
            {"InstanceId": "i-ours",
             "Tags": [{"Key": "Name", "Value": "w3"},
                      {"Key": "Project", "Value": "binn-campaign"}]},
            {"InstanceId": "i-theirs", "Tags": [{"Key": "Project", "Value": "other"}]}]}]})
        terminate = self.fake.matching("ec2", "terminate-instances")
        self.assertEqual(len(terminate), 1, "the confirmed instance was left running")
        self.assertEqual([p for p in terminate[0] if p.startswith("i-")], ["i-ours"],
                         "terminate must receive the confirmed instance and nothing else")
        self.assertNotEqual(rc, 0, "a partial teardown is not a clean one")
        self.assertIn("i-theirs", self.stdout)

    def test_iam_is_left_alone_unless_removal_is_asked_for(self):
        """Deleting the instance profile out from under a fleet that is still
        running breaks every worker's S3 access mid-cell. It must take the flag."""
        self.tear({"Reservations": []})
        self.assertEqual([c for c in self.fake.calls if c[1] == "iam"], [])
        self.tear({"Reservations": []}, ["--remove-iam"])
        self.assertTrue([c for c in self.fake.calls if c[1] == "iam"],
                        "--remove-iam did nothing")



class SharedValidityOwnerTest(unittest.TestCase):
    """The checks the three drifted copies did not have, and the one owner.

    Every test here fails against the pre-2026-08-22 tooling: `analyse_wave8`
    carried its own gate with no temporal audit, no `mechanical_status`, no
    plan/cell agreement and no magnitude check, and wave 9 and wave 10 both
    scored their cells through it.
    """

    def cell(self, **override):
        payload = {"mechanical_status": "COMPLETE", "non_finite_events": 0,
                   "classes_predicted": 20, "majority_prediction": 0.11,
                   "silent_fraction": 0.02, "saturated_fraction": 0.0,
                   "temporal_condition": "intact",
                   "epoch_max_gradient_norm": [0.4, 1.2, 0.9]}
        payload.update(override)
        return payload

    def test_all_three_analysers_share_one_owner(self):
        """Re-drift is the failure this is guarding against, not a first drift."""
        import analyse_campaign
        import analyse_wave8
        sys.path.insert(0, str(ROOT / "scripts" / "azure"))
        import analyse as azure_analyse

        import cell_validity
        for module in (analyse_wave8, analyse_campaign, azure_analyse):
            self.assertIs(
                module.validity_problems,
                cell_validity.validity_problems,
                f"{module.__name__} has its own copy of the validity gate again",
            )

    def test_a_manipulated_cell_must_carry_a_passing_audit(self):
        """The wave-9 hole: `w9shf` was scored by a gate that never read this."""
        import cell_validity
        shuffled = self.cell(temporal_condition="bin-shuffled",
                             temporal_audit={"counts_preserved": True,
                                             "relocated_fraction": 0.87})
        self.assertEqual(cell_validity.validity_problems(shuffled), [])

        no_audit = self.cell(temporal_condition="bin-shuffled")
        self.assertIn("temporal_audit missing for a manipulated cell",
                      cell_validity.validity_problems(no_audit))

        counts_moved = self.cell(temporal_condition="bin-shuffled",
                                 temporal_audit={"counts_preserved": False,
                                                 "relocated_fraction": 0.87})
        self.assertIn("counts not preserved",
                      cell_validity.validity_problems(counts_moved))

        # A "shuffle" that barely moved anything is the case the manipulation
        # check exists for: it would score as a shuffled arm while being
        # substantially the intact one.
        barely = self.cell(temporal_condition="bin-shuffled",
                           temporal_audit={"counts_preserved": True,
                                           "relocated_fraction": 0.49})
        self.assertTrue(any("relocated_fraction" in p
                            for p in cell_validity.validity_problems(barely)))

    def test_a_cell_that_ran_the_wrong_condition_is_caught(self):
        import cell_validity
        payload = self.cell(temporal_condition="intact")
        problems = cell_validity.validity_problems(payload, {"temporal": "bin-shuffled"})
        self.assertTrue(any("plan asked for" in p for p in problems), problems)

    def test_magnitude_warns_and_never_voids(self):
        """Voiding on magnitude would re-score a published run.

        The 2026-08-05 amendment's `rec+alif` cell peaked at 3.93e33 and was
        reported as a result. This gate must make that visible without
        retroactively discarding it.
        """
        import cell_validity
        marginal = self.cell(epoch_max_gradient_norm=[1e3, 3.93e33])
        self.assertEqual(cell_validity.validity_problems(marginal), [])
        self.assertTrue(any("five orders of f32 overflow" in w
                            for w in cell_validity.stability_warnings(marginal)))

        # The tier below: above anything recorded, not yet near overflow.
        loud = self.cell(epoch_max_gradient_norm=[1e3, 1e12])
        self.assertEqual(cell_validity.validity_problems(loud), [])
        self.assertTrue(any("exceeds every cell" in w
                            for w in cell_validity.stability_warnings(loud)))

        # And quiet cells stay quiet, or the notes would be noise.
        self.assertEqual(cell_validity.stability_warnings(self.cell()), [])

    def test_a_non_finite_norm_trace_is_not_silently_skipped(self):
        import cell_validity
        # JSON has no infinity literal, so a non-finite norm arrives as null.
        payload = self.cell(epoch_max_gradient_norm=[1.0, None, 2.0])
        problems = cell_validity.validity_problems(payload)
        self.assertTrue(any("non-finite" in p for p in problems), problems)

    def test_every_recorded_campaign_cell_still_passes(self):
        """The new checks must void nothing that was already published.

        A gate that retroactively invalidates the record is not a hardening,
        it is a re-scoring, and it would need its own registration.
        """
        import cell_validity
        roots = [ROOT / "results" / "shd_attention_campaign_v1" / "cells",
                 ROOT / "results" / "shd_attention_campaign_v2"]
        checked = 0
        for root in roots:
            for path in sorted(root.glob("*.json")):
                if path.name in ("manifest.json",) or path.name.startswith("plan"):
                    continue
                payload = json.loads(path.read_text())
                if "accuracy" not in payload:
                    continue
                self.assertEqual(cell_validity.validity_problems(payload), [],
                                 f"{path.name} would now be voided")
                checked += 1
        self.assertGreater(checked, 600, f"only {checked} cells checked")

    def test_a_clipped_cell_cannot_be_read_as_an_unclipped_one(self):
        import cell_validity
        self.assertEqual(cell_validity.stability_warnings(self.cell()), [])
        clipped = self.cell(clipped_samples=17)
        self.assertTrue(any("clipping bound" in w
                            for w in cell_validity.stability_warnings(clipped)))




#: Sentinel for a scripted AWS call that fails.
FAIL = object()


class Wave11AnalyserTest(unittest.TestCase):
    """Exercise the wave-11 analyser against synthetic cells.

    Written because freezing an analyser before the data did **not** make it
    correct. `analyse_wave11.py` was frozen before wave 11 launched, with two
    defects that would both have fired had the completion bar passed: it grouped
    on `surrogate_scale == 0.4` against an f32 field that records 0.400000006,
    and it keyed pairs on `cell["seed"]`, a field the emitted cell does not have
    and which the record had already flagged as missing.

    Neither ran, because the completion expectation failed first. That is luck,
    not process. An analyser has to be run against a fixture before the real
    cells land, and this is that fixture.
    """

    def cells(self, directory, spec):
        """Write synthetic cells with realistic field types.

        `surrogate_scale` is deliberately stored as the f32 value the instrument
        actually emits, and no `seed` field is written -- reproducing both traps
        rather than a tidy version of them.
        """
        for arm, scale, seed, accuracy in spec:
            stem = (f"w11rec__{arm.replace('+', '-')}__h256__e100__published-2ms"
                    f"__adjacent-sum-5__ss{scale}__s{seed}.json")
            (directory / stem).write_text(json.dumps({
                "arm": arm,
                "accuracy": accuracy,
                "non_finite_events": 0,
                "surrogate_scale": 0.400000006 if scale == 0.4 else 1.0,
            }))

    def run_analyser(self, spec):
        import analyse_wave11
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory)
            self.cells(path, spec)
            original, analyse_wave11.CELLS = analyse_wave11.CELLS, path
            try:
                out = io.StringIO()
                with contextlib.redirect_stdout(out):
                    code = analyse_wave11.main()
                return code, out.getvalue()
            finally:
                analyse_wave11.CELLS = original

    def full_grid(self):
        """24 cells: attention ahead by a constant 0.30 at every seed and scale."""
        spec = []
        for scale in (1.0, 0.4):
            for seed in range(5170001, 5170007):
                spec.append(("rec+alif", scale, seed, 0.45))
                spec.append(("rec+alif+attn", scale, seed, 0.75))
        return spec

    def test_the_scale_grouping_survives_an_f32_surrogate_scale(self):
        """The bug that would have made T4-3 a NaN. Both scale buckets must be
        populated from a field that records 0.400000006, not 0.4."""
        code, out = self.run_analyser(self.full_grid())
        self.assertEqual(code, 0)
        self.assertIn("**T4-3**", out)
        self.assertNotIn("nan", out.lower(),
                         "a scale bucket came back empty; the f32 grouping regressed")

    def test_pairing_works_without_a_seed_field_in_the_cell(self):
        """The bug that would have raised KeyError. Every planned pair must be
        found from the cell id alone."""
        code, out = self.run_analyser(self.full_grid())
        self.assertEqual(code, 0)
        self.assertIn("12/12 paired seeds agree in sign", out.replace("  ", " "),
                      f"pairing did not recover all 12 pairs:\n{out}")

    def test_a_short_wave_refuses_every_scientific_verdict(self):
        """The registered response to the completion bar, which is what actually
        fired on the real wave. It must refuse, and it must say so."""
        # 16 cells, below the registered bar of 18. The real wave landed 15.
        code, out = self.run_analyser(self.full_grid()[:16])
        self.assertEqual(code, 1)
        self.assertIn("NOT MET", out)
        self.assertIn("NOT EVALUABLE", out)
        for hypothesis in ("**T4-1**", "**T4-2**", "**T4-3**"):
            self.assertNotIn(f"{hypothesis} ", out.replace("T4-1, T4-2 and T4-3", ""),
                             f"{hypothesis} was evaluated on a short wave")

    def test_a_full_wave_with_no_effect_reports_not_supported(self):
        """The bar has to be able to say no. Identical arms must fail T4-2."""
        spec = [(arm, scale, seed, 0.45)
                for scale in (1.0, 0.4)
                for seed in range(5170001, 5170007)
                for arm in ("rec+alif", "rec+alif+attn")]
        code, out = self.run_analyser(spec)
        self.assertEqual(code, 0)
        self.assertIn("**T4-2**", out)
        self.assertIn("NOT SUPPORTED", out)


if __name__ == "__main__":
    unittest.main(verbosity=2)
