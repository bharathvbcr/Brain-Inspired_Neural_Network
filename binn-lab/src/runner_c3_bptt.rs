//! Real SuperSpike BPTT on the production C3 credit-depth graph.
//!
//! Unlike frozen `c3v2-*` [`C3V2Arm::MatchedGradient`] (oracle target pulses +
//! STDP credit), the BPTT arm unrolls surrogate gradients through layer
//! transitions using scores captured in [`DepthTrace`] and applies `Δw` directly
//! to synapses — no oracle `force_spike` correction loop.

use std::collections::BTreeMap;

use binn_areas::k_wta;
use binn_core::{Csr, Rng, Tick};
use binn_data::{draw_example, true_transition, CreditDepthExample};
use binn_engine::{Cell, CellId, Engine};
use binn_learn::{PostSynapticCredit, ThreeFactor};

use crate::c3_bptt_config::{C3BpttArm, C3BpttConfig, C3_BPTT_PROTOCOL_VERSION};
use crate::runner::clear_eligibility;

pub const C3_BPTT_SUPERSPIKE_REFERENCE: &str = "C3_SUPERSPIKE_SURROGATE_BPTT_REFERENCE";
pub const C3_BPTT_ORACLE_PULSES_CONTRAST: &str = "C3_ORACLE_TARGET_PULSES_NOT_BPTT";

#[derive(Clone, Debug, PartialEq)]
pub struct C3BpttDepthResult {
    pub arm: C3BpttArm,
    pub depth: usize,
    pub mean_accuracy: f32,
    pub variance_accuracy: f32,
    pub seed_accuracies: Vec<f32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum C3BpttVerdict {
    Pilot,
    Measured,
}

impl C3BpttVerdict {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pilot => "PILOT",
            Self::Measured => "MEASURED",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct C3BpttArmResult {
    pub arm: C3BpttArm,
    pub config_hash: String,
    pub d_star: Option<usize>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct C3BpttReport {
    pub protocol_version: u64,
    pub verdict: C3BpttVerdict,
    pub depth_results: Vec<C3BpttDepthResult>,
    pub arm_results: Vec<C3BpttArmResult>,
}

#[derive(Default)]
pub struct C3BpttRunner;

impl C3BpttRunner {
    pub fn new() -> Self {
        Self
    }

    pub fn run(&mut self, config: &C3BpttConfig) -> C3BpttReport {
        assert!(
            config.kill_gate_override,
            "C3BpttRunner requires explicit kill_gate_override"
        );
        assert!(config.min_depth >= 1);
        assert!(config.max_depth >= config.min_depth);
        assert_eq!(
            config.n_operations, 2,
            "credit-depth task oracle currently has two operations"
        );

        let mut depth_results = Vec::new();
        for depth in config.min_depth..=config.max_depth {
            let mut per_arm: BTreeMap<&'static str, Vec<f32>> = C3BpttArm::ALL
                .into_iter()
                .map(|arm| (arm.as_str(), Vec::with_capacity(config.n_seeds)))
                .collect();
            for seed in config.seeds() {
                let frozen = FrozenDepthSplit::new(config, seed, depth);
                for arm in C3BpttArm::ALL {
                    let outcome = run_arm(config, seed, depth, &frozen, arm);
                    per_arm
                        .get_mut(arm.as_str())
                        .expect("arm accumulator")
                        .push(outcome.accuracy);
                }
            }
            for arm in C3BpttArm::ALL {
                let values = per_arm.remove(arm.as_str()).expect("arm values");
                let (mean_accuracy, variance_accuracy) = mean_var(&values);
                depth_results.push(C3BpttDepthResult {
                    arm,
                    depth,
                    mean_accuracy,
                    variance_accuracy,
                    seed_accuracies: values,
                });
            }
        }

        let arm_results = C3BpttArm::ALL
            .into_iter()
            .map(|arm| {
                let d_star = depth_results
                    .iter()
                    .filter(|result| {
                        result.arm == arm && result.mean_accuracy >= config.accuracy_floor
                    })
                    .map(|result| result.depth)
                    .max();
                C3BpttArmResult {
                    arm,
                    config_hash: config.hash_string_for_arm(arm),
                    d_star,
                }
            })
            .collect();
        let verdict = if config.quick || config.n_seeds < config.scientific_n_seeds {
            C3BpttVerdict::Pilot
        } else {
            C3BpttVerdict::Measured
        };
        C3BpttReport {
            protocol_version: C3_BPTT_PROTOCOL_VERSION,
            verdict,
            depth_results,
            arm_results,
        }
    }

    pub fn render_markdown(report: &C3BpttReport, config: &C3BpttConfig) -> String {
        let mut md = String::new();
        md.push_str("# C3 BPTT — real surrogate backprop on production credit-depth\n\n");
        md.push_str(
            "**Exploratory post-G2 preregistration.** Canonical C1 protocol-v2 \
             hash `c1-118207fbc3eaba53` remains a FAIL. Frozen `c3v2-*` \
             [`matched-forward-oracle-gradient-reference`] is **not** BPTT — it \
             injects oracle target pulses after forward. This family (`c3-bptt-*`) \
             tests true SuperSpike surrogate BPTT through layer transitions.\n\n",
        );
        md.push_str("## Mechanism disclosure\n\n");
        md.push_str(&format!(
            "| arm | label | learning path |\n\
             |---|---|---|\n\
             | `{C3_BPTT_SUPERSPIKE_REFERENCE}` | superspike-bptt | backward unroll \
             through layer scores; `Δw` on CSR edges; **no** oracle correction pulses |\n\
             | `{C3_BPTT_ORACLE_PULSES_CONTRAST}` | oracle-pulses | oracle \
             `force_spike` target pulses + STDP credit (same idea as c3v2 matched \
             reference; **not BPTT**) |\n\n",
        ));
        md.push_str(&format!(
            "- protocol version: {}\n\
             - verdict: **{}**\n\
             - seeds: {}\n\
             - depth: {}..={}\n\
             - train/test per depth×seed: {}/{}\n\
             - surrogate β: {:.1}\n\
             - D* accuracy floor: {:.3}\n\n",
            report.protocol_version,
            report.verdict.as_str(),
            config.n_seeds,
            config.min_depth,
            config.max_depth,
            config.n_train,
            config.n_test,
            config.surrogate_beta,
            config.accuracy_floor,
        ));
        if report.verdict == C3BpttVerdict::Pilot {
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
            "\n## Interpretation contract\n\n\
             - SuperSpike BPTT is the scientifically meaningful gradient reference on \
             this graph; oracle pulses are a labeled contrast only.\n\
             - Outcomes do not reopen frozen `c3v2-*` or canonical C1 G2.\n",
        );
        md
    }
}

#[derive(Clone, Debug)]
struct FrozenDepthSplit {
    train: Vec<CreditDepthExample>,
    test: Vec<CreditDepthExample>,
}

impl FrozenDepthSplit {
    fn new(config: &C3BpttConfig, seed: u64, depth: usize) -> Self {
        let mut train_rng = Rng::new(seed ^ (depth as u64).wrapping_mul(0xD1B5_4A32_D192_ED03));
        let mut test_rng = Rng::new(seed ^ (depth as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9));
        let train: Vec<_> = (0..config.n_train)
            .map(|_| draw_example(&mut train_rng, depth, config.n_states))
            .collect();
        let test: Vec<_> = (0..config.n_test)
            .map(|_| draw_example(&mut test_rng, depth, config.n_states))
            .collect();
        Self { train, test }
    }
}

struct C3BpttGraph {
    engine: Engine,
    learner: ThreeFactor,
    depth: usize,
    n_states: usize,
    n_operations: usize,
    layer_stride: usize,
    eta: f32,
    t_cursor: Tick,
    oracle_pulses_applied: bool,
}

impl C3BpttGraph {
    fn new(config: &C3BpttConfig, seed: u64, depth: usize) -> Self {
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
        let mut rng = Rng::new(seed ^ (depth as u64).wrapping_mul(0xC3B7_600D));
        let weights: Vec<f32> = (0..conn.nnz())
            .map(|_| config.init_w + (rng.next_f32() - 0.5) * 0.02)
            .collect();
        let mut engine = Engine::with_cells(n_cells);
        engine.set_connectivity(conn, weights);
        Self {
            learner: ThreeFactor::new(config.eta, config.lambda, config.tau_e),
            engine,
            depth,
            n_states: config.n_states,
            n_operations: config.n_operations,
            layer_stride,
            eta: config.eta,
            t_cursor: 0,
            oracle_pulses_applied: false,
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

    fn learn_and_finish(
        &mut self,
        trace: DepthTrace,
        arm: C3BpttArm,
        train: bool,
        beta: f32,
    ) -> u64 {
        let mut applications = 0u64;
        if train {
            match arm {
                C3BpttArm::SuperSpikeBptt => {
                    applications = self.apply_superspike_bptt(&trace, beta);
                }
                C3BpttArm::OraclePulses => {
                    self.learner.observe_spikes(&mut self.engine);
                    clear_eligibility(&mut self.engine);
                    applications = applications.saturating_add(self.apply_oracle_pulses(&trace));
                    let mut signal = PostSynapticCredit::zeros(self.engine.num_cells());
                    for layer in &trace.layers {
                        signal.set(self.output(layer.layer, layer.true_next), 1.0);
                    }
                    applications = applications.saturating_add(
                        self.learner
                            .update_with_credit_counted(&mut self.engine, &signal),
                    );
                    clear_eligibility(&mut self.engine);
                }
            }
        }

        self.reset_dynamic_state(&trace.saved_cells);
        self.engine.close_inhibited_cycle();
        self.t_cursor = self.engine.time() + 5;
        applications
    }

    fn apply_oracle_pulses(&mut self, trace: &DepthTrace) -> u64 {
        self.oracle_pulses_applied = true;
        let mut correction_at = self.engine.time() + 2;
        for layer in &trace.layers {
            self.engine.force_spike(layer.source, correction_at);
            let target_output = self.output(layer.layer, layer.true_next);
            self.engine.force_spike(target_output, correction_at + 1);
            correction_at += 3;
        }
        let _ = self.engine.step_until(correction_at);
        0
    }

    fn apply_superspike_bptt(&mut self, trace: &DepthTrace, beta: f32) -> u64 {
        let mut layer_signal = vec![vec![0.0f32; self.n_states]; self.depth];
        layer_signal[self.depth - 1] = self.terminal_error(trace);
        for layer_idx in (0..self.depth - 1).rev() {
            let next_op = trace.layers[layer_idx + 1].operation;
            for state in 0..self.n_states {
                let source = self.source(layer_idx + 1, state, next_op);
                for next in 0..self.n_states {
                    if let Some(edge) = edge_index_local(
                        &self.engine.conn,
                        source,
                        self.output(layer_idx + 1, next),
                    ) {
                        layer_signal[layer_idx][state] +=
                            self.engine.edge_w[edge] * layer_signal[layer_idx + 1][next];
                    }
                }
            }
        }

        let mut updates = vec![0.0f32; self.engine.edge_w.len()];
        let lr = self.eta;
        for (layer_idx, layer_trace) in trace.layers.iter().enumerate() {
            let cutoff = layer_trace.scores.iter().copied().fold(0.0f32, f32::max);
            let source = layer_trace.source;
            #[allow(clippy::needless_range_loop)]
            for state in 0..self.n_states {
                let score = layer_trace.scores[state];
                let surrogate = 1.0 / (1.0 + beta * (score - cutoff).abs()).powi(2);
                let delta = layer_signal[layer_idx][state];
                if let Some(edge) =
                    edge_index_local(&self.engine.conn, source, self.output(layer_idx, state))
                {
                    updates[edge] += lr * delta * surrogate;
                }
            }
        }

        let mut changed = 0u64;
        for (edge, delta) in updates.into_iter().enumerate() {
            if delta.abs() <= f32::EPSILON {
                continue;
            }
            let weight = (self.engine.edge_w[edge] + delta).clamp(-8.0, 8.0);
            self.engine.edge_w[edge] = weight;
            self.engine.syn.as_mut_slice()[edge].weight = weight;
            changed = changed.saturating_add(1);
        }
        changed
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

    fn reset_dynamic_state(&mut self, saved: &[Cell]) {
        let now = self.engine.time();
        for (index, previous) in saved.iter().enumerate() {
            let cell = self.engine.cell_mut(index as CellId);
            cell.v = previous.v;
            cell.v_dend = [0.0; binn_engine::K];
            cell.theta = previous.theta;
            cell.branches = previous.branches;
            cell.last = now;
        }
    }
}

#[derive(Clone, Debug)]
struct LayerTrace {
    layer: usize,
    operation: usize,
    source: CellId,
    scores: Vec<f32>,
    #[allow(dead_code)]
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

struct ArmOutcome {
    accuracy: f32,
}

fn run_arm(
    config: &C3BpttConfig,
    seed: u64,
    depth: usize,
    split: &FrozenDepthSplit,
    arm: C3BpttArm,
) -> ArmOutcome {
    let mut graph = C3BpttGraph::new(config, seed, depth);
    for example in &split.train {
        let trace = graph.forward(example);
        let _ = graph.learn_and_finish(trace, arm, true, config.surrogate_beta);
    }
    let mut correct = 0usize;
    for example in &split.test {
        let trace = graph.forward(example);
        correct += usize::from(trace.prediction == example.target);
        let _ = graph.learn_and_finish(trace, arm, false, config.surrogate_beta);
    }
    ArmOutcome {
        accuracy: correct as f32 / split.test.len().max(1) as f32,
    }
}

fn mean_var(values: &[f32]) -> (f32, f32) {
    if values.is_empty() {
        return (0.0, 0.0);
    }
    let mean = values.iter().sum::<f32>() / values.len() as f32;
    if values.len() < 2 {
        return (mean, 0.0);
    }
    let variance = values
        .iter()
        .map(|value| (*value - mean).powi(2))
        .sum::<f32>()
        / (values.len() - 1) as f32;
    (mean, variance)
}

fn opt_depth(depth: Option<usize>) -> String {
    depth
        .map(|value| value.to_string())
        .unwrap_or_else(|| "none".into())
}

fn softmax(values: &[f32]) -> Vec<f32> {
    let max = values.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let exp: Vec<f32> = values.iter().map(|value| (*value - max).exp()).collect();
    let sum = exp.iter().sum::<f32>().max(f32::MIN_POSITIVE);
    exp.into_iter().map(|value| value / sum).collect()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{C3V2Arm, C3V2Config};

    #[test]
    fn bptt_path_skips_oracle_pulses_oracle_path_uses_them() {
        let mut config = C3BpttConfig::quick();
        config.kill_gate_override = true;
        config.n_train = 8;
        config.n_test = 4;
        let seed = config.seeds()[0];
        let split = FrozenDepthSplit::new(&config, seed, 2);
        let example = &split.train[0];

        let mut bptt_graph = C3BpttGraph::new(&config, seed, 2);
        let trace = bptt_graph.forward(example);
        let updates = bptt_graph.learn_and_finish(
            trace,
            C3BpttArm::SuperSpikeBptt,
            true,
            config.surrogate_beta,
        );
        assert!(
            !bptt_graph.oracle_pulses_applied,
            "BPTT must not run oracle correction pulses"
        );
        assert!(updates > 0, "BPTT should apply direct edge updates");

        let mut oracle_graph = C3BpttGraph::new(&config, seed, 2);
        let oracle_trace = oracle_graph.forward(example);
        let _ = oracle_graph.learn_and_finish(
            oracle_trace,
            C3BpttArm::OraclePulses,
            true,
            config.surrogate_beta,
        );
        assert!(
            oracle_graph.oracle_pulses_applied,
            "oracle contrast must apply correction pulses"
        );
    }

    #[test]
    fn hashes_distinct_from_c3v2() {
        let cfg = C3BpttConfig::scientific();
        let v2 = C3V2Config::scientific();
        for arm in C3BpttArm::ALL {
            let hash = cfg.hash_string_for_arm(arm);
            assert!(hash.starts_with(crate::c3_bptt_config::C3_BPTT_HASH_PREFIX));
            for v2_arm in C3V2Arm::ALL {
                assert_ne!(hash, v2.hash_string_for_arm(v2_arm));
            }
        }
    }

    #[test]
    fn quick_run_finishes() {
        let mut config = C3BpttConfig::quick();
        config.kill_gate_override = true;
        config.n_seeds = 1;
        config.max_depth = 2;
        config.n_train = 16;
        config.n_test = 8;
        let mut runner = C3BpttRunner::new();
        let report = runner.run(&config);
        assert_eq!(report.verdict, C3BpttVerdict::Pilot);
        assert_eq!(report.depth_results.len(), 2 * C3BpttArm::ALL.len());
        let md = C3BpttRunner::render_markdown(&report, &config);
        assert!(md.contains("SuperSpike"));
        assert!(md.contains("oracle"));
        assert!(md.contains("c3-bptt-"));
    }

    #[test]
    #[should_panic(expected = "kill_gate_override")]
    fn refuses_without_override() {
        let config = C3BpttConfig::quick();
        let mut runner = C3BpttRunner::new();
        let _ = runner.run(&config);
    }
}
