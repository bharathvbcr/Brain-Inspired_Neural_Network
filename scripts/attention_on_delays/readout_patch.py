"""Attach the attention read-out (R1) to a run copy of the pinned SNN-delays reference.

Prereg §3: the read-out reads `x` after `blocks[n_hidden_layers-1][1][1]` (the last hidden layer's
dropout; `stateful_synapse = False`, so this is exactly what the output Dcls1d reads), and its
logits `a` are added to the output membrane sequence at every step, so the reference's
Σ_t softmax(v(t)) becomes Σ_t softmax(v(t) + a). Its parameters join the reference's weight group,
with the same Adam and one-cycle schedule.

This edits `snn_delays.py` in a *run copy* by exact text replacement, the way the kernel ablation
patches `config.py`; each anchor must occur exactly once, or nothing is written. The pinned
checkout is never touched.

Enabled by `Config.attention_readout = True` on the run's `Config`. When it is absent or False, the
patched model attaches nothing: no module, no parameter, no `state_dict` entry, no optimiser
entry. The forward then differs from the reference only by holding a reference to a tensor it
already computed, so its arithmetic is the reference's. `test_readout_patch.py` checks that the
forward and loss are bit-identical to the unpatched reference.

usage: readout_patch.py <run-copy-dir>
"""

from __future__ import annotations

import shutil
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
MODULE = HERE / "attention_readout.py"

# (anchor, replacement): each anchor must occur exactly once in the pinned snn_delays.py.
EDITS = [
    (
        "from utils import set_seed\n",
        "from utils import set_seed\n"
        "from attention_readout import AttentionReadout\n",
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
        "            self.weights.extend(self.attention_readout.parameters())\n",
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
        "            out = out + self.attention_readout(readout_input).unsqueeze(0)\n\n"
        "        return out",
    ),
]


def patch_text(text: str) -> str:
    if "AttentionReadout" in text:
        raise RuntimeError("snn_delays.py already carries the read-out patch")
    for anchor, replacement in EDITS:
        count = text.count(anchor)
        if count != 1:
            raise RuntimeError(f"anchor occurs {count} times, expected 1: {anchor!r}")
        text = text.replace(anchor, replacement, 1)
    return text


def apply(copy_dir: Path) -> None:
    target = copy_dir / "snn_delays.py"
    patched = patch_text(target.read_text())
    shutil.copyfile(MODULE, copy_dir / MODULE.name)
    temporary = target.with_suffix(".py.tmp")
    temporary.write_text(patched)
    temporary.replace(target)


def main(argv: list[str]) -> int:
    if len(argv) != 1:
        print(__doc__.strip().splitlines()[-1], file=sys.stderr)
        return 2
    apply(Path(argv[0]))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
