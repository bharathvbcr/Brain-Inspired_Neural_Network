//! Dataset loaders (U12).
//!
//! Live synthetic temporal / coincidence / class-incremental streams used by the
//! C1 harness and tests. Prefer natively-temporal / spike-friendly tasks;
//! statically-encoded image benchmarks (e.g. MNIST) are intentionally **not**
//! first-class here.

use crate::encoder::Sample;
use crate::synth::{SynthConfig, SyntheticStream};

/// Temporal multi-class classification stream (synthetic).
#[derive(Clone, Debug)]
pub struct TemporalClassification {
    stream: SyntheticStream,
}

impl TemporalClassification {
    /// Open from an explicit synth config.
    #[inline]
    pub fn new(config: SynthConfig) -> Self {
        Self {
            stream: SyntheticStream::new(config),
        }
    }

    /// Default toy temporal task (seeded).
    #[inline]
    pub fn toy(seed: u64) -> Self {
        Self::new(SynthConfig::toy(seed))
    }

    /// Tunable compositional depth (C3 credit-depth knob; usable now for tests).
    pub fn with_depth(seed: u64, depth: usize) -> Self {
        let mut cfg = SynthConfig::toy(seed);
        cfg.depth = depth.max(1);
        Self::new(cfg)
    }

    /// Borrow the underlying stream.
    #[inline]
    pub fn stream(&self) -> &SyntheticStream {
        &self.stream
    }

    /// Mutable access for drawing.
    #[inline]
    pub fn stream_mut(&mut self) -> &mut SyntheticStream {
        &mut self.stream
    }

    /// Next temporal sequence.
    #[inline]
    pub fn next_sequence(&mut self) -> Vec<Sample> {
        self.stream.next_sequence()
    }

    /// Next iid sample.
    #[inline]
    pub fn next_sample(&mut self) -> Sample {
        self.stream.next_sample()
    }

    /// Config fingerprint for harness logs.
    #[inline]
    pub fn config_fingerprint(&self) -> u64 {
        self.stream.config_fingerprint()
    }
}

/// Coincidence / temporal-pairing task used by crux-style unit tests.
///
/// Positive class: two marked channels fire within a short lag.
/// Negative class: the same channels fire far apart (or only one fires).
#[derive(Clone, Debug)]
pub struct CoincidenceTask {
    stream: SyntheticStream,
    /// Max lag (in sequence index units) counted as coincident.
    pub max_lag: usize,
}

impl CoincidenceTask {
    /// Seeded coincidence task with `sequence_len` frames and binary labels.
    pub fn new(seed: u64, sequence_len: usize, max_lag: usize) -> Self {
        let max_lag = max_lag.max(1);
        let cfg = SynthConfig {
            seed,
            n_features: 2,
            n_classes: 2,
            // Guarantee a non-coincident placement always fits (len >= max_lag + 2)
            // so both classes are constructible without wrap-around.
            sequence_len: sequence_len.max(max_lag + 2),
            difficulty: 0.05,
            depth: 1,
        };
        Self {
            stream: SyntheticStream::new(cfg),
            max_lag,
        }
    }

    /// Draw one sequence and its coincidence label.
    ///
    /// Label `1` iff the peak frames of feature 0 and feature 1 differ by at
    /// most `max_lag`.
    pub fn next_trial(&mut self) -> (Vec<Sample>, u32) {
        let draw_id = self.stream.index();
        let mut seq = self.stream.next_sequence();
        let len = seq.len();
        // `stream.index()` advances by `sequence_len` per trial (one sample per
        // frame). Parity of the sample index is therefore always even when
        // `sequence_len` is even — use the trial ordinal for strict 50/50.
        let trial_id = draw_id / len.max(1) as u64;
        // Deterministic pseudo-random spread from draw_id (splitmix64), so
        // placement varies across trials without an external RNG.
        let mut h = draw_id.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 0xD1B5_4A32_D192_ED03;
        h ^= h >> 30;
        h = h.wrapping_mul(0xBF58_476D_1CE4_E5B9);
        h ^= h >> 27;
        // Strict alternation ⇒ exactly 50/50 labels over any even-length run,
        // so the constant-predictor chance rate is 0.5. Positions use *linear*
        // (non-wrapping) distance, so the constructed lag always matches the
        // label rule (the previous `% len` wrap could flip a positive to 0).
        let positive = (trial_id & 1) == 0;
        let (t0, t1) = if positive {
            let lag = (h % (self.max_lag as u64 + 1)) as usize; // 0..=max_lag
            let hi = len - 1 - lag; // keep t1 < len (no wrap)
            let a = (h >> 8) as usize % (hi + 1);
            (a, a + lag)
        } else {
            // len >= max_lag + 2 guarantees this range is non-empty.
            let span = (len - 1) - (self.max_lag + 1);
            let lag = self.max_lag + 1 + (h % (span as u64 + 1)) as usize; // > max_lag
            let hi = len - 1 - lag;
            let a = (h >> 8) as usize % (hi + 1);
            (a, a + lag)
        };
        let lag = t1 - t0; // linear distance, no wrap
        let label = u32::from(lag <= self.max_lag);
        debug_assert_eq!(
            label,
            u32::from(positive),
            "construction must agree with the coincidence label rule"
        );

        for (t, s) in seq.iter_mut().enumerate() {
            s.values = vec![0.0, 0.0];
            if t == t0 {
                s.values[0] = 0.95;
            }
            if t == t1 {
                s.values[1] = 0.95;
            }
            s.label = Some(label);
        }
        (seq, label)
    }

    /// Config fingerprint.
    #[inline]
    pub fn config_fingerprint(&self) -> u64 {
        self.stream.config_fingerprint() ^ (self.max_lag as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
    }
}

/// Placeholder handle retained for scaffold naming.
pub type Datasets = TemporalClassification;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn temporal_classification_is_seeded() {
        let mut a = TemporalClassification::toy(11);
        let mut b = TemporalClassification::toy(11);
        assert_eq!(a.next_sample(), b.next_sample());
        assert_eq!(a.config_fingerprint(), b.config_fingerprint());
    }

    #[test]
    fn with_depth_changes_fingerprint_and_labels() {
        let d1 = TemporalClassification::with_depth(5, 1);
        let d3 = TemporalClassification::with_depth(5, 3);
        assert_ne!(d1.config_fingerprint(), d3.config_fingerprint());
        assert_eq!(d1.stream().config().depth, 1);
        assert_eq!(d3.stream().config().depth, 3);
    }

    #[test]
    fn coincidence_task_labels_match_lag_rule() {
        let mut task = CoincidenceTask::new(99, 16, 2);
        for _ in 0..20 {
            let (seq, label) = task.next_trial();
            assert_eq!(seq.len(), 16);
            let mut t0 = None;
            let mut t1 = None;
            for (t, s) in seq.iter().enumerate() {
                if s.values[0] > 0.5 {
                    t0 = Some(t);
                }
                if s.values[1] > 0.5 {
                    t1 = Some(t);
                }
            }
            let lag = t0.unwrap().abs_diff(t1.unwrap());
            let expected = u32::from(lag <= 2);
            assert_eq!(label, expected);
        }
    }

    #[test]
    fn coincidence_task_is_label_balanced() {
        // Chance-rate sanity: a constant predictor must score 0.5, so labels
        // must be balanced. Strict alternation gives exactly 50/50.
        for (seed, len, max_lag) in [(7u64, 8usize, 1usize), (13, 12, 2), (21, 6, 1)] {
            let mut task = CoincidenceTask::new(seed, len, max_lag);
            let n = 400usize;
            let ones: usize = (0..n).map(|_| task.next_trial().1 as usize).sum();
            assert_eq!(ones, n / 2, "labels must be 50/50 (chance predictor = 0.5)");
        }
    }
}
