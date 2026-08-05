# BINN SHD calibration (C1-SHD-CAL)

**claim_axis:** Standard-benchmark calibration
**object_under_test:** Multiclass passthrough-spike LIF under local credit rules
**may_claim:** Software-harness calibration vs chance (1/20) with e-prop ceiling
**must_not_claim:** Gate G2; neuromorphic SOTA; drop-in SuperSpike match; “local learning impossible”; full-corpus SHD SOTA when max_train/max_test caps apply

- schedule: **SCIENTIFIC**
- config hash: `c1-shd-cal-eb3cb5d93417a638`
- protocol version: 27
- seeds: 5
- dims: N_IN=700, T=100, n_classes=20 (chance=0.0500)
- subset: n_train=2000, n_test=500 (caps max_train=2000, max_test=500)
- hidden / epochs / lr: 128 / 20 / 0.0200
- fixture: false
- note: Full SHD cache loaded; evaluation uses capped subsets (n_train=2000, n_test=500; caps max_train=2000, max_test=500). Calibration only — not full-corpus SOTA. Ceiling = true e-prop; full SuperSpike BPTT infeasible at this scale in this crate.

## Results

| arm | mean accuracy |
|---|---:|
| `SHD_BROADCAST_PM1` | 0.0544 |
| `SHD_DFA` | 0.2336 |
| `SHD_RL_REINFORCE_FB` (REINFORCE×B) | 0.0532 |
| `SHD_EPROP_CEILING` (ceiling) | 0.0920 |
| chance (1/20) | 0.0500 |

**Ceiling disclosure:** true surrogate e-prop / truncated local BPTT analogue. Full SuperSpike BPTT on SHD-scale `(N_IN≈700, T≈100+, 20-way)` is infeasible in this hand-rolled crate; do not read the ceiling as matched SuperSpike.

## Per-seed

| seed | broadcast_pm1 | dfa | rl_reinforce_fb | eprop_ceiling |
|---:|---:|---:|---:|---:|
| 11400784312508578836 | 0.0560 | 0.1800 | 0.0500 | 0.0880 |
| 4354473029077956651 | 0.0520 | 0.2620 | 0.0540 | 0.0880 |
| 15755469980507731006 | 0.0540 | 0.3000 | 0.0500 | 0.0880 |
| 8709160896129724501 | 0.0560 | 0.2100 | 0.0620 | 0.0940 |
| 1663413533332040808 | 0.0540 | 0.2160 | 0.0500 | 0.1020 |

## Reproduce

```bash
cargo run --locked --release -p binn-lab --bin c1 -- --shd-cal --quick \
--out results/c1_shd.md
# Full SHD (after offline convert into data/shd/train.bin + test.bin):
cargo run --locked --release -p binn-lab --bin c1 -- --shd-cal \
--out results/c1_shd.md
```
