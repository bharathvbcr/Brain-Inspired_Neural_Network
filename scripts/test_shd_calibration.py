#!/usr/bin/env python3
"""Mechanical tests for the SHD calibration harness."""

from __future__ import annotations

import json
import sys
import tempfile
from pathlib import Path
import unittest

# `from scripts...` is an absolute package import, so the REPOSITORY ROOT has to
# be importable - not `scripts/`, which is what running this file directly puts
# on the path. Without this the file raises `No module named 'scripts'` and can
# only be run as `python3 -m scripts.test_shd_calibration` from the root, which
# is not how any gate invokes it. It was unreachable for that reason until
# 2026-08-23, and unreachable because no gate ran it.
sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

import numpy as np  # noqa: E402

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
from scripts.shd_calibration.runner import (
    ROOT,
    all_cells,
    reference_fingerprint_matches,
    reference_source_fingerprint,
    relevant_source_fingerprint,
)


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


class ReferenceFingerprintScopeTests(unittest.TestCase):
    """A fingerprint is only meaningful with the path set it was computed over.

    `AMENDMENT_2026-08-03_REFERENCE_FINGERPRINT_SCOPE.md` was withdrawn for
    assuming a narrow fingerprint could be compared against a value frozen under
    the broad scope. They are outputs of different functions over different
    inputs. The forward-only narrowing registered in
    `AMENDMENT_2026-08-22_REFERENCE_FINGERPRINT_SCOPE_FORWARD.md` must not repeat
    that, and these tests are what stops it.
    """

    def test_the_two_scopes_are_actually_different(self) -> None:
        """If they ever coincided, every test below would pass vacuously."""
        self.assertNotEqual(
            relevant_source_fingerprint(),
            reference_source_fingerprint(),
            "the broad and narrow scopes produced the same fingerprint; the "
            "path sets must have converged and the distinction is now empty",
        )

    def test_an_artifact_declaring_no_scope_is_checked_against_the_broad_one(
        self,
    ) -> None:
        """Every reference frozen before 2026-08-22 declares nothing. The
        narrowing must not validate a single one of them."""
        self.assertTrue(
            reference_fingerprint_matches(
                {"source_fingerprint": relevant_source_fingerprint()}
            )
        )
        self.assertFalse(
            reference_fingerprint_matches(
                {"source_fingerprint": reference_source_fingerprint()}
            ),
            "an undeclared artifact was accepted on the narrow fingerprint, "
            "which is exactly the error that withdrew the 08-03 amendment",
        )

    def test_the_six_archived_references_are_still_rejected(self) -> None:
        """The change is non-retroactive, asserted against the real artifacts
        rather than argued. All six fail on fingerprint and only on fingerprint;
        this pins that the narrowing did not quietly let them through."""
        manifests = ROOT / "results/shd_instrument_v4/reference-manifests"
        if not manifests.is_dir():
            self.skipTest("reference manifests are not present in this checkout")
        seen = 0
        for mode in ("clean", "historical"):
            for seed in (5170001, 5170002, 5170003):
                path = manifests / f"{mode}-seed-{seed}.json"
                if not path.is_file():
                    continue
                seen += 1
                manifest = json.loads(path.read_text())
                self.assertNotIn(
                    "fingerprint_scope",
                    manifest,
                    f"{path.name} declares a scope; it predates the amendment",
                )
                self.assertFalse(
                    reference_fingerprint_matches(manifest),
                    f"{path.name} is now accepted. The narrowing was supposed to "
                    "be forward-only, and gate state must not move by accident",
                )
        self.assertEqual(seen, 6, "expected all six archived references")

    def test_a_declared_reference_artifact_is_checked_against_the_narrow_scope(
        self,
    ) -> None:
        self.assertTrue(
            reference_fingerprint_matches(
                {
                    "fingerprint_scope": "reference",
                    "source_fingerprint": reference_source_fingerprint(),
                }
            )
        )
        self.assertFalse(
            reference_fingerprint_matches(
                {
                    "fingerprint_scope": "reference",
                    "source_fingerprint": relevant_source_fingerprint(),
                }
            ),
            "a declared artifact was accepted on the broad fingerprint",
        )

    def test_an_unrecognised_scope_is_refused_rather_than_defaulted(self) -> None:
        """Fail closed. A scope this code does not understand is a reason to
        reject an artifact, never a reason to fall back to whichever check it
        happens to pass."""
        for scope in ("", "narrow", "instrument", None, 7):
            self.assertFalse(
                reference_fingerprint_matches(
                    {
                        "fingerprint_scope": scope,
                        "source_fingerprint": reference_source_fingerprint(),
                    }
                ),
                f"scope {scope!r} was accepted",
            )
            self.assertFalse(
                reference_fingerprint_matches(
                    {
                        "fingerprint_scope": scope,
                        "source_fingerprint": relevant_source_fingerprint(),
                    }
                ),
                f"scope {scope!r} was accepted",
            )
