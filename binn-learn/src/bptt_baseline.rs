//! Labeled surrogate-gradient / BPTT learner used **only** as a gradient reference.
//!
//! **GC1 exempt.** Do not call from production learning paths. Production code must
//! use the online three-factor rule in `three_factor`. This module exists solely for
//! Gate G2 comparisons (local-assembly vs gradient reference vs dense-local).
//!
//! **MUST NEVER BE THE PRODUCTION LEARNER** (v7 / v8 rule). Hand-rolled surrogate
//! BPTT lives here; no torch/tch/candle.

#![allow(clippy::needless_range_loop)]

use binn_core::Rng;

/// Stable label for C1 / G2 reporting.
pub const GRADIENT_REFERENCE_LABEL: &str = "BPTT_GRADIENT_REFERENCE";

/// Sequence length for the coincidence temporal task.
pub const REFERENCE_SEQUENCE_LEN: usize = 8;
const T: usize = REFERENCE_SEQUENCE_LEN;
/// Hidden units in the tiny recurrent net.
const H: usize = 4;

/// Labeled BPTT gradient reference (surrogate-gradient, hand-rolled).
#[derive(Clone, Debug)]
pub struct BpttBaseline {
    /// Learning rate.
    pub lr: f32,
    wx: Vec<f32>,
    wh: Vec<f32>,
    wy: Vec<f32>,
    by: f32,
}

/// Report produced by [`BpttBaseline::train_coincidence`].
#[derive(Clone, Debug, PartialEq)]
pub struct GradientReferenceReport {
    /// Always [`GRADIENT_REFERENCE_LABEL`].
    pub label: &'static str,
    /// Hold-out accuracy in `[0, 1]`.
    pub accuracy: f32,
    /// Mean binary cross-entropy on the hold-out set.
    pub loss: f32,
}

/// One fixed-length temporal example accepted by the gradient reference.
pub type GradientExample = ([f32; T], [f32; T], f32);

impl BpttBaseline {
    /// Fresh baseline with small deterministic weights from `seed`.
    pub fn new(lr: f32, seed: u64) -> Self {
        let mut rng = Rng::new(seed);
        let scale = 0.3f32;
        let wx: Vec<f32> = (0..H * 2)
            .map(|_| (rng.next_f32() * 2.0 - 1.0) * scale)
            .collect();
        let wh: Vec<f32> = (0..H * H)
            .map(|_| (rng.next_f32() * 2.0 - 1.0) * scale)
            .collect();
        let wy: Vec<f32> = (0..H)
            .map(|_| (rng.next_f32() * 2.0 - 1.0) * scale)
            .collect();
        Self {
            lr,
            wx,
            wh,
            wy,
            by: 0.0,
        }
    }

    /// Train on the coincidence/temporal task; return a labeled reference number.
    pub fn train_coincidence(&mut self, epochs: usize, seed: u64) -> GradientReferenceReport {
        let mut rng = Rng::new(seed ^ 0xB177_0000_00B7);
        let train_n = 128usize;
        let test_n = 64usize;
        let train = gen_dataset(train_n, &mut rng);
        let test = gen_dataset(test_n, &mut Rng::new(seed ^ 0x7E57));

        self.train_and_evaluate(epochs, &train, &test)
    }

    /// Train and evaluate on caller-supplied splits.
    ///
    /// C1 uses this entry point so the local, dense, and gradient conditions
    /// receive exactly the same per-seed examples. This method remains a
    /// GC1-exempt experimental reference and is never used by production
    /// learning paths.
    pub fn train_and_evaluate(
        &mut self,
        epochs: usize,
        train: &[GradientExample],
        test: &[GradientExample],
    ) -> GradientReferenceReport {
        assert!(!train.is_empty(), "gradient reference needs training data");
        assert!(!test.is_empty(), "gradient reference needs test data");
        for _ in 0..epochs {
            for (x1, x2, y) in train {
                let (logit, cache) = self.forward(x1, x2);
                let p = sigmoid(logit);
                let dlogit = p - y;
                let grads = self.backward(&cache, dlogit);
                self.apply_grads(&grads);
            }
        }

        let mut correct = 0usize;
        let mut loss_sum = 0.0f32;
        for (x1, x2, y) in test {
            let (logit, _) = self.forward(x1, x2);
            let p = sigmoid(logit);
            loss_sum += bce(p, *y);
            let pred = if p >= 0.5 { 1.0 } else { 0.0 };
            if (pred - y).abs() < 0.5 {
                correct += 1;
            }
        }
        GradientReferenceReport {
            label: GRADIENT_REFERENCE_LABEL,
            accuracy: correct as f32 / test.len() as f32,
            loss: loss_sum / test.len() as f32,
        }
    }

    fn forward(&self, x1: &[f32; T], x2: &[f32; T]) -> (f32, ForwardCache) {
        let mut h = [0.0f32; H];
        let mut hs = [[0.0f32; H]; T];
        let mut preacts = [[0.0f32; H]; T];
        for t in 0..T {
            let x = [x1[t], x2[t]];
            let h_prev = h;
            let mut a = [0.0f32; H];
            let mut h_next = [0.0f32; H];
            for i in 0..H {
                let mut s = 0.0f32;
                s += matmul_row(&self.wx[i * 2..(i + 1) * 2], &x);
                s += matmul_row(&self.wh[i * H..(i + 1) * H], &h_prev);
                a[i] = s;
                h_next[i] = a[i].tanh();
            }
            h = h_next;
            preacts[t] = a;
            hs[t] = h;
        }
        let logit = matmul_row(&self.wy, &h) + self.by;
        (
            logit,
            ForwardCache {
                x1: *x1,
                x2: *x2,
                hs,
                preacts,
            },
        )
    }

    /// Reverse-mode through time (GC1-exempt `backward` symbol).
    fn backward(&self, cache: &ForwardCache, dlogit: f32) -> Grads {
        let mut dwx = vec![0.0f32; H * 2];
        let mut dwh = vec![0.0f32; H * H];
        let mut dwy = vec![0.0f32; H];
        let dby = dlogit;

        let h_last = cache.hs[T - 1];
        for i in 0..H {
            dwy[i] = dlogit * h_last[i];
        }

        let mut dh = [0.0f32; H];
        for i in 0..H {
            dh[i] = dlogit * self.wy[i];
        }

        for t in (0..T).rev() {
            let h_prev = if t == 0 { [0.0f32; H] } else { cache.hs[t - 1] };
            let x = [cache.x1[t], cache.x2[t]];
            let mut da = [0.0f32; H];
            for i in 0..H {
                let sech2 = 1.0 - cache.hs[t][i] * cache.hs[t][i];
                da[i] = dh[i] * sech2;
            }
            for i in 0..H {
                dwx[i * 2] += da[i] * x[0];
                dwx[i * 2 + 1] += da[i] * x[1];
                for j in 0..H {
                    dwh[i * H + j] += da[i] * h_prev[j];
                }
            }
            let mut dh_prev = [0.0f32; H];
            for j in 0..H {
                let mut s = 0.0f32;
                for i in 0..H {
                    s += self.wh[i * H + j] * da[i];
                }
                dh_prev[j] = s;
            }
            dh = dh_prev;
            let _ = cache.preacts[t];
        }

        Grads { dwx, dwh, dwy, dby }
    }

    fn apply_grads(&mut self, g: &Grads) {
        for (w, dw) in self.wx.iter_mut().zip(g.dwx.iter()) {
            *w -= self.lr * *dw;
        }
        for (w, dw) in self.wh.iter_mut().zip(g.dwh.iter()) {
            *w -= self.lr * *dw;
        }
        for (w, dw) in self.wy.iter_mut().zip(g.dwy.iter()) {
            *w -= self.lr * *dw;
        }
        self.by -= self.lr * g.dby;
    }
}

struct ForwardCache {
    x1: [f32; T],
    x2: [f32; T],
    hs: [[f32; H]; T],
    preacts: [[f32; H]; T],
}

struct Grads {
    dwx: Vec<f32>,
    dwh: Vec<f32>,
    dwy: Vec<f32>,
    dby: f32,
}

/// Dense row·vector product (intentionally named for GC1 exemption coverage).
#[inline]
fn matmul_row(row: &[f32], v: &[f32]) -> f32 {
    debug_assert_eq!(row.len(), v.len());
    row.iter().zip(v.iter()).map(|(a, b)| a * b).sum()
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

fn gen_dataset(n: usize, rng: &mut Rng) -> Vec<GradientExample> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_forbids_production_use() {
        let src = include_str!("bptt_baseline.rs");
        assert!(
            src.contains("MUST NEVER BE THE PRODUCTION LEARNER"),
            "bptt_baseline.rs must carry the production-ban header"
        );
        assert!(src.contains("GC1 exempt") || src.contains("GC1-exempt"));
    }

    #[test]
    fn trains_coincidence_and_reports_labeled_gradient_reference() {
        let mut bptt = BpttBaseline::new(0.05, 0xB177_00B7);
        let report = bptt.train_coincidence(80, 0xC01C_1DEA);
        assert_eq!(report.label, GRADIENT_REFERENCE_LABEL);
        assert!(
            report.accuracy >= 0.65,
            "BPTT reference should learn coincidence; accuracy={}",
            report.accuracy
        );
        assert!(report.loss.is_finite());
        assert!(report.loss >= 0.0);
    }

    #[test]
    fn analytical_gradient_matches_finite_difference() {
        let model = BpttBaseline::new(0.01, 0x00F1_A17E);
        let mut x1 = [0.0f32; T];
        let mut x2 = [0.0f32; T];
        x1[1] = 1.0;
        x2[2] = 1.0;
        let y = 1.0;
        let (logit, cache) = model.forward(&x1, &x2);
        let gradients = model.backward(&cache, sigmoid(logit) - y);
        let index = gradients
            .dwx
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.abs().total_cmp(&b.abs()))
            .map(|(index, _)| index)
            .expect("non-empty input weights");
        let analytic = gradients.dwx[index];

        let eps = 1e-2f32;
        let mut plus = model.clone();
        plus.wx[index] += eps;
        let loss_plus = bce(sigmoid(plus.forward(&x1, &x2).0), y);
        let mut minus = model.clone();
        minus.wx[index] -= eps;
        let loss_minus = bce(sigmoid(minus.forward(&x1, &x2).0), y);
        let numeric = (loss_plus - loss_minus) / (2.0 * eps);
        let scale = analytic.abs().max(numeric.abs()).max(1e-4);
        assert!(
            (analytic - numeric).abs() / scale < 2e-2,
            "gradient mismatch: analytic={analytic} numeric={numeric}"
        );
    }
}
