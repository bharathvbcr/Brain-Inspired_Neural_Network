# BINN SHD calibration (C1-SHD-CAL)

**claim_axis:** Standard-benchmark calibration
**object_under_test:** Multiclass passthrough-spike LIF under local credit rules
**may_claim:** Software-harness calibration vs chance (1/20) with disclosed ceiling
**must_not_claim:** Gate G2; neuromorphic SOTA; Zenke SuperSpike drop-in on recurrent nets; “local learning impossible”; overnight p27 e-prop ceiling reinterpretation; proto-135 5-class sweep mix-in

- schedule: **PILOT (development / fixture — not a scientific SHD verdict)**
- config hash: `c1-shd-cal-cefefdd888730f8e`
- protocol version: 27
- seeds: 2
- dims: N_IN=32, T=16, n_classes=20 (chance=0.0500)
- subset: n_train=24, n_test=8 (caps max_train=24, max_test=8; 0=uncapped)
- hidden / epochs / lr: 32 / 4 / 0.0500
- fixture: true
- note: FIXTURE / smoke data — not a full-SHD scientific calibration. Fetch official SHD and convert offline (see data/shd/README.md).

## Results

| arm | mean accuracy |
|---|---:|
| `SHD_BROADCAST_PM1` | 0.0000 |
| `SHD_DFA` | 0.1250 |
| `SHD_RL_REINFORCE_FB` (REINFORCE×B) | 0.0000 |
| `SHD_EPROP_CEILING` (ceiling) | 0.1250 |
| chance (1/20) | 0.0500 |

**Ceiling disclosure:** true surrogate e-prop / truncated local BPTT analogue. Full SuperSpike BPTT is available under protocol-29 `c1-shd-full-*` (feed-forward reverse-mode). Do not read the p27 e-prop ceiling as matched SuperSpike.

## Per-seed

| seed | broadcast_pm1 | dfa | rl_reinforce_fb | eprop_ceiling |
|---:|---:|---:|---:|---:|
| 11400784312655117332 | 0.0000 | 0.1250 | 0.0000 | 0.1250 |
| 4354473029266962475 | 0.0000 | 0.1250 | 0.0000 | 0.1250 |

## Reproduce

```bash
# Rust SHD convert (no Python / h5py):
PKG_CONFIG_PATH="$(brew --prefix hdf5)/lib/pkgconfig:${PKG_CONFIG_PATH:-}" \
cargo run --locked --release -p binn-data --features shd-convert --bin convert-shd -- \
--cache-dir data/shd
cargo run --locked --release -p binn-lab --bin c1 -- --shd-cal \
--shd-hidden 32 \
--out results/c1_shd_h32.md
# equivalent: --config-hash c1-shd-cal-cefefdd888730f8e
```

## Ceiling health

| Quantity | Value |
|---|---:|
| DFA hidden-modulator RMS | 1.9931e-2 |
| E-prop hidden-modulator RMS | 1.9560e-2 |
| RMS ratio (larger / smaller) | 1.02 |
| Parity tolerance | 3.00 |


## Non-claims

- **Not Gate G2** and does not reopen `c1-118207fbc3eaba53`.
- **Not** overnight capped p27 (`c1-shd-cal-eb3cb5d93417a638` / h256) remassage.
- **Not** proto-135 5-class exploratory sweep.
- **Not** neuromorphic hardware SOTA without compute / substrate disclosure.
- SuperSpike here = feed-forward reverse-mode BPTT with SuperSpike surrogate — disclose wall time and feasibility; do not claim biology.

## Compute disclosure

- wall_time_s: 0.0
- n_train / n_test: 24 / 8
- seeds × epochs × arms: 2 × 4 × 4
- feasibility: feed-forward SuperSpike BPTT is O(T·H·N_IN) per example; full official splits (8156/2264) are runnable on a workstation CPU with multi-hour wall time — disclose this number; do not claim free SOTA.
