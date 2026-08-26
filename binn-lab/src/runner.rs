//! Experiment runner (U13): seed sweeps, C1 conditions, Gate G2 verdict.
//!
//! Conditions (clearly labeled):
//! - **local-assembly** — three-factor rule on sparse assembly wiring + k-WTA
//! - **dense-local** — same local rule and same k-winner budget on dense
//!   all-to-all connectivity (no assembly structure)
//! - **gradient-reference** — labeled surrogate-LIF BPTT (primary); optional tanh
//! - **eligibility-reference** — labeled e-prop-compatible eligibility local ref

use std::path::Path;
use std::time::Instant;

use binn_areas::{k_wta, project, soft_k_wta, wire, Area, AreaRole, Assembly, Pos, WiringPrior};
use binn_core::{Csr, Rng, Tick};
use binn_data::{
    CoincidenceTask, Encoder, LatencyEncoder, Metrics, Sample, TemporalOrderExample, WorkCosts,
    WorkCounters, TEMPORAL_ORDER_N_IN, TEMPORAL_ORDER_T,
};
use binn_engine::{CellId, Engine, K};
use binn_learn::{
    reinforce_term, BpttBaseline, DenseTemporalExample, EpropReference, FixedRandomFeedback,
    GradientExample, LearnedReinforceFeedback, Modulators, ReinforceFeedback, ShdExample,
    SurrogateLifReference, ThreeFactor, REFERENCE_SEQUENCE_LEN,
};

use crate::config::{
    Config, C1_DFA_LIVE_PROTOCOL_VERSION, C1_ELIG_RFB_PROTOCOL_VERSION, C1_ELIG_RFB_TAU_E,
    C1_ISOLATION_PROTOCOL_VERSION, C1_PROJECT_PROTOCOL_VERSION, C1_PROTOCOL_VERSION,
    C1_REINFORCE_FB_PROTOCOL_VERSION, C1_RFB_EPOCH_PROTOCOL_VERSION,
    C1_SENSITIVITY_PROTOCOL_VERSION, C1_SFB_SOFT_TEMPERATURE, C1_SPIKE_PROTOCOL_VERSION,
    C1_SPIKE_S_PROTOCOL_VERSION, C1_STRUCTURED_FB_CAPACITY_PROTOCOL_VERSION,
    C1_STRUCTURED_FB_CONT_PROTOCOL_VERSION, C1_STRUCTURED_FB_EPOCH_PROTOCOL_VERSION,
    C1_STRUCTURED_FB_FINTH_PROTOCOL_VERSION, C1_STRUCTURED_FB_PROTOCOL_VERSION,
    C1_STRUCTURED_FB_SOFT_PROTOCOL_VERSION, C1_STRUCTURED_FB_TEACH_PROTOCOL_VERSION,
};
use crate::logging::{
    trace_export_seed, trace_out_path, RunLog, StructuredLogger, TraceArea, TraceEligEdge,
    TraceProjection, TraceRecorder, TraceScore, TraceWeightEdge,
};
use crate::plots::Plots;
use crate::replay::{replay_out_path, ReplayExport, ReplayGroup, ReplayTrial};

use std::sync::atomic::{AtomicU64, Ordering};

/// Test/diagnostic counter: increments whenever [`project`] is invoked on the C1 path.
pub static C1_PROJECT_INVOKE_COUNT: AtomicU64 = AtomicU64::new(0);

/// Stable labels for G2 reporting.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConditionLabel {
    /// Sparse assembly + k-WTA + three-factor (thesis condition).
    LocalAssembly,
    /// Dense connectivity + three-factor, no assembly structure (plateau control).
    DenseLocal,
    /// Dense-local with parameter count matched to local-assembly nnz.
    DenseMatched,
    /// Labeled gradient-trained reference; not claimed to be an upper bound.
    GradientReference,
    /// Labeled e-prop-compatible eligibility reference (GC1 baseline).
    EligibilityReference,
}

impl ConditionLabel {
    pub fn as_str(self) -> &'static str {
        match self {
            ConditionLabel::LocalAssembly => "local-assembly",
            ConditionLabel::DenseLocal => "dense-local",
            ConditionLabel::DenseMatched => "dense-matched",
            ConditionLabel::GradientReference => "gradient-reference",
            ConditionLabel::EligibilityReference => "eligibility-reference",
        }
    }

    /// Parse a condition label string (CLI / isolate protocol).
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "local-assembly" => Some(Self::LocalAssembly),
            "dense-local" => Some(Self::DenseLocal),
            "dense-matched" => Some(Self::DenseMatched),
            "gradient-reference" => Some(Self::GradientReference),
            "eligibility-reference" => Some(Self::EligibilityReference),
            _ => None,
        }
    }
}

/// Per-seed accuracies for the C1 conditions.
#[derive(Clone, Debug, PartialEq)]
pub struct SeedResult {
    pub seed: u64,
    pub local_assembly: f32,
    pub dense_local: f32,
    pub gradient_reference: f32,
    pub eligibility_reference: f32,
    /// Mean activity sparsity of the local-assembly condition (GC7).
    pub activity_sparsity: f32,
    /// Mean activity sparsity of the dense-local condition (GC7).
    pub dense_activity_sparsity: f32,
    /// Optional dense-local accuracy under a parameter-matched edge budget.
    pub dense_matched: Option<f32>,
}

/// One emitted structured row (after GC7 gate).
#[derive(Clone, Debug, PartialEq)]
pub struct RunRecord {
    pub line: String,
}

/// Paired comparison summary across seeds.
#[derive(Clone, Debug, PartialEq)]
pub struct PairedSummary {
    pub mean_local: f32,
    pub mean_dense: f32,
    pub mean_gradient_reference: f32,
    pub mean_eligibility_reference: f32,
    pub var_local: f32,
    pub var_dense: f32,
    pub var_gradient_reference: f32,
    pub var_eligibility_reference: f32,
    /// Mean normalized fraction of the dense-to-reference accuracy gap closed:
    /// `(local - dense) / (gradient_reference - dense)`.
    pub mean_gap_closed: f32,
    /// Sample variance of the per-seed normalized gap-closed values.
    pub var_gap_closed: f32,
    /// Preregistered normal-approximation lower confidence bound.
    pub gap_closed_lower_95: f32,
    /// Per-seed `|local − dense|`, retained as a descriptive diagnostic.
    pub mean_dist_to_dense: f32,
    pub n: usize,
}

/// Gate G2 kill-gate verdict (four-state).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GateG2Verdict {
    /// Full run cleared both the normalized-gap confidence gate and accuracy floor.
    Pass,
    /// Full run failed at least one preregistered threshold.
    Fail,
    /// Quick / insufficient seeds; never a scientific gate decision.
    Pilot,
    /// Positive control or activity sparsity outside band — no scientific PASS/FAIL.
    InvalidHarness,
}

impl GateG2Verdict {
    pub fn as_str(self) -> &'static str {
        match self {
            GateG2Verdict::Pass => "PASS",
            GateG2Verdict::Fail => "FAIL",
            GateG2Verdict::Pilot => "PILOT",
            GateG2Verdict::InvalidHarness => "INVALID_HARNESS",
        }
    }
}

/// Disclosed compute / parameter budget for one condition.
#[derive(Clone, Debug, PartialEq)]
pub struct BudgetDisclosure {
    pub n_cells: usize,
    pub n_params: usize,
    pub work: WorkCounters,
    pub wall_secs: f64,
    pub peak_rss_bytes: u64,
    pub work_per_accuracy: f64,
}

/// Full C1 report (milestone = trustworthy verdict obtained).
#[derive(Clone, Debug, PartialEq)]
pub struct C1Report {
    pub config_hash: String,
    pub seeds: Vec<SeedResult>,
    pub summary: PairedSummary,
    pub verdict: GateG2Verdict,
    /// Mean accuracy of the positive/sanity control (local pipeline on a
    /// trivially separable task).
    pub positive_control_mean: f32,
    /// Mean local-assembly activity sparsity across seeds.
    pub mean_activity_sparsity: f32,
    /// Required scientific seed count from the preregistered power formula.
    pub required_scientific_n_seeds: usize,
    pub budgets: Vec<(ConditionLabel, BudgetDisclosure)>,
    pub emitted: Vec<RunRecord>,
    pub plot_notes: Vec<String>,
}

/// Frozen train/test identities shared by every condition for one seed.
#[derive(Clone, Debug)]
pub struct FrozenSplit {
    pub train: Vec<(Vec<Sample>, u32)>,
    pub test: Vec<(Vec<Sample>, u32)>,
}

/// Multi-seed experiment runner.
#[derive(Clone, Debug, Default)]
pub struct Runner {
    pub plots: Plots,
    pub logger: StructuredLogger,
}

impl Runner {
    pub fn new() -> Self {
        Self::default()
    }

    /// Run a single labeled condition (in-process). Used by the isolate-child CLI
    /// and by unit tests; parent C1 runs prefer [`Self::run_condition_isolated`].
    pub fn run_condition(
        &self,
        config: &Config,
        seed: u64,
        label: ConditionLabel,
        match_nnz: Option<usize>,
    ) -> BudgetDisclosure {
        let split = freeze_trials(config, seed);
        let outcome = run_labeled_condition(config, seed, label, &split, match_nnz);
        outcome.budget
    }

    /// Compact JSON line for isolate-child stdout (no serde dependency).
    pub fn condition_json(
        config: &Config,
        seed: u64,
        label: ConditionLabel,
        match_nnz: Option<usize>,
    ) -> String {
        let split = freeze_trials(config, seed);
        let o = run_labeled_condition(config, seed, label, &split, match_nnz);
        let mut json = format!(
            "{{\"condition\":\"{}\",\"seed\":{},\"accuracy\":{:.8},\"activity_sparsity\":{:.8},\"n_cells\":{},\"n_params\":{},\"wall_secs\":{:.8},\"peak_rss_bytes\":{},\"source_spikes\":{},\"synaptic_deliveries\":{},\"cell_updates\":{},\"plasticity_updates\":{},\"work_per_accuracy\":{:.8}",
            label.as_str(),
            seed,
            o.accuracy,
            o.activity_sparsity,
            o.budget.n_cells,
            o.budget.n_params,
            o.budget.wall_secs,
            o.budget.peak_rss_bytes,
            o.budget.work.source_spikes,
            o.budget.work.synaptic_deliveries,
            o.budget.work.cell_updates,
            o.budget.work.plasticity_updates,
            o.budget.work_per_accuracy
        );
        if let Some(d) = &o.mac_probe {
            json.push_str(&format!(
                ",\"n_hidden\":{},\"k_wta\":{},\"k_over_n\":{:.8},\"p_sparse\":{:.8},\"max_fan_out\":{},\"measured_nnz\":{},\"predicted_nnz\":{},\"mean_out_degree\":{:.8},\"p95_out_degree\":{:.8},\"mean_readout_fan_in\":{:.8},\"mean_hidden_fan_in\":{:.8},\"regime\":\"{}\",\"init_w\":{:.8},\"effective_init_w\":{:.8},\"readout_boost\":{:.8},\"effective_readout_gain\":{:.8},\"empty_winner_rate\":{:.8},\"matched_budget_repeat\":{},\"config_hash\":\"{}\",\"protocol_version\":{}",
                config.n_hidden,
                config.k_wta,
                config.nominal_activity_fraction(),
                config.p_sparse,
                d.max_fan_out,
                d.measured_nnz,
                d.predicted_nnz,
                d.mean_out_degree,
                d.p95_out_degree,
                d.mean_readout_fan_in,
                d.mean_hidden_fan_in,
                d.regime,
                d.init_w,
                d.effective_init_w,
                d.readout_boost,
                d.effective_readout_gain,
                d.empty_winner_rate,
                config.matched_budget_repeat,
                config.hash_string(),
                config.protocol_version()
            ));
        }
        json.push('}');
        json
    }

    /// Run C1: local-assembly vs gradient / eligibility refs vs dense-local.
    pub fn run_c1(&mut self, config: &Config) -> C1Report {
        assert!(
            config.n_seeds >= 5,
            "G2 requires ≥5 seeds (got {})",
            config.n_seeds
        );
        let config_hash = config.hash_string();
        let required_n = config.required_scientific_n_seeds();
        let mut seeds_out = Vec::with_capacity(config.n_seeds);
        let mut emitted = Vec::new();
        let mut plot_notes = Vec::new();
        let mut positive_control_acc = Vec::new();
        let mut budget_acc: Vec<(ConditionLabel, BudgetDisclosure)> = Vec::new();

        for seed in config.seeds() {
            let split = freeze_trials(config, seed);
            let local = run_condition_prefer_isolated(
                config,
                seed,
                ConditionLabel::LocalAssembly,
                &split,
                None,
            );
            let dense = run_condition_prefer_isolated(
                config,
                seed,
                ConditionLabel::DenseLocal,
                &split,
                None,
            );
            let dense_matched = if config.matched_budget_repeat {
                Some(run_condition_prefer_isolated(
                    config,
                    seed,
                    ConditionLabel::DenseMatched,
                    &split,
                    Some(local.n_params),
                ))
            } else {
                None
            };
            let grad = run_condition_prefer_isolated(
                config,
                seed,
                ConditionLabel::GradientReference,
                &split,
                None,
            );
            let eprop = run_condition_prefer_isolated(
                config,
                seed,
                ConditionLabel::EligibilityReference,
                &split,
                None,
            );
            positive_control_acc.push(run_positive_control(config, seed));

            let mut cond_outcomes: Vec<(ConditionLabel, &CondOutcome)> = vec![
                (ConditionLabel::LocalAssembly, &local),
                (ConditionLabel::DenseLocal, &dense),
                (ConditionLabel::GradientReference, &grad),
                (ConditionLabel::EligibilityReference, &eprop),
            ];
            if let Some(ref dm) = dense_matched {
                cond_outcomes.push((ConditionLabel::DenseMatched, dm));
            }
            for (cond, outcome) in cond_outcomes {
                let acct = Metrics::activity_compute_account(
                    outcome.budget.work,
                    WorkCosts::unit(),
                    outcome.budget.n_cells,
                    outcome.activity_sparsity,
                );
                let mut entry = RunLog::new(&config_hash, seed, cond.as_str())
                    .with_activity_sparsity(outcome.activity_sparsity)
                    .with_f5_account(
                        acct.event_work,
                        acct.naive_activity_work,
                        acct.work_vs_activity_ratio,
                        outcome.budget.work.source_spikes,
                        outcome.budget.work.synaptic_deliveries,
                    );
                entry.accuracy = outcome.accuracy;
                entry.work_per_accuracy = Some(outcome.budget.work_per_accuracy);
                entry.note = format!(
                    "wall_secs={:.4}_peak_rss={}_spikes={}_deliveries={}_cells={}_plasticity={}",
                    outcome.budget.wall_secs,
                    outcome.budget.peak_rss_bytes,
                    outcome.budget.work.source_spikes,
                    outcome.budget.work.synaptic_deliveries,
                    outcome.budget.work.cell_updates,
                    outcome.budget.work.plasticity_updates
                );
                let line = self
                    .logger
                    .emit_results(&entry)
                    .expect("activity_sparsity set");
                emitted.push(RunRecord { line });
                budget_acc.push((cond, outcome.budget.clone()));
            }

            seeds_out.push(SeedResult {
                seed,
                local_assembly: local.accuracy,
                dense_local: dense.accuracy,
                gradient_reference: grad.accuracy,
                eligibility_reference: eprop.accuracy,
                activity_sparsity: local.activity_sparsity,
                dense_activity_sparsity: dense.activity_sparsity,
                dense_matched: dense_matched.map(|o| o.accuracy),
            });

            let raster_note = self.plots.raster(
                "c1 local-assembly raster",
                Path::new("results/c1_raster.png"),
                &local.raster_t,
                &local.raster_cell,
            );
            plot_notes.push(format!("raster: {raster_note:?}"));
            let w_note = self.plots.weights(
                "c1 local-assembly readout weight",
                Path::new("results/c1_weights.png"),
                &local.weight_steps,
                &local.weight_trace,
            );
            plot_notes.push(format!("weights: {w_note:?}"));
        }

        let summary = summarize_paired(
            &seeds_out,
            config.g2_confidence_z,
            config.g2_min_reference_gap,
        );
        let (positive_control_mean, _) = mean_var(&positive_control_acc);
        let sparsities: Vec<f32> = seeds_out.iter().map(|s| s.activity_sparsity).collect();
        let (mean_activity_sparsity, _) = mean_var(&sparsities);
        let verdict = decide_g2_verdict(
            config,
            &summary,
            positive_control_mean,
            mean_activity_sparsity,
            required_n,
        );

        C1Report {
            config_hash,
            seeds: seeds_out,
            summary,
            verdict,
            positive_control_mean,
            mean_activity_sparsity,
            required_scientific_n_seeds: required_n,
            budgets: budget_acc,
            emitted,
            plot_notes,
        }
    }

    /// Markdown results note body (for `binn/results/c1_g2.md`).
    pub fn render_results_markdown(report: &C1Report, config: &Config) -> String {
        let mut md = String::new();
        md.push_str("# C1 / Gate G2 results note\n\n");
        md.push_str(&format!("**Config hash:** `{}`\n\n", report.config_hash));
        md.push_str(&format!(
            "**Scientific protocol version:** `{}`\n\n",
            config.protocol_version()
        ));
        if config.is_structured_fb_cont_protocol() {
            md.push_str(
                "**claim_axis:** Novel-CS\n\
                 **object_under_test:** Continuous/normalized structured B under muted-θ/k-WTA C1\n\
                 **may_claim:** Whether continuous B∝(w1−w0) beats sign-truncated v15 on gap LCB\n\
                 **must_not_claim:** Hypersearch over B constructions; remassage v15; biology\n\n",
            );
            md.push_str(&format!(
                "**Continuous structured B protocol:** `{C1_STRUCTURED_FB_CONT_PROTOCOL_VERSION}` — same live RFB path as v15, but hidden `B_i` is L2-normalized `(w→r1 − w→r0)` (not sign-truncated); single-pass; **positive control stays on broadcast ±1**; does **not** remassage v15 hash `c1-493ddd56f8714fb6` or reopen protocol-v2 `c1-118207fbc3eaba53`.\n\n"
            ));
        } else if config.is_structured_fb_finth_protocol() {
            md.push_str(
                "**claim_axis:** Integrity\n\
                 **object_under_test:** θ=∞ mute confounder under structured-B credit\n\
                 **may_claim:** Turning mute off (finite θ) under SFB changes / does not change G2\n\
                 **must_not_claim:** Spike-PC remassage; biology; remassage v15\n\n",
            );
            md.push_str(&format!(
                "**Finite-θ under SFB protocol:** `{C1_STRUCTURED_FB_FINTH_PROTOCOL_VERSION}` — v15 structured hidden `B` with **finite θ during integrate** (no θ=∞ mute) + trial-isolation resets; **positive control stays on broadcast ±1**; does **not** remassage v15 or reopen protocol-v2 `c1-118207fbc3eaba53`.\n\n"
            ));
        } else if config.is_structured_fb_soft_protocol() {
            md.push_str(
                "**claim_axis:** Novel-CS\n\
                 **object_under_test:** Soft/relaxed k-WTA winners under structured frozen B\n\
                 **may_claim:** Whether soft winners under SFB close the live transfer gap\n\
                 **must_not_claim:** Temperature grid search; remassage v15; biology\n\n",
            );
            md.push_str(&format!(
                "**Soft-WTA × structured B protocol:** `{C1_STRUCTURED_FB_SOFT_PROTOCOL_VERSION}` — v15 structured hidden `B` with soft/relaxed k-WTA at disclosed temperature `T={C1_SFB_SOFT_TEMPERATURE}` (one temp; no grid); **positive control stays on broadcast ±1**; does **not** remassage v15 hash `c1-493ddd56f8714fb6` or reopen protocol-v2 `c1-118207fbc3eaba53`.\n\n"
            ));
        } else if config.is_dfa_live_protocol() {
            md.push_str(
                "**claim_axis:** Novel-CS\n\
                 **object_under_test:** Graded DFA credit on live muted-θ / k-WTA C1\n\
                 **may_claim:** Whether matched DFA PASS transfers under one honest live map\n\
                 **must_not_claim:** Remassage matched `c1-dfa-*` / P4 spike-DFA; biology; impossibility\n\n",
            );
            md.push_str(&format!(
                "**Live graded-DFA transfer protocol:** `{C1_DFA_LIVE_PROTOCOL_VERSION}` — same muted-θ / k-WTA / single-pass C1 substrate as v2/v13; main-condition plasticity uses graded readout error × fixed-random DFA feedback (`FixedRandomFeedback`) through three-factor eligibility; observe-only on incorrect; **positive control stays on broadcast ±1**; does **not** remassage matched `c1-dfa-c8c4fe0899908b84`, P4 spiking-DFA, or reopen protocol-v2 `c1-118207fbc3eaba53`.\n\n"
            ));
        } else if config.is_structured_fb_teach_protocol() {
            md.push_str(&format!(
                "**Structured B × target teach protocol:** `{C1_STRUCTURED_FB_TEACH_PROTOCOL_VERSION}` — same structured frozen hidden `B` as v15, but incorrect trials restore a secondary target update through `ReinforceFeedback::credit(+1)` (not observe-only); **positive control stays on broadcast ±1**; does **not** remassage v15 hash `c1-493ddd56f8714fb6` or reopen protocol-v2 `c1-118207fbc3eaba53`.\n\n"
            ));
        } else if config.is_elig_rfb_protocol() {
            md.push_str(&format!(
                "**Eligibility × REINFORCE protocol:** `{C1_ELIG_RFB_PROTOCOL_VERSION}` — v15 structured hidden `B` plus eligibility timing co-designed with sampled REINFORCE (`τ_e = {C1_ELIG_RFB_TAU_E}`; mid-trial eligibility absorb after winners/readout before the REINFORCE action); **positive control stays on broadcast ±1**; does **not** remassage v13–v17 hashes or reopen protocol-v2 `c1-118207fbc3eaba53`.\n\n"
            ));
        } else if config.is_structured_fb_capacity_protocol() {
            md.push_str(&format!(
                "**Structured B × capacity protocol:** `{C1_STRUCTURED_FB_CAPACITY_PROTOCOL_VERSION}` — v15 structured hidden `B` on the Tier-B capacity substrate (richer `k_wta` / `n_hidden` / `n_train`); single-pass; **positive control stays on broadcast ±1**; does **not** remassage v15 or capacity-only `c1-d38d7644d8afc84b` or reopen protocol-v2 `c1-118207fbc3eaba53`.\n\n"
            ));
        } else if config.is_structured_fb_epoch_protocol() {
            md.push_str(&format!(
                "**Structured B × epoch-matched protocol:** `{C1_STRUCTURED_FB_EPOCH_PROTOCOL_VERSION}` — v15 structured hidden `B` plus **{}** local/dense epochs over the frozen train split (isolates single-pass handicap under aligned feedback); **positive control stays on broadcast ±1**; does **not** remassage v14/v15 hashes or reopen protocol-v2 `c1-118207fbc3eaba53`.\n\n",
                config.local_train_epochs()
            ));
        } else if config.is_structured_fb_protocol() {
            md.push_str(&format!(
                "**Structured frozen feedback protocol:** `{C1_STRUCTURED_FB_PROTOCOL_VERSION}` — same live RFB plasticity path as v13, but hidden `B_i = sign(w→readout_1 − w→readout_0)` after readout boost (not Uniform[-1,1]); single-pass; **positive control stays on broadcast ±1**; does **not** remassage v13 hash `c1-660401d74db3c88d` or reopen protocol-v2 `c1-118207fbc3eaba53`.\n\n"
            ));
        } else if config.is_reinforce_fb_epoch_protocol() {
            md.push_str(&format!(
                "**Live RFB × epoch-matched protocol:** `{C1_RFB_EPOCH_PROTOCOL_VERSION}` — same neuromodulator as v13 (`ReinforceFeedback` × `reinforce_term`), but local/dense arms train for **{}** epochs over the frozen train split (isolates single-pass handicap); **positive control stays on broadcast ±1**; does **not** remassage v13 hash `c1-660401d74db3c88d` or reopen protocol-v2 `c1-118207fbc3eaba53`.\n\n",
                config.local_train_epochs()
            ));
        } else if config.is_reinforce_fb_protocol() {
            md.push_str(&format!(
                "**Live `ReinforceFeedback` protocol:** `{C1_REINFORCE_FB_PROTOCOL_VERSION}` — same k-WTA / single-pass C1 substrate as v2; main-condition plasticity uses production `ReinforceFeedback` × sampled `reinforce_term` (Bernoulli action from soft readout policy); **positive control stays on broadcast ±1** with a disclosed longer easy-PC schedule (substrate/encoding check; G2 floors unchanged); does **not** reopen protocol-v2 kill-gate hash `c1-118207fbc3eaba53` (canonical version `{C1_PROTOCOL_VERSION}`), remassage P4 spiking-DFA, or retune P5 `rl_graded`.\n\n"
            ));
        } else if config.is_project_protocol() {
            md.push_str(&format!(
                "**Assembly-Calculus `project` protocol:** `{C1_PROJECT_PROTOCOL_VERSION}` — hidden winners from `binn_areas::project` (charge k-WTA + Hebbian imprint) instead of inline membrane-score k-WTA; trial-isolation resets applied; does **not** reopen protocol-v2 kill-gate hash `c1-118207fbc3eaba53` (canonical version `{C1_PROTOCOL_VERSION}`).\n\n"
            ));
        } else if config.is_spike_s_protocol() {
            md.push_str(&format!(
                "**Calibrated natural-spiking protocol:** `{C1_SPIKE_S_PROTOCOL_VERSION}` — finite hidden θ during integrate (no θ=∞ mute); **spike-count k-WTA** (not residual membrane) for hidden selection; disclosed multi-frame easy PC; production knobs `init_w`/`eta`/`tau_e` calibrated; trial-isolation resets; does **not** reopen v2 `c1-118207fbc3eaba53` or reinterpret v6 `c1-09442acdbdc0c752` (G2 thresholds unchanged).\n\n"
            ));
        } else if config.is_spike_protocol() {
            md.push_str(&format!(
                "**Natural-hidden-spiking protocol:** `{C1_SPIKE_PROTOCOL_VERSION}` — finite hidden θ during integrate (no θ=∞ mute); applies trial-isolation membrane + STDP pairing resets; does **not** reopen protocol-v2 kill-gate hash `c1-118207fbc3eaba53` (canonical version `{C1_PROTOCOL_VERSION}`).\n\n"
            ));
        } else if config.is_isolation_protocol() {
            md.push_str(&format!(
                "**Trial-isolation protocol:** `{C1_ISOLATION_PROTOCOL_VERSION}` — clears `ThreeFactor.last_spike` and applies C3-style full dynamic membrane reset at trial boundaries; does **not** reopen protocol-v2 kill-gate hash `c1-118207fbc3eaba53` (canonical version `{C1_PROTOCOL_VERSION}`).\n\n"
            ));
        } else if config.is_sensitivity_protocol() {
            md.push_str(&format!(
                "**Sensitivity protocol (Tier-B):** `{C1_SENSITIVITY_PROTOCOL_VERSION}` — optional confound probe; does **not** reopen protocol-v2 kill-gate hash `c1-118207fbc3eaba53` (canonical version `{C1_PROTOCOL_VERSION}`).\n\n"
            ));
        }
        md.push_str(&format!(
            "**Verdict (Gate G2):** **{}**\n\n",
            report.verdict.as_str()
        ));
        md.push_str(&format!(
            "PASS = lower confidence bound on normalized gradient gap closed > {:.3} and mean local accuracy >= {:.3}.\n",
            config.g2_min_gap_closed, config.g2_min_accuracy
        ));
        md.push_str("FAIL = a full run missed at least one preregistered threshold.\n");
        md.push_str("PILOT = quick schedule or fewer seeds than the power-analysis requirement; not a scientific G2 decision.\n");
        md.push_str(&format!(
            "INVALID_HARNESS = positive_control_mean < {:.3} or mean activity sparsity outside [{:.4}, {:.4}]; prohibits PASS/FAIL and U-NEG language.\n\n",
            config.g2_min_positive_control,
            config.activity_sparsity_min,
            config.activity_sparsity_max
        ));
        md.push_str("## Conditions\n\n");
        md.push_str("| Label | Meaning |\n|---|---|\n");
        if config.uses_live_reinforce_feedback() {
            md.push_str("| `local-assembly` | Three-factor rule + sparse assembly + k-WTA + dual readouts + **`ReinforceFeedback` × `reinforce_term`** (opt-in; not broadcast ±1) |\n");
            md.push_str("| `dense-local` | Same three-factor + k-WTA budget on dense all-to-all, **no** assembly; same `ReinforceFeedback` neuromodulator |\n");
        } else {
            md.push_str("| `local-assembly` | Three-factor rule + sparse assembly wiring + k-WTA + dual readouts + two-sided ±1 reward |\n");
            md.push_str("| `dense-local` | Same three-factor rule + same k-winner budget on dense all-to-all connectivity, **no** assembly structure |\n");
        }
        if config.matched_budget_repeat {
            md.push_str("| `dense-matched` | Dense-local with nnz matched to local-assembly (parameter-matched; measured compute disclosed below) |\n");
        }
        if config.use_surrogate_lif_reference {
            md.push_str("| `gradient-reference` | Same-architecture surrogate-LIF BPTT (primary); tanh RNN optional/secondary |\n");
        } else {
            md.push_str("| `gradient-reference` | Labeled tanh-RNN BPTT (`BpttBaseline`); secondary/optional ceiling |\n");
        }
        md.push_str("| `eligibility-reference` | E-prop-compatible eligibility local reference (rate-model approximation; feedforward-only) |\n\n");
        if config.uses_live_reinforce_feedback() {
            md.push_str(
                "Plasticity uses directional REINFORCE × frozen per-neuron feedback (`ReinforceFeedback`) by design; broadcast ±1 remains the default C1 path. Gap-closed is clamped to `[0, 1]` and seeds with `(reference − dense) < ",
            );
        } else {
            md.push_str("Plasticity uses hard ±1 reward by design (soft RPE deferred). Gap-closed is clamped to `[0, 1]` and seeds with `(reference − dense) < ");
        }
        md.push_str(&format!(
            "{:.3}` contribute `closed = 0`.\n\n",
            config.g2_min_reference_gap
        ));
        md.push_str("## Config\n\n");
        md.push_str(&format!("```\n{config:?}\n```\n\n"));
        md.push_str(&format!(
            "Power analysis: required scientific n_seeds ≥ {} (preregistered σ={:.3}, effect={:.3}; formula n=⌈(1.96+0.8416)²σ²/δ²⌉).\n\n",
            report.required_scientific_n_seeds,
            config.power_sigma_prior,
            config.power_effect_size
        ));
        md.push_str("## Per-seed accuracies\n\n");
        md.push_str("| seed | local-assembly | dense-local | gradient reference | eligibility reference | activity_sparsity (local) | activity_sparsity (dense) | dense_matched |\n");
        md.push_str("|---|---:|---:|---:|---:|---:|---:|---:|\n");
        for s in &report.seeds {
            let matched = s
                .dense_matched
                .map(|v| format!("{v:.4}"))
                .unwrap_or_else(|| "—".into());
            md.push_str(&format!(
                "| {} | {:.4} | {:.4} | {:.4} | {:.4} | {:.4} | {:.4} | {} |\n",
                s.seed,
                s.local_assembly,
                s.dense_local,
                s.gradient_reference,
                s.eligibility_reference,
                s.activity_sparsity,
                s.dense_activity_sparsity,
                matched
            ));
        }
        let sum = &report.summary;
        md.push_str("\n## Summary (paired normalized-gap analysis)\n\n");
        md.push_str(&format!(
            "- mean ± var local-assembly: {:.4} ± {:.6}\n",
            sum.mean_local, sum.var_local
        ));
        md.push_str(&format!(
            "- mean ± var dense-local:    {:.4} ± {:.6}\n",
            sum.mean_dense, sum.var_dense
        ));
        md.push_str(&format!(
            "- mean ± var gradient reference: {:.4} ± {:.6}\n",
            sum.mean_gradient_reference, sum.var_gradient_reference
        ));
        md.push_str(&format!(
            "- mean ± var eligibility reference: {:.4} ± {:.6}\n",
            sum.mean_eligibility_reference, sum.var_eligibility_reference
        ));
        md.push_str(&format!(
            "- mean normalized gap closed: {:.4} (variance {:.6})\n",
            sum.mean_gap_closed, sum.var_gap_closed
        ));
        md.push_str(&format!(
            "- lower confidence bound (z={:.3}, n={}): {:.4}\n",
            config.g2_confidence_z, sum.n, sum.gap_closed_lower_95
        ));
        md.push_str(&format!(
            "- mean |local − dense| (descriptive): {:.4}\n",
            sum.mean_dist_to_dense
        ));
        // Dual-gap harvest (reporting only; gate remains dense-local normalized).
        {
            let (chance_mean, chance_var, chance_lcb) =
                chance_normalized_gap_stats(&report.seeds, config.g2_confidence_z);
            let locals: Vec<f32> = report.seeds.iter().map(|s| s.local_assembly).collect();
            let local_min = locals.iter().copied().fold(f32::INFINITY, f32::min);
            let local_max = locals.iter().copied().fold(f32::NEG_INFINITY, f32::max);
            let frac_ge_floor = if locals.is_empty() {
                0.0
            } else {
                locals
                    .iter()
                    .filter(|&&a| a >= config.g2_min_accuracy)
                    .count() as f32
                    / locals.len() as f32
            };
            md.push_str(&format!(
                "- descriptive chance-normalized gap mean / LCB: {:.4} / {:.4} (var {:.6}; **not a gate**)\n\
                 - seed local min / max / frac≥{:.2}: {:.4} / {:.4} / {:.2}\n\n",
                chance_mean,
                chance_lcb,
                chance_var,
                config.g2_min_accuracy,
                local_min,
                local_max,
                frac_ge_floor
            ));
        }
        match report.verdict {
            GateG2Verdict::Fail => {
                md.push_str("## U-NEG\n\n");
                md.push_str("Negative result: local-assembly did **not** clear the preregistered normalized-gap confidence and absolute-accuracy gates. ");
                md.push_str("Program stops at G2; do not schedule P3+.\n\n");
            }
            GateG2Verdict::Pilot => {
                md.push_str("## Pilot limitation\n\n");
                md.push_str("This run uses a quick schedule or fewer seeds than the power-analysis requirement. It validates the harness only and is not evidence for passing or failing G2.\n\n");
            }
            GateG2Verdict::InvalidHarness => {
                md.push_str("## Invalid harness\n\n");
                md.push_str("Positive control and/or activity sparsity failed the preregistered validity gates. ");
                md.push_str(
                    "No scientific PASS/FAIL or U-NEG claim is permitted from this run.\n\n",
                );
            }
            GateG2Verdict::Pass => {}
        }
        md.push_str(&format!(
            "## Positive / sanity control\n\nMean local-pipeline accuracy on a {} task: **{:.4}** (threshold {:.3}).\n\n",
            if config.uses_temporal_positive_control() {
                "temporal coincidence-lag positive-control"
            } else if config.uses_calibrated_spike_positive_control() {
                "disclosed multi-frame spatial feature-presence (calibrated spike-s PC; main coincidence task unchanged)"
            } else {
                "trivially separable spatial feature-presence"
            },
            report.positive_control_mean, config.g2_min_positive_control
        ));
        md.push_str(&format!(
            "## Activity sparsity\n\nMean local-assembly activity_sparsity: **{:.4}** (valid band [{:.4}, {:.4}]; nominal k/N={:.4}).\n\n",
            report.mean_activity_sparsity,
            config.activity_sparsity_min,
            config.activity_sparsity_max,
            config.nominal_activity_fraction()
        ));
        md.push_str("## Parameter / compute budgets\n\n");
        md.push_str("| condition | n_cells | n_params | wall_secs | peak_rss_bytes | work_per_accuracy | spikes | deliveries | cell_updates | plasticity | event_work | naive_activity_work | work_vs_activity |\n");
        md.push_str("|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|\n");
        // Aggregate first-seed budgets for a compact disclosure table.
        let mut seen = [false; 5];
        for (cond, b) in &report.budgets {
            let idx = match cond {
                ConditionLabel::LocalAssembly => 0,
                ConditionLabel::DenseLocal => 1,
                ConditionLabel::DenseMatched => 2,
                ConditionLabel::GradientReference => 3,
                ConditionLabel::EligibilityReference => 4,
            };
            if seen[idx] {
                continue;
            }
            seen[idx] = true;
            let sparsity = match cond {
                ConditionLabel::LocalAssembly => report.mean_activity_sparsity,
                ConditionLabel::DenseLocal => {
                    report
                        .seeds
                        .iter()
                        .map(|s| s.dense_activity_sparsity)
                        .sum::<f32>()
                        / report.seeds.len().max(1) as f32
                }
                _ => report.mean_activity_sparsity,
            };
            let acct =
                Metrics::activity_compute_account(b.work, WorkCosts::unit(), b.n_cells, sparsity);
            md.push_str(&format!(
                "| {} | {} | {} | {:.4} | {} | {:.4} | {} | {} | {} | {} | {:.1} | {:.1} | {:.2} |\n",
                cond.as_str(),
                b.n_cells,
                b.n_params,
                b.wall_secs,
                b.peak_rss_bytes,
                b.work_per_accuracy,
                b.work.source_spikes,
                b.work.synaptic_deliveries,
                b.work.cell_updates,
                b.work.plasticity_updates,
                acct.event_work,
                acct.naive_activity_work,
                acct.work_vs_activity_ratio
            ));
        }
        // Matched-budget summary row (mean accuracy across seeds) when present.
        let matched_accs: Vec<f32> = report
            .seeds
            .iter()
            .filter_map(|s| s.dense_matched)
            .collect();
        if !matched_accs.is_empty() {
            let (mean_m, _) = mean_var(&matched_accs);
            md.push_str(&format!(
                "\nMatched-budget dense mean accuracy: **{:.4}** (n={}; primary G2 gap still uses unmatched dense-local).\n",
                mean_m,
                matched_accs.len()
            ));
        }
        md.push('\n');
        md.push_str("## Plots\n\n");
        for note in &report.plot_notes {
            md.push_str(&format!("- {note}\n"));
        }
        md.push_str("\n## Structured log (GC7)\n\n```\n");
        for e in &report.emitted {
            md.push_str(&e.line);
            md.push('\n');
        }
        md.push_str("```\n");
        md
    }
}

/// Geometry / integrity diagnostics for mac-probe isolate JSON.
#[derive(Clone, Debug, PartialEq)]
pub struct MacProbeDiagnostics {
    pub measured_nnz: usize,
    pub max_fan_out: usize,
    pub predicted_nnz: usize,
    pub mean_out_degree: f32,
    pub p95_out_degree: f32,
    pub mean_readout_fan_in: f32,
    pub mean_hidden_fan_in: f32,
    pub regime: &'static str,
    pub init_w: f32,
    pub effective_init_w: f32,
    pub readout_boost: f32,
    pub effective_readout_gain: f32,
    pub empty_winner_rate: f32,
}

#[derive(Clone, Debug)]
struct CondOutcome {
    accuracy: f32,
    activity_sparsity: f32,
    n_params: usize,
    budget: BudgetDisclosure,
    raster_t: Vec<f64>,
    raster_cell: Vec<f64>,
    weight_steps: Vec<f64>,
    weight_trace: Vec<f64>,
    mac_probe: Option<MacProbeDiagnostics>,
}

pub fn freeze_trials(config: &Config, seed: u64) -> FrozenSplit {
    let mut task = CoincidenceTask::new(seed, config.sequence_len, config.max_lag);
    let train = (0..config.n_train).map(|_| task.next_trial()).collect();
    let mut task_te = CoincidenceTask::new(seed ^ 0x7E57_0001, config.sequence_len, config.max_lag);
    let test = (0..config.n_test).map(|_| task_te.next_trial()).collect();
    FrozenSplit { train, test }
}

#[cfg(test)]
fn freeze_trials_shuffled(config: &Config, seed: u64) -> FrozenSplit {
    let mut split = freeze_trials(config, seed);
    let mut rng = Rng::new(seed ^ 0x5A1F_1AB3);
    // Fisher–Yates shuffle of train labels only (features unchanged).
    let n = split.train.len();
    for i in 0..n {
        let j = i + rng.gen_index(n - i);
        let li = split.train[i].1;
        let lj = split.train[j].1;
        split.train[i].1 = lj;
        split.train[j].1 = li;
        for s in &mut split.train[i].0 {
            s.label = Some(lj);
        }
        for s in &mut split.train[j].0 {
            s.label = Some(li);
        }
    }
    split
}

fn run_labeled_condition(
    config: &Config,
    seed: u64,
    label: ConditionLabel,
    split: &FrozenSplit,
    match_nnz: Option<usize>,
) -> CondOutcome {
    match label {
        ConditionLabel::LocalAssembly => run_local_assembly(config, seed, split),
        ConditionLabel::DenseLocal => run_dense_local(config, seed, split, None),
        ConditionLabel::DenseMatched => run_dense_local(config, seed, split, match_nnz),
        ConditionLabel::GradientReference => run_gradient_reference(config, seed, split),
        ConditionLabel::EligibilityReference => run_eligibility_reference(config, seed, split),
    }
}

/// Prefer a fresh subprocess so peak RSS is attributable to one condition.
/// Falls back in-process inside unit tests / when the c1 binary is unavailable.
///
/// With `--features plots`, always run in-process: the isolate JSON omits
/// raster/weight traces the plot bridge needs. Accuracies and budgets are
/// unchanged; only peak-RSS attribution is coarser during plot runs.
fn run_condition_prefer_isolated(
    config: &Config,
    seed: u64,
    label: ConditionLabel,
    split: &FrozenSplit,
    match_nnz: Option<usize>,
) -> CondOutcome {
    #[cfg(not(feature = "plots"))]
    if std::env::var_os("BINN_CONDITION_CHILD").is_none() {
        if let Some(outcome) = try_isolate_condition(config, seed, label, match_nnz) {
            return outcome;
        }
    }
    run_labeled_condition(config, seed, label, split, match_nnz)
}

#[cfg(not(feature = "plots"))]
fn try_isolate_condition(
    config: &Config,
    seed: u64,
    label: ConditionLabel,
    match_nnz: Option<usize>,
) -> Option<CondOutcome> {
    use std::process::Command;
    let exe = std::env::current_exe().ok()?;
    // Only isolate when running the c1 binary (not libtest harnesses).
    let name = exe.file_name()?.to_string_lossy();
    if !name.contains("c1") {
        return None;
    }
    let mut cmd = Command::new(&exe);
    cmd.env("BINN_CONDITION_CHILD", "1");
    cmd.arg("--isolate-condition").arg(label.as_str());
    cmd.arg("--seed").arg(seed.to_string());
    cmd.arg("--config-hash").arg(config.hash_string());
    if let Some(n) = match_nnz {
        cmd.arg("--match-nnz").arg(n.to_string());
    }
    let out = cmd.output().ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8(out.stdout).ok()?;
    let line = text.lines().find(|l| l.trim_start().starts_with('{'))?;
    parse_condition_json(line)
}

#[cfg(not(feature = "plots"))]
fn parse_condition_json(line: &str) -> Option<CondOutcome> {
    fn field<'a>(s: &'a str, key: &str) -> Option<&'a str> {
        let pat = format!("\"{key}\":");
        let i = s.find(&pat)?;
        let rest = &s[i + pat.len()..];
        let rest = rest.trim_start();
        if let Some(inner) = rest.strip_prefix('"') {
            let end = inner.find('"')?;
            Some(&inner[..end])
        } else {
            let end = rest.find([',', '}']).unwrap_or(rest.len());
            Some(rest[..end].trim())
        }
    }
    let accuracy: f32 = field(line, "accuracy")?.parse().ok()?;
    let activity_sparsity: f32 = field(line, "activity_sparsity")?.parse().ok()?;
    let n_cells: usize = field(line, "n_cells")?.parse().ok()?;
    let n_params: usize = field(line, "n_params")?.parse().ok()?;
    let wall_secs: f64 = field(line, "wall_secs")?.parse().ok()?;
    let peak_rss_bytes: u64 = field(line, "peak_rss_bytes")?.parse().ok()?;
    let source_spikes: u64 = field(line, "source_spikes")?.parse().ok()?;
    let synaptic_deliveries: u64 = field(line, "synaptic_deliveries")?.parse().ok()?;
    let cell_updates: u64 = field(line, "cell_updates")?.parse().ok()?;
    let plasticity_updates: u64 = field(line, "plasticity_updates")?.parse().ok()?;
    let work_per_accuracy: f64 = field(line, "work_per_accuracy")?.parse().ok()?;
    Some(CondOutcome {
        accuracy,
        activity_sparsity,
        n_params,
        budget: BudgetDisclosure {
            n_cells,
            n_params,
            work: WorkCounters {
                source_spikes,
                synaptic_deliveries,
                cell_updates,
                plasticity_updates,
            },
            wall_secs,
            peak_rss_bytes,
            work_per_accuracy,
        },
        raster_t: Vec::new(),
        raster_cell: Vec::new(),
        weight_steps: Vec::new(),
        weight_trace: Vec::new(),
        mac_probe: None,
    })
}

/// Convert frozen trials into the dense temporal form the shared-forward stack
/// takes.
///
/// The canonical converter for [`binn_learn::SharedTemporalNet`]. Frames are the
/// flat `timesteps x n_in` layout that `SharedTemporalNet::forward` indexes as
/// `frames[t * n_in + channel]`; the label is the trial's class id, kept as a
/// class index rather than the `f32` the two-class `GradientExample` form uses,
/// because the shared stack is multi-class.
///
/// Sequences shorter than [`REFERENCE_SEQUENCE_LEN`] are zero-padded and longer
/// ones truncated, matching [`samples_to_gradient_examples`] so the two forms of
/// the same trial contain the same data.
pub fn samples_to_dense_temporal_examples(
    trials: &[(Vec<Sample>, u32)],
    n_in: usize,
) -> Vec<DenseTemporalExample> {
    assert!(
        n_in > 0,
        "dense temporal examples need at least one channel"
    );
    trials
        .iter()
        .map(|(sequence, label)| {
            let mut frames = vec![0.0f32; REFERENCE_SEQUENCE_LEN * n_in];
            for (t, sample) in sequence.iter().enumerate().take(REFERENCE_SEQUENCE_LEN) {
                for channel in 0..n_in {
                    frames[t * n_in + channel] = sample.values.get(channel).copied().unwrap_or(0.0);
                }
            }
            DenseTemporalExample {
                frames,
                timesteps: REFERENCE_SEQUENCE_LEN,
                n_in,
                label: *label,
            }
        })
        .collect()
}

/// Temporal-order trials in the flat form [`binn_learn::SharedTemporalNet`]
/// indexes.
///
/// The sibling of [`samples_to_dense_temporal_examples`] for the temporal-order
/// task. Both the length and the channel count come from `binn_data`'s
/// [`TEMPORAL_ORDER_T`] / [`TEMPORAL_ORDER_N_IN`], so the task generator stays
/// the single owner of the geometry and no caller can frame a trial at a shape
/// the generator did not emit.
pub fn temporal_order_to_dense_examples(
    examples: &[TemporalOrderExample],
) -> Vec<DenseTemporalExample> {
    examples
        .iter()
        .map(|example| DenseTemporalExample {
            frames: example.frames.clone(),
            timesteps: TEMPORAL_ORDER_T,
            n_in: TEMPORAL_ORDER_N_IN,
            label: example.label,
        })
        .collect()
}

/// The same temporal-order trials in the form the SHD stack takes.
///
/// [`ShdExample`] and [`DenseTemporalExample`] hold the identical flat frame
/// buffer under different field names (`t` vs `timesteps`); this is that
/// rename and nothing else, so the two views of a trial always carry the same
/// data.
pub fn temporal_order_to_shd_examples(examples: &[TemporalOrderExample]) -> Vec<ShdExample> {
    examples
        .iter()
        .map(|example| ShdExample {
            frames: example.frames.clone(),
            t: TEMPORAL_ORDER_T,
            n_in: TEMPORAL_ORDER_N_IN,
            label: example.label,
        })
        .collect()
}

pub fn samples_to_gradient_examples(trials: &[(Vec<Sample>, u32)]) -> Vec<GradientExample> {
    trials
        .iter()
        .map(|(sequence, label)| {
            let mut x1 = [0.0f32; REFERENCE_SEQUENCE_LEN];
            let mut x2 = [0.0f32; REFERENCE_SEQUENCE_LEN];
            for (t, sample) in sequence.iter().enumerate().take(REFERENCE_SEQUENCE_LEN) {
                x1[t] = sample.values[0];
                x2[t] = sample.values[1];
            }
            (x1, x2, *label as f32)
        })
        .collect()
}

fn run_gradient_reference(config: &Config, seed: u64, split: &FrozenSplit) -> CondOutcome {
    assert_eq!(
        config.sequence_len, REFERENCE_SEQUENCE_LEN,
        "C1 gradient reference currently requires sequence_len={REFERENCE_SEQUENCE_LEN}"
    );
    let train = samples_to_gradient_examples(&split.train);
    let test = samples_to_gradient_examples(&split.test);
    let wall = Instant::now();
    let rss0 = peak_rss_bytes();
    let report = if config.use_surrogate_lif_reference {
        let mut r = SurrogateLifReference::new(
            config.n_hidden,
            config.bptt_lr,
            config.surrogate_beta,
            seed,
        );
        r.train_and_evaluate(config.bptt_epochs, &train, &test)
    } else {
        let mut r = BpttBaseline::new(config.bptt_lr, seed);
        r.train_and_evaluate(config.bptt_epochs, &train, &test)
    };
    let wall_secs = wall.elapsed().as_secs_f64();
    let peak_rss = peak_rss_bytes().max(rss0);
    let n_params = if config.use_surrogate_lif_reference {
        // win + wrec + wout + bias
        config.n_hidden * 2 + config.n_hidden * config.n_hidden + config.n_hidden + 1
    } else {
        // tanh RNN: hard-coded H=4 in BpttBaseline
        4 * 2 + 4 * 4 + 4 + 1
    };
    let work = WorkCounters {
        source_spikes: 0,
        synaptic_deliveries: (config.bptt_epochs * train.len() * config.sequence_len) as u64,
        cell_updates: (config.bptt_epochs * train.len() * config.n_hidden * config.sequence_len)
            as u64,
        plasticity_updates: (config.bptt_epochs * train.len() * n_params) as u64,
    };
    let wpa = Metrics::work_per_accuracy(work, WorkCosts::unit(), report.accuracy.max(1e-6) as f64);
    CondOutcome {
        accuracy: report.accuracy,
        activity_sparsity: 1.0,
        n_params,
        budget: BudgetDisclosure {
            n_cells: config.n_hidden + 2,
            n_params,
            work,
            wall_secs,
            peak_rss_bytes: peak_rss,
            work_per_accuracy: wpa,
        },
        raster_t: Vec::new(),
        raster_cell: Vec::new(),
        weight_steps: Vec::new(),
        weight_trace: Vec::new(),
        mac_probe: None,
    }
}

fn run_eligibility_reference(config: &Config, seed: u64, split: &FrozenSplit) -> CondOutcome {
    assert_eq!(
        config.sequence_len, REFERENCE_SEQUENCE_LEN,
        "C1 eligibility reference currently requires sequence_len={REFERENCE_SEQUENCE_LEN}"
    );
    let train = samples_to_gradient_examples(&split.train);
    let test = samples_to_gradient_examples(&split.test);
    let wall = Instant::now();
    let rss0 = peak_rss_bytes();
    let mut r = EpropReference::new(
        config.n_hidden,
        config.bptt_lr,
        config.surrogate_beta,
        seed ^ 0xE700_4EF1,
    );
    let report = r.train_and_evaluate(config.bptt_epochs, &train, &test);
    let wall_secs = wall.elapsed().as_secs_f64();
    let peak_rss = peak_rss_bytes().max(rss0);
    let n_params = config.n_hidden * 2 + config.n_hidden + 1;
    let work = WorkCounters {
        source_spikes: 0,
        synaptic_deliveries: (config.bptt_epochs * train.len() * config.sequence_len) as u64,
        cell_updates: (config.bptt_epochs * train.len() * config.n_hidden * config.sequence_len)
            as u64,
        plasticity_updates: (config.bptt_epochs * train.len() * n_params) as u64,
    };
    let wpa = Metrics::work_per_accuracy(work, WorkCosts::unit(), report.accuracy.max(1e-6) as f64);
    CondOutcome {
        accuracy: report.accuracy,
        activity_sparsity: 1.0,
        n_params,
        budget: BudgetDisclosure {
            n_cells: config.n_hidden + 2,
            n_params,
            work,
            wall_secs,
            peak_rss_bytes: peak_rss,
            work_per_accuracy: wpa,
        },
        raster_t: Vec::new(),
        raster_cell: Vec::new(),
        weight_steps: Vec::new(),
        weight_trace: Vec::new(),
        mac_probe: None,
    }
}

fn run_local_assembly(config: &Config, seed: u64, split: &FrozenSplit) -> CondOutcome {
    run_spiking_condition(config, seed, true, false, split, None)
}

/// Positive/sanity control: same local-assembly pipeline on a trivially
/// separable task.
///
/// Protocol-v2 defaults use a **spatial** feature-presence task. Tier-B
/// `c1-sens-temporal-pc` presets switch to a **temporal coincidence-lag**
/// positive control under the same LatencyEncoder + spike/WTA path.
pub(crate) fn run_positive_control(config: &Config, seed: u64) -> f32 {
    // Give the sanity control enough trials to clear the harness floor even on
    // the short quick schedule (scientific configs already have n_train ≥ 80).
    // Temporal coincidence needs more exposure than spatial feature-presence.
    // Calibrated spike-s uses a disclosed multi-frame easy task + longer PC
    // schedule (learner path stays natural-spiking; thresholds unchanged).
    let mut cfg = config.clone();
    let rfb_pc = cfg.uses_live_reinforce_feedback();
    // Protocol v13: PC validates encoding/WTA/substrate under default broadcast
    // ±1. Main coincidence arms still use ReinforceFeedback (disclosed in the
    // results note). Does not flip default C1; only isolates the harness check.
    if rfb_pc {
        cfg.experiment = "c1".into();
    }
    let min_train = if cfg.uses_temporal_positive_control() {
        128
    } else if cfg.uses_calibrated_spike_positive_control() || rfb_pc {
        // spike-s and v13: disclosed longer PC (G2 floor unchanged).
        96
    } else {
        48
    };
    let min_test = if cfg.uses_temporal_positive_control() {
        40
    } else if cfg.uses_calibrated_spike_positive_control() || rfb_pc {
        32
    } else {
        24
    };
    cfg.n_train = cfg.n_train.max(min_train);
    cfg.n_test = cfg.n_test.max(min_test);
    let mut easy_rng = Rng::new(seed ^ 0x0E51_EA51);
    let easy_len = cfg.sequence_len.max(2);
    let train: Vec<_> = (0..cfg.n_train)
        .map(|_| {
            if cfg.uses_temporal_positive_control() {
                temporal_easy_trial(&mut easy_rng, easy_len, cfg.max_lag)
            } else if cfg.uses_calibrated_spike_positive_control() {
                calibrated_spike_easy_trial(&mut easy_rng, easy_len)
            } else {
                easy_trial(&mut easy_rng, easy_len)
            }
        })
        .collect();
    let test: Vec<_> = (0..cfg.n_test)
        .map(|_| {
            if cfg.uses_temporal_positive_control() {
                temporal_easy_trial(&mut easy_rng, easy_len, cfg.max_lag)
            } else if cfg.uses_calibrated_spike_positive_control() {
                calibrated_spike_easy_trial(&mut easy_rng, easy_len)
            } else {
                easy_trial(&mut easy_rng, easy_len)
            }
        })
        .collect();
    let split = FrozenSplit { train, test };
    run_spiking_condition(&cfg, seed, true, true, &split, None).accuracy
}

/// Trivially separable trial: feature 0 present ⇒ label 1, feature 1 ⇒ label 0.
fn easy_trial(rng: &mut Rng, len: usize) -> (Vec<Sample>, u32) {
    let label = u32::from(rng.next_f32() < 0.5);
    let frame = len / 2;
    let mut seq: Vec<Sample> = (0..len)
        .map(|_| Sample::from_values(vec![0.0, 0.0]))
        .collect();
    let active = if label == 1 { 0usize } else { 1usize };
    seq[frame].values[active] = 0.95;
    for s in &mut seq {
        s.label = Some(label);
    }
    (seq, label)
}

/// Disclosed easier PC for calibrated natural-spiking (`c1-spike-s*` only).
///
/// Same spatial feature→label mapping as [`easy_trial`], but the active feature
/// is pulsed across three mid-sequence frames at full amplitude so spike-count
/// k-WTA receives a reliable class-selective integrate-window signal. Main
/// coincidence task is unchanged.
fn calibrated_spike_easy_trial(rng: &mut Rng, len: usize) -> (Vec<Sample>, u32) {
    let label = u32::from(rng.next_f32() < 0.5);
    let mut seq: Vec<Sample> = (0..len)
        .map(|_| Sample::from_values(vec![0.0, 0.0]))
        .collect();
    let active = if label == 1 { 0usize } else { 1usize };
    let mid = len / 2;
    let lo = mid.saturating_sub(1);
    let hi = (mid + 1).min(len.saturating_sub(1));
    for slot in &mut seq[lo..=hi] {
        slot.values[active] = 1.0;
    }
    for s in &mut seq {
        s.label = Some(label);
    }
    (seq, label)
}

/// Temporal coincidence-lag positive control under the same encoding as C1.
///
/// Equal-count peaks, fixed anchors near the decision horizon:
/// - label 1: feature 0 and feature 1 fire within `max_lag` on the last frames
///   (short-lag coincidence the membrane still holds at k-WTA time)
/// - label 0: the same two peaks, but feature 1 is placed at frame 0 so the
///   lag ≫ `max_lag` and its drive has leaked away by decision time
///
/// Proves the local LatencyEncoder + spike/WTA path can bind coincidence lag
/// when the spatial feature-presence PC is not the diagnostic of interest.
fn temporal_easy_trial(rng: &mut Rng, len: usize, max_lag: usize) -> (Vec<Sample>, u32) {
    let max_lag = max_lag.max(1);
    let len = len.max(max_lag + 3);
    let label = u32::from(rng.next_f32() < 0.5);
    let mut seq: Vec<Sample> = (0..len)
        .map(|_| Sample::from_values(vec![0.0, 0.0]))
        .collect();
    let t0 = len - 1 - max_lag.min(1);
    seq[t0].values[0] = 0.95;
    if label == 1 {
        seq[t0 + max_lag.min(1)].values[1] = 0.95;
    } else {
        seq[0].values[1] = 0.95;
    }
    for s in &mut seq {
        s.label = Some(label);
    }
    (seq, label)
}

fn run_dense_local(
    config: &Config,
    seed: u64,
    split: &FrozenSplit,
    match_nnz: Option<usize>,
) -> CondOutcome {
    run_spiking_condition(config, seed, false, false, split, match_nnz)
}

fn run_spiking_condition(
    config: &Config,
    seed: u64,
    assembly: bool,
    _easy: bool,
    split: &FrozenSplit,
    match_nnz: Option<usize>,
) -> CondOutcome {
    let n_in = 2usize;
    let n_hidden = config.n_hidden;
    let readout_0 = (n_in + n_hidden) as CellId;
    let readout_1 = readout_0 + 1;
    let n_cells = n_in + n_hidden + 2;

    let wall = Instant::now();
    let rss0 = peak_rss_bytes();

    let mut eng = Engine::with_cells(n_cells);
    let (conn, base_init_w) = if assembly {
        build_sparse_assembly(config, seed, n_in, n_hidden, readout_0, readout_1)
    } else {
        build_dense_local(
            config, seed, n_in, n_hidden, readout_0, readout_1, match_nnz,
        )
    };
    let nnz = conn.nnz();
    let geom = assembly_geometry_stats(&conn, n_in, n_hidden, readout_0, readout_1);
    let init_w = crate::mac_probe_config::effective_init_w(
        base_init_w,
        geom.mean_hidden_fan_in,
        config.init_w_rescale,
    );
    eng.set_connectivity(conn, vec![init_w; nnz]);
    // Raise readout drive so one connected hidden winner can cross θ=1.
    // Scaling the incoming readout edges (not θ) keeps the cell model shared
    // while the charge fallback remains symmetric when both/neither fire.
    // Calibrated spike-s uses a slightly higher boost (disclosed) so forced
    // winners still reach readout under noisier integrate-window eligibility.
    // Mac-probe gain-normalize holds boost × mean_readout_fan_in ≈ baseline.
    let (readout_boost, effective_readout_gain) = if config.is_spike_s_protocol() {
        let b = (1.35 / init_w.max(1e-3)).clamp(1.0, 14.0);
        (b, b * geom.mean_readout_fan_in.max(1.0))
    } else {
        crate::mac_probe_config::readout_boost_and_gain(
            init_w,
            geom.mean_readout_fan_in,
            config.readout_gain_normalize,
        )
    };
    boost_readout_incoming(&mut eng, readout_0, readout_1, readout_boost);

    let mut area = Area::new(n_in as CellId..(n_in + n_hidden) as CellId, config.k_wta);
    let mut learner = ThreeFactor::new(config.eta, config.lambda, config.tau_e);
    // Spike / project protocols also isolate trial boundaries so new mechanisms
    // are not confounded by sticky last_spike / dendrite residue.
    let trial_isolation = config.is_isolation_protocol()
        || config.is_spike_protocol()
        || config.is_project_protocol()
        || config.is_structured_fb_finth_protocol();
    let natural_spiking = config.is_spike_protocol() || config.is_structured_fb_finth_protocol();
    let spike_count_wta = config.is_spike_s_protocol();
    let use_project = config.is_project_protocol();
    // Opt-in live ReinforceFeedback family (v13–v19, v21, v23–v25). Default C1
    // keeps broadcast ±1 (reinforce_fb is None). Graded-DFA live (v20) uses
    // FixedRandomFeedback instead.
    let mut b_learned_fb = if config.is_reinforce_fb_learned_protocol() {
        Some(LearnedReinforceFeedback::new(n_cells, seed, 0.01))
    } else {
        None
    };
    let reinforce_fb = if config.uses_live_reinforce_feedback() && b_learned_fb.is_none() {
        Some(if config.uses_structured_feedback_weights() {
            structured_reinforce_feedback(
                &eng,
                n_cells,
                n_in,
                n_hidden,
                readout_0,
                readout_1,
                seed,
                config.uses_continuous_structured_feedback(),
            )
        } else {
            ReinforceFeedback::new(n_cells, seed)
        })
    } else {
        None
    };
    let dfa_live_fb = if config.is_dfa_live_protocol() {
        Some(FixedRandomFeedback::new(n_cells, 2, seed ^ 0xDFA0_11FE))
    } else {
        None
    };
    let soft_wta_temp = config.soft_k_wta_temperature();
    let elig_preabsorb = config.uses_elig_rfb_preabsorb();
    let structured_target_teach = config.uses_structured_target_teach();
    // REINFORCE action sampler for live RFB family (Bernoulli from soft policy).
    let mut reinforce_rng = Rng::new(seed ^ 0xAFB1_AC71_0000_00A1);
    // Per-frame latency bins; full event stream uses frame_offset + latency.
    let enc = LatencyEncoder::new(2, (config.sequence_len as Tick).max(1), 0);

    let mut weight_steps = Vec::new();
    let mut weight_trace = Vec::new();
    let mut t_cursor: Tick = 0;
    let mut plasticity_updates = 0u64;

    // Opt-in replay capture (viz only): read-only over engine state, no
    // effect on config hashes, accuracies, budgets, or the GC7 log.
    let replay_out = if assembly { replay_out_path() } else { None };
    let mut replay_trials: Vec<ReplayTrial> = Vec::new();

    // Opt-in JSONL trace (viz only): local-assembly, one seed.
    let mut trace = if assembly {
        match (trace_out_path(), trace_export_seed()) {
            (Some(path), Some(want)) if want == seed => Some(TraceExport {
                path,
                recorder: TraceRecorder::new(),
                assembly_hits: vec![vec![0u32; n_hidden]; 2],
            }),
            _ => None,
        }
    } else {
        None
    };
    if let Some(ref mut tr) = trace {
        tr.recorder.emit_meta(
            &config.hash_string(),
            seed,
            ConditionLabel::LocalAssembly.as_str(),
            "c1",
            2,
            config.k_wta as u32,
            n_hidden as u32,
        );
        let areas = [
            TraceArea {
                id: 0,
                name: "input".into(),
                start: 0,
                end: n_in as u32,
            },
            TraceArea {
                id: 1,
                name: "hidden".into(),
                start: n_in as u32,
                end: (n_in + n_hidden) as u32,
            },
            TraceArea {
                id: 2,
                name: "readout".into(),
                start: readout_0,
                end: readout_1 + 1,
            },
        ];
        let projections = [
            TraceProjection {
                src: 0,
                dst: 1,
                nnz: projection_nnz(
                    &eng.conn,
                    0,
                    n_in as u32,
                    n_in as u32,
                    (n_in + n_hidden) as u32,
                ),
                coupling: None,
            },
            TraceProjection {
                src: 1,
                dst: 1,
                nnz: projection_nnz(
                    &eng.conn,
                    n_in as u32,
                    (n_in + n_hidden) as u32,
                    n_in as u32,
                    (n_in + n_hidden) as u32,
                ),
                coupling: None,
            },
            TraceProjection {
                src: 1,
                dst: 2,
                nnz: projection_nnz(
                    &eng.conn,
                    n_in as u32,
                    (n_in + n_hidden) as u32,
                    readout_0,
                    readout_1 + 1,
                ),
                coupling: None,
            },
        ];
        tr.recorder.emit_topology(&areas, &projections);
        let edges = collect_weight_frame_edges(&eng, n_in, n_hidden, readout_0, readout_1);
        tr.recorder.emit_weight_frame(0, "before", &edges);
    }

    let local_epochs = config.local_train_epochs();
    let mut global_step = 0usize;
    for _epoch in 0..local_epochs {
        for (step, (seq, label)) in split.train.iter().enumerate() {
            let trial_t0 = t_cursor;
            let trial_idx = global_step as u32;
            let record_detail = global_step < TRACE_EARLY_TRAIN;
            if config.is_k_anneal_protocol() {
                let total_steps = (local_epochs * split.train.len()).max(1) as f32;
                let progress = (global_step as f32) / total_steps;
                let k_start = if config.quick { 8.0f32 } else { 16.0f32 };
                let k_end = if config.quick { 1.0f32 } else { 2.0f32 };
                let current_k = (k_start + progress * (k_end - k_start)).round().max(1.0) as usize;
                area.k = current_k;
            }
            let mut trial_trace = trace.as_mut().map(|tr| TraceTrialHook {
                recorder: &mut tr.recorder,
                trial: trial_idx,
                phase: "train",
                record_spikes_kwta: record_detail,
                record_elig: record_detail,
                step: (global_step + 1) as u32,
                assembly_hits: None,
                n_in,
            });
            let (_ok, _s, n_plas) = run_trial(
                &mut eng,
                &mut learner,
                &mut area,
                &enc,
                seq,
                *label,
                readout_0,
                readout_1,
                t_cursor,
                true,
                trial_isolation,
                natural_spiking,
                spike_count_wta,
                use_project,
                reinforce_fb.as_ref(),
                b_learned_fb.as_mut(),
                Some(&mut reinforce_rng),
                elig_preabsorb,
                structured_target_teach,
                dfa_live_fb.as_ref(),
                soft_wta_temp,
                seed ^ (global_step as u64),
                trial_trace.as_mut(),
            );
            plasticity_updates = plasticity_updates.saturating_add(n_plas);
            if replay_out.is_some() {
                replay_trials.push(ReplayTrial {
                    phase: "train",
                    label: *label,
                    t0: trial_t0,
                    t1: eng.time(),
                    correct: None,
                });
            }
            t_cursor = eng.time() + 20;
            if let Some(w) = mean_readout_weight(&eng, readout_0, readout_1) {
                weight_steps.push(global_step as f64);
                weight_trace.push(w as f64);
            }
            let _ = step; // epoch-major indexing uses global_step
            global_step += 1;
        }
    }

    if let Some(ref mut tr) = trace {
        let edges = collect_weight_frame_edges(&eng, n_in, n_hidden, readout_0, readout_1);
        tr.recorder
            .emit_weight_frame(config.n_train as u32, "after", &edges);
    }

    if config.is_k_anneal_protocol() {
        area.k = if config.quick { 1 } else { 2 };
    }

    let mut correct = 0usize;
    let mut active_total = 0usize;
    let mut pop_total = 0usize;
    let mut empty_winner_trials = 0usize;
    let mut sparsity_trials = 0usize;
    let mut raster_t = Vec::new();
    let mut raster_cell = Vec::new();

    for (trial_i, (seq, label)) in split.test.iter().enumerate() {
        let trial_t0 = t_cursor;
        let trial_idx = (config.n_train + trial_i) as u32;
        let mut trial_trace = trace.as_mut().map(|tr| TraceTrialHook {
            recorder: &mut tr.recorder,
            trial: trial_idx,
            phase: "test",
            record_spikes_kwta: true,
            record_elig: false,
            step: 0,
            assembly_hits: Some(&mut tr.assembly_hits),
            n_in,
        });
        let (pred_ok, sample, _) = run_trial(
            &mut eng,
            &mut learner,
            &mut area,
            &enc,
            seq,
            *label,
            readout_0,
            readout_1,
            t_cursor,
            false,
            trial_isolation,
            natural_spiking,
            spike_count_wta,
            use_project,
            reinforce_fb.as_ref(),
            b_learned_fb.as_mut(),
            Some(&mut reinforce_rng),
            elig_preabsorb,
            structured_target_teach,
            dfa_live_fb.as_ref(),
            soft_wta_temp,
            seed ^ 0x7E57_0000,
            trial_trace.as_mut(),
        );
        if pred_ok {
            correct += 1;
        }
        active_total += sample.active;
        pop_total += sample.population;
        sparsity_trials += 1;
        if sample.active == 0 {
            empty_winner_trials += 1;
        }
        if trial_i == 0 {
            for sp in eng.spikes().as_slice().iter().rev().take(64) {
                raster_t.push(sp.t as f64);
                raster_cell.push(sp.cell as f64);
            }
        }
        if replay_out.is_some() {
            replay_trials.push(ReplayTrial {
                phase: "test",
                label: *label,
                t0: trial_t0,
                t1: eng.time(),
                correct: Some(pred_ok),
            });
        }
        t_cursor = eng.time() + 20;
    }

    if let Some(ref mut tr) = trace {
        for (label, hits) in tr.assembly_hits.iter().enumerate() {
            let mut members = Vec::new();
            let mut member_hits = Vec::new();
            for (offset, &h) in hits.iter().enumerate() {
                if h > 0 {
                    members.push((n_in + offset) as u32);
                    member_hits.push(h);
                }
            }
            if !members.is_empty() {
                tr.recorder
                    .emit_assembly_class(label as u32, &members, Some(&member_hits));
            }
        }
        match tr.recorder.write_jsonl(&tr.path) {
            Ok(()) => eprintln!("trace export written: {}", tr.path.display()),
            Err(e) => eprintln!("trace export failed ({}): {e}", tr.path.display()),
        }
    }

    let accuracy = correct as f32 / config.n_test.max(1) as f32;
    let activity_sparsity = if pop_total == 0 {
        0.0
    } else {
        Metrics::sparsity(active_total.min(pop_total), pop_total)
    };

    let ew = eng.work();
    let mut work = WorkCounters {
        source_spikes: ew.source_spikes,
        synaptic_deliveries: ew.synaptic_deliveries,
        cell_updates: ew.cell_updates,
        plasticity_updates,
    };
    // Ensure disclosure is non-empty even if a path produced zero events.
    if work.source_spikes == 0 && work.synaptic_deliveries == 0 {
        work.cell_updates = work.cell_updates.max(1);
    }
    let wall_secs = wall.elapsed().as_secs_f64();
    let peak_rss = peak_rss_bytes().max(rss0);
    let wpa = Metrics::work_per_accuracy(work, WorkCosts::unit(), accuracy.max(1e-6) as f64);

    if let Some(path) = replay_out {
        let groups = vec![
            ReplayGroup {
                name: "input".into(),
                start: 0,
                end: n_in as u32,
            },
            ReplayGroup {
                name: "hidden".into(),
                start: n_in as u32,
                end: (n_in + n_hidden) as u32,
            },
            ReplayGroup {
                name: "readout".into(),
                start: readout_0,
                end: readout_1 + 1,
            },
        ];
        let export = ReplayExport::from_engine(
            "c1",
            config.hash_string(),
            seed,
            ConditionLabel::LocalAssembly.as_str(),
            config.k_wta,
            groups,
            replay_trials,
            &eng,
        );
        match export.write(&path) {
            Ok(()) => eprintln!("replay export written: {}", path.display()),
            Err(e) => eprintln!("replay export failed ({}): {e}", path.display()),
        }
    }

    CondOutcome {
        accuracy,
        activity_sparsity,
        n_params: nnz,
        budget: BudgetDisclosure {
            n_cells,
            n_params: nnz,
            work,
            wall_secs,
            peak_rss_bytes: peak_rss,
            work_per_accuracy: wpa,
        },
        raster_t,
        raster_cell,
        weight_steps,
        weight_trace,
        mac_probe: Some(MacProbeDiagnostics {
            measured_nnz: nnz,
            max_fan_out: config.max_fan_out,
            predicted_nnz: {
                let expected_deg = config.p_sparse * n_hidden as f32;
                let deg = expected_deg.min(config.max_fan_out as f32).round() as usize;
                n_hidden
                    .saturating_mul(deg)
                    .saturating_add(n_hidden)
                    .saturating_add(config.k_wta.saturating_mul(4))
            },
            mean_out_degree: geom.mean_out_degree,
            p95_out_degree: geom.p95_out_degree,
            mean_readout_fan_in: geom.mean_readout_fan_in,
            mean_hidden_fan_in: geom.mean_hidden_fan_in,
            regime: crate::mac_probe_config::WiringRegime::from_expected_degree(
                config.p_sparse * n_hidden as f32,
                config.max_fan_out,
            )
            .as_str(),
            init_w: base_init_w,
            effective_init_w: init_w,
            readout_boost,
            effective_readout_gain,
            empty_winner_rate: if sparsity_trials == 0 {
                0.0
            } else {
                empty_winner_trials as f32 / sparsity_trials as f32
            },
        }),
    }
}

/// Early train trials that also emit spikes / k-WTA / elig for animation.
const TRACE_EARLY_TRAIN: usize = 5;
const TRACE_ELIG_EDGE_CAP: usize = 64;
const TRACE_KWTA_SCORE_CAP: usize = 64;
const TRACE_WEIGHT_HIDDEN_SAMPLE: usize = 128;

struct TraceExport {
    path: std::path::PathBuf,
    recorder: TraceRecorder,
    /// Per-label hit counts indexed by hidden-cell offset.
    assembly_hits: Vec<Vec<u32>>,
}

struct TraceTrialHook<'a> {
    recorder: &'a mut TraceRecorder,
    trial: u32,
    phase: &'static str,
    record_spikes_kwta: bool,
    record_elig: bool,
    step: u32,
    assembly_hits: Option<&'a mut Vec<Vec<u32>>>,
    n_in: usize,
}

struct SparsitySample {
    active: usize,
    population: usize,
}

#[allow(clippy::too_many_arguments)]
fn run_trial(
    eng: &mut Engine,
    learner: &mut ThreeFactor,
    area: &mut Area,
    enc: &LatencyEncoder,
    seq: &[Sample],
    label: u32,
    readout_0: CellId,
    readout_1: CellId,
    t0: Tick,
    train: bool,
    trial_isolation: bool,
    natural_spiking: bool,
    spike_count_wta: bool,
    use_project: bool,
    reinforce_fb: Option<&ReinforceFeedback>,
    mut b_learned_fb: Option<&mut LearnedReinforceFeedback>,
    mut reinforce_rng: Option<&mut Rng>,
    elig_preabsorb: bool,
    structured_target_teach: bool,
    dfa_live_fb: Option<&FixedRandomFeedback>,
    soft_wta_temp: Option<f32>,
    soft_wta_seed: u64,
    mut trace: Option<&mut TraceTrialHook<'_>>,
) -> (bool, SparsitySample, u64) {
    // True temporal input: encode every frame (no peak collapse).
    let frame_stride = enc.max_delay().saturating_add(1);
    let hidden_cells: Vec<CellId> = area.cells.clone().collect();
    let saved_thresholds: Vec<f32> = hidden_cells
        .iter()
        .map(|&cell| eng.cell(cell).theta)
        .collect();

    let (scores, active_cells, selection_until) = if use_project {
        // Assembly-Calculus scientific path: `project` force-fires the input
        // assembly, measures delivered charge, applies k-WTA, and Hebbian-imprints.
        let mut members = Vec::new();
        for sample in seq {
            for ev in enc.encode(sample) {
                let cell = ev.cell.min(1);
                if !members.contains(&cell) {
                    members.push(cell);
                }
            }
        }
        if members.is_empty() {
            members.push(0);
        }
        let src = Assembly::from_members(members);
        C1_PROJECT_INVOKE_COUNT.fetch_add(1, Ordering::Relaxed);
        let winners_asm = project(eng, &src, area);
        let active_cells = winners_asm.members;
        let scores: Vec<(CellId, f32)> = hidden_cells
            .iter()
            .map(|&cell| (cell, eng.last_step_charge(cell)))
            .filter(|(_, v)| v.is_finite() && *v > 0.0)
            .collect();
        let selection_until = eng.time();
        (scores, active_cells, selection_until)
    } else {
        // Canonical / isolation / spike paths mute or keep finite θ then score.
        if !natural_spiking {
            for &cell in &hidden_cells {
                eng.cell_mut(cell).theta = f32::INFINITY;
            }
        }

        let mut latest_input_at = t0;
        for (frame_i, sample) in seq.iter().enumerate() {
            let encoded = enc.encode(sample);
            for ev in &encoded {
                let cell = ev.cell.min(1);
                let at = t0
                    + (frame_i as Tick)
                        .saturating_mul(frame_stride)
                        .saturating_add(ev.t);
                latest_input_at = latest_input_at.max(at);
                eng.force_spike(cell, at);
            }
        }

        let selection_until = latest_input_at
            .checked_add(eng.max_synaptic_delay().max(1))
            .expect("selection window overflow");
        let _ = eng.step_until(selection_until);

        let scores: Vec<(CellId, f32)> = if spike_count_wta {
            // Calibrated natural-spiking: LIF reset zeroes residual membrane of
            // cells that actually spiked, so membrane-score k-WTA collapses.
            // Score integrate-window spike counts (tie-break: residual v).
            let mut counts = vec![0u32; hidden_cells.len()];
            let hidden_lo = *hidden_cells.first().unwrap_or(&0);
            let hidden_hi = hidden_cells.last().copied().unwrap_or(0) + 1;
            for sp in eng.spikes().as_slice() {
                if sp.t > t0
                    && sp.t <= selection_until
                    && sp.cell >= hidden_lo
                    && sp.cell < hidden_hi
                {
                    let offset = (sp.cell - hidden_lo) as usize;
                    if offset < counts.len() {
                        counts[offset] = counts[offset].saturating_add(1);
                    }
                }
            }
            let any_spikes = counts.iter().any(|&c| c > 0);
            hidden_cells
                .iter()
                .enumerate()
                .map(|(i, &cell)| {
                    eng.cell_mut(cell).advance_to(selection_until);
                    let v = eng.cell(cell).v;
                    let score = if any_spikes {
                        counts[i] as f32 + (v.max(0.0) * 1e-3)
                    } else {
                        // Subthreshold fallback: same membrane score as mute path.
                        v
                    };
                    (cell, score)
                })
                .filter(|(_, v)| v.is_finite() && *v > 0.0)
                .collect()
        } else {
            // Membrane-state k-WTA: score Cell::v at decision time (preserves timing).
            hidden_cells
                .iter()
                .map(|&cell| {
                    eng.cell_mut(cell).advance_to(selection_until);
                    (cell, eng.cell(cell).v)
                })
                .filter(|(_, v)| v.is_finite() && *v > 0.0)
                .collect()
        };
        let active_cells = if let Some(temp) = soft_wta_temp {
            soft_k_wta(&scores, area.effective_k(), temp, soft_wta_seed ^ t0)
        } else {
            k_wta(&scores, area.effective_k())
        };
        (scores, active_cells, selection_until)
    };

    let active = active_cells.len();
    let population = area.len();
    if !use_project {
        area.log_activity(active);
    }

    // Capture k-WTA scores before zeroing membrane voltages.
    let mut kwta_scores: Option<Vec<TraceScore>> = None;
    let kwta_t = selection_until;
    if let Some(hook) = trace.as_mut() {
        if hook.record_spikes_kwta {
            let mut scored: Vec<TraceScore> = scores
                .iter()
                .map(|&(cell, v)| TraceScore { cell, v })
                .collect();
            scored.sort_by(|a, b| b.v.partial_cmp(&a.v).unwrap_or(std::cmp::Ordering::Equal));
            scored.truncate(TRACE_KWTA_SCORE_CAP);
            kwta_scores = Some(scored);
        }
        if let Some(hits) = hook.assembly_hits.as_mut() {
            let li = label as usize;
            if li < hits.len() {
                for &cell in &active_cells {
                    let offset = cell as usize - hook.n_in;
                    if offset < hits[li].len() {
                        hits[li][offset] = hits[li][offset].saturating_add(1);
                    }
                }
            }
        }
    }

    for &cell in &hidden_cells {
        eng.cell_mut(cell).v = 0.0;
    }
    let winner_at = selection_until
        .checked_add(1)
        .expect("winner time overflow");
    for &cell in &active_cells {
        eng.force_spike(cell, winner_at);
    }
    let readout_until = winner_at
        .checked_add(eng.max_synaptic_delay().max(1) + 4)
        .expect("readout horizon overflow");
    let produced = eng.step_until(readout_until);

    let fired_0 = produced.as_slice().iter().any(|sp| sp.cell == readout_0);
    let fired_1 = produced.as_slice().iter().any(|sp| sp.cell == readout_1);
    let charge_0 = eng.last_step_charge(readout_0);
    let charge_1 = eng.last_step_charge(readout_1);

    // Protocol 18: lock winner→readout STDP into eligibility *before* the
    // REINFORCE action spike rearranges pairing / dilutes traces.
    if train && elig_preabsorb {
        learner.observe_spikes(eng);
    }

    let pred = match (fired_0, fired_1) {
        (true, false) => 0u32,
        (false, true) => 1u32,
        // Symmetric decision: choose the larger accumulated charge. Break exact
        // ties with an unbiased, seed-derived parity rather than a fixed `>=`
        // (which pins the readout to a constant class-1 predictor and produces
        // the exact-0.5 / zero-variance artifact seen across all seeds).
        _ => {
            let diff = charge_1 - charge_0;
            if diff.abs() > 1e-6 {
                u32::from(diff > 0.0)
            } else {
                (t0 & 1) as u32
            }
        }
    };

    // Default C1: two-sided credit with hard ±1 reward (soft RPE deferred).
    // Opt-in v13 (`reinforce_fb`): match matched-arch REINFORCE — sample action
    // from soft readout policy, apply `reinforce_term` × frozen `B_i`; no
    // secondary broadcast +1 teach (DFA-style observe-only on target spike).
    // Opt-in v20 (`dfa_live_fb`): graded readout error × FixedRandomFeedback.
    let greedy_selected = if pred == 0 { readout_0 } else { readout_1 };
    let target = if label == 0 { readout_0 } else { readout_1 };
    let action_at = readout_until.checked_add(1).expect("action time overflow");
    let delay = eng.max_synaptic_delay().max(1) + 4;

    let mut plasticity_apps = 0u64;
    if train {
        let policy = soft_readout_policy(charge_0, charge_1);
        let (selected, reward, elig_scalar) = if reinforce_fb.is_some() || b_learned_fb.is_some() {
            let rng = reinforce_rng
                .as_mut()
                .expect("reinforce_fb training requires an RNG");
            let action = if rng.next_f32() < policy { 1.0f32 } else { 0.0 };
            let selected = if action > 0.5 { readout_1 } else { readout_0 };
            let reward = if (action > 0.5) == (label == 1) {
                1.0f32
            } else {
                -1.0
            };
            let directional = reinforce_term(reward, action, policy);
            (selected, reward, directional)
        } else if dfa_live_fb.is_some() {
            // Graded supervised error on soft policy (matched-DFA teach = -(p−y)).
            let y = if label == 1 { 1.0f32 } else { 0.0 };
            let teach = -(policy - y);
            let reward = if pred == label { 1.0f32 } else { -1.0 };
            (greedy_selected, reward, teach)
        } else {
            let reward = if pred == label { 1.0f32 } else { -1.0 };
            (greedy_selected, reward, reward)
        };
        let correct = reward > 0.0;

        eng.force_spike(selected, action_at);
        let until_sel = action_at
            .checked_add(delay)
            .expect("selected horizon overflow");
        let _ = eng.step_until(until_sel);
        let w_before = if trace.as_ref().is_some_and(|h| h.record_elig) {
            Some(eng.edge_w.clone())
        } else {
            None
        };
        if let Some(fb) = reinforce_fb {
            plasticity_apps = learner.update_with_credit_counted(eng, &fb.credit(elig_scalar));
        } else if let Some(ref mut fb) = b_learned_fb {
            plasticity_apps = learner.update_with_credit_counted(eng, &fb.credit(elig_scalar));
            let mut post_acts = vec![0.0f32; fb.weights().len()];
            for &c in &active_cells {
                if (c as usize) < post_acts.len() {
                    post_acts[c as usize] = 1.0;
                }
            }
            fb.update(elig_scalar, &post_acts);
        } else if let Some(fb) = dfa_live_fb {
            let y1 = label as f32;
            let p1 = policy;
            let p0 = 1.0 - p1;
            let errors = [1.0 - y1 - p0, y1 - p1];
            let mut signal = fb.project(&errors);
            signal.set(readout_0, errors[0]);
            signal.set(readout_1, errors[1]);
            plasticity_apps = learner.update_with_credit_counted(eng, &signal);
        } else {
            plasticity_apps = learner.update_counted(eng, Modulators::reward(reward));
        }
        if let (Some(hook), Some(before)) = (trace.as_mut(), w_before.as_ref()) {
            let edges = collect_elig_edges(eng, before, TRACE_ELIG_EDGE_CAP);
            hook.recorder
                .emit_elig_event(hook.trial, hook.step, f64::from(elig_scalar), &edges);
        }
        // Consume eligibility so a follow-up target update cannot re-gate residual
        // traces from the selected spike under a different modulator.
        clear_eligibility(eng);

        if !correct && target != selected {
            let target_at = until_sel
                .checked_add(2)
                .expect("target action time overflow");
            eng.force_spike(target, target_at);
            let until_tgt = target_at
                .checked_add(delay)
                .expect("target horizon overflow");
            let _ = eng.step_until(until_tgt);
            if reinforce_fb.is_some() || dfa_live_fb.is_some() {
                if let Some(fb) = reinforce_fb {
                    if structured_target_teach {
                        // Protocol 19: restore secondary +1 teach through structured B.
                        let w_before = if trace.as_ref().is_some_and(|h| h.record_elig) {
                            Some(eng.edge_w.clone())
                        } else {
                            None
                        };
                        plasticity_apps = plasticity_apps.saturating_add(
                            learner.update_with_credit_counted(eng, &fb.credit(1.0)),
                        );
                        if let (Some(hook), Some(before)) = (trace.as_mut(), w_before.as_ref()) {
                            let edges = collect_elig_edges(eng, before, TRACE_ELIG_EDGE_CAP);
                            hook.recorder
                                .emit_elig_event(hook.trial, hook.step, 1.0, &edges);
                        }
                    } else {
                        // Match credit DFA/eprop + matched RL: no broadcast +1 teach.
                        learner.observe_spikes(eng);
                    }
                } else {
                    // Live DFA: observe-only on incorrect (matched DFA honesty).
                    learner.observe_spikes(eng);
                }
            } else {
                let w_before = if trace.as_ref().is_some_and(|h| h.record_elig) {
                    Some(eng.edge_w.clone())
                } else {
                    None
                };
                plasticity_apps = plasticity_apps
                    .saturating_add(learner.update_counted(eng, Modulators::reward(1.0)));
                if let (Some(hook), Some(before)) = (trace.as_mut(), w_before.as_ref()) {
                    let edges = collect_elig_edges(eng, before, TRACE_ELIG_EDGE_CAP);
                    hook.recorder
                        .emit_elig_event(hook.trial, hook.step, 1.0, &edges);
                }
            }
            clear_eligibility(eng);
        }
    } else {
        eng.force_spike(greedy_selected, action_at);
        let until = action_at
            .checked_add(delay)
            .expect("trial horizon overflow");
        let _ = eng.step_until(until);
    }

    if trial_isolation {
        // Protocol v5 / c1-iso: C3-style full dynamic reset + clear STDP pairing.
        reset_c1_dynamic_state(eng, &hidden_cells, &saved_thresholds);
        learner.reset_pairing_state();
    } else {
        // Canonical protocol v2: soma v + theta only (H2 incomplete reset).
        for (&cell, &theta) in hidden_cells.iter().zip(saved_thresholds.iter()) {
            let hidden = eng.cell_mut(cell);
            hidden.theta = theta;
            hidden.v = 0.0;
        }
    }
    eng.close_inhibited_cycle();

    let t1 = eng.time();
    if let Some(hook) = trace.as_mut() {
        if hook.record_spikes_kwta {
            hook.recorder
                .emit_stimulus(hook.trial, label, t0, t1, hook.phase);
            for sp in eng.spikes().as_slice() {
                if sp.t >= t0 && sp.t <= t1 {
                    hook.recorder.emit_spike(sp.t, sp.cell, hook.trial);
                }
            }
            if let Some(ref scores) = kwta_scores {
                hook.recorder
                    .emit_kwta(hook.trial, "hidden", kwta_t, &active_cells, scores);
            }
        }
    }

    let correct = pred == label;
    (
        correct,
        SparsitySample {
            active,
            population: population.max(1),
        },
        plasticity_apps,
    )
}

pub(crate) fn build_sparse_assembly(
    config: &Config,
    seed: u64,
    n_in: usize,
    n_hidden: usize,
    readout_0: CellId,
    readout_1: CellId,
) -> (Csr, f32) {
    let n_cells = n_in + n_hidden + 2;
    let hidden = n_in as CellId..(n_in + n_hidden) as CellId;
    let areas = vec![0..n_in as CellId, hidden.clone(), readout_0..readout_1 + 1];
    let prior = WiringPrior::new(
        seed ^ 0xA55E,
        areas,
        config.p_sparse,
        config.p_sparse * 0.15,
    )
    .with_max_fan_out(config.max_fan_out.max(1));
    let csr0 = wire(AreaRole::Association, Pos::new(1), &prior);

    let mut rows: Vec<Vec<u32>> = (0..n_cells)
        .map(|pre| {
            let start = csr0.row_ptr[pre] as usize;
            let end = csr0.row_ptr[pre + 1] as usize;
            csr0.col[start..end]
                .iter()
                .copied()
                .filter(|&post| {
                    let post = post as usize;
                    if pre < n_in {
                        (n_in..n_in + n_hidden).contains(&post)
                    } else if pre < n_in + n_hidden {
                        (n_in..n_in + n_hidden).contains(&post)
                            || post == readout_0 as usize
                            || post == readout_1 as usize
                    } else {
                        false
                    }
                })
                .collect()
        })
        .collect();
    let mut rng = Rng::new(seed ^ 0x51A5);
    for row in rows.iter_mut().take(n_in) {
        let fan = config.k_wta.max(1) * 2;
        for _ in 0..fan {
            let post = n_in + rng.gen_index(n_hidden);
            if !row.contains(&(post as u32)) {
                row.push(post as u32);
            }
        }
    }
    for h in hidden {
        for &ro in &[readout_0, readout_1] {
            if !rows[h as usize].contains(&ro) && rng.next_f32() < 0.5 {
                rows[h as usize].push(ro);
            }
        }
    }
    for &ro in &[readout_0, readout_1] {
        if !(0..n_cells).any(|pre| rows[pre].contains(&ro)) {
            // Wire from a hidden cell so the decision window can deliver charge
            // (input→readout edges fire too early and leave a dead zero-charge arm).
            let h = n_in + rng.gen_index(n_hidden.max(1));
            rows[h].push(ro);
        }
    }
    for row in &mut rows {
        row.sort_unstable();
        row.dedup();
    }
    (Csr::from_adjacency(&rows), config.init_w)
}

#[derive(Clone, Debug)]
struct AssemblyGeometryStats {
    mean_out_degree: f32,
    p95_out_degree: f32,
    mean_readout_fan_in: f32,
    mean_hidden_fan_in: f32,
}

fn assembly_geometry_stats(
    conn: &Csr,
    n_in: usize,
    n_hidden: usize,
    readout_0: CellId,
    readout_1: CellId,
) -> AssemblyGeometryStats {
    let n_cells = conn.nrows();
    let mut out_degrees = Vec::with_capacity(n_hidden);
    let mut hidden_fan_in = vec![0usize; n_hidden];
    let mut readout_fan_in = [0usize; 2];
    for pre in 0..n_cells {
        let start = conn.row_ptr[pre] as usize;
        let end = conn.row_ptr[pre + 1] as usize;
        let degree = end - start;
        if (n_in..n_in + n_hidden).contains(&pre) {
            out_degrees.push(degree);
        }
        for &post in &conn.col[start..end] {
            let post = post as usize;
            if (n_in..n_in + n_hidden).contains(&post) {
                hidden_fan_in[post - n_in] += 1;
            }
            if post == readout_0 as usize {
                readout_fan_in[0] += 1;
            } else if post == readout_1 as usize {
                readout_fan_in[1] += 1;
            }
        }
    }
    let mean_out = if out_degrees.is_empty() {
        0.0
    } else {
        out_degrees.iter().sum::<usize>() as f32 / out_degrees.len() as f32
    };
    let p95 = if out_degrees.is_empty() {
        0.0
    } else {
        let mut sorted = out_degrees.clone();
        sorted.sort_unstable();
        let idx = ((sorted.len() as f32 - 1.0) * 0.95).round() as usize;
        sorted[idx.min(sorted.len() - 1)] as f32
    };
    let mean_hin = if hidden_fan_in.is_empty() {
        0.0
    } else {
        hidden_fan_in.iter().sum::<usize>() as f32 / hidden_fan_in.len() as f32
    };
    let mean_ro = (readout_fan_in[0] + readout_fan_in[1]) as f32 / 2.0;
    AssemblyGeometryStats {
        mean_out_degree: mean_out,
        p95_out_degree: p95,
        mean_readout_fan_in: mean_ro,
        mean_hidden_fan_in: mean_hin,
    }
}

pub(crate) fn build_dense_local(
    config: &Config,
    seed: u64,
    n_in: usize,
    n_hidden: usize,
    readout_0: CellId,
    readout_1: CellId,
    match_nnz: Option<usize>,
) -> (Csr, f32) {
    let n_cells = n_in + n_hidden + 2;
    let mut rows: Vec<Vec<u32>> = vec![Vec::new(); n_cells];
    for row in rows.iter_mut().take(n_in) {
        for post in n_in..(n_in + n_hidden) {
            row.push(post as u32);
        }
    }
    for (pre, row) in rows.iter_mut().enumerate().skip(n_in).take(n_hidden) {
        for post in n_in..(n_in + n_hidden) {
            if pre != post {
                row.push(post as u32);
            }
        }
        row.push(readout_0);
        row.push(readout_1);
    }

    if let Some(target) = match_nnz {
        let full_nnz: usize = rows.iter().map(Vec::len).sum();
        if target < full_nnz {
            let mut rng = Rng::new(seed ^ 0x7A7C_B001_u64 ^ (n_hidden as u64));
            let mut hidden_hidden = Vec::new();
            for (pre, row) in rows.iter().enumerate().skip(n_in).take(n_hidden) {
                for &post in row {
                    if (n_in..n_in + n_hidden).contains(&(post as usize)) {
                        hidden_hidden.push((pre, post));
                    }
                }
            }

            // Preserve the I/O roles shared with local-assembly whenever the
            // requested budget can hold all input→hidden and hidden→readout
            // edges. Spend only the remaining budget on a seeded subset of
            // hidden→hidden edges.
            let mandatory_nnz = n_in
                .saturating_mul(n_hidden)
                .saturating_add(n_hidden.saturating_mul(2));
            rows = vec![Vec::new(); n_cells];

            if target >= mandatory_nnz {
                for row in rows.iter_mut().take(n_in) {
                    for post in n_in..(n_in + n_hidden) {
                        row.push(post as u32);
                    }
                }
                for row in rows.iter_mut().skip(n_in).take(n_hidden) {
                    row.push(readout_0);
                    row.push(readout_1);
                }
                shuffle_edges(&mut hidden_hidden, &mut rng);
                for (pre, post) in hidden_hidden
                    .into_iter()
                    .take(target.saturating_sub(mandatory_nnz))
                {
                    rows[pre].push(post);
                }
            } else {
                // Defensive small-budget path for isolate CLI use: reserve one
                // hidden predecessor per readout, then sample the remaining
                // common-role edges without exceeding the exact target.
                let mut flat = Vec::with_capacity(full_nnz);
                for pre in 0..n_in {
                    for post in n_in..(n_in + n_hidden) {
                        flat.push((pre, post as u32));
                    }
                }
                flat.extend(hidden_hidden);
                for pre in n_in..(n_in + n_hidden) {
                    flat.push((pre, readout_0));
                    flat.push((pre, readout_1));
                }
                let mut reserved = Vec::new();
                if target >= 1 {
                    reserved.push((n_in, readout_0));
                }
                if target >= 2 {
                    reserved.push((n_in, readout_1));
                }
                flat.retain(|edge| !reserved.contains(edge));
                shuffle_edges(&mut flat, &mut rng);
                let remaining = target.saturating_sub(reserved.len());
                for (pre, post) in reserved.into_iter().chain(flat.into_iter().take(remaining)) {
                    rows[pre].push(post);
                }
            }

            for row in &mut rows {
                row.sort_unstable();
                row.dedup();
            }
            debug_assert_eq!(rows.iter().map(Vec::len).sum::<usize>(), target);
        }
    }

    (Csr::from_adjacency(&rows), config.init_w)
}

fn shuffle_edges(edges: &mut [(usize, u32)], rng: &mut Rng) {
    for i in 0..edges.len() {
        let j = i + rng.gen_index(edges.len() - i);
        edges.swap(i, j);
    }
}

fn mean_readout_weight(eng: &Engine, readout_0: CellId, readout_1: CellId) -> Option<f32> {
    let mut sum = 0.0f32;
    let mut n = 0usize;
    for (pre, post) in eng.conn.edges() {
        if post != readout_0 && post != readout_1 {
            continue;
        }
        if let Some(e) = edge_index(&eng.conn, pre, post) {
            sum += eng.edge_w[e];
            n += 1;
        }
    }
    if n == 0 {
        None
    } else {
        Some(sum / n as f32)
    }
}

fn projection_nnz(conn: &Csr, src_start: u32, src_end: u32, dst_start: u32, dst_end: u32) -> u64 {
    let mut n = 0u64;
    for pre in src_start..src_end {
        for post in conn.neighbors(pre as usize) {
            if post >= dst_start && post < dst_end {
                n += 1;
            }
        }
    }
    n
}

/// Readout fan-in ∪ capped sample of hidden↔hidden edges for weight frames.
fn collect_weight_frame_edges(
    eng: &Engine,
    n_in: usize,
    n_hidden: usize,
    readout_0: CellId,
    readout_1: CellId,
) -> Vec<TraceWeightEdge> {
    let hidden_lo = n_in as u32;
    let hidden_hi = (n_in + n_hidden) as u32;
    let mut edges = Vec::new();
    let mut hidden_sample = 0usize;
    for (i, (pre, post)) in eng.conn.edges().enumerate() {
        let to_readout = post == readout_0 || post == readout_1;
        let hidden_edge =
            pre >= hidden_lo && pre < hidden_hi && post >= hidden_lo && post < hidden_hi;
        if to_readout {
            edges.push(TraceWeightEdge {
                pre,
                post,
                w: eng.edge_w[i],
            });
        } else if hidden_edge && hidden_sample < TRACE_WEIGHT_HIDDEN_SAMPLE {
            edges.push(TraceWeightEdge {
                pre,
                post,
                w: eng.edge_w[i],
            });
            hidden_sample += 1;
        }
    }
    edges
}

fn collect_elig_edges(eng: &Engine, w_before: &[f32], cap: usize) -> Vec<TraceEligEdge> {
    let mut edges = Vec::new();
    let syns = eng.syn.as_slice();
    for (i, (pre, post)) in eng.conn.edges().enumerate() {
        let w = eng.edge_w[i];
        let e = syns[i].eligibility;
        let dw = w - w_before.get(i).copied().unwrap_or(w);
        if e.abs() > 1e-8 || dw.abs() > 1e-8 {
            edges.push(TraceEligEdge {
                pre,
                post,
                w,
                e,
                dw,
            });
        }
    }
    edges.sort_by(|a, b| {
        b.e.abs()
            .partial_cmp(&a.e.abs())
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    edges.truncate(cap);
    edges
}

pub(crate) fn edge_index(conn: &Csr, pre: CellId, post: CellId) -> Option<usize> {
    let row = pre as usize;
    if row >= conn.nrows() {
        return None;
    }
    let start = conn.row_ptr[row] as usize;
    let end = conn.row_ptr[row + 1] as usize;
    conn.col[start..end]
        .iter()
        .position(|&c| c == post)
        .map(|i| start + i)
}

/// Arithmetic mean of `values`; `0.0` for an empty slice.
///
/// The canonical mean for this crate. Sibling runners and the experiment
/// binaries import it from here rather than re-declaring it, so every reported
/// mean is the same single-pass `sum / len` in `f32`.
///
/// # Choosing between this and [`mean_or_nan`]
///
/// The two differ **only** on an empty slice, and that difference is the whole
/// point: this one reports `0.0`, which for an accuracy or a rate is a
/// perfectly plausible number, so a caller that can be handed nothing must not
/// use it. Use it where emptiness is impossible by construction. Where an empty
/// input means "this run measured nothing", use [`mean_or_nan`], which poisons
/// the report instead of quietly filling in a zero.
pub fn mean(values: &[f32]) -> f32 {
    if values.is_empty() {
        0.0
    } else {
        values.iter().sum::<f32>() / values.len() as f32
    }
}

/// Mean and *sample* variance (Bessel-corrected) of `xs`.
///
/// The canonical mean/variance for this crate. Returns `(0.0, 0.0)` for an
/// empty slice and a zero variance for a single sample, so seed sweeps that
/// ran one seed report a spread of zero rather than NaN.
pub fn mean_var(xs: &[f32]) -> (f32, f32) {
    let n = xs.len();
    if n == 0 {
        return (0.0, 0.0);
    }
    let mean = xs.iter().sum::<f32>() / n as f32;
    if n == 1 {
        return (mean, 0.0);
    }
    let var = xs.iter().map(|x| (x - mean).powi(2)).sum::<f32>() / (n as f32 - 1.0);
    (mean, var)
}

/// Arithmetic mean of `values`; **NaN** for an empty slice.
///
/// The fail-loud sibling of [`mean`], for the reports whose authors decided
/// that averaging nothing must be visible in the output rather than rounded to
/// a plausible `0.0`. For every non-empty input the two are the same
/// single-pass `sum / len` and return bit-identical results; they part company
/// only on the empty case.
pub fn mean_or_nan(values: &[f32]) -> f32 {
    if values.is_empty() {
        return f32::NAN;
    }
    values.iter().sum::<f32>() / values.len() as f32
}

/// Standard error of the mean, `sqrt(sample_var / n)`; `0.0` for fewer than
/// two samples.
///
/// The canonical standard error for this crate. The variance is
/// Bessel-corrected by [`mean_var`], so a seed sweep that ran a single seed
/// reports a spread of zero rather than NaN.
///
/// Note this is the standard error, *not* the standard deviation: it carries
/// the extra `/ n`. A report that wants the spread of the seeds themselves
/// rather than the precision of their mean needs `mean_var(..).1.sqrt()`, not
/// this.
pub fn std_error(values: &[f32]) -> f32 {
    if values.len() <= 1 {
        return 0.0;
    }
    let (_, var) = mean_var(values);
    (var / values.len() as f32).sqrt()
}

pub(crate) fn clear_eligibility(eng: &mut Engine) {
    for syn in eng.syn.as_mut_slice() {
        syn.eligibility = 0.0;
    }
}

/// Soft class-1 policy from dual-readout charges (live C1 `ReinforceFeedback`).
#[inline]
fn soft_readout_policy(charge_0: f32, charge_1: f32) -> f32 {
    let diff = (charge_1 - charge_0).clamp(-20.0, 20.0);
    1.0 / (1.0 + (-diff).exp())
}

/// C3-style full dynamic membrane reset for C1 / exact-forward isolation protocols.
///
/// Zeros soma `v` and all `v_dend` for every cell, stamps `last = now`, and
/// restores hidden thresholds from `saved_thresholds`. Input/readout membranes
/// are cleared too so residual charge cannot leak into the next trial's k-WTA.
///
/// Canonical protocol-v2 C1 does **not** call this (H2).
pub(crate) fn reset_c1_dynamic_state(
    eng: &mut Engine,
    hidden_cells: &[CellId],
    saved_thresholds: &[f32],
) {
    let now = eng.time();
    let n = eng.num_cells();
    for i in 0..n {
        let cell = eng.cell_mut(i as CellId);
        cell.v = 0.0;
        cell.v_dend = [0.0; K];
        cell.last = now;
    }
    for (&cell, &theta) in hidden_cells.iter().zip(saved_thresholds.iter()) {
        eng.cell_mut(cell).theta = theta;
    }
}

/// Build protocol-15 structured `B`: hidden posts get
/// `sign(w→readout_1 − w→readout_0)` after readout boost (or continuous
/// L2-normalized Δw when `continuous` is set for protocol 24); other posts
/// keep a seeded Uniform[-1,1] draw so length stays `n_cells`.
#[allow(clippy::too_many_arguments)]
fn structured_reinforce_feedback(
    eng: &Engine,
    n_cells: usize,
    n_in: usize,
    n_hidden: usize,
    readout_0: CellId,
    readout_1: CellId,
    seed: u64,
    continuous: bool,
) -> ReinforceFeedback {
    let mut rng = Rng::new(seed ^ ReinforceFeedback::SEED_MIX ^ 0x57F0_0001);
    let mut weights: Vec<f32> = (0..n_cells).map(|_| rng.next_f32() * 2.0 - 1.0).collect();
    let mut deltas = Vec::with_capacity(n_hidden);
    for h in 0..n_hidden {
        let cell = (n_in + h) as CellId;
        let w0 = edge_index(&eng.conn, cell, readout_0)
            .map(|i| eng.edge_w[i])
            .unwrap_or(0.0);
        let w1 = edge_index(&eng.conn, cell, readout_1)
            .map(|i| eng.edge_w[i])
            .unwrap_or(0.0);
        deltas.push((cell, w1 - w0));
    }
    if continuous {
        let norm = deltas
            .iter()
            .map(|(_, d)| d * d)
            .sum::<f32>()
            .sqrt()
            .max(1e-8);
        for (cell, d) in deltas {
            weights[cell as usize] = d / norm;
        }
    } else {
        for (cell, d) in deltas {
            weights[cell as usize] = if d.abs() < 1e-8 { 0.0 } else { d.signum() };
        }
    }
    ReinforceFeedback::from_weights(weights)
}

/// Scale incoming edges onto the dual readouts so k winners can reach θ=1.
pub(crate) fn boost_readout_incoming(
    eng: &mut Engine,
    readout_0: CellId,
    readout_1: CellId,
    boost: f32,
) {
    if (boost - 1.0).abs() < 1e-6 {
        return;
    }
    let conn = eng.conn.clone();
    for pre in 0..conn.nrows() {
        let start = conn.row_ptr[pre] as usize;
        let end = conn.row_ptr[pre + 1] as usize;
        for (i, &post) in conn.col[start..end].iter().enumerate() {
            if post == readout_0 || post == readout_1 {
                let idx = start + i;
                eng.edge_w[idx] *= boost;
                eng.syn.as_mut_slice()[idx].weight = eng.edge_w[idx];
            }
        }
    }
}

fn summarize_paired(
    seeds: &[SeedResult],
    confidence_z: f32,
    min_reference_gap: f32,
) -> PairedSummary {
    assert!(confidence_z.is_finite() && confidence_z >= 0.0);
    assert!(
        min_reference_gap.is_finite() && min_reference_gap >= 0.0,
        "min_reference_gap must be finite and non-negative"
    );
    let local: Vec<f32> = seeds.iter().map(|s| s.local_assembly).collect();
    let dense: Vec<f32> = seeds.iter().map(|s| s.dense_local).collect();
    let gradient: Vec<f32> = seeds.iter().map(|s| s.gradient_reference).collect();
    let eprop: Vec<f32> = seeds.iter().map(|s| s.eligibility_reference).collect();
    let (mean_local, var_local) = mean_var(&local);
    let (mean_dense, var_dense) = mean_var(&dense);
    let (mean_gradient_reference, var_gradient_reference) = mean_var(&gradient);
    let (mean_eligibility_reference, var_eligibility_reference) = mean_var(&eprop);

    let mut gap_closed = Vec::with_capacity(seeds.len());
    let mut dist_dense = Vec::new();
    for s in seeds {
        let dd = (s.local_assembly - s.dense_local).abs();
        dist_dense.push(dd);
        let reference_gap = s.gradient_reference - s.dense_local;
        // Clamp to [0, 1] and require a preregistered minimum reference gap so a
        // weak denominator cannot manufacture a false PASS (BUILD_AUDIT_v10 B).
        let closed = if reference_gap >= min_reference_gap {
            let raw = (s.local_assembly - s.dense_local) / reference_gap;
            raw.clamp(0.0, 1.0)
        } else {
            0.0
        };
        gap_closed.push(closed);
    }
    let (mean_dist_to_dense, _) = mean_var(&dist_dense);
    let (mean_gap_closed, var_gap_closed) = mean_var(&gap_closed);
    let n = seeds.len();
    let gap_closed_lower_95 = if n > 1 {
        mean_gap_closed - confidence_z * (var_gap_closed / n as f32).sqrt()
    } else {
        mean_gap_closed
    };

    PairedSummary {
        mean_local,
        mean_dense,
        mean_gradient_reference,
        mean_eligibility_reference,
        var_local,
        var_dense,
        var_gradient_reference,
        var_eligibility_reference,
        mean_gap_closed,
        var_gap_closed,
        gap_closed_lower_95,
        mean_dist_to_dense,
        n,
    }
}

/// Descriptive chance-normalized gap (reporting only; does **not** change G2).
fn chance_normalized_gap_stats(seeds: &[SeedResult], confidence_z: f32) -> (f32, f32, f32) {
    let mut gaps = Vec::with_capacity(seeds.len());
    for s in seeds {
        let denom = s.gradient_reference - 0.5;
        let g = if denom > 0.0 {
            ((s.local_assembly - 0.5) / denom).clamp(0.0, 1.0)
        } else {
            0.0
        };
        gaps.push(g);
    }
    let (mean, var) = mean_var(&gaps);
    let n = gaps.len();
    let lcb = if n > 1 {
        mean - confidence_z * (var / n as f32).sqrt()
    } else {
        mean
    };
    (mean, var, lcb)
}

/// Delegates to [`crate::guards::decide_matched_verdict`], which adds the check
/// this function never had: the local arm exceeding the gradient reference that
/// bounds it. `mean_gap_closed` is `(local - dense) / (gradient - dense)`
/// clamped to `[0,1]`, so an inverted comparison reaches
/// `gap_closed_lower_95` already flattened and reads as a decisive PASS.
///
/// `mean_dense` is this suite's floor and plays the role chance plays
/// elsewhere: it is what "the reference did not learn" means here, and it is
/// already the denominator's origin.
fn gate_g2(summary: &PairedSummary, min_gap_closed: f32, min_accuracy: f32) -> GateG2Verdict {
    crate::guards::decide_matched_verdict(
        summary.mean_gradient_reference,
        summary.mean_local,
        summary.gap_closed_lower_95,
        summary.mean_dense,
        min_accuracy,
        min_gap_closed,
        false,
    )
}

fn decide_g2_verdict(
    config: &Config,
    summary: &PairedSummary,
    positive_control_mean: f32,
    mean_activity_sparsity: f32,
    required_n: usize,
) -> GateG2Verdict {
    let sparsity_ok = (config.activity_sparsity_min..=config.activity_sparsity_max)
        .contains(&mean_activity_sparsity);
    let positive_ok = positive_control_mean >= config.g2_min_positive_control;
    if !positive_ok || !sparsity_ok {
        return GateG2Verdict::InvalidHarness;
    }
    if config.quick || config.n_seeds < required_n {
        return GateG2Verdict::Pilot;
    }
    gate_g2(summary, config.g2_min_gap_closed, config.g2_min_accuracy)
}

/// Best-effort peak resident set size in bytes (0 if unavailable).
fn peak_rss_bytes() -> u64 {
    #[cfg(target_os = "linux")]
    {
        if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
            for line in status.lines() {
                if let Some(rest) = line.strip_prefix("VmHWM:") {
                    let kb: u64 = rest
                        .split_whitespace()
                        .next()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(0);
                    return kb.saturating_mul(1024);
                }
            }
        }
        0
    }
    #[cfg(target_os = "macos")]
    {
        // `ps` reports RSS in KiB on Darwin.
        use std::process::Command;
        let pid = std::process::id().to_string();
        if let Ok(out) = Command::new("ps").args(["-o", "rss=", "-p", &pid]).output() {
            if let Ok(s) = String::from_utf8(out.stdout) {
                let kb: u64 = s.trim().parse().unwrap_or(0);
                return kb.saturating_mul(1024);
            }
        }
        0
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logging::{EmitError, RunLog};

    /// The two canonical means must stay distinguishable on the empty slice.
    ///
    /// This is the whole reason there are two. Fifteen experiment binaries had
    /// each re-declared `mean`, and the copies disagreed here and only here:
    /// nine returned `0.0`, five returned NaN. Merging the classes would turn
    /// "this run averaged nothing" into a printed accuracy of `0.0000`, which
    /// no reader can tell from a real measurement. Pin both halves so a later
    /// consolidation cannot quietly collapse them.
    #[test]
    fn mean_and_mean_or_nan_differ_only_on_the_empty_slice() {
        assert_eq!(mean(&[]), 0.0, "the quiet mean fills an empty run with 0.0");
        assert!(
            mean_or_nan(&[]).is_nan(),
            "the loud mean must poison an empty run, not score it"
        );

        // Everywhere else they are the same single-pass sum/len, bit for bit.
        for values in [
            &[0.5f32][..],
            &[0.25, 0.75],
            &[1.0, 2.0, 4.0],
            &[-3.5, 0.0, 3.5, 7.25],
        ] {
            assert_eq!(
                mean(values).to_bits(),
                mean_or_nan(values).to_bits(),
                "the two means diverged on non-empty input {values:?}"
            );
        }
    }

    /// `std_error` reproduces the hand-written copies it replaced, including
    /// their degenerate case, and stays distinct from the standard deviation.
    #[test]
    fn std_error_matches_the_copies_it_replaced() {
        // The body every converged copy carried, written out longhand.
        fn reference(values: &[f32]) -> f32 {
            if values.len() <= 1 {
                return 0.0;
            }
            let m = values.iter().sum::<f32>() / values.len() as f32;
            let var =
                values.iter().map(|v| (v - m).powi(2)).sum::<f32>() / (values.len() - 1) as f32;
            (var / values.len() as f32).sqrt()
        }
        for values in [
            &[][..],
            &[0.9],
            &[0.80, 0.84],
            &[0.71, 0.68, 0.74, 0.70, 0.69],
        ] {
            assert_eq!(
                std_error(values).to_bits(),
                reference(values).to_bits(),
                "std_error drifted from the replaced body on {values:?}"
            );
        }

        // Fewer than two samples is a zero spread, never NaN: a one-seed sweep
        // reports "no spread measured", and the report still prints a number.
        assert_eq!(std_error(&[]), 0.0);
        assert_eq!(std_error(&[0.42]), 0.0);

        // And it is the standard *error*, carrying the extra 1/sqrt(n) that the
        // standard deviation does not. `shd_frozen_attention::sd` keeps its own
        // body because it deliberately reports the other quantity.
        let values = [0.4f32, 0.6, 0.8, 1.0];
        let (_, var) = mean_var(&values);
        assert!((std_error(&values) - var.sqrt() / 2.0).abs() < 1e-6);
        assert!(std_error(&values) < var.sqrt());
    }

    #[test]
    fn c1_quick_runs_and_emits_gc7_fields() {
        let mut runner = Runner::new();
        let cfg = Config::c1_quick();
        let report = runner.run_c1(&cfg);
        assert_eq!(report.seeds.len(), 5);
        assert!(!report.emitted.is_empty());
        for e in &report.emitted {
            assert!(
                e.line.contains("activity_sparsity=") && e.line.contains("activity-sparsity="),
                "GC7 fields missing: {}",
                e.line
            );
            assert!(
                e.line.contains("work_per_accuracy=") || e.line.contains("wall_secs="),
                "work metrics missing: {}",
                e.line
            );
            assert!(
                !e.line
                    .contains("work_metric_omitted_until_measured_counters"),
                "stale omission note still present: {}",
                e.line
            );
        }
        assert!(matches!(
            report.verdict,
            GateG2Verdict::Pilot | GateG2Verdict::InvalidHarness
        ));
        for s in &report.seeds {
            assert!((0.0..=1.0).contains(&s.local_assembly));
            assert!((0.0..=1.0).contains(&s.dense_local));
            assert!((0.0..=1.0).contains(&s.gradient_reference));
            assert!((0.0..=1.0).contains(&s.eligibility_reference));
        }
    }

    #[test]
    fn gate_g2_normalized_gap_and_accuracy_pass() {
        let summary = PairedSummary {
            mean_local: 0.8,
            mean_dense: 0.5,
            mean_gradient_reference: 0.9,
            mean_eligibility_reference: 0.7,
            var_local: 0.0,
            var_dense: 0.0,
            var_gradient_reference: 0.0,
            var_eligibility_reference: 0.0,
            mean_gap_closed: 0.75,
            var_gap_closed: 0.01,
            gap_closed_lower_95: 0.66,
            mean_dist_to_dense: 0.3,
            n: 20,
        };
        assert_eq!(gate_g2(&summary, 0.5, 0.65), GateG2Verdict::Pass);
    }

    #[test]
    fn gate_g2_plateau_with_dense_fails() {
        let summary = PairedSummary {
            mean_local: 0.52,
            mean_dense: 0.50,
            mean_gradient_reference: 0.90,
            mean_eligibility_reference: 0.7,
            var_local: 0.0,
            var_dense: 0.0,
            var_gradient_reference: 0.0,
            var_eligibility_reference: 0.0,
            mean_gap_closed: 0.05,
            var_gap_closed: 0.01,
            gap_closed_lower_95: 0.01,
            mean_dist_to_dense: 0.02,
            n: 20,
        };
        assert_eq!(gate_g2(&summary, 0.5, 0.65), GateG2Verdict::Fail);
    }

    #[test]
    fn gate_g2_accuracy_floor_blocks_low_accuracy_ratio() {
        let summary = PairedSummary {
            mean_local: 0.60,
            mean_dense: 0.20,
            mean_gradient_reference: 0.70,
            mean_eligibility_reference: 0.65,
            var_local: 0.0,
            var_dense: 0.0,
            var_gradient_reference: 0.0,
            var_eligibility_reference: 0.0,
            mean_gap_closed: 0.80,
            var_gap_closed: 0.0,
            gap_closed_lower_95: 0.80,
            mean_dist_to_dense: 0.40,
            n: 20,
        };
        assert_eq!(gate_g2(&summary, 0.5, 0.65), GateG2Verdict::Fail);
    }

    #[test]
    fn four_state_invalid_harness_on_weak_positive_control() {
        let cfg = Config::c1_default();
        let summary = PairedSummary {
            mean_local: 0.8,
            mean_dense: 0.5,
            mean_gradient_reference: 0.9,
            mean_eligibility_reference: 0.7,
            var_local: 0.0,
            var_dense: 0.0,
            var_gradient_reference: 0.0,
            var_eligibility_reference: 0.0,
            mean_gap_closed: 0.75,
            var_gap_closed: 0.0,
            gap_closed_lower_95: 0.75,
            mean_dist_to_dense: 0.3,
            n: 20,
        };
        let v = decide_g2_verdict(&cfg, &summary, 0.5, 0.015, 20);
        assert_eq!(v, GateG2Verdict::InvalidHarness);
    }

    #[test]
    fn four_state_invalid_harness_on_sparsity_band() {
        let cfg = Config::c1_default();
        let summary = PairedSummary {
            mean_local: 0.8,
            mean_dense: 0.5,
            mean_gradient_reference: 0.9,
            mean_eligibility_reference: 0.7,
            var_local: 0.0,
            var_dense: 0.0,
            var_gradient_reference: 0.0,
            var_eligibility_reference: 0.0,
            mean_gap_closed: 0.75,
            var_gap_closed: 0.0,
            gap_closed_lower_95: 0.75,
            mean_dist_to_dense: 0.3,
            n: 20,
        };
        let v = decide_g2_verdict(&cfg, &summary, 0.95, 0.20, 20);
        assert_eq!(v, GateG2Verdict::InvalidHarness);
    }

    #[test]
    fn four_state_pilot_when_quick() {
        let cfg = Config::c1_quick();
        let summary = PairedSummary {
            mean_local: 0.8,
            mean_dense: 0.5,
            mean_gradient_reference: 0.9,
            mean_eligibility_reference: 0.7,
            var_local: 0.0,
            var_dense: 0.0,
            var_gradient_reference: 0.0,
            var_eligibility_reference: 0.0,
            mean_gap_closed: 0.75,
            var_gap_closed: 0.0,
            gap_closed_lower_95: 0.75,
            mean_dist_to_dense: 0.3,
            n: 5,
        };
        let v = decide_g2_verdict(
            &cfg,
            &summary,
            0.95,
            0.015,
            cfg.required_scientific_n_seeds(),
        );
        assert_eq!(v, GateG2Verdict::Pilot);
    }

    #[test]
    fn paired_summary_uses_gap_closed_and_invalidates_nonpositive_denominator() {
        let seeds = vec![
            SeedResult {
                seed: 1,
                local_assembly: 0.75,
                dense_local: 0.50,
                gradient_reference: 1.00,
                eligibility_reference: 0.70,
                activity_sparsity: 0.015,
                dense_activity_sparsity: 0.015,
                dense_matched: None,
            },
            SeedResult {
                seed: 2,
                local_assembly: 0.80,
                dense_local: 0.90,
                gradient_reference: 0.90,
                eligibility_reference: 0.70,
                activity_sparsity: 0.015,
                dense_activity_sparsity: 0.015,
                dense_matched: None,
            },
        ];
        let summary = summarize_paired(&seeds, 0.0, 0.15);
        // seed1: (0.75-0.50)/(1.00-0.50) = 0.5; seed2: ref gap 0 → closed 0 → mean 0.25
        assert!((summary.mean_gap_closed - 0.25).abs() < 1e-6);
        assert_eq!(summary.gap_closed_lower_95, summary.mean_gap_closed);
    }

    #[test]
    fn weak_reference_cannot_inflate_gap_closed_above_one() {
        let seed = SeedResult {
            seed: 1,
            local_assembly: 0.65,
            dense_local: 0.60,
            gradient_reference: 0.61, // gap = 0.01 ≪ min_reference_gap
            eligibility_reference: 0.50,
            activity_sparsity: 0.015,
            dense_activity_sparsity: 0.015,
            dense_matched: None,
        };
        let seeds = vec![seed; 20];
        let summary = summarize_paired(&seeds, 1.96, 0.15);
        assert!(
            summary.mean_gap_closed <= 1.0 + 1e-6,
            "gap_closed must be clamped to [0, 1]"
        );
        assert_eq!(
            summary.mean_gap_closed, 0.0,
            "weak reference gap must contribute closed = 0"
        );
        // Was `Fail`. This fixture's reference (0.61) sits below its treatment
        // (0.65) and barely above the dense floor (0.60), so the comparison is
        // both dead and inverted — and `FAIL` is a scientific claim about the
        // arm, which this data cannot support. `guards::decide_matched_verdict`
        // now says so. The `closed = 0` mechanism above is unchanged and still
        // asserted; what moved is only what a run on such a reference is
        // allowed to conclude.
        assert_eq!(
            gate_g2(&summary, 0.5, 0.65),
            GateG2Verdict::InvalidHarness,
            "a reference that cannot bound its treatment must not yield a FAIL"
        );
    }

    #[test]
    fn a_healthy_reference_with_no_gap_closed_still_fails() {
        // The companion to the test above, and the reason it is not weakened by
        // the change: when the reference IS usable, a treatment that closes
        // nothing must still read `FAIL` and not be excused as a bad harness.
        let seed = SeedResult {
            seed: 1,
            local_assembly: 0.55,
            dense_local: 0.50,
            gradient_reference: 0.90, // healthy: well above dense, above the arm
            eligibility_reference: 0.50,
            activity_sparsity: 0.015,
            dense_activity_sparsity: 0.015,
            dense_matched: None,
        };
        let seeds = vec![seed; 20];
        let summary = summarize_paired(&seeds, 1.96, 0.15);
        assert_eq!(gate_g2(&summary, 0.5, 0.65), GateG2Verdict::Fail);
    }

    #[test]
    fn gap_closed_clamps_above_one() {
        let seeds = vec![SeedResult {
            seed: 1,
            local_assembly: 0.90,
            dense_local: 0.50,
            gradient_reference: 0.70, // gap = 0.20 ≥ 0.15; raw closed = 2.0
            eligibility_reference: 0.70,
            activity_sparsity: 0.015,
            dense_activity_sparsity: 0.015,
            dense_matched: None,
        }];
        let summary = summarize_paired(&seeds, 0.0, 0.15);
        assert!((summary.mean_gap_closed - 1.0).abs() < 1e-6);
    }

    #[test]
    fn dual_readout_builders_disclose_two_readouts() {
        let cfg = Config::c1_quick();
        let n_in = 2;
        let n_h = cfg.n_hidden;
        let r0 = (n_in + n_h) as CellId;
        let r1 = r0 + 1;
        let (sparse, _) = build_sparse_assembly(&cfg, 1, n_in, n_h, r0, r1);
        let (dense, _) = build_dense_local(&cfg, 1, n_in, n_h, r0, r1, None);
        assert_eq!(sparse.nrows(), n_in + n_h + 2);
        assert_eq!(dense.nrows(), n_in + n_h + 2);
        assert!(sparse.nnz() > 0);
        assert!(dense.nnz() >= sparse.nnz());
    }

    #[test]
    fn mac_probe_max_fan_out_binds_nnz_at_n2k() {
        let mut lo = Config::c1_quick();
        lo.n_hidden = 2000;
        lo.k_wta = 8;
        lo.max_fan_out = 10;
        lo.init_w_rescale = true;
        lo.readout_gain_normalize = true;
        lo.matched_budget_repeat = false;
        lo.experiment = "c1-mac-probe-n2000-f10".into();
        let mut hi = lo.clone();
        hi.max_fan_out = 64;
        hi.experiment = "c1-mac-probe-n2000-f64".into();
        let n_in = 2;
        let r0 = (n_in + lo.n_hidden) as CellId;
        let r1 = r0 + 1;
        let (a, _) = build_sparse_assembly(&lo, 7, n_in, lo.n_hidden, r0, r1);
        let (b, _) = build_sparse_assembly(&hi, 7, n_in, hi.n_hidden, r0, r1);
        assert!(
            b.nnz() > a.nnz(),
            "fan↑ must raise measured nnz: fan10={} fan64={}",
            a.nnz(),
            b.nnz()
        );
        let geom = assembly_geometry_stats(&a, n_in, lo.n_hidden, r0, r1);
        assert!(
            geom.mean_out_degree <= lo.max_fan_out as f32 + 2.0,
            "mean out-degree {} should respect fan cap {}",
            geom.mean_out_degree,
            lo.max_fan_out
        );
    }

    #[test]
    fn mac_probe_condition_json_emits_geometry_fields() {
        let mp = crate::MacProbeConfig::syn_matched(512, true);
        let cfg = mp.to_config();
        let line =
            Runner::condition_json(&cfg, cfg.seeds()[0], ConditionLabel::LocalAssembly, None);
        for key in [
            "measured_nnz",
            "max_fan_out",
            "predicted_nnz",
            "mean_out_degree",
            "p95_out_degree",
            "mean_readout_fan_in",
            "regime",
            "effective_init_w",
            "effective_readout_gain",
            "empty_winner_rate",
            "wall_secs",
            "peak_rss_bytes",
        ] {
            assert!(line.contains(key), "missing {key} in {line}");
        }
        assert!(
            line.contains("\"regime\":\"Bernoulli\"") || line.contains("\"regime\":\"capped\"")
        );
    }

    #[test]
    fn dense_matched_preserves_roles_exact_budget_and_seed_variation() {
        let cfg = Config::c1_quick();
        let n_in = 2usize;
        let n_hidden = cfg.n_hidden;
        let readout_0 = (n_in + n_hidden) as CellId;
        let readout_1 = readout_0 + 1;
        let mandatory_nnz = n_in * n_hidden + 2 * n_hidden;
        let target = mandatory_nnz + n_hidden;
        let (a, _) =
            build_dense_local(&cfg, 11, n_in, n_hidden, readout_0, readout_1, Some(target));
        let (a_replay, _) =
            build_dense_local(&cfg, 11, n_in, n_hidden, readout_0, readout_1, Some(target));
        let (b, _) =
            build_dense_local(&cfg, 12, n_in, n_hidden, readout_0, readout_1, Some(target));

        assert_eq!(a.nnz(), target);
        assert_eq!(a.row_ptr, a_replay.row_ptr);
        assert_eq!(a.col, a_replay.col);
        assert_ne!(a.col, b.col, "matched hidden topology must vary by seed");
        for pre in 0..n_in {
            for post in n_in..(n_in + n_hidden) {
                assert!(
                    edge_index(&a, pre as CellId, post as CellId).is_some(),
                    "input→hidden role was removed"
                );
            }
        }
        for pre in n_in..(n_in + n_hidden) {
            assert!(edge_index(&a, pre as CellId, readout_0).is_some());
            assert!(edge_index(&a, pre as CellId, readout_1).is_some());
        }
        assert!(
            a.row_cols(readout_0 as usize).is_empty() && a.row_cols(readout_1 as usize).is_empty(),
            "readouts must not gain outgoing edges absent from local-assembly"
        );
    }

    #[test]
    fn readout_boost_makes_single_connected_winner_spike() {
        let cfg = Config::c1_quick();
        let n_in = 2usize;
        let n_hidden = cfg.n_hidden;
        let readout_0 = (n_in + n_hidden) as CellId;
        let readout_1 = readout_0 + 1;
        let n_cells = n_in + n_hidden + 2;
        let (conn, init_w) = build_sparse_assembly(&cfg, 19, n_in, n_hidden, readout_0, readout_1);
        let pre = conn
            .edges()
            .find_map(|(pre, post)| (post == readout_0).then_some(pre))
            .expect("sparse builder keeps readout reachable");
        let nnz = conn.nnz();
        let mut eng = Engine::with_cells(n_cells);
        eng.set_connectivity(conn, vec![init_w; nnz]);
        let boost = (1.15 / init_w.max(1e-3)).clamp(1.0, 12.0);
        boost_readout_incoming(&mut eng, readout_0, readout_1, boost);
        eng.force_spike(pre, 1);
        let produced = eng.step_until(eng.max_synaptic_delay().max(1) + 6);
        assert!(
            produced.as_slice().iter().any(|sp| sp.cell == readout_0),
            "one connected hidden winner must make the readout spike"
        );
    }

    #[test]
    fn temporal_encode_emits_more_than_two_events() {
        let enc = LatencyEncoder::new(2, 8, 0);
        let seq: Vec<Sample> = (0..8)
            .map(|t| {
                // Non-silent background so every frame still emits (encode probe).
                let mut v = vec![0.1, 0.1];
                if t == 2 {
                    v[0] = 0.95;
                }
                if t == 3 {
                    v[1] = 0.95;
                }
                Sample::from_values(v)
            })
            .collect();
        let mut n = 0usize;
        for s in &seq {
            n += enc.encode(s).len();
        }
        assert_eq!(n, 16, "full stream encodes every frame × 2 channels");
    }

    #[test]
    fn silence_encoder_skips_zero_features() {
        let enc = LatencyEncoder::new(2, 8, 0);
        let silent = Sample::from_values(vec![0.0, 0.0]);
        assert!(enc.encode(&silent).is_empty());
        let peak = Sample::from_values(vec![0.95, 0.0]);
        let spikes = enc.encode(&peak);
        assert_eq!(spikes.len(), 1);
        assert_eq!(spikes[0].cell, 0);
    }

    /// Build equal-count coincident vs non-coincident sequences and compare
    /// membrane-state k-WTA winner sets (timing must survive scoring).
    #[test]
    fn equal_count_coincident_differs_from_noncoincident_winners() {
        let cfg = Config::c1_quick();
        let n_in = 2usize;
        let n_hidden = cfg.n_hidden;
        let readout_0 = (n_in + n_hidden) as CellId;
        let readout_1 = readout_0 + 1;
        let (conn, init_w) = build_sparse_assembly(&cfg, 42, n_in, n_hidden, readout_0, readout_1);
        let nnz = conn.nnz();

        let coincident = equal_count_pair_seq(true, cfg.sequence_len, cfg.max_lag);
        let noncoin = equal_count_pair_seq(false, cfg.sequence_len, cfg.max_lag);
        assert_eq!(
            spike_count(&coincident),
            spike_count(&noncoin),
            "equal spike counts required"
        );

        let (w_coin, s_coin) = membrane_scores(&cfg, &conn, init_w, nnz, &coincident);
        let (w_non, s_non) = membrane_scores(&cfg, &conn, init_w, nnz, &noncoin);
        let scores_differ = s_coin.len() != s_non.len()
            || s_coin
                .iter()
                .zip(s_non.iter())
                .any(|(a, b)| a.0 != b.0 || (a.1 - b.1).abs() > 1e-5);
        assert!(
            w_coin != w_non || scores_differ,
            "equal-count coincident vs non-coincident must differ in winners or score vectors; coin_w={w_coin:?} non_w={w_non:?} coin_s={s_coin:?} non_s={s_non:?}"
        );
    }

    #[test]
    fn easy_feature0_vs_feature1_different_winners() {
        let cfg = Config::c1_quick();
        let n_in = 2usize;
        let n_hidden = cfg.n_hidden;
        let readout_0 = (n_in + n_hidden) as CellId;
        let readout_1 = readout_0 + 1;
        let (conn, init_w) = build_sparse_assembly(&cfg, 7, n_in, n_hidden, readout_0, readout_1);
        let nnz = conn.nnz();
        let len = cfg.sequence_len.max(2);
        let mut f0 = vec![Sample::from_values(vec![0.0, 0.0]); len];
        let mut f1 = vec![Sample::from_values(vec![0.0, 0.0]); len];
        f0[len / 2].values[0] = 0.95;
        f1[len / 2].values[1] = 0.95;
        let w0 = membrane_scores(&cfg, &conn, init_w, nnz, &f0).0;
        let w1 = membrane_scores(&cfg, &conn, init_w, nnz, &f1).0;
        assert_ne!(
            w0, w1,
            "feature-0 vs feature-1 must prefer different winners"
        );
    }

    #[test]
    fn readout_weights_diverge_by_class_under_pm1_reward() {
        let mut cfg = Config::c1_quick();
        cfg.n_train = 12;
        cfg.n_test = 4;
        let mut easy_rng = Rng::new(0x0E51_EA51);
        let easy_len = cfg.sequence_len.max(2);
        let train: Vec<_> = (0..cfg.n_train)
            .map(|_| easy_trial(&mut easy_rng, easy_len))
            .collect();
        let test: Vec<_> = (0..cfg.n_test)
            .map(|_| easy_trial(&mut easy_rng, easy_len))
            .collect();
        let split = FrozenSplit { train, test };
        let n_in = 2usize;
        let n_hidden = cfg.n_hidden;
        let readout_0 = (n_in + n_hidden) as CellId;
        let readout_1 = readout_0 + 1;
        let mut eng = Engine::with_cells(n_in + n_hidden + 2);
        let (conn, init_w) = build_sparse_assembly(&cfg, 99, n_in, n_hidden, readout_0, readout_1);
        let nnz = conn.nnz();
        eng.set_connectivity(conn, vec![init_w; nnz]);
        let boost = (1.15 / init_w.max(1e-3)).clamp(1.0, 12.0);
        boost_readout_incoming(&mut eng, readout_0, readout_1, boost);
        let w0_before = mean_edge_weight_to(&eng, readout_0);
        let w1_before = mean_edge_weight_to(&eng, readout_1);
        let mut area = Area::new(n_in as CellId..(n_in + n_hidden) as CellId, cfg.k_wta);
        let mut learner = ThreeFactor::new(cfg.eta, cfg.lambda, cfg.tau_e);
        let enc = LatencyEncoder::new(2, (cfg.sequence_len as Tick).max(1), 0);
        let mut t_cursor: Tick = 0;
        for (seq, label) in &split.train {
            let _ = run_trial(
                &mut eng,
                &mut learner,
                &mut area,
                &enc,
                seq,
                *label,
                readout_0,
                readout_1,
                t_cursor,
                true,
                false,
                false,
                false,
                false,
                None,
                None,
                None,
                false,
                false,
                None,
                None,
                0,
                None,
            );
            t_cursor = eng.time() + 20;
        }
        let w0_after = mean_edge_weight_to(&eng, readout_0);
        let w1_after = mean_edge_weight_to(&eng, readout_1);
        let d0 = (w0_after - w0_before).abs();
        let d1 = (w1_after - w1_before).abs();
        assert!(
            d0 > 1e-6 || d1 > 1e-6,
            "at least one readout's incoming weights must move under ±1 reward"
        );
        assert!(
            (w0_after - w1_after).abs() > 1e-6 || (d0 - d1).abs() > 1e-6,
            "class-specific readout weights should diverge (w0={w0_after} w1={w1_after})"
        );
    }

    #[test]
    fn positive_control_floor_on_quick_seed() {
        let cfg = Config::c1_quick();
        let accs: Vec<f32> = cfg
            .seeds()
            .into_iter()
            .map(|seed| run_positive_control(&cfg, seed))
            .collect();
        let (mean, _) = mean_var(&accs);
        assert!(
            mean >= 0.90,
            "positive control mean must clear 0.90 floor on quick seeds; mean={mean} accs={accs:?}"
        );
    }

    #[test]
    fn temporal_positive_control_floor_on_sensitivity_quick() {
        let cfg = Config::c1_temporal_pc_sensitivity_quick();
        assert!(cfg.uses_temporal_positive_control());
        let accs: Vec<f32> = cfg
            .seeds()
            .into_iter()
            .map(|seed| run_positive_control(&cfg, seed))
            .collect();
        let (mean, _) = mean_var(&accs);
        assert!(
            mean >= 0.90,
            "temporal coincidence-lag PC must clear 0.90 on sensitivity-quick; mean={mean} accs={accs:?}"
        );
    }

    #[test]
    fn calibrated_spike_s_positive_control_floor_on_quick() {
        let cfg = Config::c1_spike_s_quick();
        assert!(cfg.is_spike_s_protocol());
        assert!(cfg.uses_calibrated_spike_positive_control());
        let accs: Vec<f32> = cfg
            .seeds()
            .into_iter()
            .map(|seed| run_positive_control(&cfg, seed))
            .collect();
        let (mean, _) = mean_var(&accs);
        assert!(
            mean >= 0.90,
            "calibrated spike-s PC must clear 0.90 on quick seeds; mean={mean} accs={accs:?}"
        );
    }

    #[test]
    fn capacity_sensitivity_quick_keeps_sparsity_band() {
        let cfg = Config::c1_capacity_sensitivity_quick();
        let frac = cfg.nominal_activity_fraction();
        assert!(
            (cfg.activity_sparsity_min..=cfg.activity_sparsity_max).contains(&frac),
            "capacity quick nominal k/N={frac} outside [{}, {}]",
            cfg.activity_sparsity_min,
            cfg.activity_sparsity_max
        );
        assert_ne!(cfg.hash_string(), Config::c1_default().hash_string());
        assert_eq!(cfg.protocol_version(), C1_SENSITIVITY_PROTOCOL_VERSION);
    }

    #[test]
    fn shuffled_labels_stay_near_chance() {
        let mut cfg = Config::c1_quick();
        cfg.n_train = 24;
        cfg.n_test = 16;
        let seed = cfg.seeds()[0];
        let split = freeze_trials_shuffled(&cfg, seed);
        let outcome = run_local_assembly(&cfg, seed, &split);
        assert!(
            (outcome.accuracy - 0.5).abs() <= 0.30,
            "shuffled-label local accuracy should stay near chance; got {}",
            outcome.accuracy
        );
    }

    #[test]
    fn plasticity_counts_synapse_applications() {
        let cfg = Config::c1_quick();
        let seed = cfg.seeds()[0];
        let split = freeze_trials(&cfg, seed);
        let outcome = run_local_assembly(&cfg, seed, &split);
        let nnz = outcome.n_params as u64;
        let min_expected = (cfg.n_train as u64).saturating_mul(nnz);
        let max_expected = min_expected.saturating_mul(2); // +target teach when wrong
        let got = outcome.budget.work.plasticity_updates;
        assert!(
            got >= min_expected && got <= max_expected && got.is_multiple_of(nnz),
            "plasticity_updates should be n_train×nnz .. 2×n_train×nnz; got {got} nnz={nnz}"
        );
    }

    fn spike_count(seq: &[Sample]) -> usize {
        let enc = LatencyEncoder::new(2, 8, 0);
        seq.iter().map(|s| enc.encode(s).len()).sum()
    }

    fn equal_count_pair_seq(coincident: bool, len: usize, max_lag: usize) -> Vec<Sample> {
        let len = len.max(max_lag + 2);
        let mut seq: Vec<Sample> = (0..len)
            .map(|_| Sample::from_values(vec![0.0, 0.0]))
            .collect();
        let t0 = 1usize;
        let t1 = if coincident {
            t0 + max_lag.min(1)
        } else {
            (t0 + max_lag + 1).min(len - 1)
        };
        seq[t0].values[0] = 0.95;
        seq[t1].values[1] = 0.95;
        seq
    }

    fn membrane_scores(
        config: &Config,
        conn: &Csr,
        init_w: f32,
        nnz: usize,
        seq: &[Sample],
    ) -> (Vec<CellId>, Vec<(CellId, f32)>) {
        let n_in = 2usize;
        let n_hidden = config.n_hidden;
        let mut eng = Engine::with_cells(n_in + n_hidden + 2);
        eng.set_connectivity(conn.clone(), vec![init_w; nnz]);
        let area = Area::new(n_in as CellId..(n_in + n_hidden) as CellId, config.k_wta);
        let enc = LatencyEncoder::new(2, (config.sequence_len as Tick).max(1), 0);
        let frame_stride = enc.max_delay().saturating_add(1);
        let hidden_cells: Vec<CellId> = area.cells.clone().collect();
        for &cell in &hidden_cells {
            eng.cell_mut(cell).theta = f32::INFINITY;
        }
        let t0 = 0u64;
        let mut latest = t0;
        for (frame_i, sample) in seq.iter().enumerate() {
            for ev in enc.encode(sample) {
                let cell = ev.cell.min(1);
                let at = t0
                    + (frame_i as Tick)
                        .saturating_mul(frame_stride)
                        .saturating_add(ev.t);
                latest = latest.max(at);
                eng.force_spike(cell, at);
            }
        }
        let selection_until = latest
            .checked_add(eng.max_synaptic_delay().max(1))
            .expect("overflow");
        let _ = eng.step_until(selection_until);
        let scores: Vec<(CellId, f32)> = hidden_cells
            .iter()
            .map(|&cell| {
                eng.cell_mut(cell).advance_to(selection_until);
                (cell, eng.cell(cell).v)
            })
            .collect();
        let positive: Vec<(CellId, f32)> = scores
            .iter()
            .copied()
            .filter(|(_, v)| v.is_finite() && *v > 0.0)
            .collect();
        let winners = k_wta(&positive, area.effective_k());
        (winners, scores)
    }

    fn mean_edge_weight_to(eng: &Engine, post: CellId) -> f32 {
        let mut sum = 0.0f32;
        let mut n = 0usize;
        for (pre, p) in eng.conn.edges() {
            if p != post {
                continue;
            }
            if let Some(e) = edge_index(&eng.conn, pre, p) {
                sum += eng.edge_w[e];
                n += 1;
            }
        }
        if n == 0 {
            0.0
        } else {
            sum / n as f32
        }
    }

    #[test]
    fn harness_refuses_results_without_sparsity() {
        let mut logger = StructuredLogger::new();
        let entry = RunLog::new("c1-test", 0, "local-assembly");
        assert_eq!(
            logger.emit_results(&entry).unwrap_err(),
            EmitError::MissingActivitySparsity
        );
    }

    #[test]
    fn render_mentions_four_state_verdicts() {
        let cfg = Config::c1_quick();
        let report = C1Report {
            config_hash: cfg.hash_string(),
            seeds: Vec::new(),
            summary: PairedSummary {
                mean_local: 0.0,
                mean_dense: 0.0,
                mean_gradient_reference: 0.0,
                mean_eligibility_reference: 0.0,
                var_local: 0.0,
                var_dense: 0.0,
                var_gradient_reference: 0.0,
                var_eligibility_reference: 0.0,
                mean_gap_closed: 0.0,
                var_gap_closed: 0.0,
                gap_closed_lower_95: 0.0,
                mean_dist_to_dense: 0.0,
                n: 0,
            },
            verdict: GateG2Verdict::InvalidHarness,
            positive_control_mean: 0.4,
            mean_activity_sparsity: 0.2,
            required_scientific_n_seeds: 20,
            budgets: Vec::new(),
            emitted: Vec::new(),
            plot_notes: Vec::new(),
        };
        let md = Runner::render_results_markdown(&report, &cfg);
        assert!(md.contains("INVALID_HARNESS"));
        assert!(md.contains("PILOT"));
        assert!(!md.contains("PILOT_ONLY"));
        assert!(md.contains("Invalid harness"));
        assert!(
            !md.contains("## U-NEG"),
            "InvalidHarness must not emit a U-NEG section"
        );
    }

    /// Pass-3 H2: without full reset, dendrite residue can survive into a later
    /// decision window via lazy `advance_to` from a stale `last`.
    #[test]
    fn incomplete_reset_leaves_dendrite_residue_for_next_window() {
        let mut eng = Engine::with_cells(2);
        eng.cell_mut(0).v_dend[0] = 3.0;
        eng.cell_mut(0).v = 1.5;
        eng.cell_mut(0).last = 0;
        // Advance engine clock without touching cell 0.
        let _ = eng.step_until(50);
        eng.cell_mut(0).advance_to(50);
        assert!(
            eng.cell(0).v_dend[0].abs() > 1e-3 || eng.cell(0).v.abs() > 1e-3,
            "residue should remain after incomplete path (got v_dend={:?} v={})",
            eng.cell(0).v_dend,
            eng.cell(0).v
        );
    }

    /// Isolation helper mirrors C3 v2: clears v_dend / stamps last / restores θ.
    #[test]
    fn reset_c1_dynamic_state_clears_all_cells_for_next_kwta() {
        let mut eng = Engine::with_cells(4);
        let hidden = [2u32, 3u32];
        let saved_theta = [1.0f32, 1.0];
        for i in 0..4 {
            let c = eng.cell_mut(i);
            c.v = 2.0 + i as f32;
            c.v_dend = [4.0, 3.0, 2.0, 1.0];
            c.theta = 9.0;
            c.last = 0;
        }
        let _ = eng.step_until(40);
        reset_c1_dynamic_state(&mut eng, &hidden, &saved_theta);
        let now = eng.time();
        for i in 0..4 {
            let c = eng.cell(i);
            assert_eq!(c.v, 0.0, "cell {i} soma");
            assert_eq!(c.v_dend, [0.0; K], "cell {i} dendrites");
            assert_eq!(c.last, now, "cell {i} last");
        }
        assert_eq!(eng.cell(2).theta, 1.0);
        assert_eq!(eng.cell(3).theta, 1.0);
        // After isolation reset, a fresh advance must stay at rest (no leak).
        eng.cell_mut(2).advance_to(now + 10);
        assert!(
            eng.cell(2).v.abs() < 1e-6 && eng.cell(2).v_dend.iter().all(|v| v.abs() < 1e-6),
            "isolated membrane must not resurrect residue into next k-WTA window"
        );
    }

    #[test]
    fn isolation_protocol_hash_distinct_and_render_discloses() {
        let iso = Config::c1_isolation_quick();
        assert!(iso.is_isolation_protocol());
        assert_eq!(iso.protocol_version(), C1_ISOLATION_PROTOCOL_VERSION);
        assert_ne!(iso.hash_string(), Config::c1_default().hash_string());
        let report = C1Report {
            config_hash: iso.hash_string(),
            seeds: Vec::new(),
            summary: PairedSummary {
                mean_local: 0.0,
                mean_dense: 0.0,
                mean_gradient_reference: 0.0,
                mean_eligibility_reference: 0.0,
                var_local: 0.0,
                var_dense: 0.0,
                var_gradient_reference: 0.0,
                var_eligibility_reference: 0.0,
                mean_gap_closed: 0.0,
                var_gap_closed: 0.0,
                gap_closed_lower_95: 0.0,
                mean_dist_to_dense: 0.0,
                n: 0,
            },
            verdict: GateG2Verdict::Pilot,
            positive_control_mean: 1.0,
            mean_activity_sparsity: 0.015,
            required_scientific_n_seeds: 20,
            budgets: Vec::new(),
            emitted: Vec::new(),
            plot_notes: Vec::new(),
        };
        let md = Runner::render_results_markdown(&report, &iso);
        assert!(md.contains("Trial-isolation protocol"));
        assert!(md.contains("c1-118207fbc3eaba53"));
    }

    /// Adversarial: with finite θ (spike protocol path), hidden cells can spike
    /// during the integrate window *before* forced k-WTA winners. With θ=∞ mute
    /// (canonical C1), they cannot.
    #[test]
    fn natural_spiking_path_allows_hidden_spikes_before_wta_mute_does_not() {
        let n_in = 2usize;
        let n_hidden = 4usize;
        let n_cells = n_in + n_hidden + 2;
        let hidden_start = n_in as CellId;
        let hidden_end = (n_in + n_hidden) as CellId;

        // Strong feedforward so a finite-θ hidden cell must cross threshold.
        let mut eng_mute = Engine::with_cells(n_cells);
        let mut eng_nat = Engine::with_cells(n_cells);
        let mut adj: Vec<Vec<u32>> = vec![Vec::new(); n_cells];
        let mut weights = Vec::new();
        for row in adj.iter_mut().take(n_in) {
            for post in hidden_start..hidden_end {
                row.push(post);
                weights.push(8.0f32);
            }
        }
        let conn = Csr::from_adjacency(&adj);
        assert_eq!(conn.nnz(), weights.len());
        eng_mute.set_connectivity(conn.clone(), weights.clone());
        eng_nat.set_connectivity(conn, weights);

        for cell in hidden_start..hidden_end {
            eng_mute.cell_mut(cell).theta = f32::INFINITY;
            eng_nat.cell_mut(cell).theta = 1.0; // finite resting-scale threshold
        }

        eng_mute.force_spike(0, 1);
        eng_mute.force_spike(1, 1);
        eng_nat.force_spike(0, 1);
        eng_nat.force_spike(1, 1);
        let until = 1 + eng_mute.max_synaptic_delay().max(1) + 2;
        let mute_produced = eng_mute.step_until(until);
        let nat_produced = eng_nat.step_until(until);

        let mute_hidden = mute_produced
            .as_slice()
            .iter()
            .filter(|sp| sp.cell >= hidden_start && sp.cell < hidden_end)
            .count();
        let nat_hidden = nat_produced
            .as_slice()
            .iter()
            .filter(|sp| sp.cell >= hidden_start && sp.cell < hidden_end)
            .count();
        assert_eq!(
            mute_hidden, 0,
            "θ=∞ mute must prevent natural hidden spikes before WTA"
        );
        assert!(
            nat_hidden > 0,
            "finite-θ spike protocol path must allow hidden spikes before WTA; got {nat_hidden}"
        );
    }

    #[test]
    fn spike_protocol_hash_distinct_and_render_discloses() {
        let spike = Config::c1_spike_quick();
        assert!(spike.is_spike_protocol());
        assert_eq!(spike.protocol_version(), C1_SPIKE_PROTOCOL_VERSION);
        assert_ne!(spike.hash_string(), Config::c1_default().hash_string());
        assert_ne!(spike.hash_string(), Config::c1_isolation().hash_string());
        let report = C1Report {
            config_hash: spike.hash_string(),
            seeds: Vec::new(),
            summary: PairedSummary {
                mean_local: 0.0,
                mean_dense: 0.0,
                mean_gradient_reference: 0.0,
                mean_eligibility_reference: 0.0,
                var_local: 0.0,
                var_dense: 0.0,
                var_gradient_reference: 0.0,
                var_eligibility_reference: 0.0,
                mean_gap_closed: 0.0,
                var_gap_closed: 0.0,
                gap_closed_lower_95: 0.0,
                mean_dist_to_dense: 0.0,
                n: 0,
            },
            verdict: GateG2Verdict::Pilot,
            positive_control_mean: 1.0,
            mean_activity_sparsity: 0.015,
            required_scientific_n_seeds: 20,
            budgets: Vec::new(),
            emitted: Vec::new(),
            plot_notes: Vec::new(),
        };
        let md = Runner::render_results_markdown(&report, &spike);
        assert!(md.contains("Natural-hidden-spiking protocol"));
        assert!(md.contains("c1-118207fbc3eaba53"));
        assert!(md.contains("no θ=∞ mute") || md.contains("finite hidden θ"));
    }

    #[test]
    fn spike_s_protocol_hash_distinct_and_render_discloses() {
        let spike_s = Config::c1_spike_s_quick();
        assert!(spike_s.is_spike_s_protocol());
        assert!(spike_s.is_spike_protocol());
        assert_eq!(spike_s.protocol_version(), C1_SPIKE_S_PROTOCOL_VERSION);
        assert_ne!(spike_s.hash_string(), Config::c1_spike().hash_string());
        assert_ne!(spike_s.hash_string(), "c1-09442acdbdc0c752");
        assert_ne!(spike_s.hash_string(), "c1-118207fbc3eaba53");
        let report = C1Report {
            config_hash: spike_s.hash_string(),
            seeds: Vec::new(),
            summary: PairedSummary {
                mean_local: 0.0,
                mean_dense: 0.0,
                mean_gradient_reference: 0.0,
                mean_eligibility_reference: 0.0,
                var_local: 0.0,
                var_dense: 0.0,
                var_gradient_reference: 0.0,
                var_eligibility_reference: 0.0,
                mean_gap_closed: 0.0,
                var_gap_closed: 0.0,
                gap_closed_lower_95: 0.0,
                mean_dist_to_dense: 0.0,
                n: 0,
            },
            verdict: GateG2Verdict::Pass,
            positive_control_mean: 1.0,
            mean_activity_sparsity: 0.015,
            required_scientific_n_seeds: 20,
            budgets: Vec::new(),
            emitted: Vec::new(),
            plot_notes: Vec::new(),
        };
        let md = Runner::render_results_markdown(&report, &spike_s);
        assert!(md.contains("Calibrated natural-spiking protocol"));
        assert!(md.contains("spike-count k-WTA"));
        assert!(md.contains("c1-09442acdbdc0c752"));
        assert!(md.contains("c1-118207fbc3eaba53"));
    }

    #[test]
    fn project_protocol_invokes_assembly_project_and_discloses() {
        C1_PROJECT_INVOKE_COUNT.store(0, Ordering::Relaxed);
        let cfg = Config::c1_project_quick();
        assert!(cfg.is_project_protocol());
        assert_eq!(cfg.protocol_version(), C1_PROJECT_PROTOCOL_VERSION);
        assert_ne!(cfg.hash_string(), Config::c1_default().hash_string());
        assert_ne!(cfg.hash_string(), "c1-118207fbc3eaba53");

        // One local-assembly seed is enough to prove `project` is on the path.
        let split = freeze_trials(&cfg, cfg.seeds()[0]);
        let _ = run_local_assembly(&cfg, cfg.seeds()[0], &split);
        assert!(
            C1_PROJECT_INVOKE_COUNT.load(Ordering::Relaxed) > 0,
            "c1-project path must invoke binn_areas::project"
        );

        let report = C1Report {
            config_hash: cfg.hash_string(),
            seeds: Vec::new(),
            summary: PairedSummary {
                mean_local: 0.0,
                mean_dense: 0.0,
                mean_gradient_reference: 0.0,
                mean_eligibility_reference: 0.0,
                var_local: 0.0,
                var_dense: 0.0,
                var_gradient_reference: 0.0,
                var_eligibility_reference: 0.0,
                mean_gap_closed: 0.0,
                var_gap_closed: 0.0,
                gap_closed_lower_95: 0.0,
                mean_dist_to_dense: 0.0,
                n: 0,
            },
            verdict: GateG2Verdict::Pilot,
            positive_control_mean: 1.0,
            mean_activity_sparsity: 0.015,
            required_scientific_n_seeds: 20,
            budgets: Vec::new(),
            emitted: Vec::new(),
            plot_notes: Vec::new(),
        };
        let md = Runner::render_results_markdown(&report, &cfg);
        assert!(md.contains("Assembly-Calculus `project` protocol") || md.contains("project"));
        assert!(md.contains("c1-118207fbc3eaba53"));
    }

    #[test]
    fn reinforce_fb_protocol_hash_distinct_and_render_discloses() {
        let rfb = Config::c1_reinforce_fb_quick();
        assert!(rfb.is_reinforce_fb_protocol());
        assert_eq!(rfb.protocol_version(), C1_REINFORCE_FB_PROTOCOL_VERSION);
        assert_ne!(rfb.hash_string(), Config::c1_default().hash_string());
        assert_ne!(rfb.hash_string(), "c1-118207fbc3eaba53");
        assert_eq!(rfb.hash_string(), "c1-a57975f13b73a599");

        let report = C1Report {
            config_hash: rfb.hash_string(),
            seeds: Vec::new(),
            summary: PairedSummary {
                mean_local: 0.0,
                mean_dense: 0.0,
                mean_gradient_reference: 0.0,
                mean_eligibility_reference: 0.0,
                var_local: 0.0,
                var_dense: 0.0,
                var_gradient_reference: 0.0,
                var_eligibility_reference: 0.0,
                mean_gap_closed: 0.0,
                var_gap_closed: 0.0,
                gap_closed_lower_95: 0.0,
                mean_dist_to_dense: 0.0,
                n: 0,
            },
            verdict: GateG2Verdict::Pilot,
            positive_control_mean: 1.0,
            mean_activity_sparsity: 0.015,
            required_scientific_n_seeds: 20,
            budgets: Vec::new(),
            emitted: Vec::new(),
            plot_notes: Vec::new(),
        };
        let md = Runner::render_results_markdown(&report, &rfb);
        assert!(md.contains("Live `ReinforceFeedback` protocol"));
        assert!(md.contains("reinforce_term") || md.contains("ReinforceFeedback"));
        assert!(md.contains("c1-118207fbc3eaba53"));
        assert!(!md.contains("two-sided ±1 reward"));
    }

    #[test]
    fn reinforce_fb_positive_control_uses_broadcast_and_clears_floor() {
        let cfg = Config::c1_reinforce_fb_quick();
        assert!(cfg.is_reinforce_fb_protocol());
        let accs: Vec<f32> = cfg
            .seeds()
            .into_iter()
            .map(|seed| run_positive_control(&cfg, seed))
            .collect();
        let (mean, _) = mean_var(&accs);
        assert!(
            mean >= cfg.g2_min_positive_control,
            "rfb PC (broadcast substrate check) mean {mean} < {}",
            cfg.g2_min_positive_control
        );
    }

    #[test]
    fn gap_close_protocols_render_discloses_and_freeze_hashes() {
        let cases: [(&str, Config, &str); 6] = [
            (
                "v14",
                Config::c1_reinforce_fb_epoch_quick(),
                "Live RFB × epoch-matched protocol",
            ),
            (
                "v15",
                Config::c1_structured_fb_quick(),
                "Structured frozen feedback protocol",
            ),
            (
                "v16",
                Config::c1_structured_fb_epoch_quick(),
                "Structured B × epoch-matched protocol",
            ),
            (
                "v17",
                Config::c1_structured_fb_capacity_quick(),
                "Structured B × capacity protocol",
            ),
            (
                "v18",
                Config::c1_elig_rfb_quick(),
                "Eligibility × REINFORCE protocol",
            ),
            (
                "v19",
                Config::c1_structured_fb_teach_quick(),
                "Structured B × target teach protocol",
            ),
        ];
        for (label, cfg, needle) in cases {
            assert!(
                cfg.uses_live_reinforce_feedback(),
                "{label} should use live RFB plasticity"
            );
            let report = C1Report {
                config_hash: cfg.hash_string(),
                seeds: Vec::new(),
                summary: PairedSummary {
                    mean_local: 0.0,
                    mean_dense: 0.0,
                    mean_gradient_reference: 0.0,
                    mean_eligibility_reference: 0.0,
                    var_local: 0.0,
                    var_dense: 0.0,
                    var_gradient_reference: 0.0,
                    var_eligibility_reference: 0.0,
                    mean_gap_closed: 0.0,
                    var_gap_closed: 0.0,
                    gap_closed_lower_95: 0.0,
                    mean_dist_to_dense: 0.0,
                    n: 0,
                },
                verdict: GateG2Verdict::Pilot,
                positive_control_mean: 1.0,
                mean_activity_sparsity: 0.015,
                required_scientific_n_seeds: 20,
                budgets: Vec::new(),
                emitted: Vec::new(),
                plot_notes: Vec::new(),
            };
            let md = Runner::render_results_markdown(&report, &cfg);
            assert!(
                md.contains(needle),
                "{label} markdown missing disclosure `{needle}`"
            );
            assert!(
                md.contains("c1-118207fbc3eaba53"),
                "{label} must refuse reopening v2"
            );
            assert!(
                !md.contains("two-sided ±1 reward"),
                "{label} must not describe default ±1 main-arm language"
            );
        }
        // Scientific hash freeze (paper table).
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
    }

    #[test]
    fn structured_feedback_signs_follow_readout_columns_after_boost() {
        let cfg = Config::c1_structured_fb_quick();
        let n_in = 2usize;
        let n_hidden = cfg.n_hidden;
        let readout_0 = (n_in + n_hidden) as CellId;
        let readout_1 = readout_0 + 1;
        let n_cells = n_in + n_hidden + 2;
        let mut eng = Engine::with_cells(n_cells);
        let (conn, init_w) = build_sparse_assembly(&cfg, 99, n_in, n_hidden, readout_0, readout_1);
        let nnz = conn.nnz();
        eng.set_connectivity(conn, vec![init_w; nnz]);
        let boost = (1.15 / init_w.max(1e-3)).clamp(1.0, 12.0);
        boost_readout_incoming(&mut eng, readout_0, readout_1, boost);
        let fb = structured_reinforce_feedback(
            &eng, n_cells, n_in, n_hidden, readout_0, readout_1, 99, false,
        );
        let mut signed = 0usize;
        for h in 0..n_hidden {
            let cell = (n_in + h) as CellId;
            let w0 = edge_index(&eng.conn, cell, readout_0)
                .map(|i| eng.edge_w[i])
                .unwrap_or(0.0);
            let w1 = edge_index(&eng.conn, cell, readout_1)
                .map(|i| eng.edge_w[i])
                .unwrap_or(0.0);
            let d = w1 - w0;
            let b = fb.weights()[cell as usize];
            if d.abs() < 1e-8 {
                assert!(b.abs() < 1e-6, "zero-diff hidden should get B=0, got {b}");
            } else {
                assert_eq!(
                    b.signum(),
                    d.signum(),
                    "structured B must match sign(w1-w0) for cell {cell}"
                );
                signed += 1;
            }
        }
        assert!(
            signed > 0,
            "expected at least one hidden with nonzero readout column diff"
        );
    }

    #[test]
    fn soft_readout_policy_is_sigmoid_of_charge_diff() {
        assert!((soft_readout_policy(0.0, 0.0) - 0.5).abs() < 1e-6);
        assert!(soft_readout_policy(0.0, 5.0) > 0.9);
        assert!(soft_readout_policy(5.0, 0.0) < 0.1);
    }
}
