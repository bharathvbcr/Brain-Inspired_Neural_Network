# BINN SHD full-corpus + SuperSpike ceiling (C1-SHD-FULL)

**claim_axis:** Standard-benchmark calibration
**object_under_test:** Multiclass passthrough-spike LIF under local credit rules
**may_claim:** Software-harness calibration vs chance (1/20) with disclosed ceiling
**must_not_claim:** Gate G2; neuromorphic SOTA; Zenke SuperSpike drop-in on recurrent nets; “local learning impossible”; overnight p27 e-prop ceiling reinterpretation; proto-135 5-class sweep mix-in

- schedule: **SCIENTIFIC**
- config hash: `c1-shd-full-2c93117075740ed0`
- protocol version: 29
- seeds: 5
- dims: N_IN=700, T=100, n_classes=20 (chance=0.0500)
- subset: n_train=8156, n_test=2264 (caps max_train=0, max_test=0; 0=uncapped)
- hidden / epochs / lr: 128 / 20 / 0.0200
- fixture: false
- note: Full official SHD splits (n_train=8156, n_test=2264; uncapped). Ceiling = true SuperSpike reverse-mode BPTT on feed-forward hard-reset LIF. Calibration / software-harness only — not Gate G2, not neuromorphic SOTA.

## Results

| arm | mean accuracy |
|---|---:|
| `SHD_BROADCAST_PM1` | 0.0513 |
| `SHD_DFA` | 0.3210 |
| `SHD_RL_REINFORCE_FB` (REINFORCE×B) | 0.0493 |
| `SHD_SUPERSPIKE_CEILING` (ceiling) | 0.4315 |
| chance (1/20) | 0.0500 |

**Ceiling disclosure:** true SuperSpike reverse-mode BPTT on the same feed-forward hard-reset LIF used by the local arms (no `W_rec`). Surrogate `σ'(u)=1/(1+β|u|)²`; hard reset cuts the membrane adjoint. This is the nearest feasible BPTT ceiling at SHD scale in this crate — **not** a Zenke SuperSpike drop-in on a recurrent net, and **not** the overnight capped e-prop ceiling (~0.09–0.10 under p27 2000/500).

## Per-seed

| seed | broadcast_pm1 | dfa | rl_reinforce_fb | superspike_ceiling |
|---:|---:|---:|---:|---:|
| 11400784312163597332 | 0.0539 | 0.2716 | 0.0481 | 0.3825 |
| 4354473028747655211 | 0.0534 | 0.3644 | 0.0481 | 0.4077 |
| 15755469980177429566 | 0.0495 | 0.2557 | 0.0481 | 0.4170 |
| 8709160895788937301 | 0.0495 | 0.3803 | 0.0486 | 0.4801 |
| 1663413532989156456 | 0.0504 | 0.3330 | 0.0534 | 0.4704 |

## Reproduce

```bash
# Rust SHD convert (no Python / h5py):
PKG_CONFIG_PATH="$(brew --prefix hdf5)/lib/pkgconfig:${PKG_CONFIG_PATH:-}" \
cargo run --locked --release -p binn-data --features shd-convert --bin convert-shd -- \
--cache-dir data/shd
cargo run --locked --release -p binn-lab --bin c1 -- --shd-full \
--out results/c1_shd_full.md
# hash: c1-shd-full-2c93117075740ed0 (frozen scientific: c1-shd-full-2c93117075740ed0)
```

## Non-claims

- **Not Gate G2** and does not reopen `c1-118207fbc3eaba53`.
- **Not** overnight capped p27 (`c1-shd-cal-eb3cb5d93417a638` / h256) remassage.
- **Not** proto-135 5-class exploratory sweep.
- **Not** neuromorphic hardware SOTA without compute / substrate disclosure.
- SuperSpike here = feed-forward reverse-mode BPTT with SuperSpike surrogate — disclose wall time and feasibility; do not claim biology.

## Compute disclosure

- wall_time_s: 16016.6
- n_train / n_test: 8156 / 2264
- seeds × epochs × arms: 5 × 20 × 4
- feasibility: feed-forward SuperSpike BPTT is O(T·H·N_IN) per example; full official splits (8156/2264) are runnable on a workstation CPU with multi-hour wall time — disclose this number; do not claim free SOTA.
