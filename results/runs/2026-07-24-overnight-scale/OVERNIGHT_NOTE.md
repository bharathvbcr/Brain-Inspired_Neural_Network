# Overnight note — causal size science (2026-07-24)

Camp: `results/runs/2026-07-24-overnight-scale/`  
Machine: Apple M5 Pro (64 GB). Script: `scripts/overnight_scale.sh`.

**Not Gate G2. Not Foundation Microcircuit (~10⁶ syn). Not G4 mitigation. Not biology.**

---

## Disclosed formulas (hashed into `c1-mac-probe-*`)

| Knob | Formula |
|---|---|
| Syn-matched fan | `max_fan_out = round(1e5 / N)` → {195, 50, 10} at N∈{512, 2000, 10000} |
| Fixed k | `k_wta = 8` (not αN) |
| Init rescale | `init_w_eff = init_w · √(REF_MEAN_FAN_IN / mean_hidden_fan_in)` with `REF_MEAN_FAN_IN = 45` |
| Readout gain | target `(1.15/0.15)·REF_RO_FAN_IN` with `REF_RO_FAN_IN = 64`; `boost = target / mean_readout_fan_in`, clamped to `[1, 64]` |
| Matched budget | `matched_budget_repeat = false` |
| Protocol | `C1_MAC_PROBE_PROTOCOL_VERSION = 1` |

**Caveat:** when `mean_readout_fan_in` ≫ 64 (capped N≥2k), boost floors at 1.0 so effective gain is **not** held to ~491 — disclosed; still better than legacy `1.15/init_w` unchecked growth with init alone.

---

## cells ≠ synapses

| Label | N (cells) | fan | regime | measured nnz | role |
|---|---:|---:|---|---:|---|
| H1 Probe | 512 | 195 | **Bernoulli** | ~9.2e4 | syn-matched ~1e5 |
| H1 mid | 2000 | 50 | **capped** | ~1.02e5 | syn-matched ~1e5 |
| H1 Micro OP | 10000 | 10 | **capped** | ~1.10e5 | **syn-matched-1e5 @ N=1e4** (not Foundation Micro ~10⁶) |
| H3 fan10 | 2000 | 10 | capped | 2.21e4 | density cross |
| H3 fan32 | 2000 | 32 | capped | 6.61e4 | density cross |
| H3 fan64 | 2000 | 64 | capped | 1.30e5 | density cross |
| H3 fan256 | 2000 | 256 | capped | 5.14e5 | density cross |

Never cite Bernoulli `p·N²` for capped rows. Always use **measured nnz + fan + regime**.

---

## H3 — Degree, not width, dominates work/RSS at fixed N

**Geometry:** N=2000, k=8, fan∈{10,32,64,256}, init/readout-rescaled, quick, seed=1.

| fan | measured nnz | predicted nnz | wall_secs | peak_rss_bytes | regime |
|---:|---:|---:|---:|---:|---|
| 10 | 22087 | 22032 | 0.006 | 5.1e6 | capped |
| 32 | 66130 | 66032 | 0.012 | 8.2e6 | capped |
| 64 | 130193 | 130032 | 0.020 | 1.2e7 | capped |
| 256 | 514275 | 514032 | 0.077 | 3.8e7 | capped |

- nnz monotone in fan: **yes** (22k → 66k → 130k → 514k).
- wall + peak RSS monotone in fan: **yes**.
- measured ≈ a·N·fan (predicted): relative error ≪ 1%.

### Verdict: **Accept**

Artifacts: `h3-n2000-fan{10,32,64,256}-quick.{json,md}`  
Hashes (MacProbeConfig): `c1-mac-probe-6209068146f3ef3b` … `c1-mac-probe-5dbec164bb86df5f` (per-fan; see JSON `config_hash` for runner fingerprint).

---

## H1 — Synapse-matched width is inert under ±1

**Geometry:** nnz≈1e5, k=8, init/readout-rescaled, pm1 only, quick, seeds={1,2}.

| N | fan | regime | measured nnz | acc seed1 | acc seed2 | activity | empty_winner |
|---:|---:|---|---:|---:|---:|---:|---:|
| 512 | 195 | Bernoulli | 92276 / 92254 | 0.50 | 0.50 | 0.0156 | 0 |
| 2000 | 50 | capped | 102166 / 102150 | 0.50 | 0.50 | 0.0040 | 0 |
| 10000 | 10 | capped | 109996 | 0.50 | — | 0.0008 | 0 |

- |acc(2000)−acc(512)| = 0 < 0.05; both at chance (0.50).
- Synapse budgets within ±10% of 1e5.
- No OOM; RSS ≪ 48 GB; empty-winner rate 0.
- Micro OP (N=10k fan=10) wall ≪ 20 min/seed — **engineering footnote Pass**, still labeled **syn-matched-1e5 @ 1e4**, not Foundation Micro.

### Verdict: **Accept**

(Width alone does not move live ±1 under this harness. Does **not** reopen G2.)

Hashes: `c1-mac-probe-2eb207d182a9d278` (N=512), `c1-mac-probe-0a6ddda5b3a63df7` (N=2k), `c1-mac-probe-8fea3cc9f1b16a58` (N=10k).

---

## H2 — SFB width-transfer at Pass geometry (N=2k syn-matched)

**Geometry:** N=2000, fan=50, k=8, quick, seed=1. **New hashes only** (no remassage of frozen v13–v19 / v20 in place).

| Arm | MacProbeConfig hash | runner config_hash | acc |
|---|---|---|---:|
| pm1 | `c1-mac-probe-0a6ddda5b3a63df7` | `c1-mac-probe-816eed1c538a6efc` | 0.50 |
| structured-fb | `c1-mac-probe-a34ed07aa40fb96d` | `c1-399b3d83d205a5d4` (proto 15 lineage @ mac geometry) | 0.50 |
| dfa-live | `c1-mac-probe-680952d91d51f28b` | `c1-06283fab4e941916` (proto 20 lineage @ mac geometry) | 0.75 |

- SFB local mean = 0.50 < 0.60 → **Reject-floor** (structured-B does not transfer under width on this quick smoke).
- No SFB signal vs pm1 → **did not promote n=8 / scientific**.
- dfa-live 0.75 on n_test=12 is a single-seed blip; treat as **Inconclusive** for graded-DFA transfer (not scientific-ized).

### Verdict: **Reject-floor** (SFB); dfa-live **Inconclusive** (quick only)

Gap LCB not estimated (no reference arm in isolate-only smoke). If a later Reject-gap (LCB>0.5) appears: **preregister a size protocol** — do not claim PASS on mac-probe / massage knobs.

---

## SHD scientific (complete)

**Completed:** 2026-07-24 ~00:23 local (wall ≈ **3963 s / 66.1 min** from clean restart ~23:17; log `3962.63 real`).  
**PID lineage:** original 5196 died incomplete → restarted bare `--shd-cal` (no `--config-hash`, no `--quick`) → finished EXIT:0.

| Field | Value |
|---|---|
| Config hash | `c1-shd-cal-eb3cb5d93417a638` |
| Protocol | **27** (SCIENTIFIC; includes RL×B) |
| Schedule | SCIENTIFIC · fixture=false |
| Seeds | 5 |
| Geometry | N_IN=700, T=100, hidden=128, epochs=20, lr=0.02 |
| Subset | n_train=2000, n_test=500 (capped; not full-corpus SOTA) |
| Chance | **0.0500 (1/20)** |

### Arm means vs chance

| arm | mean accuracy | vs chance 0.05 |
|---|---:|---|
| `SHD_BROADCAST_PM1` | 0.0544 | ≈ chance |
| `SHD_DFA` | 0.2336 | well above chance |
| `SHD_RL_REINFORCE_FB` (REINFORCE×B) | 0.0532 | ≈ chance |
| `SHD_EPROP_CEILING` (ceiling) | 0.0920 | above chance, below DFA |
| chance (1/20) | 0.0500 | — |

**Ceiling disclosure:** true surrogate e-prop / truncated local BPTT analogue. Full SuperSpike BPTT on SHD-scale `(N_IN≈700, T≈100+, 20-way)` is infeasible in this hand-rolled crate; do **not** read the ceiling as matched SuperSpike.

**Non-claims:** Not Gate G2. Not neuromorphic SOTA. Not drop-in SuperSpike match. Not “local learning impossible.” Not full-corpus SHD SOTA under train/test caps. SHD / mac-probe accuracy ≠ Gate G2.

**Protocol label:** overnight **p27** C1-SHD-CAL (20-way, chance 0.05). Do **not** mix with exploratory **proto-135** `shd-scientific-sweep` (5-class, chance 0.20) in [`shd_scientific_results.md`](../../shd_scientific_results.md).

**Optional `shd_hidden=256`:** initially skipped overnight; later run cleanly via `--shd-hidden 256` (see **SHD h256** below). Frozen p27 h128 artifacts/hash untouched.

Artifacts: `results/c1_shd.md`, camp `c1_shd.md`, `c1_shd_scientific.log` (start `2026-07-24T06:27:24Z` → end `2026-07-24T07:23:44Z`).

---

## SHD h256 scientific (complete)

**Completed:** 2026-07-24T17:51:45Z (wall **8247.16 s / ≈137.5 min**; `time -p` real).  
**PID:** 81291 (wrapper `/tmp/run_shd_h256.sh` → EXIT:0).  
**CLI:** `./target/release/c1 --shd-cal --shd-hidden 256 --out results/c1_shd_h256.md`

| Field | Value |
|---|---|
| Config hash | `c1-shd-cal-bafa6835d8de7eb8` |
| Protocol | **27** (SCIENTIFIC; includes RL×B) — same protocol family as frozen h128 |
| Schedule | SCIENTIFIC · fixture=false |
| Seeds | 5 (same seed list as h128) |
| Geometry | N_IN=700, T=100, **hidden=256**, epochs=20, lr=0.02 |
| Subset | n_train=2000, n_test=500 (capped; not full-corpus SOTA) |
| Chance | **0.0500 (1/20)** |

### Arm means vs chance

| arm | mean accuracy | vs chance 0.05 |
|---|---:|---|
| `SHD_BROADCAST_PM1` | 0.0528 | ≈ chance |
| `SHD_DFA` | 0.2088 | well above chance |
| `SHD_RL_REINFORCE_FB` (REINFORCE×B) | 0.0504 | ≈ chance |
| `SHD_EPROP_CEILING` (ceiling) | 0.1000 | above chance, below DFA |
| chance (1/20) | 0.0500 | — |

**Non-claims:** Not Gate G2. Not neuromorphic SOTA. Not drop-in SuperSpike match. Not “local learning impossible.” Not full-corpus SHD SOTA under caps. Does **not** reopen G2. **Frozen p27 h128** (`c1-shd-cal-eb3cb5d93417a638`, `results/c1_shd.md`) untouched.

Artifacts: `results/c1_shd_h256.md`, camp `c1_shd_h256.md`, `c1_shd_h256_scientific.log` (start `2026-07-24T15:34:17Z` → end `2026-07-24T17:51:45Z`), `c1_shd_h256_end.txt`.

## Non-claims (mandatory)

1. Not G2 PASS / FAIL reinterpretation.
2. Not Foundation Microcircuit (~10⁶ syn) — Micro OP is **syn-matched-1e5 @ N=1e4**.
3. Not G4 mitigation; not biology; not “any broadcast fails.”
4. Never full C1 (dense + SurrogateLif) at N≥2k — isolate-only enforced.
5. Never remassage frozen v13–v19 / P4 / P5 in place — H2 uses **new** mac hashes.
6. Never α-scale k and N together on the primary ladder.
7. Never compare fan=256 cell ladder to fan=10 Micro without synapse-matching.

---

## Code landed (Phase 0)

- `binn-lab/src/mac_probe_config.rs` — presets, hash, formulas, refuse-full-C1
- `Config::{max_fan_out, init_w_rescale, readout_gain_normalize}` + mac-aware hash mix
- `build_sparse_assembly` → `WiringPrior::with_max_fan_out`
- Isolate JSON geometry/integrity/systems fields
- CLI: `--mac-probe`, `--syn-matched`, `--n-hidden`, `--max-fan-out`, `--k-wta`, `--mac-mode`
- `scripts/overnight_scale.sh` (SHD stage no-op)
- Tests: `cargo test -p binn-lab mac_probe` (10 passed); frozen hash suite still green
- SHD h256: `ShdCalConfig::scientific_hidden256()` + frozen hashes `C1_SHD_CAL_SCIENTIFIC_HASH` / `C1_SHD_CAL_HIDDEN256_SCIENTIFIC_HASH`; CLI `--shd-hidden 256`; unit test round-trip ≠ p27

---

## Overnight completion stamp

- SHD p27 scientific **DONE** at ~2026-07-24T00:23 local (UTC write finalize 2026-07-24T15:04:51Z).
- Hash `c1-shd-cal-eb3cb5d93417a638`; wall ≈ 66.1 min; frozen h128 untouched.
- SHD h256 scientific **DONE** 2026-07-24T17:51:45Z; hash `c1-shd-cal-bafa6835d8de7eb8`; wall ≈ 8247 s / 137.5 min (EXIT:0).
- Size-science H1–H3 already filled above; SHD sections filled from camp artifacts.
