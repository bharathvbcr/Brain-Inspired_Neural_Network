> **SUPERSEDED 2026-07-25** — see `results/HARD_AUDIT_v12_2026-07-25.md`.
>
> Same ~56x ceiling modulator-scale deficit as `c1_shd_h128.md`.
>
> Fixes landed in the same commit; re-run before citing any number from this file.

# BINN SHD calibration (C1-SHD-CAL)

**claim_axis:** Standard-benchmark calibration
**object_under_test:** Multiclass passthrough-spike LIF under local credit rules
**may_claim:** Software-harness calibration vs chance (1/20) with disclosed ceiling
**must_not_claim:** Gate G2; neuromorphic SOTA; Zenke SuperSpike drop-in on recurrent nets; “local learning impossible”; overnight p27 e-prop ceiling reinterpretation; proto-135 5-class sweep mix-in

- schedule: **SCIENTIFIC**
- config hash: `c1-shd-cal-bafa6835d8de7eb8`
- protocol version: 27
- seeds: 5
- dims: N_IN=700, T=100, n_classes=20 (chance=0.0500)
- subset: n_train=2000, n_test=500 (caps max_train=2000, max_test=500; 0=uncapped)
- hidden / epochs / lr: 256 / 20 / 0.0200
- fixture: false
- note: Full SHD cache loaded; evaluation uses capped subsets (n_train=2000, n_test=500; caps max_train=2000, max_test=500). Calibration only — not full-corpus SOTA. Ceiling = true e-prop; see protocol-29 `c1-shd-full-*` for SuperSpike BPTT on full splits.

## Results

| arm | mean accuracy |
|---|---:|
| `SHD_BROADCAST_PM1` | 0.0528 |
| `SHD_DFA` | 0.2088 |
| `SHD_RL_REINFORCE_FB` (REINFORCE×B) | 0.0504 |
| `SHD_EPROP_CEILING` (ceiling) | 0.1000 |
| chance (1/20) | 0.0500 |

**Ceiling disclosure:** true surrogate e-prop / truncated local BPTT analogue. Full SuperSpike BPTT is available under protocol-29 `c1-shd-full-*` (feed-forward reverse-mode). Do not read the p27 e-prop ceiling as matched SuperSpike.

## Per-seed

| seed | broadcast_pm1 | dfa | rl_reinforce_fb | eprop_ceiling |
|---:|---:|---:|---:|---:|
| 11400784312508578836 | 0.0560 | 0.2360 | 0.0620 | 0.0940 |
| 4354473029077956651 | 0.0520 | 0.2040 | 0.0360 | 0.0740 |
| 15755469980507731006 | 0.0460 | 0.1980 | 0.0500 | 0.0820 |
| 8709160896129724501 | 0.0560 | 0.1560 | 0.0540 | 0.1380 |
| 1663413533332040808 | 0.0540 | 0.2500 | 0.0500 | 0.1120 |

## Reproduce

```bash
# Rust SHD convert (no Python / h5py):
PKG_CONFIG_PATH="$(brew --prefix hdf5)/lib/pkgconfig:${PKG_CONFIG_PATH:-}" \
cargo run --locked --release -p binn-data --features shd-convert --bin convert-shd -- \
--cache-dir data/shd
cargo run --locked --release -p binn-lab --bin c1 -- --shd-cal --shd-hidden 256 \
--out results/c1_shd_h256.md
# equivalent: --config-hash c1-shd-cal-bafa6835d8de7eb8
```

## Non-claims

- **Not Gate G2** and does not reopen `c1-118207fbc3eaba53`.
- **Not** overnight capped p27 (`c1-shd-cal-eb3cb5d93417a638` / h256) remassage.
- **Not** proto-135 5-class exploratory sweep.
- **Not** neuromorphic hardware SOTA without compute / substrate disclosure.
- SuperSpike here = feed-forward reverse-mode BPTT with SuperSpike surrogate — disclose wall time and feasibility; do not claim biology.

## Compute disclosure

- wall_time_s: 7866.9
- n_train / n_test: 2000 / 500
- seeds × epochs × arms: 5 × 20 × 4
- feasibility: feed-forward SuperSpike BPTT is O(T·H·N_IN) per example; full official splits (8156/2264) are runnable on a workstation CPU with multi-hour wall time — disclose this number; do not claim free SOTA.
