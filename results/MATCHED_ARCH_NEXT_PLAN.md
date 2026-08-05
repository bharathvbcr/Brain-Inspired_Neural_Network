# Matched-Architecture Control — next plan / handoff

**Date:** 2026-07-23 · **Owner:** Bharath · **Status:** P1–P4 closed; **P5** graded-primary FAIL; **P6** `rl_reinforce_fb` primary **PASS**; **P7** depth-locality vs inflated `readout_only` **closed (careful)**; **P8** product neuromodulator (v12 family) **wired**; **P9** live C1 opt-in `ReinforceFeedback` (protocol v13) **wired** — see `MATCHED_ARCH_LIVE_REINFORCE.md`.

Related artifacts in `results/`:
- `MATCHED_ARCH_CONTROL.md` — protocol v4 prereg (`c1-match-5dc6822e71229e9e`, FAIL).
- `MATCHED_ARCH_DFA_CONTROL.md` — protocol v5 prereg (`c1-dfa-c8c4fe0899908b84`, **PASS**).
- `MATCHED_ARCH_DFA_SPIKE_CONTROL.md` — protocol v10 prereg (`c1x-dfa-spike-true-dfa-a911e793e590b0ed`, **FAIL**).
- `MATCHED_ARCH_RL_CONTROL.md` — protocol **v12** prereg (`c1-rl-42eddc9c801308e9`, **PASS**); v11 graded-primary FAIL archived (`c1-rl-ef504db58916720d`).
- `MATCHED_ARCH_PRODUCT_NEUROMOD.md` — P8 production `ReinforceFeedback` + `reinforce_term` (no C1 default flip).
- `MATCHED_ARCH_LIVE_REINFORCE.md` — P9 live C1 opt-in protocol v13 (`c1-660401d74db3c88d`).
- `MATCHED_ARCH_DEPTH_LOCALITY.md` — P7 depth vs `readout_only` (NumPy; mid init preferred).
- `MATCHED_ARCH_FINDINGS.md` / `MATCHED_ARCH_DEEP_FINDINGS.md` (+ §E P3, §F P7).
- `c1_match.md`, `c1_dfa.md`, `c1_rl.md`, `c1_rl_v11_graded_primary.md`, `c1_rl_product_neuromod_quick.md`, `c1_rfb_quick.md`, `c1_rfb.md`, `credit_dfa_spike.md`, `deep_xor_thresh.json`, `deep_xnor.json`, `deep_depth_locality*.json`.

---

## Closed this round

| Item | Result |
|---|---|
| **P1** deep forward trainable | done prior (depth BPTT ~0.83; DFA ~0.84) |
| **P2** Rust DFA recipe, protocol v5 | **PASS** — DFA 0.9387 / grad 0.8963 / gap LCB 0.6894 · hash `c1-dfa-c8c4fe0899908b84` |
| **P2** spiking substrate note | credit `dfa-exact-forward` still fails G2 (k-WTA handicap) |
| **P3** second nonlinear task | `xor_thresh` (early cut=3): broadcast 0.50 / DFA 0.83 — locality flip confirmed |
| **P4** spiking-path DFA rescue | **FAIL** — true-dfa 0.6513 / gap LCB 0.0733 / surrogate 0.7238 · hash `c1x-dfa-spike-true-dfa-a911e793e590b0ed` (valid harness; one honest attempt) |
| **P5** in-family RL port (graded primary) | **FAIL** — graded 0.5900 / gap LCB 0.0182; contrast `rl_reinforce_fb` 0.9112 · hash `c1-rl-ef504db58916720d` |
| **P6** RL `rl_reinforce_fb` as primary | **PASS** — fb 0.9200 / gap LCB 0.6846 / graded contrast 0.5250 / flat 0.5113 / grad 0.8887 · hash `c1-rl-42eddc9c801308e9` |
| **P7** depth locality vs inflated `readout_only` | **Closed careful** — strong: DFA exLCB +0.023 / rl_fb exLCB −0.016 (no C3-style claim); **mid**: readout 0.51, DFA/rl_fb clear excess **but broadcast also** (~0.82) — depth help ≠ locality flip. See `MATCHED_ARCH_DEPTH_LOCALITY.md`. |
| **P8** product neuromodulator (v12 family) | **Wired** — production `ReinforceFeedback` + `reinforce_term`; matched arm shares `B_i` lineage; default C1 still broadcast ±1. See `MATCHED_ARCH_PRODUCT_NEUROMOD.md`. |
| **P9** live C1 opt-in `ReinforceFeedback` (v13) | **FAIL** — local 0.4900 / gap LCB 0.0737 / PC 0.9488 · hash `c1-660401d74db3c88d` (valid harness; default ±1 untouched). See `MATCHED_ARCH_LIVE_REINFORCE.md`. |

Reproduce P2/P3/P4/P5/P6/P7/P8/P9:

```bash
cargo run --locked --release -p binn-lab --bin c1 -- --matched-dfa --out results/c1_dfa.md
python3 -m scripts.matched_arch_deep --exp xor_thresh --seeds 12 --epochs 90 --out results/deep_xor_thresh.json
cargo run --locked --release -p binn-lab --bin credit-assignment -- --dfa-spike --out results/credit_dfa_spike.md
cargo run --locked --release -p binn-lab --bin c1 -- --matched-rl --out results/c1_rl.md
python3 -m scripts.matched_arch_deep --exp depth_locality --seeds 12 --epochs 90 \
  --init-preset mid --out results/deep_depth_locality_mid.json
cargo test -p binn-learn --lib reinforce
cargo run --locked --release -p binn-lab --bin c1 -- --matched-rl --quick \
  --out results/c1_rl_product_neuromod_quick.md
cargo test -p binn-lab --lib reinforce_fb
cargo run --locked --release -p binn-lab --bin c1 -- --reinforce-fb --quick \
  --out results/c1_rfb_quick.md
cargo run --locked --release -p binn-lab --bin c1 -- --reinforce-fb \
  --out results/c1_rfb.md
cargo run --locked --release -p binn-lab --bin c1 -- --reinforce-fb \
  --config-hash c1-660401d74db3c88d --out results/c1_rfb_replay.md
```

---

## True next steps (after P9)

Matched-arch optional list + the post-P8 live-engine hypothesis are **done**.
Remaining items are stop-rule hygiene and any *new* hypothesis (new protocol +
hash), not more massage of closed fails:

1. **Do not** further massage spiking DFA rescue knobs without a new protocol + new hypothesis (P4 stop rule: one honest attempt done).
2. **Do not** retune failed v11 `rl_graded` without a new hypothesis + new hash (P5 stop rule).
3. **Do not** remassage live C1 `ReinforceFeedback` knobs without a new hypothesis + new hash (P9 stop rule: one honest attempt done — FAIL).
4. **Do not** treat XNOR as a locality-flip confirmation (broadcast also solves).
5. **Do not** claim NumPy `rl_graded` ~0.81 as a Rust matched-arch result (P5/P6 revise that preview).
6. **Do not** claim C3-style depth locality from P1 strong-init / inflated `readout_only` (P7). Prefer mid-init excess stats; note broadcast also wins there.
7. Locality evidence to cite remains **1-layer XOR / `xor_thresh`**, not 2-layer depth.
8. **Reading:** v12 `rl_reinforce_fb` PASSes on dense-LIF matched; the same family **FAIL**s G2 on live k-WTA C1 (P9). Do not claim live-engine transfer from the matched PASS.
9. **Tier-B capacity footnote (2026-07-23 campaign):** scientific `c1-d38d7644d8afc84b` local mean **0.6775** clears the accuracy floor but gap LCB stays **0.0000** vs dense/grad — descriptive only, **not** a G2 PASS / not a P9 remassage.
10. **Gap-close (new hashes):** protocols **v14–v19** — see `GAP_CLOSE_RFB_TRANSFER.md`. Do not treat these as v13 remassages.
    - v14 `--rfb-epoch` **FAIL** (local 0.4838)
    - v15 `--structured-fb` **FAIL** but clears floor (local **0.7262**, gap LCB 0.2567) — best accuracy
    - v16 `--structured-fb-epoch` **FAIL** (local 0.5200) — epochs hurt structured B
    - v17 `--structured-fb-capacity` **FAIL** (local **0.6825**, gap LCB **0.3127** — best gap LCB; still short of G2)
    - v18 `--elig-rfb` **FAIL** (local **0.7125**, gap LCB 0.2351) — eligibility co-design does not beat v15
    - v19 `--structured-fb-teach` **FAIL** (local **0.6700**, gap LCB 0.2238) — restored target teach does not beat v15

### Suggested new-hypothesis candidates (only with fresh hash)

- ~~Structured (non-random) feedback / learned `B` under k-WTA sparsity.~~ → **v15 wired**; FAIL (acc floor cleared).
- ~~Two-pass / epoch-matched credit on live C1 with production `ReinforceFeedback`.~~ → **v14 wired**; FAIL.
- ~~Structured × epoch combo.~~ → **v16 wired**; FAIL (regression vs v15).
- ~~Structured × capacity.~~ → **v17 wired**; FAIL (gap LCB 0.3127, floor ✓).
- ~~Eligibility-trace timing co-designed with sampled REINFORCE.~~ → **v18 wired**; FAIL (no gain vs v15).
- ~~Structured B × restored target teach (`credit(+1)`).~~ → **v19 wired**; FAIL (no gain vs v15).
- ~~Paper/write-up packaging of closed matched-arch series (P1–P9) + gap-close negatives.~~ → **done** — [`PUBLISHABLE_CLAIMS.md`](PUBLISHABLE_CLAIMS.md) + [`PAPER_SKELETON.md`](PAPER_SKELETON.md) filled from claim freeze (abstract draft + transfer section).

### Active next (writing, not experiments)

1. ~~Expand skeleton into venue draft prose.~~ → [`PAPER_DRAFT.md`](PAPER_DRAFT.md)
2. Generate figures from [`PAPER_FIGURE_SPEC.md`](PAPER_FIGURE_SPEC.md)
3. Venue formatting / bibliography; ship files in [`REPRO_ARTIFACT_CHECKLIST.md`](REPRO_ARTIFACT_CHECKLIST.md) §E
4. Optional: exact `--config-hash` replays into `*_replay.md` twins before submission

## Guardrails (unchanged)

- New experiment ⇒ new protocol version + hash. Never reuse `c1-118207fbc3eaba53`.
- G2 thresholds fixed (0.5 gap LCB, 0.65 floor).
- `*_baseline.rs` arms stay GC1-exempt; production stays on `three_factor`.
