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

/// Decoded event entry for reusable partitioned-stepping scratch buffers.
#[derive(Clone, Copy, Default)]
struct DecodedEvent {
    ordinal: usize,
    amount: f32,
    branch: u8,
    forced: bool,
    fired: bool,
}

/// Reusable scratch buffers for [`Engine::step_until_partitioned_threshold`] so
/// neither thin nor wide ticks allocate per-tick maps or per-cell event vectors.
#[derive(Default)]
struct PartitionScratch {
    batch_events: Vec<Event>,
    cell_to_active: Vec<u32>,
    active_cells: Vec<CellId>,
    bucket_counts: Vec<usize>,
    bucket_offsets: Vec<usize>,
    bucket_cursors: Vec<usize>,
    raw_decoded: Vec<(u32, DecodedEvent)>,
    grouped_events: Vec<DecodedEvent>,
    part_counts: Vec<usize>,
    part_offsets: Vec<usize>,
    ordered_active: Vec<u32>,
    active_rank: Vec<u32>,
    fired: Vec<(usize, CellId)>,
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
    /// Cells whose `last_step_charge` became non-zero during the current step,
    /// so resetting `last_step_charge` is O(D) instead of an O(n) memset.
    dirty_charge_cells: Vec<CellId>,
    /// Cells touched since the last [`Self::reset_state`] (`O(D)` lazy reset).
    dirty_cells: Vec<CellId>,
    /// Deduplication flags (`0` = guaranteed at rest, `1` = queued in `dirty_cells`).
    cell_dirty: Vec<u8>,
    /// Armed lazily on the first [`Self::reset_state`] so one-shot engines pay
    /// zero `cell_dirty` allocation or per-delivery dirty-list overhead.
    dirty_cells_armed: bool,
    /// Reusable partitioned-step scratch buffers.
    scratch: PartitionScratch,
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
            dirty_charge_cells: Vec::new(),
            dirty_cells: Vec::new(),
            cell_dirty: Vec::new(),
            dirty_cells_armed: false,
            scratch: PartitionScratch::default(),
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
        self.syn.set_weight_keeping_delay(edge, self.edge_w[edge]);
    }

    /// Largest installed synaptic delay, or zero when disconnected.
    #[inline]
    pub fn max_synaptic_delay(&self) -> Tick {
        self.syn.max_delay()
    }

    /// Close a bounded externally inhibited cycle and discard later activity.
    ///
    /// Assembly projection uses this after its measurement window. General
    /// simulations should normally retain pending events and not call it.
    pub fn close_inhibited_cycle(&mut self) {
        self.queue.clear();
    }

    #[inline]
    fn clear_last_step_charge(&mut self) {
        if self.dirty_charge_cells.len() > self.last_step_charge.len() / 2 {
            self.last_step_charge.fill(0.0);
            self.dirty_charge_cells.clear();
        } else {
            for cell in self.dirty_charge_cells.drain(..) {
                self.last_step_charge[cell as usize] = 0.0;
            }
        }
    }

    #[inline]
    fn add_step_charge(&mut self, cell: CellId, amount: f32) {
        let next = self.last_step_charge[cell as usize] + amount;
        self.set_step_charge(cell, next);
    }

    #[inline]
    fn set_step_charge(&mut self, cell: CellId, next: f32) {
        let slot = &mut self.last_step_charge[cell as usize];
        let prev = *slot;
        *slot = next;
        if prev.to_bits() == 0 && next.to_bits() != 0 {
            self.dirty_charge_cells.push(cell);
        }
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

    #[inline]
    fn mark_cell_dirty(&mut self, cell: CellId) {
        if !self.dirty_cells_armed {
            return;
        }
        let flag = &mut self.cell_dirty[cell as usize];
        if *flag == 0 {
            *flag = 1;
            self.dirty_cells.push(cell);
        }
    }

    /// Mutably borrow cell `id` (does not advance time).
    #[inline]
    pub fn cell_mut(&mut self, id: CellId) -> &mut Cell {
        self.mark_cell_dirty(id);
        &mut self.cells[id as usize]
    }

    /// Append a cell; returns its id.
    pub fn push_cell(&mut self, cell: Cell) -> CellId {
        let id = self.cells.len() as CellId;
        let dirty = !cell.is_at_rest();
        self.cells.push(cell);
        self.last_step_charge.push(0.0);
        if self.dirty_cells_armed {
            self.cell_dirty.push(u8::from(dirty));
            if dirty {
                self.dirty_cells.push(id);
            }
        }
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
        self.clear_last_step_charge();

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

        struct ParallelCellJob<'a> {
            cell_id: CellId,
            cell: Cell,
            events: &'a mut [DecodedEvent],
            charge: f32,
        }

        let mut produced = SpikeLog::new();
        let mut profile = ParallelismProfile::default();
        self.clear_last_step_charge();

        let mut scratch = std::mem::take(&mut self.scratch);
        if scratch.cell_to_active.len() < self.cells.len() {
            scratch.cell_to_active.resize(self.cells.len(), u32::MAX);
        }

        while self
            .queue
            .peek_earliest_tick()
            .is_some_and(|tick| tick <= until)
        {
            let tick = self
                .queue
                .drain_earliest_into(&mut scratch.batch_events)
                .expect("earliest delta bucket existed");
            self.t = tick;
            let n_events = scratch.batch_events.len();

            // Common sparse case: single-event tick below parallel threshold.
            if n_events == 1 && 1 < parallel_threshold {
                profile.record_bucket(1, 1);
                let (cell, branch, amount, forced) = decode_event(scratch.batch_events[0]);
                self.work.cell_updates = self.work.cell_updates.saturating_add(1);
                if !forced {
                    self.work.synaptic_deliveries = self.work.synaptic_deliveries.saturating_add(1);
                }
                self.deliver(cell, branch, amount, forced, tick, &mut produced);
                continue;
            }

            scratch.active_cells.clear();
            scratch.bucket_counts.clear();
            scratch.raw_decoded.clear();
            scratch.fired.clear();

            for (ordinal, &event) in scratch.batch_events.iter().enumerate() {
                let (cell, branch, amount, forced) = decode_event(event);
                let slot = &mut scratch.cell_to_active[cell as usize];
                let bucket = if *slot == u32::MAX {
                    let b = scratch.active_cells.len() as u32;
                    *slot = b;
                    scratch.active_cells.push(cell);
                    scratch.bucket_counts.push(1);
                    b
                } else {
                    let b = *slot;
                    scratch.bucket_counts[b as usize] += 1;
                    b
                };
                scratch.raw_decoded.push((
                    bucket,
                    DecodedEvent {
                        ordinal,
                        amount,
                        branch,
                        forced,
                        fired: false,
                    },
                ));
                self.work.cell_updates = self.work.cell_updates.saturating_add(1);
                if !forced {
                    self.work.synaptic_deliveries = self.work.synaptic_deliveries.saturating_add(1);
                }
            }
            if self.dirty_cells_armed
                && self.dirty_cells.capacity() < self.dirty_cells.len() + scratch.active_cells.len()
            {
                self.dirty_cells.reserve(scratch.active_cells.len());
            }
            if self.dirty_cells_armed {
                for &cell in &scratch.active_cells {
                    scratch.cell_to_active[cell as usize] = u32::MAX;
                    self.mark_cell_dirty(cell);
                }
            } else {
                for &cell in &scratch.active_cells {
                    scratch.cell_to_active[cell as usize] = u32::MAX;
                }
            }

            let distinct = scratch.active_cells.len();
            // Profile uses the production threshold so headroom stays comparable.
            profile.record_bucket(distinct, n_events);

            if distinct < parallel_threshold {
                // Thin multi-event tick: integrate directly in ordinal order.
                // Each cell's state and `last_step_charge` slot are disjoint from
                // every other cell's, `advance_to(tick)` is idempotent on equal
                // ticks (`dt == 0` early-returns), and `fired` lands in strictly
                // increasing `ordinal` order with zero sorting.
                for &(bucket, event) in &scratch.raw_decoded {
                    let cell_id = scratch.active_cells[bucket as usize];
                    let did_fire = {
                        let cell = &mut self.cells[cell_id as usize];
                        cell.advance_to(tick);
                        if event.forced {
                            cell.v = cell.theta;
                            cell.try_fire()
                        } else {
                            let res = cell.deposit(event.branch, event.amount);
                            self.add_step_charge(cell_id, event.amount);
                            res
                        }
                    };
                    if did_fire {
                        scratch.fired.push((event.ordinal, cell_id));
                    }
                }
            } else {
                // O(D + P) counting sort by partition for Rayon locality.
                let n_parts = plan.n_partitions().max(1);
                scratch.part_counts.clear();
                scratch.part_counts.resize(n_parts, 0);
                for &cell_id in &scratch.active_cells {
                    let p = plan.owner(cell_id).min(n_parts - 1);
                    scratch.part_counts[p] += 1;
                }
                scratch.part_offsets.clear();
                scratch.part_offsets.reserve(n_parts + 1);
                let mut sum_p = 0usize;
                for &cnt in &scratch.part_counts {
                    scratch.part_offsets.push(sum_p);
                    sum_p += cnt;
                }
                scratch.part_offsets.push(sum_p);

                scratch.ordered_active.resize(distinct, 0);
                scratch.active_rank.resize(distinct, 0);
                for (orig_b, &cell_id) in scratch.active_cells.iter().enumerate() {
                    let p = plan.owner(cell_id).min(n_parts - 1);
                    let rank = scratch.part_offsets[p];
                    scratch.part_offsets[p] = rank + 1;
                    scratch.ordered_active[rank] = orig_b as u32;
                    scratch.active_rank[orig_b] = rank as u32;
                }

                // Prefix offsets in partition-sorted active order and scatter
                // `raw_decoded` into contiguous per-cell spans of `grouped_events`.
                scratch.bucket_offsets.clear();
                scratch.bucket_offsets.reserve(distinct + 1);
                let mut ev_off = 0usize;
                for &orig_b in &scratch.ordered_active[..distinct] {
                    scratch.bucket_offsets.push(ev_off);
                    ev_off += scratch.bucket_counts[orig_b as usize];
                }
                scratch.bucket_offsets.push(ev_off);
                scratch.bucket_cursors.clear();
                scratch
                    .bucket_cursors
                    .extend_from_slice(&scratch.bucket_offsets[..distinct]);

                scratch
                    .grouped_events
                    .resize(n_events, DecodedEvent::default());
                for &(orig_b, ev) in &scratch.raw_decoded {
                    let rank = scratch.active_rank[orig_b as usize] as usize;
                    let dst = scratch.bucket_cursors[rank];
                    scratch.bucket_cursors[rank] = dst + 1;
                    scratch.grouped_events[dst] = ev;
                }

                let mut jobs = Vec::with_capacity(distinct);
                let mut rem = &mut scratch.grouped_events[..n_events];
                for &orig_b in &scratch.ordered_active[..distinct] {
                    let len = scratch.bucket_counts[orig_b as usize];
                    let (events_slice, rest) = rem.split_at_mut(len);
                    rem = rest;
                    let cell_id = scratch.active_cells[orig_b as usize];
                    jobs.push(ParallelCellJob {
                        cell_id,
                        cell: self.cells[cell_id as usize].clone(),
                        events: events_slice,
                        charge: self.last_step_charge[cell_id as usize],
                    });
                }

                jobs.par_iter_mut().for_each(|job| {
                    job.cell.advance_to(tick);
                    for event in job.events.iter_mut() {
                        let did_fire = if event.forced {
                            job.cell.v = job.cell.theta;
                            job.cell.try_fire()
                        } else {
                            job.charge += event.amount;
                            job.cell.deposit(event.branch, event.amount)
                        };
                        if did_fire {
                            event.fired = true;
                        }
                    }
                });

                for job in jobs {
                    self.cells[job.cell_id as usize] = job.cell;
                    self.set_step_charge(job.cell_id, job.charge);
                    for ev in job.events {
                        if ev.fired {
                            scratch.fired.push((ev.ordinal, job.cell_id));
                        }
                    }
                }
                scratch.fired.sort_unstable_by_key(|&(ordinal, _)| ordinal);
            }

            for &(_, cell) in &scratch.fired {
                self.work.source_spikes = self.work.source_spikes.saturating_add(1);
                self.spikes.push(tick, cell);
                produced.push(tick, cell);
                self.schedule_fan_out(cell, tick);
            }
        }
        self.scratch = scratch;
        self.t = until;
        (produced, profile)
    }

    /// Full recorded spike train.
    #[inline]
    pub fn spikes(&self) -> &SpikeLog {
        &self.spikes
    }

    /// Split-borrow the immutable spike log and connectivity matrices alongside
    /// mutable synapse storage so L5 plasticity (`ThreeFactor::absorb_spikes`)
    /// can walk chronological engine spikes with zero intermediate `Vec` copies.
    #[inline]
    pub fn spikes_and_topology_with_syn_mut(&mut self) -> (&SpikeLog, &Csr, &Csc, &mut Synapses) {
        (&self.spikes, &self.conn, &self.conn_rev, &mut self.syn)
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
        if !self.dirty_cells_armed {
            for c in &mut self.cells {
                c.reset();
            }
            self.cell_dirty.clear();
            self.cell_dirty.resize(self.cells.len(), 0);
            self.dirty_cells.clear();
            self.dirty_cells.reserve(self.cells.len().min(1_024));
            self.dirty_cells_armed = true;
        } else if self.dirty_cells.len() > self.cells.len() / 2 {
            for c in &mut self.cells {
                c.reset();
            }
            for id in self.dirty_cells.drain(..) {
                self.cell_dirty[id as usize] = 0;
            }
        } else {
            for id in self.dirty_cells.drain(..) {
                let idx = id as usize;
                self.cell_dirty[idx] = 0;
                self.cells[idx].reset();
            }
        }
        self.queue.clear();
        self.spikes.clear();
        self.clear_last_step_charge();
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
        self.mark_cell_dirty(cell);
        let fired = {
            let c = &mut self.cells[cell as usize];
            c.advance_to(at);
            if forced {
                c.v = c.theta;
                c.try_fire()
            } else {
                let res = c.deposit(branch, amount);
                self.add_step_charge(cell, amount);
                res
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
        if let Some(d) = self.syn.uniform_delay() {
            let delivery_at = at.checked_add(d).expect("synaptic delay overflow");
            if self.syn.is_edge_w_synced() && self.edge_w.len() == self.syn.len() {
                let weights = &self.edge_w[start..end];
                debug_assert!(
                    weights
                        .iter()
                        .zip(&self.syn.as_slice()[start..end])
                        .all(|(w, s)| w.to_bits() == s.weight.to_bits()),
                    "Engine::edge_w diverged from Synapses while marked synced"
                );
                self.queue.insert_batch_at(
                    delivery_at,
                    cols.iter()
                        .zip(weights.iter())
                        .enumerate()
                        .filter(|&(_, (_, &weight))| weight != 0.0)
                        .map(|(offset, (&post, &weight))| {
                            let branch = ((start + offset) % K) as u8;
                            encode_event(post, branch, weight, false)
                        }),
                );
            } else {
                let syns = &self.syn.as_slice()[start..end];
                self.queue.insert_batch_at(
                    delivery_at,
                    cols.iter()
                        .zip(syns.iter())
                        .enumerate()
                        .filter(|&(_, (_, syn))| syn.weight != 0.0)
                        .map(|(offset, (&post, syn))| {
                            let branch = ((start + offset) % K) as u8;
                            encode_event(post, branch, syn.weight, false)
                        }),
                );
            }
        } else {
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

    #[test]
    fn mixed_thin_wide_and_cancelling_charges_match_sequential_bit_exact() {
        use crate::parallel::PartitionPlan;

        let n = 24usize;
        let rows: Vec<Vec<u32>> = (0..n)
            .map(|r| vec![((r + 1) % n) as u32, ((r + 5) % n) as u32])
            .collect();
        let conn = Csr::from_adjacency(&rows);
        let weights = vec![0.35_f32; conn.nnz()];
        let plan = PartitionPlan::degree_balanced(&conn, 4);

        let mut seq = Engine::with_cells(n);
        seq.set_connectivity(conn.clone(), weights.clone());
        let mut par_adapt = Engine::with_cells(n);
        par_adapt.set_connectivity(conn.clone(), weights.clone());
        let mut par_always = Engine::with_cells(n);
        par_always.set_connectivity(conn, weights);

        let mut rng = Rng::new(0xE176_17E5_2026);
        for round in 0..3u64 {
            let base = round * 20;
            // Cancelling deposits (+0.5 then -0.5) followed by +0.25 on same cell.
            for eng in [&mut seq, &mut par_adapt, &mut par_always] {
                eng.inject_weighted(3, 0, base + 1, 0.5);
                eng.inject_weighted(3, 0, base + 1, -0.5);
                eng.inject_weighted(3, 1, base + 2, 0.25);
            }
            // Interleaved multi-event thin tick (base + 4) and wide burst (base + 6).
            for i in 0..5usize {
                let c = (i % 3) as CellId;
                let w = 0.4 + 0.1 * (i as f32);
                for eng in [&mut seq, &mut par_adapt, &mut par_always] {
                    eng.inject_weighted(c, (i % K) as u8, base + 4, w);
                }
            }
            for i in 0..18usize {
                let c = ((i * 7) % n) as CellId;
                let w = 0.6 + 0.05 * ((i % 4) as f32);
                for eng in [&mut seq, &mut par_adapt, &mut par_always] {
                    if i % 5 == 0 {
                        eng.force_spike(c, base + 6);
                    } else {
                        eng.inject_weighted(c, (i % K) as u8, base + 6, w);
                    }
                }
            }
            let _ = rng.next_f32();

            let until = base + 15;
            let s_out = seq.step_until(until);
            let a_out = par_adapt.step_until_partitioned(until, &plan);
            let (w_out, _) = par_always.step_until_partitioned_threshold(until, &plan, 1);
            assert_eq!(a_out, s_out);
            assert_eq!(w_out, s_out);
            assert_eq!(par_adapt.work(), seq.work());
            assert_eq!(par_always.work(), seq.work());
            for c in 0..n as CellId {
                assert_eq!(
                    par_adapt.last_step_charge(c).to_bits(),
                    seq.last_step_charge(c).to_bits()
                );
                assert_eq!(
                    par_always.last_step_charge(c).to_bits(),
                    seq.last_step_charge(c).to_bits()
                );
                assert_eq!(par_adapt.cell(c).v.to_bits(), seq.cell(c).v.to_bits());
                assert_eq!(par_always.cell(c).v.to_bits(), seq.cell(c).v.to_bits());
            }
        }

        // Empty step must clear all previously dirty charges to exact +0.0.
        let _ = seq.step_until(100);
        let _ = par_adapt.step_until_partitioned(100, &plan);
        for c in 0..n as CellId {
            assert_eq!(seq.last_step_charge(c).to_bits(), 0);
            assert_eq!(par_adapt.last_step_charge(c).to_bits(), 0);
        }
    }

    #[test]
    fn lazy_reset_state_and_batched_fan_out_match_eager_reset_bit_for_bit() {
        let n = 64usize;
        let adj: Vec<Vec<u32>> = (0..n)
            .map(|r| {
                let mut v: Vec<u32> = (1..=6).map(|k| ((r + k * 5) % n) as u32).collect();
                v.sort_unstable();
                v.dedup();
                v
            })
            .collect();
        let conn = Csr::from_adjacency(&adj);
        let weights: Vec<f32> = (0..conn.nnz())
            .map(|i| {
                if i % 7 == 0 {
                    0.0
                } else {
                    0.45 + 0.1 * ((i % 4) as f32)
                }
            })
            .collect();

        let mut eng = Engine::with_cells(n);
        let mut custom = Cell::new(15.0);
        custom.v = -0.0;
        let extra = eng.push_cell(custom);
        assert_eq!(extra as usize, n);
        // Re-align connectivity to n+1 cells.
        let mut adj_ext = adj;
        adj_ext.push(vec![0, 1]);
        let conn_ext = Csr::from_adjacency(&adj_ext);
        let mut w_ext = weights;
        w_ext.extend_from_slice(&[0.8, 0.6]);
        eng.set_connectivity(conn_ext.clone(), w_ext.clone());

        // Trial 1: sparse touches (< 50% dirty) + cell_mut with -0.0.
        eng.cell_mut(3).v_dend[2] = -0.0;
        eng.force_spike(0, 2);
        let log1 = eng.step_until(8);
        assert!(!log1.is_empty());
        eng.reset_state();
        for id in 0..=n as CellId {
            assert!(
                eng.cell(id).is_at_rest(),
                "cell {id} not cleanly reset after sparse trial"
            );
        }
        assert!((eng.cell(extra).tau_m - 15.0).abs() <= 1e-6);

        // Trial 2: dense touches (> 50% dirty) via partitioned stepping + unsynced Synapses.
        let plan = PartitionPlan::degree_balanced(&conn_ext, 4);
        eng.syn.as_mut_slice_keeping_delays()[0].eligibility = 0.5;
        assert!(!eng.syn.is_edge_w_synced());
        for c in 0..40u32 {
            eng.inject_weighted(c, (c as usize % K) as u8, 1, 1.1);
        }
        let _ = eng.step_until_partitioned(10, &plan);
        eng.reset_state();
        for id in 0..=n as CellId {
            assert!(
                eng.cell(id).is_at_rest(),
                "cell {id} not cleanly reset after dense trial"
            );
        }

        // Trial 3: re-synced edge_w must produce bit-identical spikes to fresh engine.
        eng.syn.mark_edge_w_synced();
        let mut fresh = Engine::with_cells(n);
        let mut fresh_custom = Cell::new(15.0);
        fresh_custom.reset();
        fresh.push_cell(fresh_custom);
        fresh.set_connectivity(conn_ext, w_ext);

        for target in [&mut eng, &mut fresh] {
            target.force_spike(0, 1);
            target.inject_weighted(extra, 1, 2, 1.25);
        }
        let got = eng.step_until(12);
        let exp = fresh.step_until(12);
        assert_eq!(got, exp);
        for id in 0..=n as CellId {
            assert_eq!(eng.cell(id).v.to_bits(), fresh.cell(id).v.to_bits());
            assert_eq!(eng.cell(id).theta.to_bits(), fresh.cell(id).theta.to_bits());
        }
    }
}
