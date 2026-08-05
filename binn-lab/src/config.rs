//! Experiment config + stable config hashing (U13).

/// Prefix used when printing the default C1 config hash in docs / notes.
pub const C1_DEFAULT_HASH_PREFIX: &str = "c1-";

/// Scientific protocol version mixed into every C1 config hash.
///
/// Increment this whenever training, evaluation, control construction, or gate
/// semantics change without adding a new [`Config`] field.
pub const C1_PROTOCOL_VERSION: u64 = 2;

/// Tier-B optional sensitivity protocol (temporal PC / capacity).
///
/// Mixed into hashes for `c1-sens-*` presets only. Does **not** reopen or
/// replace the protocol-v2 kill-gate hash `c1-118207fbc3eaba53`.
pub const C1_SENSITIVITY_PROTOCOL_VERSION: u64 = 3;

/// Trial-isolation integrity protocol (clear `last_spike` + C3-style membrane reset).
///
/// Mixed into hashes for `c1-iso*` presets only. Does **not** reopen or
/// reinterpret protocol-v2 hash `c1-118207fbc3eaba53`.
pub const C1_ISOLATION_PROTOCOL_VERSION: u64 = 5;

/// Natural-hidden-spiking protocol (finite θ during integrate; no θ=∞ mute).
///
/// Mixed into hashes for `c1-spike*` presets only (not `c1-spike-s*`). Does
/// **not** reopen or reinterpret protocol-v2 hash `c1-118207fbc3eaba53`.
/// Historical scientific hash `c1-09442acdbdc0c752` remains INVALID_HARNESS.
pub const C1_SPIKE_PROTOCOL_VERSION: u64 = 6;

/// Assembly-Calculus `project` protocol (wires `binn_areas::project` into C1).
///
/// Mixed into hashes for `c1-project*` presets only. Does **not** reopen or
/// reinterpret protocol-v2 hash `c1-118207fbc3eaba53`.
pub const C1_PROJECT_PROTOCOL_VERSION: u64 = 7;

/// Calibrated natural-spiking protocol (spike-count k-WTA + PC knobs).
///
/// Mixed into hashes for `c1-spike-s*` presets only. Keeps finite θ on the
/// scientific learner path; does **not** reopen v2 or reinterpret v6
/// `c1-09442acdbdc0c752`. G2 accuracy/gap/PC thresholds unchanged.
pub const C1_SPIKE_S_PROTOCOL_VERSION: u64 = 9;

/// Live C1 opt-in `ReinforceFeedback` neuromodulator (matched-arch v12 family).
///
/// Mixed into hashes for `c1-rfb*` presets only (not `c1-rfb-em*`). Same k-WTA /
/// single-pass substrate as protocol-v2 C1; swaps broadcast ±1 for production
/// `ReinforceFeedback` × `reinforce_term`. Does **not** flip default C1,
/// remassage P4 spiking-DFA, or retune P5 `rl_graded`.
pub const C1_REINFORCE_FB_PROTOCOL_VERSION: u64 = 13;

/// Live RFB × epoch-matched exposure (protocol 14).
///
/// Same neuromodulator family as v13, but the local/dense arms train for a
/// disclosed multi-epoch loop over the frozen train split (isolates single-pass
/// handicap). Does **not** remassage v13 knobs / hash `c1-660401d74db3c88d`.
pub const C1_RFB_EPOCH_PROTOCOL_VERSION: u64 = 14;

/// Disclosed local-train epoch counts for protocol 14 (not a Config field —
/// baked into protocol semantics so v2 hashes stay frozen).
pub const C1_RFB_EPOCH_LOCAL_EPOCHS_SCIENTIFIC: usize = 20;
/// Quick/PILOT local-train epochs for protocol 14.
pub const C1_RFB_EPOCH_LOCAL_EPOCHS_QUICK: usize = 4;

/// Structured frozen feedback under k-WTA (protocol 15).
///
/// Same live RFB plasticity path, but `B_i` for hidden posts is
/// `sign(w→readout_1 − w→readout_0)` after readout boost (not Uniform[-1,1]).
/// Does **not** remassage v13 random-B FAIL.
pub const C1_STRUCTURED_FB_PROTOCOL_VERSION: u64 = 15;

/// Structured frozen B × epoch-matched (protocol 16).
///
/// Combines v15 `B` construction with v14 multi-epoch exposure. Fresh hash;
/// does **not** remassage v14/v15 fails in place.
pub const C1_STRUCTURED_FB_EPOCH_PROTOCOL_VERSION: u64 = 16;

/// Structured B × capacity schedule (protocol 17).
///
/// Same structured frozen `B` as v15, but on the Tier-B capacity substrate
/// (richer `k_wta` / `n_hidden` / `n_train`). Fresh hash; does **not** remassage
/// v15 or the capacity sensitivity FAIL (`c1-d38d7644d8afc84b`).
pub const C1_STRUCTURED_FB_CAPACITY_PROTOCOL_VERSION: u64 = 17;

/// Eligibility × REINFORCE co-design (protocol 18).
///
/// Structured frozen `B` (v15) plus eligibility timing co-designed with sampled
/// REINFORCE: longer `τ_e` spanning encode→winner→action→credit, and a mid-trial
/// eligibility absorb after winners/readout before the REINFORCE action spike.
/// Fresh hash; does **not** remassage v13–v17 FAILs in place.
pub const C1_ELIG_RFB_PROTOCOL_VERSION: u64 = 18;

/// Disclosed eligibility time constant for protocol 18 (ticks).
///
/// Default C1 uses `τ_e = 40`. Protocol 18 uses 4× so traces survive the
/// multi-frame integrate + delayed REINFORCE credit horizon.
pub const C1_ELIG_RFB_TAU_E: f32 = 160.0;

/// Structured B × restored target teach (protocol 19).
///
/// Same structured frozen `B` as v15, but incorrect trials apply a secondary
/// target update through `ReinforceFeedback::credit(+1)` (not observe-only).
/// Isolates whether RFB-family transfer fails partly from dropping the default
/// C1 teach path. Fresh hash; does **not** remassage v15.
pub const C1_STRUCTURED_FB_TEACH_PROTOCOL_VERSION: u64 = 19;

/// Live graded-DFA transfer onto muted-θ / k-WTA C1 (protocol 20).
///
/// Honest map of matched DFA credit (graded error × fixed-random feedback) onto
/// the live C1 substrate that keeps θ=∞ mute + hard k-WTA. Fresh hash; does
/// **not** remassage matched `c1-dfa-*`, P4 spiking-DFA, or v13–v19.
pub const C1_DFA_LIVE_PROTOCOL_VERSION: u64 = 20;

/// Soft/relaxed k-WTA × structured frozen B (protocol 21).
///
/// Same structured `B` as v15, but hidden winners are sampled with disclosed
/// temperature soft k-WTA (`T = `[`C1_SFB_SOFT_TEMPERATURE`]). One temp; no
/// grid. Fresh hash; does **not** remassage v15.
pub const C1_STRUCTURED_FB_SOFT_PROTOCOL_VERSION: u64 = 21;

/// Disclosed soft-WTA temperature for protocol 21 (not a Config field).
pub const C1_SFB_SOFT_TEMPERATURE: f32 = 1.0;

/// Finite-θ (mute off) under structured B (protocol 23).
///
/// Same structured frozen `B` as v15, but hidden thresholds stay finite during
/// integrate (no θ=∞ mute) with trial-isolation resets. Integrity / motif
/// ablation of the mute confounder under SFB credit. Fresh hash; does **not**
/// remassage v15 or reopen spike-PC remassages.
pub const C1_STRUCTURED_FB_FINTH_PROTOCOL_VERSION: u64 = 23;

/// Continuous / normalized structured B (protocol 24).
///
/// Same live RFB path as v15, but hidden `B_i ∝ (w→r1 − w→r0)` normalized by
/// the L2 norm of hidden Δw (not sign-truncated). One construction; no
/// hypersearch. Fresh hash; does **not** remassage v15.
pub const C1_STRUCTURED_FB_CONT_PROTOCOL_VERSION: u64 = 24;

/// Online Learned B_i live transfer (protocol 25).
pub const C1_RFB_LEARNED_PROTOCOL_VERSION: u64 = 25;

/// Adaptive k-WTA schedule protocol (protocol 28, k: 16 -> 2).
pub const C1_K_ANNEAL_PROTOCOL_VERSION: u64 = 28;

/// Experiment-name prefix that marks a Tier-B sensitivity preset.
pub const C1_SENSITIVITY_EXPERIMENT_PREFIX: &str = "c1-sens";

/// Experiment-name prefix that marks a trial-isolation integrity preset.
pub const C1_ISOLATION_EXPERIMENT_PREFIX: &str = "c1-iso";

/// Experiment-name prefix that marks a natural-hidden-spiking preset.
pub const C1_SPIKE_EXPERIMENT_PREFIX: &str = "c1-spike";

/// Experiment-name prefix that marks a calibrated natural-spiking preset.
pub const C1_SPIKE_S_EXPERIMENT_PREFIX: &str = "c1-spike-s";

/// Experiment-name prefix that marks an Assembly-Calculus `project` preset.
pub const C1_PROJECT_EXPERIMENT_PREFIX: &str = "c1-project";

/// Experiment-name prefix that marks live C1 `ReinforceFeedback` opt-in (v13).
pub const C1_REINFORCE_FB_EXPERIMENT_PREFIX: &str = "c1-rfb";

/// Experiment-name prefix that marks live RFB × epoch-matched (v14).
pub const C1_RFB_EPOCH_EXPERIMENT_PREFIX: &str = "c1-rfb-em";

/// Experiment-name prefix that marks structured frozen feedback (v15).
pub const C1_STRUCTURED_FB_EXPERIMENT_PREFIX: &str = "c1-sfb";

/// Experiment-name prefix that marks structured B × epoch-matched (v16).
pub const C1_STRUCTURED_FB_EPOCH_EXPERIMENT_PREFIX: &str = "c1-sfb-em";

/// Experiment-name prefix that marks structured B × capacity (v17).
pub const C1_STRUCTURED_FB_CAPACITY_EXPERIMENT_PREFIX: &str = "c1-sfb-cap";

/// Experiment-name prefix that marks eligibility × REINFORCE (v18).
pub const C1_ELIG_RFB_EXPERIMENT_PREFIX: &str = "c1-elig-rfb";

/// Experiment-name prefix that marks structured B × target teach (v19).
pub const C1_STRUCTURED_FB_TEACH_EXPERIMENT_PREFIX: &str = "c1-sfb-teach";

/// Experiment-name prefix that marks live graded-DFA transfer (v20).
pub const C1_DFA_LIVE_EXPERIMENT_PREFIX: &str = "c1-dfa-live";

/// Experiment-name prefix that marks soft-WTA × structured B (v21).
pub const C1_STRUCTURED_FB_SOFT_EXPERIMENT_PREFIX: &str = "c1-sfb-soft";

/// Experiment-name prefix that marks finite-θ under structured B (v23).
pub const C1_STRUCTURED_FB_FINTH_EXPERIMENT_PREFIX: &str = "c1-sfb-finth";

/// Experiment-name prefix that marks continuous structured B (v24).
pub const C1_STRUCTURED_FB_CONT_EXPERIMENT_PREFIX: &str = "c1-sfb-cont";

/// Experiment-name prefix that marks online learned B_i transfer (v25).
pub const C1_RFB_LEARNED_EXPERIMENT_PREFIX: &str = "c1-rfb-learned";

/// Experiment-name prefix that marks adaptive k-WTA schedule (v28).
pub const C1_K_ANNEAL_EXPERIMENT_PREFIX: &str = "c1-k-anneal";

/// Public, hashable C1 / harness configuration.
///
/// Identical field values under the same protocol version ⇒ identical
/// [`Config::hash`] (GC3). Changing any scientific knob or protocol version
/// changes the hash so a results note can cite it.
#[derive(Clone, Debug, PartialEq)]
pub struct Config {
    /// Experiment name (`"c1"`, …).
    pub experiment: String,
    /// Master seed; per-seed runs use `master_seed ^ seed_i`.
    pub master_seed: u64,
    /// Number of independent seeds (five is pilot-only; default uses twenty).
    pub n_seeds: usize,
    /// Coincidence sequence length (frames).
    pub sequence_len: usize,
    /// Max lag counted as coincident.
    pub max_lag: usize,
    /// Hidden population size (assembly / dense share this N).
    pub n_hidden: usize,
    /// k-WTA cap for the local-assembly hidden area.
    pub k_wta: usize,
    /// Sparse intra-area edge probability (local-assembly).
    pub p_sparse: f32,
    /// Initial synaptic weight.
    pub init_w: f32,
    /// Three-factor `η`.
    pub eta: f32,
    /// Three-factor `λ`.
    pub lambda: f32,
    /// Eligibility `τ_e`.
    pub tau_e: f32,
    /// Training trials per seed.
    pub n_train: usize,
    /// Held-out test trials per seed.
    pub n_test: usize,
    /// BPTT / surrogate-LIF epochs (labeled gradient reference).
    pub bptt_epochs: usize,
    /// BPTT / surrogate-LIF learning rate.
    pub bptt_lr: f32,
    /// Required lower confidence bound on the fraction of gradient gap closed.
    pub g2_min_gap_closed: f32,
    /// Absolute local-assembly accuracy floor for a positive verdict.
    pub g2_min_accuracy: f32,
    /// Critical value used for the preregistered lower confidence bound.
    pub g2_confidence_z: f32,
    /// Minimum mean positive-control accuracy for a valid harness (else INVALID_HARNESS).
    pub g2_min_positive_control: f32,
    /// Minimum (gradient_reference − dense) before a seed's gap_closed counts.
    ///
    /// Seeds with a weaker reference gap contribute `closed = 0`, closing the
    /// false-PASS route where a tiny denominator inflates the normalized gap.
    pub g2_min_reference_gap: f32,
    /// Inclusive lower bound on mean activity sparsity for a valid harness.
    pub activity_sparsity_min: f32,
    /// Inclusive upper bound on mean activity sparsity for a valid harness.
    pub activity_sparsity_max: f32,
    /// Minimum seeds required for a scientific (non-Pilot) G2 decision.
    pub scientific_n_seeds: usize,
    /// Preregistered prior σ of `gap_closed` for power analysis (hashed).
    pub power_sigma_prior: f32,
    /// Preregistered detectable effect size for power analysis (hashed).
    pub power_effect_size: f32,
    /// Use same-architecture surrogate-LIF as primary gradient reference.
    pub use_surrogate_lif_reference: bool,
    /// Surrogate steepness β (ignored when surrogate-LIF is off).
    pub surrogate_beta: f32,
    /// When true, re-run dense-local under a parameter-matched edge budget.
    pub matched_budget_repeat: bool,
    /// When true, use the short deterministic smoke schedule (CI / unit tests).
    pub quick: bool,
    /// Per-cell fan-out cap passed to [`binn_areas::WiringPrior::with_max_fan_out`].
    ///
    /// Default 256 matches the historical wiring prior. Hashed only for
    /// mac-probe experiments (see [`Self::is_mac_probe_geometry`]).
    pub max_fan_out: usize,
    /// When true: `init_w_eff = init_w * sqrt(REF_MEAN_FAN_IN / mean_fan_in)`.
    pub init_w_rescale: bool,
    /// When true: normalize readout boost × mean readout fan-in toward baseline.
    pub readout_gain_normalize: bool,
}

impl Config {
    /// Default C1 scientific config (≥20 seeds, ~1.56% activity, full trial counts).
    pub fn c1_default() -> Self {
        Self {
            experiment: "c1".into(),
            master_seed: 0xC160_0000_0001,
            n_seeds: 20,
            sequence_len: 8,
            max_lag: 1,
            n_hidden: 128,
            k_wta: 2,
            p_sparse: 0.35,
            init_w: 0.15,
            eta: 0.35,
            lambda: 0.002,
            tau_e: 40.0,
            n_train: 80,
            n_test: 40,
            bptt_epochs: 80,
            bptt_lr: 0.05,
            g2_min_gap_closed: 0.5,
            g2_min_accuracy: 0.65,
            g2_confidence_z: 1.96,
            g2_min_positive_control: 0.90,
            g2_min_reference_gap: 0.15,
            activity_sparsity_min: 0.005,
            activity_sparsity_max: 0.03,
            scientific_n_seeds: 20,
            power_sigma_prior: 0.15,
            power_effect_size: 0.10,
            use_surrogate_lif_reference: true,
            surrogate_beta: 5.0,
            matched_budget_repeat: true,
            quick: false,
            max_fan_out: 256,
            init_w_rescale: false,
            readout_gain_normalize: false,
        }
    }

    /// Short deterministic schedule for unit tests / CI (still ≥5 seeds).
    ///
    /// Keeps `k/N ≈ 0.015` (N=64, k=1) so sparsity stays in the scientific band.
    pub fn c1_quick() -> Self {
        let mut c = Self::c1_default();
        c.quick = true;
        c.n_seeds = 5;
        c.n_train = 24;
        c.n_test = 16;
        c.bptt_epochs = 40;
        c.n_hidden = 64;
        c.k_wta = 1;
        c.matched_budget_repeat = false;
        c
    }

    /// Tier-B capacity sensitivity (protocol v3): richer `k_wta` / `n_train`
    /// schedule from `results/SURROGATE_REF_AND_CAPACITY_PATCH.md`, adapted to
    /// the live N=128 baseline so nominal sparsity stays ~1.56%.
    ///
    /// Does **not** alter protocol-v2 defaults or hash `c1-118207fbc3eaba53`.
    pub fn c1_capacity_sensitivity() -> Self {
        let mut c = Self::c1_default();
        c.experiment = format!("{C1_SENSITIVITY_EXPERIMENT_PREFIX}-capacity");
        c.master_seed = 0xC1CA_0000_0001;
        // Patch intent: more winners + more exposure. Scale N with k so k/N
        // remains in the scientific sparsity band (4/256 ≈ 0.0156).
        c.n_hidden = 256;
        c.k_wta = 4;
        c.p_sparse = 0.30;
        c.n_train = 200;
        c.n_test = 100;
        c.eta = 0.20;
        c.bptt_epochs = 150;
        c.bptt_lr = 0.02;
        c
    }

    /// Quick/PILOT schedule for the capacity sensitivity (CI + smoke).
    pub fn c1_capacity_sensitivity_quick() -> Self {
        let mut c = Self::c1_capacity_sensitivity();
        c.quick = true;
        c.n_seeds = 5;
        c.n_train = 48;
        c.n_test = 24;
        c.bptt_epochs = 40;
        c.n_hidden = 128;
        c.k_wta = 2;
        c.matched_budget_repeat = false;
        c
    }

    /// Tier-B temporal positive-control sensitivity (protocol v3): harness PC
    /// is a fixed-position coincidence-lag task under the same LatencyEncoder
    /// + local spike/WTA path (not the spatial feature-presence PC of v2).
    ///
    /// Does **not** alter protocol-v2 defaults or hash `c1-118207fbc3eaba53`.
    pub fn c1_temporal_pc_sensitivity() -> Self {
        let mut c = Self::c1_default();
        c.experiment = format!("{C1_SENSITIVITY_EXPERIMENT_PREFIX}-temporal-pc");
        c.master_seed = 0xC17E_0000_0001; // temporal-PC sensitivity lineage
        c
    }

    /// Quick/PILOT schedule for the temporal-PC sensitivity.
    pub fn c1_temporal_pc_sensitivity_quick() -> Self {
        let mut c = Self::c1_temporal_pc_sensitivity();
        c.quick = true;
        c.n_seeds = 5;
        c.n_train = 128;
        c.n_test = 40;
        c.bptt_epochs = 40;
        // Keep nominal k/N in the scientific sparsity band (~1.56%).
        c.n_hidden = 128;
        c.k_wta = 2;
        c.eta = 0.45;
        c.matched_budget_repeat = false;
        c
    }

    /// Protocol-v5 trial-isolation schedule: same G2 thresholds / substrate as
    /// canonical C1, but clears `ThreeFactor.last_spike` and applies a
    /// C3-style full dynamic membrane reset at every trial boundary.
    ///
    /// Fresh experiment name + seed lineage ⇒ new config hash. Does **not**
    /// alter protocol-v2 defaults or hash `c1-118207fbc3eaba53`.
    pub fn c1_isolation() -> Self {
        let mut c = Self::c1_default();
        c.experiment = C1_ISOLATION_EXPERIMENT_PREFIX.into();
        c.master_seed = 0xC150_0000_0001;
        c
    }

    /// Quick/PILOT schedule for the trial-isolation protocol.
    pub fn c1_isolation_quick() -> Self {
        let mut c = Self::c1_isolation();
        c.quick = true;
        c.n_seeds = 5;
        c.n_train = 24;
        c.n_test = 16;
        c.bptt_epochs = 40;
        c.n_hidden = 64;
        c.k_wta = 1;
        c.matched_budget_repeat = false;
        c
    }

    /// Protocol-v6 natural-hidden-spiking schedule: same G2 thresholds / substrate
    /// as canonical C1, but keeps finite hidden θ during the integrate window
    /// (no θ=∞ mute). Also applies trial-isolation membrane + pairing resets so
    /// natural spikes are not confounded by sticky `last_spike`.
    ///
    /// Fresh experiment name + seed lineage ⇒ new config hash. Does **not**
    /// alter protocol-v2 defaults or hash `c1-118207fbc3eaba53`.
    ///
    /// Historical: scientific `c1-09442acdbdc0c752` is **INVALID_HARNESS** (PC
    /// collapse under membrane-score k-WTA + LIF reset). Prefer [`Self::c1_spike_s`].
    pub fn c1_spike() -> Self {
        let mut c = Self::c1_default();
        c.experiment = C1_SPIKE_EXPERIMENT_PREFIX.into();
        c.master_seed = 0xC15A_0000_0001;
        c
    }

    /// Quick/PILOT schedule for the natural-hidden-spiking protocol.
    pub fn c1_spike_quick() -> Self {
        let mut c = Self::c1_spike();
        c.quick = true;
        c.n_seeds = 5;
        c.n_train = 24;
        c.n_test = 16;
        c.bptt_epochs = 40;
        c.n_hidden = 64;
        c.k_wta = 1;
        c.matched_budget_repeat = false;
        c
    }

    /// Protocol-v9 calibrated natural-spiking schedule: finite θ (no mute on the
    /// learner path), spike-count k-WTA during integrate, and modest production
    /// knobs (`init_w` / `eta` / `tau_e`) so the positive control can clear
    /// `g2_min_positive_control` without threshold massage.
    ///
    /// Diverges from frozen v6 `c1-09442acdbdc0c752` and canonical v2. G2
    /// accuracy / gap / PC **floors unchanged**.
    pub fn c1_spike_s() -> Self {
        let mut c = Self::c1_default();
        c.experiment = C1_SPIKE_S_EXPERIMENT_PREFIX.into();
        c.master_seed = 0xC15B_0000_0001;
        // Stronger feedforward + eligibility so class-selective natural spikes
        // survive integrate-window competition under spike-count WTA.
        c.init_w = 0.22;
        c.eta = 0.45;
        c.tau_e = 48.0;
        c
    }

    /// Quick/PILOT schedule for the calibrated natural-spiking protocol.
    pub fn c1_spike_s_quick() -> Self {
        let mut c = Self::c1_spike_s();
        c.quick = true;
        c.n_seeds = 5;
        c.n_train = 24;
        c.n_test = 16;
        c.bptt_epochs = 40;
        c.n_hidden = 64;
        c.k_wta = 1;
        c.matched_budget_repeat = false;
        c
    }

    /// Protocol-v7 Assembly-Calculus `project` schedule: hidden winners come from
    /// [`binn_areas::project`] (charge k-WTA + Hebbian imprint) instead of the
    /// inline membrane-score k-WTA path. Applies trial-isolation resets.
    ///
    /// Fresh experiment name + seed lineage ⇒ new config hash. Does **not**
    /// alter protocol-v2 defaults or hash `c1-118207fbc3eaba53`.
    pub fn c1_project() -> Self {
        let mut c = Self::c1_default();
        c.experiment = C1_PROJECT_EXPERIMENT_PREFIX.into();
        c.master_seed = 0xC170_0000_0001;
        c
    }

    /// Quick/PILOT schedule for the Assembly-Calculus `project` protocol.
    pub fn c1_project_quick() -> Self {
        let mut c = Self::c1_project();
        c.quick = true;
        c.n_seeds = 5;
        c.n_train = 24;
        c.n_test = 16;
        c.bptt_epochs = 40;
        c.n_hidden = 64;
        c.k_wta = 1;
        c.matched_budget_repeat = false;
        c
    }

    /// Protocol-v13 live C1 `ReinforceFeedback` schedule: identical substrate /
    /// G2 thresholds / knobs as canonical C1, but plasticity uses production
    /// directional REINFORCE × frozen per-neuron `B_i` instead of broadcast ±1.
    ///
    /// Fresh experiment name + seed lineage ⇒ new config hash. Does **not**
    /// alter protocol-v2 defaults or hash `c1-118207fbc3eaba53`.
    pub fn c1_reinforce_fb() -> Self {
        let mut c = Self::c1_default();
        c.experiment = C1_REINFORCE_FB_EXPERIMENT_PREFIX.into();
        // Same master-seed lineage as protocol-v2 C1 so the A/B isolates the
        // neuromodulator (experiment name + protocol v13 still mint a new hash).
        c.master_seed = 0xC160_0000_0001;
        c
    }

    /// Quick/PILOT schedule for the live C1 `ReinforceFeedback` protocol.
    pub fn c1_reinforce_fb_quick() -> Self {
        let mut c = Self::c1_reinforce_fb();
        c.quick = true;
        c.n_seeds = 5;
        c.n_train = 24;
        c.n_test = 16;
        c.bptt_epochs = 40;
        c.n_hidden = 64;
        c.k_wta = 1;
        c.matched_budget_repeat = false;
        c
    }

    /// Protocol 14: live RFB × disclosed multi-epoch local train (scientific).
    ///
    /// Fresh experiment name ⇒ new hash. Does **not** alter v13
    /// `c1-660401d74db3c88d` or protocol-v2.
    pub fn c1_reinforce_fb_epoch() -> Self {
        let mut c = Self::c1_reinforce_fb();
        c.experiment = C1_RFB_EPOCH_EXPERIMENT_PREFIX.into();
        c
    }

    /// Quick/PILOT for protocol 14.
    pub fn c1_reinforce_fb_epoch_quick() -> Self {
        let mut c = Self::c1_reinforce_fb_quick();
        c.experiment = C1_RFB_EPOCH_EXPERIMENT_PREFIX.into();
        c
    }

    /// Protocol 15: structured frozen `B` under k-WTA (scientific).
    pub fn c1_structured_fb() -> Self {
        let mut c = Self::c1_reinforce_fb();
        c.experiment = C1_STRUCTURED_FB_EXPERIMENT_PREFIX.into();
        c
    }

    /// Quick/PILOT for protocol 15.
    pub fn c1_structured_fb_quick() -> Self {
        let mut c = Self::c1_reinforce_fb_quick();
        c.experiment = C1_STRUCTURED_FB_EXPERIMENT_PREFIX.into();
        c
    }

    /// Protocol 16: structured B × epoch-matched (scientific).
    pub fn c1_structured_fb_epoch() -> Self {
        let mut c = Self::c1_structured_fb();
        c.experiment = C1_STRUCTURED_FB_EPOCH_EXPERIMENT_PREFIX.into();
        c
    }

    /// Quick/PILOT for protocol 16.
    pub fn c1_structured_fb_epoch_quick() -> Self {
        let mut c = Self::c1_structured_fb_quick();
        c.experiment = C1_STRUCTURED_FB_EPOCH_EXPERIMENT_PREFIX.into();
        c
    }

    /// Protocol 17: structured B × capacity substrate (scientific).
    ///
    /// Capacity knobs from Tier-B (`n_hidden=256`, `k_wta=4`, richer train) with
    /// v15 structured `B` and live RFB plasticity. Fresh experiment name ⇒ new
    /// hash. Does **not** remassage capacity-only or v15 FAILs.
    pub fn c1_structured_fb_capacity() -> Self {
        let mut c = Self::c1_capacity_sensitivity();
        c.experiment = C1_STRUCTURED_FB_CAPACITY_EXPERIMENT_PREFIX.into();
        // Align seed lineage with RFB family so A/B isolates B+capacity vs v13.
        c.master_seed = 0xC160_0000_0001;
        c
    }

    /// Quick/PILOT for protocol 17.
    pub fn c1_structured_fb_capacity_quick() -> Self {
        let mut c = Self::c1_capacity_sensitivity_quick();
        c.experiment = C1_STRUCTURED_FB_CAPACITY_EXPERIMENT_PREFIX.into();
        c.master_seed = 0xC160_0000_0001;
        c
    }

    /// Protocol 18: eligibility × REINFORCE co-design (scientific).
    ///
    /// Structured `B` + `τ_e = `[`C1_ELIG_RFB_TAU_E`] + mid-trial eligibility
    /// absorb (runner). Fresh experiment name ⇒ new hash.
    pub fn c1_elig_rfb() -> Self {
        let mut c = Self::c1_structured_fb();
        c.experiment = C1_ELIG_RFB_EXPERIMENT_PREFIX.into();
        c.tau_e = C1_ELIG_RFB_TAU_E;
        c
    }

    /// Quick/PILOT for protocol 18.
    pub fn c1_elig_rfb_quick() -> Self {
        let mut c = Self::c1_structured_fb_quick();
        c.experiment = C1_ELIG_RFB_EXPERIMENT_PREFIX.into();
        c.tau_e = C1_ELIG_RFB_TAU_E;
        c
    }

    /// Protocol 19: structured B × restored target teach (scientific).
    pub fn c1_structured_fb_teach() -> Self {
        let mut c = Self::c1_structured_fb();
        c.experiment = C1_STRUCTURED_FB_TEACH_EXPERIMENT_PREFIX.into();
        c
    }

    /// Quick/PILOT for protocol 19.
    pub fn c1_structured_fb_teach_quick() -> Self {
        let mut c = Self::c1_structured_fb_quick();
        c.experiment = C1_STRUCTURED_FB_TEACH_EXPERIMENT_PREFIX.into();
        c
    }

    /// Protocol 20: live graded-DFA on muted-θ / k-WTA C1 (scientific).
    pub fn c1_dfa_live() -> Self {
        let mut c = Self::c1_default();
        c.experiment = C1_DFA_LIVE_EXPERIMENT_PREFIX.into();
        c.master_seed = 0xC160_0000_0001;
        c
    }

    /// Quick/PILOT for protocol 20.
    pub fn c1_dfa_live_quick() -> Self {
        let mut c = Self::c1_reinforce_fb_quick();
        c.experiment = C1_DFA_LIVE_EXPERIMENT_PREFIX.into();
        c
    }

    /// Protocol 21: soft/relaxed k-WTA × structured B (scientific).
    pub fn c1_structured_fb_soft() -> Self {
        let mut c = Self::c1_structured_fb();
        c.experiment = C1_STRUCTURED_FB_SOFT_EXPERIMENT_PREFIX.into();
        c
    }

    /// Quick/PILOT for protocol 21.
    pub fn c1_structured_fb_soft_quick() -> Self {
        let mut c = Self::c1_structured_fb_quick();
        c.experiment = C1_STRUCTURED_FB_SOFT_EXPERIMENT_PREFIX.into();
        c
    }

    /// Protocol 23: finite-θ (mute off) under structured B (scientific).
    pub fn c1_structured_fb_finth() -> Self {
        let mut c = Self::c1_structured_fb();
        c.experiment = C1_STRUCTURED_FB_FINTH_EXPERIMENT_PREFIX.into();
        c
    }

    /// Quick/PILOT for protocol 23.
    pub fn c1_structured_fb_finth_quick() -> Self {
        let mut c = Self::c1_structured_fb_quick();
        c.experiment = C1_STRUCTURED_FB_FINTH_EXPERIMENT_PREFIX.into();
        c
    }

    /// Protocol 24: continuous/normalized structured B (scientific).
    pub fn c1_structured_fb_cont() -> Self {
        let mut c = Self::c1_structured_fb();
        c.experiment = C1_STRUCTURED_FB_CONT_EXPERIMENT_PREFIX.into();
        c
    }

    /// Quick/PILOT for protocol 24.
    pub fn c1_structured_fb_cont_quick() -> Self {
        let mut c = Self::c1_structured_fb_quick();
        c.experiment = C1_STRUCTURED_FB_CONT_EXPERIMENT_PREFIX.into();
        c
    }

    /// Protocol 25: online learned B_i transfer (scientific).
    pub fn c1_reinforce_fb_learned() -> Self {
        let mut c = Self::c1_reinforce_fb();
        c.experiment = C1_RFB_LEARNED_EXPERIMENT_PREFIX.into();
        c
    }

    /// Quick/PILOT for protocol 25.
    pub fn c1_reinforce_fb_learned_quick() -> Self {
        let mut c = Self::c1_reinforce_fb_quick();
        c.experiment = C1_RFB_LEARNED_EXPERIMENT_PREFIX.into();
        c
    }

    /// Protocol 28: adaptive k-WTA schedule (k=16 -> 2 over training).
    pub fn c1_k_anneal() -> Self {
        let mut c = Self::c1_default();
        c.experiment = C1_K_ANNEAL_EXPERIMENT_PREFIX.into();
        c.k_wta = 16;
        c
    }

    /// Quick/PILOT for protocol 28.
    pub fn c1_k_anneal_quick() -> Self {
        let mut c = Self::c1_quick();
        c.experiment = C1_K_ANNEAL_EXPERIMENT_PREFIX.into();
        c.k_wta = 8;
        c
    }

    /// True when this config is a Tier-B `c1-sens-*` sensitivity preset.
    #[inline]
    pub fn is_sensitivity_protocol(&self) -> bool {
        self.experiment
            .starts_with(C1_SENSITIVITY_EXPERIMENT_PREFIX)
    }

    /// True when this config is a trial-isolation `c1-iso*` integrity preset.
    #[inline]
    pub fn is_isolation_protocol(&self) -> bool {
        self.experiment.starts_with(C1_ISOLATION_EXPERIMENT_PREFIX)
    }

    /// True when this config is a natural-hidden-spiking `c1-spike*` preset
    /// (includes calibrated `c1-spike-s*`).
    #[inline]
    pub fn is_spike_protocol(&self) -> bool {
        self.experiment.starts_with(C1_SPIKE_EXPERIMENT_PREFIX)
    }

    /// True when this config is the calibrated natural-spiking `c1-spike-s*` preset.
    #[inline]
    pub fn is_spike_s_protocol(&self) -> bool {
        self.experiment.starts_with(C1_SPIKE_S_EXPERIMENT_PREFIX)
    }

    /// True when this config wires Assembly-Calculus `project` into C1.
    #[inline]
    pub fn is_project_protocol(&self) -> bool {
        self.experiment.starts_with(C1_PROJECT_EXPERIMENT_PREFIX)
    }

    /// True when this config is live RFB × epoch-matched (protocol 14).
    #[inline]
    pub fn is_reinforce_fb_epoch_protocol(&self) -> bool {
        self.experiment.starts_with(C1_RFB_EPOCH_EXPERIMENT_PREFIX)
    }

    /// True when this config is structured B × target teach (protocol 19).
    #[inline]
    pub fn is_structured_fb_teach_protocol(&self) -> bool {
        self.experiment
            .starts_with(C1_STRUCTURED_FB_TEACH_EXPERIMENT_PREFIX)
    }

    /// True when this config is live graded-DFA transfer (protocol 20).
    #[inline]
    pub fn is_dfa_live_protocol(&self) -> bool {
        self.experiment.starts_with(C1_DFA_LIVE_EXPERIMENT_PREFIX)
    }

    /// True when this config is soft-WTA × structured B (protocol 21).
    #[inline]
    pub fn is_structured_fb_soft_protocol(&self) -> bool {
        self.experiment
            .starts_with(C1_STRUCTURED_FB_SOFT_EXPERIMENT_PREFIX)
    }

    /// True when this config is finite-θ under structured B (protocol 23).
    #[inline]
    pub fn is_structured_fb_finth_protocol(&self) -> bool {
        self.experiment
            .starts_with(C1_STRUCTURED_FB_FINTH_EXPERIMENT_PREFIX)
    }

    /// True when this config is continuous structured B (protocol 24).
    #[inline]
    pub fn is_structured_fb_cont_protocol(&self) -> bool {
        self.experiment
            .starts_with(C1_STRUCTURED_FB_CONT_EXPERIMENT_PREFIX)
    }

    /// True when this config is eligibility × REINFORCE (protocol 18).
    #[inline]
    pub fn is_elig_rfb_protocol(&self) -> bool {
        self.experiment.starts_with(C1_ELIG_RFB_EXPERIMENT_PREFIX)
    }

    /// True when this config is structured B × capacity (protocol 17).
    #[inline]
    pub fn is_structured_fb_capacity_protocol(&self) -> bool {
        self.experiment
            .starts_with(C1_STRUCTURED_FB_CAPACITY_EXPERIMENT_PREFIX)
    }

    /// True when this config is structured B × epoch-matched (protocol 16).
    #[inline]
    pub fn is_structured_fb_epoch_protocol(&self) -> bool {
        self.experiment
            .starts_with(C1_STRUCTURED_FB_EPOCH_EXPERIMENT_PREFIX)
    }

    /// True when this config uses structured frozen feedback (protocol 15 only;
    /// excludes `c1-sfb-em*` / `c1-sfb-cap*` / `c1-sfb-teach*` / soft/finth/cont).
    #[inline]
    pub fn is_structured_fb_protocol(&self) -> bool {
        self.experiment
            .starts_with(C1_STRUCTURED_FB_EXPERIMENT_PREFIX)
            && !self.is_structured_fb_epoch_protocol()
            && !self.is_structured_fb_capacity_protocol()
            && !self.is_structured_fb_teach_protocol()
            && !self.is_structured_fb_soft_protocol()
            && !self.is_structured_fb_finth_protocol()
            && !self.is_structured_fb_cont_protocol()
    }

    /// True when this config is the v13 single-pass live `ReinforceFeedback` preset
    /// (`c1-rfb*`, excluding `c1-rfb-em*` / `c1-rfb-learned*`).
    #[inline]
    pub fn is_reinforce_fb_protocol(&self) -> bool {
        self.experiment
            .starts_with(C1_REINFORCE_FB_EXPERIMENT_PREFIX)
            && !self.is_reinforce_fb_epoch_protocol()
            && !self.is_reinforce_fb_learned_protocol()
    }

    /// True when this config is online learned B_i transfer (protocol 25).
    #[inline]
    pub fn is_reinforce_fb_learned_protocol(&self) -> bool {
        self.experiment
            .starts_with(C1_RFB_LEARNED_EXPERIMENT_PREFIX)
    }

    /// True when this config is adaptive k-WTA schedule (protocol 28).
    #[inline]
    pub fn is_k_anneal_protocol(&self) -> bool {
        self.experiment.starts_with(C1_K_ANNEAL_EXPERIMENT_PREFIX)
    }

    /// True when main-arm plasticity uses production `ReinforceFeedback` credit
    /// (v13–v19, v21, v23–v25). Excludes graded-DFA live (v20).
    #[inline]
    pub fn uses_live_reinforce_feedback(&self) -> bool {
        self.is_reinforce_fb_protocol()
            || self.is_reinforce_fb_epoch_protocol()
            || self.is_reinforce_fb_learned_protocol()
            || self.is_structured_fb_protocol()
            || self.is_structured_fb_epoch_protocol()
            || self.is_structured_fb_capacity_protocol()
            || self.is_elig_rfb_protocol()
            || self.is_structured_fb_teach_protocol()
            || self.is_structured_fb_soft_protocol()
            || self.is_structured_fb_finth_protocol()
            || self.is_structured_fb_cont_protocol()
    }

    /// True when hidden `B` is structured from readout columns (v15–v19, v21, v23–v24).
    #[inline]
    pub fn uses_structured_feedback_weights(&self) -> bool {
        self.is_structured_fb_protocol()
            || self.is_structured_fb_epoch_protocol()
            || self.is_structured_fb_capacity_protocol()
            || self.is_elig_rfb_protocol()
            || self.is_structured_fb_teach_protocol()
            || self.is_structured_fb_soft_protocol()
            || self.is_structured_fb_finth_protocol()
            || self.is_structured_fb_cont_protocol()
    }

    /// True when structured `B` uses continuous/normalized Δw (protocol 24).
    #[inline]
    pub fn uses_continuous_structured_feedback(&self) -> bool {
        self.is_structured_fb_cont_protocol()
    }

    /// True when hidden winners use soft/relaxed k-WTA (protocol 21).
    #[inline]
    pub fn uses_soft_k_wta(&self) -> bool {
        self.is_structured_fb_soft_protocol()
    }

    /// Disclosed soft-WTA temperature (protocol 21); `None` otherwise.
    #[inline]
    pub fn soft_k_wta_temperature(&self) -> Option<f32> {
        if self.uses_soft_k_wta() {
            Some(C1_SFB_SOFT_TEMPERATURE)
        } else {
            None
        }
    }

    /// True when the runner should absorb eligibility after winners/readout
    /// before sampling the REINFORCE action (protocol 18).
    #[inline]
    pub fn uses_elig_rfb_preabsorb(&self) -> bool {
        self.is_elig_rfb_protocol()
    }

    /// True when incorrect trials restore a secondary target teach through
    /// structured `B` (`credit(+1)`), instead of observe-only (protocol 19).
    #[inline]
    pub fn uses_structured_target_teach(&self) -> bool {
        self.is_structured_fb_teach_protocol()
    }

    /// Local-assembly / dense-local train epochs over the frozen split.
    ///
    /// Protocols 14 and 16 use a disclosed multi-epoch loop; all other C1
    /// families stay single-pass (`1`). Not a Config field — keeps v2 hashes frozen.
    #[inline]
    pub fn local_train_epochs(&self) -> usize {
        if self.is_reinforce_fb_epoch_protocol() || self.is_structured_fb_epoch_protocol() {
            if self.quick {
                C1_RFB_EPOCH_LOCAL_EPOCHS_QUICK
            } else {
                C1_RFB_EPOCH_LOCAL_EPOCHS_SCIENTIFIC
            }
        } else {
            1
        }
    }

    /// Protocol version mixed into [`Self::hash`].
    #[inline]
    pub fn protocol_version(&self) -> u64 {
        if self.is_k_anneal_protocol() {
            C1_K_ANNEAL_PROTOCOL_VERSION
        } else if self.is_reinforce_fb_learned_protocol() {
            C1_RFB_LEARNED_PROTOCOL_VERSION
        } else if self.is_structured_fb_cont_protocol() {
            C1_STRUCTURED_FB_CONT_PROTOCOL_VERSION
        } else if self.is_structured_fb_finth_protocol() {
            C1_STRUCTURED_FB_FINTH_PROTOCOL_VERSION
        } else if self.is_structured_fb_soft_protocol() {
            C1_STRUCTURED_FB_SOFT_PROTOCOL_VERSION
        } else if self.is_dfa_live_protocol() {
            C1_DFA_LIVE_PROTOCOL_VERSION
        } else if self.is_structured_fb_teach_protocol() {
            C1_STRUCTURED_FB_TEACH_PROTOCOL_VERSION
        } else if self.is_elig_rfb_protocol() {
            C1_ELIG_RFB_PROTOCOL_VERSION
        } else if self.is_structured_fb_capacity_protocol() {
            C1_STRUCTURED_FB_CAPACITY_PROTOCOL_VERSION
        } else if self.is_structured_fb_epoch_protocol() {
            C1_STRUCTURED_FB_EPOCH_PROTOCOL_VERSION
        } else if self.is_structured_fb_protocol() {
            C1_STRUCTURED_FB_PROTOCOL_VERSION
        } else if self.is_reinforce_fb_epoch_protocol() {
            C1_RFB_EPOCH_PROTOCOL_VERSION
        } else if self.is_reinforce_fb_protocol() {
            C1_REINFORCE_FB_PROTOCOL_VERSION
        } else if self.is_project_protocol() {
            C1_PROJECT_PROTOCOL_VERSION
        } else if self.is_spike_s_protocol() {
            C1_SPIKE_S_PROTOCOL_VERSION
        } else if self.is_spike_protocol() {
            C1_SPIKE_PROTOCOL_VERSION
        } else if self.is_isolation_protocol() {
            C1_ISOLATION_PROTOCOL_VERSION
        } else if self.is_sensitivity_protocol() {
            C1_SENSITIVITY_PROTOCOL_VERSION
        } else {
            C1_PROTOCOL_VERSION
        }
    }

    /// True when the harness positive control uses coincidence-lag trials.
    #[inline]
    pub fn uses_temporal_positive_control(&self) -> bool {
        self.experiment.contains("temporal-pc")
    }

    /// True when the harness PC uses the disclosed multi-frame easy task
    /// (calibrated spike-s only; main coincidence task unchanged).
    #[inline]
    pub fn uses_calibrated_spike_positive_control(&self) -> bool {
        self.is_spike_s_protocol()
    }

    /// Known presets (canonical + sensitivity + isolation + spike) for hash round-trips / CLI tips.
    pub fn known_presets() -> Vec<Self> {
        vec![
            Self::c1_default(),
            Self::c1_quick(),
            Self::c1_capacity_sensitivity(),
            Self::c1_capacity_sensitivity_quick(),
            Self::c1_temporal_pc_sensitivity(),
            Self::c1_temporal_pc_sensitivity_quick(),
            Self::c1_isolation(),
            Self::c1_isolation_quick(),
            Self::c1_spike(),
            Self::c1_spike_quick(),
            Self::c1_spike_s(),
            Self::c1_spike_s_quick(),
            Self::c1_project(),
            Self::c1_project_quick(),
            Self::c1_reinforce_fb(),
            Self::c1_reinforce_fb_quick(),
            Self::c1_reinforce_fb_epoch(),
            Self::c1_reinforce_fb_epoch_quick(),
            Self::c1_structured_fb(),
            Self::c1_structured_fb_quick(),
            Self::c1_structured_fb_epoch(),
            Self::c1_structured_fb_epoch_quick(),
            Self::c1_structured_fb_capacity(),
            Self::c1_structured_fb_capacity_quick(),
            Self::c1_elig_rfb(),
            Self::c1_elig_rfb_quick(),
            Self::c1_structured_fb_teach(),
            Self::c1_structured_fb_teach_quick(),
            Self::c1_dfa_live(),
            Self::c1_dfa_live_quick(),
            Self::c1_structured_fb_soft(),
            Self::c1_structured_fb_soft_quick(),
            Self::c1_structured_fb_finth(),
            Self::c1_structured_fb_finth_quick(),
            Self::c1_structured_fb_cont(),
            Self::c1_structured_fb_cont_quick(),
            Self::c1_reinforce_fb_learned(),
            Self::c1_reinforce_fb_learned_quick(),
            Self::c1_k_anneal(),
            Self::c1_k_anneal_quick(),
        ]
    }

    /// Reproduce a run from a previously printed hex hash of a known preset.
    pub fn from_hash(hash: &str) -> Option<Self> {
        let h = hash
            .trim()
            .trim_start_matches("0x")
            .trim_start_matches(C1_DEFAULT_HASH_PREFIX)
            .to_lowercase();
        for preset in Self::known_presets() {
            if h == format!("{:016x}", preset.hash()) {
                return Some(preset);
            }
        }
        None
    }

    /// Stable FNV-1a 64-bit fingerprint of the protocol and every public field.
    pub fn hash(&self) -> u64 {
        self.hash_for_protocol(self.protocol_version())
    }

    fn hash_for_protocol(&self, protocol_version: u64) -> u64 {
        let mut h = 0xcbf2_9ce4_8422_2325u64;
        fn mix(h: &mut u64, word: u64) {
            *h ^= word;
            *h = (*h).wrapping_mul(0x100_0000_01b3);
        }
        mix(&mut h, protocol_version);
        for b in self.experiment.as_bytes() {
            mix(&mut h, *b as u64);
        }
        mix(&mut h, self.master_seed);
        mix(&mut h, self.n_seeds as u64);
        mix(&mut h, self.sequence_len as u64);
        mix(&mut h, self.max_lag as u64);
        mix(&mut h, self.n_hidden as u64);
        mix(&mut h, self.k_wta as u64);
        mix(&mut h, self.p_sparse.to_bits() as u64);
        mix(&mut h, self.init_w.to_bits() as u64);
        mix(&mut h, self.eta.to_bits() as u64);
        mix(&mut h, self.lambda.to_bits() as u64);
        mix(&mut h, self.tau_e.to_bits() as u64);
        mix(&mut h, self.n_train as u64);
        mix(&mut h, self.n_test as u64);
        mix(&mut h, self.bptt_epochs as u64);
        mix(&mut h, self.bptt_lr.to_bits() as u64);
        mix(&mut h, self.g2_min_gap_closed.to_bits() as u64);
        mix(&mut h, self.g2_min_accuracy.to_bits() as u64);
        mix(&mut h, self.g2_confidence_z.to_bits() as u64);
        mix(&mut h, self.g2_min_positive_control.to_bits() as u64);
        mix(&mut h, self.g2_min_reference_gap.to_bits() as u64);
        mix(&mut h, self.activity_sparsity_min.to_bits() as u64);
        mix(&mut h, self.activity_sparsity_max.to_bits() as u64);
        mix(&mut h, self.scientific_n_seeds as u64);
        mix(&mut h, self.power_sigma_prior.to_bits() as u64);
        mix(&mut h, self.power_effect_size.to_bits() as u64);
        mix(&mut h, u64::from(self.use_surrogate_lif_reference));
        mix(&mut h, self.surrogate_beta.to_bits() as u64);
        mix(&mut h, u64::from(self.matched_budget_repeat));
        mix(&mut h, u64::from(self.quick));
        // Mac-probe geometry knobs: mix only when active so frozen C1 hashes stay.
        if self.is_mac_probe_geometry() {
            mix(&mut h, self.max_fan_out as u64);
            mix(&mut h, u64::from(self.init_w_rescale));
            mix(&mut h, u64::from(self.readout_gain_normalize));
        }
        h
    }

    /// True when width / fan / rescale knobs are part of the scientific object.
    #[inline]
    pub fn is_mac_probe_geometry(&self) -> bool {
        self.experiment.starts_with("c1-mac-probe")
            || self.experiment.starts_with("c1-micro")
            || self.experiment.contains("-mac-n")
            || self.init_w_rescale
            || self.readout_gain_normalize
            || (self.max_fan_out != 256 && self.n_hidden >= 512)
    }

    /// Hex string form used in logs and results notes.
    #[inline]
    pub fn hash_string(&self) -> String {
        if self.experiment.starts_with("c1-micro") {
            format!("c1-micro-{:016x}", self.hash())
        } else if self.experiment.starts_with("c1-mac-probe") {
            format!("c1-mac-probe-{:016x}", self.hash())
        } else {
            format!("{}{:016x}", C1_DEFAULT_HASH_PREFIX, self.hash())
        }
    }

    /// Seed list of length `n_seeds` derived from `master_seed`.
    pub fn seeds(&self) -> Vec<u64> {
        const GOLDEN: u64 = 0x9E37_79B9_7F4A_7C15;
        (0..self.n_seeds)
            .map(|i| self.master_seed ^ GOLDEN.wrapping_mul(i as u64 + 1))
            .collect()
    }

    /// Preregistered one-sample normal-approximation seed count for 80% power
    /// at two-sided α≈0.05 against `power_effect_size` with prior σ.
    ///
    /// Formula (hashed via the config fields it depends on):
    /// `n = ceil((z_{1-α/2} + z_{1-β})² · σ² / δ²)` with
    /// `z_{1-α/2}=1.96`, `z_{0.80}=0.8416`.
    pub fn n_seeds_for_80_power(&self) -> usize {
        n_for_80_percent_power(self.power_sigma_prior, self.power_effect_size)
    }

    /// Required scientific seed count: `max(scientific_n_seeds, n_for_80%_power)`.
    pub fn required_scientific_n_seeds(&self) -> usize {
        self.scientific_n_seeds
            .max(20)
            .max(self.n_seeds_for_80_power())
    }

    /// Nominal activity fraction `k_wta / n_hidden`.
    #[inline]
    pub fn nominal_activity_fraction(&self) -> f32 {
        if self.n_hidden == 0 {
            0.0
        } else {
            self.k_wta as f32 / self.n_hidden as f32
        }
    }
}

/// `n = ceil((1.96 + 0.8416)² · σ² / δ²)` for a one-sample mean test.
pub fn n_for_80_percent_power(sigma: f32, effect: f32) -> usize {
    assert!(sigma.is_finite() && sigma > 0.0, "sigma must be positive");
    assert!(
        effect.is_finite() && effect > 0.0,
        "effect must be positive"
    );
    let z = 1.96f64 + 0.8416;
    let n = (z * z) * (sigma as f64).powi(2) / (effect as f64).powi(2);
    n.ceil().max(1.0) as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_stable_and_sensitive() {
        let a = Config::c1_default();
        let b = Config::c1_default();
        assert_eq!(a.hash(), b.hash());
        assert_ne!(
            a.hash_for_protocol(C1_PROTOCOL_VERSION - 1),
            a.hash_for_protocol(C1_PROTOCOL_VERSION),
            "protocol changes must invalidate prior config hashes"
        );
        let mut c = Config::c1_default();
        c.eta *= 1.01;
        assert_ne!(a.hash(), c.hash());
        let mut d = Config::c1_default();
        d.g2_min_positive_control = 0.91;
        assert_ne!(a.hash(), d.hash());
        let mut e = Config::c1_default();
        e.g2_min_reference_gap = 0.20;
        assert_ne!(a.hash(), e.hash());
    }

    #[test]
    fn from_hash_round_trips_presets() {
        for preset in Config::known_presets() {
            assert_eq!(
                Config::from_hash(&preset.hash_string()).unwrap(),
                preset,
                "round-trip failed for experiment={}",
                preset.experiment
            );
        }
    }

    #[test]
    fn scientific_defaults_target_one_to_two_percent_activity() {
        let d = Config::c1_default();
        let frac = d.nominal_activity_fraction();
        assert!((frac - 2.0 / 128.0).abs() < 1e-6);
        assert!((0.005..=0.03).contains(&frac));
        let q = Config::c1_quick();
        let qf = q.nominal_activity_fraction();
        assert!((qf - 1.0 / 64.0).abs() < 1e-6);
        assert!((qf - 0.015).abs() < 0.002);
        let cap = Config::c1_capacity_sensitivity();
        let cf = cap.nominal_activity_fraction();
        assert!((cf - 4.0 / 256.0).abs() < 1e-6);
        assert!((0.005..=0.03).contains(&cf));
    }

    #[test]
    fn power_analysis_respects_floor() {
        let c = Config::c1_default();
        assert!(c.required_scientific_n_seeds() >= 20);
        assert_eq!(n_for_80_percent_power(0.15, 0.10), c.n_seeds_for_80_power());
    }

    #[test]
    fn canonical_v2_hash_unchanged() {
        let d = Config::c1_default();
        assert_eq!(d.protocol_version(), C1_PROTOCOL_VERSION);
        assert_eq!(
            d.hash_string(),
            "c1-118207fbc3eaba53",
            "protocol-v2 canonical C1 hash must stay pinned"
        );
        assert!(!d.is_sensitivity_protocol());
        assert!(!d.uses_temporal_positive_control());
    }

    #[test]
    fn sensitivity_presets_use_protocol_v3_and_diverge() {
        let d = Config::c1_default();
        let cap = Config::c1_capacity_sensitivity();
        let tpc = Config::c1_temporal_pc_sensitivity();
        assert_eq!(cap.protocol_version(), C1_SENSITIVITY_PROTOCOL_VERSION);
        assert_eq!(tpc.protocol_version(), C1_SENSITIVITY_PROTOCOL_VERSION);
        assert!(cap.is_sensitivity_protocol());
        assert!(tpc.is_sensitivity_protocol());
        assert!(tpc.uses_temporal_positive_control());
        assert!(!cap.uses_temporal_positive_control());
        assert_ne!(d.hash(), cap.hash());
        assert_ne!(d.hash(), tpc.hash());
        assert_ne!(cap.hash(), tpc.hash());
        assert_eq!(cap.hash_string(), "c1-d38d7644d8afc84b");
        assert_eq!(tpc.hash_string(), "c1-a49deeaedb495a09");
        assert_eq!(
            Config::c1_capacity_sensitivity_quick().hash_string(),
            "c1-e519403aff33b384"
        );
        assert_eq!(
            Config::c1_temporal_pc_sensitivity_quick().hash_string(),
            "c1-097696ca34d8a34d"
        );
    }

    #[test]
    fn isolation_presets_use_protocol_v5_and_diverge_from_v2() {
        let d = Config::c1_default();
        let iso = Config::c1_isolation();
        let iso_q = Config::c1_isolation_quick();
        assert_eq!(iso.protocol_version(), C1_ISOLATION_PROTOCOL_VERSION);
        assert_eq!(iso_q.protocol_version(), C1_ISOLATION_PROTOCOL_VERSION);
        assert!(iso.is_isolation_protocol());
        assert!(iso_q.is_isolation_protocol());
        assert!(!iso.is_sensitivity_protocol());
        assert_ne!(d.hash_string(), iso.hash_string());
        assert_ne!(iso.hash_string(), iso_q.hash_string());
        assert_ne!(iso.hash_string(), "c1-118207fbc3eaba53");
        assert_eq!(d.hash_string(), "c1-118207fbc3eaba53");
        assert_eq!(iso.hash_string(), "c1-8ec031907a3426d0");
        assert_eq!(iso_q.hash_string(), "c1-befbfe8f014bda18");
    }

    #[test]
    fn spike_presets_use_protocol_v6_and_diverge_from_v2_and_iso() {
        let d = Config::c1_default();
        let iso = Config::c1_isolation();
        let spike = Config::c1_spike();
        let spike_q = Config::c1_spike_quick();
        assert_eq!(spike.protocol_version(), C1_SPIKE_PROTOCOL_VERSION);
        assert_eq!(spike_q.protocol_version(), C1_SPIKE_PROTOCOL_VERSION);
        assert!(spike.is_spike_protocol());
        assert!(spike_q.is_spike_protocol());
        assert!(!spike.is_spike_s_protocol());
        assert!(!spike.is_isolation_protocol());
        assert!(!spike.is_sensitivity_protocol());
        assert_ne!(d.hash_string(), spike.hash_string());
        assert_ne!(iso.hash_string(), spike.hash_string());
        assert_ne!(spike.hash_string(), spike_q.hash_string());
        assert_ne!(spike.hash_string(), "c1-118207fbc3eaba53");
        assert_eq!(d.hash_string(), "c1-118207fbc3eaba53");
        assert_eq!(spike.hash_string(), "c1-09442acdbdc0c752");
        assert_eq!(spike_q.hash_string(), "c1-d6b811cec7feed26");
    }

    #[test]
    fn spike_s_presets_use_protocol_v9_and_diverge_from_v6_and_v2() {
        let d = Config::c1_default();
        let spike = Config::c1_spike();
        let spike_s = Config::c1_spike_s();
        let spike_s_q = Config::c1_spike_s_quick();
        assert_eq!(spike_s.protocol_version(), C1_SPIKE_S_PROTOCOL_VERSION);
        assert_eq!(spike_s_q.protocol_version(), C1_SPIKE_S_PROTOCOL_VERSION);
        assert!(spike_s.is_spike_protocol());
        assert!(spike_s.is_spike_s_protocol());
        assert!(spike_s.uses_calibrated_spike_positive_control());
        assert!(!spike.uses_calibrated_spike_positive_control());
        assert_ne!(d.hash_string(), spike_s.hash_string());
        assert_ne!(spike.hash_string(), spike_s.hash_string());
        assert_ne!(spike_s.hash_string(), spike_s_q.hash_string());
        assert_ne!(spike_s.hash_string(), "c1-118207fbc3eaba53");
        assert_ne!(spike_s.hash_string(), "c1-09442acdbdc0c752");
        assert_eq!(d.hash_string(), "c1-118207fbc3eaba53");
        assert_eq!(spike.hash_string(), "c1-09442acdbdc0c752");
        assert_eq!(spike_s.hash_string(), "c1-c3e47b1e5f564df6");
        assert_eq!(spike_s_q.hash_string(), "c1-078cdbd91088c2f6");
    }

    #[test]
    fn project_presets_use_protocol_v7_and_diverge() {
        let d = Config::c1_default();
        let proj = Config::c1_project();
        let proj_q = Config::c1_project_quick();
        assert_eq!(proj.protocol_version(), C1_PROJECT_PROTOCOL_VERSION);
        assert!(proj.is_project_protocol());
        assert!(!proj.is_spike_protocol());
        assert!(!proj.is_isolation_protocol());
        assert_ne!(d.hash_string(), proj.hash_string());
        assert_ne!(proj.hash_string(), Config::c1_spike().hash_string());
        assert_ne!(proj.hash_string(), proj_q.hash_string());
        assert_ne!(proj.hash_string(), "c1-118207fbc3eaba53");
        assert_eq!(proj.hash_string(), "c1-8cc19eccba9c70aa");
        assert_eq!(proj_q.hash_string(), "c1-41458c2941a9d96e");
    }

    #[test]
    fn reinforce_fb_presets_use_protocol_v13_and_diverge_from_v2() {
        let d = Config::c1_default();
        let rfb = Config::c1_reinforce_fb();
        let rfb_q = Config::c1_reinforce_fb_quick();
        assert_eq!(rfb.protocol_version(), C1_REINFORCE_FB_PROTOCOL_VERSION);
        assert_eq!(rfb_q.protocol_version(), C1_REINFORCE_FB_PROTOCOL_VERSION);
        assert!(rfb.is_reinforce_fb_protocol());
        assert!(rfb_q.is_reinforce_fb_protocol());
        assert!(!rfb.is_spike_protocol());
        assert!(!rfb.is_isolation_protocol());
        assert!(!rfb.is_project_protocol());
        assert!(!rfb.is_sensitivity_protocol());
        // Same scientific knobs as v2 except experiment + master_seed + protocol.
        assert_eq!(rfb.n_hidden, d.n_hidden);
        assert_eq!(rfb.k_wta, d.k_wta);
        assert_eq!(rfb.eta, d.eta);
        assert_eq!(rfb.g2_min_gap_closed, d.g2_min_gap_closed);
        assert_eq!(rfb.g2_min_accuracy, d.g2_min_accuracy);
        assert_ne!(d.hash_string(), rfb.hash_string());
        assert_ne!(rfb.hash_string(), rfb_q.hash_string());
        assert_ne!(rfb.hash_string(), "c1-118207fbc3eaba53");
        assert_eq!(d.hash_string(), "c1-118207fbc3eaba53");
        assert_eq!(rfb.hash_string(), "c1-660401d74db3c88d");
        assert_eq!(rfb_q.hash_string(), "c1-a57975f13b73a599");
    }

    #[test]
    fn rfb_epoch_and_structured_fb_mint_distinct_protocol_hashes() {
        let v13 = Config::c1_reinforce_fb();
        let em = Config::c1_reinforce_fb_epoch();
        let em_q = Config::c1_reinforce_fb_epoch_quick();
        let sfb = Config::c1_structured_fb();
        let sfb_q = Config::c1_structured_fb_quick();
        assert!(em.is_reinforce_fb_epoch_protocol());
        assert!(!em.is_reinforce_fb_protocol());
        assert!(em.uses_live_reinforce_feedback());
        assert_eq!(em.protocol_version(), C1_RFB_EPOCH_PROTOCOL_VERSION);
        assert_eq!(
            em.local_train_epochs(),
            C1_RFB_EPOCH_LOCAL_EPOCHS_SCIENTIFIC
        );
        assert_eq!(em_q.local_train_epochs(), C1_RFB_EPOCH_LOCAL_EPOCHS_QUICK);
        assert_eq!(v13.local_train_epochs(), 1);
        assert!(sfb.is_structured_fb_protocol());
        assert!(!sfb.is_reinforce_fb_protocol());
        assert!(sfb.uses_live_reinforce_feedback());
        assert_eq!(sfb.protocol_version(), C1_STRUCTURED_FB_PROTOCOL_VERSION);
        assert_eq!(sfb.local_train_epochs(), 1);
        assert_ne!(em.hash_string(), v13.hash_string());
        assert_ne!(sfb.hash_string(), v13.hash_string());
        assert_ne!(em.hash_string(), sfb.hash_string());
        assert_ne!(em.hash_string(), em_q.hash_string());
        assert_ne!(sfb.hash_string(), sfb_q.hash_string());
        assert_ne!(em.hash_string(), "c1-118207fbc3eaba53");
        assert_ne!(sfb.hash_string(), "c1-660401d74db3c88d");
        let sfb_em = Config::c1_structured_fb_epoch();
        let sfb_em_q = Config::c1_structured_fb_epoch_quick();
        assert!(sfb_em.is_structured_fb_epoch_protocol());
        assert!(!sfb_em.is_structured_fb_protocol());
        assert!(sfb_em.uses_live_reinforce_feedback());
        assert!(sfb_em.uses_structured_feedback_weights());
        assert_eq!(
            sfb_em.protocol_version(),
            C1_STRUCTURED_FB_EPOCH_PROTOCOL_VERSION
        );
        assert_eq!(
            sfb_em.local_train_epochs(),
            C1_RFB_EPOCH_LOCAL_EPOCHS_SCIENTIFIC
        );
        assert_eq!(
            sfb_em_q.local_train_epochs(),
            C1_RFB_EPOCH_LOCAL_EPOCHS_QUICK
        );
        assert_ne!(sfb_em.hash_string(), sfb.hash_string());
        assert_ne!(sfb_em.hash_string(), em.hash_string());
        assert_ne!(sfb_em.hash_string(), sfb_em_q.hash_string());
        // Round-trip via known_presets / from_hash.
        assert_eq!(
            Config::from_hash(&em.hash_string()).unwrap().hash_string(),
            em.hash_string()
        );
        assert_eq!(
            Config::from_hash(&sfb.hash_string()).unwrap().hash_string(),
            sfb.hash_string()
        );
        assert_eq!(
            Config::from_hash(&sfb_em.hash_string())
                .unwrap()
                .hash_string(),
            sfb_em.hash_string()
        );
        let sfb_cap = Config::c1_structured_fb_capacity();
        let sfb_cap_q = Config::c1_structured_fb_capacity_quick();
        assert!(sfb_cap.is_structured_fb_capacity_protocol());
        assert!(!sfb_cap.is_structured_fb_protocol());
        assert!(!sfb_cap.is_sensitivity_protocol());
        assert!(sfb_cap.uses_structured_feedback_weights());
        assert_eq!(
            sfb_cap.protocol_version(),
            C1_STRUCTURED_FB_CAPACITY_PROTOCOL_VERSION
        );
        assert_eq!(sfb_cap.k_wta, 4);
        assert_eq!(sfb_cap.n_hidden, 256);
        assert_ne!(sfb_cap.hash_string(), sfb.hash_string());
        assert_ne!(sfb_cap.hash_string(), sfb_em.hash_string());
        assert_ne!(
            sfb_cap.hash_string(),
            Config::c1_capacity_sensitivity().hash_string()
        );
        assert_ne!(sfb_cap.hash_string(), sfb_cap_q.hash_string());
        assert_eq!(
            Config::from_hash(&sfb_cap.hash_string())
                .unwrap()
                .hash_string(),
            sfb_cap.hash_string()
        );
        let elig = Config::c1_elig_rfb();
        let elig_q = Config::c1_elig_rfb_quick();
        assert!(elig.is_elig_rfb_protocol());
        assert!(!elig.is_structured_fb_protocol());
        assert!(elig.uses_live_reinforce_feedback());
        assert!(elig.uses_structured_feedback_weights());
        assert!(elig.uses_elig_rfb_preabsorb());
        assert_eq!(elig.protocol_version(), C1_ELIG_RFB_PROTOCOL_VERSION);
        assert!((elig.tau_e - C1_ELIG_RFB_TAU_E).abs() < 1e-6);
        assert_ne!(elig.hash_string(), sfb.hash_string());
        assert_ne!(elig.hash_string(), sfb_cap.hash_string());
        assert_ne!(elig.hash_string(), elig_q.hash_string());
        assert_eq!(
            Config::from_hash(&elig.hash_string())
                .unwrap()
                .hash_string(),
            elig.hash_string()
        );
        let teach = Config::c1_structured_fb_teach();
        let teach_q = Config::c1_structured_fb_teach_quick();
        assert!(teach.is_structured_fb_teach_protocol());
        assert!(!teach.is_structured_fb_protocol());
        assert!(teach.uses_live_reinforce_feedback());
        assert!(teach.uses_structured_feedback_weights());
        assert!(teach.uses_structured_target_teach());
        assert!(!teach.uses_elig_rfb_preabsorb());
        assert_eq!(
            teach.protocol_version(),
            C1_STRUCTURED_FB_TEACH_PROTOCOL_VERSION
        );
        assert_ne!(teach.hash_string(), sfb.hash_string());
        assert_ne!(teach.hash_string(), elig.hash_string());
        assert_ne!(teach.hash_string(), teach_q.hash_string());
        assert_eq!(
            Config::from_hash(&teach.hash_string())
                .unwrap()
                .hash_string(),
            teach.hash_string()
        );
    }

    /// Paper-cited scientific hashes must not drift silently (camera-ready freeze).
    #[test]
    fn paper_scientific_hashes_are_frozen() {
        assert_eq!(Config::c1_default().hash_string(), "c1-118207fbc3eaba53");
        assert_eq!(
            Config::c1_reinforce_fb().hash_string(),
            "c1-660401d74db3c88d"
        );
        assert_eq!(
            Config::c1_reinforce_fb_epoch().hash_string(),
            "c1-714c115e14a3eeed"
        );
        assert_eq!(
            Config::c1_structured_fb().hash_string(),
            "c1-493ddd56f8714fb6"
        );
        assert_eq!(
            Config::c1_structured_fb_epoch().hash_string(),
            "c1-677df7f7cbe4f8ec"
        );
        assert_eq!(
            Config::c1_structured_fb_capacity().hash_string(),
            "c1-983ee5303c00b147"
        );
        assert_eq!(Config::c1_elig_rfb().hash_string(), "c1-c7d2c86a2b1927f6");
        assert_eq!(
            Config::c1_structured_fb_teach().hash_string(),
            "c1-dfab4a7ec19f17c2"
        );
        assert!(Config::c1_structured_fb_teach().is_structured_fb_teach_protocol());
        assert!(!Config::c1_structured_fb_teach().is_structured_fb_protocol());
        assert!(Config::c1_structured_fb_teach().uses_structured_target_teach());
        assert_ne!(
            Config::c1_structured_fb_teach().hash_string(),
            Config::c1_structured_fb().hash_string()
        );
        assert_eq!(
            Config::c1_capacity_sensitivity().hash_string(),
            "c1-d38d7644d8afc84b"
        );
        assert_eq!(Config::c1_isolation().hash_string(), "c1-8ec031907a3426d0");
        assert_eq!(
            Config::c1_temporal_pc_sensitivity().hash_string(),
            "c1-a49deeaedb495a09"
        );
        assert!((Config::c1_elig_rfb().tau_e - C1_ELIG_RFB_TAU_E).abs() < 1e-6);
        assert!(Config::c1_elig_rfb().uses_elig_rfb_preabsorb());
        assert!(!Config::c1_structured_fb().uses_elig_rfb_preabsorb());
    }

    #[test]
    fn paper_known_presets_round_trip_every_family() {
        for preset in Config::known_presets() {
            let h = preset.hash_string();
            let back = Config::from_hash(&h).expect("known preset must round-trip");
            assert_eq!(back.hash_string(), h);
            assert_eq!(back.experiment, preset.experiment);
            assert_eq!(back.protocol_version(), preset.protocol_version());
        }
    }

    #[test]
    fn break_it_v20_v24_presets_mint_distinct_hashes() {
        let v15 = Config::c1_structured_fb();
        let v20 = Config::c1_dfa_live();
        let v21 = Config::c1_structured_fb_soft();
        let v23 = Config::c1_structured_fb_finth();
        let v24 = Config::c1_structured_fb_cont();
        assert_eq!(v20.protocol_version(), C1_DFA_LIVE_PROTOCOL_VERSION);
        assert_eq!(
            v21.protocol_version(),
            C1_STRUCTURED_FB_SOFT_PROTOCOL_VERSION
        );
        assert_eq!(
            v23.protocol_version(),
            C1_STRUCTURED_FB_FINTH_PROTOCOL_VERSION
        );
        assert_eq!(
            v24.protocol_version(),
            C1_STRUCTURED_FB_CONT_PROTOCOL_VERSION
        );
        assert!(v20.is_dfa_live_protocol());
        assert!(!v20.uses_live_reinforce_feedback());
        assert!(v21.uses_soft_k_wta());
        assert!(v21.uses_structured_feedback_weights());
        assert!(!v21.is_structured_fb_protocol());
        assert!(v23.uses_structured_feedback_weights());
        assert!(v24.uses_continuous_structured_feedback());
        assert!(!v24.is_structured_fb_protocol());
        for p in [&v20, &v21, &v23, &v24] {
            assert_ne!(p.hash_string(), v15.hash_string());
            assert_ne!(p.hash_string(), "c1-118207fbc3eaba53");
            assert_ne!(p.hash_string(), "c1-660401d74db3c88d");
        }
        assert_ne!(v20.hash_string(), v21.hash_string());
        assert_ne!(v21.hash_string(), v23.hash_string());
        assert_ne!(v23.hash_string(), v24.hash_string());
        assert!((v21.soft_k_wta_temperature().unwrap() - C1_SFB_SOFT_TEMPERATURE).abs() < 1e-6);
    }

    #[test]
    fn print_known_preset_hashes_for_docs() {
        for p in Config::known_presets() {
            eprintln!(
                "HASH {} proto={} exp={} quick={}",
                p.hash_string(),
                p.protocol_version(),
                p.experiment,
                p.quick
            );
        }
    }
}
