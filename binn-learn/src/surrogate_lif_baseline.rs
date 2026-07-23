//! Labeled **surrogate-gradient LIF** learner used **only** as an
//! *attainable, same-architecture* gradient reference for Gate G2.
//!
//! **GC1 exempt** (this is a `*_baseline.rs` file). Do not call from production
//! learning paths. Production code must use the online three-factor rule in
//! `three_factor`. This module exists solely so the C1 / G2 comparison has a
//! gradient reference that lives on the **same LIF substrate** as the
//! `local-assembly` condition — the only difference being the *learning rule*
//! (surrogate-gradient BPTT here vs. online three-factor there), not the model
//! family.
//!
//! **MUST NEVER BE THE PRODUCTION LEARNER** (v7 / v8 rule). Hand-rolled
//! surrogate-gradient BPTT lives here; no torch/tch/candle/burn/dfdx.
//!
//! ## Why this exists (scientific motivation)
//!
//! The original [`crate::bptt_baseline::BpttBaseline`] is a `tanh` RNN — a
//! *different model family* from the spiking substrate. The G2 `gap_closed`
//! metric `(local − dense) / (reference − dense)` is therefore a cross-family
//! comparison, so a low value conflates "sparse-assembly local learning can't do
//! this" with "spiking is simply worse than a real-valued RNN." This module
//! removes that confound: it trains the **same discrete-time LIF dynamics** the
//! engine uses (leak `α = exp(−1/τ_m)`, threshold `θ = THETA_REST`, reset toward
//! `V_RESET`) with a SuperSpike-style surrogate gradient. When this is the
//! reference, `gap_closed` answers the intended question — *how much of what
//! gradients achieve on this architecture does the local rule recover?*
//!
//! ## Model
//!
//! Recurrent LIF layer of `hidden` units driven by two input channels, read out
//! by a rate (summed-spike) linear head:
//!
//! ```text
//! I_i[t] = Σ_j Win[i,j] x_j[t] + Σ_k Wrec[i,k] S_k[t-1]
//! U_i[t] = α U_i[t-1] + I_i[t] − θ · S_i[t-1]          (soft reset)
//! S_i[t] = 1{ U_i[t] ≥ θ }                             (spike; θ = THETA_REST = 1)
//! logit  = Σ_i Wout_i · (Σ_t S_i[t]) + b               (rate readout)
//! ```
//!
//! Backward is exact BPTT except the non-differentiable spike `∂S/∂U` is replaced
//! by the SuperSpike surrogate `σ'(u) = 1 / (1 + β|u − θ|)²`.

#![allow(clippy::needless_range_loop)]

use binn_core::Rng;
use binn_engine::{DEFAULT_TAU_M, THETA_REST, V_RESET};

// Re-use the reference contract so this drops into the C1 runner exactly where
// `BpttBaseline` does (same example type, same report type).
pub use crate::bptt_baseline::{GradientExample, GradientReferenceReport, REFERENCE_SEQUENCE_LEN};

/// Fixed input-channel count for the C1 coincidence task.
const N_IN: usize = 2;
const T: usize = REFERENCE_SEQUENCE_LEN;

/// Stable label for C1 / G2 reporting (distinct from the tanh-RNN reference so
/// results notes can say which ceiling was used).
pub const SURROGATE_LIF_REFERENCE_LABEL: &str = "SURROGATE_LIF_GRADIENT_REFERENCE";

/// Default surrogate steepness `β` for `σ'(u) = 1/(1 + β|u−θ|)²`.
pub const DEFAULT_SURROGATE_BETA: f32 = 5.0;

/// Labeled surrogate-gradient LIF reference (hand-rolled BPTT on LIF dynamics).
///
/// Hidden width is a runtime parameter so the reference can be sized to match
/// the substrate's `n_hidden`, keeping the architectures comparable.
#[derive(Clone, Debug)]
pub struct SurrogateLifReference {
    hidden: usize,
    lr: f32,
    beta: f32,
    /// Membrane leak factor `α = exp(−1/τ_m)`, τ_m shared with the engine.
    alpha: f32,
    win: Vec<f32>,  // hidden × N_IN
    wrec: Vec<f32>, // hidden × hidden
    wout: Vec<f32>, // hidden
    by: f32,
}

struct ForwardCache {
    x: [[f32; N_IN]; T],
    u: Vec<[f32; T]>, // per hidden unit, membrane over time
    s: Vec<[f32; T]>, // per hidden unit, spikes over time
    rates: Vec<f32>,  // Σ_t S_i[t]
    logit: f32,
}

struct Grads {
    dwin: Vec<f32>,
    dwrec: Vec<f32>,
    dwout: Vec<f32>,
    dby: f32,
}

impl SurrogateLifReference {
    /// Fresh reference with small deterministic weights from `seed`.
    ///
    /// `hidden` should mirror the substrate's `n_hidden`; `lr` is the SGD step;
    /// `beta` is the surrogate steepness (use [`DEFAULT_SURROGATE_BETA`]).
    pub fn new(hidden: usize, lr: f32, beta: f32, seed: u64) -> Self {
        assert!(hidden >= 1, "surrogate LIF reference needs ≥1 hidden unit");
        assert!(beta > 0.0, "surrogate beta must be positive");
        let mut rng = Rng::new(seed ^ 0x5171_0000_00F1);
        // Modest input/recurrent scale keeps early membranes near threshold so
        // gradients flow; readout starts small.
        let in_scale = 0.5f32;
        let rec_scale = 0.3f32 / (hidden as f32).sqrt();
        let out_scale = 0.2f32;
        let win: Vec<f32> = (0..hidden * N_IN)
            .map(|_| (rng.next_f32() * 2.0 - 1.0) * in_scale)
            .collect();
        let wrec: Vec<f32> = (0..hidden * hidden)
            .map(|_| (rng.next_f32() * 2.0 - 1.0) * rec_scale)
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
            wrec,
            wout,
            by: 0.0,
        }
    }

    /// Convenience constructor mirroring `BpttBaseline::new`'s default steepness.
    pub fn with_defaults(hidden: usize, lr: f32, seed: u64) -> Self {
        Self::new(hidden, lr, DEFAULT_SURROGATE_BETA, seed)
    }

    /// Train and evaluate on caller-supplied splits.
    ///
    /// Same signature and return type as [`crate::bptt_baseline::BpttBaseline::train_and_evaluate`]
    /// so the C1 runner can select this reference without changing the data path.
    /// GC1-exempt experimental reference; never used by production learning.
    pub fn train_and_evaluate(
        &mut self,
        epochs: usize,
        train: &[GradientExample],
        test: &[GradientExample],
    ) -> GradientReferenceReport {
        assert!(!train.is_empty(), "surrogate reference needs training data");
        assert!(!test.is_empty(), "surrogate reference needs test data");
        for _ in 0..epochs {
            for (x1, x2, y) in train {
                let cache = self.forward(x1, x2);
                let p = sigmoid(cache.logit);
                let dlogit = p - *y;
                let grads = self.backward(&cache, dlogit);
                self.apply_grads(&grads);
            }
        }

        let mut correct = 0usize;
        let mut loss_sum = 0.0f32;
        for (x1, x2, y) in test {
            let cache = self.forward(x1, x2);
            let p = sigmoid(cache.logit);
            loss_sum += bce(p, *y);
            let pred = if p >= 0.5 { 1.0 } else { 0.0 };
            if (pred - y).abs() < 0.5 {
                correct += 1;
            }
        }
        GradientReferenceReport {
            // `GradientReferenceReport.label` is `&'static str`; report which
            // ceiling produced the number.
            label: SURROGATE_LIF_REFERENCE_LABEL,
            accuracy: correct as f32 / test.len() as f32,
            loss: loss_sum / test.len() as f32,
        }
    }

    fn forward(&self, x1: &[f32; T], x2: &[f32; T]) -> ForwardCache {
        let h = self.hidden;
        let theta = THETA_REST;
        let mut x = [[0.0f32; N_IN]; T];
        for t in 0..T {
            x[t][0] = x1[t];
            x[t][1] = x2[t];
        }
        let mut u = vec![[0.0f32; T]; h];
        let mut s = vec![[0.0f32; T]; h];

        for t in 0..T {
            for i in 0..h {
                // Input current from both channels.
                let mut cur = self.win[i * N_IN] * x[t][0] + self.win[i * N_IN + 1] * x[t][1];
                // Recurrent current from previous-step spikes.
                if t > 0 {
                    for k in 0..h {
                        cur += self.wrec[i * h + k] * s[k][t - 1];
                    }
                }
                let u_prev = if t > 0 { u[i][t - 1] } else { V_RESET };
                let s_prev = if t > 0 { s[i][t - 1] } else { 0.0 };
                // Soft reset: subtract θ on the step following a spike.
                let ui = self.alpha * u_prev + cur - theta * s_prev;
                u[i][t] = ui;
                s[i][t] = if ui >= theta { 1.0 } else { 0.0 };
            }
        }

        let mut rates = vec![0.0f32; h];
        let mut logit = self.by;
        for i in 0..h {
            let r: f32 = (0..T).map(|t| s[i][t]).sum();
            rates[i] = r;
            logit += self.wout[i] * r;
        }

        ForwardCache {
            x,
            u,
            s,
            rates,
            logit,
        }
    }

    /// Reverse-mode BPTT with the SuperSpike surrogate (GC1-exempt `backward`).
    fn backward(&self, cache: &ForwardCache, dlogit: f32) -> Grads {
        let h = self.hidden;
        let theta = THETA_REST;
        let mut dwin = vec![0.0f32; h * N_IN];
        let mut dwrec = vec![0.0f32; h * h];
        let mut dwout = vec![0.0f32; h];
        let dby = dlogit;

        // Readout grads and the per-unit constant gradient into every spike:
        // r_i = Σ_t S_i[t]  ⇒  ∂L/∂S_i[t] gets +g_r_i at every t.
        let mut g_r = vec![0.0f32; h];
        for i in 0..h {
            dwout[i] = dlogit * cache.rates[i];
            g_r[i] = dlogit * self.wout[i];
        }

        // du_next[i] = dL/dU_i[t+1]; zero beyond the horizon.
        let mut du_next = vec![0.0f32; h];
        for t in (0..T).rev() {
            let mut du = vec![0.0f32; h];
            for i in 0..h {
                // dS_i[t] = readout term
                //         + recurrent coupling Σ_m dU_m[t+1] · Wrec[m,i]
                //         − reset coupling dU_i[t+1]   (soft reset −θ·S term)
                let mut ds = g_r[i] - du_next[i];
                for m in 0..h {
                    ds += du_next[m] * self.wrec[m * h + i];
                }
                let surr = surrogate(cache.u[i][t] - theta, self.beta);
                // dU_i[t] = dS_i[t]·σ'(u) + α·dU_i[t+1]
                du[i] = ds * surr + self.alpha * du_next[i];
            }
            // Accumulate weight grads at this step.
            for i in 0..h {
                dwin[i * N_IN] += du[i] * cache.x[t][0];
                dwin[i * N_IN + 1] += du[i] * cache.x[t][1];
                if t > 0 {
                    for k in 0..h {
                        // ∂U_i[t]/∂Wrec[i,k] = S_k[t-1]
                        dwrec[i * h + k] += du[i] * cache.s[k][t - 1];
                    }
                }
            }
            du_next = du;
        }

        Grads {
            dwin,
            dwrec,
            dwout,
            dby,
        }
    }

    fn apply_grads(&mut self, g: &Grads) {
        for (w, dw) in self.win.iter_mut().zip(g.dwin.iter()) {
            *w -= self.lr * *dw;
        }
        for (w, dw) in self.wrec.iter_mut().zip(g.dwrec.iter()) {
            *w -= self.lr * *dw;
        }
        for (w, dw) in self.wout.iter_mut().zip(g.dwout.iter()) {
            *w -= self.lr * *dw;
        }
        self.by -= self.lr * g.dby;
    }
}

/// SuperSpike surrogate derivative `σ'(u) = 1 / (1 + β|u|)²`.
#[inline]
fn surrogate(u_minus_theta: f32, beta: f32) -> f32 {
    let d = 1.0 + beta * u_minus_theta.abs();
    1.0 / (d * d)
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

    /// Build the same coincidence task the C1 reference consumes: two one-hot
    /// spikes; label 1 iff their frames are within ±1.
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
        let src = include_str!("surrogate_lif_baseline.rs");
        assert!(
            src.contains("MUST NEVER BE THE PRODUCTION LEARNER"),
            "surrogate_lif_baseline.rs must carry the production-ban header"
        );
        assert!(src.contains("GC1 exempt") || src.contains("GC1-exempt"));
    }

    #[test]
    fn shares_lif_constants_with_engine() {
        // The whole point is a same-architecture reference: θ and reset must be
        // the engine's, and the leak must be exp(−1/τ_m).
        let model = SurrogateLifReference::with_defaults(8, 0.05, 0xABCD);
        assert_eq!(THETA_REST, 1.0);
        assert_eq!(V_RESET, 0.0);
        assert!((model.alpha - (-1.0f32 / DEFAULT_TAU_M).exp()).abs() < 1e-7);
    }

    #[test]
    fn analytical_gradient_matches_finite_difference() {
        let model = SurrogateLifReference::with_defaults(6, 0.01, 0x00F1_A17E);
        let mut x1 = [0.0f32; T];
        let mut x2 = [0.0f32; T];
        x1[2] = 1.0;
        x2[3] = 1.0;
        let y = 1.0;
        let cache = model.forward(&x1, &x2);
        let grads = model.backward(&cache, sigmoid(cache.logit) - y);

        // Check the readout weight gradient (fully differentiable path, so it
        // must match finite differences tightly; the surrogate only affects the
        // hidden pre-spike terms).
        let idx = 0usize;
        let analytic = grads.dwout[idx];
        let eps = 1e-2f32;
        let mut plus = model.clone();
        plus.wout[idx] += eps;
        let loss_plus = bce(sigmoid(plus.forward(&x1, &x2).logit), y);
        let mut minus = model.clone();
        minus.wout[idx] -= eps;
        let loss_minus = bce(sigmoid(minus.forward(&x1, &x2).logit), y);
        let numeric = (loss_plus - loss_minus) / (2.0 * eps);
        let scale = analytic.abs().max(numeric.abs()).max(1e-4);
        assert!(
            (analytic - numeric).abs() / scale < 2e-2,
            "readout gradient mismatch: analytic={analytic} numeric={numeric}"
        );
    }

    #[test]
    fn learns_coincidence_above_chance() {
        // Sized to the substrate's capacity-scaled hidden width.
        let mut model = SurrogateLifReference::with_defaults(32, 0.02, 0xC01C_1DEA);
        let train = gen_examples(256, 0xB177_00B7);
        let test = gen_examples(128, 0x7E57_0002);
        let report = model.train_and_evaluate(150, &train, &test);
        assert_eq!(report.label, SURROGATE_LIF_REFERENCE_LABEL);
        assert!(report.loss.is_finite() && report.loss >= 0.0);
        // A gradient reference on this architecture must clear the same 0.65
        // floor the tanh RNN reference is held to; otherwise it is not a usable
        // ceiling and the harness (not the local rule) is the problem.
        assert!(
            report.accuracy >= 0.65,
            "surrogate-LIF reference should learn coincidence; accuracy={}",
            report.accuracy
        );
    }

    #[test]
    fn deterministic_same_seed() {
        let train = gen_examples(64, 1);
        let test = gen_examples(32, 2);
        let mut a = SurrogateLifReference::with_defaults(12, 0.02, 777);
        let mut b = SurrogateLifReference::with_defaults(12, 0.02, 777);
        let ra = a.train_and_evaluate(30, &train, &test);
        let rb = b.train_and_evaluate(30, &train, &test);
        assert_eq!(ra, rb);
    }
}
