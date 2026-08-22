//! Matched-architecture DFA recipe runner (protocol v5).
//!
//! Paired arms share one dense-LIF forward ([`MatchedArch`]); learning rule is
//! the only variable. Primary gated arm: [`MatchedDfa`] (directional graded
//! error × fixed-random feedback). Contrast arm: [`MatchedBroadcastErr`]
//! (same graded error, broadcast scalar). Ceiling: [`MatchedGradient`].
//!
//! Does not reopen protocol-v2 or mutate protocol-v4 `c1-match-*`.

use binn_learn::{
    MatchedBroadcastErr, MatchedDfa, MatchedGradient, DEFAULT_MATCHED_BETA,
    MATCHED_BROADCAST_ERR_LABEL, MATCHED_DFA_LABEL, MATCHED_GRADIENT_LABEL, REFERENCE_SEQUENCE_LEN,
};

use crate::dfa_match_config::{DfaMatchConfig, C1_DFA_CHANCE_BASELINE};
use crate::runner::{freeze_trials, mean_var, samples_to_gradient_examples, GateG2Verdict};
use crate::runner_match::gap_closed_matched;

/// Per-seed accuracies and gap_closed_dfa (DFA vs gradient).
#[derive(Clone, Debug, PartialEq)]
pub struct DfaMatchSeedResult {
    pub seed: u64,
    pub matched_dfa: f32,
    pub matched_broadcast_err: f32,
    pub matched_gradient: f32,
    pub gap_closed_dfa: f32,
}

/// Aggregated matched-DFA report.
#[derive(Clone, Debug, PartialEq)]
pub struct DfaMatchReport {
    pub config_hash: String,
    pub protocol_version: u64,
    pub seeds: Vec<DfaMatchSeedResult>,
    pub mean_matched_dfa: f32,
    pub variance_matched_dfa: f32,
    pub mean_matched_broadcast_err: f32,
    pub variance_matched_broadcast_err: f32,
    pub mean_matched_gradient: f32,
    pub variance_matched_gradient: f32,
    pub mean_gap_closed_dfa: f32,
    pub variance_gap_closed_dfa: f32,
    pub gap_closed_dfa_lower_95: f32,
    pub verdict: GateG2Verdict,
    pub pilot: bool,
}

#[derive(Default)]
pub struct DfaMatchRunner;

impl DfaMatchRunner {
    pub fn new() -> Self {
        Self
    }

    /// Run DFA + broadcast-err + gradient on identical frozen splits.
    pub fn run(&mut self, config: &DfaMatchConfig) -> DfaMatchReport {
        assert!(config.base.n_seeds >= 1);
        assert!(config.base.bptt_epochs >= 1);
        assert_eq!(
            config.base.sequence_len, REFERENCE_SEQUENCE_LEN,
            "matched-dfa requires sequence_len={REFERENCE_SEQUENCE_LEN}"
        );
        assert!(
            (config.chance_baseline - C1_DFA_CHANCE_BASELINE).abs() < 1e-6,
            "chance baseline is locked at {C1_DFA_CHANCE_BASELINE}"
        );

        let mut seeds = Vec::with_capacity(config.base.n_seeds);
        for seed in config.seeds() {
            seeds.push(run_seed(config, seed));
        }

        let summary = summarize(config, &seeds);
        let pilot = config.quick || config.base.n_seeds < config.scientific_n_seeds;
        DfaMatchReport {
            config_hash: config.hash_string(),
            protocol_version: config.protocol_version,
            seeds,
            mean_matched_dfa: summary.mean_dfa,
            variance_matched_dfa: summary.var_dfa,
            mean_matched_broadcast_err: summary.mean_broadcast,
            variance_matched_broadcast_err: summary.var_broadcast,
            mean_matched_gradient: summary.mean_gradient,
            variance_matched_gradient: summary.var_gradient,
            mean_gap_closed_dfa: summary.mean_gap,
            variance_gap_closed_dfa: summary.var_gap,
            gap_closed_dfa_lower_95: summary.gap_lcb,
            verdict: summary.verdict,
            pilot,
        }
    }

    /// Render a self-contained results note.
    pub fn render_markdown(report: &DfaMatchReport, config: &DfaMatchConfig) -> String {
        let mut md = String::new();
        md.push_str("# BINN matched-architecture DFA recipe (C1-DFA)\n\n");
        md.push_str(
            "**Does not reopen protocol-v2:** hash `c1-118207fbc3eaba53` and every \
             G2 threshold remain unchanged. This is matched-arch protocol **v5** \
             with a fresh `c1-dfa-*` hash (distinct from trial-isolation `c1-iso*` \
             which also uses integer 5). Mechanism: **directional graded error × \
             fixed-random DFA feedback** on the dense-LIF matched forward (feed-forward `wrec=0`, matching the NumPy DFA preview).\n\n",
        );
        md.push_str(&format!(
            "- schedule: **{}**\n\
             - config hash: `{}`\n\
             - protocol version: {}\n\
             - seeds: {}\n\
             - train/test: {}/{}\n\
             - hidden / epochs / β: {} / {} / {:.1}\n\
             - gradient lr / local η / λ: {:.4} / {:.4} / {:.4}\n\
             - chance baseline: {:.1}\n\n",
            if report.pilot {
                "PILOT (development only — not a scientific verdict)"
            } else {
                "SCIENTIFIC"
            },
            report.config_hash,
            report.protocol_version,
            config.base.n_seeds,
            config.base.n_train,
            config.base.n_test,
            config.base.n_hidden,
            config.base.bptt_epochs,
            config.base.surrogate_beta,
            config.base.bptt_lr,
            config.base.eta,
            config.base.lambda,
            config.chance_baseline,
        ));

        md.push_str("## Results\n\n");
        md.push_str(&format!(
            "| arm | mean accuracy | variance |\n\
             |---|---:|---:|\n\
             | `{MATCHED_DFA_LABEL}` (graded error × DFA) | {:.4} | {:.6} |\n\
             | `{MATCHED_BROADCAST_ERR_LABEL}` (graded error, broadcast) | {:.4} | {:.6} |\n\
             | `{MATCHED_GRADIENT_LABEL}` (SuperSpike BPTT ceiling) | {:.4} | {:.6} |\n\n",
            report.mean_matched_dfa,
            report.variance_matched_dfa,
            report.mean_matched_broadcast_err,
            report.variance_matched_broadcast_err,
            report.mean_matched_gradient,
            report.variance_matched_gradient,
        ));
        md.push_str(&format!(
            "- `gap_closed_dfa` mean: **{:.4}**  (var {:.6})\n\
             - lower 95% CB (z={:.2}): **{:.4}**  (needs > {:.2})\n\
             - accuracy floor (matched-dfa ≥ {:.2}): {}\n\
             - harness validity (matched-gradient ≥ {:.2}): {}\n\
             - **verdict: {}**\n\n",
            report.mean_gap_closed_dfa,
            report.variance_gap_closed_dfa,
            config.base.g2_confidence_z,
            report.gap_closed_dfa_lower_95,
            config.base.g2_min_gap_closed,
            config.base.g2_min_accuracy,
            if report.mean_matched_dfa >= config.base.g2_min_accuracy {
                "met"
            } else {
                "not met"
            },
            config.base.g2_min_accuracy,
            if report.mean_matched_gradient >= config.base.g2_min_accuracy {
                "met"
            } else {
                "FAILED → INVALID_HARNESS"
            },
            report.verdict.as_str(),
        ));

        md.push_str("## Per-seed\n\n");
        md.push_str(
            "| seed | dfa | broadcast-err | gradient | gap_closed_dfa |\n\
             |---:|---:|---:|---:|---:|\n",
        );
        for s in &report.seeds {
            md.push_str(&format!(
                "| {} | {:.4} | {:.4} | {:.4} | {:.4} |\n",
                s.seed,
                s.matched_dfa,
                s.matched_broadcast_err,
                s.matched_gradient,
                s.gap_closed_dfa
            ));
        }

        md.push_str(
            "\n## Gate (unchanged thresholds)\n\n\
             Primary arm = DFA. `gap_closed_dfa = (matched_dfa − 0.5) / \
             (matched_gradient − 0.5)`, clamped to [0,1]; seeds with \
             `(matched_gradient − 0.5) < g2_min_reference_gap` contribute \
             `closed = 0`. PASS requires gap LCB > 0.5 and mean matched-dfa \
             ≥ 0.65; mean matched-gradient < 0.65 ⇒ INVALID_HARNESS.\n\n",
        );
        md.push_str(
            "## Spiking-substrate note\n\n\
             The dense-LIF result above isolates the **rule**. On the real BINN \
             path (LatencyEncoder + k-WTA + single online pass), the exact-forward \
             DFA arm is already preregistered under credit-assignment \
             (`dfa-exact-forward`, hashes `c1x-dfa-exact-forward-*` / \
             `c1x-iso-dfa-exact-forward-*`). Those runs do **not** clear G2 — the \
             k-WTA / single-pass substrate re-introduces the handicap even when \
             the learning signal is graded + directional. See \
             `results/credit_assignment_iso_SUMMARY.md`.\n\n",
        );
        md.push_str(
            "## Reproduce\n\n\
             ```bash\n\
             cargo run --locked --release -p binn-lab --bin c1 -- --matched-dfa --quick\n\
             cargo run --locked --release -p binn-lab --bin c1 -- --matched-dfa --out results/c1_dfa.md\n\
             ```\n",
        );
        md
    }
}

struct SummaryParts {
    mean_dfa: f32,
    var_dfa: f32,
    mean_broadcast: f32,
    var_broadcast: f32,
    mean_gradient: f32,
    var_gradient: f32,
    mean_gap: f32,
    var_gap: f32,
    gap_lcb: f32,
    verdict: GateG2Verdict,
}

fn run_seed(config: &DfaMatchConfig, seed: u64) -> DfaMatchSeedResult {
    let split = freeze_trials(&config.base, seed);
    let train = samples_to_gradient_examples(&split.train);
    let test = samples_to_gradient_examples(&split.test);
    let beta = if config.base.surrogate_beta > 0.0 {
        config.base.surrogate_beta
    } else {
        DEFAULT_MATCHED_BETA
    };
    let epochs = config.base.bptt_epochs;

    let mut gradient =
        MatchedGradient::new_feedforward(config.base.n_hidden, config.base.bptt_lr, beta, seed);
    let grad_report = gradient.train_and_evaluate(epochs, &train, &test);

    let mut dfa = MatchedDfa::new(
        config.base.n_hidden,
        config.base.eta,
        config.base.lambda,
        beta,
        seed,
    );
    let dfa_report = dfa.train_and_evaluate(epochs, &train, &test);

    let mut broadcast = MatchedBroadcastErr::new(
        config.base.n_hidden,
        config.base.eta,
        config.base.lambda,
        beta,
        seed,
    );
    let broadcast_report = broadcast.train_and_evaluate(epochs, &train, &test);

    let gap = gap_closed_matched(
        dfa_report.accuracy,
        grad_report.accuracy,
        config.chance_baseline,
        config.base.g2_min_reference_gap,
    );
    DfaMatchSeedResult {
        seed,
        matched_dfa: dfa_report.accuracy,
        matched_broadcast_err: broadcast_report.accuracy,
        matched_gradient: grad_report.accuracy,
        gap_closed_dfa: gap,
    }
}

fn summarize(config: &DfaMatchConfig, seeds: &[DfaMatchSeedResult]) -> SummaryParts {
    let dfa: Vec<f32> = seeds.iter().map(|s| s.matched_dfa).collect();
    let broadcast: Vec<f32> = seeds.iter().map(|s| s.matched_broadcast_err).collect();
    let gradient: Vec<f32> = seeds.iter().map(|s| s.matched_gradient).collect();
    let gaps: Vec<f32> = seeds.iter().map(|s| s.gap_closed_dfa).collect();
    let (mean_dfa, var_dfa) = mean_var(&dfa);
    let (mean_broadcast, var_broadcast) = mean_var(&broadcast);
    let (mean_gradient, var_gradient) = mean_var(&gradient);
    let (mean_gap, var_gap) = mean_var(&gaps);
    let n = gaps.len();
    let gap_lcb = if n > 1 {
        mean_gap - config.base.g2_confidence_z * (var_gap / n as f32).sqrt()
    } else {
        mean_gap
    };

    let verdict = decide_verdict(config, mean_dfa, mean_gradient, gap_lcb);
    SummaryParts {
        mean_dfa,
        var_dfa,
        mean_broadcast,
        var_broadcast,
        mean_gradient,
        var_gradient,
        mean_gap,
        var_gap,
        gap_lcb,
        verdict,
    }
}

fn decide_verdict(
    config: &DfaMatchConfig,
    mean_dfa: f32,
    mean_gradient: f32,
    gap_lcb: f32,
) -> GateG2Verdict {
    if mean_gradient < config.base.g2_min_accuracy {
        return GateG2Verdict::InvalidHarness;
    }
    if config.quick || config.base.n_seeds < config.scientific_n_seeds {
        return GateG2Verdict::Pilot;
    }
    if gap_lcb > config.base.g2_min_gap_closed && mean_dfa >= config.base.g2_min_accuracy {
        GateG2Verdict::Pass
    } else {
        GateG2Verdict::Fail
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dfa_match_config::C1_DFA_PROTOCOL_VERSION;

    #[test]
    fn gap_metric_matches_matched_arch() {
        assert!((gap_closed_matched(0.9, 1.0, 0.5, 0.05) - 0.8).abs() < 1e-6);
        assert_eq!(gap_closed_matched(0.9, 0.52, 0.5, 0.05), 0.0); // ref gap too small
    }

    #[test]
    fn protocol_is_v5() {
        assert_eq!(C1_DFA_PROTOCOL_VERSION, 5);
        assert_eq!(DfaMatchConfig::scientific().protocol_version, 5);
    }

    #[test]
    fn quick_run_is_finite_and_pilot() {
        let config = DfaMatchConfig::quick();
        let mut runner = DfaMatchRunner::new();
        let report = runner.run(&config);
        assert!(report.pilot);
        assert_eq!(report.verdict, GateG2Verdict::Pilot);
        assert!(report.mean_matched_dfa.is_finite());
        assert!(report.mean_matched_broadcast_err.is_finite());
        assert!(report.mean_matched_gradient.is_finite());
        assert_eq!(report.seeds.len(), config.base.n_seeds);
    }
}
