# Exact-forward credit sparsity-calibrated isolation (`c1x-iso-s-*`)

Authority: Rust sources + on-disk notes from the commands below. Does **not** reopen
protocol-v2 `c1-118207fbc3eaba53`, frozen non-isolated `c1x-*`, or prior
`c1x-iso-*` (historical INVALID_HARNESS). **G2 thresholds and the sparsity band
were not relaxed.**

---

## Diagnosis (why prior `c1x-iso-*` scientific was INVALID_HARNESS)

Activity sparsity is `mean(winners.len() / n_hidden)` on the epoch-matched
broadcast arm's test set (`runner_credit.rs`).

Legacy exact-forward k-WTA scores only membranes with `v > 0`. Under
**non-isolated** `c1x-*`, incomplete membrane reset leaves dendrite residue (H2),
so nearly every trial has positive scores → sparsity ≈ nominal `k/N = 2/128 =
0.0156`.

Under trial isolation, `reset_c1_dynamic_state` clears all `v` / `v_dend` /
`last`. After multi-epoch weight updates, many trials have **no** positive
membrane → empty winner sets → scientific mean sparsity **0.0031** (below
`[0.005, 0.03]`) → **INVALID_HARNESS**. Quick `c1x-iso-*` stayed in-band only
because the short schedule kept more positive scores.

Changing `n_hidden` / `k_wta` while keeping `k/N` fixed cannot fix an ~80% empty
rate. Calibration must restore a reliable winner floor without widening the band.

---

## Calibration (architecture knob, not threshold massage)

| Knob | Prior `c1x-iso-*` | Calibrated `c1x-iso-s-*` |
|---|---|---|
| Hash prefix | `c1x-iso-` | `c1x-iso-s-` |
| Experiment | `c1x-iso-credit-reprereg` | `c1x-iso-s-credit-reprereg` |
| Arm protocol versions | base + 10 (14…17) | base + 20 (24…27) |
| Trial isolation (pairing + membrane) | ON | ON |
| k-WTA score gate | `v > 0` only | all finite `v` (winner floor) |
| `n_hidden` / `k_wta` / `p_sparse` | 128 / 2 / 0.35 | unchanged |
| `g2_min_gap_closed` / `g2_min_accuracy` / sparsity band | unchanged | unchanged |

CLI: `cargo run --locked --release -p binn-lab --bin credit-assignment -- --isolation-calibrated …`

---

## Minted scientific hashes (`c1x-iso-s-*`, n=20)

| Arm | Protocol | Hash |
|---|---:|---|
| broadcast-one-pass | 24 | `c1x-iso-s-broadcast-one-pass-6abe723b6700113c` |
| broadcast-epoch-matched | 24 | `c1x-iso-s-broadcast-epoch-matched-4e3236f8f60433d0` |
| rpe-three-factor | 25 | `c1x-iso-s-rpe-three-factor-e1fd914d40873269` |
| eprop-exact-forward | 26 | `c1x-iso-s-eprop-exact-forward-552924e96f2dded4` |
| dfa-exact-forward | 27 | `c1x-iso-s-dfa-exact-forward-d2c8d3c929a68bd2` |
| surrogate-gradient-exact-forward | 24 | `c1x-iso-s-surrogate-gradient-exact-forward-75f280fac365d671` |
| dense-epoch-matched | 24 | `c1x-iso-s-dense-epoch-matched-1f81769d0d7623b0` |

Quick/PILOT hashes: see [`credit_assignment_iso_s_quick.md`](credit_assignment_iso_s_quick.md).

Prior historical INVALID_HARNESS (`c1x-iso-*`) hashes remain evidence only — not the calibrated object.

---

## Commands

```bash
# From binn/
cargo test --locked -p binn-learn -p binn-lab --lib

cargo run --locked --release -p binn-lab --bin credit-assignment -- \
  --isolation-calibrated --quick --out results/credit_assignment_iso_s_quick.md

cargo run --locked --release -p binn-lab --bin credit-assignment -- \
  --isolation-calibrated --out results/credit_assignment_iso_s.md
```

---

## Outcomes

### Quick / PILOT — [`credit_assignment_iso_s_quick.md`](credit_assignment_iso_s_quick.md)

- Harness valid: PC **1.0000**, sparsity **0.0156** (in band)
- All arms **PILOT** (n=5); parity PASS
- Not a scientific verdict

### Scientific n=20 — [`credit_assignment_iso_s.md`](credit_assignment_iso_s.md)

- Positive control **0.9875** (≥ 0.9000) — OK
- Activity sparsity **0.0156** — **inside** `[0.0050, 0.0300]`
- Exact-forward parity **PASS**
- **Valid G2 harness** — all arms **FAIL** (gap LCB 0.0000; accuracy floor not jointly cleared)
- G2 thresholds were **not** relaxed

| Arm | Mean acc | Gap LCB | Verdict |
|---|---:|---:|---|
| broadcast-one-pass | 0.6563 | 0.0000 | **FAIL** |
| broadcast-epoch-matched | 0.6650 | 0.0000 | **FAIL** |
| rpe-three-factor | 0.6175 | 0.0000 | **FAIL** |
| eprop-exact-forward | 0.7138 | 0.0000 | **FAIL** |
| dfa-exact-forward | 0.6738 | 0.0000 | **FAIL** |
| surrogate-gradient-exact-forward | 0.5700 | 0.0000 | **FAIL** |
| dense-epoch-matched | 1.0000 | 0.0000 | **FAIL** |

Interpretation: under sparsity-calibrated trial-isolated exact-forward credit,
the suite is a **valid** G2-style decision object and every arm **FAIL**s the
unchanged gap/accuracy bars. This does not rescue frozen `c1x-*` FAILs or
reinterpret prior `c1x-iso-*` INVALID_HARNESS runs.

---

## Residual risks

1. Winner-floor k-WTA (all finite membranes) is a protocol choice; disclose when citing `c1x-iso-s-*`.
2. Isolation fixes pairing + membrane; θ=∞ mute-hidden + hybrid e-prop/DFA naming remain.
3. Quick `c1x-iso-s-*` hashes are pilot-only.
4. Prior `c1x-iso-*` scientific remains INVALID_HARNESS (sparsity); do not cite as G2 FAIL/PASS.
