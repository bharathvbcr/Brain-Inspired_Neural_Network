"""The run-copy patch: disabled it changes nothing, enabled it trains, shuffled it moves no draw (prereg §3, §8).

Builds run copies of the pinned reference (d169b4e) in a temporary directory the way the campaign's
bootstrap builds them: `num_workers=4 -> 0` (asserted 8 times, as `bootstrap.sh` asserts it). Then
max_delay 25 (A arms) or the kernel ablation's max_delay 1 text patch (B arms), and the read-out
patch or not. Each copy runs `forward_probe.py` in its own subprocess. The probe takes one training
step and one evaluation through the copy's own `SHD_dataloaders`, with a synthetic dataset whose
longest sample is T_in frames.

Every check runs at three batch input lengths T_in. `pad_sequence_collate` pads each batch to its
longest sample, so the length the read-out sees varies by batch (amendment 2026-11-11). Measured
over 64,000 simulated shuffled training batches, the padded T_in runs from 89 to 124. The lengths
here are those two extremes and the 100 that the registration assumed.

- **Installed but disabled is bit-identical** to the unpatched reference: the batch, output, loss,
  gradients, state_dict, the read-out's input, the RNG state after the step, and the evaluation
  loss.
- **The read-out (A10/B10)** runs with T_h = T_in + 24 at max_delay 25 and T_in at max_delay 1. It
  leaves the substrate and the RNG alone, and trains in the weight group only.
- **The hidden shuffle (A10ₕ)** pairs with the read-out arm: the same batch, read-out input and RNG
  state, and a different W_o gradient. The first loss is equal, because W_o = 0 at
  initialisation makes the read-out a mean over time, which no permutation changes. Every training and test batch finds its sample indices. No row of
  the permutation is the identity.
- **Composition with the memo patch** is checked when `ATTN_HARNESS_PATCH` names the campaign's
  `harness_patch.py`, which is not in BINN. Otherwise that test is skipped, and says so. The
  prereg §8 hooks-disabled check on the campaign instance type is the authoritative test of the
  campaign's full composition.

run: python -m unittest test_readout_patch   (from this directory, in the reference env)
"""

from __future__ import annotations

import importlib.util
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
HARNESS_PATCH = os.environ.get("ATTN_HARNESS_PATCH")
B_PATCH = ("    max_delay = 250//time_step\n", "    max_delay = 1\n")
BATCH, SEED = 6, 5170001
MAX_DELAYS = (25, 1)
T_IN = (100, 89, 124)
KINDS = {"reference": 0, "disabled": 0, "enabled": 1, "shuffled": 2}


def expected_lengths(max_delay: int, t_in: int) -> tuple[int, int]:
    """(T_h, T_out): each Dcls1d of the pinned forward adds 12 steps at max_delay 25, none at 1."""
    growth = 12 if max_delay == 25 else 0
    return t_in + 2 * growth, t_in + 3 * growth


def load_harness_patch():
    spec = importlib.util.spec_from_file_location("harness_patch", HARNESS_PATCH)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def make_copy(root: Path, name: str, *, max_delay: int, patched: bool, memo: bool = False) -> Path:
    copy = root / name
    shutil.copytree(CHECKOUT, copy, ignore=shutil.ignore_patterns(".git"))
    datasets = copy / "datasets.py"
    text = datasets.read_text()
    if text.count("num_workers=4") != 8:
        raise RuntimeError("expected num_workers=4 eight times in datasets.py, as bootstrap.sh asserts")
    datasets.write_text(text.replace("num_workers=4", "num_workers=0"))
    if max_delay == 1:
        config = copy / "config.py"
        text = config.read_text()
        if text.count(B_PATCH[0]) != 1:
            raise RuntimeError("expected exactly one max_delay line in config.py")
        config.write_text(text.replace(*B_PATCH, 1))
    if memo:
        load_harness_patch().apply(datasets)
    if patched:
        readout_patch.apply(copy)
    return copy


def probe(copy: Path, mode: int, t_in: int) -> dict:
    env = dict(os.environ, PYTHONPATH=str(copy), PYTHONHASHSEED="5170001", WANDB_MODE="disabled",
               OMP_NUM_THREADS="1")
    done = subprocess.run(
        [sys.executable, str(HERE / "forward_probe.py"), str(mode), str(BATCH), str(SEED), str(t_in)],
        cwd=copy, env=env, capture_output=True, text=True, timeout=900)
    if done.returncode != 0:
        raise RuntimeError(f"probe failed in {copy.name}, mode {mode}, T_in {t_in}:\n{done.stderr[-3000:]}")
    return json.loads(done.stdout.strip().splitlines()[-1])


class ReadoutPatch(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        head = subprocess.run(["git", "-C", str(CHECKOUT), "rev-parse", "HEAD"],
                              capture_output=True, text=True, check=True).stdout.strip()
        if head != PINNED:
            raise RuntimeError(f"{CHECKOUT} is at {head}, not the pinned {PINNED}")
        cls.tmp = tempfile.TemporaryDirectory(prefix="readout-patch-")
        cls.root = Path(cls.tmp.name)
        cls.copies = {}
        cls.results = {}
        for max_delay in MAX_DELAYS:
            plain = make_copy(cls.root, f"plain-{max_delay}", max_delay=max_delay, patched=False)
            patched = make_copy(cls.root, f"patched-{max_delay}", max_delay=max_delay, patched=True)
            cls.copies[max_delay] = patched
            for t_in in T_IN:
                cls.results[(max_delay, t_in)] = {
                    kind: probe(plain if kind == "reference" else patched, mode, t_in)
                    for kind, mode in KINDS.items()
                }

    @classmethod
    def tearDownClass(cls) -> None:
        cls.tmp.cleanup()

    def test_every_case_ran(self) -> None:
        self.assertEqual(set(self.results), {(m, t) for m in MAX_DELAYS for t in T_IN})
        for runs in self.results.values():
            self.assertEqual(set(runs), set(KINDS))

    def test_disabled_is_bit_identical_to_the_reference(self) -> None:
        for case, runs in self.results.items():
            reference, disabled = runs["reference"], runs["disabled"]
            self.assertFalse(disabled["attached"] or disabled["shuffled"])
            for key in ("batch_sha", "output_sha", "loss_bits", "reference_grad_sha", "state_keys", "state_sha",
                        "readout_input_sha", "rng_after_sha", "t_h", "t_out", "eval_loss_bits",
                        "train_chain", "test_chain"):
                self.assertEqual(disabled[key], reference[key], f"(max_delay, T_in) {case}: {key}")

    def test_t_h_follows_the_batch_length(self) -> None:
        for (max_delay, t_in), runs in self.results.items():
            t_h, t_out = expected_lengths(max_delay, t_in)
            for kind, run in runs.items():
                self.assertEqual((run["max_delay"], run["t_in"]), (max_delay, t_in), kind)
                self.assertEqual((run["t_h"], run["t_out"]), (t_h, t_out),
                                 f"{kind} at max_delay {max_delay}, T_in {t_in}")
        # The registration's 124 / 100 is the T_in = 100 case of the relation.
        self.assertEqual(expected_lengths(25, 100), (124, 136))
        self.assertEqual(expected_lengths(1, 100), (100, 100))

    def test_enabled_leaves_the_substrate_and_its_draws_alone(self) -> None:
        for case, runs in self.results.items():
            enabled, reference = runs["enabled"], runs["reference"]
            self.assertTrue(enabled["attached"])
            self.assertFalse(enabled["shuffled"])
            for key in ("batch_sha", "readout_input_sha", "rng_after_sha"):
                self.assertEqual(enabled[key], reference[key], f"{case}: {key}")
            self.assertNotEqual(enabled["loss_bits"], reference["loss_bits"],
                                "W_a is non-zero at initialisation, so the read-out must move the loss")

    def test_shuffled_pairs_with_the_read_out_arm(self) -> None:
        for case, runs in self.results.items():
            shuffled, enabled = runs["shuffled"], runs["enabled"]
            self.assertTrue(shuffled["attached"] and shuffled["shuffled"])
            # Same batch, same substrate output, same draws: the shuffle moved no RNG.
            for key in ("batch_sha", "readout_input_sha", "rng_after_sha", "t_h"):
                self.assertEqual(shuffled[key], enabled[key], f"{case}: {key}")
            # At initialisation W_o = 0, so the blocks add nothing and pooled = mean_t(W_e s + pos),
            # which no permutation of s changes: the first loss is the read-out arm's. Order enters
            # through the attention pattern, so the first gradient that sees it is W_o's.
            self.assertNotEqual(shuffled["readout"]["w_o_grad_sha"], enabled["readout"]["w_o_grad_sha"],
                                f"{case}: the shuffle did not reach the read-out")
            report = shuffled["shuffle"]
            self.assertEqual(report["train_shape"], [BATCH, shuffled["t_h"]])
            self.assertEqual(report["train_identity_rows"], 0, f"{case}")
            # 1 training batch + 1 test batch (eval_model) + 2 more training passes.
            self.assertEqual(report["presentations"], 4, f"{case}")
            self.assertEqual(report["extra_batches"], 2, f"{case}")
            self.assertEqual(report["pending_after"], 0, f"{case}")
            self.assertEqual(shuffled["train_chain"], ["IndexRecorder", "FakeShd"])
            self.assertEqual(shuffled["test_chain"], ["IndexRecorder", "FakeShd"])

    def test_readout_reads_post_dropout_spikes(self) -> None:
        # The tensor the read-out reads holds 0 and the dropout-scaled spike 1/(1-p) in training,
        # which is the value the parity fixture's `dropout` sample carries.
        for case, runs in self.results.items():
            for kind in ("enabled", "shuffled"):
                run = runs[kind]
                scaled = float(np.float32(1.0) / (np.float32(1.0) - np.float32(run["dropout_p"])))
                values = run["readout_input_values"]
                self.assertEqual(len(values), 2, f"{case} {kind}: {values}")
                self.assertEqual(values[0], 0.0, f"{case} {kind}")
                self.assertAlmostEqual(values[1], scaled, places=6, msg=f"{case} {kind}: {values}")

    def test_read_out_trains_in_the_weight_group(self) -> None:
        for case, runs in self.results.items():
            for kind in ("enabled", "shuffled"):
                readout = runs[kind]["readout"]
                self.assertTrue(readout["in_weight_group"], f"{case} {kind}")
                self.assertFalse(readout["in_other_groups"], f"{case} {kind}")
                self.assertTrue(readout["grads_present"] and readout["grads_finite"], f"{case} {kind}")
                self.assertTrue(readout["w_o_zero_at_init"], f"{case} {kind}")
                self.assertTrue(readout["w_o_moved_by_step"], f"{case} {kind}: the optimiser step did not run")
                self.assertGreater(readout["w_o_grad_abs_sum"], 0.0, f"{case} {kind}")

    def test_shuffle_without_the_read_out_is_refused(self) -> None:
        copy = self.copies[25]
        env = dict(os.environ, PYTHONPATH=str(copy), WANDB_MODE="disabled")
        code = ("from config import Config\nfrom snn_delays import SnnDelays\n"
                "Config.hidden_shuffle = True\nSnnDelays(Config)\n")
        done = subprocess.run([sys.executable, "-c", code], cwd=copy, env=env,
                              capture_output=True, text=True, timeout=600)
        self.assertNotEqual(done.returncode, 0)
        self.assertIn("it needs attention_readout", done.stderr)

    def test_patch_refuses_a_text_it_does_not_recognise(self) -> None:
        text = (CHECKOUT / "snn_delays.py").read_text()
        with self.assertRaises(RuntimeError):
            readout_patch.patch_text(text.replace("from utils import set_seed\n", "", 1))
        patched = readout_patch.patch_text(text)
        with self.assertRaises(RuntimeError):
            readout_patch.patch_text(patched)  # applying it twice is refused
        pinned_datasets = (CHECKOUT / "datasets.py").read_text()
        with self.assertRaises(RuntimeError):
            readout_patch.patch_datasets_text(pinned_datasets)  # num_workers=4: a worker would record
        datasets = pinned_datasets.replace("num_workers=4", "num_workers=0")
        patched = readout_patch.patch_datasets_text(datasets)
        self.assertEqual(patched.count("IndexRecorder("), 2)
        with self.assertRaises(RuntimeError):
            readout_patch.patch_datasets_text(patched)

    def test_composes_with_the_memo_patch_as_the_outermost_wrapper(self) -> None:
        if not HARNESS_PATCH:
            self.skipTest("ATTN_HARNESS_PATCH not set: memo composition NOT checked "
                          "(binn-wsp-study/aws_attn/harness_patch.py is not in BINN)")
        memo = make_copy(self.root, "memo-patched-25", max_delay=25, patched=True, memo=True)
        shuffled = probe(memo, 2, 100)
        enabled = probe(memo, 1, 100)
        self.assertEqual(shuffled["train_chain"], ["IndexRecorder", "_MemoisedDataset", "FakeShd"])
        self.assertEqual(shuffled["test_chain"], ["IndexRecorder", "_MemoisedDataset", "FakeShd"])
        # The second and third passes are served from the memo; the recorder still sees them.
        self.assertEqual(shuffled["shuffle"]["extra_batches"], 2)
        self.assertEqual(shuffled["shuffle"]["pending_after"], 0)
        # The memo is bit-exact, so the memo copy's arms equal the plain copy's.
        for kind, run in (("shuffled", shuffled), ("enabled", enabled)):
            plain = self.results[(25, 100)][kind]
            for key in ("batch_sha", "output_sha", "loss_bits", "readout_input_sha", "rng_after_sha",
                        "eval_loss_bits"):
                self.assertEqual(run[key], plain[key], f"memo {kind}: {key}")
        # Applying the memo after the recorder is refused, so the wrong order cannot be built.
        wrong = make_copy(self.root, "wrong-order-25", max_delay=25, patched=True)
        with self.assertRaises(RuntimeError):
            load_harness_patch().apply(wrong / "datasets.py")


if __name__ == "__main__":
    unittest.main()
