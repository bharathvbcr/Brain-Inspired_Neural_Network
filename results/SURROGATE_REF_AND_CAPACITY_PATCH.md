# Patch: same-architecture surrogate reference + capacity-scaled C1

**Date:** 22 July 2026
**Status:** DRAFT — written without a Rust toolchain in the review sandbox. Every
edit below must be applied and confirmed with `cargo build/test/clippy` on a host
before it is trusted. See the checklist at the end.

**Goal.** Make the Gate-G2 negative *publishable* rather than *ambiguous* by
removing two remaining confounds the v9 audit could not fix in code:

1. **Cross-family reference.** The `gradient-reference` condition is a `tanh` RNN
   (`bptt_baseline.rs`, hard-coded `H=4`), a different model family from the LIF
   substrate. `gap_closed = (local − dense)/(reference − dense)` therefore mixes
   "local rule is weak" with "spiking ≠ real-valued RNN." Fix: a gradient
   reference on the **same LIF dynamics** — new file
   `binn-learn/src/surrogate_lif_baseline.rs` (already drafted).
2. **Starved capacity.** The C1 default gives the readout only `k_wta = 2` active
   cells out of `n_hidden = 12`. In the last run local accuracy was pinned to
   exactly `0.0` or `0.5` every seed — a constant/coin-flip readout, i.e. no
   usable code, not a fair test of the rule. Fix: a `c1_capacity()` preset with
   room for the **positive control** to pass (the gating diagnostic).

---

## 1. New module (already written, additive, safe)

`binn-learn/src/surrogate_lif_baseline.rs` implements `SurrogateLifReference`:
recurrent LIF layer (leak `α = exp(−1/τ_m)`, `θ = THETA_REST`, reset `V_RESET`,
all imported from `binn-engine`) + rate readout, trained by SuperSpike
surrogate-gradient BPTT. It reuses `GradientExample` / `GradientReferenceReport`
so it is a **drop-in** for `BpttBaseline` in the runner's data path. It is a
`*_baseline.rs` file, so GC1 auto-exempts its `backward(`/`matmul` symbols.

Because it is not yet declared as a `mod`, it does not compile into the crate and
cannot break the current green build until you opt in with step 2.

Bundled tests: production-ban header, shared-LIF-constants, finite-difference
gradient check on the readout, `learns_coincidence_above_chance` (≥ 0.65), and a
same-seed determinism check (feeds GC3).

## 2. Activate it — `binn-learn/src/lib.rs`

Add the module and re-export next to the existing `bptt_baseline` lines:

```rust
pub mod bptt_baseline;
pub mod eligibility;
pub mod modulators;
pub mod surrogate_lif_baseline;   // NEW
pub mod three_factor;

pub use bptt_baseline::{
    BpttBaseline, GradientExample, GradientReferenceReport, REFERENCE_SEQUENCE_LEN,
};
pub use eligibility::{decay, stdp, Eligibility};
pub use modulators::Modulators;
pub use surrogate_lif_baseline::{
    SurrogateLifReference, DEFAULT_SURROGATE_BETA, SURROGATE_LIF_REFERENCE_LABEL, // NEW
};
pub use three_factor::{Learner, ThreeFactor};
```

## 3. Capacity-scaled preset — `binn-learn/../binn-lab/src/config.rs`

Add two fields so "which reference" and "surrogate steepness" are hashed
(reproducibility), then a preset. **Adding fields means updating `hash()`** — do
not skip those two `mix(...)` lines or two different configs could collide.

Add to `struct Config`:

```rust
    /// Use the same-architecture surrogate-LIF gradient reference instead of the
    /// tanh-RNN `BpttBaseline`. Makes `gap_closed` a within-architecture metric.
    pub use_surrogate_lif_reference: bool,
    /// Surrogate steepness β for the LIF reference (ignored when the flag is off).
    pub surrogate_beta: f32,
```

Add to `hash()` (order does not matter, but must be present for every field):

```rust
        mix(&mut h, u64::from(self.use_surrogate_lif_reference));
        mix(&mut h, self.surrogate_beta.to_bits() as u64);
```

Set them in `c1_default()` (keep current science unchanged — flag OFF so the
existing hash logic for old presets is only extended, not silently altered):

```rust
            use_surrogate_lif_reference: false,
            surrogate_beta: 5.0,
```

Mirror the same two lines in `c1_quick()` (it derives from `c1_default()`, so no
change needed there). Then add the new preset:

```rust
    /// Capacity-scaled C1: wider hidden area and a larger k-winner budget so the
    /// readout sees a usable code, more training exposure, and the
    /// same-architecture surrogate-LIF reference. Sized so the positive/sanity
    /// control has room to pass; if it still floors, the bottleneck is the
    /// encoder/thresholds, not capacity.
    pub fn c1_capacity() -> Self {
        let mut c = Self::c1_default();
        c.master_seed = 0xC1CA_0000_0001;   // distinct seed lineage from default
        c.n_hidden = 32;                     // 12 → 32: richer hidden code
        c.k_wta = 4;                         // 2 → 4: readout sees 4 active bits
        c.p_sparse = 0.30;                   // keep assemblies sparse at N=32
        c.n_train = 200;                     // 80 → 200: more exposure for the local rule
        c.n_test = 100;                      // tighter accuracy estimate
        c.eta = 0.20;                        // smaller step for the larger fan-in
        c.bptt_epochs = 150;                 // reference gets a fair budget too
        c.bptt_lr = 0.02;
        c.use_surrogate_lif_reference = true;
        c.surrogate_beta = 5.0;
        // sequence_len stays 8 / max_lag 1 (the reference forward is fixed at T=8).
        c
    }
```

Extend the `from_hash` round-trip and its test to include `c1_capacity()` if you
want hash reproduction for the new preset (optional but recommended).

## 4. Route the reference — `binn-lab/src/runner.rs`

Only `run_bptt` needs to branch. Replace its body so it picks the reference the
config selected; the **data path (`gradient_examples`) is unchanged**, so both
references see identical per-seed examples.

```rust
use binn_learn::SurrogateLifReference; // add to the existing binn_learn use-list

fn run_bptt(config: &Config, seed: u64) -> CondOutcome {
    assert_eq!(
        config.sequence_len, REFERENCE_SEQUENCE_LEN,
        "C1 gradient reference currently requires sequence_len={REFERENCE_SEQUENCE_LEN}"
    );
    let train = gradient_examples(config, seed, config.n_train);
    let test = gradient_examples(config, seed ^ 0x7E57_0001, config.n_test);

    let report = if config.use_surrogate_lif_reference {
        // Same-architecture ceiling: hidden width matches the substrate.
        let mut r = SurrogateLifReference::new(
            config.n_hidden, config.bptt_lr, config.surrogate_beta, seed,
        );
        r.train_and_evaluate(config.bptt_epochs, &train, &test)
    } else {
        let mut r = BpttBaseline::new(config.bptt_lr, seed);
        r.train_and_evaluate(config.bptt_epochs, &train, &test)
    };

    CondOutcome {
        accuracy: report.accuracy,
        activity_sparsity: 1.0,
        raster_t: Vec::new(),
        raster_cell: Vec::new(),
        weight_steps: Vec::new(),
        weight_trace: Vec::new(),
    }
}
```

Update the condition-meaning row in `render_results_markdown` so the note states
which ceiling was used (read `config.use_surrogate_lif_reference`), e.g. append
"same-architecture surrogate-LIF" vs "tanh-RNN reference".

## 5. Optional: make the C1 binary select the preset

In `binn-lab/experiments/c1.rs`, pick `Config::c1_capacity()` when e.g.
`--capacity` is passed (or an env var), so both the legacy default and the
capacity run are reproducible from one binary and each writes its own
config-hashed note.

---

## Why these specific capacity numbers

The observed failure mode was not "local rule loses to the reference" — it was
"local readout carries no signal" (accuracy ∈ {0.0, 0.5} exactly). Three levers
address that directly:

- **`n_hidden` 12 → 32, `k_wta` 2 → 4.** With `k = 2` the readout integrates two
  spikes; near-degenerate. Four winners over a 32-cell population give the
  three-factor readout a code with enough distinct active-set patterns to
  separate coincident vs non-coincident trials.
- **`n_train` 80 → 200.** Online three-factor learning needs more exposures than
  a batched gradient loop; 80 trials is thin for a credit-assignment rule.
- **`eta` 0.35 → 0.20.** Larger fan-in (more presynaptic winners) raises the
  per-update weight change; a smaller step keeps the readout from saturating.

The **positive control is the gate on all of this.** After the change, read
`positive_control_mean` first:

- **High (≳ 0.9):** the harness can learn an easy mapping at this capacity, so a
  low `local-assembly` accuracy is now a *real* negative — publishable.
- **Still low:** capacity was not the bottleneck. Look at the encoder
  (`LatencyEncoder`) and the k-WTA thresholding in `run_trial`, not the config.

---

## Host verification checklist (must pass before trusting any number)

```bash
cd binn
cargo build --locked --workspace
cargo test  --locked -p binn-learn surrogate_lif        # new module's tests
cargo test  --locked --workspace                        # full suite still green
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
./scripts/gc_checks.sh                                  # GC1 must still pass (file is *_baseline.rs)
cargo run -p binn-lab --bin c1                          # regenerate results/c1_g2.md
```

Then inspect the regenerated `results/c1_g2.md`:
1. `positive_control_mean` high → verdict is interpretable.
2. `gradient-reference` label reads `SURROGATE_LIF_GRADIENT_REFERENCE`.
3. `dense-local` sits near ~0.5 (fair plateau), not 0.0.
4. Whatever the G2 verdict, it is now a clean within-architecture comparison.

---

## Further improvements (beyond this patch, ranked)

1. **Fold the surrogate-LIF determinism test into GC3.** Add its check to
   `scripts/check_gc3.sh` so the reference is covered by the same-seed fingerprint
   gate like every other stateful path.
2. **Generalize the reference to `config.sequence_len`.** Both references are
   hard-wired to `T = 8` via fixed `[f32; 8]` arrays; a `Vec`-based example type
   would let C1 vary lag/length and stress the temporal-credit claim (relevant to
   U15/C3 later).
3. **Report `work_per_accuracy` for real.** The runner currently emits
   `work_metric_omitted_until_measured_counters`. Wiring the engine's event
   counters into `Metrics::work_per_accuracy` turns G5's efficiency claim from
   aspirational into measured, and gives the G2 note an honest cost axis.
4. **Add a shuffled-label control.** A fourth condition with permuted labels
   pins the true empirical chance rate per seed and catches any residual leakage
   the balance fix might miss.
5. **Sweep `k_wta` as a curve, not a point.** One `k` is a single sample of a
   capacity axis; a small sweep (2/4/8) shows whether local learning is
   capacity-limited or rule-limited — the difference between "scale it up" and
   "the thesis fails," which is exactly what G2 is meant to decide.
