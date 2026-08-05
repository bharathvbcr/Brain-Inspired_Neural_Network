//! Spike log (U05).

use binn_core::time::Tick;

use crate::cell::CellId;

/// One recorded spike.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Spike {
    /// Emission tick.
    pub t: Tick,
    /// Firing cell.
    pub cell: CellId,
}

/// Recorded spike train (append-only, deterministic order).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SpikeLog {
    events: Vec<Spike>,
}

impl SpikeLog {
    /// Empty log.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of recorded spikes.
    #[inline]
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// True when no spikes have been recorded.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Clear recorded spikes.
    #[inline]
    pub fn clear(&mut self) {
        self.events.clear();
    }

    /// Append a spike.
    #[inline]
    pub fn push(&mut self, t: Tick, cell: CellId) {
        self.events.push(Spike { t, cell });
    }

    /// Borrow the spike sequence.
    #[inline]
    pub fn as_slice(&self) -> &[Spike] {
        &self.events
    }

    /// Iterate spikes in recording order.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &Spike> {
        self.events.iter()
    }

    /// Append all spikes from `other`.
    pub fn extend_from(&mut self, other: &SpikeLog) {
        self.events.extend_from_slice(other.as_slice());
    }
}
