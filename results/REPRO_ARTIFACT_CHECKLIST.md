# BINN camera-ready reproducibility checklist

> **CITATION WARNING (added 2026-08-07).** This document cites the
> `track-b-rescue` **v130** row (`1.0000`, gap LCB `0.9988`, PASS matched) as a
> matched-substrate result. That report is stale: the source is **v131**, and the
> 130→131 bump is precisely the clamp-and-separation-gate fix for the defect the
> row exhibits. Under current code the arm **cannot be reported as PASS**. Do not
> cite it until `track-b-rescue` has been re-run. The DFA
> (`c1-dfa-c8c4fe0899908b84`) and RL (`c1-rl-42eddc9c801308e9`) matched PASSes are
> **not** affected by this defect; they ran through the clamped `runner.rs` path.
> See `AUDIT_2026-08-07_JULY_CAMPAIGN_SCORING_PATH.md` and
> `TODO_2026-08-07_OPEN_WORK.md` §1.
> **RESOLVED 2026-08-19.** The re-run landed. At v131 the arm reports
> **INVALID_HARNESS**, not PASS: the ceiling-inverted warning fires on 3 of 20
> learned-FB seeds and the code refuses to emit a PASS while it is present.
> The v130 PASS is **withdrawn**. See
> `RESULT_2026-08-19_TRACK_B_V130_PASS_WITHDRAWN.md` and
> `track_b_results_v131.md`.



Workspace root for commands: `binn/` (crate workspace). Do not change G2 thresholds or reopen `c1-118207fbc3eaba53`.

**Paper package index**

| Doc | Role |
|---|---|
| [`PUBLISHABLE_CLAIMS.md`](PUBLISHABLE_CLAIMS.md) | Claim ladder + non-claims |
| [`PAPER_RESULTS_TABLE.md`](PAPER_RESULTS_TABLE.md) | Cite-every-number table |
| [`PAPER_METRICS_FULL.md`](PAPER_METRICS_FULL.md) | Dual-gap + seed diagnostics + axes |
| [`HARD_AUDIT.md`](HARD_AUDIT.md) | Overclaim / dual-gap / confounders |
| [`CLAIM_AXIS.md`](CLAIM_AXIS.md) | Novel-CS / Motif / Integrity taxonomy |
| [`DIFF_CLOSURE.md`](DIFF_CLOSURE.md) | No-deferrals closure matrix |
| [`APPENDIX_POST_G2.md`](APPENDIX_POST_G2.md) | C2/C3/R/hybrid/deep appendix |
| [`PAPER_SKELETON.md`](PAPER_SKELETON.md) | Outline + figure map |
| [`PAPER_DRAFT.md`](PAPER_DRAFT.md) | Camera-ready prose draft |
| [`PAPER_FIGURE_SPEC.md`](PAPER_FIGURE_SPEC.md) | Figure/table specs |
| [`CAMPAIGN_2026-07-23_CLAIM_FREEZE.md`](CAMPAIGN_2026-07-23_CLAIM_FREEZE.md) | Campaign number freeze |
| [`GAP_CLOSE_RFB_TRANSFER.md`](GAP_CLOSE_RFB_TRANSFER.md) | Transfer suite |
| [`VENUE_FORMATTING.md`](VENUE_FORMATTING.md) | Venue skeleton + figure inventory |
| [`references.bib`](references.bib) | Bibliography stubs |
| [`runs/2026-07-23-paper-hard-both/`](runs/2026-07-23-paper-hard-both/) | Hardened camp (verify + v20–v24) |

---

## Integrity fix policy (read first)

- [x] Optional integrity fixes are **new protocols** with **new hashes** — never reopen `c1-118207fbc3eaba53`
- [x] Must **NOT** threshold-massage G2 floors
- [x] Must **NOT** cite quick/PILOT hashes as scientific verdicts
- [x] Must disclose DFA-schedule broadcast-graded contrast (0.9863) when discussing coincidence locality
- [x] Lead FAIL labeled **broadcast ±1 three-factor** (not bare “broadcast credit topology”)
- [x] XOR cited as locality evidence; Figure M **spec** wired in [`PAPER_FIGURE_SPEC.md`](PAPER_FIGURE_SPEC.md) (camp artwork present — see §G)
- [x] Discussion / Limitations covers F1/F2/F5, appendix-only G3/G4/H0, discrete EventProp — the **FAIL is WITHDRAWN**, and the retired hash `c1-eventprop-5bb083d5e88d0ad2` with it; it PASSes at 0.9450 ff / 0.8900 rec and remains discrete, with no comparison to continuous Wunderlich–Pehle claimed in either direction — v131 matched-only honesty, falsifier ([`PAPER_DRAFT.md`](PAPER_DRAFT.md) §4)
- [x] No new experiment hashes remassaging frozen G2 in MUST packaging pass
- [x] Hybrid T=2.0 ≠ live v21 (T=1) disclosed where soft-WTA is discussed
- [x] v131 / `live-transfer-rescue` relabeled matched-only (not live k-WTA PASS)

---

## Build / sanity

```bash
cd binn
cargo test --locked --workspace
cargo fmt --all -- --check
./scripts/gc_checks.sh   # if present
```

---

## A. Matched-arch (primary)

> **The three commands here could not run, and were published for four days.**
> Until 2026-08-29 they passed `c1-match-5dc6822e71229e9e`,
> `c1-dfa-c8c4fe0899908b84` and `c1-rl-42eddc9c801308e9`. Those hashes were
> **deliberately retired** on 2026-08-25 — `MATCHED_INPUT_SCALE` was not mixed
> into them, so each named two different experiments either side of the
> silent-initialisation repair — and `from_hash` returns `None` for all three
> ([`RESULT_2026-08-25_MATCHED_ARCH_RERUN.md`](RESULT_2026-08-25_MATCHED_ARCH_RERUN.md) §8).
> This is the document a reviewer runs, and its headline section named hashes
> the binary rejects. `scripts/test_published_hashes_resolve.py` now fails on
> any `--config-hash` in the record that names a retired hash.

```bash
cargo run --locked --release -p binn-lab --bin c1 -- --matched-arch \
  --config-hash c1-match-6f6000f148f7d30c --out results/c1_match_replay.md
cargo run --locked --release -p binn-lab --bin c1 -- --matched-dfa \
  --config-hash c1-dfa-f79c01ea36fe27d7 --out results/c1_dfa_replay.md
cargo run --locked --release -p binn-lab --bin c1 -- --matched-rl \
  --config-hash c1-rl-d35e13c758e522f8 --out results/c1_rl_replay.md
```

Each hash above is the one its config's frozen-hash test asserts, and each of
those tests also asserts `from_hash(&hash) == Some(&scientific())` — so a green
`cargo test -p binn-lab` is what establishes these three commands resolve.

Ship notes: `matched_rerun_2026-08-25/`. The July `c1_match.md`, `c1_dfa.md`
and `c1_rl.md` are the **pre-repair** run records and are read as history.

---

## B. Canonical C1 / Gate G2 (secondary)

```bash
cargo run --locked --release -p binn-lab --bin c1 -- \
  --config-hash c1-118207fbc3eaba53 --out results/c1_g2_replay.md
```

Cross-check [`U-NEG_protocol_v2.md`](U-NEG_protocol_v2.md). Integrity appendix required in paper.

---

## C. Live RFB + gap-close (transfer)

```bash
cargo run --locked --release -p binn-lab --bin c1 -- --reinforce-fb \
  --config-hash c1-660401d74db3c88d --out results/c1_rfb_replay.md
cargo run --locked --release -p binn-lab --bin c1 -- --rfb-epoch \
  --config-hash c1-714c115e14a3eeed --out results/c1_rfb_em_replay.md
cargo run --locked --release -p binn-lab --bin c1 -- --structured-fb \
  --config-hash c1-493ddd56f8714fb6 --out results/c1_sfb_replay.md
cargo run --locked --release -p binn-lab --bin c1 -- --structured-fb-epoch \
  --config-hash c1-677df7f7cbe4f8ec --out results/c1_sfb_em_replay.md
cargo run --locked --release -p binn-lab --bin c1 -- --structured-fb-capacity \
  --config-hash c1-983ee5303c00b147 --out results/c1_sfb_cap_replay.md
cargo run --locked --release -p binn-lab --bin c1 -- --elig-rfb \
  --config-hash c1-c7d2c86a2b1927f6 --out results/c1_elig_rfb_replay.md
cargo run --locked --release -p binn-lab --bin c1 -- --structured-fb-teach \
  --config-hash c1-dfab4a7ec19f17c2 --out results/c1_sfb_teach_replay.md

# Break-it v20–v24 (camp: results/runs/2026-07-23-paper-hard-both/)
cargo run --locked --release -p binn-lab --bin c1 -- --dfa-live \
  --config-hash c1-4db53e645405fae0 --out results/c1_dfa_live.md
cargo run --locked --release -p binn-lab --bin c1 -- --structured-fb-soft \
  --config-hash c1-f975db8fb3e5d569 --out results/c1_sfb_soft.md
cargo run --locked --release -p binn-lab --bin c1 -- --matched-arch --match-undertrain \
  --config-hash c1-match-b46b23549b37d90a --out results/c1_match_ep4.md
cargo run --locked --release -p binn-lab --bin c1 -- --structured-fb-finth \
  --config-hash c1-4bbaf4b24c2d1da2 --out results/c1_sfb_finth.md
cargo run --locked --release -p binn-lab --bin c1 -- --structured-fb-cont \
  --config-hash c1-840f820b7c07b512 --out results/c1_sfb_cont.md
```

---

## D. Supporting (cite only if used)

```bash
# Capacity / isolation / temporal-PC
cargo run --locked --release -p binn-lab --bin c1 -- --sensitivity capacity \
  --config-hash c1-d38d7644d8afc84b --out results/c1_sens_capacity_replay.md
cargo run --locked --release -p binn-lab --bin c1 -- --isolation \
  --config-hash c1-8ec031907a3426d0 --out results/c1_iso_replay.md

# Spiking DFA rescue (P4 stop — one honest attempt)
cargo run --locked --release -p binn-lab --bin credit-assignment -- --dfa-spike \
  --out results/credit_dfa_spike.md

# NumPy XOR / depth
python3 -m scripts.matched_arch_deep --exp xor_thresh --seeds 12 --epochs 90 \
  --out results/deep_xor_thresh.json
python3 -m scripts.matched_arch_deep --exp depth_locality --seeds 12 --epochs 90 \
  --init-preset mid --out results/deep_depth_locality_mid.json
```

---

## E. Files that must ship

### Claim / draft package

- [x] `PUBLISHABLE_CLAIMS.md`
- [x] `PAPER_RESULTS_TABLE.md`
- [x] `PAPER_SKELETON.md`
- [x] `PAPER_DRAFT.md`
- [x] `PAPER_FIGURE_SPEC.md`
- [x] `REPRO_ARTIFACT_CHECKLIST.md` (this file)
- [x] `CAMPAIGN_2026-07-23_CLAIM_FREEZE.md`
- [x] `GAP_CLOSE_RFB_TRANSFER.md`

### Primary result notes

- [x] `c1_match.md` · `c1_dfa.md` · `c1_rl.md`
- [x] `MATCHED_ARCH_CONTROL.md` · `MATCHED_ARCH_DFA_CONTROL.md` · `MATCHED_ARCH_RL_CONTROL.md`
- [x] `MATCHED_ARCH_LIVE_REINFORCE.md`

### Secondary / transfer notes

- [x] `c1_g2.md` · `U-NEG_protocol_v2.md`
- [x] `c1_rfb.md` · `c1_rfb_em.md` · `c1_sfb.md` · `c1_sfb_em.md` · `c1_sfb_cap.md` · `c1_elig_rfb.md`
- [x] `c1_iso.md` · `c1_sens_capacity_full.md` · `c1_sens_temporal_pc_full.md`
- [x] `credit_dfa_spike.md`
- [x] `deep_xor_thresh.json` · `deep_depth_locality_mid.json`

### Source / lock

- [x] `Cargo.lock`
- [x] `binn-learn/src/{matched_local_baseline,three_factor,credit,matched_rl_baseline}.rs`
- [x] `binn-lab/src/{match_config,runner_match,dfa_match_config,rl_match_config,runner,config}.rs`
- [x] `binn/README.md`

---

## F. Hash inventory (scientific only)

| Hash | Role |
|---|---|
| `c1-118207fbc3eaba53` | Canonical C1 v2 FAIL |
| `c1-match-6f6000f148f7d30c` | Matched broadcast ±1 three-factor FAIL (0.5000 ff / 0.5100 rec) |
| `c1-dfa-f79c01ea36fe27d7` | Matched DFA PASS (disclose broadcast-graded 0.9975) |
| `c1-rl-d35e13c758e522f8` | Matched RL reinforce_fb PASS |
| `c1-rl-ef504db58916720d` | Archived RL graded FAIL |
| `c1-660401d74db3c88d` | Live RFB v13 FAIL |
| `c1-714c115e14a3eeed` | Gap-close v14 FAIL |
| `c1-493ddd56f8714fb6` | Gap-close v15 FAIL (floor ✓) |
| `c1-677df7f7cbe4f8ec` | Gap-close v16 FAIL |
| `c1-983ee5303c00b147` | Gap-close v17 FAIL (best LCB) |
| `c1-c7d2c86a2b1927f6` | Gap-close v18 FAIL |
| `c1-dfab4a7ec19f17c2` | Gap-close v19 FAIL |
| `c1-4db53e645405fae0` | Break-it v20 live DFA FAIL (floor ✓) |
| `c1-f975db8fb3e5d569` | Break-it v21 soft-WTA FAIL |
| `c1-match-b46b23549b37d90a` | Break-it v22 undertrain FAIL |
| `c1-4bbaf4b24c2d1da2` | Break-it v23 finth FAIL (floor ✓) |
| `c1-840f820b7c07b512` | Break-it v24 continuous B FAIL |
| `c1-d38d7644d8afc84b` | Capacity sensitivity FAIL (floor ✓) |
| `c1-8ec031907a3426d0` | Isolation FAIL |
| `c1-a49deeaedb495a09` | Temporal-PC FAIL |
| `c1x-dfa-spike-true-dfa-a911e793e590b0ed` | Spiking DFA FAIL |
| `c1-eventprop-f1e841c29755b1c8` | Discrete EventProp-style H2H **PASS** — the archived FAIL is withdrawn (≠ continuous Wunderlich–Pehle) |
| `c1-match-5dc6822e71229e9e` | **RETIRED** 2026-08-25 — pre-repair, does not resolve; superseded by `c1-match-6f6000f148f7d30c` |
| `c1-dfa-c8c4fe0899908b84` | **RETIRED** 2026-08-25 — pre-repair, does not resolve; superseded by `c1-dfa-f79c01ea36fe27d7` |
| `c1-rl-42eddc9c801308e9` | **RETIRED** 2026-08-25 — pre-repair, does not resolve; superseded by `c1-rl-d35e13c758e522f8` |
| `c1-eventprop-5bb083d5e88d0ad2` | **RETIRED** 2026-08-25 — pre-repair, does not resolve; superseded by `c1-eventprop-f1e841c29755b1c8`, and the FAIL it carried is withdrawn |
| `track-b-rescue` | **WITHDRAWN** under v131 (`INVALID_HARNESS` from ceiling-inverted warnings; `RESULT_2026-08-19_TRACK_B_V130_PASS_WITHDRAWN.md`) |

---

## F2. SHD attention campaign artifacts

| Campaign | Cells | Pinned binary | Role | Manifest |
|---|---|---|---|---|
| `shd_attention_campaign_v1/cells/` | 528 complete of 552 planned (24 diverged, 0 voided) | `22d97c51ab02` | Waves 1–7 (budget ladder, sample efficiency, width/geometry contrasts) plus the 36 `r1cal__` recalibration cells | `manifest.json` + `gates/` (per-instance) |
| `shd_attention_campaign_v2/`, cells `w8*__` | 72 complete (0 voided) | `22d97c51ab02` | Wave 8: d32/L4 headline at e400 (0.8320), width & geometry scope | `manifest.json` (`cell_count` 96 = wave 8 + wave 9) |
| `shd_attention_campaign_v2/`, cells `w9dim__` / `w9shf__` | 24 complete (0 voided) | `22d97c51ab02` | Wave 9: temporal order mechanism proof (M-1/M-2 +0.1337 shuffle drop) | same `manifest.json` (`cell_count` 96 = wave 8 + wave 9) |

---

## G. Paper text gates (camera-ready MUST)

- [x] Lead figure = matched-arch rule swap
- [x] **Figure M** mechanism artwork at `runs/2026-07-23-paper-hard-both/figures/figM_mechanism_richness_addressability.{png,pdf}`
- [x] Contrast PASSes (DFA / RL) with honesty note on broadcast-graded
- [x] Claim language uses **broadcast ±1 three-factor** wherever bare “broadcast credit topology” would misread
- [x] Engine C1 secondary + integrity appendix
- [x] Transfer section: matched PASS ≠ live PASS; floor ≠ gate; v131 matched-only honesty
- [x] XOR locality; no depth locality-flip claim
- [x] Discussion / Limitations: F1/F2/F5, G3/G4/H0 appendix-only, discrete EventProp FAIL, falsifier, G4 NO-GO redirect
- [x] Thesis matches [`PUBLISHABLE_CLAIMS.md`](PUBLISHABLE_CLAIMS.md)
- [x] Non-claims block present
- [x] Every numeric claim traced to [`PAPER_RESULTS_TABLE.md`](PAPER_RESULTS_TABLE.md)
- [x] Generate figures from [`PAPER_FIGURE_SPEC.md`](PAPER_FIGURE_SPEC.md) → camp `figures/` (**present:** fig0/1/2/3/4/5/D, figM, graphical abstract)
- [x] [`DIFF_CLOSURE.md`](DIFF_CLOSURE.md) has **zero** empty / deferred cells
- [x] Dual-gap harvest on live break-it arms ([`PAPER_METRICS_FULL.md`](PAPER_METRICS_FULL.md))
- [x] Venue formatting / bibliography skeleton ([`VENUE_FORMATTING.md`](VENUE_FORMATTING.md), [`references.bib`](references.bib)) — final venue style pass still open
- [x] Primary scientific hash replays bit-stable — [`PAPER_VERIFY.md`](PAPER_VERIFY.md)
- [x] Paper hash freeze unit tests (`paper_scientific_hashes_are_frozen`, match/dfa/rl freezes)
- [x] Gap-close disclosure + structured-B sign tests
- [x] MUST package = M1 Discussion + M2 Figure M **spec** + M3 claim hygiene (no Mac/SHD/Micro as G2 reinterpretation)

### EventProp replay (supporting)

```bash
cargo run --locked --release -p binn-lab --bin c1 -- --eventprop \
  --config-hash c1-eventprop-f1e841c29755b1c8 --out results/c1_eventprop_replay.md
```

The **archived** hash `c1-eventprop-5bb083d5e88d0ad2` is retired and does not
resolve. It named a run on a forward that emitted no spikes, which is why a
spike-adjoint method scored 0.5000 there; the arm PASSes at 0.9450 ff / 0.8900
rec on a forward that can spike. Ship note: `matched_rerun_2026-08-25/`.
