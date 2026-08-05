# Tier-B sensitivity protocols (optional)

**Status:** harness + unit/integration tests + **full n=20 scientific runs recorded**.
These are protocol-v3 sensitivity probes — they do **not** reopen Gate G2 under
protocol v2, and they do **not** mutate or alias kill-gate hash
`c1-118207fbc3eaba53`. Scoped U-NEG (v2 + folded v3): [`U-NEG_protocol_v2.md`](U-NEG_protocol_v2.md).

## What these are

Optional confound checks listed in the remaining-work plan. Each uses scientific
**protocol version 3** (`C1_SENSITIVITY_PROTOCOL_VERSION`) mixed into the config
hash so results never alias the canonical kill-gate:

| Preset | Experiment name | Protocol | Config hash | Purpose |
|---|---|---:|---|---|
| Canonical C1 (unchanged) | `c1` | 2 | `c1-118207fbc3eaba53` | Kill-gate |
| Temporal PC (full) | `c1-sens-temporal-pc` | 3 | `c1-a49deeaedb495a09` | Coincidence-lag PC under same encoding |
| Temporal PC (quick/PILOT) | `c1-sens-temporal-pc` | 3 | `c1-097696ca34d8a34d` | CI / smoke for temporal PC |
| Capacity (full) | `c1-sens-capacity` | 3 | `c1-d38d7644d8afc84b` | Richer `k_wta` / `n_train` schedule |
| Capacity (quick/PILOT) | `c1-sens-capacity` | 3 | `c1-e519403aff33b384` | CI / smoke for capacity |

Protocol-v2 defaults (spatial feature-presence positive control, `N=128`, `k=2`,
`n_train=80`) are untouched.

## Full scientific runs (n=20, not `--quick`)

Recorded 2026-07-23 via `cargo build -p binn-lab --bin c1 --release` then
`./target/release/c1 --sensitivity …`. Both runs are protocol **v3**; neither
reopens kill-gate hash `c1-118207fbc3eaba53`.

| Preset | Result note | Hash | Verdict | PC mean | Local acc | Gap LCB | Notes |
|---|---|---|---|---:|---:|---:|---|
| Temporal PC | [`c1_sens_temporal_pc_full.md`](c1_sens_temporal_pc_full.md) | `c1-a49deeaedb495a09` | **FAIL** | 0.9675 | 0.5263 | −0.0118 | Temporal coincidence-lag PC **clears** (≥0.9); G2 still FAIL |
| Capacity | [`c1_sens_capacity_full.md`](c1_sens_capacity_full.md) | `c1-d38d7644d8afc84b` | **FAIL** | 1.0000 | 0.6775 | 0.0000 | Local mean clears 0.65, but dense-local=0.9400 so gap-closed=0; not a capacity reclassification PASS |

**Scoped interpretation (v3 ≠ v2 reopen; scientific ≠ PILOT):**

1. **Temporal PC cleared** the harness floor (0.9675 ≥ 0.9) under the same
   LatencyEncoder / local path, yet local still missed accuracy and gap gates →
   **hardens** the scoped U-NEG (local can learn coincidence lag; still fails G2).
2. **Capacity changed the local/gap story** (local 0.6775 vs canonical ~0.49;
   dense jumps to 0.94) but did **not** produce a sensitivity PASS (gap LCB=0).
   The negative is **not** reclassified as a schedule/front-end capacity confound.
3. Neither result is a new protocol-v2 G2 decision. Do **not** schedule P3+ from
   these probes alone.

Quick/PILOT counterparts remain harness-only (`*_quick.md`); do not cite them as
scientific PASS/FAIL.

## How to run

From the `binn/` workspace:

```bash
# Temporal coincidence-lag positive-control sensitivity (full schedule)
cargo run -p binn-lab --release --bin c1 -- --sensitivity temporal-pc \
  --out results/c1_sens_temporal_pc_full.md

# Capacity schedule sensitivity (full schedule)
cargo run -p binn-lab --release --bin c1 -- --sensitivity capacity \
  --out results/c1_sens_capacity_full.md
# equivalent: --capacity

# PILOT / smoke (5 seeds, short train) — preferred for CI and quick checks
cargo run -p binn-lab --bin c1 -- --sensitivity temporal-pc --quick \
  --out results/c1_sens_temporal_pc_quick.md
cargo run -p binn-lab --bin c1 -- --sensitivity capacity --quick \
  --out results/c1_sens_capacity_quick.md
```

Reproduce via printed hash:

```bash
cargo run -p binn-lab --release --bin c1 -- --config-hash <printed-c1-sens-hash>
```

Focused tests:

```bash
cargo test -p binn-lab --lib config::tests
cargo test -p binn-lab --lib runner::tests::temporal_positive_control
cargo test -p binn-lab --lib runner::tests::capacity_sensitivity
cargo test -p binn-lab --lib runner::tests::positive_control_floor
```

## Interpretation rules

1. **Do not claim a new G2 PASS** from a `--quick` / PILOT sensitivity run.
2. A sensitivity FAIL with temporal PC ≥ 0.9 strengthens the scoped U-NEG
   (local path can learn coincidence lag, yet still misses gates).
3. A sensitivity PASS reclassifies the v2 failure as schedule/front-end capacity
   — it is **not** an automatic P3 license without an explicit new G2 decision
   under a freshly preregistered protocol.
4. Never reuse or silently mutate hash `c1-118207fbc3eaba53`.
5. Full n=20 v3 FAIL (temporal PC clear; capacity no PASS) **hardens** the
   negative; it does not reopen or rewrite the canonical v2 kill-gate record.

## Capacity knobs (protocol v3)

Relative to live v2 defaults (`N=128`, `k=2`, `n_train=80`):

- `n_hidden` 128 → 256, `k_wta` 2 → 4 (keeps nominal activity ≈ 1.56%)
- `n_train` 80 → 200, `n_test` 40 → 100
- `eta` 0.35 → 0.20, `bptt_epochs` 80 → 150, `bptt_lr` 0.05 → 0.02
- `p_sparse` 0.35 → 0.30
- Distinct `master_seed` lineage `0xC1CA_0000_0001`

## Temporal PC knobs (protocol v3)

- Same substrate knobs as v2 defaults
- Distinct `master_seed` lineage `0xC17E_0000_0001`
- Harness positive control uses late-window short-lag coincidence (both features
  near decision) vs equal-count non-coincidence (feature 1 early, leaked by
  decision) under the same LatencyEncoder, instead of spatial feature-presence
