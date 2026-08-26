//! Labeled **matched-architecture RL** reference (U-MATCH-RL / protocol v12).
//!
//! **GC1 exempt** (this is a `*_baseline.rs` file). Do not call from production
//! learning paths. Production code must use the online three-factor rule in
//! `three_factor`. This module ports the NumPy **in-family reward** recipes from
//! `results/MATCHED_ARCH_DEEP_FINDINGS.md` — `rl_graded` and `rl_reinforce_fb` —
//! onto the identical dense-LIF feed-forward used by
//! [`crate::matched_dfa_baseline`].
//!
//! **MUST NEVER BE THE PRODUCTION LEARNER** (v7 / v8 rule).
//!
//! ## Contrast
//!
//! Held identical with protocol-v5 DFA matched-arch: forward graph (`wrec=0`),
//! width, encoding, epochs, splits, seed lineage knobs, LIF constants, η=0.05,
//! λ=0, minibatch=20.
//!
//! - [`MatchedRlFlat`] — production-faithful ±1 reward broadcast (NumPy `rl_flat`).
//! - [`MatchedRlGraded`] — graded correctness − running baseline, broadcast
//!   (NumPy `rl_graded`). v11 primary (FAIL); v12 contrast — not retuned.
//! - [`MatchedRlReinforceFb`] — REINFORCE `(r·(a−p))` × frozen random feedback
//!   (NumPy `rl_reinforce_fb`). **v12 primary** gated arm.
//!
//! Readout always uses the REINFORCE term `r·(a−p)` (policy gradient on the
//! Bernoulli readout). Only the **hidden** modulator differs across arms.
//!
//! The gradient ceiling remains [`crate::matched_local_baseline::MatchedGradient`].

#![allow(clippy::needless_range_loop)]

use crate::credit::{
    reinforce_term, CreditSignal, LearnedReinforceFeedback, LearnedRpeCritic, ReinforceFeedback,
};
use crate::matched_local_baseline::{
    ForwardCache, GradientExample, GradientReferenceReport, MatchedArch, MatchedForward,
};
use crate::REFERENCE_SEQUENCE_LEN;
use binn_core::Rng;
use binn_engine::{DEFAULT_TAU_M, THETA_REST, V_RESET};

const N_IN: usize = 2;
const T: usize = REFERENCE_SEQUENCE_LEN;
/// Minibatch size matching `scripts/matched_arch_deep.py`.
const DEFAULT_BATCH: usize = 20;
/// EMA rate for the graded-correctness baseline (NumPy `0.9*base+0.1*mean`).
const GRADED_BASELINE_EMA: f32 = 0.1;

/// Stable label for the flat ±1 reward broadcast arm.
pub const MATCHED_RL_FLAT_LABEL: &str = "MATCHED_ARCH_RL_FLAT";
/// Stable label for the graded-reward broadcast arm (v11 primary / v12 contrast).
pub const MATCHED_RL_GRADED_LABEL: &str = "MATCHED_ARCH_RL_GRADED";
/// Stable label for REINFORCE × fixed-random feedback (v12 primary).
pub const MATCHED_RL_REINFORCE_FB_LABEL: &str = "MATCHED_ARCH_RL_REINFORCE_FB";
/// Stable label for RPE critic broadcast arm.
pub const MATCHED_RL_RPE_LABEL: &str = "MATCHED_ARCH_RL_RPE";
/// Stable label for online learned feedback alignment arm.
pub const MATCHED_RL_LEARNED_FB_LABEL: &str = "MATCHED_ARCH_RL_LEARNED_FB";

pub use crate::matched_local_baseline::DEFAULT_MATCHED_BETA as DEFAULT_RL_MATCHED_BETA;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RlHiddenRule {
    Flat,
    Graded,
    ReinforceFb,
}

/// Production-faithful ±1 reward broadcast on the matched feed-forward.
#[derive(Clone, Debug)]
pub struct MatchedRlFlat {
    pub(crate) arch: MatchedArch,
    eta: f32,
    lambda: f32,
    rng: Rng,
}

impl MatchedRlFlat {
    /// New flat-reward arm sized to `hidden`.
    pub fn new(hidden: usize, eta: f32, lambda: f32, beta: f32, seed: u64) -> Self {
        // Feed-forward is this arm's historical graph, preserved so no
        // archived number moves. Name the graph through `on` instead.
        Self::on(MatchedForward::FeedForward, hidden, eta, lambda, beta, seed)
    }

    /// This arm on an explicitly named graph. See [`MatchedForward`].
    pub fn on(
        forward: MatchedForward,
        hidden: usize,
        eta: f32,
        lambda: f32,
        beta: f32,
        seed: u64,
    ) -> Self {
        Self {
            arch: MatchedArch::on(forward, hidden, beta, seed),
            eta,
            lambda,
            rng: Rng::new(seed ^ 0xF1A7_7012_0000_00F1),
        }
    }

    /// Train with ±1 broadcast reward and evaluate.
    pub fn train_and_evaluate(
        &mut self,
        epochs: usize,
        train: &[GradientExample],
        test: &[GradientExample],
    ) -> GradientReferenceReport {
        assert!(!train.is_empty(), "matched RL-flat needs training data");
        assert!(!test.is_empty(), "matched RL-flat needs test data");
        train_minibatches(
            &mut self.arch,
            &mut self.rng,
            epochs,
            train,
            self.eta,
            self.lambda,
            RlHiddenRule::Flat,
            None,
        );
        let (accuracy, loss) = self.arch.evaluate(test);
        GradientReferenceReport {
            label: MATCHED_RL_FLAT_LABEL,
            accuracy,
            loss,
        }
    }
}

/// Graded correctness − baseline, broadcast to all hidden units.
#[derive(Clone, Debug)]
pub struct MatchedRlGraded {
    pub(crate) arch: MatchedArch,
    eta: f32,
    lambda: f32,
    rng: Rng,
}

impl MatchedRlGraded {
    /// New graded-reward arm sized to `hidden`.
    pub fn new(hidden: usize, eta: f32, lambda: f32, beta: f32, seed: u64) -> Self {
        // Feed-forward is this arm's historical graph, preserved so no
        // archived number moves. Name the graph through `on` instead.
        Self::on(MatchedForward::FeedForward, hidden, eta, lambda, beta, seed)
    }

    /// This arm on an explicitly named graph. See [`MatchedForward`].
    pub fn on(
        forward: MatchedForward,
        hidden: usize,
        eta: f32,
        lambda: f32,
        beta: f32,
        seed: u64,
    ) -> Self {
        Self {
            arch: MatchedArch::on(forward, hidden, beta, seed),
            eta,
            lambda,
            rng: Rng::new(seed ^ 0x61AD_7012_0000_00F1),
        }
    }

    /// Train with graded broadcast reward and evaluate.
    pub fn train_and_evaluate(
        &mut self,
        epochs: usize,
        train: &[GradientExample],
        test: &[GradientExample],
    ) -> GradientReferenceReport {
        assert!(!train.is_empty(), "matched RL-graded needs training data");
        assert!(!test.is_empty(), "matched RL-graded needs test data");
        train_minibatches(
            &mut self.arch,
            &mut self.rng,
            epochs,
            train,
            self.eta,
            self.lambda,
            RlHiddenRule::Graded,
            None,
        );
        let (accuracy, loss) = self.arch.evaluate(test);
        GradientReferenceReport {
            label: MATCHED_RL_GRADED_LABEL,
            accuracy,
            loss,
        }
    }
}

/// REINFORCE `(r·(a−p))` × frozen random feedback matrix `B_i`.
#[derive(Clone, Debug)]
pub struct MatchedRlReinforceFb {
    pub(crate) arch: MatchedArch,
    feedback: Vec<f32>,
    eta: f32,
    lambda: f32,
    rng: Rng,
}

impl MatchedRlReinforceFb {
    /// New reinforce-fb arm sized to `hidden`.
    pub fn new(hidden: usize, eta: f32, lambda: f32, beta: f32, seed: u64) -> Self {
        // Feed-forward is this arm's historical graph, preserved so no
        // archived number moves. Name the graph through `on` instead.
        Self::on(MatchedForward::FeedForward, hidden, eta, lambda, beta, seed)
    }

    /// This arm on an explicitly named graph. See [`MatchedForward`].
    pub fn on(
        forward: MatchedForward,
        hidden: usize,
        eta: f32,
        lambda: f32,
        beta: f32,
        seed: u64,
    ) -> Self {
        let fb = ReinforceFeedback::new(hidden, seed);
        Self {
            arch: MatchedArch::on(forward, hidden, beta, seed),
            feedback: fb.weights().to_vec(),
            eta,
            lambda,
            rng: Rng::new(seed ^ 0x00FB_A0C1_ED17_u64),
        }
    }

    /// Train with REINFORCE × frozen feedback matrix and evaluate.
    pub fn train_and_evaluate(
        &mut self,
        epochs: usize,
        train: &[GradientExample],
        test: &[GradientExample],
    ) -> GradientReferenceReport {
        assert!(
            !train.is_empty(),
            "matched RL-reinforce-fb needs training data"
        );
        assert!(!test.is_empty(), "matched RL-reinforce-fb needs test data");
        let fb = self.feedback.clone();
        train_minibatches(
            &mut self.arch,
            &mut self.rng,
            epochs,
            train,
            self.eta,
            self.lambda,
            RlHiddenRule::ReinforceFb,
            Some(fb.as_slice()),
        );
        let (accuracy, loss) = self.arch.evaluate(test);
        GradientReferenceReport {
            label: MATCHED_RL_REINFORCE_FB_LABEL,
            accuracy,
            loss,
        }
    }

    /// Immutable feedback weights (preregistration / determinism checks).
    pub fn feedback_weights(&self) -> &[f32] {
        &self.feedback
    }
}

/// Continuous RPE critic broadcast arm on matched feed-forward.
#[derive(Clone, Debug)]
pub struct MatchedRlRpe {
    pub(crate) arch: MatchedArch,
    critic: LearnedRpeCritic,
    eta: f32,
    lambda: f32,
    rng: Rng,
}

impl MatchedRlRpe {
    pub fn new(hidden: usize, eta: f32, lambda: f32, eta_v: f32, beta: f32, seed: u64) -> Self {
        Self {
            arch: MatchedArch::feedforward(hidden, beta, seed),
            critic: LearnedRpeCritic::new(hidden, eta_v),
            eta,
            lambda,
            rng: Rng::new(seed ^ 0x0012_E0C1_ED17_u64),
        }
    }

    pub fn train_and_evaluate(
        &mut self,
        epochs: usize,
        train: &[GradientExample],
        test: &[GradientExample],
    ) -> GradientReferenceReport {
        assert!(!train.is_empty(), "matched RL-RPE needs training data");
        assert!(!test.is_empty(), "matched RL-RPE needs test data");
        train_minibatches_rpe(
            &mut self.arch,
            &mut self.critic,
            &mut self.rng,
            epochs,
            train,
            self.eta,
            self.lambda,
        );
        let (accuracy, loss) = self.arch.evaluate(test);
        GradientReferenceReport {
            label: MATCHED_RL_RPE_LABEL,
            accuracy,
            loss,
        }
    }
}

/// Online learned feedback alignment arm on matched feed-forward.
#[derive(Clone, Debug)]
pub struct MatchedRlLearnedFb {
    pub(crate) arch: MatchedArch,
    feedback: LearnedReinforceFeedback,
    eta: f32,
    lambda: f32,
    rng: Rng,
}

impl MatchedRlLearnedFb {
    pub fn new(hidden: usize, eta: f32, lambda: f32, eta_b: f32, beta: f32, seed: u64) -> Self {
        // Feed-forward is this arm's historical graph, preserved so no
        // archived number moves. Name the graph through `on` instead.
        Self::on(
            MatchedForward::FeedForward,
            hidden,
            eta,
            lambda,
            eta_b,
            beta,
            seed,
        )
    }

    /// This arm on an explicitly named graph. See [`MatchedForward`].
    pub fn on(
        forward: MatchedForward,
        hidden: usize,
        eta: f32,
        lambda: f32,
        eta_b: f32,
        beta: f32,
        seed: u64,
    ) -> Self {
        Self {
            arch: MatchedArch::on(forward, hidden, beta, seed),
            feedback: LearnedReinforceFeedback::new(hidden, seed, eta_b),
            eta,
            lambda,
            rng: Rng::new(seed ^ 0x0034_B0C1_ED17_u64),
        }
    }

    pub fn train_and_evaluate(
        &mut self,
        epochs: usize,
        train: &[GradientExample],
        test: &[GradientExample],
    ) -> GradientReferenceReport {
        assert!(
            !train.is_empty(),
            "matched RL-learned-FB needs training data"
        );
        assert!(!test.is_empty(), "matched RL-learned-FB needs test data");
        train_minibatches_learned_fb(
            &mut self.arch,
            &mut self.feedback,
            &mut self.rng,
            epochs,
            train,
            self.eta,
            self.lambda,
        );
        let (accuracy, loss) = self.arch.evaluate(test);
        GradientReferenceReport {
            label: MATCHED_RL_LEARNED_FB_LABEL,
            accuracy,
            loss,
        }
    }

    pub fn evaluate(&self, test: &[GradientExample]) -> f32 {
        self.arch.evaluate(test).0
    }

    pub fn feedback_weights(&self) -> &[f32] {
        self.feedback.weights()
    }
}

/// 2-Hidden-Layer Deep SNN with Online Learned Feedback Alignment (Suite 1).
#[derive(Clone, Debug)]
pub struct MatchedRlDeepLearnedFb {
    pub hidden1: usize,
    pub hidden2: usize,
    pub beta: f32,
    pub eta: f32,
    pub lambda: f32,
    pub w_in: Vec<f32>,    // hidden1 × N_IN
    pub w_h1_h2: Vec<f32>, // hidden2 × hidden1
    pub w_out: Vec<f32>,   // hidden2
    pub b1: LearnedReinforceFeedback,
    pub b2: LearnedReinforceFeedback,
    pub rng: Rng,
}

impl MatchedRlDeepLearnedFb {
    pub fn new(
        hidden1: usize,
        hidden2: usize,
        eta: f32,
        lambda: f32,
        eta_b: f32,
        beta: f32,
        seed: u64,
    ) -> Self {
        let mut rng = Rng::new(seed ^ 0x007C_D001_00F3_u64);
        let in_scale = 0.5f32;
        let h_scale = 0.3f32 / (hidden1 as f32).sqrt();
        let out_scale = 0.2f32;

        let w_in: Vec<f32> = (0..hidden1 * N_IN)
            .map(|_| (rng.next_f32() * 2.0 - 1.0) * in_scale)
            .collect();
        let w_h1_h2: Vec<f32> = (0..hidden2 * hidden1)
            .map(|_| (rng.next_f32() * 2.0 - 1.0) * h_scale)
            .collect();
        let w_out: Vec<f32> = (0..hidden2)
            .map(|_| (rng.next_f32() * 2.0 - 1.0) * out_scale)
            .collect();

        let b1 = LearnedReinforceFeedback::new(hidden1, seed ^ 0x1111, eta_b);
        let b2 = LearnedReinforceFeedback::new(hidden2, seed ^ 0x2222, eta_b);

        Self {
            hidden1,
            hidden2,
            beta,
            eta,
            lambda,
            w_in,
            w_h1_h2,
            w_out,
            b1,
            b2,
            rng,
        }
    }

    pub fn train_and_evaluate(
        &mut self,
        epochs: usize,
        train: &[GradientExample],
        test: &[GradientExample],
    ) -> GradientReferenceReport {
        let alpha = (-1.0f32 / DEFAULT_TAU_M).exp();
        for _epoch in 0..epochs {
            for (x1, x2, label) in train {
                let mut u1 = vec![0.0f32; self.hidden1];
                let mut s1 = vec![0.0f32; self.hidden1];
                let mut u2 = vec![0.0f32; self.hidden2];
                let mut s2 = vec![0.0f32; self.hidden2];

                let mut e_in = vec![0.0f32; self.hidden1 * N_IN];
                let mut e_h = vec![0.0f32; self.hidden2 * self.hidden1];
                let mut e_out = vec![0.0f32; self.hidden2];

                for t in 0..T {
                    let in_val = [x1[t], x2[t]];
                    // Layer 1
                    for i in 0..self.hidden1 {
                        let mut drive = 0.0f32;
                        for c in 0..N_IN {
                            drive += self.w_in[i * N_IN + c] * in_val[c];
                        }
                        u1[i] = alpha * u1[i] + drive;
                        let sp = if u1[i] >= THETA_REST { 1.0 } else { 0.0 };
                        if sp > 0.0 {
                            u1[i] = V_RESET;
                        }
                        s1[i] += sp;

                        let surr = 1.0 / (1.0 + self.beta * (u1[i] - THETA_REST).abs()).powi(2);
                        for c in 0..N_IN {
                            let idx = i * N_IN + c;
                            e_in[idx] = alpha * e_in[idx] + surr * in_val[c];
                        }
                    }

                    // Layer 2
                    for j in 0..self.hidden2 {
                        let mut drive = 0.0f32;
                        for i in 0..self.hidden1 {
                            drive += self.w_h1_h2[j * self.hidden1 + i] * (s1[i] / T as f32);
                        }
                        u2[j] = alpha * u2[j] + drive;
                        let sp = if u2[j] >= THETA_REST { 1.0 } else { 0.0 };
                        if sp > 0.0 {
                            u2[j] = V_RESET;
                        }
                        s2[j] += sp;

                        let surr = 1.0 / (1.0 + self.beta * (u2[j] - THETA_REST).abs()).powi(2);
                        for i in 0..self.hidden1 {
                            let idx = j * self.hidden1 + i;
                            e_h[idx] = alpha * e_h[idx] + surr * (s1[i] / T as f32);
                        }
                    }
                }

                // Readout policy
                let mut logit = 0.0f32;
                for j in 0..self.hidden2 {
                    let rate = s2[j] / T as f32;
                    logit += self.w_out[j] * rate;
                    e_out[j] = rate;
                }
                let p = 1.0 / (1.0 + (-logit).exp());
                let a = if self.rng.next_f32() < p { 1.0 } else { 0.0 };
                let r = if (a - *label).abs() < 0.5 { 1.0 } else { -1.0 };
                let re = reinforce_term(r, a, p);

                let fb1_sig = self.b1.credit(re);
                let fb2_sig = self.b2.credit(re);
                self.b1.update(re, &s1);
                self.b2.update(re, &s2);

                // Update weights
                for j in 0..self.hidden2 {
                    let g2 = fb2_sig.for_post(j as u32);
                    self.w_out[j] += self.eta * (a - p) * e_out[j];
                    for i in 0..self.hidden1 {
                        let idx = j * self.hidden1 + i;
                        self.w_h1_h2[idx] +=
                            self.eta * e_h[idx] * g2 - self.lambda * self.w_h1_h2[idx];
                    }
                }
                for i in 0..self.hidden1 {
                    let g1 = fb1_sig.for_post(i as u32);
                    for c in 0..N_IN {
                        let idx = i * N_IN + c;
                        self.w_in[idx] += self.eta * e_in[idx] * g1 - self.lambda * self.w_in[idx];
                    }
                }
            }
        }

        // Evaluate
        let mut correct = 0;
        for (x1, x2, label) in test {
            let mut u1 = vec![0.0f32; self.hidden1];
            let mut s1 = vec![0.0f32; self.hidden1];
            let mut u2 = vec![0.0f32; self.hidden2];
            let mut s2 = vec![0.0f32; self.hidden2];

            for t in 0..T {
                let in_val = [x1[t], x2[t]];
                for i in 0..self.hidden1 {
                    let mut drive = 0.0f32;
                    for c in 0..N_IN {
                        drive += self.w_in[i * N_IN + c] * in_val[c];
                    }
                    u1[i] = alpha * u1[i] + drive;
                    if u1[i] >= THETA_REST {
                        u1[i] = V_RESET;
                        s1[i] += 1.0;
                    }
                }
                for j in 0..self.hidden2 {
                    let mut drive = 0.0f32;
                    for i in 0..self.hidden1 {
                        drive += self.w_h1_h2[j * self.hidden1 + i] * (s1[i] / T as f32);
                    }
                    u2[j] = alpha * u2[j] + drive;
                    if u2[j] >= THETA_REST {
                        u2[j] = V_RESET;
                        s2[j] += 1.0;
                    }
                }
            }
            let mut logit = 0.0f32;
            for j in 0..self.hidden2 {
                logit += self.w_out[j] * (s2[j] / T as f32);
            }
            let pred = if logit >= 0.0 { 1.0 } else { 0.0 };
            if (pred - *label).abs() < 0.5 {
                correct += 1;
            }
        }

        let accuracy = correct as f32 / test.len() as f32;
        GradientReferenceReport {
            label: "MATCHED_RL_DEEP_LEARNED_FB",
            accuracy,
            loss: 0.0,
        }
    }
}

/// 3-Hidden-Layer Deep SNN with Online Learned Feedback Alignment (Suite 1).
#[derive(Clone, Debug)]
pub struct MatchedRl3LayerLearnedFb {
    pub hidden1: usize,
    pub hidden2: usize,
    pub hidden3: usize,
    pub beta: f32,
    pub eta: f32,
    pub lambda: f32,
    pub w_in: Vec<f32>,
    pub w_h1_h2: Vec<f32>,
    pub w_h2_h3: Vec<f32>,
    pub w_out: Vec<f32>,
    pub b1: LearnedReinforceFeedback,
    pub b2: LearnedReinforceFeedback,
    pub b3: LearnedReinforceFeedback,
    pub rng: Rng,
}

impl MatchedRl3LayerLearnedFb {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        hidden1: usize,
        hidden2: usize,
        hidden3: usize,
        eta: f32,
        lambda: f32,
        eta_b: f32,
        beta: f32,
        seed: u64,
    ) -> Self {
        let mut rng = Rng::new(seed ^ 0x007C_D003_00F3_u64);
        let in_scale = 0.5f32;
        let h1_scale = 0.3f32 / (hidden1 as f32).sqrt();
        let h2_scale = 0.3f32 / (hidden2 as f32).sqrt();
        let out_scale = 0.2f32;

        let w_in: Vec<f32> = (0..hidden1 * N_IN)
            .map(|_| (rng.next_f32() * 2.0 - 1.0) * in_scale)
            .collect();
        let w_h1_h2: Vec<f32> = (0..hidden2 * hidden1)
            .map(|_| (rng.next_f32() * 2.0 - 1.0) * h1_scale)
            .collect();
        let w_h2_h3: Vec<f32> = (0..hidden3 * hidden2)
            .map(|_| (rng.next_f32() * 2.0 - 1.0) * h2_scale)
            .collect();
        let w_out: Vec<f32> = (0..hidden3)
            .map(|_| (rng.next_f32() * 2.0 - 1.0) * out_scale)
            .collect();

        let b1 = LearnedReinforceFeedback::new(hidden1, seed ^ 0x1111, eta_b);
        let b2 = LearnedReinforceFeedback::new(hidden2, seed ^ 0x2222, eta_b);
        let b3 = LearnedReinforceFeedback::new(hidden3, seed ^ 0x3333, eta_b);

        Self {
            hidden1,
            hidden2,
            hidden3,
            beta,
            eta,
            lambda,
            w_in,
            w_h1_h2,
            w_h2_h3,
            w_out,
            b1,
            b2,
            b3,
            rng,
        }
    }

    pub fn train_and_evaluate(
        &mut self,
        epochs: usize,
        train: &[GradientExample],
        test: &[GradientExample],
    ) -> GradientReferenceReport {
        let alpha = (-1.0f32 / DEFAULT_TAU_M).exp();
        for _epoch in 0..epochs {
            for (x1, x2, label) in train {
                let mut u1 = vec![0.0f32; self.hidden1];
                let mut s1 = vec![0.0f32; self.hidden1];
                let mut u2 = vec![0.0f32; self.hidden2];
                let mut s2 = vec![0.0f32; self.hidden2];
                let mut u3 = vec![0.0f32; self.hidden3];
                let mut s3 = vec![0.0f32; self.hidden3];

                let mut e_in = vec![0.0f32; self.hidden1 * N_IN];
                let mut e_h1 = vec![0.0f32; self.hidden2 * self.hidden1];
                let mut e_h2 = vec![0.0f32; self.hidden3 * self.hidden2];
                let mut e_out = vec![0.0f32; self.hidden3];

                for t in 0..T {
                    let in_val = [x1[t], x2[t]];
                    // Layer 1
                    for i in 0..self.hidden1 {
                        let mut drive = 0.0f32;
                        for c in 0..N_IN {
                            drive += self.w_in[i * N_IN + c] * in_val[c];
                        }
                        u1[i] = alpha * u1[i] + drive;
                        let sp = if u1[i] >= THETA_REST { 1.0 } else { 0.0 };
                        if sp > 0.0 {
                            u1[i] = V_RESET;
                        }
                        s1[i] += sp;
                        let surr = 1.0 / (1.0 + self.beta * (u1[i] - THETA_REST).abs()).powi(2);
                        for c in 0..N_IN {
                            let idx = i * N_IN + c;
                            e_in[idx] = alpha * e_in[idx] + surr * in_val[c];
                        }
                    }

                    // Layer 2
                    for j in 0..self.hidden2 {
                        let mut drive = 0.0f32;
                        for i in 0..self.hidden1 {
                            drive += self.w_h1_h2[j * self.hidden1 + i] * (s1[i] / T as f32);
                        }
                        u2[j] = alpha * u2[j] + drive;
                        let sp = if u2[j] >= THETA_REST { 1.0 } else { 0.0 };
                        if sp > 0.0 {
                            u2[j] = V_RESET;
                        }
                        s2[j] += sp;
                        let surr = 1.0 / (1.0 + self.beta * (u2[j] - THETA_REST).abs()).powi(2);
                        for i in 0..self.hidden1 {
                            let idx = j * self.hidden1 + i;
                            e_h1[idx] = alpha * e_h1[idx] + surr * (s1[i] / T as f32);
                        }
                    }

                    // Layer 3
                    for k in 0..self.hidden3 {
                        let mut drive = 0.0f32;
                        for j in 0..self.hidden2 {
                            drive += self.w_h2_h3[k * self.hidden2 + j] * (s2[j] / T as f32);
                        }
                        u3[k] = alpha * u3[k] + drive;
                        let sp = if u3[k] >= THETA_REST { 1.0 } else { 0.0 };
                        if sp > 0.0 {
                            u3[k] = V_RESET;
                        }
                        s3[k] += sp;
                        let surr = 1.0 / (1.0 + self.beta * (u3[k] - THETA_REST).abs()).powi(2);
                        for j in 0..self.hidden2 {
                            let idx = k * self.hidden2 + j;
                            e_h2[idx] = alpha * e_h2[idx] + surr * (s2[j] / T as f32);
                        }
                    }
                }

                // Readout
                let mut logit = 0.0f32;
                for k in 0..self.hidden3 {
                    let rate = s3[k] / T as f32;
                    logit += self.w_out[k] * rate;
                    e_out[k] = rate;
                }
                let p = 1.0 / (1.0 + (-logit).exp());
                let a = if self.rng.next_f32() < p { 1.0 } else { 0.0 };
                let r = if (a - *label).abs() < 0.5 { 1.0 } else { -1.0 };
                let re = reinforce_term(r, a, p);

                let fb1_sig = self.b1.credit(re);
                let fb2_sig = self.b2.credit(re);
                let fb3_sig = self.b3.credit(re);
                self.b1.update(re, &s1);
                self.b2.update(re, &s2);
                self.b3.update(re, &s3);

                // Update weights
                for k in 0..self.hidden3 {
                    let g3 = fb3_sig.for_post(k as u32);
                    self.w_out[k] += self.eta * (a - p) * e_out[k];
                    for j in 0..self.hidden2 {
                        let idx = k * self.hidden2 + j;
                        self.w_h2_h3[idx] +=
                            self.eta * e_h2[idx] * g3 - self.lambda * self.w_h2_h3[idx];
                    }
                }
                for j in 0..self.hidden2 {
                    let g2 = fb2_sig.for_post(j as u32);
                    for i in 0..self.hidden1 {
                        let idx = j * self.hidden1 + i;
                        self.w_h1_h2[idx] +=
                            self.eta * e_h1[idx] * g2 - self.lambda * self.w_h1_h2[idx];
                    }
                }
                for i in 0..self.hidden1 {
                    let g1 = fb1_sig.for_post(i as u32);
                    for c in 0..N_IN {
                        let idx = i * N_IN + c;
                        self.w_in[idx] += self.eta * e_in[idx] * g1 - self.lambda * self.w_in[idx];
                    }
                }
            }
        }

        // Evaluate
        let mut correct = 0;
        for (x1, x2, label) in test {
            let mut u1 = vec![0.0f32; self.hidden1];
            let mut s1 = vec![0.0f32; self.hidden1];
            let mut u2 = vec![0.0f32; self.hidden2];
            let mut s2 = vec![0.0f32; self.hidden2];
            let mut u3 = vec![0.0f32; self.hidden3];
            let mut s3 = vec![0.0f32; self.hidden3];

            for t in 0..T {
                let in_val = [x1[t], x2[t]];
                for i in 0..self.hidden1 {
                    let mut drive = 0.0f32;
                    for c in 0..N_IN {
                        drive += self.w_in[i * N_IN + c] * in_val[c];
                    }
                    u1[i] = alpha * u1[i] + drive;
                    if u1[i] >= THETA_REST {
                        u1[i] = V_RESET;
                        s1[i] += 1.0;
                    }
                }
                for j in 0..self.hidden2 {
                    let mut drive = 0.0f32;
                    for i in 0..self.hidden1 {
                        drive += self.w_h1_h2[j * self.hidden1 + i] * (s1[i] / T as f32);
                    }
                    u2[j] = alpha * u2[j] + drive;
                    if u2[j] >= THETA_REST {
                        u2[j] = V_RESET;
                        s2[j] += 1.0;
                    }
                }
                for k in 0..self.hidden3 {
                    let mut drive = 0.0f32;
                    for j in 0..self.hidden2 {
                        drive += self.w_h2_h3[k * self.hidden2 + j] * (s2[j] / T as f32);
                    }
                    u3[k] = alpha * u3[k] + drive;
                    if u3[k] >= THETA_REST {
                        u3[k] = V_RESET;
                        s3[k] += 1.0;
                    }
                }
            }
            let mut logit = 0.0f32;
            for k in 0..self.hidden3 {
                logit += self.w_out[k] * (s3[k] / T as f32);
            }
            let pred = if logit >= 0.0 { 1.0 } else { 0.0 };
            if (pred - *label).abs() < 0.5 {
                correct += 1;
            }
        }

        let accuracy = correct as f32 / test.len() as f32;
        GradientReferenceReport {
            label: "MATCHED_RL_3LAYER_LEARNED_FB",
            accuracy,
            loss: 0.0,
        }
    }
}

/// 4-Hidden-Layer Deep SNN with Online Learned Feedback Alignment (Suite 1).
#[derive(Clone, Debug)]
pub struct MatchedRl4LayerLearnedFb {
    pub hidden1: usize,
    pub hidden2: usize,
    pub hidden3: usize,
    pub hidden4: usize,
    pub beta: f32,
    pub eta: f32,
    pub lambda: f32,
    pub w_in: Vec<f32>,
    pub w_h1_h2: Vec<f32>,
    pub w_h2_h3: Vec<f32>,
    pub w_h3_h4: Vec<f32>,
    pub w_out: Vec<f32>,
    pub b1: LearnedReinforceFeedback,
    pub b2: LearnedReinforceFeedback,
    pub b3: LearnedReinforceFeedback,
    pub b4: LearnedReinforceFeedback,
    pub rng: Rng,
}

impl MatchedRl4LayerLearnedFb {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        hidden1: usize,
        hidden2: usize,
        hidden3: usize,
        hidden4: usize,
        eta: f32,
        lambda: f32,
        eta_b: f32,
        beta: f32,
        seed: u64,
    ) -> Self {
        let mut rng = Rng::new(seed ^ 0x007C_D004_00F3_u64);
        let in_scale = 0.5f32;
        let h1_scale = 0.3f32 / (hidden1 as f32).sqrt();
        let h2_scale = 0.3f32 / (hidden2 as f32).sqrt();
        let h3_scale = 0.3f32 / (hidden3 as f32).sqrt();
        let out_scale = 0.2f32;

        let w_in: Vec<f32> = (0..hidden1 * N_IN)
            .map(|_| (rng.next_f32() * 2.0 - 1.0) * in_scale)
            .collect();
        let w_h1_h2: Vec<f32> = (0..hidden2 * hidden1)
            .map(|_| (rng.next_f32() * 2.0 - 1.0) * h1_scale)
            .collect();
        let w_h2_h3: Vec<f32> = (0..hidden3 * hidden2)
            .map(|_| (rng.next_f32() * 2.0 - 1.0) * h2_scale)
            .collect();
        let w_h3_h4: Vec<f32> = (0..hidden4 * hidden3)
            .map(|_| (rng.next_f32() * 2.0 - 1.0) * h3_scale)
            .collect();
        let w_out: Vec<f32> = (0..hidden4)
            .map(|_| (rng.next_f32() * 2.0 - 1.0) * out_scale)
            .collect();

        let b1 = LearnedReinforceFeedback::new(hidden1, seed ^ 0x1111, eta_b);
        let b2 = LearnedReinforceFeedback::new(hidden2, seed ^ 0x2222, eta_b);
        let b3 = LearnedReinforceFeedback::new(hidden3, seed ^ 0x3333, eta_b);
        let b4 = LearnedReinforceFeedback::new(hidden4, seed ^ 0x4444, eta_b);

        Self {
            hidden1,
            hidden2,
            hidden3,
            hidden4,
            beta,
            eta,
            lambda,
            w_in,
            w_h1_h2,
            w_h2_h3,
            w_h3_h4,
            w_out,
            b1,
            b2,
            b3,
            b4,
            rng,
        }
    }

    pub fn train_and_evaluate(
        &mut self,
        epochs: usize,
        train: &[GradientExample],
        test: &[GradientExample],
    ) -> GradientReferenceReport {
        let alpha = (-1.0f32 / DEFAULT_TAU_M).exp();
        for _epoch in 0..epochs {
            for (x1, x2, label) in train {
                let mut u1 = vec![0.0f32; self.hidden1];
                let mut s1 = vec![0.0f32; self.hidden1];
                let mut u2 = vec![0.0f32; self.hidden2];
                let mut s2 = vec![0.0f32; self.hidden2];
                let mut u3 = vec![0.0f32; self.hidden3];
                let mut s3 = vec![0.0f32; self.hidden3];
                let mut u4 = vec![0.0f32; self.hidden4];
                let mut s4 = vec![0.0f32; self.hidden4];

                let mut e_in = vec![0.0f32; self.hidden1 * N_IN];
                let mut e_h1 = vec![0.0f32; self.hidden2 * self.hidden1];
                let mut e_h2 = vec![0.0f32; self.hidden3 * self.hidden2];
                let mut e_h3 = vec![0.0f32; self.hidden4 * self.hidden3];
                let mut e_out = vec![0.0f32; self.hidden4];

                for t in 0..T {
                    let in_val = [x1[t], x2[t]];
                    // Layer 1
                    for i in 0..self.hidden1 {
                        let mut drive = 0.0f32;
                        for c in 0..N_IN {
                            drive += self.w_in[i * N_IN + c] * in_val[c];
                        }
                        u1[i] = alpha * u1[i] + drive;
                        let sp = if u1[i] >= THETA_REST { 1.0 } else { 0.0 };
                        if sp > 0.0 {
                            u1[i] = V_RESET;
                        }
                        s1[i] += sp;
                        let surr = 1.0 / (1.0 + self.beta * (u1[i] - THETA_REST).abs()).powi(2);
                        for c in 0..N_IN {
                            let idx = i * N_IN + c;
                            e_in[idx] = alpha * e_in[idx] + surr * in_val[c];
                        }
                    }

                    // Layer 2
                    for j in 0..self.hidden2 {
                        let mut drive = 0.0f32;
                        for i in 0..self.hidden1 {
                            drive += self.w_h1_h2[j * self.hidden1 + i] * (s1[i] / T as f32);
                        }
                        u2[j] = alpha * u2[j] + drive;
                        let sp = if u2[j] >= THETA_REST { 1.0 } else { 0.0 };
                        if sp > 0.0 {
                            u2[j] = V_RESET;
                        }
                        s2[j] += sp;
                        let surr = 1.0 / (1.0 + self.beta * (u2[j] - THETA_REST).abs()).powi(2);
                        for i in 0..self.hidden1 {
                            let idx = j * self.hidden1 + i;
                            e_h1[idx] = alpha * e_h1[idx] + surr * (s1[i] / T as f32);
                        }
                    }

                    // Layer 3
                    for k in 0..self.hidden3 {
                        let mut drive = 0.0f32;
                        for j in 0..self.hidden2 {
                            drive += self.w_h2_h3[k * self.hidden2 + j] * (s2[j] / T as f32);
                        }
                        u3[k] = alpha * u3[k] + drive;
                        let sp = if u3[k] >= THETA_REST { 1.0 } else { 0.0 };
                        if sp > 0.0 {
                            u3[k] = V_RESET;
                        }
                        s3[k] += sp;
                        let surr = 1.0 / (1.0 + self.beta * (u3[k] - THETA_REST).abs()).powi(2);
                        for j in 0..self.hidden2 {
                            let idx = k * self.hidden2 + j;
                            e_h2[idx] = alpha * e_h2[idx] + surr * (s2[j] / T as f32);
                        }
                    }

                    // Layer 4
                    for l in 0..self.hidden4 {
                        let mut drive = 0.0f32;
                        for k in 0..self.hidden3 {
                            drive += self.w_h3_h4[l * self.hidden3 + k] * (s3[k] / T as f32);
                        }
                        u4[l] = alpha * u4[l] + drive;
                        let sp = if u4[l] >= THETA_REST { 1.0 } else { 0.0 };
                        if sp > 0.0 {
                            u4[l] = V_RESET;
                        }
                        s4[l] += sp;
                        let surr = 1.0 / (1.0 + self.beta * (u4[l] - THETA_REST).abs()).powi(2);
                        for k in 0..self.hidden3 {
                            let idx = l * self.hidden3 + k;
                            e_h3[idx] = alpha * e_h3[idx] + surr * (s3[k] / T as f32);
                        }
                    }
                }

                // Readout
                let mut logit = 0.0f32;
                for l in 0..self.hidden4 {
                    let rate = s4[l] / T as f32;
                    logit += self.w_out[l] * rate;
                    e_out[l] = rate;
                }
                let p = 1.0 / (1.0 + (-logit).exp());
                let a = if self.rng.next_f32() < p { 1.0 } else { 0.0 };
                let r = if (a - *label).abs() < 0.5 { 1.0 } else { -1.0 };
                let re = reinforce_term(r, a, p);

                let fb1_sig = self.b1.credit(re);
                let fb2_sig = self.b2.credit(re);
                let fb3_sig = self.b3.credit(re);
                let fb4_sig = self.b4.credit(re);
                self.b1.update(re, &s1);
                self.b2.update(re, &s2);
                self.b3.update(re, &s3);
                self.b4.update(re, &s4);

                // Update weights
                for l in 0..self.hidden4 {
                    let g4 = fb4_sig.for_post(l as u32);
                    self.w_out[l] += self.eta * (a - p) * e_out[l];
                    for k in 0..self.hidden3 {
                        let idx = l * self.hidden3 + k;
                        self.w_h3_h4[idx] +=
                            self.eta * e_h3[idx] * g4 - self.lambda * self.w_h3_h4[idx];
                    }
                }
                for k in 0..self.hidden3 {
                    let g3 = fb3_sig.for_post(k as u32);
                    for j in 0..self.hidden2 {
                        let idx = k * self.hidden2 + j;
                        self.w_h2_h3[idx] +=
                            self.eta * e_h2[idx] * g3 - self.lambda * self.w_h2_h3[idx];
                    }
                }
                for j in 0..self.hidden2 {
                    let g2 = fb2_sig.for_post(j as u32);
                    for i in 0..self.hidden1 {
                        let idx = j * self.hidden1 + i;
                        self.w_h1_h2[idx] +=
                            self.eta * e_h1[idx] * g2 - self.lambda * self.w_h1_h2[idx];
                    }
                }
                for i in 0..self.hidden1 {
                    let g1 = fb1_sig.for_post(i as u32);
                    for c in 0..N_IN {
                        let idx = i * N_IN + c;
                        self.w_in[idx] += self.eta * e_in[idx] * g1 - self.lambda * self.w_in[idx];
                    }
                }
            }
        }

        // Evaluate
        let mut correct = 0;
        for (x1, x2, label) in test {
            let mut u1 = vec![0.0f32; self.hidden1];
            let mut s1 = vec![0.0f32; self.hidden1];
            let mut u2 = vec![0.0f32; self.hidden2];
            let mut s2 = vec![0.0f32; self.hidden2];
            let mut u3 = vec![0.0f32; self.hidden3];
            let mut s3 = vec![0.0f32; self.hidden3];
            let mut u4 = vec![0.0f32; self.hidden4];
            let mut s4 = vec![0.0f32; self.hidden4];

            for t in 0..T {
                let in_val = [x1[t], x2[t]];
                for i in 0..self.hidden1 {
                    let mut drive = 0.0f32;
                    for c in 0..N_IN {
                        drive += self.w_in[i * N_IN + c] * in_val[c];
                    }
                    u1[i] = alpha * u1[i] + drive;
                    if u1[i] >= THETA_REST {
                        u1[i] = V_RESET;
                        s1[i] += 1.0;
                    }
                }
                for j in 0..self.hidden2 {
                    let mut drive = 0.0f32;
                    for i in 0..self.hidden1 {
                        drive += self.w_h1_h2[j * self.hidden1 + i] * (s1[i] / T as f32);
                    }
                    u2[j] = alpha * u2[j] + drive;
                    if u2[j] >= THETA_REST {
                        u2[j] = V_RESET;
                        s2[j] += 1.0;
                    }
                }
                for k in 0..self.hidden3 {
                    let mut drive = 0.0f32;
                    for j in 0..self.hidden2 {
                        drive += self.w_h2_h3[k * self.hidden2 + j] * (s2[j] / T as f32);
                    }
                    u3[k] = alpha * u3[k] + drive;
                    if u3[k] >= THETA_REST {
                        u3[k] = V_RESET;
                        s3[k] += 1.0;
                    }
                }
                for l in 0..self.hidden4 {
                    let mut drive = 0.0f32;
                    for k in 0..self.hidden3 {
                        drive += self.w_h3_h4[l * self.hidden3 + k] * (s3[k] / T as f32);
                    }
                    u4[l] = alpha * u4[l] + drive;
                    if u4[l] >= THETA_REST {
                        u4[l] = V_RESET;
                        s4[l] += 1.0;
                    }
                }
            }
            let mut logit = 0.0f32;
            for l in 0..self.hidden4 {
                logit += self.w_out[l] * (s4[l] / T as f32);
            }
            let pred = if logit >= 0.0 { 1.0 } else { 0.0 };
            if (pred - *label).abs() < 0.5 {
                correct += 1;
            }
        }

        let accuracy = correct as f32 / test.len() as f32;
        GradientReferenceReport {
            label: "MATCHED_RL_4LAYER_LEARNED_FB",
            accuracy,
            loss: 0.0,
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn train_minibatches(
    arch: &mut MatchedArch,
    rng: &mut Rng,
    epochs: usize,
    train: &[GradientExample],
    eta: f32,
    lambda: f32,
    rule: RlHiddenRule,
    feedback: Option<&[f32]>,
) {
    let n = train.len();
    let mut order: Vec<usize> = (0..n).collect();
    let mut graded_base = 0.0f32;
    for _ in 0..epochs {
        for i in (1..n).rev() {
            let j = rng.gen_index(i + 1);
            order.swap(i, j);
        }
        let mut start = 0;
        while start < n {
            let end = (start + DEFAULT_BATCH).min(n);
            apply_rl_batch(
                arch,
                rng,
                train,
                &order[start..end],
                eta,
                lambda,
                rule,
                feedback,
                &mut graded_base,
            );
            start = end;
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn apply_rl_batch(
    arch: &mut MatchedArch,
    rng: &mut Rng,
    train: &[GradientExample],
    indices: &[usize],
    eta: f32,
    lambda: f32,
    rule: RlHiddenRule,
    feedback: Option<&[f32]>,
    graded_base: &mut f32,
) {
    let h = arch.hidden;
    let nb = indices.len().max(1) as f32;

    let mut dwin = vec![0.0f32; h * N_IN];
    let mut dwout = vec![0.0f32; h];
    let mut dby = 0.0f32;
    let mut pcorr_sum = 0.0f32;

    for &idx in indices {
        let (x1, x2, y) = &train[idx];
        let cache = arch.forward(x1, x2);
        let p = sigmoid(cache.logit);
        let a = if rng.next_f32() < p { 1.0f32 } else { 0.0 };
        let r = if (a - *y).abs() < 0.5 { 1.0f32 } else { -1.0 };
        let pcorr = if *y > 0.5 { p } else { 1.0 - p };
        pcorr_sum += pcorr;
        let reinforce = r * (a - p);

        // Readout: always REINFORCE (NumPy `r * (a - p)`).
        for i in 0..h {
            dwout[i] += reinforce * cache.rates[i];
        }
        dby += reinforce;

        let e_in = eligibility_in(arch, &cache);
        let mod_vec: Vec<f32> = match rule {
            RlHiddenRule::Flat => vec![r; h],
            RlHiddenRule::Graded => {
                let teach = pcorr - *graded_base;
                vec![teach; h]
            }
            RlHiddenRule::ReinforceFb => {
                let fb = feedback.expect("reinforce-fb requires frozen feedback");
                assert_eq!(fb.len(), h, "RL feedback width must match hidden");
                fb.iter().map(|b| *b * reinforce).collect()
            }
        };
        for i in 0..h {
            let m = mod_vec[i];
            for j in 0..N_IN {
                let wi = i * N_IN + j;
                dwin[wi] += m * e_in[wi];
            }
        }
    }

    if rule == RlHiddenRule::Graded {
        // NumPy: `base = 0.9*base + 0.1*float(pcorr.mean())` after the batch.
        let mean_pcorr = pcorr_sum / nb;
        *graded_base =
            (1.0 - GRADED_BASELINE_EMA) * *graded_base + GRADED_BASELINE_EMA * mean_pcorr;
    }

    for i in 0..h {
        let w = &mut arch.wout[i];
        *w += eta * (dwout[i] / nb) - lambda * *w;
    }
    arch.by += eta * (dby / nb);
    for i in 0..h * N_IN {
        let w = &mut arch.win[i];
        *w += eta * (dwin[i] / nb) - lambda * *w;
    }
}

fn eligibility_in(arch: &MatchedArch, cache: &ForwardCache) -> Vec<f32> {
    let h = arch.hidden;
    let alpha = arch.alpha;
    let beta = arch.beta;
    let theta = THETA_REST;
    let mut e_in = vec![0.0f32; h * N_IN];
    for i in 0..h {
        let mut ei0 = 0.0f32;
        let mut ei1 = 0.0f32;
        for t in 0..T {
            let surr = surrogate(cache.u[i][t] - theta, beta);
            ei0 = alpha * ei0 + surr * cache.x[t][0];
            ei1 = alpha * ei1 + surr * cache.x[t][1];
        }
        e_in[i * N_IN] = ei0;
        e_in[i * N_IN + 1] = ei1;
    }
    e_in
}

fn train_minibatches_rpe(
    arch: &mut MatchedArch,
    critic: &mut LearnedRpeCritic,
    rng: &mut Rng,
    epochs: usize,
    train: &[GradientExample],
    eta: f32,
    lambda: f32,
) {
    let n = train.len();
    let mut order: Vec<usize> = (0..n).collect();
    for _ in 0..epochs {
        for i in (1..n).rev() {
            let j = rng.gen_index(i + 1);
            order.swap(i, j);
        }
        let mut start = 0;
        while start < n {
            let end = (start + DEFAULT_BATCH).min(n);
            apply_rl_batch_rpe(arch, critic, rng, train, &order[start..end], eta, lambda);
            start = end;
        }
    }
}

fn apply_rl_batch_rpe(
    arch: &mut MatchedArch,
    critic: &mut LearnedRpeCritic,
    rng: &mut Rng,
    train: &[GradientExample],
    indices: &[usize],
    eta: f32,
    lambda: f32,
) {
    let h = arch.hidden;
    let nb = indices.len().max(1) as f32;

    let mut dwin = vec![0.0f32; h * N_IN];
    let mut dwout = vec![0.0f32; h];
    let mut dby = 0.0f32;

    for &idx in indices {
        let (x1, x2, y) = &train[idx];
        let cache = arch.forward(x1, x2);
        let p = sigmoid(cache.logit);
        let a = if rng.next_f32() < p { 1.0f32 } else { 0.0 };
        let r = if (a - *y).abs() < 0.5 { 1.0f32 } else { -1.0 };
        let reinforce = r * (a - p);

        for i in 0..h {
            dwout[i] += reinforce * cache.rates[i];
        }
        dby += reinforce;

        let rpe = critic.rpe_and_update(r, &cache.rates);
        let e_in = eligibility_in(arch, &cache);
        for i in 0..h {
            for j in 0..N_IN {
                let wi = i * N_IN + j;
                dwin[wi] += rpe * e_in[wi];
            }
        }
    }

    for i in 0..h {
        let w = &mut arch.wout[i];
        *w += eta * (dwout[i] / nb) - lambda * *w;
    }
    arch.by += eta * (dby / nb);
    for i in 0..h * N_IN {
        let w = &mut arch.win[i];
        *w += eta * (dwin[i] / nb) - lambda * *w;
    }
}

fn train_minibatches_learned_fb(
    arch: &mut MatchedArch,
    feedback: &mut LearnedReinforceFeedback,
    rng: &mut Rng,
    epochs: usize,
    train: &[GradientExample],
    eta: f32,
    lambda: f32,
) {
    let n = train.len();
    let mut order: Vec<usize> = (0..n).collect();
    for _ in 0..epochs {
        for i in (1..n).rev() {
            let j = rng.gen_index(i + 1);
            order.swap(i, j);
        }
        let mut start = 0;
        while start < n {
            let end = (start + DEFAULT_BATCH).min(n);
            apply_rl_batch_learned_fb(arch, feedback, rng, train, &order[start..end], eta, lambda);
            start = end;
        }
    }
}

fn apply_rl_batch_learned_fb(
    arch: &mut MatchedArch,
    feedback: &mut LearnedReinforceFeedback,
    rng: &mut Rng,
    train: &[GradientExample],
    indices: &[usize],
    eta: f32,
    lambda: f32,
) {
    let h = arch.hidden;
    let nb = indices.len().max(1) as f32;

    let mut dwin = vec![0.0f32; h * N_IN];
    let mut dwout = vec![0.0f32; h];
    let mut dby = 0.0f32;

    for &idx in indices {
        let (x1, x2, y) = &train[idx];
        let cache = arch.forward(x1, x2);
        let p = sigmoid(cache.logit);
        let a = if rng.next_f32() < p { 1.0f32 } else { 0.0 };
        let r = if (a - *y).abs() < 0.5 { 1.0f32 } else { -1.0 };
        let reinforce = r * (a - p);

        for i in 0..h {
            dwout[i] += reinforce * cache.rates[i];
        }
        dby += reinforce;

        let credit = feedback.credit(reinforce);
        feedback.update(reinforce, &cache.rates);
        let e_in = eligibility_in(arch, &cache);
        for i in 0..h {
            let m = credit.for_post(i as u32);
            for j in 0..N_IN {
                let wi = i * N_IN + j;
                dwin[wi] += m * e_in[wi];
            }
        }
    }

    for i in 0..h {
        let w = &mut arch.wout[i];
        *w += eta * (dwout[i] / nb) - lambda * *w;
    }
    arch.by += eta * (dby / nb);
    for i in 0..h * N_IN {
        let w = &mut arch.win[i];
        *w += eta * (dwin[i] / nb) - lambda * *w;
    }
}

#[inline]
fn surrogate(u_minus_theta: f32, beta: f32) -> f32 {
    let d = 1.0 + beta * u_minus_theta.abs();
    1.0 / (d * d)
}

#[inline]
fn sigmoid(z: f32) -> f32 {
    1.0 / (1.0 + (-z).exp())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::matched_local_baseline::{MatchedGradient, DEFAULT_MATCHED_BETA};
    use binn_core::Rng;

    fn gen_examples(n: usize, seed: u64) -> Vec<GradientExample> {
        let mut rng = Rng::new(seed);
        let mut out = Vec::with_capacity(n);
        for _ in 0..n {
            let mut x1 = [0.0f32; T];
            let mut x2 = [0.0f32; T];
            let t1 = rng.gen_index(T);
            x1[t1] = 1.0;
            let coincident = rng.next_f32() < 0.5;
            let t2 = if coincident {
                if t1 + 1 < T && rng.next_f32() < 0.5 {
                    t1 + 1
                } else if t1 > 0 {
                    t1 - 1
                } else {
                    t1
                }
            } else {
                let mut t = rng.gen_index(T);
                while (t as isize - t1 as isize).abs() <= 1 {
                    t = rng.gen_index(T);
                }
                t
            };
            x2[t2] = 1.0;
            let y = if (t1 as isize - t2 as isize).abs() <= 1 {
                1.0
            } else {
                0.0
            };
            out.push((x1, x2, y));
        }
        out
    }

    #[test]
    fn header_forbids_production_use() {
        let src = include_str!("matched_rl_baseline.rs");
        assert!(src.contains("MUST NEVER BE THE PRODUCTION LEARNER"));
        assert!(src.contains("GC1 exempt") || src.contains("GC1-exempt"));
    }

    #[test]
    fn rl_arms_share_matched_forward_init() {
        let flat = MatchedRlFlat::new(8, 0.05, 0.0, DEFAULT_MATCHED_BETA, 0xABCD);
        let graded = MatchedRlGraded::new(8, 0.05, 0.0, DEFAULT_MATCHED_BETA, 0xABCD);
        let fb = MatchedRlReinforceFb::new(8, 0.05, 0.0, DEFAULT_MATCHED_BETA, 0xABCD);
        let g = MatchedGradient::new_feedforward(8, 0.02, DEFAULT_MATCHED_BETA, 0xABCD);
        let mut x1 = [0.0f32; T];
        let mut x2 = [0.0f32; T];
        x1[2] = 1.0;
        x2[3] = 1.0;
        let l_flat = flat.arch.forward(&x1, &x2).logit;
        assert_eq!(l_flat, graded.arch.forward(&x1, &x2).logit);
        assert_eq!(l_flat, fb.arch.forward(&x1, &x2).logit);
        assert_eq!(l_flat, g.arch.forward(&x1, &x2).logit);
    }

    #[test]
    fn reinforce_fb_is_frozen_and_deterministic() {
        let a = MatchedRlReinforceFb::new(16, 0.05, 0.0, DEFAULT_MATCHED_BETA, 77);
        let b = MatchedRlReinforceFb::new(16, 0.05, 0.0, DEFAULT_MATCHED_BETA, 77);
        assert_eq!(a.feedback_weights(), b.feedback_weights());
        let c = MatchedRlReinforceFb::new(16, 0.05, 0.0, DEFAULT_MATCHED_BETA, 78);
        assert_ne!(a.feedback_weights(), c.feedback_weights());
    }

    #[test]
    fn matched_reinforce_fb_uses_product_neuromodulator_weights() {
        use crate::credit::ReinforceFeedback;
        let seed = 0xC1A1_6000_0012;
        let matched = MatchedRlReinforceFb::new(32, 0.05, 0.0, DEFAULT_MATCHED_BETA, seed);
        let product = ReinforceFeedback::new(32, seed);
        assert_eq!(
            matched.feedback_weights(),
            product.weights(),
            "v12 matched arm must share B_i lineage with production ReinforceFeedback"
        );
    }

    #[test]
    fn graded_is_deterministic() {
        let train = gen_examples(24, 1);
        let test = gen_examples(16, 2);
        let mut a = MatchedRlGraded::new(16, 0.05, 0.002, DEFAULT_MATCHED_BETA, 777);
        let mut b = MatchedRlGraded::new(16, 0.05, 0.002, DEFAULT_MATCHED_BETA, 777);
        let ra = a.train_and_evaluate(4, &train, &test);
        let rb = b.train_and_evaluate(4, &train, &test);
        assert_eq!(ra.accuracy, rb.accuracy);
        assert_eq!(ra.loss, rb.loss);
    }

    #[test]
    fn graded_zero_eta_is_a_noop() {
        let train = gen_examples(12, 3);
        let test = gen_examples(8, 4);
        let mut d = MatchedRlGraded::new(12, 0.0, 0.0, DEFAULT_MATCHED_BETA, 42);
        let before = d.arch.win.clone();
        let _ = d.train_and_evaluate(3, &train, &test);
        assert_eq!(before, d.arch.win);
    }

    #[test]
    fn graded_learns_above_floor_on_coincidence() {
        let train = gen_examples(80, 0xA1_C01C);
        let test = gen_examples(40, 0xA1_7E57);
        let mut graded = MatchedRlGraded::new(128, 0.05, 0.0, DEFAULT_MATCHED_BETA, 7);
        let report = graded.train_and_evaluate(80, &train, &test);
        assert!(
            report.accuracy >= 0.65,
            "rl_graded should learn coincidence; got {:.3}",
            report.accuracy
        );
        assert!(report.loss.is_finite());
    }

    #[test]
    fn reinforce_fb_learns_above_floor_on_coincidence() {
        let train = gen_examples(80, 0xA1_C01C);
        let test = gen_examples(40, 0xA1_7E57);
        let mut fb = MatchedRlReinforceFb::new(128, 0.05, 0.0, DEFAULT_MATCHED_BETA, 0xFB01_1EA5);
        let report = fb.train_and_evaluate(80, &train, &test);
        assert!(
            report.accuracy >= 0.65,
            "rl_reinforce_fb should learn coincidence; got {:.3}",
            report.accuracy
        );
    }

    #[test]
    fn rpe_arm_runs_and_returns_finite_report() {
        let train = gen_examples(40, 0xA1_C01C);
        let test = gen_examples(20, 0xA1_7E57);
        let mut arm = MatchedRlRpe::new(64, 0.05, 0.0, 0.01, DEFAULT_MATCHED_BETA, 123);
        let report = arm.train_and_evaluate(10, &train, &test);
        assert!(report.loss.is_finite());
        assert_eq!(report.label, MATCHED_RL_RPE_LABEL);
    }

    #[test]
    fn learned_fb_arm_runs_and_returns_finite_report() {
        let train = gen_examples(40, 0xA1_C01C);
        let test = gen_examples(20, 0xA1_7E57);
        let mut arm = MatchedRlLearnedFb::new(64, 0.05, 0.0, 0.01, DEFAULT_MATCHED_BETA, 456);
        let report = arm.train_and_evaluate(10, &train, &test);
        assert!(report.loss.is_finite());
        assert_eq!(report.label, MATCHED_RL_LEARNED_FB_LABEL);
    }

    #[test]
    fn deep_learned_fb_arm_runs_and_learns() {
        let train = gen_examples(60, 0xA1_C01C);
        let test = gen_examples(30, 0xA1_7E57);
        let mut arm =
            MatchedRlDeepLearnedFb::new(64, 64, 0.05, 0.0, 0.01, DEFAULT_MATCHED_BETA, 789);
        let report = arm.train_and_evaluate(40, &train, &test);
        assert!(report.accuracy >= 0.50);
        assert_eq!(report.label, "MATCHED_RL_DEEP_LEARNED_FB");
    }
}
