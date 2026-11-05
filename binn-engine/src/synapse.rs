//! Synapse storage (U05).
//!
//! Weight, trainable delay, and eligibility live here. Plasticity update rules
//! are applied by L5 (`binn-learn`); this module is storage only.

use std::sync::atomic::{AtomicU64, Ordering};

use binn_core::time::Tick;

const DIRTY_MAX_DELAY: u64 = u64::MAX;

/// Synaptic parameters (storage only; `Δw` is owned by L5).
#[derive(Clone, Debug)]
pub struct Synapse {
    /// Synaptic weight.
    pub weight: f32,
    /// Axonal / synaptic delay in ticks.
    pub delay: Tick,
    /// Eligibility trace (decayed / STDP-updated by L5).
    pub eligibility: f32,
    /// Tick of the last eligibility touch (for lazy decay in L5).
    pub last_elig_update: Tick,
    /// Slow eligibility trace (decayed / STDP-updated by L5 for temporal context).
    pub elig_slow: f32,
}

impl Synapse {
    /// Synapse with the given weight and delay; eligibility starts at zero.
    #[inline]
    pub fn new(weight: f32, delay: Tick) -> Self {
        // A zero delay makes `Engine::step_until` non-terminating. Its loop
        // drains the queue while the earliest tick is <= `until`, and delivery
        // re-inserts at `at + delay`; with `delay == 0` the new event lands on
        // the *same* tick, so the earliest tick never advances. The only damper
        // is the per-spike threshold rise of 0.2, which stops damping once
        // theta reaches 2^22, where an f32 half-ulp exceeds 0.2 and the
        // addition becomes a no-op — after which a zero-delay cycle re-fires
        // forever at a fixed threshold.
        //
        // Nothing reaches that today: `set_connectivity` hardcodes a delay of 1
        // and no call site passes 0. But `delay` is a public field on a public
        // struct, `Synapses::get_mut` hands out `&mut Synapse`, and the module
        // header advertises trainable delay as intended future work — so the
        // invariant is asserted rather than assumed. Several callers already
        // write `max_synaptic_delay().max(1)`, defending against a zero they
        // believe possible.
        assert!(delay > 0, "Synapse delay must be >= 1 (got {delay})");
        Self {
            weight,
            delay,
            eligibility: 0.0,
            last_elig_update: 0,
            elig_slow: 0.0,
        }
    }
}

#[inline]
fn is_clean_finite_weight(w: f32) -> bool {
    w.is_finite() && w.to_bits() != (-0.0f32).to_bits()
}

/// Flat synapse table owned by the engine.
///
/// Entries are aligned with [`crate::engine::Engine::edge_w`] / CSR nnz order
/// after [`Self::rebuild_from_weights`].
#[derive(Debug)]
pub struct Synapses {
    items: Vec<Synapse>,
    edge_w_synced: bool,
    weights_clean_finite: bool,
    dirty_elig_edges: Vec<u32>,
    elig_dirty_flag: Vec<u8>,
    elig_all_dirty: bool,
    elig_floor_tick: Tick,
    cached_max_delay: AtomicU64,
    uniform_delay: Option<Tick>,
}

impl Default for Synapses {
    #[inline]
    fn default() -> Self {
        Self {
            items: Vec::new(),
            edge_w_synced: true,
            weights_clean_finite: true,
            dirty_elig_edges: Vec::new(),
            elig_dirty_flag: Vec::new(),
            elig_all_dirty: false,
            elig_floor_tick: 0,
            cached_max_delay: AtomicU64::new(0),
            uniform_delay: None,
        }
    }
}

impl Clone for Synapses {
    fn clone(&self) -> Self {
        Self {
            items: self.items.clone(),
            edge_w_synced: self.edge_w_synced,
            weights_clean_finite: self.weights_clean_finite,
            dirty_elig_edges: self.dirty_elig_edges.clone(),
            elig_dirty_flag: self.elig_dirty_flag.clone(),
            elig_all_dirty: self.elig_all_dirty,
            elig_floor_tick: self.elig_floor_tick,
            cached_max_delay: AtomicU64::new(self.cached_max_delay.load(Ordering::Relaxed)),
            uniform_delay: self.uniform_delay,
        }
    }
}

impl Synapses {
    /// Empty table.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of stored synapses.
    #[inline]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// True when no synapses are stored.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Materialize any deferred `elig_floor_tick` onto idle synapses.
    pub fn flush_elig_floor_tick(&mut self) {
        let floor = self.elig_floor_tick;
        if floor == 0 {
            return;
        }
        for syn in &mut self.items {
            if syn.last_elig_update < floor {
                syn.last_elig_update = floor;
            }
        }
        self.elig_floor_tick = 0;
    }

    /// Append a synapse; returns its index.
    pub fn push(&mut self, mut syn: Synapse) -> usize {
        let id = self.items.len();
        let d = syn.delay;
        self.uniform_delay = match (self.items.is_empty(), self.uniform_delay) {
            (true, _) => Some(d),
            (false, Some(prev)) if prev == d => Some(prev),
            _ => None,
        };
        let cur = self.cached_max_delay.load(Ordering::Relaxed);
        if cur != DIRTY_MAX_DELAY {
            self.cached_max_delay.store(cur.max(d), Ordering::Relaxed);
        }
        self.edge_w_synced = false;
        if !is_clean_finite_weight(syn.weight) {
            self.weights_clean_finite = false;
        }
        if syn.last_elig_update < self.elig_floor_tick && syn.eligibility.to_bits() == 0 {
            syn.last_elig_update = self.elig_floor_tick;
        } else if syn.last_elig_update < self.elig_floor_tick {
            self.flush_elig_floor_tick();
        }
        let elig_dirty = syn.eligibility.to_bits() != 0;
        self.elig_dirty_flag.push(u8::from(elig_dirty));
        if elig_dirty && !self.elig_all_dirty {
            self.dirty_elig_edges.push(id as u32);
        }
        self.items.push(syn);
        id
    }

    /// Borrow synapse `id`.
    #[inline]
    pub fn get(&self, id: usize) -> Option<&Synapse> {
        self.items.get(id)
    }

    /// Mutably borrow synapse `id` (invalidates cached delay summary in case
    /// caller mutates [`Synapse::delay`]).
    #[inline]
    pub fn get_mut(&mut self, id: usize) -> Option<&mut Synapse> {
        self.flush_elig_floor_tick();
        self.cached_max_delay
            .store(DIRTY_MAX_DELAY, Ordering::Relaxed);
        self.uniform_delay = None;
        self.edge_w_synced = false;
        self.weights_clean_finite = false;
        self.elig_all_dirty = true;
        self.items.get_mut(id)
    }

    /// Borrow the underlying storage.
    #[inline]
    pub fn as_slice(&self) -> &[Synapse] {
        &self.items
    }

    /// Mutably borrow the underlying storage, invalidating cached delay
    /// summaries in case the caller mutates [`Synapse::delay`].
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [Synapse] {
        self.flush_elig_floor_tick();
        self.cached_max_delay
            .store(DIRTY_MAX_DELAY, Ordering::Relaxed);
        self.uniform_delay = None;
        self.edge_w_synced = false;
        self.weights_clean_finite = false;
        self.elig_all_dirty = true;
        &mut self.items
    }

    /// Mutably borrow the underlying storage for plasticity passes that mutate
    /// only weights / eligibility traces and leave [`Synapse::delay`] intact.
    #[inline]
    pub fn as_mut_slice_keeping_delays(&mut self) -> &mut [Synapse] {
        self.flush_elig_floor_tick();
        self.edge_w_synced = false;
        self.weights_clean_finite = false;
        self.elig_all_dirty = true;
        &mut self.items
    }

    /// True when every synapse may carry non-zero eligibility (`elig_all_dirty`).
    #[inline]
    pub fn is_all_elig_dirty(&self) -> bool {
        self.elig_all_dirty
    }

    /// True when synapse `id` may carry non-zero eligibility.
    #[inline]
    pub fn is_elig_dirty(&self, id: usize) -> bool {
        self.elig_all_dirty || self.elig_dirty_flag[id] != 0
    }

    /// Advance `last_elig_update` to `t` on a synapse whose eligibility is
    /// already guaranteed `+0.0`, without adding it to `dirty_elig_edges`.
    #[inline]
    pub fn stamp_clean_elig_time(&mut self, id: usize, t: Tick) {
        if t < self.elig_floor_tick {
            self.flush_elig_floor_tick();
        }
        self.items[id].last_elig_update = t;
    }

    /// Mutably borrow the underlying storage purely for eligibility/timestamp
    /// updates when [`Self::is_all_elig_dirty`] is already true (preserves
    /// `edge_w_synced`, `weights_clean_finite`, and delay caches).
    #[inline]
    pub fn as_mut_slice_for_all_dirty_elig(&mut self) -> &mut [Synapse] {
        debug_assert!(self.elig_all_dirty);
        self.flush_elig_floor_tick();
        &mut self.items
    }

    /// Mutably borrow one synapse purely for eligibility / timestamp updates
    /// (preserves delay summaries, `edge_w_synced`, and sparse dirty-eligibility tracking).
    #[inline]
    pub fn touch_elig(&mut self, id: usize) -> &mut Synapse {
        if !self.elig_all_dirty {
            let flag = &mut self.elig_dirty_flag[id];
            if *flag == 0 {
                *flag = 1;
                self.dirty_elig_edges.push(id as u32);
            }
        }
        let item = &mut self.items[id];
        if item.last_elig_update < self.elig_floor_tick {
            item.last_elig_update = self.elig_floor_tick;
        }
        item
    }

    /// True when sparse dirty-eligibility tracking is active and every stored
    /// weight is guaranteed finite and not `-0.0`.
    #[inline]
    pub fn can_sparse_apply_weights(&self) -> bool {
        !self.elig_all_dirty
            && self.weights_clean_finite
            && self.edge_w_synced
            && self.dirty_elig_edges.len() <= self.items.len() / 2
    }

    /// Apply a closure over only the synapses in `dirty_elig_edges`, keeping
    /// `edge_w_synced`, `elig_floor_tick`, and `dirty_elig_edges` synchronized
    /// in `O(|dirty_elig_edges|)`.
    pub fn apply_dirty_elig_weights(&mut self, now: Tick, mut f: impl FnMut(usize, &mut Synapse)) {
        let floor = self.elig_floor_tick;
        let items = self.items.as_mut_slice();
        let flags = self.elig_dirty_flag.as_mut_slice();
        let mut clean = self.weights_clean_finite;
        let mut write_pos = 0usize;
        for read_pos in 0..self.dirty_elig_edges.len() {
            let edge = self.dirty_elig_edges[read_pos];
            let idx = edge as usize;
            let syn = &mut items[idx];
            if syn.last_elig_update < floor {
                syn.last_elig_update = floor;
            }
            f(idx, syn);
            if !is_clean_finite_weight(syn.weight) {
                clean = false;
            }
            if syn.eligibility.to_bits() == 0 {
                flags[idx] = 0;
            } else {
                self.dirty_elig_edges[write_pos] = edge;
                write_pos += 1;
            }
        }
        self.dirty_elig_edges.truncate(write_pos);
        self.weights_clean_finite = clean;
        if now > self.elig_floor_tick {
            self.elig_floor_tick = now;
        }
    }

    /// Apply a closure over every synapse in a single pass when the caller
    /// mirrors updated weights into `Engine::edge_w` (zero extra per-synapse
    /// buffer writes or checks).
    #[inline]
    pub fn apply_all_weights(&mut self, mut f: impl FnMut(usize, &mut Synapse)) {
        self.flush_elig_floor_tick();
        self.elig_all_dirty = true;
        self.weights_clean_finite = false;
        self.edge_w_synced = true;
        for (idx, syn) in self.items.iter_mut().enumerate() {
            f(idx, syn);
        }
    }

    /// Update one synapse weight while caller also updates `Engine::edge_w[id]`,
    /// preserving cached delay summaries and `edge_w_synced`.
    #[inline]
    pub fn set_weight_keeping_delay(&mut self, id: usize, weight: f32) {
        self.items.get_mut(id).expect("aligned synapse").weight = weight;
        if !is_clean_finite_weight(weight) {
            self.weights_clean_finite = false;
        }
    }

    /// True when [`Synapse::weight`] has not been mutated through an unmirrored
    /// `&mut Synapse` slice since `Engine::edge_w` was last synchronized.
    #[inline]
    pub fn is_edge_w_synced(&self) -> bool {
        self.edge_w_synced
    }

    /// Mark `Engine::edge_w` as synchronized with [`Synapse::weight`] after a
    /// bulk pass that updated both arrays together.
    #[inline]
    pub fn mark_edge_w_synced(&mut self) {
        self.edge_w_synced = true;
    }

    /// Reset all non-zero eligibility traces to `0.0` in `O(dirty)` when tracked,
    /// without dirtying already zeroed cache lines or invalidating cached delay summaries.
    pub fn zero_eligibilities(&mut self) {
        if self.elig_all_dirty || self.dirty_elig_edges.len() > self.items.len() / 2 {
            let mut clean = true;
            for syn in &mut self.items {
                if syn.eligibility.to_bits() != 0 {
                    syn.eligibility = 0.0;
                }
                if !is_clean_finite_weight(syn.weight) {
                    clean = false;
                }
            }
            for edge in self.dirty_elig_edges.drain(..) {
                self.elig_dirty_flag[edge as usize] = 0;
            }
            if self.elig_all_dirty {
                self.elig_dirty_flag.fill(0);
                self.elig_dirty_flag.resize(self.items.len(), 0);
                self.elig_all_dirty = false;
            }
            self.weights_clean_finite = clean;
        } else {
            for edge in self.dirty_elig_edges.drain(..) {
                let idx = edge as usize;
                self.elig_dirty_flag[idx] = 0;
                if self.items[idx].eligibility.to_bits() != 0 {
                    self.items[idx].eligibility = 0.0;
                }
            }
        }
    }

    /// Largest installed synaptic delay, or `0` when empty (`O(1)` when cached).
    #[inline]
    pub fn max_delay(&self) -> Tick {
        let cached = self.cached_max_delay.load(Ordering::Relaxed);
        if cached != DIRTY_MAX_DELAY {
            return cached;
        }
        let max_d = self.items.iter().map(|syn| syn.delay).max().unwrap_or(0);
        self.cached_max_delay.store(max_d, Ordering::Relaxed);
        max_d
    }

    /// Uniform delay shared by every synapse when guaranteed unchanged since
    /// [`Self::rebuild_from_weights`] or [`Self::push`].
    #[inline]
    pub fn uniform_delay(&self) -> Option<Tick> {
        self.uniform_delay
    }

    /// Rebuild one synapse per edge weight (eligibility reset to zero).
    ///
    /// Index `i` matches CSR nnz order / `Engine::edge_w[i]`.
    pub fn rebuild_from_weights(&mut self, weights: &[f32], delay: Tick) {
        assert!(delay > 0, "Synapse delay must be >= 1 (got {delay})");
        self.items.clear();
        self.items.reserve(weights.len());
        let mut clean = true;
        for &weight in weights {
            if !is_clean_finite_weight(weight) {
                clean = false;
            }
            self.items.push(Synapse {
                weight,
                delay,
                eligibility: 0.0,
                last_elig_update: 0,
                elig_slow: 0.0,
            });
        }
        self.edge_w_synced = true;
        self.weights_clean_finite = clean;
        self.dirty_elig_edges.clear();
        self.elig_dirty_flag.clear();
        self.elig_dirty_flag.resize(weights.len(), 0);
        self.elig_all_dirty = false;
        self.elig_floor_tick = 0;
        if weights.is_empty() {
            self.cached_max_delay.store(0, Ordering::Relaxed);
            self.uniform_delay = None;
        } else {
            self.cached_max_delay.store(delay, Ordering::Relaxed);
            self.uniform_delay = Some(delay);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "Synapse delay must be >= 1")]
    fn a_zero_delay_is_refused_at_construction() {
        // `Engine::step_until` drains while the earliest queued tick is <=
        // `until`, and delivery re-inserts at `at + delay`. A zero delay lands
        // the new event on the same tick, so that condition never stops
        // holding and the loop does not terminate.
        Synapse::new(1.0, 0);
    }

    #[test]
    fn a_delay_of_one_is_the_smallest_accepted() {
        // The bound is >= 1, not > 1. Every real path goes through
        // `set_connectivity`, which uses exactly 1, so this pins that the
        // assert did not overshoot and break the only delay in use.
        assert_eq!(Synapse::new(1.0, 1).delay, 1);
    }

    #[test]
    #[should_panic(expected = "Synapse delay must be >= 1")]
    fn rebuilding_from_weights_refuses_a_zero_delay_too() {
        // The bulk path must not be a way around the constructor's check.
        Synapses::default().rebuild_from_weights(&[0.5, 0.25], 0);
    }

    #[test]
    fn max_delay_cache_stays_valid_for_eligibility_ops_and_invalidates_on_mut() {
        let mut s = Synapses::new();
        assert_eq!(s.max_delay(), 0);
        assert_eq!(s.uniform_delay(), None);

        s.rebuild_from_weights(&[0.1, 0.2, 0.3], 2);
        assert_eq!(s.max_delay(), 2);
        assert_eq!(s.uniform_delay(), Some(2));

        // Weight/eligibility-only operations keep the delay summary valid.
        s.set_weight_keeping_delay(1, 0.9);
        assert_eq!(s.get(1).expect("syn 1").weight, 0.9);
        assert!(s.is_edge_w_synced());
        s.as_mut_slice_keeping_delays()[0].eligibility = 1.25;
        assert!(!s.is_edge_w_synced());
        s.mark_edge_w_synced();
        assert!(s.is_edge_w_synced());
        s.as_mut_slice_keeping_delays()[2].eligibility = -0.0;
        s.mark_edge_w_synced();
        s.zero_eligibilities();
        assert!(s.is_edge_w_synced());
        assert!(s.can_sparse_apply_weights());
        assert_eq!(s.get(0).expect("syn 0").eligibility.to_bits(), 0);
        assert_eq!(s.get(2).expect("syn 2").eligibility.to_bits(), 0);
        assert_eq!(s.max_delay(), 2);
        assert_eq!(s.uniform_delay(), Some(2));

        // Direct `get_mut` / `as_mut_slice` invalidates so delay edits are never stale.
        s.get_mut(1).expect("syn 1").delay = 7;
        assert_eq!(s.uniform_delay(), None);
        assert_eq!(s.max_delay(), 7);
        s.as_mut_slice()[2].delay = 11;
        assert_eq!(s.max_delay(), 11);
    }

    #[test]
    fn sparse_dirty_eligibility_tracks_touches_zeros_and_floor_ticks() {
        let mut s = Synapses::new();
        s.rebuild_from_weights(&[0.2, 0.4, 0.6, 0.8], 1);
        assert!(s.can_sparse_apply_weights());

        s.touch_elig(2).eligibility = 0.5;
        s.touch_elig(2).last_elig_update = 10;
        s.apply_dirty_elig_weights(20, |_idx, syn| {
            syn.weight += 0.1 * syn.eligibility;
        });
        assert!((s.get(2).expect("syn 2").weight - 0.65).abs() < 1e-6);
        assert!(s.is_edge_w_synced());

        // Idle synapse acquires floor tick lazily on next touch or flush.
        assert_eq!(s.touch_elig(0).last_elig_update, 20);
        s.zero_eligibilities();
        assert_eq!(s.get(2).expect("syn 2").eligibility.to_bits(), 0);
        s.flush_elig_floor_tick();
        assert_eq!(s.get(1).expect("syn 1").last_elig_update, 20);
        assert_eq!(s.get(3).expect("syn 3").last_elig_update, 20);
    }
}
