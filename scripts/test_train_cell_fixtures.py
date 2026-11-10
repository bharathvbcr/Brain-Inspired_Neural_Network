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


if __name__ == "__main__":
    unittest.main()
