# BINN SHD calibration (C1-SHD-CAL)

**claim_axis:** Standard-benchmark calibration
**object_under_test:** Multiclass passthrough-spike LIF under local credit rules
**may_claim:** Software-harness calibration vs chance (1/20) with disclosed ceiling
**must_not_claim:** Gate G2; neuromorphic SOTA; Zenke SuperSpike drop-in on recurrent nets; “local learning impossible”; overnight p27 e-prop ceiling reinterpretation; proto-135 5-class sweep mix-in

- schedule: **SCIENTIFIC**
- config hash: `c1-shd-cal-eb3cb5d93417a638`
- protocol version: 27
- seeds: 5
- dims: N_IN=700, T=100, n_classes=20 (chance=0.0500)
- subset: n_train=2000, n_test=500 (caps max_train=2000, max_test=500; 0=uncapped)
- hidden / epochs / lr: 128 / 20 / 0.0200
- fixture: false
- note: Full SHD cache loaded; evaluation uses capped subsets (n_train=2000, n_test=500; caps max_train=2000, max_test=500). Calibration only — not full-corpus SOTA. Ceiling = true e-prop; see protocol-29 `c1-shd-full-*` for SuperSpike BPTT on full splits.

## Results

| arm | mean accuracy |
|---|---:|
| `SHD_BROADCAST_PM1` | 0.0544 |
| `SHD_DFA` | 0.0848 |
| `SHD_RL_REINFORCE_FB` (REINFORCE×B) | 0.0532 |
| `SHD_EPROP_CEILING` (ceiling) | 0.3020 |
| chance (1/20) | 0.0500 |

**Ceiling disclosure:** true surrogate e-prop / truncated local BPTT analogue. Full SuperSpike BPTT is available under protocol-29 `c1-shd-full-*` (feed-forward reverse-mode). Do not read the p27 e-prop ceiling as matched SuperSpike.

## Per-seed

| seed | broadcast_pm1 | dfa | rl_reinforce_fb | eprop_ceiling |
|---:|---:|---:|---:|---:|
| 11400784312508578836 | 0.0560 | 0.1140 | 0.0500 | 0.2920 |
| 4354473029077956651 | 0.0520 | 0.0560 | 0.0540 | 0.3380 |
| 15755469980507731006 | 0.0540 | 0.0960 | 0.0500 | 0.2860 |
| 8709160896129724501 | 0.0560 | 0.0600 | 0.0620 | 0.3020 |
| 1663413533332040808 | 0.0540 | 0.0980 | 0.0500 | 0.2920 |

## Reproduce

```bash
# Rust SHD convert (no Python / h5py):
PKG_CONFIG_PATH="$(brew --prefix hdf5)/lib/pkgconfig:${PKG_CONFIG_PATH:-}" \
cargo run --locked --release -p binn-data --features shd-convert --bin convert-shd -- \
--cache-dir data/shd
cargo run --locked --release -p binn-lab --bin c1 -- --shd-cal \
--shd-hidden 128 \
--out results/c1_shd_h128.md
# equivalent: --config-hash c1-shd-cal-eb3cb5d93417a638
```

## Ceiling health

| Quantity | Value |
|---|---:|
| DFA hidden-modulator RMS | 1.3212e-2 |
| E-prop hidden-modulator RMS | 1.5562e-1 |
| RMS ratio (larger / smaller) | 11.78 |
| Parity tolerance | 3.50 |

> **MODULATOR-SCALE MISMATCH.** The DFA arm and the ceiling apply hidden-layer updates differing by 11.8× in magnitude at a shared learning rate (lr = 0.0200). The comparison measures effective step size, not credit-assignment quality, and must not be reported as a ceiling result.


## Non-claims

- **Not Gate G2** and does not reopen `c1-118207fbc3eaba53`.
- **Not** overnight capped p27 (`c1-shd-cal-eb3cb5d93417a638` / h256) remassage.
- **Not** proto-135 5-class exploratory sweep.
- **Not** neuromorphic hardware SOTA without compute / substrate disclosure.
- SuperSpike here = feed-forward reverse-mode BPTT with SuperSpike surrogate — disclose wall time and feasibility; do not claim biology.

## Compute disclosure

- wall_time_s: 3932.9
- n_train / n_test: 2000 / 500
- seeds × epochs × arms: 5 × 20 × 4
- feasibility: feed-forward SuperSpike BPTT is O(T·H·N_IN) per example; full official splits (8156/2264) are runnable on a workstation CPU with multi-hour wall time — disclose this number; do not claim free SOTA.
