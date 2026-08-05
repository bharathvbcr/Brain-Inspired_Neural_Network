# Calibration artifact status

**Status: `INVALID_HARNESS`**

The v1 manifest and parity artifacts were created before a clippy-driven source
correction to the matched Rust implementation. No reference or matrix cell was
run. These artifacts are preserved for provenance but must not be reused.

The calibration runner moved to `results/shd_instrument_v2/`; it requires a new
immutable manifest and fresh parity execution under the corrected source
fingerprint.
