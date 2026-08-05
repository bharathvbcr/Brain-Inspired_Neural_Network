# Protocol v148 — shortcut-accessibility contrast

Registered 2026-07-26 before scientific execution.

Protocol v147 is already frozen and executed as
`temporal-elig-v147-17a7bfb76ca5896a`. This experiment therefore uses fresh
protocol v148 and does not rewrite, reinterpret, or delete v147.

## Single-experiment contract

Hash: `shortcut-access-v148-953d6f24133cafb6`

The executable `shortcut-accessibility-contrast` always runs both variants.
There is no CLI selector for A or B.

Both variants use:

- the same four-class `SharedTemporalNet` local-feedback code path;
- the same true shared-forward BPTT reference;
- the same initialization, immutable feedback, seed, examples, labels,
  nuisance realization, width, learning rates, exposure order, and epochs;
- three fresh paired seeds derived from `0x7E4A514800000001`;
- 200 training examples, 100 test examples, 20 epochs, and hidden width 64;
- temporal difficulty `(jitter=0, distractors=4)`.

The sole intervention is shortcut accessibility:

- **A — rate-accessible:** add 16 events to the class-indexed input channel.
  Marker times and total marker count are identical across classes.
- **B — rate-immune:** exact v144 construction, with byte-identical
  per-channel counts within every four-label quartet.

The runner must recover B exactly after subtracting A's frozen marker. Failure
is `INVALID_HARNESS`.

## Required report fields

For each seed and aggregate:

- raw-rate test accuracy;
- local train/test accuracy, predicted-class count, and majority fraction;
- BPTT train/test accuracy and predicted-class count;
- held-out final hidden mean rate, active fraction (`rate >= 0.01`), and
  saturated fraction (`rate >= 0.99`) for local and BPTT;
- local feedback-modulator RMS evaluated after training without updates;
- parameter-preserving evaluation, paired-intervention conformance, and exact
  local/BPTT replay.

## Validity gates

- A raw-rate accuracy `>= 0.90`;
- B raw-rate accuracy `<= 0.30`;
- A and B BPTT accuracy `>= 0.90`;
- A raw-rate and both BPTT arms predict all four classes;
- all activity and modulator diagnostics are finite;
- local modulator RMS is positive in both variants;
- every replay and paired-intervention check passes.

Failure of a validity gate is `INVALID_HARNESS`; no shortcut claim may be made.

## Frozen outcomes

Using `local_high = 0.80` and `chance_like = 0.30`:

1. A local `>= 0.80`, B local `<= 0.30`:
   **PASS — local learning depends on shortcut accessibility.**
2. A local `<= 0.30`, B local `<= 0.30`:
   **INVALID_HARNESS — the multiclass local path failed its positive control.**
   Stop adding experiments and make the multiclass local arm learn a known
   rate-accessible task before any further scientific claim.
3. A local `>= 0.80`, B local `>= 0.80`:
   **FAIL — v144 was a difficulty artifact, not a rate-shortcut result.**
4. Any intermediate pattern:
   **FAIL — stop without a claim or follow-up sweep.**

No threshold, marker-count, optimizer, width, epoch, or difficulty sweep follows
this result.

## Reproduction

```bash
cargo run --locked --release -p binn-lab \
  --bin shortcut-accessibility-contrast -- \
  --config-hash shortcut-access-v148-953d6f24133cafb6 \
  --out results/shortcut_accessibility_contrast_v148.md
```
