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
        # The *archived* set. `reference-manifests/` held these until the
        # 2026-08-23 re-run replaced its contents with six freshly-produced
        # artifacts that legitimately declare `fingerprint_scope: reference`;
        # this test kept reading that path and started asserting the opposite of
        # what it means. The superseded artifacts are the ones that must stay
        # rejected, and they live here.
        manifests = ROOT / (
            "results/shd_instrument_v4/references-superseded-2026-07-27"
            "/reference-manifests"
        )
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


class ReferenceCheckoutLockTests(unittest.TestCase):
    """The lock that stops concurrent reference cells racing on one git clone.

    `ensure_checkout` and `prepare_seed_worktree` both operate on the single
    shared clone under `reference-cache`. Without serialisation, concurrent cells
    contend on `index.lock` and the losers die before training starts -- which is
    exactly what happened on 2026-08-23, killing two of three historical cells in
    the same second. See `DEFECT_2026-08-23_REFERENCE_SETUP_HAS_NO_LOCK.md`.

    A lock that does not actually exclude is worse than no lock, because it
    retires the vigilance that would otherwise stagger the launches. So this
    tests exclusion by observation, not by inspecting the code.
    """

    def test_two_holders_never_overlap(self) -> None:
        """Run two threads through the lock and check the critical sections are
        disjoint in time. A no-op lock interleaves them and fails this."""
        import threading
        import time

        from scripts.shd_calibration.runner import reference_checkout_lock

        intervals: list[tuple[float, float]] = []
        errors: list[BaseException] = []
        barrier = threading.Barrier(2)

        def hold() -> None:
            try:
                barrier.wait(timeout=10)
                with reference_checkout_lock():
                    entered = time.monotonic()
                    # Long enough that an unsynchronised pair reliably overlaps.
                    time.sleep(0.25)
                    intervals.append((entered, time.monotonic()))
            except BaseException as exc:  # noqa: BLE001 - surfaced below
                errors.append(exc)

        threads = [threading.Thread(target=hold) for _ in range(2)]
        for thread in threads:
            thread.start()
        for thread in threads:
            thread.join(timeout=30)

        self.assertEqual(errors, [], f"a holder raised: {errors}")
        self.assertEqual(len(intervals), 2, "both holders must have run")
        first, second = sorted(intervals)
        self.assertLessEqual(
            first[1],
            second[0] + 1e-6,
            "the two critical sections overlapped, so the lock does not exclude",
        )

    # There was a `test_the_lock_is_released_when_the_body_raises` here. It
    # could not fail: CPython refcounting closes the file handle as soon as it
    # leaves scope, and closing an fd releases its flock, so the lock is
    # released whether or not the `finally` block runs. The test passed against
    # a deliberately leaking implementation -- verified by mutation -- which
    # makes it exactly the kind of check this workspace keeps finding and
    # deleting. It is gone rather than left green.
    #
    # The `finally` in `reference_checkout_lock` stays. It is defensive, not
    # load-bearing, and it costs nothing.
