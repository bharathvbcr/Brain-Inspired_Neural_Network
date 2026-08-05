# BINN SHD full-corpus + SuperSpike ceiling (C1-SHD-FULL)

**claim_axis:** Standard-benchmark calibration
**object_under_test:** Multiclass passthrough-spike LIF under local credit rules
**may_claim:** Software-harness calibration vs chance (1/20) with disclosed ceiling
**must_not_claim:** Gate G2; neuromorphic SOTA; Zenke SuperSpike drop-in on recurrent nets; “local learning impossible”; overnight p27 e-prop ceiling reinterpretation; proto-135 5-class sweep mix-in

- schedule: **SCIENTIFIC**
- config hash: `c1-shd-full-a9542a730cb22c74`
- protocol version: 29
- seeds: 2
- dims: N_IN=700, T=100, n_classes=20 (chance=0.0500)
- subset: n_train=400, n_test=100 (caps max_train=400, max_test=100; 0=uncapped)
- hidden / epochs / lr: 128 / 8 / 0.0200
- fixture: false
- note: SHD cache loaded; evaluation uses capped subsets (n_train=400, n_test=100; caps max_train=400, max_test=100). Protocol-29 SuperSpike path (subset). Ceiling = true SuperSpike BPTT; not full-corpus SOTA under caps. Not Gate G2.

## Results

| arm | mean accuracy |
|---|---:|
| `SHD_BROADCAST_PM1` | 0.0650 |
| `SHD_DFA` | 0.0750 |
| `SHD_RL_REINFORCE_FB` (REINFORCE×B) | 0.0400 |
| `SHD_SUPERSPIKE_CEILING` (ceiling) | 0.1250 |
| chance (1/20) | 0.0500 |

**Ceiling disclosure:** true SuperSpike reverse-mode BPTT on the same feed-forward hard-reset LIF used by the local arms (no `W_rec`). Surrogate `σ'(u)=1/(1+β|u|)²`; hard reset cuts the membrane adjoint. This is the nearest feasible BPTT ceiling at SHD scale in this crate — **not** a Zenke SuperSpike drop-in on a recurrent net, and **not** the overnight capped e-prop ceiling (~0.09–0.10 under p27 2000/500).

## Per-seed

| seed | broadcast_pm1 | dfa | rl_reinforce_fb | superspike_ceiling |
|---:|---:|---:|---:|---:|
| 11400784312163576852 | 0.0800 | 0.0600 | 0.0500 | 0.1400 |
| 4354473028747634731 | 0.0500 | 0.0900 | 0.0300 | 0.1100 |

## Reproduce

```bash
# Rust SHD convert (no Python / h5py):
PKG_CONFIG_PATH="$(brew --prefix hdf5)/lib/pkgconfig:${PKG_CONFIG_PATH:-}" \
cargo run --locked --release -p binn-data --features shd-convert --bin convert-shd -- \
--cache-dir data/shd
cargo run --locked --release -p binn-lab --bin c1 -- --shd-full --smoke \
--out results/c1_shd_full_smoke.md
# hash: c1-shd-full-a9542a730cb22c74
```

## Non-claims

- **Not Gate G2** and does not reopen `c1-118207fbc3eaba53`.
- **Not** overnight capped p27 (`c1-shd-cal-eb3cb5d93417a638` / h256) remassage.
- **Not** proto-135 5-class exploratory sweep.
- **Not** neuromorphic hardware SOTA without compute / substrate disclosure.
- SuperSpike here = feed-forward reverse-mode BPTT with SuperSpike surrogate — disclose wall time and feasibility; do not claim biology.

## Compute disclosure

- wall_time_s: 129.7
- n_train / n_test: 400 / 100
- seeds × epochs × arms: 2 × 8 × 4
- feasibility: feed-forward SuperSpike BPTT is O(T·H·N_IN) per example; full official splits (8156/2264) are runnable on a workstation CPU with multi-hour wall time — disclose this number; do not claim free SOTA.
