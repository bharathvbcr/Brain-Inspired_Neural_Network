"""Clean-test fork of the pinned SNN-delays SHD recipe.

This file is copied into a pinned reference worktree. It intentionally reuses
the reference model, optimizer, scheduler, data loader, and configuration while
removing all official-test evaluation/checkpoint selection during training.
"""

from __future__ import annotations

import json
import os
import time

import numpy as np
import torch
import torch.nn.functional as F
from tqdm import tqdm

from config import Config
from datasets import SHD_dataloaders
from snn_delays import SnnDelays
from utils import set_seed


def _select_device() -> torch.device:
    """Pick the torch device, defaulting to the historical CPU/CUDA behaviour.

    The original line was `cuda if cuda.is_available() else cpu`. This host is
    Apple Silicon and has no CUDA, so the frozen 150-epoch SHD reference has
    always run single-device on CPU (~4.45 h) while the GPU sat idle.

    MPS is available and would cut that substantially, but it is **opt-in**, not
    the default. This is a pinned reference baseline that other results are
    calibrated against: MPS and CPU do not produce bit-identical float results,
    so silently switching backends would move a number that downstream claims
    treat as fixed. Set BINN_SHD_TORCH_DEVICE=mps to take the speedup, and treat
    the result as a new baseline rather than a continuation of the CPU series.

    The chosen device is already recorded in the emitted payload's `device`
    field, so any run is self-describing.
    """
    requested = os.environ.get("BINN_SHD_TORCH_DEVICE")
    if requested:
        if requested == "mps" and not torch.backends.mps.is_available():
            raise RuntimeError("BINN_SHD_TORCH_DEVICE=mps but MPS is unavailable")
        if requested == "cuda" and not torch.cuda.is_available():
            raise RuntimeError("BINN_SHD_TORCH_DEVICE=cuda but CUDA is unavailable")
        return torch.device(requested)
    return torch.device("cuda" if torch.cuda.is_available() else "cpu")


def main() -> None:
    if Config.dataset != "shd" or Config.epochs != 150:
        raise RuntimeError("clean reference requires frozen SHD/150-epoch config")
    device = _select_device()
    train_loader, official_test_loader = SHD_dataloaders(Config)
    model = SnnDelays(Config).to(device)
    set_seed(Config.seed)
    optimizers = model.optimizers()
    schedulers = model.schedulers(optimizers)
    started = time.monotonic()
    train_curve = []
    test_reads_during_training = 0

    for epoch in range(Config.epochs):
        model.train()
        losses = []
        accuracies = []
        for x, y, _ in tqdm(train_loader, desc=f"clean epoch {epoch + 1}/150"):
            y = F.one_hot(y, Config.n_outputs).float()
            x = x.permute(1, 0, 2).float().to(device)
            y = y.to(device)
            for optimizer in optimizers:
                optimizer.zero_grad()
            output = model.forward(x)
            loss = model.calc_loss(output, y)
            loss.backward()
            for optimizer in optimizers:
                optimizer.step()
            losses.append(float(loss.detach().cpu()))
            accuracies.append(float(model.calc_metric(output, y)))
            model.reset_model(train=True)
        for scheduler in schedulers:
            scheduler.step()
        model.decrease_sig(epoch)
        train_curve.append(
            {
                "epoch": epoch + 1,
                "loss": float(np.mean(losses)),
                "accuracy": float(np.mean(accuracies)),
            }
        )

    test_loss, test_accuracy = model.eval_model(official_test_loader, device)
    payload = {
        "schema": "shd-reference-v1",
        "mode": "clean",
        "seed": Config.seed,
        "accuracy": float(test_accuracy),
        "test_loss": float(test_loss),
        "epochs": Config.epochs,
        "device": str(device),
        "test_reads_during_training": test_reads_during_training,
        "final_test_reads": 1,
        "checkpoint_selected_on": "none-final-epoch",
        "exposure_status": "CLEAN_TEST_ONCE",
        "train_curve": train_curve,
        "wall_secs": time.monotonic() - started,
    }
    output = os.environ["BINN_SHD_REFERENCE_RESULT"]
    temporary = f"{output}.tmp-{os.getpid()}"
    with open(temporary, "w", encoding="utf-8") as handle:
        json.dump(payload, handle, indent=2, sort_keys=True)
        handle.write("\n")
        handle.flush()
        os.fsync(handle.fileno())
    os.replace(temporary, output)
    print("BINN_SHD_REFERENCE_RESULT=" + json.dumps(payload, sort_keys=True))


if __name__ == "__main__":
    main()
