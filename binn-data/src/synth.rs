//! Synthetic event-stream generators with known ground truth (U12).
//!
//! Spike-/time-friendly by construction — not statically-encoded image benchmarks.

use binn_core::Rng;

use crate::encoder::Sample;

/// Public config that reproduces a stream byte-for-byte (same seed ⇒ same samples).
#[derive(Clone, Debug, PartialEq)]
pub struct SynthConfig {
    /// ChaCha seed (GC3).
    pub seed: u64,
    /// Feature dimensionality.
    pub n_features: usize,
    /// Number of discrete classes.
    pub n_classes: usize,
    /// Length of each temporal sequence (1 = iid samples).
    pub sequence_len: usize,
    /// Noise / class-overlap knob in `[0, 1]` (0 = linearly separable).
    pub difficulty: f32,
    /// Compositional depth (reserved for C3 credit-depth tasks; ≥ 1).
    pub depth: usize,
}

impl SynthConfig {
    /// Sensible defaults for unit tests (temporal, multi-class).
    pub fn toy(seed: u64) -> Self {
        Self {
            seed,
            n_features: 4,
            n_classes: 3,
            sequence_len: 8,
            difficulty: 0.1,
            depth: 1,
        }
    }

    /// Stable fingerprint of the public config (not the drawn samples).
    pub fn fingerprint(&self) -> u64 {
        // FNV-1a 64 over the config fields.
        let mut h = 0xcbf2_9ce4_8422_2325u64;
        for word in [
            self.seed,
            self.n_features as u64,
            self.n_classes as u64,
            self.sequence_len as u64,
            self.difficulty.to_bits() as u64,
            self.depth as u64,
        ] {
            h ^= word;
            h = h.wrapping_mul(0x100_0000_01b3);
        }
        h
    }
}

/// Parametric, seeded, ground-truthed synthetic stream.
#[derive(Clone, Debug)]
pub struct SyntheticStream {
    config: SynthConfig,
    rng: Rng,
    index: u64,
}

impl SyntheticStream {
    /// Open a stream from `config`. Identical configs yield identical draws.
    ///
    /// # Panics
    ///
    /// Panics if `n_features == 0`, `n_classes == 0`, `sequence_len == 0`, or
    /// `depth == 0`.
    pub fn new(config: SynthConfig) -> Self {
        assert!(config.n_features > 0);
        assert!(config.n_classes > 0);
        assert!(config.sequence_len > 0);
        assert!(config.depth > 0);
        let rng = Rng::new(config.seed);
        Self {
            config,
            rng,
            index: 0,
        }
    }

    /// Borrow the public config.
    #[inline]
    pub fn config(&self) -> &SynthConfig {
        &self.config
    }

    /// Number of samples drawn so far.
    #[inline]
    pub fn index(&self) -> u64 {
        self.index
    }

    /// Config fingerprint (dataset byte-identity for harness logs).
    #[inline]
    pub fn config_fingerprint(&self) -> u64 {
        self.config.fingerprint()
    }

    /// Draw the next labeled sample.
    ///
    /// Ground truth: class `c` owns a one-hot-ish prototype on a cyclic feature
    /// subset; `difficulty` mixes in uniform noise. Depth > 1 XOR-folds extra
    /// latent bits into the label (compositional / credit-depth knob).
    pub fn next_sample(&mut self) -> Sample {
        let n_f = self.config.n_features;
        let n_c = self.config.n_classes;
        let diff = self.config.difficulty.clamp(0.0, 1.0);

        let base_class = self.rng.gen_index(n_c) as u32;
        let mut label = base_class;
        // Compositional depth: XOR additional latent bits into the label.
        for _ in 1..self.config.depth {
            let bit = self.rng.gen_index(n_c) as u32;
            label ^= bit;
        }
        label %= n_c as u32;

        let mut values = Vec::with_capacity(n_f);
        for i in 0..n_f {
            let proto = if i % n_c == label as usize {
                0.85
            } else {
                0.15
            };
            let noise = self.rng.next_f32();
            let v = (1.0 - diff) * proto + diff * noise;
            values.push(v.clamp(0.0, 1.0));
        }

        self.index += 1;
        Sample::with_label(values, label)
    }

    /// Draw a temporal sequence of length `config.sequence_len`.
    ///
    /// All frames share one ground-truth label (spike-friendly temporal object).
    pub fn next_sequence(&mut self) -> Vec<Sample> {
        let len = self.config.sequence_len;
        let first = self.next_sample();
        let label = first.label;
        let mut seq = Vec::with_capacity(len);
        seq.push(first);
        for _ in 1..len {
            let mut s = self.next_sample();
            // Preserve sequence-level label; keep per-frame feature jitter.
            s.label = label;
            seq.push(s);
        }
        // We over-advanced index by `len` already via next_sample; fine.
        seq
    }
}

/// Backward-compatible alias used by early stubs.
pub type Synth = SyntheticStream;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_reproduces_samples() {
        let cfg = SynthConfig::toy(0xB177_DA7A);
        let mut a = SyntheticStream::new(cfg.clone());
        let mut b = SyntheticStream::new(cfg);
        for _ in 0..32 {
            assert_eq!(a.next_sample(), b.next_sample());
        }
        assert_eq!(a.config_fingerprint(), b.config_fingerprint());
    }

    #[test]
    fn different_seeds_diverge() {
        let mut a = SyntheticStream::new(SynthConfig::toy(1));
        let mut b = SyntheticStream::new(SynthConfig::toy(2));
        assert_ne!(a.next_sample(), b.next_sample());
    }

    #[test]
    fn sequences_are_labeled_and_temporal() {
        let mut s = SyntheticStream::new(SynthConfig::toy(7));
        let seq = s.next_sequence();
        assert_eq!(seq.len(), 8);
        let label = seq[0].label;
        assert!(label.is_some());
        assert!(seq.iter().all(|x| x.label == label));
    }
}
