//! Labeled **e-prop-compatible eligibility** reference used only as a C1 / G2
//! comparison condition (local eligibility learning on the same temporal
//! examples as the gradient reference).
//!
//! **GC1 exempt** (this is a `*_baseline.rs` file). Do not call from production
//! learning paths. Production code must use the online three-factor rule in
//! `three_factor`. This module exists so C1 can disclose an eligibility-based
//! local ceiling next to the surrogate-LIF gradient reference.
//!
//! **MUST NEVER BE THE PRODUCTION LEARNER.**
//!
//! ## Model (e-prop style)
//!
//! Discrete-time leaky units with SuperSpike-style surrogate eligibility:
//!
//! ```text
//! I_i[t] = Σ_j Win[i,j] x_j[t]
//! U_i[t] = α U_i[t-1] + I_i[t]
//! S_i[t] ≈ σ(U_i[t])                 (surrogate rate for learning)
//! e_ij[t] = α·e_ij[t-1] + σ'(U_i[t]) · x_j[t]
//! ΔW ∝ −η · δ · e                    (δ = y − ˆy from rate readout)
//! ```

#![allow(clippy::needless_range_loop)]

use binn_core::Rng;
use binn_engine::DEFAULT_TAU_M;

pub use crate::bptt_baseline::{GradientExample, GradientReferenceReport, REFERENCE_SEQUENCE_LEN};

const N_IN: usize = 2;
const T: usize = REFERENCE_SEQUENCE_LEN;

/// Stable label for C1 / G2 reporting.
pub const EPROP_REFERENCE_LABEL: &str = "EPROP_ELIGIBILITY_REFERENCE";

/// Default surrogate steepness.
pub const DEFAULT_EPROP_BETA: f32 = 5.0;

/// Eligibility-based local reference (e-prop compatible).
#[derive(Clone, Debug)]
pub struct EpropReference {
    hidden: usize,
    lr: f32,
    beta: f32,
    alpha: f32,
    win: Vec<f32>,  // hidden × N_IN
    wout: Vec<f32>, // hidden
    by: f32,
}

impl EpropReference {
    /// Fresh reference with deterministic weights from `seed`.
    pub fn new(hidden: usize, lr: f32, beta: f32, seed: u64) -> Self {
        assert!(hidden >= 1, "e-prop reference needs ≥1 hidden unit");
        assert!(beta > 0.0, "surrogate beta must be positive");
        let mut rng = Rng::new(seed ^ 0xE700_0000_00F1);
        let in_scale = 0.6f32;
        let out_scale = 0.25f32;
        let win: Vec<f32> = (0..hidden * N_IN)
            .map(|_| (rng.next_f32() * 2.0 - 1.0) * in_scale)
            .collect();
        let wout: Vec<f32> = (0..hidden)
            .map(|_| (rng.next_f32() * 2.0 - 1.0) * out_scale)
            .collect();
        Self {
            hidden,
            lr,
            beta,
            alpha: (-1.0f32 / DEFAULT_TAU_M).exp(),
            win,
            wout,
            by: 0.0,
        }
    }

    /// Convenience constructor with [`DEFAULT_EPROP_BETA`].
    pub fn with_defaults(hidden: usize, lr: f32, seed: u64) -> Self {
        Self::new(hidden, lr, DEFAULT_EPROP_BETA, seed)
    }

    /// Train and evaluate on caller-supplied splits (same contract as BPTT refs).
    pub fn train_and_evaluate(
        &mut self,
        epochs: usize,
        train: &[GradientExample],
        test: &[GradientExample],
    ) -> GradientReferenceReport {
        assert!(!train.is_empty(), "e-prop reference needs training data");
        assert!(!test.is_empty(), "e-prop reference needs test data");
        for _ in 0..epochs {
            for (x1, x2, y) in train {
                self.train_one(x1, x2, *y);
            }
        }

        let mut correct = 0usize;
        let mut loss_sum = 0.0f32;
        for (x1, x2, y) in test {
            let (logit, _) = self.forward_rates(x1, x2);
            let p = sigmoid(logit);
            loss_sum += bce(p, *y);
            let pred = if p >= 0.5 { 1.0 } else { 0.0 };
            if (pred - y).abs() < 0.5 {
                correct += 1;
            }
        }
        GradientReferenceReport {
            label: EPROP_REFERENCE_LABEL,
            accuracy: correct as f32 / test.len() as f32,
            loss: loss_sum / test.len() as f32,
        }
    }

    fn train_one(&mut self, x1: &[f32; T], x2: &[f32; T], y: f32) {
        let h = self.hidden;
        let mut u = vec![0.0f32; h];
        let mut e = vec![0.0f32; h * N_IN];
        let mut rates = vec![0.0f32; h];

        for t in 0..T {
            let x = [x1[t], x2[t]];
            for i in 0..h {
                let cur = self.win[i * N_IN] * x[0] + self.win[i * N_IN + 1] * x[1];
                u[i] = self.alpha * u[i] + cur;
                let surr = surrogate(u[i], self.beta);
                let soft = soft_spike(u[i], self.beta);
                rates[i] += soft;
                for j in 0..N_IN {
                    let idx = i * N_IN + j;
                    e[idx] = self.alpha * e[idx] + surr * x[j];
                }
            }
        }

        let mut logit = self.by;
        for i in 0..h {
            logit += self.wout[i] * rates[i];
        }
        let p = sigmoid(logit);
        let delta = p - y; // learning uses −η δ e  (δ here is ˆy − y)

        for i in 0..h {
            self.wout[i] -= self.lr * delta * rates[i];
            for j in 0..N_IN {
                let idx = i * N_IN + j;
                // Chain through readout: ∂L/∂win ∝ δ · wout · e
                self.win[idx] -= self.lr * delta * self.wout[i] * e[idx];
            }
        }
        self.by -= self.lr * delta;
    }

    fn forward_rates(&self, x1: &[f32; T], x2: &[f32; T]) -> (f32, Vec<f32>) {
        let h = self.hidden;
        let mut u = vec![0.0f32; h];
        let mut rates = vec![0.0f32; h];
        for t in 0..T {
            let x = [x1[t], x2[t]];
            for i in 0..h {
                let cur = self.win[i * N_IN] * x[0] + self.win[i * N_IN + 1] * x[1];
                u[i] = self.alpha * u[i] + cur;
                rates[i] += soft_spike(u[i], self.beta);
            }
        }
        let mut logit = self.by;
        for i in 0..h {
            logit += self.wout[i] * rates[i];
        }
        (logit, rates)
    }
}

#[inline]
fn surrogate(u: f32, beta: f32) -> f32 {
    let d = 1.0 + beta * u.abs();
    1.0 / (d * d)
}

#[inline]
fn soft_spike(u: f32, beta: f32) -> f32 {
    // Smooth rate proxy in (0, 1); not a hard threshold.
    sigmoid(beta * u)
}

#[inline]
fn sigmoid(z: f32) -> f32 {
    1.0 / (1.0 + (-z).exp())
}

#[inline]
fn bce(p: f32, y: f32) -> f32 {
    let p = p.clamp(1e-6, 1.0 - 1e-6);
    -(y * p.ln() + (1.0 - y) * (1.0 - p).ln())
}

#[cfg(test)]
mod tests {
    use super::*;
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
        let src = include_str!("eprop_baseline.rs");
        assert!(src.contains("MUST NEVER BE THE PRODUCTION LEARNER"));
        assert!(src.contains("GC1 exempt") || src.contains("GC1-exempt"));
    }

    #[test]
    fn learns_coincidence_above_chance() {
        let mut model = EpropReference::with_defaults(32, 0.05, 0xE001_C01A);
        let train = gen_examples(256, 0xB177_00B7);
        let test = gen_examples(128, 0x7E57_0002);
        let report = model.train_and_evaluate(120, &train, &test);
        assert_eq!(report.label, EPROP_REFERENCE_LABEL);
        assert!(
            report.accuracy >= 0.60,
            "e-prop reference should learn coincidence; accuracy={}",
            report.accuracy
        );
    }

    #[test]
    fn deterministic_same_seed() {
        let train = gen_examples(64, 1);
        let test = gen_examples(32, 2);
        let mut a = EpropReference::with_defaults(12, 0.05, 777);
        let mut b = EpropReference::with_defaults(12, 0.05, 777);
        let ra = a.train_and_evaluate(40, &train, &test);
        let rb = b.train_and_evaluate(40, &train, &test);
        assert_eq!(ra, rb);
    }
}
