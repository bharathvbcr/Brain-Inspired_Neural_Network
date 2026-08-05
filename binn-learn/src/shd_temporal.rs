//! Temporal-information manipulations for `PREREG_2026-08-02_SHD_TEMPORAL_INFORMATION`.
//!
//! All four conditions preserve **per-channel total spike counts exactly** and
//! differ only in what temporal structure survives:
//!
//! | Condition | Destroys | Preserves |
//! |---|---|---|
//! | `intact` | - | - |
//! | `bin-shuffled` | temporal order | per-channel counts, within-bin synchrony |
//! | `channel-shuffled` | order **and** cross-channel synchrony | per-channel counts |
//! | `reversed` | direction | order magnitude, synchrony, counts |
//!
//! The `bin-shuffled` / `channel-shuffled` contrast is what separates *order*
//! from *synchrony*; most published shuffle controls conflate the two.
//!
//! # Manipulation check is code, not hope
//!
//! Prereg §5.1 makes count preservation a blocking gate: if a manipulation
//! changed per-channel totals it would be altering *rate*, and the whole
//! experiment would be measuring the wrong thing. [`apply_temporal`] therefore
//! recomputes per-channel totals before and after and refuses to return a
//! sample that fails. [`TemporalAudit`] carries the evidence into the cell
//! record so a passing run can be checked after the fact.

use crate::shd_matched::{MatchedShdSample, PortableRng};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TemporalCondition {
    Intact,
    BinShuffled,
    ChannelShuffled,
    Reversed,
}

impl TemporalCondition {
    pub const ALL: [Self; 4] = [
        Self::Intact,
        Self::BinShuffled,
        Self::ChannelShuffled,
        Self::Reversed,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Intact => "intact",
            Self::BinShuffled => "bin-shuffled",
            Self::ChannelShuffled => "channel-shuffled",
            Self::Reversed => "reversed",
        }
    }

    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "intact" => Ok(Self::Intact),
            "bin-shuffled" => Ok(Self::BinShuffled),
            "channel-shuffled" => Ok(Self::ChannelShuffled),
            "reversed" => Ok(Self::Reversed),
            other => Err(format!(
                "unknown temporal condition {other:?}; expected intact, bin-shuffled, \
                 channel-shuffled or reversed"
            )),
        }
    }

    pub const fn is_identity(self) -> bool {
        matches!(self, Self::Intact)
    }
}

/// Evidence that the manipulation did what it claims, emitted per cell.
#[derive(Clone, Debug, PartialEq)]
pub struct TemporalAudit {
    pub samples: usize,
    /// Prereg §5.1. False here voids the run.
    pub counts_preserved: bool,
    /// Samples whose per-channel totals changed. Must be 0.
    pub count_mismatches: usize,
    /// Mean |new_bin - old_bin| over relocated entries, in bins.
    ///
    /// A manipulation that reports `counts_preserved` but near-zero
    /// displacement has silently done nothing - the failure mode that would
    /// make a null result meaningless.
    pub mean_bin_displacement: f64,
    /// Fraction of (bin, channel) entries that changed bin. ~0 is a red flag
    /// for every condition except `intact`.
    pub relocated_fraction: f64,
    /// Mean occupied bins per sample, before and after. Equal for bin-shuffled
    /// and reversed; may fall for channel-shuffled as entries collide into
    /// fewer distinct bins.
    pub occupied_bins_before: f64,
    pub occupied_bins_after: f64,
}

impl Default for TemporalAudit {
    /// `counts_preserved` defaults to **true**, not `false`.
    ///
    /// [`TemporalAudit::merge`] folds it with `&=`, so the default value is the
    /// identity element of a conjunction over zero samples - vacuously true,
    /// nothing has violated it yet. Deriving `Default` gives `false`, which is
    /// absorbing rather than identity: every fold starting from a derived
    /// default would report `counts_preserved: false` no matter what the samples
    /// actually did, writing a false gate violation into every cell record.
    fn default() -> Self {
        Self {
            samples: 0,
            counts_preserved: true,
            count_mismatches: 0,
            mean_bin_displacement: 0.0,
            relocated_fraction: 0.0,
            occupied_bins_before: 0.0,
            occupied_bins_after: 0.0,
        }
    }
}

/// Per-channel totals, summed in bin order.
///
/// # Why the bit-comparison against these is safe
///
/// `apply_temporal` compares before/after totals with `to_bits() == to_bits()`,
/// and shuffling changes the order the addends arrive in. Float addition is not
/// associative, so in general that comparison could fail on a manipulation that
/// preserved every count — a spurious hard failure voiding a valid run, on the
/// prereg's blocking gate 5.1.
///
/// It is safe here because framed counts are **integer-valued**: `frame_events`
/// counts events per (bin, channel), and `adjacent-sum-5` sums integers. f32
/// represents every integer below `2^24` exactly and their sums are therefore
/// order-independent, and per-channel totals over an SHD sample are in the
/// thousands at most.
///
/// This is an assumption, not an invariant the type system carries. If counts
/// ever become non-integral — normalisation, weighting, a float-valued
/// geometry — the bit-comparison must become an exact-sum or tolerance
/// comparison at the same time. `integer_counts_survive_reordering` pins it.
fn channel_totals(sample: &MatchedShdSample) -> Vec<f32> {
    let mut totals = vec![0.0_f32; sample.n_inputs];
    for frame in &sample.frames {
        for &(channel, count) in frame {
            totals[channel] += count;
        }
    }
    totals
}

fn occupied(sample: &MatchedShdSample) -> usize {
    sample.frames.iter().filter(|frame| !frame.is_empty()).count()
}

/// Rebuild `frames` from `(bin, channel, count)` triples, restoring the framing
/// invariant that each bin is sorted ascending by channel and holds one entry
/// per channel.
fn rebuild(steps: usize, entries: Vec<(usize, usize, f32)>) -> Vec<Vec<(usize, f32)>> {
    let mut frames: Vec<Vec<(usize, f32)>> = vec![Vec::new(); steps];
    for (bin, channel, count) in entries {
        frames[bin].push((channel, count));
    }
    for frame in frames.iter_mut() {
        frame.sort_by_key(|&(channel, _)| channel);
        // Independent per-channel permutation can land two entries for the same
        // channel in one bin; merging keeps counts exact and the invariant true.
        let mut merged: Vec<(usize, f32)> = Vec::with_capacity(frame.len());
        for &(channel, count) in frame.iter() {
            match merged.last_mut() {
                Some((last_channel, last_count)) if *last_channel == channel => {
                    *last_count += count;
                }
                _ => merged.push((channel, count)),
            }
        }
        *frame = merged;
    }
    frames
}

/// Apply `condition` in place. `seed` must be derived from the cell seed so the
/// manipulation is reproducible and identical across backends.
pub fn apply_temporal(
    sample: &mut MatchedShdSample,
    condition: TemporalCondition,
    seed: u64,
) -> Result<TemporalAudit, String> {
    let steps = sample.frames.len();
    let before_totals = channel_totals(sample);
    let before_occupied = occupied(sample);

    let mut displacement_sum = 0.0_f64;
    let mut relocated = 0_usize;
    let mut entries_total = 0_usize;

    match condition {
        TemporalCondition::Intact => {}
        TemporalCondition::Reversed => {
            sample.frames.reverse();
            for (new_bin, frame) in sample.frames.iter().enumerate() {
                let old_bin = steps - 1 - new_bin;
                for _ in frame {
                    entries_total += 1;
                    if new_bin != old_bin {
                        relocated += 1;
                        displacement_sum += (new_bin as f64 - old_bin as f64).abs();
                    }
                }
            }
        }
        TemporalCondition::BinShuffled => {
            // One permutation shared by every channel: order dies, within-bin
            // synchrony survives.
            let mut rng = PortableRng::new(seed);
            let mut permutation: Vec<usize> = (0..steps).collect();
            rng.shuffle(&mut permutation);
            let mut entries = Vec::new();
            for (old_bin, frame) in sample.frames.iter().enumerate() {
                let new_bin = permutation[old_bin];
                for &(channel, count) in frame {
                    entries_total += 1;
                    if new_bin != old_bin {
                        relocated += 1;
                        displacement_sum += (new_bin as f64 - old_bin as f64).abs();
                    }
                    entries.push((new_bin, channel, count));
                }
            }
            sample.frames = rebuild(steps, entries);
        }
        TemporalCondition::ChannelShuffled => {
            // One permutation per channel: order and cross-channel synchrony
            // both die. Seeds are offset by channel so no two channels share a
            // permutation.
            let mut permutations: Vec<Vec<usize>> = Vec::with_capacity(sample.n_inputs);
            for channel in 0..sample.n_inputs {
                let mut rng = PortableRng::new(seed ^ ((channel as u64 + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15)));
                let mut permutation: Vec<usize> = (0..steps).collect();
                rng.shuffle(&mut permutation);
                permutations.push(permutation);
            }
            let mut entries = Vec::new();
            for (old_bin, frame) in sample.frames.iter().enumerate() {
                for &(channel, count) in frame {
                    let new_bin = permutations[channel][old_bin];
                    entries_total += 1;
                    if new_bin != old_bin {
                        relocated += 1;
                        displacement_sum += (new_bin as f64 - old_bin as f64).abs();
                    }
                    entries.push((new_bin, channel, count));
                }
            }
            sample.frames = rebuild(steps, entries);
        }
    }

    let after_totals = channel_totals(sample);
    let preserved = before_totals
        .iter()
        .zip(after_totals.iter())
        .all(|(before, after)| before.to_bits() == after.to_bits());
    if !preserved {
        return Err(format!(
            "temporal condition {} changed per-channel spike counts - prereg gate 5.1 voids this run",
            condition.label()
        ));
    }

    Ok(TemporalAudit {
        samples: 1,
        counts_preserved: true,
        count_mismatches: 0,
        mean_bin_displacement: if relocated == 0 {
            0.0
        } else {
            displacement_sum / relocated as f64
        },
        relocated_fraction: if entries_total == 0 {
            0.0
        } else {
            relocated as f64 / entries_total as f64
        },
        occupied_bins_before: before_occupied as f64,
        occupied_bins_after: occupied(sample) as f64,
    })
}

impl TemporalAudit {
    /// Fold a per-sample audit into a running dataset-level audit.
    pub fn merge(&mut self, other: &TemporalAudit) {
        let total = self.samples + other.samples;
        let weight = |value_a: f64, count_a: usize, value_b: f64, count_b: usize| {
            if count_a + count_b == 0 {
                0.0
            } else {
                (value_a * count_a as f64 + value_b * count_b as f64) / (count_a + count_b) as f64
            }
        };
        self.mean_bin_displacement = weight(
            self.mean_bin_displacement,
            self.samples,
            other.mean_bin_displacement,
            other.samples,
        );
        self.relocated_fraction = weight(
            self.relocated_fraction,
            self.samples,
            other.relocated_fraction,
            other.samples,
        );
        self.occupied_bins_before = weight(
            self.occupied_bins_before,
            self.samples,
            other.occupied_bins_before,
            other.samples,
        );
        self.occupied_bins_after = weight(
            self.occupied_bins_after,
            self.samples,
            other.occupied_bins_after,
            other.samples,
        );
        self.counts_preserved &= other.counts_preserved;
        self.count_mismatches += other.count_mismatches;
        self.samples = total;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Gate 5.1 compares per-channel totals bit-exactly, and shuffling changes
    /// the order they are summed in. That is only sound because counts are
    /// integer-valued — see [`channel_totals`]. This pins both halves: large
    /// integer counts survive reordering exactly, and non-integral ones do not.
    ///
    /// The second assertion is the point. It documents that the current gate
    /// would start failing spuriously if counts ever stopped being integers,
    /// so the failure arrives here rather than as a voided campaign run.
    #[test]
    fn integer_counts_survive_reordering() {
        let integral: Vec<f32> = (1..=64).map(|k| (k * 7 % 23) as f32).collect();
        let forward: f32 = integral.iter().sum();
        let backward: f32 = integral.iter().rev().sum();
        assert_eq!(
            forward.to_bits(),
            backward.to_bits(),
            "integer-valued counts must sum order-independently in f32",
        );

        // The bound is real, and this is where it bites: 2^24 is the largest
        // integer f32 represents exactly, so above it addition stops being
        // order-independent. Summing high-to-low loses both ones; low-to-high
        // keeps them.
        //
        // An earlier version of this test tried to demonstrate the same point
        // with fractional counts `k * 0.1` and asserted the two orders differ.
        // They did not — that particular sequence reassociates exactly — which
        // is a good reminder that "floats are inexact" is not a property you
        // can assert with an arbitrary example. The 2^24 boundary is the
        // documented claim, so it is the one worth pinning.
        let past_the_bound = [16_777_216.0_f32, 1.0, 1.0];
        let high_first: f32 = past_the_bound.iter().sum();
        let low_first: f32 = past_the_bound.iter().rev().sum();
        assert_ne!(
            high_first.to_bits(),
            low_first.to_bits(),
            "2^24 is the stated bound on order-independence; if this stops \
             holding the comment on channel_totals is wrong",
        );
        assert_eq!(low_first, 16_777_218.0, "low-to-high keeps both increments");
    }

    fn sample() -> MatchedShdSample {
        let mut frames = Vec::new();
        for t in 0..40 {
            let mut frame = Vec::new();
            for k in 0..5 {
                frame.push((((t * 3 + k * 7) % 20) as usize, 1.0_f32));
            }
            frame.sort_by_key(|&(channel, _)| channel);
            frame.dedup_by_key(|entry| entry.0);
            frames.push(frame);
        }
        MatchedShdSample { label: 3, frames, n_inputs: 20, dt_ms: 10.0 }
    }

    /// PREREG GATE 5.1. Every condition must preserve per-channel totals exactly.
    #[test]
    fn every_condition_preserves_channel_counts() {
        for condition in TemporalCondition::ALL {
            let original = sample();
            let before = channel_totals(&original);
            let mut manipulated = original.clone();
            apply_temporal(&mut manipulated, condition, 77).expect(condition.label());
            let after = channel_totals(&manipulated);
            assert_eq!(before, after, "condition {} changed counts", condition.label());
        }
    }

    /// A manipulation that silently does nothing would make a null meaningless.
    #[test]
    fn non_identity_conditions_actually_move_spikes() {
        for condition in [
            TemporalCondition::BinShuffled,
            TemporalCondition::ChannelShuffled,
            TemporalCondition::Reversed,
        ] {
            let mut manipulated = sample();
            let audit = apply_temporal(&mut manipulated, condition, 77).unwrap();
            assert!(
                audit.relocated_fraction > 0.5,
                "condition {} relocated only {:.3} of entries",
                condition.label(),
                audit.relocated_fraction
            );
            assert!(audit.mean_bin_displacement > 1.0, "{}", condition.label());
            assert_ne!(manipulated.frames, sample().frames, "{}", condition.label());
        }
    }

    #[test]
    fn intact_is_a_no_op() {
        let mut manipulated = sample();
        let audit = apply_temporal(&mut manipulated, TemporalCondition::Intact, 77).unwrap();
        assert_eq!(manipulated.frames, sample().frames);
        assert_eq!(audit.relocated_fraction, 0.0);
    }

    /// bin-shuffled keeps whole bins together; channel-shuffled does not.
    /// That difference is the order/synchrony decomposition the prereg rests on.
    #[test]
    fn bin_shuffle_preserves_within_bin_synchrony_channel_shuffle_does_not() {
        let mut binned = sample();
        apply_temporal(&mut binned, TemporalCondition::BinShuffled, 5).unwrap();
        let original_bins: Vec<Vec<(usize, f32)>> = sample().frames;
        for frame in &binned.frames {
            if frame.is_empty() {
                continue;
            }
            assert!(
                original_bins.contains(frame),
                "bin-shuffled must permute intact bins, found a bin that never existed"
            );
        }
        let mut channelled = sample();
        apply_temporal(&mut channelled, TemporalCondition::ChannelShuffled, 5).unwrap();
        let novel = channelled
            .frames
            .iter()
            .filter(|frame| !frame.is_empty() && !original_bins.contains(frame))
            .count();
        assert!(novel > 0, "channel-shuffled should create bins that never co-occurred");
    }

    #[test]
    fn reversed_is_its_own_inverse() {
        let mut once = sample();
        apply_temporal(&mut once, TemporalCondition::Reversed, 0).unwrap();
        apply_temporal(&mut once, TemporalCondition::Reversed, 0).unwrap();
        assert_eq!(once.frames, sample().frames);
    }

    /// Regression: a derived `Default` made `counts_preserved` absorbing under
    /// the `&=` fold, so every merged audit reported a gate violation that had
    /// not happened.
    #[test]
    fn default_audit_is_the_identity_of_the_merge_fold() {
        let empty = TemporalAudit::default();
        assert!(empty.counts_preserved, "default must be the conjunction identity");
        assert_eq!(empty.samples, 0);

        let mut folded = TemporalAudit::default();
        let mut manipulated = sample();
        let single = apply_temporal(&mut manipulated, TemporalCondition::BinShuffled, 1).unwrap();
        folded.merge(&single);
        assert_eq!(folded, single, "folding one audit into the default must be a no-op");
    }

    #[test]
    fn audit_merge_averages_by_sample_count() {
        let mut total = TemporalAudit::default();
        for seed in 0..4 {
            let mut manipulated = sample();
            let audit = apply_temporal(&mut manipulated, TemporalCondition::BinShuffled, seed).unwrap();
            total.merge(&audit);
        }
        assert_eq!(total.samples, 4);
        assert!(total.counts_preserved);
        assert!(total.mean_bin_displacement > 1.0);
    }
}
