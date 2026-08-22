"""Tests for the bit-identity discharge of the provenance freeze.

Registered in `results/AMENDMENT_2026-08-03_PROVENANCE_DISCHARGE_BY_BIT_IDENTITY.md`.

The tests that matter most here are the negative ones. A discharge path is only
as good as what it still refuses, so most of this file is about evidence that
must NOT be accepted: wrong binary, too few cells, a single geometry, a single
width, a failing report, and — unconditionally — any change to the data.

Run: .venv-shd/bin/python scripts/test_provenance_discharge.py
"""

from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from shd_calibration import runner  # noqa: E402


def cell_id(geometry: str, width: int, seed: int = 5170001) -> str:
    return f"rust__published-2ms__{geometry}__h{width}__e20__s{seed}"


def report(
    binary_sha: str,
    cells: list[str],
    status: str = "PASS",
    failures: int = 0,
) -> dict[str, object]:
    return {
        "binary": "/tmp/shd-instrument",
        "binary_sha256": binary_sha,
        "cells": len(cells),
        "failures": failures,
        "status": status,
        "results": [{"cell": cell, "status": "BIT_IDENTICAL"} for cell in cells],
    }


def broad_cells() -> list[str]:
    """8 cells spanning two geometries and two widths."""
    return [
        cell_id(geometry, width, seed)
        for geometry in ("adjacent-sum-5", "channels-700")
        for width in (128, 512)
        for seed in (5170001, 5170002)
    ]


class DischargeEvidenceTest(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        (self.root / "gate-f-rust").mkdir(parents=True)
        self._saved_root = runner.RESULT_ROOT
        runner.RESULT_ROOT = self.root

    def tearDown(self) -> None:
        runner.RESULT_ROOT = self._saved_root
        self.tmp.cleanup()

    def write(self, *records: dict[str, object]) -> None:
        path = self.root / "gate-f-rust" / "runs.jsonl"
        with path.open("w") as handle:
            for record in records:
                handle.write(json.dumps(record) + "\n")

    def test_accepts_broad_passing_report_for_the_current_binary(self) -> None:
        self.write(report("aa" * 32, broad_cells()))
        found = runner.gate_f_discharge("aa" * 32)
        self.assertIsNotNone(found)
        self.assertEqual(found["cells"], 8)

    def test_rejects_evidence_for_a_different_binary(self) -> None:
        """The whole point: evidence must attach to the binary being run."""
        self.write(report("aa" * 32, broad_cells()))
        self.assertIsNone(runner.gate_f_discharge("bb" * 32))

    def test_rejects_too_few_cells(self) -> None:
        self.write(report("aa" * 32, broad_cells()[:4]))
        self.assertIsNone(runner.gate_f_discharge("aa" * 32))

    def test_rejects_single_geometry_however_many_cells(self) -> None:
        """Breadth, not count. This is the parity-fixture lesson."""
        cells = [
            cell_id("adjacent-sum-5", width, seed)
            for width in (128, 512)
            for seed in range(5170001, 5170007)
        ]
        self.assertGreaterEqual(len(cells), runner.PROVENANCE_MIN_GATE_F_CELLS)
        self.write(report("aa" * 32, cells))
        self.assertIsNone(runner.gate_f_discharge("aa" * 32))

    def test_rejects_single_width_however_many_cells(self) -> None:
        cells = [
            cell_id(geometry, 128, seed)
            for geometry in ("adjacent-sum-5", "channels-700")
            for seed in range(5170001, 5170007)
        ]
        self.assertGreaterEqual(len(cells), runner.PROVENANCE_MIN_GATE_F_CELLS)
        self.write(report("aa" * 32, cells))
        self.assertIsNone(runner.gate_f_discharge("aa" * 32))

    def test_rejects_failing_report(self) -> None:
        self.write(report("aa" * 32, broad_cells(), status="FAIL", failures=1))
        self.assertIsNone(runner.gate_f_discharge("aa" * 32))

    def test_rejects_pass_status_with_nonzero_failures(self) -> None:
        """Defence against a malformed or hand-edited report."""
        self.write(report("aa" * 32, broad_cells(), status="PASS", failures=2))
        self.assertIsNone(runner.gate_f_discharge("aa" * 32))

    def test_missing_history_is_not_an_error(self) -> None:
        self.assertIsNone(runner.gate_f_discharge("aa" * 32))

    def test_malformed_lines_are_skipped_not_fatal(self) -> None:
        path = self.root / "gate-f-rust" / "runs.jsonl"
        path.write_text("not json\n\n" + json.dumps(report("aa" * 32, broad_cells())) + "\n")
        self.assertIsNotNone(runner.gate_f_discharge("aa" * 32))

    def test_picks_the_broadest_qualifying_report(self) -> None:
        wide = broad_cells() + [cell_id("channels-700", 256, 5170003)]
        self.write(report("aa" * 32, broad_cells()), report("aa" * 32, wide))
        found = runner.gate_f_discharge("aa" * 32)
        self.assertEqual(found["cells"], len(wide))


class DefaultOffTest(unittest.TestCase):
    def test_discharge_is_enabled_by_human_authorization(self) -> None:
        """Enabled 2026-08-05 by explicit human authorization.

        This previously asserted the flag was *false* — the guarantee being that
        an agent could not silently discharge a provenance freeze it had itself
        proposed. That guarantee did its job: the flag shipped off, the decision
        was escalated, and a human made it.

        The assertion is inverted rather than deleted so the flag's state stays
        pinned in both directions. If it flips back to False without a
        corresponding amendment, this fails and asks why.
        """
        self.assertTrue(runner.PROVENANCE_DISCHARGE_ENABLED)

    def test_enabling_does_not_weaken_the_evidence_bar(self) -> None:
        """The flag authorises the mechanism; it does not lower its bar.

        This is the property that makes enabling safe, so it is asserted
        directly rather than trusted: with the flag on, evidence that was
        insufficient before is still insufficient.
        """
        self.assertTrue(runner.PROVENANCE_DISCHARGE_ENABLED)
        self.assertGreaterEqual(runner.PROVENANCE_MIN_GATE_F_CELLS, 8)

    def test_data_files_are_never_dischargeable(self) -> None:
        """Read the guard out of the source rather than trusting the comment.

        A kernel proven to reproduce recorded cells says nothing about whether
        its inputs changed, so these must stay unconditionally fatal.
        """
        source = Path(runner.__file__).read_text()
        marker = "undischargeable = {"
        block = source[source.index(marker) : source.index("}", source.index(marker))]
        for name in ("train_h5", "test_h5", "train_events", "test_events"):
            self.assertIn(name, block)
        for name in ("rust_binary", "cargo_lock"):
            self.assertNotIn(name, block)


if __name__ == "__main__":
    unittest.main(verbosity=2)
