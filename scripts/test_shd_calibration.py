#!/usr/bin/env python3
"""Mechanical tests for the SHD calibration harness."""

from __future__ import annotations

import tempfile
from pathlib import Path
import unittest

import numpy as np

from scripts.shd_calibration.data import (
    Contract,
    frame_events,
    fixture_samples,
    read_event_cache,
    write_fixture_cache,
)
from scripts.shd_calibration.reference import (
    parse_historical_validation_curve,
    verify_clean_source,
)
from scripts.shd_calibration.runner import ROOT, all_cells


class DataContractTests(unittest.TestCase):
    def test_fixture_cache_roundtrip_and_all_labels(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "fixture.events"
            write_fixture_cache(path)
            samples = read_event_cache(path)
        self.assertEqual([sample.label for sample in samples], list(range(20)))
        self.assertTrue(all(len(sample.times) == 6 for sample in samples))

    def test_published_count_preservation_and_alignment(self) -> None:
        framed = frame_events(
            fixture_samples()[0], Contract("published", 10), "channels-700"
        )
        self.assertEqual(framed.retained_events, 6)
        self.assertEqual(framed.clipped_events, 0)
        self.assertEqual(framed.frames[0], [(0, np.float32(2.0))])

    def test_fixed_window_clips_instead_of_saturating(self) -> None:
        framed = frame_events(fixture_samples()[0], Contract("fixed", 100), "channels-700")
        self.assertEqual(framed.valid_steps, 100)
        self.assertEqual(framed.retained_events, 5)
        self.assertEqual(framed.clipped_events, 1)

    def test_frequency_sum_is_adjacent_nonoverlapping(self) -> None:
        framed = frame_events(
            fixture_samples()[0], Contract("published", 10), "adjacent-sum-5"
        )
        self.assertEqual(framed.n_inputs, 140)
        self.assertEqual(framed.frames[1], [(0, np.float32(1.0))])
        self.assertEqual(framed.frames[2], [(1, np.float32(1.0))])


class ProtocolTests(unittest.TestCase):
    def test_matrix_has_exactly_432_unique_cells(self) -> None:
        cells = all_cells()
        self.assertEqual(len(cells), 432)
        self.assertEqual(len({cell.id for cell in cells}), 432)

    def test_clean_reference_has_one_post_training_test_read(self) -> None:
        verify_clean_source(ROOT / "scripts/shd_calibration/reference_clean_main.py")

    def test_historical_parser_ignores_best_accuracy(self) -> None:
        log = "\n".join(
            (
                "=====> Epoch 0 : ",
                "Loss Train = 1.000  |  Acc Train = 10.00%",
                "Loss Valid = 0.900  |  Acc Valid = 20.00%  |  Best Acc Valid = 99.00%",
                "=====> Epoch 1 : ",
                "Loss Train = 0.500  |  Acc Train = 50.00%",
                "Loss Valid = 0.400  |  Acc Valid = 60.00%  |  Best Acc Valid = 99.50%",
            )
        )
        self.assertEqual(
            parse_historical_validation_curve(log, expected_epochs=2),
            [0.20, 0.60],
        )

    def test_historical_parser_requires_contiguous_epoch_coverage(self) -> None:
        log = "\n".join(
            (
                "=====> Epoch 1 : ",
                "Loss Train = 0.500  |  Acc Train = 50.00%",
                "Loss Valid = 0.400  |  Acc Valid = 60.00%  |  Best Acc Valid = 99.50%",
            )
        )
        with self.assertRaisesRegex(RuntimeError, "epoch coverage mismatch"):
            parse_historical_validation_curve(log, expected_epochs=2)


if __name__ == "__main__":
    unittest.main()
