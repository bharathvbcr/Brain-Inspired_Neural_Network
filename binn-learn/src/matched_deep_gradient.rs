//! Depth-matched surrogate-gradient ceiling for the deep matched-arch stack.
//!
//! # Why this exists
//!
//! `deep_snn_scaling` compared 2-, 3- and 4-hidden-layer learned-feedback arms
//! against a **1-hidden-layer** gradient ceiling. When the deep arms collapsed
//! from 1.00 to ~0.45, the experiment could not distinguish:
//!
//! * feedback alignment failing to scale with depth,
//! * the optimiser/learning-rate being wrong for deep stacks, or
//! * the task having no depth structure to exploit.
//!
//! A depth-matched ceiling turns depth into a one-variable contrast: identical
//! forward, identical width, identical epochs and seed lineage, **only the
//! credit pathway swapped** (transported weights vs. learned random feedback).
//!
//! # Modulator-scale parity
//!
//! The SHD suite shipped a ceiling whose hidden modulator was
//! `δ_i = Σ_k wout[k,i]·δ_k` with `wout ~ U[−1,1]·0.2/√h`, while the treatment
//! used `B ~ U[−1,1]`. At `h = 128` that is a **56×** difference in effective
//! hidden step size at the same learning rate, which is the entire reason the
//! "ceiling" scored below the arm it was supposed to bound.
//!
//! This module therefore (a) defaults to RMS-normalising the transported error
//! per layer, so the learning rate alone sets step size, and (b) records the
//! realised modulator RMS per layer via [`ModulatorScale`] so any residual
//! asymmetry is visible in the report instead of silently inverting the result.

#![allow(clippy::needless_range_loop)]

use binn_core::Rng;
use binn_engine::{DEFAULT_TAU_M, THETA_REST, V_RESET};

pub use crate::bptt_baseline::{GradientExample, GradientReferenceReport, REFERENCE_SEQUENCE_LEN};

const N_IN: usize = 2;
const T: usize = REFERENCE_SEQUENCE_LEN;

/// Stable label for the depth-matched gradient ceiling.
pub const MATCHED_DEEP_GRADIENT_LABEL: &str = "MATCHED_ARCH_DEEP_GRADIENT_CEILING";

/// Running RMS of a credit modulator, for cross-arm scale comparison.
///
/// Report this for every arm. If two arms differ by an order of magnitude, the
/// comparison is measuring learning rate, not credit-assignment quality.
#[derive(Clone, Copy, Debug, Default)]
pub struct ModulatorScale {
    sum_sq: f64,
    n: u64,
}

impl ModulatorScale {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn observe(&mut self, values: &[f32]) {
        for &v in values {
            if v.is_finite() {
                self.sum_sq += (v as f64) * (v as f64);
                self.n += 1;
            }
        }
    }

    pub fn rms(&self) -> f32 {
        if self.n == 0 {
            return 0.0;
        }
        (self.sum_sq / self.n as f64).sqrt() as f32
    }

    pub fn n_observed(&self) -> u64 {
        self.n
    }

    /// Fold another accumulator in, e.g. to pool across seeds.
    pub fn merge(&mut self, other: &ModulatorScale) {
        self.sum_sq += other.sum_sq;
        self.n += other.n;
    }

    /// Ratio of the larger RMS to the smaller. `1.0` is perfect parity.
    pub fn ratio(a: &ModulatorScale, b: &ModulatorScale) -> f32 {
        let (x, y) = (a.rms(), b.rms());
        let (hi, lo) = if x >= y { (x, y) } else { (y, x) };
        if lo <= f32::EPSILON {
            return f32::INFINITY;
        }
        hi / lo
    }

    /// Whether two arms are within `tolerance`× of each other.
    ///
    /// A comparison outside this band is a harness defect, not a result.
    pub fn parity_within(a: &ModulatorScale, b: &ModulatorScale, tolerance: f32) -> bool {
        Self::ratio(a, b) <= tolerance
    }
}

/// RMS-normalise a vector in place; returns the pre-normalisation RMS.
fn rms_normalise(v: &mut [f32]) -> f32 {
    if v.is_empty() {
        return 0.0;
    }
    let ss: f32 = v.iter().map(|x| x * x).sum();
    let rms = (ss / v.len() as f32).sqrt();
    if rms > 1e-12 {
        for x in v.iter_mut() {
            *x /= rms;
        }
    }
    rms
}

/// Depth-matched surrogate-gradient ceiling, 1..=N hidden layers.
///
/// The forward is byte-identical in structure to the learned-feedback deep arms
/// in [`crate::matched_rl_baseline`]: same init scales, same `α = exp(−1/τ_m)`,
/// same hard reset, same rate-coded inter-layer signal, same rate readout.
#[derive(Clone, Debug)]
pub struct MatchedDeepGradient {
    /// Hidden widths, outermost first. `len()` is the depth.
    layers: Vec<usize>,
    beta: f32,
    alpha: f32,
    lr: f32,
    lambda: f32,
    /// `layers[0] × N_IN`
    w_in: Vec<f32>,
    /// `w_hh[l]` is `layers[l + 1] × layers[l]`
    w_hh: Vec<Vec<f32>>,
    /// `layers[last]`
    w_out: Vec<f32>,
    by: f32,
    /// Whether transported error is RMS-normalised per layer.
    normalise_transport: bool,
    /// Realised RMS of the modulator reaching the *input* layer.
    input_modulator_scale: ModulatorScale,
}

impl MatchedDeepGradient {
    /// Construct a depth-matched ceiling.
    ///
    /// `layers` must be non-empty. Init scales mirror
    /// `MatchedRlDeepLearnedFb::new` exactly so the two arms start from the same
    /// forward distribution.
    ///
    /// # Panics
    ///
    /// Panics if `layers` is empty or contains a zero width.
    pub fn new(layers: &[usize], lr: f32, lambda: f32, beta: f32, seed: u64) -> Self {
        assert!(!layers.is_empty(), "deep gradient ceiling needs >= 1 layer");
        assert!(
            layers.iter().all(|&w| w >= 1),
            "every hidden layer needs >= 1 unit"
        );
        assert!(beta > 0.0, "surrogate beta must be positive");

        let mut rng = Rng::new(seed ^ 0x007C_D001_00F3_u64);
        let in_scale = 0.5f32;
        let out_scale = 0.2f32;

        let w_in: Vec<f32> = (0..layers[0] * N_IN)
            .map(|_| (rng.next_f32() * 2.0 - 1.0) * in_scale)
            .collect();

        let mut w_hh = Vec::with_capacity(layers.len().saturating_sub(1));
        for l in 0..layers.len().saturating_sub(1) {
            let h_scale = 0.3f32 / (layers[l] as f32).sqrt();
            let w: Vec<f32> = (0..layers[l + 1] * layers[l])
                .map(|_| (rng.next_f32() * 2.0 - 1.0) * h_scale)
                .collect();
            w_hh.push(w);
        }

        let last = *layers.last().expect("non-empty");
        let w_out: Vec<f32> = (0..last)
            .map(|_| (rng.next_f32() * 2.0 - 1.0) * out_scale)
            .collect();

        Self {
            layers: layers.to_vec(),
            beta,
            alpha: (-1.0f32 / DEFAULT_TAU_M).exp(),
            lr,
            lambda,
            w_in,
            w_hh,
            w_out,
            by: 0.0,
            normalise_transport: true,
            input_modulator_scale: ModulatorScale::new(),
        }
    }

    /// Disable per-layer RMS normalisation of transported error.
    ///
    /// Use only to *demonstrate* the scale pathology; do not report a ceiling
    /// built this way without also reporting [`Self::input_modulator_scale`].
    pub fn with_raw_transport(mut self) -> Self {
        self.normalise_transport = false;
        self
    }

    pub fn depth(&self) -> usize {
        self.layers.len()
    }

    pub fn input_modulator_scale(&self) -> ModulatorScale {
        self.input_modulator_scale
    }

    /// Forward pass. Returns per-layer spike counts, eligibility traces and the
    /// readout logit.
    #[allow(clippy::type_complexity)]
    fn forward(
        &self,
        x1: &[f32; T],
        x2: &[f32; T],
    ) -> (Vec<Vec<f32>>, Vec<f32>, Vec<Vec<f32>>, f32) {
        let depth = self.layers.len();
        let mut u: Vec<Vec<f32>> = self.layers.iter().map(|&n| vec![0.0f32; n]).collect();
        let mut s: Vec<Vec<f32>> = self.layers.iter().map(|&n| vec![0.0f32; n]).collect();
        let mut e_in = vec![0.0f32; self.layers[0] * N_IN];
        let mut e_hh: Vec<Vec<f32>> = (0..depth.saturating_sub(1))
            .map(|l| vec![0.0f32; self.layers[l + 1] * self.layers[l]])
            .collect();

        for t in 0..T {
            let in_val = [x1[t], x2[t]];

            // Layer 0 is driven by the raw input.
            for i in 0..self.layers[0] {
                let mut drive = 0.0f32;
                for c in 0..N_IN {
                    drive += self.w_in[i * N_IN + c] * in_val[c];
                }
                u[0][i] = self.alpha * u[0][i] + drive;
                if u[0][i] >= THETA_REST {
                    u[0][i] = V_RESET;
                    s[0][i] += 1.0;
                }
                let surr = 1.0 / (1.0 + self.beta * (u[0][i] - THETA_REST).abs()).powi(2);
                for c in 0..N_IN {
                    let idx = i * N_IN + c;
                    e_in[idx] = self.alpha * e_in[idx] + surr * in_val[c];
                }
            }

            // Deeper layers are driven by the running rate of the layer below,
            // matching `MatchedRlDeepLearnedFb` exactly.
            for l in 1..depth {
                let n_prev = self.layers[l - 1];
                for j in 0..self.layers[l] {
                    let mut drive = 0.0f32;
                    for i in 0..n_prev {
                        drive += self.w_hh[l - 1][j * n_prev + i] * (s[l - 1][i] / T as f32);
                    }
                    u[l][j] = self.alpha * u[l][j] + drive;
                    if u[l][j] >= THETA_REST {
                        u[l][j] = V_RESET;
                        s[l][j] += 1.0;
                    }
                    let surr = 1.0 / (1.0 + self.beta * (u[l][j] - THETA_REST).abs()).powi(2);
                    for i in 0..n_prev {
                        let idx = j * n_prev + i;
                        e_hh[l - 1][idx] =
                            self.alpha * e_hh[l - 1][idx] + surr * (s[l - 1][i] / T as f32);
                    }
                }
            }
        }

        let last = self.layers.len() - 1;
        let mut logit = self.by;
        for j in 0..self.layers[last] {
            logit += self.w_out[j] * (s[last][j] / T as f32);
        }

        (s, e_in, e_hh, logit)
    }

    fn step(&mut self, x1: &[f32; T], x2: &[f32; T], label: f32) {
        let depth = self.layers.len();
        let (s, e_in, e_hh, logit) = self.forward(x1, x2);
        let p = 1.0 / (1.0 + (-logit).exp());
        // dL/dlogit for BCE, ascending: (y − p).
        let delta_out = label - p;

        // Transported error, deepest layer first.
        let last = depth - 1;
        let mut deltas: Vec<Vec<f32>> = self.layers.iter().map(|&n| vec![0.0f32; n]).collect();
        for j in 0..self.layers[last] {
            deltas[last][j] = self.w_out[j] * delta_out;
        }
        for l in (0..last).rev() {
            let n_this = self.layers[l];
            let n_next = self.layers[l + 1];
            for i in 0..n_this {
                let mut acc = 0.0f32;
                for j in 0..n_next {
                    acc += self.w_hh[l][j * n_this + i] * deltas[l + 1][j];
                }
                deltas[l][i] = acc;
            }
        }

        // Scale parity: normalise each layer's modulator so `lr` alone sets the
        // step size. Without this, deep stacks are silently crippled by the
        // shrinking product of small init weights.
        if self.normalise_transport {
            for d in deltas.iter_mut() {
                rms_normalise(d);
                // Re-apply the readout error magnitude so the sign and the
                // strength of the outcome still modulate learning.
                for v in d.iter_mut() {
                    *v *= delta_out;
                }
            }
        }
        self.input_modulator_scale.observe(&deltas[0]);

        // Readout.
        for j in 0..self.layers[last] {
            let rate = s[last][j] / T as f32;
            self.w_out[j] += self.lr * delta_out * rate - self.lambda * self.w_out[j];
        }
        self.by += self.lr * delta_out;

        // Hidden-to-hidden.
        for l in 0..last {
            let n_this = self.layers[l];
            for j in 0..self.layers[l + 1] {
                let g = deltas[l + 1][j];
                for i in 0..n_this {
                    let idx = j * n_this + i;
                    self.w_hh[l][idx] +=
                        self.lr * g * e_hh[l][idx] - self.lambda * self.w_hh[l][idx];
                }
            }
        }

        // Input weights.
        for i in 0..self.layers[0] {
            let g = deltas[0][i];
            for c in 0..N_IN {
                let idx = i * N_IN + c;
                self.w_in[idx] += self.lr * g * e_in[idx] - self.lambda * self.w_in[idx];
            }
        }
    }

    /// Train by transported surrogate gradient and evaluate on held-out data.
    ///
    /// # Panics
    ///
    /// Panics if `train` or `test` is empty.
    pub fn train_and_evaluate(
        &mut self,
        epochs: usize,
        train: &[GradientExample],
        test: &[GradientExample],
    ) -> GradientReferenceReport {
        assert!(
            !train.is_empty(),
            "deep gradient ceiling needs training data"
        );
        assert!(!test.is_empty(), "deep gradient ceiling needs test data");

        for _ in 0..epochs {
            for (x1, x2, label) in train {
                self.step(x1, x2, *label);
            }
        }

        let mut correct = 0usize;
        for (x1, x2, label) in test {
            let (_, _, _, logit) = self.forward(x1, x2);
            let pred = if logit >= 0.0 { 1.0f32 } else { 0.0 };
            if (pred - *label).abs() < 0.5 {
                correct += 1;
            }
        }

        GradientReferenceReport {
            label: MATCHED_DEEP_GRADIENT_LABEL,
            accuracy: correct as f32 / test.len() as f32,
            loss: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn toy_data(n: usize) -> Vec<GradientExample> {
        // Separable: class 1 has both channels peaking at t = 2.
        (0..n)
            .map(|i| {
                let mut x1 = [0.0f32; T];
                let mut x2 = [0.0f32; T];
                let positive = i % 2 == 0;
                x1[2] = 1.0;
                x2[if positive { 2 } else { T - 1 }] = 1.0;
                (x1, x2, if positive { 1.0 } else { 0.0 })
            })
            .collect()
    }

    #[test]
    fn depth_is_configurable() {
        for depth in 1..=4 {
            let layers = vec![8usize; depth];
            let g = MatchedDeepGradient::new(&layers, 0.05, 0.0, 5.0, 7);
            assert_eq!(g.depth(), depth);
        }
    }

    #[test]
    /// **Mechanical completion only — this is not evidence that the ceiling
    /// learns.**
    ///
    /// The assertions below (`is_finite`, in `[0, 1]`) are satisfied by a
    /// constant predictor at chance, and that is exactly what this ceiling turned
    /// out to be: `deep-snn-scaling` v134 measured it at 0.4880 / 0.5000 / 0.5000
    /// / 0.5000 on a two-class task, across 20 seeds, on splits the treatment
    /// solves at 1.0000 in the same process.
    /// See `RESULT_2026-08-20_DEEP_SNN_V134_CEILING_IS_AT_CHANCE.md`.
    ///
    /// The test is deliberately **not** strengthened to assert learning, because
    /// it would fail and this module has no fix yet. It is left as the
    /// smoke test it always was, with its limits stated so that a green tick here
    /// is never read as a working reference. `guards::CeilingHealth` is what
    /// prevents that reading downstream; `shared_bptt` is the validated
    /// replacement for new work.
    fn trains_at_every_depth_without_panicking() {
        let train = toy_data(40);
        let test = toy_data(20);
        for depth in 1..=4 {
            let layers = vec![16usize; depth];
            let mut g = MatchedDeepGradient::new(&layers, 0.05, 0.0, 5.0, 11);
            let r = g.train_and_evaluate(5, &train, &test);
            assert!(r.accuracy.is_finite());
            assert!((0.0..=1.0).contains(&r.accuracy));
        }
    }

    /// The exact SHD pathology, reproduced and detected.
    ///
    /// Raw transport through small init weights shrinks the input-layer
    /// modulator by orders of magnitude as depth grows; normalised transport
    /// does not. `ModulatorScale` makes the difference measurable.
    #[test]
    fn raw_transport_collapses_deep_modulator_scale() {
        let train = toy_data(40);
        let test = toy_data(20);
        let layers = vec![32usize; 4];

        let mut raw = MatchedDeepGradient::new(&layers, 0.05, 0.0, 5.0, 3).with_raw_transport();
        raw.train_and_evaluate(3, &train, &test);

        let mut normed = MatchedDeepGradient::new(&layers, 0.05, 0.0, 5.0, 3);
        normed.train_and_evaluate(3, &train, &test);

        let ratio = ModulatorScale::ratio(
            &normed.input_modulator_scale(),
            &raw.input_modulator_scale(),
        );
        assert!(
            ratio > 2.0,
            "expected raw transport to attenuate the deep modulator; ratio = {ratio}"
        );
        assert!(
            !ModulatorScale::parity_within(
                &normed.input_modulator_scale(),
                &raw.input_modulator_scale(),
                2.0
            ),
            "parity check must reject the attenuated arm"
        );
    }

    #[test]
    fn modulator_scale_arithmetic() {
        let mut a = ModulatorScale::new();
        a.observe(&[3.0, 4.0]); // rms = sqrt(12.5) ≈ 3.5355
        assert!((a.rms() - 3.5355).abs() < 1e-3);
        let mut b = ModulatorScale::new();
        b.observe(&[7.0710, 7.0710]); // 2x
        assert!((ModulatorScale::ratio(&a, &b) - 2.0).abs() < 0.01);
        assert!(ModulatorScale::parity_within(&a, &b, 2.5));
        assert!(!ModulatorScale::parity_within(&a, &b, 1.5));
        // Empty accumulator -> infinite ratio rather than a divide-by-zero.
        let empty = ModulatorScale::new();
        assert!(ModulatorScale::ratio(&a, &empty).is_infinite());
    }

    #[test]
    fn rms_normalise_is_scale_free() {
        let mut v = vec![2.0f32, -2.0, 2.0, -2.0];
        let pre = rms_normalise(&mut v);
        assert!((pre - 2.0).abs() < 1e-6);
        let post: f32 = (v.iter().map(|x| x * x).sum::<f32>() / v.len() as f32).sqrt();
        assert!((post - 1.0).abs() < 1e-6);
        // All-zero vector must not produce NaN.
        let mut z = vec![0.0f32; 4];
        assert_eq!(rms_normalise(&mut z), 0.0);
        assert!(z.iter().all(|x| x.is_finite()));
    }

    // ---- characterization of a known defect, 2026-08-22 --------------------
    //
    // `deep-snn-scaling` v134/v135 measured this ceiling at chance on
    // `CoincidenceTask` at every depth. These tests localise that and **pin the
    // broken behaviour so it cannot silently change**. They assert the defect,
    // not the fix: there is no fix yet, and a green suite must not imply one.
    //
    // If someone repairs `MatchedDeepGradient`, these tests fail. That is the
    // intended signal — the repair must be registered and the record updated,
    // not slipped in under a passing build.
    //
    // See `FINDING_2026-08-22_MATCHED_DEEP_GRADIENT_COLLAPSES_TO_SILENCE.md`.

    /// It never learns — on its own separable fixture, at any depth.
    #[test]
    fn defect_the_deep_ceiling_never_learns_its_own_fixture() {
        let train = toy_data(40);
        let test = toy_data(20);
        for depth in 1..=4 {
            let layers = vec![8usize; depth];
            let mut g = MatchedDeepGradient::new(&layers, 0.05, 0.0, 5.0, 7);
            let acc = g.train_and_evaluate(200, &train, &test).accuracy;
            assert!(
                (acc - 0.5).abs() < 1e-6,
                "depth {depth} scored {acc} - if this ceiling now learns, the \
                 defect is fixed and the record must be updated"
            );
        }
    }

    /// The plain reference solves the same fixture perfectly, with the same
    /// learning rate, surrogate width and seed. So the defect is **this
    /// implementation**, not the task, not depth, and not the hyperparameters.
    #[test]
    fn defect_is_localised_the_plain_reference_solves_the_same_fixture() {
        use crate::matched_local_baseline::MatchedGradient;
        let train = toy_data(40);
        let test = toy_data(20);
        let acc = MatchedGradient::new(8, 0.05, 5.0, 7)
            .train_and_evaluate(200, &train, &test)
            .accuracy;
        assert!(
            acc > 0.99,
            "the plain reference scored {acc}, expected ~1.0"
        );
    }

    /// The mechanism: **training drives the network silent.** With enough input
    /// drive the layer spikes at initialisation, and after training it does not
    /// spike at all — so `rate` is zero, the readout freezes, and the logit is
    /// just the bias. Both classes then produce the *same* logit, which is a
    /// constant predictor by construction.
    #[test]
    fn defect_training_collapses_activity_to_zero() {
        let boosted: Vec<GradientExample> = toy_data(4)
            .iter()
            .map(|(a, b, l)| {
                let (mut x1, mut x2) = (*a, *b);
                for v in x1.iter_mut() {
                    *v *= 8.0;
                }
                for v in x2.iter_mut() {
                    *v *= 8.0;
                }
                (x1, x2, *l)
            })
            .collect();

        let mut g = MatchedDeepGradient::new(&[8], 0.05, 0.0, 5.0, 7);
        let (s_init, _, _, _) = g.forward(&boosted[0].0, &boosted[0].1);
        let spikes_before: f32 = s_init[0].iter().sum();
        assert!(
            spikes_before > 0.0,
            "the boosted fixture must spike at init"
        );

        g.train_and_evaluate(200, &boosted, &boosted);

        let mut logits = Vec::new();
        for (x1, x2, _) in &boosted {
            let (s, _, _, logit) = g.forward(x1, x2);
            assert_eq!(
                s[0].iter().sum::<f32>(),
                0.0,
                "activity survived training - the collapse mechanism has changed"
            );
            logits.push(logit);
        }
        assert!(
            logits.windows(2).all(|w| w[0].to_bits() == w[1].to_bits()),
            "logits differ across classes: {logits:?} - it is no longer a \
             constant predictor and the record must be updated"
        );
    }
}
