# BINN hard-research loop — campaign note

**Started:** 2026-07-24T02:51Z · **Closed:** 2026-07-24T03:04Z
**Camp:** `results/runs/2026-07-23-binn-hard-loop/`
**Claim freeze:** [`CAMPAIGN_2026-07-23_CLAIM_FREEZE.md`](../../CAMPAIGN_2026-07-23_CLAIM_FREEZE.md) · camp copy `wave3/CLAIM_FREEZE.md`
**Guardrails:** do not remassage closed P4 / P5 / P9; do not reopen `c1-118207fbc3eaba53`.

---

## Prior locked state (pre-campaign)

| Protocol | Hash | Verdict | Mean (primary) | Gap LCB |
|---|---|---|---:|---:|
| v2 C1 / G2 | `c1-118207fbc3eaba53` | **FAIL** | local ~chance | — |
| v4 matched broadcast | `c1-match-5dc6822e71229e9e` | **FAIL** | local 0.5000 / grad 0.8963 | 0.0000 |
| v5 matched DFA | `c1-dfa-c8c4fe0899908b84` | **PASS** | DFA 0.9387 / grad 0.8963 | 0.6894 |
| v10 spiking DFA | `c1x-dfa-spike-true-dfa-a911e793e590b0ed` | **FAIL** | true-dfa 0.6513 | 0.0733 |
| v11 RL graded primary | `c1-rl-ef504db58916720d` | **FAIL** | graded 0.5900 | 0.0182 |
| v12 RL reinforce_fb | `c1-rl-42eddc9c801308e9` | **PASS** | fb 0.9200 / grad 0.8887 | 0.6846 |
| v13 live ReinforceFeedback | `c1-660401d74db3c88d` | **FAIL** | local 0.4900 | 0.0737 |
| Hybrid H0 | `binn-hybrid-h0-v3-caedeec1a47475a5` | **HYBRID_NO_GO** | teacher D*=2 | — |

**Thesis (from `PUBLISHABLE_CLAIMS.md`):** broadcast-scalar three-factor fails a matched dense-LIF gate; live k-WTA C1 is a softer operationalized pipeline negative. Do not claim biology / impossibility / AC PASS.

---

## Wave 1 harvest (complete)

| Trial | Verdict / key numbers |
|---|---|
| match / DFA / RL hash replays | **FAIL** / **PASS** / **PASS** (exact) |
| RFB flag+hash | originally CLI-refused; fixed — see Fixes below |
| sens temporal-pc / capacity quick | **PILOT** (gap far from bar) |
| iso / project / credit / dfa-spike quick | PILOT / INVALID_HARNESS / PILOT / PILOT |
| `xor_thresh` | broadcast **0.501** / DFA **0.827** — locality flip |
| `depth_locality` mid | broadcast **0.816** / DFA **0.825** — not a locality flip |

## Wave 2 harvest (complete)

| Trial | Hash | Verdict | Key numbers |
|---|---|---|---|
| Live RFB replay | `c1-660401d74db3c88d` | **FAIL** | local 0.4900 / gap LCB **0.0737** |
| Capacity scientific | `c1-d38d7644d8afc84b` | **FAIL** | local **0.6775** (floor ✓) / gap LCB **0.0000** |
| True e-prop | `c1x-eprop-true-…0e2aeb90d68ac5f9` | methods | true-surrogate **0.7125** |
| Canonical v2 C1 | `c1-118207fbc3eaba53` | **FAIL** | local 0.4912 / gap LCB −0.0048 |

## Wave 3 harvest (complete)

| Trial | Hash | Verdict | Key numbers |
|---|---|---|---|
| Temporal-PC scientific | `c1-a49deeaedb495a09` | **FAIL** | local 0.5263 / gap mean 0.0947 |
| Isolation scientific | `c1-8ec031907a3426d0` | **FAIL** | local 0.5188 / gap mean 0.2109 |
| Claim freeze | — | written | durable + camp copies |

**Campaign stop:** integrity matrix closed. Capacity is the only live arm that clears the accuracy floor; no Tier-B arm closes the gap gate.

---

## Gap-close wave (v14–v19) — 2026-07-23/24

Camp: `results/runs/2026-07-23-gap-close/` · note: `results/GAP_CLOSE_RFB_TRANSFER.md`  
Verify/more: `results/runs/2026-07-23-more-results/` (v14/16/17/18/cap/iso bit-stable; v19 sci minted)

| Protocol | Hash | Verdict | Local | Gap LCB |
|---|---|---|---:|---:|
| v14 `--rfb-epoch` | `c1-714c115e14a3eeed` | **FAIL** | 0.4838 | −0.0100 |
| v15 `--structured-fb` | `c1-493ddd56f8714fb6` | **FAIL** | **0.7262** | 0.2567 |
| v16 `--structured-fb-epoch` | `c1-677df7f7cbe4f8ec` | **FAIL** | 0.5200 | 0.0844 |
| v17 `--structured-fb-capacity` | `c1-983ee5303c00b147` | **FAIL** | **0.6825** | **0.3127** |
| v18 `--elig-rfb` | `c1-c7d2c86a2b1927f6` | **FAIL** | **0.7125** | 0.2351 |
| v19 `--structured-fb-teach` | `c1-dfab4a7ec19f17c2` | **FAIL** | **0.6700** | 0.2238 |

**Reading:** structured `B` (v15) is the accuracy lever; capacity×structured (v17) is the best gap LCB so far still short of G2; epochs alone / epochs under structured B / eligibility co-design / restored target teach do not close the gate.

---

## Fixes applied post-campaign

1. **`c1` CLI:** `--reinforce-fb` / `--isolation` / `--sensitivity` / `--spike*` / `--project` / gap-close flags may now pair with `--config-hash` (matched-* style). Wrong-family hashes exit 2 with a clear refuse message.
2. Docs: `MATCHED_ARCH_LIVE_REINFORCE.md`, `MATCHED_ARCH_NEXT_PLAN.md`, `GAP_CLOSE_RFB_TRANSFER.md`.
3. Durable claim freeze: `results/CAMPAIGN_2026-07-23_CLAIM_FREEZE.md` (includes v14–v17).
4. Idle `/loop` heartbeats killed (campaign closed).

---

## Hard research synthesis

### A. What BINN shows

1. **Rule topology (clean):** On identical dense-LIF forward, ±1 broadcast three-factor stays at chance while SuperSpike BPTT succeeds. Graded DFA and REINFORCE×fixed-random feedback both clear the matched gate.
2. **Engine transfer (hard):** The same REINFORCE×`B_i` family **FAIL**s on live muted-θ / k-WTA C1. Spiking-path true DFA also fails. Hybrid diagnostics: hard winner discretization + sparse eligibility discard edge-specific terminal credit.
3. **Gap-close:** Aligning `B` with readout columns (v15) clears the accuracy floor; stacking epochs (v16) regresses; capacity×structured (v17) improves gap LCB vs capacity-only; eligibility×REINFORCE (v18) and restored target teach (v19) do not beat v15 — all still FAIL G2.

### B. Literature anchors

| Paper | Why it matters |
|---|---|
| Bellec et al. 2020 e-prop ([doi:10.1038/s41467-020-17236-y](https://doi.org/10.1038/s41467-020-17236-y)) | Online eligibility ≈ BPTT; BINN true-eprop is methods contrast, not broadcast rescue |
| Ororbia 2023 survey ([arXiv:2312.09257](http://arxiv.org/abs/2312.09257)) | Local-credit taxonomy |
| Hong et al. 2022 ([doi:10.1038/s41467-022-30827-1](https://doi.org/10.1038/s41467-022-30827-1)) | Neuromodulator × eligibility; biology richer than ±1 |
| Pawlak 2010 ([doi:10.3389/fnsyn.2010.00146](https://doi.org/10.3389/fnsyn.2010.00146)) | Three-factor STDP gate |
| Lillicrap 2016 FA ([doi:10.1038/ncomms13276](https://doi.org/10.1038/ncomms13276)) | Fixed random `B` teaching signals |
| Nøkland 2016 DFA ([arXiv:1609.01596](https://arxiv.org/abs/1609.01596)) | Direct output→hidden random feedback |
| Launay et al. 2019 ([arXiv:1906.04554](https://arxiv.org/abs/1906.04554)) | Narrow-layer DFA bottleneck ↔ sparse k-WTA |

### C. Unified reading

```
signal richness ──► ±1 flat  <  graded/RPE  <  supervised error / REINFORCE×(a−p)
credit locality ──► broadcast OK on linear tasks; DFA needed on XOR
substrate gap   ──► dense continuous PASS ≠ hard k-WTA / sparse eligibility PASS
B structure     ──► random B FAIL; structured B clears acc floor; still gap-short of G2
```

### D. Next (new hash required)

1. ~~Structured / learned feedback `B` under k-WTA~~ → v15 **FAIL** (floor ✓)
2. ~~Two-pass / epoch-matched credit on live C1~~ → v14 **FAIL**; v16 combo **FAIL**
3. ~~Structured × capacity~~ → v17 **FAIL** (best gap LCB 0.3127)
4. ~~Eligibility timing co-designed with REINFORCE~~ → v18 **FAIL** (no gain vs v15)
5. ~~Structured B × restored target teach~~ → v19 **FAIL** (local 0.6700, LCB 0.2238; no gain vs v15)
6. ~~Paper packaging of P1–P9 + gap-close negatives + hybrid NO-GO~~ → hardened package:
   [`PAPER_RESULTS_TABLE.md`](../../PAPER_RESULTS_TABLE.md),
   [`PAPER_DRAFT.md`](../../PAPER_DRAFT.md),
   [`PAPER_FIGURE_SPEC.md`](../../PAPER_FIGURE_SPEC.md),
   [`PUBLISHABLE_CLAIMS.md`](../../PUBLISHABLE_CLAIMS.md),
   [`REPRO_ARTIFACT_CHECKLIST.md`](../../REPRO_ARTIFACT_CHECKLIST.md)

**Active next:** figure artwork + venue bibliography (no new experiments unless new hypothesis + hash).

## Stop rules (active)

- No knob massage of P4 spiking-DFA, P5 `rl_graded`, P9 live RFB, or gap-close v14–v19.
- New experiment ⇒ new protocol version + hash.
- G2 thresholds fixed (gap LCB > 0.5, acc ≥ 0.65, PC ≥ 0.90 where applicable).
