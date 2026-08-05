//! Matched-architecture **mechanism diagnostic** (protocol `c1-mech-*`).
//!
//! **GC1 exempt** (this is a `*_baseline.rs` file). Do not call from production
//! learning paths. **MUST NEVER BE THE PRODUCTION LEARNER.**
//!
//! On a **frozen** feed-forward dense-LIF coincidence forward
//! ([`MatchedArch::feedforward`]), measure one-step update usefulness for:
//! broadcast ±1, graded broadcast, DFA, REINFORCE×B, and SuperSpike BPTT.
//!
//! Primary columns:
//! - **loss_drop** — BCE decrease after a unit-norm one-step update
//! - **loss_drop_rotate** — same after a coordinate-shuffle control
//! - **elig_energy_capture** — fraction of SuperSpike grad energy on synapses
//!   with nonzero eligibility
//!
//! Secondary only: cosine / sign agreement with −∇L (SuperSpike).

#![allow(clippy::needless_range_loop)]

use binn_core::Rng;
use binn_engine::THETA_REST;

use crate::credit::ReinforceFeedback;
use crate::matched_local_baseline::{ForwardCache, MatchedArch, DEFAULT_MATCHED_BETA};
use crate::{GradientExample, REFERENCE_SEQUENCE_LEN};

const N_IN: usize = 2;
const T: usize = REFERENCE_SEQUENCE_LEN;
/// Minimum |E| counted as “nonzero eligibility” for energy capture.
const ELIG_EPS: f32 = 1e-8;
/// Applied one-step L2 step size (fair magnitude across arms).
const UNIT_STEP: f32 = 0.25;
/// Warm-start epochs so the frozen forward sits past silent init.
const WARM_EPOCHS: usize = 30;

/// Stable arm labels for reporting.
pub const MECH_ARM_BROADCAST_PM1: &str = "broadcast_pm1";
pub const MECH_ARM_GRADED_BROADCAST: &str = "graded_broadcast";
pub const MECH_ARM_DFA: &str = "dfa";
pub const MECH_ARM_RL_FB: &str = "rl_reinforce_fb";
pub const MECH_ARM_SUPERSPIKE: &str = "superspike";

/// One arm’s diagnostic on a frozen forward + example set.
#[derive(Clone, Debug, PartialEq)]
pub struct MechArmMetrics {
    pub arm: &'static str,
    /// Mean BCE drop after unit-norm proposed update (higher ⇒ more useful).
    pub loss_drop: f32,
    /// Mean BCE drop after shuffling the same-norm update (control).
    pub loss_drop_rotate: f32,
    /// Fraction of SuperSpike ‖g‖² on synapses with |E| > ε (shared E from SS).
    pub elig_energy_capture: f32,
    /// Secondary: mean cosine(Δw, −g_SS).
    pub cosine_vs_ss: f32,
    /// Secondary: mean fraction of agreeing signs with −g_SS.
    pub sign_agree_vs_ss: f32,
}

/// Full diagnostic report for one seed / frozen arch.
#[derive(Clone, Debug, PartialEq)]
pub struct MechDiagnosticReport {
    pub arms: Vec<MechArmMetrics>,
}

/// Run the mechanism diagnostic on a feed-forward matched arch.
///
/// Warms with a few SuperSpike steps, **freezes** weights, then probes one-step
/// updates on `train` (no further multi-epoch training of the compared arms).
pub fn run_mech_diagnostic(
    hidden: usize,
    beta: f32,
    seed: u64,
    train: &[GradientExample],
) -> MechDiagnosticReport {
    assert!(!train.is_empty(), "mech diagnostic needs ≥1 example");
    let beta = if beta > 0.0 {
        beta
    } else {
        DEFAULT_MATCHED_BETA
    };
    let mut arch = MatchedArch::feedforward(hidden, beta, seed);
    warm_superspike(&mut arch, train, 0.02, WARM_EPOCHS);
    let feedback = ReinforceFeedback::new(hidden, seed ^ 0xDFA0_00FB)
        .weights()
        .to_vec();
    let mut rng = Rng::new(seed ^ 0xCEC4_D1A6_0000_00F1);

    let mut sums = ArmAccum::default();
    let mut n = 0usize;
    for (x1, x2, y) in train {
        let cache = arch.forward(x1, x2);
        let e_in = eligibility_in(&arch, &cache);
        let g_win = superspike_dwin(&arch, &cache, *y);
        let elig_cap = energy_capture(&g_win, &e_in);

        let proposals = [
            (
                MECH_ARM_BROADCAST_PM1,
                propose_broadcast_pm1(&arch, &cache, &e_in, *y, &mut rng),
            ),
            (
                MECH_ARM_GRADED_BROADCAST,
                propose_graded_broadcast(&cache, &e_in, *y),
            ),
            (MECH_ARM_DFA, propose_dfa(&cache, &e_in, &feedback, *y)),
            (
                MECH_ARM_RL_FB,
                propose_rl_fb(&arch, &cache, &e_in, &feedback, *y, &mut rng),
            ),
            (MECH_ARM_SUPERSPIKE, propose_superspike(&g_win)),
        ];

        for (arm, delta) in proposals {
            let (drop, drop_rot, cos, sign) =
                evaluate_proposal(&arch, x1, x2, *y, &delta, &g_win, &mut rng);
            sums.add(arm, drop, drop_rot, elig_cap, cos, sign);
        }
        n += 1;
    }

    MechDiagnosticReport {
        arms: sums.finish(n.max(1)),
    }
}

fn warm_superspike(arch: &mut MatchedArch, train: &[GradientExample], lr: f32, epochs: usize) {
    for _ in 0..epochs {
        for (x1, x2, y) in train {
            let cache = arch.forward(x1, x2);
            let g = superspike_dwin(arch, &cache, *y);
            for (w, dw) in arch.win.iter_mut().zip(g.iter()) {
                *w -= lr * *dw;
            }
            let dlogit = sigmoid(cache.logit) - *y;
            for i in 0..arch.hidden {
                arch.wout[i] -= lr * dlogit * cache.rates[i];
            }
            arch.by -= lr * dlogit;
        }
    }
}

#[derive(Default)]
struct ArmAccum {
    rows: Vec<AccumRow>,
}

struct AccumRow {
    arm: &'static str,
    loss_drop: f32,
    loss_drop_rotate: f32,
    elig_energy_capture: f32,
    cosine_vs_ss: f32,
    sign_agree_vs_ss: f32,
}

impl ArmAccum {
    fn add(
        &mut self,
        arm: &'static str,
        loss_drop: f32,
        loss_drop_rotate: f32,
        elig_energy_capture: f32,
        cosine_vs_ss: f32,
        sign_agree_vs_ss: f32,
    ) {
        if let Some(row) = self.rows.iter_mut().find(|r| r.arm == arm) {
            row.loss_drop += loss_drop;
            row.loss_drop_rotate += loss_drop_rotate;
            row.elig_energy_capture += elig_energy_capture;
            row.cosine_vs_ss += cosine_vs_ss;
            row.sign_agree_vs_ss += sign_agree_vs_ss;
        } else {
            self.rows.push(AccumRow {
                arm,
                loss_drop,
                loss_drop_rotate,
                elig_energy_capture,
                cosine_vs_ss,
                sign_agree_vs_ss,
            });
        }
    }

    fn finish(self, n: usize) -> Vec<MechArmMetrics> {
        let inv = 1.0 / n as f32;
        self.rows
            .into_iter()
            .map(|r| MechArmMetrics {
                arm: r.arm,
                loss_drop: r.loss_drop * inv,
                loss_drop_rotate: r.loss_drop_rotate * inv,
                elig_energy_capture: r.elig_energy_capture * inv,
                cosine_vs_ss: r.cosine_vs_ss * inv,
                sign_agree_vs_ss: r.sign_agree_vs_ss * inv,
            })
            .collect()
    }
}

fn evaluate_proposal(
    arch: &MatchedArch,
    x1: &[f32; T],
    x2: &[f32; T],
    y: f32,
    delta: &[f32],
    g_win: &[f32],
    rng: &mut Rng,
) -> (f32, f32, f32, f32) {
    let step = unit_norm(delta, UNIT_STEP);
    let loss0 = example_loss(arch, x1, x2, y);
    let loss1 = example_loss_after(arch, x1, x2, y, &step);
    let loss_drop = loss0 - loss1;

    let mut rotated = step.clone();
    shuffle_inplace(&mut rotated, rng);
    let loss_r = example_loss_after(arch, x1, x2, y, &rotated);
    let loss_drop_rotate = loss0 - loss_r;

    // SuperSpike descends on +g; useful local updates should align with −g.
    let target: Vec<f32> = g_win.iter().map(|g| -*g).collect();
    let cos = cosine(&step, &target);
    let sign = sign_agree(&step, &target);
    (loss_drop, loss_drop_rotate, cos, sign)
}

fn example_loss(arch: &MatchedArch, x1: &[f32; T], x2: &[f32; T], y: f32) -> f32 {
    let p = sigmoid(arch.forward(x1, x2).logit);
    bce(p, y)
}

fn example_loss_after(
    arch: &MatchedArch,
    x1: &[f32; T],
    x2: &[f32; T],
    y: f32,
    dwin: &[f32],
) -> f32 {
    let mut probe = arch.clone();
    for (w, dw) in probe.win.iter_mut().zip(dwin.iter()) {
        *w += *dw;
    }
    example_loss(&probe, x1, x2, y)
}

fn propose_broadcast_pm1(
    arch: &MatchedArch,
    cache: &ForwardCache,
    e_in: &[f32],
    y: f32,
    rng: &mut Rng,
) -> Vec<f32> {
    let p = sigmoid(cache.logit);
    let a = if rng.next_f32() < p { 1.0f32 } else { 0.0 };
    let reward = if (a - y).abs() < 0.5 { 1.0f32 } else { -1.0 };
    // Production three-factor: Δw ∝ M · E (ascent on reward); ignore λ for probe.
    let _ = arch;
    e_in.iter().map(|e| reward * *e).collect()
}

fn propose_graded_broadcast(cache: &ForwardCache, e_in: &[f32], y: f32) -> Vec<f32> {
    let p = sigmoid(cache.logit);
    let teach = -(p - y); // graded broadcast modulator
    e_in.iter().map(|e| teach * *e).collect()
}

fn propose_dfa(cache: &ForwardCache, e_in: &[f32], feedback: &[f32], y: f32) -> Vec<f32> {
    let h = feedback.len();
    let p = sigmoid(cache.logit);
    let teach = -(p - y);
    let mut out = vec![0.0f32; h * N_IN];
    for i in 0..h {
        let m = feedback[i] * teach;
        for j in 0..N_IN {
            out[i * N_IN + j] = m * e_in[i * N_IN + j];
        }
    }
    out
}

fn propose_rl_fb(
    arch: &MatchedArch,
    cache: &ForwardCache,
    e_in: &[f32],
    feedback: &[f32],
    y: f32,
    rng: &mut Rng,
) -> Vec<f32> {
    let h = feedback.len();
    let p = sigmoid(cache.logit);
    let a = if rng.next_f32() < p { 1.0f32 } else { 0.0 };
    let reward = if (a - y).abs() < 0.5 { 1.0f32 } else { -1.0 };
    let direction = reward * (a - p);
    let _ = arch;
    let mut out = vec![0.0f32; h * N_IN];
    for i in 0..h {
        let m = feedback[i] * direction;
        for j in 0..N_IN {
            out[i * N_IN + j] = m * e_in[i * N_IN + j];
        }
    }
    out
}

fn propose_superspike(g_win: &[f32]) -> Vec<f32> {
    // Gradient descent step direction on win: −∇L.
    g_win.iter().map(|g| -*g).collect()
}

fn superspike_dwin(arch: &MatchedArch, cache: &ForwardCache, y: f32) -> Vec<f32> {
    let h = arch.hidden;
    let theta = THETA_REST;
    let alpha = arch.alpha;
    let beta = arch.beta;
    let dlogit = sigmoid(cache.logit) - y;
    let mut g_r = vec![0.0f32; h];
    for i in 0..h {
        g_r[i] = dlogit * arch.wout[i];
    }
    let mut dwin = vec![0.0f32; h * N_IN];
    let mut du_next = vec![0.0f32; h];
    for t in (0..T).rev() {
        let mut du = vec![0.0f32; h];
        for i in 0..h {
            let mut ds = g_r[i] - du_next[i];
            for m in 0..h {
                ds += du_next[m] * arch.wrec[m * h + i];
            }
            let surr = surrogate(cache.u[i][t] - theta, beta);
            du[i] = ds * surr + alpha * du_next[i];
        }
        for i in 0..h {
            dwin[i * N_IN] += du[i] * cache.x[t][0];
            dwin[i * N_IN + 1] += du[i] * cache.x[t][1];
        }
        du_next = du;
    }
    dwin
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

fn energy_capture(g: &[f32], e: &[f32]) -> f32 {
    assert_eq!(g.len(), e.len());
    let mut total = 0.0f32;
    let mut captured = 0.0f32;
    for (gi, ei) in g.iter().zip(e.iter()) {
        let e2 = gi * gi;
        total += e2;
        if ei.abs() > ELIG_EPS {
            captured += e2;
        }
    }
    if total < 1e-20 {
        0.0
    } else {
        captured / total
    }
}

fn unit_norm(v: &[f32], scale: f32) -> Vec<f32> {
    let n2: f32 = v.iter().map(|x| x * x).sum();
    let n = n2.sqrt();
    if n < 1e-20 {
        return vec![0.0; v.len()];
    }
    v.iter().map(|x| x * (scale / n)).collect()
}

fn shuffle_inplace(v: &mut [f32], rng: &mut Rng) {
    for i in (1..v.len()).rev() {
        let j = rng.gen_index(i + 1);
        v.swap(i, j);
    }
}

fn cosine(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len());
    let mut dot = 0.0f32;
    let mut na = 0.0f32;
    let mut nb = 0.0f32;
    for (x, y) in a.iter().zip(b.iter()) {
        dot += x * y;
        na += x * x;
        nb += y * y;
    }
    let den = na.sqrt() * nb.sqrt();
    if den < 1e-20 {
        0.0
    } else {
        (dot / den).clamp(-1.0, 1.0)
    }
}

fn sign_agree(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len());
    let mut agree = 0usize;
    let mut n = 0usize;
    for (x, y) in a.iter().zip(b.iter()) {
        if x.abs() < 1e-12 && y.abs() < 1e-12 {
            continue;
        }
        n += 1;
        if x.signum() == y.signum() {
            agree += 1;
        }
    }
    if n == 0 {
        0.0
    } else {
        agree as f32 / n as f32
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
        let src = include_str!("matched_mech_baseline.rs");
        assert!(src.contains("MUST NEVER BE THE PRODUCTION LEARNER"));
        assert!(src.contains("GC1 exempt") || src.contains("GC1-exempt"));
    }

    #[test]
    fn diagnostic_is_deterministic_and_finite() {
        let train = gen_examples(24, 0xCEC4_0001);
        let a = run_mech_diagnostic(32, DEFAULT_MATCHED_BETA, 0xCEC4_5EED, &train);
        let b = run_mech_diagnostic(32, DEFAULT_MATCHED_BETA, 0xCEC4_5EED, &train);
        assert_eq!(a, b);
        assert_eq!(a.arms.len(), 5);
        for arm in &a.arms {
            assert!(arm.loss_drop.is_finite());
            assert!(arm.loss_drop_rotate.is_finite());
            assert!((0.0..=1.0).contains(&arm.elig_energy_capture));
            assert!((-1.0..=1.0).contains(&arm.cosine_vs_ss));
            assert!((0.0..=1.0).contains(&arm.sign_agree_vs_ss));
        }
    }

    #[test]
    fn superspike_loss_drop_beats_rotate_on_average() {
        let train = gen_examples(64, 0xCEC4_0002);
        let report = run_mech_diagnostic(64, DEFAULT_MATCHED_BETA, 0xCEC4_0055, &train);
        let ss = report
            .arms
            .iter()
            .find(|a| a.arm == MECH_ARM_SUPERSPIKE)
            .expect("superspike arm");
        assert!(
            ss.loss_drop > ss.loss_drop_rotate + 1e-5,
            "SS useful direction should beat rotate: drop={} rot={}",
            ss.loss_drop,
            ss.loss_drop_rotate
        );
    }
}
