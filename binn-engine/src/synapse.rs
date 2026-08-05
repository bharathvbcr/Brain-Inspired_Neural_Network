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
