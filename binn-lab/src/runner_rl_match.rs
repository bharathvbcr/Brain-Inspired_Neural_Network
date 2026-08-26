//! Matched-architecture in-family RL recipe runner (protocol v12).
//!
//! Paired arms share one dense-LIF feed-forward ([`MatchedArch`]); learning rule
//! is the only variable. Primary gated arm: [`MatchedRlReinforceFb`]
//! (REINFORCE × fixed-random feedback). Contrasts: [`MatchedRlGraded`]
//! (graded correctness − baseline, broadcast; v11 primary, now ungated),
//! [`MatchedRlFlat`] (±1 broadcast), ceiling [`MatchedGradient`].
//!
//! Does not reopen protocol-v2, mutate `c1-dfa-*`, remassage spiking DFA, or
//! retune failed v11 `rl_graded` knobs.

use binn_learn::{
    MatchedGradient, MatchedRlFlat, MatchedRlGraded, MatchedRlReinforceFb, DEFAULT_MATCHED_BETA,
    MATCHED_GRADIENT_LABEL, MATCHED_RL_FLAT_LABEL, MATCHED_RL_GRADED_LABEL,
    MATCHED_RL_REINFORCE_FB_LABEL, REFERENCE_SEQUENCE_LEN,
};

use crate::rl_match_config::{RlMatchConfig, C1_RL_CHANCE_BASELINE, C1_RL_PRIMARY_ARM};
use crate::runner::{freeze_trials, mean_var, samples_to_gradient_examples, GateG2Verdict};
use crate::runner_match::gap_closed_matched;

/// Per-seed accuracies and gap_closed_rl (reinforce_fb vs gradient).
#[derive(Clone, Debug, PartialEq)]
pub struct RlMatchSeedResult {
    pub seed: u64,
    pub matched_rl_graded: f32,
    pub matched_rl_reinforce_fb: f32,
    pub matched_rl_flat: f32,
    pub matched_gradient: f32,
    pub gap_closed_rl: f32,
}

/// Aggregated matched-RL report.
#[derive(Clone, Debug, PartialEq)]
pub struct RlMatchReport {
    pub config_hash: String,
    pub protocol_version: u64,
    pub seeds: Vec<RlMatchSeedResult>,
    pub mean_matched_rl_graded: f32,
    pub variance_matched_rl_graded: f32,
    pub mean_matched_rl_reinforce_fb: f32,
    pub variance_matched_rl_reinforce_fb: f32,
    pub mean_matched_rl_flat: f32,
    pub variance_matched_rl_flat: f32,
    pub mean_matched_gradient: f32,
    pub variance_matched_gradient: f32,
    pub mean_gap_closed_rl: f32,
    pub variance_gap_closed_rl: f32,
    pub gap_closed_rl_lower_95: f32,
    pub verdict: GateG2Verdict,
    pub pilot: bool,
}

#[derive(Default)]
pub struct RlMatchRunner;

impl RlMatchRunner {
    pub fn new() -> Self {
        Self
    }

    /// Run reinforce-fb + graded + flat + gradient on identical frozen splits.
    pub fn run(&mut self, config: &RlMatchConfig) -> RlMatchReport {
        assert!(config.base.n_seeds >= 1);
        assert!(config.base.bptt_epochs >= 1);
        assert_eq!(
            config.base.sequence_len, REFERENCE_SEQUENCE_LEN,
            "matched-rl requires sequence_len={REFERENCE_SEQUENCE_LEN}"
        );
        assert!(
            (config.chance_baseline - C1_RL_CHANCE_BASELINE).abs() < 1e-6,
            "chance baseline is locked at {C1_RL_CHANCE_BASELINE}"
        );
        assert_eq!(
            config.primary_arm, C1_RL_PRIMARY_ARM,
            "protocol v12 primary arm is locked at {C1_RL_PRIMARY_ARM}"
        );

        let mut seeds = Vec::with_capacity(config.base.n_seeds);
        for seed in config.seeds() {
            seeds.push(run_seed(config, seed));
        }

        let summary = summarize(config, &seeds);
        let pilot = config.quick || config.base.n_seeds < config.scientific_n_seeds;
        RlMatchReport {
            config_hash: config.hash_string(),
            protocol_version: config.protocol_version,
            seeds,
            mean_matched_rl_graded: summary.mean_graded,
            variance_matched_rl_graded: summary.var_graded,
            mean_matched_rl_reinforce_fb: summary.mean_fb,
            variance_matched_rl_reinforce_fb: summary.var_fb,
            mean_matched_rl_flat: summary.mean_flat,
            variance_matched_rl_flat: summary.var_flat,
            mean_matched_gradient: summary.mean_gradient,
            variance_matched_gradient: summary.var_gradient,
            mean_gap_closed_rl: summary.mean_gap,
            variance_gap_closed_rl: summary.var_gap,
            gap_closed_rl_lower_95: summary.gap_lcb,
            verdict: summary.verdict,
            pilot,
        }
    }

    /// Render a self-contained results note.
    pub fn render_markdown(report: &RlMatchReport, config: &RlMatchConfig) -> String {
        let mut md = String::new();
        md.push_str("# BINN matched-architecture in-family RL recipe (C1-RL)\n\n");
        md.push_str(
            "**Does not reopen protocol-v2:** hash `c1-118207fbc3eaba53` and every \
             G2 threshold remain unchanged. This is matched-arch protocol **v12** \
             with a fresh `c1-rl-*` hash. Mechanism: **REINFORCE × fixed-random \
             feedback** (`rl_reinforce_fb`) as the **primary** gated arm on the \
             dense-LIF matched forward (feed-forward `wrec=0`). Graded / flat remain \
             contrasts. Does **not** retune failed v11 `rl_graded` \
             (`c1-rl-ef504db58916720d`), mutate `c1-dfa-*`, or remassage \
             `c1x-dfa-spike-*`.\n\n",
        );
        md.push_str(&format!(
            "- schedule: **{}**\n\
             - config hash: `{}`\n\
             - protocol version: {}\n\
             - primary arm: `{}`\n\
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
            config.primary_arm,
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
             | `{MATCHED_RL_REINFORCE_FB_LABEL}` (REINFORCE × DFA fb) **primary** | {:.4} | {:.6} |\n\
             | `{MATCHED_RL_GRADED_LABEL}` (graded reward, broadcast) contrast | {:.4} | {:.6} |\n\
             | `{MATCHED_RL_FLAT_LABEL}` (±1 reward, broadcast) contrast | {:.4} | {:.6} |\n\
             | `{MATCHED_GRADIENT_LABEL}` (SuperSpike BPTT ceiling) | {:.4} | {:.6} |\n\n",
            report.mean_matched_rl_reinforce_fb,
            report.variance_matched_rl_reinforce_fb,
            report.mean_matched_rl_graded,
            report.variance_matched_rl_graded,
            report.mean_matched_rl_flat,
            report.variance_matched_rl_flat,
            report.mean_matched_gradient,
            report.variance_matched_gradient,
        ));
        md.push_str(&format!(
            "- `gap_closed_rl` mean: **{:.4}**  (var {:.6})\n\
             - lower 95% CB (z={:.2}): **{:.4}**  (needs > {:.2})\n\
             - accuracy floor (matched-rl-reinforce-fb ≥ {:.2}): {}\n\
             - harness validity (matched-gradient ≥ {:.2}): {}\n\
             - **verdict: {}**\n\n",
            report.mean_gap_closed_rl,
            report.variance_gap_closed_rl,
            config.base.g2_confidence_z,
            report.gap_closed_rl_lower_95,
            config.base.g2_min_gap_closed,
            config.base.g2_min_accuracy,
            if report.mean_matched_rl_reinforce_fb >= config.base.g2_min_accuracy {
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
            "| seed | rl_reinforce_fb | rl_graded | rl_flat | gradient | gap_closed_rl |\n\
             |---:|---:|---:|---:|---:|---:|\n",
        );
        for s in &report.seeds {
            md.push_str(&format!(
                "| {} | {:.4} | {:.4} | {:.4} | {:.4} | {:.4} |\n",
                s.seed,
                s.matched_rl_reinforce_fb,
                s.matched_rl_graded,
                s.matched_rl_flat,
                s.matched_gradient,
                s.gap_closed_rl
            ));
        }

        md.push_str(
            "\n## Gate (unchanged thresholds)\n\n\
             Primary arm = `rl_reinforce_fb`. `gap_closed_rl = (matched_rl_reinforce_fb − 0.5) / \
             (matched_gradient − 0.5)`, clamped to [0,1]; seeds with \
             `(matched_gradient − 0.5) < g2_min_reference_gap` contribute \
             `closed = 0`. PASS requires gap LCB > 0.5 and mean matched-rl-reinforce-fb \
             ≥ 0.65; mean matched-gradient < 0.65 ⇒ INVALID_HARNESS.\n\n",
        );
        md.push_str(
            "## Recipe notes\n\n\
             - Readout always uses REINFORCE `r·(a−p)` (Bernoulli policy).\n\
             - Hidden `rl_reinforce_fb` (**primary**): frozen `B_i ∈ [-1,1]` × `r·(a−p)`.\n\
             - Hidden `rl_graded` (contrast; v11 primary): broadcast \
               `(p_correct − baseline)` with EMA baseline.\n\
             - Hidden `rl_flat` (contrast): broadcast ±1 reward (production impoverishment).\n\
             - Supervised DFA (`c1-dfa-*`) remains a separate protocol; this suite \
               asks whether an **RL** modulator can close the gap in-family.\n\
             - v11 graded-primary FAIL is archived at `c1-rl-ef504db58916720d` — \
               not retuned here.\n\n",
        );
        md.push_str(
            "## Reproduce\n\n\
             ```bash\n\
             cargo run --locked --release -p binn-lab --bin c1 -- --matched-rl --quick\n\
             cargo run --locked --release -p binn-lab --bin c1 -- --matched-rl --out results/c1_rl.md\n\
             ```\n",
        );
        md
    }
}

struct SummaryParts {
    mean_graded: f32,
    var_graded: f32,
    mean_fb: f32,
    var_fb: f32,
    mean_flat: f32,
    var_flat: f32,
    mean_gradient: f32,
    var_gradient: f32,
    mean_gap: f32,
    var_gap: f32,
    gap_lcb: f32,
    verdict: GateG2Verdict,
}

fn run_seed(config: &RlMatchConfig, seed: u64) -> RlMatchSeedResult {
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

    let mut graded = MatchedRlGraded::on(
        config.forward,
        config.base.n_hidden,
        config.base.eta,
        config.base.lambda,
        beta,
        seed,
    );
    let graded_report = graded.train_and_evaluate(epochs, &train, &test);

    let mut fb = MatchedRlReinforceFb::on(
        config.forward,
        config.base.n_hidden,
        config.base.eta,
        config.base.lambda,
        beta,
        seed,
    );
    let fb_report = fb.train_and_evaluate(epochs, &train, &test);

    let mut flat = MatchedRlFlat::on(
        config.forward,
        config.base.n_hidden,
        config.base.eta,
        config.base.lambda,
        beta,
        seed,
    );
    let flat_report = flat.train_and_evaluate(epochs, &train, &test);

    // Primary gap uses reinforce_fb (v12), not graded (v11).
    let gap = gap_closed_matched(
        fb_report.accuracy,
        grad_report.accuracy,
        config.chance_baseline,
        config.base.g2_min_reference_gap,
    );
    RlMatchSeedResult {
        seed,
        matched_rl_graded: graded_report.accuracy,
        matched_rl_reinforce_fb: fb_report.accuracy,
        matched_rl_flat: flat_report.accuracy,
        matched_gradient: grad_report.accuracy,
        gap_closed_rl: gap,
    }
}

fn summarize(config: &RlMatchConfig, seeds: &[RlMatchSeedResult]) -> SummaryParts {
    let graded: Vec<f32> = seeds.iter().map(|s| s.matched_rl_graded).collect();
    let fb: Vec<f32> = seeds.iter().map(|s| s.matched_rl_reinforce_fb).collect();
    let flat: Vec<f32> = seeds.iter().map(|s| s.matched_rl_flat).collect();
    let gradient: Vec<f32> = seeds.iter().map(|s| s.matched_gradient).collect();
    let gaps: Vec<f32> = seeds.iter().map(|s| s.gap_closed_rl).collect();
    let (mean_graded, var_graded) = mean_var(&graded);
    let (mean_fb, var_fb) = mean_var(&fb);
    let (mean_flat, var_flat) = mean_var(&flat);
    let (mean_gradient, var_gradient) = mean_var(&gradient);
    let (mean_gap, var_gap) = mean_var(&gaps);
    let n = gaps.len();
    let gap_lcb = if n > 1 {
        mean_gap - config.base.g2_confidence_z * (var_gap / n as f32).sqrt()
    } else {
        mean_gap
    };

    let verdict = decide_verdict(config, mean_fb, mean_gradient, gap_lcb);
    SummaryParts {
        mean_graded,
        var_graded,
        mean_fb,
        var_fb,
        mean_flat,
        var_flat,
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
    config: &RlMatchConfig,
    mean_primary: f32,
    mean_gradient: f32,
    gap_lcb: f32,
) -> GateG2Verdict {
    crate::guards::decide_matched_verdict(
        mean_gradient,
        mean_primary,
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
    use crate::rl_match_config::C1_RL_PROTOCOL_VERSION;

    #[test]
    fn protocol_is_v12() {
        assert_eq!(C1_RL_PROTOCOL_VERSION, 12);
        assert_eq!(RlMatchConfig::scientific().protocol_version, 12);
        assert_eq!(RlMatchConfig::scientific().primary_arm, "rl_reinforce_fb");
    }

    #[test]
    fn quick_run_is_finite_and_pilot() {
        let config = RlMatchConfig::quick();
        let mut runner = RlMatchRunner::new();
        let report = runner.run(&config);
        assert!(report.pilot);
        assert_eq!(report.verdict, GateG2Verdict::Pilot);
        assert!(report.mean_matched_rl_graded.is_finite());
        assert!(report.mean_matched_rl_reinforce_fb.is_finite());
        assert!(report.mean_matched_rl_flat.is_finite());
        assert!(report.mean_matched_gradient.is_finite());
        assert_eq!(report.seeds.len(), config.base.n_seeds);
    }
}
