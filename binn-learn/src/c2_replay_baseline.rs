//! Capacity / replay-matched **gradient** baseline for Gate G3 / C2.
//!
//! **GC1 exempt** (`*_baseline.rs`). Must never be the production learner.
//! Stores a fixed-capacity raw-example replay buffer — disclosed and labeled
//! so it is **not** confused with the local-assembly path (which stores none).
//!
//! Hand-rolled multiclass logistic SGD (no torch/tch/candle). Used only to
//! answer: does local three-factor forget less than a capacity-matched
//! gradient learner that is allowed replay under the same memory budget?

#![allow(clippy::needless_range_loop)]

use binn_core::Rng;

/// Stable label for C2 / G3 reporting.
pub const C2_REPLAY_BASELINE_LABEL: &str = "C2_CAPACITY_REPLAY_GRADIENT_BASELINE";

/// One flat example for the replay baseline.
#[derive(Clone, Debug)]
pub struct ReplayExample {
    pub features: Vec<f32>,
    pub label: u32,
}

/// Multiclass logistic regressor with optional fixed-capacity replay.
#[derive(Clone, Debug)]
pub struct C2ReplayBaseline {
    n_in: usize,
    n_classes: usize,
    lr: f32,
    /// Weights: `n_classes × n_in`.
    w: Vec<f32>,
    b: Vec<f32>,
    /// Fixed-capacity raw replay buffer (disclosed; baseline-only).
    replay: Vec<ReplayExample>,
    replay_capacity: usize,
    replay_cursor: usize,
}

impl C2ReplayBaseline {
    /// Build a baseline with `replay_capacity` stored raw examples.
    ///
    /// Capacity matching: callers should set `n_in` / `n_classes` so
    /// `n_params ≈ n_classes * (n_in + 1)` is comparable to the local path's
    /// disclosed parameter count (or to a disclosed budget).
    pub fn new(n_in: usize, n_classes: usize, lr: f32, replay_capacity: usize, seed: u64) -> Self {
        assert!(n_in > 0 && n_classes >= 2);
        assert!(lr > 0.0);
        let mut rng = Rng::new(seed ^ 0xC2_BA5E);
        let n_w = n_classes * n_in;
        let mut w = vec![0.0; n_w];
        for wi in w.iter_mut() {
            *wi = (rng.next_f32() - 0.5) * 0.05;
        }
        Self {
            n_in,
            n_classes,
            lr,
            w,
            b: vec![0.0; n_classes],
            replay: Vec::with_capacity(replay_capacity),
            replay_capacity,
            replay_cursor: 0,
        }
    }

    /// Parameter count (`n_classes * (n_in + 1)`).
    #[inline]
    pub fn n_params(&self) -> usize {
        self.n_classes * (self.n_in + 1)
    }

    /// Disclosed replay occupancy.
    #[inline]
    pub fn replay_len(&self) -> usize {
        self.replay.len()
    }

    /// Disclosed replay capacity (raw examples stored — baseline only).
    #[inline]
    pub fn replay_capacity(&self) -> usize {
        self.replay_capacity
    }

    /// Observe one example: update online, then maybe store in replay.
    pub fn observe(&mut self, features: &[f32], label: u32) {
        assert_eq!(features.len(), self.n_in);
        self.sgd_step(features, label);
        if self.replay_capacity == 0 {
            return;
        }
        let ex = ReplayExample {
            features: features.to_vec(),
            label,
        };
        if self.replay.len() < self.replay_capacity {
            self.replay.push(ex);
        } else {
            self.replay[self.replay_cursor % self.replay_capacity] = ex;
            self.replay_cursor = self.replay_cursor.wrapping_add(1);
        }
        // Replay rehearsal under the same capacity budget.
        let n_replay = (self.replay.len() / 4).max(1).min(self.replay.len());
        let start = self.replay_cursor % self.replay.len().max(1);
        for i in 0..n_replay {
            let j = (start + i) % self.replay.len();
            let feat = self.replay[j].features.clone();
            let lab = self.replay[j].label;
            self.sgd_step(&feat, lab);
        }
    }

    /// Predict class for `features`.
    pub fn predict(&self, features: &[f32]) -> u32 {
        self.logits(features)
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i as u32)
            .unwrap_or(0)
    }

    /// Accuracy on a probe set.
    pub fn accuracy(&self, probe: &[(Vec<f32>, u32)]) -> f32 {
        if probe.is_empty() {
            return 0.0;
        }
        let ok = probe.iter().filter(|(x, y)| self.predict(x) == *y).count();
        ok as f32 / probe.len() as f32
    }

    fn logits(&self, features: &[f32]) -> Vec<f32> {
        let mut out = self.b.clone();
        for c in 0..self.n_classes {
            let row = c * self.n_in;
            let mut s = self.b[c];
            for i in 0..self.n_in {
                s += self.w[row + i] * features[i];
            }
            out[c] = s;
        }
        out
    }

    fn sgd_step(&mut self, features: &[f32], label: u32) {
        let logits = self.logits(features);
        let max_l = logits.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
        let mut exps: Vec<f32> = logits.iter().map(|l| (l - max_l).exp()).collect();
        let sum: f32 = exps.iter().sum::<f32>().max(1e-12);
        for e in &mut exps {
            *e /= sum;
        }
        let y = label as usize;
        for c in 0..self.n_classes {
            let target = if c == y { 1.0 } else { 0.0 };
            let err = exps[c] - target;
            let row = c * self.n_in;
            for i in 0..self.n_in {
                self.w[row + i] -= self.lr * err * features[i];
            }
            self.b[c] -= self.lr * err;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replay_capacity_is_disclosed_and_bounded() {
        let mut b = C2ReplayBaseline::new(4, 4, 0.2, 8, 1);
        assert_eq!(b.replay_capacity(), 8);
        for i in 0..20 {
            let mut x = vec![0.1; 4];
            x[i % 4] = 0.9;
            b.observe(&x, (i % 4) as u32);
        }
        assert!(b.replay_len() <= 8);
        assert_eq!(b.replay_len(), 8);
    }

    #[test]
    fn learns_separable_prototypes() {
        let mut b = C2ReplayBaseline::new(3, 3, 0.35, 16, 42);
        for _ in 0..40 {
            for c in 0..3u32 {
                let mut x = vec![0.1; 3];
                x[c as usize] = 0.9;
                b.observe(&x, c);
            }
        }
        let probe: Vec<_> = (0..3u32)
            .map(|c| {
                let mut x = vec![0.1; 3];
                x[c as usize] = 0.9;
                (x, c)
            })
            .collect();
        assert!(b.accuracy(&probe) >= 0.99);
    }
}
