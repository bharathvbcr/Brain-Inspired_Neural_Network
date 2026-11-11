"""The run-copy patch: disabled it changes nothing, enabled it trains (prereg §3, §8).

Builds run copies of the pinned reference (d169b4e) in a temporary directory, unpatched and
patched, at max_delay 25 (A arms) and at max_delay 1 (B arms, by the kernel ablation's own
config.py text patch, as `body_cell.sh` applies it). Each copy runs `forward_probe.py` in its own
subprocess: one training-mode forward, loss and backward on the same fixed synthetic batch.

Every check runs at three batch input lengths T_in. `pad_sequence_collate` pads each batch to its
longest sample, so the length the read-out sees varies by batch (amendment 2026-11-11). Measured
over 64,000 simulated shuffled training batches, the padded T_in runs from 89 to 124. The lengths
here are those two extremes and the 100 that the registration assumed.

- **Disabled is bit-identical.** With the patch applied and the read-out off, the output, loss,
  gradients, state_dict (keys and bytes), the captured read-out input and the global RNG state
  after the step all equal the unpatched reference's.
- **Enabled runs, with T_h following the batch.** T_h = T_in + 24 at max_delay 25 and T_in at
  max_delay 1, and the output sequence is T_in + 36 and T_in. The substrate's input to the
  read-out and the RNG state equal the reference's, so attaching it moved no draw. Its parameters
  are in the weight group only, every one has a finite gradient, and W_o, zero at
  initialisation, has a non-zero gradient, as the Rust module's documentation says it must at
  the first step.

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
MAX_DELAYS = (25, 1)
T_IN = (100, 89, 124)


def expected_lengths(max_delay: int, t_in: int) -> tuple[int, int]:
    """(T_h, T_out): each Dcls1d of the pinned forward adds 12 steps at max_delay 25, none at 1."""
    growth = 12 if max_delay == 25 else 0
    return t_in + 2 * growth, t_in + 3 * growth


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


def probe(copy: Path, attention: bool, t_in: int) -> dict:
    env = dict(os.environ, PYTHONPATH=str(copy), PYTHONHASHSEED="5170001", WANDB_MODE="disabled",
               OMP_NUM_THREADS="1")
    done = subprocess.run(
        [sys.executable, str(HERE / "forward_probe.py"), "1" if attention else "0", str(BATCH), str(SEED),
         str(t_in)],
        cwd=copy, env=env, capture_output=True, text=True, timeout=600)
    if done.returncode != 0:
        raise RuntimeError(f"probe failed in {copy.name} at T_in {t_in}:\n{done.stderr[-3000:]}")
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
        for max_delay in MAX_DELAYS:
            plain = make_copy(root, f"plain-{max_delay}", max_delay=max_delay, patched=False)
            patched = make_copy(root, f"patched-{max_delay}", max_delay=max_delay, patched=True)
            for t_in in T_IN:
                cls.results[(max_delay, t_in)] = {
                    "reference": probe(plain, attention=False, t_in=t_in),
                    "disabled": probe(patched, attention=False, t_in=t_in),
                    "enabled": probe(patched, attention=True, t_in=t_in),
                }

    @classmethod
    def tearDownClass(cls) -> None:
        cls.tmp.cleanup()

    def test_every_length_ran(self) -> None:
        self.assertEqual(set(self.results), {(m, t) for m in MAX_DELAYS for t in T_IN})

    def test_disabled_is_bit_identical_to_the_reference(self) -> None:
        for case, runs in self.results.items():
            reference, disabled = runs["reference"], runs["disabled"]
            self.assertFalse(disabled["attached"])
            for key in ("output_sha", "loss_bits", "reference_grad_sha", "state_keys", "state_sha",
                        "readout_input_sha", "rng_after_sha", "t_h", "t_out"):
                self.assertEqual(disabled[key], reference[key], f"(max_delay, T_in) {case}: {key}")

    def test_t_h_follows_the_batch_length(self) -> None:
        for (max_delay, t_in), runs in self.results.items():
            t_h, t_out = expected_lengths(max_delay, t_in)
            for kind in ("reference", "enabled"):
                run = runs[kind]
                self.assertEqual((run["max_delay"], run["t_in"]), (max_delay, t_in))
                self.assertEqual((run["t_h"], run["t_out"]), (t_h, t_out),
                                 f"{kind} at max_delay {max_delay}, T_in {t_in}")
        # The registration's 124 / 100 is the T_in = 100 case of the relation.
        self.assertEqual(expected_lengths(25, 100), (124, 136))
        self.assertEqual(expected_lengths(1, 100), (100, 100))

    def test_enabled_leaves_the_substrate_and_its_draws_alone(self) -> None:
        for case, runs in self.results.items():
            enabled, reference = runs["enabled"], runs["reference"]
            self.assertTrue(enabled["attached"])
            self.assertEqual(enabled["readout_input_sha"], reference["readout_input_sha"], f"{case}")
            self.assertEqual(enabled["rng_after_sha"], reference["rng_after_sha"], f"{case}")
            self.assertNotEqual(enabled["loss_bits"], reference["loss_bits"],
                                "W_a is non-zero at initialisation, so the read-out must move the loss")

    def test_readout_reads_post_dropout_spikes(self) -> None:
        # The tensor the read-out reads holds 0 and the dropout-scaled spike 1/(1-p) in training,
        # which is the value the parity fixture's `dropout` sample carries.
        for case, runs in self.results.items():
            run = runs["enabled"]
            scaled = float(np.float32(1.0) / (np.float32(1.0) - np.float32(run["dropout_p"])))
            values = run["readout_input_values"]
            self.assertEqual(values[0], 0.0, f"{case}")
            self.assertEqual(len(values), 2, f"{case}: {values}")
            self.assertAlmostEqual(values[1], scaled, places=6, msg=f"{case}: {values}")

    def test_enabled_trains_in_the_weight_group(self) -> None:
        for case, runs in self.results.items():
            readout = runs["enabled"]["readout"]
            self.assertTrue(readout["in_weight_group"], f"{case}")
            self.assertFalse(readout["in_other_groups"], f"{case}")
            self.assertTrue(readout["grads_present"] and readout["grads_finite"], f"{case}")
            self.assertTrue(readout["w_o_is_zero"])
            self.assertGreater(readout["w_o_grad_abs_sum"], 0.0, f"{case}")

    def test_patch_refuses_a_text_it_does_not_recognise(self) -> None:
        text = (CHECKOUT / "snn_delays.py").read_text()
        with self.assertRaises(RuntimeError):
            readout_patch.patch_text(text.replace("from utils import set_seed\n", "", 1))
        patched = readout_patch.patch_text(text)
        with self.assertRaises(RuntimeError):
            readout_patch.patch_text(patched)  # applying it twice is refused


if __name__ == "__main__":
    unittest.main()
