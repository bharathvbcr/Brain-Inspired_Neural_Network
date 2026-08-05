//! Labeled **matched-architecture DFA** reference (U-MATCH-DFA / protocol v5).
//!
//! **GC1 exempt** (this is a `*_baseline.rs` file). Do not call from production
//! learning paths. Production code must use the online three-factor rule in
//! `three_factor`. This module ports the NumPy "winning recipe" from
//! `results/MATCHED_ARCH_DEEP_FINDINGS.md` — a **directional graded error**
//! delivered through **per-neuron fixed-random feedback (DFA)** — onto the
//! identical dense-LIF forward used by [`crate::matched_local_baseline`].
//!
//! **MUST NEVER BE THE PRODUCTION LEARNER** (v7 / v8 rule).
//!
//! ## Contrast
//!
//! Held identical with the protocol-v4 matched-arch control: forward graph,
//! width, encoding, epochs, splits, seed lineage, LIF constants.
//!
//! - [`MatchedBroadcastErr`] — supervised error `-(p−y)` as a **single broadcast
//!   scalar** (still local; no weight transport). Solves coincidence; fails XOR.
//! - [`MatchedDfa`] — same graded error × **fixed random feedback** per hidden
//!   unit (feedback alignment). Solves coincidence *and* nonlinear XOR in the
//!   NumPy preview. No backward graph through `wout`; feedback is frozen.
//!
//! Training uses minibatch averaging (default 20), matching the NumPy preview;
//! credit structure (broadcast vs DFA) is identical to the online form.
//! Recurrent weights stay at shared init (`PLASTIC_REC=False` in the preview).
//!
//! The gradient ceiling remains [`crate::matched_local_baseline::MatchedGradient`].

#![allow(clippy::needless_range_loop)]

use binn_core::Rng;
use binn_engine::THETA_REST;

use crate::matched_local_baseline::{ForwardCache, MatchedArch};
use crate::{GradientExample, GradientReferenceReport, REFERENCE_SEQUENCE_LEN};

const N_IN: usize = 2;
const T: usize = REFERENCE_SEQUENCE_LEN;
/// Minibatch size matching `scripts/matched_arch_experiments.py` / deep preview.
const DEFAULT_BATCH: usize = 20;

/// Stable label for the broadcast graded-error arm.
pub const MATCHED_BROADCAST_ERR_LABEL: &str = "MATCHED_ARCH_BROADCAST_GRADED_ERROR";
/// Stable label for the DFA graded-error arm.
pub const MATCHED_DFA_LABEL: &str = "MATCHED_ARCH_DFA_GRADED_ERROR";

pub use crate::matched_local_baseline::DEFAULT_MATCHED_BETA as DEFAULT_DFA_MATCHED_BETA;

/// Broadcast supervised-error arm on the shared matched forward.
#[derive(Clone, Debug)]
pub struct MatchedBroadcastErr {
    pub(crate) arch: MatchedArch,
    eta: f32,
    lambda: f32,
    rng: Rng,
}

impl MatchedBroadcastErr {
    /// New broadcast graded-error arm sized to `hidden`.
    pub fn new(hidden: usize, eta: f32, lambda: f32, beta: f32, seed: u64) -> Self {
        Self {
            arch: MatchedArch::feedforward(hidden, beta, seed),
            eta,
            lambda,
            rng: Rng::new(seed ^ 0xBC05_7012_0000_00F1),
        }
    }

    /// Train with broadcast graded error and evaluate.
    pub fn train_and_evaluate(
        &mut self,
        epochs: usize,
        train: &[GradientExample],
        test: &[GradientExample],
    ) -> GradientReferenceReport {
        assert!(
            !train.is_empty(),
            "matched broadcast-err needs training data"
        );
        assert!(!test.is_empty(), "matched broadcast-err needs test data");
        train_minibatches(
            &mut self.arch,
            &mut self.rng,
            epochs,
            train,
            self.eta,
            self.lambda,
            None,
        );
        let (accuracy, loss) = self.arch.evaluate(test);
        GradientReferenceReport {
            label: MATCHED_BROADCAST_ERR_LABEL,
            accuracy,
            loss,
        }
    }
}

/// DFA graded-error arm: fixed-random per-neuron feedback × supervised error.
#[derive(Clone, Debug)]
pub struct MatchedDfa {
    pub(crate) arch: MatchedArch,
    eta: f32,
    lambda: f32,
    /// Frozen per-hidden feedback in [-1, 1] (NumPy DFA preview).
    feedback: Vec<f32>,
    rng: Rng,
}

impl MatchedDfa {
    /// New DFA arm. Feedback matrix is frozen at construction (`seed`-derived).
    pub fn new(hidden: usize, eta: f32, lambda: f32, beta: f32, seed: u64) -> Self {
        let mut frng = Rng::new(seed ^ 0x00DF_A0C1_ED17);
        let feedback: Vec<f32> = (0..hidden).map(|_| frng.next_f32() * 2.0 - 1.0).collect();
        Self {
            arch: MatchedArch::feedforward(hidden, beta, seed),
            eta,
            lambda,
            feedback,
            rng: Rng::new(seed ^ 0xDFA0_7012_0000_00F1),
        }
    }

    /// Train with DFA graded error and evaluate.
    pub fn train_and_evaluate(
        &mut self,
        epochs: usize,
        train: &[GradientExample],
        test: &[GradientExample],
    ) -> GradientReferenceReport {
        assert!(!train.is_empty(), "matched DFA needs training data");
        assert!(!test.is_empty(), "matched DFA needs test data");
        let fb = self.feedback.clone();
        train_minibatches(
            &mut self.arch,
            &mut self.rng,
            epochs,
            train,
            self.eta,
            self.lambda,
            Some(fb.as_slice()),
        );
        let (accuracy, loss) = self.arch.evaluate(test);
        GradientReferenceReport {
            label: MATCHED_DFA_LABEL,
            accuracy,
            loss,
        }
    }

    /// Immutable feedback weights (preregistration / determinism checks).
    pub fn feedback_weights(&self) -> &[f32] {
        &self.feedback
    }
}

fn train_minibatches(
    arch: &mut MatchedArch,
    rng: &mut Rng,
    epochs: usize,
    train: &[GradientExample],
    eta: f32,
    lambda: f32,
    feedback: Option<&[f32]>,
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
            apply_graded_batch(arch, train, &order[start..end], eta, lambda, feedback);
            start = end;
        }
    }
}

fn apply_graded_batch(
    arch: &mut MatchedArch,
    train: &[GradientExample],
    indices: &[usize],
    eta: f32,
    lambda: f32,
    feedback: Option<&[f32]>,
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
        let d = p - *y;
        let teach = -d;

        for i in 0..h {
            dwout[i] += d * cache.rates[i];
        }
        dby += d;

        let e_in = eligibility_in(arch, &cache);
        let mod_vec: Vec<f32> = if let Some(fb) = feedback {
            assert_eq!(fb.len(), h, "DFA feedback width must match hidden");
            fb.iter().map(|b| *b * teach).collect()
        } else {
            vec![teach; h]
        };
        for i in 0..h {
            let m = mod_vec[i];
            for j in 0..N_IN {
                let wi = i * N_IN + j;
                dwin[wi] += m * e_in[wi];
            }
        }
    }

    for i in 0..h {
        let w = &mut arch.wout[i];
        *w -= eta * (dwout[i] / nb) + lambda * *w;
    }
    arch.by -= eta * (dby / nb);
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
    use crate::matched_local_baseline::{
        MatchedGradient, MatchedLocal, DEFAULT_MATCHED_BETA, MATCHED_LOCAL_LABEL,
    };
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
        let src = include_str!("matched_dfa_baseline.rs");
        assert!(src.contains("MUST NEVER BE THE PRODUCTION LEARNER"));
        assert!(src.contains("GC1 exempt") || src.contains("GC1-exempt"));
    }

    #[test]
    fn dfa_and_broadcast_share_matched_forward_init() {
        let dfa = MatchedDfa::new(8, 0.05, 0.0, DEFAULT_MATCHED_BETA, 0xABCD);
        let bc = MatchedBroadcastErr::new(8, 0.05, 0.0, DEFAULT_MATCHED_BETA, 0xABCD);
        let g = MatchedGradient::new_feedforward(8, 0.02, DEFAULT_MATCHED_BETA, 0xABCD);
        let mut x1 = [0.0f32; T];
        let mut x2 = [0.0f32; T];
        x1[2] = 1.0;
        x2[3] = 1.0;
        assert_eq!(
            dfa.arch.forward(&x1, &x2).logit,
            bc.arch.forward(&x1, &x2).logit
        );
        assert_eq!(
            dfa.arch.forward(&x1, &x2).logit,
            g.arch.forward(&x1, &x2).logit
        );
    }

    #[test]
    fn dfa_feedback_is_frozen_and_deterministic() {
        let a = MatchedDfa::new(16, 0.05, 0.0, DEFAULT_MATCHED_BETA, 77);
        let b = MatchedDfa::new(16, 0.05, 0.0, DEFAULT_MATCHED_BETA, 77);
        assert_eq!(a.feedback_weights(), b.feedback_weights());
        let c = MatchedDfa::new(16, 0.05, 0.0, DEFAULT_MATCHED_BETA, 78);
        assert_ne!(a.feedback_weights(), c.feedback_weights());
    }

    #[test]
    fn dfa_is_deterministic() {
        let train = gen_examples(24, 1);
        let test = gen_examples(16, 2);
        let mut a = MatchedDfa::new(16, 0.05, 0.002, DEFAULT_MATCHED_BETA, 777);
        let mut b = MatchedDfa::new(16, 0.05, 0.002, DEFAULT_MATCHED_BETA, 777);
        let ra = a.train_and_evaluate(4, &train, &test);
        let rb = b.train_and_evaluate(4, &train, &test);
        assert_eq!(ra.accuracy, rb.accuracy);
        assert_eq!(ra.loss, rb.loss);
    }

    #[test]
    fn dfa_zero_eta_is_a_noop() {
        let train = gen_examples(12, 3);
        let test = gen_examples(8, 4);
        let mut d = MatchedDfa::new(12, 0.0, 0.0, DEFAULT_MATCHED_BETA, 42);
        let before = d.arch.win.clone();
        let _ = d.train_and_evaluate(3, &train, &test);
        assert_eq!(before, d.arch.win);
    }

    #[test]
    fn dfa_learns_above_floor_on_coincidence() {
        // λ=0 matches protocol-v5 scientific; graded-error η=0.05.
        let train = gen_examples(80, 0xDFA0_C01C);
        let test = gen_examples(40, 0xDFA0_7E57);
        let mut dfa = MatchedDfa::new(128, 0.05, 0.0, DEFAULT_MATCHED_BETA, 7);
        let report = dfa.train_and_evaluate(80, &train, &test);
        assert!(
            report.accuracy >= 0.65,
            "DFA graded error should learn coincidence; got {:.3}",
            report.accuracy
        );
        assert!(report.loss.is_finite());
    }

    #[test]
    fn broadcast_err_learns_above_floor_on_coincidence() {
        // Same data lineage as the DFA learning test (broadcast is seed-sensitive
        // on tiny n; the scientific harness uses freeze_trials + n=20).
        let train = gen_examples(80, 0xDFA0_C01C);
        let test = gen_examples(40, 0xDFA0_7E57);
        let mut bc = MatchedBroadcastErr::new(128, 0.05, 0.0, DEFAULT_MATCHED_BETA, 0xDFA0_1EA5);
        let report = bc.train_and_evaluate(80, &train, &test);
        assert!(
            report.accuracy >= 0.65,
            "broadcast graded error should learn coincidence; got {:.3}",
            report.accuracy
        );
    }

    #[test]
    fn production_local_label_unchanged() {
        assert_eq!(MATCHED_LOCAL_LABEL, "MATCHED_ARCH_LOCAL_THREE_FACTOR");
        let _ = MatchedLocal::new(4, 0.1, 0.0, DEFAULT_MATCHED_BETA, 1);
    }
}
