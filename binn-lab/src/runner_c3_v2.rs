//! Production-faithful C3 v2 credit-depth harness.
//!
//! Unlike C3 v1's tabular proxy, every transition here is executed by the
//! event-driven [`Engine`]. Source spikes drive a layer-specific state area,
//! membrane scores select one hard k-WTA winner, and production
//! [`ThreeFactor`] eligibility receives broadcast or postsynaptic credit.

use std::collections::BTreeMap;

use binn_areas::k_wta;
use binn_core::{Csr, Rng, Tick};
use binn_data::{draw_example, true_transition, CreditDepthExample};
use binn_engine::{Cell, CellId, Engine, K};
use binn_learn::{
    FixedRandomFeedback, Modulators, PostSynapticCredit, RunningMeanBaseline, ThreeFactor,
};

use crate::c3_v2_config::{C3V2Arm, C3V2Config, C3_V2_PROTOCOL_VERSION};
use crate::runner::{clear_eligibility, mean_var};

pub const C3_V2_MATCHED_GRADIENT_REFERENCE: &str =
    "C3_V2_MATCHED_FORWARD_ORACLE_SURROGATE_GRADIENT_REFERENCE";

#[derive(Clone, Debug, PartialEq)]
pub struct C3V2DepthResult {
    pub arm: C3V2Arm,
    pub depth: usize,
    pub mean_accuracy: f32,
    pub variance_accuracy: f32,
    pub seed_accuracies: Vec<f32>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct C3V2ArmResult {
    pub arm: C3V2Arm,
    pub config_hash: String,
    pub d_star: Option<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum C3V2Verdict {
    Pilot,
    Measured,
}

impl C3V2Verdict {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pilot => "PILOT",
            Self::Measured => "MEASURED",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct C3V2ParityEvidence {
    pub topology_equal: bool,
    pub initialization_equal: bool,
    pub frozen_examples_equal: bool,
    pub initial_predictions_equal: bool,
    pub initial_winners_equal: bool,
    pub initial_charges_equal: bool,
    pub target_independent_forward: bool,
    pub test_updates_absent: bool,
}

impl C3V2ParityEvidence {
    pub fn all_pass(&self) -> bool {
        self.topology_equal
            && self.initialization_equal
            && self.frozen_examples_equal
            && self.initial_predictions_equal
            && self.initial_winners_equal
            && self.initial_charges_equal
            && self.target_independent_forward
            && self.test_updates_absent
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct C3V2Report {
    pub protocol_version: u64,
    pub verdict: C3V2Verdict,
    pub depth_results: Vec<C3V2DepthResult>,
    pub arm_results: Vec<C3V2ArmResult>,
    pub parity: C3V2ParityEvidence,
}

impl C3V2Report {
    pub fn arm_result(&self, arm: C3V2Arm) -> &C3V2ArmResult {
        self.arm_results
            .iter()
            .find(|result| result.arm == arm)
            .expect("C3 v2 arm result missing")
    }
}

#[derive(Default)]
pub struct C3V2Runner;

impl C3V2Runner {
    pub fn new() -> Self {
        Self
    }

    pub fn run(&mut self, config: &C3V2Config) -> C3V2Report {
        assert!(
            config.kill_gate_override,
            "C3V2Runner requires explicit kill_gate_override"
        );
        assert!(config.min_depth >= 1);
        assert!(config.max_depth >= config.min_depth);
        assert_eq!(
            config.n_operations, 2,
            "credit-depth task oracle currently has two operations"
        );

        let mut depth_results = Vec::new();
        let mut parity = C3V2ParityEvidence {
            topology_equal: true,
            initialization_equal: true,
            frozen_examples_equal: true,
            initial_predictions_equal: true,
            initial_winners_equal: true,
            initial_charges_equal: true,
            target_independent_forward: true,
            test_updates_absent: true,
        };

        for depth in config.min_depth..=config.max_depth {
            let mut per_arm: BTreeMap<&'static str, Vec<f32>> = C3V2Arm::ALL
                .into_iter()
                .map(|arm| (arm.as_str(), Vec::with_capacity(config.n_seeds)))
                .collect();
            for seed in config.seeds() {
                let frozen = FrozenDepthSplit::new(config, seed, depth);
                let probe = parity_probe(config, seed, depth, &frozen);
                parity = and_parity(parity, probe);
                for arm in C3V2Arm::ALL {
                    let outcome = run_arm(config, seed, depth, &frozen, arm);
                    parity.test_updates_absent &= outcome.test_weights_unchanged;
                    per_arm
                        .get_mut(arm.as_str())
                        .expect("arm accumulator")
                        .push(outcome.accuracy);
                }
            }
            for arm in C3V2Arm::ALL {
                let values = per_arm.remove(arm.as_str()).expect("arm values");
                let (mean_accuracy, variance_accuracy) = mean_var(&values);
                depth_results.push(C3V2DepthResult {
                    arm,
                    depth,
                    mean_accuracy,
                    variance_accuracy,
                    seed_accuracies: values,
                });
            }
        }
        assert!(
            parity.all_pass(),
            "C3 v2 forward-parity contract failed: {parity:?}"
        );

        let arm_results = C3V2Arm::ALL
            .into_iter()
            .map(|arm| {
                let d_star = depth_results
                    .iter()
                    .filter(|result| {
                        result.arm == arm && result.mean_accuracy >= config.accuracy_floor
                    })
                    .map(|result| result.depth)
                    .max();
                C3V2ArmResult {
                    arm,
                    config_hash: config.hash_string_for_arm(arm),
                    d_star,
                }
            })
            .collect();
        let verdict = if config.quick || config.n_seeds < config.scientific_n_seeds {
            C3V2Verdict::Pilot
        } else {
            C3V2Verdict::Measured
        };
        C3V2Report {
            protocol_version: C3_V2_PROTOCOL_VERSION,
            verdict,
            depth_results,
            arm_results,
            parity,
        }
    }

    pub fn render_markdown(report: &C3V2Report, config: &C3V2Config) -> String {
        let mut md = String::new();
        md.push_str("# C3 v2 — production-engine credit assignment versus depth\n\n");
        md.push_str(
            "**Exploratory post-G2 preregistration.** Canonical C1 protocol-v2 \
             hash `c1-118207fbc3eaba53` remains a FAIL. C3 v1 remains preserved \
             as a tabular terminal-reward proxy and is not evidence about the \
             production `ThreeFactor` learner.\n\n",
        );
        md.push_str(&format!(
            "- protocol version: {}\n- verdict: **{}**\n- seeds: {}\n\
             - depth: {}..={}\n- train/test per depth×seed: {}/{}\n\
             - D* accuracy floor: {:.3}\n- production forward parity: **{}**\n\n",
            report.protocol_version,
            report.verdict.as_str(),
            config.n_seeds,
            config.min_depth,
            config.max_depth,
            config.n_train,
            config.n_test,
            config.accuracy_floor,
            if report.parity.all_pass() {
                "PASS"
            } else {
                "FAIL"
            }
        ));
        if report.verdict == C3V2Verdict::Pilot {
            md.push_str(
                "> PILOT only: development seeds validate mechanics and cannot support a scientific D* claim.\n\n",
            );
        }

        md.push_str("## Arm hashes and D*\n\n");
        md.push_str("| arm | hash | D* |\n|---|---|---:|\n");
        for result in &report.arm_results {
            md.push_str(&format!(
                "| `{}` | `{}` | {} |\n",
                result.arm.as_str(),
                result.config_hash,
                opt_depth(result.d_star)
            ));
        }

        md.push_str("\n## Accuracy by depth\n\n");
        md.push_str("| depth | arm | mean | variance |\n|---:|---|---:|---:|\n");
        for result in &report.depth_results {
            md.push_str(&format!(
                "| {} | `{}` | {:.4} | {:.6} |\n",
                result.depth,
                result.arm.as_str(),
                result.mean_accuracy,
                result.variance_accuracy
            ));
        }

        md.push_str(
            "\n## Forward and leakage contract\n\n\
             | check | passed |\n\
             |---|---|\n",
        );
        for (label, passed) in [
            ("topology", report.parity.topology_equal),
            ("initial weights", report.parity.initialization_equal),
            ("frozen examples", report.parity.frozen_examples_equal),
            (
                "initial predictions",
                report.parity.initial_predictions_equal,
            ),
            ("initial winners", report.parity.initial_winners_equal),
            ("initial charges", report.parity.initial_charges_equal),
            (
                "forward target independence",
                report.parity.target_independent_forward,
            ),
            ("no test updates", report.parity.test_updates_absent),
        ] {
            md.push_str(&format!("| {label} | {passed} |\n"));
        }

        md.push_str(
            "\n## Protocol\n\n\
             Each layer is a real event-engine transition area. A forced \
             `(state, operation)` source spike deposits through CSR synapses; \
             membrane charge selects one hard k-WTA state winner; the winner is \
             force-spiked so production STDP eligibility records the selected \
             transition. Broadcast and RPE arms receive only terminal reward. \
             E-prop transports current downstream-weight signals, DFA uses an \
             immutable random projection, and the matched reference receives \
             oracle per-layer target pulses only after the shared forward rollout.\n\n\
             Every arm shares topology, initialization, frozen examples, forward \
             predictions/winners/charges at initialization, target-independent \
             forward execution, and test non-update checks. Oracle correction \
             pulses are executed after prediction in every training arm; only \
             the matched reference learns from them.\n",
        );
        md
    }
}

#[derive(Clone, Debug)]
struct FrozenDepthSplit {
    train: Vec<CreditDepthExample>,
    test: Vec<CreditDepthExample>,
    fingerprint: u64,
}

impl FrozenDepthSplit {
    fn new(config: &C3V2Config, seed: u64, depth: usize) -> Self {
        let mut train_rng = Rng::new(seed ^ (depth as u64).wrapping_mul(0xD1B5_4A32_D192_ED03));
        let mut test_rng = Rng::new(seed ^ (depth as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9));
        let train: Vec<_> = (0..config.n_train)
            .map(|_| draw_example(&mut train_rng, depth, config.n_states))
            .collect();
        let test: Vec<_> = (0..config.n_test)
            .map(|_| draw_example(&mut test_rng, depth, config.n_states))
            .collect();
        let fingerprint = fingerprint_examples(&train, &test);
        Self {
            train,
            test,
            fingerprint,
        }
    }
}

struct C3ProductionGraph {
    engine: Engine,
    learner: ThreeFactor,
    feedback: FixedRandomFeedback,
    baseline: RunningMeanBaseline,
    depth: usize,
    n_states: usize,
    n_operations: usize,
    layer_stride: usize,
    t_cursor: Tick,
    topology_fingerprint: u64,
    initial_weight_fingerprint: u64,
}

impl C3ProductionGraph {
    fn new(config: &C3V2Config, seed: u64, depth: usize) -> Self {
        let n_sources_per_layer = config.n_states * config.n_operations;
        let layer_stride = n_sources_per_layer + config.n_states;
        let n_cells = depth * layer_stride;
        let mut rows = vec![Vec::<u32>::new(); n_cells];
        for layer in 0..depth {
            for state in 0..config.n_states {
                for operation in 0..config.n_operations {
                    let source = layer * layer_stride + state * config.n_operations + operation;
                    for next in 0..config.n_states {
                        rows[source]
                            .push((layer * layer_stride + n_sources_per_layer + next) as CellId);
                    }
                }
            }
        }
        let conn = Csr::from_adjacency(&rows);
        let topology_fingerprint = fingerprint_topology(&conn);
        let mut rng = Rng::new(seed ^ (depth as u64).wrapping_mul(0xC3F2_600D));
        let weights: Vec<f32> = (0..conn.nnz())
            .map(|_| config.init_w + (rng.next_f32() - 0.5) * 0.02)
            .collect();
        let initial_weight_fingerprint = fingerprint_weights(&weights);
        let mut engine = Engine::with_cells(n_cells);
        engine.set_connectivity(conn, weights);
        Self {
            learner: ThreeFactor::new(config.eta, config.lambda, config.tau_e),
            feedback: FixedRandomFeedback::new(n_cells, config.n_states, seed ^ 0xDFA0_C3F2),
            baseline: RunningMeanBaseline::new(),
            engine,
            depth,
            n_states: config.n_states,
            n_operations: config.n_operations,
            layer_stride,
            t_cursor: 0,
            topology_fingerprint,
            initial_weight_fingerprint,
        }
    }

    fn source(&self, layer: usize, state: usize, operation: usize) -> CellId {
        (layer * self.layer_stride + state * self.n_operations + operation) as CellId
    }

    fn output(&self, layer: usize, state: usize) -> CellId {
        (layer * self.layer_stride + self.n_states * self.n_operations + state) as CellId
    }

    fn forward(&mut self, example: &CreditDepthExample) -> DepthTrace {
        let saved: Vec<Cell> = self.engine.cells().to_vec();
        let mut state = example.start;
        let mut layers = Vec::with_capacity(self.depth);
        let mut at = self.t_cursor;
        for (layer, &operation) in example.operations.iter().enumerate() {
            let source = self.source(layer, state, operation);
            let outputs: Vec<_> = (0..self.n_states)
                .map(|next| self.output(layer, next))
                .collect();
            for &output in &outputs {
                self.engine.cell_mut(output).theta = f32::INFINITY;
            }
            self.engine.force_spike(source, at);
            let score_at = at + self.engine.max_synaptic_delay().max(1);
            let _ = self.engine.step_until(score_at);
            let scores: Vec<(CellId, f32)> = outputs
                .iter()
                .map(|&output| {
                    self.engine.cell_mut(output).advance_to(score_at);
                    (output, self.engine.cell(output).v)
                })
                .collect();
            let winner = k_wta(&scores, 1)[0];
            let predicted_next =
                (winner as usize) - (layer * self.layer_stride + self.n_states * self.n_operations);
            let true_next = true_transition(state, operation, self.n_states);
            for &output in &outputs {
                self.engine.cell_mut(output).v = 0.0;
            }
            let winner_at = score_at + 1;
            self.engine.force_spike(winner, winner_at);
            let _ = self.engine.step_until(winner_at + 1);
            layers.push(LayerTrace {
                layer,
                operation,
                source,
                scores: scores.iter().map(|(_, score)| *score).collect(),
                predicted_next,
                true_next,
            });
            state = predicted_next;
            at = winner_at + 3;
        }
        DepthTrace {
            target: example.target,
            prediction: state,
            layers,
            saved_cells: saved,
        }
    }

    fn learn_and_finish(&mut self, trace: DepthTrace, arm: C3V2Arm, train: bool) -> u64 {
        let mut applications = 0u64;
        if train {
            let reward = if trace.prediction == trace.target {
                1.0
            } else {
                -1.0
            };
            match arm {
                C3V2Arm::Broadcast => {
                    applications = self
                        .learner
                        .update_counted(&mut self.engine, Modulators::reward(reward));
                }
                C3V2Arm::Rpe => {
                    let advantage = self.baseline.advantage_and_observe(reward);
                    applications = self
                        .learner
                        .update_counted(&mut self.engine, Modulators::reward(advantage));
                }
                C3V2Arm::Eprop => {
                    let signal = self.eprop_credit(&trace);
                    applications = self
                        .learner
                        .update_with_credit_counted(&mut self.engine, &signal);
                }
                C3V2Arm::Dfa => {
                    let signal = self.dfa_credit(&trace);
                    applications = self
                        .learner
                        .update_with_credit_counted(&mut self.engine, &signal);
                }
                C3V2Arm::MatchedGradient => {
                    // Consume the shared forward spikes. Supervised target
                    // eligibility is constructed in the correction phase below.
                    self.learner.observe_spikes(&mut self.engine);
                }
            }
            clear_eligibility(&mut self.engine);

            // Identical post-forward oracle correction pulses for every arm.
            // Only the matched reference converts them into a weight update.
            let mut correction_at = self.engine.time() + 2;
            for layer in &trace.layers {
                self.engine.force_spike(layer.source, correction_at);
                let target_output = self.output(layer.layer, layer.true_next);
                self.engine.force_spike(target_output, correction_at + 1);
                correction_at += 3;
            }
            let _ = self.engine.step_until(correction_at);
            if arm == C3V2Arm::MatchedGradient {
                let mut signal = PostSynapticCredit::zeros(self.engine.num_cells());
                for layer in &trace.layers {
                    signal.set(self.output(layer.layer, layer.true_next), 1.0);
                }
                applications = applications.saturating_add(
                    self.learner
                        .update_with_credit_counted(&mut self.engine, &signal),
                );
            } else {
                self.learner.observe_spikes(&mut self.engine);
            }
            clear_eligibility(&mut self.engine);
        }

        self.reset_dynamic_state(&trace.saved_cells);
        self.engine.close_inhibited_cycle();
        self.t_cursor = self.engine.time() + 5;
        applications
    }

    fn reset_dynamic_state(&mut self, saved: &[Cell]) {
        let now = self.engine.time();
        for (index, previous) in saved.iter().enumerate() {
            let cell = self.engine.cell_mut(index as CellId);
            cell.v = previous.v;
            cell.v_dend = [0.0; K];
            cell.theta = previous.theta;
            cell.branches = previous.branches;
            cell.last = now;
        }
    }

    fn terminal_error(&self, trace: &DepthTrace) -> Vec<f32> {
        let final_layer = trace.layers.last().expect("depth >= 1");
        let probabilities = softmax(&final_layer.scores);
        probabilities
            .into_iter()
            .enumerate()
            .map(|(state, probability)| f32::from(state == trace.target) - probability)
            .collect()
    }

    fn eprop_credit(&self, trace: &DepthTrace) -> PostSynapticCredit {
        let mut layer_signal = vec![vec![0.0f32; self.n_states]; self.depth];
        layer_signal[self.depth - 1] = self.terminal_error(trace);
        for layer in (0..self.depth - 1).rev() {
            let next_op = trace.layers[layer + 1].operation;
            for state in 0..self.n_states {
                let source = self.source(layer + 1, state, next_op);
                for next in 0..self.n_states {
                    if let Some(edge) =
                        edge_index_local(&self.engine.conn, source, self.output(layer + 1, next))
                    {
                        layer_signal[layer][state] +=
                            self.engine.edge_w[edge] * layer_signal[layer + 1][next];
                    }
                }
            }
        }
        let mut signal = PostSynapticCredit::zeros(self.engine.num_cells());
        for (layer, values) in layer_signal.iter().enumerate() {
            for (state, value) in values.iter().enumerate() {
                signal.set(self.output(layer, state), *value);
            }
        }
        signal
    }

    fn dfa_credit(&self, trace: &DepthTrace) -> PostSynapticCredit {
        let terminal = self.terminal_error(trace);
        let mut signal = self.feedback.project(&terminal);
        for (state, value) in terminal.iter().enumerate() {
            signal.set(self.output(self.depth - 1, state), *value);
        }
        signal
    }
}

#[derive(Clone, Debug)]
struct LayerTrace {
    layer: usize,
    operation: usize,
    source: CellId,
    scores: Vec<f32>,
    predicted_next: usize,
    true_next: usize,
}

#[derive(Clone, Debug)]
struct DepthTrace {
    target: usize,
    prediction: usize,
    layers: Vec<LayerTrace>,
    saved_cells: Vec<Cell>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct DepthSignature {
    prediction: usize,
    winners: Vec<usize>,
    charge_bits: Vec<Vec<u32>>,
}

impl From<&DepthTrace> for DepthSignature {
    fn from(trace: &DepthTrace) -> Self {
        Self {
            prediction: trace.prediction,
            winners: trace
                .layers
                .iter()
                .map(|layer| layer.predicted_next)
                .collect(),
            charge_bits: trace
                .layers
                .iter()
                .map(|layer| layer.scores.iter().map(|score| score.to_bits()).collect())
                .collect(),
        }
    }
}

struct ArmOutcome {
    accuracy: f32,
    test_weights_unchanged: bool,
}

fn run_arm(
    config: &C3V2Config,
    seed: u64,
    depth: usize,
    split: &FrozenDepthSplit,
    arm: C3V2Arm,
) -> ArmOutcome {
    let mut graph = C3ProductionGraph::new(config, seed, depth);
    for example in &split.train {
        let trace = graph.forward(example);
        let _ = graph.learn_and_finish(trace, arm, true);
    }
    let before_test = fingerprint_weights(&graph.engine.edge_w);
    let mut correct = 0usize;
    for example in &split.test {
        let trace = graph.forward(example);
        correct += usize::from(trace.prediction == example.target);
        let _ = graph.learn_and_finish(trace, arm, false);
    }
    let after_test = fingerprint_weights(&graph.engine.edge_w);
    ArmOutcome {
        accuracy: correct as f32 / split.test.len().max(1) as f32,
        test_weights_unchanged: before_test == after_test,
    }
}

fn parity_probe(
    config: &C3V2Config,
    seed: u64,
    depth: usize,
    split: &FrozenDepthSplit,
) -> C3V2ParityEvidence {
    let mut topologies = Vec::new();
    let mut weights = Vec::new();
    let mut signatures = Vec::new();
    for _arm in C3V2Arm::ALL {
        let mut graph = C3ProductionGraph::new(config, seed, depth);
        topologies.push(graph.topology_fingerprint);
        weights.push(graph.initial_weight_fingerprint);
        signatures.push(DepthSignature::from(&graph.forward(&split.train[0])));
    }
    let mut alternate_target = split.train[0].clone();
    alternate_target.target = (alternate_target.target + 1) % config.n_states;
    let mut original_graph = C3ProductionGraph::new(config, seed, depth);
    let original_signature = DepthSignature::from(&original_graph.forward(&split.train[0]));
    let mut alternate_graph = C3ProductionGraph::new(config, seed, depth);
    let alternate_signature = DepthSignature::from(&alternate_graph.forward(&alternate_target));
    C3V2ParityEvidence {
        topology_equal: all_equal(topologies),
        initialization_equal: all_equal(weights),
        frozen_examples_equal: split.fingerprint == fingerprint_examples(&split.train, &split.test),
        initial_predictions_equal: all_equal(
            signatures.iter().map(|signature| signature.prediction),
        ),
        initial_winners_equal: all_equal(
            signatures.iter().map(|signature| signature.winners.clone()),
        ),
        initial_charges_equal: all_equal(
            signatures
                .iter()
                .map(|signature| signature.charge_bits.clone()),
        ),
        target_independent_forward: original_signature == alternate_signature,
        test_updates_absent: true,
    }
}

fn and_parity(a: C3V2ParityEvidence, b: C3V2ParityEvidence) -> C3V2ParityEvidence {
    C3V2ParityEvidence {
        topology_equal: a.topology_equal && b.topology_equal,
        initialization_equal: a.initialization_equal && b.initialization_equal,
        frozen_examples_equal: a.frozen_examples_equal && b.frozen_examples_equal,
        initial_predictions_equal: a.initial_predictions_equal && b.initial_predictions_equal,
        initial_winners_equal: a.initial_winners_equal && b.initial_winners_equal,
        initial_charges_equal: a.initial_charges_equal && b.initial_charges_equal,
        target_independent_forward: a.target_independent_forward && b.target_independent_forward,
        test_updates_absent: a.test_updates_absent && b.test_updates_absent,
    }
}

fn all_equal<T: PartialEq>(values: impl IntoIterator<Item = T>) -> bool {
    let mut values = values.into_iter();
    let Some(first) = values.next() else {
        return true;
    };
    values.all(|value| value == first)
}

fn softmax(values: &[f32]) -> Vec<f32> {
    let max = values.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let exp: Vec<f32> = values.iter().map(|value| (*value - max).exp()).collect();
    let sum = exp.iter().sum::<f32>().max(f32::MIN_POSITIVE);
    exp.into_iter().map(|value| value / sum).collect()
}

fn opt_depth(depth: Option<usize>) -> String {
    depth
        .map(|value| value.to_string())
        .unwrap_or_else(|| "none".into())
}

fn edge_index_local(conn: &Csr, pre: CellId, post: CellId) -> Option<usize> {
    let row = pre as usize;
    let start = *conn.row_ptr.get(row)? as usize;
    let end = *conn.row_ptr.get(row + 1)? as usize;
    conn.col[start..end]
        .iter()
        .position(|candidate| *candidate == post)
        .map(|offset| start + offset)
}

fn fingerprint_topology(conn: &Csr) -> u64 {
    let mut hash = Fnv64::new();
    for value in &conn.row_ptr {
        hash.mix(u64::from(*value));
    }
    for value in &conn.col {
        hash.mix(u64::from(*value));
    }
    hash.finish()
}

fn fingerprint_weights(weights: &[f32]) -> u64 {
    let mut hash = Fnv64::new();
    for value in weights {
        hash.mix(u64::from(value.to_bits()));
    }
    hash.finish()
}

fn fingerprint_examples(train: &[CreditDepthExample], test: &[CreditDepthExample]) -> u64 {
    let mut hash = Fnv64::new();
    for (partition, examples) in [(0u64, train), (1, test)] {
        hash.mix(partition);
        for example in examples {
            hash.mix(example.start as u64);
            for operation in &example.operations {
                hash.mix(*operation as u64);
            }
            hash.mix(example.target as u64);
        }
    }
    hash.finish()
}

struct Fnv64(u64);

impl Fnv64 {
    fn new() -> Self {
        Self(0xcbf2_9ce4_8422_2325)
    }

    fn mix(&mut self, value: u64) {
        self.0 ^= value;
        self.0 = self.0.wrapping_mul(0x100_0000_01b3);
    }

    fn finish(self) -> u64 {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quick_production_c3_is_pilot_and_parity_checked() {
        let mut config = C3V2Config::quick();
        config.kill_gate_override = true;
        config.n_seeds = 1;
        config.max_depth = 2;
        config.n_train = 16;
        config.n_test = 8;
        let mut runner = C3V2Runner::new();
        let report = runner.run(&config);
        assert_eq!(report.verdict, C3V2Verdict::Pilot);
        assert!(report.parity.all_pass());
        assert_eq!(report.depth_results.len(), 2 * C3V2Arm::ALL.len());
    }

    #[test]
    #[should_panic(expected = "kill_gate_override")]
    fn production_c3_refuses_without_override() {
        let config = C3V2Config::quick();
        let mut runner = C3V2Runner::new();
        let _ = runner.run(&config);
    }

    #[test]
    fn output_mapping_is_layer_local() {
        let config = C3V2Config::quick();
        let graph = C3ProductionGraph::new(&config, 7, 3);
        assert_ne!(graph.output(0, 0), graph.output(1, 0));
        assert_ne!(graph.source(0, 0, 0), graph.source(0, 0, 1));
    }

    #[test]
    fn production_forward_does_not_read_target() {
        let config = C3V2Config::quick();
        let seed = config.seeds()[0];
        let split = FrozenDepthSplit::new(&config, seed, 2);
        assert!(parity_probe(&config, seed, 2, &split).target_independent_forward);
    }
}
