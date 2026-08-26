//! Labeled **matched-architecture local** reference (U-MATCH / protocol v4).
//!
//! **GC1 exempt** (this is a `*_baseline.rs` file). Do not call from production
//! learning paths. Production code must use the online three-factor rule in
//! `three_factor`. This module exists solely to close residual confound #2 in
//! `results/U-NEG_protocol_v2.md` — *"shared LIF constants ≠ matched
//! computational graph."*
//!
//! **MUST NEVER BE THE PRODUCTION LEARNER** (v7 / v8 rule).
//!
//! ## Why this exists (scientific motivation)
//!
//! Under protocol v2 the Gate-G2 `gap_closed` metric compares a spiking local
//! path (LatencyEncoder + k-WTA + single online pass) against a gradient
//! reference that runs a *different* forward graph (dense recurrent LIF,
//! continuous frames, rate readout, many epochs). A low `gap_closed` therefore
//! conflates "the local **rule** is too weak" with "the local **path** is
//! handicapped."
//!
//! This module removes that confound by running **both** arms on the *identical*
//! forward graph — [`MatchedArch::forward`], the same dense-LIF forward used by
//! [`crate::surrogate_lif_baseline::SurrogateLifReference`] — and swapping **only
//! the weight-update rule**:
//!
//! - [`MatchedGradient`] — SuperSpike BPTT (the ceiling; same math as the
//!   surrogate-LIF reference), and
//! - [`MatchedLocal`] — the **production broadcast three-factor rule** ported
//!   onto that forward: per-synapse local eligibility times a **single broadcast
//!   scalar modulator** (reward `±1`), minus weight decay. No backward graph, no
//!   weight transport, O(1) in sequence length — exactly the credit structure of
//!   [`crate::three_factor::ThreeFactor`].
//!
//! Because the forward, hidden width, input encoding, rate readout, epoch count,
//! data splits and seed lineage are all shared, the pair is a one-variable
//! contrast: **architecture fixed, learning rule swapped.** `gap_closed_matched`
//! then answers the question the metric is meant to answer.
//!
//! ## Readout as a policy (faithful broadcast credit)
//!
//! The shared forward has a single rate logit. The local arm treats the readout
//! as a Bernoulli policy `p = σ(logit)`, samples an action `a`, and receives a
//! **broadcast** scalar reward `M = +1` iff `a == label` else `−1`. The readout
//! synapses use the locally available post-minus-expected term `(a − p)`; the
//! hidden synapses see **only** the broadcast `M` times their own local
//! eligibility — no per-unit feedback weights (that would be weight transport /
//! e-prop, which is [`crate::eprop_baseline`], not the production rule). This is
//! the honest production-faithful port, including its known weakness: every
//! hidden synapse receives the same modulator sign.

#![allow(clippy::needless_range_loop)]

use binn_core::Rng;
use binn_engine::{DEFAULT_TAU_M, THETA_REST, V_RESET};

pub use crate::bptt_baseline::{GradientExample, GradientReferenceReport, REFERENCE_SEQUENCE_LEN};

const N_IN: usize = 2;
const T: usize = REFERENCE_SEQUENCE_LEN;

/// Stable label for the matched-gradient ceiling arm.
pub const MATCHED_GRADIENT_LABEL: &str = "MATCHED_ARCH_GRADIENT_CEILING";
/// Stable label for the matched-local (production-rule) arm.
pub const MATCHED_LOCAL_LABEL: &str = "MATCHED_ARCH_LOCAL_THREE_FACTOR";

/// Input-weight initialisation scale, shared by every arm in this family.
///
/// Raised from `0.5` on 2026-08-23 under
/// `results/PREREG_2026-08-23_MATCHED_ARCH_REPAIR.md`. At `0.5` the largest
/// membrane two adjacent unit impulses could reach was `alpha*0.5 + 0.5 =
/// 0.952419` against a threshold of `1.0`: the hidden layer **could not emit a
/// spike at any seed**, so the rate readout saw zeros and the logit was a bias
/// with no input dependence. Measured before the repair: 0 spikes in 400
/// forwards, max membrane 0.974568.
///
/// The value is the smallest rung of a doubling ladder from `0.5` whose initial
/// mean firing rate lies inside `[ACTIVITY_MIN, ACTIVITY_MAX]` at widths 16, 64
/// and 256 across 50 seeds. **Accuracy was not an input to the choice.** See
/// `matched_input_scale_is_the_smallest_rung_inside_the_activity_band`, which
/// re-runs that selection against the real constructor.
///
/// # What the repair costs
///
/// At `0.5` a single channel contributed at most 0.5 and two coincident channels
/// at most 1.0, so the architecture was built as a **coincidence detector** —
/// the right shape for `CoincidenceTask`. That selectivity does not survive:
/// measured at the chosen scale, the initial firing rate is 0.050 on coincident
/// input and 0.059 on split input, so single channels now cross threshold on
/// their own. The arm can still separate the classes by rate, but it is no
/// longer detecting coincidence per se, and any reading of these arms as
/// coincidence detection is now wrong.
pub const MATCHED_INPUT_SCALE: f32 = 2.0;

/// Default surrogate steepness `β` for `σ'(u) = 1/(1 + β|u−θ|)²`.
pub const DEFAULT_MATCHED_BETA: f32 = 5.0;

/// Shared dense-LIF forward weights (identical structure for both arms).
///
/// `win`: hidden × N_IN, `wrec`: hidden × hidden, `wout`: hidden, plus bias.
/// Fields are `pub(crate)` so sibling baseline modules (e.g. matched DFA) can
/// share the identical forward without duplicating init/dynamics.
#[derive(Clone, Debug)]
pub struct MatchedArch {
    pub(crate) hidden: usize,
    pub(crate) beta: f32,
    /// Membrane leak `α = exp(−1/τ_m)`, τ_m shared with the engine.
    pub(crate) alpha: f32,
    pub(crate) win: Vec<f32>,
    pub(crate) wrec: Vec<f32>,
    pub(crate) wout: Vec<f32>,
    pub(crate) by: f32,
}

/// Forward cache shared by both update rules.
pub struct ForwardCache {
    pub(crate) x: [[f32; N_IN]; T],
    pub(crate) u: Vec<[f32; T]>,
    pub(crate) s: Vec<[f32; T]>,
    pub(crate) rates: Vec<f32>,
    pub(crate) logit: f32,
}

/// Which forward graph a matched arm and its ceiling are built on.
///
/// # Why this is a parameter rather than a convention
///
/// Every matched learner picked its own forward at construction and none of
/// them said so in its report. `MatchedLocal` and `MatchedEventProp` build
/// [`MatchedArch::new`] — **recurrent**, carrying a `hidden x hidden` matrix —
/// while `MatchedDfa` and every `MatchedRl*` build
/// [`MatchedArch::feedforward`], where that matrix is absent. Within each pair
/// the arm and its ceiling agree, so nothing inverted and nothing warned.
///
/// Across pairs they do not agree, and the paper's central contrast is exactly
/// a comparison across pairs: broadcast +/-1 fails, and graded DFA and
/// REINFORCE x frozen `B_i` clear the same gate. That reads as one forward with
/// the rule varied, and it is two forwards with the rule varied. The
/// preregistration in `results/MATCHED_ARCH_RL_CONTROL.md` names
/// `new_feedforward`; protocol v4 predates it and was never migrated.
///
/// Making it an argument turns a confound into an axis: every arm can be run on
/// both graphs, and whether the forward matters becomes a measurement instead
/// of an assumption. The historical default of each learner is preserved by its
/// existing `new`, so no recorded number moves by accident.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MatchedForward {
    /// `wrec = 0`. The graph the DFA and REINFORCE arms have always used.
    FeedForward,
    /// A live `hidden x hidden` recurrent matrix. Protocol v4's graph.
    Recurrent,
}

impl MatchedForward {
    /// `true` when the graph carries a recurrent matrix.
    pub const fn is_recurrent(self) -> bool {
        matches!(self, MatchedForward::Recurrent)
    }

    /// The label that must appear beside any number measured on this graph.
    pub const fn label(self) -> &'static str {
        match self {
            MatchedForward::FeedForward => "feedforward",
            MatchedForward::Recurrent => "recurrent",
        }
    }
}

impl MatchedArch {
    /// Fresh shared forward on an explicitly named graph.
    pub fn on(forward: MatchedForward, hidden: usize, beta: f32, seed: u64) -> Self {
        Self::with_options(hidden, beta, seed, forward.is_recurrent())
    }

    /// Fresh shared forward with small deterministic weights from `seed`.
    ///
    /// Weight init mirrors [`crate::surrogate_lif_baseline::SurrogateLifReference`]
    /// so the two arms start from the *same* architecture and scale.
    pub fn new(hidden: usize, beta: f32, seed: u64) -> Self {
        Self::with_options(hidden, beta, seed, true)
    }

    /// Feed-forward matched arch (`wrec = 0`), matching the NumPy DFA preview.
    pub fn feedforward(hidden: usize, beta: f32, seed: u64) -> Self {
        Self::with_options(hidden, beta, seed, false)
    }

    fn with_options(hidden: usize, beta: f32, seed: u64, recurrent: bool) -> Self {
        Self::with_scales(hidden, beta, seed, recurrent, MATCHED_INPUT_SCALE)
    }

    pub(crate) fn with_scales(
        hidden: usize,
        beta: f32,
        seed: u64,
        recurrent: bool,
        in_scale: f32,
    ) -> Self {
        assert!(hidden >= 1, "matched arch needs ≥1 hidden unit");
        assert!(beta > 0.0, "surrogate beta must be positive");
        let mut rng = Rng::new(seed ^ 0x5171_0000_00F1);
        let rec_scale = 0.3f32 / (hidden as f32).sqrt();
        let out_scale = 0.2f32;
        let win: Vec<f32> = (0..hidden * N_IN)
            .map(|_| (rng.next_f32() * 2.0 - 1.0) * in_scale)
            .collect();
        let mut wrec: Vec<f32> = (0..hidden * hidden)
            .map(|_| (rng.next_f32() * 2.0 - 1.0) * rec_scale)
            .collect();
        if !recurrent {
            for w in wrec.iter_mut() {
                *w = 0.0;
            }
        }
        let wout: Vec<f32> = (0..hidden)
            .map(|_| (rng.next_f32() * 2.0 - 1.0) * out_scale)
            .collect();
        Self {
            hidden,
            beta,
            alpha: (-1.0f32 / DEFAULT_TAU_M).exp(),
            win,
            wrec,
            wout,
            by: 0.0,
        }
    }

    /// Identical dense-LIF forward used by both arms (θ, reset, α from engine).
    pub fn forward(&self, x1: &[f32; T], x2: &[f32; T]) -> ForwardCache {
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
                let mut cur = self.win[i * N_IN] * x[t][0] + self.win[i * N_IN + 1] * x[t][1];
                if t > 0 {
                    for k in 0..h {
                        cur += self.wrec[i * h + k] * s[k][t - 1];
                    }
                }
                let u_prev = if t > 0 { u[i][t - 1] } else { V_RESET };
                let s_prev = if t > 0 { s[i][t - 1] } else { 0.0 };
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

    pub(crate) fn evaluate(&self, test: &[GradientExample]) -> (f32, f32) {
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
        (
            correct as f32 / test.len().max(1) as f32,
            loss_sum / test.len().max(1) as f32,
        )
    }
}

/// Matched-gradient ceiling arm: SuperSpike BPTT on the shared forward.
#[derive(Clone, Debug)]
pub struct MatchedGradient {
    pub(crate) arch: MatchedArch,
    lr: f32,
}

impl MatchedGradient {
    /// New ceiling arm sized to `hidden`.
    pub fn new(hidden: usize, lr: f32, beta: f32, seed: u64) -> Self {
        Self {
            arch: MatchedArch::new(hidden, beta, seed),
            lr,
        }
    }

    /// Ceiling arm on the feed-forward matched graph (protocol-v5 DFA recipe).
    pub fn new_feedforward(hidden: usize, lr: f32, beta: f32, seed: u64) -> Self {
        Self::on(MatchedForward::FeedForward, hidden, lr, beta, seed)
    }

    /// Ceiling arm on an explicitly named graph. See [`MatchedForward`].
    pub fn on(forward: MatchedForward, hidden: usize, lr: f32, beta: f32, seed: u64) -> Self {
        Self {
            arch: MatchedArch::on(forward, hidden, beta, seed),
            lr,
        }
    }

    /// Train by BPTT and evaluate. Same contract as the other references.
    pub fn train_and_evaluate(
        &mut self,
        epochs: usize,
        train: &[GradientExample],
        test: &[GradientExample],
    ) -> GradientReferenceReport {
        assert!(!train.is_empty(), "matched gradient needs training data");
        assert!(!test.is_empty(), "matched gradient needs test data");
        for _ in 0..epochs {
            for (x1, x2, y) in train {
                let cache = self.arch.forward(x1, x2);
                let dlogit = sigmoid(cache.logit) - *y;
                self.backward_step(&cache, dlogit);
            }
        }
        let (accuracy, loss) = self.arch.evaluate(test);
        GradientReferenceReport {
            label: MATCHED_GRADIENT_LABEL,
            accuracy,
            loss,
        }
    }

    pub fn evaluate(&self, test: &[GradientExample]) -> f32 {
        self.arch.evaluate(test).0
    }

    fn backward_step(&mut self, cache: &ForwardCache, dlogit: f32) {
        let h = self.arch.hidden;
        let theta = THETA_REST;
        let alpha = self.arch.alpha;
        let beta = self.arch.beta;
        let mut dwin = vec![0.0f32; h * N_IN];
        let mut dwrec = vec![0.0f32; h * h];
        let mut dwout = vec![0.0f32; h];
        let dby = dlogit;
        let mut g_r = vec![0.0f32; h];
        for i in 0..h {
            dwout[i] = dlogit * cache.rates[i];
            g_r[i] = dlogit * self.arch.wout[i];
        }
        let mut du_next = vec![0.0f32; h];
        for t in (0..T).rev() {
            let mut du = vec![0.0f32; h];
            for i in 0..h {
                let mut ds = g_r[i] - du_next[i];
                for m in 0..h {
                    ds += du_next[m] * self.arch.wrec[m * h + i];
                }
                let surr = surrogate(cache.u[i][t] - theta, beta);
                du[i] = ds * surr + alpha * du_next[i];
            }
            for i in 0..h {
                dwin[i * N_IN] += du[i] * cache.x[t][0];
                dwin[i * N_IN + 1] += du[i] * cache.x[t][1];
                if t > 0 {
                    for k in 0..h {
                        dwrec[i * h + k] += du[i] * cache.s[k][t - 1];
                    }
                }
            }
            du_next = du;
        }
        for (w, dw) in self.arch.win.iter_mut().zip(dwin.iter()) {
            *w -= self.lr * *dw;
        }
        for (w, dw) in self.arch.wrec.iter_mut().zip(dwrec.iter()) {
            *w -= self.lr * *dw;
        }
        for (w, dw) in self.arch.wout.iter_mut().zip(dwout.iter()) {
            *w -= self.lr * *dw;
        }
        self.arch.by -= self.lr * dby;
    }
}

/// Matched-local arm: production broadcast three-factor rule on the shared
/// forward. Eligibility × single broadcast modulator − weight decay. No backward
/// graph, no weight transport.
#[derive(Clone, Debug)]
pub struct MatchedLocal {
    arch: MatchedArch,
    eta: f32,
    lambda: f32,
    rng: Rng,
}

impl MatchedLocal {
    /// New local arm sized to `hidden`.
    ///
    /// `eta` = three-factor learning rate, `lambda` = weight decay. The action
    /// sampler is seeded so runs are deterministic (GC3).
    pub fn new(hidden: usize, eta: f32, lambda: f32, beta: f32, seed: u64) -> Self {
        // Recurrent is protocol v4's historical graph, preserved so that no
        // archived number moves by accident. Every new comparison should name
        // the graph through `on` instead.
        Self::on(MatchedForward::Recurrent, hidden, eta, lambda, beta, seed)
    }

    /// The local arm on an explicitly named graph. See [`MatchedForward`].
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
            rng: Rng::new(seed ^ 0x3FAC_7012_0000_00F1),
        }
    }

    /// Train with the local rule and evaluate. Same contract as the references.
    pub fn train_and_evaluate(
        &mut self,
        epochs: usize,
        train: &[GradientExample],
        test: &[GradientExample],
    ) -> GradientReferenceReport {
        assert!(!train.is_empty(), "matched local needs training data");
        assert!(!test.is_empty(), "matched local needs test data");
        for _ in 0..epochs {
            for (x1, x2, y) in train {
                self.local_step(x1, x2, *y);
            }
        }
        let (accuracy, loss) = self.arch.evaluate(test);
        GradientReferenceReport {
            label: MATCHED_LOCAL_LABEL,
            accuracy,
            loss,
        }
    }

    /// One online three-factor update on a single example.
    fn local_step(&mut self, x1: &[f32; T], x2: &[f32; T], y: f32) {
        let h = self.arch.hidden;
        let alpha = self.arch.alpha;
        let beta = self.arch.beta;
        let cache = self.arch.forward(x1, x2);

        // Readout as a Bernoulli policy; reward is the single broadcast scalar.
        let p = sigmoid(cache.logit);
        let a = if self.rng.next_f32() < p { 1.0f32 } else { 0.0 };
        let reward = if (a - y).abs() < 0.5 { 1.0f32 } else { -1.0 };
        let m = reward; // broadcast modulator M (attention·(reward+novelty), att=1)

        // Per-synapse hidden eligibility: e_ij = Σ_t α·e + σ'(u_i[t]) · pre_j[t].
        // Input eligibility (pre = x_j) and recurrent eligibility (pre = s_k[t-1]).
        let mut e_in = vec![0.0f32; h * N_IN];
        let mut e_rec = vec![0.0f32; h * h];
        let theta = THETA_REST;
        for i in 0..h {
            let mut ei0 = 0.0f32;
            let mut ei1 = 0.0f32;
            let mut erow = vec![0.0f32; h];
            for t in 0..T {
                let surr = surrogate(cache.u[i][t] - theta, beta);
                ei0 = alpha * ei0 + surr * cache.x[t][0];
                ei1 = alpha * ei1 + surr * cache.x[t][1];
                if t > 0 {
                    for k in 0..h {
                        erow[k] = alpha * erow[k] + surr * cache.s[k][t - 1];
                    }
                } else {
                    for k in 0..h {
                        erow[k] *= alpha;
                    }
                }
            }
            e_in[i * N_IN] = ei0;
            e_in[i * N_IN + 1] = ei1;
            for k in 0..h {
                e_rec[i * h + k] = erow[k];
            }
        }

        // Readout uses the locally available post-minus-expected term (a − p).
        let e_out_scale = a - p;
        for i in 0..h {
            let e_out = e_out_scale * cache.rates[i];
            let w = &mut self.arch.wout[i];
            *w += self.eta * m * e_out - self.lambda * *w;
        }
        self.arch.by += self.eta * m * e_out_scale;

        // Hidden synapses see ONLY the broadcast modulator times local
        // eligibility (no wout feedback — that would be weight transport).
        for i in 0..h {
            for j in 0..N_IN {
                let idx = i * N_IN + j;
                let w = &mut self.arch.win[idx];
                *w += self.eta * m * e_in[idx] - self.lambda * *w;
            }
            for k in 0..h {
                let idx = i * h + k;
                let w = &mut self.arch.wrec[idx];
                *w += self.eta * m * e_rec[idx] - self.lambda * *w;
            }
        }
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

#[inline]
fn bce(p: f32, y: f32) -> f32 {
    let p = p.clamp(1e-6, 1.0 - 1e-6);
    -(y * p.ln() + (1.0 - y) * (1.0 - p).ln())
}

#[cfg(test)]
mod tests {
    use super::*;
    use binn_core::Rng;

    /// Same coincidence task the other references consume.
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

    /// Re-runs the activity-band selection registered in
    /// `results/PREREG_2026-08-23_MATCHED_ARCH_REPAIR.md` section 1 against the
    /// real constructor.
    ///
    /// Asserts two things, not one: that the shipped scale qualifies, and that
    /// **every smaller rung does not**. Without the second half this would pass
    /// for any sufficiently large value, which is the same as not pinning the
    /// choice.
    #[test]
    fn matched_input_scale_is_the_smallest_rung_inside_the_activity_band() {
        let qualifies = |scale: f32| {
            let mut lo = f32::INFINITY;
            let mut hi = 0.0f32;
            for &hidden in &[16usize, 64, 256] {
                for seed in 0..50u64 {
                    let arch =
                        MatchedArch::with_scales(hidden, DEFAULT_MATCHED_BETA, seed, true, scale);
                    let mut spikes = 0.0f32;
                    for case in 0..2 {
                        let mut x1 = [0.0f32; T];
                        let mut x2 = [0.0f32; T];
                        x1[2] = 1.0;
                        x2[if case == 0 { 2 } else { T - 1 }] = 1.0;
                        spikes += arch.forward(&x1, &x2).rates.iter().sum::<f32>();
                    }
                    let rate = spikes / (2.0 * hidden as f32 * T as f32);
                    lo = lo.min(rate);
                    hi = hi.max(rate);
                }
            }
            lo >= 0.001 && hi <= 0.500
        };

        assert!(
            qualifies(MATCHED_INPUT_SCALE),
            "the shipped input scale {MATCHED_INPUT_SCALE} leaves some layer \
             outside the activity band at initialisation"
        );
        let mut rung = 0.5f32;
        while rung < MATCHED_INPUT_SCALE {
            assert!(
                !qualifies(rung),
                "rung {rung} also qualifies, so {MATCHED_INPUT_SCALE} is not the \
                 smallest qualifying rung and the registered rule was not followed"
            );
            rung *= 2.0;
        }
        assert!(
            (rung - MATCHED_INPUT_SCALE).abs() < 1e-6,
            "{MATCHED_INPUT_SCALE} is not on the doubling ladder from 0.5"
        );
    }

    /// The forward can spike at all.
    ///
    /// Before 2026-08-23 it could not, at any seed: `alpha*0.5 + 0.5 = 0.952419`
    /// against a threshold of 1.0. 400 forwards produced zero spikes and a peak
    /// membrane of 0.974568, so every arm in this family read a zero rate vector
    /// and emitted a logit equal to its bias.
    #[test]
    fn the_matched_forward_is_not_silent() {
        let mut total = 0.0f32;
        for seed in 0..20u64 {
            let arch = MatchedArch::new(64, DEFAULT_MATCHED_BETA, seed);
            let mut x1 = [0.0f32; T];
            let mut x2 = [0.0f32; T];
            x1[2] = 1.0;
            x2[2] = 1.0;
            total += arch.forward(&x1, &x2).rates.iter().sum::<f32>();
        }
        assert!(
            total > 0.0,
            "the matched architecture emitted no spikes across 20 seeds; it is \
             silent again and every arm built on it reports a bias, not a readout"
        );
    }

    #[test]
    fn header_forbids_production_use() {
        let src = include_str!("matched_local_baseline.rs");
        assert!(
            src.contains("MUST NEVER BE THE PRODUCTION LEARNER"),
            "matched_local_baseline.rs must carry the production-ban header"
        );
        assert!(src.contains("GC1 exempt") || src.contains("GC1-exempt"));
    }

    #[test]
    fn both_arms_share_the_same_forward() {
        // Structural matched guarantee: identical init + identical forward math.
        let a = MatchedGradient::new(8, 0.02, DEFAULT_MATCHED_BETA, 0xABCD);
        let b = MatchedLocal::new(8, 0.2, 0.0, DEFAULT_MATCHED_BETA, 0xABCD);
        let mut x1 = [0.0f32; T];
        let mut x2 = [0.0f32; T];
        x1[2] = 1.0;
        x2[3] = 1.0;
        let la = a.arch.forward(&x1, &x2).logit;
        let lb = b.arch.forward(&x1, &x2).logit;
        assert_eq!(la, lb, "arms must start on an identical forward graph");
    }

    #[test]
    fn shares_engine_lif_constants() {
        let g = MatchedGradient::new(8, 0.02, DEFAULT_MATCHED_BETA, 1);
        assert_eq!(THETA_REST, 1.0);
        assert_eq!(V_RESET, 0.0);
        assert!((g.arch.alpha - (-1.0f32 / DEFAULT_TAU_M).exp()).abs() < 1e-7);
    }

    #[test]
    fn matched_gradient_learns_above_floor() {
        // The ceiling arm must clear the 0.65 floor or the matched harness is
        // invalid (the same requirement the surrogate-LIF reference meets).
        let mut g = MatchedGradient::new(32, 0.02, DEFAULT_MATCHED_BETA, 0xC01C_1DEA);
        let train = gen_examples(256, 0xB177_00B7);
        let test = gen_examples(128, 0x7E57_0002);
        let r = g.train_and_evaluate(150, &train, &test);
        assert_eq!(r.label, MATCHED_GRADIENT_LABEL);
        assert!(
            r.accuracy >= 0.65,
            "matched gradient ceiling must learn coincidence; acc={}",
            r.accuracy
        );
    }

    #[test]
    fn matched_local_is_deterministic() {
        let train = gen_examples(64, 1);
        let test = gen_examples(32, 2);
        let mut a = MatchedLocal::new(16, 0.2, 0.002, DEFAULT_MATCHED_BETA, 777);
        let mut b = MatchedLocal::new(16, 0.2, 0.002, DEFAULT_MATCHED_BETA, 777);
        let ra = a.train_and_evaluate(20, &train, &test);
        let rb = b.train_and_evaluate(20, &train, &test);
        assert_eq!(ra, rb);
    }

    #[test]
    fn matched_gradient_readout_grad_matches_finite_difference() {
        // The differentiable readout path must match finite differences tightly;
        // this guards the BPTT ceiling against silent sign/chain errors.
        let g = MatchedGradient::new(6, 0.01, DEFAULT_MATCHED_BETA, 0x00F1_A17E);
        let mut x1 = [0.0f32; T];
        let mut x2 = [0.0f32; T];
        x1[2] = 1.0;
        x2[3] = 1.0;
        let y = 1.0f32;
        let cache = g.arch.forward(&x1, &x2);
        let dlogit = sigmoid(cache.logit) - y;
        let analytic = dlogit * cache.rates[0]; // ∂L/∂wout[0]

        let eps = 1e-2f32;
        let mut plus = g.arch.clone();
        plus.wout[0] += eps;
        let lp = bce(sigmoid(plus.forward(&x1, &x2).logit), y);
        let mut minus = g.arch.clone();
        minus.wout[0] -= eps;
        let lm = bce(sigmoid(minus.forward(&x1, &x2).logit), y);
        let numeric = (lp - lm) / (2.0 * eps);
        let scale = analytic.abs().max(numeric.abs()).max(1e-4);
        assert!(
            (analytic - numeric).abs() / scale < 2e-2,
            "readout gradient mismatch: analytic={analytic} numeric={numeric}"
        );
    }

    #[test]
    fn matched_local_zero_eta_is_a_noop() {
        // With η=0 the local rule must not move any weight (isolates that all
        // motion comes from the plasticity term, not bookkeeping drift).
        let mut l = MatchedLocal::new(12, 0.0, 0.0, DEFAULT_MATCHED_BETA, 42);
        let before = (l.arch.win.clone(), l.arch.wrec.clone(), l.arch.wout.clone());
        let train = gen_examples(40, 3);
        let test = gen_examples(20, 4);
        let _ = l.train_and_evaluate(10, &train, &test);
        assert_eq!(l.arch.win, before.0, "win moved under η=0");
        assert_eq!(l.arch.wrec, before.1, "wrec moved under η=0");
        assert_eq!(l.arch.wout, before.2, "wout moved under η=0");
    }

    #[test]
    fn matched_local_actually_updates_weights() {
        // Sanity: with η>0 the rule *does* move weights (guards against a dead
        // eligibility / zeroed modulator regression).
        //
        // Under the shared init scale, one-hot frames often leave the LIF silent
        // at t=0, so rate-gated `wout` may stay put while `win` (surrogate
        // eligibility × broadcast M) and bias still move. Assert any plastic
        // parameter changes — that is the dead-rule regression we care about.
        let mut l = MatchedLocal::new(16, 0.05, 0.0, DEFAULT_MATCHED_BETA, 7);
        let w0 = l.arch.wout.clone();
        let win0 = l.arch.win.clone();
        let by0 = l.arch.by;
        let train = gen_examples(80, 5);
        let test = gen_examples(20, 6);
        let _ = l.train_and_evaluate(20, &train, &test);
        let wout_moved = w0
            .iter()
            .zip(l.arch.wout.iter())
            .any(|(a, b)| (a - b).abs() > 1e-6);
        let win_moved = win0
            .iter()
            .zip(l.arch.win.iter())
            .any(|(a, b)| (a - b).abs() > 1e-6);
        let by_moved = (l.arch.by - by0).abs() > 1e-6;
        assert!(
            wout_moved || win_moved || by_moved,
            "some plastic parameter should change under η>0 (wout/win/by)"
        );
    }

    #[test]
    fn matched_local_runs_and_reports_finite() {
        // We do not assert PASS/FAIL here — that is the scientific question the
        // harness decides on n=20. We only assert the arm is well-formed.
        let mut l = MatchedLocal::new(32, 0.2, 0.002, DEFAULT_MATCHED_BETA, 0x1234);
        let train = gen_examples(80, 11);
        let test = gen_examples(40, 22);
        let r = l.train_and_evaluate(80, &train, &test);
        assert_eq!(r.label, MATCHED_LOCAL_LABEL);
        assert!(r.accuracy.is_finite() && (0.0..=1.0).contains(&r.accuracy));
        assert!(r.loss.is_finite() && r.loss >= 0.0);
    }
}
