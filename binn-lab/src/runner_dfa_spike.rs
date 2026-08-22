//! Spiking-path DFA rescue runner (`c1x-dfa-spike-*`).
//!
//! Forward matches the exact-forward C1 graph (LatencyEncoder, muted θ,
//! membrane k-WTA, forced winners) with disclosed rescue knobs: burst-richer
//! latency encode, winner-floor k-WTA, denser `p_sparse`, multi-pass, η=0.05.
//! Primary arm applies **true graded DFA** (fixed-random B × σ′ eligibility;
//! no STDP absorb). Contrast: hybrid STDP×DFA. Ceiling: surrogate gradient.
//!
//! Does not reopen protocol-v2 or frozen credit DFA hashes.

use std::collections::BTreeMap;

use binn_areas::{k_wta, Area};
use binn_core::Tick;
use binn_data::{Encoder, LatencyEncoder, Metrics, Sample};
use binn_engine::{CellId, Engine};
use binn_learn::{CreditSignal, FixedRandomFeedback, PostSynapticCredit, ThreeFactor};

use crate::dfa_spike_config::{
    DfaSpikeArm, DfaSpikeConfig, DFA_SPIKE_CHANCE_BASELINE, DFA_SPIKE_PROTOCOL_VERSION,
};
use crate::runner::{
    boost_readout_incoming, build_sparse_assembly, clear_eligibility, edge_index, freeze_trials,
    mean, mean_var, reset_c1_dynamic_state, run_positive_control, FrozenSplit, GateG2Verdict,
};
use crate::runner_match::gap_closed_matched;

pub const TRUE_DFA_SPIKE_REFERENCE: &str = "TRUE_GRADED_DFA_SPIKE_RESCUE";
pub const HYBRID_STDP_DFA_CONTRAST: &str = "HYBRID_STDP_DFA_SPIKE_CONTRAST";
pub const SURROGATE_GRADIENT_SPIKE_CEILING: &str = "SURROGATE_GRADIENT_SPIKE_CEILING";

#[derive(Clone, Debug, PartialEq)]
pub struct DfaSpikeArmSummary {
    pub arm: DfaSpikeArm,
    pub config_hash: String,
    pub protocol_version: u64,
    pub mean_accuracy: f32,
    pub variance_accuracy: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DfaSpikeSeedResult {
    pub seed: u64,
    pub true_dfa_accuracy: f32,
    pub hybrid_stdp_dfa_accuracy: f32,
    pub surrogate_gradient_accuracy: f32,
    pub gap_closed_dfa: f32,
    pub true_dfa_updates: u64,
    pub hybrid_stdp_dfa_updates: u64,
    pub surrogate_gradient_updates: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DfaSpikeReport {
    pub protocol_version: u64,
    pub seeds: Vec<DfaSpikeSeedResult>,
    pub summaries: Vec<DfaSpikeArmSummary>,
    pub mean_true_dfa: f32,
    pub mean_hybrid_stdp_dfa: f32,
    pub mean_surrogate_gradient: f32,
    pub mean_gap_closed_dfa: f32,
    pub variance_gap_closed_dfa: f32,
    pub gap_closed_dfa_lower_95: f32,
    pub positive_control_mean: f32,
    pub mean_activity_sparsity: f32,
    pub verdict: GateG2Verdict,
    pub pilot: bool,
}

#[derive(Default)]
pub struct DfaSpikeRunner;

impl DfaSpikeRunner {
    pub fn new() -> Self {
        Self
    }

    pub fn run(&mut self, config: &DfaSpikeConfig) -> DfaSpikeReport {
        assert!(config.base.n_seeds >= 1);
        assert!(config.matched_epochs >= 1);
        assert!(config.burst_count >= 1);
        assert!(
            (config.chance_baseline - DFA_SPIKE_CHANCE_BASELINE).abs() < 1e-6,
            "chance baseline locked at {DFA_SPIKE_CHANCE_BASELINE}"
        );

        let mut seeds = Vec::with_capacity(config.base.n_seeds);
        let mut positive_controls = Vec::with_capacity(config.base.n_seeds);
        let mut sparsities = Vec::with_capacity(config.base.n_seeds);

        for seed in config.seeds() {
            let split = freeze_trials(&config.base, seed);
            let true_out = run_arm(config, seed, &split, DfaSpikeArm::TrueDfa);
            let hybrid_out = run_arm(config, seed, &split, DfaSpikeArm::HybridStdpDfa);
            let grad_out = run_arm(config, seed, &split, DfaSpikeArm::SurrogateGradient);
            sparsities.push(true_out.activity_sparsity);
            let gap = gap_closed_matched(
                true_out.accuracy,
                grad_out.accuracy,
                config.chance_baseline,
                config.base.g2_min_reference_gap,
            );
            seeds.push(DfaSpikeSeedResult {
                seed,
                true_dfa_accuracy: true_out.accuracy,
                hybrid_stdp_dfa_accuracy: hybrid_out.accuracy,
                surrogate_gradient_accuracy: grad_out.accuracy,
                gap_closed_dfa: gap,
                true_dfa_updates: true_out.training_updates,
                hybrid_stdp_dfa_updates: hybrid_out.training_updates,
                surrogate_gradient_updates: grad_out.training_updates,
            });
            positive_controls.push(run_positive_control(&config.base, seed));
        }

        let summaries = DfaSpikeArm::ALL
            .into_iter()
            .map(|arm| summarize_arm(config, &seeds, arm))
            .collect();

        let true_accs: Vec<f32> = seeds.iter().map(|s| s.true_dfa_accuracy).collect();
        let hybrid_accs: Vec<f32> = seeds.iter().map(|s| s.hybrid_stdp_dfa_accuracy).collect();
        let grad_accs: Vec<f32> = seeds
            .iter()
            .map(|s| s.surrogate_gradient_accuracy)
            .collect();
        let gaps: Vec<f32> = seeds.iter().map(|s| s.gap_closed_dfa).collect();
        let (mean_true_dfa, _) = mean_var(&true_accs);
        let (mean_hybrid_stdp_dfa, _) = mean_var(&hybrid_accs);
        let (mean_surrogate_gradient, _) = mean_var(&grad_accs);
        let (mean_gap_closed_dfa, variance_gap_closed_dfa) = mean_var(&gaps);
        let n = gaps.len();
        let gap_closed_dfa_lower_95 = if n > 1 {
            mean_gap_closed_dfa
                - config.base.g2_confidence_z * (variance_gap_closed_dfa / n as f32).sqrt()
        } else {
            mean_gap_closed_dfa
        };

        let pilot = config.quick || config.base.n_seeds < config.scientific_n_seeds;
        let positive_control_mean = mean(&positive_controls);
        let mean_activity_sparsity = mean(&sparsities);
        let verdict = decide_verdict(
            config,
            mean_true_dfa,
            mean_surrogate_gradient,
            gap_closed_dfa_lower_95,
            positive_control_mean,
            mean_activity_sparsity,
            pilot,
        );

        DfaSpikeReport {
            protocol_version: DFA_SPIKE_PROTOCOL_VERSION,
            seeds,
            summaries,
            mean_true_dfa,
            mean_hybrid_stdp_dfa,
            mean_surrogate_gradient,
            mean_gap_closed_dfa,
            variance_gap_closed_dfa,
            gap_closed_dfa_lower_95,
            positive_control_mean,
            mean_activity_sparsity,
            verdict,
            pilot,
        }
    }

    pub fn render_markdown(report: &DfaSpikeReport, config: &DfaSpikeConfig) -> String {
        let mut md = String::new();
        md.push_str("# BINN spiking-path DFA rescue (`c1x-dfa-spike-*`)\n\n");
        md.push_str(
            "**Canonical C1 is immutable:** protocol-v2 hash `c1-118207fbc3eaba53` \
             and every G2 threshold remain unchanged. This is a **separate** protocol \
             family (`c1x-dfa-spike-*`) and does **not** reopen frozen \
             `c1x-dfa-exact-forward-*` / `c1x-iso-s-dfa-*` or dense-LIF `c1-dfa-*`.\n\n",
        );
        md.push_str("## Mechanism disclosure\n\n");
        md.push_str(&format!(
            "| arm | label | update |\n\
             |---|---|---|\n\
             | `{TRUE_DFA_SPIKE_REFERENCE}` | true-dfa (primary) | graded output error × \
             fixed-random DFA feedback × σ′(score) · pre; **no** STDP absorb |\n\
             | `{HYBRID_STDP_DFA_CONTRAST}` | hybrid-stdp-dfa | production STDP eligibility × \
             DFA-projected credit (frozen credit-DFA mechanism) |\n\
             | `{SURROGATE_GRADIENT_SPIKE_CEILING}` | surrogate-gradient | same-forward \
             straight-through ceiling for gap / harness |\n\n",
        ));
        md.push_str("## Substrate rescue knobs (disclosed)\n\n");
        md.push_str(&format!(
            "| knob | value | role |\n\
             |---|---|---|\n\
             | multi-pass (`matched_epochs`) | {} | exposure parity with BPTT schedule |\n\
             | richer encoder (`burst_count` × `burst_stride`) | {} × {} ticks | \
             repeated latency spikes into membrane k-WTA |\n\
             | calibrated k-WTA | winner-floor (all finite `v`) | reliable scores after reset |\n\
             | denser assembly (`p_sparse`) | {:.2} | more pathways under hard WTA |\n\
             | η / λ (DFA arms) | {:.2} / {:.2} | graded-DFA recipe (not production 0.35) |\n\
             | surrogate η (ceiling) | {:.2} | production-scale for harness validity |\n\
             | trial isolation | pairing clear + full membrane reset | no cross-trial residue |\n\n",
            config.matched_epochs,
            config.burst_count,
            config.burst_stride,
            config.base.p_sparse,
            config.base.eta,
            config.base.lambda,
            config.surrogate_eta,
        ));
        md.push_str(&format!(
            "- protocol version: {}\n\
             - schedule: **{}**\n\
             - seeds: {}\n\
             - positive control mean: {:.4}\n\
             - mean activity sparsity: {:.4} (band [{:.4}, {:.4}])\n\
             - true-dfa mean / gap LCB: {:.4} / {:.4}\n\
             - surrogate-gradient mean: {:.4}\n\
             - hybrid-stdp-dfa mean: {:.4}\n\
             - **verdict: {}**\n\n",
            report.protocol_version,
            if report.pilot { "PILOT" } else { "SCIENTIFIC" },
            config.base.n_seeds,
            report.positive_control_mean,
            report.mean_activity_sparsity,
            config.base.activity_sparsity_min,
            config.base.activity_sparsity_max,
            report.mean_true_dfa,
            report.gap_closed_dfa_lower_95,
            report.mean_surrogate_gradient,
            report.mean_hybrid_stdp_dfa,
            report.verdict.as_str(),
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
            "| seed | true-dfa | hybrid-stdp-dfa | surrogate-grad | gap_closed |\n\
             |---:|---:|---:|---:|---:|\n",
        );
        for seed in &report.seeds {
            md.push_str(&format!(
                "| {} | {:.4} | {:.4} | {:.4} | {:.4} |\n",
                seed.seed,
                seed.true_dfa_accuracy,
                seed.hybrid_stdp_dfa_accuracy,
                seed.surrogate_gradient_accuracy,
                seed.gap_closed_dfa
            ));
        }

        md.push_str("\n## Interpretation contract\n\n");
        md.push_str(
            "- True graded DFA tests whether the matched dense-LIF recipe can \
             express on LatencyEncoder + k-WTA after disclosed substrate knobs.\n\
             - Hybrid STDP×DFA is a labeled contrast to frozen `c1x-dfa-exact-forward-*`.\n\
             - Gap uses chance baseline `(dfa − 0.5)/(grad − 0.5)` with unchanged G2 bars.\n\
             - No outcome reopens canonical protocol-v2 G2 or mutates `c1-dfa-*`.\n",
        );
        md
    }
}

struct ArmOutcome {
    accuracy: f32,
    activity_sparsity: f32,
    training_updates: u64,
}

struct DfaSpikeGraph {
    engine: Engine,
    area: Area,
    encoder: LatencyEncoder,
    learner: ThreeFactor,
    feedback: FixedRandomFeedback,
    readout_0: CellId,
    readout_1: CellId,
    n_in: usize,
    n_hidden: usize,
    eta: f32,
    surrogate_eta: f32,
    burst_count: usize,
    burst_stride: Tick,
    kwta_all_finite: bool,
    t_cursor: Tick,
    stdp_absorb_used: bool,
}

impl DfaSpikeGraph {
    fn new(config: &DfaSpikeConfig, seed: u64) -> Self {
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
            feedback: FixedRandomFeedback::new(n_cells, 2, seed ^ 0xDFA0_5EED),
            readout_0,
            readout_1,
            n_in,
            n_hidden,
            eta: config.base.eta,
            surrogate_eta: config.surrogate_eta,
            burst_count: config.burst_count,
            burst_stride: config.burst_stride.max(1),
            kwta_all_finite: config.kwta_all_finite,
            t_cursor: 0,
            stdp_absorb_used: false,
        }
    }

    fn forward(&mut self, seq: &[Sample], label: u32) -> TrialTrace {
        let t0 = self.t_cursor;
        let frame_stride = self.encoder.max_delay().saturating_add(1).saturating_add(
            self.burst_stride
                .saturating_mul((self.burst_count.saturating_sub(1)) as Tick),
        );
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
                for burst_i in 0..self.burst_count {
                    input_counts[cell as usize] = input_counts[cell as usize].saturating_add(1);
                    let at = t0
                        + (frame_i as Tick)
                            .saturating_mul(frame_stride)
                            .saturating_add(event.t)
                            .saturating_add(self.burst_stride.saturating_mul(burst_i as Tick));
                    latest_input_at = latest_input_at.max(at);
                    self.engine.force_spike(cell, at);
                }
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
            .filter(|(_, score)| score.is_finite() && (self.kwta_all_finite || *score > 0.0))
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

    fn finish_trial(&mut self, trace: TrialTrace, arm: DfaSpikeArm, train: bool, beta: f32) -> u64 {
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
                DfaSpikeArm::TrueDfa => {
                    applications = self.apply_true_dfa(&trace, beta);
                }
                DfaSpikeArm::HybridStdpDfa => {
                    let signal = self.dfa_credit(&trace);
                    self.stdp_absorb_used = true;
                    applications = self
                        .learner
                        .update_with_credit_counted(&mut self.engine, &signal);
                }
                DfaSpikeArm::SurrogateGradient => {
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
            if train && arm == DfaSpikeArm::HybridStdpDfa {
                self.learner.observe_spikes(&mut self.engine);
                self.stdp_absorb_used = true;
            }
            clear_eligibility(&mut self.engine);
        }

        // Trial isolation: clear STDP pairing + full membrane (new hash family).
        reset_c1_dynamic_state(
            &mut self.engine,
            &trace.hidden_cells,
            &trace.saved_thresholds,
        );
        self.learner.reset_pairing_state();
        self.engine.close_inhibited_cycle();
        self.t_cursor = self.engine.time() + 20;
        applications
    }

    /// True graded DFA: fixed-random B × output error × σ′ · pre; no STDP.
    fn apply_true_dfa(&mut self, trace: &TrialTrace, beta: f32) -> u64 {
        let errors = output_error(trace);
        let dfa = self.feedback.project(&errors);
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
            let hidden_error = dfa.for_post(hidden);
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

        apply_weight_updates(&mut self.engine, &updates)
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
        let lr = self.surrogate_eta;

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

        apply_weight_updates(&mut self.engine, &updates)
    }

    fn dfa_credit(&self, trace: &TrialTrace) -> PostSynapticCredit {
        let errors = output_error(trace);
        let mut signal = self.feedback.project(&errors);
        signal.set(self.readout_0, errors[0]);
        signal.set(self.readout_1, errors[1]);
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

fn apply_weight_updates(engine: &mut Engine, updates: &[f32]) -> u64 {
    let mut changed = 0u64;
    for (edge, delta) in updates.iter().copied().enumerate() {
        if delta.abs() <= f32::EPSILON {
            continue;
        }
        let weight = (engine.edge_w[edge] + delta).clamp(-8.0, 8.0);
        engine.edge_w[edge] = weight;
        engine.syn.as_mut_slice()[edge].weight = weight;
        changed = changed.saturating_add(1);
    }
    changed
}

fn run_arm(
    config: &DfaSpikeConfig,
    seed: u64,
    split: &FrozenSplit,
    arm: DfaSpikeArm,
) -> ArmOutcome {
    let mut graph = DfaSpikeGraph::new(config, seed);
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
    let mut population = 0usize;
    for (sequence, label) in &split.test {
        let trace = graph.forward(sequence, *label);
        correct += usize::from(trace.prediction == *label);
        active += trace.winners.len();
        population += graph.n_hidden;
        let _ = graph.finish_trial(trace, arm, false, config.surrogate_beta);
    }
    ArmOutcome {
        accuracy: correct as f32 / split.test.len().max(1) as f32,
        activity_sparsity: Metrics::sparsity(active.min(population), population.max(1)),
        training_updates,
    }
}

fn summarize_arm(
    config: &DfaSpikeConfig,
    seeds: &[DfaSpikeSeedResult],
    arm: DfaSpikeArm,
) -> DfaSpikeArmSummary {
    let values: Vec<f32> = seeds
        .iter()
        .map(|seed| match arm {
            DfaSpikeArm::TrueDfa => seed.true_dfa_accuracy,
            DfaSpikeArm::HybridStdpDfa => seed.hybrid_stdp_dfa_accuracy,
            DfaSpikeArm::SurrogateGradient => seed.surrogate_gradient_accuracy,
        })
        .collect();
    let (mean_accuracy, variance_accuracy) = mean_var(&values);
    DfaSpikeArmSummary {
        arm,
        config_hash: config.hash_string_for_arm(arm),
        protocol_version: DFA_SPIKE_PROTOCOL_VERSION,
        mean_accuracy,
        variance_accuracy,
    }
}

fn decide_verdict(
    config: &DfaSpikeConfig,
    mean_dfa: f32,
    mean_gradient: f32,
    gap_lcb: f32,
    positive_control_mean: f32,
    mean_activity_sparsity: f32,
    pilot: bool,
) -> GateG2Verdict {
    let sparsity_ok = (config.base.activity_sparsity_min..=config.base.activity_sparsity_max)
        .contains(&mean_activity_sparsity);
    let positive_ok = positive_control_mean >= config.base.g2_min_positive_control;
    if !positive_ok || !sparsity_ok || mean_gradient < config.base.g2_min_accuracy {
        return GateG2Verdict::InvalidHarness;
    }
    if pilot {
        return GateG2Verdict::Pilot;
    }
    if gap_lcb > config.base.g2_min_gap_closed && mean_dfa >= config.base.g2_min_accuracy {
        GateG2Verdict::Pass
    } else {
        GateG2Verdict::Fail
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

    #[test]
    fn true_dfa_does_not_use_stdp_absorb() {
        let mut config = DfaSpikeConfig::quick();
        config.base.n_seeds = 1;
        config.matched_epochs = 1;
        let split = freeze_trials(&config.base, config.seeds()[0]);
        let mut graph = DfaSpikeGraph::new(&config, config.seeds()[0]);
        let (sequence, label) = &split.train[0];
        let trace = graph.forward(sequence, *label);
        let _ = graph.finish_trial(trace, DfaSpikeArm::TrueDfa, true, config.surrogate_beta);
        assert!(
            !graph.stdp_absorb_used,
            "true DFA must not call ThreeFactor STDP absorb"
        );
    }

    #[test]
    fn hybrid_contrast_uses_stdp_path() {
        let mut config = DfaSpikeConfig::quick();
        config.base.n_seeds = 1;
        config.matched_epochs = 1;
        let split = freeze_trials(&config.base, config.seeds()[0]);
        let mut graph = DfaSpikeGraph::new(&config, config.seeds()[0]);
        let (sequence, label) = &split.train[0];
        let trace = graph.forward(sequence, *label);
        let _ = graph.finish_trial(
            trace,
            DfaSpikeArm::HybridStdpDfa,
            true,
            config.surrogate_beta,
        );
        assert!(
            graph.stdp_absorb_used,
            "hybrid contrast must use STDP absorb"
        );
    }

    #[test]
    fn quick_run_finishes_and_discloses_rescue() {
        let mut config = DfaSpikeConfig::quick();
        config.base.n_seeds = 1;
        config.matched_epochs = 1;
        let mut runner = DfaSpikeRunner::new();
        let report = runner.run(&config);
        assert_eq!(report.seeds.len(), 1);
        assert!(report.pilot);
        let md = DfaSpikeRunner::render_markdown(&report, &config);
        assert!(md.contains("c1x-dfa-spike-*"));
        assert!(md.contains("burst_count"));
        assert!(md.contains("winner-floor"));
        assert!(md.contains("c1-118207fbc3eaba53"));
        assert!(!md.contains("c1x-dfa-exact-forward-4a1601e725edbc80"));
    }

    #[test]
    fn hashes_diverge_from_frozen_credit_dfa() {
        let cfg = DfaSpikeConfig::scientific();
        let hash = cfg.hash_string_for_arm(DfaSpikeArm::TrueDfa);
        assert!(hash.starts_with("c1x-dfa-spike-true-dfa-"));
        assert_ne!(hash, "c1x-dfa-exact-forward-4a1601e725edbc80");
        assert_ne!(hash, "c1x-iso-s-dfa-exact-forward-d2c8d3c929a68bd2");
    }
}
