//! Development-only credit-interface diagnostics on the production event engine.
//!
//! The hard forward path uses [`Engine`], forced source spikes, delayed synaptic
//! delivery, membrane charge, and hard k-WTA winners. The terminal teacher is
//! a finite-difference-checked soft relaxation over the identical production
//! topology and weights. It receives only the terminal label. Production STDP
//! eligibility is captured from the hard event trace and used unchanged by the
//! two postsynaptic arms.
//!
//! This protocol is separate from frozen H0 and from the smooth diagnostic
//! family. It is development evidence only and cannot reopen H1-H3.

use std::collections::BTreeMap;

use binn_areas::k_wta;
use binn_core::{Csr, Rng, Tick};
use binn_data::{draw_example, true_transition, CreditDepthExample};
use binn_engine::{Cell, CellId, Engine, K};
use binn_hybrid_learn::fnv1a64;
use binn_learn::{Eligibility, PostSynapticCredit, ThreeFactor};

use crate::benchmark::seeds;

pub const PRODUCTION_DIAGNOSTIC_PROTOCOL_VERSION: u32 = 3;
const PRODUCTION_DIAGNOSTIC_SEED_MASTER: u64 = 0x4842_5052_4f44_0001;
const TEST_SEED_XOR: u64 = 0x5052_4f44_5445_5354;
const MECHANISM_SEED_XOR: u64 = 0x5052_4f44_4d45_4348;
const GRAPH_SEED_XOR: u64 = 0x5052_4f44_4752_4150;
const CONFIDENCE_Z: f32 = 1.96;
const ACCURACY_FLOOR: f32 = 0.65;
const N_STATES: usize = 4;
const N_OPERATIONS: usize = 2;
const MIN_DEPTH: usize = 1;
const MAX_DEPTH: usize = 8;
const WEIGHT_LIMIT: f32 = 2.0;

#[derive(Clone, Debug, PartialEq)]
pub struct ProductionDiagnosticConfig {
    pub quick: bool,
    pub n_seeds: usize,
    pub budgets: Vec<usize>,
    pub learning_rates: Vec<f32>,
    pub test_examples: usize,
    pub mechanism_examples: usize,
    pub init_weight: f32,
    pub eligibility_tau: f32,
}

impl ProductionDiagnosticConfig {
    pub fn quick() -> Self {
        Self {
            quick: true,
            n_seeds: 3,
            budgets: vec![60, 240],
            learning_rates: vec![0.015, 0.035],
            test_examples: 160,
            mechanism_examples: 16,
            init_weight: 0.0,
            eligibility_tau: 40.0,
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
            init_weight: 0.0,
            eligibility_tau: 40.0,
        }
    }

    pub fn hash(&self) -> u64 {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&PRODUCTION_DIAGNOSTIC_PROTOCOL_VERSION.to_le_bytes());
        bytes.push(u8::from(self.quick));
        for value in [
            self.n_seeds,
            self.test_examples,
            self.mechanism_examples,
            N_STATES,
            N_OPERATIONS,
            MIN_DEPTH,
            MAX_DEPTH,
        ] {
            bytes.extend_from_slice(&(value as u64).to_le_bytes());
        }
        bytes.extend_from_slice(&self.init_weight.to_bits().to_le_bytes());
        bytes.extend_from_slice(&self.eligibility_tau.to_bits().to_le_bytes());
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
            "binn-hybrid-production-diagnostic-v{PRODUCTION_DIAGNOSTIC_PROTOCOL_VERSION}-{:016x}",
            self.hash()
        )
    }

    pub fn diagnostic_seeds(&self) -> Vec<u64> {
        seeds(PRODUCTION_DIAGNOSTIC_SEED_MASTER, self.n_seeds)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProductionDiagnosticArm {
    ExistingPostSynaptic,
    OraclePostSynaptic,
    DirectTerminal,
    PrivilegedIntermediate,
    ShuffledLabel,
}

impl ProductionDiagnosticArm {
    pub const MAIN: [Self; 3] = [
        Self::ExistingPostSynaptic,
        Self::OraclePostSynaptic,
        Self::DirectTerminal,
    ];
    pub const CONTROLS: [Self; 2] = [Self::PrivilegedIntermediate, Self::ShuffledLabel];
    pub const ALL: [Self; 5] = [
        Self::ExistingPostSynaptic,
        Self::OraclePostSynaptic,
        Self::DirectTerminal,
        Self::PrivilegedIntermediate,
        Self::ShuffledLabel,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::ExistingPostSynaptic => "production-existing-post-synaptic",
            Self::OraclePostSynaptic => "production-least-squares-post-synaptic",
            Self::DirectTerminal => "production-direct-terminal",
            Self::PrivilegedIntermediate => "production-privileged-intermediate-target",
            Self::ShuffledLabel => "production-shuffled-label",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProductionSweepRow {
    pub seed: u64,
    pub depth: usize,
    pub budget: usize,
    pub learning_rate: f32,
    pub arm: ProductionDiagnosticArm,
    pub accuracy: f32,
    pub test_weights_unchanged: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProductionSweepSummary {
    pub depth: usize,
    pub budget: usize,
    pub learning_rate: f32,
    pub arm: ProductionDiagnosticArm,
    pub mean_accuracy: f32,
    pub variance: f32,
    pub lower_95: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProductionMechanismRow {
    pub seed: u64,
    pub depth: usize,
    pub gradient_norm: f32,
    pub surrogate_direct_loss_drop: f32,
    pub surrogate_rotated_loss_drop: f32,
    pub event_direct_loss_drop: f32,
    pub event_rotated_loss_drop: f32,
    pub existing_cosine: f32,
    pub oracle_cosine: f32,
    pub existing_mse: f32,
    pub oracle_mse: f32,
    pub existing_sign_agreement: f32,
    pub oracle_sign_agreement: f32,
    pub eligibility_nonzero_fraction: f32,
    pub target_energy_on_eligible_edges: f32,
}

#[derive(Clone, Debug)]
pub struct ProductionDiagnosticReport {
    pub protocol_hash: String,
    pub config: ProductionDiagnosticConfig,
    pub seeds: Vec<u64>,
    pub rows: Vec<ProductionSweepRow>,
    pub summaries: Vec<ProductionSweepSummary>,
    pub mechanisms: Vec<ProductionMechanismRow>,
    pub best_d_star: Vec<(ProductionDiagnosticArm, Option<usize>)>,
    pub all_test_weights_unchanged: bool,
    pub production_forward_contract_passed: bool,
}

impl ProductionDiagnosticReport {
    pub fn render_markdown(&self) -> String {
        let mut output = format!(
            "# BINN-Hybrid terminal-credit hierarchy on the production event engine\n\n\
             - protocol: `{}`\n\
             - schedule: **{}**\n\
             - seeds: {}\n\
             - depths: {} through {}\n\
             - budgets: {:?}\n\
             - learning rates: {:?}\n\
             - frozen test examples per cell: {}\n\
             - event-forward contract: **{}**\n\
             - all test weights unchanged: **{}**\n\
             - scientific gate effect: **none**\n\n\
             > Development-only successor diagnostic. Canonical C1 protocol v2 \
             remains a G2 failure and frozen H0 remains `HYBRID_NO_GO`. These \
             seeds are disjoint from H0, the smooth diagnostic, and the unused \
             held-out family. H1-H3 remain stopped.\n\n",
            self.protocol_hash,
            if self.config.quick {
                "PILOT"
            } else {
                "FULL DEVELOPMENT DIAGNOSTIC"
            },
            self.seeds.len(),
            MIN_DEPTH,
            MAX_DEPTH,
            self.config.budgets,
            self.config.learning_rates,
            self.config.test_examples,
            if self.production_forward_contract_passed {
                "PASS"
            } else {
                "FAIL"
            },
            self.all_test_weights_unchanged,
        );
        output.push_str(
            "## Mechanism contract\n\n\
             The evaluated forward path is the production event engine: forced \
             source spikes, delayed CSR synaptic delivery, membrane charge, and \
             hard k-WTA winners. One recurrent transition graph is reused at \
             every composition step, matching the shared transition parameters \
             of the smooth diagnostic. Its identity residual is delivered as a \
             real weighted external event. The terminal teacher is the exact \
             gradient of the original residual relaxation over that same topology \
             and the live production weights. It sees only the final label and \
             its gradient is checked by \
             central finite differences. The two postsynaptic arms use actual \
             production STDP eligibility captured from that hard event trace.\n\n\
             The privileged control receives true per-layer states and therefore \
             has up to `depth` supervised corrections. It is an inadmissible \
             solvability ceiling, not a matched learner.\n\n",
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
        for depth in MIN_DEPTH..=MAX_DEPTH {
            for arm in ProductionDiagnosticArm::MAIN {
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
        for depth in MIN_DEPTH..=MAX_DEPTH {
            if let Some(best_direct) =
                self.best_summary(depth, ProductionDiagnosticArm::DirectTerminal)
            {
                let direct_oracle = self.paired_effect(
                    depth,
                    best_direct.budget,
                    best_direct.learning_rate,
                    ProductionDiagnosticArm::DirectTerminal,
                    ProductionDiagnosticArm::OraclePostSynaptic,
                );
                let oracle_existing = self.paired_effect(
                    depth,
                    best_direct.budget,
                    best_direct.learning_rate,
                    ProductionDiagnosticArm::OraclePostSynaptic,
                    ProductionDiagnosticArm::ExistingPostSynaptic,
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
        for depth in MIN_DEPTH..=MAX_DEPTH {
            for arm in ProductionDiagnosticArm::CONTROLS {
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
             | depth | gradient norm | surrogate direct drop | surrogate rotated drop | event direct drop | event rotated drop | existing cosine | oracle cosine | existing MSE | oracle MSE | existing sign | oracle sign | eligibility support | target energy eligible |\n\
             |---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|\n",
        );
        for depth in MIN_DEPTH..=MAX_DEPTH {
            let rows = self
                .mechanisms
                .iter()
                .filter(|row| row.depth == depth)
                .collect::<Vec<_>>();
            output.push_str(&format!(
                "| {} | {:.6} | {:.6} | {:.6} | {:.6} | {:.6} | {:.4} | {:.4} | {:.8} | {:.8} | {:.4} | {:.4} | {:.4} | {:.4} |\n",
                depth,
                mean_refs(&rows, |row| row.gradient_norm),
                mean_refs(&rows, |row| row.surrogate_direct_loss_drop),
                mean_refs(&rows, |row| row.surrogate_rotated_loss_drop),
                mean_refs(&rows, |row| row.event_direct_loss_drop),
                mean_refs(&rows, |row| row.event_rotated_loss_drop),
                mean_refs(&rows, |row| row.existing_cosine),
                mean_refs(&rows, |row| row.oracle_cosine),
                mean_refs(&rows, |row| row.existing_mse),
                mean_refs(&rows, |row| row.oracle_mse),
                mean_refs(&rows, |row| row.existing_sign_agreement),
                mean_refs(&rows, |row| row.oracle_sign_agreement),
                mean_refs(&rows, |row| row.eligibility_nonzero_fraction),
                mean_refs(&rows, |row| row.target_energy_on_eligible_edges),
            ));
        }
        output.push_str(
            "\n## Interpretation limits\n\n\
             - Best configurations are selected on these development seeds; D* \
             values are descriptive, not confirmatory.\n\
             - Hard k-WTA is nondifferentiable. The terminal teacher is a \
             disclosed differentiable relaxation, not a derivative of the \
             discontinuous winner operation.\n\
             - The production hierarchy reproduces only if ordering and controls \
             agree; exact D* equality with the smooth diagnostic is not required.\n\
             - Raw seed-level sweep and mechanism rows are emitted beside this report.\n",
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
        let mut output = "protocol,seed,depth,gradient_norm,surrogate_direct_loss_drop,surrogate_rotated_loss_drop,event_direct_loss_drop,event_rotated_loss_drop,existing_cosine,oracle_cosine,existing_mse,oracle_mse,existing_sign_agreement,oracle_sign_agreement,eligibility_nonzero_fraction,target_energy_on_eligible_edges\n".to_string();
        for row in &self.mechanisms {
            output.push_str(&format!(
                "{},{:016x},{},{:.9},{:.9},{:.9},{:.9},{:.9},{:.9},{:.9},{:.9},{:.9},{:.9},{:.9},{:.9},{:.9}\n",
                self.protocol_hash,
                row.seed,
                row.depth,
                row.gradient_norm,
                row.surrogate_direct_loss_drop,
                row.surrogate_rotated_loss_drop,
                row.event_direct_loss_drop,
                row.event_rotated_loss_drop,
                row.existing_cosine,
                row.oracle_cosine,
                row.existing_mse,
                row.oracle_mse,
                row.existing_sign_agreement,
                row.oracle_sign_agreement,
                row.eligibility_nonzero_fraction,
                row.target_energy_on_eligible_edges,
            ));
        }
        output
    }

    fn best_summary(
        &self,
        depth: usize,
        arm: ProductionDiagnosticArm,
    ) -> Option<&ProductionSweepSummary> {
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
        treatment: ProductionDiagnosticArm,
        control: ProductionDiagnosticArm,
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
        paired_lower_95(&treatment_by_seed, &control_by_seed)
    }
}

pub fn run_production_diagnostics(
    config: &ProductionDiagnosticConfig,
) -> ProductionDiagnosticReport {
    assert!(!config.budgets.is_empty());
    assert!(!config.learning_rates.is_empty());
    assert!(config.n_seeds >= 2);
    assert!(config.eligibility_tau > 0.0);
    let protocol_hash = config.hash_string();
    let diagnostic_seeds = config.diagnostic_seeds();
    let production_forward_contract_passed = verify_forward_contract(config, &diagnostic_seeds);
    assert!(
        production_forward_contract_passed,
        "production forward contract failed"
    );

    let mut rows = Vec::new();
    for depth in MIN_DEPTH..=MAX_DEPTH {
        for &budget in &config.budgets {
            for &learning_rate in &config.learning_rates {
                for &seed in &diagnostic_seeds {
                    let split = FrozenSplit::new(seed, depth, budget, config.test_examples);
                    for arm in ProductionDiagnosticArm::MAIN {
                        rows.push(run_cell(
                            config,
                            seed,
                            depth,
                            budget,
                            learning_rate,
                            &split,
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
            for &seed in &diagnostic_seeds {
                let split = FrozenSplit::new(seed, depth, max_budget, config.test_examples);
                for arm in ProductionDiagnosticArm::CONTROLS {
                    rows.push(run_cell(
                        config,
                        seed,
                        depth,
                        max_budget,
                        learning_rate,
                        &split,
                        arm,
                    ));
                }
            }
        }
    }

    let summaries = summarize_rows(&rows);
    let mechanisms = run_mechanisms(config, &diagnostic_seeds);
    let best_d_star = ProductionDiagnosticArm::ALL
        .into_iter()
        .map(|arm| {
            let depth = (MIN_DEPTH..=MAX_DEPTH)
                .filter(|&depth| {
                    summaries
                        .iter()
                        .filter(|summary| summary.depth == depth && summary.arm == arm)
                        .any(|summary| summary.lower_95 >= ACCURACY_FLOOR)
                })
                .max();
            (arm, depth)
        })
        .collect();
    let all_test_weights_unchanged = rows.iter().all(|row| row.test_weights_unchanged);
    ProductionDiagnosticReport {
        protocol_hash,
        config: config.clone(),
        seeds: diagnostic_seeds,
        rows,
        summaries,
        mechanisms,
        best_d_star,
        all_test_weights_unchanged,
        production_forward_contract_passed,
    }
}

#[derive(Clone)]
struct FrozenSplit {
    train: Vec<CreditDepthExample>,
    test: Vec<CreditDepthExample>,
}

impl FrozenSplit {
    fn new(seed: u64, depth: usize, train_count: usize, test_count: usize) -> Self {
        let mut train_rng = Rng::new(seed ^ (depth as u64).wrapping_mul(0xd1b5_4a32_d192_ed03));
        let mut test_rng =
            Rng::new(seed ^ TEST_SEED_XOR ^ (depth as u64).wrapping_mul(0xbf58_476d_1ce4_e5b9));
        let train = (0..train_count)
            .map(|_| draw_example(&mut train_rng, depth, N_STATES))
            .collect();
        let test = (0..test_count)
            .map(|_| draw_example(&mut test_rng, depth, N_STATES))
            .collect();
        Self { train, test }
    }
}

fn run_cell(
    config: &ProductionDiagnosticConfig,
    seed: u64,
    depth: usize,
    budget: usize,
    learning_rate: f32,
    split: &FrozenSplit,
    arm: ProductionDiagnosticArm,
) -> ProductionSweepRow {
    let mut graph = ProductionGraph::new(config, seed, depth);
    for example in &split.train {
        graph.train(example, arm, learning_rate);
    }
    let before_test = fingerprint_weights(&graph.engine.edge_w);
    let correct = split
        .test
        .iter()
        .filter(|example| graph.predict(example) == example.target)
        .count();
    let after_test = fingerprint_weights(&graph.engine.edge_w);
    ProductionSweepRow {
        seed,
        depth,
        budget,
        learning_rate,
        arm,
        accuracy: correct as f32 / split.test.len().max(1) as f32,
        test_weights_unchanged: before_test == after_test,
    }
}

struct ProductionGraph {
    engine: Engine,
    learner: ThreeFactor,
    depth: usize,
    t_cursor: Tick,
    eligibility_tau: f32,
}

impl ProductionGraph {
    fn new(config: &ProductionDiagnosticConfig, seed: u64, depth: usize) -> Self {
        let n_sources = N_STATES * N_OPERATIONS;
        let n_cells = n_sources + N_STATES;
        let mut rows = vec![Vec::<u32>::new(); n_cells];
        for state in 0..N_STATES {
            for operation in 0..N_OPERATIONS {
                let source = state * N_OPERATIONS + operation;
                rows[source].extend((0..N_STATES).map(|next| (n_sources + next) as CellId));
            }
        }
        let conn = Csr::from_adjacency(&rows);
        let mut rng = Rng::new(seed ^ GRAPH_SEED_XOR);
        let weights = (0..conn.nnz())
            .map(|_| config.init_weight + (rng.next_f32() - 0.5) * 0.02)
            .collect();
        let mut engine = Engine::with_cells(n_cells);
        engine.set_connectivity(conn, weights);
        Self {
            engine,
            learner: ThreeFactor::new(1.0, 0.0, config.eligibility_tau),
            depth,
            t_cursor: 1,
            eligibility_tau: config.eligibility_tau,
        }
    }

    fn source(&self, state: usize, operation: usize) -> CellId {
        (state * N_OPERATIONS + operation) as CellId
    }

    fn output(&self, state: usize) -> CellId {
        (N_STATES * N_OPERATIONS + state) as CellId
    }

    fn edge(&self, state: usize, operation: usize, post: usize) -> usize {
        edge_index(
            &self.engine.conn,
            self.source(state, operation),
            self.output(post),
        )
        .expect("production layer edge")
    }

    fn forward(&mut self, example: &CreditDepthExample) -> EventTrace {
        assert_eq!(example.operations.len(), self.depth);
        clear_eligibility(&mut self.engine);
        self.learner.reset_pairing_state();
        let saved_cells = self.engine.cells().to_vec();
        let mut state = example.start;
        let mut layers = Vec::with_capacity(self.depth);
        let mut at = self.t_cursor;
        for &operation in &example.operations {
            let source = self.source(state, operation);
            let outputs = (0..N_STATES)
                .map(|next| self.output(next))
                .collect::<Vec<_>>();
            for &output in &outputs {
                self.engine.cell_mut(output).theta = f32::INFINITY;
            }
            self.engine.force_spike(source, at);
            let score_at = at + self.engine.max_synaptic_delay().max(1);
            self.engine
                .inject_weighted(self.output(state), 0, score_at, 1.0);
            let _ = self.engine.step_until(score_at);
            let scores = outputs
                .iter()
                .map(|&output| {
                    self.engine.cell_mut(output).advance_to(score_at);
                    self.engine.cell(output).v
                })
                .collect::<Vec<_>>();
            let scored = outputs
                .iter()
                .copied()
                .zip(scores.iter().copied())
                .collect::<Vec<_>>();
            let winner = k_wta(&scored, 1)[0];
            let predicted_next = winner as usize - N_STATES * N_OPERATIONS;
            let true_next = true_transition(state, operation, N_STATES);
            for &output in &outputs {
                self.engine.cell_mut(output).v = 0.0;
            }
            let winner_at = score_at + 1;
            self.engine.force_spike(winner, winner_at);
            let _ = self.engine.step_until(winner_at + 1);
            layers.push(EventLayerTrace {
                source,
                scores,
                predicted_next,
                true_next,
            });
            state = predicted_next;
            self.restore_cells(&saved_cells);
            at = winner_at + 3;
        }
        EventTrace {
            prediction: state,
            layers,
            saved_cells,
        }
    }

    fn capture_production_eligibility(&mut self) -> Vec<f32> {
        self.learner.observe_spikes(&mut self.engine);
        let now = self.engine.time();
        Eligibility::new(self.eligibility_tau).decay_all_to(self.engine.syn.as_mut_slice(), now);
        self.engine
            .syn
            .as_slice()
            .iter()
            .map(|synapse| synapse.eligibility)
            .collect()
    }

    fn terminal_teacher(
        &self,
        example: &CreditDepthExample,
        terminal_label: usize,
        learning_rate: f32,
    ) -> SoftTeacherTargets {
        assert!(terminal_label < N_STATES);
        let mut probabilities = Vec::with_capacity(self.depth + 1);
        let mut state = vec![0.0f32; N_STATES];
        state[example.start] = 1.0;
        probabilities.push(state.clone());
        for &operation in &example.operations {
            let mut next = state.clone();
            for (pre, &pre_probability) in state.iter().enumerate() {
                for (post, next_value) in next.iter_mut().enumerate() {
                    *next_value +=
                        self.engine.edge_w[self.edge(pre, operation, post)] * pre_probability;
                }
            }
            state = next;
            probabilities.push(state.clone());
        }
        let terminal_probabilities = softmax(&state);
        let loss = -terminal_probabilities[terminal_label].max(1e-12).ln();
        let mut edge_deltas = vec![0.0f32; self.engine.edge_w.len()];
        let mut post_credits = vec![0.0f32; self.engine.num_cells()];
        let mut delta = terminal_probabilities
            .iter()
            .enumerate()
            .map(|(index, probability)| f32::from(index == terminal_label) - probability)
            .collect::<Vec<_>>();
        for layer in (0..self.depth).rev() {
            let operation = example.operations[layer];
            let previous = &probabilities[layer];
            for (post, &value) in delta.iter().enumerate() {
                post_credits[self.output(post) as usize] += learning_rate * value;
            }
            for (pre, &pre_probability) in previous.iter().enumerate() {
                for (post, &post_delta) in delta.iter().enumerate() {
                    let edge = self.edge(pre, operation, post);
                    edge_deltas[edge] += learning_rate * pre_probability * post_delta;
                }
            }
            if layer > 0 {
                let mut previous_credit = delta.clone();
                for (pre, credit) in previous_credit.iter_mut().enumerate() {
                    for (post, &post_delta) in delta.iter().enumerate() {
                        *credit += self.engine.edge_w[self.edge(pre, operation, post)] * post_delta;
                    }
                }
                delta = previous_credit;
            }
        }
        SoftTeacherTargets {
            loss,
            edge_deltas,
            post_credits,
        }
    }

    fn privileged_deltas(&self, trace: &EventTrace, learning_rate: f32) -> Vec<f32> {
        let mut deltas = vec![0.0f32; self.engine.edge_w.len()];
        for layer in &trace.layers {
            let probabilities = softmax(&layer.scores);
            for (post, probability) in probabilities.into_iter().enumerate() {
                let edge = edge_index(&self.engine.conn, layer.source, self.output(post))
                    .expect("active production edge");
                deltas[edge] += learning_rate * (f32::from(post == layer.true_next) - probability);
            }
        }
        deltas
    }

    fn apply_post_credit(&mut self, values: &[f32]) {
        let signal = PostSynapticCredit::from_values(values.to_vec());
        let _ = self
            .learner
            .update_with_credit_counted(&mut self.engine, &signal);
        self.clamp_weights();
    }

    fn apply_deltas(&mut self, deltas: &[f32]) {
        assert_eq!(deltas.len(), self.engine.edge_w.len());
        for (edge, &delta) in deltas.iter().enumerate() {
            if delta == 0.0 {
                continue;
            }
            let weight = (self.engine.edge_w[edge] + delta).clamp(-WEIGHT_LIMIT, WEIGHT_LIMIT);
            self.engine.edge_w[edge] = weight;
            self.engine.syn.as_mut_slice()[edge].weight = weight;
        }
    }

    fn clamp_weights(&mut self) {
        for (edge, weight) in self.engine.edge_w.iter_mut().enumerate() {
            *weight = weight.clamp(-WEIGHT_LIMIT, WEIGHT_LIMIT);
            self.engine.syn.as_mut_slice()[edge].weight = *weight;
        }
    }

    fn finish_trial(&mut self, trace: &EventTrace) {
        self.restore_cells(&trace.saved_cells);
        clear_eligibility(&mut self.engine);
        self.learner.reset_pairing_state();
        self.engine.close_inhibited_cycle();
        self.t_cursor = self.engine.time() + 5;
    }

    fn restore_cells(&mut self, saved_cells: &[Cell]) {
        let now = self.engine.time();
        for (index, previous) in saved_cells.iter().enumerate() {
            let cell = self.engine.cell_mut(index as CellId);
            cell.v = previous.v;
            cell.v_dend = [0.0; K];
            cell.theta = previous.theta;
            cell.tau_m = previous.tau_m;
            cell.tau_d = previous.tau_d;
            cell.g_c = previous.g_c;
            cell.branches = previous.branches;
            cell.last = now;
        }
    }

    fn train(
        &mut self,
        example: &CreditDepthExample,
        arm: ProductionDiagnosticArm,
        learning_rate: f32,
    ) {
        let trace = self.forward(example);
        let eligibility = self.capture_production_eligibility();
        match arm {
            ProductionDiagnosticArm::ExistingPostSynaptic
            | ProductionDiagnosticArm::OraclePostSynaptic
            | ProductionDiagnosticArm::DirectTerminal
            | ProductionDiagnosticArm::ShuffledLabel => {
                let label = if arm == ProductionDiagnosticArm::ShuffledLabel {
                    (example.target + 1) % N_STATES
                } else {
                    example.target
                };
                let targets = self.terminal_teacher(example, label, learning_rate);
                let factorization =
                    production_factorization(&targets, &eligibility, &self.engine.conn.col);
                match arm {
                    ProductionDiagnosticArm::ExistingPostSynaptic => {
                        self.apply_post_credit(&targets.post_credits);
                    }
                    ProductionDiagnosticArm::OraclePostSynaptic => {
                        self.apply_post_credit(&factorization.oracle_post_credits);
                    }
                    ProductionDiagnosticArm::DirectTerminal
                    | ProductionDiagnosticArm::ShuffledLabel => {
                        self.apply_deltas(&targets.edge_deltas);
                    }
                    ProductionDiagnosticArm::PrivilegedIntermediate => unreachable!(),
                }
            }
            ProductionDiagnosticArm::PrivilegedIntermediate => {
                let deltas = self.privileged_deltas(&trace, learning_rate);
                self.apply_deltas(&deltas);
            }
        }
        self.finish_trial(&trace);
    }

    fn predict(&mut self, example: &CreditDepthExample) -> usize {
        let trace = self.forward(example);
        let prediction = trace.prediction;
        let _ = self.capture_production_eligibility();
        self.finish_trial(&trace);
        prediction
    }

    fn event_loss(trace: &EventTrace, label: usize) -> f32 {
        let probabilities = softmax(&trace.layers.last().expect("depth >= 1").scores);
        -probabilities[label].max(1e-12).ln()
    }
}

#[derive(Clone)]
struct EventLayerTrace {
    source: CellId,
    scores: Vec<f32>,
    predicted_next: usize,
    true_next: usize,
}

#[derive(Clone)]
struct EventTrace {
    prediction: usize,
    layers: Vec<EventLayerTrace>,
    saved_cells: Vec<Cell>,
}

#[derive(Clone)]
struct SoftTeacherTargets {
    loss: f32,
    edge_deltas: Vec<f32>,
    post_credits: Vec<f32>,
}

struct ProductionFactorization {
    oracle_post_credits: Vec<f32>,
    existing_cosine: f32,
    oracle_cosine: f32,
    existing_mse: f32,
    oracle_mse: f32,
    existing_sign_agreement: f32,
    oracle_sign_agreement: f32,
    eligibility_nonzero_fraction: f32,
    target_energy_on_eligible_edges: f32,
}

fn production_factorization(
    targets: &SoftTeacherTargets,
    eligibility: &[f32],
    edge_posts: &[u32],
) -> ProductionFactorization {
    assert_eq!(targets.edge_deltas.len(), eligibility.len());
    assert_eq!(targets.edge_deltas.len(), edge_posts.len());
    let existing_deltas = eligibility
        .iter()
        .zip(edge_posts)
        .map(|(edge_eligibility, post)| edge_eligibility * targets.post_credits[*post as usize])
        .collect::<Vec<_>>();
    let mut numerators = vec![0.0f32; targets.post_credits.len()];
    let mut denominators = vec![0.0f32; targets.post_credits.len()];
    for ((&edge_eligibility, &target), &post) in
        eligibility.iter().zip(&targets.edge_deltas).zip(edge_posts)
    {
        numerators[post as usize] += edge_eligibility * target;
        denominators[post as usize] += edge_eligibility * edge_eligibility;
    }
    let oracle_post_credits = numerators
        .into_iter()
        .zip(denominators)
        .map(|(numerator, denominator)| {
            if denominator > 1e-12 {
                numerator / denominator
            } else {
                0.0
            }
        })
        .collect::<Vec<_>>();
    let oracle_deltas = eligibility
        .iter()
        .zip(edge_posts)
        .map(|(edge_eligibility, post)| edge_eligibility * oracle_post_credits[*post as usize])
        .collect::<Vec<_>>();
    let nonzero = eligibility
        .iter()
        .filter(|value| value.abs() > 1e-8)
        .count();
    let total_energy = targets
        .edge_deltas
        .iter()
        .map(|value| value * value)
        .sum::<f32>();
    let eligible_energy = targets
        .edge_deltas
        .iter()
        .zip(eligibility)
        .filter(|(_, value)| value.abs() > 1e-8)
        .map(|(target, _)| target * target)
        .sum::<f32>();
    ProductionFactorization {
        existing_cosine: cosine(&existing_deltas, &targets.edge_deltas),
        oracle_cosine: cosine(&oracle_deltas, &targets.edge_deltas),
        existing_mse: mse(&existing_deltas, &targets.edge_deltas),
        oracle_mse: mse(&oracle_deltas, &targets.edge_deltas),
        existing_sign_agreement: sign_agreement(&existing_deltas, &targets.edge_deltas),
        oracle_sign_agreement: sign_agreement(&oracle_deltas, &targets.edge_deltas),
        eligibility_nonzero_fraction: nonzero as f32 / eligibility.len().max(1) as f32,
        target_energy_on_eligible_edges: if total_energy > 1e-20 {
            eligible_energy / total_energy
        } else {
            0.0
        },
        oracle_post_credits,
    }
}

fn run_mechanisms(
    config: &ProductionDiagnosticConfig,
    diagnostic_seeds: &[u64],
) -> Vec<ProductionMechanismRow> {
    let mut rows = Vec::new();
    for depth in MIN_DEPTH..=MAX_DEPTH {
        for &seed in diagnostic_seeds {
            let mut rng = Rng::new(
                seed ^ MECHANISM_SEED_XOR ^ (depth as u64).wrapping_mul(0x94d0_49bb_1331_11eb),
            );
            let mut values = MechanismAccumulator::default();
            for index in 0..config.mechanism_examples {
                let example = draw_example(&mut rng, depth, N_STATES);
                let graph_seed =
                    seed ^ MECHANISM_SEED_XOR ^ (index as u64).wrapping_mul(0x9e37_79b9);
                let mut direct = ProductionGraph::new(config, graph_seed, depth);
                let trace = direct.forward(&example);
                let event_loss_before = ProductionGraph::event_loss(&trace, example.target);
                let eligibility = direct.capture_production_eligibility();
                let targets = direct.terminal_teacher(&example, example.target, 0.001);
                let factorization =
                    production_factorization(&targets, &eligibility, &direct.engine.conn.col);
                values.gradient_norm.push(
                    targets
                        .edge_deltas
                        .iter()
                        .map(|delta| delta * delta)
                        .sum::<f32>()
                        .sqrt()
                        / 0.001,
                );
                values.existing_cosine.push(factorization.existing_cosine);
                values.oracle_cosine.push(factorization.oracle_cosine);
                values.existing_mse.push(factorization.existing_mse);
                values.oracle_mse.push(factorization.oracle_mse);
                values
                    .existing_sign
                    .push(factorization.existing_sign_agreement);
                values.oracle_sign.push(factorization.oracle_sign_agreement);
                values
                    .eligibility_support
                    .push(factorization.eligibility_nonzero_fraction);
                values
                    .target_energy_eligible
                    .push(factorization.target_energy_on_eligible_edges);
                direct.apply_deltas(&targets.edge_deltas);
                direct.finish_trial(&trace);
                let surrogate_after = direct.terminal_teacher(&example, example.target, 1.0).loss;
                let after_trace = direct.forward(&example);
                let event_after = ProductionGraph::event_loss(&after_trace, example.target);
                let _ = direct.capture_production_eligibility();
                direct.finish_trial(&after_trace);
                values
                    .surrogate_direct_drop
                    .push(targets.loss - surrogate_after);
                values
                    .event_direct_drop
                    .push(event_loss_before - event_after);

                let mut rotated = ProductionGraph::new(config, graph_seed, depth);
                let rotated_trace = rotated.forward(&example);
                let rotated_event_before =
                    ProductionGraph::event_loss(&rotated_trace, example.target);
                let _ = rotated.capture_production_eligibility();
                let rotated_targets = rotated.terminal_teacher(&example, example.target, 0.001);
                let mut rotated_deltas = rotated_targets.edge_deltas.clone();
                rotate_nonzero_blocks(&mut rotated_deltas);
                rotated.apply_deltas(&rotated_deltas);
                rotated.finish_trial(&rotated_trace);
                let rotated_surrogate_after =
                    rotated.terminal_teacher(&example, example.target, 1.0).loss;
                let rotated_after_trace = rotated.forward(&example);
                let rotated_event_after =
                    ProductionGraph::event_loss(&rotated_after_trace, example.target);
                let _ = rotated.capture_production_eligibility();
                rotated.finish_trial(&rotated_after_trace);
                values
                    .surrogate_rotated_drop
                    .push(rotated_targets.loss - rotated_surrogate_after);
                values
                    .event_rotated_drop
                    .push(rotated_event_before - rotated_event_after);
            }
            rows.push(values.finish(seed, depth));
        }
    }
    rows
}

#[derive(Default)]
struct MechanismAccumulator {
    gradient_norm: Vec<f32>,
    surrogate_direct_drop: Vec<f32>,
    surrogate_rotated_drop: Vec<f32>,
    event_direct_drop: Vec<f32>,
    event_rotated_drop: Vec<f32>,
    existing_cosine: Vec<f32>,
    oracle_cosine: Vec<f32>,
    existing_mse: Vec<f32>,
    oracle_mse: Vec<f32>,
    existing_sign: Vec<f32>,
    oracle_sign: Vec<f32>,
    eligibility_support: Vec<f32>,
    target_energy_eligible: Vec<f32>,
}

impl MechanismAccumulator {
    fn finish(self, seed: u64, depth: usize) -> ProductionMechanismRow {
        ProductionMechanismRow {
            seed,
            depth,
            gradient_norm: mean(&self.gradient_norm),
            surrogate_direct_loss_drop: mean(&self.surrogate_direct_drop),
            surrogate_rotated_loss_drop: mean(&self.surrogate_rotated_drop),
            event_direct_loss_drop: mean(&self.event_direct_drop),
            event_rotated_loss_drop: mean(&self.event_rotated_drop),
            existing_cosine: mean(&self.existing_cosine),
            oracle_cosine: mean(&self.oracle_cosine),
            existing_mse: mean(&self.existing_mse),
            oracle_mse: mean(&self.oracle_mse),
            existing_sign_agreement: mean(&self.existing_sign),
            oracle_sign_agreement: mean(&self.oracle_sign),
            eligibility_nonzero_fraction: mean(&self.eligibility_support),
            target_energy_on_eligible_edges: mean(&self.target_energy_eligible),
        }
    }
}

fn rotate_nonzero_blocks(values: &mut [f32]) {
    let nonzero = values
        .iter()
        .enumerate()
        .filter_map(|(index, value)| (value.abs() > 1e-12).then_some(index))
        .collect::<Vec<_>>();
    if nonzero.len() < 2 {
        values.rotate_left(1);
        return;
    }
    let original = nonzero
        .iter()
        .map(|&index| values[index])
        .collect::<Vec<_>>();
    for (position, &index) in nonzero.iter().enumerate() {
        values[index] = original[(position + 1) % original.len()];
    }
}

fn verify_forward_contract(config: &ProductionDiagnosticConfig, diagnostic_seeds: &[u64]) -> bool {
    for depth in MIN_DEPTH..=MAX_DEPTH {
        for &seed in diagnostic_seeds {
            let split = FrozenSplit::new(seed, depth, 1, 1);
            let original = &split.train[0];
            let mut altered = original.clone();
            altered.target = (altered.target + 1) % N_STATES;
            let mut first = ProductionGraph::new(config, seed, depth);
            let mut second = ProductionGraph::new(config, seed, depth);
            if fingerprint_weights(&first.engine.edge_w)
                != fingerprint_weights(&second.engine.edge_w)
            {
                return false;
            }
            let first_trace = first.forward(original);
            let second_trace = second.forward(&altered);
            if event_signature(&first_trace) != event_signature(&second_trace) {
                return false;
            }
        }
    }
    true
}

fn event_signature(trace: &EventTrace) -> (usize, Vec<usize>, Vec<Vec<u32>>) {
    (
        trace.prediction,
        trace
            .layers
            .iter()
            .map(|layer| layer.predicted_next)
            .collect(),
        trace
            .layers
            .iter()
            .map(|layer| layer.scores.iter().map(|score| score.to_bits()).collect())
            .collect(),
    )
}

fn summarize_rows(rows: &[ProductionSweepRow]) -> Vec<ProductionSweepSummary> {
    let mut groups: BTreeMap<(usize, usize, u32, ProductionDiagnosticArm), Vec<f32>> =
        BTreeMap::new();
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
            ProductionSweepSummary {
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

fn paired_lower_95(treatment: &BTreeMap<u64, f32>, control: &BTreeMap<u64, f32>) -> (f32, f32) {
    let differences = treatment
        .iter()
        .filter_map(|(seed, value)| control.get(seed).map(|other| value - other))
        .collect::<Vec<_>>();
    let difference_mean = mean(&differences);
    let difference_variance = variance(&differences, difference_mean);
    let lower_95 = if differences.len() > 1 {
        difference_mean - CONFIDENCE_Z * (difference_variance / differences.len() as f32).sqrt()
    } else {
        difference_mean
    };
    (difference_mean, lower_95)
}

fn edge_index(conn: &Csr, pre: CellId, post: CellId) -> Option<usize> {
    let row = pre as usize;
    let start = *conn.row_ptr.get(row)? as usize;
    let end = *conn.row_ptr.get(row + 1)? as usize;
    conn.col[start..end]
        .iter()
        .position(|candidate| *candidate == post)
        .map(|offset| start + offset)
}

fn clear_eligibility(engine: &mut Engine) {
    let now = engine.time();
    for synapse in engine.syn.as_mut_slice() {
        synapse.eligibility = 0.0;
        synapse.last_elig_update = now;
    }
}

fn softmax(values: &[f32]) -> Vec<f32> {
    let max = values.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let mut probabilities = values
        .iter()
        .map(|value| (*value - max).exp())
        .collect::<Vec<_>>();
    let sum = probabilities.iter().sum::<f32>().max(f32::MIN_POSITIVE);
    for probability in &mut probabilities {
        *probability /= sum;
    }
    probabilities
}

fn cosine(left: &[f32], right: &[f32]) -> f32 {
    let dot = left.iter().zip(right).map(|(a, b)| a * b).sum::<f32>();
    let left_norm = left.iter().map(|value| value * value).sum::<f32>().sqrt();
    let right_norm = right.iter().map(|value| value * value).sum::<f32>().sqrt();
    if left_norm <= 1e-12 || right_norm <= 1e-12 {
        0.0
    } else {
        dot / (left_norm * right_norm)
    }
}

fn sign_agreement(left: &[f32], right: &[f32]) -> f32 {
    left.iter()
        .zip(right)
        .filter(|(a, b)| a.signum() == b.signum())
        .count() as f32
        / left.len().max(1) as f32
}

fn mse(left: &[f32], right: &[f32]) -> f32 {
    left.iter()
        .zip(right)
        .map(|(a, b)| (a - b).powi(2))
        .sum::<f32>()
        / left.len().max(1) as f32
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

fn mean_refs<F>(rows: &[&ProductionMechanismRow], value: F) -> f32
where
    F: Fn(&ProductionMechanismRow) -> f32,
{
    if rows.is_empty() {
        0.0
    } else {
        rows.iter().map(|row| value(row)).sum::<f32>() / rows.len() as f32
    }
}

fn fingerprint_weights(weights: &[f32]) -> u64 {
    let mut bytes = Vec::with_capacity(weights.len() * 4);
    for weight in weights {
        bytes.extend_from_slice(&weight.to_bits().to_le_bytes());
    }
    fnv1a64(&bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tiny_config() -> ProductionDiagnosticConfig {
        ProductionDiagnosticConfig {
            n_seeds: 2,
            budgets: vec![12],
            learning_rates: vec![0.015],
            test_examples: 16,
            mechanism_examples: 2,
            ..ProductionDiagnosticConfig::quick()
        }
    }

    #[test]
    fn protocol_hash_is_sensitive_and_seed_families_are_disjoint() {
        let config = ProductionDiagnosticConfig::quick();
        let mut changed = config.clone();
        changed.eligibility_tau += 1.0;
        assert_ne!(config.hash(), changed.hash());
        let production = config.diagnostic_seeds();
        let frozen_v3_development = seeds(0x4842_4445_5600_0001, 5);
        let frozen_v3_pilot = seeds(0x4842_5049_4c4f_5402, 3);
        let frozen_v3_held_out = seeds(0x4842_4652_4553_4802, 20);
        let smooth_diagnostic = seeds(0x4842_4449_4147_0001, 20);
        assert!(production.iter().all(|seed| {
            !frozen_v3_development.contains(seed)
                && !frozen_v3_pilot.contains(seed)
                && !frozen_v3_held_out.contains(seed)
                && !smooth_diagnostic.contains(seed)
        }));
    }

    #[test]
    fn production_forward_is_target_independent_and_deterministic() {
        let config = tiny_config();
        assert!(verify_forward_contract(&config, &config.diagnostic_seeds()));
    }

    #[test]
    fn event_scores_equal_shared_residual_transition() {
        let config = tiny_config();
        let seed = config.diagnostic_seeds()[0];
        let mut example = FrozenSplit::new(seed, 1, 1, 1).train.remove(0);
        example.start = 2;
        example.operations[0] = 1;
        example.target = true_transition(example.start, 1, N_STATES);
        let mut graph = ProductionGraph::new(&config, seed, 1);
        let trace = graph.forward(&example);
        let layer = &trace.layers[0];
        for post in 0..N_STATES {
            let expected = f32::from(post == example.start)
                + graph.engine.edge_w[graph.edge(example.start, 1, post)];
            assert!(
                (layer.scores[post] - expected).abs() < 1e-6,
                "post={post} event={} expected={expected}",
                layer.scores[post]
            );
        }
        let _ = graph.capture_production_eligibility();
        graph.finish_trial(&trace);
    }

    #[test]
    fn captured_eligibility_is_sparse_production_stdp() {
        let config = tiny_config();
        let seed = config.diagnostic_seeds()[0];
        let example = FrozenSplit::new(seed, 4, 1, 1).train.remove(0);
        let mut graph = ProductionGraph::new(&config, seed, 4);
        let trace = graph.forward(&example);
        let eligibility = graph.capture_production_eligibility();
        assert!(eligibility.iter().any(|value| value.abs() > 1e-6));
        assert!(
            eligibility
                .iter()
                .filter(|value| value.abs() > 1e-6)
                .count()
                < eligibility.len() / 2
        );
        for (captured, synapse) in eligibility.iter().zip(graph.engine.syn.as_slice()) {
            assert_eq!(captured.to_bits(), synapse.eligibility.to_bits());
        }
        graph.finish_trial(&trace);
    }

    #[test]
    fn existing_arm_applies_production_eligibility_times_post_credit() {
        let config = tiny_config();
        let seed = config.diagnostic_seeds()[0];
        let example = FrozenSplit::new(seed, 1, 1, 1).train.remove(0);
        let mut graph = ProductionGraph::new(&config, seed, 1);
        let trace = graph.forward(&example);
        let eligibility = graph.capture_production_eligibility();
        let targets = graph.terminal_teacher(&example, example.target, 0.001);
        let before = graph.engine.edge_w.clone();
        graph.apply_post_credit(&targets.post_credits);
        for edge in 0..before.len() {
            let post = graph.engine.conn.col[edge] as usize;
            let expected = eligibility[edge] * targets.post_credits[post];
            let actual = graph.engine.edge_w[edge] - before[edge];
            assert!(
                (actual - expected).abs() < 1e-6,
                "edge={edge} actual={actual} expected={expected}"
            );
        }
        graph.finish_trial(&trace);
    }

    #[test]
    fn terminal_teacher_matches_finite_difference() {
        let config = tiny_config();
        let seed = config.diagnostic_seeds()[0];
        let example = FrozenSplit::new(seed, 3, 1, 1).train.remove(0);
        let graph = ProductionGraph::new(&config, seed, 3);
        let analytic = graph.terminal_teacher(&example, example.target, 1.0);
        let epsilon = 1e-3;
        for edge in 0..graph.engine.edge_w.len() {
            let mut plus = ProductionGraph::new(&config, seed, 3);
            plus.engine.edge_w.clone_from(&graph.engine.edge_w);
            plus.engine.syn.as_mut_slice()[edge].weight += epsilon;
            plus.engine.edge_w[edge] += epsilon;
            let plus_loss = plus.terminal_teacher(&example, example.target, 1.0).loss;
            let mut minus = ProductionGraph::new(&config, seed, 3);
            minus.engine.edge_w.clone_from(&graph.engine.edge_w);
            minus.engine.syn.as_mut_slice()[edge].weight -= epsilon;
            minus.engine.edge_w[edge] -= epsilon;
            let minus_loss = minus.terminal_teacher(&example, example.target, 1.0).loss;
            let numerical_update = (minus_loss - plus_loss) / (2.0 * epsilon);
            assert!(
                (analytic.edge_deltas[edge] - numerical_update).abs() < 2e-4,
                "edge={edge} analytic={} numerical={numerical_update}",
                analytic.edge_deltas[edge]
            );
        }
    }

    #[test]
    fn least_squares_residual_is_orthogonal_and_not_worse() {
        let config = tiny_config();
        let seed = config.diagnostic_seeds()[0];
        let example = FrozenSplit::new(seed, 4, 1, 1).train.remove(0);
        let mut graph = ProductionGraph::new(&config, seed, 4);
        let _trace = graph.forward(&example);
        let eligibility = graph.capture_production_eligibility();
        let targets = graph.terminal_teacher(&example, example.target, 0.015);
        let audit = production_factorization(&targets, &eligibility, &graph.engine.conn.col);
        assert!(audit.oracle_mse <= audit.existing_mse + 1e-10);
        let reconstructed = eligibility
            .iter()
            .zip(&graph.engine.conn.col)
            .map(|(edge_eligibility, post)| {
                edge_eligibility * audit.oracle_post_credits[*post as usize]
            })
            .collect::<Vec<_>>();
        for post in 0..graph.engine.num_cells() {
            let residual = eligibility
                .iter()
                .zip(&targets.edge_deltas)
                .zip(&reconstructed)
                .zip(&graph.engine.conn.col)
                .filter(|(_, edge_post)| **edge_post as usize == post)
                .map(|(((edge_eligibility, target), reconstructed), _)| {
                    edge_eligibility * (target - reconstructed)
                })
                .sum::<f32>();
            assert!(residual.abs() < 1e-6, "post={post} residual={residual}");
        }
    }

    #[test]
    fn direct_step_improves_teacher_loss_more_than_rotated_direction() {
        let mut config = tiny_config();
        config.mechanism_examples = 16;
        let mechanisms = run_mechanisms(&config, &config.diagnostic_seeds());
        let direct = mechanisms
            .iter()
            .map(|row| row.surrogate_direct_loss_drop)
            .sum::<f32>();
        let rotated = mechanisms
            .iter()
            .map(|row| row.surrogate_rotated_loss_drop)
            .sum::<f32>();
        assert!(direct > 0.0);
        assert!(direct > rotated);
    }

    #[test]
    fn no_test_updates_and_exact_replay() {
        let config = tiny_config();
        let first = run_production_diagnostics(&config);
        let second = run_production_diagnostics(&config);
        assert_eq!(first.protocol_hash, second.protocol_hash);
        assert_eq!(first.rows, second.rows);
        assert_eq!(first.summaries, second.summaries);
        assert_eq!(first.mechanisms, second.mechanisms);
        assert!(first.all_test_weights_unchanged);
        assert!(first.production_forward_contract_passed);
        assert_eq!(first.render_sweep_csv(), second.render_sweep_csv());
        assert_eq!(first.render_mechanism_csv(), second.render_mechanism_csv());
    }
}
