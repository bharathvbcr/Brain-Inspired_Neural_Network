"""Pinned historical and clean SHD reference orchestration."""

from __future__ import annotations

import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys

from . import REFERENCE_COMMIT
from .provenance import sha256_file, write_cell_state


REFERENCE_URL = "https://github.com/Thvnvtos/SNN-delays.git"
SPIKINGJELLY_URL = "https://github.com/fangwei123456/spikingjelly.git"
# Last SHD loader revision from the SNN-delays development period. Newer
# SpikingJelly refactors removed the attributes consumed by datasets.py.
SPIKINGJELLY_COMMIT = "6dca147afe684b5e78d9c9d430e8761f921437b2"
HISTORICAL_EPOCH_RE = re.compile(
    r"^=====> Epoch ([0-9]+) :\s*$"
    r".*?"
    r"^Loss Valid = [^\n]*?\|\s+Acc Valid = ([0-9]+(?:\.[0-9]+)?)%"
    r"\s+\|\s+Best Acc Valid = [0-9]+(?:\.[0-9]+)?%\s*$",
    re.MULTILINE | re.DOTALL,
)


def parse_historical_validation_curve(
    log_text: str, expected_epochs: int = 150
) -> list[float]:
    """Parse exactly one current validation accuracy for every historical epoch."""
    records = [
        (int(epoch), float(accuracy) / 100.0)
        for epoch, accuracy in HISTORICAL_EPOCH_RE.findall(log_text.replace("\r", "\n"))
    ]
    epochs = [epoch for epoch, _ in records]
    expected = list(range(expected_epochs))
    if epochs != expected:
        raise RuntimeError(
            "historical log epoch coverage mismatch: "
            f"observed={epochs[:3]}...{epochs[-3:] if epochs else []} "
            f"count={len(epochs)} expected=0..{expected_epochs - 1}"
        )
    return [accuracy for _, accuracy in records]


def historical_payload_from_log(log_text: str, seed: int) -> dict[str, object]:
    values = parse_historical_validation_curve(log_text)
    return {
        "schema": "shd-reference-v2",
        "mode": "historical",
        "seed": seed,
        "accuracy": max(values),
        "final_accuracy": values[-1],
        "epochs": 150,
        "test_reads_during_training": 150,
        "final_test_reads": 0,
        "checkpoint_selected_on": "official-test-best-accuracy",
        "exposure_status": "EXPOSURE_TAINTED_DESCRIPTIVE",
        "validation_curve": values,
    }


def ensure_checkout(cache_root: Path) -> Path:
    checkout = cache_root / "SNN-delays"
    cache_root.mkdir(parents=True, exist_ok=True)
    if not (checkout / ".git").is_dir():
        subprocess.run(
            ["git", "clone", "--no-tags", REFERENCE_URL, str(checkout)], check=True
        )
    subprocess.run(["git", "fetch", "--quiet", "origin", REFERENCE_COMMIT], cwd=checkout, check=True)
    subprocess.run(["git", "checkout", "--quiet", "--detach", REFERENCE_COMMIT], cwd=checkout, check=True)
    head = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=checkout, text=True).strip()
    if head != REFERENCE_COMMIT:
        raise RuntimeError(f"reference checkout mismatch: {head}")
    return checkout


def prepare_seed_worktree(
    checkout: Path,
    work_root: Path,
    seed: int,
    dataset_path: Path,
    mode: str,
    clean_template: Path,
) -> Path:
    worktree = work_root / f"{mode}-seed-{seed}"
    if worktree.exists():
        subprocess.run(
            ["git", "worktree", "remove", "--force", str(worktree)],
            cwd=checkout,
            check=False,
        )
        if worktree.exists():
            shutil.rmtree(worktree)
    subprocess.run(["git", "worktree", "prune"], cwd=checkout, check=True)
    subprocess.run(
        ["git", "worktree", "add", "--quiet", "--detach", str(worktree), REFERENCE_COMMIT],
        cwd=checkout,
        check=True,
    )
    source = (worktree / "best_config_SHD.py").read_text()
    source = source.replace("seed = 0", f"seed = {seed}", 1)
    source = source.replace(
        "datasets_path = 'Datasets/SHD'",
        f"datasets_path = {str(dataset_path)!r}",
        1,
    )
    source = source.replace(
        "run_name = 'Wandb Run Name'",
        f"run_name = 'BINN-{mode}-seed-{seed}'",
        1,
    )
    (worktree / "config.py").write_text(source)
    # macOS uses multiprocessing "spawn"; the upstream main.py has no
    # __main__ guard, so num_workers=4 recursively imports the training entry
    # point. A zero-worker loader preserves samples, shuffle RNG, batches, and
    # optimization while changing only platform data-loading concurrency.
    datasets_source = (worktree / "datasets.py").read_text()
    datasets_source = datasets_source.replace("num_workers=4", "num_workers=0")
    (worktree / "datasets.py").write_text(datasets_source)
    if mode == "clean":
        shutil.copy2(clean_template, worktree / "clean_main.py")
    return worktree


def run_reference(
    checkout: Path,
    work_root: Path,
    python: Path,
    dataset_path: Path,
    seed: int,
    mode: str,
    result_path: Path,
    log_path: Path,
    clean_template: Path,
    prepared_worktree: Path | None = None,
) -> dict[str, object]:
    if mode not in ("historical", "clean"):
        raise ValueError(mode)
    worktree = prepared_worktree or prepare_seed_worktree(
        checkout, work_root, seed, dataset_path, mode, clean_template
    )
    result_path.parent.mkdir(parents=True, exist_ok=True)
    log_path.parent.mkdir(parents=True, exist_ok=True)
    command = [str(python), "main.py" if mode == "historical" else "clean_main.py"]
    environment = os.environ.copy()
    environment["PYTHONHASHSEED"] = str(seed)
    environment["BINN_SHD_REFERENCE_RESULT"] = str(result_path.resolve())
    with log_path.open("w", encoding="utf-8") as log:
        completed = subprocess.run(
            command,
            cwd=worktree,
            env=environment,
            stdout=log,
            stderr=subprocess.STDOUT,
        )
    if completed.returncode:
        raise RuntimeError(f"{mode} reference seed {seed} failed; see {log_path}")
    if mode == "clean":
        payload = json.loads(result_path.read_text())
    else:
        payload = historical_payload_from_log(
            log_path.read_text(errors="replace"), seed
        )
        from .data import write_json_atomic

        write_json_atomic(result_path, payload)
    return payload


def verify_clean_source(template: Path) -> None:
    source = template.read_text()
    forbidden = (
        "eval_model(official_test_loader" in source.split("for epoch", 1)[1].split(
            "test_loss, test_accuracy", 1
        )[0]
    )
    if forbidden:
        raise RuntimeError("clean reference evaluates official test inside training loop")
    if source.count("eval_model(official_test_loader") != 1:
        raise RuntimeError("clean reference must evaluate official test exactly once")
