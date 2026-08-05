#!/usr/bin/env python3
"""Capture or verify hashes and mtimes for code-first transfer artifacts."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_PATHS = (
    "binn-data/src/lib.rs",
    "binn-data/src/temporal_order.rs",
    "binn-data/src/transfer_bundle.rs",
    "binn-learn/src/lib.rs",
    "binn-learn/src/input_rate_control.rs",
    "binn-learn/src/shared_bptt.rs",
    "binn-learn/src/shd_alif.rs",
    "binn-learn/src/shd_eprop_baseline.rs",
    "binn-lab/src/lib.rs",
    "binn-lab/src/transfer_harness.rs",
    "binn-lab/Cargo.toml",
    "binn-lab/experiments/shd_input_control.rs",
    "binn-lab/experiments/temporal_deep_campaign.rs",
    "binn-lab/experiments/temporal_eligibility_diagnostic.rs",
    "binn-lab/experiments/transfer_falsifier.rs",
    "scripts/transfer_numpy.py",
    "scripts/test_transfer_numpy.py",
    "scripts/run_code_transfer_campaign.sh",
    "scripts/capture_transfer_manifest.py",
    "results/CODE_FIRST_TRANSFER_STATUS.md",
    "results/PREREG_2026-07-26_CODE_TRANSFER_PROTOCOLS.md",
    "results/PREREG_2026-07-26_TEMPORAL_ELIGIBILITY_V147.md",
    "results/shd_0c1_v143/capped-alif-ff-fixed-predictions.tsv",
    "results/shd_0c1_v143/capped-alif-ff-fixed.md",
    "results/shd_0c1_v143/full-superspike-predictions.tsv",
    "results/shd_0c1_v143/full-superspike.md",
    "results/shd_0c1_v143/SHD_0C1_DECISION.md",
    "results/temporal_calibration_v144.md",
    "results/temporal_eligibility_diagnostic_v147.md",
    "results/TEMPORAL_ELIGIBILITY_V147_DECISION.md",
    "results/shd_0c1_v143_quick/capped-alif-ff-fixed-predictions.tsv",
    "results/shd_0c1_v143_quick/capped-alif-ff-fixed.md",
    "results/shd_0c1_v143_quick/full-superspike-predictions.tsv",
    "results/shd_0c1_v143_quick/full-superspike.md",
    "results/temporal_calibration_v144_quick.md",
    "results/transfer_falsifier_v146_quick.md",
    "results/transfer_v146_quick_bundles/micro-numpy-live-replay.json",
    "results/transfer_v146_quick_bundles/micro-numpy-live.json",
    "results/transfer_v146_quick_bundles/micro-numpy-matched-replay.json",
    "results/transfer_v146_quick_bundles/micro-numpy-matched.json",
    "results/transfer_v146_quick_bundles/micro-rust-live.json",
    "results/transfer_v146_quick_bundles/micro-rust-matched.json",
    "results/transfer_v146_quick_bundles/seed-00.binntrf1",
)


def record(relative: str) -> dict:
    path = ROOT / relative
    data = path.read_bytes()
    stat = path.stat()
    return {
        "path": relative,
        "bytes": len(data),
        "mtime_ns": stat.st_mtime_ns,
        "sha256": hashlib.sha256(data).hexdigest(),
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--manifest",
        type=Path,
        default=ROOT / "results/transfer_implementation_manifest.json",
    )
    parser.add_argument("--verify", action="store_true")
    args = parser.parse_args()
    current = {"format": "BINN_TRANSFER_MANIFEST_V1", "files": [record(p) for p in DEFAULT_PATHS]}
    if args.verify:
        expected = json.loads(args.manifest.read_text(encoding="utf-8"))
        if expected != current:
            expected_by_path = {item["path"]: item for item in expected["files"]}
            changed = [
                item["path"]
                for item in current["files"]
                if expected_by_path.get(item["path"]) != item
            ]
            raise SystemExit(f"manifest mismatch: {', '.join(changed)}")
        print(f"manifest verified: {len(current['files'])} files")
        return 0
    args.manifest.parent.mkdir(parents=True, exist_ok=True)
    args.manifest.write_text(
        json.dumps(current, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    print(f"manifest captured: {len(current['files'])} files -> {args.manifest}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
