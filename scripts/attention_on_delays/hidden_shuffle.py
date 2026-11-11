"""The `hidden-shuffled` order operator for the delay+attention series (prereg §3, amendment 2026-11-11 §6).

The operator permutes the time axis of s(t), the read-out's input only, per sample, in training and
test alike. The input, the substrate and the reference's own read-out see the unpermuted tensors.
The permutation is handed to `AttentionReadout.forward(s, presentation)`, which gathers along T
before W_e.

**Construction (amendment §6).** Each step t of sample i gets a 64-bit key from a `PortableRng`
stream seeded by (series seed, i, t), and the sample's permutation sorts its T_h steps by key
(stable, so a tie, which needs a 64-bit collision, falls back to t). The key depends on
(seed, i, t) only, so:
- the same (seed, i, T_h) gives the same permutation in every epoch;
- for T_h < T_h', the permutation at T_h is the order of steps 0 … T_h − 1 within the permutation
  at T_h'. The padded length varies by batch, and this is how "the same permutation" survives it.

The seed of step t's stream is derived as
`splitmix64(splitmix64(splitmix64(seed ^ SHUFFLE_KEY) ^ i) ^ t)`, and its key is that stream's first
`PortableRng.next_u64()`. SplitMix64's finaliser decorrelates neighbouring (i, t). This derivation
is an implementation detail, like the read-out's frequency constants: the registered inputs are
(series seed, i, t) and nothing else. Train sample i and test sample i therefore share a key
stream, as the registration's wording implies.

**Where i comes from.** The reference's loader yields `(x, y, x_lengths)`, with no sample index.
`IndexRecorder` wraps a dataset and records every index the loader fetches. With `num_workers = 0`
(the campaign's patch), the single-process loader fetches one batch's indices in batch order and
then collates them, so after a fetch the recorder holds exactly the batch's indices, in row order.
The forward takes them with `take_batch_indices(n)`, which fails loudly on any mismatch: a count
that is not the batch size, indices from two datasets, or a fetch inside a worker process. The
recorder must be the outermost wrapper. A memo outside it would stop the recorder from seeing
indices after the first epoch.

**No global RNG.** Keys are integer hashes. Neither the recorder nor the permutation draws from
torch's or numpy's global generators, so the sampler order and dropout masks stay those of the
intact arm.
"""

from __future__ import annotations

import numpy as np
import torch
import torch.utils.data

from attention_readout import PortableRng

SHUFFLE_KEY = 0x4154544E53485546  # "ATTNSHUF"
_MASK64 = (1 << 64) - 1
_U64 = np.uint64


def _splitmix64_int(z: int) -> int:
    z = (z + 0x9E3779B97F4A7C15) & _MASK64
    z = ((z ^ (z >> 30)) * 0xBF58476D1CE4E5B9) & _MASK64
    z = ((z ^ (z >> 27)) * 0x94D049BB133111EB) & _MASK64
    return z ^ (z >> 31)


def scalar_key(seed: int, index: int, t: int) -> int:
    """The key of step t of sample `index`, through the scalar `PortableRng` (the reference path)."""
    sample = _splitmix64_int(_splitmix64_int((seed & _MASK64) ^ SHUFFLE_KEY) ^ index)
    return PortableRng(_splitmix64_int(sample ^ t)).next_u64()


def _splitmix64(z: np.ndarray) -> np.ndarray:
    z = z + _U64(0x9E3779B97F4A7C15)
    z = (z ^ (z >> _U64(30))) * _U64(0xBF58476D1CE4E5B9)
    z = (z ^ (z >> _U64(27))) * _U64(0x94D049BB133111EB)
    return z ^ (z >> _U64(31))


def _portable_first(seed: np.ndarray) -> np.ndarray:
    value = np.maximum(seed, _U64(1))
    value = value ^ (value >> _U64(12))
    value = value ^ (value << _U64(25))
    value = value ^ (value >> _U64(27))
    return value * _U64(0x2545F4914F6CDD1D)


def step_keys(seed: int, indices, t_steps: int) -> np.ndarray:
    """Keys [len(indices), t_steps] as uint64; equal to `scalar_key` element for element."""
    if t_steps < 1:
        raise ValueError("t_steps must be positive")
    index = np.asarray(indices, dtype=np.int64)
    if index.ndim != 1 or (index < 0).any():
        raise ValueError("indices must be a 1-D sequence of non-negative dataset indices")
    base = _splitmix64(np.array([(seed & _MASK64) ^ SHUFFLE_KEY], dtype=_U64))
    sample = _splitmix64(base ^ index.astype(_U64))
    steps = np.arange(t_steps, dtype=_U64)
    return _portable_first(_splitmix64(sample[:, None] ^ steps[None, :]))


def permutations(seed: int, indices, t_steps: int) -> np.ndarray:
    """Per-sample presentation orders [len(indices), t_steps] (int64): steps sorted by key."""
    return np.argsort(step_keys(seed, indices, t_steps), axis=1, kind="stable")


_pending: list[tuple[str, int]] = []


def reset_pending() -> None:
    """Forget recorded indices (tests only; a run never needs this)."""
    _pending.clear()


def pending_count() -> int:
    return len(_pending)


class IndexRecorder(torch.utils.data.Dataset):
    """Records each index the loader fetches; must be the outermost dataset wrapper."""

    def __init__(self, inner, source: str) -> None:
        self.inner = inner
        self.source = source

    def __len__(self) -> int:
        return len(self.inner)

    def __getitem__(self, i):
        if torch.utils.data.get_worker_info() is not None:
            raise RuntimeError("IndexRecorder needs num_workers = 0: a worker's record never reaches the forward")
        _pending.append((self.source, int(i)))
        return self.inner[i]


def take_batch_indices(n: int) -> list[int]:
    """The indices of the batch just collated, in row order; raises on any mismatch."""
    if len(_pending) != n:
        count = len(_pending)
        _pending.clear()
        raise RuntimeError(f"hidden shuffle: {count} indices recorded for a batch of {n}; "
                           "was this batch fetched through IndexRecorder by a num_workers=0 loader?")
    sources = {source for source, _ in _pending}
    if len(sources) != 1:
        _pending.clear()
        raise RuntimeError(f"hidden shuffle: one batch recorded indices from {sorted(sources)}")
    taken = [index for _, index in _pending]
    _pending.clear()
    return taken


class HiddenShuffle:
    """Builds each batch's presentation order from the recorded sample indices."""

    def __init__(self, seed: int) -> None:
        self.seed = int(seed)

    def presentation(self, batch: int, t_steps: int, device: torch.device | str = "cpu") -> torch.Tensor:
        indices = take_batch_indices(batch)
        return torch.from_numpy(permutations(self.seed, indices, t_steps)).to(device)
