# Amendment 2026-11-11 — the read-out sees the padded batch

**Registered:** 2026-11-11, before any stage-1 run.
**Amends:** [`PREREG_2026-11-08_ATTENTION_ON_THE_DELAY_SUBSTRATE.md`](PREREG_2026-11-08_ATTENTION_ON_THE_DELAY_SUBSTRATE.md): §3 ("Lengths", `pos`, the parity gate), the `hidden-shuffled` operator, and the §8 stage-1 item on T_h.
**Decided by:** the owner, 2026-11-11. Of the two options put to them, they chose to keep the padded batch length rather than mask each sample to its own length.

## 0. Why this amendment is admissible now

The registration allows dated amendments "only while no run exists". Seven runs exist: stage 0 (A00 ×3, A00′, B00 ×3), whose HA-0 passed.
- Those runs are the plain reference. They ran with `hooks: "absent"` and contain no read-out or shuffle code (§4). Nothing they measured involves the definitions changed here.
- §8 says the stage-1 code "may not change any arm, bar, seed or definition… If it would need to, the change is a dated amendment, made before any stage-1 run." No stage-1 run exists. This amendment is that case.
- **Nothing else changes:** no arm, seed, bar, hypothesis, configuration (§2), analyser constant or record field. HA-0's reading stands as read.

## 1. What §3 got wrong

§3 says: "T_h = 124 at `max_delay = 25` and T_h = 100 at `max_delay = 1` … These lengths are inferred from the padding arithmetic." The padding arithmetic is right. The input length it assumed is not.
- **The input length was assumed fixed.** The reference's loader (`SHD_dataloaders`, spikingjelly `pad_sequence_collate`, `batch_size = 256`) pads each batch to its longest sample. So the input length T_in is a property of the batch, not a constant. The probe that "asserted" 124 / 100 fed a synthetic batch with T_in = 100.
- **Per-sample lengths** were measured directly from the binned frame counts at `time_step = 10` ms:
  - train: 23–124 frames, median 65, with 53 of 8,156 samples over 100;
  - test: 31–105 frames, with 5 of 2,264 over 100.
- **Training batches** (64,000 simulated shuffled batches over the measured lengths): padded T_in runs from 89 to 124, median 105, with 1% / 10% / 90% / 99% quantiles of 95 / 99 / 110 / 124. T_in = 100 in 7.7% of batches.
- **Test batches** (fixed order, 9 batches): padded T_in = 105, 104, 92, 95, 101, 84, 87, 94, 95. This assumes the loader lists files in sorted class and file order, as torchvision's `DatasetFolder` does, so these values are *inferred*, not read from the loader.
- **The corrected lengths are a relation, not a pair of constants:**
  - at max_delay 25, T_h = T_in + 24 and the output sequence has T_in + 36 steps;
  - at max_delay 1, both are T_in.
  - §3's "136 and 100" output lengths are the T_in = 100 case of the same relation.

## 2. The definition, as amended

The read-out's time axis is the padded batch length, the same axis the reference's own read-out sums over.

```text
T_in     = padded input length of the batch (its longest sample)
T_h      = T_in + 24 at max_delay 25, T_in at max_delay 1
s(t)     = post-dropout spikes of the last hidden layer, t = 0 … T_h − 1, including padded steps
pos(t)   = the instrument's sinusoidal code at u = t / (T_h − 1)
z, the 4 blocks, pooled = mean over all T_h steps, a = W_a pooled     (unchanged)
```

- **A sample's `a` depends on its batch.** It is a function of the sample and its batch: its normalised positions, its mean-pool and its attention all span T_h, and T_h is set by the batch's longest member.
- **This is the reference's own convention.** `model.py` discards the per-sample lengths (`for i, (x, y, _) in enumerate(...)`, lines 212 and 358), and `calc_loss` sums softmax over every output step, padded ones included. With this amendment, R1 and the reference's read-out see the same steps.
- **Why per-sample masking was not chosen:** it would add two more run-copy patches (to thread `x_lengths` from `clean_main` and `eval_model` into the forward), a masked mean, a masked softmax and a per-sample u. That is more code in the path whose disabled form must stay bit-identical, and it would treat R1's steps differently from the reference's.

## 3. This is a deviation from the Rust instrument

§3 opens "ported with an unchanged definition". After this amendment, that holds for the per-step operations, the weights and their initialisation, which the parity test checks to ≤1e-5. It does not hold for the length over which position and pooling run.
- **The anchor:** h128, `published-2ms`, `adjacent-sum-5`, +0.1258. The instrument runs each sample alone at its own length, with no padding.
- **The closest Rust measurement** is the `fixed-tN` ladder (Appendix I): a fixed 1,400 ms window, trailing silence included in the mean and the positions, gain +0.1927 at `fixed-t100`.
  - It is cited as an analogy, not an equivalence: there the length was the same for every sample, and here it varies with the batch.
- **How to read the result:** a gap between this series' G and the instrument's gain may partly reflect this difference. The series' contrasts (G, Gₕ, DiD, G_B) are unaffected in fairness, because every arm reads the same batches (§5).

## 4. What padded steps carry

Padded steps are not silent in s(t). Measured on an unpatched run copy of `d169b4e`: untrained, in training mode, on a synthetic batch of 8 samples with lengths 30–124, zero-padded to 124. The rate is the fraction of non-zero entries.
- **At max_delay 25:** steps more than 48 past a sample's end, beyond the two delay layers' reach, still fire at 1.9–8.5%. Inside the sample's span the rate is 6.4–11.1%.
- **At max_delay 1:** padded steps never fire (0.0 at every length).
- **The mechanism is not established.** Batch normalisation in training mode, acting on the zero input, is one untested account.
- **A trained model was not measured.**

Whatever the substrate produces on those steps reaches the reference's output `Dcls1d` and R1 identically.

## 5. Why the gates are unaffected

- **Batching is deterministic.** Batch composition, and so every batch's T_h, is a deterministic function of the seed. The training sampler shuffles from the RNG that `set_seed` fixes, and the test loader does not shuffle.
  - So the A/A controls, HA-0(c) and the hooks-disabled check compare like with like.
- **The read-out moves no draw.** Enabling it leaves the global RNG unchanged. `test_attention_readout` checks this for construction, and `test_readout_patch` for one forward and backward.
  - So at a given seed, A00, A10 and A10ₕ see the same batches in the same order.
  - B00 and B10 see the same batches as each other, with T_h = T_in.

## 6. Consequence for `hidden-shuffled`

The order operator permutes what the read-out reads. The read-out now reads all T_h padded steps, so the permutation spans all T_h steps of each sample, padded ones included. Permuting only a sample's own span would leave an ordered padded tail, and that tail fires at max_delay 25 (§4).

T_h varies across epochs for the same sample, so "the same permutation in every epoch" cannot hold literally. The construction that keeps its intent:
- each step t of sample i draws a key from a `PortableRng` stream seeded by (series seed, i, t);
- the permutation sorts the steps by key.
- Two epochs that give a sample different T_h then agree on the relative order of every step they share.
- The key stream draws nothing from the global torch RNG.

The hidden-shuffle tests are restated accordingly:
- per-unit spike counts are preserved;
- time order is changed;
- the same (seed, i, T_h) gives the same permutation;
- for T_h < T_h′, the permutation at T_h is the order of steps 0 … T_h − 1 within the permutation at T_h′.

## 7. Tests and checklist, as amended

- **The parity gate is unchanged:** T_h ∈ {124, 100}, ≤1e-5 max-abs on `pooled` and `a`. It tests the forward at a given length, and the length is a parameter.
- **The §8 stage-1 item** "with T_h asserted at 124 and 100" becomes: "with T_h = T_in + 24 (max_delay 25) and T_in (max_delay 1), and the output sequence T_in + 36 and T_in, asserted for the batch's padded T_in." The local test runs it at T_in ∈ {89, 100, 124}: the measured training extremes and the registration's value.
  - Disabled bit-identity is asserted at each length.

## 8. Other items recorded here

- **The parity test was not committed with the registration.** §3 said it would be. It was committed in BINN `00142d8` on 2026-11-11, after stage 0 and before any stage-1 run. Stage 0 used no read-out code. Its fixture was generated from `binn-learn` at `68a32a3`, and the fixture rebuilt at `e3017d0` is value-for-value identical.
- **The positional code's 16 frequencies are embedded as float32 bit patterns** taken from the Rust forward. Rust's float32 `powf` agrees with neither numpy's nor macOS libm's at 7 of 16 values. Recomputing them would move the code by up to 4.6e-5, beyond the parity bar. This is an implementation detail. The ladder is unchanged, and each constant is within 1.93 ulps of the exact value.

## What did not change

No arm, seed, bar, hypothesis, configuration, analyser constant or record field changed, and no number was re-measured. The seven stage-0 records and HA-0's PASS are untouched.
