//! T=2.0 winner-temperature ablation suite.
//!
//! Extends the one-task soft→hard ladder with ablations across **depth
//! schedules**, **state/area width**, and **connectivity patterns**. Fresh hash
//! family (`binn-hybrid-winner-temp-ablate-v1-*`). Does **not** reopen frozen
//! H0 (`HYBRID_NO_GO`) or authorize H1–H3 / G2 rescue.

#![allow(clippy::needless_borrow, clippy::needless_range_loop)]

use std::collections::BTreeMap;

use binn_hybrid_learn::fnv1a64;

use crate::benchmark::seeds;
use crate::teacher::LocalRng;
use crate::temperature_ladder::{LadderArm, WinnerTemperature};

pub const TEMPERATURE_ABLATION_PROTOCOL_VERSION: u32 = 1;
const ABLATE_SEED_MASTER: u64 = 0x4842_4142_4c41_0001;
const TEST_SEED_XOR: u64 = 0x4142_4c41_5445_5354;
const ACCURACY_FLOOR: f32 = 0.65;
const CONFIDENCE_Z: f32 = 1.96;
const N_OPERATIONS: usize = 2;

/// Connectivity pattern over the residual transition operator.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ConnectivityPattern {
    /// Full dense op×pre→post transitions (canonical ladder geometry).
    Dense,
    /// Keep each edge independently with probability `p` (Bernoulli mask).
    SparseBernoulli { p: f32 },
    /// Local band: keep |pre−post| ≤ radius (mod n_states) plus identity residual.
    Banded { radius: usize },
}

impl ConnectivityPattern {
    pub fn as_str(self) -> String {
        match self {
            Self::Dense => "dense".to_string(),
            Self::SparseBernoulli { p } => format!("sparse-bernoulli-{p:.2}"),
            Self::Banded { radius } => format!("banded-r{radius}"),
        }
    }

    fn tag(self) -> u8 {
        match self {
            Self::Dense => 0,
            Self::SparseBernoulli { .. } => 1,
            Self::Banded { .. } => 2,
        }
    }
}

/// One ablation cell: task/depth window × width × connectivity.
#[derive(Clone, Debug, PartialEq)]
pub struct AblationVariant {
    pub name: &'static str,
    pub n_states: usize,
    pub min_depth: usize,
    pub max_depth: usize,
    pub connectivity: ConnectivityPattern,
}

impl AblationVariant {
    pub fn suite() -> Vec<Self> {
        vec![
            Self {
                name: "baseline-dense-d1-8-s4",
                n_states: 4,
                min_depth: 1,
                max_depth: 8,
                connectivity: ConnectivityPattern::Dense,
            },
            Self {
                name: "width-s8-dense-d1-8",
                n_states: 8,
                min_depth: 1,
                max_depth: 8,
                connectivity: ConnectivityPattern::Dense,
            },
            Self {
                name: "sparse-p0.50-d1-8-s4",
                n_states: 4,
                min_depth: 1,
                max_depth: 8,
                connectivity: ConnectivityPattern::SparseBernoulli { p: 0.50 },
            },
            Self {
                name: "shallow-dense-d1-3-s4",
                n_states: 4,
                min_depth: 1,
                max_depth: 3,
                connectivity: ConnectivityPattern::Dense,
            },
            Self {
                name: "deep-dense-d5-8-s4",
                n_states: 4,
                min_depth: 5,
                max_depth: 8,
                connectivity: ConnectivityPattern::Dense,
            },
        ]
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TemperatureAblationConfig {
    pub quick: bool,
    pub n_seeds: usize,
    pub budgets: Vec<usize>,
    pub learning_rates: Vec<f32>,
    pub temperatures: Vec<WinnerTemperature>,
    pub test_examples: usize,
    pub mechanism_examples: usize,
    pub variants: Vec<AblationVariant>,
}

impl TemperatureAblationConfig {
    pub fn quick() -> Self {
        Self {
            quick: true,
            n_seeds: 2,
            budgets: vec![48],
            learning_rates: vec![0.035],
            temperatures: vec![
                WinnerTemperature::Soft,
                WinnerTemperature::Finite(2.0),
                WinnerTemperature::Hard,
            ],
            test_examples: 64,
            mechanism_examples: 8,
            variants: AblationVariant::suite(),
        }
    }

    /// Scientific-ish development grid: enough to localize T=2.0 collapse across
    /// ablations without replaying the full one-task 16.8k-row ladder.
    pub fn scientific() -> Self {
        Self {
            quick: false,
            n_seeds: 8,
            budgets: vec![240, 960],
            learning_rates: vec![0.035, 0.070],
            temperatures: vec![
                WinnerTemperature::Soft,
                WinnerTemperature::Finite(2.0),
                WinnerTemperature::Finite(1.0),
                WinnerTemperature::Hard,
            ],
            test_examples: 400,
            mechanism_examples: 32,
            variants: AblationVariant::suite(),
        }
    }

    pub fn hash(&self) -> u64 {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&TEMPERATURE_ABLATION_PROTOCOL_VERSION.to_le_bytes());
        bytes.push(u8::from(self.quick));
        for value in [self.n_seeds, self.test_examples, self.mechanism_examples] {
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
        bytes.push(0xfd);
        for variant in &self.variants {
            bytes.extend_from_slice(variant.name.as_bytes());
            bytes.push(0);
            bytes.extend_from_slice(&(variant.n_states as u64).to_le_bytes());
            bytes.extend_from_slice(&(variant.min_depth as u64).to_le_bytes());
            bytes.extend_from_slice(&(variant.max_depth as u64).to_le_bytes());
            bytes.push(variant.connectivity.tag());
            match variant.connectivity {
                ConnectivityPattern::Dense => {}
                ConnectivityPattern::SparseBernoulli { p } => {
                    bytes.extend_from_slice(&p.to_bits().to_le_bytes());
                }
                ConnectivityPattern::Banded { radius } => {
                    bytes.extend_from_slice(&(radius as u64).to_le_bytes());
                }
            }
        }
        fnv1a64(&bytes)
    }

    pub fn hash_string(&self) -> String {
        format!(
            "binn-hybrid-winner-temp-ablate-v{TEMPERATURE_ABLATION_PROTOCOL_VERSION}-{:016x}",
            self.hash()
        )
    }

    pub fn ladder_seeds(&self) -> Vec<u64> {
        seeds(ABLATE_SEED_MASTER, self.n_seeds)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AblationSweepRow {
    pub variant: String,
    pub n_states: usize,
    pub connectivity: String,
    pub seed: u64,
    pub depth: usize,
    pub budget: usize,
    pub learning_rate: f32,
    pub temperature: WinnerTemperature,
    pub arm: LadderArm,
    pub accuracy: f32,
    pub measured_nnz: usize,
    pub test_weights_unchanged: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AblationSweepSummary {
    pub variant: String,
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
pub struct AblationCollapseRow {
    pub variant: String,
    pub soft_d_star: Option<usize>,
    pub t2_d_star: Option<usize>,
    pub collapse_temperature: Option<WinnerTemperature>,
}

#[derive(Clone, Debug)]
pub struct TemperatureAblationReport {
    pub protocol_hash: String,
    pub config: TemperatureAblationConfig,
    pub seeds: Vec<u64>,
    pub rows: Vec<AblationSweepRow>,
    pub summaries: Vec<AblationSweepSummary>,
    pub collapses: Vec<AblationCollapseRow>,
    pub all_test_weights_unchanged: bool,
}

impl TemperatureAblationReport {
    pub fn render_markdown(&self) -> String {
        let mut output = format!(
            "# BINN-Hybrid T=2.0 winner-temperature ablation suite\n\n\
             - protocol: `{}`\n\
             - schedule: **{}**\n\
             - seeds: {}\n\
             - budgets: {:?}\n\
             - learning rates: {:?}\n\
             - temperatures: {}\n\
             - variants: {}\n\
             - all test weights unchanged: **{}**\n\
             - scientific gate effect: **none**\n\n\
             > Fresh ablation hash family. Canonical H0 remains `HYBRID_NO_GO`; \
             held-out seeds unused; H1–H3 stopped. This suite asks whether the \
             soft→hard transfer collapse near **T=2.0** is stable under depth \
             window, residual width, and connectivity changes. It is **not** a \
             G2 rescue and does not remassage the one-task ladder \
             `binn-hybrid-winner-temp-v1-*`.\n\n",
            self.protocol_hash,
            if self.config.quick {
                "PILOT / SMOKE"
            } else {
                "SCIENTIFIC-ISH DEVELOPMENT ABLATION"
            },
            self.seeds.len(),
            self.config.budgets,
            self.config.learning_rates,
            self.config
                .temperatures
                .iter()
                .map(|t| t.as_str())
                .collect::<Vec<_>>()
                .join(", "),
            self.config.variants.len(),
            self.all_test_weights_unchanged,
        );
        output.push_str(
            "## Ablation axes\n\n\
             | variant | n_states | depths | connectivity |\n\
             |---|---:|---|---|\n",
        );
        for v in &self.config.variants {
            output.push_str(&format!(
                "| `{}` | {} | {}–{} | {} |\n",
                v.name,
                v.n_states,
                v.min_depth,
                v.max_depth,
                v.connectivity.as_str()
            ));
        }
        output.push_str(
            "\n## Collapse summary (direct-terminal)\n\n\
             | variant | soft D* | T=2.0 D* | collapse temperature |\n\
             |---|---:|---:|---|\n",
        );
        for row in &self.collapses {
            output.push_str(&format!(
                "| `{}` | {} | {} | {} |\n",
                row.variant,
                row.soft_d_star
                    .map(|d| d.to_string())
                    .unwrap_or_else(|| "none".into()),
                row.t2_d_star
                    .map(|d| d.to_string())
                    .unwrap_or_else(|| "none".into()),
                row.collapse_temperature
                    .map(|t| t.as_str())
                    .unwrap_or_else(|| "none".into()),
            ));
        }
        output.push_str(
            "\n## Direct-terminal transfer (best depth per temperature × variant)\n\n\
             | variant | temperature | D* | mean@D* | L95@D* | budget | lr | mean nnz |\n\
             |---|---|---:|---:|---:|---:|---:|---:|\n",
        );
        for variant in &self.config.variants {
            for temperature in &self.config.temperatures {
                let d_star =
                    self.d_star_for(&variant.name, *temperature, LadderArm::DirectTerminal);
                let nnz = mean_nnz_for(&self.rows, variant.name);
                if let Some(depth) = d_star {
                    if let Some(best) = self.best_summary(variant.name, *temperature, depth) {
                        output.push_str(&format!(
                            "| `{}` | {} | {} | {:.4} | {:.4} | {} | {:.4} | {:.0} |\n",
                            variant.name,
                            temperature.as_str(),
                            depth,
                            best.mean_accuracy,
                            best.lower_95,
                            best.budget,
                            best.learning_rate,
                            nnz,
                        ));
                        continue;
                    }
                }
                output.push_str(&format!(
                    "| `{}` | {} | none | — | — | — | — | {:.0} |\n",
                    variant.name,
                    temperature.as_str(),
                    nnz,
                ));
            }
        }
        output.push_str(
            "\n## Limits\n\n\
             - Development seeds only; disjoint from frozen H0 / diagnostic families.\n\
             - Soft teacher remains a disclosed residual relaxation.\n\
             - Sparse / banded masks are fixed per seed (not learned).\n\
             - Cannot reopen H0 or authorize H1–H3 / Gate G2.\n",
        );
        output
    }

    pub fn render_sweep_csv(&self) -> String {
        let mut output = String::from(
            "variant,n_states,connectivity,seed,depth,budget,learning_rate,temperature,arm,\
             accuracy,measured_nnz,test_weights_unchanged\n",
        );
        for row in &self.rows {
            output.push_str(&format!(
                "{},{},{},{},{},{},{:.6},{},{},{:.6},{},{}\n",
                row.variant,
                row.n_states,
                row.connectivity,
                row.seed,
                row.depth,
                row.budget,
                row.learning_rate,
                row.temperature.as_str(),
                row.arm.as_str(),
                row.accuracy,
                row.measured_nnz,
                row.test_weights_unchanged,
            ));
        }
        output
    }

    fn d_star_for(
        &self,
        variant: &str,
        temperature: WinnerTemperature,
        arm: LadderArm,
    ) -> Option<usize> {
        let v = self.config.variants.iter().find(|x| x.name == variant)?;
        (v.min_depth..=v.max_depth)
            .filter(|&depth| {
                self.summaries.iter().any(|s| {
                    s.variant == variant
                        && s.temperature == temperature
                        && s.arm == arm
                        && s.depth == depth
                        && s.lower_95 >= ACCURACY_FLOOR
                })
            })
            .max()
    }

    fn best_summary(
        &self,
        variant: &str,
        temperature: WinnerTemperature,
        depth: usize,
    ) -> Option<&AblationSweepSummary> {
        self.summaries
            .iter()
            .filter(|s| {
                s.variant == variant
                    && s.temperature == temperature
                    && s.arm == LadderArm::DirectTerminal
                    && s.depth == depth
            })
            .max_by(|a, b| {
                a.lower_95
                    .partial_cmp(&b.lower_95)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }
}

fn mean_nnz_for(rows: &[AblationSweepRow], variant: &str) -> f32 {
    let vals: Vec<f32> = rows
        .iter()
        .filter(|r| r.variant == variant)
        .map(|r| r.measured_nnz as f32)
        .collect();
    if vals.is_empty() {
        0.0
    } else {
        vals.iter().sum::<f32>() / vals.len() as f32
    }
}

pub fn run_temperature_ablation(config: &TemperatureAblationConfig) -> TemperatureAblationReport {
    assert!(!config.budgets.is_empty());
    assert!(!config.learning_rates.is_empty());
    assert!(!config.temperatures.is_empty());
    assert!(!config.variants.is_empty());
    assert!(config.n_seeds >= 2);

    let protocol_hash = config.hash_string();
    let ladder_seeds = config.ladder_seeds();
    let mut rows = Vec::new();

    for variant in &config.variants {
        assert!(variant.n_states >= 2);
        assert!(variant.min_depth >= 1);
        assert!(variant.max_depth >= variant.min_depth);
        for depth in variant.min_depth..=variant.max_depth {
            for &budget in &config.budgets {
                for &learning_rate in &config.learning_rates {
                    for &seed in &ladder_seeds {
                        for arm in LadderArm::MAIN {
                            rows.extend(run_cell(
                                config,
                                variant,
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
        for depth in variant.min_depth..=variant.max_depth {
            for &learning_rate in &config.learning_rates {
                for &seed in &ladder_seeds {
                    for arm in LadderArm::CONTROLS {
                        rows.extend(run_cell(
                            config,
                            variant,
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
    }

    let summaries = summarize_rows(&rows);
    let mut collapses = Vec::new();
    for variant in &config.variants {
        let soft_d = d_star(&summaries, variant, WinnerTemperature::Soft);
        let t2_d = d_star(&summaries, variant, WinnerTemperature::Finite(2.0));
        let collapse = soft_d.and_then(|soft_depth| {
            config.temperatures.iter().copied().find(|temperature| {
                if *temperature == WinnerTemperature::Soft {
                    return false;
                }
                match d_star(&summaries, variant, *temperature) {
                    Some(depth) => depth < soft_depth,
                    None => true,
                }
            })
        });
        collapses.push(AblationCollapseRow {
            variant: variant.name.to_string(),
            soft_d_star: soft_d,
            t2_d_star: t2_d,
            collapse_temperature: collapse,
        });
    }

    TemperatureAblationReport {
        protocol_hash,
        config: config.clone(),
        seeds: ladder_seeds,
        all_test_weights_unchanged: rows.iter().all(|r| r.test_weights_unchanged),
        rows,
        summaries,
        collapses,
    }
}

fn d_star(
    summaries: &[AblationSweepSummary],
    variant: &AblationVariant,
    temperature: WinnerTemperature,
) -> Option<usize> {
    (variant.min_depth..=variant.max_depth)
        .filter(|&depth| {
            summaries.iter().any(|s| {
                s.variant == variant.name
                    && s.temperature == temperature
                    && s.arm == LadderArm::DirectTerminal
                    && s.depth == depth
                    && s.lower_95 >= ACCURACY_FLOOR
            })
        })
        .max()
}

fn run_cell(
    config: &TemperatureAblationConfig,
    variant: &AblationVariant,
    seed: u64,
    depth: usize,
    budget: usize,
    learning_rate: f32,
    arm: LadderArm,
) -> Vec<AblationSweepRow> {
    let train = make_examples(seed, variant.n_states, depth, budget);
    let test = make_examples(
        seed ^ TEST_SEED_XOR,
        variant.n_states,
        depth,
        config.test_examples,
    );
    let mut model = ResidualModel::new(seed, variant.n_states, variant.connectivity);
    let measured_nnz = model.measured_nnz();
    for example in &train {
        train_example(&mut model, example, arm, learning_rate);
    }
    let before = weight_fingerprint(&model);
    let mut rows = Vec::with_capacity(config.temperatures.len());
    for &temperature in &config.temperatures {
        let accuracy = tempered_accuracy(&model, &test, temperature);
        let after = weight_fingerprint(&model);
        rows.push(AblationSweepRow {
            variant: variant.name.to_string(),
            n_states: variant.n_states,
            connectivity: variant.connectivity.as_str(),
            seed,
            depth,
            budget,
            learning_rate,
            temperature,
            arm,
            accuracy,
            measured_nnz,
            test_weights_unchanged: before == after,
        });
    }
    rows
}

#[derive(Clone, Debug)]
struct AblationExample {
    initial_state: usize,
    operations: Vec<usize>,
    label: usize,
}

struct ResidualModel {
    n_states: usize,
    /// Flat weights: [op][pre][post]
    weights: Vec<f32>,
    /// Active edge mask (same layout); inactive edges stay zero and receive no update.
    active: Vec<bool>,
}

impl ResidualModel {
    fn new(seed: u64, n_states: usize, connectivity: ConnectivityPattern) -> Self {
        let n_edges = N_OPERATIONS * n_states * n_states;
        let mut rng = LocalRng::new(seed ^ 0xc3ab_1a7e);
        let mut weights = (0..n_edges)
            .map(|_| (rng.next_f32() * 2.0 - 1.0) * 0.04)
            .collect::<Vec<_>>();
        let mut active = vec![true; n_edges];
        match connectivity {
            ConnectivityPattern::Dense => {}
            ConnectivityPattern::SparseBernoulli { p } => {
                for (i, flag) in active.iter_mut().enumerate() {
                    let keep = rng.next_f32() < p;
                    *flag = keep;
                    if !keep {
                        weights[i] = 0.0;
                    }
                }
            }
            ConnectivityPattern::Banded { radius } => {
                for op in 0..N_OPERATIONS {
                    for pre in 0..n_states {
                        for post in 0..n_states {
                            let idx = edge_index(n_states, op, pre, post);
                            let dist = circular_dist(pre, post, n_states);
                            let keep = dist <= radius;
                            active[idx] = keep;
                            if !keep {
                                weights[idx] = 0.0;
                            }
                        }
                    }
                }
            }
        }
        Self {
            n_states,
            weights,
            active,
        }
    }

    fn measured_nnz(&self) -> usize {
        self.active.iter().filter(|&&a| a).count()
    }

    fn edge(&self, operation: usize, pre: usize, post: usize) -> usize {
        edge_index(self.n_states, operation, pre, post)
    }

    fn apply_deltas(&mut self, deltas: &[f32]) {
        for (i, (w, d)) in self.weights.iter_mut().zip(deltas).enumerate() {
            if self.active[i] {
                *w += d;
            }
        }
    }

    fn forward_soft(&self, example: &AblationExample) -> (Vec<Vec<f32>>, usize) {
        let mut state = vec![0.0f32; self.n_states];
        state[example.initial_state] = 1.0;
        let mut states = vec![state.clone()];
        for &operation in &example.operations {
            let mut next = state.clone();
            for post in 0..self.n_states {
                for pre in 0..self.n_states {
                    next[post] += self.weights[self.edge(operation, pre, post)] * state[pre];
                }
            }
            state = next;
            states.push(state.clone());
        }
        let prediction = argmax_slice(&state);
        (states, prediction)
    }

    fn teacher_deltas(
        &self,
        states: &[Vec<f32>],
        operations: &[usize],
        label: usize,
        learning_rate: f32,
    ) -> (f32, Vec<f32>) {
        let final_state = states.last().expect("terminal");
        let probabilities = softmax_slice(final_state, 1.0);
        let loss = -probabilities[label].max(1e-12).ln();
        let mut downstream: Vec<f32> = probabilities
            .iter()
            .enumerate()
            .map(|(i, &p)| f32::from(i == label) - p)
            .collect();
        let mut edge_deltas = vec![0.0f32; self.weights.len()];
        for step in (0..operations.len()).rev() {
            let operation = operations[step];
            let previous = &states[step];
            for pre in 0..self.n_states {
                for post in 0..self.n_states {
                    let idx = self.edge(operation, pre, post);
                    if self.active[idx] {
                        edge_deltas[idx] += learning_rate * previous[pre] * downstream[post];
                    }
                }
            }
            let mut previous_credit = downstream.clone();
            for pre in 0..self.n_states {
                for post in 0..self.n_states {
                    let idx = self.edge(operation, pre, post);
                    previous_credit[pre] += self.weights[idx] * downstream[post];
                }
            }
            downstream = previous_credit;
        }
        (loss, edge_deltas)
    }
}

fn edge_index(n_states: usize, operation: usize, pre: usize, post: usize) -> usize {
    (operation * n_states + pre) * n_states + post
}

fn circular_dist(a: usize, b: usize, n: usize) -> usize {
    let d = a.abs_diff(b);
    d.min(n - d)
}

fn make_examples(seed: u64, n_states: usize, depth: usize, count: usize) -> Vec<AblationExample> {
    let mut rng = LocalRng::new(seed ^ 0xc3ab_e001 ^ (n_states as u64) ^ ((depth as u64) << 8));
    (0..count)
        .map(|_| {
            let mut state = rng.index(n_states);
            let initial_state = state;
            let mut operations = Vec::with_capacity(depth);
            for _ in 0..depth {
                let op = rng.index(N_OPERATIONS);
                operations.push(op);
                state = true_transition(state, op, n_states);
            }
            AblationExample {
                initial_state,
                operations,
                label: state,
            }
        })
        .collect()
}

fn true_transition(state: usize, operation: usize, n_states: usize) -> usize {
    if operation == 0 {
        (state + 1) % n_states
    } else {
        // Fixed involution-style scramble, width-agnostic.
        (state * 3 + 1) % n_states
    }
}

fn train_example(
    model: &mut ResidualModel,
    example: &AblationExample,
    arm: LadderArm,
    learning_rate: f32,
) {
    match arm {
        LadderArm::PrivilegedIntermediate => {
            let mut state = example.initial_state;
            let mut deltas = vec![0.0f32; model.weights.len()];
            for &operation in &example.operations {
                let next = true_transition(state, operation, model.n_states);
                let local = AblationExample {
                    initial_state: state,
                    operations: vec![operation],
                    label: next,
                };
                let (states, _) = model.forward_soft(&local);
                let (_, step_deltas) =
                    model.teacher_deltas(&states, &local.operations, next, learning_rate);
                for (total, d) in deltas.iter_mut().zip(step_deltas) {
                    *total += d;
                }
                state = next;
            }
            model.apply_deltas(&deltas);
        }
        LadderArm::DirectTerminal | LadderArm::ShuffledLabel => {
            let (states, _) = model.forward_soft(example);
            let label = if arm == LadderArm::ShuffledLabel {
                (example.label + 1) % model.n_states
            } else {
                example.label
            };
            let (_, deltas) =
                model.teacher_deltas(&states, &example.operations, label, learning_rate);
            model.apply_deltas(&deltas);
        }
    }
}

fn tempered_accuracy(
    model: &ResidualModel,
    examples: &[AblationExample],
    temperature: WinnerTemperature,
) -> f32 {
    if examples.is_empty() {
        return 0.0;
    }
    let correct = examples
        .iter()
        .filter(|ex| tempered_predict(model, ex, temperature) == ex.label)
        .count();
    correct as f32 / examples.len() as f32
}

fn tempered_predict(
    model: &ResidualModel,
    example: &AblationExample,
    temperature: WinnerTemperature,
) -> usize {
    argmax_slice(&tempered_forward(model, example, temperature))
}

fn tempered_forward(
    model: &ResidualModel,
    example: &AblationExample,
    temperature: WinnerTemperature,
) -> Vec<f32> {
    let mut state = vec![0.0f32; model.n_states];
    state[example.initial_state] = 1.0;
    for &operation in &example.operations {
        let mut scores = state.clone();
        for post in 0..model.n_states {
            for pre in 0..model.n_states {
                scores[post] += model.weights[model.edge(operation, pre, post)] * state[pre];
            }
        }
        state = match temperature {
            WinnerTemperature::Soft => scores,
            WinnerTemperature::Finite(t) => softmax_slice(&scores, t),
            WinnerTemperature::Hard => {
                let mut one = vec![0.0f32; model.n_states];
                one[argmax_slice(&scores)] = 1.0;
                one
            }
        };
    }
    state
}

fn softmax_slice(values: &[f32], temperature: f32) -> Vec<f32> {
    let t = if temperature > 0.0 && temperature.is_finite() {
        temperature
    } else {
        1.0
    };
    let max_v = values.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let exps: Vec<f32> = values.iter().map(|v| ((v - max_v) / t).exp()).collect();
    let sum: f32 = exps.iter().sum::<f32>().max(1e-12);
    exps.into_iter().map(|e| e / sum).collect()
}

fn argmax_slice(values: &[f32]) -> usize {
    values
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.total_cmp(b))
        .map(|(i, _)| i)
        .unwrap_or(0)
}

fn weight_fingerprint(model: &ResidualModel) -> u64 {
    let mut bytes = Vec::with_capacity(model.weights.len() * 4);
    for w in &model.weights {
        bytes.extend_from_slice(&w.to_bits().to_le_bytes());
    }
    fnv1a64(&bytes)
}

type SweepKey = (String, usize, usize, u32, (u8, u32), LadderArm);

fn summarize_rows(rows: &[AblationSweepRow]) -> Vec<AblationSweepSummary> {
    let mut groups: BTreeMap<SweepKey, Vec<f32>> = BTreeMap::new();
    for row in rows {
        groups
            .entry((
                row.variant.clone(),
                row.depth,
                row.budget,
                row.learning_rate.to_bits(),
                row.temperature.sort_key(),
                row.arm,
            ))
            .or_default()
            .push(row.accuracy);
    }
    let mut summaries = Vec::new();
    for ((variant, depth, budget, lr_bits, temp_key, arm), accuracies) in groups {
        let n = accuracies.len() as f32;
        let mean = accuracies.iter().sum::<f32>() / n;
        let variance = if accuracies.len() > 1 {
            accuracies
                .iter()
                .map(|a| {
                    let d = a - mean;
                    d * d
                })
                .sum::<f32>()
                / (n - 1.0)
        } else {
            0.0
        };
        let se = (variance / n.max(1.0)).sqrt();
        let lower_95 = mean - CONFIDENCE_Z * se;
        let temperature = match temp_key.0 {
            0 => WinnerTemperature::Soft,
            2 => WinnerTemperature::Hard,
            _ => WinnerTemperature::Finite(f32::from_bits(temp_key.1)),
        };
        summaries.push(AblationSweepSummary {
            variant,
            depth,
            budget,
            learning_rate: f32::from_bits(lr_bits),
            temperature,
            arm,
            mean_accuracy: mean,
            variance,
            lower_95,
        });
    }
    summaries
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_sensitive_to_variant_and_schedule() {
        let a = TemperatureAblationConfig::quick();
        let mut b = a.clone();
        b.budgets[0] += 1;
        assert_ne!(a.hash_string(), b.hash_string());
        let mut c = a.clone();
        c.variants[0].n_states = 6;
        assert_ne!(a.hash_string(), c.hash_string());
        assert!(a
            .hash_string()
            .starts_with("binn-hybrid-winner-temp-ablate-v1-"));
    }

    #[test]
    fn suite_covers_at_least_two_ablation_axes() {
        let suite = AblationVariant::suite();
        let widths: std::collections::BTreeSet<_> = suite.iter().map(|v| v.n_states).collect();
        let conns: std::collections::BTreeSet<_> =
            suite.iter().map(|v| v.connectivity.as_str()).collect();
        let depths: std::collections::BTreeSet<_> =
            suite.iter().map(|v| (v.min_depth, v.max_depth)).collect();
        assert!(widths.len() >= 2, "width axis");
        assert!(conns.len() >= 2, "connectivity axis");
        assert!(depths.len() >= 2, "depth axis");
    }

    #[test]
    fn sparse_reduces_nnz_vs_dense() {
        let dense = ResidualModel::new(7, 4, ConnectivityPattern::Dense);
        let sparse = ResidualModel::new(7, 4, ConnectivityPattern::SparseBernoulli { p: 0.5 });
        assert_eq!(dense.measured_nnz(), N_OPERATIONS * 4 * 4);
        assert!(sparse.measured_nnz() < dense.measured_nnz());
        assert!(sparse.measured_nnz() > 0);
    }

    #[test]
    fn smoke_ablation_runs_and_is_deterministic() {
        let config = TemperatureAblationConfig {
            n_seeds: 2,
            budgets: vec![8],
            learning_rates: vec![0.05],
            temperatures: vec![WinnerTemperature::Soft, WinnerTemperature::Finite(2.0)],
            test_examples: 8,
            mechanism_examples: 2,
            variants: vec![AblationVariant {
                name: "tiny",
                n_states: 4,
                min_depth: 1,
                max_depth: 2,
                connectivity: ConnectivityPattern::Dense,
            }],
            ..TemperatureAblationConfig::quick()
        };
        let a = run_temperature_ablation(&config);
        let b = run_temperature_ablation(&config);
        assert_eq!(a.protocol_hash, b.protocol_hash);
        assert_eq!(a.rows, b.rows);
        assert!(a.all_test_weights_unchanged);
        assert!(!a.collapses.is_empty());
    }

    #[test]
    fn seed_family_disjoint_from_canonical_ladder() {
        let ablate = TemperatureAblationConfig::quick().ladder_seeds();
        let ladder = seeds(0x4842_5445_4d50_0001, 5);
        let h0_dev = seeds(0x4842_4445_5600_0001, 5);
        assert!(ablate
            .iter()
            .all(|s| !ladder.contains(s) && !h0_dev.contains(s)));
    }
}
