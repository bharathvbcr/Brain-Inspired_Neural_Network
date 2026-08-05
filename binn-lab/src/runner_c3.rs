//! C3 / U15 v1 tabular proxy: terminal reward vs compositional depth.
//!
//! Opt-in only — the CLI refuses to run unless `--enable-c3` /
//! `--override-g2-for c3` (or `BINN_OVERRIDE_G2_FOR=c3`) is set.
//!
//! This runner does **not** instantiate the production event engine or
//! [`binn_learn::ThreeFactor`]. It is preserved as a tabular mechanism proxy.
//! Production-faithful claims belong to `runner_c3_v2`.

use binn_core::Rng;
use binn_data::{draw_example, true_transition, CreditDepthExample};

use crate::c3_config::C3Config;

/// Honest label for the disclosed teacher-forced oracle reference.
pub const C3_ORACLE_TEACHER_FORCED_REFERENCE: &str = "C3_V1_ORACLE_TEACHER_FORCED_REFERENCE";

/// Backward-compatible symbol; the value explicitly says oracle teacher forced.
pub const C3_GRADIENT_CREDIT_REFERENCE: &str = C3_ORACLE_TEACHER_FORCED_REFERENCE;

/// Per-depth aggregated accuracies.
#[derive(Clone, Debug, PartialEq)]
pub struct DepthResult {
    pub depth: usize,
    pub mean_accuracy_local: f32,
    pub mean_accuracy_gradient: f32,
    pub variance_local: f32,
    pub seed_accuracies_local: Vec<f32>,
    pub seed_accuracies_gradient: Vec<f32>,
}

/// Aggregated C3 report.
#[derive(Clone, Debug, PartialEq)]
pub struct C3Report {
    pub config_hash: String,
    pub protocol_version: u64,
    pub kill_gate_override: bool,
    pub baseline_label: &'static str,
    pub depth_results: Vec<DepthResult>,
    /// Max depth at which local mean accuracy ≥ floor (None if never).
    pub d_star: Option<usize>,
    /// Max depth at which gradient reference meets the same floor.
    pub d_star_gradient: Option<usize>,
    pub verdict: C3Verdict,
}

/// C3 reporting verdict (exploratory; requires kill-gate override).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum C3Verdict {
    /// PILOT / quick schedule — not a scientific D* claim.
    Pilot,
    /// Scientific schedule completed and D* measured (may be `None`).
    Measured,
}

impl C3Verdict {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pilot => "PILOT",
            Self::Measured => "MEASURED",
        }
    }
}

/// C3 experiment runner.
#[derive(Default)]
pub struct C3Runner;

impl C3Runner {
    pub fn new() -> Self {
        Self
    }

    /// Run the full C3 depth sweep for `config`.
    ///
    /// Panics if `kill_gate_override` is false — the CLI must set it after
    /// parsing `--enable-c3` / `--override-g2-for c3`.
    pub fn run_c3(&mut self, config: &C3Config) -> C3Report {
        assert!(
            config.kill_gate_override,
            "C3Runner::run_c3 requires kill_gate_override (CLI --enable-c3)"
        );
        assert!(config.min_depth >= 1);
        assert!(config.max_depth >= config.min_depth);

        let mut depth_results = Vec::new();
        for depth in config.min_depth..=config.max_depth {
            let mut seed_local = Vec::with_capacity(config.n_seeds);
            let mut seed_grad = Vec::with_capacity(config.n_seeds);
            for seed in config.seeds() {
                seed_local.push(run_local_at_depth(config, seed, depth));
                seed_grad.push(run_gradient_at_depth(config, seed, depth));
            }
            let mean_local = mean(&seed_local);
            let mean_grad = mean(&seed_grad);
            depth_results.push(DepthResult {
                depth,
                mean_accuracy_local: mean_local,
                mean_accuracy_gradient: mean_grad,
                variance_local: sample_variance(&seed_local, mean_local),
                seed_accuracies_local: seed_local,
                seed_accuracies_gradient: seed_grad,
            });
        }

        let d_star = depth_results
            .iter()
            .filter(|r| r.mean_accuracy_local >= config.accuracy_floor)
            .map(|r| r.depth)
            .max();
        let d_star_gradient = depth_results
            .iter()
            .filter(|r| r.mean_accuracy_gradient >= config.accuracy_floor)
            .map(|r| r.depth)
            .max();

        let verdict = if config.quick || config.n_seeds < config.scientific_n_seeds {
            C3Verdict::Pilot
        } else {
            C3Verdict::Measured
        };

        C3Report {
            config_hash: config.hash_string(),
            protocol_version: crate::c3_config::C3_PROTOCOL_VERSION,
            kill_gate_override: config.kill_gate_override,
            baseline_label: C3_ORACLE_TEACHER_FORCED_REFERENCE,
            depth_results,
            d_star,
            d_star_gradient,
            verdict,
        }
    }

    /// Render a results markdown note.
    pub fn render_results_markdown(report: &C3Report, config: &C3Config) -> String {
        let mut md = String::new();
        md.push_str("# C3 / U15 — credit assignment vs compositional depth\n\n");
        md.push_str(
            "**Kill-gate override:** this run is an **exploratory post-G2 branch**. \
             Gate G2 FAIL under `c1-118207fbc3eaba53` still stands. C3 does **not** \
             reopen the v8 kill-gate; it requires `--enable-c3` / `--override-g2-for c3`.\n\n",
        );
        md.push_str(&format!("- config hash: `{}`\n", report.config_hash));
        md.push_str(&format!(
            "- protocol version: {}\n",
            report.protocol_version
        ));
        md.push_str(&format!("- quick/PILOT: {}\n", config.quick));
        md.push_str(&format!("- seeds: {}\n", config.n_seeds));
        md.push_str(&format!(
            "- depth sweep: {}..= {}\n",
            config.min_depth, config.max_depth
        ));
        md.push_str(&format!(
            "- states / operations: {} / {}\n",
            config.n_states, config.n_operations
        ));
        md.push_str(&format!(
            "- train / test per depth×seed: {} / {}\n",
            config.n_train, config.n_test
        ));
        md.push_str(&format!(
            "- baseline: `{}` (lr={})\n",
            report.baseline_label, config.gradient_lr
        ));
        md.push_str(&format!(
            "- D* accuracy floor: {:.3}\n",
            config.accuracy_floor
        ));
        md.push_str(&format!(
            "- measured D* (local): **{}**\n",
            opt_depth(report.d_star)
        ));
        md.push_str(&format!(
            "- measured D* (gradient ref): **{}**\n",
            opt_depth(report.d_star_gradient)
        ));
        md.push_str(&format!("- verdict: **{}**\n\n", report.verdict.as_str()));
        if config.quick {
            md.push_str(
                "> PILOT only: the quick schedule validates the harness and cannot \
                 support a scientific depth claim.\n\n",
            );
        }
        md.push_str("## Accuracy versus depth\n\n");
        md.push_str(
            "| depth | local mean | local var | oracle mean | chance |\n\
             |---:|---:|---:|---:|---:|\n",
        );
        let chance = 1.0 / config.n_states.max(1) as f32;
        for r in &report.depth_results {
            md.push_str(&format!(
                "| {} | {:.4} | {:.6} | {:.4} | {:.4} |\n",
                r.depth, r.mean_accuracy_local, r.variance_local, r.mean_accuracy_gradient, chance
            ));
        }
        md.push_str(
            "\n## Protocol\n\n\
             Local path: each layer chooses a next state from locally stored \
             transition synapses. The only teaching signal is terminal `+1/-1` \
             reward; earlier layers receive exponentially decayed eligibility \
             (three-factor style). No target transport across layers.\n\n\
             Oracle reference (`C3_V1_ORACLE_TEACHER_FORCED_REFERENCE`): \
             disclosed teacher-forced updates with the true next-state at every \
             layer. This is a tabular oracle control, not a gradient run on the \
             production learner or event graph.\n\n",
        );
        md.push_str(
            "## Full scientific schedule\n\n\
             ```bash\n\
             cargo run -p binn-lab --release --bin c3 -- --enable-c3 \\\n\
               --out results/c3_credit_depth.md\n\
             ```\n",
        );
        md
    }
}

fn opt_depth(d: Option<usize>) -> String {
    d.map(|x| x.to_string()).unwrap_or_else(|| "none".into())
}

fn mean(values: &[f32]) -> f32 {
    if values.is_empty() {
        0.0
    } else {
        values.iter().sum::<f32>() / values.len() as f32
    }
}

fn sample_variance(values: &[f32], mean_value: f32) -> f32 {
    if values.len() < 2 {
        return 0.0;
    }
    values
        .iter()
        .map(|x| {
            let d = *x - mean_value;
            d * d
        })
        .sum::<f32>()
        / (values.len() - 1) as f32
}

fn run_local_at_depth(config: &C3Config, seed: u64, depth: usize) -> f32 {
    let mut train_rng = Rng::new(seed ^ (depth as u64).wrapping_mul(0xD1B5_4A32_D192_ED03));
    let mut learner = LocalCreditLearner::new(config, depth, seed ^ 0x51A7_EC3D_0000_0001);
    for _ in 0..config.n_train {
        let ex = draw_example(&mut train_rng, depth, config.n_states);
        learner.observe(&ex, &mut train_rng);
    }
    let mut test_rng = Rng::new(seed ^ (depth as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9));
    let correct = (0..config.n_test)
        .filter(|_| {
            let ex = draw_example(&mut test_rng, depth, config.n_states);
            learner.predict(&ex) == ex.target
        })
        .count();
    correct as f32 / config.n_test.max(1) as f32
}

fn run_gradient_at_depth(config: &C3Config, seed: u64, depth: usize) -> f32 {
    let mut train_rng = Rng::new(seed ^ (depth as u64).wrapping_mul(0xD1B5_4A32_D192_ED03));
    let mut learner = GradientCreditReference::new(config, depth, seed ^ 0xC3A0_D1E7_0000_0001);
    for _ in 0..config.n_train {
        let ex = draw_example(&mut train_rng, depth, config.n_states);
        learner.observe_supervised(&ex);
    }
    let mut test_rng = Rng::new(seed ^ (depth as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9));
    let correct = (0..config.n_test)
        .filter(|_| {
            let ex = draw_example(&mut test_rng, depth, config.n_states);
            learner.predict(&ex) == ex.target
        })
        .count();
    correct as f32 / config.n_test.max(1) as f32
}

/// Reward-modulated local transition learner (terminal reward only).
struct LocalCreditLearner {
    depth: usize,
    n_states: usize,
    n_operations: usize,
    weights: Vec<f32>,
    eta: f32,
    eligibility_decay: f32,
    exploration: f32,
}

impl LocalCreditLearner {
    fn new(config: &C3Config, depth: usize, seed: u64) -> Self {
        let n = depth * config.n_states * config.n_operations * config.n_states;
        let mut rng = Rng::new(seed ^ 0xC3C0_ED17);
        let weights = (0..n).map(|_| (rng.next_f32() - 0.5) * 0.02).collect();
        Self {
            depth,
            n_states: config.n_states,
            n_operations: config.n_operations,
            weights,
            eta: config.eta,
            eligibility_decay: config.eligibility_decay,
            exploration: config.exploration,
        }
    }

    fn edge_index(&self, layer: usize, state: usize, operation: usize, next: usize) -> usize {
        (((layer * self.n_states + state) * self.n_operations + operation) * self.n_states) + next
    }

    fn choose(
        &self,
        layer: usize,
        state: usize,
        operation: usize,
        rng: &mut Rng,
        explore: bool,
    ) -> usize {
        if explore && rng.next_f32() < self.exploration {
            return rng.gen_index(self.n_states);
        }
        (0..self.n_states)
            .max_by(|&a, &b| {
                let wa = self.weights[self.edge_index(layer, state, operation, a)];
                let wb = self.weights[self.edge_index(layer, state, operation, b)];
                wa.partial_cmp(&wb)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| b.cmp(&a))
            })
            .unwrap_or(0)
    }

    fn act(
        &self,
        example: &CreditDepthExample,
        rng: &mut Rng,
        explore: bool,
    ) -> (usize, Vec<(usize, usize, usize, usize)>) {
        debug_assert_eq!(example.operations.len(), self.depth);
        let mut state = example.start;
        let mut visited = Vec::with_capacity(self.depth);
        for (layer, &operation) in example.operations.iter().enumerate() {
            let next = self.choose(layer, state, operation, rng, explore);
            visited.push((layer, state, operation, next));
            state = next;
        }
        (state, visited)
    }

    fn observe(&mut self, example: &CreditDepthExample, rng: &mut Rng) -> bool {
        let (prediction, visited) = self.act(example, rng, true);
        let correct = prediction == example.target;
        let reward = if correct { 1.0 } else { -1.0 };
        for (layer, state, operation, chosen) in visited {
            let delay = self.depth - 1 - layer;
            let credit = self.eligibility_decay.powi(delay as i32);
            let delta = self.eta * reward * credit;
            let chosen_idx = self.edge_index(layer, state, operation, chosen);
            self.weights[chosen_idx] = (self.weights[chosen_idx] + delta).clamp(-8.0, 8.0);
            let anti = delta / (self.n_states.saturating_sub(1).max(1) as f32);
            for alternative in 0..self.n_states {
                if alternative != chosen {
                    let idx = self.edge_index(layer, state, operation, alternative);
                    self.weights[idx] = (self.weights[idx] - anti).clamp(-8.0, 8.0);
                }
            }
        }
        correct
    }

    fn predict(&self, example: &CreditDepthExample) -> usize {
        let mut deterministic_rng = Rng::new(0);
        self.act(example, &mut deterministic_rng, false).0
    }
}

/// Disclosed supervised reference: oracle next-state at every layer (GC1-exempt).
struct GradientCreditReference {
    n_states: usize,
    n_operations: usize,
    weights: Vec<f32>,
    lr: f32,
}

impl GradientCreditReference {
    fn new(config: &C3Config, depth: usize, seed: u64) -> Self {
        let n = depth * config.n_states * config.n_operations * config.n_states;
        let mut rng = Rng::new(seed ^ 0xC3_6AAD_0017);
        let weights = (0..n).map(|_| (rng.next_f32() - 0.5) * 0.02).collect();
        Self {
            n_states: config.n_states,
            n_operations: config.n_operations,
            weights,
            lr: config.gradient_lr,
        }
    }

    fn edge_index(&self, layer: usize, state: usize, operation: usize, next: usize) -> usize {
        (((layer * self.n_states + state) * self.n_operations + operation) * self.n_states) + next
    }

    fn observe_supervised(&mut self, example: &CreditDepthExample) {
        let mut state = example.start;
        for (layer, &operation) in example.operations.iter().enumerate() {
            let target_next = true_transition(state, operation, self.n_states);
            for next in 0..self.n_states {
                let idx = self.edge_index(layer, state, operation, next);
                let y = if next == target_next { 1.0 } else { 0.0 };
                // Softmax-free linear push toward the oracle next-state.
                let pred = self.weights[idx];
                self.weights[idx] = (pred + self.lr * (y - pred.clamp(0.0, 1.0))).clamp(-8.0, 8.0);
            }
            // Teacher-forced rollout for the gradient reference.
            state = target_next;
        }
    }

    fn predict(&self, example: &CreditDepthExample) -> usize {
        let mut state = example.start;
        for (layer, &operation) in example.operations.iter().enumerate() {
            state = (0..self.n_states)
                .max_by(|&a, &b| {
                    let wa = self.weights[self.edge_index(layer, state, operation, a)];
                    let wb = self.weights[self.edge_index(layer, state, operation, b)];
                    wa.partial_cmp(&wb)
                        .unwrap_or(std::cmp::Ordering::Equal)
                        .then_with(|| b.cmp(&a))
                })
                .unwrap_or(0);
        }
        state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn c3_quick_pilot_runs_with_override() {
        let mut cfg = C3Config::c3_quick();
        cfg.kill_gate_override = true;
        cfg.n_seeds = 1;
        cfg.n_train = 80;
        cfg.n_test = 40;
        cfg.max_depth = 2;
        let mut runner = C3Runner::new();
        let report = runner.run_c3(&cfg);
        assert_eq!(report.verdict, C3Verdict::Pilot);
        assert!(report.kill_gate_override);
        assert!(report.config_hash.starts_with("c3-"));
        assert_eq!(report.depth_results.len(), 2);
        assert_eq!(report.baseline_label, C3_GRADIENT_CREDIT_REFERENCE);
    }

    #[test]
    #[should_panic(expected = "kill_gate_override")]
    fn c3_refuses_without_override() {
        let cfg = C3Config::c3_quick();
        let mut runner = C3Runner::new();
        let _ = runner.run_c3(&cfg);
    }

    #[test]
    fn gradient_reference_beats_chance_at_shallow_depth() {
        let mut cfg = C3Config::c3_quick();
        cfg.kill_gate_override = true;
        cfg.n_seeds = 1;
        cfg.min_depth = 1;
        cfg.max_depth = 1;
        cfg.n_train = 400;
        cfg.n_test = 100;
        let mut runner = C3Runner::new();
        let report = runner.run_c3(&cfg);
        let r = &report.depth_results[0];
        let chance = 1.0 / cfg.n_states as f32;
        assert!(
            r.mean_accuracy_gradient > chance + 0.15,
            "grad={:.3} chance={:.3}",
            r.mean_accuracy_gradient,
            chance
        );
    }
}
