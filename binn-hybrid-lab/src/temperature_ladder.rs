//! Soft-to-hard winner-temperature ladder for terminal-gradient transfer.
//!
//! Separate successor protocol. It does not reopen frozen H0 (`HYBRID_NO_GO`),
//! does not consume held-out seeds, and cannot authorize H1-H3.
//!
//! Design (preregistered):
//! - Train with the matched soft residual terminal teacher (same geometry as the
//!   smooth diagnostic / production teacher). Direct-terminal updates do not
//!   depend on winner temperature.
//! - Evaluate the same trained weights under a winner-temperature ladder from
//!   soft linear residual propagation to hard one-hot winners.
//! - Localize where soft-teacher terminal gradients stop transferring into
//!   tempered / hard winner forwards.

use std::collections::BTreeMap;

use binn_hybrid_learn::fnv1a64;

use crate::benchmark::{c3_examples, seeds, C3CompositionModel, C3Example};

pub const TEMPERATURE_LADDER_PROTOCOL_VERSION: u32 = 1;
const LADDER_SEED_MASTER: u64 = 0x4842_5445_4d50_0001;
const TEST_SEED_XOR: u64 = 0x5445_4d50_5445_5354;
const MECHANISM_SEED_XOR: u64 = 0x5445_4d50_4d45_4348;
const ACCURACY_FLOOR: f32 = 0.65;
const CONFIDENCE_Z: f32 = 1.96;
const C3_STATES: usize = 4;
const MIN_DEPTH: usize = 1;
const MAX_DEPTH: usize = 8;

/// Winner temperature. `Soft` is linear residual propagation; `Hard` is
/// one-hot argmax; finite values use `softmax(scores / T)`.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum WinnerTemperature {
    Soft,
    Finite(f32),
    Hard,
}

impl WinnerTemperature {
    pub fn as_str(self) -> String {
        match self {
            Self::Soft => "soft".to_string(),
            Self::Finite(temperature) => format!("{temperature:.4}"),
            Self::Hard => "hard".to_string(),
        }
    }

    pub fn sort_key(self) -> (u8, u32) {
        match self {
            Self::Soft => (0, 0),
            Self::Finite(temperature) => (1, temperature.to_bits()),
            Self::Hard => (2, 0),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TemperatureLadderConfig {
    pub quick: bool,
    pub n_seeds: usize,
    pub budgets: Vec<usize>,
    pub learning_rates: Vec<f32>,
    pub temperatures: Vec<WinnerTemperature>,
    pub test_examples: usize,
    pub mechanism_examples: usize,
}

impl TemperatureLadderConfig {
    pub fn quick() -> Self {
        Self {
            quick: true,
            n_seeds: 3,
            budgets: vec![60, 240],
            learning_rates: vec![0.035],
            temperatures: vec![
                WinnerTemperature::Soft,
                WinnerTemperature::Finite(1.0),
                WinnerTemperature::Hard,
            ],
            test_examples: 160,
            mechanism_examples: 16,
        }
    }

    pub fn full() -> Self {
        Self {
            quick: false,
            n_seeds: 20,
            budgets: vec![480, 1_920, 7_680],
            learning_rates: vec![0.015, 0.035, 0.070],
            temperatures: vec![
                WinnerTemperature::Soft,
                WinnerTemperature::Finite(2.0),
                WinnerTemperature::Finite(1.0),
                WinnerTemperature::Finite(0.5),
                WinnerTemperature::Finite(0.25),
                WinnerTemperature::Finite(0.1),
                WinnerTemperature::Hard,
            ],
            test_examples: 1_000,
            mechanism_examples: 128,
        }
    }

    pub fn hash(&self) -> u64 {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&TEMPERATURE_LADDER_PROTOCOL_VERSION.to_le_bytes());
        bytes.push(u8::from(self.quick));
        for value in [
            self.n_seeds,
            self.test_examples,
            self.mechanism_examples,
            MIN_DEPTH,
            MAX_DEPTH,
            C3_STATES,
        ] {
            bytes.extend_from_slice(&(value as u64).to_le_bytes());
        }
        for &budget in &self.budgets {
            bytes.extend_from_slice(&(budget as u64).to_le_bytes());
        }
        bytes.push(0xff);
        for &rate in &self.learning_rates {
            bytes.extend_from_slice(&rate.to_bits().to_le_bytes());
        }
        bytes.push(0xfe);
        for temperature in &self.temperatures {
            match *temperature {
                WinnerTemperature::Soft => bytes.push(0),
                WinnerTemperature::Finite(value) => {
                    bytes.push(1);
                    bytes.extend_from_slice(&value.to_bits().to_le_bytes());
                }
                WinnerTemperature::Hard => bytes.push(2),
            }
        }
        fnv1a64(&bytes)
    }

    pub fn hash_string(&self) -> String {
        format!(
            "binn-hybrid-winner-temp-v{TEMPERATURE_LADDER_PROTOCOL_VERSION}-{:016x}",
            self.hash()
        )
    }

    pub fn ladder_seeds(&self) -> Vec<u64> {
        seeds(LADDER_SEED_MASTER, self.n_seeds)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum LadderArm {
    DirectTerminal,
    PrivilegedIntermediate,
    ShuffledLabel,
}

impl LadderArm {
    pub const MAIN: [Self; 1] = [Self::DirectTerminal];
    pub const CONTROLS: [Self; 2] = [Self::PrivilegedIntermediate, Self::ShuffledLabel];
    pub const ALL: [Self; 3] = [
        Self::DirectTerminal,
        Self::PrivilegedIntermediate,
        Self::ShuffledLabel,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::DirectTerminal => "direct-terminal",
            Self::PrivilegedIntermediate => "privileged-intermediate-target",
            Self::ShuffledLabel => "shuffled-label",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LadderSweepRow {
    pub seed: u64,
    pub depth: usize,
    pub budget: usize,
    pub learning_rate: f32,
    pub temperature: WinnerTemperature,
    pub arm: LadderArm,
    pub accuracy: f32,
    pub test_weights_unchanged: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LadderSweepSummary {
    pub depth: usize,
    pub budget: usize,
    pub learning_rate: f32,
    pub temperature: WinnerTemperature,
    pub arm: LadderArm,
    pub mean_accuracy: f32,
    pub variance: f32,
    pub lower_95: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LadderMechanismRow {
    pub seed: u64,
    pub depth: usize,
    pub temperature: WinnerTemperature,
    pub gradient_norm: f32,
    pub soft_direct_loss_drop: f32,
    pub soft_rotated_loss_drop: f32,
    pub tempered_direct_loss_drop: f32,
    pub tempered_rotated_loss_drop: f32,
}

#[derive(Clone, Debug)]
pub struct TemperatureLadderReport {
    pub protocol_hash: String,
    pub config: TemperatureLadderConfig,
    pub seeds: Vec<u64>,
    pub rows: Vec<LadderSweepRow>,
    pub summaries: Vec<LadderSweepSummary>,
    pub mechanisms: Vec<LadderMechanismRow>,
    pub best_d_star: Vec<(WinnerTemperature, LadderArm, Option<usize>)>,
    pub collapse_temperature: Option<WinnerTemperature>,
    pub all_test_weights_unchanged: bool,
}

impl TemperatureLadderReport {
    pub fn render_markdown(&self) -> String {
        let mut output = format!(
            "# BINN-Hybrid soft-to-hard winner-temperature ladder\n\n\
             - protocol: `{}`\n\
             - schedule: **{}**\n\
             - seeds: {}\n\
             - depths: {} through {}\n\
             - budgets: {:?}\n\
             - learning rates: {:?}\n\
             - temperatures: {}\n\
             - frozen test examples per cell: {}\n\
             - all test weights unchanged: **{}**\n\
             - scientific gate effect: **none**\n\
             - transfer collapse temperature: **{}**\n\n\
             > Separately preregistered successor diagnostic. Canonical H0 \
             remains `HYBRID_NO_GO`; held-out seeds remain unused; H1-H3 remain \
             stopped. This ladder localizes where soft residual terminal \
             gradients stop transferring across winner discretization. It is \
             not post-hoc tuning of frozen H0 or the production diagnostic.\n\n",
            self.protocol_hash,
            if self.config.quick {
                "PILOT"
            } else {
                "FULL DEVELOPMENT LADDER"
            },
            self.seeds.len(),
            MIN_DEPTH,
            MAX_DEPTH,
            self.config.budgets,
            self.config.learning_rates,
            self.config
                .temperatures
                .iter()
                .map(|temperature| temperature.as_str())
                .collect::<Vec<_>>()
                .join(", "),
            self.config.test_examples,
            self.all_test_weights_unchanged,
            self.collapse_temperature
                .map(|temperature| temperature.as_str())
                .unwrap_or_else(|| "none".to_string()),
        );
        output.push_str(
            "## Mechanism contract\n\n\
             Training uses only the matched soft residual terminal teacher: \
             linear composition of shared transition weights, terminal softmax \
             cross-entropy, and exact edge gradients. Direct-terminal updates \
             are therefore temperature-independent. Evaluation applies a \
             winner operator to the same residual scores: `soft` keeps the \
             linear state, finite `T` uses `softmax(scores / T)`, and `hard` \
             uses one-hot argmax. Privileged intermediate targets remain an \
             inadmissible ceiling; shuffled labels remain a leakage control.\n\n",
        );
        output.push_str(
            "## Best observed development D* by temperature\n\n\
             | temperature | arm | D* at lower-95 accuracy ≥ 0.65 |\n\
             |---|---|---:|\n",
        );
        for (temperature, arm, depth) in &self.best_d_star {
            output.push_str(&format!(
                "| {} | {} | {} |\n",
                temperature.as_str(),
                arm.as_str(),
                depth
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "none".to_string())
            ));
        }
        output.push_str(
            "\n## Direct-terminal transfer curve\n\n\
             | temperature | D* | best depth mean | best depth lower 95% | budget | lr |\n\
             |---|---:|---:|---:|---:|---:|\n",
        );
        for temperature in &self.config.temperatures {
            let d_star = self
                .best_d_star
                .iter()
                .find(|(candidate, arm, _)| {
                    candidate == temperature && *arm == LadderArm::DirectTerminal
                })
                .and_then(|(_, _, depth)| *depth);
            if let Some(depth) = d_star {
                let best = self
                    .summaries
                    .iter()
                    .filter(|summary| {
                        summary.temperature == *temperature
                            && summary.arm == LadderArm::DirectTerminal
                            && summary.depth == depth
                    })
                    .max_by(|left, right| {
                        left.lower_95
                            .partial_cmp(&right.lower_95)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    });
                if let Some(summary) = best {
                    output.push_str(&format!(
                        "| {} | {} | {:.4} | {:.4} | {} | {:.4} |\n",
                        temperature.as_str(),
                        depth,
                        summary.mean_accuracy,
                        summary.lower_95,
                        summary.budget,
                        summary.learning_rate,
                    ));
                }
            } else {
                let best = self
                    .summaries
                    .iter()
                    .filter(|summary| {
                        summary.temperature == *temperature
                            && summary.arm == LadderArm::DirectTerminal
                    })
                    .max_by(|left, right| {
                        left.lower_95
                            .partial_cmp(&right.lower_95)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    });
                if let Some(summary) = best {
                    output.push_str(&format!(
                        "| {} | none | {:.4} | {:.4} | {} | {:.4} |\n",
                        temperature.as_str(),
                        summary.mean_accuracy,
                        summary.lower_95,
                        summary.budget,
                        summary.learning_rate,
                    ));
                } else {
                    output.push_str(&format!(
                        "| {} | none | — | — | — | — |\n",
                        temperature.as_str()
                    ));
                }
            }
        }
        output.push_str(
            "\n## Collapse rule\n\n\
             Transfer collapse temperature is the softest ladder point at which \
             direct-terminal D* falls strictly below the soft-endpoint D*. If \
             every tempered/hard point preserves the soft D*, collapse is \
             `none` under this development grid.\n\n",
        );
        output.push_str("## Mechanisms by temperature\n\n");
        output.push_str(
            "| temperature | depth | grad norm | soft direct drop | soft rotated drop | \
             tempered direct drop | tempered rotated drop |\n\
             |---|---:|---:|---:|---:|---:|---:|\n",
        );
        for temperature in &self.config.temperatures {
            for depth in MIN_DEPTH..=MAX_DEPTH {
                let rows = self
                    .mechanisms
                    .iter()
                    .filter(|row| row.temperature == *temperature && row.depth == depth)
                    .collect::<Vec<_>>();
                if rows.is_empty() {
                    continue;
                }
                output.push_str(&format!(
                    "| {} | {} | {:.4} | {:.6} | {:.6} | {:.6} | {:.6} |\n",
                    temperature.as_str(),
                    depth,
                    mean_refs(&rows, |row| row.gradient_norm),
                    mean_refs(&rows, |row| row.soft_direct_loss_drop),
                    mean_refs(&rows, |row| row.soft_rotated_loss_drop),
                    mean_refs(&rows, |row| row.tempered_direct_loss_drop),
                    mean_refs(&rows, |row| row.tempered_rotated_loss_drop),
                ));
            }
        }
        output.push_str(
            "\n## Limits\n\n\
             - Development seeds and hyperparameter selection only.\n\
             - Soft teacher remains a disclosed differentiable residual \
               relaxation; hard winners have no ordinary derivative.\n\
             - Finite temperatures use softmax winners, which are not identical \
               to the soft linear endpoint; `soft` is an explicit anchor.\n\
             - Privileged ceiling is not budget-matched.\n\
             - Cannot reopen H0 or authorize H1-H3.\n",
        );
        output
    }

    pub fn render_sweep_csv(&self) -> String {
        let mut output = String::from(
            "seed,depth,budget,learning_rate,temperature,arm,accuracy,test_weights_unchanged\n",
        );
        for row in &self.rows {
            output.push_str(&format!(
                "{},{},{},{:.6},{},{},{:.6},{}\n",
                row.seed,
                row.depth,
                row.budget,
                row.learning_rate,
                row.temperature.as_str(),
                row.arm.as_str(),
                row.accuracy,
                row.test_weights_unchanged,
            ));
        }
        output
    }

    pub fn render_mechanism_csv(&self) -> String {
        let mut output = String::from(
            "seed,depth,temperature,gradient_norm,soft_direct_loss_drop,soft_rotated_loss_drop,\
             tempered_direct_loss_drop,tempered_rotated_loss_drop\n",
        );
        for row in &self.mechanisms {
            output.push_str(&format!(
                "{},{},{},{:.6},{:.6},{:.6},{:.6},{:.6}\n",
                row.seed,
                row.depth,
                row.temperature.as_str(),
                row.gradient_norm,
                row.soft_direct_loss_drop,
                row.soft_rotated_loss_drop,
                row.tempered_direct_loss_drop,
                row.tempered_rotated_loss_drop,
            ));
        }
        output
    }
}

pub fn run_temperature_ladder(config: &TemperatureLadderConfig) -> TemperatureLadderReport {
    assert!(!config.budgets.is_empty());
    assert!(!config.learning_rates.is_empty());
    assert!(!config.temperatures.is_empty());
    assert!(config.n_seeds >= 2);
    for temperature in &config.temperatures {
        if let WinnerTemperature::Finite(value) = *temperature {
            assert!(value > 0.0 && value.is_finite());
        }
    }

    let protocol_hash = config.hash_string();
    let ladder_seeds = config.ladder_seeds();
    let mut rows = Vec::new();
    for depth in MIN_DEPTH..=MAX_DEPTH {
        for &budget in &config.budgets {
            for &learning_rate in &config.learning_rates {
                for &seed in &ladder_seeds {
                    for arm in LadderArm::MAIN {
                        rows.extend(run_trained_ladder_cell(
                            config,
                            seed,
                            depth,
                            budget,
                            learning_rate,
                            arm,
                        ));
                    }
                }
            }
        }
    }
    let max_budget = *config.budgets.iter().max().expect("budget");
    for depth in MIN_DEPTH..=MAX_DEPTH {
        for &learning_rate in &config.learning_rates {
            for &seed in &ladder_seeds {
                for arm in LadderArm::CONTROLS {
                    rows.extend(run_trained_ladder_cell(
                        config,
                        seed,
                        depth,
                        max_budget,
                        learning_rate,
                        arm,
                    ));
                }
            }
        }
    }

    let summaries = summarize_rows(&rows);
    let mechanisms = run_mechanisms(config, &ladder_seeds);
    let mut best_d_star = Vec::new();
    for temperature in config.temperatures.iter().copied() {
        for arm in LadderArm::ALL {
            let depth = (MIN_DEPTH..=MAX_DEPTH)
                .filter(|&depth| {
                    summaries.iter().any(|summary| {
                        summary.temperature == temperature
                            && summary.arm == arm
                            && summary.depth == depth
                            && summary.lower_95 >= ACCURACY_FLOOR
                    })
                })
                .max();
            best_d_star.push((temperature, arm, depth));
        }
    }
    let soft_d_star = best_d_star
        .iter()
        .find(|(temperature, arm, _)| {
            *temperature == WinnerTemperature::Soft && *arm == LadderArm::DirectTerminal
        })
        .and_then(|(_, _, depth)| *depth);
    let collapse_temperature = soft_d_star.and_then(|soft_depth| {
        config.temperatures.iter().copied().find(|temperature| {
            if *temperature == WinnerTemperature::Soft {
                return false;
            }
            let tempered_depth = best_d_star
                .iter()
                .find(|(candidate, arm, _)| {
                    candidate == temperature && *arm == LadderArm::DirectTerminal
                })
                .and_then(|(_, _, depth)| *depth);
            match tempered_depth {
                Some(depth) => depth < soft_depth,
                None => true,
            }
        })
    });
    let all_test_weights_unchanged = rows.iter().all(|row| row.test_weights_unchanged);
    TemperatureLadderReport {
        protocol_hash,
        config: config.clone(),
        seeds: ladder_seeds,
        rows,
        summaries,
        mechanisms,
        best_d_star,
        collapse_temperature,
        all_test_weights_unchanged,
    }
}

fn run_trained_ladder_cell(
    config: &TemperatureLadderConfig,
    seed: u64,
    depth: usize,
    budget: usize,
    learning_rate: f32,
    arm: LadderArm,
) -> Vec<LadderSweepRow> {
    let train = c3_examples(seed, depth, budget);
    let test = c3_examples(seed ^ TEST_SEED_XOR, depth, config.test_examples);
    let mut model = C3CompositionModel::new(seed);
    for example in &train {
        train_example(&mut model, example, arm, learning_rate);
    }
    let before_test = weight_fingerprint(&model);
    let mut rows = Vec::with_capacity(config.temperatures.len());
    for &temperature in &config.temperatures {
        let accuracy = tempered_accuracy(&model, &test, temperature);
        let after_test = weight_fingerprint(&model);
        rows.push(LadderSweepRow {
            seed,
            depth,
            budget,
            learning_rate,
            temperature,
            arm,
            accuracy,
            test_weights_unchanged: before_test == after_test,
        });
    }
    rows
}

fn train_example(
    model: &mut C3CompositionModel,
    example: &C3Example,
    arm: LadderArm,
    learning_rate: f32,
) {
    match arm {
        LadderArm::PrivilegedIntermediate => {
            apply_privileged_intermediate(model, example, learning_rate);
        }
        LadderArm::DirectTerminal | LadderArm::ShuffledLabel => {
            let trace = model.forward(example);
            let label = if arm == LadderArm::ShuffledLabel {
                (example.label + 1) % C3_STATES
            } else {
                example.label
            };
            let targets = model.teacher_targets(&trace, label, learning_rate, None);
            model.apply_deltas(&targets.edge_deltas);
        }
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
        (state + 1) % C3_STATES
    } else {
        [1, 3, 0, 2][state]
    }
}

fn tempered_accuracy(
    model: &C3CompositionModel,
    examples: &[C3Example],
    temperature: WinnerTemperature,
) -> f32 {
    if examples.is_empty() {
        return 0.0;
    }
    let correct = examples
        .iter()
        .filter(|example| tempered_predict(model, example, temperature) == example.label)
        .count();
    correct as f32 / examples.len() as f32
}

fn tempered_predict(
    model: &C3CompositionModel,
    example: &C3Example,
    temperature: WinnerTemperature,
) -> usize {
    let state = tempered_forward(model, example, temperature);
    argmax(&state)
}

fn tempered_forward(
    model: &C3CompositionModel,
    example: &C3Example,
    temperature: WinnerTemperature,
) -> [f32; C3_STATES] {
    let mut state = [0.0f32; C3_STATES];
    state[example.initial_state] = 1.0;
    for &operation in &example.operations {
        // Match C3CompositionModel / production event scores: identity residual
        // plus the shared transition operator.
        let mut scores = state;
        for (post, score) in scores.iter_mut().enumerate() {
            for (pre, &pre_value) in state.iter().enumerate() {
                *score += model.engine.edge_w[model.edge(operation, pre, post)] * pre_value;
            }
        }
        state = match temperature {
            WinnerTemperature::Soft => scores,
            WinnerTemperature::Finite(value) => softmax_temperature(&scores, value),
            WinnerTemperature::Hard => one_hot(argmax(&scores)),
        };
    }
    state
}

fn tempered_nll(
    model: &C3CompositionModel,
    example: &C3Example,
    label: usize,
    temperature: WinnerTemperature,
) -> f32 {
    let state = tempered_forward(model, example, temperature);
    let probabilities = softmax_temperature(&state, 1.0);
    -probabilities[label].max(1e-12).ln()
}

fn run_mechanisms(
    config: &TemperatureLadderConfig,
    ladder_seeds: &[u64],
) -> Vec<LadderMechanismRow> {
    let mut rows = Vec::new();
    for depth in MIN_DEPTH..=MAX_DEPTH {
        for &seed in ladder_seeds {
            let examples = c3_examples(seed ^ MECHANISM_SEED_XOR, depth, config.mechanism_examples);
            for &temperature in &config.temperatures {
                let mut gradient_norms = Vec::new();
                let mut soft_direct = Vec::new();
                let mut soft_rotated = Vec::new();
                let mut tempered_direct = Vec::new();
                let mut tempered_rotated = Vec::new();
                for (index, example) in examples.iter().enumerate() {
                    let model = C3CompositionModel::new(seed ^ index as u64);
                    let soft_trace = model.forward(example);
                    let targets = model.teacher_targets(&soft_trace, example.label, 0.001, None);
                    gradient_norms.push(
                        targets
                            .edge_deltas
                            .iter()
                            .map(|delta| delta * delta)
                            .sum::<f32>()
                            .sqrt()
                            / 0.001,
                    );

                    let soft_before = model
                        .teacher_targets(&soft_trace, example.label, 1.0, None)
                        .loss;
                    let tempered_before = tempered_nll(&model, example, example.label, temperature);

                    let mut direct = clone_model(&model);
                    direct.apply_deltas(&targets.edge_deltas);
                    let soft_direct_loss = direct
                        .teacher_targets(&direct.forward(example), example.label, 1.0, None)
                        .loss;
                    let tempered_direct_loss =
                        tempered_nll(&direct, example, example.label, temperature);
                    soft_direct.push(soft_before - soft_direct_loss);
                    tempered_direct.push(tempered_before - tempered_direct_loss);

                    let mut rotated = clone_model(&model);
                    let mut rotated_deltas = targets.edge_deltas.clone();
                    rotated_deltas.rotate_left(1);
                    rotated.apply_deltas(&rotated_deltas);
                    let soft_rotated_loss = rotated
                        .teacher_targets(&rotated.forward(example), example.label, 1.0, None)
                        .loss;
                    let tempered_rotated_loss =
                        tempered_nll(&rotated, example, example.label, temperature);
                    soft_rotated.push(soft_before - soft_rotated_loss);
                    tempered_rotated.push(tempered_before - tempered_rotated_loss);
                }
                rows.push(LadderMechanismRow {
                    seed,
                    depth,
                    temperature,
                    gradient_norm: mean(&gradient_norms),
                    soft_direct_loss_drop: mean(&soft_direct),
                    soft_rotated_loss_drop: mean(&soft_rotated),
                    tempered_direct_loss_drop: mean(&tempered_direct),
                    tempered_rotated_loss_drop: mean(&tempered_rotated),
                });
            }
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

type SweepGroupKey = (usize, usize, u32, (u8, u32), LadderArm);

fn summarize_rows(rows: &[LadderSweepRow]) -> Vec<LadderSweepSummary> {
    let mut groups: BTreeMap<SweepGroupKey, Vec<f32>> = BTreeMap::new();
    for row in rows {
        groups
            .entry((
                row.depth,
                row.budget,
                row.learning_rate.to_bits(),
                row.temperature.sort_key(),
                row.arm,
            ))
            .or_default()
            .push(row.accuracy);
    }
    groups
        .into_iter()
        .map(
            |((depth, budget, rate_bits, temperature_key, arm), values)| {
                let mean_accuracy = mean(&values);
                let variance = variance(&values, mean_accuracy);
                let lower_95 =
                    mean_accuracy - CONFIDENCE_Z * (variance / values.len() as f32).sqrt();
                let temperature = match temperature_key.0 {
                    0 => WinnerTemperature::Soft,
                    2 => WinnerTemperature::Hard,
                    _ => WinnerTemperature::Finite(f32::from_bits(temperature_key.1)),
                };
                LadderSweepSummary {
                    depth,
                    budget,
                    learning_rate: f32::from_bits(rate_bits),
                    temperature,
                    arm,
                    mean_accuracy,
                    variance,
                    lower_95,
                }
            },
        )
        .collect()
}

fn softmax_temperature(values: &[f32; C3_STATES], temperature: f32) -> [f32; C3_STATES] {
    let inv_t = 1.0 / temperature;
    let max = values
        .iter()
        .map(|value| value * inv_t)
        .fold(f32::NEG_INFINITY, f32::max);
    let mut probabilities = [0.0f32; C3_STATES];
    let mut sum = 0.0f32;
    for (slot, value) in probabilities.iter_mut().zip(values) {
        *slot = (value * inv_t - max).exp();
        sum += *slot;
    }
    for probability in &mut probabilities {
        *probability /= sum.max(1e-12);
    }
    probabilities
}

fn argmax(values: &[f32; C3_STATES]) -> usize {
    values
        .iter()
        .enumerate()
        .max_by(|(_, left), (_, right)| left.total_cmp(right))
        .map(|(index, _)| index)
        .unwrap_or(0)
}

fn one_hot(index: usize) -> [f32; C3_STATES] {
    let mut state = [0.0f32; C3_STATES];
    state[index] = 1.0;
    state
}

fn mean(values: &[f32]) -> f32 {
    if values.is_empty() {
        0.0
    } else {
        values.iter().sum::<f32>() / values.len() as f32
    }
}

fn variance(values: &[f32], values_mean: f32) -> f32 {
    if values.len() < 2 {
        return 0.0;
    }
    values
        .iter()
        .map(|value| (value - values_mean).powi(2))
        .sum::<f32>()
        / (values.len() - 1) as f32
}

fn mean_refs<F>(rows: &[&LadderMechanismRow], value: F) -> f32
where
    F: Fn(&LadderMechanismRow) -> f32,
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

    fn tiny_config() -> TemperatureLadderConfig {
        TemperatureLadderConfig {
            n_seeds: 2,
            budgets: vec![12],
            learning_rates: vec![0.035],
            temperatures: vec![
                WinnerTemperature::Soft,
                WinnerTemperature::Finite(1.0),
                WinnerTemperature::Hard,
            ],
            test_examples: 16,
            mechanism_examples: 2,
            ..TemperatureLadderConfig::quick()
        }
    }

    #[test]
    fn protocol_hash_is_sensitive_and_seed_families_are_disjoint() {
        let config = TemperatureLadderConfig::quick();
        let mut changed = config.clone();
        changed.budgets[0] += 1;
        assert_ne!(config.hash(), changed.hash());
        let ladder = config.ladder_seeds();
        let frozen_v3_development = seeds(0x4842_4445_5600_0001, 5);
        let frozen_v3_pilot = seeds(0x4842_5049_4c4f_5402, 3);
        let frozen_v3_held_out = seeds(0x4842_4652_4553_4802, 20);
        let smooth_diagnostic = seeds(0x4842_4449_4147_0001, 20);
        let production = seeds(0x4842_5052_4f44_0001, 20);
        assert!(ladder.iter().all(|seed| {
            !frozen_v3_development.contains(seed)
                && !frozen_v3_pilot.contains(seed)
                && !frozen_v3_held_out.contains(seed)
                && !smooth_diagnostic.contains(seed)
                && !production.contains(seed)
        }));
    }

    #[test]
    fn soft_endpoint_matches_composition_model_prediction() {
        let model = C3CompositionModel::new(11);
        let example = c3_examples(11, 4, 1).remove(0);
        let soft = tempered_predict(&model, &example, WinnerTemperature::Soft);
        assert_eq!(soft, model.forward(&example).prediction);
    }

    #[test]
    fn hard_endpoint_is_one_hot_winner_chain() {
        let model = C3CompositionModel::new(13);
        let example = c3_examples(13, 3, 1).remove(0);
        let mut state = example.initial_state;
        for &operation in &example.operations {
            let mut scores = [0.0f32; C3_STATES];
            scores[state] = 1.0;
            for (post, score) in scores.iter_mut().enumerate() {
                *score += model.engine.edge_w[model.edge(operation, state, post)];
            }
            state = argmax(&scores);
        }
        assert_eq!(
            tempered_predict(&model, &example, WinnerTemperature::Hard),
            state
        );
    }

    #[test]
    fn finite_temperature_is_between_soft_mass_and_hard_peak() {
        let model = C3CompositionModel::new(17);
        let example = C3Example {
            initial_state: 0,
            operations: vec![0],
            label: 1,
        };
        let soft = tempered_forward(&model, &example, WinnerTemperature::Soft);
        let tempered = tempered_forward(&model, &example, WinnerTemperature::Finite(0.5));
        let hard = tempered_forward(&model, &example, WinnerTemperature::Hard);
        assert!((tempered.iter().sum::<f32>() - 1.0).abs() < 1e-5);
        assert!((hard.iter().sum::<f32>() - 1.0).abs() < 1e-5);
        let soft_energy = soft.iter().map(|value| value * value).sum::<f32>();
        let tempered_energy = tempered.iter().map(|value| value * value).sum::<f32>();
        assert!(tempered_energy > soft_energy.min(1.0) * 0.1);
        assert_eq!(hard.iter().filter(|value| **value > 0.5).count(), 1);
    }

    #[test]
    fn no_test_updates_and_exact_replay() {
        let config = tiny_config();
        let first = run_temperature_ladder(&config);
        let second = run_temperature_ladder(&config);
        assert_eq!(first.protocol_hash, second.protocol_hash);
        assert_eq!(first.rows, second.rows);
        assert_eq!(first.summaries, second.summaries);
        assert_eq!(first.mechanisms, second.mechanisms);
        assert!(first.all_test_weights_unchanged);
        let expected_rows = (MAX_DEPTH - MIN_DEPTH + 1)
            * (config.budgets.len() * config.learning_rates.len() * config.n_seeds
                + config.learning_rates.len() * config.n_seeds * LadderArm::CONTROLS.len())
            * config.temperatures.len();
        assert_eq!(first.rows.len(), expected_rows);
        assert_eq!(first.render_sweep_csv(), second.render_sweep_csv());
        assert_eq!(first.render_mechanism_csv(), second.render_mechanism_csv());
    }
}
