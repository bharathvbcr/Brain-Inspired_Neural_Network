"""One forward and backward pass of a reference run copy on a fixed synthetic batch.

Run as a subprocess with the run copy as the working directory, so the copy's own `config`,
`model` and `snn_delays` modules are the ones imported. It prints one JSON object with
SHA-256 digests of the output, loss, gradients and state, the length T_h of the tensor the
read-out reads, and, when the read-out is attached, facts about its parameters.

usage: forward_probe.py <attention_readout 0|1> <batch> <seed>
"""

from __future__ import annotations

import hashlib
import json
import sys

import numpy as np
import torch
import torch.nn.functional as F


def digest(tensors) -> str:
    h = hashlib.sha256()
    for tensor in tensors:
        array = tensor.detach().cpu().contiguous().numpy()
        h.update(str(array.dtype).encode() + str(array.shape).encode())
        h.update(array.tobytes())
    return h.hexdigest()


def main(argv: list[str]) -> int:
    attention, batch, seed = argv[0] == "1", int(argv[1]), int(argv[2])
    from config import Config
    from snn_delays import SnnDelays
    from utils import set_seed

    if attention:
        Config.attention_readout = True
    torch.set_num_threads(1)
    model = SnnDelays(Config)
    set_seed(Config.seed)
    optimizers = model.optimizers()
    model.train()

    captured = {}
    last_dropout = model.blocks[Config.n_hidden_layers - 1][1][1]
    hook = last_dropout.register_forward_hook(lambda _m, _i, out: captured.__setitem__("s", out))

    generator = torch.Generator().manual_seed(seed)
    t_in = 100
    x = (torch.rand(t_in, batch, Config.n_inputs, generator=generator) < 0.05).float()
    labels = torch.randint(0, Config.n_outputs, (batch,), generator=generator)
    y = F.one_hot(labels, Config.n_outputs).float()

    for optimizer in optimizers:
        optimizer.zero_grad()
    output = model.forward(x)
    loss = model.calc_loss(output, y)
    loss.backward()
    hook.remove()
    readout = getattr(model, "attention_readout", None)

    reference_params = [p for group in optimizers[0].param_groups for p in group["params"]
                        if readout is None
                        or all(p is not q for q in readout.parameters())]
    reference_params += [p for opt in optimizers[1:] for g in opt.param_groups for p in g["params"]]
    result = {
        "max_delay": int(Config.max_delay),
        "t_h": int(captured["s"].shape[0]),
        "t_out": int(output.shape[0]),
        "readout_input_sha": digest([captured["s"]]),
        "readout_input_values": sorted({float(v) for v in torch.unique(captured["s"]).tolist()}),
        "dropout_p": float(Config.dropout_p),
        "output_sha": digest([output]),
        "loss_bits": int(np.float32(loss.item()).view("<u4")),
        "loss": float(loss.item()),
        "reference_grad_sha": digest([p.grad if p.grad is not None else torch.zeros(0) for p in reference_params]),
        "state_keys": sorted(model.state_dict().keys()),
        "state_sha": digest([model.state_dict()[k] for k in sorted(model.state_dict().keys())]),
        "rng_after_sha": digest([torch.get_rng_state()]),
        "attached": readout is not None,
    }
    if readout is not None:
        readout_ids = {id(p) for p in readout.parameters()}
        group0 = {id(p) for p in optimizers[0].param_groups[0]["params"]}
        result["readout"] = {
            "in_weight_group": readout_ids <= group0,
            "in_other_groups": any(id(p) in readout_ids for opt in optimizers
                                   for g in opt.param_groups[(1 if opt is optimizers[0] else 0):]
                                   for p in g["params"]),
            "grads_present": all(p.grad is not None for p in readout.parameters()),
            "grads_finite": all(bool(torch.isfinite(p.grad).all()) for p in readout.parameters()),
            "w_o_grad_abs_sum": float(readout.w_o.grad.abs().sum()),
            "w_o_is_zero": bool((readout.w_o == 0).all()),
        }
    print(json.dumps(result, sort_keys=True))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
