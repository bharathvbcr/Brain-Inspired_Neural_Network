# Matched-Architecture — product neuromodulator (v12 family)

**Date:** 2026-07-23 · **Status:** production API wired (not a new G2 gate)
**Family:** directional REINFORCE × per-neuron frozen feedback (`rl_reinforce_fb`)
**Does NOT** reopen `c1-118207fbc3eaba53`, retune failed v11 `rl_graded`, remassage
spiking DFA (P4 stop), or flip default C1 off broadcast ±1.

---

## What this closes

Protocol v12 PASSed on the dense-LIF matched harness with primary
`rl_reinforce_fb` (`c1-rl-42eddc9c801308e9`: fb 0.9200 / gap LCB 0.6846). The
optional product follow-up was to expose that **same** neuromodulator family on
the production learning path (`three_factor` + `CreditSignal`), without claiming a
new spiking-path G2 pass.

## Production API

| Symbol | Role |
|---|---|
| `reinforce_term(r, a, p)` | directional scalar `r·(a−p)` |
| `ReinforceFeedback` | frozen `B_i ∼ Uniform[-1,1]` (seed mix `0x00FB_A0C1_ED17`) |
| `ReinforceFeedback::credit(directional)` | per-post `B_i · directional` → `PostSynapticCredit` |
| `ThreeFactor::update_with_credit_counted` | applies per-neuron gate (already present) |

Default C1 still uses broadcast `Modulators::reward(±1)`. Callers opt into the
v12 family explicitly:

```rust
let fb = ReinforceFeedback::new(n_cells, seed);
let m = reinforce_term(reward, action, policy);
learner.update_with_credit_counted(engine, &fb.credit(m));
```

`MatchedRlReinforceFb` now draws `B_i` via production `ReinforceFeedback` (same
lineage; v12 hashes unchanged).

## Verification

| Check | Result |
|---|---|
| Unit: `reinforce_term` / frozen `B` / credit routing | PASS (`binn-learn` `credit::tests`) |
| Unit: ThreeFactor opposite-sign `B` ⇒ opposite `Δw` | PASS (`reinforce_feedback_modulator_differentiates_posts`) |
| Unit: matched arm `B` ≡ product `ReinforceFeedback` | PASS |
| Quick matched-rl (parity smoke) | `c1-rl-9a41bc8f876617a5` · PILOT · fb 0.8000 (same quick hash as prior) |
| Scientific v12 | **unchanged** — `c1-rl-42eddc9c801308e9` PASS (not re-gated; B lineage preserved) |

Reproduce:

```bash
cargo test -p binn-learn --lib reinforce
cargo run --locked --release -p binn-lab --bin c1 -- --matched-rl --quick \
  --out results/c1_rl_product_neuromod_quick.md
```

Code: `binn-learn/src/credit.rs` (`ReinforceFeedback`, `reinforce_term`),
`modulators.rs` (docs), `matched_rl_baseline.rs` (uses product `B`),
`three_factor.rs` (integration test).

## Live C1 follow-up (P9)

Opt-in live engine path under protocol v13 (`--reinforce-fb`, hash
`c1-660401d74db3c88d`) **FAIL**ed G2. See
[`MATCHED_ARCH_LIVE_REINFORCE.md`](MATCHED_ARCH_LIVE_REINFORCE.md). Default C1
±1 path unchanged.

## Explicit non-claims

- Not a flip of production C1 default neuromodulator.
- Not a spiking-substrate rescue (P4 stop still holds).
- Not a retune of `rl_graded` (P5 stop still holds).
- Locality to cite remains 1-layer XOR / `xor_thresh`, not 2-layer depth (P7).
