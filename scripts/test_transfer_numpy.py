#!/usr/bin/env python3

import unittest

import numpy as np

from transfer_numpy import (
    KWTA_K,
    N_CLASSES,
    N_IN,
    TIMESTEPS,
    Bundle,
    Example,
    Pole,
    TransferModel,
    select_spikes,
)


def fixture(hidden: int = 8) -> Bundle:
    frames = np.zeros((TIMESTEPS, N_IN), dtype=np.float32)
    frames[0, 0] = 1.0
    frames[8, 1] = 1.0
    example = Example(frames, 0)
    return Bundle(
        seed=7,
        hidden=hidden,
        train=[example],
        test=[example],
        delays=np.tile(np.arange(1, 5, dtype=np.uint32), hidden * N_IN // 4),
        input_weights=np.full(hidden * N_IN, 0.2, dtype=np.float32),
        feedback=np.linspace(-0.2, 0.2, hidden * N_CLASSES, dtype=np.float32),
        readout=np.linspace(-0.1, 0.1, N_CLASSES * hidden, dtype=np.float32),
        readout_bias=np.zeros(N_CLASSES, dtype=np.float32),
    )


class TransferNumpyTests(unittest.TestCase):
    def test_kwta_ties_use_lowest_indices(self) -> None:
        hidden = max(12, KWTA_K)
        spikes = select_spikes(
            np.full(hidden, 2.0, dtype=np.float32),
            np.ones(hidden, dtype=np.float32),
            "kwta",
        )
        np.testing.assert_array_equal(spikes[:KWTA_K], 1.0)
        np.testing.assert_array_equal(spikes[KWTA_K:], 0.0)

    def test_event_order_and_delayed_delivery(self) -> None:
        bundle = fixture()
        trace = TransferModel(bundle).micro_step(bundle.train[0], Pole.live(), 0.001)
        self.assertTrue(trace["event_ticks"])
        self.assertEqual(trace["event_ticks"], sorted(trace["event_ticks"]))
        self.assertTrue(all(0 <= unit < bundle.hidden for unit in trace["recipients"]))

    def test_adaptive_threshold_and_hard_reset_path(self) -> None:
        bundle = fixture()
        bundle.input_weights.fill(2.0)
        trace = TransferModel(bundle).micro_step(bundle.train[0], Pole.live(), 0.001)
        self.assertTrue(any(theta > 1.0 for theta in trace["final_thresholds"]))

    def test_trace_decay_reduces_norm(self) -> None:
        bundle = fixture()
        held = TransferModel(bundle).micro_step(
            bundle.train[0], Pole.matched(), 0.001
        )
        decay_pole = Pole("all", "sync", "fixed_soft", "decay")
        decayed = TransferModel(bundle).micro_step(
            bundle.train[0], decay_pole, 0.001
        )
        self.assertLess(
            np.abs(decayed["eligibility"]).sum(),
            np.abs(held["eligibility"]).sum(),
        )

    def test_update_signs_and_replay(self) -> None:
        bundle = fixture()
        a = TransferModel(bundle).micro_step(bundle.train[0], Pole.matched(), 0.001)
        b = TransferModel(bundle).micro_step(bundle.train[0], Pole.matched(), 0.001)
        self.assertEqual(a, b)
        self.assertTrue(any(delta != 0.0 for delta in a["weight_delta"]))
        self.assertTrue(
            any(delta > 0.0 for delta in a["weight_delta"])
            or any(delta < 0.0 for delta in a["weight_delta"])
        )

    def test_evaluation_has_no_updates(self) -> None:
        bundle = fixture()
        result = TransferModel(bundle).evaluate(bundle.test, Pole.matched())
        self.assertTrue(result["no_test_update"])


if __name__ == "__main__":
    unittest.main()
