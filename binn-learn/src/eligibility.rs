//! Eligibility traces (U09).
//!
//! Continuous-time rule on each synapse:
//!
//! ```text
//! de/dt = −e/τ_e + STDP(pre, post)
//! ```
//!
//! Discrete closed-form decay between events: `e ← e · exp(−Δt/τ_e)`.
//! STDP is the classic exponential nearest-neighbor kernel: pre-before-post
//! (Δt = t_post − t_pre > 0) raises the trace; post-before-pre lowers it.

use binn_core::Tick;
use binn_engine::Synapse;

/// LTP amplitude (pre-before-post).
pub const A_PLUS: f32 = 1.0;
/// LTD amplitude (post-before-pre).
pub const A_MINUS: f32 = 1.0;
/// LTP time constant (ticks).
pub const TAU_PLUS: f32 = 20.0;
/// LTD time constant (ticks).
pub const TAU_MINUS: f32 = 20.0;

/// Classic exponential STDP kernel on `dt = t_post − t_pre` (ticks as `f32`).
///
/// - `dt > 0` (pre before post): `+A₊ · exp(−dt/τ₊)`
/// - `dt < 0` (post before pre): `−A₋ · exp(−|dt|/τ₋)`
/// - `dt == 0`: `0`
#[inline]
pub fn stdp(dt: f32) -> f32 {
    if dt > 0.0 {
        A_PLUS * (-dt / TAU_PLUS).exp()
    } else if dt < 0.0 {
        // dt is negative ⇒ exp(dt/τ) = exp(−|dt|/τ)
        -A_MINUS * (dt / TAU_MINUS).exp()
    } else {
        0.0
    }
}

/// Closed-form eligibility decay over `dt` ticks: `e · exp(−dt/τ_e)`.
#[inline]
pub fn decay(e: f32, dt: f32, tau_e: f32) -> f32 {
    debug_assert!(tau_e > 0.0, "tau_e must be positive");
    if dt <= 0.0 {
        return e;
    }
    e * (-dt / tau_e).exp()
}

/// One Euler / event step: decay then add an STDP contribution.
#[inline]
pub fn step(e: f32, dt: f32, tau_e: f32, stdp_contrib: f32) -> f32 {
    decay(e, dt, tau_e) + stdp_contrib
}

/// Eligibility-trace helper bound to a decay time constant.
#[derive(Clone, Debug)]
pub struct Eligibility {
    /// Trace decay time constant `τ_e` (ticks).
    pub tau_e: f32,
}

impl Eligibility {
    /// Construct with decay constant `τ_e`.
    #[inline]
    pub fn new(tau_e: f32) -> Self {
        assert!(tau_e > 0.0, "tau_e must be positive");
        Self { tau_e }
    }

    /// Lazy-decay one synapse from `last_elig_update` to event/now time `t`.
    ///
    /// Updates `eligibility` and advances `last_elig_update` to `t`.
    #[inline]
    pub fn decay_to(&self, syn: &mut Synapse, t: Tick) {
        let dt = if t >= syn.last_elig_update {
            (t - syn.last_elig_update) as f32
        } else {
            0.0
        };
        syn.eligibility = decay(syn.eligibility, dt, self.tau_e);
        syn.last_elig_update = t;
    }

    /// Decay every synapse eligibility by `dt` ticks (optional hygiene).
    ///
    /// Prefer [`Self::decay_to`] for event-time STDP; this does not advance
    /// `last_elig_update`.
    pub fn decay_all(&self, synapses: &mut [Synapse], dt: f32) {
        for syn in synapses {
            syn.eligibility = decay(syn.eligibility, dt, self.tau_e);
        }
    }

    /// Lazy-decay every synapse to absolute time `t` (optional hygiene).
    pub fn decay_all_to(&self, synapses: &mut [Synapse], t: Tick) {
        for syn in synapses {
            self.decay_to(syn, t);
        }
    }

    /// Add STDP(`t_post − t_pre`) onto one synapse (no decay).
    #[inline]
    pub fn add_stdp(&self, syn: &mut Synapse, t_pre: Tick, t_post: Tick) {
        let dt = t_post as f32 - t_pre as f32;
        syn.eligibility += stdp(dt);
    }

    /// Decay by `dt` then add STDP for a single pre/post pair.
    pub fn update_pair(&self, syn: &mut Synapse, dt: f32, t_pre: Tick, t_post: Tick) {
        let contrib = stdp(t_post as f32 - t_pre as f32);
        syn.eligibility = step(syn.eligibility, dt, self.tau_e, contrib);
    }
}

/// STDP kernel weighted by somatic membrane proximity to threshold.
///
/// `contrib = STDP(t_post - t_pre) / (1 + beta * |v_post - theta_post|)^2`
#[inline]
pub fn stdp_surrogate(dt: f32, v_post: f32, theta_post: f32, beta: f32) -> f32 {
    let base = stdp(dt);
    if base == 0.0 {
        return 0.0;
    }
    let dist = (v_post - theta_post).abs();
    let prox = 1.0 / (1.0 + beta * dist).powi(2);
    base * prox
}

/// Surrogate-weighted eligibility trace helper.
#[derive(Clone, Debug)]
pub struct SurrogateEligibility {
    pub tau_e: f32,
    pub beta: f32,
}

impl SurrogateEligibility {
    pub fn new(tau_e: f32, beta: f32) -> Self {
        assert!(tau_e > 0.0, "tau_e must be positive");
        assert!(beta >= 0.0, "beta must be non-negative");
        Self { tau_e, beta }
    }

    #[inline]
    pub fn add_stdp(
        &self,
        syn: &mut Synapse,
        t_pre: Tick,
        t_post: Tick,
        v_post: f32,
        theta_post: f32,
    ) {
        let dt = t_post as f32 - t_pre as f32;
        syn.eligibility += stdp_surrogate(dt, v_post, theta_post, self.beta);
    }
}

/// Dual-timescale eligibility trace: fast (spike timing) + slow (temporal context).
///
/// Combined trace: `e = α · e_fast + (1−α) · e_slow`
/// where `e_fast` decays with `tau_fast` and `e_slow` with `tau_slow`.
///
/// The fast trace captures precise spike-timing correlations.
/// The slow trace maintains longer temporal context for delayed credit.
#[derive(Clone, Debug)]
pub struct DualEligibility {
    pub tau_fast: f32,
    pub tau_slow: f32,
    pub alpha: f32,
}

impl DualEligibility {
    pub fn new(tau_fast: f32, tau_slow: f32, alpha: f32) -> Self {
        assert!(tau_fast > 0.0, "tau_fast must be positive");
        assert!(tau_slow > 0.0, "tau_slow must be positive");
        assert!((0.0..=1.0).contains(&alpha), "alpha must be in [0, 1]");
        Self {
            tau_fast,
            tau_slow,
            alpha,
        }
    }

    #[inline]
    pub fn decay_to(&self, syn: &mut Synapse, t: Tick) {
        let dt = if t >= syn.last_elig_update {
            (t - syn.last_elig_update) as f32
        } else {
            0.0
        };
        let fast_decayed = decay(syn.eligibility, dt, self.tau_fast);
        let slow_decayed = decay(syn.elig_slow, dt, self.tau_slow);
        syn.elig_slow = slow_decayed;
        syn.eligibility = self.alpha * fast_decayed + (1.0 - self.alpha) * slow_decayed;
        syn.last_elig_update = t;
    }

    pub fn decay_all_to(&self, synapses: &mut [Synapse], t: Tick) {
        for syn in synapses {
            self.decay_to(syn, t);
        }
    }

    #[inline]
    pub fn add_stdp(&self, syn: &mut Synapse, t_pre: Tick, t_post: Tick) {
        let dt = t_post as f32 - t_pre as f32;
        let stdp_val = stdp(dt);
        syn.eligibility += stdp_val;
        syn.elig_slow += stdp_val;
    }
}

/// Dendritic plateau-gated eligibility trace helper.
///
/// Eligibility traces are updated ONLY when the synapse's target dendritic branch
/// generates a local plateau potential (v_dend >= threshold).
#[derive(Clone, Debug)]
pub struct PlateauGatedEligibility {
    pub tau_e: f32,
    pub plateau_threshold: f32,
}

impl PlateauGatedEligibility {
    pub fn new(tau_e: f32, plateau_threshold: f32) -> Self {
        assert!(tau_e > 0.0);
        assert!(plateau_threshold > 0.0);
        Self {
            tau_e,
            plateau_threshold,
        }
    }

    #[inline]
    pub fn add_stdp_gated(&self, syn: &mut Synapse, t_pre: Tick, t_post: Tick, branch_v: f32) {
        if branch_v >= self.plateau_threshold {
            let dt = t_post as f32 - t_pre as f32;
            syn.eligibility += stdp(dt);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use binn_engine::Synapse;

    #[test]
    fn pre_before_post_raises_trace() {
        let e = Eligibility::new(50.0);
        let mut syn = Synapse::new(0.5, 1);
        // pre at 10, post at 15 ⇒ Δt = +5 > 0 ⇒ LTP
        e.add_stdp(&mut syn, 10, 15);
        assert!(
            syn.eligibility > 0.0,
            "pre-before-post must raise eligibility, got {}",
            syn.eligibility
        );
        let expected = stdp(5.0);
        assert!((syn.eligibility - expected).abs() < 1e-6);
    }

    #[test]
    fn post_before_pre_lowers_trace() {
        let e = Eligibility::new(50.0);
        let mut syn = Synapse::new(0.5, 1);
        // post at 10, pre at 15 ⇒ Δt = −5 < 0 ⇒ LTD
        e.add_stdp(&mut syn, 15, 10);
        assert!(
            syn.eligibility < 0.0,
            "post-before-pre must lower eligibility, got {}",
            syn.eligibility
        );
        let expected = stdp(-5.0);
        assert!((syn.eligibility - expected).abs() < 1e-6);
    }

    #[test]
    fn decay_matches_closed_form() {
        let tau = 10.0f32;
        let e0 = 1.0f32;
        for &t in &[0.0f32, 1.0, 2.5, 5.0, 10.0, 20.0] {
            let got = decay(e0, t, tau);
            let expected = e0 * (-t / tau).exp();
            assert!(
                (got - expected).abs() < 1e-6,
                "decay mismatch at t={t}: got={got} expected={expected}"
            );
        }
    }

    #[test]
    fn step_decay_then_stdp() {
        let tau = 20.0f32;
        let e0 = 0.5f32;
        let dt = 4.0f32;
        let contrib = stdp(3.0);
        let got = step(e0, dt, tau, contrib);
        let expected = e0 * (-dt / tau).exp() + contrib;
        assert!((got - expected).abs() < 1e-6);
    }

    #[test]
    fn decay_to_uses_last_elig_update() {
        let e = Eligibility::new(10.0);
        let mut syn = Synapse::new(0.5, 1);
        syn.eligibility = 1.0;
        syn.last_elig_update = 0;
        e.decay_to(&mut syn, 5);
        let expected = decay(1.0, 5.0, 10.0);
        assert!((syn.eligibility - expected).abs() < 1e-6);
        assert_eq!(syn.last_elig_update, 5);
        // Second call at same t is a no-op.
        e.decay_to(&mut syn, 5);
        assert!((syn.eligibility - expected).abs() < 1e-6);
    }

    #[test]
    fn stdp_surrogate_weights_by_membrane_proximity() {
        let base_stdp = stdp(5.0);
        let near = stdp_surrogate(5.0, 0.9, 1.0, 1.0); // dist = 0.1
        let far = stdp_surrogate(5.0, 0.0, 1.0, 1.0); // dist = 1.0
        assert!(
            near > far,
            "near threshold should have higher surrogate eligibility"
        );
        assert!(
            near < base_stdp,
            "surrogate proximity should attenuate STDP when dist > 0"
        );
    }

    #[test]
    fn dual_eligibility_combines_traces() {
        let de = DualEligibility::new(10.0, 100.0, 0.5);
        let mut syn = Synapse::new(0.5, 1);
        syn.eligibility = 1.0;
        syn.elig_slow = 1.0;
        syn.last_elig_update = 0;

        de.decay_to(&mut syn, 10);

        let expected_fast = decay(1.0, 10.0, 10.0);
        let expected_slow = decay(1.0, 10.0, 100.0);
        let expected_combined = 0.5 * expected_fast + 0.5 * expected_slow;

        assert!((syn.elig_slow - expected_slow).abs() < 1e-6);
        assert!((syn.eligibility - expected_combined).abs() < 1e-6);
    }

    #[test]
    fn dual_eligibility_adds_stdp_to_both() {
        let de = DualEligibility::new(10.0, 100.0, 0.5);
        let mut syn = Synapse::new(0.5, 1);
        syn.eligibility = 0.0;
        syn.elig_slow = 0.0;

        de.add_stdp(&mut syn, 10, 15); // pre before post -> dt = 5

        let expected = stdp(5.0);
        assert!((syn.eligibility - expected).abs() < 1e-6);
        assert!((syn.elig_slow - expected).abs() < 1e-6);
    }

    #[test]
    fn test_plateau_gated_eligibility() {
        let e = PlateauGatedEligibility::new(20.0, 0.8);
        let mut syn = Synapse::new(0.5, 1);
        syn.eligibility = 0.0;

        // Under threshold: no STDP added
        e.add_stdp_gated(&mut syn, 10, 15, 0.7);
        assert_eq!(syn.eligibility, 0.0);

        // Above threshold: STDP added
        e.add_stdp_gated(&mut syn, 10, 15, 0.9);
        assert!(syn.eligibility > 0.0);
    }
}
