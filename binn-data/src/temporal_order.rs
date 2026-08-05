//! Shortcut-resistant four-class temporal-order task.
//!
//! Each balanced group contains all four labels and shares the exact same
//! per-channel event counts. Labels change only the order of two motifs and
//! whether their lag is short or long. Consequently a classifier that sees
//! only channel totals receives byte-identical features for all four members
//! of every group.

use binn_core::Rng;

/// Fixed task input width.
pub const TEMPORAL_ORDER_N_IN: usize = 32;
/// Fixed number of timesteps.
pub const TEMPORAL_ORDER_T: usize = 32;
/// Fixed number of classes.
pub const TEMPORAL_ORDER_N_CLASSES: usize = 4;
/// Chance accuracy.
pub const TEMPORAL_ORDER_CHANCE: f32 = 0.25;
/// Number of fixed-total marker events added to rate-accessible examples.
pub const RATE_ACCESSIBLE_MARKER_EVENTS: usize = 16;

const MOTIF_A: [usize; 4] = [0, 1, 2, 3];
const MOTIF_B: [usize; 4] = [4, 5, 6, 7];

/// Whether class identity is available from per-channel event counts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RateAccessibility {
    /// Add a fixed-total, class-specific channel-count marker.
    Accessible,
    /// Preserve the v144 byte-identical channel-count construction.
    Immune,
}

/// One dense temporal spike-count example.
#[derive(Clone, Debug, PartialEq)]
pub struct TemporalOrderExample {
    /// Row-major `[T × N_IN]` frames.
    pub frames: Vec<f32>,
    pub label: u32,
}

impl TemporalOrderExample {
    /// Per-channel rates. These are deliberately label-insufficient.
    pub fn rate_features(&self) -> [f32; TEMPORAL_ORDER_N_IN] {
        let mut out = [0.0f32; TEMPORAL_ORDER_N_IN];
        for frame in self.frames.chunks_exact(TEMPORAL_ORDER_N_IN) {
            for (dst, &value) in out.iter_mut().zip(frame) {
                *dst += value / TEMPORAL_ORDER_T as f32;
            }
        }
        out
    }
}

/// Frozen difficulty candidate.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TemporalDifficulty {
    pub jitter_radius: usize,
    pub distractor_events: usize,
}

impl TemporalDifficulty {
    pub const fn new(jitter_radius: usize, distractor_events: usize) -> Self {
        Self {
            jitter_radius,
            distractor_events,
        }
    }
}

/// The only calibration candidates allowed by protocol.
pub const TEMPORAL_DIFFICULTIES: [TemporalDifficulty; 4] = [
    TemporalDifficulty::new(0, 4),
    TemporalDifficulty::new(1, 8),
    TemporalDifficulty::new(2, 12),
    TemporalDifficulty::new(3, 16),
];

/// A deterministic split generated from one seed family.
#[derive(Clone, Debug, PartialEq)]
pub struct TemporalOrderSplit {
    pub train: Vec<TemporalOrderExample>,
    pub test: Vec<TemporalOrderExample>,
    pub difficulty: TemporalDifficulty,
    pub seed: u64,
}

impl TemporalOrderSplit {
    /// Generate balanced train/test splits. Sizes must be multiples of four so
    /// each quartet supplies one example of every class.
    pub fn generate(
        n_train: usize,
        n_test: usize,
        difficulty: TemporalDifficulty,
        seed: u64,
    ) -> Result<Self, String> {
        validate_size(n_train)?;
        validate_size(n_test)?;
        if difficulty.jitter_radius > 3 {
            return Err("temporal-order jitter radius must be <= 3".into());
        }
        if difficulty.distractor_events > 16 {
            return Err("temporal-order distractors must be <= 16".into());
        }
        Ok(Self {
            train: generate_examples(n_train, difficulty, seed ^ 0x7A11_0000_0000_0001),
            test: generate_examples(n_test, difficulty, seed ^ 0x7E57_0000_0000_0001),
            difficulty,
            seed,
        })
    }

    /// Generate one side of the paired shortcut-accessibility contrast.
    ///
    /// Both variants begin from the exact v144 task. The accessible variant
    /// adds the same total number of events to every example, on the channel
    /// indexed by its class. Event times are identical across classes, so the
    /// intended intervention is the per-channel count vector.
    pub fn generate_with_rate_accessibility(
        n_train: usize,
        n_test: usize,
        difficulty: TemporalDifficulty,
        seed: u64,
        accessibility: RateAccessibility,
    ) -> Result<Self, String> {
        let mut split = Self::generate(n_train, n_test, difficulty, seed)?;
        if accessibility == RateAccessibility::Accessible {
            for example in split.train.iter_mut().chain(&mut split.test) {
                add_rate_marker(example);
            }
        }
        Ok(split)
    }

    /// Stable FNV-1a fingerprint of public config plus all examples.
    pub fn fingerprint(&self) -> u64 {
        let mut hash = 0xcbf2_9ce4_8422_2325u64;
        mix(&mut hash, self.seed);
        mix(&mut hash, self.difficulty.jitter_radius as u64);
        mix(&mut hash, self.difficulty.distractor_events as u64);
        for ex in self.train.iter().chain(&self.test) {
            mix(&mut hash, ex.label as u64);
            for &value in &ex.frames {
                mix(&mut hash, value.to_bits() as u64);
            }
        }
        hash
    }
}

/// Return a deterministic time-shuffled copy. The same permutation is applied
/// to every quartet member, preserving paired nuisance structure while
/// destroying absolute motif order.
pub fn time_shuffle(examples: &[TemporalOrderExample], seed: u64) -> Vec<TemporalOrderExample> {
    assert_eq!(examples.len() % TEMPORAL_ORDER_N_CLASSES, 0);
    let mut out = examples.to_vec();
    let mut rng = Rng::new(seed ^ 0x71AE_5A11_0000_0001);
    for quartet in out.chunks_exact_mut(TEMPORAL_ORDER_N_CLASSES) {
        let mut permutation: Vec<usize> = (0..TEMPORAL_ORDER_T).collect();
        for i in (1..permutation.len()).rev() {
            let j = rng.gen_index(i + 1);
            permutation.swap(i, j);
        }
        for ex in quartet {
            let original = ex.frames.clone();
            for (dst_t, &src_t) in permutation.iter().enumerate() {
                let dst = dst_t * TEMPORAL_ORDER_N_IN;
                let src = src_t * TEMPORAL_ORDER_N_IN;
                ex.frames[dst..dst + TEMPORAL_ORDER_N_IN]
                    .copy_from_slice(&original[src..src + TEMPORAL_ORDER_N_IN]);
            }
        }
    }
    out
}

fn validate_size(n: usize) -> Result<(), String> {
    if n == 0 || !n.is_multiple_of(TEMPORAL_ORDER_N_CLASSES) {
        return Err(format!(
            "temporal-order split size must be a positive multiple of {TEMPORAL_ORDER_N_CLASSES}"
        ));
    }
    Ok(())
}

fn generate_examples(
    n: usize,
    difficulty: TemporalDifficulty,
    seed: u64,
) -> Vec<TemporalOrderExample> {
    let mut rng = Rng::new(seed);
    let mut out = Vec::with_capacity(n);
    for _group in 0..n / TEMPORAL_ORDER_N_CLASSES {
        // Nuisance variables are drawn once per quartet, so every label gets
        // identical channel counts and a matched noise realization.
        let common_shift = signed_jitter(&mut rng, difficulty.jitter_radius);
        let motif_a_jitter = signed_jitter(&mut rng, difficulty.jitter_radius);
        let motif_b_jitter = signed_jitter(&mut rng, difficulty.jitter_radius);
        let distractors: Vec<(usize, usize)> = (0..difficulty.distractor_events)
            .map(|_| {
                let time = rng.gen_index(TEMPORAL_ORDER_T);
                let channel = 8 + rng.gen_index(TEMPORAL_ORDER_N_IN - 8);
                (time, channel)
            })
            .collect();

        for label in 0..TEMPORAL_ORDER_N_CLASSES {
            let reverse = label >= 2;
            let lag = if label % 2 == 0 { 6isize } else { 14isize };
            let first = clamp_time(7 + common_shift);
            let second = clamp_time(first as isize + lag);
            let (a_time, b_time) = if reverse {
                (
                    clamp_time(second as isize + motif_a_jitter),
                    clamp_time(first as isize + motif_b_jitter),
                )
            } else {
                (
                    clamp_time(first as isize + motif_a_jitter),
                    clamp_time(second as isize + motif_b_jitter),
                )
            };
            let mut frames = vec![0.0f32; TEMPORAL_ORDER_T * TEMPORAL_ORDER_N_IN];
            add_motif(&mut frames, a_time, &MOTIF_A);
            add_motif(&mut frames, b_time, &MOTIF_B);
            for &(time, channel) in &distractors {
                frames[time * TEMPORAL_ORDER_N_IN + channel] += 1.0;
            }
            out.push(TemporalOrderExample {
                frames,
                label: label as u32,
            });
        }
    }
    out
}

fn add_motif(frames: &mut [f32], center: usize, channels: &[usize]) {
    for (offset, &channel) in channels.iter().enumerate() {
        let time = (center + offset).min(TEMPORAL_ORDER_T - 1);
        frames[time * TEMPORAL_ORDER_N_IN + channel] += 1.0;
    }
}

fn add_rate_marker(example: &mut TemporalOrderExample) {
    let channel = example.label as usize;
    assert!(channel < TEMPORAL_ORDER_N_CLASSES);
    for event in 0..RATE_ACCESSIBLE_MARKER_EVENTS {
        let time = event * TEMPORAL_ORDER_T / RATE_ACCESSIBLE_MARKER_EVENTS;
        example.frames[time * TEMPORAL_ORDER_N_IN + channel] += 1.0;
    }
}

fn signed_jitter(rng: &mut Rng, radius: usize) -> isize {
    if radius == 0 {
        0
    } else {
        rng.gen_index(radius * 2 + 1) as isize - radius as isize
    }
}

fn clamp_time(time: isize) -> usize {
    time.clamp(0, (TEMPORAL_ORDER_T - 4) as isize) as usize
}

fn mix(hash: &mut u64, word: u64) {
    *hash ^= word;
    *hash = hash.wrapping_mul(0x100_0000_01b3);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_quartet_has_byte_identical_rate_features() {
        for difficulty in TEMPORAL_DIFFICULTIES {
            let split = TemporalOrderSplit::generate(40, 20, difficulty, 7).unwrap();
            for quartet in split.train.chunks_exact(4) {
                let expected = quartet[0].rate_features();
                for ex in &quartet[1..] {
                    assert_eq!(ex.rate_features(), expected);
                }
                assert_eq!(
                    quartet.iter().map(|ex| ex.label).collect::<Vec<_>>(),
                    vec![0, 1, 2, 3]
                );
            }
        }
    }

    #[test]
    fn splits_and_shuffle_replay_byte_identically() {
        let difficulty = TEMPORAL_DIFFICULTIES[2];
        let a = TemporalOrderSplit::generate(40, 20, difficulty, 91).unwrap();
        let b = TemporalOrderSplit::generate(40, 20, difficulty, 91).unwrap();
        assert_eq!(a, b);
        assert_eq!(a.fingerprint(), b.fingerprint());
        assert_eq!(time_shuffle(&a.test, 12), time_shuffle(&a.test, 12));
    }

    #[test]
    fn labels_are_balanced() {
        let split = TemporalOrderSplit::generate(100, 20, TEMPORAL_DIFFICULTIES[3], 42).unwrap();
        let mut counts = [0usize; 4];
        for ex in split.train {
            counts[ex.label as usize] += 1;
        }
        assert_eq!(counts, [25, 25, 25, 25]);
    }

    #[test]
    fn immune_variant_is_exactly_the_v144_task() {
        let difficulty = TEMPORAL_DIFFICULTIES[0];
        let original = TemporalOrderSplit::generate(40, 20, difficulty, 17).unwrap();
        let immune = TemporalOrderSplit::generate_with_rate_accessibility(
            40,
            20,
            difficulty,
            17,
            RateAccessibility::Immune,
        )
        .unwrap();
        assert_eq!(original, immune);
        assert_eq!(original.fingerprint(), immune.fingerprint());
    }

    #[test]
    fn accessible_variant_changes_only_the_fixed_total_class_marker() {
        let difficulty = TEMPORAL_DIFFICULTIES[0];
        let immune = TemporalOrderSplit::generate_with_rate_accessibility(
            40,
            20,
            difficulty,
            19,
            RateAccessibility::Immune,
        )
        .unwrap();
        let accessible = TemporalOrderSplit::generate_with_rate_accessibility(
            40,
            20,
            difficulty,
            19,
            RateAccessibility::Accessible,
        )
        .unwrap();
        for (base, marked) in immune.train.iter().zip(&accessible.train) {
            assert_eq!(base.label, marked.label);
            let mut recovered = marked.clone();
            for event in 0..RATE_ACCESSIBLE_MARKER_EVENTS {
                let time = event * TEMPORAL_ORDER_T / RATE_ACCESSIBLE_MARKER_EVENTS;
                recovered.frames[time * TEMPORAL_ORDER_N_IN + base.label as usize] -= 1.0;
            }
            assert_eq!(&recovered, base);
            let base_total: f32 = base.frames.iter().sum();
            let marked_total: f32 = marked.frames.iter().sum();
            assert_eq!(
                marked_total - base_total,
                RATE_ACCESSIBLE_MARKER_EVENTS as f32
            );
        }
        for quartet in accessible.train.chunks_exact(TEMPORAL_ORDER_N_CLASSES) {
            let features: Vec<_> = quartet
                .iter()
                .map(TemporalOrderExample::rate_features)
                .collect();
            for left in 0..features.len() {
                for right in left + 1..features.len() {
                    assert_ne!(features[left], features[right]);
                }
            }
        }
    }
}
