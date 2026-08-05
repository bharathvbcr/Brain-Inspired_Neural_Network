# SHD instrument calibration protocol

**Instrument state:** `UNCALIBRATED`  
**Matrix:** 432 matched BPTT cells  
**Reference commit:** `d169b4e3049a3d5bff56c84a8b2f0c4e835aafda`
**Compatible SpikingJelly commit:** `6dca147afe684b5e78d9c9d430e8761f921437b2`

This protocol implements the audit in `results/SHD_INSTRUMENT_STATUS.md`.
Historical SHD results remain frozen and are not replayed into this ledger.
V1 was invalidated by a pre-reference source correction; v2 passed its
mechanical parity gates but exposed a reference-environment setup defect before
any reference or matrix run. The authoritative fresh ledger is
`results/shd_instrument_v3/`.

## Execution

```bash
cd binn

.venv-shd/bin/python scripts/run_shd_instrument.py prepare
.venv-shd/bin/python scripts/run_shd_instrument.py preflight
.venv-shd/bin/python scripts/run_shd_instrument.py replay-smoke
.venv-shd/bin/python scripts/run_shd_instrument.py setup-reference

for seed in 5170001 5170002 5170003; do
  .venv-shd/bin/python scripts/run_shd_instrument.py reference \
    --mode historical --seed "$seed"
done
for seed in 5170001 5170002 5170003; do
  .venv-shd/bin/python scripts/run_shd_instrument.py reference \
    --mode clean --seed "$seed"
done

# Refuses to start unless every prerequisite gate passed.
.venv-shd/bin/python scripts/run_shd_instrument.py run-matrix
```

No `.done` marker is authoritative. A cached cell is reusable only when its
atomic state is `COMPLETE`, its result exists, and its source/core/cell manifest
fingerprints match.

## Interpretation

- Historical reference: `EXPOSURE_TAINTED_DESCRIPTIVE`; mean must be within
  `0.05` of published `0.951`.
- The macOS reference worktree changes only `DataLoader num_workers` from 4 to
  0 because the upstream unguarded entry point is incompatible with spawn.
  This platform adaptation is hashed and disclosed per seed.
- Clean reference: official test is read once after epoch 150; every seed must
  reach at least `0.80`.
- A matched configuration calibrates only if all three seeds pass in both
  backends, each paired accuracy gap is at most `0.05`, all classes are
  predicted, majority prediction is below `0.30`, activity is nondegenerate,
  and provenance/non-finite gates pass.
- Mechanical completion, harness validity, scientific status, invalidity, and
  pending work are separate ledger fields.
