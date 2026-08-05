# Gap-close protocols — live RFB transfer (v14–v19)

**Date:** 2026-07-23/24 · **Status:** scientific schedules complete (v14–v19)  
**Does not** remassage P9 v13 (`c1-660401d74db3c88d`), P4 spike-DFA, or P5 `rl_graded`.

## Gap being closed

Matched dense-LIF `rl_reinforce_fb` / DFA **PASS**; live k-WTA v13 RFB **FAIL** (~0.49).
Separable hypotheses for the transfer gap:

| Protocol | Hash family | Hypothesis | Sci verdict |
|---|---|---|---|
| **v14** `c1-rfb-em` | epoch-matched live RFB | Single-pass exposure is the handicap; multi-epoch with the **same** random `B` closes more gap. | **FAIL** local 0.4838 / gap LCB −0.0100 |
| **v15** `c1-sfb` | structured frozen B | Random `B_i` under k-WTA is the bottleneck; `B_i = sign(w→r1 − w→r0)` aligns feedback with decoder columns. | **FAIL** local **0.7262** (clears floor) / gap LCB **0.2567** |
| **v16** `c1-sfb-em` | structured × epoch | v15 `B` + v14 exposure closes remaining gap. | **FAIL** local 0.5200 / gap LCB 0.0844 — epochs **hurt** structured B |
| **v17** `c1-sfb-cap` | structured × capacity | v15 `B` on richer k/N/train substrate closes gap vs dense/grad. | **FAIL** local **0.6825** (floor ✓) / gap LCB **0.3127** — best gap LCB |
| **v18** `c1-elig-rfb` | eligibility × REINFORCE | STDP eligibility mistimed vs terminal REINFORCE; longer `τ_e=160` + mid-trial absorb before action. | **FAIL** local **0.7125** (floor ✓) / gap LCB 0.2351 — no gain vs v15 |
| **v19** `c1-sfb-teach` | structured B × target teach | RFB-family observe-only on incorrect trials drops default C1 teach; restore `credit(+1)` through structured `B`. | **FAIL** local **0.6700** (floor ✓) / gap LCB 0.2238 — **no gain vs v15** (slightly worse) |

## Reproduce

```bash
cargo test -p binn-lab --lib rfb_epoch
cargo run --locked --release -p binn-lab --bin c1 -- --rfb-epoch --out results/c1_rfb_em.md
cargo run --locked --release -p binn-lab --bin c1 -- --structured-fb --out results/c1_sfb.md
cargo run --locked --release -p binn-lab --bin c1 -- --structured-fb-epoch --out results/c1_sfb_em.md
cargo run --locked --release -p binn-lab --bin c1 -- --structured-fb-capacity --out results/c1_sfb_cap.md
cargo run --locked --release -p binn-lab --bin c1 -- --elig-rfb --out results/c1_elig_rfb.md
cargo run --locked --release -p binn-lab --bin c1 -- --structured-fb-teach --out results/c1_sfb_teach.md
```

Frozen scientific hashes:

- v14 sci: `c1-714c115e14a3eeed` · quick: `c1-dd9071dd8eb68679`
- v15 sci: `c1-493ddd56f8714fb6` · quick: `c1-a6f182c8af2b0faa`
- v16 sci: `c1-677df7f7cbe4f8ec` · quick: `c1-e8722ed71f60d5d4`
- v17 sci: `c1-983ee5303c00b147` · quick: `c1-b72fb5d90427b358`
- v18 sci: `c1-c7d2c86a2b1927f6` · quick: `c1-ce55c43d56f4746a`
- v19 sci: `c1-dfab4a7ec19f17c2` · quick: `c1-b1f3abcaee5650b6`

## Scientific summary (n=20)

| Arm | local | dense | grad | gap mean | gap LCB | Verdict |
|---|---:|---:|---:|---:|---:|---|
| v13 rfb (baseline) | 0.4900 | — | — | — | 0.0737 | FAIL |
| v14 rfb-em | 0.4838 | 0.5350 | 0.8938 | 0.1148 | −0.0100 | FAIL |
| v15 sfb | **0.7262** | 0.5025 | 0.8938 | **0.4690** | 0.2567 | FAIL (acc floor ✓) |
| v16 sfb-em | 0.5200 | 0.5113 | 0.8938 | 0.2231 | 0.0844 | FAIL |
| v17 sfb-cap | **0.6825** | 0.4400 | 0.9940 | 0.4670 | **0.3127** | FAIL (acc floor ✓) |
| v18 elig-rfb | **0.7125** | 0.4950 | 0.8938 | 0.4287 | 0.2351 | FAIL (acc floor ✓) |
| v19 sfb-teach | **0.6700** | 0.5088 | 0.8938 | 0.4120 | 0.2238 | FAIL (acc floor ✓) |
| capacity-only (Tier-B) | 0.6775 | — | — | — | 0.0000 | FAIL |

## Locked reading

1. **Exposure alone (v14) does not rescue** random-B live RFB.
2. **Structured `B` (v15) is the strongest accuracy lever** — clears the 0.65 floor (0.7262) with gap LCB 0.2567.
3. **Epoch remassage under structured B (v16) regresses** — do not stack epochs on v15 without a new hypothesis.
4. **Capacity × structured (v17) best gap LCB so far (0.3127)** while still clearing the floor — still short of G2 (needs >0.5).
5. **Eligibility × REINFORCE (v18) does not beat v15** — longer `τ_e` + mid-trial absorb is not the missing transfer ingredient under this honest mapping.
6. **Restored target teach (v19) does not beat v15** — secondary `credit(+1)` through structured `B` clears the floor (0.6700) but is slightly worse than observe-only v15; dropping default teach is not the transfer bottleneck.
7. Do **not** remassage v13–v19 knobs in place; next work is paper packaging or a *new* hypothesis + hash.

## Stop rules

- Do not retune v13–v19 knobs in place if these FAIL.
- G2 floors unchanged (gap LCB > 0.5 and local ≥ 0.65).
- Camp artifacts: `results/runs/2026-07-23-gap-close/` · verify/more: `results/runs/2026-07-23-more-results/`.
