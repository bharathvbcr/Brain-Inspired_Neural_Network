//! Recurrent / adaptive-threshold SHD architecture with matched-scale credit arms.
//!
//! **MUST NEVER BE THE PRODUCTION LEARNER.** GC1-exempt reference module.
//!
//! # Why this module exists
//!
//! The `c1-shd-cal-*` suite reports DFA ≈ 0.234 on SHD (20 classes, chance 0.05)
//! and reads that as a statement about local credit assignment. It is not
//! necessarily one. [`crate::shd_eprop_baseline::ShdArch`] is **feed-forward with
//! no `W_rec` and a fixed threshold**, and the published local-learning
//! literature is explicit that both ingredients are required on this dataset:
//!
//! > "when using local plasticity, threshold adaptation in spiking neurons and a
//! > recurrent topology are necessary to learn spatio-temporal patterns with a
//! > rich temporal structure"
//! > — Quintana et al., *ETLP: event-based three-factor local plasticity for
//! > online learning with neuromorphic hardware*, Neuromorph. Comput. Eng. 2024
//!
//! For reference on the same dataset: ETLP (fully local, three-factor,
//! hardware-targeted) reaches **74.59%**; e-prop reaches **80.79%**; BPTT with
//! learned delays reaches **95.1%**. A 0.234 result is therefore ~3× below the
//! nearest comparable local rule, and the leading hypothesis is that the
//! *architecture*, not the locality of the rule, is the binding constraint.
//!
//! This module makes that hypothesis falsifiable by exposing the two ingredients
//! as independent ablation axes:
//!
//! | Axis | `false` | `true` |
//! |---|---|---|
//! | [`ShdAlifConfig::recurrent`] | feed-forward (reproduces the current result) | `W_rec` with zero diagonal |
//! | [`ShdAlifConfig::adaptive`] | fixed `θ = THETA_REST` | `θ_i(t) = THETA_REST + β_a · a_i(t)` |
//!
//! Running the 2×2 answers a question the existing suite cannot: **is 0.234 a
//! limit of local credit assignment, or a limit of a feed-forward fixed-threshold
//! forward model?**
//!
//! # Modulator-scale parity is enforced here by construction
//!
//! Every arm's hidden modulator is built at the same scale
//! ([`crate::shd_eprop_baseline::shd_out_scale`]) and its realised RMS is
//! recorded via [`ModulatorScale`]. This is the defect that made the original
//! e-prop ceiling score *below* its own DFA treatment (a ~56× effective
//! learning-rate deficit at `h = 128`); it must not recur here.
//!
//! # ALIF eligibility state
//!
//! Every trainable input and recurrent synapse carries the two surrogate ALIF
//! states disclosed by the protocol:
//! `ε_v ← α·ε_v + pre`,
//! `ε_a ← σ'·ε_v + (ρ − σ'·β_a)·ε_a`, and
//! `e = σ'·(ε_v − β_a·ε_a)`.
//! Fixed-threshold cells use the same implementation with `β_a = 0`, which
//! avoids changing eligibility semantics across the architecture ablation.

#![allow(clippy::needless_range_loop)]
#![allow(clippy::too_many_arguments)]

use binn_core::Rng;
use binn_engine::{DEFAULT_TAU_M, THETA_REST, V_RESET};

use crate::matched_deep_gradient::ModulatorScale;
use crate::shd_eprop_baseline::{shd_out_scale, ShdArmReport, ShdExample};

/// Stable arm labels.
pub const SHD_ALIF_DFA_LABEL: &str = "SHD_ALIF_DFA";
pub const SHD_ALIF_EPROP_LABEL: &str = "SHD_ALIF_EPROP_CEILING";
pub const SHD_ALIF_BROADCAST_LABEL: &str = "SHD_ALIF_BROADCAST_PM1";

/// Default adaptation time constant (matches `binn_engine::TAU_THETA`).
pub const DEFAULT_TAU_A: f32 = 20.0;
/// Default adaptation strength.
pub const DEFAULT_BETA_A: f32 = 0.18;

/// Which credit pathway drives the hidden layer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShdAlifRule {
    /// Fixed random feedback, scale-matched to the readout.
    Dfa,
    /// Transported readout weights (the ceiling).
    EpropCeiling,
    /// Scalar ±1 reward broadcast to every hidden unit.
    BroadcastPm1,
}

impl ShdAlifRule {
    pub const fn label(self) -> &'static str {
        match self {
            ShdAlifRule::Dfa => SHD_ALIF_DFA_LABEL,
            ShdAlifRule::EpropCeiling => SHD_ALIF_EPROP_LABEL,
            ShdAlifRule::BroadcastPm1 => SHD_ALIF_BROADCAST_LABEL,
        }
    }

    /// Whether this arm is a reference rather than a hypothesis under test.
    pub const fn is_ceiling(self) -> bool {
        matches!(self, ShdAlifRule::EpropCeiling)
    }
}

/// Ablation + training configuration.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShdAlifConfig {
    pub hidden: usize,
    pub n_classes: usize,
    pub lr: f32,
    /// Surrogate steepness `β` in `σ'(u−θ) = 1/(1 + β|u−θ|)²`.
    pub beta: f32,
    pub epochs: usize,
    /// **Ablation axis 1**: enable `W_rec`.
    pub recurrent: bool,
    /// **Ablation axis 2**: enable the adaptive threshold.
    pub adaptive: bool,
    pub tau_a: f32,
    pub beta_a: f32,
}

impl ShdAlifConfig {
    /// Baseline matching the existing feed-forward fixed-threshold arch.
    pub fn feedforward_fixed(hidden: usize, n_classes: usize, lr: f32, epochs: usize) -> Self {
        Self {
            hidden,
            n_classes,
            lr,
            beta: 5.0,
            epochs,
            recurrent: false,
            adaptive: false,
            tau_a: DEFAULT_TAU_A,
            beta_a: DEFAULT_BETA_A,
        }
    }

    pub fn with_recurrent(mut self, on: bool) -> Self {
        self.recurrent = on;
        self
    }

    pub fn with_adaptive(mut self, on: bool) -> Self {
        self.adaptive = on;
        self
    }

    /// Short cell label for report rows, e.g. `rec+alif`.
    pub fn arch_label(&self) -> &'static str {
        match (self.recurrent, self.adaptive) {
            (false, false) => "ff+fixed",
            (false, true) => "ff+alif",
            (true, false) => "rec+fixed",
            (true, true) => "rec+alif",
        }
    }
}

/// Recurrent / adaptive-threshold feed-forward-or-recurrent LIF with a linear
/// multiclass rate readout.
#[derive(Clone, Debug)]
pub struct ShdAlifArch {
    pub hidden: usize,
    pub n_in: usize,
    pub t: usize,
    pub n_classes: usize,
    beta: f32,
    alpha: f32,
    rho: f32,
    beta_a: f32,
    adaptive: bool,
    recurrent: bool,
    /// `hidden × n_in`
    win: Vec<f32>,
    /// `hidden × hidden`, zero when `!recurrent`, zero diagonal always.
    wrec: Vec<f32>,
    /// `n_classes × hidden`
    wout: Vec<f32>,
    bout: Vec<f32>,
}

/// Per-example forward products needed by every update rule.
struct AlifForward {
    rates: Vec<f32>,
    /// `hidden × n_in`
    e_in: Vec<f32>,
    /// `hidden × hidden`, empty when `!recurrent`
    e_rec: Vec<f32>,
    logits: Vec<f32>,
    /// Mean spikes per neuron per timestep, for sparsity disclosure.
    activity: f32,
}

/// Advance the two-state surrogate ALIF eligibility for one synapse.
#[inline]
fn alif_eligibility_step(
    epsilon_v: &mut f32,
    epsilon_a: &mut f32,
    pre: f32,
    surrogate: f32,
    alpha: f32,
    rho: f32,
    beta_a: f32,
    adaptive: bool,
) -> f32 {
    *epsilon_v = alpha * *epsilon_v + pre;
    if adaptive {
        *epsilon_a = surrogate * *epsilon_v + (rho - surrogate * beta_a) * *epsilon_a;
        surrogate * (*epsilon_v - beta_a * *epsilon_a)
    } else {
        *epsilon_a = 0.0;
        surrogate * *epsilon_v
    }
}

/// Match the realised RMS of a transported output-error vector to the RMS
/// implied by the frozen initial transport scale. Direction is preserved.
///
/// Without this guard, learned readout weights can grow while DFA feedback
/// stays frozen, silently changing the effective hidden-layer learning rate.
fn normalize_hidden_modulator(mods: &mut [f32], output_delta: &[f32], hidden: usize) {
    if mods.is_empty() || output_delta.is_empty() {
        return;
    }
    let delta_l2 = output_delta.iter().map(|d| d * d).sum::<f32>().sqrt();
    let target_rms = shd_out_scale(hidden) * delta_l2 / 3.0f32.sqrt();
    let actual_rms = (mods.iter().map(|m| m * m).sum::<f32>() / mods.len() as f32).sqrt();
    if actual_rms > f32::EPSILON && target_rms.is_finite() {
        let scale = target_rms / actual_rms;
        for value in mods {
            *value *= scale;
        }
    }
}

impl ShdAlifArch {
    pub fn new(ex: &ShdExample, cfg: &ShdAlifConfig, seed: u64) -> Self {
        assert!(ex.n_in >= 1 && ex.t >= 1);
        assert!(cfg.n_classes >= 2 && cfg.hidden >= 1);
        assert!(cfg.beta > 0.0, "surrogate beta must be positive");
        assert!(cfg.tau_a > 0.0, "adaptation tau must be positive");

        let mut rng = Rng::new(seed ^ 0x54D0_A11F_0000_00F1);
        let in_scale = 0.35f32 / (ex.n_in as f32).sqrt();
        let rec_scale = 0.3f32 / (cfg.hidden as f32).sqrt();
        let out_scale = shd_out_scale(cfg.hidden);

        let win: Vec<f32> = (0..cfg.hidden * ex.n_in)
            .map(|_| (rng.next_f32() * 2.0 - 1.0) * in_scale)
            .collect();

        let mut wrec = vec![0.0f32; cfg.hidden * cfg.hidden];
        if cfg.recurrent {
            for i in 0..cfg.hidden {
                for k in 0..cfg.hidden {
                    // No self-connection: a self-loop is a threshold change in
                    // disguise and would confound the adaptive axis.
                    if i != k {
                        wrec[i * cfg.hidden + k] = (rng.next_f32() * 2.0 - 1.0) * rec_scale;
                    }
                }
            }
        }

        let wout: Vec<f32> = (0..cfg.n_classes * cfg.hidden)
            .map(|_| (rng.next_f32() * 2.0 - 1.0) * out_scale)
            .collect();

        Self {
            hidden: cfg.hidden,
            n_in: ex.n_in,
            t: ex.t,
            n_classes: cfg.n_classes,
            beta: cfg.beta,
            alpha: (-1.0f32 / DEFAULT_TAU_M).exp(),
            rho: (-1.0f32 / cfg.tau_a).exp(),
            beta_a: cfg.beta_a,
            adaptive: cfg.adaptive,
            recurrent: cfg.recurrent,
            win,
            wrec,
            wout,
            bout: vec![0.0; cfg.n_classes],
        }
    }

    fn surrogate(&self, u_minus_theta: f32) -> f32 {
        let d = 1.0 + self.beta * u_minus_theta.abs();
        1.0 / (d * d)
    }

    /// Forward pass with eligibility accumulation.
    ///
    /// Input frames are scanned sparsely: SHD is a spike raster, so the active
    /// channel list per timestep is far shorter than `n_in`.
    fn forward(&self, frames: &[f32], want_elig: bool) -> AlifForward {
        let h = self.hidden;
        let n_in = self.n_in;
        let t_len = self.t;
        // The architecture is sized from `train[0]`. If any example has a
        // shorter raster this would index out of bounds deep inside the loop;
        // assert up front so the message names the real problem.
        assert!(
            frames.len() >= t_len * n_in,
            "frame buffer too small: got {} floats, need T*n_in = {}*{} = {}. \
             All SHD examples must share the same (T, n_in) as train[0].",
            frames.len(),
            t_len,
            n_in,
            t_len * n_in
        );

        let mut u = vec![0.0f32; h];
        let mut a = vec![0.0f32; h];
        let mut s_prev = vec![0.0f32; h];
        let mut s_now = vec![0.0f32; h];
        let mut rates = vec![0.0f32; h];
        let mut surr = vec![0.0f32; h];

        let mut e_in = if want_elig {
            vec![0.0f32; h * n_in]
        } else {
            Vec::new()
        };
        let mut epsilon_v_in = if want_elig {
            vec![0.0f32; h * n_in]
        } else {
            Vec::new()
        };
        let mut epsilon_a_in = if want_elig {
            vec![0.0f32; h * n_in]
        } else {
            Vec::new()
        };
        let mut e_rec = if want_elig && self.recurrent {
            vec![0.0f32; h * h]
        } else {
            Vec::new()
        };
        let mut epsilon_v_rec = if want_elig && self.recurrent {
            vec![0.0f32; h * h]
        } else {
            Vec::new()
        };
        let mut epsilon_a_rec = if want_elig && self.recurrent {
            vec![0.0f32; h * h]
        } else {
            Vec::new()
        };

        let mut active: Vec<(usize, f32)> = Vec::with_capacity(64);
        let mut total_spikes = 0.0f32;

        for t in 0..t_len {
            let base = t * n_in;
            active.clear();
            for j in 0..n_in {
                let v = frames[base + j];
                if v != 0.0 {
                    active.push((j, v));
                }
            }

            for i in 0..h {
                let mut cur = 0.0f32;
                let row = i * n_in;
                for &(j, v) in &active {
                    cur += self.win[row + j] * v;
                }
                if self.recurrent {
                    let rrow = i * h;
                    for k in 0..h {
                        let sp = s_prev[k];
                        if sp != 0.0 {
                            cur += self.wrec[rrow + k] * sp;
                        }
                    }
                }

                let ui = self.alpha * u[i] + cur;
                let theta_i = if self.adaptive {
                    THETA_REST + self.beta_a * a[i]
                } else {
                    THETA_REST
                };
                let sg = self.surrogate(ui - theta_i);
                surr[i] = sg;
                let spike = if ui >= theta_i { 1.0f32 } else { 0.0 };
                u[i] = if spike > 0.5 { V_RESET } else { ui };
                if self.adaptive {
                    a[i] = self.rho * a[i] + spike;
                }
                s_now[i] = spike;
                rates[i] += spike;
                total_spikes += spike;
            }

            if want_elig {
                for i in 0..h {
                    let sg = surr[i];
                    let row = i * n_in;
                    for j in 0..n_in {
                        let pre = frames[base + j];
                        e_in[row + j] = alif_eligibility_step(
                            &mut epsilon_v_in[row + j],
                            &mut epsilon_a_in[row + j],
                            pre,
                            sg,
                            self.alpha,
                            self.rho,
                            self.beta_a,
                            self.adaptive,
                        );
                    }
                    if self.recurrent {
                        let rrow = i * h;
                        for k in 0..h {
                            e_rec[rrow + k] = alif_eligibility_step(
                                &mut epsilon_v_rec[rrow + k],
                                &mut epsilon_a_rec[rrow + k],
                                s_prev[k],
                                sg,
                                self.alpha,
                                self.rho,
                                self.beta_a,
                                self.adaptive,
                            );
                        }
                    }
                }
            }

            s_prev.copy_from_slice(&s_now);
        }

        let mut logits = self.bout.clone();
        for c in 0..self.n_classes {
            let mut z = logits[c];
            for i in 0..h {
                z += self.wout[c * h + i] * rates[i];
            }
            logits[c] = z;
        }

        AlifForward {
            rates,
            e_in,
            e_rec,
            logits,
            activity: total_spikes / (h as f32 * t_len as f32),
        }
    }

    pub fn evaluate(&self, test: &[ShdExample]) -> (f32, f32) {
        let e = self.evaluate_detailed(test);
        (e.accuracy, e.loss)
    }

    /// Per-example predictions for paired confirmatory statistics.
    pub fn predictions(&self, test: &[ShdExample]) -> Vec<u32> {
        test.iter()
            .map(|ex| {
                let forward = self.forward(&ex.frames, false);
                softmax(&forward.logits)
                    .iter()
                    .enumerate()
                    .max_by(|left, right| left.1.total_cmp(right.1))
                    .map(|(index, _)| index as u32)
                    .unwrap_or(0)
            })
            .collect()
    }

    /// Evaluation with the diagnostics needed to detect a degenerate arm.
    ///
    /// # Why the extra fields
    ///
    /// A recurrent net can collapse (silent) or saturate (every unit spiking
    /// every step). Both produce chance-level accuracy. Read as a bare number,
    /// that is indistinguishable from "recurrence does not help" — which would
    /// invert the conclusion of the architecture ablation. `n_distinct_predicted`
    /// and `mean_activity` make the difference visible.
    pub fn evaluate_detailed(&self, test: &[ShdExample]) -> AlifEval {
        let mut correct = 0usize;
        let mut loss_sum = 0.0f32;
        let mut activity_sum = 0.0f32;
        let mut pred_counts = vec![0usize; self.n_classes];

        for ex in test {
            let fwd = self.forward(&ex.frames, false);
            activity_sum += fwd.activity;
            let p = softmax(&fwd.logits);
            let mut pred = 0usize;
            let mut best = f32::NEG_INFINITY;
            for (i, &pi) in p.iter().enumerate() {
                if pi > best {
                    best = pi;
                    pred = i;
                }
            }
            pred_counts[pred] += 1;
            loss_sum += -p[ex.label as usize].max(1e-12).ln();
            if pred == ex.label as usize {
                correct += 1;
            }
        }

        let n = test.len().max(1) as f32;
        let n_distinct_predicted = pred_counts.iter().filter(|&&c| c > 0).count();
        let majority = pred_counts.iter().copied().max().unwrap_or(0);
        AlifEval {
            accuracy: correct as f32 / n,
            loss: loss_sum / n,
            n_distinct_predicted,
            majority_pred_frac: majority as f32 / n,
            mean_activity: activity_sum / n,
            diverged: false,
        }
    }
}

/// Evaluation result plus degeneracy diagnostics.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AlifEval {
    pub accuracy: f32,
    pub loss: f32,
    /// How many distinct classes the arm ever predicted. `1` means collapse.
    pub n_distinct_predicted: usize,
    /// Fraction of test samples assigned to the single most-predicted class.
    pub majority_pred_frac: f32,
    /// Mean spikes per neuron per timestep at evaluation time.
    pub mean_activity: f32,
    /// Training produced non-finite weights or activity.
    ///
    /// Reported rather than panicked so that one diverging cell of a
    /// learning-rate sweep does not destroy the rest of the sweep. A diverged
    /// arm is always degenerate, so it can never be selected or cited.
    pub diverged: bool,
}

impl AlifEval {
    /// Sentinel for a run that blew up. Accuracy is zeroed rather than left at
    /// whatever a NaN forward pass happened to argmax to.
    pub fn diverged() -> Self {
        Self {
            accuracy: 0.0,
            loss: f32::INFINITY,
            n_distinct_predicted: 0,
            majority_pred_frac: 1.0,
            mean_activity: f32::NAN,
            diverged: true,
        }
    }
}

/// Healthy spiking band. Outside this the arm is dead or saturated, and its
/// accuracy says nothing about the credit rule.
pub const ACTIVITY_MIN: f32 = 0.001;
pub const ACTIVITY_MAX: f32 = 0.500;

/// Fraction of test samples in one predicted class above which the arm counts
/// as collapsed to a constant predictor.
pub const MAJORITY_PRED_MAX: f32 = 0.95;

impl AlifEval {
    /// Reasons this arm's accuracy is not interpretable, empty when healthy.
    pub fn defects(&self) -> Vec<&'static str> {
        let mut v = Vec::new();
        if self.diverged {
            v.push("DIVERGED (non-finite weights or activity during training)");
            return v;
        }
        if self.n_distinct_predicted <= 1 {
            v.push("COLLAPSED (predicts a single class)");
        } else if self.majority_pred_frac > MAJORITY_PRED_MAX {
            v.push("NEAR-COLLAPSED (>95% of predictions in one class)");
        }
        if !self.mean_activity.is_finite() {
            v.push("NON-FINITE ACTIVITY");
        } else if self.mean_activity < ACTIVITY_MIN {
            v.push("SILENT (hidden layer barely spikes)");
        } else if self.mean_activity > ACTIVITY_MAX {
            v.push("SATURATED (runaway recurrent activity)");
        }
        v
    }

    pub fn is_degenerate(&self) -> bool {
        !self.defects().is_empty()
    }
}

/// One trainable arm: architecture + credit rule + scale bookkeeping.
#[derive(Clone, Debug)]
pub struct ShdAlifArm {
    arch: ShdAlifArch,
    rule: ShdAlifRule,
    lr: f32,
    /// `hidden × n_classes`, used only by [`ShdAlifRule::Dfa`].
    feedback: Vec<f32>,
    modulator: ModulatorScale,
    /// Mean hidden activity over the last training epoch (sparsity disclosure).
    last_activity: f32,
    rng: Rng,
}

impl ShdAlifArm {
    pub fn new(ex: &ShdExample, cfg: &ShdAlifConfig, rule: ShdAlifRule, seed: u64) -> Self {
        let arch = ShdAlifArch::new(ex, cfg, seed);
        let mut rng = Rng::new(seed ^ 0xA11F_DFA0_0000_00F1);
        // Scale-matched to `wout` by construction. See module docs.
        let fb_scale = shd_out_scale(cfg.hidden);
        let feedback: Vec<f32> = (0..cfg.hidden * cfg.n_classes)
            .map(|_| (rng.next_f32() * 2.0 - 1.0) * fb_scale)
            .collect();
        Self {
            arch,
            rule,
            lr: cfg.lr,
            feedback,
            modulator: ModulatorScale::new(),
            last_activity: 0.0,
            rng: Rng::new(seed ^ 0xA11F_B10A_0000_00F1),
        }
    }

    pub fn modulator_scale(&self) -> ModulatorScale {
        self.modulator
    }

    pub fn mean_activity(&self) -> f32 {
        self.last_activity
    }

    pub fn arch(&self) -> &ShdAlifArch {
        &self.arch
    }

    /// Per-example predictions retained by the paired 0c-1 protocol.
    pub fn predictions(&self, test: &[ShdExample]) -> Vec<u32> {
        self.arch.predictions(test)
    }

    /// Train for `epochs` and evaluate on `test`.
    ///
    /// # Panics
    ///
    /// Panics if `train` or `test` is empty.
    pub fn train_and_evaluate(
        &mut self,
        epochs: usize,
        train: &[ShdExample],
        test: &[ShdExample],
    ) -> ShdArmReport {
        let e = self.train_and_evaluate_detailed(epochs, train, test);
        ShdArmReport {
            label: self.rule.label(),
            accuracy: e.accuracy,
            loss: e.loss,
        }
    }

    /// Train and evaluate, returning degeneracy diagnostics alongside accuracy.
    ///
    /// Divergence is **reported, not panicked**: a learning-rate sweep must be
    /// able to include a rate that blows up without losing the rest of the
    /// sweep. A diverged arm is always degenerate, so it can never be selected
    /// as a best cell or cited as a result.
    ///
    /// # Panics
    ///
    /// Panics only if `train` or `test` is empty.
    pub fn train_and_evaluate_detailed(
        &mut self,
        epochs: usize,
        train: &[ShdExample],
        test: &[ShdExample],
    ) -> AlifEval {
        assert!(!train.is_empty(), "SHD ALIF arm needs training data");
        assert!(!test.is_empty(), "SHD ALIF arm needs test data");

        for _ in 0..epochs {
            let mut act_sum = 0.0f32;
            for ex in train {
                act_sum += self.step(ex);
            }
            self.last_activity = act_sum / train.len() as f32;

            // Bail out the moment anything goes non-finite: continuing would
            // burn the remaining epochs and then argmax a NaN logit vector into
            // a plausible-looking accuracy.
            if !self.last_activity.is_finite() || !self.weights_finite() {
                return AlifEval::diverged();
            }
        }

        self.arch.evaluate_detailed(test)
    }

    fn weights_finite(&self) -> bool {
        self.arch.win.iter().all(|w| w.is_finite())
            && self.arch.wrec.iter().all(|w| w.is_finite())
            && self.arch.wout.iter().all(|w| w.is_finite())
            && self.arch.bout.iter().all(|w| w.is_finite())
    }

    /// One example. Returns the mean hidden activity for this example.
    fn step(&mut self, ex: &ShdExample) -> f32 {
        let fwd = self.arch.forward(&ex.frames, true);
        let h = self.arch.hidden;
        let n_in = self.arch.n_in;
        let c = self.arch.n_classes;
        let y = ex.label as usize;

        // Hidden modulator, per rule. All three are built at comparable scale.
        let mods: Vec<f32> = match self.rule {
            ShdAlifRule::Dfa | ShdAlifRule::EpropCeiling => {
                let probs = softmax(&fwd.logits);
                let mut delta = probs;
                delta[y] -= 1.0; // dL/dz

                // Snapshot the transport matrix BEFORE the readout update.
                let wout_snapshot = self.arch.wout.clone();

                for k in 0..c {
                    let dk = delta[k];
                    for i in 0..h {
                        self.arch.wout[k * h + i] -= self.lr * dk * fwd.rates[i];
                    }
                    self.arch.bout[k] -= self.lr * dk;
                }

                let mut projected: Vec<f32> = (0..h)
                    .map(|i| {
                        let mut m = 0.0f32;
                        for k in 0..c {
                            let src = match self.rule {
                                ShdAlifRule::EpropCeiling => wout_snapshot[k * h + i],
                                _ => self.feedback[i * c + k],
                            };
                            m += src * (-delta[k]);
                        }
                        m
                    })
                    .collect();
                normalize_hidden_modulator(&mut projected, &delta, h);
                projected
            }
            ShdAlifRule::BroadcastPm1 => {
                let probs = softmax(&fwd.logits);
                let action = sample_categorical(&probs, &mut self.rng);
                let reward = if action == y { 1.0f32 } else { -1.0 };
                let adv = 1.0 - probs[action];
                for k in 0..c {
                    let indicator = if k == action { 1.0f32 } else { 0.0 };
                    let g = reward * (indicator - probs[k]);
                    for i in 0..h {
                        self.arch.wout[k * h + i] += self.lr * g * fwd.rates[i];
                    }
                    self.arch.bout[k] += self.lr * g;
                }
                // Scale the scalar broadcast so its magnitude is comparable to
                // the graded arms rather than arbitrarily larger.
                let scalar = reward * adv * shd_out_scale(h);
                vec![scalar; h]
            }
        };

        self.modulator.observe(&mods);

        // Hidden plasticity: local eligibility × per-unit modulator.
        for i in 0..h {
            let m = mods[i];
            if m == 0.0 {
                continue;
            }
            let row = i * n_in;
            for j in 0..n_in {
                let e = fwd.e_in[row + j];
                if e != 0.0 {
                    self.arch.win[row + j] += self.lr * m * e;
                }
            }
            if self.arch.recurrent {
                let rrow = i * h;
                for k in 0..h {
                    if i == k {
                        continue; // keep the diagonal pinned at zero
                    }
                    let e = fwd.e_rec[rrow + k];
                    if e != 0.0 {
                        self.arch.wrec[rrow + k] += self.lr * m * e;
                    }
                }
            }
        }

        fwd.activity
    }
}

/// Return a copy of `train` with labels permuted.
///
/// # The control this enables
///
/// If an arm scores materially above chance on shuffled labels, its accuracy is
/// not evidence of learning the task — it is leakage, an evaluation-set artifact,
/// or a class-prior effect. The `c1-shd-cal-*` suite has never run this control,
/// so the 0.234 DFA figure has no negative control behind it. Every SHD claim
/// should be accompanied by this number.
pub fn shuffle_labels(train: &[ShdExample], seed: u64) -> Vec<ShdExample> {
    let mut out = train.to_vec();
    let mut rng = Rng::new(seed ^ 0x5A1F_1AB3_0000_00F1);
    let n = out.len();
    for i in (1..n).rev() {
        let j = rng.gen_index(i + 1);
        let li = out[i].label;
        let lj = out[j].label;
        out[i].label = lj;
        out[j].label = li;
    }
    out
}

fn softmax(logits: &[f32]) -> Vec<f32> {
    let m = logits.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let ex: Vec<f32> = logits.iter().map(|z| (z - m).exp()).collect();
    let s: f32 = ex.iter().sum::<f32>().max(1e-12);
    ex.iter().map(|e| e / s).collect()
}

fn sample_categorical(probs: &[f32], rng: &mut Rng) -> usize {
    let u = rng.next_f32();
    let mut acc = 0.0f32;
    for (i, &p) in probs.iter().enumerate() {
        acc += p;
        if u <= acc {
            return i;
        }
    }
    probs.len().saturating_sub(1)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Dense enough that the LIF layer reliably spikes: several channels active
    /// at every timestep. A silent network would make the forward-pass ablation
    /// assertions vacuous.
    fn toy(n: usize, n_in: usize, t: usize, n_classes: usize, seed: u64) -> Vec<ShdExample> {
        let mut rng = Rng::new(seed);
        (0..n)
            .map(|_| {
                let label = rng.gen_index(n_classes) as u32;
                let mut frames = vec![0.0f32; t * n_in];
                for tt in 0..t {
                    // Class-specific channel block, plus jitter.
                    for rep in 0..4 {
                        let ch = (label as usize * 3 + rep + rng.gen_index(2)) % n_in;
                        frames[tt * n_in + ch] = 3.0;
                    }
                }
                ShdExample {
                    frames,
                    t,
                    n_in,
                    label,
                }
            })
            .collect()
    }

    fn total_spikes(f: &AlifForward) -> f32 {
        f.rates.iter().sum()
    }

    fn cfg(hidden: usize, n_classes: usize) -> ShdAlifConfig {
        ShdAlifConfig::feedforward_fixed(hidden, n_classes, 0.05, 6)
    }

    #[test]
    fn header_forbids_production() {
        let src = include_str!("shd_alif.rs");
        assert!(src.contains("MUST NEVER BE THE PRODUCTION LEARNER"));
        assert!(src.contains("GC1"));
    }

    #[test]
    fn all_four_ablation_cells_run() {
        let train = toy(40, 16, 9, 4, 1);
        let test = toy(20, 16, 9, 4, 2);
        for &rec in &[false, true] {
            for &ad in &[false, true] {
                let c = cfg(24, 4).with_recurrent(rec).with_adaptive(ad);
                let mut arm = ShdAlifArm::new(&train[0], &c, ShdAlifRule::Dfa, 7);
                let r = arm.train_and_evaluate(c.epochs, &train, &test);
                assert!(r.accuracy.is_finite(), "{} produced NaN", c.arch_label());
                assert!((0.0..=1.0).contains(&r.accuracy));
            }
        }
    }

    /// The recurrence axis must actually change the computation. If `W_rec` is
    /// inert the ablation is meaningless.
    #[test]
    fn recurrence_changes_the_forward_pass() {
        let train = toy(8, 16, 30, 4, 3);
        let ff = cfg(24, 4).with_recurrent(false);
        let rec = cfg(24, 4).with_recurrent(true);
        let a_ff = ShdAlifArch::new(&train[0], &ff, 11);
        let a_rec = ShdAlifArch::new(&train[0], &rec, 11);
        let f1 = a_ff.forward(&train[0].frames, false);
        let f2 = a_rec.forward(&train[0].frames, false);
        assert!(
            total_spikes(&f1) > 0.0,
            "test fixture produced a silent network; raise the input drive"
        );
        assert_ne!(
            f1.rates, f2.rates,
            "enabling W_rec must change hidden rates"
        );
    }

    /// The adaptation axis must actually change the computation.
    #[test]
    fn adaptive_threshold_changes_the_forward_pass() {
        let train = toy(8, 16, 30, 4, 4);
        let fixed = cfg(24, 4).with_adaptive(false);
        let alif = cfg(24, 4).with_adaptive(true);
        let a_fix = ShdAlifArch::new(&train[0], &fixed, 13);
        let a_ad = ShdAlifArch::new(&train[0], &alif, 13);
        let f1 = a_fix.forward(&train[0].frames, false);
        let f2 = a_ad.forward(&train[0].frames, false);
        let s1 = total_spikes(&f1);
        let s2 = total_spikes(&f2);
        assert!(
            s1 > 0.0,
            "test fixture produced a silent network; raise the input drive"
        );
        assert_ne!(
            f1.rates, f2.rates,
            "enabling threshold adaptation must change hidden rates"
        );
        // Adaptation raises the threshold after a spike, so it cannot increase
        // the total spike count.
        assert!(
            s2 <= s1 + 1e-3,
            "adaptation must not increase firing: {s1} -> {s2}"
        );
    }

    /// Recurrent self-connections stay pinned at zero, before and after training.
    #[test]
    fn recurrent_diagonal_stays_zero() {
        let train = toy(24, 16, 9, 4, 5);
        let test = toy(12, 16, 9, 4, 6);
        let c = cfg(16, 4).with_recurrent(true);
        let mut arm = ShdAlifArm::new(&train[0], &c, ShdAlifRule::Dfa, 17);
        arm.train_and_evaluate(3, &train, &test);
        let h = arm.arch.hidden;
        for i in 0..h {
            assert_eq!(
                arm.arch.wrec[i * h + i],
                0.0,
                "self-connection {i} is nonzero"
            );
        }
    }

    /// The regression that produced a ceiling below its own treatment.
    #[test]
    fn dfa_and_eprop_modulator_scales_match() {
        let train = toy(60, 16, 9, 4, 7);
        let test = toy(20, 16, 9, 4, 8);
        let c = cfg(32, 4).with_recurrent(true).with_adaptive(true);
        let mut dfa = ShdAlifArm::new(&train[0], &c, ShdAlifRule::Dfa, 19);
        dfa.train_and_evaluate(c.epochs, &train, &test);
        let mut ceil = ShdAlifArm::new(&train[0], &c, ShdAlifRule::EpropCeiling, 19);
        ceil.train_and_evaluate(c.epochs, &train, &test);
        let ratio = ModulatorScale::ratio(&dfa.modulator_scale(), &ceil.modulator_scale());
        assert!(
            ratio <= 3.5,
            "modulator RMS ratio {ratio:.2} — arms run at different effective learning rates \
             (DFA {:.3e}, e-prop {:.3e})",
            dfa.modulator_scale().rms(),
            ceil.modulator_scale().rms()
        );
    }

    #[test]
    fn alif_eligibility_carries_adaptation_cross_term() {
        let mut epsilon_v = 0.0;
        let mut epsilon_a = 0.0;
        let first = alif_eligibility_step(
            &mut epsilon_v,
            &mut epsilon_a,
            1.0,
            0.5,
            0.8,
            0.9,
            0.2,
            true,
        );
        assert!((epsilon_v - 1.0).abs() < 1e-6);
        assert!((epsilon_a - 0.5).abs() < 1e-6);
        assert!((first - 0.45).abs() < 1e-6);

        let second = alif_eligibility_step(
            &mut epsilon_v,
            &mut epsilon_a,
            0.0,
            0.5,
            0.8,
            0.9,
            0.2,
            true,
        );
        assert!((epsilon_v - 0.8).abs() < 1e-6);
        assert!((epsilon_a - 0.8).abs() < 1e-6);
        assert!((second - 0.32).abs() < 1e-6);
    }

    #[test]
    fn fixed_threshold_eligibility_is_beta_zero_limit() {
        let mut epsilon_v = 0.0;
        let mut epsilon_a = 7.0;
        let eligibility = alif_eligibility_step(
            &mut epsilon_v,
            &mut epsilon_a,
            1.0,
            0.5,
            0.8,
            0.9,
            0.2,
            false,
        );
        assert!((eligibility - 0.5).abs() < 1e-6);
        assert_eq!(epsilon_a, 0.0);
    }

    #[test]
    fn hidden_modulator_normalization_preserves_direction_and_target_rms() {
        let mut mods = vec![2.0, -4.0, 1.0, -2.0];
        let before = mods.clone();
        let delta = [0.25, -0.75, 0.5];
        let hidden = mods.len();
        normalize_hidden_modulator(&mut mods, &delta, hidden);
        let target = shd_out_scale(mods.len()) * delta.iter().map(|d| d * d).sum::<f32>().sqrt()
            / 3.0f32.sqrt();
        let rms = (mods.iter().map(|m| m * m).sum::<f32>() / mods.len() as f32).sqrt();
        assert!((rms - target).abs() < 1e-6);
        for (new, old) in mods.iter().zip(before) {
            assert_eq!(new.signum(), old.signum());
        }
    }

    /// A collapsed arm must be detectable. Without this, a recurrent net that
    /// predicts one class scores at chance and gets read as "recurrence does not
    /// help" — inverting the ablation's conclusion.
    #[test]
    fn collapsed_and_saturated_arms_are_flagged() {
        let collapsed = AlifEval {
            accuracy: 0.05,
            loss: 3.0,
            n_distinct_predicted: 1,
            majority_pred_frac: 1.0,
            mean_activity: 0.05,
            diverged: false,
        };
        assert!(collapsed.is_degenerate());
        assert!(collapsed.defects().iter().any(|d| d.contains("COLLAPSED")));

        let saturated = AlifEval {
            accuracy: 0.05,
            loss: 3.0,
            n_distinct_predicted: 8,
            majority_pred_frac: 0.2,
            mean_activity: 0.9,
            diverged: false,
        };
        assert!(saturated.is_degenerate());
        assert!(saturated.defects().iter().any(|d| d.contains("SATURATED")));

        let silent = AlifEval {
            accuracy: 0.05,
            loss: 3.0,
            n_distinct_predicted: 3,
            majority_pred_frac: 0.5,
            mean_activity: 0.0,
            diverged: false,
        };
        assert!(silent.defects().iter().any(|d| d.contains("SILENT")));

        let healthy = AlifEval {
            accuracy: 0.42,
            loss: 2.0,
            n_distinct_predicted: 12,
            majority_pred_frac: 0.3,
            mean_activity: 0.05,
            diverged: false,
        };
        assert!(!healthy.is_degenerate(), "{:?}", healthy.defects());
    }

    /// A diverged arm must be reported, never panicked, and never citable.
    /// A learning-rate sweep that includes a rate which blows up has to survive.
    #[test]
    fn divergence_is_reported_not_panicked() {
        let d = AlifEval::diverged();
        assert!(d.diverged);
        assert!(d.is_degenerate());
        assert!(d.defects().iter().any(|x| x.contains("DIVERGED")));
        assert_eq!(d.accuracy, 0.0, "a diverged arm must not carry an accuracy");

        // End to end: an absurd learning rate must return rather than unwind.
        let train = toy(30, 16, 9, 4, 51);
        let test = toy(15, 16, 9, 4, 52);
        let mut c = cfg(24, 4).with_recurrent(true);
        c.lr = 1.0e37;
        let mut arm = ShdAlifArm::new(&train[0], &c, ShdAlifRule::Dfa, 53);
        let e = arm.train_and_evaluate_detailed(c.epochs, &train, &test);
        assert!(
            e.is_degenerate(),
            "lr=1e37 produced a non-degenerate result: {e:?}"
        );
    }

    /// Frames shorter than the architecture's (T, n_in) must fail with a message
    /// that names the cause, not an opaque slice-index panic.
    #[test]
    #[should_panic(expected = "frame buffer too small")]
    fn short_frame_buffer_is_rejected() {
        let train = toy(4, 16, 9, 4, 61);
        let c = cfg(8, 4);
        let arch = ShdAlifArch::new(&train[0], &c, 67);
        arch.forward(&[0.0f32; 8], false);
    }

    /// `evaluate_detailed` must actually populate the diagnostics from a real
    /// forward pass, not return placeholder values.
    #[test]
    fn evaluate_detailed_reports_real_diagnostics() {
        let train = toy(40, 16, 9, 4, 31);
        let test = toy(20, 16, 9, 4, 32);
        let c = cfg(24, 4);
        let mut arm = ShdAlifArm::new(&train[0], &c, ShdAlifRule::Dfa, 37);
        let e = arm.train_and_evaluate_detailed(c.epochs, &train, &test);
        assert!(e.accuracy.is_finite() && (0.0..=1.0).contains(&e.accuracy));
        assert!(e.n_distinct_predicted >= 1 && e.n_distinct_predicted <= 4);
        assert!((0.0..=1.0).contains(&e.majority_pred_frac));
        assert!(
            e.mean_activity > 0.0,
            "fixture produced a silent network; raise the drive"
        );
        // Consistency with the simple wrapper.
        let mut arm2 = ShdAlifArm::new(&train[0], &c, ShdAlifRule::Dfa, 37);
        let r = arm2.train_and_evaluate(c.epochs, &train, &test);
        assert!((r.accuracy - e.accuracy).abs() < 1e-6);
    }

    /// Recurrence must not blow up at the shipped init scale. A saturated
    /// network is the most likely way the ablation silently produces a wrong
    /// architecture conclusion.
    #[test]
    fn recurrent_arm_stays_in_the_activity_band() {
        let train = toy(60, 16, 9, 4, 41);
        let test = toy(20, 16, 9, 4, 42);
        let c = cfg(32, 4).with_recurrent(true).with_adaptive(true);
        let mut arm = ShdAlifArm::new(&train[0], &c, ShdAlifRule::Dfa, 43);
        let e = arm.train_and_evaluate_detailed(c.epochs, &train, &test);
        assert!(
            e.mean_activity >= ACTIVITY_MIN && e.mean_activity <= ACTIVITY_MAX,
            "recurrent activity {:.4} outside [{ACTIVITY_MIN}, {ACTIVITY_MAX}] — \
             adjust rec_scale before trusting the ablation",
            e.mean_activity
        );
    }

    #[test]
    fn shuffling_permutes_labels_but_preserves_frames() {
        let train = toy(50, 16, 9, 4, 9);
        let shuffled = shuffle_labels(&train, 21);
        assert_eq!(shuffled.len(), train.len());
        for (a, b) in train.iter().zip(shuffled.iter()) {
            assert_eq!(a.frames, b.frames, "frames must be untouched");
        }
        let mut hist_a = [0usize; 4];
        let mut hist_b = [0usize; 4];
        for e in &train {
            hist_a[e.label as usize] += 1;
        }
        for e in &shuffled {
            hist_b[e.label as usize] += 1;
        }
        assert_eq!(hist_a, hist_b, "label multiset must be preserved");
        let moved = train
            .iter()
            .zip(shuffled.iter())
            .filter(|(a, b)| a.label != b.label)
            .count();
        assert!(moved > 0, "shuffle did not move any label");
    }

    /// A shuffled-label arm must not beat chance by much. This is the control
    /// the SHD suite has never run.
    #[test]
    fn shuffled_labels_collapse_to_chance() {
        let n_classes = 4;
        let train = toy(80, 16, 9, n_classes, 11);
        let test = toy(40, 16, 9, n_classes, 12);
        let shuffled = shuffle_labels(&train, 23);
        let c = cfg(32, n_classes);
        let mut arm = ShdAlifArm::new(&train[0], &c, ShdAlifRule::Dfa, 29);
        let r = arm.train_and_evaluate(c.epochs, &shuffled, &test);
        let chance = 1.0 / n_classes as f32;
        assert!(
            r.accuracy < chance + 0.30,
            "shuffled-label control reached {:.4} (chance {chance:.4}) — \
             the pipeline has a leak",
            r.accuracy
        );
    }
}
