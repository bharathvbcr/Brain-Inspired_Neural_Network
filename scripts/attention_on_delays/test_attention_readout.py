"""Parity of the PyTorch read-out with the Rust instrument (prereg §3, implementation parity gate).

The fixture `fixtures/attention_parity_fixture.json` is written by `fixture-gen` from
`binn-learn/src/shd_attention.rs` itself (`AttentionParams::deterministic`, `positional_code`,
`attention_forward_presented`, `attention_logits`); every float in it is float32 bits.

The bar is the registered one: `pooled` and `a` within 1e-5 max-abs at T_h = 124 and 100. The
initial weights and the positional code are held to bit equality, because both are computed
with the same float32 operations on both sides and any difference would be a porting error.

run: python -m unittest test_attention_readout   (from this directory, in the reference env)
"""

from __future__ import annotations

import json
import unittest
from pathlib import Path

import numpy as np
import torch

import attention_readout as ar

HERE = Path(__file__).resolve().parent
FIXTURE = HERE / "fixtures" / "attention_parity_fixture.json"
PARITY_BAR = 1e-5
T_H = (124, 100)


def f32(hexed: str) -> np.ndarray:
    return np.frombuffer(bytes.fromhex(hexed), dtype="<f4").copy()


def bits(values: np.ndarray) -> np.ndarray:
    return np.ascontiguousarray(values, dtype="<f4").view("<u4")


def fixture_params(entry: dict, fx: dict) -> dict[str, np.ndarray]:
    d, hidden, classes = fx["d_model"], fx["hidden"], fx["n_classes"]
    params = {"w_e": f32(entry["w_e"]).reshape(hidden, d), "w_a": f32(entry["w_a"]).reshape(classes, d)}
    for name in ("w_q", "w_k", "w_v", "w_o"):
        params[name] = np.stack([f32(block[name]).reshape(d, d) for block in entry["blocks"]])
    return params


class ParityWithRust(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.fx = json.loads(FIXTURE.read_text())
        fx = cls.fx
        assert fx["schema"] == "attention-parity-fixture-v1"
        assert (fx["hidden"], fx["n_classes"], fx["d_model"], fx["layers"]) == (256, 20, 32, 4)

    def test_initialisation_is_attention_params_deterministic_bit_for_bit(self) -> None:
        mine = ar.deterministic_params(256, 20, self.fx["seed"])
        rust = fixture_params(self.fx["params"]["init"], self.fx)
        for name in ("w_e", "w_q", "w_k", "w_v", "w_o", "w_a"):
            self.assertEqual(mine[name].shape, rust[name].shape, name)
            self.assertTrue(np.array_equal(bits(mine[name]), bits(rust[name])), f"{name} differs from Rust")

    def test_initialisation_scheme(self) -> None:
        params = ar.deterministic_params(256, 20, 5170002)
        self.assertTrue((params["w_o"] == 0).all(), "W_o must start at zero")
        for name, fan_in, fan_out in (("w_e", 256, 32), ("w_q", 32, 32), ("w_k", 32, 32),
                                      ("w_v", 32, 32), ("w_a", 32, 20)):
            limit = ar.glorot_limit(fan_in, fan_out)
            self.assertAlmostEqual(float(limit), float(np.sqrt(6.0 / (fan_in + fan_out))), places=6)
            values = params[name]
            self.assertTrue((np.abs(values) <= limit).all(), f"{name} outside its Glorot bound")
            self.assertGreater(float(np.abs(values).max()), 0.9 * float(limit), f"{name} does not span its bound")
        # Different seeds give different draws; the series seeds are distinct initialisations.
        other = ar.deterministic_params(256, 20, 5170003)
        self.assertFalse(np.array_equal(params["w_e"], other["w_e"]))

    def test_module_draws_nothing_from_torch_global_rng(self) -> None:
        torch.manual_seed(123)
        before = torch.get_rng_state().clone()
        module = ar.AttentionReadout(256, 20, 5170001)
        self.assertTrue(torch.equal(before, torch.get_rng_state()))
        rust = fixture_params(self.fx["params"]["init"], self.fx)
        for name in ("w_e", "w_q", "w_k", "w_v", "w_o", "w_a"):
            self.assertTrue(np.array_equal(bits(getattr(module, name).detach().numpy()), bits(rust[name])))

    def test_tau_cycles_are_the_registered_ladder(self) -> None:
        constants = ar.tau_cycles()
        self.assertTrue(np.array_equal(bits(constants), bits(f32(self.fx["tau_cycles"]))),
                        "frequency constants differ from the Rust forward's")
        # Each constant is the 1..64-cycle ladder to within float32 rounding: measured against
        # the exact 2*pi*64^(pair/15), Rust's values are at most 1.93 ulps off (float32 powf, then
        # a rounded TAU and a rounded product), and numpy's float32 power at most 1.28 ulps.
        exponents = np.array([np.float32(p) / np.float32(15) for p in range(16)], dtype=np.float64)
        exact = 2.0 * np.pi * 64.0 ** exponents
        ulp = np.spacing(exact.astype(np.float32)).astype(np.float64)
        self.assertLessEqual(float(np.abs((constants - exact) / ulp).max()), 2.5,
                             "a constant is not the 1..64-cycle ladder")
        self.assertLessEqual(float(np.abs((ar.tau_cycles_formula() - exact) / ulp).max()), 2.5)
        cycles = constants / np.float32(2 * np.pi)
        self.assertAlmostEqual(float(cycles[0]), 1.0, places=5)
        self.assertAlmostEqual(float(cycles[-1]), 64.0, places=3)

    def test_positional_code_is_bit_identical(self) -> None:
        for t_steps in T_H:
            rust = f32(self.fx["pos"][str(t_steps)]).reshape(t_steps, 32)
            mine = ar.positional_code(t_steps)
            self.assertTrue(np.array_equal(bits(mine), bits(rust)), f"pos differs at T={t_steps}")

    def test_spike_streams_regenerate_from_portable_rng(self) -> None:
        # The fixture's spike trains and permutation come from PortableRng; regenerating them here
        # checks the port's `uniform` over 10^5 draws and its `shuffle`.
        fire_p = np.float32(self.fx["fire_p"])
        value = np.float32(1.0) / (np.float32(1.0) - np.float32(self.fx["dropout_p"]))
        for t_steps in T_H:
            rng = ar.PortableRng(self.fx["seed"] ^ self.fx["spike_key"] ^ (t_steps << 32))
            n = t_steps * 256
            binary = np.array([1.0 if rng.uniform(np.float32(0), np.float32(1)) < fire_p else 0.0
                               for _ in range(n)], dtype=np.float32)
            scaled = np.array([value if rng.uniform(np.float32(0), np.float32(1)) < fire_p else 0.0
                               for _ in range(n)], dtype=np.float32)
            order = list(range(t_steps))
            rng.shuffle(order)
            sample = self.fx["samples"][str(t_steps)]
            self.assertTrue(np.array_equal(bits(binary), bits(f32(sample["binary"]))))
            self.assertTrue(np.array_equal(bits(scaled), bits(f32(sample["dropout"]))))
            self.assertEqual(order, sample["order"])

    def test_pooled_and_logits_match_rust_within_the_bar(self) -> None:
        module = ar.AttentionReadout(256, 20, self.fx["seed"])
        worst = {"pooled": 0.0, "a": 0.0}
        covered = set()
        for t_steps in T_H:
            sample = self.fx["samples"][str(t_steps)]
            names = ["binary", "dropout", "silent"]
            # One batch holding every sample, so the batched path is what is tested.
            batch = np.stack([f32(sample[n]).reshape(t_steps, 256) for n in names], axis=1)
            s = torch.from_numpy(batch)
            order = torch.tensor(sample["order"], dtype=torch.long)
            for case in self.fx["cases"]:
                if case["t_steps"] != t_steps:
                    continue
                module.load_numpy(fixture_params(self.fx["params"][case["params"]], self.fx))
                presentation = None
                if case["presented"]:
                    identity = torch.arange(t_steps)
                    presentation = torch.stack([order if n == case["sample"] else identity for n in names])
                with torch.no_grad():
                    pooled = module.pooled(s, presentation)
                    logits = module(s, presentation)
                column = names.index(case["sample"])
                for key, mine in (("pooled", pooled[column]), ("a", logits[column])):
                    err = float(np.abs(mine.numpy() - f32(case[key])).max())
                    worst[key] = max(worst[key], err)
                    label = f"T={t_steps} {case['params']}/{case['sample']} presented={case['presented']} {key}"
                    self.assertLessEqual(err, PARITY_BAR, f"{label}: max-abs {err:.3e}")
                covered.add((t_steps, case["params"], case["presented"]))
        self.assertEqual(covered, {(t, p, pr) for t in T_H for p, pr in
                                   (("init", False), ("wo", False), ("wo", True))})
        print(f"\nparity max-abs: pooled {worst['pooled']:.3e}, a {worst['a']:.3e} (bar {PARITY_BAR:.0e})")

    def test_blocks_are_exercised_by_the_wo_weights(self) -> None:
        # Guard against a parity pass that never ran attention: with the fixture's non-zero W_o the
        # read-out must differ from the same weights with W_o zeroed.
        module = ar.AttentionReadout(256, 20, self.fx["seed"])
        params = fixture_params(self.fx["params"]["wo"], self.fx)
        s = torch.from_numpy(f32(self.fx["samples"]["124"]["binary"]).reshape(124, 1, 256))
        module.load_numpy(params)
        with torch.no_grad():
            with_blocks = module(s)
            module.load_numpy({"w_o": np.zeros_like(params["w_o"])})
            without = module(s)
        self.assertGreater(float((with_blocks - without).abs().max()), 1e-3)

    def test_presentation_must_be_a_permutation(self) -> None:
        module = ar.AttentionReadout(256, 20, 1)
        s = torch.zeros(5, 2, 256)
        bad = torch.tensor([[0, 1, 2, 3, 4], [0, 0, 2, 3, 4]])
        with self.assertRaises(ValueError):
            module(s, bad)
        with self.assertRaises(ValueError):
            module(torch.zeros(5, 2, 255))


if __name__ == "__main__":
    unittest.main()
