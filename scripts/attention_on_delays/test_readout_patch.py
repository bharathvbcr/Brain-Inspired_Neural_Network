"""The run-copy patch: disabled it changes nothing, enabled it trains (prereg §3, §8).

Builds run copies of the pinned reference (d169b4e) in a temporary directory, unpatched and
patched, at max_delay 25 (A arms) and at max_delay 1 (B arms, by the kernel ablation's own
config.py text patch, as `body_cell.sh` applies it). Each copy runs `forward_probe.py` in its own
subprocess: one training-mode forward, loss and backward on the same fixed synthetic batch.

- **Disabled is bit-identical.** With the patch applied and the read-out off, the output, loss,
  gradients, state_dict (keys and bytes), the captured read-out input and the global RNG state
  after the step all equal the unpatched reference's.
- **Enabled runs, at the registered lengths.** T_h is 124 at max_delay 25 and 100 at max_delay 1
  (and the output sequence 136 and 100). The substrate's input to the read-out and the RNG state
  equal the reference's, so attaching it moved no draw. Its parameters are in the weight group
  only, every one has a finite gradient, and W_o, zero at initialisation, has a non-zero
  gradient, as the Rust module's documentation says it must at the first step.

The prereg's own §8 gate runs these on the campaign instance type; this test is the local form.

run: python -m unittest test_readout_patch   (from this directory, in the reference env)
"""

from __future__ import annotations

import json
import os
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

import numpy as np

import readout_patch

HERE = Path(__file__).resolve().parent
PINNED = "d169b4e3049a3d5bff56c84a8b2f0c4e835aafda"
CHECKOUT = Path(os.environ.get(
    "ATTN_REFERENCE_CHECKOUT",
    HERE.parents[1] / "results/shd_instrument_v4/reference-cache/SNN-delays"))
B_PATCH = ("    max_delay = 250//time_step\n", "    max_delay = 1\n")
BATCH, SEED = 6, 5170001
LENGTHS = {25: (124, 136), 1: (100, 100)}


def make_copy(root: Path, name: str, *, max_delay: int, patched: bool) -> Path:
    copy = root / name
    shutil.copytree(CHECKOUT, copy, ignore=shutil.ignore_patterns(".git"))
    if max_delay == 1:
        config = copy / "config.py"
        text = config.read_text()
        if text.count(B_PATCH[0]) != 1:
            raise RuntimeError("expected exactly one max_delay line in config.py")
        config.write_text(text.replace(*B_PATCH, 1))
    if patched:
        readout_patch.apply(copy)
    return copy


def probe(copy: Path, attention: bool) -> dict:
    env = dict(os.environ, PYTHONPATH=str(copy), PYTHONHASHSEED="5170001", WANDB_MODE="disabled",
               OMP_NUM_THREADS="1")
    done = subprocess.run(
        [sys.executable, str(HERE / "forward_probe.py"), "1" if attention else "0", str(BATCH), str(SEED)],
        cwd=copy, env=env, capture_output=True, text=True, timeout=600)
    if done.returncode != 0:
        raise RuntimeError(f"probe failed in {copy.name}:\n{done.stderr[-3000:]}")
    return json.loads(done.stdout.strip().splitlines()[-1])


class ReadoutPatch(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        head = subprocess.run(["git", "-C", str(CHECKOUT), "rev-parse", "HEAD"],
                              capture_output=True, text=True, check=True).stdout.strip()
        if head != PINNED:
            raise RuntimeError(f"{CHECKOUT} is at {head}, not the pinned {PINNED}")
        cls.tmp = tempfile.TemporaryDirectory(prefix="readout-patch-")
        root = Path(cls.tmp.name)
        cls.results = {}
        for max_delay in LENGTHS:
            plain = make_copy(root, f"plain-{max_delay}", max_delay=max_delay, patched=False)
            patched = make_copy(root, f"patched-{max_delay}", max_delay=max_delay, patched=True)
            cls.results[max_delay] = {
                "reference": probe(plain, attention=False),
                "disabled": probe(patched, attention=False),
                "enabled": probe(patched, attention=True),
            }

    @classmethod
    def tearDownClass(cls) -> None:
        cls.tmp.cleanup()

    def test_disabled_is_bit_identical_to_the_reference(self) -> None:
        for max_delay, runs in self.results.items():
            reference, disabled = runs["reference"], runs["disabled"]
            self.assertFalse(disabled["attached"])
            for key in ("output_sha", "loss_bits", "reference_grad_sha", "state_keys", "state_sha",
                        "readout_input_sha", "rng_after_sha", "t_h", "t_out"):
                self.assertEqual(disabled[key], reference[key], f"max_delay {max_delay}: {key}")

    def test_enabled_lengths_are_the_registered_ones(self) -> None:
        for max_delay, (t_h, t_out) in LENGTHS.items():
            for kind in ("reference", "enabled"):
                run = self.results[max_delay][kind]
                self.assertEqual(run["max_delay"], max_delay)
                self.assertEqual((run["t_h"], run["t_out"]), (t_h, t_out), f"{kind} at max_delay {max_delay}")

    def test_enabled_leaves_the_substrate_and_its_draws_alone(self) -> None:
        for max_delay, runs in self.results.items():
            enabled, reference = runs["enabled"], runs["reference"]
            self.assertTrue(enabled["attached"])
            self.assertEqual(enabled["readout_input_sha"], reference["readout_input_sha"], f"max_delay {max_delay}")
            self.assertEqual(enabled["rng_after_sha"], reference["rng_after_sha"], f"max_delay {max_delay}")
            self.assertNotEqual(enabled["loss_bits"], reference["loss_bits"],
                                "W_a is non-zero at initialisation, so the read-out must move the loss")

    def test_readout_reads_post_dropout_spikes(self) -> None:
        # The tensor the read-out reads holds 0 and the dropout-scaled spike 1/(1-p) in training,
        # which is the value the parity fixture's `dropout` sample carries.
        for max_delay, runs in self.results.items():
            run = runs["enabled"]
            scaled = float(np.float32(1.0) / (np.float32(1.0) - np.float32(run["dropout_p"])))
            values = run["readout_input_values"]
            self.assertEqual(values[0], 0.0, f"max_delay {max_delay}")
            self.assertEqual(len(values), 2, f"max_delay {max_delay}: {values}")
            self.assertAlmostEqual(values[1], scaled, places=6, msg=f"max_delay {max_delay}: {values}")

    def test_enabled_trains_in_the_weight_group(self) -> None:
        for max_delay, runs in self.results.items():
            readout = runs["enabled"]["readout"]
            self.assertTrue(readout["in_weight_group"], f"max_delay {max_delay}")
            self.assertFalse(readout["in_other_groups"], f"max_delay {max_delay}")
            self.assertTrue(readout["grads_present"] and readout["grads_finite"], f"max_delay {max_delay}")
            self.assertTrue(readout["w_o_is_zero"])
            self.assertGreater(readout["w_o_grad_abs_sum"], 0.0, f"max_delay {max_delay}")

    def test_patch_refuses_a_text_it_does_not_recognise(self) -> None:
        text = (CHECKOUT / "snn_delays.py").read_text()
        with self.assertRaises(RuntimeError):
            readout_patch.patch_text(text.replace("from utils import set_seed\n", "", 1))
        patched = readout_patch.patch_text(text)
        with self.assertRaises(RuntimeError):
            readout_patch.patch_text(patched)  # applying it twice is refused


if __name__ == "__main__":
    unittest.main()
