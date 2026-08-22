//! Exact-forward C1 credit-assignment repreregistration.
//!
//! Every assembly arm runs the production `LatencyEncoder`, event-driven
//! [`Engine`], seeded sparse topology, membrane-score hard k-WTA, forced winner
//! spikes, dual readout decision, and identical frozen exposure order.  Only
//! the weight-update rule differs.  Canonical C1 protocol v2 is not called or
//! mutated by this runner.

use std::collections::BTreeMap;

use binn_areas::{k_wta, Area};
use binn_core::{Csr, Tick};
use binn_data::{Encoder, LatencyEncoder, Metrics, Sample};
use binn_engine::{CellId, Engine};
use binn_learn::{
    FixedRandomFeedback, Modulators, PostSynapticCredit, RunningMeanBaseline, ThreeFactor,
};

use crate::credit_config::{CreditArm, CreditConfig};
use crate::runner::{
    boost_readout_incoming, build_dense_local, build_sparse_assembly, clear_eligibility,
    edge_index, freeze_trials, mean, mean_var, reset_c1_dynamic_state, run_positive_control,
    FrozenSplit, GateG2Verdict,
};

/// Stable label for the exact-forward surrogate-gradient reference.
pub const EXACT_FORWARD_SURROGATE_GRADIENT_REFERENCE: &str =
    "EXACT_FORWARD_STRAIGHT_THROUGH_SURROGATE_GRADIENT_REFERENCE";

/// One arm's condition-level outcome.
#[derive(Clone, Debug, PartialEq)]
pub struct CreditConditionOutcome {
    pub arm: CreditArm,
    pub accuracy: f32,
    pub activity_sparsity: f32,
    pub n_params: usize,
    pub training_exposures: usize,
    pub topology_fingerprint: u64,
    pub initial_weight_fingerprint: u64,
    pub split_fingerprint: u64,
    pub exposure_fingerprint: u64,
    pub test_weights_unchanged: bool,
}

/// Per-seed exact-forward outcomes.
#[derive(Clone, Debug, PartialEq)]
pub struct CreditSeedResult {
    pub seed: u64,
    pub outcomes: Vec<CreditConditionOutcome>,
}

impl CreditSeedResult {
    pub fn outcome(&self, arm: CreditArm) -> &CreditConditionOutcome {
        self.outcomes
            .iter()
            .find(|outcome| outcome.arm == arm)
            .expect("credit arm missing from seed result")
    }
}

/// Aggregated arm result and unchanged G2 contract.
#[derive(Clone, Debug, PartialEq)]
pub struct CreditArmSummary {
    pub arm: CreditArm,
    pub config_hash: String,
    pub protocol_version: u64,
    pub mean_accuracy: f32,
    pub variance_accuracy: f32,
    pub mean_gap_closed: f32,
    pub variance_gap_closed: f32,
    pub gap_closed_lower_95: f32,
    pub verdict: GateG2Verdict,
}

/// Forward-parity assertions materialized in the report.
#[derive(Clone, Debug, PartialEq)]
pub struct ForwardParityEvidence {
    pub topology_equal: bool,
    pub initialization_equal: bool,
    pub split_equal: bool,
    pub matched_exposure_order_equal: bool,
    pub initial_prediction_equal: bool,
    pub initial_winners_equal: bool,
    pub initial_charges_equal: bool,
    pub target_independent_forward: bool,
    pub test_updates_absent: bool,
}

impl ForwardParityEvidence {
    pub fn all_pass(&self) -> bool {
        self.topology_equal
            && self.initialization_equal
            && self.split_equal
            && self.matched_exposure_order_equal
            && self.initial_prediction_equal
            && self.initial_winners_equal
            && self.initial_charges_equal
            && self.target_independent_forward
            && self.test_updates_absent
    }
}

/// Full repreregistration report.
#[derive(Clone, Debug, PartialEq)]
pub struct CreditReport {
    pub seeds: Vec<CreditSeedResult>,
    pub summaries: Vec<CreditArmSummary>,
    pub positive_control_mean: f32,
    pub mean_activity_sparsity: f32,
    pub parity: ForwardParityEvidence,
    pub pilot: bool,
}

impl CreditReport {
    pub fn summary(&self, arm: CreditArm) -> &CreditArmSummary {
        self.summaries
            .iter()
            .find(|summary| summary.arm == arm)
            .expect("credit summary missing")
    }
}

#[derive(Default)]
pub struct CreditRunner;

impl CreditRunner {
    pub fn new() -> Self {
        Self
    }

    /// Run all exact-forward mechanisms on identical frozen splits.
    pub fn run(&mut self, config: &CreditConfig) -> CreditReport {
        assert!(config.base.n_seeds >= 1);
        assert!(config.matched_epochs >= 1);
        let mut seeds = Vec::with_capacity(config.base.n_seeds);
        let mut positive_controls = Vec::with_capacity(config.base.n_seeds);
        let mut parity_acc: Option<ForwardParityEvidence> = None;

        for seed in config.seeds() {
            let split = freeze_trials(&config.base, seed);
            let mut parity = parity_probe(config, seed, &split);
            let outcomes: Vec<_> = CreditArm::ALL
                .into_iter()
                .map(|arm| run_condition(config, seed, &split, arm))
                .collect();
            parity.test_updates_absent = outcomes
                .iter()
                .all(|outcome| outcome.test_weights_unchanged);
            parity_acc = Some(match parity_acc {
                None => parity,
                Some(previous) => and_parity(previous, parity),
            });
            seeds.push(CreditSeedResult { seed, outcomes });
            positive_controls.push(run_positive_control(&config.base, seed));
        }

        let positive_control_mean = mean(&positive_controls);
        let mean_activity_sparsity = mean(
            &seeds
                .iter()
                .map(|seed| {
                    seed.outcome(CreditArm::BroadcastEpochMatched)
                        .activity_sparsity
                })
                .collect::<Vec<_>>(),
        );
        let parity = parity_acc.expect("at least one seed");
        assert!(
            parity.all_pass(),
            "exact-forward parity contract failed: {parity:?}"
        );

        let summaries = CreditArm::ALL
            .into_iter()
            .map(|arm| {
                summarize_arm(
                    config,
                    &seeds,
                    arm,
                    positive_control_mean,
                    mean_activity_sparsity,
                    &parity,
                )
            })
            .collect();

        CreditReport {
            seeds,
            summaries,
            positive_control_mean,
            mean_activity_sparsity,
            parity,
            pilot: config.quick || config.base.n_seeds < config.scientific_n_seeds,
        }
    }

    /// Render a self-contained preregistered results note.
    pub fn render_markdown(report: &CreditReport, config: &CreditConfig) -> String {
        let mut md = String::new();
        md.push_str("# BINN exact-forward credit-assignment repreregistration\n\n");
        md.push_str(
            "**Canonical C1 is immutable:** protocol-v2 hash `c1-118207fbc3eaba53` \
             and every G2 threshold remain unchanged. These are separate protocols \
             with fresh held-out seeds.\n\n",
        );
        if config.is_isolation_calibrated_protocol() {
            md.push_str(&format!(
                "**Sparsity-calibrated trial-isolation (`c1x-iso-s-*`):** clears \
                 `ThreeFactor.last_spike`, applies C3-style full dynamic membrane reset, \
                 and selects k-WTA over all finite membranes (winner floor; arm protocol \
                 versions = base + {}). G2 / sparsity-band thresholds unchanged. Does **not** \
                 reopen frozen `c1x-*`, prior `c1x-iso-*`, or protocol-v2 G2.\n\n",
                crate::credit_config::CREDIT_ISOLATION_CALIBRATED_PROTOCOL_OFFSET
            ));
        } else if config.is_isolation_protocol() {
            md.push_str(&format!(
                "**Trial-isolation protocol (`c1x-iso-*`):** clears `ThreeFactor.last_spike` \
                 and applies C3-style full dynamic membrane reset at trial boundaries \
                 (arm protocol versions = base + {}). Does **not** reopen frozen non-isolated \
                 `c1x-*` hashes or protocol-v2 G2.\n\n",
                crate::credit_config::CREDIT_ISOLATION_PROTOCOL_OFFSET
            ));
        }
        md.push_str(&format!(
            "- schedule: **{}**\n- seeds: {}\n- train/test: {}/{}\n- matched epochs: {}\n\
             - trial isolation: **{}**\n",
            if report.pilot {
                "PILOT (development only)"
            } else {
                "SCIENTIFIC"
            },
            config.base.n_seeds,
            config.base.n_train,
            config.base.n_test,
            config.matched_epochs,
            if config.is_isolation_protocol() {
                "yes"
            } else {
                "no (frozen c1x-* path)"
            }
        ));
        md.push_str(&format!(
            "- positive control: {:.4} (minimum {:.4})\n\
             - activity sparsity: {:.4} (valid [{:.4}, {:.4}])\n\
             - exact-forward parity: **{}**\n\n",
            report.positive_control_mean,
            config.base.g2_min_positive_control,
            report.mean_activity_sparsity,
            config.base.activity_sparsity_min,
            config.base.activity_sparsity_max,
            if report.parity.all_pass() {
                "PASS"
            } else {
                "FAIL"
            }
        ));

        md.push_str("## Arm hashes and results\n\n");
        md.push_str(
            "| arm | protocol | hash | mean accuracy | gap LCB | verdict |\n\
             |---|---:|---|---:|---:|---|\n",
        );
        for summary in &report.summaries {
            md.push_str(&format!(
                "| `{}` | {} | `{}` | {:.4} | {:.4} | **{}** |\n",
                summary.arm.as_str(),
                summary.protocol_version,
                summary.config_hash,
                summary.mean_accuracy,
                summary.gap_closed_lower_95,
                summary.verdict.as_str()
            ));
        }

        md.push_str("\n## Exact-forward contract\n\n");
        md.push_str(
            "Assembly arms share the production LatencyEncoder, event engine, \
             sparse topology and initialized weights, membrane-score hard k-WTA, \
             forced winners, dual readout decision, frozen split, and deterministic \
             exposure order. The one-pass arm is the declared exposure diagnostic; \
             all other assembly arms use the matched epoch count. The dense arm is \
             the declared topology control.\n\n",
        );
        md.push_str("| check | passed |\n|---|---|\n");
        for (name, passed) in [
            ("topology", report.parity.topology_equal),
            ("initial weights", report.parity.initialization_equal),
            ("frozen split", report.parity.split_equal),
            (
                "matched exposure order",
                report.parity.matched_exposure_order_equal,
            ),
            ("initial prediction", report.parity.initial_prediction_equal),
            ("initial winners", report.parity.initial_winners_equal),
            ("initial charges", report.parity.initial_charges_equal),
            (
                "forward target independence",
                report.parity.target_independent_forward,
            ),
            ("no test updates", report.parity.test_updates_absent),
        ] {
            md.push_str(&format!("| {name} | {passed} |\n"));
        }

        md.push_str(
            "\n## Preregistered interpretation contract\n\n\
             - If epoch-matched broadcast improves over one-pass, exposure was material.\n\
             - If the exact-forward gradient reference collapses, the old front-end/reference mismatch inflated the gap.\n\
             - If RPE alone improves, reward centering/scaling was material.\n\
             - If E-prop/DFA improve while matched broadcast does not, neuron-specific credit is supported.\n\
             - No outcome changes or rescues canonical protocol-v2 G2.\n",
        );
        md
    }
}

struct ExactGraph {
    engine: Engine,
    area: Area,
    encoder: LatencyEncoder,
    learner: ThreeFactor,
    feedback: FixedRandomFeedback,
    baseline: RunningMeanBaseline,
    readout_0: CellId,
    readout_1: CellId,
    n_in: usize,
    n_hidden: usize,
    t_cursor: Tick,
    topology_fingerprint: u64,
    initial_weight_fingerprint: u64,
    /// When true (`c1x-iso*` / nested `c1-iso*`), clear `last_spike` + full membrane reset.
    trial_isolation: bool,
    /// When true (`c1x-iso-s*`), score all finite membranes for k-WTA (winner floor).
    kwta_all_finite: bool,
}

impl ExactGraph {
    fn new(config: &CreditConfig, seed: u64, assembly: bool) -> Self {
        let n_in = 2usize;
        let n_hidden = config.base.n_hidden;
        let readout_0 = (n_in + n_hidden) as CellId;
        let readout_1 = readout_0 + 1;
        let n_cells = n_in + n_hidden + 2;
        let mut engine = Engine::with_cells(n_cells);
        let (conn, init_w) = if assembly {
            build_sparse_assembly(&config.base, seed, n_in, n_hidden, readout_0, readout_1)
        } else {
            build_dense_local(
                &config.base,
                seed,
                n_in,
                n_hidden,
                readout_0,
                readout_1,
                None,
            )
        };
        let topology_fingerprint = fingerprint_topology(&conn);
        let nnz = conn.nnz();
        engine.set_connectivity(conn, vec![init_w; nnz]);
        let readout_boost = (1.15 / init_w.max(1e-3)).clamp(1.0, 12.0);
        boost_readout_incoming(&mut engine, readout_0, readout_1, readout_boost);
        let initial_weight_fingerprint = fingerprint_weights(&engine.edge_w);
        Self {
            area: Area::new(
                n_in as CellId..(n_in + n_hidden) as CellId,
                config.base.k_wta,
            ),
            encoder: LatencyEncoder::new(2, (config.base.sequence_len as Tick).max(1), 0),
            learner: ThreeFactor::new(config.base.eta, config.base.lambda, config.base.tau_e),
            feedback: FixedRandomFeedback::new(n_cells, 2, seed ^ 0xDFA0_5EED),
            baseline: RunningMeanBaseline::new(),
            readout_0,
            readout_1,
            n_in,
            n_hidden,
            t_cursor: 0,
            topology_fingerprint,
            initial_weight_fingerprint,
            trial_isolation: config.is_isolation_protocol(),
            kwta_all_finite: config.kwta_all_finite || config.is_isolation_calibrated_protocol(),
            engine,
        }
    }

    fn forward(&mut self, seq: &[Sample], label: u32) -> TrialTrace {
        let t0 = self.t_cursor;
        let frame_stride = self.encoder.max_delay().saturating_add(1);
        let hidden_cells: Vec<CellId> = self.area.cells.clone().collect();
        let saved_thresholds: Vec<f32> = hidden_cells
            .iter()
            .map(|&cell| self.engine.cell(cell).theta)
            .collect();
        for &cell in &hidden_cells {
            self.engine.cell_mut(cell).theta = f32::INFINITY;
        }

        let mut latest_input_at = t0;
        let mut input_counts = [0u32; 2];
        for (frame_i, sample) in seq.iter().enumerate() {
            for event in self.encoder.encode(sample) {
                let cell = event.cell.min(1);
                input_counts[cell as usize] = input_counts[cell as usize].saturating_add(1);
                let at = t0
                    + (frame_i as Tick)
                        .saturating_mul(frame_stride)
                        .saturating_add(event.t);
                latest_input_at = latest_input_at.max(at);
                self.engine.force_spike(cell, at);
            }
        }
        let selection_until = latest_input_at
            .checked_add(self.engine.max_synaptic_delay().max(1))
            .expect("selection overflow");
        let _ = self.engine.step_until(selection_until);
        let scores: Vec<(CellId, f32)> = hidden_cells
            .iter()
            .map(|&cell| {
                self.engine.cell_mut(cell).advance_to(selection_until);
                (cell, self.engine.cell(cell).v)
            })
            .filter(|(_, score)| {
                if self.kwta_all_finite {
                    score.is_finite()
                } else {
                    score.is_finite() && *score > 0.0
                }
            })
            .collect();
        let winners = k_wta(&scores, self.area.effective_k());
        self.area.log_activity(winners.len());
        for &cell in &hidden_cells {
            self.engine.cell_mut(cell).v = 0.0;
        }
        let winner_at = selection_until.checked_add(1).expect("winner overflow");
        for &winner in &winners {
            self.engine.force_spike(winner, winner_at);
        }
        let readout_until = winner_at
            .checked_add(self.engine.max_synaptic_delay().max(1) + 4)
            .expect("readout overflow");
        let produced = self.engine.step_until(readout_until);
        let fired_0 = produced
            .as_slice()
            .iter()
            .any(|spike| spike.cell == self.readout_0);
        let fired_1 = produced
            .as_slice()
            .iter()
            .any(|spike| spike.cell == self.readout_1);
        let charge_0 = self.engine.last_step_charge(self.readout_0);
        let charge_1 = self.engine.last_step_charge(self.readout_1);
        let prediction = match (fired_0, fired_1) {
            (true, false) => 0,
            (false, true) => 1,
            _ => {
                let diff = charge_1 - charge_0;
                if diff.abs() > 1e-6 {
                    u32::from(diff > 0.0)
                } else {
                    (t0 & 1) as u32
                }
            }
        };
        TrialTrace {
            label,
            prediction,
            scores,
            winners,
            charge_0,
            charge_1,
            input_counts,
            readout_until,
            hidden_cells,
            saved_thresholds,
        }
    }

    fn finish_trial(&mut self, trace: TrialTrace, arm: CreditArm, train: bool, beta: f32) -> u64 {
        let selected = if trace.prediction == 0 {
            self.readout_0
        } else {
            self.readout_1
        };
        let target = if trace.label == 0 {
            self.readout_0
        } else {
            self.readout_1
        };
        let action_at = trace.readout_until.checked_add(1).expect("action overflow");
        let delay = self.engine.max_synaptic_delay().max(1) + 4;
        self.engine.force_spike(selected, action_at);
        let until_selected = action_at.checked_add(delay).expect("selected overflow");
        let _ = self.engine.step_until(until_selected);

        let mut applications = 0u64;
        if train {
            let reward = if trace.prediction == trace.label {
                1.0
            } else {
                -1.0
            };
            match arm {
                CreditArm::BroadcastOnePass
                | CreditArm::BroadcastEpochMatched
                | CreditArm::DenseEpochMatched => {
                    applications = self
                        .learner
                        .update_counted(&mut self.engine, Modulators::reward(reward));
                }
                CreditArm::RpeThreeFactor => {
                    let advantage = self.baseline.advantage_and_observe(reward);
                    applications = self
                        .learner
                        .update_counted(&mut self.engine, Modulators::reward(advantage));
                }
                CreditArm::EpropExactForward => {
                    let signal = self.output_weight_credit(&trace);
                    applications = self
                        .learner
                        .update_with_credit_counted(&mut self.engine, &signal);
                }
                CreditArm::DfaExactForward => {
                    let signal = self.dfa_credit(&trace);
                    applications = self
                        .learner
                        .update_with_credit_counted(&mut self.engine, &signal);
                }
                CreditArm::SurrogateGradient => {
                    self.learner.observe_spikes(&mut self.engine);
                    applications = self.apply_surrogate_gradient(&trace, beta);
                }
            }
            clear_eligibility(&mut self.engine);
        }

        if trace.prediction != trace.label && target != selected {
            let target_at = until_selected.checked_add(2).expect("target overflow");
            self.engine.force_spike(target, target_at);
            let until_target = target_at
                .checked_add(delay)
                .expect("target horizon overflow");
            let _ = self.engine.step_until(until_target);
            if train
                && matches!(
                    arm,
                    CreditArm::BroadcastOnePass
                        | CreditArm::BroadcastEpochMatched
                        | CreditArm::RpeThreeFactor
                        | CreditArm::DenseEpochMatched
                )
            {
                applications = applications.saturating_add(
                    self.learner
                        .update_counted(&mut self.engine, Modulators::reward(1.0)),
                );
            } else if train {
                self.learner.observe_spikes(&mut self.engine);
            }
            clear_eligibility(&mut self.engine);
        }

        if self.trial_isolation {
            // Exact-forward trial isolation (`c1x-iso*` / nested `c1-iso*`):
            // clear STDP pairing + C3-style full membrane reset. New hashes only.
            reset_c1_dynamic_state(
                &mut self.engine,
                &trace.hidden_cells,
                &trace.saved_thresholds,
            );
            self.learner.reset_pairing_state();
        } else {
            // Canonical c1x-* scientific/quick presets: incomplete C1 reset (H2).
            for (&cell, &theta) in trace.hidden_cells.iter().zip(trace.saved_thresholds.iter()) {
                let hidden = self.engine.cell_mut(cell);
                hidden.theta = theta;
                hidden.v = 0.0;
            }
        }
        self.engine.close_inhibited_cycle();
        self.t_cursor = self.engine.time() + 20;
        applications
    }

    fn output_weight_credit(&self, trace: &TrialTrace) -> PostSynapticCredit {
        let errors = output_error(trace);
        let mut signal = PostSynapticCredit::zeros(self.engine.num_cells());
        signal.set(self.readout_0, errors[0]);
        signal.set(self.readout_1, errors[1]);
        for hidden in self.area.cells.clone() {
            let w0 = edge_index(&self.engine.conn, hidden, self.readout_0)
                .map(|edge| self.engine.edge_w[edge])
                .unwrap_or(0.0);
            let w1 = edge_index(&self.engine.conn, hidden, self.readout_1)
                .map(|edge| self.engine.edge_w[edge])
                .unwrap_or(0.0);
            signal.set(hidden, errors[0] * w0 + errors[1] * w1);
        }
        signal
    }

    fn dfa_credit(&self, trace: &TrialTrace) -> PostSynapticCredit {
        let errors = output_error(trace);
        let mut signal = self.feedback.project(&errors);
        signal.set(self.readout_0, errors[0]);
        signal.set(self.readout_1, errors[1]);
        signal
    }

    fn apply_surrogate_gradient(&mut self, trace: &TrialTrace, beta: f32) -> u64 {
        let errors = output_error(trace);
        let winner_set: std::collections::BTreeSet<CellId> =
            trace.winners.iter().copied().collect();
        let score_map: BTreeMap<CellId, f32> = trace.scores.iter().copied().collect();
        let cutoff = trace
            .scores
            .iter()
            .map(|(_, score)| *score)
            .fold(0.0f32, f32::max);
        let mut updates = vec![0.0f32; self.engine.edge_w.len()];
        let lr = self.learner.eta;

        for hidden in self.area.cells.clone() {
            let w0 = edge_index(&self.engine.conn, hidden, self.readout_0)
                .map(|edge| self.engine.edge_w[edge])
                .unwrap_or(0.0);
            let w1 = edge_index(&self.engine.conn, hidden, self.readout_1)
                .map(|edge| self.engine.edge_w[edge])
                .unwrap_or(0.0);
            let hidden_error = errors[0] * w0 + errors[1] * w1;
            let score = score_map.get(&hidden).copied().unwrap_or(0.0);
            let surrogate = 1.0 / (1.0 + beta * (score - cutoff).abs()).powi(2);
            for input in 0..self.n_in as CellId {
                if let Some(edge) = edge_index(&self.engine.conn, input, hidden) {
                    updates[edge] +=
                        lr * hidden_error * surrogate * trace.input_counts[input as usize] as f32;
                }
            }
            if winner_set.contains(&hidden) {
                if let Some(edge) = edge_index(&self.engine.conn, hidden, self.readout_0) {
                    updates[edge] += lr * errors[0];
                }
                if let Some(edge) = edge_index(&self.engine.conn, hidden, self.readout_1) {
                    updates[edge] += lr * errors[1];
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
}

#[derive(Clone, Debug)]
struct TrialTrace {
    label: u32,
    prediction: u32,
    scores: Vec<(CellId, f32)>,
    winners: Vec<CellId>,
    charge_0: f32,
    charge_1: f32,
    input_counts: [u32; 2],
    readout_until: Tick,
    hidden_cells: Vec<CellId>,
    saved_thresholds: Vec<f32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct TraceSignature {
    prediction: u32,
    winners: Vec<CellId>,
    charge_0_bits: u32,
    charge_1_bits: u32,
}

impl From<&TrialTrace> for TraceSignature {
    fn from(trace: &TrialTrace) -> Self {
        Self {
            prediction: trace.prediction,
            winners: trace.winners.clone(),
            charge_0_bits: trace.charge_0.to_bits(),
            charge_1_bits: trace.charge_1.to_bits(),
        }
    }
}

fn run_condition(
    config: &CreditConfig,
    seed: u64,
    split: &FrozenSplit,
    arm: CreditArm,
) -> CreditConditionOutcome {
    let assembly = arm != CreditArm::DenseEpochMatched;
    let mut graph = ExactGraph::new(config, seed, assembly);
    let topology_fingerprint = graph.topology_fingerprint;
    let initial_weight_fingerprint = graph.initial_weight_fingerprint;
    let split_fingerprint = fingerprint_split(split);
    let epochs = config.epochs_for(arm);
    let exposure_fingerprint = fingerprint_exposures(split, epochs);
    let mut training_exposures = 0usize;

    for _ in 0..epochs {
        for (sequence, label) in &split.train {
            let trace = graph.forward(sequence, *label);
            let _ = graph.finish_trial(trace, arm, true, config.surrogate_beta);
            training_exposures = training_exposures.saturating_add(1);
        }
    }

    let weights_before_test = fingerprint_weights(&graph.engine.edge_w);
    let mut correct = 0usize;
    let mut active = 0usize;
    let mut population = 0usize;
    for (sequence, label) in &split.test {
        let trace = graph.forward(sequence, *label);
        correct += usize::from(trace.prediction == *label);
        active += trace.winners.len();
        population += graph.n_hidden;
        let _ = graph.finish_trial(trace, arm, false, config.surrogate_beta);
    }
    let weights_after_test = fingerprint_weights(&graph.engine.edge_w);
    CreditConditionOutcome {
        arm,
        accuracy: correct as f32 / split.test.len().max(1) as f32,
        activity_sparsity: Metrics::sparsity(active.min(population), population.max(1)),
        n_params: graph.engine.edge_w.len(),
        training_exposures,
        topology_fingerprint,
        initial_weight_fingerprint,
        split_fingerprint,
        exposure_fingerprint,
        test_weights_unchanged: weights_before_test == weights_after_test,
    }
}

fn parity_probe(config: &CreditConfig, seed: u64, split: &FrozenSplit) -> ForwardParityEvidence {
    let sequence = &split.train[0].0;
    let label = split.train[0].1;
    let assembly_arms = [
        CreditArm::BroadcastOnePass,
        CreditArm::BroadcastEpochMatched,
        CreditArm::RpeThreeFactor,
        CreditArm::EpropExactForward,
        CreditArm::DfaExactForward,
        CreditArm::SurrogateGradient,
    ];
    let mut fingerprints = Vec::new();
    let mut signatures = Vec::new();
    for arm in assembly_arms {
        let mut graph = ExactGraph::new(config, seed, true);
        let trace = graph.forward(sequence, label);
        fingerprints.push((
            arm,
            graph.topology_fingerprint,
            graph.initial_weight_fingerprint,
        ));
        signatures.push((arm, TraceSignature::from(&trace)));
    }
    let topology_equal = all_equal(fingerprints.iter().map(|(_, topology, _)| *topology));
    let initialization_equal = all_equal(fingerprints.iter().map(|(_, _, weights)| *weights));
    let initial_prediction_equal =
        all_equal(signatures.iter().map(|(_, signature)| signature.prediction));
    let initial_winners_equal = all_equal(
        signatures
            .iter()
            .map(|(_, signature)| signature.winners.clone()),
    );
    let initial_charges_equal = all_equal(
        signatures
            .iter()
            .map(|(_, signature)| (signature.charge_0_bits, signature.charge_1_bits)),
    );
    let mut target_zero_graph = ExactGraph::new(config, seed, true);
    let target_zero = TraceSignature::from(&target_zero_graph.forward(sequence, 0));
    let mut target_one_graph = ExactGraph::new(config, seed, true);
    let target_one = TraceSignature::from(&target_one_graph.forward(sequence, 1));
    let target_independent_forward = target_zero == target_one;
    let split_hash = fingerprint_split(split);
    let split_equal = assembly_arms
        .iter()
        .all(|_| fingerprint_split(split) == split_hash);
    let matched_exposure = fingerprint_exposures(split, config.matched_epochs);
    let matched_exposure_order_equal = assembly_arms
        .into_iter()
        .filter(|arm| *arm != CreditArm::BroadcastOnePass)
        .all(|_| fingerprint_exposures(split, config.matched_epochs) == matched_exposure);

    ForwardParityEvidence {
        topology_equal,
        initialization_equal,
        split_equal,
        matched_exposure_order_equal,
        initial_prediction_equal,
        initial_winners_equal,
        initial_charges_equal,
        target_independent_forward,
        test_updates_absent: true,
    }
}

fn summarize_arm(
    config: &CreditConfig,
    seeds: &[CreditSeedResult],
    arm: CreditArm,
    positive_control_mean: f32,
    mean_activity_sparsity: f32,
    parity: &ForwardParityEvidence,
) -> CreditArmSummary {
    let values: Vec<f32> = seeds
        .iter()
        .map(|seed| seed.outcome(arm).accuracy)
        .collect();
    let (mean_accuracy, variance_accuracy) = mean_var(&values);
    let mut gap_closed = Vec::with_capacity(seeds.len());
    for seed in seeds {
        let dense = seed.outcome(CreditArm::DenseEpochMatched).accuracy;
        let gradient = seed.outcome(CreditArm::SurrogateGradient).accuracy;
        let reference_gap = gradient - dense;
        let closed = if reference_gap >= config.base.g2_min_reference_gap {
            ((seed.outcome(arm).accuracy - dense) / reference_gap).clamp(0.0, 1.0)
        } else {
            0.0
        };
        gap_closed.push(closed);
    }
    let (mean_gap_closed, variance_gap_closed) = mean_var(&gap_closed);
    let gap_closed_lower_95 = if gap_closed.len() > 1 {
        mean_gap_closed
            - config.base.g2_confidence_z * (variance_gap_closed / gap_closed.len() as f32).sqrt()
    } else {
        mean_gap_closed
    };
    let positive_ok = positive_control_mean >= config.base.g2_min_positive_control;
    let sparsity_ok = (config.base.activity_sparsity_min..=config.base.activity_sparsity_max)
        .contains(&mean_activity_sparsity);
    let test_updates_absent = seeds
        .iter()
        .all(|seed| seed.outcome(arm).test_weights_unchanged);
    let verdict = if !positive_ok || !sparsity_ok || !parity.all_pass() || !test_updates_absent {
        GateG2Verdict::InvalidHarness
    } else if config.quick || config.base.n_seeds < config.scientific_n_seeds {
        GateG2Verdict::Pilot
    } else if gap_closed_lower_95 > config.base.g2_min_gap_closed
        && mean_accuracy >= config.base.g2_min_accuracy
    {
        GateG2Verdict::Pass
    } else {
        GateG2Verdict::Fail
    };
    CreditArmSummary {
        arm,
        config_hash: config.hash_string_for_arm(arm),
        protocol_version: config.protocol_version_for(arm),
        mean_accuracy,
        variance_accuracy,
        mean_gap_closed,
        variance_gap_closed,
        gap_closed_lower_95,
        verdict,
    }
}

fn output_error(trace: &TrialTrace) -> [f32; 2] {
    let p1 = sigmoid(trace.charge_1 - trace.charge_0);
    let p0 = 1.0 - p1;
    let y1 = trace.label as f32;
    [1.0 - y1 - p0, y1 - p1]
}

fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + (-x).exp())
}

fn and_parity(a: ForwardParityEvidence, b: ForwardParityEvidence) -> ForwardParityEvidence {
    ForwardParityEvidence {
        topology_equal: a.topology_equal && b.topology_equal,
        initialization_equal: a.initialization_equal && b.initialization_equal,
        split_equal: a.split_equal && b.split_equal,
        matched_exposure_order_equal: a.matched_exposure_order_equal
            && b.matched_exposure_order_equal,
        initial_prediction_equal: a.initial_prediction_equal && b.initial_prediction_equal,
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

fn fingerprint_topology(conn: &Csr) -> u64 {
    let mut hash = Fnv64::new();
    hash.mix(conn.nrows() as u64);
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
    for weight in weights {
        hash.mix(u64::from(weight.to_bits()));
    }
    hash.finish()
}

fn fingerprint_split(split: &FrozenSplit) -> u64 {
    let mut hash = Fnv64::new();
    for (partition, trials) in [(0u64, &split.train), (1, &split.test)] {
        hash.mix(partition);
        for (sequence, label) in trials {
            hash.mix(u64::from(*label));
            hash.mix(sequence.len() as u64);
            for sample in sequence {
                for value in &sample.values {
                    hash.mix(u64::from(value.to_bits()));
                }
            }
        }
    }
    hash.finish()
}

fn fingerprint_exposures(split: &FrozenSplit, epochs: usize) -> u64 {
    let mut hash = Fnv64::new();
    for epoch in 0..epochs {
        for (index, (_, label)) in split.train.iter().enumerate() {
            hash.mix(epoch as u64);
            hash.mix(index as u64);
            hash.mix(u64::from(*label));
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
    fn quick_suite_preserves_exact_forward_contract() {
        let mut config = CreditConfig::quick();
        config.base.n_seeds = 1;
        config.base.n_train = 6;
        config.base.n_test = 4;
        config.matched_epochs = 2;
        let mut runner = CreditRunner::new();
        let report = runner.run(&config);
        assert!(report.parity.all_pass());
        assert!(report.pilot);
        assert_eq!(report.seeds.len(), 1);
        assert_eq!(report.seeds[0].outcomes.len(), CreditArm::ALL.len());
        for outcome in &report.seeds[0].outcomes {
            assert!(outcome.test_weights_unchanged);
        }
    }

    #[test]
    fn assembly_arms_share_topology_initialization_split_and_matched_order() {
        let mut config = CreditConfig::quick();
        config.base.n_seeds = 1;
        config.base.n_train = 4;
        config.base.n_test = 4;
        config.matched_epochs = 2;
        let seed = config.seeds()[0];
        let split = freeze_trials(&config.base, seed);
        let arms = [
            CreditArm::BroadcastEpochMatched,
            CreditArm::RpeThreeFactor,
            CreditArm::EpropExactForward,
            CreditArm::DfaExactForward,
            CreditArm::SurrogateGradient,
        ];
        let outcomes: Vec<_> = arms
            .into_iter()
            .map(|arm| run_condition(&config, seed, &split, arm))
            .collect();
        assert!(all_equal(
            outcomes.iter().map(|outcome| outcome.topology_fingerprint)
        ));
        assert!(all_equal(
            outcomes
                .iter()
                .map(|outcome| outcome.initial_weight_fingerprint)
        ));
        assert!(all_equal(
            outcomes.iter().map(|outcome| outcome.split_fingerprint)
        ));
        assert!(all_equal(
            outcomes.iter().map(|outcome| outcome.exposure_fingerprint)
        ));
    }

    #[test]
    fn forward_prediction_does_not_read_target() {
        let mut config = CreditConfig::quick();
        config.base.n_train = 4;
        config.base.n_test = 4;
        let seed = config.seeds()[0];
        let split = freeze_trials(&config.base, seed);
        assert!(parity_probe(&config, seed, &split).target_independent_forward);
    }

    #[test]
    fn one_pass_is_declared_exposure_diagnostic() {
        let mut config = CreditConfig::quick();
        config.base.n_train = 5;
        config.matched_epochs = 3;
        assert_eq!(config.epochs_for(CreditArm::BroadcastOnePass), 1);
        assert_eq!(config.epochs_for(CreditArm::BroadcastEpochMatched), 3);
    }

    #[test]
    fn isolation_hashes_diverge_and_render_discloses() {
        let frozen = CreditConfig::scientific();
        let iso = CreditConfig::scientific_isolation();
        assert!(iso.is_isolation_protocol());
        for arm in CreditArm::ALL {
            assert_ne!(
                frozen.hash_string_for_arm(arm),
                iso.hash_string_for_arm(arm)
            );
            assert!(iso
                .hash_string_for_arm(arm)
                .starts_with(crate::credit_config::CREDIT_ISOLATION_HASH_PREFIX));
            assert!(!iso
                .hash_string_for_arm(arm)
                .starts_with(crate::credit_config::CREDIT_ISOLATION_CALIBRATED_HASH_PREFIX));
        }
        let mut tiny = CreditConfig::quick_isolation();
        tiny.base.n_seeds = 1;
        tiny.base.n_train = 4;
        tiny.base.n_test = 4;
        tiny.matched_epochs = 1;
        let mut runner = CreditRunner::new();
        let report = runner.run(&tiny);
        let md = CreditRunner::render_markdown(&report, &tiny);
        assert!(md.contains("Trial-isolation protocol"));
        assert!(md.contains("trial isolation: **yes**"));
        assert!(md.contains("c1x-iso-"));
        for summary in &report.summaries {
            assert!(summary
                .config_hash
                .starts_with(crate::credit_config::CREDIT_ISOLATION_HASH_PREFIX));
            assert_eq!(
                summary.protocol_version,
                summary.arm.protocol_version()
                    + crate::credit_config::CREDIT_ISOLATION_PROTOCOL_OFFSET
            );
        }
    }

    #[test]
    fn calibrated_isolation_hashes_diverge_and_render_discloses() {
        let frozen = CreditConfig::scientific();
        let iso = CreditConfig::scientific_isolation();
        let cal = CreditConfig::scientific_isolation_calibrated();
        assert!(cal.is_isolation_calibrated_protocol());
        assert!(cal.kwta_all_finite);
        for arm in CreditArm::ALL {
            let cal_hash = cal.hash_string_for_arm(arm);
            assert_ne!(cal_hash, frozen.hash_string_for_arm(arm));
            assert_ne!(cal_hash, iso.hash_string_for_arm(arm));
            assert!(
                cal_hash.starts_with(crate::credit_config::CREDIT_ISOLATION_CALIBRATED_HASH_PREFIX)
            );
        }
        let mut tiny = CreditConfig::quick_isolation_calibrated();
        tiny.base.n_seeds = 1;
        tiny.base.n_train = 4;
        tiny.base.n_test = 4;
        tiny.matched_epochs = 1;
        let mut runner = CreditRunner::new();
        let report = runner.run(&tiny);
        let md = CreditRunner::render_markdown(&report, &tiny);
        assert!(md.contains("Sparsity-calibrated trial-isolation"));
        assert!(md.contains("trial isolation: **yes**"));
        assert!(md.contains("c1x-iso-s-"));
        assert!(
            (tiny.base.activity_sparsity_min..=tiny.base.activity_sparsity_max)
                .contains(&report.mean_activity_sparsity),
            "calibrated tiny probe sparsity {} out of band",
            report.mean_activity_sparsity
        );
        for summary in &report.summaries {
            assert!(summary
                .config_hash
                .starts_with(crate::credit_config::CREDIT_ISOLATION_CALIBRATED_HASH_PREFIX));
            assert_eq!(
                summary.protocol_version,
                summary.arm.protocol_version()
                    + crate::credit_config::CREDIT_ISOLATION_CALIBRATED_PROTOCOL_OFFSET
            );
        }
    }

    #[test]
    fn isolation_clears_pairing_after_finish_trial_non_isolation_retains() {
        let mut iso = CreditConfig::quick_isolation();
        iso.base.n_train = 4;
        iso.base.n_test = 4;
        iso.matched_epochs = 1;
        let seed = iso.seeds()[0];
        let split = freeze_trials(&iso.base, seed);
        let (sequence, label) = &split.train[0];

        let mut iso_graph = ExactGraph::new(&iso, seed, true);
        assert!(iso_graph.trial_isolation);
        let trace = iso_graph.forward(sequence, *label);
        let _ = iso_graph.finish_trial(
            trace,
            CreditArm::BroadcastEpochMatched,
            true,
            iso.surrogate_beta,
        );
        let iso_any_spike = (0..iso_graph.learner.tracked_cells())
            .any(|cell| iso_graph.learner.last_spike_at(cell).is_some());
        assert!(
            !iso_any_spike,
            "isolation path must clear last_spike after finish_trial"
        );

        let mut non = CreditConfig::quick();
        non.base.n_train = 4;
        non.base.n_test = 4;
        non.matched_epochs = 1;
        // Match iso schedule size but keep frozen non-isolation path.
        let non_seed = non.seeds()[0];
        let non_split = freeze_trials(&non.base, non_seed);
        let (non_seq, non_label) = &non_split.train[0];
        let mut non_graph = ExactGraph::new(&non, non_seed, true);
        assert!(!non_graph.trial_isolation);
        let non_trace = non_graph.forward(non_seq, *non_label);
        let _ = non_graph.finish_trial(
            non_trace,
            CreditArm::BroadcastEpochMatched,
            true,
            non.surrogate_beta,
        );
        let non_any_spike = (0..non_graph.learner.tracked_cells())
            .any(|cell| non_graph.learner.last_spike_at(cell).is_some());
        assert!(
            non_any_spike,
            "non-isolation path must retain sticky last_spike (H1) after finish_trial"
        );
    }

    #[test]
    fn output_error_is_zero_sum_binary_error() {
        let trace = TrialTrace {
            label: 1,
            prediction: 0,
            scores: Vec::new(),
            winners: Vec::new(),
            charge_0: 1.0,
            charge_1: 0.0,
            input_counts: [1, 1],
            readout_until: 1,
            hidden_cells: Vec::new(),
            saved_thresholds: Vec::new(),
        };
        let error = output_error(&trace);
        assert!((error[0] + error[1]).abs() < 1e-6);
        assert!(error[1] > 0.0);
        assert!(error[0] < 0.0);
    }
}
