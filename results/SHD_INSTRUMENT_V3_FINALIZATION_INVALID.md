# SHD instrument v3 finalization status

Status: `INVALID_HARNESS` for historical-reference finalization only.

The three historical training processes completed all 150 epochs and their logs
remain immutable under `results/shd_instrument_v3/references/`. The v3 parser
matched both `Acc Valid` and the adjacent `Best Acc Valid` field on every epoch
line, producing 300 matches instead of the required 150. It therefore left the
historical states at `RUNNING` and correctly kept the matrix gate closed.

The clean reference results, v3 manifests, logs, gate record, and zero-cell
matrix ledger are preserved unchanged. No v3 artifact is relabeled as fresh
calibration.

Revision v4 repairs the parser, requires contiguous epoch coverage `0..149`,
records the original v3 manifest/state/log hashes, and distinguishes the
original training source fingerprint from the finalizer source fingerprint.
The completed training is not rerun or silently replaced.
