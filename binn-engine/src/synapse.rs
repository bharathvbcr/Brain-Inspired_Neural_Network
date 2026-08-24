//! Synapse storage (U05).
//!
//! Weight, trainable delay, and eligibility live here. Plasticity update rules
//! are applied by L5 (`binn-learn`); this module is storage only.

use binn_core::time::Tick;

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

/// Flat synapse table owned by the engine.
///
/// Entries are aligned with [`crate::engine::Engine::edge_w`] / CSR nnz order
/// after [`Self::rebuild_from_weights`].
#[derive(Clone, Debug, Default)]
pub struct Synapses {
    items: Vec<Synapse>,
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

    /// Append a synapse; returns its index.
    pub fn push(&mut self, syn: Synapse) -> usize {
        let id = self.items.len();
        self.items.push(syn);
        id
    }

    /// Borrow synapse `id`.
    #[inline]
    pub fn get(&self, id: usize) -> Option<&Synapse> {
        self.items.get(id)
    }

    /// Mutably borrow synapse `id`.
    #[inline]
    pub fn get_mut(&mut self, id: usize) -> Option<&mut Synapse> {
        self.items.get_mut(id)
    }

    /// Borrow the underlying storage.
    #[inline]
    pub fn as_slice(&self) -> &[Synapse] {
        &self.items
    }

    /// Mutably borrow the underlying storage (L5 plasticity hot path).
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [Synapse] {
        &mut self.items
    }

    /// Rebuild one synapse per edge weight (eligibility reset to zero).
    ///
    /// Index `i` matches CSR nnz order / `Engine::edge_w[i]`.
    pub fn rebuild_from_weights(&mut self, weights: &[f32], delay: Tick) {
        self.items.clear();
        self.items.reserve(weights.len());
        for &w in weights {
            self.items.push(Synapse::new(w, delay));
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
}
