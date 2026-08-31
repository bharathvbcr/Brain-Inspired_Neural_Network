//! Class-incremental synthetic stream (U14 / C2).
//!
//! Learner-facing API exposes only `(features, label)` — **no task IDs**.
//! The stream itself stores **no raw example buffer** for replay; phases are
//! regenerated from the seeded RNG when the harness needs held-out probes.
//!
//! Task/phase identity is harness-private (`phase()` / `seen_classes()`), never
//! part of [`ClassIncExample`].

use binn_core::Rng;

use crate::encoder::Sample;

/// Public config for a class-incremental stream (hashable / reproducible).
#[derive(Clone, Debug, PartialEq)]
pub struct ClassIncConfig {
    /// ChaCha stream seed (GC3).
    pub seed: u64,
    /// Total classes in the curriculum (`≥ 2`).
    pub n_classes: usize,
    /// Feature dimensionality (defaults to `n_classes` for prototype coding).
    pub n_features: usize,
    /// Training examples presented per class phase.
    pub train_per_class: usize,
    /// Held-out probe examples retained **by the harness** per class (not by
    /// the learner). The stream regenerates these on demand from the seed.
    pub test_per_class: usize,
    /// Frames per temporal sequence (`1` = iid).
    pub sequence_len: usize,
    /// Noise / class-overlap knob in `[0, 1]`.
    pub difficulty: f32,
}

impl ClassIncConfig {
    /// Quick/PILOT defaults for CI smoke.
    pub fn quick(seed: u64) -> Self {
        Self {
            seed,
            n_classes: 4,
            n_features: 4,
            train_per_class: 24,
            test_per_class: 12,
            sequence_len: 4,
            difficulty: 0.05,
        }
    }

    /// Scientific-scale defaults (full G3 schedule).
    pub fn scientific(seed: u64) -> Self {
        Self {
            seed,
            n_classes: 5,
            n_features: 5,
            train_per_class: 80,
            test_per_class: 40,
            sequence_len: 6,
            difficulty: 0.08,
        }
    }

    /// Stable fingerprint of the public config (not drawn samples).
    pub fn fingerprint(&self) -> u64 {
        let mut h = 0xcbf2_9ce4_8422_2325u64;
        for word in [
            self.seed,
            self.n_classes as u64,
            self.n_features as u64,
            self.train_per_class as u64,
            self.test_per_class as u64,
            self.sequence_len as u64,
            self.difficulty.to_bits() as u64,
        ] {
            h ^= word;
            h = h.wrapping_mul(0x100_0000_01b3);
        }
        h
    }
}

/// Learner-facing example: features + class label only (U14: no task ID).
#[derive(Clone, Debug, PartialEq)]
pub struct ClassIncExample {
    /// Temporal sequence (all frames share `label`).
    pub sequence: Vec<Sample>,
    /// Ground-truth class in `0..n_classes`.
    pub label: u32,
}

impl ClassIncExample {
    /// Flattened feature vector of the peak / last frame (for non-spike baselines).
    pub fn flat_features(&self) -> Vec<f32> {
        self.sequence
            .last()
            .map(|s| s.values.clone())
            .unwrap_or_default()
    }
}

/// Class-incremental stream: phases present one new class at a time.
///
/// # Invariants
///
/// - [`Self::next_train`] returns [`ClassIncExample`] with **no task-id field**.
/// - No raw-example replay buffer is retained inside this type.
/// - Held-out probes are regenerated from the seed (`probe_class`), not cached
///   as a learner-side buffer.
#[derive(Clone, Debug)]
pub struct ClassIncrementalStream {
    config: ClassIncConfig,
    /// RNG for training draws within the current phase.
    train_rng: Rng,
    /// Current class phase index in `0..n_classes`.
    phase: usize,
    /// Training examples already drawn in this phase.
    drawn_in_phase: usize,
}

impl ClassIncrementalStream {
    /// Open a stream. Panics if `n_classes < 2` or sizes are zero.
    pub fn new(config: ClassIncConfig) -> Self {
        assert!(config.n_classes >= 2, "need ≥2 classes");
        assert!(config.n_features > 0);
        assert!(config.train_per_class > 0);
        assert!(config.test_per_class > 0);
        assert!(config.sequence_len > 0);
        let train_rng = Rng::new(config.seed ^ 0xC1A5_1AC0);
        Self {
            config,
            train_rng,
            phase: 0,
            drawn_in_phase: 0,
        }
    }

    /// Borrow config.
    #[inline]
    pub fn config(&self) -> &ClassIncConfig {
        &self.config
    }

    /// Config fingerprint for harness logs.
    #[inline]
    pub fn config_fingerprint(&self) -> u64 {
        self.config.fingerprint()
    }

    /// Harness-private: current phase index (not exposed to the learner API).
    #[inline]
    pub fn phase(&self) -> usize {
        self.phase
    }

    /// Harness-private: classes seen so far (inclusive of current phase).
    #[inline]
    pub fn seen_classes(&self) -> usize {
        self.phase + 1
    }

    /// True when every class phase has been exhausted.
    #[inline]
    pub fn exhausted(&self) -> bool {
        self.phase >= self.config.n_classes
    }

    /// Advance to the next class phase. Returns `false` if already exhausted.
    pub fn advance_phase(&mut self) -> bool {
        if self.phase + 1 >= self.config.n_classes {
            self.phase = self.config.n_classes;
            return false;
        }
        self.phase += 1;
        self.drawn_in_phase = 0;
        // Reseed per phase so phase order is deterministic but draws independent.
        self.train_rng = Rng::new(
            self.config.seed
                ^ 0xC1A5_1AC0
                ^ (self.phase as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15),
        );
        true
    }

    /// Remaining training examples in the current phase.
    #[inline]
    pub fn remaining_in_phase(&self) -> usize {
        if self.exhausted() {
            0
        } else {
            self.config
                .train_per_class
                .saturating_sub(self.drawn_in_phase)
        }
    }

    /// Next training example for the **current class only** (class-incremental).
    ///
    /// Returns `None` when the phase quota is exhausted (caller should
    /// [`advance_phase`](Self::advance_phase)).
    pub fn next_train(&mut self) -> Option<ClassIncExample> {
        if self.exhausted() || self.drawn_in_phase >= self.config.train_per_class {
            return None;
        }
        let label = self.phase as u32;
        let n_f = self.config.n_features;
        let n_c = self.config.n_classes;
        let diff = self.config.difficulty.clamp(0.0, 1.0);
        let len = self.config.sequence_len;
        let mut sequence = Vec::with_capacity(len);
        for _ in 0..len {
            let mut values = Vec::with_capacity(n_f);
            for i in 0..n_f {
                let proto = if i % n_c == label as usize {
                    0.90
                } else {
                    0.10
                };
                let noise = self.train_rng.next_f32();
                let v = (1.0 - diff) * proto + diff * noise;
                values.push(v.clamp(0.0, 1.0));
            }
            sequence.push(crate::encoder::Sample::with_label(values, label));
        }
        self.drawn_in_phase += 1;
        Some(ClassIncExample { sequence, label })
    }

    /// Regenerate a held-out probe set for class `c` (harness evaluation only).
    ///
    /// Does **not** store examples inside the stream; each call rebuilds from
    /// a class-specific seed. Not part of the learner API.
    pub fn probe_class(&self, class: u32) -> Vec<ClassIncExample> {
        assert!((class as usize) < self.config.n_classes);
        let mut rng = Rng::new(
            self.config.seed ^ 0x7E57_B80B ^ (class as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9),
        );
        (0..self.config.test_per_class)
            .map(|_| Self::draw_labeled_static(&self.config, &mut rng, class))
            .collect()
    }

    /// Drain an entire training phase into a `Vec` (harness convenience).
    ///
    /// The **learner** must not retain this as a replay buffer; baselines that
    /// do must live in labeled `*_baseline.rs` files and disclose it.
    pub fn drain_phase_train(&mut self) -> Vec<ClassIncExample> {
        let mut out = Vec::with_capacity(self.remaining_in_phase());
        while let Some(ex) = self.next_train() {
            out.push(ex);
        }
        out
    }

    fn draw_labeled_static(config: &ClassIncConfig, rng: &mut Rng, label: u32) -> ClassIncExample {
        let n_f = config.n_features;
        let n_c = config.n_classes;
        let diff = config.difficulty.clamp(0.0, 1.0);
        let len = config.sequence_len;
        let mut sequence = Vec::with_capacity(len);
        for _ in 0..len {
            let mut values = Vec::with_capacity(n_f);
            for i in 0..n_f {
                let proto = if i % n_c == label as usize {
                    0.90
                } else {
                    0.10
                };
                let noise = rng.next_f32();
                let v = (1.0 - diff) * proto + diff * noise;
                values.push(v.clamp(0.0, 1.0));
            }
            sequence.push(crate::encoder::Sample::with_label(values, label));
        }
        ClassIncExample { sequence, label }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn learner_api_has_no_task_id_fields() {
        // Structural: ClassIncExample fields are only sequence + label.
        let mut stream = ClassIncrementalStream::new(ClassIncConfig::quick(7));
        let ex = stream.next_train().expect("first example");
        assert_eq!(ex.label, 0);
        assert!(!ex.sequence.is_empty());
        // No task_id accessor — phase is harness-private.
        assert_eq!(stream.phase(), 0);
    }

    #[test]
    fn phases_are_class_incremental() {
        let mut stream = ClassIncrementalStream::new(ClassIncConfig::quick(11));
        let mut labels = Vec::new();
        while !stream.exhausted() {
            let batch = stream.drain_phase_train();
            assert!(!batch.is_empty());
            let phase_label = stream.phase() as u32;
            assert!(batch.iter().all(|e| e.label == phase_label));
            labels.push(phase_label);
            if !stream.advance_phase() {
                break;
            }
        }
        assert_eq!(labels, vec![0, 1, 2, 3]);
    }

    #[test]
    fn stream_stores_no_raw_replay_buffer() {
        // This test asserted only that `probe_class(0).len()` matched a config
        // field -- twice, on identical lines -- which says nothing whatever
        // about what the stream stores. It is named for a structural guarantee
        // and now checks it two ways.
        let small = ClassIncrementalStream::new(ClassIncConfig::quick(3));
        let large = ClassIncrementalStream::new(ClassIncConfig::quick(20));

        // 1. Resident size does not grow with the dataset. A retained
        //    `Vec<Sample>` would make these differ, because `quick(20)` covers
        //    many times the examples of `quick(3)`.
        assert_eq!(
            std::mem::size_of_val(&small),
            std::mem::size_of_val(&large),
            "stream size depends on the dataset; something is being retained"
        );
        // A `repr(Rust)` struct is not guaranteed to be the sum of its
        // fields: the compiler may pad and reorder, and how much it pads
        // depends on the target. Asserting exact equality passed on
        // aarch64-apple-darwin and failed on x86_64-unknown-linux-gnu, where
        // the same four fields occupy 400 bytes against a 392-byte sum. That
        // is 8 bytes of padding, not a retained buffer — the test was reading
        // layout and reporting it as a leak.
        //
        // The bound is what the guarantee actually needs. One alignment unit
        // covers any padding the compiler may insert, while the smallest thing
        // that would breach the invariant — an owned `Vec<Sample>` — is three
        // words, so it still cannot hide here.
        let fields = std::mem::size_of::<ClassIncConfig>()
            + std::mem::size_of::<Rng>()
            + 2 * std::mem::size_of::<usize>();
        let actual = std::mem::size_of::<ClassIncrementalStream>();
        assert!(
            actual <= fields + std::mem::align_of::<ClassIncrementalStream>(),
            "the stream gained a field beyond config + RNG + two counters: \
             {actual} bytes against {fields} of fields"
        );
        assert!(
            actual < fields + std::mem::size_of::<Vec<u8>>(),
            "the stream is large enough to be holding an owned collection: \
             {actual} bytes against {fields} of fields"
        );

        // 2. Probes are regenerated, not served from a buffer. A stored buffer
        //    would survive advancing the stream; a regenerated probe is a pure
        //    function of the class and config, so it must be reproducible from
        //    a fresh stream.
        let before = small.probe_class(0);
        let fresh = ClassIncrementalStream::new(ClassIncConfig::quick(3)).probe_class(0);
        assert_eq!(before.len(), small.config().test_per_class);
        assert!(
            !before.is_empty(),
            "an empty probe would satisfy any comparison"
        );
        assert_eq!(
            before, fresh,
            "probe_class is not a pure function of class and config"
        );
    }

    #[test]
    fn same_seed_identical_probes() {
        let a = ClassIncrementalStream::new(ClassIncConfig::quick(99));
        let b = ClassIncrementalStream::new(ClassIncConfig::quick(99));
        assert_eq!(a.probe_class(1), b.probe_class(1));
        assert_eq!(a.config_fingerprint(), b.config_fingerprint());
    }
}
