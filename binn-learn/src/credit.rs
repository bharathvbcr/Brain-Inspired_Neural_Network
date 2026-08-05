//! Credit-signal primitives for three-factor learning.
//!
//! The production rule remains synapse-local: every update is eligibility
//! multiplied by a signal selected from the postsynaptic cell.  The canonical
//! [`crate::Modulators`] path implements [`CreditSignal`] as a broadcast scalar,
//! while preregistered experiments can supply a deterministic per-neuron
//! vector without changing eligibility construction.
//!
//! The matched-arch protocol-v12 primary (`rl_reinforce_fb`) maps onto production
//! via [`ReinforceFeedback`] + [`reinforce_term`]: directional REINFORCE × frozen
//! per-neuron `B_i ∈ [-1,1]`, applied through
//! [`crate::ThreeFactor::update_with_credit_counted`]. Default C1 still uses
//! broadcast [`Modulators::reward`]; this module does not flip that path.

#![allow(clippy::needless_range_loop)]

use binn_core::Rng;
use binn_engine::CellId;

use crate::Modulators;

/// Directional REINFORCE term `r · (a − p)` (v12 / NumPy `rl_reinforce_fb`).
#[inline]
pub fn reinforce_term(reward: f32, action: f32, policy: f32) -> f32 {
    reward * (action - policy)
}

/// Learning signal consumed by a synapse according to its postsynaptic cell.
pub trait CreditSignal {
    /// Credit assigned to synapses whose postsynaptic endpoint is `post`.
    fn for_post(&self, post: CellId) -> f32;
}

impl CreditSignal for Modulators {
    #[inline]
    fn for_post(&self, _post: CellId) -> f32 {
        self.scalar()
    }
}

/// Explicit per-postsynaptic-cell learning signal.
#[derive(Clone, Debug, PartialEq)]
pub struct PostSynapticCredit {
    values: Vec<f32>,
}

impl PostSynapticCredit {
    /// Zero-initialized signal for `n_cells` postsynaptic cells.
    pub fn zeros(n_cells: usize) -> Self {
        Self {
            values: vec![0.0; n_cells],
        }
    }

    /// Construct from finite per-cell values.
    pub fn from_values(values: Vec<f32>) -> Self {
        assert!(
            values.iter().all(|x| x.is_finite()),
            "credit values must be finite"
        );
        Self { values }
    }

    /// Number of addressable postsynaptic cells.
    #[inline]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Whether the signal addresses no cells.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// Set one cell's credit.
    pub fn set(&mut self, post: CellId, value: f32) {
        assert!(value.is_finite(), "credit value must be finite");
        let slot = self
            .values
            .get_mut(post as usize)
            .expect("postsynaptic credit cell out of range");
        *slot = value;
    }

    /// Borrow the underlying vector for reporting and deterministic tests.
    #[inline]
    pub fn values(&self) -> &[f32] {
        &self.values
    }
}

impl CreditSignal for PostSynapticCredit {
    #[inline]
    fn for_post(&self, post: CellId) -> f32 {
        self.values.get(post as usize).copied().unwrap_or(0.0)
    }
}

/// Causal running-mean reward baseline.
///
/// The advantage is computed against the mean of *previous* rewards.  Only
/// after returning the advantage is the current reward incorporated, which
/// prevents target/test leakage and fixes the update ordering in the protocol.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RunningMeanBaseline {
    count: u64,
    mean: f32,
}

impl RunningMeanBaseline {
    /// Empty baseline (`count=0`, prediction `0`).
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Current prediction from past rewards only.
    #[inline]
    pub fn prediction(self) -> f32 {
        self.mean
    }

    /// Number of rewards incorporated so far.
    #[inline]
    pub fn count(self) -> u64 {
        self.count
    }

    /// Return `reward - previous_mean`, then incorporate `reward`.
    pub fn advantage_and_observe(&mut self, reward: f32) -> f32 {
        assert!(reward.is_finite(), "reward must be finite");
        let advantage = reward - self.mean;
        self.count = self.count.saturating_add(1);
        self.mean += (reward - self.mean) / self.count as f32;
        advantage
    }
}

/// Fixed random direct-feedback projection used by the DFA arm.
///
/// Rows address postsynaptic cells; columns address output-error components.
/// The matrix is sampled once from deterministic ±1 entries scaled by
/// `1/sqrt(n_outputs)` and is never mutated afterward.
#[derive(Clone, Debug, PartialEq)]
pub struct FixedRandomFeedback {
    n_posts: usize,
    n_outputs: usize,
    weights: Vec<f32>,
}

impl FixedRandomFeedback {
    /// Deterministic fixed feedback matrix.
    pub fn new(n_posts: usize, n_outputs: usize, seed: u64) -> Self {
        assert!(n_posts > 0, "DFA requires at least one postsynaptic cell");
        assert!(n_outputs > 0, "DFA requires at least one output");
        let mut rng = Rng::new(seed ^ 0x00DF_A0C1_ED17_u64);
        let scale = 1.0 / (n_outputs as f32).sqrt();
        let weights = (0..n_posts * n_outputs)
            .map(|_| if rng.next_f32() < 0.5 { -scale } else { scale })
            .collect();
        Self {
            n_posts,
            n_outputs,
            weights,
        }
    }

    /// Project one output-error vector into per-postsynaptic credit.
    pub fn project(&self, output_error: &[f32]) -> PostSynapticCredit {
        assert_eq!(
            output_error.len(),
            self.n_outputs,
            "DFA output-error width mismatch"
        );
        assert!(
            output_error.iter().all(|x| x.is_finite()),
            "DFA output errors must be finite"
        );
        let mut values = vec![0.0f32; self.n_posts];
        for (post, value) in values.iter_mut().enumerate() {
            let row = &self.weights[post * self.n_outputs..(post + 1) * self.n_outputs];
            *value = row
                .iter()
                .zip(output_error.iter())
                .map(|(w, e)| w * e)
                .sum();
        }
        PostSynapticCredit::from_values(values)
    }

    /// Immutable feedback weights; exposed for preregistration verification.
    #[inline]
    pub fn weights(&self) -> &[f32] {
        &self.weights
    }
}

/// Frozen per-neuron feedback for the production directional RL neuromodulator.
///
/// Matches matched-arch protocol v12 `rl_reinforce_fb`: each postsynaptic cell
/// gets a fixed `B_i ∼ Uniform[-1, 1]` drawn once at construction. Distinct from
/// supervised DFA [`FixedRandomFeedback`] (matrix with ±1/√n_out entries).
///
/// Production path: `ThreeFactor::update_with_credit_counted(engine, &fb.credit(reinforce_term(...)))`.
#[derive(Clone, Debug, PartialEq)]
pub struct ReinforceFeedback {
    weights: Vec<f32>,
}

impl ReinforceFeedback {
    /// Seed mix shared with [`crate::MatchedRlReinforceFb`] (stable lineage).
    pub const SEED_MIX: u64 = 0x00FB_A0C1_ED17;

    /// Draw frozen `B_i ∈ [-1, 1]` for `n_posts` postsynaptic cells.
    pub fn new(n_posts: usize, seed: u64) -> Self {
        assert!(
            n_posts > 0,
            "reinforce feedback requires at least one postsynaptic cell"
        );
        let mut rng = Rng::new(seed ^ Self::SEED_MIX);
        let weights = (0..n_posts).map(|_| rng.next_f32() * 2.0 - 1.0).collect();
        Self { weights }
    }

    /// Wrap already-drawn finite feedback weights (tests / replay).
    pub fn from_weights(weights: Vec<f32>) -> Self {
        assert!(
            !weights.is_empty(),
            "reinforce feedback weights must be non-empty"
        );
        assert!(
            weights.iter().all(|x| x.is_finite()),
            "reinforce feedback weights must be finite"
        );
        Self { weights }
    }

    /// Per-postsynaptic credit `B_i · directional` (typically [`reinforce_term`]).
    pub fn credit(&self, directional: f32) -> PostSynapticCredit {
        assert!(directional.is_finite(), "directional term must be finite");
        PostSynapticCredit::from_values(self.weights.iter().map(|b| b * directional).collect())
    }

    /// Immutable feedback weights (preregistration / determinism).
    #[inline]
    pub fn weights(&self) -> &[f32] {
        &self.weights
    }

    /// Number of addressable postsynaptic cells.
    #[inline]
    pub fn len(&self) -> usize {
        self.weights.len()
    }

    /// Whether no posts are addressed.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.weights.is_empty()
    }
}

/// Online learned linear reward prediction critic $V(s) = \mathbf{w}_v^\top \mathbf{a} + b_v$.
///
/// Computes continuous reward prediction error $\delta = r - V(s)$ for broadcast modulation
/// and updates weights via delta rule: $\Delta \mathbf{w}_v = \eta_v \cdot \delta \cdot \mathbf{a}$.
#[derive(Clone, Debug, PartialEq)]
pub struct LearnedRpeCritic {
    pub weights: Vec<f32>,
    pub bias: f32,
    pub lr: f32,
}

impl LearnedRpeCritic {
    pub fn new(n_features: usize, lr: f32) -> Self {
        assert!(lr > 0.0, "critic learning rate must be positive");
        Self {
            weights: vec![0.0; n_features],
            bias: 0.0,
            lr,
        }
    }

    pub fn predict(&self, features: &[f32]) -> f32 {
        assert_eq!(
            features.len(),
            self.weights.len(),
            "critic feature dimension mismatch"
        );
        let dot: f32 = self
            .weights
            .iter()
            .zip(features.iter())
            .map(|(w, x)| w * x)
            .sum();
        dot + self.bias
    }

    pub fn rpe_and_update(&mut self, reward: f32, features: &[f32]) -> f32 {
        assert!(reward.is_finite(), "reward must be finite");
        let pred = self.predict(features);
        let delta = reward - pred;
        for (w, &x) in self.weights.iter_mut().zip(features.iter()) {
            *w += self.lr * delta * x;
        }
        self.bias += self.lr * delta;
        delta
    }
}

/// Mutable per-neuron feedback vector $B_i \in [-1, 1]$ with online alignment learning.
///
/// $B_i \leftarrow \text{clamp}(B_i + \eta_B \cdot r \cdot (a_i - p_i) \cdot x_i, -1, 1)$.
#[derive(Clone, Debug, PartialEq)]
pub struct LearnedReinforceFeedback {
    weights: Vec<f32>,
    pub lr_b: f32,
}

impl LearnedReinforceFeedback {
    pub fn new(n_posts: usize, seed: u64, lr_b: f32) -> Self {
        let base = ReinforceFeedback::new(n_posts, seed);
        Self {
            weights: base.weights().to_vec(),
            lr_b,
        }
    }

    pub fn credit(&self, directional: f32) -> PostSynapticCredit {
        assert!(directional.is_finite(), "directional term must be finite");
        PostSynapticCredit::from_values(self.weights.iter().map(|b| b * directional).collect())
    }

    pub fn update(&mut self, directional: f32, post_activities: &[f32]) {
        assert_eq!(post_activities.len(), self.weights.len());
        for (b, &act) in self.weights.iter_mut().zip(post_activities.iter()) {
            let db = self.lr_b * directional * act;
            *b = (*b + db).clamp(-1.0, 1.0);
        }
    }

    pub fn weights(&self) -> &[f32] {
        &self.weights
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn broadcast_modulators_preserve_legacy_scalar() {
        let m = Modulators::new(0.5, 0.25, 2.0);
        assert!((m.for_post(0) - 1.5).abs() < 1e-6);
        assert_eq!(m.for_post(999), m.for_post(0));
    }

    #[test]
    fn postsynaptic_credit_routes_by_cell() {
        let mut signal = PostSynapticCredit::zeros(4);
        signal.set(1, -0.25);
        signal.set(3, 0.75);
        assert_eq!(signal.for_post(0), 0.0);
        assert_eq!(signal.for_post(1), -0.25);
        assert_eq!(signal.for_post(3), 0.75);
    }

    #[test]
    fn reward_baseline_is_causal_and_updates_after_advantage() {
        let mut b = RunningMeanBaseline::new();
        assert_eq!(b.advantage_and_observe(1.0), 1.0);
        assert_eq!(b.prediction(), 1.0);
        assert_eq!(b.advantage_and_observe(-1.0), -2.0);
        assert_eq!(b.prediction(), 0.0);
        assert_eq!(b.count(), 2);
    }

    #[test]
    fn fixed_feedback_is_deterministic_and_seeded() {
        let a = FixedRandomFeedback::new(8, 2, 7);
        let b = FixedRandomFeedback::new(8, 2, 7);
        let c = FixedRandomFeedback::new(8, 2, 8);
        assert_eq!(a, b);
        assert_ne!(a.weights(), c.weights());
        assert_eq!(a.project(&[-0.5, 0.5]), b.project(&[-0.5, 0.5]));
    }

    #[test]
    fn reinforce_term_is_reward_times_advantage() {
        assert!((reinforce_term(1.0, 1.0, 0.25) - 0.75).abs() < 1e-6);
        assert!((reinforce_term(-1.0, 0.0, 0.8) - 0.8).abs() < 1e-6);
    }

    #[test]
    fn reinforce_feedback_is_frozen_uniform_and_seeded() {
        let a = ReinforceFeedback::new(16, 77);
        let b = ReinforceFeedback::new(16, 77);
        let c = ReinforceFeedback::new(16, 78);
        assert_eq!(a, b);
        assert_ne!(a.weights(), c.weights());
        assert!(a.weights().iter().all(|&w| (-1.0..=1.0).contains(&w)));
        let directional = reinforce_term(1.0, 1.0, 0.3);
        let credit = a.credit(directional);
        assert_eq!(credit.len(), 16);
        for (i, &b_i) in a.weights().iter().enumerate() {
            assert!((credit.for_post(i as CellId) - b_i * directional).abs() < 1e-6);
        }
    }

    #[test]
    fn learned_reinforce_feedback_updates_weights() {
        let mut fb = LearnedReinforceFeedback::new(4, 42, 0.1);
        let w0 = fb.weights().to_vec();
        let post_act = vec![1.0, 0.0, 1.0, 0.5];
        fb.update(0.5, &post_act);
        let w1 = fb.weights().to_vec();
        assert_ne!(w0[0], w1[0]);
        assert_eq!(w0[1], w1[1]); // zero activity cell unaffected
    }

    #[test]
    fn learned_rpe_critic_reduces_error() {
        let mut critic = LearnedRpeCritic::new(2, 0.1);
        let features = vec![1.0, 0.5];
        let rpe1 = critic.rpe_and_update(1.0, &features);
        let rpe2 = critic.rpe_and_update(1.0, &features);
        assert!(
            rpe2.abs() < rpe1.abs(),
            "RPE should decrease as critic learns target"
        );
    }
}

/// Multi-Channel Neuromodulator (Suite 2).
///
/// Combines 3 biologically distinct neuromodulatory pathways:
/// - **Dopamine (DA)**: Global RPE / reward prediction error `r - V(s)`.
/// - **Acetylcholine (ACh)**: Somatic membrane proximity / salience `1 / (1 + β|v - θ|)^2`.
/// - **Noradrenaline (NE)**: Novelty / surprise multiplier `|r - V(s)|`.
///
/// All three channels preserve the sign of the directional dopamine term.
/// ACh gates that term by membrane proximity and NE amplifies it by surprise;
/// neither channel injects an unsigned positive update.
#[derive(Clone, Debug)]
pub struct MultiChannelNeuromodulator {
    pub da_weight: f32,
    pub ach_weight: f32,
    pub ne_weight: f32,
    pub feedback: LearnedReinforceFeedback,
}

/// Individually observable channel contributions before their configured
/// weights are applied.
///
/// Keeping the channels inspectable makes mechanism tests exact: callers do
/// not need to estimate an ACh term by subtracting two mixed total signals.
#[derive(Clone, Debug, PartialEq)]
pub struct MultiChannelComponents {
    /// Directional per-cell dopamine term `B_i · rpe`.
    pub dopamine: PostSynapticCredit,
    /// Dopamine gated by somatic threshold proximity.
    pub acetylcholine: PostSynapticCredit,
    /// Dopamine amplified by unsigned surprise `|rpe|`.
    pub noradrenaline: PostSynapticCredit,
}

impl MultiChannelNeuromodulator {
    pub fn new(n_cells: usize, seed: u64, eta_b: f32) -> Self {
        Self {
            da_weight: 1.0,
            ach_weight: 0.5,
            ne_weight: 0.3,
            feedback: LearnedReinforceFeedback::new(n_cells, seed, eta_b),
        }
    }

    /// Compute the three sign-preserving channel components.
    pub fn components(
        &self,
        rpe: f32,
        v_soma: &[f32],
        theta: f32,
        beta: f32,
    ) -> MultiChannelComponents {
        assert!(rpe.is_finite(), "RPE must be finite");
        assert!(theta.is_finite(), "threshold must be finite");
        assert!(
            beta.is_finite() && beta > 0.0,
            "surrogate beta must be finite and positive"
        );
        assert_eq!(
            v_soma.len(),
            self.feedback.weights().len(),
            "somatic-voltage width must match feedback width"
        );

        let n = v_soma.len();
        let dopamine = self.feedback.credit(rpe);
        let mut acetylcholine = PostSynapticCredit::zeros(n);
        let mut noradrenaline = PostSynapticCredit::zeros(n);

        for i in 0..n {
            let ach = 1.0 / (1.0 + beta * (v_soma[i] - theta).abs()).powi(2);
            let da_i = dopamine.for_post(i as u32);
            acetylcholine.set(i as u32, ach * da_i);
            noradrenaline.set(i as u32, rpe.abs() * da_i);
        }

        MultiChannelComponents {
            dopamine,
            acetylcholine,
            noradrenaline,
        }
    }

    pub fn compute_signal(
        &self,
        _reward: f32,
        rpe: f32,
        v_soma: &[f32],
        theta: f32,
        beta: f32,
    ) -> PostSynapticCredit {
        let components = self.components(rpe, v_soma, theta, beta);
        let mut credit = PostSynapticCredit::zeros(v_soma.len());
        for i in 0..v_soma.len() {
            let total_signal = self.da_weight * components.dopamine.for_post(i as u32)
                + self.ach_weight * components.acetylcholine.for_post(i as u32)
                + self.ne_weight * components.noradrenaline.for_post(i as u32);
            credit.set(i as u32, total_signal);
        }

        credit
    }
}

/// Scales per-neuron credit by proximity to the k-WTA decision boundary.
///
/// Neurons near the boundary (where v ≈ v_{k+1}) get full credit strength;
/// neurons far from the boundary get attenuated credit. This focuses
/// plasticity on the synapses that could flip a winner selection.
///
/// Margin weight: `φ(v_i) = exp(−(v_i − v_boundary)² / (2σ²))`
#[derive(Clone, Debug)]
pub struct MarginScaledCredit<S> {
    inner: S,
    margin_weights: Vec<f32>,
}

impl<S> MarginScaledCredit<S> {
    pub fn new(inner: S, n_cells: usize) -> Self {
        Self {
            inner,
            margin_weights: vec![1.0; n_cells],
        }
    }

    pub fn update_margins(&mut self, membranes: &[f32], v_boundary: f32, sigma: f32) {
        assert_eq!(membranes.len(), self.margin_weights.len());
        if !v_boundary.is_finite() || sigma <= 0.0 {
            self.margin_weights.fill(1.0);
            return;
        }
        let denom = 2.0 * sigma * sigma;
        for (w, &v) in self.margin_weights.iter_mut().zip(membranes.iter()) {
            let diff = v - v_boundary;
            *w = (-(diff * diff) / denom).exp();
        }
    }

    pub fn inner(&self) -> &S {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut S {
        &mut self.inner
    }
}

impl<S: CreditSignal> CreditSignal for MarginScaledCredit<S> {
    fn for_post(&self, post: CellId) -> f32 {
        let base_credit = self.inner.for_post(post);
        let weight = self
            .margin_weights
            .get(post as usize)
            .copied()
            .unwrap_or(1.0);
        base_credit * weight
    }
}

#[cfg(test)]
mod tests_extra {
    use super::*;

    #[test]
    fn multi_channel_neuromodulator_computes_combined_signal() {
        let mod_system = MultiChannelNeuromodulator::new(4, 123, 0.01);
        let v_soma = vec![0.0, 0.5, 1.0, -0.5];
        let sig = mod_system.compute_signal(1.0, 0.5, &v_soma, 1.0, 5.0);
        assert_eq!(sig.len(), 4);
    }

    #[test]
    fn multi_channel_signal_reverses_with_rpe_sign() {
        let mod_system = MultiChannelNeuromodulator::new(8, 123, 0.01);
        let v_soma = vec![0.0, 0.5, 1.0, -0.5, 0.8, 1.2, 1.5, 2.0];
        let positive = mod_system.compute_signal(1.0, 0.8, &v_soma, 1.0, 5.0);
        let negative = mod_system.compute_signal(1.0, -0.8, &v_soma, 1.0, 5.0);
        for (&pos, &neg) in positive.values().iter().zip(negative.values()) {
            assert!(
                (pos + neg).abs() < 1e-6,
                "RPE sign must reverse credit exactly: {pos} vs {neg}"
            );
        }
    }

    #[test]
    fn acetylcholine_component_is_stronger_near_threshold() {
        let mod_system = MultiChannelNeuromodulator::new(4, 123, 0.01);
        let near = mod_system.components(0.8, &[1.0; 4], 1.0, 5.0);
        let far = mod_system.components(0.8, &[2.0; 4], 1.0, 5.0);
        for (&near_i, &far_i) in near
            .acetylcholine
            .values()
            .iter()
            .zip(far.acetylcholine.values())
        {
            assert!(
                near_i.abs() > far_i.abs(),
                "ACh gate must concentrate directional credit near threshold"
            );
        }
    }

    #[test]
    fn margin_scaled_credit_modulates_near_boundary() {
        let base = PostSynapticCredit::from_values(vec![1.0, 1.0, 1.0]);
        let mut scaled = MarginScaledCredit::new(base, 3);
        let v_soma = [1.0, 2.0, 5.0];
        // boundary = 2.0, sigma = 1.0
        scaled.update_margins(&v_soma, 2.0, 1.0);

        let c1 = scaled.for_post(1);
        let c0 = scaled.for_post(0);
        let c2 = scaled.for_post(2);

        assert!((c1 - 1.0).abs() < 1e-6); // diff=0 -> exp(0)=1
        assert!((c0 - (-0.5f32).exp()).abs() < 1e-6); // diff=1 -> exp(-1/2)
        assert!(c2 < c0); // diff=3 -> exp(-9/2)
    }
}
