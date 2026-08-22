//! True surrogate-derivative e-prop on the exact-forward C1 graph.
//!
//! Forward matches [`crate::runner_credit::ExactGraph`] (muted hidden θ, membrane
//! k-WTA, forced winners). The true-surrogate arm computes eligibility as
//! `e ∝ σ'(score − cutoff) · pre_activity` and applies `Δw = η · δ · e` with δ
//! transported through readout weights — **without** `ThreeFactor::absorb_spikes`.
//! The hybrid contrast arm uses STDP eligibility × transported M (frozen
//! `c1x-eprop-exact-forward-*` mechanism).

use std::collections::BTreeMap;

use binn_areas::{k_wta, Area};
use binn_core::Tick;
use binn_data::{Encoder, LatencyEncoder, Metrics, Sample};
use binn_engine::{CellId, Engine};
use binn_learn::{PostSynapticCredit, ThreeFactor};

use crate::eprop_true_config::{EpropTrueArm, EpropTrueConfig, EPROP_TRUE_PROTOCOL_VERSION};
use crate::runner::{
    boost_readout_incoming, build_sparse_assembly, clear_eligibility, edge_index, freeze_trials,
    mean, mean_var, run_positive_control, FrozenSplit,
};

pub const TRUE_SURROGATE_EPROP_REFERENCE: &str = "TRUE_SURROGATE_DERIVATIVE_EPROP_REFERENCE";
pub const HYBRID_STDP_EPROP_CONTRAST: &str = "HYBRID_STDP_ELIGIBILITY_EPROP_CONTRAST";

#[derive(Clone, Debug, PartialEq)]
pub struct EpropTrueArmSummary {
    pub arm: EpropTrueArm,
    pub config_hash: String,
    pub protocol_version: u64,
    pub mean_accuracy: f32,
    pub variance_accuracy: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EpropTrueSeedResult {
    pub seed: u64,
    pub true_surrogate_accuracy: f32,
    pub hybrid_stdp_accuracy: f32,
    pub true_surrogate_updates: u64,
    pub hybrid_stdp_updates: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EpropTrueReport {
    pub protocol_version: u64,
    pub seeds: Vec<EpropTrueSeedResult>,
    pub summaries: Vec<EpropTrueArmSummary>,
    pub positive_control_mean: f32,
    pub mean_activity_sparsity: f32,
    pub pilot: bool,
}

#[derive(Default)]
pub struct EpropTrueRunner;

impl EpropTrueRunner {
    pub fn new() -> Self {
        Self
    }

    pub fn run(&mut self, config: &EpropTrueConfig) -> EpropTrueReport {
        assert!(config.base.n_seeds >= 1);
        assert!(config.matched_epochs >= 1);
        let mut seeds = Vec::with_capacity(config.base.n_seeds);
        let mut positive_controls = Vec::with_capacity(config.base.n_seeds);
        let mut sparsities = Vec::with_capacity(config.base.n_seeds);

        for seed in config.seeds() {
            let split = freeze_trials(&config.base, seed);
            let true_out = run_arm(config, seed, &split, EpropTrueArm::TrueSurrogate);
            let hybrid_out = run_arm(config, seed, &split, EpropTrueArm::HybridStdp);
            sparsities.push(true_out.activity_sparsity);
            seeds.push(EpropTrueSeedResult {
                seed,
                true_surrogate_accuracy: true_out.accuracy,
                hybrid_stdp_accuracy: hybrid_out.accuracy,
                true_surrogate_updates: true_out.training_updates,
                hybrid_stdp_updates: hybrid_out.training_updates,
            });
            positive_controls.push(run_positive_control(&config.base, seed));
        }

        let summaries = EpropTrueArm::ALL
            .into_iter()
            .map(|arm| summarize_arm(config, &seeds, arm))
            .collect();

        EpropTrueReport {
            protocol_version: EPROP_TRUE_PROTOCOL_VERSION,
            seeds,
            summaries,
            positive_control_mean: mean(&positive_controls),
            mean_activity_sparsity: mean(&sparsities),
            pilot: config.quick || config.base.n_seeds < config.scientific_n_seeds,
        }
    }

    pub fn render_markdown(report: &EpropTrueReport, config: &EpropTrueConfig) -> String {
        let mut md = String::new();
        md.push_str("# BINN true surrogate e-prop on exact-forward C1\n\n");
        md.push_str(
            "**Canonical C1 is immutable:** protocol-v2 hash `c1-118207fbc3eaba53` \
             and every G2 threshold remain unchanged. This is a **separate** protocol \
             family (`c1x-eprop-true-*`) and does **not** reopen frozen hybrid \
             `c1x-eprop-exact-forward-fcedc76a80ff0f0e`.\n\n",
        );
        md.push_str("## Mechanism disclosure\n\n");
        md.push_str(&format!(
            "| arm | label | eligibility construction |\n\
             |---|---|---|\n\
             | `{TRUE_SURROGATE_EPROP_REFERENCE}` | true-surrogate | \
             `e ∝ σ'(score − cutoff) · pre_activity`; `Δw = η · δ · e` with δ from \
             output error transported through readout weights; **no** \
             `ThreeFactor::absorb_spikes` / STDP pairing |\n\
             | `{HYBRID_STDP_EPROP_CONTRAST}` | hybrid-stdp (contrast) | production \
             STDP eligibility × output-weight-transported M (same as frozen \
             `c1x-eprop-exact-forward-*`) |\n\n",
        ));
        md.push_str(&format!(
            "- protocol version: {}\n\
             - schedule: **{}**\n\
             - seeds: {}\n\
             - matched epochs: {}\n\
             - surrogate β: {:.1}\n\
             - positive control mean: {:.4}\n\
             - mean activity sparsity: {:.4}\n\n",
            report.protocol_version,
            if report.pilot { "PILOT" } else { "SCIENTIFIC" },
            config.base.n_seeds,
            config.matched_epochs,
            config.surrogate_beta,
            report.positive_control_mean,
            report.mean_activity_sparsity,
        ));

        md.push_str("## Arm hashes\n\n");
        md.push_str("| arm | hash | mean accuracy | variance |\n|---|---|---:|---:|\n");
        for summary in &report.summaries {
            md.push_str(&format!(
                "| `{}` | `{}` | {:.4} | {:.6} |\n",
                summary.arm.as_str(),
                summary.config_hash,
                summary.mean_accuracy,
                summary.variance_accuracy
            ));
        }

        md.push_str("\n## Per-seed\n\n");
        md.push_str(
            "| seed | true-surrogate | hybrid-stdp | true updates | hybrid updates |\n\
             |---:|---:|---:|---:|---:|\n",
        );
        for seed in &report.seeds {
            md.push_str(&format!(
                "| {} | {:.4} | {:.4} | {} | {} |\n",
                seed.seed,
                seed.true_surrogate_accuracy,
                seed.hybrid_stdp_accuracy,
                seed.true_surrogate_updates,
                seed.hybrid_stdp_updates
            ));
        }

        md.push_str(
            "\n## Interpretation contract\n\n\
             - True surrogate e-prop tests whether explicit σ′ eligibility (not STDP \
             absorb) can assign credit on the exact-forward graph.\n\
             - Hybrid STDP×M is included only as a labeled contrast to frozen \
             `c1x-eprop-exact-forward-*`; outcomes are not comparable across hash families.\n\
             - No outcome reopens canonical protocol-v2 G2.\n",
        );
        md
    }
}

struct ArmOutcome {
    accuracy: f32,
    activity_sparsity: f32,
    training_updates: u64,
}

struct TrueEpropGraph {
    engine: Engine,
    area: Area,
    encoder: LatencyEncoder,
    learner: ThreeFactor,
    readout_0: CellId,
    readout_1: CellId,
    n_in: usize,
    n_hidden: usize,
    eta: f32,
    t_cursor: Tick,
    /// Set when hybrid path calls STDP absorb (test introspection).
    stdp_absorb_used: bool,
}

impl TrueEpropGraph {
    fn new(config: &EpropTrueConfig, seed: u64) -> Self {
        let n_in = 2usize;
        let n_hidden = config.base.n_hidden;
        let readout_0 = (n_in + n_hidden) as CellId;
        let readout_1 = readout_0 + 1;
        let n_cells = n_in + n_hidden + 2;
        let mut engine = Engine::with_cells(n_cells);
        let (conn, init_w) =
            build_sparse_assembly(&config.base, seed, n_in, n_hidden, readout_0, readout_1);
        let nnz = conn.nnz();
        engine.set_connectivity(conn, vec![init_w; nnz]);
        let readout_boost = (1.15 / init_w.max(1e-3)).clamp(1.0, 12.0);
        boost_readout_incoming(&mut engine, readout_0, readout_1, readout_boost);
        Self {
            engine,
            area: Area::new(
                n_in as CellId..(n_in + n_hidden) as CellId,
                config.base.k_wta,
            ),
            encoder: LatencyEncoder::new(2, (config.base.sequence_len as Tick).max(1), 0),
            learner: ThreeFactor::new(config.base.eta, config.base.lambda, config.base.tau_e),
            readout_0,
            readout_1,
            n_in,
            n_hidden,
            eta: config.base.eta,
            t_cursor: 0,
            stdp_absorb_used: false,
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
            .filter(|(_, score)| score.is_finite() && *score > 0.0)
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

    fn finish_trial(
        &mut self,
        trace: TrialTrace,
        arm: EpropTrueArm,
        train: bool,
        beta: f32,
    ) -> u64 {
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
            match arm {
                EpropTrueArm::TrueSurrogate => {
                    applications = self.apply_true_surrogate_eprop(&trace, beta);
                }
                EpropTrueArm::HybridStdp => {
                    let signal = self.output_weight_credit(&trace);
                    self.stdp_absorb_used = true;
                    applications = self
                        .learner
                        .update_with_credit_counted(&mut self.engine, &signal);
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
            if train && arm == EpropTrueArm::HybridStdp {
                self.learner.observe_spikes(&mut self.engine);
                self.stdp_absorb_used = true;
            }
            clear_eligibility(&mut self.engine);
        }

        for (&cell, &theta) in trace.hidden_cells.iter().zip(trace.saved_thresholds.iter()) {
            let hidden = self.engine.cell_mut(cell);
            hidden.theta = theta;
            hidden.v = 0.0;
        }
        self.engine.close_inhibited_cycle();
        self.t_cursor = self.engine.time() + 20;
        applications
    }

    /// True surrogate e-prop: explicit σ′ eligibility, no STDP absorb.
    fn apply_true_surrogate_eprop(&mut self, trace: &TrialTrace, beta: f32) -> u64 {
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
        let lr = self.eta;

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

fn run_arm(
    config: &EpropTrueConfig,
    seed: u64,
    split: &FrozenSplit,
    arm: EpropTrueArm,
) -> ArmOutcome {
    let mut graph = TrueEpropGraph::new(config, seed);
    let mut training_updates = 0u64;
    for _ in 0..config.matched_epochs {
        for (sequence, label) in &split.train {
            let trace = graph.forward(sequence, *label);
            training_updates = training_updates.saturating_add(graph.finish_trial(
                trace,
                arm,
                true,
                config.surrogate_beta,
            ));
        }
    }
    let mut correct = 0usize;
    let mut active = 0usize;
    let population = graph.n_hidden;
    for (sequence, label) in &split.test {
        let trace = graph.forward(sequence, *label);
        correct += usize::from(trace.prediction == *label);
        active += trace.winners.len();
        let _ = graph.finish_trial(trace, arm, false, config.surrogate_beta);
    }
    ArmOutcome {
        accuracy: correct as f32 / split.test.len().max(1) as f32,
        activity_sparsity: Metrics::sparsity(active.min(population), population.max(1)),
        training_updates,
    }
}

fn summarize_arm(
    config: &EpropTrueConfig,
    seeds: &[EpropTrueSeedResult],
    arm: EpropTrueArm,
) -> EpropTrueArmSummary {
    let values: Vec<f32> = seeds
        .iter()
        .map(|seed| match arm {
            EpropTrueArm::TrueSurrogate => seed.true_surrogate_accuracy,
            EpropTrueArm::HybridStdp => seed.hybrid_stdp_accuracy,
        })
        .collect();
    let (mean_accuracy, variance_accuracy) = mean_var(&values);
    EpropTrueArmSummary {
        arm,
        config_hash: config.hash_string_for_arm(arm),
        protocol_version: EPROP_TRUE_PROTOCOL_VERSION,
        mean_accuracy,
        variance_accuracy,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CreditArm, CreditConfig};

    #[test]
    fn true_eprop_does_not_use_stdp_absorb() {
        let mut config = EpropTrueConfig::quick();
        config.base.n_train = 4;
        config.base.n_test = 4;
        config.matched_epochs = 1;
        let seed = config.seeds()[0];
        let split = freeze_trials(&config.base, seed);
        let (sequence, label) = &split.train[0];

        let mut true_graph = TrueEpropGraph::new(&config, seed);
        let trace = true_graph.forward(sequence, *label);
        let updates = true_graph.finish_trial(
            trace,
            EpropTrueArm::TrueSurrogate,
            true,
            config.surrogate_beta,
        );
        assert!(!true_graph.stdp_absorb_used);
        assert!(
            true_graph.learner.last_spike_at(0).is_none(),
            "true path must not absorb spikes into ThreeFactor"
        );
        assert!(updates > 0, "true path should apply direct weight updates");

        let mut hybrid_graph = TrueEpropGraph::new(&config, seed);
        let hybrid_trace = hybrid_graph.forward(sequence, *label);
        let _ = hybrid_graph.finish_trial(
            hybrid_trace,
            EpropTrueArm::HybridStdp,
            true,
            config.surrogate_beta,
        );
        assert!(
            hybrid_graph.stdp_absorb_used,
            "hybrid contrast must use STDP absorb"
        );
    }

    #[test]
    fn hash_distinct_from_frozen_hybrid_eprop() {
        let cfg = EpropTrueConfig::scientific();
        let frozen = CreditConfig::scientific().hash_string_for_arm(CreditArm::EpropExactForward);
        for arm in EpropTrueArm::ALL {
            assert_ne!(cfg.hash_string_for_arm(arm), frozen);
        }
    }

    #[test]
    fn quick_run_finishes() {
        let mut config = EpropTrueConfig::quick();
        config.base.n_seeds = 1;
        config.base.n_train = 6;
        config.base.n_test = 4;
        config.matched_epochs = 2;
        let mut runner = EpropTrueRunner::new();
        let report = runner.run(&config);
        assert!(report.pilot);
        assert_eq!(report.seeds.len(), 1);
        assert_eq!(report.summaries.len(), EpropTrueArm::ALL.len());
        let md = EpropTrueRunner::render_markdown(&report, &config);
        assert!(md.contains("true surrogate"));
        assert!(md.contains("hybrid"));
        assert!(md.contains("c1x-eprop-exact-forward-fcedc76a80ff0f0e"));
    }
}
