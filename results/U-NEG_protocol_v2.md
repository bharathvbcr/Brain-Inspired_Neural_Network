# U-NEG — scoped negative note (protocol v2)

**Unit:** U-NEG (v8 kill-gate packaging after Gate G2 FAIL)  
**Date packaged:** 2026-07-23  
**Canonical config hash:** `c1-118207fbc3eaba53`  
**Scientific protocol version:** `2`  
**n seeds:** 20 (preregistered scientific schedule)

This note is the publishable U-NEG claim for BINN C1 / Gate G2. Raw harness output lives in [`c1_g2.md`](c1_g2.md) and the dated run artifacts under [`runs/2026-07-22/`](runs/2026-07-22/). Protocol-v3 Tier-B sensitivities (folded below) are optional confound probes — they do **not** reopen the v2 kill-gate; harness detail: [`SENSITIVITY_PROTOCOLS.md`](SENSITIVITY_PROTOCOLS.md).

---

## Verdict

Under protocol v2 hash `c1-118207fbc3eaba53`, **Gate G2 = FAIL**.

Both preregistered PASS thresholds were missed:

| Gate | Requirement | Observed | Cleared? |
|---|---|---|---|
| Gap LCB | lower 95% bound on normalized gap closed **> 0.5** | **−0.0048** | no |
| Accuracy floor | mean local-assembly accuracy **≥ 0.65** | **0.4912** | no |

Harness validity held (not `INVALID_HARNESS`):

- positive control mean **0.9488** (≥ 0.900)
- mean local activity sparsity **0.0156** (band [0.0050, 0.0300])

Config-hash replay is **identical** (same seeds, per-seed accuracies, summaries, and verdict): compare [`runs/2026-07-22/c1_g2_full_v11.md`](runs/2026-07-22/c1_g2_full_v11.md) with [`runs/2026-07-22/c1_g2_full_v11_replay.md`](runs/2026-07-22/c1_g2_full_v11_replay.md). Audit context: [`BUILD_AUDIT_v11_RESOLUTION.md`](BUILD_AUDIT_v11_RESOLUTION.md).

**Program status:** stop before P3+ (no C2/C3/R1/R2, Hub, or U18–U23 under the v8 G2 kill-gate). Protocol-v3 sensitivities below **harden** this stop; they do **not** license P3+.

---

## What this claims

1. On this hashed pipeline (protocol v2, `c1-118207fbc3eaba53`, n=20), local three-factor / assembly learning **did not** close the preregistered normalized gradient gap or clear the absolute accuracy floor.
2. The harness was valid; the FAIL is a scientific gate decision, not an invalid-run discard.
3. On the **same train/test splits**, gradient and eligibility references succeed while local and dense three-factor arms stay at/near chance — including the parameter-matched dense arm.
4. Per v8, the scheduled program ends at G2 packaging (this note); P3+ is not scheduled from this result.
5. Full n=20 protocol-v3 sensitivities (temporal PC; capacity) also **FAIL** Gate G2 under distinct hashes — hardening the scoped negative without reopening or rewriting `c1-118207fbc3eaba53`.

## What this does **not** claim

- That local three-factor learning is impossible **in principle**
- That every capacity, encoder, lag, or schedule would fail
- A **matched-architecture-only** gap (refs and local path do not share the same front-end / training graph; see residual confounds)
- Biological or neuromorphic hardware failure
- Anything beyond C1 / Gate G2 (no multi-area, sleep, pruning, G5 “beats dense,” etc.)
- Reopening or reinterpreting hash `c1-118207fbc3eaba53` (v3 sensitivities mint **new** hashes; they are not a v2 kill-gate reopen)
- That P3+ is licensed from the v3 probes (harden ≠ reclassify ≠ schedule)

---

## Plateau table (means, n=20)

Canonical numbers from [`c1_g2.md`](c1_g2.md) / full v11 run:

| Condition | Mean accuracy | Variance | Role |
|---|---:|---:|---|
| local-assembly | **0.4912** | 0.002452 | three-factor + sparse assembly + k-WTA |
| dense-local | **0.5000** | 0.000000 | same rule, dense connectivity, no assembly |
| dense-matched | **0.5000** | (exact 0.5 all seeds) | nnz matched to local-assembly (param-matched) |
| gradient-reference | **0.8938** | 0.027163 | surrogate-LIF BPTT (primary ceiling) |
| eligibility-reference | **1.0000** | 0.000000 | e-prop-compatible rate FF reference |

| Aggregate | Value | Threshold |
|---|---:|---|
| mean normalized gap closed | 0.0189 | — |
| gap closed LCB (z=1.96, n=20) | **−0.0048** | need **> 0.5** |
| mean \|local − dense\| (descriptive) | 0.0262 | — |
| positive control | 0.9488 | ≥ 0.9 |
| activity sparsity (local) | 0.0156 | ∈ [0.005, 0.030] |

**Reading the plateau:** local-assembly and both dense three-factor arms sit at chance (~0.5). References clear the task on the same splits. The failure is therefore “local/dense three-factor path does not learn this C1 task under this hash,” not “the dataset is unsolvable.” Dense-local / dense-matched landing at exact 0.5 with zero variance is a symmetric no-signal attractor, which **weakens** (does not strengthen) any assembly-vs-dense contrast — both fail together.

---

## Residual confounds (honest scope)

These limit how far the negative generalizes; they are **not** grounds to treat the harness as invalid:

1. **Canonical positive control is spatial, not temporal.** On v2 it shows the local pipeline can learn a trivial feature-presence task (≥0.9). The protocol-v3 temporal-PC probe (folded above) clears coincidence-lag PC under the same encoder (≥0.9) while G2 still FAIL — so the spatial-PC caveat no longer carries the negative alone, but architecture/capacity mismatches remain.
2. **Architecture matching is partial.** References use continuous frames + epoch BPTT / e-prop; the local path uses LatencyEncoder, θ=∞ force-spike k-WTA (`k=2`, `N=128` on v2), and one online pass. Shared LIF constants ≠ matched computational graph.
3. **Capacity is thin on the kill-gate.** Live v2 stayed `n_hidden=128`, `k_wta=2`, `n_train=80`. A richer capacity schedule was run as protocol-v3 hash `c1-d38d7644d8afc84b` (see folded sensitivities) — still FAIL, not a reclassification PASS.
4. **Eligibility reference is a strong but mismatched ceiling** (accuracy 1.0 with a small feedforward rate model), which shows task solvability while weakening “matched local ceiling” language.
5. **Plot figures are diagnostic only** (last-seed overwrite of fixed PNG paths; see Plots section). They do not enter the G2 decision.
6. **Verification is same-harness replay**, not an independent second-stack reimplementation.

None of the above reopen G2 under `c1-118207fbc3eaba53`. Optional sensitivities mint new protocol-v3 hashes and must not silently reuse the kill-gate hash.

---

## Addendum — confound #2 closed (protocol v4 matched-architecture)

**Date:** 2026-07-23 · **Hash:** `c1-match-5dc6822e71229e9e` · **Does not reopen** `c1-118207fbc3eaba53`.

Under a fresh protocol-v4 hash that holds one shared dense-LIF forward fixed and swaps **only** the learning rule, matched-local (production broadcast ±1 three-factor) **FAILS** the unchanged G2 thresholds (mean 0.5000, gap LCB 0.0000) while matched-gradient clears the floor (mean 0.8963). **Confound #2 is closed: the FAIL is the rule, not the path.** See [`c1_match.md`](c1_match.md) / [`MATCHED_ARCH_CONTROL.md`](MATCHED_ARCH_CONTROL.md).

### Follow-on — graded error × DFA passes (protocol v5; does not reopen v2)

**Date:** 2026-07-23 · **Hash:** `c1-dfa-c8c4fe0899908b84` · **Does not reopen** `c1-118207fbc3eaba53`.

Replacing the ±1 broadcast reward with a **directional graded error × fixed-random DFA feedback** on the same matched feed-forward graph **PASSES** unchanged G2 thresholds (DFA mean **0.9387**, gap LCB **0.6894**; gradient ceiling 0.8963). Broadcast graded error also clears coincidence (0.9863) — locality is required only on nonlinear tasks (see P3 / `MATCHED_ARCH_DEEP_FINDINGS.md` §E). Spiking-substrate DFA (`c1x-dfa-exact-forward-*`) still fails G2; one honest rescue (`c1x-dfa-spike-true-dfa-a911e793e590b0ed`, true-dfa 0.6513 / gap LCB 0.0733) also **FAIL**s — see [`MATCHED_ARCH_DFA_SPIKE_CONTROL.md`](MATCHED_ARCH_DFA_SPIKE_CONTROL.md). See [`c1_dfa.md`](c1_dfa.md) / [`MATCHED_ARCH_DFA_CONTROL.md`](MATCHED_ARCH_DFA_CONTROL.md).

---

## Protocol-v3 sensitivities (folded; do not reopen v2)

Tier-B full scientific runs (n=20, protocol version **3**). Detail and run commands: [`SENSITIVITY_PROTOCOLS.md`](SENSITIVITY_PROTOCOLS.md). **v3 ≠ reopen of the v2 kill-gate.**

| Preset | Hash | Verdict | PC | Local | Gap LCB |
|---|---|---|---:|---:|---:|
| Canonical G2 (v2) | `c1-118207fbc3eaba53` | **FAIL** | 0.9488 | 0.4912 | −0.0048 |
| Temporal PC | `c1-a49deeaedb495a09` | **FAIL** | 0.9675 | 0.5263 | −0.0118 |
| Capacity | `c1-d38d7644d8afc84b` | **FAIL** | 1.0000 | 0.6775 | 0.0000 |

Raw notes: [`c1_sens_temporal_pc_full.md`](c1_sens_temporal_pc_full.md), [`c1_sens_capacity_full.md`](c1_sens_capacity_full.md).

**Interpretation (preserve):**

1. **Temporal PC** cleared (≥0.9) under the same LatencyEncoder / local path while G2 still FAIL → **hardens** scoped U-NEG (local can learn coincidence lag; still misses accuracy and gap gates).
2. **Capacity** raises local (clears 0.65: 0.6775) but dense-local jumps to 0.9400 so gap-closed LCB stays 0.0000 → **hardened, not reclassified** (not a schedule/front-end capacity PASS).
3. Neither probe is a new protocol-v2 G2 decision. Do **not** claim impossibility in principle. Do **not** claim P3+ is licensed.

---

## Post-G2 override full runs (exploratory; do not reopen v2)

Owner-requested engineering completion after Gate G2 FAIL produced **full** C2–R2 schedules under explicit `--enable-*` / env overrides. Index: [`OVERRIDE_FULL_RUNS.md`](OVERRIDE_FULL_RUNS.md); per-exp notes: [`C2_OVERRIDE.md`](C2_OVERRIDE.md), [`C3_OVERRIDE.md`](C3_OVERRIDE.md), [`R1_OVERRIDE.md`](R1_OVERRIDE.md), [`R2_OVERRIDE.md`](R2_OVERRIDE.md). These runs mint distinct hashes and are **exploratory only**. They were **built under `--enable-*` override flags**; they do **not** reopen kill-gate `c1-118207fbc3eaba53`; they do **not** license P3+ as default under the v8 G2 stop.

| Exp | Verdict | Hash | Pointer |
|---|---|---|---|
| C2 / G3 | **FAIL** | `c2-c45f08841f2f9df9` | [`c2_g3.md`](c2_g3.md) |
| C3 v1 tabular proxy | **MEASURED** (D\* terminal reward=3 vs teacher-forced oracle=8) | `c3-445aa8de7761d4f4` | [`c3_credit_depth.md`](c3_credit_depth.md) |
| R1 | **ADDITIVE** | `r1-5d30383e334b9cbe` | [`r1_composition.md`](r1_composition.md) |
| R2 / G4 | **NO-GO** (degrade) | `r2-afafa0fa6f43e3fc` | [`r2_scaling.md`](r2_scaling.md) |

Separately indexed under the same override policy: U18–U20 / Gate G5 in [`u20_efficiency.md`](u20_efficiency.md) (**G5 FAIL** — exploratory work/accuracy disclosure, not a G2 reopen) and U21–U23 in [`POST_G2_BUILD.md`](POST_G2_BUILD.md) / [`u21_consolidation.md`](u21_consolidation.md), [`u22_pruning.md`](u22_pruning.md), [`u23_resting.md`](u23_resting.md). Same framing: override-gated engineering evidence; kill-gate hash unchanged; no default P3+ license.

---

## Plots (U-NEG packaging)

Raster / weight figures for hash `c1-118207fbc3eaba53` were produced on 2026-07-23 with the optional `plots` feature. **Scientific verdict and numbers are unchanged** vs [`c1_g2.md`](c1_g2.md) (same FAIL, gap LCB −0.0048, local 0.4912, PC 0.9488, sparsity 0.0156).

| Figure | Path |
|---|---|
| Local-assembly spike raster (last seed) | [`plots/c1_g2_raster.png`](plots/c1_g2_raster.png) |
| Local-assembly readout weight trace (last seed) | [`plots/c1_g2_weights.png`](plots/c1_g2_weights.png) |
| Harness overwrite targets (same content as above after the G2 plots run) | [`c1_raster.png`](c1_raster.png), [`c1_weights.png`](c1_weights.png) |
| Markdown note with `Written` plot status | [`c1_g2_plots.md`](c1_g2_plots.md) |
| Quick-schedule pilot figures (hash `c1-e0dfdbf4e3d2936b`, not a G2 decision) | [`plots/c1_quick_raster.png`](plots/c1_quick_raster.png), [`plots/c1_quick_weights.png`](plots/c1_quick_weights.png) |

**How plots were enabled (local, documented):**

- System Python is **3.14**; workspace `pyo3` **0.22** supports ≤**3.13**.
- Local venv: `binn/.venv` (Python **3.12.13** + `matplotlib`), created via `./scripts/setup_plots_venv.sh` / `requirements-plots.txt` (gitignored `.venv/`).
- Build/run with `PYO3_PYTHON` pointing at that interpreter; runtime needs `VIRTUAL_ENV` so the embed finds venv `site-packages`, and `MPLBACKEND=Agg` for headless savefig.
- Convenience wrapper: `./scripts/run_c1_plots.sh --config-hash c1-118207fbc3eaba53 --out results/c1_g2_plots.md`
- With `--features plots`, the harness runs conditions **in-process** so raster/weight traces are retained (isolate JSON omits them). Accuracies/budgets match the scientific path; peak-RSS isolation is coarser during plot runs only.

Reproduce:

```bash
./scripts/setup_plots_venv.sh   # once
./scripts/run_c1_plots.sh --config-hash c1-118207fbc3eaba53 --out results/c1_g2_plots.md
# then copy harness PNGs if desired:
#   cp results/c1_raster.png results/plots/c1_g2_raster.png
#   cp results/c1_weights.png results/plots/c1_g2_weights.png
```

---

## Artifacts cited

| Path | Role |
|---|---|
| [`c1_g2.md`](c1_g2.md) | Canonical full-run results note (auto-emitted U-NEG stub) |
| [`c1_g2_plots.md`](c1_g2_plots.md) | Same hash / FAIL with plots feature `Written` |
| [`plots/c1_g2_raster.png`](plots/c1_g2_raster.png) | C1 local-assembly raster (canonical hash) |
| [`plots/c1_g2_weights.png`](plots/c1_g2_weights.png) | C1 local-assembly weight trace (canonical hash) |
| [`runs/2026-07-22/c1_g2_full_v11.md`](runs/2026-07-22/c1_g2_full_v11.md) | Protocol-v2 full scientific run |
| [`runs/2026-07-22/c1_g2_full_v11_replay.md`](runs/2026-07-22/c1_g2_full_v11_replay.md) | Identical replay |
| [`runs/2026-07-22/c1_g2_quick_v11.md`](runs/2026-07-22/c1_g2_quick_v11.md) | Pilot only (not a G2 decision) |
| [`BUILD_AUDIT_v11_RESOLUTION.md`](BUILD_AUDIT_v11_RESOLUTION.md) | v11 harness fixes + verification |
| [`SENSITIVITY_PROTOCOLS.md`](SENSITIVITY_PROTOCOLS.md) | Tier-B protocol-v3 harness + interpretation rules |
| [`c1_sens_temporal_pc_full.md`](c1_sens_temporal_pc_full.md) | v3 temporal PC full run (`c1-a49deeaedb495a09`) |
| [`c1_sens_capacity_full.md`](c1_sens_capacity_full.md) | v3 capacity full run (`c1-d38d7644d8afc84b`) |
| [`OVERRIDE_FULL_RUNS.md`](OVERRIDE_FULL_RUNS.md) | C2–R2 override full-run index (exploratory) |
| [`POST_G2_BUILD.md`](POST_G2_BUILD.md) | U18–U23 / G5 override engineering index |

---

## One-line claim (citation form)

> Under BINN C1 protocol v2 config hash `c1-118207fbc3eaba53` (n=20), Gate G2 FAIL: gap-closed LCB −0.0048 (need >0.5) and local accuracy 0.4912 (need ≥0.65); harness valid (PC 0.9488, sparsity 0.0156); replay identical; refs succeed on same splits while local/dense three-factor stay at chance including param-matched dense; protocol-v3 temporal-PC and capacity sensitivities also FAIL under distinct hashes (harden, do not reopen v2 or license P3+); owner-requested C2–R2 / U18–U23 override full runs are exploratory under `--enable-*` only and do not reopen `c1-118207fbc3eaba53` or license P3+ as default; program stops before P3+.
