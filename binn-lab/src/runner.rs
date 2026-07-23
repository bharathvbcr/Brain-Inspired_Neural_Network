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

use binn_areas::{k_wta, wire, Area, AreaRole, Pos, WiringPrior};
use binn_core::{Csr, Rng, Tick};
use binn_data::{
    CoincidenceTask, Encoder, LatencyEncoder, Metrics, Sample, WorkCosts, WorkCounters,
};
use binn_engine::{CellId, Engine};
use binn_learn::{
    BpttBaseline, EpropReference, GradientExample, Modulators, SurrogateLifReference, ThreeFactor,
    REFERENCE_SEQUENCE_LEN,
};

use crate::config::Config;
use crate::logging::{RunLog, StructuredLogger};
use crate::plots::Plots;

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
struct FrozenSplit {
    train: Vec<(Vec<Sample>, u32)>,
    test: Vec<(Vec<Sample>, u32)>,
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
        format!(
            "{{\"condition\":\"{}\",\"seed\":{},\"accuracy\":{:.8},\"activity_sparsity\":{:.8},\"n_cells\":{},\"n_params\":{},\"wall_secs\":{:.8},\"peak_rss_bytes\":{},\"source_spikes\":{},\"synaptic_deliveries\":{},\"cell_updates\":{},\"plasticity_updates\":{},\"work_per_accuracy\":{:.8}}}",
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
        )
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
                let mut entry = RunLog::new(&config_hash, seed, cond.as_str())
                    .with_activity_sparsity(outcome.activity_sparsity);
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

        let summary = summarize_paired(&seeds_out, config.g2_confidence_z);
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
        md.push_str("| `local-assembly` | Three-factor rule + sparse assembly wiring + k-WTA + dual readouts + ±1 reward |\n");
        md.push_str("| `dense-local` | Same three-factor rule + same k-winner budget on dense all-to-all connectivity, **no** assembly structure |\n");
        if config.matched_budget_repeat {
            md.push_str("| `dense-matched` | Dense-local with nnz matched to local-assembly (compute-matched disclosure) |\n");
        }
        if config.use_surrogate_lif_reference {
            md.push_str("| `gradient-reference` | Same-architecture surrogate-LIF BPTT (primary); tanh RNN optional/secondary |\n");
        } else {
            md.push_str("| `gradient-reference` | Labeled tanh-RNN BPTT (`BpttBaseline`); secondary/optional ceiling |\n");
        }
        md.push_str("| `eligibility-reference` | E-prop-compatible eligibility local reference (GC1 baseline) |\n\n");
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
            "- mean |local − dense| (descriptive): {:.4}\n\n",
            sum.mean_dist_to_dense
        ));
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
            "## Positive / sanity control\n\nMean local-pipeline accuracy on a trivially separable task: **{:.4}** (threshold {:.3}).\n\n",
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
        md.push_str("| condition | n_cells | n_params | wall_secs | peak_rss_bytes | work_per_accuracy | spikes | deliveries | cell_updates | plasticity |\n");
        md.push_str("|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|\n");
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
            md.push_str(&format!(
                "| {} | {} | {} | {:.4} | {} | {:.4} | {} | {} | {} | {} |\n",
                cond.as_str(),
                b.n_cells,
                b.n_params,
                b.wall_secs,
                b.peak_rss_bytes,
                b.work_per_accuracy,
                b.work.source_spikes,
                b.work.synaptic_deliveries,
                b.work.cell_updates,
                b.work.plasticity_updates
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
}

fn freeze_trials(config: &Config, seed: u64) -> FrozenSplit {
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
fn run_condition_prefer_isolated(
    config: &Config,
    seed: u64,
    label: ConditionLabel,
    split: &FrozenSplit,
    match_nnz: Option<usize>,
) -> CondOutcome {
    if std::env::var_os("BINN_CONDITION_CHILD").is_none() {
        if let Some(outcome) = try_isolate_condition(config, seed, label, match_nnz) {
            return outcome;
        }
    }
    run_labeled_condition(config, seed, label, split, match_nnz)
}

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
    })
}

fn samples_to_gradient_examples(trials: &[(Vec<Sample>, u32)]) -> Vec<GradientExample> {
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
    }
}

fn run_local_assembly(config: &Config, seed: u64, split: &FrozenSplit) -> CondOutcome {
    run_spiking_condition(config, seed, true, false, split, None)
}

/// Positive/sanity control: same local-assembly pipeline on a trivially
/// separable task.
fn run_positive_control(config: &Config, seed: u64) -> f32 {
    // Give the sanity control enough trials to clear the harness floor even on
    // the short quick schedule (scientific configs already have n_train ≥ 80).
    let mut cfg = config.clone();
    cfg.n_train = cfg.n_train.max(48);
    cfg.n_test = cfg.n_test.max(24);
    let mut easy_rng = Rng::new(seed ^ 0x0E51_EA51);
    let easy_len = cfg.sequence_len.max(2);
    let train: Vec<_> = (0..cfg.n_train)
        .map(|_| easy_trial(&mut easy_rng, easy_len))
        .collect();
    let test: Vec<_> = (0..cfg.n_test)
        .map(|_| easy_trial(&mut easy_rng, easy_len))
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
    let (conn, init_w) = if assembly {
        build_sparse_assembly(config, seed, n_in, n_hidden, readout_0, readout_1)
    } else {
        build_dense_local(config, n_in, n_hidden, readout_0, readout_1, match_nnz)
    };
    let nnz = conn.nnz();
    eng.set_connectivity(conn, vec![init_w; nnz]);

    let mut area = Area::new(n_in as CellId..(n_in + n_hidden) as CellId, config.k_wta);
    let mut learner = ThreeFactor::new(config.eta, config.lambda, config.tau_e);
    // Per-frame latency bins; full event stream uses frame_offset + latency.
    let enc = LatencyEncoder::new(2, (config.sequence_len as Tick).max(1), 0);

    let mut weight_steps = Vec::new();
    let mut weight_trace = Vec::new();
    let mut t_cursor: Tick = 0;
    let mut plasticity_updates = 0u64;

    for (step, (seq, label)) in split.train.iter().enumerate() {
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
        );
        plasticity_updates = plasticity_updates.saturating_add(n_plas);
        t_cursor = eng.time() + 20;
        if let Some(w) = mean_readout_weight(&eng, readout_0, readout_1) {
            weight_steps.push(step as f64);
            weight_trace.push(w as f64);
        }
    }

    let mut correct = 0usize;
    let mut active_total = 0usize;
    let mut pop_total = 0usize;
    let mut raster_t = Vec::new();
    let mut raster_cell = Vec::new();

    for (trial_i, (seq, label)) in split.test.iter().enumerate() {
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
        );
        if pred_ok {
            correct += 1;
        }
        active_total += sample.active;
        pop_total += sample.population;
        if trial_i == 0 {
            for sp in eng.spikes().as_slice().iter().rev().take(64) {
                raster_t.push(sp.t as f64);
                raster_cell.push(sp.cell as f64);
            }
        }
        t_cursor = eng.time() + 20;
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
    }
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
) -> (bool, SparsitySample, u64) {
    // True temporal input: encode every frame (no peak collapse).
    let frame_stride = enc.max_delay().saturating_add(1);
    let hidden_cells: Vec<CellId> = area.cells.clone().collect();
    let saved_thresholds: Vec<f32> = hidden_cells
        .iter()
        .map(|&cell| eng.cell(cell).theta)
        .collect();
    for &cell in &hidden_cells {
        eng.cell_mut(cell).theta = f32::INFINITY;
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

    // Membrane-state k-WTA: score Cell::v at decision time (preserves timing).
    let scores: Vec<(CellId, f32)> = hidden_cells
        .iter()
        .map(|&cell| {
            eng.cell_mut(cell).advance_to(selection_until);
            (cell, eng.cell(cell).v)
        })
        .filter(|(_, v)| v.is_finite() && *v > 0.0)
        .collect();
    let active_cells = k_wta(&scores, area.effective_k());
    let active = active_cells.len();
    let population = area.len();
    area.log_activity(active);

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

    // Force-spike the selected action readout so STDP sees a postsynaptic event.
    let selected = if pred == 0 { readout_0 } else { readout_1 };
    let action_at = readout_until.checked_add(1).expect("action time overflow");
    eng.force_spike(selected, action_at);
    let until = action_at
        .checked_add(eng.max_synaptic_delay().max(1) + 4)
        .expect("trial horizon overflow");
    let _ = eng.step_until(until);

    let mut plasticity_apps = 0u64;
    if train {
        let correct = pred == label;
        let reward = if correct { 1.0 } else { -1.0 };
        plasticity_apps = learner.update_counted(eng, Modulators::reward(reward));
    }

    for (&cell, &theta) in hidden_cells.iter().zip(saved_thresholds.iter()) {
        let hidden = eng.cell_mut(cell);
        hidden.theta = theta;
        hidden.v = 0.0;
    }
    eng.close_inhibited_cycle();

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

fn build_sparse_assembly(
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
    );
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
            rows[n_in].push(ro);
        }
    }
    for row in &mut rows {
        row.sort_unstable();
        row.dedup();
    }
    (Csr::from_adjacency(&rows), config.init_w)
}

fn build_dense_local(
    config: &Config,
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
    for post in n_in..(n_in + n_hidden) {
        rows[readout_0 as usize].push(post as u32);
        rows[readout_1 as usize].push(post as u32);
    }

    if let Some(target) = match_nnz {
        // Parameter-matched disclosure: subsample hidden→hidden edges to ≈ target nnz.
        let mut flat: Vec<(usize, u32)> = Vec::new();
        for (pre, row) in rows.iter().enumerate() {
            for &post in row {
                flat.push((pre, post));
            }
        }
        if flat.len() > target && target > 0 {
            let mut rng = Rng::new(0x7A7C_B001_u64 ^ (n_hidden as u64));
            // Fisher–Yates partial shuffle then keep prefix.
            for i in 0..flat.len() {
                let j = i + rng.gen_index(flat.len() - i);
                flat.swap(i, j);
            }
            flat.truncate(target);
            rows = vec![Vec::new(); n_cells];
            for (pre, post) in flat {
                rows[pre].push(post);
            }
            for row in &mut rows {
                row.sort_unstable();
                row.dedup();
            }
            // Keep both readouts reachable.
            for &ro in &[readout_0, readout_1] {
                if !(0..n_cells).any(|pre| rows[pre].contains(&ro)) {
                    rows[n_in].push(ro);
                }
            }
        }
    }

    (Csr::from_adjacency(&rows), config.init_w)
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

fn edge_index(conn: &Csr, pre: CellId, post: CellId) -> Option<usize> {
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

fn mean_var(xs: &[f32]) -> (f32, f32) {
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

fn summarize_paired(seeds: &[SeedResult], confidence_z: f32) -> PairedSummary {
    assert!(confidence_z.is_finite() && confidence_z >= 0.0);
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
        let closed = if reference_gap > 1e-6 {
            (s.local_assembly - s.dense_local) / reference_gap
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

fn gate_g2(summary: &PairedSummary, min_gap_closed: f32, min_accuracy: f32) -> GateG2Verdict {
    if summary.gap_closed_lower_95 > min_gap_closed && summary.mean_local >= min_accuracy {
        GateG2Verdict::Pass
    } else {
        GateG2Verdict::Fail
    }
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
        let summary = summarize_paired(&seeds, 0.0);
        assert!((summary.mean_gap_closed - 0.25).abs() < 1e-6);
        assert_eq!(summary.gap_closed_lower_95, summary.mean_gap_closed);
    }

    #[test]
    fn dual_readout_builders_disclose_two_readouts() {
        let cfg = Config::c1_quick();
        let n_in = 2;
        let n_h = cfg.n_hidden;
        let r0 = (n_in + n_h) as CellId;
        let r1 = r0 + 1;
        let (sparse, _) = build_sparse_assembly(&cfg, 1, n_in, n_h, r0, r1);
        let (dense, _) = build_dense_local(&cfg, n_in, n_h, r0, r1, None);
        assert_eq!(sparse.nrows(), n_in + n_h + 2);
        assert_eq!(dense.nrows(), n_in + n_h + 2);
        assert!(sparse.nnz() > 0);
        assert!(dense.nnz() >= sparse.nnz());
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
        let expected = (cfg.n_train as u64).saturating_mul(nnz);
        assert_eq!(
            outcome.budget.work.plasticity_updates, expected,
            "plasticity_updates should be n_train × nnz"
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
}
