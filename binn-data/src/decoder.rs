//! Fixed decoder (U12). No `train` / `fit` until P4+ (GC4).

use crate::encoder::{CellId, Encoder, LatencyEncoder, PopulationEncoder, SpikeEvent};

/// Append-only spike train for decoding (data-layer; independent of engine).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SpikeLog {
    events: Vec<SpikeEvent>,
}

impl SpikeLog {
    /// Empty log.
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Wrap an owned event list.
    #[inline]
    pub fn from_events(events: Vec<SpikeEvent>) -> Self {
        Self { events }
    }

    /// Number of spikes.
    #[inline]
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// True when empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Borrow events.
    #[inline]
    pub fn as_slice(&self) -> &[SpikeEvent] {
        &self.events
    }

    /// Append one spike.
    #[inline]
    pub fn push(&mut self, event: SpikeEvent) {
        self.events.push(event);
    }

    /// Append many spikes.
    pub fn extend_from_slice(&mut self, events: &[SpikeEvent]) {
        self.events.extend_from_slice(events);
    }
}

impl From<Vec<SpikeEvent>> for SpikeLog {
    #[inline]
    fn from(events: Vec<SpikeEvent>) -> Self {
        Self::from_events(events)
    }
}

impl From<&[SpikeEvent]> for SpikeLog {
    #[inline]
    fn from(events: &[SpikeEvent]) -> Self {
        Self::from_events(events.to_vec())
    }
}

/// Decoder output: reconstructed features and optional class vote.
#[derive(Clone, Debug, PartialEq)]
pub struct Prediction {
    /// Reconstructed feature values in `[0, 1]`.
    pub values: Vec<f32>,
    /// Optional discrete class (when a labeled vote is available).
    pub class: Option<u32>,
    /// Confidence in `[0, 1]` (1 = fully confident).
    pub confidence: f32,
}

/// Fixed (non-learned) spike decoder.
///
/// # GC4
/// Implementations must not expose `train` / `fit`.
pub trait Decoder {
    /// Map a spike log back to a [`Prediction`].
    fn decode(&self, spikes: &SpikeLog) -> Prediction;
}

/// Inverse of [`LatencyEncoder`]: latency → intensity.
#[derive(Clone, Debug)]
pub struct LatencyDecoder {
    encoder: LatencyEncoder,
}

impl LatencyDecoder {
    /// Pair with an existing latency encoder (shares geometry).
    #[inline]
    pub fn from_encoder(encoder: LatencyEncoder) -> Self {
        Self { encoder }
    }

    /// Convenience constructor matching [`LatencyEncoder::new`].
    #[inline]
    pub fn new(n_features: usize, max_delay: binn_core::Tick, cell_offset: CellId) -> Self {
        Self::from_encoder(LatencyEncoder::new(n_features, max_delay, cell_offset))
    }
}

impl Decoder for LatencyDecoder {
    fn decode(&self, spikes: &SpikeLog) -> Prediction {
        let values = self.encoder.reconstruct(spikes.as_slice());
        Prediction {
            values,
            class: None,
            confidence: (1.0 - self.encoder.info_loss()).clamp(0.0, 1.0),
        }
    }
}

/// Inverse of [`PopulationEncoder`]: population center → intensity.
#[derive(Clone, Debug)]
pub struct PopulationDecoder {
    encoder: PopulationEncoder,
}

impl PopulationDecoder {
    /// Pair with an existing population encoder.
    #[inline]
    pub fn from_encoder(encoder: PopulationEncoder) -> Self {
        Self { encoder }
    }

    /// Convenience constructor matching [`PopulationEncoder::new`].
    #[inline]
    pub fn new(
        n_features: usize,
        cells_per_feature: usize,
        radius: usize,
        cell_offset: CellId,
    ) -> Self {
        Self::from_encoder(PopulationEncoder::new(
            n_features,
            cells_per_feature,
            radius,
            cell_offset,
        ))
    }
}

impl Decoder for PopulationDecoder {
    fn decode(&self, spikes: &SpikeLog) -> Prediction {
        let values = self.encoder.reconstruct(spikes.as_slice());
        Prediction {
            values,
            class: None,
            confidence: (1.0 - self.encoder.info_loss()).clamp(0.0, 1.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoder::{Encoder, Sample};

    #[test]
    fn latency_round_trip_near_identity_on_grid() {
        let enc = LatencyEncoder::new(3, 32, 0);
        let dec = LatencyDecoder::from_encoder(enc.clone());
        let sample = Sample::from_values(vec![0.0, 0.5, 1.0]);
        let spikes = SpikeLog::from(enc.encode(&sample));
        let pred = dec.decode(&spikes);
        for (a, b) in sample.values.iter().zip(pred.values.iter()) {
            assert!((a - b).abs() < 1.0 / 32.0 + 1e-5, "a={a} b={b}");
        }
    }
}
