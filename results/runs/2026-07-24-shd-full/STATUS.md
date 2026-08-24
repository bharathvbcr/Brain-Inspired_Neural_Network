# BINN SHD full-corpus + SuperSpike ceiling (protocol 29)

**claim_axis:** Standard-benchmark calibration  
**object_under_test:** Multiclass passthrough-spike LIF under local credit rules on official SHD splits  
**may_claim:** Software-harness arm table vs chance (1/20) with **true SuperSpike reverse-mode BPTT** ceiling and disclosed wall time  
**must_not_claim:** Gate G2; neuromorphic SOTA; Zenke SuperSpike drop-in on recurrent nets; overnight p27 e-prop ceiling (~0.09–0.10) reinterpretation; proto-135 5-class mix-in; biology

Frozen p27 hashes **untouched:** `c1-shd-cal-eb3cb5d93417a638` (h128), `c1-shd-cal-bafa6835d8de7eb8` (h256).

## Protocol freeze

| Preset | Hash | Splits | Seeds | Epochs | Ceiling |
|---|---|---|---:|---:|---|
| Full scientific | `c1-shd-full-2c93117075740ed0` | official **8156 / 2264** (uncapped) | 5 | 20 | SuperSpike BPTT |
| Path-proof smoke | `c1-shd-full-a9542a730cb22c74` | 400 / 100 | 2 | 8 | SuperSpike BPTT |
| Fixture quick | `c1-shd-full-1b53d4f8a6ac3d41` | fixture | 2 | 4 | SuperSpike BPTT |

CLI:

```bash
# Rust convert only (scripts/convert_shd.py retired):
PKG_CONFIG_PATH="$(brew --prefix hdf5)/lib/pkgconfig:${PKG_CONFIG_PATH:-}" \
  cargo run --locked --release -p binn-data --features shd-convert --bin convert-shd -- \
    --cache-dir data/shd

cargo run --locked --release -p binn-lab --bin c1 -- --shd-full --smoke \
  --out results/c1_shd_full_smoke.md
cargo run --locked --release -p binn-lab --bin c1 -- --shd-full \
  --out results/c1_shd_full.md
```

## Path-proof smoke (DONE)

Artifact: [`c1_shd_full_smoke.md`](c1_shd_full_smoke.md) · camp [`runs/2026-07-24-shd-full/c1_shd_full_smoke.md`](c1_shd_full_smoke.md)

| arm | mean accuracy |
|---|---:|
| `SHD_BROADCAST_PM1` | 0.0650 |
| `SHD_DFA` | 0.0750 |
| `SHD_RL_REINFORCE_FB` | 0.0400 |
| `SHD_SUPERSPIKE_CEILING` | **0.1250** |
| chance (1/20) | 0.0500 |

**Compute:** wall **129.7 s** · n_train=400 · n_test=100 · 2 seeds × 8 epochs × 4 arms.

**Path proof:** SuperSpike ceiling clears chance on real BINNSHD1 bins under protocol 29; local pm1 / RL×B stay near chance; DFA weakly above chance. Not full-corpus SOTA.

## Full official run (DONE — attempt 3)

Artifact: [`results/c1_shd_full.md`](c1_shd_full.md) · camp [`runs/2026-07-24-shd-full/c1_shd_full.md`](c1_shd_full.md)

| Field | Value |
|---|---|
| Hash | `c1-shd-full-2c93117075740ed0` |
| Attempt 1 | 2026-07-24T18:44:59Z · PID 35094 · died ~T+32m (agent shell teardown) |
| Attempt 2 | 2026-07-24T19:18:58Z · PID 56587 · died ~T+20s (same) |
| Attempt 3 start (UTC) | 2026-07-24T19:19:35Z |
| Attempt 3 PID | **56772** (double-fork daemon via `run_scientific.sh`) |
| Attempt 3 end (UTC) | 2026-07-24T23:46:32Z |
| Exit | **0** |
| Log | `results/runs/2026-07-24-shd-full/c1_shd_full_scientific.log` |
| Out | `results/c1_shd_full.md` |
| Wall | **16016.6 s** (~4.45 h; `time` real 16016.73) |

| arm | mean accuracy |
|---|---:|
| `SHD_BROADCAST_PM1` | 0.0513 |
| `SHD_DFA` | 0.3210 |
| `SHD_RL_REINFORCE_FB` (REINFORCE×B) | 0.0493 |
| `SHD_SUPERSPIKE_CEILING` (ceiling) | **0.4315** |
| chance (1/20) | 0.0500 |

**Compute:** wall **16016.6 s** · n_train=8156 · n_test=2264 · 5 seeds × 20 epochs × 4 arms · protocol 29 · hash `c1-shd-full-2c93117075740ed0`.

**Readout:** SuperSpike ceiling and DFA clear chance on full official splits; pm1 and RL×B stay near chance. Calibration / software-harness only — not Gate G2, not neuromorphic SOTA. Frozen p27 hashes untouched.

## Ceiling disclosure

True **SuperSpike reverse-mode BPTT** on the same feed-forward hard-reset LIF as the local arms (no `W_rec`). Surrogate `σ'(u)=1/(1+β|u|)²`; hard reset cuts the membrane adjoint (`du[t]=ds·σ'+α·du[t+1]·(1−s[t])`).

This is the **nearest feasible BPTT ceiling** at SHD scale in this hand-rolled crate — **not** Zenke SuperSpike on a recurrent net, and **not** the overnight capped e-prop ceiling.

## Non-claims

- Not Gate G2 / does not reopen `c1-118207fbc3eaba53`.
- Not remassage of frozen p27 overnight SHD.
- Not proto-135 5-class exploratory sweep.
- Not neuromorphic hardware SOTA without compute / substrate disclosure.
- Do not claim biology.

## Convert path

End-to-end Rust: `binn-data` feature `shd-convert` → `convert-shd`. `scripts/convert_shd.py` exits 2 with redirect. See [`data/shd/README.md`](../../../data/shd/README.md).
