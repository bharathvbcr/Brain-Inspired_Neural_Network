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
//!
//! # Counterfactual eligibility
//!
//! Every trace above is built from spikes the engine actually emitted. Under
//! hard k-WTA that is a strictly smaller set than the cells that competed: a
//! loser is muted before it can fire, so its afferent synapses end the trial at
//! `e = 0` and `Δw = η · e · M` is zero for it *whatever* credit `M` says. A
//! rule that only reweights `M` therefore cannot move a near-miss, and measures
//! nothing when applied to one.
//!
//! [`CandidateSpike`] is the missing half. It credits a non-winner's **afferent**
//! synapses with a scaled share of the STDP a real spike at the same tick would
//! have written, without emitting a spike, without touching the engine, and
//! without entering the pairing table. Forward dynamics stay bit-identical; only
//! the trace the credit signal multiplies gets richer.

use binn_core::{Csc, Csr, Tick};
use binn_engine::{CellId, Engine};

use crate::eligibility::{self, Eligibility};
use crate::modulators::Modulators;
use crate::CreditSignal;

/// A near-miss postsynaptic event: credit for a spike that competition prevented.
///
/// `cell` lost k-WTA, so the engine's spike log has no record of it and its
/// afferent synapses would otherwise carry no eligibility at all. Queuing one of
/// these on [`ThreeFactor::queue_candidate_spikes`] deposits `scale` times the
/// STDP that a real spike at `t` would have written on those synapses.
///
/// `scale` is the caller's margin term: how nearly this cell won. It is applied
/// to the eligibility, not to the credit signal, so the winners' updates are
/// left exactly as they were and a `scale` of zero is indistinguishable from not
/// running the mechanism.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CandidateSpike {
    /// The cell that lost selection.
    pub cell: CellId,
    /// The tick the winners fired — the time this cell would have fired.
    pub t: Tick,
    /// Fraction of a real spike's afferent STDP to deposit. Finite and `> 0`.
    pub scale: f32,
}

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
    /// Near-miss events to fold into the next absorb, then drained.
    pending_candidates: Vec<CandidateSpike>,
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
            pending_candidates: Vec::new(),
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
            + self.pending_candidates.capacity() * std::mem::size_of::<CandidateSpike>()
    }

    /// Number of cells tracked by the last-spike table.
    #[inline]
    pub fn tracked_cells(&self) -> usize {
        self.last_spike.len()
    }

    /// Clear nearest-neighbor STDP pairing times (`last_spike`).
    ///
    /// Call at trial boundaries when the scientific protocol requires
    /// trial-isolated eligibility. Does **not** rewind `spike_cursor` (already
    /// absorbed spikes stay consumed) and does **not** zero synapse eligibility
    /// traces — pair with the harness `clear_eligibility` helper for that.
    ///
    /// Canonical C1 protocol v2 does **not** call this; isolation schedules
    /// (e.g. `c1-iso` / protocol v5) do.
    pub fn reset_pairing_state(&mut self) {
        self.last_spike.fill(None);
        // A queued near-miss belongs to the selection event that produced it. If
        // the trial ended without an absorb, it must not leak into the next one
        // and credit a cell for losing a competition that is already over.
        self.pending_candidates.clear();
    }

    /// Fully reset pairing state and spike cursor for clean trial boundaries.
    pub fn reset_full_trial_state(&mut self) {
        self.reset_pairing_state();
        self.spike_cursor = 0;
        self.last_update = 0;
    }

    /// Queue near-miss events for the next absorb, which drains them.
    ///
    /// Entries with a non-positive or non-finite `scale` are dropped rather than
    /// queued: a zero-scale candidate adds no eligibility but *would* still
    /// advance its synapses' lazy-decay clock to `t`, splitting one decay into
    /// two and perturbing the last bits of every afferent trace. Dropping them
    /// is what makes "the mechanism off" and "the mechanism on at `λ_c = 0`"
    /// the same numbers rather than merely close ones.
    ///
    /// Returns the number actually queued, which the caller must compare against
    /// what it offered before reporting coverage.
    pub fn queue_candidate_spikes(
        &mut self,
        candidates: impl IntoIterator<Item = CandidateSpike>,
    ) -> usize {
        let before = self.pending_candidates.len();
        for candidate in candidates {
            if candidate.scale.is_finite() && candidate.scale > 0.0 {
                self.pending_candidates.push(candidate);
            }
        }
        self.pending_candidates.len() - before
    }

    /// Near-miss events queued but not yet absorbed (tests / diagnostics).
    #[inline]
    pub fn pending_candidate_count(&self) -> usize {
        self.pending_candidates.len()
    }

    /// Peek last-spike table (tests / diagnostics).
    #[inline]
    pub fn last_spike_at(&self, cell: usize) -> Option<Tick> {
        self.last_spike.get(cell).copied().flatten()
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
        // Topology is immutable for the duration of this loop, so borrow it
        // rather than deep-cloning it.
        //
        // This previously read `engine.conn.clone()` / `engine.conn_rev.clone()`.
        // That copied the entire CSR *and* CSC — roughly `3·nnz + nrows + 2·ncols`
        // u32s across 5 heap allocations — on every plasticity step, purely to
        // dodge a borrow conflict with `engine.syn.as_mut_slice()` below. At
        // nnz ≈ 2.5e6 that is ~30 MB of memcpy per step, which on M5 Pro memory
        // bandwidth is on the order of a millisecond of pure waste per step.
        //
        // `syn`, `conn` and `conn_rev` are separate public fields of `Engine`, so
        // NLL accepts the disjoint borrows: `conn`/`conn_rev` shared, `syn` unique.
        // No numerics change — same values, same order, same iteration.
        let conn = &engine.conn;
        let conn_rev = &engine.conn_rev;

        if self.pending_candidates.is_empty() {
            for (cell, t) in new_spikes {
                apply_spike_stdp(
                    engine.syn.as_mut_slice(),
                    conn,
                    conn_rev,
                    &mut self.last_spike,
                    &elig,
                    cell,
                    t,
                );
            }
            return;
        }

        // Near-miss deposits are interleaved into the same chronological stream
        // rather than appended after it. `Eligibility::decay_to` rewinds a
        // synapse's decay clock to whatever time it is handed, so depositing at
        // `winner_at` *after* absorbing a later action spike would re-decay that
        // synapse from an earlier instant and over-decay every trace on it.
        //
        // `new_spikes` is already sorted by time and `sort_by_key` is stable, so
        // real events keep their engine-log order among themselves and a real
        // event precedes a candidate at the same tick — a candidate is a spike
        // that did not happen, and may not pre-empt one that did.
        let mut events: Vec<(Tick, u8, CellId, f32)> = new_spikes
            .into_iter()
            .map(|(cell, t)| (t, 0u8, cell, 0.0))
            .collect();
        events.extend(
            self.pending_candidates
                .iter()
                .map(|c| (c.t, 1u8, c.cell, c.scale)),
        );
        events.sort_by_key(|&(t, is_candidate, _, _)| (t, is_candidate));

        for (t, is_candidate, cell, scale) in events {
            if is_candidate == 0 {
                apply_spike_stdp(
                    engine.syn.as_mut_slice(),
                    conn,
                    conn_rev,
                    &mut self.last_spike,
                    &elig,
                    cell,
                    t,
                );
            } else {
                apply_candidate_stdp(
                    engine.syn.as_mut_slice(),
                    conn_rev,
                    &self.last_spike,
                    &elig,
                    cell,
                    t,
                    scale,
                );
            }
        }
        self.pending_candidates.clear();
    }

    fn apply_weights<S: CreditSignal>(&self, engine: &mut Engine, signal: &S, now: Tick) -> u64 {
        let elig = Eligibility::new(self.tau_e);
        let posts = &engine.conn.col;
        let syns = engine.syn.as_mut_slice();
        assert_eq!(syns.len(), engine.edge_w.len());
        assert_eq!(posts.len(), syns.len());
        let n = syns.len() as u64;
        for (i, syn) in syns.iter_mut().enumerate() {
            // Decay to weight-apply time from last STDP/touch.
            elig.decay_to(syn, now);
            let e = if self.use_eligibility {
                syn.eligibility
            } else {
                0.0
            };
            // Decay only synapses with nonzero eligibility so idle readout edges
            // are not bled to zero between sparse credit events (BUILD_AUDIT_v10 A3).
            let decay_term = if e.abs() > 1e-8 {
                self.lambda * syn.weight
            } else {
                0.0
            };
            let gate = if self.use_modulator {
                signal.for_post(posts[i])
            } else {
                0.0
            };
            let dw = self.eta * e * gate - decay_term;
            syn.weight += dw;
            engine.edge_w[i] = syn.weight;
        }
        n
    }

    /// Absorb spikes, apply weights, and return synapse applications (~nnz).
    pub fn update_counted(&mut self, engine: &mut Engine, m: Modulators) -> u64 {
        self.update_with_credit_counted(engine, &m)
    }

    /// Absorb spikes and apply an explicit postsynaptic credit signal.
    pub fn update_with_credit_counted<S: CreditSignal>(
        &mut self,
        engine: &mut Engine,
        signal: &S,
    ) -> u64 {
        let now = engine.time();
        self.absorb_spikes(engine);
        let n = self.apply_weights(engine, signal, now);
        self.last_update = now;
        n
    }

    /// Advance eligibility from newly observed spikes without changing weights.
    ///
    /// Experimental matched-forward arms use this to consume identical
    /// action/target spikes even when their learning rule updates weights
    /// directly instead of through [`ThreeFactor`].
    pub fn observe_spikes(&mut self, engine: &mut Engine) {
        self.absorb_spikes(engine);
        self.last_update = engine.time();
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

/// Afferent-only STDP for a spike that competition prevented.
///
/// Deliberately narrower than [`apply_spike_stdp`] in two ways, both of which
/// are what keeps the forward pass intact:
///
/// * **No efferent (CSR) edges.** Those carry `pre → post` credit for a spike
///   this cell never delivered. Writing them would let a cell that stayed silent
///   depress or potentiate its targets, which is a change to the network's
///   behaviour and not to its credit assignment.
/// * **No `last_spike` write.** The pairing table is the forward path's memory
///   of what fired. A phantom entry there would re-time the STDP of every
///   *real* spike that pairs against this cell afterwards, so the arm would no
///   longer share a forward pass with the one it is compared to.
///
/// What remains is exactly the counterfactual question: of the inputs that
/// arrived, which ones would have been responsible had this cell won?
#[allow(clippy::needless_range_loop)]
fn apply_candidate_stdp(
    syns: &mut [binn_engine::Synapse],
    conn_rev: &Csc,
    last_spike: &[Option<Tick>],
    elig: &Eligibility,
    cell: CellId,
    t: Tick,
    scale: f32,
) {
    debug_assert!(scale.is_finite() && scale > 0.0, "scale must be positive");
    let c = cell as usize;
    if c >= conn_rev.ncols() {
        return;
    }
    for (pre, edge) in conn_rev.incoming(c) {
        let e = edge as usize;
        elig.decay_to(&mut syns[e], t);
        if let Some(t_pre) = last_spike[pre as usize] {
            syns[e].eligibility += scale * eligibility::stdp(t as f32 - t_pre as f32);
        }
    }
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
mod counterfactual_tests {
    use super::*;
    use crate::credit::PostSynapticCredit;
    use binn_core::Csr;

    /// Two inputs (0, 1) drive two competitors (2, 3); each competitor drives a
    /// shared downstream cell (4).
    ///
    /// Edge order is fixed and named below because the assertions address edges
    /// by index: 0→2, 0→3, 1→2, 1→3, 2→4, 3→4.
    const E_0_2: usize = 0;
    const E_0_3: usize = 1;
    const E_1_2: usize = 2;
    const E_1_3: usize = 3;
    const E_2_4: usize = 4;
    const E_3_4: usize = 5;

    const WINNER: CellId = 2;
    const LOSER: CellId = 3;

    fn competition_engine() -> Engine {
        let mut eng = Engine::with_cells(5);
        let row_ptr = vec![0u32, 2, 4, 5, 6, 6];
        let col = vec![2u32, 3, 2, 3, 4, 4];
        let conn = Csr::from_parts(row_ptr, col).expect("competition CSR");
        eng.set_connectivity(conn, vec![0.1; 6]);
        eng
    }

    /// Drive both inputs, then fire only the k-WTA winner. `LOSER` competed and
    /// lost, so it never spikes — exactly the live C1 sequence, where hidden
    /// thresholds are muted during integrate and only winners are force-fired.
    fn run_selection(eng: &mut Engine) {
        eng.force_spike(0, 10);
        eng.force_spike(1, 11);
        let _ = eng.step_until(19);
        eng.force_spike(WINNER, 20);
        let _ = eng.step_until(30);
    }

    /// Credit large enough that any non-zero eligibility would be visible.
    fn credit_for(cell: CellId) -> PostSynapticCredit {
        let mut signal = PostSynapticCredit::zeros(5);
        signal.set(cell, 1000.0);
        signal
    }

    fn elig_bits(eng: &Engine) -> Vec<u32> {
        eng.syn
            .as_slice()
            .iter()
            .map(|s| s.eligibility.to_bits())
            .collect()
    }

    fn weight_bits(eng: &Engine) -> Vec<u32> {
        eng.edge_w.iter().map(|w| w.to_bits()).collect()
    }

    #[test]
    fn a_loser_cannot_be_taught_by_credit_alone() {
        // The defect the counterfactual term exists to fix, stated as a test on
        // the mechanism-off path: eligibility is built from emitted spikes, the
        // loser emitted none, so `Δw = η · e · M` is zero on its afferents no
        // matter how large `M` is. Any rule that only reshapes `M` — margin
        // scaling included — is a no-op here and measures nothing.
        let mut eng = competition_engine();
        let mut learner = ThreeFactor::new(0.5, 0.0, 40.0);
        let w0 = eng.edge_w.clone();

        run_selection(&mut eng);
        learner.update_with_credit_counted(&mut eng, &credit_for(LOSER));

        assert_eq!(
            eng.edge_w[E_0_3].to_bits(),
            w0[E_0_3].to_bits(),
            "afferent 0->loser moved without the counterfactual term"
        );
        assert_eq!(
            eng.edge_w[E_1_3].to_bits(),
            w0[E_1_3].to_bits(),
            "afferent 1->loser moved without the counterfactual term"
        );
        // The same credit on the winner does move weights, so the null above is
        // about the loser's missing eligibility and not about a dead harness.
        let mut eng_w = competition_engine();
        let mut learner_w = ThreeFactor::new(0.5, 0.0, 40.0);
        run_selection(&mut eng_w);
        learner_w.update_with_credit_counted(&mut eng_w, &credit_for(WINNER));
        assert!(
            eng_w.edge_w[E_0_2] > w0[E_0_2],
            "winner afferent did not potentiate: harness is not measuring anything"
        );
    }

    #[test]
    fn a_candidate_spike_credits_the_losers_afferents() {
        let mut eng = competition_engine();
        let mut learner = ThreeFactor::new(0.5, 0.0, 40.0);
        let w0 = eng.edge_w.clone();

        run_selection(&mut eng);
        let queued = learner.queue_candidate_spikes([CandidateSpike {
            cell: LOSER,
            t: 20,
            scale: 1.0,
        }]);
        assert_eq!(queued, 1);
        learner.update_with_credit_counted(&mut eng, &credit_for(LOSER));

        assert!(
            eng.edge_w[E_0_3] > w0[E_0_3],
            "afferent 0->loser did not potentiate under counterfactual credit"
        );
        assert!(
            eng.edge_w[E_1_3] > w0[E_1_3],
            "afferent 1->loser did not potentiate under counterfactual credit"
        );
    }

    #[test]
    fn a_candidate_writes_neither_efferent_edges_nor_the_pairing_table() {
        // Both properties are what keep the forward pass shared with the arm
        // this one is compared against. An efferent write would credit cells
        // downstream of a spike that never arrived; a pairing-table write would
        // re-time the STDP of every real spike that pairs against this cell
        // afterwards.
        let mut eng = competition_engine();
        let mut learner = ThreeFactor::new(0.5, 0.0, 40.0);

        run_selection(&mut eng);
        learner.queue_candidate_spikes([CandidateSpike {
            cell: LOSER,
            t: 20,
            scale: 1.0,
        }]);
        learner.update_with_credit_counted(&mut eng, &credit_for(LOSER));

        assert_eq!(
            eng.syn.as_slice()[E_3_4].eligibility.to_bits(),
            0.0_f32.to_bits(),
            "candidate wrote its efferent edge loser->downstream"
        );
        assert_eq!(
            learner.last_spike_at(LOSER as usize),
            None,
            "candidate entered the pairing table as if it had fired"
        );
    }

    #[test]
    fn deposited_eligibility_is_linear_in_scale() {
        // `λ_c` is the swept knob, so its effect on the trace must be the plain
        // multiplier the protocol says it is, not a saturating or thresholded one.
        let mut half = competition_engine();
        let mut learner_half = ThreeFactor::new(0.5, 0.0, 40.0);
        run_selection(&mut half);
        learner_half.queue_candidate_spikes([CandidateSpike {
            cell: LOSER,
            t: 20,
            scale: 0.25,
        }]);
        learner_half.observe_spikes(&mut half);

        let mut full = competition_engine();
        let mut learner_full = ThreeFactor::new(0.5, 0.0, 40.0);
        run_selection(&mut full);
        learner_full.queue_candidate_spikes([CandidateSpike {
            cell: LOSER,
            t: 20,
            scale: 0.5,
        }]);
        learner_full.observe_spikes(&mut full);

        for edge in [E_0_3, E_1_3] {
            let e_half = half.syn.as_slice()[edge].eligibility;
            let e_full = full.syn.as_slice()[edge].eligibility;
            assert!(e_half > 0.0, "edge {edge} carried no counterfactual trace");
            assert!(
                (e_full - 2.0 * e_half).abs() < 1e-6,
                "edge {edge}: doubling scale gave {e_full}, not 2 x {e_half}"
            );
        }
    }

    #[test]
    fn no_candidates_is_bit_identical_to_the_mechanism_being_absent() {
        // The `λ_c = 0` rung of the ladder is the arm it is compared against, so
        // "identical" here has to mean every bit of every weight and trace, not
        // agreement to a tolerance.
        let mut plain = competition_engine();
        let mut learner_plain = ThreeFactor::new(0.5, 0.001, 40.0);
        run_selection(&mut plain);
        learner_plain.update_with_credit_counted(&mut plain, &credit_for(WINNER));

        let mut queued = competition_engine();
        let mut learner_queued = ThreeFactor::new(0.5, 0.001, 40.0);
        run_selection(&mut queued);
        // Offered and rejected: a zero-scale candidate adds nothing but would
        // still advance its afferents' lazy-decay clock, splitting one decay
        // into two and perturbing the low bits.
        let accepted = learner_queued.queue_candidate_spikes([
            CandidateSpike {
                cell: LOSER,
                t: 20,
                scale: 0.0,
            },
            CandidateSpike {
                cell: LOSER,
                t: 20,
                scale: f32::NAN,
            },
            CandidateSpike {
                cell: LOSER,
                t: 20,
                scale: -1.0,
            },
        ]);
        assert_eq!(accepted, 0, "a degenerate scale was queued");
        learner_queued.update_with_credit_counted(&mut queued, &credit_for(WINNER));

        assert_eq!(weight_bits(&plain), weight_bits(&queued));
        assert_eq!(elig_bits(&plain), elig_bits(&queued));
    }

    #[test]
    fn a_candidate_does_not_disturb_a_later_real_spike() {
        // `decay_to` rewinds a synapse's decay clock to whatever time it is
        // given, so a deposit appended *after* a later real spike would
        // re-decay that synapse from an earlier instant. The deposit is
        // therefore interleaved chronologically; this checks that it was, by
        // pinning every edge the candidate does not own.
        let mut plain = competition_engine();
        let mut learner_plain = ThreeFactor::new(0.5, 0.0, 40.0);
        run_selection(&mut plain);
        plain.force_spike(1, 40);
        let _ = plain.step_until(50);
        learner_plain.observe_spikes(&mut plain);

        let mut with_cand = competition_engine();
        let mut learner_cand = ThreeFactor::new(0.5, 0.0, 40.0);
        run_selection(&mut with_cand);
        with_cand.force_spike(1, 40);
        let _ = with_cand.step_until(50);
        learner_cand.queue_candidate_spikes([CandidateSpike {
            cell: LOSER,
            t: 20,
            scale: 1.0,
        }]);
        learner_cand.observe_spikes(&mut with_cand);

        // Untouched edges: everything not afferent to the loser.
        for edge in [E_0_2, E_1_2, E_2_4, E_3_4] {
            assert_eq!(
                plain.syn.as_slice()[edge].eligibility.to_bits(),
                with_cand.syn.as_slice()[edge].eligibility.to_bits(),
                "edge {edge} changed although no candidate addressed it"
            );
        }
    }

    #[test]
    fn one_absorb_drains_the_queue() {
        // A near-miss belongs to the selection event that produced it. Surviving
        // into the next absorb would credit a cell for losing a competition that
        // has already been resolved and its eligibility cleared.
        let mut eng = competition_engine();
        let mut learner = ThreeFactor::new(0.5, 0.0, 40.0);
        run_selection(&mut eng);
        learner.queue_candidate_spikes([CandidateSpike {
            cell: LOSER,
            t: 20,
            scale: 1.0,
        }]);
        assert_eq!(learner.pending_candidate_count(), 1);
        learner.observe_spikes(&mut eng);
        assert_eq!(learner.pending_candidate_count(), 0);

        let after_first: Vec<u32> = elig_bits(&eng);
        learner.observe_spikes(&mut eng);
        assert_eq!(
            after_first,
            elig_bits(&eng),
            "a second absorb re-deposited a drained candidate"
        );
    }

    #[test]
    fn a_trial_boundary_reset_drops_unabsorbed_candidates() {
        let mut eng = competition_engine();
        let mut learner = ThreeFactor::new(0.5, 0.0, 40.0);
        run_selection(&mut eng);
        learner.queue_candidate_spikes([CandidateSpike {
            cell: LOSER,
            t: 20,
            scale: 1.0,
        }]);
        learner.reset_pairing_state();
        assert_eq!(learner.pending_candidate_count(), 0);
    }
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

    /// Pass-3 H1: sticky `last_spike` lets a prior-trial post pair with a new
    /// trial's pre → spurious LTD (`stdp(t_old − t_new) < 0`).
    #[test]
    fn last_spike_cross_trial_contamination_without_reset() {
        let mut eng = coincidence_engine(0.0);
        let mut learner = ThreeFactor::new(0.0, 0.0, 20.0);

        // Trial N: post alone at t=20.
        eng.force_spike(2, 20);
        eng.step_until(20);
        learner.update(&mut eng, Modulators::zero());
        assert_eq!(learner.last_spike_at(2), Some(20));
        for syn in eng.syn.as_mut_slice() {
            syn.eligibility = 0.0;
        }

        // Trial N+1: pre at t=80 without clearing pairing state.
        eng.force_spike(0, 80);
        eng.step_until(80);
        learner.update(&mut eng, Modulators::zero());
        let e = eng.syn.as_slice()[EDGE_A].eligibility;
        let expected_ltd = eligibility::stdp(20.0 - 80.0);
        assert!(
            e < -0.01 && (e - expected_ltd).abs() < 1e-5,
            "sticky last_spike must induce cross-trial LTD: e={e} expected={expected_ltd}"
        );
    }

    /// Isolation API: clearing `last_spike` between trials removes the pairing.
    #[test]
    fn reset_pairing_state_isolates_cross_trial_stdp() {
        let mut eng = coincidence_engine(0.0);
        let mut learner = ThreeFactor::new(0.0, 0.0, 20.0);

        eng.force_spike(2, 20);
        eng.step_until(20);
        learner.update(&mut eng, Modulators::zero());
        for syn in eng.syn.as_mut_slice() {
            syn.eligibility = 0.0;
        }

        learner.reset_pairing_state();
        assert_eq!(learner.last_spike_at(2), None);

        eng.force_spike(0, 80);
        eng.step_until(80);
        learner.update(&mut eng, Modulators::zero());
        let e = eng.syn.as_slice()[EDGE_A].eligibility;
        assert!(
            e.abs() < 1e-6,
            "after reset_pairing_state, lone pre must not pair with prior post: e={e}"
        );
        assert_eq!(learner.last_spike_at(0), Some(80));
    }

    /// Product neuromodulator (v12 family): per-neuron `B_i · r(a−p)` differentiates
    /// postsynaptic gates under the production ThreeFactor credit path.
    #[test]
    fn reinforce_feedback_modulator_differentiates_posts() {
        use crate::credit::{reinforce_term, ReinforceFeedback};
        use binn_core::Csr;

        // Two posts (cells 2, 3), each with one pre edge from cell 0 / 1.
        let mut eng = Engine::with_cells(4);
        let row_ptr = vec![0u32, 1, 2, 2, 2];
        let col = vec![2u32, 3];
        let conn = Csr::from_parts(row_ptr, col).expect("two-post CSR");
        eng.set_connectivity(conn, vec![0.2; 2]);

        let fb = ReinforceFeedback::from_weights(vec![0.0, 0.0, 0.8, -0.5]);
        let directional = reinforce_term(1.0, 1.0, 0.25); // 0.75
        let signal = fb.credit(directional);
        assert!((signal.for_post(2) - 0.6).abs() < 1e-6);
        assert!((signal.for_post(3) + 0.375).abs() < 1e-6);

        let mut learner = ThreeFactor::new(0.5, 0.0, 40.0);
        // Drive eligibility on both edges: pre0→post2 and pre1→post3.
        eng.force_spike(0, 10);
        eng.force_spike(1, 10);
        eng.step_until(10);
        eng.force_spike(2, 20);
        eng.force_spike(3, 20);
        eng.step_until(20);

        let w0 = eng.edge_w.clone();
        let _ = learner.update_with_credit_counted(&mut eng, &signal);
        let dw0 = eng.edge_w[0] - w0[0];
        let dw1 = eng.edge_w[1] - w0[1];
        // Opposite-sign B ⇒ opposite-sign weight updates when eligibility matches.
        assert!(
            dw0 > 0.0 && dw1 < 0.0,
            "expected opposite plasticity from signed B; dw0={dw0} dw1={dw1}"
        );
        // Magnitude tracks |B|: |B2|=0.8 > |B3|=0.5 ⇒ |dw0| > |dw1| for equal elig.
        assert!(
            dw0.abs() > dw1.abs(),
            "larger |B| should move more: |dw0|={} |dw1|={}",
            dw0.abs(),
            dw1.abs()
        );
    }
}
