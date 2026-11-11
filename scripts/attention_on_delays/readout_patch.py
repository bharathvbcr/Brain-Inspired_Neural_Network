"""Attach the attention read-out (R1) and its hidden-shuffle hook to a run copy of the pinned SNN-delays reference.

Prereg §3: the read-out reads `x` after `blocks[n_hidden_layers-1][1][1]` (the last hidden layer's
dropout; `stateful_synapse = False`, so this is exactly what the output Dcls1d reads), and its
logits `a` are added to the output membrane sequence at every step, so the reference's
Σ_t softmax(v(t)) becomes Σ_t softmax(v(t) + a). Its parameters join the reference's weight group,
with the same Adam and one-cycle schedule.

The `hidden-shuffled` arm (amendment 2026-11-11 §6) adds a per-sample permutation of what the
read-out reads, built by `hidden_shuffle.HiddenShuffle` from each batch's sample indices. The
indices are recorded by `hidden_shuffle.IndexRecorder`, which this patch wraps around the SHD
datasets in `SHD_dataloaders`.

This edits `snn_delays.py` and `datasets.py` in a *run copy* by exact text replacement, the way the
kernel ablation patches `config.py`. Each anchor must occur exactly once, or nothing is written.
The pinned checkout is never touched.

**Order of patches.** Apply it after the campaign's `num_workers=4 -> 0` patch, which it requires,
and after the memo patch (`harness_patch.py`) when that is used. The recorder must be the
outermost dataset wrapper, because a memo hit never reaches an inner one.

**Switches,** both read from the run's `Config`:
- `attention_readout = True` attaches R1;
- `hidden_shuffle = True` also shuffles what R1 reads, and requires `attention_readout`.

When neither is set, the patched model attaches nothing (no module, parameter, `state_dict` entry
or optimiser entry), and the loaders get no wrapper. The forward then differs from the reference
only by holding a reference to a tensor it already computed. `test_readout_patch.py` checks that
the forward, loss, evaluation and RNG state are bit-identical to the unpatched reference.

usage: readout_patch.py <run-copy-dir>
"""

from __future__ import annotations

import shutil
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
MODULES = (HERE / "attention_readout.py", HERE / "hidden_shuffle.py")

# (anchor, replacement): each anchor must occur exactly once in the pinned snn_delays.py.
EDITS = [
    (
        "from utils import set_seed\n",
        "from utils import set_seed\n"
        "from attention_readout import AttentionReadout\n"
        "from hidden_shuffle import HiddenShuffle\n",
    ),
    (
        "        super().__init__(config)\n\n        self.config = config\n",
        "        super().__init__(config)\n\n        self.config = config\n"
        "        # Attention read-out R1 (prereg §3). Attached only when the run's Config asks for it,\n"
        "        # so a disabled read-out leaves no module, parameter or optimiser entry behind.\n"
        "        self.attention_readout = None\n"
        "        if getattr(config, 'attention_readout', False):\n"
        "            if config.stateful_synapse or config.loss != 'sum':\n"
        "                raise RuntimeError('the read-out is registered for stateful_synapse=False, loss=sum')\n"
        "            self.attention_readout = AttentionReadout(config.n_hidden_neurons, config.n_outputs, config.seed)\n"
        "            self.weights.extend(self.attention_readout.parameters())\n"
        "        # Hidden-shuffle hook (amendment 2026-11-11 §6): permutes only what the read-out reads.\n"
        "        self.hidden_shuffle = None\n"
        "        if getattr(config, 'hidden_shuffle', False):\n"
        "            if self.attention_readout is None:\n"
        "                raise RuntimeError('hidden_shuffle permutes what the read-out reads; it needs attention_readout')\n"
        "            self.hidden_shuffle = HiddenShuffle(config.seed)\n",
    ),
    (
        "        # Finally, we apply same transforms for the output layer\n        x = x.permute(1,2,0)\n",
        "        # Finally, we apply same transforms for the output layer\n"
        "        readout_input = x  # post-dropout spikes of the last hidden layer, [T_h, batch, hidden]\n"
        "        x = x.permute(1,2,0)\n",
    ),
    (
        "                out = self.blocks[-1][1][0].v_seq\n\n        return out",
        "                out = self.blocks[-1][1][0].v_seq\n\n"
        "        if self.attention_readout is not None:\n"
        "            presentation = None\n"
        "            if self.hidden_shuffle is not None:\n"
        "                presentation = self.hidden_shuffle.presentation(\n"
        "                    readout_input.shape[1], readout_input.shape[0], readout_input.device)\n"
        "            out = out + self.attention_readout(readout_input, presentation).unsqueeze(0)\n\n"
        "        return out",
    ),
]

SHD_START = "def SHD_dataloaders(config):\n"
SHD_END = "def SSC_dataloaders(config):\n"
TRAIN_LOADER = "  train_loader = DataLoader("
RECORDER = (
    "  if getattr(config, 'hidden_shuffle', False):\n"
    "    # Hidden-shuffle hook (amendment 2026-11-11 §6): record each batch's sample indices.\n"
    "    # Outermost wrapper, so a memo hit still records; it needs num_workers=0.\n"
    "    from hidden_shuffle import IndexRecorder\n"
    "    train_dataset = IndexRecorder(train_dataset, 'train')\n"
    "    test_dataset = IndexRecorder(test_dataset, 'test')\n"
)


def patch_text(text: str) -> str:
    if "AttentionReadout" in text:
        raise RuntimeError("snn_delays.py already carries the read-out patch")
    for anchor, replacement in EDITS:
        count = text.count(anchor)
        if count != 1:
            raise RuntimeError(f"anchor occurs {count} times, expected 1: {anchor!r}")
        text = text.replace(anchor, replacement, 1)
    return text


def patch_datasets_text(text: str) -> str:
    if "IndexRecorder" in text:
        raise RuntimeError("datasets.py already carries the hidden-shuffle recorder")
    if text.count(SHD_START) != 1 or text.count(SHD_END) != 1:
        raise RuntimeError("SHD_dataloaders / SSC_dataloaders not found exactly once; reference moved?")
    start = text.index(SHD_START)
    end = text.index(SHD_END)
    if end < start:
        raise RuntimeError("SSC_dataloaders precedes SHD_dataloaders; reference moved?")
    body = text[start:end]
    if "num_workers=4" in body or body.count("num_workers=0") != 2:
        raise RuntimeError("apply the num_workers=0 patch first: the recorder only sees main-process fetches")
    if body.count(TRAIN_LOADER) != 1:
        raise RuntimeError("SHD train loader not found exactly once in SHD_dataloaders")
    body = body.replace(TRAIN_LOADER, RECORDER + TRAIN_LOADER, 1)
    return text[:start] + body + text[end:]


def _write(target: Path, text: str) -> None:
    temporary = target.with_suffix(".py.tmp")
    temporary.write_text(text)
    temporary.replace(target)


def apply(copy_dir: Path) -> None:
    model = copy_dir / "snn_delays.py"
    datasets = copy_dir / "datasets.py"
    patched_model = patch_text(model.read_text())
    patched_datasets = patch_datasets_text(datasets.read_text())
    for module in MODULES:
        shutil.copyfile(module, copy_dir / module.name)
    _write(model, patched_model)
    _write(datasets, patched_datasets)


def main(argv: list[str]) -> int:
    if len(argv) != 1:
        print(__doc__.strip().splitlines()[-1], file=sys.stderr)
        return 2
    apply(Path(argv[0]))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
