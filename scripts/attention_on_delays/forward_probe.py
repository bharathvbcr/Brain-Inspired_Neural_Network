"""One training step and one evaluation of a reference run copy, fed through the copy's own loaders.

Run as a subprocess with the run copy as the working directory, so the copy's own `config`,
`datasets`, `model` and `snn_delays` modules are the ones imported. The batches come from the
copy's `SHD_dataloaders`, as in `clean_main`: the real `DataLoader`, sampler and
`pad_sequence_collate`, and any dataset wrappers the patches installed. Only the SHD dataset class
is swapped for a synthetic one, `FakeShd`, so no data files are needed.

The synthetic samples have different lengths, the longest being `t_in` frames. Each batch is
therefore padded to `t_in`, the padded length of a real batch: T_h is T_in + 24 at max_delay 25
and T_in at max_delay 1 (amendment 2026-11-11). A sample's content depends only on its index and
split, and draws from no global RNG.

The probe prints one JSON object of SHA-256 digests (batch, output, loss, gradients, state, RNG),
the lengths, the evaluation loss and, when the hooks are attached, facts about them.

usage: forward_probe.py <mode 0|1|2> <batch> <seed> [t_in, default 100]
       mode 0: no switch set; 1: attention_readout; 2: attention_readout + hidden_shuffle
"""

from __future__ import annotations

import hashlib
import json
import sys

import numpy as np
import torch
import torch.nn.functional as F

N_INPUTS = 140


def digest(tensors) -> str:
    h = hashlib.sha256()
    for tensor in tensors:
        array = tensor.detach().cpu().contiguous().numpy()
        h.update(str(array.dtype).encode() + str(array.shape).encode())
        h.update(array.tobytes())
    return h.hexdigest()


class FakeShd:
    """Stands in for `BinnedSpikingHeidelbergDigits`: same constructor call, `(frames, label)` items."""

    size = 6
    t_in = 100

    def __init__(self, root, n_bins, train=True, data_type="frame", duration=None) -> None:
        self.train = bool(train)

    def __len__(self) -> int:
        return FakeShd.size

    def __getitem__(self, i: int):
        length = FakeShd.t_in if i == 0 else FakeShd.t_in - (i * 13) % 31
        rng = np.random.default_rng((0 if self.train else 1 << 20) + i)
        frames = (rng.random((length, N_INPUTS)) < 0.05).astype(np.float64)
        return frames, i % 20


def dataset_chain(dataset) -> list[str]:
    chain = []
    while dataset is not None:
        chain.append(type(dataset).__name__)
        dataset = getattr(dataset, "inner", None)
    return chain


def main(argv: list[str]) -> int:
    mode, batch, seed = int(argv[0]), int(argv[1]), int(argv[2])
    t_in = int(argv[3]) if len(argv) > 3 else 100
    if mode not in (0, 1, 2):
        raise SystemExit("mode must be 0, 1 or 2")
    import datasets
    from config import Config
    from snn_delays import SnnDelays
    from utils import set_seed

    if mode >= 1:
        Config.attention_readout = True
    if mode == 2:
        Config.hidden_shuffle = True
    Config.batch_size = batch
    FakeShd.size, FakeShd.t_in = batch, t_in
    datasets.BinnedSpikingHeidelbergDigits = FakeShd
    torch.set_num_threads(1)

    # clean_main's order: loaders, model, set_seed, optimisers.
    train_loader, test_loader = datasets.SHD_dataloaders(Config)
    model = SnnDelays(Config)
    set_seed(Config.seed)
    optimizers = model.optimizers()
    model.train()

    captured = {}
    last_dropout = model.blocks[Config.n_hidden_layers - 1][1][1]
    hook = last_dropout.register_forward_hook(lambda _m, _i, out: captured.__setitem__("s", out))
    shuffle = getattr(model, "hidden_shuffle", None)
    presentations = []
    if shuffle is not None:
        build = shuffle.presentation
        shuffle.presentation = lambda *a: presentations.append(build(*a)) or presentations[-1]

    readout = getattr(model, "attention_readout", None)
    w_o_zero_at_init = readout is not None and bool((readout.w_o == 0).all())

    x, y, _ = next(iter(train_loader))
    batch_sha = digest([x, y])
    target = F.one_hot(y, Config.n_outputs).float()
    stream = x.permute(1, 0, 2).float()
    for optimizer in optimizers:
        optimizer.zero_grad()
    output = model.forward(stream)
    loss = model.calc_loss(output, target)
    loss.backward()
    for optimizer in optimizers:
        optimizer.step()  # state_sha below is the state after one optimiser step
    hook.remove()
    rng_after = digest([torch.get_rng_state()])
    state =model.state_dict()
    result = {
        "mode": mode,
        "max_delay": int(Config.max_delay),
        "t_in": int(x.shape[1]),
        "t_h": int(captured["s"].shape[0]),
        "t_out": int(output.shape[0]),
        "batch_sha": batch_sha,
        "readout_input_sha": digest([captured["s"]]),
        "readout_input_values": sorted({float(v) for v in torch.unique(captured["s"]).tolist()}),
        "dropout_p": float(Config.dropout_p),
        "output_sha": digest([output]),
        "loss_bits": int(np.float32(loss.item()).view("<u4")),
        "state_keys": sorted(state.keys()),
        "state_sha": digest([state[k] for k in sorted(state.keys())]),
        "rng_after_sha": rng_after,
        "attached": readout is not None,
        "shuffled": shuffle is not None,
        "train_chain": dataset_chain(train_loader.dataset),
        "test_chain": dataset_chain(test_loader.dataset),
    }
    reference_params = [p for group in optimizers[0].param_groups for p in group["params"]
                        if readout is None or all(p is not q for q in readout.parameters())]
    reference_params += [p for opt in optimizers[1:] for g in opt.param_groups for p in g["params"]]
    result["reference_grad_sha"] = digest([p.grad if p.grad is not None else torch.zeros(0)
                                           for p in reference_params])

    model.reset_model(train=True)
    eval_loss, eval_metric = model.eval_model(test_loader, "cpu")
    result["eval_loss_bits"] = int(np.float64(eval_loss).view("<u8"))

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
            "w_o_grad_sha": digest([readout.w_o.grad]),
            "w_o_zero_at_init": w_o_zero_at_init,
            "w_o_moved_by_step": bool((readout.w_o != 0).any()),
        }
    if shuffle is not None:
        import hidden_shuffle
        # Two more passes over the training loader: every batch must find its indices, including
        # when an inner memo serves the samples from cache.
        extra = 0
        for _ in range(2):
            for xb, _yb, _ in train_loader:
                shuffle.presentation(xb.shape[0], xb.shape[1])
                extra += 1
        first = presentations[0]
        result["shuffle"] = {
            "presentations": len(presentations),
            "extra_batches": extra,
            "pending_after": hidden_shuffle.pending_count(),
            "train_shape": list(first.shape),
            "train_identity_rows": int((first == torch.arange(first.shape[1])).all(dim=1).sum()),
            "train_sha": digest([first]),
        }
    print(json.dumps(result, sort_keys=True))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
