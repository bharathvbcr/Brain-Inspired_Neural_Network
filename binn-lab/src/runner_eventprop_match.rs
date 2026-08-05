//! Matched-architecture EventProp H2H runner (protocol v28).
//!
//! Paired arms share one recurrent dense-LIF forward ([`MatchedArch`]); learning
//! rule is the only variable. Primary arm: [`MatchedEventProp`] (discrete
//! spike-triggered adjoint). Ceiling: [`MatchedGradient`] (SuperSpike BPTT).
//!
//! Does not reopen protocol-v2 or mutate `c1-match-*` / `c1-dfa-*` / `c1-rl-*`.

use binn_data::Sample;
use binn_learn::{
    GradientExample, MatchedEventProp, MatchedGradient, DEFAULT_MATCHED_BETA,
    MATCHED_EVENTPROP_LABEL, MATCHED_GRADIENT_LABEL, REFERENCE_SEQUENCE_LEN,
};

use crate::eventprop_match_config::{EventPropMatchConfig, C1_EVENTPROP_CHANCE_BASELINE};
use crate::runner::{freeze_trials, GateG2Verdict};
use crate::runner_match::gap_closed_matched;

/// Per-seed accuracies and gap_closed_eventprop.
#[derive(Clone, Debug, PartialEq)]
pub struct EventPropMatchSeedResult {
    pub seed: u64,
    pub matched_eventprop: f32,
    pub matched_gradient: f32,
    pub gap_closed_eventprop: f32,
}

/// Aggregated matched-EventProp report.
#[derive(Clone, Debug, PartialEq)]
pub struct EventPropMatchReport {
    pub config_hash: String,
    pub protocol_version: u64,
    pub seeds: Vec<EventPropMatchSeedResult>,
    pub mean_matched_eventprop: f32,
    pub variance_matched_eventprop: f32,
    pub mean_matched_gradient: f32,
    pub variance_matched_gradient: f32,
    pub mean_gap_closed_eventprop: f32,
    pub variance_gap_closed_eventprop: f32,
    pub gap_closed_eventprop_lower_95: f32,
    pub verdict: GateG2Verdict,
    pub pilot: bool,
}

#[derive(Default)]
pub struct EventPropMatchRunner;

impl EventPropMatchRunner {
    pub fn new() -> Self {
        Self
    }

    /// Run EventProp + SuperSpike on identical frozen splits.
    pub fn run(&mut self, config: &EventPropMatchConfig) -> EventPropMatchReport {
        assert!(config.base.n_seeds >= 1);
        assert!(config.base.bptt_epochs >= 1);
        assert_eq!(
            config.base.sequence_len, REFERENCE_SEQUENCE_LEN,
            "matched-eventprop requires sequence_len={REFERENCE_SEQUENCE_LEN}"
        );
        assert!(
            (config.chance_baseline - C1_EVENTPROP_CHANCE_BASELINE).abs() < 1e-6,
            "chance baseline is locked at {C1_EVENTPROP_CHANCE_BASELINE}"
        );

        let mut seeds = Vec::with_capacity(config.base.n_seeds);
        for seed in config.seeds() {
            seeds.push(run_seed(config, seed));
        }

        let summary = summarize(config, &seeds);
        let pilot = config.quick || config.base.n_seeds < config.scientific_n_seeds;
        EventPropMatchReport {
            config_hash: config.hash_string(),
            protocol_version: config.protocol_version,
            seeds,
            mean_matched_eventprop: summary.mean_eventprop,
            variance_matched_eventprop: summary.var_eventprop,
            mean_matched_gradient: summary.mean_gradient,
            variance_matched_gradient: summary.var_gradient,
            mean_gap_closed_eventprop: summary.mean_gap,
            variance_gap_closed_eventprop: summary.var_gap,
            gap_closed_eventprop_lower_95: summary.gap_lcb,
            verdict: summary.verdict,
            pilot,
        }
    }

    /// Render a self-contained results note.
    pub fn render_markdown(report: &EventPropMatchReport, config: &EventPropMatchConfig) -> String {
        let mut md = String::new();
        md.push_str("# BINN matched-architecture EventProp H2H (C1-EVENTPROP)\n\n");
        md.push_str(
            "**Does not reopen protocol-v2:** hash `c1-118207fbc3eaba53` and every \
             G2 threshold remain unchanged. Does **not** mutate frozen \
             `c1-match-5dc6822e71229e9e`, `c1-dfa-*`, or `c1-rl-*`. This is \
             protocol **v28** with a fresh `c1-eventprop-*` hash. Mechanism: \
             **discrete EventProp-style spike-triggered adjoint** vs SuperSpike \
             BPTT on the recurrent dense-LIF matched coincidence forward \
             (rule-only; same architecture).\n\n",
        );
        md.push_str(&format!(
            "- schedule: **{}**\n\
             - config hash: `{}`\n\
             - protocol version: {}\n\
             - seeds: {}\n\
             - train/test: {}/{}\n\
             - hidden / epochs / β: {} / {} / {:.1}\n\
             - gradient lr (shared): {:.4}\n\
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
            config.chance_baseline,
        ));

        md.push_str("## Results\n\n");
        md.push_str(&format!(
            "| arm | mean accuracy | variance |\n\
             |---|---:|---:|\n\
             | `{MATCHED_EVENTPROP_LABEL}` (EventProp-style spike adjoint) | {:.4} | {:.6} |\n\
             | `{MATCHED_GRADIENT_LABEL}` (SuperSpike BPTT ceiling) | {:.4} | {:.6} |\n\n",
            report.mean_matched_eventprop,
            report.variance_matched_eventprop,
            report.mean_matched_gradient,
            report.variance_matched_gradient,
        ));
        md.push_str(&format!(
            "- `gap_closed_eventprop` mean: **{:.4}**  (var {:.6})\n\
             - lower 95% CB (z={:.2}): **{:.4}**  (needs > {:.2})\n\
             - accuracy floor (matched-eventprop ≥ {:.2}): {}\n\
             - harness validity (matched-gradient ≥ {:.2}): {}\n\
             - **verdict: {}**\n\n",
            report.mean_gap_closed_eventprop,
            report.variance_gap_closed_eventprop,
            config.base.g2_confidence_z,
            report.gap_closed_eventprop_lower_95,
            config.base.g2_min_gap_closed,
            config.base.g2_min_accuracy,
            if report.mean_matched_eventprop >= config.base.g2_min_accuracy {
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
            "| seed | eventprop | gradient | gap_closed_eventprop |\n\
             |---:|---:|---:|---:|\n",
        );
        for s in &report.seeds {
            md.push_str(&format!(
                "| {} | {:.4} | {:.4} | {:.4} |\n",
                s.seed, s.matched_eventprop, s.matched_gradient, s.gap_closed_eventprop
            ));
        }

        md.push_str(
            "\n## Gate (unchanged thresholds)\n\n\
             Primary arm = EventProp. `gap_closed_eventprop = (matched_eventprop − 0.5) / \
             (matched_gradient − 0.5)`, clamped to [0,1]; seeds with \
             `(matched_gradient − 0.5) < g2_min_reference_gap` contribute \
             `closed = 0`. PASS requires gap LCB > 0.5 and mean matched-eventprop \
             ≥ 0.65; mean matched-gradient < 0.65 ⇒ INVALID_HARNESS.\n\n",
        );
        md.push_str(
            "## Honesty / approximations\n\n\
             - **Not neuromorphic HW.** CPU discrete-time simulation only.\n\
             - **Not textbook continuous EventProp.** Wunderlich & Pehle (2021) \
               adjoint is for hybrid continuous/discrete dynamics with exact jump \
               conditions. Here the forward is the same discrete hard-threshold \
               LIF as SuperSpike matched-arch; the reverse uses a **hard spike \
               gate** (adjoint jump only when `s_i[t]=1`, jump scale \
               `min(1/max(|I_eff|,ε), JUMP_MAX)` as a discrete stand-in for \
               continuous `1/|du/dt|`) — **not** SuperSpike’s soft `σ'(u)` at \
               every timestep.\n\
             - **Cold-start disclosure:** hard spike gating provides **no** \
               hidden-weight gradient on silent timesteps. Under the shared \
               matched init, networks that rarely spike cannot bootstrap the way \
               SuperSpike’s soft surrogate does; exact chance (0.5) is the \
               expected symptom when only the readout bias moves.\n\
             - **Rule-only H2H:** identical recurrent `MatchedArch` forward, \
               splits, seeds lineage, epochs, and lr as the SuperSpike ceiling; \
               only the backward credit rule differs.\n\
             - **Not a production-learner claim** (GC1-exempt `*_baseline.rs`).\n\n",
        );
        md.push_str(
            "## Reproduce\n\n\
             ```bash\n\
             cargo run --locked --release -p binn-lab --bin c1 -- --eventprop --quick\n\
             cargo run --locked --release -p binn-lab --bin c1 -- --eventprop --out results/c1_eventprop.md\n\
             ```\n",
        );
        md
    }
}

struct SummaryParts {
    mean_eventprop: f32,
    var_eventprop: f32,
    mean_gradient: f32,
    var_gradient: f32,
    mean_gap: f32,
    var_gap: f32,
    gap_lcb: f32,
    verdict: GateG2Verdict,
}

fn run_seed(config: &EventPropMatchConfig, seed: u64) -> EventPropMatchSeedResult {
    let split = freeze_trials(&config.base, seed);
    let train = samples_to_gradient_examples(&split.train);
    let test = samples_to_gradient_examples(&split.test);
    let beta = if config.base.surrogate_beta > 0.0 {
        config.base.surrogate_beta
    } else {
        DEFAULT_MATCHED_BETA
    };
    let epochs = config.base.bptt_epochs;

    let mut gradient = MatchedGradient::new(config.base.n_hidden, config.base.bptt_lr, beta, seed);
    let grad_report = gradient.train_and_evaluate(epochs, &train, &test);

    let mut eventprop =
        MatchedEventProp::new(config.base.n_hidden, config.base.bptt_lr, beta, seed);
    let ep_report = eventprop.train_and_evaluate(epochs, &train, &test);

    let gap = gap_closed_matched(
        ep_report.accuracy,
        grad_report.accuracy,
        config.chance_baseline,
        config.base.g2_min_reference_gap,
    );
    EventPropMatchSeedResult {
        seed,
        matched_eventprop: ep_report.accuracy,
        matched_gradient: grad_report.accuracy,
        gap_closed_eventprop: gap,
    }
}

fn summarize(config: &EventPropMatchConfig, seeds: &[EventPropMatchSeedResult]) -> SummaryParts {
    let eventprop: Vec<f32> = seeds.iter().map(|s| s.matched_eventprop).collect();
    let gradient: Vec<f32> = seeds.iter().map(|s| s.matched_gradient).collect();
    let gaps: Vec<f32> = seeds.iter().map(|s| s.gap_closed_eventprop).collect();
    let (mean_eventprop, var_eventprop) = mean_var(&eventprop);
    let (mean_gradient, var_gradient) = mean_var(&gradient);
    let (mean_gap, var_gap) = mean_var(&gaps);
    let n = gaps.len();
    let gap_lcb = if n > 1 {
        mean_gap - config.base.g2_confidence_z * (var_gap / n as f32).sqrt()
    } else {
        mean_gap
    };

    let verdict = decide_verdict(config, mean_eventprop, mean_gradient, gap_lcb);
    SummaryParts {
        mean_eventprop,
        var_eventprop,
        mean_gradient,
        var_gradient,
        mean_gap,
        var_gap,
        gap_lcb,
        verdict,
    }
}

fn decide_verdict(
    config: &EventPropMatchConfig,
    mean_eventprop: f32,
    mean_gradient: f32,
    gap_lcb: f32,
) -> GateG2Verdict {
    if mean_gradient < config.base.g2_min_accuracy {
        return GateG2Verdict::InvalidHarness;
    }
    if config.quick || config.base.n_seeds < config.scientific_n_seeds {
        return GateG2Verdict::Pilot;
    }
    if gap_lcb > config.base.g2_min_gap_closed && mean_eventprop >= config.base.g2_min_accuracy {
        GateG2Verdict::Pass
    } else {
        GateG2Verdict::Fail
    }
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

fn mean(values: &[f32]) -> f32 {
    if values.is_empty() {
        0.0
    } else {
        values.iter().sum::<f32>() / values.len() as f32
    }
}

fn mean_var(values: &[f32]) -> (f32, f32) {
    let mean = mean(values);
    if values.len() < 2 {
        return (mean, 0.0);
    }
    let variance = values
        .iter()
        .map(|v| {
            let d = *v - mean;
            d * d
        })
        .sum::<f32>()
        / (values.len() - 1) as f32;
    (mean, variance)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eventprop_match_config::C1_EVENTPROP_PROTOCOL_VERSION;

    #[test]
    fn protocol_is_v28() {
        assert_eq!(C1_EVENTPROP_PROTOCOL_VERSION, 28);
        assert_eq!(EventPropMatchConfig::scientific().protocol_version, 28);
    }

    #[test]
    fn quick_run_is_finite_and_pilot() {
        let config = EventPropMatchConfig::quick();
        let mut runner = EventPropMatchRunner::new();
        let report = runner.run(&config);
        assert!(report.pilot);
        assert_eq!(report.verdict, GateG2Verdict::Pilot);
        assert!(report.mean_matched_eventprop.is_finite());
        assert!(report.mean_matched_gradient.is_finite());
        assert_eq!(report.seeds.len(), config.base.n_seeds);
    }
}
