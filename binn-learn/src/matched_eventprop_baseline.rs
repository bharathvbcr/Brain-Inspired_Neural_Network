//! Labeled **EventProp-style** credit reference on the matched dense-LIF forward.
//!
//! **GC1 exempt** (this is a `*_baseline.rs` file). Do not call from production
//! learning paths. Production code must use the online three-factor rule in
//! `three_factor`. This module exists solely for a **rule-only** head-to-head
//! against SuperSpike BPTT ([`crate::matched_local_baseline::MatchedGradient`])
//! on the identical [`MatchedArch`] coincidence forward.
//!
//! **MUST NEVER BE THE PRODUCTION LEARNER.**
//!
//! ## What this is (Wunderlich & Pehle 2021 spirit)
//!
//! Textbook EventProp is an event-based adjoint method for hybrid continuous /
//! discrete SNNs: free adjoint dynamics between spikes, jump conditions at
//! spike times, and weight gradients accumulated from spike-triggered terms
//! (typically involving the inverse membrane velocity at threshold).
//!
//! This crate runs **discrete-time hard-threshold LIF** (same as matched-arch).
//! We therefore implement a **discrete EventProp-style approximation**:
//!
//! - Same forward as [`MatchedGradient`] / [`MatchedLocal`] ([`MatchedArch`]).
//! - Reverse-mode adjoint with **hard spike gating** — membrane adjoint receives
//!   the spike/error jump **only when** `s_i[t] = 1`, not via a soft SuperSpike
//!   surrogate `σ'(u)` at every timestep.
//! - Jump scale ≈ `1 / max(|I_eff|, ε)` where `I_eff` is the discrete drive into
//!   the membrane at that step (stand-in for continuous `1/|du/dt|` at threshold).
//! - Free adjoint between spikes: leak factor `α` only.
//!
//! ## Explicitly not claimed
//!
//! - Not neuromorphic hardware EventProp.
//! - Not a bit-exact reimplementation of the continuous hybrid adjoint in
//!   Wunderlich & Pehle (2021).
//! - Not a BINN substrate / production-rule rescue.
//!
//! The scientific question answered here is rule-only: on the matched dense-LIF
//! coincidence task, does spike-triggered adjoint credit close the same gap
//! SuperSpike BPTT closes?

#![allow(clippy::needless_range_loop)]

use binn_engine::{THETA_REST, V_RESET};

pub use crate::bptt_baseline::{GradientExample, GradientReferenceReport, REFERENCE_SEQUENCE_LEN};
pub use crate::matched_local_baseline::{
    ForwardCache, MatchedArch, MatchedForward, DEFAULT_MATCHED_BETA, MATCHED_GRADIENT_LABEL,
};

const N_IN: usize = 2;
const T: usize = REFERENCE_SEQUENCE_LEN;

/// Floor under the discrete `|I_eff|` inverse-velocity proxy (avoids blow-up).
const EVENTPROP_UDOT_EPS: f32 = 1e-3;
/// Cap on `1/|I_eff|` so a near-threshold crossing does not explode the step.
const EVENTPROP_JUMP_MAX: f32 = 10.0;

/// Stable label for the matched EventProp arm.
pub const MATCHED_EVENTPROP_LABEL: &str = "MATCHED_ARCH_EVENTPROP";

/// Matched EventProp-style arm: spike-triggered adjoint on the shared forward.
#[derive(Clone, Debug)]
pub struct MatchedEventProp {
    pub(crate) arch: MatchedArch,
    lr: f32,
}

impl MatchedEventProp {
    /// New EventProp arm on the **recurrent** matched graph (same as SuperSpike
    /// matched-arch ceiling).
    pub fn new(hidden: usize, lr: f32, beta: f32, seed: u64) -> Self {
        // Protocol v28's historical graph, preserved. See [`MatchedForward`].
        Self::on(MatchedForward::Recurrent, hidden, lr, beta, seed)
    }

    /// The spike-adjoint arm on an explicitly named graph.
    pub fn on(forward: MatchedForward, hidden: usize, lr: f32, beta: f32, seed: u64) -> Self {
        Self {
            arch: MatchedArch::on(forward, hidden, beta, seed),
            lr,
        }
    }

    /// Train by EventProp-style adjoint and evaluate.
    pub fn train_and_evaluate(
        &mut self,
        epochs: usize,
        train: &[GradientExample],
        test: &[GradientExample],
    ) -> GradientReferenceReport {
        assert!(!train.is_empty(), "matched EventProp needs training data");
        assert!(!test.is_empty(), "matched EventProp needs test data");
        for _ in 0..epochs {
            for (x1, x2, y) in train {
                let cache = self.arch.forward(x1, x2);
                let dlogit = sigmoid(cache.logit) - *y;
                self.eventprop_backward_step(&cache, dlogit);
            }
        }
        let (accuracy, loss) = self.arch.evaluate(test);
        GradientReferenceReport {
            label: MATCHED_EVENTPROP_LABEL,
            accuracy,
            loss,
        }
    }

    /// Discrete EventProp-style reverse step (hard spike gate + inv-drive jump).
    fn eventprop_backward_step(&mut self, cache: &ForwardCache, dlogit: f32) {
        let h = self.arch.hidden;
        let theta = THETA_REST;
        let alpha = self.arch.alpha;
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
                // Spike adjoint: rate loss + reset coupling + recurrent through spikes.
                let mut ds = g_r[i] - du_next[i];
                for m in 0..h {
                    ds += du_next[m] * self.arch.wrec[m * h + i];
                }
                let spiked = cache.s[i][t] > 0.5;
                if spiked {
                    // Discrete stand-in for EventProp's 1/|du/dt| at threshold:
                    // reconstruct effective drive into the membrane at this step.
                    let u_prev = if t > 0 { cache.u[i][t - 1] } else { V_RESET };
                    let s_prev = if t > 0 { cache.s[i][t - 1] } else { 0.0 };
                    // u[t] = α u_prev + I_eff - θ s_prev  ⇒  I_eff = u[t] - α u_prev + θ s_prev
                    let i_eff = cache.u[i][t] - alpha * u_prev + theta * s_prev;
                    let jump = (1.0 / i_eff.abs().max(EVENTPROP_UDOT_EPS)).min(EVENTPROP_JUMP_MAX);
                    du[i] = ds * jump + alpha * du_next[i];
                } else {
                    // Free adjoint: leak only (no soft σ' at silent timesteps).
                    du[i] = alpha * du_next[i];
                }
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

#[inline]
fn sigmoid(z: f32) -> f32 {
    1.0 / (1.0 + (-z).exp())
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
        let src = include_str!("matched_eventprop_baseline.rs");
        assert!(
            src.contains("MUST NEVER BE THE PRODUCTION LEARNER"),
            "matched_eventprop_baseline.rs must carry the production-ban header"
        );
        assert!(src.contains("GC1 exempt") || src.contains("GC1-exempt"));
    }

    #[test]
    fn shares_forward_with_superspike_ceiling() {
        use crate::matched_local_baseline::MatchedGradient;
        let ep = MatchedEventProp::new(8, 0.02, DEFAULT_MATCHED_BETA, 0xABCD);
        let ss = MatchedGradient::new(8, 0.02, DEFAULT_MATCHED_BETA, 0xABCD);
        let mut x1 = [0.0f32; T];
        let mut x2 = [0.0f32; T];
        x1[2] = 1.0;
        x2[3] = 1.0;
        let la = ep.arch.forward(&x1, &x2).logit;
        let lb = ss.arch.forward(&x1, &x2).logit;
        assert_eq!(
            la, lb,
            "EventProp and SuperSpike must share identical forward"
        );
    }

    #[test]
    fn eventprop_is_deterministic() {
        let train = gen_examples(64, 1);
        let test = gen_examples(32, 2);
        let mut a = MatchedEventProp::new(16, 0.05, DEFAULT_MATCHED_BETA, 777);
        let mut b = MatchedEventProp::new(16, 0.05, DEFAULT_MATCHED_BETA, 777);
        let ra = a.train_and_evaluate(20, &train, &test);
        let rb = b.train_and_evaluate(20, &train, &test);
        assert_eq!(ra, rb);
    }

    #[test]
    fn eventprop_zero_lr_is_a_noop() {
        let mut ep = MatchedEventProp::new(12, 0.0, DEFAULT_MATCHED_BETA, 42);
        let before = (
            ep.arch.win.clone(),
            ep.arch.wrec.clone(),
            ep.arch.wout.clone(),
        );
        let train = gen_examples(40, 3);
        let test = gen_examples(20, 4);
        let _ = ep.train_and_evaluate(10, &train, &test);
        assert_eq!(ep.arch.win, before.0, "win moved under lr=0");
        assert_eq!(ep.arch.wrec, before.1, "wrec moved under lr=0");
        assert_eq!(ep.arch.wout, before.2, "wout moved under lr=0");
    }

    #[test]
    fn eventprop_can_learn_coincidence_smoke() {
        // Not a scientific floor claim — just that the adjoint path is alive.
        let mut ep = MatchedEventProp::new(32, 0.05, DEFAULT_MATCHED_BETA, 0xE7E7_0001);
        let train = gen_examples(256, 0xE7E7_00B7);
        let test = gen_examples(128, 0xE7E7_0002);
        let r = ep.train_and_evaluate(120, &train, &test);
        assert_eq!(r.label, MATCHED_EVENTPROP_LABEL);
        assert!(r.accuracy.is_finite());
        assert!(
            r.accuracy > 0.45,
            "EventProp smoke should beat near-chance; acc={}",
            r.accuracy
        );
    }
}
