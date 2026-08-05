# Exact-forward credit trial isolation (`c1x-iso-*`)

Authority: Rust sources + on-disk notes from the commands below. Does **not** reopen
protocol-v2 `c1-118207fbc3eaba53` or frozen non-isolated `c1x-*` hashes. G2
thresholds are unchanged.

---

## Protocol

| Knob | Non-isolated (frozen) | Isolated (new) |
|---|---|---|
| Hash prefix | `c1x-` | `c1x-iso-` |
| Experiment | `c1-credit-reprereg` | `c1x-iso-credit-reprereg` |
| Arm protocol versions | 4 / 5 / 6 / 7 | 14 / 15 / 16 / 17 (= base + 10) |
| `ThreeFactor.last_spike` | sticky across trials (H1) | cleared each trial |
| Membrane reset | incomplete hidden `v`+θ (H2) | `reset_c1_dynamic_state` (all cells) |
| G2 thresholds | unchanged | unchanged |

CLI: `cargo run --locked --release -p binn-lab --bin credit-assignment -- --isolation …`

---

## Minted scientific hashes (`c1x-iso-*`, n=20)

| Arm | Protocol | Hash |
|---|---:|---|
| broadcast-one-pass | 14 | `c1x-iso-broadcast-one-pass-4265d41a3ecad902` |
| broadcast-epoch-matched | 14 | `c1x-iso-broadcast-epoch-matched-7becb435b63868c6` |
| rpe-three-factor | 15 | `c1x-iso-rpe-three-factor-7be2092f12f5a653` |
| eprop-exact-forward | 16 | `c1x-iso-eprop-exact-forward-1c2d2e8835df30ca` |
| dfa-exact-forward | 17 | `c1x-iso-dfa-exact-forward-3fd0919313abbe04` |
| surrogate-gradient-exact-forward | 14 | `c1x-iso-surrogate-gradient-exact-forward-28adc822a3a572d3` |
| dense-epoch-matched | 14 | `c1x-iso-dense-epoch-matched-376a3ba027ad5bba` |

Quick/PILOT hashes (cite only as pilot): see [`credit_assignment_iso_quick.md`](credit_assignment_iso_quick.md).

Frozen non-isolated scientific hashes remain:

- `c1x-broadcast-one-pass-ec3c5a4d19ccd57e`
- `c1x-broadcast-epoch-matched-911a03a2a45feaf2`
- `c1x-rpe-three-factor-872e9eda9303f5df`
- `c1x-eprop-exact-forward-fcedc76a80ff0f0e`
- `c1x-dfa-exact-forward-4a1601e725edbc80`
- `c1x-surrogate-gradient-exact-forward-cfe9a2c8d3e22257`
- `c1x-dense-epoch-matched-1387104803fe7e0a`

---

## Commands

```bash
# From binn/
cargo test --locked -p binn-learn -p binn-lab --lib

cargo run --locked --release -p binn-lab --bin credit-assignment -- \
  --isolation --quick --out results/credit_assignment_iso_quick.md

cargo run --locked --release -p binn-lab --bin credit-assignment -- \
  --isolation --out results/credit_assignment_iso.md
```

---

## Outcomes

### Quick / PILOT — [`credit_assignment_iso_quick.md`](credit_assignment_iso_quick.md)

- Harness valid: PC **0.9417**, sparsity **0.0125** (in band)
- All arms **PILOT** (n=5); parity PASS
- Not a scientific verdict

### Scientific n=20 — [`credit_assignment_iso.md`](credit_assignment_iso.md)

- Positive control **0.9488** (≥ 0.9000) — OK
- Activity sparsity **0.0031** — **below** preregistered band `[0.0050, 0.0300]`
- Exact-forward parity **PASS**
- All arms **INVALID_HARNESS** (sparsity gate; G2 thresholds were **not** relaxed)
- Accuracies still below a clearable G2-style bar even if sparsity were ignored (gap LCBs ≪ 0.5; e.g. matched broadcast mean **0.4250**)

Interpretation: under trial-isolated exact-forward credit, this scientific schedule does **not** produce a valid G2-style decision object because sparsity falls out of band. That is a pipeline/integrity outcome under the new protocol — not a rescue of frozen `c1x-*` FAILs, and not a reason to massage thresholds.

**Follow-up:** sparsity-calibrated isolation (`c1x-iso-s-*`) restores a valid harness without relaxing G2 thresholds — see [`credit_assignment_iso_sparsity.md`](credit_assignment_iso_sparsity.md).

---

## Residual risks

1. Scientific `c1x-iso-*` is **INVALID_HARNESS** on sparsity; do not cite arm accuracies as G2 FAIL/PASS.
2. Isolation fixes pairing + membrane only; θ=∞ mute-hidden + hybrid e-prop/DFA naming remain.
3. Quick `c1x-iso-*` hashes are pilot-only.
4. Non-isolated `c1x-*` still carry H1/H2; disclose when citing those hashes.
