//! Fixed encoder (U12). No `train` / `fit` until P4+ (GC4).
//!
//! Latency and population codes only. Information loss is *measured* (round-trip
//! reconstruction MSE), never optimized away by a trainable I/O boundary.

use binn_core::Tick;

/// Cell index used by spike events (matches engine `CellId` width).
pub type CellId = u32;

/// One encoded spike produced by an [`Encoder`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SpikeEvent {
    /// Emission tick (relative to encode origin).
    pub t: Tick,
    /// Target / source cell.
    pub cell: CellId,
}

/// One labeled feature vector for encoding.
#[derive(Clone, Debug, PartialEq)]
pub struct Sample {
    /// Feature values in `[0, 1]` (callers clamp before encode).
    pub values: Vec<f32>,
    /// Optional ground-truth class label.
    pub label: Option<u32>,
}

impl Sample {
    /// Unlabeled sample.
    #[inline]
    pub fn from_values(values: Vec<f32>) -> Self {
        Self {
            values,
            label: None,
        }
    }

    /// Labeled sample.
    #[inline]
    pub fn with_label(values: Vec<f32>, label: u32) -> Self {
        Self {
            values,
            label: Some(label),
        }
    }
}

/// Fixed (non-learned) spike encoder.
///
/// # GC4
/// Implementations must not expose `train` / `fit`. Info-loss is reported via
/// [`info_loss`](Encoder::info_loss), never optimized by gradient descent here.
pub trait Encoder {
    /// Deterministic encode of `x` into a spike event list.
    fn encode(&self, x: &Sample) -> Vec<SpikeEvent>;

    /// Measured information loss in `[0, 1]` (1 = total loss).
    ///
    /// Computed from encode→decode round-trip reconstruction error on a fixed
    /// probe grid; not a trainable objective.
    fn info_loss(&self) -> f32;
}

/// Latency code: higher feature intensity → earlier spike.
///
/// Feature `i` with value `v ∈ [0,1]` maps to cell `cell_offset + i` at
/// `t = round((1 − v) · max_delay)`.
#[derive(Clone, Debug)]
pub struct LatencyEncoder {
    n_features: usize,
    max_delay: Tick,
    cell_offset: CellId,
    /// Round-trip MSE on the unit probe grid (set at construction).
    info_loss: f32,
}

impl LatencyEncoder {
    /// Build a latency encoder and measure its quantization info-loss.
    ///
    /// # Panics
    ///
    /// Panics if `n_features == 0` or `max_delay == 0`.
    pub fn new(n_features: usize, max_delay: Tick, cell_offset: CellId) -> Self {
        assert!(n_features > 0, "latency encoder requires n_features > 0");
        assert!(max_delay > 0, "latency encoder requires max_delay > 0");
        let mut enc = Self {
            n_features,
            max_delay,
            cell_offset,
            info_loss: 0.0,
        };
        enc.info_loss = measure_latency_info_loss(&enc);
        enc
    }

    /// Number of input features.
    #[inline]
    pub fn n_features(&self) -> usize {
        self.n_features
    }

    /// Maximum latency bin (ticks).
    #[inline]
    pub fn max_delay(&self) -> Tick {
        self.max_delay
    }

    /// Decode spikes produced by this encoder back to feature values.
    pub fn reconstruct(&self, spikes: &[SpikeEvent]) -> Vec<f32> {
        let mut out = vec![0.0f32; self.n_features];
        let mut seen = vec![false; self.n_features];
        for ev in spikes {
            let cell = ev.cell;
            if cell < self.cell_offset {
                continue;
            }
            let i = (cell - self.cell_offset) as usize;
            if i >= self.n_features || seen[i] {
                continue;
            }
            seen[i] = true;
            let t = ev.t.min(self.max_delay) as f32;
            let v = 1.0 - t / self.max_delay as f32;
            out[i] = v.clamp(0.0, 1.0);
        }
        out
    }
}

impl Encoder for LatencyEncoder {
    fn encode(&self, x: &Sample) -> Vec<SpikeEvent> {
        assert_eq!(
            x.values.len(),
            self.n_features,
            "sample feature count must match encoder"
        );
        let mut events = Vec::with_capacity(self.n_features);
        for (i, &raw) in x.values.iter().enumerate() {
            let v = raw.clamp(0.0, 1.0);
            // Near-zero / silent features emit no spike (true silence).
            if v <= 0.0 {
                continue;
            }
            let t = ((1.0 - v) * self.max_delay as f32).round() as Tick;
            events.push(SpikeEvent {
                t: t.min(self.max_delay),
                cell: self.cell_offset + i as CellId,
            });
        }
        // Deterministic order: sort by (t, cell).
        events.sort_by(|a, b| a.t.cmp(&b.t).then(a.cell.cmp(&b.cell)));
        events
    }

    #[inline]
    fn info_loss(&self) -> f32 {
        self.info_loss
    }
}

fn measure_latency_info_loss(enc: &LatencyEncoder) -> f32 {
    // Dense probe on the unit interval; MSE of round-trip quantization.
    const STEPS: usize = 33;
    let mut sse = 0.0f64;
    let mut n = 0usize;
    for step in 0..STEPS {
        let v = step as f32 / (STEPS - 1) as f32;
        let sample = Sample::from_values(vec![v; enc.n_features]);
        let spikes = enc.encode(&sample);
        let recon = enc.reconstruct(&spikes);
        for (a, b) in sample.values.iter().zip(recon.iter()) {
            let d = f64::from(*a - *b);
            sse += d * d;
            n += 1;
        }
    }
    if n == 0 {
        return 0.0;
    }
    (sse / n as f64) as f32
}

/// Population code: each feature activates a local neighborhood of cells.
///
/// Feature `i` with value `v` centers activity at
/// `cell_offset + i · cells_per_feature + round(v · (cells_per_feature − 1))`,
/// then fires a symmetric window of radius `radius`.
#[derive(Clone, Debug)]
pub struct PopulationEncoder {
    n_features: usize,
    cells_per_feature: usize,
    radius: usize,
    cell_offset: CellId,
    info_loss: f32,
}

impl PopulationEncoder {
    /// Build a population encoder and measure its quantization info-loss.
    ///
    /// # Panics
    ///
    /// Panics if `n_features == 0` or `cells_per_feature == 0`.
    pub fn new(
        n_features: usize,
        cells_per_feature: usize,
        radius: usize,
        cell_offset: CellId,
    ) -> Self {
        assert!(n_features > 0, "population encoder requires n_features > 0");
        assert!(
            cells_per_feature > 0,
            "population encoder requires cells_per_feature > 0"
        );
        let mut enc = Self {
            n_features,
            cells_per_feature,
            radius,
            cell_offset,
            info_loss: 0.0,
        };
        enc.info_loss = measure_population_info_loss(&enc);
        enc
    }

    /// Cells allocated per feature.
    #[inline]
    pub fn cells_per_feature(&self) -> usize {
        self.cells_per_feature
    }

    /// Reconstruct feature values from population spike centers.
    pub fn reconstruct(&self, spikes: &[SpikeEvent]) -> Vec<f32> {
        let mut out = vec![0.0f32; self.n_features];
        let mut counts = vec![0u32; self.n_features];
        let span = (self.cells_per_feature.saturating_sub(1)).max(1) as f32;
        for ev in spikes {
            if ev.cell < self.cell_offset {
                continue;
            }
            let local = (ev.cell - self.cell_offset) as usize;
            let feat = local / self.cells_per_feature;
            if feat >= self.n_features {
                continue;
            }
            let bin = local % self.cells_per_feature;
            out[feat] += bin as f32 / span;
            counts[feat] += 1;
        }
        for i in 0..self.n_features {
            if counts[i] > 0 {
                out[i] = (out[i] / counts[i] as f32).clamp(0.0, 1.0);
            }
        }
        out
    }
}

impl Encoder for PopulationEncoder {
    fn encode(&self, x: &Sample) -> Vec<SpikeEvent> {
        assert_eq!(
            x.values.len(),
            self.n_features,
            "sample feature count must match encoder"
        );
        let span = (self.cells_per_feature.saturating_sub(1)).max(1);
        let mut events = Vec::new();
        for (i, &raw) in x.values.iter().enumerate() {
            let v = raw.clamp(0.0, 1.0);
            let center = (v * span as f32).round() as isize;
            let base = (i * self.cells_per_feature) as isize;
            let lo = (center - self.radius as isize).max(0);
            let hi = (center + self.radius as isize).min(self.cells_per_feature as isize - 1);
            for bin in lo..=hi {
                let cell = self.cell_offset + (base + bin) as CellId;
                events.push(SpikeEvent { t: 0, cell });
            }
        }
        events.sort_by(|a, b| a.t.cmp(&b.t).then(a.cell.cmp(&b.cell)));
        events
    }

    #[inline]
    fn info_loss(&self) -> f32 {
        self.info_loss
    }
}

fn measure_population_info_loss(enc: &PopulationEncoder) -> f32 {
    const STEPS: usize = 33;
    let mut sse = 0.0f64;
    let mut n = 0usize;
    for step in 0..STEPS {
        let v = step as f32 / (STEPS - 1) as f32;
        let sample = Sample::from_values(vec![v; enc.n_features]);
        let spikes = enc.encode(&sample);
        let recon = enc.reconstruct(&spikes);
        for (a, b) in sample.values.iter().zip(recon.iter()) {
            let d = f64::from(*a - *b);
            sse += d * d;
            n += 1;
        }
    }
    if n == 0 {
        return 0.0;
    }
    (sse / n as f64) as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn latency_encode_is_deterministic() {
        let enc = LatencyEncoder::new(4, 10, 0);
        let sample = Sample::with_label(vec![0.0, 0.25, 0.5, 1.0], 1);
        let a = enc.encode(&sample);
        let b = enc.encode(&sample);
        assert_eq!(a, b);
        // v=0 is silent (no spike); remaining three features fire.
        assert_eq!(a.len(), 3);
        // v=1 → t=0; v=0 → no event
        assert!(a.iter().any(|e| e.cell == 3 && e.t == 0));
        assert!(!a.iter().any(|e| e.cell == 0));
        let recon = enc.reconstruct(&a);
        assert_eq!(recon[0], 0.0, "missing spikes reconstruct as silence");
    }

    #[test]
    fn latency_info_loss_is_reported_and_finite() {
        let coarse = LatencyEncoder::new(2, 2, 0);
        let fine = LatencyEncoder::new(2, 64, 0);
        let loss_coarse = coarse.info_loss();
        let loss_fine = fine.info_loss();
        assert!(loss_coarse.is_finite() && loss_coarse >= 0.0);
        assert!(loss_fine.is_finite() && loss_fine >= 0.0);
        assert!(
            loss_coarse > loss_fine,
            "finer latency bins should lose less info: coarse={loss_coarse} fine={loss_fine}"
        );
    }

    #[test]
    fn population_encode_is_deterministic() {
        let enc = PopulationEncoder::new(2, 8, 1, 100);
        let sample = Sample::from_values(vec![0.0, 1.0]);
        assert_eq!(enc.encode(&sample), enc.encode(&sample));
        assert!(enc.info_loss().is_finite());
    }
}
