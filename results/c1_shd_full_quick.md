# BINN SHD full-corpus + SuperSpike ceiling (C1-SHD-FULL)

**claim_axis:** Standard-benchmark calibration
**object_under_test:** Multiclass passthrough-spike LIF under local credit rules
**may_claim:** Software-harness calibration vs chance (1/20) with disclosed ceiling
**must_not_claim:** Gate G2; neuromorphic SOTA; Zenke SuperSpike drop-in on recurrent nets; “local learning impossible”; overnight p27 e-prop ceiling reinterpretation; proto-135 5-class sweep mix-in

- schedule: **PILOT (development / fixture — not a scientific SHD verdict)**
- config hash: `c1-shd-full-1b53d4f8a6ac3d41`
- protocol version: 29
- seeds: 2
- dims: N_IN=32, T=16, n_classes=20 (chance=0.0500)
- subset: n_train=24, n_test=8 (caps max_train=24, max_test=8; 0=uncapped)
- hidden / epochs / lr: 32 / 4 / 0.0500
- fixture: true
- note: FIXTURE / smoke data — not a full-SHD scientific calibration. Fetch official SHD and convert offline (see data/shd/README.md).

## Results

| arm | mean accuracy |
|---|---:|
| `SHD_BROADCAST_PM1` | 0.0625 |
| `SHD_DFA` | 0.1250 |
| `SHD_RL_REINFORCE_FB` (REINFORCE×B) | 0.0000 |
| `SHD_SUPERSPIKE_CEILING` (ceiling) | 0.1250 |
| chance (1/20) | 0.0500 |

**Ceiling disclosure:** true SuperSpike reverse-mode BPTT on the same feed-forward hard-reset LIF used by the local arms (no `W_rec`). Surrogate `σ'(u)=1/(1+β|u|)²`; hard reset cuts the membrane adjoint. This is the nearest feasible BPTT ceiling at SHD scale in this crate — **not** a Zenke SuperSpike drop-in on a recurrent net, and **not** the overnight capped e-prop ceiling (~0.09–0.10 under p27 2000/500).

## Per-seed

| seed | broadcast_pm1 | dfa | rl_reinforce_fb | superspike_ceiling |
|---:|---:|---:|---:|---:|
| 11400784312163610616 | 0.0000 | 0.1250 | 0.0000 | 0.1250 |
| 4354473028747602887 | 0.1250 | 0.1250 | 0.0000 | 0.1250 |

## Reproduce

```bash
# Rust SHD convert (no Python / h5py):
PKG_CONFIG_PATH="$(brew --prefix hdf5)/lib/pkgconfig:${PKG_CONFIG_PATH:-}" \
cargo run --locked --release -p binn-data --features shd-convert --bin convert-shd -- \
--cache-dir data/shd
cargo run --locked --release -p binn-lab --bin c1 -- --shd-full --quick \
--out results/c1_shd_full_quick.md
```

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
