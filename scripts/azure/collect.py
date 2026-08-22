#!/usr/bin/env python3
"""Download every Azure campaign artifact and run the frozen analysis."""

from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent.parent
if str(REPO_ROOT) not in sys.path:
    sys.path.insert(0, str(REPO_ROOT))

from scripts.azure.launch import CONTAINER, ROOT, STORAGE


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", default=str(ROOT / "results/azure-d32l4-scope-v1"))
    args = parser.parse_args()
    destination = Path(args.out).resolve()
    destination.mkdir(parents=True, exist_ok=True)
    for prefix in ("input/cells.json", "input/manifest.json", "results/*", "gates/*",
                   "failures/*", "summaries/*", "hosts/*", "hostlogs/*"):
        subprocess.run([
            "az", "storage", "blob", "download-batch", "--account-name", STORAGE,
            "--source", CONTAINER, "--destination", str(destination),
            "--pattern", prefix, "--auth-mode", "key", "--no-progress",
        ], check=True)
    report = destination / "VERDICT.md"
    return subprocess.run([
        sys.executable, str(ROOT / "scripts/azure/analyse.py"),
        "--plan", str(destination / "input/cells.json"),
        "--results", str(destination / "results"),
        "--gates", str(destination / "gates"),
        "--failures", str(destination / "failures"),
        "--out", str(report),
    ], cwd=ROOT, check=False).returncode


if __name__ == "__main__":
    raise SystemExit(main())
