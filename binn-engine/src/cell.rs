//! Multi-compartment dendritic LIF cell (U05).
//!
//! Lazy state update: sub-threshold dynamics are integrated exactly only when
//! the cell is touched (`advance_to`). Silent cells cost nothing.
//!
//! Each of [`K`] dendritic compartments holds its own voltage `v_dend` with
//! leak toward its tonic branch current. The soma couples to compartments via
//! conductance [`Cell::g_c`] and alone owns spike threshold / reset.

use binn_core::time::Tick;

/// Cell index into the engine's cell table.
pub type CellId = u32;

/// Number of dendritic compartments per cell.
pub const K: usize = 4;

/// Resting / asymptotic adaptive threshold.
pub const THETA_REST: f32 = 1.0;
/// Adaptive-threshold time constant (ticks).
pub const TAU_THETA: f32 = 20.0;
/// Threshold increment applied on each spike.
pub const DELTA_THETA: f32 = 0.2;
/// Post-spike membrane reset (soma only).
pub const V_RESET: f32 = 0.0;
/// Default membrane (soma) time constant (ticks).
pub const DEFAULT_TAU_M: f32 = 10.0;
/// Default dendritic compartment time constant (ticks).
pub const DEFAULT_TAU_D: f32 = 10.0;
/// Default dendrite→soma coupling conductance.
///
/// With the instantaneous deposit coupling `v += g_c · amount`, `g_c = 1`
/// preserves unit-weight inject → spike at rest threshold.
pub const DEFAULT_G_C: f32 = 1.0;
/// Default voltage impulse for external inject events.
pub const INJECT_WEIGHT: f32 = 1.0;
/// Voltage threshold for dendritic plateau detection.
pub const PLATEAU_THRESHOLD: f32 = 0.8;

/// Multi-compartment dendritic LIF cell with adaptive somatic threshold.
///
/// Sub-threshold dynamics (piecewise-constant branch currents `I[i]`):
///
/// ```text
/// τ_d  dv_dend[i]/dt = −v_dend[i] + I[i]
/// τ_m  dv/dt          = −v + Σ_i g_c · (v_dend[i] − v)
/// ```
///
/// Synaptic events are voltage impulses into a dendritic compartment via
/// [`Cell::deposit`] (not a routing tag onto the soma). Tonic drive is set with
/// [`Cell::set_branch_current`]. Adaptive threshold relaxes toward
/// [`THETA_REST`] with time constant [`TAU_THETA`].
#[derive(Clone, Debug)]
pub struct Cell {
    /// Somatic membrane potential.
    pub v: f32,
    /// Per-compartment dendritic voltages.
    pub v_dend: [f32; K],
    /// Adaptive spike threshold (soma).
    pub theta: f32,
    /// Soma membrane time constant (ticks).
    pub tau_m: f32,
    /// Dendritic compartment time constant (ticks).
    pub tau_d: f32,
    /// Dendrite→soma coupling conductance.
    pub g_c: f32,
    /// Tonic current into each dendritic compartment (piecewise constant).
    pub branches: [f32; K],
    /// Tick of the last state update.
    pub last: Tick,
}

impl Cell {
    /// Resting cell at tick 0 with the given soma membrane time constant.
    ///
    /// Dendritic `τ_d` defaults to [`DEFAULT_TAU_D`]; coupling to [`DEFAULT_G_C`].
    #[inline]
    pub fn new(tau_m: f32) -> Self {
        Self {
            v: 0.0,
            v_dend: [0.0; K],
            theta: THETA_REST,
            tau_m,
            tau_d: DEFAULT_TAU_D,
            g_c: DEFAULT_G_C,
            branches: [0.0; K],
            last: 0,
        }
    }

    /// Resting cell with [`DEFAULT_TAU_M`].
    #[inline]
    pub fn default_params() -> Self {
        Self::new(DEFAULT_TAU_M)
    }

    /// Instantaneous somatic coupling drive `Σ g_c · (v_dend[i] − v)`.
    #[inline]
    pub fn coupling_current(&self) -> f32 {
        let mut s = 0.0f32;
        for &vd in &self.v_dend {
            s += self.g_c * (vd - self.v);
        }
        s
    }

    /// Sum of tonic branch currents (dendritic drive targets).
    #[inline]
    pub fn input(&self) -> f32 {
        let mut s = 0.0f32;
        for &b in &self.branches {
            s += b;
        }
        s
    }

    /// Set the piecewise-constant tonic current into one dendritic compartment.
    pub fn set_branch_current(&mut self, branch: u8, current: f32) {
        let b = branch as usize;
        assert!(b < K, "branch index {branch} out of range (K={K})");
        assert!(current.is_finite(), "branch current must be finite");
        self.branches[b] = current;
    }

    /// Exact lazy integrate of dendrites, soma coupling, and adaptive threshold
    /// from `last` to `t`.
    ///
    /// No-op when `t == last`. Panics if `t < last` (time must be monotone).
    ///
    /// When `g_c = 0`, dendrites and soma decouple: each dendrite is an
    /// independent LIF toward its branch current, and the soma is pure leak.
    pub fn advance_to(&mut self, t: Tick) {
        assert!(
            t >= self.last,
            "Cell::advance_to requires t >= last (t={t}, last={})",
            self.last
        );
        let dt = t - self.last;
        if dt == 0 {
            return;
        }
        let dtf = dt as f32;

        // Dendrites: exact independent LIF toward tonic branch current.
        let decay_d = (-dtf / self.tau_d).exp();
        let mut sum_i = 0.0f32;
        let mut sum_dev0 = 0.0f32;
        for i in 0..K {
            let i_branch = self.branches[i];
            let vd0 = self.v_dend[i];
            sum_i += i_branch;
            sum_dev0 += vd0 - i_branch;
            self.v_dend[i] = i_branch + (vd0 - i_branch) * decay_d;
        }

        // Soma: τ_m v' = −v + Σ g_c (v_d − v) = −(1 + K g_c) v + g_c Σ v_d(t)
        let g = self.g_c;
        if g == 0.0 {
            let decay_s = (-dtf / self.tau_m).exp();
            self.v *= decay_s;
        } else {
            let k_g = (K as f32) * g;
            let alpha = (1.0 + k_g) / self.tau_m;
            let scale = g / self.tau_m;
            // v_d[i](s) = I[i] + (vd0[i]−I[i]) e^{−s/τ_d}
            // v' = −α v + scale · (Σ I + Σ(vd0−I) e^{−s/τ_d})
            let v0 = self.v;
            let e_a = (-alpha * dtf).exp();
            let particular_const = if alpha.abs() < 1e-12 {
                scale * sum_i * dtf
            } else {
                scale * sum_i * (1.0 - e_a) / alpha
            };
            let particular_exp = if sum_dev0.abs() < 1e-20 {
                0.0
            } else {
                let inv_td = 1.0 / self.tau_d;
                let diff = alpha - inv_td;
                let integral = if diff.abs() < 1e-7 {
                    // α ≈ 1/τ_d: ∫_0^dt e^{−α(dt−s)} e^{−s/τ_d} ds = dt · e^{−α dt}
                    dtf * e_a
                } else {
                    // (e^{−dt/τ_d} − e^{−α dt}) / (α − 1/τ_d)
                    (decay_d - e_a) / diff
                };
                scale * sum_dev0 * integral
            };
            self.v = e_a * v0 + particular_const + particular_exp;
        }

        // Adaptive threshold relaxes toward rest.
        let decay_th = (-dtf / TAU_THETA).exp();
        self.theta = THETA_REST + (self.theta - THETA_REST) * decay_th;

        self.last = t;
    }

    /// Apply an instantaneous voltage impulse into dendritic compartment `branch`.
    ///
    /// Updates `v_dend[branch]` (not soma voltage as a routing tag). A fraction
    /// `g_c` of the impulse is coupled onto the soma instantly so same-tick
    /// threshold checks remain meaningful under event delivery. Returns `true`
    /// when the soma crosses threshold.
    ///
    /// Caller must have already advanced the cell to the event tick.
    pub fn deposit(&mut self, branch: u8, amount: f32) -> bool {
        let b = branch as usize;
        assert!(b < K, "branch index {branch} out of range (K={K})");
        assert!(amount.is_finite(), "deposit amount must be finite");
        self.v_dend[b] += amount;
        // Instantaneous dendrite→soma coupling of the impulse.
        self.v += self.g_c * amount;
        self.try_fire()
    }

    /// Spike if soma `v >= theta`: reset soma membrane, raise threshold.
    /// Dendritic voltages are left unchanged. Returns whether a spike was emitted.
    #[inline]
    pub fn try_fire(&mut self) -> bool {
        if self.v < self.theta {
            return false;
        }
        self.v = V_RESET;
        self.theta += DELTA_THETA;
        true
    }

    /// Complete reset of cell state to resting parameters at tick 0.
    pub fn reset(&mut self) {
        self.v = V_RESET;
        self.v_dend = [0.0; K];
        self.theta = THETA_REST;
        self.branches = [0.0; K];
        self.last = 0;
    }

    /// Nonlinear sub-compartment dendritic coincidence score.
    #[inline]
    pub fn dendritic_coincidence_score(&self) -> f32 {
        let mut s = 0.0f32;
        for &vd in &self.v_dend {
            if vd > 0.0 {
                s += vd * vd;
            }
        }
        s
    }

    /// True when dendritic branch `branch` voltage crosses the plateau threshold.
    #[inline]
    pub fn branch_plateau(&self, branch: u8) -> bool {
        let b = branch as usize;
        if b < K {
            self.v_dend[b] >= PLATEAU_THRESHOLD
        } else {
            false
        }
    }
}

/// Analytic single-compartment LIF membrane under constant input `i`:
/// `v(t) = i + (v0 − i) · exp(−t / τ)`.
///
/// Holds for a dendritic compartment, and for the soma when `g_c = 0` (pure leak
/// is the `i = 0` case).
#[inline]
pub fn analytic_lif(v0: f32, i: f32, tau_m: f32, dt: f32) -> f32 {
    i + (v0 - i) * (-dt / tau_m).exp()
}

/// Batch Euler advance for cells that share a common `last` and the same `dt`.
///
/// One forward-Euler step of the multi-compartment system. The event-driven
/// single-cell path uses exact integration ([`Cell::advance_to`]).
///
/// # Panics
///
/// Panics if cells do not all share the same `last` tick.
pub fn batch_advance_euler(cells: &mut [Cell], dt: Tick) {
    if cells.is_empty() || dt == 0 {
        return;
    }
    let last0 = cells[0].last;
    assert!(
        cells.iter().all(|c| c.last == last0),
        "batch_advance_euler requires a shared last tick"
    );

    let dtf = dt as f32;
    let decay_th = (-dtf / TAU_THETA).exp();
    for c in cells.iter_mut() {
        let mut vd_new = [0.0f32; K];
        for (i, vd_slot) in vd_new.iter_mut().enumerate() {
            let vd = c.v_dend[i];
            let i_b = c.branches[i];
            *vd_slot = vd + (dtf / c.tau_d) * (-vd + i_b);
        }
        let mut couple = 0.0f32;
        for &vd in &c.v_dend {
            couple += c.g_c * (vd - c.v);
        }
        c.v += (dtf / c.tau_m) * (-c.v + couple);
        c.v_dend = vd_new;
        c.theta = THETA_REST + (c.theta - THETA_REST) * decay_th;
        c.last = last0 + dt;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ATOL: f32 = 1e-5;

    #[test]
    fn dendrite_analytic_when_uncoupled() {
        // g_c = 0: compartment is an independent LIF; soma is pure leak.
        let tau = 10.0f32;
        let i = 0.5f32;
        let v0 = 0.0f32;

        for &t in &[1u64, 5, 10, 25, 50, 100, 250] {
            let mut cell = Cell::new(tau);
            cell.g_c = 0.0;
            cell.branches[0] = i;
            cell.advance_to(t);

            let expected_d = analytic_lif(v0, i, cell.tau_d, t as f32);
            let err_d = (cell.v_dend[0] - expected_d).abs();
            assert!(
                err_d <= ATOL,
                "t={t}: v_dend={vd} expected={expected_d} err={err_d}",
                vd = cell.v_dend[0]
            );
            assert!(
                cell.v.abs() <= ATOL,
                "t={t}: uncoupled soma must stay at 0, got {}",
                cell.v
            );
        }
    }

    #[test]
    fn soma_pure_leak_when_g_c_zero() {
        let tau = 8.0f32;
        let v0 = 0.75f32;
        let mut cell = Cell::new(tau);
        cell.g_c = 0.0;
        cell.v = v0;
        cell.branches[1] = 0.25; // must not reach soma
        cell.advance_to(40);
        let expected = analytic_lif(v0, 0.0, tau, 40.0);
        assert!((cell.v - expected).abs() <= ATOL);
    }

    #[test]
    fn lazy_update_matches_single_jump() {
        let tau = 12.0f32;
        let i = 0.4f32;

        let mut a = Cell::new(tau);
        a.branches[0] = i;
        a.advance_to(30);
        a.advance_to(70);

        let mut b = Cell::new(tau);
        b.branches[0] = i;
        b.advance_to(70);

        assert!((a.v - b.v).abs() <= ATOL);
        assert!((a.v_dend[0] - b.v_dend[0]).abs() <= ATOL);
        assert!((a.theta - b.theta).abs() <= ATOL);
        assert_eq!(a.last, b.last);
    }

    #[test]
    fn compartment_coupling_drives_soma() {
        // Clamp one dendrite high with tonic current; soma must rise via g_c.
        let mut cell = Cell::new(DEFAULT_TAU_M);
        cell.g_c = 0.5;
        cell.set_branch_current(0, 1.0);
        cell.advance_to(200);

        // Dendrite → ~1.0; soma asymptote: 0 = −v + g_c (1 − v) + (K−1)·g_c(0−v)
        // ⇒ 0 = −v + g_c − K g_c v  ⇒ v = g_c / (1 + K g_c)
        let expected = cell.g_c / (1.0 + (K as f32) * cell.g_c);
        assert!(
            (cell.v_dend[0] - 1.0).abs() < 1e-3,
            "dendrite should track tonic current, got {}",
            cell.v_dend[0]
        );
        assert!(
            (cell.v - expected).abs() < 1e-3,
            "soma asymptote: got {} expected {expected}",
            cell.v
        );
        assert!(cell.v > 0.1, "coupling must raise soma above rest");
    }

    #[test]
    fn deposit_updates_compartment_not_soma_when_uncoupled() {
        let mut cell = Cell::new(DEFAULT_TAU_M);
        cell.g_c = 0.0;
        assert!(!cell.deposit(2, 0.4));
        assert!((cell.v_dend[2] - 0.4).abs() < 1e-6);
        assert!(cell.v.abs() < 1e-6, "g_c=0 deposit must not move soma");
        assert_eq!(cell.branches, [0.0; K]);
    }

    #[test]
    fn adaptive_threshold_rises_on_spike_and_relaxes() {
        let mut cell = Cell::new(DEFAULT_TAU_M);
        let theta0 = cell.theta;
        assert!(
            cell.deposit(0, 2.0),
            "large deposit must spike (g_c={}, v would be {})",
            cell.g_c,
            2.0 * cell.g_c
        );
        assert!(cell.theta > theta0, "threshold must rise on spike");
        let theta_spike = cell.theta;
        // Soma reset; dendrite keeps the deposit.
        assert!((cell.v - V_RESET).abs() < 1e-6);
        assert!((cell.v_dend[0] - 2.0).abs() < 1e-6);

        cell.advance_to(1_000);
        assert!(cell.theta < theta_spike, "threshold must relax after spike");
        assert!(
            (cell.theta - THETA_REST).abs() < 1e-3,
            "threshold should approach rest, got {}",
            cell.theta
        );
    }

    #[test]
    fn no_spike_when_below_threshold() {
        let mut cell = Cell::new(DEFAULT_TAU_M);
        assert!(!cell.deposit(0, 0.5));
        assert!((cell.v - 0.5 * cell.g_c).abs() < 1e-6);
        assert!((cell.v_dend[0] - 0.5).abs() < 1e-6);
        assert!((cell.theta - THETA_REST).abs() < 1e-6);
    }

    #[test]
    fn impulse_does_not_create_permanent_branch_drive() {
        let mut cell = Cell::new(10.0);
        assert!(!cell.deposit(2, 0.25));
        assert_eq!(cell.branches, [0.0; K]);
        cell.advance_to(100);
        assert!(
            cell.v.abs() < 1e-3,
            "soma impulse coupling must decay, got {}",
            cell.v
        );
        assert!(
            cell.v_dend[2].abs() < 1e-3,
            "dendritic impulse must decay, got {}",
            cell.v_dend[2]
        );
    }

    #[test]
    fn batch_euler_advances_shared_last() {
        let mut cells: Vec<Cell> = (0..16)
            .map(|i| {
                let mut c = Cell::new(DEFAULT_TAU_M);
                c.g_c = 0.0; // decouple: dendrite Euler only
                c.branches[0] = 0.1 * (i as f32);
                c
            })
            .collect();
        batch_advance_euler(&mut cells, 1);
        for c in &cells {
            assert_eq!(c.last, 1);
        }
        // Euler one step on dendrite: vd ← vd + (I − vd)·(dt/τ) with vd0=0 ⇒ vd = I/τ
        assert!(cells[0].v_dend[0].abs() < 1e-6);
        assert!((cells[5].v_dend[0] - (0.5 / DEFAULT_TAU_D)).abs() < 1e-5);
        assert!(cells[5].v.abs() < 1e-6);
    }

    #[test]
    #[should_panic(expected = "shared last tick")]
    fn batch_euler_rejects_mixed_last() {
        let mut cells = vec![Cell::new(DEFAULT_TAU_M), Cell::new(DEFAULT_TAU_M)];
        cells[1].last = 3;
        batch_advance_euler(&mut cells, 1);
    }

    #[test]
    fn test_branch_plateau() {
        let mut c = Cell::new(DEFAULT_TAU_M);
        c.v_dend[0] = 0.79;
        c.v_dend[1] = 0.81;
        assert!(!c.branch_plateau(0));
        assert!(c.branch_plateau(1));
        assert!(!c.branch_plateau(99)); // out of bounds check
    }
}
