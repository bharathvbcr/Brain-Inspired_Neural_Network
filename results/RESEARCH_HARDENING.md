# BINN research hardening — checklist closure

Authority: Rust sources + on-disk notes produced by the commands below. Numbers are from those runs only.

**Does not reopen** protocol-v2 hash `c1-118207fbc3eaba53`. Matched-arch remains the primary scientific claim; engine C1 (v2 / v5 / v6 / v7) is secondary.

---

## Object-under-test checklist (before → after)

| Item | Before | After |
|---|---|---|
| Event LIF + dendrites exist | yes | yes (unchanged) |
| STDP eligibility + three-factor algebra live | yes | yes (unchanged) |
| Trial-isolated `last_spike` / full membrane reset | yes under `c1-iso` / `c1x-iso-*` | yes (unchanged; also inherited by spike/project) |
| Natural hidden spiking during C1 integrate | **no** (θ=∞) | **yes under `c1-spike-*` (v6) and `c1-spike-s-*` (v9)** |
| Assembly Calculus `project` on C1 | **no** | **yes under `c1-project-*` (protocol v7)** |
| True e-prop on exact-forward | **no** (hybrid STDP×M only) | **yes under `c1x-eprop-true-*`** |
| C3 matched = real BPTT | **no** (oracle pulses) | **yes under `c3-bptt-*` (SuperSpike BPTT arm)** |
| Brain model | **no** | **no** (explicit non-claim) |

---

## What changed (code)

### 1. Natural hidden spiking — protocol v6 / `c1-spike-*`

| Surface | Change |
|---|---|
| `config.rs` | `C1_SPIKE_PROTOCOL_VERSION=6`, `c1_spike()` / `c1_spike_quick()`, `is_spike_protocol()` |
| `runner.rs` | Gate θ=∞ mute behind `!natural_spiking`; spike protocol also applies isolation resets |
| `experiments/c1.rs` | `--spike` / `--natural-spike` |

**Tests:** `natural_spiking_path_allows_hidden_spikes_before_wta_mute_does_not`, `spike_protocol_hash_distinct_and_render_discloses`, `spike_presets_use_protocol_v6_and_diverge_from_v2_and_iso`.

**Minted hashes:** scientific `c1-09442acdbdc0c752`; quick `c1-d6b811cec7feed26`.

### 1b. Calibrated natural spiking — protocol v9 / `c1-spike-s-*` (one honest attempt)

| Surface | Change |
|---|---|
| `config.rs` | `C1_SPIKE_S_PROTOCOL_VERSION=9`, `c1_spike_s()` / `c1_spike_s_quick()`, knobs `init_w=0.22`, `eta=0.45`, `tau_e=48` |
| `runner.rs` | Spike-count k-WTA under `spike_count_wta`; disclosed multi-frame PC; readout boost 1.35/init_w |
| `experiments/c1.rs` | `--spike-s` / `--calibrated-spike` |

**Diagnosis (v6 PC collapse):** membrane-score k-WTA reads residual `v` after integrate; finite-θ LIF reset zeroes well-driven cells, so class-selective winners are lost (mute θ=∞ had prevented this).

**Honest knobs (production path + disclosed PC):** spike-count WTA (not mute); `init_w`/`eta`/`tau_e`; higher readout boost; multi-frame easy PC (main coincidence task unchanged). **No** θ=∞ on learner path; **no** G2 threshold changes.

**Tests:** `spike_s_presets_use_protocol_v9_and_diverge_from_v6_and_v2`, `spike_s_protocol_hash_distinct_and_render_discloses`, `calibrated_spike_s_positive_control_floor_on_quick`.

**Minted hashes:** scientific `c1-c3e47b1e5f564df6`; quick `c1-078cdbd91088c2f6`.

**Outcome:** scientific still **INVALID_HARNESS** (PC **0.8413** < 0.90; sparsity in-band). Stopped after one calibration attempt — no threshold massage. See [`c1_spike_s.md`](c1_spike_s.md).

### 2. Assembly Calculus `project` — protocol v7 / `c1-project-*`

| Surface | Change |
|---|---|
| `config.rs` | `C1_PROJECT_PROTOCOL_VERSION=7`, `c1_project()` / `c1_project_quick()` |
| `runner.rs` | `use_project` path calls `binn_areas::project`; `C1_PROJECT_INVOKE_COUNT` diagnostic |
| `experiments/c1.rs` | `--project` / `--ac-project` |

**Tests:** `project_protocol_invokes_assembly_project_and_discloses`, `project_presets_use_protocol_v7_and_diverge`.

**Minted hashes:** scientific `c1-8cc19eccba9c70aa`; quick `c1-41458c2941a9d96e`.

### 3. True surrogate e-prop — `c1x-eprop-true-*`

| Surface | Change |
|---|---|
| `eprop_true_config.rs` | New family; protocol version 8; arms `true-surrogate-eprop` + hybrid contrast |
| `runner_eprop_true.rs` | Exact-forward substrate; σ′×pre eligibility (no STDP absorb) vs hybrid STDP×M |
| `experiments/credit_assignment.rs` | `--true-eprop` |

**Tests:** `true_eprop_does_not_use_stdp_absorb`, hash divergence from frozen `c1x-eprop-exact-forward-fcedc76a80ff0f0e`, `quick_run_finishes`.

**Scientific hashes:**
- `c1x-eprop-true-true-surrogate-eprop-0e2aeb90d68ac5f9`
- `c1x-eprop-true-hybrid-stdp-eprop-92333bf4bd223098`

Frozen hybrid `c1x-eprop-exact-forward-fcedc76a80ff0f0e` untouched.

### 4. C3 SuperSpike BPTT — `c3-bptt-*`

| Surface | Change |
|---|---|
| `c3_bptt_config.rs` | New family; arms `superspike-bptt` + labeled `oracle-pulses` contrast |
| `runner_c3_bptt.rs` | Surrogate BPTT through depth traces (no oracle pulses on BPTT arm) |
| `experiments/c3_production.rs` | `--bptt-reference` |

**Tests:** `bptt_path_skips_oracle_pulses_oracle_path_uses_them`, hashes distinct from `c3v2-*`.

**Scientific hashes:**
- `c3-bptt-superspike-bptt-a1efec9cf8a24968`
- `c3-bptt-oracle-pulses-fc574f1d7c8c8d4f`

Frozen `c3v2-*` untouched.

---

## Commands run

Workspace root: `binn/`.

```bash
cargo test --locked -p binn-lab -p binn-learn --lib

cargo run --locked --release -p binn-lab --bin c1 -- --spike --quick --out results/c1_spike_quick.md
cargo run --locked --release -p binn-lab --bin c1 -- --spike --out results/c1_spike.md
cargo run --locked --release -p binn-lab --bin c1 -- --spike-s --quick --out results/c1_spike_s_quick.md
cargo run --locked --release -p binn-lab --bin c1 -- --spike-s --out results/c1_spike_s.md
cargo run --locked --release -p binn-lab --bin c1 -- --project --quick --out results/c1_project_quick.md
cargo run --locked --release -p binn-lab --bin c1 -- --project --out results/c1_project.md

cargo run --locked --release -p binn-lab --bin credit-assignment -- --true-eprop --quick --out results/credit_eprop_true_quick.md
cargo run --locked --release -p binn-lab --bin credit-assignment -- --true-eprop --out results/credit_eprop_true.md

cargo run --locked --release -p binn-lab --bin c3-production -- --enable-c3-v2 --bptt-reference --quick --out results/c3_bptt_quick.md
cargo run --locked --release -p binn-lab --bin c3-production -- --enable-c3-v2 --bptt-reference --out results/c3_bptt.md
```

---

## Experiment outcomes (this session)

### Natural spiking — `c1-09442acdbdc0c752`

- Verdict: **INVALID_HARNESS** (PC mean **0.7738** < 0.90; sparsity **0.0089** in-band)
- local **0.4850**, dense **0.4250**, gradient-ref **0.8438**, elig-ref **1.0000**
- Framing: finite-θ path is wired and adversarially tested; harness PC fails under natural spiking + isolation — **not** a G2 PASS/FAIL object and **not** a reinterpretation of v2.
- Quick: `c1-d6b811cec7feed26` also INVALID_HARNESS (PC 0.7833)
- Notes: [`c1_spike.md`](c1_spike.md), [`c1_spike_quick.md`](c1_spike_quick.md)

### Calibrated natural spiking — `c1-c3e47b1e5f564df6` (protocol v9)

- Verdict: **INVALID_HARNESS** (PC mean **0.8413** < 0.90; sparsity **0.0074** in-band)
- local **0.4700**, dense **0.4250**, gradient-ref **0.9387**, elig-ref **1.0000**
- gap-closed mean **0.0927**, LCB **0.0511** (informational only; harness invalid)
- Quick: `c1-078cdbd91088c2f6` PILOT with PC **1.0000** (floor clears on short schedule; not scientific)
- Framing: one honest calibration (spike-count WTA + knobs + disclosed multi-frame PC) improved PC vs v6 but did **not** clear 0.90; **stopped** — no threshold massage; not PASS/FAIL.
- Notes: [`c1_spike_s.md`](c1_spike_s.md), [`c1_spike_s_quick.md`](c1_spike_s_quick.md), [`C1_SPIKE_S_CALIBRATION.md`](C1_SPIKE_S_CALIBRATION.md)

### Assembly `project` — `c1-8cc19eccba9c70aa`

- Verdict: **FAIL** (valid harness: PC **0.9163**, sparsity **0.0156**)
- local **0.5000**, dense **0.5000**, gradient-ref **0.8438**, elig-ref **1.0000**
- gap-closed mean **0.0000**, LCB **0.0000**
- Quick: `c1-41458c2941a9d96e` INVALID_HARNESS (PC 0.8000) — pilot only
- Notes: [`c1_project.md`](c1_project.md), [`c1_project_quick.md`](c1_project_quick.md)

### True e-prop — scientific

- true-surrogate mean accuracy **0.7125** (`c1x-eprop-true-true-surrogate-eprop-0e2aeb90d68ac5f9`)
- hybrid-stdp contrast **0.7350** (`c1x-eprop-true-hybrid-stdp-eprop-92333bf4bd223098`)
- Does **not** reopen frozen hybrid `c1x-eprop-exact-forward-fcedc76a80ff0f0e`
- Note: [`credit_eprop_true.md`](credit_eprop_true.md)

### C3 BPTT — scientific

- Verdict: **MEASURED**
- `superspike-bptt` D\*=**4** (`c3-bptt-superspike-bptt-a1efec9cf8a24968`)
- `oracle-pulses` D\*=**8** (`c3-bptt-oracle-pulses-fc574f1d7c8c8d4f`) — labeled **not BPTT**
- Does **not** reopen frozen `c3v2-*`
- Note: [`c3_bptt.md`](c3_bptt.md)

### Prior anchors (unchanged)

| Hash | Role |
|---|---|
| `c1-118207fbc3eaba53` | Canonical v2 — **immutable** |
| `c1-8ec031907a3426d0` | Isolation scientific (protocol v5) |
| `c1-match-5dc6822e71229e9e` | Matched-arch scientific (primary claim) |

---

## Claim strength after checklist closure

| Claim | Change |
|---|---|
| **Primary — matched-arch broadcast insufficiency** | Unchanged; still lead claim (`c1-match-5dc6822e71229e9e`). |
| **Secondary — engine pipeline negative** | Stronger: isolation (v5) + project (v7 FAIL) + spike path disclosed. |
| **True e-prop / C3 BPTT** | Supporting methods notes only; new hash families; do not widen to biology. |

---

## Still must not claim

1. Biology / cortex / brain equivalence.
2. That natural-spiking INVALID_HARNESS (v6 or calibrated v9) is a G2 FAIL/PASS (it is not).
3. That AC `project` PASS — scientific project is **FAIL**.
4. That true e-prop rescues v2 or matched-arch.
5. That C3 oracle-pulses arm is BPTT (it is the labeled contrast).
6. Reopening or threshold-massaging `c1-118207fbc3eaba53`.
7. Neuromorphic hardware; impossibility in principle.

---

## Residual gaps / risks

- **Natural-spiking harness:** v6 PC ~0.77; calibrated v9 PC **0.8413** still < 0.90 after one honest attempt — do not massage thresholds; further work would need a new protocol family (not reinterpreting these hashes).
- **Project path** still uses forced readout spikes + three-factor after `project`; Hebbian imprint inside `project` coexists with three-factor (disclose).
- **True e-prop** is surrogate-on-exact-forward (σ′×pre), not Bellec online e-prop with eligibility traces through delayed event graph — disclose naming carefully.
- **C3 BPTT** is SuperSpike-style surrogate BPTT on production depth graph; still exploratory post-G2.
- Matched-arch remains the cleanest publishable negative about **broadcast credit topology**.
