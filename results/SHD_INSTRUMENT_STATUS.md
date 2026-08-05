# SHD instrument status

**State: `UNCALIBRATED` — calibration-only authorization**

This status is an interpretation and execution guard. It does not rewrite any
historical protocol hash, result, threshold, or verdict.

## Authorized work

- canonical SHD preprocessing and data-contract checks;
- Python/Rust forward, gradient, optimizer, and fixture parity;
- historical and clean gradient-reference reproduction;
- fresh-process replay and provenance validation;
- the matched BPTT matrix only after every prerequisite gate passes.

## Blocked work

- new SHD local-learning or architecture-ablation campaigns;
- new transfer campaigns;
- new optimizer/budget searches intended to support a scientific claim.

The Rust experiment entry points for the affected campaign families enforce
this state. The calibration runner has no flag that bypasses prerequisites.

## Why

Protocol v143 measured the full SuperSpike hidden arm at `0.4157`, below the
raw-input-rate control at `0.4428`. The paired bootstrap interval for
hidden-minus-input was entirely negative, and neither readout was degenerate.
The legacy result is a valid stop signal, but it is not a calibrated gradient
reference or a fresh-training replay.

Calibration PASS requires:

1. canonical data and Python/Rust fixture parity;
2. matched forward/gradient/update parity;
3. successful pinned historical reproduction, labeled exposure-tainted;
4. three clean reference seeds, each at least `0.80`;
5. at least one matched Python/Rust configuration meeting all registered
   accuracy, backend-gap, class-coverage, majority, activity, provenance, and
   non-finite gates.

Until then, accuracy from a local rule is descriptive engineering telemetry,
not evidence about locality or credit assignment.
