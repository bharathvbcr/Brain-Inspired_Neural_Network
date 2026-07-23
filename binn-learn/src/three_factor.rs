//! Online three-factor plasticity (U10).
//!
//! ```text
//! Δw = η · e · M − λ · w
//! ```
//!
//! Forward-only: eligibility lives on synapses; modulators are broadcast.
//! Memory is **O(1) in sequence length** (state is O(cells + synapses)).
//! Eligibility is lazy-decayed per synapse from `last_elig_update` to event/now
//! time; postsynaptic fan-in uses the engine CSC reverse index.

use binn_core::{Csc, Csr, Tick};
use binn_engine::{CellId, Engine};

use crate::eligibility::{self, Eligibility};
use crate::modulators::Modulators;

/// Production learning interface (no backward pass).
pub trait Learner {
    /// Apply one plasticity update given the current engine state and modulators.
    fn update(&mut self, engine: &mut Engine, m: Modulators);
}

/// Online three-factor learner: eligibility × modulator − weight decay.
#[derive(Clone, Debug)]
pub struct ThreeFactor {
    /// Learning rate `η`.
    pub eta: f32,
    /// Weight decay `λ`.
    pub lambda: f32,
    /// Eligibility decay constant `τ_e`.
    pub tau_e: f32,
    use_eligibility: bool,
    use_modulator: bool,
    last_update: Tick,
    spike_cursor: usize,
    last_spike: Vec<Option<Tick>>,
}

impl ThreeFactor {
    /// Construct the production learner.
    pub fn new(eta: f32, lambda: f32, tau_e: f32) -> Self {
        assert!(tau_e > 0.0, "tau_e must be positive");
        Self {
            eta,
            lambda,
            tau_e,
            use_eligibility: true,
            use_modulator: true,
            last_update: 0,
            spike_cursor: 0,
            last_spike: Vec::new(),
        }
    }

    /// Ablation: ignore eligibility (`e ≡ 0`).
    #[inline]
    pub fn without_eligibility(mut self) -> Self {
        self.use_eligibility = false;
        self
    }

    /// Ablation: ignore modulators (`M ≡ 0`).
    #[inline]
    pub fn without_modulator(mut self) -> Self {
        self.use_modulator = false;
        self
    }

    /// Resident bytes attributable to the learner (excludes engine synapses).
    pub fn resident_bytes(&self) -> usize {
        std::mem::size_of::<Self>()
            + self.last_spike.capacity() * std::mem::size_of::<Option<Tick>>()
    }

    /// Number of cells tracked by the last-spike table.
    #[inline]
    pub fn tracked_cells(&self) -> usize {
        self.last_spike.len()
    }

    fn ensure_cells(&mut self, n: usize) {
        if self.last_spike.len() < n {
            self.last_spike.resize(n, None);
        }
    }

    fn absorb_spikes(&mut self, engine: &mut Engine) {
        let n = engine.num_cells();
        self.ensure_cells(n);
        let elig = Eligibility::new(self.tau_e);

        let spikes = engine.spikes().as_slice();
        if self.spike_cursor > spikes.len() {
            self.spike_cursor = 0;
        }
        // Copy new spike identities so we can mutably borrow synapses next.
        // Sort by time so per-synapse lazy decay sees events chronologically.
        let mut new_spikes: Vec<(CellId, Tick)> = spikes[self.spike_cursor..]
            .iter()
            .map(|sp| (sp.cell, sp.t))
            .collect();
        new_spikes.sort_by_key(|&(_, t)| t);
        self.spike_cursor = spikes.len();
        let conn = engine.conn.clone();
        let conn_rev = engine.conn_rev.clone();
        for (cell, t) in new_spikes {
            apply_spike_stdp(
                engine.syn.as_mut_slice(),
                &conn,
                &conn_rev,
                &mut self.last_spike,
                &elig,
                cell,
                t,
            );
        }
    }

    fn apply_weights(&self, engine: &mut Engine, m: Modulators, now: Tick) -> u64 {
        let gate = if self.use_modulator { m.scalar() } else { 0.0 };
        let elig = Eligibility::new(self.tau_e);
        let syns = engine.syn.as_mut_slice();
        assert_eq!(syns.len(), engine.edge_w.len());
        let n = syns.len() as u64;
        for (i, syn) in syns.iter_mut().enumerate() {
            // Decay to weight-apply time from last STDP/touch.
            elig.decay_to(syn, now);
            let e = if self.use_eligibility {
                syn.eligibility
            } else {
                0.0
            };
            let dw = self.eta * e * gate - self.lambda * syn.weight;
            syn.weight += dw;
            engine.edge_w[i] = syn.weight;
        }
        n
    }

    /// Absorb spikes, apply weights, and return synapse applications (~nnz).
    pub fn update_counted(&mut self, engine: &mut Engine, m: Modulators) -> u64 {
        let now = engine.time();
        self.absorb_spikes(engine);
        let n = self.apply_weights(engine, m, now);
        self.last_update = now;
        n
    }
}

impl Learner for ThreeFactor {
    fn update(&mut self, engine: &mut Engine, m: Modulators) {
        let _ = self.update_counted(engine, m);
    }
}

#[allow(clippy::needless_range_loop)]
fn apply_spike_stdp(
    syns: &mut [binn_engine::Synapse],
    conn: &Csr,
    conn_rev: &Csc,
    last_spike: &mut [Option<Tick>],
    elig: &Eligibility,
    cell: CellId,
    t: Tick,
) {
    let c = cell as usize;
    assert!(c < last_spike.len());

    // Presynaptic spike: STDP on outgoing CSR edges (pre → post).
    if c < conn.nrows() {
        let start = conn.row_ptr[c] as usize;
        let end = conn.row_ptr[c + 1] as usize;
        for e in start..end {
            let post = conn.col[e] as usize;
            elig.decay_to(&mut syns[e], t);
            if let Some(t_post) = last_spike[post] {
                syns[e].eligibility += eligibility::stdp(t_post as f32 - t as f32);
            }
        }
    }

    // Postsynaptic spike: STDP on incoming CSC edges (pre → cell).
    if c < conn_rev.ncols() {
        for (pre, edge) in conn_rev.incoming(c) {
            let e = edge as usize;
            elig.decay_to(&mut syns[e], t);
            if let Some(t_pre) = last_spike[pre as usize] {
                syns[e].eligibility += eligibility::stdp(t as f32 - t_pre as f32);
            }
        }
    }

    last_spike[c] = Some(t);
}

/// Tiny coincidence wiring for acceptance tests.
///
/// Cells: `0 = pre_a`, `1 = pre_b`, `2 = post`, `3 = distractor`.
/// Edges: `0→2`, `1→2`, `3→2`.
pub fn coincidence_engine(init_w: f32) -> Engine {
    let mut eng = Engine::with_cells(4);
    let row_ptr = vec![0u32, 1, 2, 2, 3];
    let col = vec![2u32, 2, 2];
    let conn = Csr::from_parts(row_ptr, col).expect("coincidence CSR");
    eng.set_connectivity(conn, vec![init_w; 3]);
    eng
}

pub const EDGE_A: usize = 0;
pub const EDGE_B: usize = 1;
pub const EDGE_DIST: usize = 2;

pub fn run_coincidence_trial(
    engine: &mut Engine,
    learner: &mut ThreeFactor,
    m: Modulators,
    t0: Tick,
) {
    engine.inject(0, 0, t0);
    engine.inject(1, 0, t0 + 1);
    engine.inject(2, 0, t0 + 2);
    engine.step_until(t0 + 5);
    learner.update(engine, m);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eligibility::decay;

    #[test]
    fn coincidence_moves_eligible_modulated_weights() {
        let mut eng = coincidence_engine(0.1);
        let mut learner = ThreeFactor::new(0.5, 0.0, 40.0);
        let w0: Vec<f32> = eng.edge_w.clone();

        for i in 0..8u64 {
            run_coincidence_trial(&mut eng, &mut learner, Modulators::reward(1.0), 10 + i * 20);
        }

        let dw_a = eng.edge_w[EDGE_A] - w0[EDGE_A];
        let dw_b = eng.edge_w[EDGE_B] - w0[EDGE_B];
        let dw_d = eng.edge_w[EDGE_DIST] - w0[EDGE_DIST];

        assert!(
            dw_a > 0.0 && dw_b > 0.0,
            "causal edges must potentiate: {dw_a} {dw_b}"
        );
        assert!(
            dw_a > dw_d && dw_b > dw_d,
            "distractor must move less: {dw_d}"
        );
    }

    #[test]
    fn weights_move_only_where_eligibility_and_modulation_coincide() {
        let mut eng_m0 = coincidence_engine(0.2);
        let mut learn_m0 = ThreeFactor::new(0.5, 0.0, 40.0);
        let w0 = eng_m0.edge_w[EDGE_A];
        run_coincidence_trial(&mut eng_m0, &mut learn_m0, Modulators::zero(), 10);
        assert!((eng_m0.edge_w[EDGE_A] - w0).abs() < 1e-6);

        let mut eng_e0 = coincidence_engine(0.2);
        let mut learn_e0 = ThreeFactor::new(0.5, 0.0, 40.0).without_eligibility();
        let w0e = eng_e0.edge_w[EDGE_A];
        run_coincidence_trial(&mut eng_e0, &mut learn_e0, Modulators::reward(1.0), 10);
        assert!((eng_e0.edge_w[EDGE_A] - w0e).abs() < 1e-6);

        let mut eng = coincidence_engine(0.2);
        let mut learn = ThreeFactor::new(0.5, 0.0, 40.0);
        let w0f = eng.edge_w[EDGE_A];
        run_coincidence_trial(&mut eng, &mut learn, Modulators::reward(1.0), 10);
        assert!(eng.edge_w[EDGE_A] > w0f);
    }

    #[test]
    fn ablation_no_modulator_degrades() {
        let mut full = coincidence_engine(0.1);
        let mut learn_full = ThreeFactor::new(0.5, 0.0, 40.0);
        let mut ablated = coincidence_engine(0.1);
        let mut learn_abl = ThreeFactor::new(0.5, 0.0, 40.0).without_modulator();

        for i in 0..6u64 {
            let t = 10 + i * 20;
            run_coincidence_trial(&mut full, &mut learn_full, Modulators::reward(1.0), t);
            run_coincidence_trial(&mut ablated, &mut learn_abl, Modulators::reward(1.0), t);
        }

        let gain_full = full.edge_w[EDGE_A] - 0.1;
        let gain_abl = ablated.edge_w[EDGE_A] - 0.1;
        assert!(
            gain_full > gain_abl + 1e-4,
            "full={gain_full} abl={gain_abl}"
        );
    }

    #[test]
    fn ablation_no_eligibility_degrades() {
        let mut full = coincidence_engine(0.1);
        let mut learn_full = ThreeFactor::new(0.5, 0.0, 40.0);
        let mut ablated = coincidence_engine(0.1);
        let mut learn_abl = ThreeFactor::new(0.5, 0.0, 40.0).without_eligibility();

        for i in 0..6u64 {
            let t = 10 + i * 20;
            run_coincidence_trial(&mut full, &mut learn_full, Modulators::reward(1.0), t);
            run_coincidence_trial(&mut ablated, &mut learn_abl, Modulators::reward(1.0), t);
        }

        let gain_full = full.edge_w[EDGE_A] - 0.1;
        let gain_abl = ablated.edge_w[EDGE_A] - 0.1;
        assert!(
            gain_full > gain_abl + 1e-4,
            "full={gain_full} abl={gain_abl}"
        );
    }

    #[test]
    fn memory_flat_in_sequence_length() {
        let mut short = coincidence_engine(0.1);
        let mut learn_short = ThreeFactor::new(0.2, 0.0, 30.0);
        for i in 0..4u64 {
            run_coincidence_trial(
                &mut short,
                &mut learn_short,
                Modulators::reward(1.0),
                10 + i * 20,
            );
        }
        let bytes_short = learn_short.resident_bytes();
        let cells_short = learn_short.tracked_cells();

        let mut long = coincidence_engine(0.1);
        let mut learn_long = ThreeFactor::new(0.2, 0.0, 30.0);
        for i in 0..200u64 {
            run_coincidence_trial(
                &mut long,
                &mut learn_long,
                Modulators::reward(1.0),
                10 + i * 20,
            );
        }
        let bytes_long = learn_long.resident_bytes();
        let cells_long = learn_long.tracked_cells();

        assert_eq!(cells_short, cells_long);
        assert_eq!(bytes_short, bytes_long);
        assert!(long.spikes().len() > short.spikes().len());
    }

    /// STDP contributions are decayed from last synapse touch / event time,
    /// not from a single trial-global `decay_all(dt)` before historical adds.
    #[test]
    fn stdp_uses_per_synapse_event_time_decay() {
        let tau_e = 20.0f32;
        // Zero weights: forced spikes only (no synaptic fan-out noise).
        let mut eng = coincidence_engine(0.0);
        let mut learner = ThreeFactor::new(0.0, 0.0, tau_e);

        // pre=0 at t=10, post=2 at t=20 ⇒ LTP on EDGE_A, then pre again at t=30 ⇒ LTD
        // with inter-event decay of the first contribution.
        eng.force_spike(0, 10);
        eng.step_until(10);
        eng.force_spike(2, 20);
        eng.step_until(20);
        eng.force_spike(0, 30);
        eng.step_until(30);

        learner.update(&mut eng, Modulators::zero());

        // Event-time path:
        //   t=20 (post): e = stdp(20-10)
        //   t=30 (pre):  e = decay(stdp(10), 10, tau) + stdp(20-30)
        // Weight apply at now=30: last_elig_update already 30 ⇒ no further decay.
        let expected = decay(eligibility::stdp(10.0), 10.0, tau_e) + eligibility::stdp(-10.0);
        let got = eng.syn.as_slice()[EDGE_A].eligibility;
        assert!(
            (got - expected).abs() < 1e-5,
            "event-time decay mismatch: got={got} expected={expected} (old trial-global would be {})",
            eligibility::stdp(10.0) + eligibility::stdp(-10.0)
        );
        // Must differ from the no-inter-event-decay (trial-global) value.
        let no_inter = eligibility::stdp(10.0) + eligibility::stdp(-10.0);
        assert!(
            (got - no_inter).abs() > 1e-4,
            "must not match trial-global no-inter-event decay"
        );
    }

    #[test]
    fn conn_rev_covers_coincidence_fan_in() {
        let eng = coincidence_engine(0.1);
        let into_post: Vec<_> = eng.conn_rev.incoming(2).collect();
        assert_eq!(into_post, vec![(0, 0), (1, 1), (3, 2)]);
        assert_eq!(eng.conn_rev.nnz(), eng.conn.nnz());
    }
}
