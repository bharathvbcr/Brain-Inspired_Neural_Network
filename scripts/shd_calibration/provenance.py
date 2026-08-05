"""Immutable manifests and fingerprint-safe atomic cell states."""

from __future__ import annotations

from dataclasses import dataclass, asdict
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess
import sys
from typing import Iterable


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def source_fingerprint(root: Path, paths: Iterable[Path]) -> str:
    digest = hashlib.sha256()
    for path in sorted((root / path for path in paths), key=lambda item: str(item)):
        if not path.is_file():
            continue
        relative = path.relative_to(root).as_posix().encode()
        digest.update(len(relative).to_bytes(4, "little"))
        digest.update(relative)
        digest.update(bytes.fromhex(sha256_file(path)))
    return digest.hexdigest()


@dataclass(frozen=True)
class Exposure:
    train_reads: int
    validation_reads: int
    test_reads_during_training: int
    final_test_reads: int
    checkpoint_selected_on: str


def environment_record(root: Path) -> dict[str, object]:
    def command(*args: str) -> str:
        try:
            return subprocess.check_output(args, cwd=root, text=True).strip()
        except (OSError, subprocess.CalledProcessError):
            return "unavailable"

    return {
        "captured_at": datetime.now(timezone.utc).isoformat(),
        "platform": platform.platform(),
        "machine": platform.machine(),
        "python": sys.version,
        "rustc": command("rustc", "--version"),
        "cargo": command("cargo", "--version"),
        "git_head": command("git", "rev-parse", "HEAD"),
        "git_status_sha256": hashlib.sha256(
            command("git", "status", "--porcelain=v1").encode()
        ).hexdigest(),
    }


def immutable_manifest(path: Path, payload: dict[str, object]) -> None:
    encoded = (json.dumps(payload, indent=2, sort_keys=True) + "\n").encode()
    if path.exists():
        if path.read_bytes() != encoded:
            raise RuntimeError(f"immutable manifest mismatch: {path}")
        return
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_suffix(f".tmp-{os.getpid()}")
    with temporary.open("wb") as handle:
        handle.write(encoded)
        handle.flush()
        os.fsync(handle.fileno())
    os.replace(temporary, path)


def write_cell_state(path: Path, payload: dict[str, object]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    encoded = (json.dumps(payload, indent=2, sort_keys=True) + "\n").encode()
    temporary = path.with_suffix(f".tmp-{os.getpid()}")
    with temporary.open("wb") as handle:
        handle.write(encoded)
        handle.flush()
        os.fsync(handle.fileno())
    os.replace(temporary, path)


def cache_compatible(
    state_path: Path, expected_fingerprint: str, expected_manifest_sha256: str
) -> bool:
    if not state_path.is_file():
        return False
    try:
        state = json.loads(state_path.read_text())
    except (OSError, json.JSONDecodeError):
        return False
    return (
        state.get("mechanical_status") == "COMPLETE"
        and state.get("source_fingerprint") == expected_fingerprint
        and state.get("manifest_sha256") == expected_manifest_sha256
        and Path(state.get("result_path", "")).is_file()
    )
