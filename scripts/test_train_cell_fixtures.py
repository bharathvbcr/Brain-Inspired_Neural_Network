#!/usr/bin/env python3
"""train_cell_fixtures.py must tell a reproduction from silence.

The fixture script is a bit-exactness gate for kernel changes, so its own
failure modes matter more than its happy path: an instrument that crashes,
hangs, or exits 0 without writing must never read as PASS, a fixture with no
recorded hash must never read as PASS, and `--record` must refuse a binary that
does not reproduce the pinned oracle. Everything here runs against a fake
instrument (a small python script) on synthetic inputs; no real binary, no data.
"""

from __future__ import annotations

import json
import stat
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import train_cell_fixtures as F  # noqa: E402

FAKE = r'''#!{python}
import hashlib, json, sys
mode = open("{mode_file}").read().strip()
argv = sys.argv[1:]
def flag(name):
    return argv[argv.index(name) + 1] if name in argv else None
if mode == "hang":
    import time; time.sleep(600)
if mode == "crash":
    sys.stderr.write("boom\n"); sys.exit(3)
if argv[0] == "init":
    open(flag("--weights"), "wb").write(b"init")
    open(flag("--orders"), "wb").write(b"orders")
    sys.exit(0)
if mode == "silent":
    sys.exit(0)
if mode == "empty-cell":
    open(flag("--out"), "w").write("")
    open(flag("--save-final-weights"), "wb").write(b"w")
    sys.exit(0)
seed_text = "|".join(a for a in argv if not a.startswith("/"))
payload = {{"arm": flag("--arm"), "accuracy": len(seed_text), "wall_secs": __import__("time").time(),
            "emitted_unix_s": 1, "emitted_utc": "now"}}
if flag("--numerics") == "blas" and mode != "noblas":
    payload["numerics"] = "blas"
    payload["numerics_platform"] = ("macOS 99.0 (99Z99)" if mode == "otherplatform"
                                       else "macOS 26.1 (25B78)" if mode != "noplatform" else "")
elif mode == "noblas":
    payload["numerics"] = "exact"
open(flag("--out"), "w").write(json.dumps(payload))
body = (seed_text if mode != "other" else seed_text + "x").encode()
open(flag("--save-final-weights"), "wb").write(body)
'''


class Fake:
    def __init__(self, root: Path) -> None:
        self.mode_file = root / "mode"
        self.binary = root / "fake-instrument"
        self.binary.write_text(FAKE.format(python=sys.executable, mode_file=self.mode_file))
        self.binary.chmod(self.binary.stat().st_mode | stat.S_IEXEC)
        self.set("ok")

    def set(self, mode: str) -> None:
        self.mode_file.write_text(mode)


class CommandTests(unittest.TestCase):
    def test_default_scratch_directory_is_inside_this_checkout(self) -> None:
        # The default once named another agent session's scratchpad, which
        # exists on one machine only and was shared with a running replay.
        out = F.DEFAULT_OUT.resolve()
        self.assertTrue(out.is_relative_to(F.ROOT / "target"), out)

    def test_names_are_unique_and_the_oracle_is_present(self) -> None:
        names = [fx.name for fx in F.FIXTURES]
        self.assertEqual(len(names), len(set(names)))
        self.assertIn(F.ORACLE_NAME, names)

    def test_every_readout_and_the_shuffles_are_covered(self) -> None:
        readouts = {fx.readout for fx in F.FIXTURES if fx.attention}
        self.assertEqual(readouts, {None, "no-position", "qk-norm", "no-position+qk-norm"})
        temporal = {fx.temporal for fx in F.FIXTURES}
        self.assertTrue({"hidden-shuffled", "bin-shuffled"} <= temporal)

    def test_attention_dims_include_non_multiples_of_16_and_4(self) -> None:
        dims = {fx.attn_dim for fx in F.FIXTURES if fx.attention}
        self.assertTrue(any(d % 16 for d in dims))
        self.assertTrue(any(d % 4 for d in dims))
        self.assertTrue(all(d % 2 == 0 for d in dims), "the positional code needs even d_model")

    def test_oracle_command_matches_the_reference_invocation(self) -> None:
        fx = next(f for f in F.FIXTURES if f.name == F.ORACLE_NAME)
        work, events = Path("/w"), Path("/e")
        init = F.init_command(fx, Path("/bin/x"), work)
        self.assertEqual(init[init.index("--n-train") + 1], "6987")
        self.assertEqual(init[init.index("--attn-dim") + 1], "32")
        self.assertEqual(init[init.index("--attn-layers") + 1], "4")
        train = F.train_command(fx, Path("/bin/x"), work, events)
        self.assertEqual(train[train.index("--contract") + 1], "published-2ms")
        self.assertEqual(train[train.index("--val-speakers") + 1], "3,6")
        self.assertIn("--save-final-weights", train)
        self.assertNotIn("--temporal", train)

    def test_base_arm_has_no_attention_or_speaker_flags(self) -> None:
        fx = next(f for f in F.FIXTURES if f.name == "base-h512-channels-700-fixed-t500")
        init = F.init_command(fx, Path("/b"), Path("/w"))
        train = F.train_command(fx, Path("/b"), Path("/w"), Path("/e"))
        self.assertEqual(init[init.index("--n-inputs") + 1], "700")
        self.assertEqual(init[init.index("--n-train") + 1], "8156")
        self.assertNotIn("--attn-dim", init)
        self.assertNotIn("--val-speakers", train)

    def test_readout_and_temporal_reach_the_right_command(self) -> None:
        fx = F.Fixture("t", arm="ff+fixed+attn", attn_dim=8, attn_layers=1,
                       readout="qk-norm", temporal="hidden-shuffled", speaker_split=True)
        init = F.init_command(fx, Path("/b"), Path("/w"))
        train = F.train_command(fx, Path("/b"), Path("/w"), Path("/e"))
        self.assertEqual(init[init.index("--attn-readout") + 1], "qk-norm")
        self.assertEqual(train[train.index("--temporal") + 1], "hidden-shuffled")
        # The instrument exits 2 on a manipulated run with no seed (found live).
        self.assertEqual(train[train.index("--temporal-seed") + 1], str(F.SEED))

    def test_intact_runs_carry_no_temporal_flags(self) -> None:
        fx = next(f for f in F.FIXTURES if f.temporal is None)
        train = F.train_command(fx, Path("/b"), Path("/w"), Path("/e"))
        self.assertNotIn("--temporal-seed", train)


class HashTests(unittest.TestCase):
    def test_clock_fields_do_not_move_the_cell_hash(self) -> None:
        a = json.dumps({"accuracy": 0.5, "wall_secs": 1.0, "emitted_unix_s": 5, "emitted_utc": "a"})
        b = json.dumps({"emitted_utc": "b", "accuracy": 0.5, "wall_secs": 99.0, "emitted_unix_s": 6})
        self.assertEqual(F.canonical_cell_sha256(a), F.canonical_cell_sha256(b))

    def test_a_measurement_moves_the_cell_hash(self) -> None:
        a = json.dumps({"accuracy": 0.5, "wall_secs": 1.0})
        b = json.dumps({"accuracy": 0.5000001, "wall_secs": 1.0})
        self.assertNotEqual(F.canonical_cell_sha256(a), F.canonical_cell_sha256(b))

    def test_non_cells_are_rejected_rather_than_hashed(self) -> None:
        for text in ("", "not json", "[1]", json.dumps({"accuracy": 0.5})):
            with self.assertRaises(ValueError, msg=text):
                F.canonical_cell_sha256(text)


class JudgeTests(unittest.TestCase):
    OBS = {"weights_sha256": "w", "cell_sha256": "c"}

    def test_match_passes(self) -> None:
        self.assertEqual(F.judge("a", self.OBS, {"a": dict(self.OBS)})["status"], F.PASS)

    def test_either_hash_differing_is_a_mismatch(self) -> None:
        for key in self.OBS:
            want = {**self.OBS, key: "other"}
            verdict = F.judge("a", self.OBS, {"a": want})
            self.assertEqual(verdict["status"], F.MISMATCH)
            self.assertIn(key, verdict["detail"])

    def test_unrecorded_is_not_a_pass(self) -> None:
        self.assertEqual(F.judge("a", self.OBS, {})["status"], F.UNRECORDED)

    def test_exit_code_requires_a_non_empty_all_pass(self) -> None:
        self.assertEqual(F.verdict_exit_code([F.PASS, F.PASS]), 0)
        self.assertEqual(F.verdict_exit_code([]), 1)
        for bad in (F.MISMATCH, F.ERROR, F.TIMEOUT, F.UNRECORDED):
            self.assertEqual(F.verdict_exit_code([F.PASS, bad]), 1)

    def test_stale_recorded_names_are_reported(self) -> None:
        self.assertEqual(F.stale_recorded({"gone": {}, F.ORACLE_NAME: {}}, F.FIXTURES), ["gone"])

    def test_select_rejects_unknown_names(self) -> None:
        with self.assertRaises(SystemExit):
            F.select(F.FIXTURES, ["no-such-fixture"])


class RunTests(unittest.TestCase):
    def setUp(self) -> None:
        self._tmp = tempfile.TemporaryDirectory()
        self.root = Path(self._tmp.name)
        self.fake = Fake(self.root)
        self.out = self.root / "out"
        self.fx = F.Fixture("tiny", speaker_split=True)

    def tearDown(self) -> None:
        self._tmp.cleanup()

    def run_fx(self, timeout: float = 30.0) -> dict:
        return F.run_fixture(self.fx, self.fake.binary, self.out, self.root / "ev", timeout)

    def test_a_good_run_is_hashed_and_repeatable(self) -> None:
        first = self.run_fx()
        second = self.run_fx()
        self.assertEqual(first["status"], "RAN")
        self.assertEqual(first["observed"], second["observed"])

    def test_a_different_binary_output_changes_the_weights_hash(self) -> None:
        first = self.run_fx()
        self.fake.set("other")
        second = self.run_fx()
        self.assertNotEqual(first["observed"]["weights_sha256"],
                            second["observed"]["weights_sha256"])

    def test_crash_is_an_error(self) -> None:
        self.fake.set("crash")
        result = self.run_fx()
        self.assertEqual(result["status"], F.ERROR)
        self.assertIn("boom", result["detail"])

    def test_hang_is_a_timeout(self) -> None:
        self.fake.set("hang")
        self.assertEqual(self.run_fx(timeout=2.0)["status"], F.TIMEOUT)

    def test_exit_zero_without_output_is_an_error(self) -> None:
        self.fake.set("silent")
        self.assertEqual(self.run_fx()["status"], F.ERROR)

    def test_previous_outputs_are_not_reused_when_the_instrument_goes_silent(self) -> None:
        self.assertEqual(self.run_fx()["status"], "RAN")
        self.fake.set("silent")
        self.assertEqual(self.run_fx()["status"], F.ERROR)

    def test_an_unreadable_cell_is_an_error(self) -> None:
        self.fake.set("empty-cell")
        self.assertEqual(self.run_fx()["status"], F.ERROR)

    def test_missing_binary_is_an_error_not_a_pass(self) -> None:
        self.assertEqual(F.main(["--binary", str(self.root / "absent")]), 2)


class MainTests(unittest.TestCase):
    """End to end with the fake instrument and a one-fixture matrix."""

    def setUp(self) -> None:
        self._tmp = tempfile.TemporaryDirectory()
        self.root = Path(self._tmp.name)
        self.fake = Fake(self.root)
        self.json = self.root / "fixtures.json"
        self._saved = F.FIXTURES
        self.oracle = next(f for f in F.FIXTURES if f.name == F.ORACLE_NAME)
        F.FIXTURES = (self.oracle,)
        self.args = ["--binary", str(self.fake.binary), "--out", str(self.root / "out"),
                     "--fixtures-json", str(self.json), "--events", str(self.root / "ev")]

    def tearDown(self) -> None:
        F.FIXTURES = self._saved
        self._tmp.cleanup()

    def test_record_refuses_a_binary_that_misses_the_pinned_oracle(self) -> None:
        # The fake's weights are not the pinned reference bytes.
        self.assertEqual(F.main([*self.args, "--record"]), 1)
        self.assertFalse(self.json.exists())

    def test_compare_flow_with_the_pin_lifted(self) -> None:
        saved = F.ORACLE_WEIGHTS_SHA256
        try:
            self.fake.set("ok")
            # Adopt whatever the fake produces as the pin, as a reference binary would.
            probe = F.run_fixture(self.oracle, self.fake.binary, self.root / "p",
                                  self.root / "ev", 30.0)
            F.ORACLE_WEIGHTS_SHA256 = probe["observed"]["weights_sha256"]
            self.assertEqual(F.main([*self.args, "--record"]), 0)
            self.assertEqual(F.main(self.args), 0)
            self.fake.set("other")
            self.assertEqual(F.main(self.args), 1)
            self.fake.set("crash")
            self.assertEqual(F.main(self.args), 1)
        finally:
            F.ORACLE_WEIGHTS_SHA256 = saved

    def test_compare_without_a_recorded_file_fails(self) -> None:
        self.assertEqual(F.main(self.args), 2)

    def test_a_recorded_name_with_no_fixture_fails_the_run(self) -> None:
        saved = F.ORACLE_WEIGHTS_SHA256
        try:
            probe = F.run_fixture(self.oracle, self.fake.binary, self.root / "p",
                                  self.root / "ev", 30.0)
            F.ORACLE_WEIGHTS_SHA256 = probe["observed"]["weights_sha256"]
            self.assertEqual(F.main([*self.args, "--record"]), 0)
        finally:
            F.ORACLE_WEIGHTS_SHA256 = saved
        payload = json.loads(self.json.read_text())
        payload["fixtures"]["retired-fixture"] = {"weights_sha256": "x", "cell_sha256": "y"}
        self.json.write_text(json.dumps(payload))
        self.assertEqual(F.main(self.args), 1)


class NumericsCommandTests(unittest.TestCase):
    def fx(self) -> "F.Fixture":
        return next(f for f in F.FIXTURES if f.name == F.ORACLE_NAME)

    def test_exact_commands_never_carry_the_flag(self) -> None:
        for fx in F.FIXTURES:
            default = F.train_command(fx, Path("/b"), Path("/w"), Path("/e"))
            explicit = F.train_command(fx, Path("/b"), Path("/w"), Path("/e"), "exact")
            self.assertEqual(default, explicit, fx.name)
            self.assertNotIn("--numerics", default, fx.name)

    def test_blas_appends_exactly_the_flag_to_train_cell_only(self) -> None:
        for fx in F.FIXTURES:
            exact = F.train_command(fx, Path("/b"), Path("/w"), Path("/e"))
            blas = F.train_command(fx, Path("/b"), Path("/w"), Path("/e"), "blas")
            self.assertEqual(blas, [*exact, "--numerics", "blas"], fx.name)
            self.assertNotIn("--numerics", F.init_command(fx, Path("/b"), Path("/w")), fx.name)

    def test_each_mode_selects_its_own_hash_file(self) -> None:
        self.assertEqual(F.fixtures_path("exact"), F.FIXTURE_JSON)
        self.assertEqual(F.fixtures_path("blas"), F.FIXTURE_JSON_BLAS)
        self.assertNotEqual(F.FIXTURE_JSON, F.FIXTURE_JSON_BLAS)
        self.assertEqual(F.FIXTURE_JSON_BLAS.name, "train_cell_fixtures_blas.json")

    def test_the_committed_blas_file_records_every_fixture_on_one_platform(self) -> None:
        # Recorded 2026-11-11 with an `accelerate` build of numerics/integrate
        # and reproduced 23/23 on a second run. Every hash is filled, and the
        # platform it was recorded on is named, because Accelerate's kernels
        # may change with the OS.
        payload = json.loads(F.FIXTURE_JSON_BLAS.read_text())
        self.assertEqual(payload["numerics"], "blas")
        self.assertRegex(payload["numerics_platform"], r"^macOS \d+(\.\d+)* \([0-9A-Za-z]+\)$")
        self.assertEqual(sorted(payload["fixtures"]), sorted(fx.name for fx in F.FIXTURES))
        self.assertEqual(len(payload["fixtures"]), 23)
        hashes = [h for entry in payload["fixtures"].values() for h in entry.values()]
        self.assertEqual(len(hashes), 46)
        self.assertEqual(sum(isinstance(h, str) and len(h) == 64 for h in hashes), 46)

    def test_base_arms_train_the_same_weights_in_both_modes(self) -> None:
        # A base arm has no attention read-out, so it never reaches BLAS: its
        # final weights must be identical in exact and blas mode. Every
        # attention fixture must differ, or the blas path did not run.
        exact = json.loads(F.FIXTURE_JSON.read_text())["fixtures"]
        blas = json.loads(F.FIXTURE_JSON_BLAS.read_text())["fixtures"]
        same = sorted(n for n in exact if exact[n]["weights_sha256"] == blas[n]["weights_sha256"])
        base = sorted(fx.name for fx in F.FIXTURES if fx.name.startswith("base-"))
        self.assertEqual(len(base), 7)
        self.assertEqual(same, base)

    def test_the_exact_file_is_untouched_by_the_flag(self) -> None:
        payload = json.loads(F.FIXTURE_JSON.read_text())
        self.assertNotIn("numerics", payload)
        self.assertEqual(sorted(payload["fixtures"]), sorted(fx.name for fx in F.FIXTURES))
        for entry in payload["fixtures"].values():
            self.assertEqual(sorted(entry), ["cell_sha256", "weights_sha256"])
            self.assertTrue(all(isinstance(h, str) and len(h) == 64 for h in entry.values()))


class NullHashTests(unittest.TestCase):
    OBS = {"weights_sha256": "w", "cell_sha256": "c"}

    def test_null_hashes_are_an_error_never_a_pass_or_a_comparison(self) -> None:
        for entry in ({"weights_sha256": None, "cell_sha256": None},
                      {"weights_sha256": "w", "cell_sha256": None},
                      {"weights_sha256": None, "cell_sha256": "c"},
                      {"weights_sha256": "", "cell_sha256": "c"},
                      {}, None):
            verdict = F.judge("a", self.OBS, {"a": entry})
            self.assertEqual(verdict["status"], F.ERROR, entry)
            self.assertIn("null or missing", verdict["detail"])

    def test_a_null_entry_cannot_match_a_null_observation(self) -> None:
        """None == None must not read as agreement."""
        verdict = F.judge("a", {"weights_sha256": None, "cell_sha256": None},
                          {"a": {"weights_sha256": None, "cell_sha256": None}})
        self.assertEqual(verdict["status"], F.ERROR)

    def test_a_null_entry_fails_the_exit_code(self) -> None:
        self.assertEqual(F.verdict_exit_code([F.PASS, F.ERROR]), 1)


class BlasMainTests(unittest.TestCase):
    """End to end in blas mode with the fake instrument and a one-fixture matrix."""

    def setUp(self) -> None:
        self._tmp = tempfile.TemporaryDirectory()
        self.root = Path(self._tmp.name)
        self.fake = Fake(self.root)
        self.json = self.root / "fixtures_blas.json"
        self._saved = F.FIXTURES
        self.oracle = next(f for f in F.FIXTURES if f.name == F.ORACLE_NAME)
        F.FIXTURES = (self.oracle,)
        self.args = ["--binary", str(self.fake.binary), "--out", str(self.root / "out"),
                     "--fixtures-json", str(self.json), "--events", str(self.root / "ev"),
                     "--numerics", "blas"]

    def tearDown(self) -> None:
        F.FIXTURES = self._saved
        self._tmp.cleanup()

    def write_null_file(self) -> None:
        self.json.write_text(json.dumps({
            "schema": F.SCHEMA, "numerics": "blas", "numerics_platform": None,
            "fixtures": {F.ORACLE_NAME: {"weights_sha256": None, "cell_sha256": None}}}))

    def test_compare_against_null_hashes_is_an_error_exit_1(self) -> None:
        self.write_null_file()
        self.assertEqual(F.main(self.args), 1)

    def test_blas_record_needs_no_oracle_pin_and_records_the_platform(self) -> None:
        # The exact oracle pin does not apply to blas; this fake would miss it.
        self.assertEqual(F.main([*self.args, "--record"]), 0)
        payload = json.loads(self.json.read_text())
        self.assertEqual(payload["numerics"], "blas")
        self.assertEqual(payload["numerics_platform"], "macOS 26.1 (25B78)")
        entry = payload["fixtures"][F.ORACLE_NAME]
        self.assertTrue(all(isinstance(entry[k], str) and len(entry[k]) == 64 for k in entry))

    def test_blas_record_then_compare_round_trips(self) -> None:
        self.write_null_file()
        self.assertEqual(F.main(self.args), 1)
        self.assertEqual(F.main([*self.args, "--record"]), 0)
        self.assertEqual(F.main(self.args), 0)
        self.fake.set("other")
        self.assertEqual(F.main(self.args), 1)

    def test_the_exact_oracle_pin_still_guards_exact_record(self) -> None:
        args = [a for a in self.args if a not in ("--numerics", "blas")]
        self.assertEqual(F.main([*args, "--record"]), 1)
        self.assertFalse(self.json.exists())

    def test_a_binary_that_ignores_the_flag_is_an_error_not_a_record(self) -> None:
        self.fake.set("noblas")
        self.assertEqual(F.main([*self.args, "--record"]), 1)
        self.assertFalse(self.json.exists())

    def test_a_blas_cell_without_a_platform_is_an_error(self) -> None:
        self.fake.set("noplatform")
        result = F.run_fixture(self.oracle, self.fake.binary, self.root / "p",
                               self.root / "ev", 30.0, "blas")
        self.assertEqual(result["status"], F.ERROR)
        self.assertIn("numerics_platform", result["detail"])

    def test_hashes_do_not_cross_modes(self) -> None:
        """A blas file refuses an exact run and an exact file refuses a blas run."""
        self.assertEqual(F.main([*self.args, "--record"]), 0)
        exact_args = [a for a in self.args if a not in ("--numerics", "blas")]
        self.assertEqual(F.main(exact_args), 2)
        exact_json = self.root / "exact.json"
        exact_json.write_text(json.dumps({
            "schema": F.SCHEMA,
            "fixtures": {F.ORACLE_NAME: {"weights_sha256": "w", "cell_sha256": "c"}}}))
        swapped = [*exact_args[:exact_args.index("--fixtures-json") + 1], str(exact_json),
                   *exact_args[exact_args.index("--fixtures-json") + 2:], "--numerics", "blas"]
        self.assertEqual(F.main(swapped), 2)

    def _compare(self, mode: str) -> tuple[int, str]:
        """Record under the default fake, then compare with the fake in `mode`."""
        import contextlib
        import io
        self.assertEqual(F.main([*self.args, "--record"]), 0)
        self.fake.set(mode)
        buf = io.StringIO()
        with contextlib.redirect_stdout(buf):
            code = F.main(self.args)
        return code, buf.getvalue()

    def test_platform_mismatch_is_not_comparable_never_a_hash_verdict(self) -> None:
        # The weights hash matches the recording; the platform differs, and
        # since numerics_platform is part of the hashed cell, so does
        # cell_sha256. Before NOT_COMPARABLE this read as a MISMATCH.
        code, out = self._compare("otherplatform")
        self.assertEqual(code, 1)
        self.assertIn("[NOT_COMPARABLE", out)
        self.assertIn("macOS 26.1 (25B78)", out)
        self.assertIn("macOS 99.0 (99Z99)", out)
        self.assertNotIn("[MISMATCH", out)
        self.assertNotIn("[PASS", out)

    def test_platform_mismatch_outranks_a_hash_difference(self) -> None:
        judged = F.judge("n", {"weights_sha256": "x", "cell_sha256": "y"},
                         {"n": {"weights_sha256": "a", "cell_sha256": "b"}}, ("A", "B"))
        self.assertEqual(judged["status"], F.NOT_COMPARABLE)

    def test_same_platform_still_passes_or_mismatches_on_hashes(self) -> None:
        code, out = self._compare("ok")
        self.assertEqual(code, 0)
        self.assertIn("[PASS", out)
        code, out = self._compare("other")
        self.assertEqual(code, 1)
        self.assertIn("[MISMATCH", out)
        self.assertNotIn("NOT_COMPARABLE", out)

    def test_a_recorded_file_without_a_platform_is_an_error(self) -> None:
        self.assertEqual(F.main([*self.args, "--record"]), 0)
        payload = json.loads(self.json.read_text())
        del payload["numerics_platform"]
        self.json.write_text(json.dumps(payload))
        self.assertEqual(F.main(self.args), 1)
        entry = {"n": {"weights_sha256": "a", "cell_sha256": "b"}}
        self.assertEqual(F.judge("n", {}, entry, (None, "B"))["status"], F.ERROR)

    def test_exact_mode_ignores_platform(self) -> None:
        exact = {"weights_sha256": "a", "cell_sha256": "b"}
        self.assertEqual(F.judge("n", exact, {"n": exact})["status"], F.PASS)
        # An exact run records and compares with no platform, whatever the cell says.
        saved = F.ORACLE_WEIGHTS_SHA256
        args = [a for a in self.args if a not in ("--numerics", "blas")]
        try:
            probe = F.run_fixture(self.oracle, self.fake.binary, self.root / "p",
                                  self.root / "ev", 30.0)
            F.ORACLE_WEIGHTS_SHA256 = probe["observed"]["weights_sha256"]
            self.assertEqual(F.main([*args, "--record"]), 0)
            self.assertNotIn("numerics_platform", json.loads(self.json.read_text()))
            self.assertEqual(F.main(args), 0)
        finally:
            F.ORACLE_WEIGHTS_SHA256 = saved


if __name__ == "__main__":
    unittest.main()
