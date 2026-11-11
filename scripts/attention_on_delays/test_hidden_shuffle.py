"""The hidden-shuffle operator and its index recorder (amendment 2026-11-11 §6).

- **Parity of the vectorised keys** with the scalar path through `attention_readout.PortableRng`,
  over a few hundred (seed, i, t), including seeds above 2^63.
- **The amendment's four properties:**
  - per-unit spike counts are preserved;
  - time order is changed;
  - the same (seed, i, T_h) gives the same permutation;
  - for T_h < T_h′ the permutation at T_h is the order of steps 0 … T_h − 1 within the
    permutation at T_h′.
  They are checked at the padded lengths the series sees (T_h 89–148).
- **Row alignment through torch's real fetcher.** A variable-length dataset whose sample i carries
  i in its data, wrapped in `IndexRecorder`, is read through `DataLoader(collate_fn=
  pad_sequence_collate, shuffle=True, num_workers=0)`. The batch size does not divide the dataset,
  so the last batch is short. Over two passes, every batch's recorded indices equal the indices in
  its rows, in order.
- **Guards:**
  - a count mismatch is refused;
  - a batch mixing train and test indices is refused;
  - a fetch inside a worker is refused;
  - the permutation draws nothing from torch's or numpy's global RNG.

run: python -m unittest test_hidden_shuffle   (from this directory, in the reference env)
"""

from __future__ import annotations

import unittest
from unittest import mock

import numpy as np
import torch
from spikingjelly.datasets import pad_sequence_collate

import hidden_shuffle as hs

SEED = 5170001
LENGTHS = (89, 100, 113, 124, 148)


class TaggedSamples(torch.utils.data.Dataset):
    """Sample i is a [length_i, 3] array whose first column is i."""

    def __init__(self, n: int) -> None:
        self.n = n

    def __len__(self) -> int:
        return self.n

    def __getitem__(self, i: int):
        length = 5 + (i * 7) % 11
        frames = np.zeros((length, 3))
        frames[:, 0] = i
        return frames, i % 4


class Keys(unittest.TestCase):
    def test_vectorised_keys_equal_the_scalar_portable_rng_path(self) -> None:
        rng = np.random.default_rng(7)
        seeds = [0, 1, SEED, 5170003, (1 << 63) + 12345, (1 << 64) - 1]
        checked = 0
        for seed in seeds:
            indices = [0, 1, 8155] + [int(v) for v in rng.integers(0, 8156, 5)]
            keys = hs.step_keys(seed, indices, 150)
            self.assertEqual(keys.dtype, np.uint64)
            for row, index in enumerate(indices):
                for t in (0, 1, 23, 99, 124, 149) + tuple(int(v) for v in rng.integers(0, 150, 2)):
                    self.assertEqual(int(keys[row, t]), hs.scalar_key(seed, index, t), (seed, index, t))
                    checked += 1
        self.assertGreaterEqual(checked, 300)

    def test_keys_spread_over_64_bits(self) -> None:
        keys = hs.step_keys(SEED, range(64), 148).ravel()
        self.assertEqual(len(np.unique(keys)), keys.size)
        self.assertGreater(int(keys.max()), 1 << 63)
        self.assertLess(int(keys.min()), 1 << 58)

    def test_bad_inputs_are_refused(self) -> None:
        with self.assertRaises(ValueError):
            hs.step_keys(SEED, [-1], 10)
        with self.assertRaises(ValueError):
            hs.step_keys(SEED, [[0, 1]], 10)
        with self.assertRaises(ValueError):
            hs.step_keys(SEED, [0], 0)


class Permutations(unittest.TestCase):
    def test_each_row_is_a_permutation(self) -> None:
        for t_steps in LENGTHS:
            perm = hs.permutations(SEED, range(40), t_steps)
            self.assertEqual(perm.shape, (40, t_steps))
            self.assertTrue((np.sort(perm, axis=1) == np.arange(t_steps)).all())

    def test_time_order_is_changed(self) -> None:
        for t_steps in LENGTHS:
            perm = hs.permutations(SEED, range(256), t_steps)
            identity = np.arange(t_steps)
            self.assertFalse((perm == identity).all(axis=1).any())
            # A real shuffle: few fixed points, as for a random permutation (mean 1).
            self.assertLess(float((perm == identity).sum(axis=1).mean()), 3.0)

    def test_same_seed_index_and_length_give_the_same_permutation(self) -> None:
        first = hs.permutations(SEED, [3, 17, 8155], 124)
        again = hs.permutations(SEED, [8155, 3, 17], 124)  # batch position does not matter
        self.assertTrue(np.array_equal(first, again[[1, 2, 0]]))
        self.assertTrue(np.array_equal(first, hs.permutations(SEED, [3, 17, 8155], 124)))

    def test_seed_and_index_change_the_permutation(self) -> None:
        base = hs.permutations(SEED, [3], 124)
        self.assertFalse(np.array_equal(base, hs.permutations(SEED + 1, [3], 124)))
        self.assertFalse(np.array_equal(base, hs.permutations(SEED, [4], 124)))

    def test_shorter_length_is_the_prefix_order_within_the_longer(self) -> None:
        indices = list(range(0, 8156, 97))
        for short, long in ((89, 148), (100, 124), (113, 148), (124, 148)):
            within = hs.permutations(SEED, indices, long)
            expected = np.stack([row[row < short] for row in within])
            self.assertTrue(np.array_equal(hs.permutations(SEED, indices, short), expected), (short, long))

    def test_per_unit_spike_counts_are_preserved(self) -> None:
        # The gather AttentionReadout applies, on post-dropout values {0, 1/(1-p)}.
        rng = np.random.default_rng(11)
        scaled = np.float32(1.0) / (np.float32(1.0) - np.float32(0.4))
        s = torch.from_numpy(np.where(rng.random((148, 6, 256)) < 0.08, scaled, 0).astype(np.float32))
        perm = torch.from_numpy(hs.permutations(SEED, range(6), 148))
        stream = s.permute(1, 0, 2)
        shown = torch.gather(stream, 1, perm.unsqueeze(2).expand(-1, -1, 256))
        self.assertTrue(torch.equal((shown != 0).sum(1), (stream != 0).sum(1)))
        self.assertTrue(torch.equal(torch.sort(shown, dim=1).values, torch.sort(stream, dim=1).values))
        self.assertFalse(torch.equal(shown, stream))

    def test_no_global_rng_is_drawn(self) -> None:
        torch.manual_seed(1)
        np.random.seed(1)
        torch_before, numpy_before = torch.get_rng_state().clone(), np.random.get_state()[1].copy()
        hs.permutations(SEED, range(256), 148)
        hs.reset_pending()
        recorder = hs.IndexRecorder(TaggedSamples(10), "train")
        recorder[3]
        hs.HiddenShuffle(SEED).presentation(1, 30)
        self.assertTrue(torch.equal(torch_before, torch.get_rng_state()))
        self.assertTrue(np.array_equal(numpy_before, np.random.get_state()[1]))


class Recorder(unittest.TestCase):
    def setUp(self) -> None:
        hs.reset_pending()

    def tearDown(self) -> None:
        hs.reset_pending()

    def test_recorded_indices_align_with_rows_through_the_real_loader(self) -> None:
        torch.manual_seed(SEED)
        dataset = hs.IndexRecorder(TaggedSamples(23), "train")
        loader = torch.utils.data.DataLoader(dataset, collate_fn=pad_sequence_collate, batch_size=5,
                                             shuffle=True, num_workers=0)
        shuffle = hs.HiddenShuffle(SEED)
        orders = []
        for _ in range(2):
            sizes, order = [], []
            for x, _y, lengths in loader:
                rows = [int(x[b, 0, 0]) for b in range(x.shape[0])]
                taken = hs.take_batch_indices(x.shape[0])
                self.assertEqual(taken, rows)
                # And the permutation built in the forward is the one for those indices.
                for row in rows:
                    dataset.inner[row]  # reading the inner dataset records nothing
                self.assertEqual(hs.pending_count(), 0)
                sizes.append(x.shape[0])
                order += rows
            self.assertEqual(sizes, [5, 5, 5, 5, 3])
            self.assertEqual(sorted(order), list(range(23)))
            orders.append(order)
        self.assertNotEqual(orders[0], orders[1], "the sampler reshuffles each pass")
        # presentation() consumes exactly one batch's indices
        dataset[4], dataset[9]
        perm = shuffle.presentation(2, 12)
        self.assertTrue(torch.equal(perm, torch.from_numpy(hs.permutations(SEED, [4, 9], 12))))

    def test_count_mismatch_is_refused_and_clears(self) -> None:
        dataset = hs.IndexRecorder(TaggedSamples(10), "train")
        dataset[1], dataset[2], dataset[3]
        with self.assertRaises(RuntimeError):
            hs.take_batch_indices(2)
        self.assertEqual(hs.pending_count(), 0)
        with self.assertRaises(RuntimeError):
            hs.HiddenShuffle(SEED).presentation(1, 10)  # nothing recorded

    def test_mixed_sources_are_refused(self) -> None:
        train, test = hs.IndexRecorder(TaggedSamples(10), "train"), hs.IndexRecorder(TaggedSamples(10), "test")
        train[1], test[2]
        with self.assertRaises(RuntimeError):
            hs.take_batch_indices(2)

    def test_a_worker_fetch_is_refused(self) -> None:
        dataset = hs.IndexRecorder(TaggedSamples(10), "train")
        with mock.patch.object(torch.utils.data, "get_worker_info", return_value=object()):
            with self.assertRaises(RuntimeError):
                dataset[1]
        self.assertEqual(hs.pending_count(), 0)


if __name__ == "__main__":
    unittest.main()
