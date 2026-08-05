# Matched-Architecture — live C1 `ReinforceFeedback` (protocol v13)

**Date:** 2026-07-23 · **Status:** scientific schedule run — **FAIL**
**Config hash (scientific):** `c1-660401d74db3c88d`
**Config hash (quick):** `c1-a57975f13b73a599`
**Does NOT** reopen `c1-118207fbc3eaba53`, remassage P4 spiking-DFA, retune P5
`rl_graded`, or flip default C1 off broadcast ±1.

---

## 1. Hypothesis (post-P8)

Protocol v12 PASSed `rl_reinforce_fb` on the **dense-LIF matched** harness.
P8 exposed the same family as production `ReinforceFeedback` + `reinforce_term`
without changing default C1. The open question is whether that neuromodulator
helps on the **live** C1 path (k-WTA / single-pass / muted-θ integrate), where
broadcast ±1 historically fails G2.

**H:** Replacing broadcast ±1 with sampled directional REINFORCE × frozen
per-neuron `B_i` on the live engine closes more of the dense→gradient gap than
broadcast, because per-post credit breaks scalar-modulator homogenization under
sparse k-WTA winners.

## 2. Protocol (v13)

| Knob | Value |
|---|---|
| Substrate | Identical to protocol-v2 C1 (mute θ, membrane k-WTA, incomplete H2 reset) |
| Master seed | Same lineage as v2 (`0xC160_0000_0001`) — A/B isolates neuromodulator |
| Main-arm plasticity | `ReinforceFeedback::credit(reinforce_term(r, a, p))` via `update_with_credit_counted` |
| Action | Bernoulli sample from soft readout policy `σ(charge_1 − charge_0)` |
| Eval | Greedy charge argmax (unchanged) |
| Positive control | **Broadcast ±1** + disclosed longer easy-PC schedule (substrate check; floors unchanged) |
| G2 floors | Unchanged (gap LCB > 0.5, acc ≥ 0.65, PC ≥ 0.90) |

## 3. Gate

Same Gate G2 thresholds as canonical C1. Verdict language: PASS / FAIL / PILOT /
INVALID_HARNESS.

## 4. Results

### Quick (PILOT)

See [`c1_rfb_quick.md`](c1_rfb_quick.md):

| Metric | Value |
|---|---:|
| Hash | `c1-a57975f13b73a599` |
| local-assembly | 0.6000 |
| dense-local | 0.4625 |
| gradient-reference | 0.6500 |
| gap LCB | −0.0801 |
| positive_control | 1.0000 |
| **Verdict** | **PILOT** (n_seeds=5 < 20) |

### Scientific — **FAIL**

See [`c1_rfb.md`](c1_rfb.md):

| Metric | Value |
|---|---:|
| Hash | `c1-660401d74db3c88d` |
| local-assembly | **0.4900** |
| dense-local | 0.4700 |
| gradient-reference | 0.8938 |
| gap_closed mean / LCB | 0.2135 / **0.0737** |
| positive_control | 0.9488 |
| **Verdict** | **FAIL** |

**Reading:** Valid harness. The v12 family neuromodulator does **not** transfer
to live k-WTA / single-pass C1 under this honest mapping (sampled REINFORCE ×
frozen `B_i`). Do not claim live-engine rescue from the matched dense-LIF PASS.
Stop rule: no remassage without a new hypothesis + new hash.

## 5. Reproduce

```bash
cargo test -p binn-lab --lib reinforce_fb
cargo run --locked --release -p binn-lab --bin c1 -- --reinforce-fb --quick \
  --out results/c1_rfb_quick.md
cargo run --locked --release -p binn-lab --bin c1 -- --reinforce-fb \
  --out results/c1_rfb.md
# exact scientific replay (flag + hash, matched-* style):
cargo run --locked --release -p binn-lab --bin c1 -- --reinforce-fb \
  --config-hash c1-660401d74db3c88d --out results/c1_rfb_replay.md
# scientific hash: c1-660401d74db3c88d
```

Code: `binn-lab/src/config.rs` (`c1_reinforce_fb*`, protocol 13),
`binn-lab/src/runner.rs` (opt-in live path), `binn-lab/experiments/c1.rs`
(`--reinforce-fb`), production API in `binn-learn/src/credit.rs`.

## 6. Explicit non-claims / stop rules

- Default C1 path remains broadcast ±1 (`c1-118207fbc3eaba53` untouched).
- Not a P4 spiking-DFA remassage.
- Not a P5 `rl_graded` retune.
- One honest live-engine attempt done (P9 FAIL) — do not retune knobs in-place.
- Locality evidence to cite remains 1-layer XOR / `xor_thresh` (P7), not this live C1 run.
