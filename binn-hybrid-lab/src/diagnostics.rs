//! Development-only robustness and mechanism diagnostics.
//!
//! This protocol is intentionally separate from frozen H0 protocol v3. It may
//! explain a negative result, but it cannot reverse or pass the scientific gate.

use std::collections::BTreeMap;

use binn_hybrid_learn::fnv1a64;

use crate::benchmark::{c3_accuracy, c3_examples, seeds, C3CompositionModel, C3Example};
use crate::factorization::factorization_audit;

pub const DIAGNOSTIC_PROTOCOL_VERSION: u32 = 3;
const DIAGNOSTIC_SEED_MASTER: u64 = 0x4842_4449_4147_0001;
const TEST_SEED_XOR: u64 = 0x5445_5354_4449_4147;
const C3_ACCURACY_FLOOR: f32 = 0.65;
const CONFIDENCE_Z: f32 = 1.96;

#[derive(Clone, Debug, PartialEq)]
pub struct DiagnosticConfig {
    pub quick: bool,
    pub n_seeds: usize,
    pub budgets: Vec<usize>,
    pub learning_rates: Vec<f32>,
    pub test_examples: usize,
    pub mechanism_examples: usize,
}

impl DiagnosticConfig {
    pub fn quick() -> Self {
        Self {
            quick: true,
            n_seeds: 3,
            budgets: vec![60, 240],
            learning_rates: vec![0.015, 0.035],
            test_examples: 160,
            mechanism_examples: 16,
        }
    }

    pub fn full() -> Self {
        Self {
            quick: false,
            n_seeds: 20,
            budgets: vec![120, 480, 1_920, 7_680],
            learning_rates: vec![0.002, 0.005, 0.015, 0.035, 0.070],
            test_examples: 1_000,
            mechanism_examples: 128,
        }
    }

    pub fn hash(&self) -> u64 {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&DIAGNOSTIC_PROTOCOL_VERSION.to_le_bytes());
        bytes.push(u8::from(self.quick));
        bytes.extend_from_slice(&(self.n_seeds as u64).to_le_bytes());
        bytes.extend_from_slice(&(self.test_examples as u64).to_le_bytes());
        bytes.extend_from_slice(&(self.mechanism_examples as u64).to_le_bytes());
        for &budget in &self.budgets {
            bytes.extend_from_slice(&(budget as u64).to_le_bytes());
        }
        bytes.push(0xff);
        for &rate in &self.learning_rates {
            bytes.extend_from_slice(&rate.to_bits().to_le_bytes());
        }
        fnv1a64(&bytes)
    }

    pub fn hash_string(&self) -> String {
        format!(
            "binn-hybrid-diagnostic-v{DIAGNOSTIC_PROTOCOL_VERSION}-{:016x}",
            self.hash()
        )
    }

    pub fn diagnostic_seeds(&self) -> Vec<u64> {
        seeds(DIAGNOSTIC_SEED_MASTER, self.n_seeds)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum DiagnosticArm {
    ExistingPostSynaptic,
    OraclePostSynaptic,
    DirectTerminal,
    PrivilegedIntermediate,
    ShuffledLabel,
}

impl DiagnosticArm {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ExistingPostSynaptic => "existing-post-synaptic",
            Self::OraclePostSynaptic => "least-squares-post-synaptic",
            Self::DirectTerminal => "direct-terminal",
            Self::PrivilegedIntermediate => "privileged-intermediate-target",
            Self::ShuffledLabel => "shuffled-label",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SweepRow {
    pub seed: u64,
    pub depth: usize,
    pub budget: usize,
    pub learning_rate: f32,
    pub arm: DiagnosticArm,
    pub accuracy: f32,
    pub test_weights_unchanged: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SweepSummary {
    pub depth: usize,
    pub budget: usize,
    pub learning_rate: f32,
    pub arm: DiagnosticArm,
    pub mean_accuracy: f32,
    pub variance: f32,
    pub lower_95: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MechanismRow {
    pub seed: u64,
    pub depth: usize,
    pub gradient_norm: f32,
    pub direct_loss_drop: f32,
    pub shuffled_loss_drop: f32,
    pub existing_cosine: f32,
    pub oracle_cosine: f32,
    pub existing_sign_agreement: f32,
    pub oracle_sign_agreement: f32,
}

#[derive(Clone, Debug)]
pub struct DiagnosticReport {
    pub protocol_hash: String,
    pub config: DiagnosticConfig,
    pub seeds: Vec<u64>,
    pub rows: Vec<SweepRow>,
    pub summaries: Vec<SweepSummary>,
    pub mechanisms: Vec<MechanismRow>,
    pub best_d_star: Vec<(DiagnosticArm, Option<usize>)>,
    pub all_test_weights_unchanged: bool,
}

impl DiagnosticReport {
    pub fn render_markdown(&self) -> String {
        let mut output = format!(
            "# BINN-Hybrid diagnostic robustness study\n\n\
             - protocol: `{}`\n\
             - schedule: **{}**\n\
             - seeds: {}\n\
             - budgets: {:?}\n\
             - learning rates: {:?}\n\
             - all test weights unchanged: **{}**\n\
             - scientific gate effect: **none**\n\n\
             > Development-only diagnostics. Frozen H0 protocol v3 remains \
             `HYBRID_NO_GO`; these sweeps cannot reverse it and use no fresh \
             held-out seeds. The privileged arm receives true intermediate \
             states and one supervised correction per composition step, so it \
             is a harness ceiling with up to `depth` times the supervision and \
             update magnitude, not an admissible or budget-matched learner.\n\n",
            self.protocol_hash,
            if self.config.quick {
                "PILOT"
            } else {
                "FULL DIAGNOSTIC"
            },
            self.seeds.len(),
            self.config.budgets,
            self.config.learning_rates,
            self.all_test_weights_unchanged,
        );
        output.push_str(
            "## Best observed development D*\n\n\
             | arm | D* at lower-95 accuracy ≥ 0.65 |\n\
             |---|---:|\n",
        );
        for (arm, depth) in &self.best_d_star {
            output.push_str(&format!(
                "| {} | {} |\n",
                arm.as_str(),
                depth
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "none".to_string())
            ));
        }
        output.push_str(
            "\n## Best configuration by depth\n\n\
             | depth | arm | budget | learning rate | mean | lower 95% |\n\
             |---:|---|---:|---:|---:|---:|\n",
        );
        for depth in 1..=8 {
            for arm in [
                DiagnosticArm::ExistingPostSynaptic,
                DiagnosticArm::OraclePostSynaptic,
                DiagnosticArm::DirectTerminal,
            ] {
                if let Some(summary) = self.best_summary(depth, arm) {
                    output.push_str(&format!(
                        "| {} | {} | {} | {:.4} | {:.4} | {:.4} |\n",
                        depth,
                        arm.as_str(),
                        summary.budget,
                        summary.learning_rate,
                        summary.mean_accuracy,
                        summary.lower_95,
                    ));
                }
            }
        }
        output.push_str(
            "\n## Paired effects at the direct-terminal optimum\n\n\
             | depth | direct - oracle mean | lower 95% | oracle - existing mean | lower 95% |\n\
             |---:|---:|---:|---:|---:|\n",
        );
        for depth in 1..=8 {
            if let Some(best_direct) = self.best_summary(depth, DiagnosticArm::DirectTerminal) {
                let direct_oracle = self.paired_effect(
                    depth,
                    best_direct.budget,
                    best_direct.learning_rate,
                    DiagnosticArm::DirectTerminal,
                    DiagnosticArm::OraclePostSynaptic,
                );
                let oracle_existing = self.paired_effect(
                    depth,
                    best_direct.budget,
                    best_direct.learning_rate,
                    DiagnosticArm::OraclePostSynaptic,
                    DiagnosticArm::ExistingPostSynaptic,
                );
                output.push_str(&format!(
                    "| {} | {:.4} | {:.4} | {:.4} | {:.4} |\n",
                    depth, direct_oracle.0, direct_oracle.1, oracle_existing.0, oracle_existing.1,
                ));
            }
        }
        output.push_str(
            "\n## Controls at maximum budget and best development rate\n\n\
             | depth | control | learning rate | mean | lower 95% |\n\
             |---:|---|---:|---:|---:|\n",
        );
        for depth in 1..=8 {
            for arm in [
                DiagnosticArm::PrivilegedIntermediate,
                DiagnosticArm::ShuffledLabel,
            ] {
                if let Some(summary) = self.best_summary(depth, arm) {
                    output.push_str(&format!(
                        "| {} | {} | {:.4} | {:.4} | {:.4} |\n",
                        depth,
                        arm.as_str(),
                        summary.learning_rate,
                        summary.mean_accuracy,
                        summary.lower_95,
                    ));
                }
            }
        }
        output.push_str(
            "\n## Mechanistic diagnostics at initialization\n\n\
             | depth | gradient norm | direct loss drop | shuffled loss drop | existing cosine | oracle cosine | existing sign | oracle sign |\n\
             |---:|---:|---:|---:|---:|---:|---:|---:|\n",
        );
        for depth in 1..=8 {
            let rows = self
                .mechanisms
                .iter()
                .filter(|row| row.depth == depth)
                .collect::<Vec<_>>();
            output.push_str(&format!(
                "| {} | {:.6} | {:.6} | {:.6} | {:.4} | {:.4} | {:.4} | {:.4} |\n",
                depth,
                mean_refs(&rows, |row| row.gradient_norm),
                mean_refs(&rows, |row| row.direct_loss_drop),
                mean_refs(&rows, |row| row.shuffled_loss_drop),
                mean_refs(&rows, |row| row.existing_cosine),
                mean_refs(&rows, |row| row.oracle_cosine),
                mean_refs(&rows, |row| row.existing_sign_agreement),
                mean_refs(&rows, |row| row.oracle_sign_agreement),
            ));
        }
        output.push_str(
            "\nRaw seed-level sweep and mechanism rows are emitted beside this report as CSV.\n",
        );
        output
    }

    pub fn render_sweep_csv(&self) -> String {
        let mut output =
            "protocol,seed,depth,budget,learning_rate,arm,accuracy,test_weights_unchanged\n"
                .to_string();
        for row in &self.rows {
            output.push_str(&format!(
                "{},{:016x},{},{},{:.8},{},{:.8},{}\n",
                self.protocol_hash,
                row.seed,
                row.depth,
                row.budget,
                row.learning_rate,
                row.arm.as_str(),
                row.accuracy,
                row.test_weights_unchanged,
            ));
        }
        output
    }

    pub fn render_mechanism_csv(&self) -> String {
        let mut output = "protocol,seed,depth,gradient_norm,direct_loss_drop,shuffled_loss_drop,existing_cosine,oracle_cosine,existing_sign_agreement,oracle_sign_agreement\n".to_string();
        for row in &self.mechanisms {
            output.push_str(&format!(
                "{},{:016x},{},{:.9},{:.9},{:.9},{:.9},{:.9},{:.9},{:.9}\n",
                self.protocol_hash,
                row.seed,
                row.depth,
                row.gradient_norm,
                row.direct_loss_drop,
                row.shuffled_loss_drop,
                row.existing_cosine,
                row.oracle_cosine,
                row.existing_sign_agreement,
                row.oracle_sign_agreement,
            ));
        }
        output
    }

    fn best_summary(&self, depth: usize, arm: DiagnosticArm) -> Option<&SweepSummary> {
        self.summaries
            .iter()
            .filter(|summary| summary.depth == depth && summary.arm == arm)
            .max_by(|left, right| {
                left.lower_95
                    .total_cmp(&right.lower_95)
                    .then_with(|| left.mean_accuracy.total_cmp(&right.mean_accuracy))
            })
    }

    fn paired_effect(
        &self,
        depth: usize,
        budget: usize,
        learning_rate: f32,
        treatment: DiagnosticArm,
        control: DiagnosticArm,
    ) -> (f32, f32) {
        let mut treatment_by_seed = BTreeMap::new();
        let mut control_by_seed = BTreeMap::new();
        for row in &self.rows {
            if row.depth == depth
                && row.budget == budget
                && row.learning_rate.to_bits() == learning_rate.to_bits()
            {
                if row.arm == treatment {
                    treatment_by_seed.insert(row.seed, row.accuracy);
                } else if row.arm == control {
                    control_by_seed.insert(row.seed, row.accuracy);
                }
            }
        }
        let differences = treatment_by_seed
            .iter()
            .filter_map(|(seed, treatment_value)| {
                control_by_seed
                    .get(seed)
                    .map(|control_value| treatment_value - control_value)
            })
            .collect::<Vec<_>>();
        let mean_difference = mean(&differences);
        let variance_difference = variance(&differences, mean_difference);
        let lower_95 = if differences.len() > 1 {
            mean_difference - CONFIDENCE_Z * (variance_difference / differences.len() as f32).sqrt()
        } else {
            mean_difference
        };
        (mean_difference, lower_95)
    }
}

pub fn run_diagnostics(config: &DiagnosticConfig) -> DiagnosticReport {
    assert!(!config.budgets.is_empty());
    assert!(!config.learning_rates.is_empty());
    assert!(config.n_seeds >= 2);
    let protocol_hash = config.hash_string();
    let diagnostic_seeds = config.diagnostic_seeds();
    let mut rows = Vec::new();
    for depth in 1..=8 {
        for &budget in &config.budgets {
            for &learning_rate in &config.learning_rates {
                for &seed in &diagnostic_seeds {
                    for arm in [
                        DiagnosticArm::ExistingPostSynaptic,
                        DiagnosticArm::OraclePostSynaptic,
                        DiagnosticArm::DirectTerminal,
                    ] {
                        rows.push(run_cell(
                            seed,
                            depth,
                            budget,
                            learning_rate,
                            config.test_examples,
                            arm,
                        ));
                    }
                }
            }
        }
    }
    let max_budget = *config.budgets.iter().max().expect("budget");
    for depth in 1..=8 {
        for &learning_rate in &config.learning_rates {
            for &seed in &diagnostic_seeds {
                for arm in [
                    DiagnosticArm::PrivilegedIntermediate,
                    DiagnosticArm::ShuffledLabel,
                ] {
                    rows.push(run_cell(
                        seed,
                        depth,
                        max_budget,
                        learning_rate,
                        config.test_examples,
                        arm,
                    ));
                }
            }
        }
    }
    let summaries = summarize_rows(&rows);
    let mechanisms = run_mechanisms(config, &diagnostic_seeds);
    let best_d_star = [
        DiagnosticArm::ExistingPostSynaptic,
        DiagnosticArm::OraclePostSynaptic,
        DiagnosticArm::DirectTerminal,
        DiagnosticArm::PrivilegedIntermediate,
        DiagnosticArm::ShuffledLabel,
    ]
    .into_iter()
    .map(|arm| {
        let depth = (1..=8)
            .filter(|&depth| {
                summaries
                    .iter()
                    .filter(|summary| summary.depth == depth && summary.arm == arm)
                    .any(|summary| summary.lower_95 >= C3_ACCURACY_FLOOR)
            })
            .max();
        (arm, depth)
    })
    .collect();
    let all_test_weights_unchanged = rows.iter().all(|row| row.test_weights_unchanged);
    DiagnosticReport {
        protocol_hash,
        config: config.clone(),
        seeds: diagnostic_seeds,
        rows,
        summaries,
        mechanisms,
        best_d_star,
        all_test_weights_unchanged,
    }
}

fn run_cell(
    seed: u64,
    depth: usize,
    budget: usize,
    learning_rate: f32,
    test_count: usize,
    arm: DiagnosticArm,
) -> SweepRow {
    let mut model = C3CompositionModel::new(seed);
    let train = c3_examples(seed, depth, budget);
    let test = c3_examples(seed ^ TEST_SEED_XOR, depth, test_count);
    for example in &train {
        match arm {
            DiagnosticArm::PrivilegedIntermediate => {
                apply_privileged_intermediate(&mut model, example, learning_rate);
            }
            DiagnosticArm::ExistingPostSynaptic
            | DiagnosticArm::OraclePostSynaptic
            | DiagnosticArm::DirectTerminal
            | DiagnosticArm::ShuffledLabel => {
                let trace = model.forward(example);
                let label = if arm == DiagnosticArm::ShuffledLabel {
                    (example.label + 1) % 4
                } else {
                    example.label
                };
                let targets = model.teacher_targets(&trace, label, learning_rate, None);
                let audit = factorization_audit(&targets, &model.edge_posts());
                let deltas = match arm {
                    DiagnosticArm::ExistingPostSynaptic => audit.existing_post_deltas,
                    DiagnosticArm::OraclePostSynaptic => audit.oracle_post_deltas,
                    DiagnosticArm::DirectTerminal | DiagnosticArm::ShuffledLabel => {
                        audit.direct_edge_deltas
                    }
                    DiagnosticArm::PrivilegedIntermediate => unreachable!(),
                };
                model.apply_deltas(&deltas);
            }
        }
    }
    let before_test = weight_fingerprint(&model);
    let accuracy = c3_accuracy(&model, &test);
    let test_weights_unchanged = before_test == weight_fingerprint(&model);
    SweepRow {
        seed,
        depth,
        budget,
        learning_rate,
        arm,
        accuracy,
        test_weights_unchanged,
    }
}

fn apply_privileged_intermediate(
    model: &mut C3CompositionModel,
    example: &C3Example,
    learning_rate: f32,
) {
    let mut state = example.initial_state;
    let mut deltas = vec![0.0f32; model.engine.edge_w.len()];
    for &operation in &example.operations {
        let next = true_transition(state, operation);
        let local = C3Example {
            initial_state: state,
            operations: vec![operation],
            label: next,
        };
        let trace = model.forward(&local);
        let targets = model.teacher_targets(&trace, next, learning_rate, None);
        for (total, delta) in deltas.iter_mut().zip(targets.edge_deltas) {
            *total += delta;
        }
        state = next;
    }
    model.apply_deltas(&deltas);
}

fn true_transition(state: usize, operation: usize) -> usize {
    if operation == 0 {
        (state + 1) % 4
    } else {
        [1, 3, 0, 2][state]
    }
}

fn run_mechanisms(config: &DiagnosticConfig, diagnostic_seeds: &[u64]) -> Vec<MechanismRow> {
    let mut rows = Vec::new();
    for depth in 1..=8 {
        for &seed in diagnostic_seeds {
            let examples = c3_examples(seed ^ 0x4d45_4348, depth, config.mechanism_examples);
            let mut gradient_norms = Vec::new();
            let mut direct_drops = Vec::new();
            let mut shuffled_drops = Vec::new();
            let mut existing_cosines = Vec::new();
            let mut oracle_cosines = Vec::new();
            let mut existing_signs = Vec::new();
            let mut oracle_signs = Vec::new();
            for (index, example) in examples.iter().enumerate() {
                let model = C3CompositionModel::new(seed ^ index as u64);
                let trace = model.forward(example);
                let targets = model.teacher_targets(&trace, example.label, 0.001, None);
                let audit = factorization_audit(&targets, &model.edge_posts());
                gradient_norms.push(
                    targets
                        .edge_deltas
                        .iter()
                        .map(|delta| delta * delta)
                        .sum::<f32>()
                        .sqrt()
                        / 0.001,
                );
                let mut direct = clone_model(&model);
                direct.apply_deltas(&targets.edge_deltas);
                let direct_loss = direct
                    .teacher_targets(&direct.forward(example), example.label, 1.0, None)
                    .loss;
                direct_drops.push(targets.loss - direct_loss);

                let mut shuffled = clone_model(&model);
                let mut shuffled_deltas = targets.edge_deltas.clone();
                shuffled_deltas.rotate_left(1);
                shuffled.apply_deltas(&shuffled_deltas);
                let shuffled_loss = shuffled
                    .teacher_targets(&shuffled.forward(example), example.label, 1.0, None)
                    .loss;
                shuffled_drops.push(targets.loss - shuffled_loss);
                existing_cosines.push(audit.existing_cosine);
                oracle_cosines.push(audit.oracle_cosine);
                existing_signs.push(audit.existing_sign_agreement);
                oracle_signs.push(audit.oracle_sign_agreement);
            }
            rows.push(MechanismRow {
                seed,
                depth,
                gradient_norm: mean(&gradient_norms),
                direct_loss_drop: mean(&direct_drops),
                shuffled_loss_drop: mean(&shuffled_drops),
                existing_cosine: mean(&existing_cosines),
                oracle_cosine: mean(&oracle_cosines),
                existing_sign_agreement: mean(&existing_signs),
                oracle_sign_agreement: mean(&oracle_signs),
            });
        }
    }
    rows
}

fn clone_model(model: &C3CompositionModel) -> C3CompositionModel {
    let mut clone = C3CompositionModel::new(1);
    clone.engine.edge_w.clone_from(&model.engine.edge_w);
    clone
        .engine
        .syn
        .rebuild_from_weights(&clone.engine.edge_w, 1);
    clone
}

fn weight_fingerprint(model: &C3CompositionModel) -> u64 {
    let mut bytes = Vec::with_capacity(model.engine.edge_w.len() * 4);
    for weight in &model.engine.edge_w {
        bytes.extend_from_slice(&weight.to_bits().to_le_bytes());
    }
    fnv1a64(&bytes)
}

fn summarize_rows(rows: &[SweepRow]) -> Vec<SweepSummary> {
    let mut groups: BTreeMap<(usize, usize, u32, DiagnosticArm), Vec<f32>> = BTreeMap::new();
    for row in rows {
        groups
            .entry((row.depth, row.budget, row.learning_rate.to_bits(), row.arm))
            .or_default()
            .push(row.accuracy);
    }
    groups
        .into_iter()
        .map(|((depth, budget, rate_bits, arm), values)| {
            let mean_accuracy = mean(&values);
            let variance = variance(&values, mean_accuracy);
            let lower_95 = mean_accuracy - CONFIDENCE_Z * (variance / values.len() as f32).sqrt();
            SweepSummary {
                depth,
                budget,
                learning_rate: f32::from_bits(rate_bits),
                arm,
                mean_accuracy,
                variance,
                lower_95,
            }
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

fn variance(values: &[f32], mean: f32) -> f32 {
    if values.len() < 2 {
        return 0.0;
    }
    values
        .iter()
        .map(|value| {
            let delta = value - mean;
            delta * delta
        })
        .sum::<f32>()
        / (values.len() - 1) as f32
}

fn mean_refs<F>(rows: &[&MechanismRow], value: F) -> f32
where
    F: Fn(&MechanismRow) -> f32,
{
    if rows.is_empty() {
        0.0
    } else {
        rows.iter().map(|row| value(row)).sum::<f32>() / rows.len() as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagnostic_hash_is_sensitive_and_seed_families_are_disjoint() {
        let config = DiagnosticConfig::quick();
        let mut changed = config.clone();
        changed.budgets[0] += 1;
        assert_ne!(config.hash(), changed.hash());
        let diagnostic = config.diagnostic_seeds();
        let frozen_v3_development = seeds(0x4842_4445_5600_0001, 5);
        let frozen_v3_pilot = seeds(0x4842_5049_4c4f_5402, 3);
        let frozen_v3_held_out = seeds(0x4842_4652_4553_4802, 20);
        assert!(diagnostic.iter().all(|seed| {
            !frozen_v3_development.contains(seed)
                && !frozen_v3_pilot.contains(seed)
                && !frozen_v3_held_out.contains(seed)
        }));
    }

    #[test]
    fn direct_gradient_step_beats_rotated_direction_on_average() {
        let config = DiagnosticConfig {
            n_seeds: 2,
            mechanism_examples: 32,
            ..DiagnosticConfig::quick()
        };
        let mechanisms = run_mechanisms(&config, &config.diagnostic_seeds());
        let direct = mechanisms
            .iter()
            .map(|row| row.direct_loss_drop)
            .sum::<f32>();
        let shuffled = mechanisms
            .iter()
            .map(|row| row.shuffled_loss_drop)
            .sum::<f32>();
        assert!(direct > 0.0);
        assert!(direct > shuffled);
    }

    #[test]
    fn privileged_control_uses_no_test_updates() {
        let row = run_cell(7, 8, 120, 0.035, 64, DiagnosticArm::PrivilegedIntermediate);
        assert!(row.test_weights_unchanged);
        assert!(row.accuracy.is_finite());
    }

    #[test]
    fn quick_diagnostic_replays_exactly() {
        let config = DiagnosticConfig {
            n_seeds: 2,
            budgets: vec![20],
            learning_rates: vec![0.015],
            test_examples: 20,
            mechanism_examples: 4,
            ..DiagnosticConfig::quick()
        };
        let first = run_diagnostics(&config);
        let second = run_diagnostics(&config);
        assert_eq!(first.protocol_hash, second.protocol_hash);
        assert_eq!(first.rows, second.rows);
        assert_eq!(first.summaries, second.summaries);
        assert_eq!(first.mechanisms, second.mechanisms);
        let expected_rows = 8
            * (config.budgets.len() * config.learning_rates.len() * config.n_seeds * 3
                + config.learning_rates.len() * config.n_seeds * 2);
        assert_eq!(first.rows.len(), expected_rows);
        assert!(first.all_test_weights_unchanged);
        assert!(first.rows.iter().all(|row| row.accuracy.is_finite()));
        assert!(first.mechanisms.iter().all(|row| {
            row.gradient_norm.is_finite()
                && row.direct_loss_drop.is_finite()
                && row.shuffled_loss_drop.is_finite()
        }));
        assert_eq!(first.render_sweep_csv(), second.render_sweep_csv());
        assert_eq!(first.render_mechanism_csv(), second.render_mechanism_csv());
    }
}
