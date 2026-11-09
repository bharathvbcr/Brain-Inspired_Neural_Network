"""Tests for the frozen analyser on synthetic record corpora (stdlib unittest, CPU only)."""

from __future__ import annotations

import io
import json
import sys
import tempfile
import unittest
from contextlib import redirect_stdout
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import analyse_attention_on_delays as an  # noqa: E402

S1 = an.STAGE1_SEEDS
S2 = an.STAGE2_SEEDS
# A sample configuration for the synthetic corpora. The registered one is
# written into the analyser from the probe's pick; these tests set their own.
SAMPLE_CONFIG = {"torch_threads": 16, "mkldnn": "bwd-off", "instance_type": "c8g.4xlarge"}


REGISTERED_CONFIG = an.CONFIG


def setUpModule() -> None:
    an.CONFIG = dict(SAMPLE_CONFIG)


def tearDownModule() -> None:
    an.CONFIG = REGISTERED_CONFIG


class RegisteredConfiguration(unittest.TestCase):
    def test_registered_configuration_is_the_probe_pick(self):
        # prereg §2: c8g.8xlarge, 32 threads, oneDNN off (fresh_config_probe/RESULT.md).
        self.assertEqual(REGISTERED_CONFIG,
                         {"torch_threads": 32, "mkldnn": "off", "instance_type": "c8g.8xlarge"})


def record(arm: str, seed: int, acc: float, **over) -> dict:
    rec = {
        **an.FIXED, **SAMPLE_CONFIG, **an.ARMS[arm],
        "arm": arm, "seed": seed, "accuracy": float(acc), "memo": True,
        "instance_id": f"i-{arm.lower()}-{seed}",
        "test_loss": 0.2, "train_curve": [], "wall_secs": 3600.0,
    }
    rec.update(over)
    return rec


def stage0(a=(0.938, 0.936, 0.939), rerun=0.938, b=(0.626, 0.630, 0.627)) -> list[dict]:
    recs = [record("A00", s, x) for s, x in zip(S1, a)]
    recs.append(record("A00r", an.RERUN_SEED, rerun))
    recs += [record("B00", s, x) for s, x in zip(S1, b)]
    return recs


def stage1(a10, a10h, b10) -> list[dict]:
    return ([record("A10", s, x) for s, x in zip(S1, a10)]
            + [record("A10h", s, x) for s, x in zip(S1, a10h)]
            + [record("B10", s, x) for s, x in zip(S1, b10)])


class Analyser(unittest.TestCase):
    def run_corpus(self, records: list[dict]) -> tuple[int, str, dict]:
        with tempfile.TemporaryDirectory() as tmp:
            for i, rec in enumerate(records):
                Path(tmp, f"r{i:03d}.json").write_text(json.dumps(rec))
            out_json = Path(tmp, "out", "verdicts.json")
            out_json.parent.mkdir()
            buf = io.StringIO()
            with redirect_stdout(buf):
                code = an.main([tmp, "--json", str(out_json)])
            data = json.loads(out_json.read_text()) if out_json.exists() else {}
        return code, buf.getvalue(), data

    # --- evaluability -----------------------------------------------------
    def test_empty_corpus_is_not_evaluable(self):
        code, out, _ = self.run_corpus([])
        self.assertEqual(code, 2)
        self.assertIn("HA-0: NOT EVALUABLE", out)

    def test_missing_one_stage0_cell_is_not_evaluable(self):
        recs = stage0()[:-1]
        code, out, _ = self.run_corpus(recs)
        self.assertEqual(code, 2)
        self.assertIn("('B00', 5170003)", out)

    def test_stage0_alone_is_evaluable_with_stage1_absent(self):
        code, out, data = self.run_corpus(stage0())
        self.assertEqual(code, 0, out)
        self.assertEqual(data["HA-0"]["status"], "PASS")
        self.assertIn("stage 1: NOT STARTED", out)

    def test_partial_stage1_is_not_evaluable(self):
        recs = stage0() + stage1((0.96,) * 3, (0.94,) * 3, (0.70,) * 3)[:-1]
        code, out, _ = self.run_corpus(recs)
        self.assertEqual(code, 2)
        self.assertIn("stage 1: NOT EVALUABLE", out)

    # --- HA-0 -------------------------------------------------------------
    def test_ha0_fails_on_kernel_substrate_drift(self):
        code, out, data = self.run_corpus(stage0(a=(0.925, 0.926, 0.927), rerun=0.925))
        self.assertEqual(code, 3)
        self.assertFalse(data["HA-0"]["a"]["pass"])
        self.assertIn("Series stops", out)

    def test_ha0_fails_on_kernel_removed_drift_and_stops_stage1(self):
        recs = stage0(b=(0.60, 0.61, 0.612)) + stage1((0.96,) * 3, (0.94,) * 3, (0.70,) * 3)
        code, out, data = self.run_corpus(recs)
        self.assertEqual(code, 3)
        self.assertNotIn("HA-1", data)

    def test_ha0_bar_is_inclusive(self):
        # mean exactly 0.9376 - 0.010 = 0.9276 sits on the bar.
        code, _, data = self.run_corpus(stage0(a=(0.9276, 0.9276, 0.9276), rerun=0.9276))
        self.assertEqual(data["HA-0"]["status"], "PASS")

    def test_identical_rerun_is_reported(self):
        _, out, data = self.run_corpus(stage0(rerun=0.938))
        self.assertTrue(data["HA-0"]["rerun"]["identical"])
        self.assertIn("(identical)", out)

    # --- contract -----------------------------------------------------------
    def test_mps_record_is_invalid(self):
        recs = stage0()
        recs[0]["device"] = "mps"
        code, out, _ = self.run_corpus(recs)
        self.assertEqual(code, 1)
        self.assertIn("device", out)

    def test_unregistered_configuration_fails_closed(self):
        an.CONFIG = None
        try:
            code, out, _ = self.run_corpus(stage0())
        finally:
            an.CONFIG = dict(SAMPLE_CONFIG)
        self.assertEqual(code, 1)
        self.assertIn("no run configuration is registered", out)

    def test_record_at_another_configuration_is_invalid(self):
        recs = stage0()
        recs[3]["mkldnn"] = "on"  # the rerun at the archived oneDNN setting
        code, out, _ = self.run_corpus(recs)
        self.assertEqual(code, 1)
        self.assertIn("mkldnn", out)

    def test_rerun_on_the_same_box_is_invalid(self):
        recs = stage0()
        recs[3]["instance_id"] = recs[0]["instance_id"]  # A00r on A00 seed 5170001's box
        code, out, _ = self.run_corpus(recs)
        self.assertEqual(code, 1)
        self.assertIn("different box", out)

    def test_missing_instance_id_is_invalid(self):
        recs = stage0()
        del recs[5]["instance_id"]
        code, _, _ = self.run_corpus(recs)
        self.assertEqual(code, 1)

    def test_arm_fields_must_agree(self):
        recs = stage0()
        recs[4]["max_delay"] = 25  # a B00 record claiming the kernel
        code, out, _ = self.run_corpus(recs)
        self.assertEqual(code, 1)
        self.assertIn("arm B00 needs max_delay", out)

    def test_duplicate_cell_is_invalid(self):
        recs = stage0() + [record("A00", 5170002, 0.937)]
        code, out, _ = self.run_corpus(recs)
        self.assertEqual(code, 1)
        self.assertIn("duplicate cell", out)

    def test_unregistered_seed_is_invalid(self):
        recs = stage0() + [record("B10", 5170004, 0.7)]
        code, out, _ = self.run_corpus(recs)
        self.assertEqual(code, 1)
        self.assertIn("not a registered cell", out)

    def test_mixed_memo_is_invalid(self):
        recs = stage0()
        recs[2]["memo"] = False
        code, out, _ = self.run_corpus(recs)
        self.assertEqual(code, 1)
        self.assertIn("memo differs", out)

    def test_test_read_during_training_is_invalid(self):
        recs = stage0()
        recs[1]["test_reads_during_training"] = 1
        code, _, _ = self.run_corpus(recs)
        self.assertEqual(code, 1)

    # --- stage 1 verdicts ---------------------------------------------------
    def test_met_with_order_dependence(self):
        recs = stage0() + stage1((0.962, 0.960, 0.961), (0.940, 0.937, 0.940), (0.75, 0.76, 0.74))
        code, out, data = self.run_corpus(recs)
        self.assertEqual(code, 0, out)
        self.assertEqual(data["HA-1"]["verdict"], "MET")
        self.assertEqual(data["HA-1"]["n"], 3)
        self.assertTrue(data["HA-2"]["met"])
        self.assertFalse(data["stage2_triggered"])
        self.assertTrue(data["HA-4"]["met"])

    def test_g_exactly_on_stage1_bar_is_met(self):
        a00 = (0.938, 0.936, 0.939)
        recs = stage0(a=a00) + stage1(tuple(x + 0.020 for x in a00), a00, (0.75,) * 3)
        _, _, data = self.run_corpus(recs)
        self.assertEqual(data["stage1"]["HA-1"], "MET")

    def test_g_just_under_stage1_bar_is_ambiguous(self):
        a00 = (0.938, 0.936, 0.939)
        recs = stage0(a=a00) + stage1(tuple(x + 0.0195 for x in a00), a00, (0.75,) * 3)
        _, _, data = self.run_corpus(recs)
        self.assertEqual(data["stage1"]["HA-1"], "AMBIGUOUS")

    def test_refuted(self):
        recs = stage0() + stage1((0.938, 0.935, 0.939), (0.937, 0.936, 0.938), (0.63, 0.63, 0.63))
        code, _, data = self.run_corpus(recs)
        self.assertEqual(code, 0)
        self.assertEqual(data["HA-1"]["verdict"], "REFUTED")
        self.assertIn("status", data["HA-2"])  # HA-2 not read

    def test_ambiguous_triggers_stage2_and_waits(self):
        # G = +0.010, 3/3 positive: inside [0.003, 0.020) -> stage 2 required.
        recs = stage0() + stage1((0.948, 0.946, 0.949), (0.94, 0.94, 0.94), (0.70,) * 3)
        code, out, data = self.run_corpus(recs)
        self.assertEqual(code, 2)
        self.assertTrue(data["stage2_triggered"])
        self.assertIn("stage 2: TRIGGERED, NOT EVALUABLE", out)

    def test_large_gain_with_two_of_three_is_ambiguous(self):
        recs = stage0() + stage1((0.97, 0.97, 0.938), (0.94, 0.94, 0.94), (0.70,) * 3)
        _, _, data = self.run_corpus(recs)
        self.assertEqual(data["stage1"]["HA-1"], "AMBIGUOUS")

    def test_stage2_read_at_n5(self):
        recs = stage0() + stage1((0.948, 0.946, 0.949), (0.94, 0.94, 0.94), (0.70,) * 3)
        recs += [record("A00", s, 0.937) for s in S2]
        recs += [record("A10", s, 0.949) for s in S2]
        recs += [record("A10h", s, 0.938) for s in S2]
        code, out, data = self.run_corpus(recs)
        self.assertEqual(code, 0, out)
        self.assertEqual(data["HA-1"]["n"], 5)
        self.assertEqual(data["HA-1"]["verdict"], "MET")  # G ~ +0.0114 >= 0.010, 5/5

    def test_noisy_rerun_forces_stage2_even_when_met(self):
        recs = stage0(rerun=0.930) + stage1((0.962, 0.960, 0.961), (0.940, 0.937, 0.940), (0.75,) * 3)
        code, _, data = self.run_corpus(recs)
        self.assertEqual(code, 2)
        self.assertTrue(data["stage2_triggered"])

    def test_untriggered_stage2_cells_are_invalid(self):
        recs = stage0() + stage1((0.962, 0.960, 0.961), (0.940, 0.937, 0.940), (0.75,) * 3)
        recs.append(record("A10", 5170004, 0.96))
        code, out, _ = self.run_corpus(recs)
        self.assertEqual(code, 1)
        self.assertIn("not triggered", out)

    def test_headroom_share_uses_this_series_a00(self):
        recs = stage0() + stage1((0.962, 0.960, 0.961), (0.940, 0.937, 0.940), (0.75,) * 3)
        _, _, data = self.run_corpus(recs)
        a00 = sum((0.938, 0.936, 0.939)) / 3
        g = sum((0.962, 0.960, 0.961)) / 3 - a00
        self.assertAlmostEqual(data["HA-3"]["share"], g / (1 - a00))


if __name__ == "__main__":
    unittest.main()
