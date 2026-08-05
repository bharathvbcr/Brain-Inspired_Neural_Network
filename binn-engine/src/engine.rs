//! Event-driven engine loop (U05).
//!
//! Pop event → lazy-integrate multi-compartment cell → deposit into a dendritic
//! compartment → maybe somatic spike → log spike.
//! Work scales with events, not with the idle cell population.

use binn_core::sparse::{Csc, Csr};
use binn_core::time::Tick;
use rayon::prelude::*;

use crate::cell::{Cell, CellId, INJECT_WEIGHT, K};
use crate::parallel::{ParallelismProfile, PartitionPlan, PARALLEL_CELL_THRESHOLD};
use crate::queue::{Event, TimingWheel};
use crate::spikelog::SpikeLog;
use crate::synapse::Synapses;

/// Disjoint event-work counters for U12 / U13 efficiency disclosure.
///
/// Mirrors `binn_data::WorkCounters` without creating a crate cycle; the lab
/// copies these into `WorkCounters` when emitting metrics.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EngineWorkCounters {
    pub source_spikes: u64,
    pub synaptic_deliveries: u64,
    pub cell_updates: u64,
}

/// Event-driven substrate engine.
pub struct Engine {
    cells: Vec<Cell>,
    /// Synapse storage (plasticity rules live in L5).
    pub syn: Synapses,
    /// Directed sparse connectivity: row = presynaptic cell, col = postsynaptic cell.
    ///
    /// Topology is owned here so L4 (`project` / `associate`) can read and update
    /// edges without a second world object. Generation lives in `binn-areas::wire`.
    pub conn: Csr,
    /// Reverse adjacency (CSC) over `conn` for O(fan-in) postsynaptic lookup.
    /// Edge indices point into CSR nnz / synapse / `edge_w` order.
    pub conn_rev: Csc,
    /// Per-edge weights aligned with [`Csr::col`] / `conn.nnz()` order.
    pub edge_w: Vec<f32>,
    queue: TimingWheel,
    /// Current simulation time (last processed event tick, or `step_until` target).
    t: Tick,
    spikes: SpikeLog,
    /// Charge delivered to each cell during the most recent bounded step.
    last_step_charge: Vec<f32>,
    /// Cumulative work counters since construction / [`Self::reset_work`].
    work: EngineWorkCounters,
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

impl Engine {
    /// Empty engine at tick 0.
    pub fn new() -> Self {
        Self {
            cells: Vec::new(),
            syn: Synapses::new(),
            conn: Csr::empty(0),
            conn_rev: Csc::empty(0),
            edge_w: Vec::new(),
            queue: TimingWheel::new(),
            t: 0,
            spikes: SpikeLog::new(),
            last_step_charge: Vec::new(),
            work: EngineWorkCounters::default(),
        }
    }

    /// Engine with `n` default-parameter cells.
    pub fn with_cells(n: usize) -> Self {
        let mut eng = Self::new();
        eng.cells.resize_with(n, Cell::default_params);
        eng.last_step_charge.resize(n, 0.0);
        eng.conn = Csr::empty(n);
        eng.conn_rev = Csc::empty(n);
        eng.edge_w.clear();
        eng
    }

    /// Install directed connectivity and per-edge weights.
    ///
    /// # Panics
    ///
    /// Panics if `weights.len() != conn.nnz()`, if `conn.nrows()` is neither `0`
    /// nor `num_cells()`, or if any column index is out of range.
    pub fn set_connectivity(&mut self, conn: Csr, weights: Vec<f32>) {
        assert_eq!(
            weights.len(),
            conn.nnz(),
            "edge weights must align with CSR nnz ({} vs {})",
            weights.len(),
            conn.nnz()
        );
        let n = self.cells.len();
        assert!(
            conn.nrows() == n || (conn.nrows() == 0 && n == 0),
            "connectivity nrows ({}) must equal num_cells ({n})",
            conn.nrows()
        );
        for &c in &conn.col {
            assert!(
                (c as usize) < n,
                "connectivity column {c} out of range (n={n})"
            );
        }
        self.conn_rev = Csc::from_csr(&conn);
        self.conn = conn;
        self.edge_w = weights;
        // Keep synapse table (eligibility storage) aligned with CSR nnz order.
        self.syn.rebuild_from_weights(&self.edge_w, 1);
    }

    /// Install connectivity with unit weights on every edge.
    pub fn set_connectivity_unit(&mut self, conn: Csr) {
        let w = vec![1.0f32; conn.nnz()];
        self.set_connectivity(conn, w);
    }

    /// Potentiate one existing CSR edge while keeping both weight views aligned.
    pub fn potentiate_edge(&mut self, edge: usize, delta: f32) {
        assert!(edge < self.edge_w.len(), "edge index out of range");
        assert!(delta.is_finite(), "weight delta must be finite");
        self.edge_w[edge] += delta;
        self.syn.get_mut(edge).expect("aligned synapse").weight = self.edge_w[edge];
    }

    /// Largest installed synaptic delay, or zero when disconnected.
    pub fn max_synaptic_delay(&self) -> Tick {
        self.syn
            .as_slice()
            .iter()
            .map(|syn| syn.delay)
            .max()
            .unwrap_or(0)
    }

    /// Close a bounded externally inhibited cycle and discard later activity.
    ///
    /// Assembly projection uses this after its measurement window. General
    /// simulations should normally retain pending events and not call it.
    pub fn close_inhibited_cycle(&mut self) {
        self.queue = TimingWheel::new();
    }

    /// Current simulation time.
    #[inline]
    pub fn time(&self) -> Tick {
        self.t
    }

    /// Number of cells.
    #[inline]
    pub fn num_cells(&self) -> usize {
        self.cells.len()
    }

    /// Borrow cell table.
    #[inline]
    pub fn cells(&self) -> &[Cell] {
        &self.cells
    }

    /// Borrow cell `id`.
    #[inline]
    pub fn cell(&self, id: CellId) -> &Cell {
        &self.cells[id as usize]
    }

    /// Mutably borrow cell `id` (does not advance time).
    #[inline]
    pub fn cell_mut(&mut self, id: CellId) -> &mut Cell {
        &mut self.cells[id as usize]
    }

    /// Append a cell; returns its id.
    pub fn push_cell(&mut self, cell: Cell) -> CellId {
        let id = self.cells.len() as CellId;
        self.cells.push(cell);
        self.last_step_charge.push(0.0);
        id
    }

    /// Schedule an external inject onto `cell` / `branch` at tick `at`.
    ///
    /// # Panics
    ///
    /// Panics if `at < self.time()`, `cell` is out of range, or `branch >= K`.
    pub fn inject(&mut self, cell: CellId, branch: u8, at: Tick) {
        self.inject_weighted(cell, branch, at, INJECT_WEIGHT);
    }

    /// Schedule a finite weighted voltage impulse.
    pub fn inject_weighted(&mut self, cell: CellId, branch: u8, at: Tick, amount: f32) {
        assert!(
            at >= self.t,
            "inject requires at >= engine time (at={at}, t={})",
            self.t
        );
        assert!(
            (cell as usize) < self.cells.len(),
            "cell id {cell} out of range (n={})",
            self.cells.len()
        );
        assert!(
            (branch as usize) < K,
            "branch index {branch} out of range (K={K})"
        );
        assert!(amount.is_finite(), "inject amount must be finite");
        self.queue
            .insert(at, encode_event(cell, branch, amount, false));
    }

    /// Explicitly activate a cell at `at` (Assembly-Calculus `fire` primitive).
    /// The emitted spike propagates through ordinary weighted synapses.
    pub fn force_spike(&mut self, cell: CellId, at: Tick) {
        assert!(at >= self.t, "force_spike requires at >= engine time");
        assert!((cell as usize) < self.cells.len(), "cell id out of range");
        self.queue.insert(at, encode_event(cell, 0, 0.0, true));
    }

    /// Advance simulation until tick `until` (inclusive upper bound on events).
    ///
    /// Processes every queued event with `at <= until` in tick order (insertion
    /// order for ties). Returns the spikes produced during this call; the full
    /// train is available via [`spikes`](Self::spikes).
    pub fn step_until(&mut self, until: Tick) -> SpikeLog {
        assert!(
            until >= self.t,
            "step_until requires until >= engine time (until={until}, t={})",
            self.t
        );

        let mut produced = SpikeLog::new();
        self.last_step_charge.fill(0.0);

        while self
            .queue
            .peek_earliest_tick()
            .is_some_and(|at| at <= until)
        {
            let (at, ev) = self
                .queue
                .pop_earliest()
                .expect("peek reported a queued event");
            self.t = at;
            let (cell, branch, amount, forced) = decode_event(ev);
            self.work.cell_updates = self.work.cell_updates.saturating_add(1);
            if !forced {
                self.work.synaptic_deliveries = self.work.synaptic_deliveries.saturating_add(1);
            }
            self.deliver(cell, branch, amount, forced, at, &mut produced);
        }

        self.t = until;
        produced
    }

    /// Deterministic graph-partitioned delta stepping (U18 / F1).
    ///
    /// All events at the same tick are drained as one delta bucket, grouped by
    /// target cell. Wide buckets (`>= PARALLEL_CELL_THRESHOLD` distinct cells)
    /// integrate in parallel; thin buckets stay sequential in-place to avoid
    /// rayon overhead. Spike logging and fan-out remain FIFO-ordered so the
    /// observable result matches [`Self::step_until`].
    pub fn step_until_partitioned(&mut self, until: Tick, plan: &PartitionPlan) -> SpikeLog {
        self.step_until_partitioned_profiled(until, plan).0
    }

    /// Same as [`Self::step_until_partitioned`], plus an F1 [`ParallelismProfile`].
    pub fn step_until_partitioned_profiled(
        &mut self,
        until: Tick,
        plan: &PartitionPlan,
    ) -> (SpikeLog, ParallelismProfile) {
        self.step_until_partitioned_threshold(until, plan, PARALLEL_CELL_THRESHOLD)
    }

    /// Partitioned step with an explicit parallel-width threshold (F1 benches).
    ///
    /// Pass `threshold = 0` or `1` to force the rayon path on every non-empty
    /// bucket (legacy always-parallel behavior). Observables still match
    /// [`Self::step_until`] when the plan covers all cells.
    pub fn step_until_partitioned_threshold(
        &mut self,
        until: Tick,
        plan: &PartitionPlan,
        parallel_threshold: usize,
    ) -> (SpikeLog, ParallelismProfile) {
        assert!(
            until >= self.t,
            "step_until_partitioned requires until >= engine time"
        );
        assert_eq!(
            plan.n_cells(),
            self.cells.len(),
            "partition plan must cover every cell"
        );

        #[derive(Clone, Copy)]
        struct Decoded {
            ordinal: usize,
            branch: u8,
            amount: f32,
            forced: bool,
        }
        struct CellJob {
            cell_id: CellId,
            partition: usize,
            cell: Cell,
            events: Vec<Decoded>,
            charge: f32,
            fired_ordinals: Vec<usize>,
        }

        let mut produced = SpikeLog::new();
        let mut profile = ParallelismProfile::default();
        self.last_step_charge.fill(0.0);
        while self
            .queue
            .peek_earliest_tick()
            .is_some_and(|tick| tick <= until)
        {
            let (tick, events) = self
                .queue
                .pop_earliest_batch()
                .expect("earliest delta bucket existed");
            self.t = tick;
            let n_events = events.len();
            let mut grouped: std::collections::BTreeMap<CellId, Vec<Decoded>> =
                std::collections::BTreeMap::new();
            for (ordinal, event) in events.into_iter().enumerate() {
                let (cell, branch, amount, forced) = decode_event(event);
                grouped.entry(cell).or_default().push(Decoded {
                    ordinal,
                    branch,
                    amount,
                    forced,
                });
                self.work.cell_updates = self.work.cell_updates.saturating_add(1);
                if !forced {
                    self.work.synaptic_deliveries = self.work.synaptic_deliveries.saturating_add(1);
                }
            }
            let distinct = grouped.len();
            // Profile uses the production threshold so headroom stays comparable.
            profile.record_bucket(distinct, n_events);

            let mut fired = Vec::new();
            if distinct < parallel_threshold {
                // Thin tick: integrate in place (no clone / rayon).
                // Split the borrow once so `cells` and `last_step_charge` can be
                // held simultaneously. Previously the `&mut self.cells[..]`
                // index sat *inside* the per-event loop — re-running a
                // bounds-checked index `E` times per tick instead of `D` — only
                // because `self.last_step_charge` is also touched while `cell`
                // is borrowed. Same arithmetic, same order.
                let cells = &mut self.cells;
                let charge = &mut self.last_step_charge;
                for (cell_id, cell_events) in grouped {
                    let cell = &mut cells[cell_id as usize];
                    // Hoisted out of the event loop: `advance_to` early-returns
                    // when `dt == 0`, so the repeated calls were cheap, but they
                    // are also plainly redundant — every event here shares one
                    // `tick`.
                    cell.advance_to(tick);
                    for event in &cell_events {
                        let did_fire = if event.forced {
                            cell.v = cell.theta;
                            cell.try_fire()
                        } else {
                            charge[cell_id as usize] += event.amount;
                            cell.deposit(event.branch, event.amount)
                        };
                        if did_fire {
                            fired.push((event.ordinal, cell_id));
                        }
                    }
                }
            } else {
                let mut jobs: Vec<CellJob> = grouped
                    .into_iter()
                    .map(|(cell_id, events)| CellJob {
                        cell_id,
                        partition: plan.owner(cell_id),
                        cell: self.cells[cell_id as usize].clone(),
                        events,
                        charge: 0.0,
                        fired_ordinals: Vec::new(),
                    })
                    .collect();
                // `sort_by_key` is a *stable* sort: driftsort allocates a
                // temporary of up to `jobs.len() / 2` elements, and `CellJob` is
                // ~128 bytes. There is exactly one job per cell, so the key
                // `(partition, cell_id)` is unique and stability buys nothing —
                // no two elements ever compare equal, so the resulting order is
                // identical either way.
                //
                // The sort itself only exists for rayon partition locality: the
                // parallel closure below touches only per-job state, write-back
                // touches disjoint cells, and `fired` is re-sorted by unique
                // `ordinal` afterwards. A counting sort over `n_partitions`
                // (typically 4) would make this O(D) — see
                // results/PERF_AUDIT_2026-08-02.md §6.
                jobs.sort_unstable_by_key(|job| (job.partition, job.cell_id));
                jobs.par_iter_mut().for_each(|job| {
                    // Hoisted out of the event loop, as in the thin path above.
                    //
                    // Equivalence argument: `advance_to` writes `self.last = t`
                    // only on a non-zero `dt` (cell.rs:195) and early-returns
                    // when `dt == 0` (cell.rs:139). The only other write to
                    // `last` is `Cell::reset` (cell.rs:234), which neither
                    // `deposit` nor `try_fire` calls. So after the first call at
                    // `tick`, every later call at the same `tick` was already an
                    // exact no-op. `job.events` is never empty — `grouped` only
                    // creates an entry by pushing to it.
                    job.cell.advance_to(tick);
                    for event in &job.events {
                        let did_fire = if event.forced {
                            job.cell.v = job.cell.theta;
                            job.cell.try_fire()
                        } else {
                            job.charge += event.amount;
                            job.cell.deposit(event.branch, event.amount)
                        };
                        if did_fire {
                            job.fired_ordinals.push(event.ordinal);
                        }
                    }
                });
                for job in jobs {
                    self.cells[job.cell_id as usize] = job.cell;
                    self.last_step_charge[job.cell_id as usize] += job.charge;
                    fired.extend(
                        job.fired_ordinals
                            .into_iter()
                            .map(|ordinal| (ordinal, job.cell_id)),
                    );
                }
            }
            fired.sort_unstable_by_key(|&(ordinal, _)| ordinal);
            for (_, cell) in fired {
                self.work.source_spikes = self.work.source_spikes.saturating_add(1);
                self.spikes.push(tick, cell);
                produced.push(tick, cell);
                self.schedule_fan_out(cell, tick);
            }
        }
        self.t = until;
        (produced, profile)
    }

    /// Full recorded spike train.
    #[inline]
    pub fn spikes(&self) -> &SpikeLog {
        &self.spikes
    }

    /// Cumulative work counters (source spikes, deliveries, cell updates).
    #[inline]
    pub fn work(&self) -> EngineWorkCounters {
        self.work
    }

    /// Zero the work counters (does not clear spikes or connectivity).
    #[inline]
    pub fn reset_work(&mut self) {
        self.work = EngineWorkCounters::default();
    }

    /// Completely reset all cell states, clear event queue and spike log, and reset tick to 0.
    pub fn reset_state(&mut self) {
        for c in &mut self.cells {
            c.reset();
        }
        self.queue = TimingWheel::new();
        self.spikes.clear();
        self.last_step_charge.fill(0.0);
        self.t = 0;
    }

    /// Charge delivered to `cell` during the most recent bounded step.
    #[inline]
    pub fn last_step_charge(&self, cell: CellId) -> f32 {
        self.last_step_charge[cell as usize]
    }

    fn deliver(
        &mut self,
        cell: CellId,
        branch: u8,
        amount: f32,
        forced: bool,
        at: Tick,
        produced: &mut SpikeLog,
    ) {
        let fired = {
            let c = &mut self.cells[cell as usize];
            c.advance_to(at);
            if forced {
                c.v = c.theta;
                c.try_fire()
            } else {
                self.last_step_charge[cell as usize] += amount;
                c.deposit(branch, amount)
            }
        };

        if fired {
            self.work.source_spikes = self.work.source_spikes.saturating_add(1);
            self.spikes.push(at, cell);
            produced.push(at, cell);
            self.schedule_fan_out(cell, at);
        }
    }

    fn schedule_fan_out(&mut self, pre: CellId, at: Tick) {
        if self.conn.nrows() == 0 {
            return;
        }
        let row = pre as usize;
        let start = self.conn.row_ptr[row] as usize;
        let end = self.conn.row_ptr[row + 1] as usize;
        assert_eq!(
            self.syn.len(),
            self.conn.nnz(),
            "synapses must align with CSR"
        );

        // Take both spans as slices once, so the per-edge bounds check and the
        // `Option` branch from `self.syn.get(edge).expect(..)` collapse into a
        // single check here. The `assert_eq!` above already guarantees
        // `end <= syn.len()`.
        //
        // Note this still strides the 32-byte AoS `Synapse` to read 12 bytes of
        // payload (`weight`, `delay`) — 2 synapses per cache line instead of 5.
        // `edge_w` already holds the weights in SoA form; splitting `delay` out
        // too would let this loop stream two dense arrays. Left for a measured
        // pass, see results/PERF_AUDIT_2026-08-02.md §5.
        let cols = &self.conn.col[start..end];
        let syns = &self.syn.as_slice()[start..end];
        for (offset, (&post, syn)) in cols.iter().zip(syns.iter()).enumerate() {
            if syn.weight == 0.0 {
                continue;
            }
            let delivery_at = at.checked_add(syn.delay).expect("synaptic delay overflow");
            let branch = ((start + offset) % K) as u8;
            self.queue
                .insert(delivery_at, encode_event(post, branch, syn.weight, false));
        }
    }
}

/// Pack `(cell, branch)` into a wheel [`Event`].
#[inline]
fn encode_event(cell: CellId, branch: u8, amount: f32, forced: bool) -> Event {
    const FORCE_BIT: u64 = 1 << 63;
    let id = ((cell as u64) << 8) | u64::from(branch) | if forced { FORCE_BIT } else { 0 };
    Event::with_amount(id, amount)
}

/// Unpack a wheel [`Event`] into `(cell, branch)`.
#[inline]
fn decode_event(ev: Event) -> (CellId, u8, f32, bool) {
    const FORCE_BIT: u64 = 1 << 63;
    let forced = ev.id & FORCE_BIT != 0;
    let cell = ((ev.id & !FORCE_BIT) >> 8) as CellId;
    let branch = (ev.id & 0xff) as u8;
    (cell, branch, ev.amount(), forced)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell::{analytic_lif, THETA_REST};
    use binn_core::Rng;

    #[test]
    fn analytic_membrane_via_engine_touch() {
        // Uncoupled dendrite: tonic branch current → analytic LIF on v_dend.
        let tau = 10.0f32;
        let i = 0.5f32;
        let mut eng = Engine::new();
        let mut cell = Cell::new(tau);
        cell.g_c = 0.0;
        cell.set_branch_current(0, i);
        let id = eng.push_cell(cell);

        eng.cell_mut(id).advance_to(100);
        let expected = analytic_lif(0.0, i, tau, 100.0);
        assert!((eng.cell(id).v_dend[0] - expected).abs() <= 1e-5);
        assert!(eng.cell(id).v.abs() <= 1e-5);
    }

    #[test]
    fn inject_and_step_produces_spike() {
        let mut eng = Engine::with_cells(1);
        // v=0, theta=1, inject weight=1 ⇒ spike
        eng.inject(0, 0, 5);
        let produced = eng.step_until(10);
        assert_eq!(produced.len(), 1);
        assert_eq!(produced.as_slice()[0].t, 5);
        assert_eq!(produced.as_slice()[0].cell, 0);
        assert_eq!(eng.spikes().len(), 1);
        assert!(eng.cell(0).theta > THETA_REST);
        assert_eq!(eng.time(), 10);
    }

    #[test]
    fn step_until_leaves_future_events() {
        let mut eng = Engine::with_cells(1);
        eng.inject(0, 0, 5);
        eng.inject_weighted(0, 0, 50, 2.0);
        let p1 = eng.step_until(10);
        assert_eq!(p1.len(), 1);
        assert_eq!(eng.spikes().len(), 1);
        let p2 = eng.step_until(60);
        assert_eq!(p2.len(), 1);
        assert_eq!(eng.spikes().len(), 2);
        assert_eq!(eng.spikes().as_slice()[1].t, 50);
    }

    #[test]
    fn partial_step_allows_intermediate_injection_with_multiple_future_events() {
        let mut eng = Engine::with_cells(1);
        eng.inject(0, 0, 100);
        eng.inject(0, 0, 200);
        assert!(eng.step_until(50).is_empty());
        eng.inject(0, 0, 75);
        let produced = eng.step_until(80);
        assert_eq!(produced.as_slice()[0].t, 75);
    }

    #[test]
    fn spike_propagates_with_weight_and_delay() {
        let mut eng = Engine::with_cells(2);
        let conn = Csr::from_adjacency(&[vec![1], vec![]]);
        eng.set_connectivity(conn, vec![1.0]);
        eng.force_spike(0, 5);
        let produced = eng.step_until(6);
        let observed: Vec<(Tick, CellId)> = produced.iter().map(|s| (s.t, s.cell)).collect();
        assert_eq!(observed, vec![(5, 0), (6, 1)]);
        assert_eq!(eng.last_step_charge(1), 1.0);
    }

    /// U05 / G0: weighted spikes propagate across a 2→3-cell chain at exact delays.
    ///
    /// Topology `0 → 1 → 2` with distinct synaptic delays (2 and 3 ticks). Force
    /// spike cell 0 at t=5 ⇒ cell 1 at t=7, cell 2 at t=10.
    #[test]
    fn spike_propagates_across_three_cell_delayed_network() {
        let mut eng = Engine::with_cells(3);
        let conn = Csr::from_adjacency(&[vec![1], vec![2], vec![]]);
        eng.set_connectivity(conn, vec![1.0, 1.0]);
        eng.syn.get_mut(0).expect("0→1 synapse").delay = 2;
        eng.syn.get_mut(1).expect("1→2 synapse").delay = 3;

        eng.force_spike(0, 5);
        let produced = eng.step_until(20);
        let observed: Vec<(Tick, CellId)> = produced.iter().map(|s| (s.t, s.cell)).collect();
        assert_eq!(
            observed,
            vec![(5, 0), (7, 1), (10, 2)],
            "2→3-cell chain must fire at exact cumulative delays"
        );
        assert_eq!(eng.last_step_charge(2), 1.0);
    }

    #[test]
    fn equal_tick_events_follow_insertion_order() {
        let mut eng = Engine::with_cells(3);
        eng.inject(2, 0, 10);
        eng.inject(0, 0, 10);
        eng.inject(1, 0, 10);
        let produced = eng.step_until(10);
        let cells: Vec<CellId> = produced.iter().map(|s| s.cell).collect();
        assert_eq!(cells, vec![2, 0, 1]);
    }

    /// GC3 / G0: same seed ⇒ identical spike train.
    fn spike_train_for_seed(seed: u64) -> SpikeLog {
        let mut rng = Rng::new(seed);
        let n_cells = 8usize;
        let mut eng = Engine::with_cells(n_cells);
        let mut t: Tick = 0;
        for _ in 0..64 {
            t = t.saturating_add(1 + rng.gen_index(4) as Tick);
            let cell = rng.gen_index(n_cells) as CellId;
            let branch = rng.gen_index(K) as u8;
            eng.inject(cell, branch, t);
        }
        eng.step_until(t.saturating_add(1));
        eng.spikes().clone()
    }

    #[test]
    fn seed_identical_spike_train() {
        let seed = 0xB177_C0DE_0000_0005;
        let a = spike_train_for_seed(seed);
        let b = spike_train_for_seed(seed);
        assert_eq!(a, b, "same seed must yield identical spike trains");
        assert!(
            !a.is_empty(),
            "expected a non-empty spike train for determinism coverage"
        );

        let other = spike_train_for_seed(seed ^ 0x9E37_79B9_7F4A_7C15);
        // Different seed almost surely differs; if equal, still determinism holds.
        let _ = other;
    }

    #[test]
    fn different_seeds_diverge() {
        let a = spike_train_for_seed(1);
        let b = spike_train_for_seed(2);
        assert_ne!(a, b);
    }

    /// F1: wide same-tick buckets take the rayon path and still match sequential.
    #[test]
    fn wide_tick_partitioned_matches_sequential() {
        use crate::parallel::{PartitionPlan, PARALLEL_CELL_THRESHOLD};

        let n = PARALLEL_CELL_THRESHOLD * 2;
        let rows: Vec<Vec<u32>> = (0..n).map(|_| Vec::new()).collect();
        let conn = Csr::from_adjacency(&rows);
        let mut sequential = Engine::with_cells(n);
        sequential.set_connectivity_unit(conn.clone());
        let mut parallel = Engine::with_cells(n);
        parallel.set_connectivity_unit(conn.clone());
        for cell in 0..n as u32 {
            sequential.force_spike(cell, 5);
            parallel.force_spike(cell, 5);
        }
        let expected = sequential.step_until(5);
        let plan = PartitionPlan::degree_balanced(&conn, 4);
        let (observed, profile) = parallel.step_until_partitioned_profiled(5, &plan);
        assert_eq!(observed, expected);
        assert_eq!(parallel.work(), sequential.work());
        assert_eq!(profile.parallel_ticks, 1);
        assert_eq!(profile.max_width, n);
        assert!(profile.width_headroom() >= 1.0 - 1e-12);
    }
}
