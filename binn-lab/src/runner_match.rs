//! Matched-architecture C1 control runner (protocol v4).
//!
//! Paired arms share one dense-LIF forward ([`MatchedArch`]); only the learning
//! rule differs ([`MatchedLocal`] broadcast three-factor vs [`MatchedGradient`]
//! SuperSpike BPTT). Canonical C1 protocol v2 is never called or mutated.

use binn_learn::{
    MatchedGradient, MatchedLocal, DEFAULT_MATCHED_BETA, MATCHED_GRADIENT_LABEL,
    MATCHED_LOCAL_LABEL, REFERENCE_SEQUENCE_LEN,
};

use crate::match_config::{MatchConfig, C1_MATCH_CHANCE_BASELINE};
use crate::runner::{freeze_trials, mean_var, samples_to_gradient_examples, GateG2Verdict};

/// Per-seed paired accuracies and gap_closed_matched.
#[derive(Clone, Debug, PartialEq)]
pub struct MatchSeedResult {
    pub seed: u64,
    pub matched_local: f32,
    pub matched_gradient: f32,
    pub gap_closed_matched: f32,
}

/// Aggregated matched-architecture report.
#[derive(Clone, Debug, PartialEq)]
pub struct MatchReport {
    pub config_hash: String,
    pub protocol_version: u64,
    pub seeds: Vec<MatchSeedResult>,
    pub mean_matched_local: f32,
    pub variance_matched_local: f32,
    pub mean_matched_gradient: f32,
    pub variance_matched_gradient: f32,
    pub mean_gap_closed_matched: f32,
    pub variance_gap_closed_matched: f32,
    pub gap_closed_matched_lower_95: f32,
    pub verdict: GateG2Verdict,
    pub pilot: bool,
}

#[derive(Default)]
pub struct MatchRunner;

impl MatchRunner {
    pub fn new() -> Self {
        Self
    }

    /// Run paired MatchedLocal vs MatchedGradient on identical frozen splits.
    pub fn run(&mut self, config: &MatchConfig) -> MatchReport {
        assert!(config.base.n_seeds >= 1);
        assert!(config.base.bptt_epochs >= 1);
        assert_eq!(
            config.base.sequence_len, REFERENCE_SEQUENCE_LEN,
            "matched-arch requires sequence_len={REFERENCE_SEQUENCE_LEN}"
        );
        assert!(
            (config.chance_baseline - C1_MATCH_CHANCE_BASELINE).abs() < 1e-6,
            "chance baseline is locked at {C1_MATCH_CHANCE_BASELINE}"
        );

        let mut seeds = Vec::with_capacity(config.base.n_seeds);
        for seed in config.seeds() {
            seeds.push(run_seed(config, seed));
        }

        let summary = summarize(config, &seeds);
        let pilot = config.quick || config.base.n_seeds < config.scientific_n_seeds;
        MatchReport {
            config_hash: config.hash_string(),
            protocol_version: config.protocol_version,
            seeds,
            mean_matched_local: summary.mean_local,
            variance_matched_local: summary.var_local,
            mean_matched_gradient: summary.mean_gradient,
            variance_matched_gradient: summary.var_gradient,
            mean_gap_closed_matched: summary.mean_gap,
            variance_gap_closed_matched: summary.var_gap,
            gap_closed_matched_lower_95: summary.gap_lcb,
            verdict: summary.verdict,
            pilot,
        }
    }

    /// Render a self-contained results note.
    pub fn render_markdown(report: &MatchReport, config: &MatchConfig) -> String {
        let mut md = String::new();
        md.push_str("# BINN matched-architecture control (C1-MATCH)\n\n");
        if config.is_undertrain_protocol() {
            md.push_str(
                "**claim_axis:** Integrity\n\
                 **object_under_test:** Matched dense-LIF broadcast three-factor under 4× epochs\n\
                 **may_claim:** Under protocol 22, whether the v4 FAIL survives 4× training exposure\n\
                 **must_not_claim:** Remassage of `c1-match-5dc6822e71229e9e`; impossibility; biology\n\n",
            );
            md.push_str(&format!(
                "**Matched undertrain protocol:** `{}` — same matched dense-LIF + broadcast \
                 three-factor as v4, but local/gradient arms train for **{}×** epochs \
                 (`bptt_epochs={}`); does **not** remassage `c1-match-5dc6822e71229e9e` \
                 or reopen protocol-v2 `c1-118207fbc3eaba53`.\n\n",
                crate::match_config::C1_MATCH_UNDERTRAIN_PROTOCOL_VERSION,
                crate::match_config::C1_MATCH_UNDERTRAIN_EPOCH_MULT,
                config.base.bptt_epochs,
            ));
        } else {
            md.push_str(
                "**Does not reopen protocol-v2:** hash `c1-118207fbc3eaba53` and every \
                 G2 threshold remain unchanged. This is protocol **v4** with a fresh \
                 `c1-match-*` hash. Mechanism label: **broadcast three-factor on \
                 dense-LIF** (not a BINN substrate rescue).\n\n",
            );
        }
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

        // The forward graph, recorded so a number can be read without opening
        // the source. Two of these four suites have always run on the recurrent
        // graph and two on the feed-forward one, and no report said which.
        md.push_str(&format!(
            "- forward graph: **{}**\n\n",
            config.forward.label()
        ));
        md.push_str("## Results\n\n");
        md.push_str(&format!(
            "| arm | mean accuracy | variance |\n\
             |---|---:|---:|\n\
             | `{MATCHED_LOCAL_LABEL}` (broadcast three-factor) | {:.4} | {:.6} |\n\
             | `{MATCHED_GRADIENT_LABEL}` (SuperSpike BPTT ceiling) | {:.4} | {:.6} |\n\n",
            report.mean_matched_local,
            report.variance_matched_local,
            report.mean_matched_gradient,
            report.variance_matched_gradient,
        ));
        md.push_str(&format!(
            "- `gap_closed_matched` mean: **{:.4}**  (var {:.6})\n\
             - lower 95% CB (z={:.2}): **{:.4}**  (needs > {:.2})\n\
             - accuracy floor (matched-local ≥ {:.2}): {}\n\
             - harness validity (matched-gradient ≥ {:.2}): {}\n\
             - **verdict: {}**\n\n",
            report.mean_gap_closed_matched,
            report.variance_gap_closed_matched,
            config.base.g2_confidence_z,
            report.gap_closed_matched_lower_95,
            config.base.g2_min_gap_closed,
            config.base.g2_min_accuracy,
            if report.mean_matched_local >= config.base.g2_min_accuracy {
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
            "| seed | matched-local | matched-gradient | gap_closed_matched |\n\
             |---:|---:|---:|---:|\n",
        );
        for s in &report.seeds {
            md.push_str(&format!(
                "| {} | {:.4} | {:.4} | {:.4} |\n",
                s.seed, s.matched_local, s.matched_gradient, s.gap_closed_matched
            ));
        }

        md.push_str(
            "\n## Gate (unchanged thresholds)\n\n\
             `gap_closed_matched = (matched_local − 0.5) / (matched_gradient − 0.5)`, \
             clamped to [0,1]; seeds with `(matched_gradient − 0.5) < g2_min_reference_gap` \
             contribute `closed = 0`. PASS requires gap LCB > 0.5 and mean matched-local \
             ≥ 0.65; mean matched-gradient < 0.65 ⇒ INVALID_HARNESS.\n\n",
        );
        md.push_str(
            "## Reproduce\n\n\
             ```bash\n\
             cargo run --locked --release -p binn-lab --bin c1 -- --matched-arch --quick\n\
             cargo run --locked --release -p binn-lab --bin c1 -- --matched-arch --out results/c1_match.md\n\
             ```\n",
        );
        md
    }
}

struct SummaryParts {
    mean_local: f32,
    var_local: f32,
    mean_gradient: f32,
    var_gradient: f32,
    mean_gap: f32,
    var_gap: f32,
    gap_lcb: f32,
    verdict: GateG2Verdict,
}

fn run_seed(config: &MatchConfig, seed: u64) -> MatchSeedResult {
    let split = freeze_trials(&config.base, seed);
    let train = samples_to_gradient_examples(&split.train);
    let test = samples_to_gradient_examples(&split.test);
    let beta = if config.base.surrogate_beta > 0.0 {
        config.base.surrogate_beta
    } else {
        DEFAULT_MATCHED_BETA
    };
    let epochs = config.base.bptt_epochs;

    let mut gradient = MatchedGradient::on(
        config.forward,
        config.base.n_hidden,
        config.base.bptt_lr,
        beta,
        seed,
    );
    let grad_report = gradient.train_and_evaluate(epochs, &train, &test);

    let mut local = MatchedLocal::on(
        config.forward,
        config.base.n_hidden,
        config.base.eta,
        config.base.lambda,
        beta,
        seed,
    );
    let local_report = local.train_and_evaluate(epochs, &train, &test);

    let gap = gap_closed_matched(
        local_report.accuracy,
        grad_report.accuracy,
        config.chance_baseline,
        config.base.g2_min_reference_gap,
    );
    MatchSeedResult {
        seed,
        matched_local: local_report.accuracy,
        matched_gradient: grad_report.accuracy,
        gap_closed_matched: gap,
    }
}

/// Preregistered gap metric with chance baseline 0.5.
pub fn gap_closed_matched(
    matched_local: f32,
    matched_gradient: f32,
    chance_baseline: f32,
    min_reference_gap: f32,
) -> f32 {
    let reference_gap = matched_gradient - chance_baseline;
    if reference_gap < min_reference_gap {
        0.0
    } else {
        ((matched_local - chance_baseline) / reference_gap).clamp(0.0, 1.0)
    }
}

fn summarize(config: &MatchConfig, seeds: &[MatchSeedResult]) -> SummaryParts {
    let local: Vec<f32> = seeds.iter().map(|s| s.matched_local).collect();
    let gradient: Vec<f32> = seeds.iter().map(|s| s.matched_gradient).collect();
    let gaps: Vec<f32> = seeds.iter().map(|s| s.gap_closed_matched).collect();
    let (mean_local, var_local) = mean_var(&local);
    let (mean_gradient, var_gradient) = mean_var(&gradient);
    let (mean_gap, var_gap) = mean_var(&gaps);
    let n = gaps.len();
    let gap_lcb = if n > 1 {
        mean_gap - config.base.g2_confidence_z * (var_gap / n as f32).sqrt()
    } else {
        mean_gap
    };

    let verdict = decide_verdict(config, mean_local, mean_gradient, gap_lcb);
    SummaryParts {
        mean_local,
        var_local,
        mean_gradient,
        var_gradient,
        mean_gap,
        var_gap,
        gap_lcb,
        verdict,
    }
}

/// Delegates to the single owner in [`crate::guards::decide_matched_verdict`].
///
/// This was one of four byte-identical copies, and all four shared the same
/// hole: they checked that the reference was not too *weak* and never that the
/// treatment had exceeded it. See that function for what the hole let through.
fn decide_verdict(
    config: &MatchConfig,
    mean_local: f32,
    mean_gradient: f32,
    gap_lcb: f32,
) -> GateG2Verdict {
    crate::guards::decide_matched_verdict(
        mean_gradient,
        mean_local,
        gap_lcb,
        config.chance_baseline,
        config.base.g2_min_accuracy,
        config.base.g2_min_gap_closed,
        config.quick || config.base.n_seeds < config.scientific_n_seeds,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::match_config::C1_MATCH_PROTOCOL_VERSION;

    #[test]
    fn gap_math_clamps_and_guards_weak_denominator() {
        // Strong local → closed = 1
        assert!((gap_closed_matched(0.9, 0.9, 0.5, 0.15) - 1.0).abs() < 1e-6);
        // Chance local → closed = 0
        assert!((gap_closed_matched(0.5, 0.8, 0.5, 0.15) - 0.0).abs() < 1e-6);
        // Midway
        assert!((gap_closed_matched(0.65, 0.8, 0.5, 0.15) - 0.5).abs() < 1e-6);
        // Below chance local clamps to 0
        assert_eq!(gap_closed_matched(0.4, 0.8, 0.5, 0.15), 0.0);
        // Weak reference gap → 0
        assert_eq!(gap_closed_matched(0.9, 0.55, 0.5, 0.15), 0.0);
        // Overshoot clamps to 1
        assert_eq!(gap_closed_matched(0.95, 0.8, 0.5, 0.15), 1.0);
    }

    #[test]
    fn invalid_harness_when_ceiling_below_floor() {
        let config = MatchConfig::scientific();
        let verdict = decide_verdict(&config, 0.9, 0.50, 0.9);
        assert_eq!(verdict, GateG2Verdict::InvalidHarness);
    }

    #[test]
    fn pass_requires_lcb_and_floor() {
        let config = MatchConfig::scientific();
        assert_eq!(
            decide_verdict(&config, 0.70, 0.80, 0.55),
            GateG2Verdict::Pass
        );
        assert_eq!(
            decide_verdict(&config, 0.70, 0.80, 0.40),
            GateG2Verdict::Fail
        );
        assert_eq!(
            decide_verdict(&config, 0.60, 0.80, 0.70),
            GateG2Verdict::Fail
        );
    }

    #[test]
    fn quick_is_always_pilot() {
        let config = MatchConfig::quick();
        assert_eq!(
            decide_verdict(&config, 0.90, 0.90, 0.90),
            GateG2Verdict::Pilot
        );
    }

    #[test]
    fn paired_arms_are_deterministic_under_same_seed() {
        let mut config = MatchConfig::quick();
        config.base.n_seeds = 1;
        config.base.n_train = 12;
        config.base.n_test = 8;
        config.base.bptt_epochs = 4;
        config.base.n_hidden = 16;
        let seed = config.seeds()[0];
        let a = run_seed(&config, seed);
        let b = run_seed(&config, seed);
        assert_eq!(a, b);
        assert_eq!(a.seed, seed);
    }

    #[test]
    fn protocol_is_v4() {
        assert_eq!(C1_MATCH_PROTOCOL_VERSION, 4);
        assert_eq!(MatchConfig::scientific().protocol_version, 4);
    }
}
