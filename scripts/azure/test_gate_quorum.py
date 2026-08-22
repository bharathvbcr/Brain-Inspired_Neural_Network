#!/usr/bin/env python3
"""Adversarial tests for the Azure-local Gate F quorum."""

from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path

from scripts.azure.gate_quorum import (
    QuorumMismatch,
    QuorumPending,
    build_attestation,
    validate_quorum,
)


FIELDS = {
    "accuracy": 0.61,
    "mean_loss": 1.72,
    "mean_gradient_norm": 0.27,
    "mean_update_rms": 0.001,
    "mean_firing_rate": 0.18,
    "majority_prediction": 0.11,
    "classes_predicted": 20,
    "silent_fraction": 0.0,
    "saturated_fraction": 0.0,
    "non_finite_events": 0,
    "n_train": 8156,
    "n_test": 2264,
}


class GateQuorumTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.output = self.root / "gate-output"
        self.output.mkdir()
        self.cell = "rust__fixed-t100__adjacent-sum-5__h128__e20__s5170001"
        (self.output / f"{self.cell}.json").write_text(json.dumps(FIELDS))
        self.gate_report = self.output / "report.json"
        self.gate_report.write_text(json.dumps({
            "binary_sha256": "binary-a",
            "cells": 1,
            "failures": 1,
            "status": "FAIL",
            "results": [{"cell": self.cell, "status": "REGRESSION"}],
        }))

    def tearDown(self) -> None:
        self.temp.cleanup()

    def attestation(self, node: int, **overrides: object) -> dict[str, object]:
        report = build_attestation(
            node=node,
            source_sha256="source-a",
            binary_sha256="binary-a",
            gate_report=self.gate_report,
            gate_output_dir=self.output,
            utc="2026-08-21T06:30:56Z",
        )
        report.update(overrides)
        return report

    def write_reports(self, reports: list[dict[str, object]]) -> Path:
        directory = self.root / "reports"
        directory.mkdir(exist_ok=True)
        for report in reports:
            node = int(report["node"])
            (directory / f"node-{node}.json").write_text(json.dumps(report))
        return directory

    def test_four_identical_observations_pass_despite_cross_platform_fail(self) -> None:
        reports = [self.attestation(node) for node in range(4)]
        result = validate_quorum(
            self.write_reports(reports), node_count=4,
            source_sha256="source-a", binary_sha256="binary-a",
            expected_cross_platform_status="FAIL",
            expected_gate_cells=1,
        )
        self.assertEqual(result["status"], "PASS")
        self.assertEqual(result["nodes"], [0, 1, 2, 3])
        self.assertEqual(result["cross_platform_gate_f"], "FAIL")

    def test_one_changed_scientific_field_fails_closed(self) -> None:
        reports = [self.attestation(node) for node in range(4)]
        reports[3]["observed_results"][0]["fields"]["accuracy"] = 0.62
        with self.assertRaises(QuorumMismatch):
            validate_quorum(
                self.write_reports(reports), node_count=4,
                source_sha256="source-a", binary_sha256="binary-a",
                expected_cross_platform_status="FAIL",
                expected_gate_cells=1,
            )

    def test_tampered_observation_digest_fails_closed(self) -> None:
        reports = [self.attestation(node) for node in range(4)]
        reports[2]["observation_sha256"] = "forged"
        with self.assertRaises(QuorumMismatch):
            validate_quorum(
                self.write_reports(reports), node_count=4,
                source_sha256="source-a", binary_sha256="binary-a",
                expected_cross_platform_status="FAIL",
                expected_gate_cells=1,
            )

    def test_old_source_report_is_pending_not_accepted(self) -> None:
        reports = [self.attestation(node) for node in range(4)]
        reports[1]["source_sha256"] = "previous-launch"
        with self.assertRaises(QuorumPending):
            validate_quorum(
                self.write_reports(reports), node_count=4,
                source_sha256="source-a", binary_sha256="binary-a",
                expected_cross_platform_status="FAIL",
                expected_gate_cells=1,
            )

    def test_missing_node_is_pending_not_accepted(self) -> None:
        reports = [self.attestation(node) for node in range(3)]
        with self.assertRaises(QuorumPending):
            validate_quorum(
                self.write_reports(reports), node_count=4,
                source_sha256="source-a", binary_sha256="binary-a",
                expected_cross_platform_status="FAIL",
                expected_gate_cells=1,
            )

    def test_binary_or_cross_platform_status_disagreement_fails_closed(self) -> None:
        for field, value in (("binary_sha256", "binary-b"),
                             ("cross_platform_gate_f", "PASS")):
            with self.subTest(field=field):
                reports = [self.attestation(node) for node in range(4)]
                reports[1][field] = value
                with self.assertRaises(QuorumMismatch):
                    validate_quorum(
                        self.write_reports(reports), node_count=4,
                        source_sha256="source-a", binary_sha256="binary-a",
                        expected_cross_platform_status="FAIL",
                        expected_gate_cells=1,
                    )

    def test_malformed_current_observation_fails_closed(self) -> None:
        reports = [self.attestation(node) for node in range(4)]
        reports[0]["observed_results"] = [{"cell": self.cell, "fields": {}, "traces": {}}]
        with self.assertRaises(QuorumMismatch):
            validate_quorum(
                self.write_reports(reports), node_count=4,
                source_sha256="source-a", binary_sha256="binary-a",
                expected_cross_platform_status="FAIL", expected_gate_cells=1,
            )


if __name__ == "__main__":
    unittest.main()
