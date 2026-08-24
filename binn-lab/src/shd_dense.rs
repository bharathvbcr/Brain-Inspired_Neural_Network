//! SHD event cache → the dense temporal form the shared-forward stack takes.
//!
//! # What this adds, and what it deliberately does not re-implement
//!
//! The canonical SHD reader is [`binn_data::read_event_cache`] and the canonical
//! framing is [`binn_data::frame_events`]. Both are on the `shd-instrument`
//! bit-identity path that `scripts/gate_f_rust.py` regresses, so they are
//! **called** here and never copied: every contract (`published-2ms/4ms/10ms`,
//! `fixed-t100/250/500`) and every geometry (`channels-700`, `adjacent-sum-5`)
//! is decided by that one owner.
//!
//! What was missing is the last hop. `frame_events` yields a sparse
//! [`FramedShdSample`], while [`binn_learn::SharedTemporalNet::forward`] indexes
//! a flat `timesteps * n_in` buffer. [`crate::samples_to_dense_temporal_examples`]
//! performs exactly that hop for the synthetic `CoincidenceTask` trials; this is
//! its SHD sibling and the two agree on layout, on the label type, and on the
//! padding rule.
//!
//! Frames are the flat `timesteps x n_in` layout that `SharedTemporalNet::forward`
//! indexes as `frames[t * n_in + channel]`; the value is the **event count** in
//! that `(frame, channel)` cell, unscaled, exactly as the matched instrument
//! feeds it — `shd_matched::loss_and_gradient` multiplies `w_in` by the raw
//! count. The label is the SHD class id, kept as a class index in `0..20`
//! because the shared stack is multi-class.
//!
//! Sequences shorter than `timesteps` are zero-padded and longer ones truncated,
//! matching [`crate::samples_to_dense_temporal_examples`] so the two forms of the
//! same trial contain the same data. Under the `fixed-t*` contracts
//! `frame_events` already emits exactly that many steps and neither branch
//! fires; the `published-*ms` contracts have per-utterance lengths and do.

use std::path::Path;

use binn_data::{
    frame_events, read_event_cache, FramedShdSample, FrequencyGeometry, ShdEventContract, ShdSample,
};
use binn_learn::{DenseTemporalExample, ShdExample, MATCHED_PHYSICAL_TAU_MS};

/// One cached SHD sample in the form the SHD learning stack takes.
///
/// A field rename only: [`ShdSample`] and [`ShdExample`] carry the same flat
/// `t * n_in` frame buffer, and the per-sample `t` / `n_in` are copied from the
/// sample rather than re-derived, so a split framed under any contract converts
/// without the caller restating its geometry.
pub fn shd_sample_to_example(sample: &ShdSample) -> ShdExample {
    ShdExample {
        frames: sample.frames.clone(),
        t: sample.t,
        n_in: sample.n_in,
        label: sample.label,
    }
}

/// Membrane decay for one framing contract, `exp(-dt_ms / tau)`.
///
/// This is the same rule `binn_learn::shd_matched::loss_and_gradient` applies to
/// every recorded instrument cell, against the same
/// [`MATCHED_PHYSICAL_TAU_MS`]. That function computes it inline and is on the
/// bit-identity path, so it is not refactored to call this; the shared constant
/// is what keeps the two from drifting.
pub fn contract_alpha(contract: ShdEventContract) -> f32 {
    (-contract.dt_ms() / MATCHED_PHYSICAL_TAU_MS).exp()
}

/// Fixed step count for a contract that has one, `None` for the
/// `published-*ms` contracts whose length is per-utterance.
///
/// The shared-forward stack is built for a single `timesteps`, so a contract
/// that returns `None` here needs a caller-registered padding length.
pub const fn contract_timesteps(contract: ShdEventContract) -> Option<usize> {
    match contract {
        ShdEventContract::FixedWindow { frames, .. } => Some(frames),
        ShdEventContract::PublishedDuration { .. } => None,
    }
}

/// One framed SHD utterance in the dense form the shared stack takes.
pub fn framed_to_dense_temporal_example(
    framed: &FramedShdSample,
    timesteps: usize,
) -> DenseTemporalExample {
    assert!(timesteps > 0, "dense temporal examples need a step count");
    assert!(
        framed.n_inputs > 0,
        "dense temporal examples need at least one channel"
    );
    let n_in = framed.n_inputs;
    let mut frames = vec![0.0f32; timesteps * n_in];
    for (t, frame) in framed.frames.iter().enumerate().take(timesteps) {
        for &(channel, count) in &frame.values {
            assert!(
                channel < n_in,
                "framed sample reports channel {channel} outside its own {n_in} inputs"
            );
            frames[t * n_in + channel] = count;
        }
    }
    DenseTemporalExample {
        frames,
        timesteps,
        n_in,
        label: framed.label,
    }
}

/// Read an `SHDEVT1` event cache and return it in dense temporal form.
///
/// `max_samples` takes a prefix of the cache, which is how
/// `shd-instrument train-cell --max-train/--max-test` caps a cell; the cache
/// written by `scripts/shd_calibration/data.py` is already shuffled, so a prefix
/// carries every class. That is not assumed — [`class_histogram`] is reported by
/// the caller so an unbalanced prefix is visible rather than silent.
pub fn load_shd_dense_examples(
    events: &Path,
    contract: ShdEventContract,
    geometry: FrequencyGeometry,
    timesteps: usize,
    max_samples: Option<usize>,
) -> Result<Vec<DenseTemporalExample>, String> {
    let raw = read_event_cache(events, max_samples)?;
    if raw.is_empty() {
        return Err(format!("empty SHD event cache: {}", events.display()));
    }
    Ok(raw
        .iter()
        .map(|sample| {
            framed_to_dense_temporal_example(&frame_events(sample, contract, geometry), timesteps)
        })
        .collect())
}

/// Ground-truth class counts, one entry per class.
///
/// # Panics
///
/// Panics on a label outside `0..n_classes`, which would silently shrink the
/// realised chance rate if it were tolerated.
pub fn class_histogram(examples: &[DenseTemporalExample], n_classes: usize) -> Vec<usize> {
    assert!(n_classes > 0, "a class histogram needs at least one class");
    let mut histogram = vec![0usize; n_classes];
    for example in examples {
        let label = example.label as usize;
        assert!(
            label < n_classes,
            "example carries label {label} outside the registered {n_classes} classes"
        );
        histogram[label] += 1;
    }
    histogram
}

/// Accuracy of the best constant predictor on this split.
///
/// This, not `1 / n_classes`, is what a ceiling has to clear before anything is
/// measurable against it when the split is not exactly balanced — see
/// [`crate::guards::CeilingHealth::evaluate`], whose `chance` argument is
/// documented as the realised majority-class rate for that case.
pub fn majority_class_rate(examples: &[DenseTemporalExample], n_classes: usize) -> f32 {
    assert!(
        !examples.is_empty(),
        "a majority-class rate needs at least one example"
    );
    let histogram = class_histogram(examples, n_classes);
    let top = histogram.into_iter().max().unwrap_or(0);
    top as f32 / examples.len() as f32
}

#[cfg(test)]
mod tests {
    use super::*;
    use binn_data::{ShdEvent, ShdEventSample, SparseFrame};

    fn framed(frames: Vec<Vec<(usize, f32)>>, n_inputs: usize, label: u32) -> FramedShdSample {
        FramedShdSample {
            label,
            frames: frames
                .into_iter()
                .map(|values| SparseFrame { values })
                .collect(),
            n_inputs,
            dt_ms: 14.0,
            original_events: 0,
            retained_events: 0,
            clipped_events: 0,
            first_time_s: 0.0,
            last_time_s: 0.0,
        }
    }

    #[test]
    fn dense_layout_is_row_major_over_timesteps() {
        let sample = framed(vec![vec![(0, 2.0)], vec![(2, 1.0), (3, 5.0)]], 4, 7);
        let dense = framed_to_dense_temporal_example(&sample, 2);
        assert_eq!(dense.timesteps, 2);
        assert_eq!(dense.n_in, 4);
        assert_eq!(dense.label, 7);
        // `SharedTemporalNet::forward` reads `frames[t * n_in + channel]`.
        assert_eq!(dense.frames, vec![2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 5.0]);
    }

    #[test]
    fn short_sequences_are_zero_padded_and_long_ones_truncated() {
        let short = framed(vec![vec![(1, 3.0)]], 2, 0);
        let padded = framed_to_dense_temporal_example(&short, 3);
        assert_eq!(padded.frames, vec![0.0, 3.0, 0.0, 0.0, 0.0, 0.0]);

        let long = framed(vec![vec![(0, 1.0)], vec![(0, 2.0)], vec![(0, 9.0)]], 1, 0);
        let cut = framed_to_dense_temporal_example(&long, 2);
        assert_eq!(cut.frames, vec![1.0, 2.0]);
    }

    #[test]
    fn counts_are_carried_through_unscaled() {
        // A collision in one framing cell must reach the model as `3`, because
        // that is what `shd_matched::loss_and_gradient` multiplies `w_in` by.
        let sample = framed(vec![vec![(0, 3.0)]], 1, 1);
        let dense = framed_to_dense_temporal_example(&sample, 1);
        assert_eq!(dense.frames, vec![3.0]);
    }

    #[test]
    fn framing_the_canonical_way_agrees_with_this_converter() {
        let events = ShdEventSample {
            label: 3,
            events: vec![
                ShdEvent {
                    time_s: 0.0,
                    channel: 0,
                },
                ShdEvent {
                    time_s: 0.0,
                    channel: 1,
                },
                ShdEvent {
                    time_s: 0.9,
                    channel: 699,
                },
            ],
        };
        let contract = ShdEventContract::fixed(100).expect("fixed-t100 is a registered contract");
        let geometry = FrequencyGeometry::AdjacentSum5;
        let sample = frame_events(&events, contract, geometry);
        let dense = framed_to_dense_temporal_example(&sample, 100);
        assert_eq!(dense.n_in, geometry.n_inputs());
        assert_eq!(dense.timesteps, 100);
        // Channels 0 and 1 both fold onto input 0 under adjacent-sum-5, so the
        // first cell must carry 2, not 1: the collision is preserved.
        assert_eq!(dense.frames[0], 2.0);
        // 0.9 s into a 1.4 s window split into 100 frames is frame 64;
        // channel 699 folds onto input 139.
        assert_eq!(dense.frames[64 * 140 + 139], 1.0);
        assert_eq!(dense.frames.iter().sum::<f32>(), 3.0);
    }

    #[test]
    fn alpha_follows_the_contract_step_and_the_matched_tau() {
        let t100 = ShdEventContract::fixed(100).expect("registered");
        assert!((t100.dt_ms() - 14.0).abs() < 1e-6);
        let expected = (-14.0f32 / MATCHED_PHYSICAL_TAU_MS).exp();
        assert!((contract_alpha(t100) - expected).abs() < 1e-7);
        // Shorter frames leak less between steps.
        let published = ShdEventContract::published(2).expect("registered");
        assert!(contract_alpha(published) > contract_alpha(t100));
    }

    #[test]
    fn only_the_fixed_contracts_carry_their_own_step_count() {
        assert_eq!(
            contract_timesteps(ShdEventContract::fixed(250).expect("registered")),
            Some(250)
        );
        assert_eq!(
            contract_timesteps(ShdEventContract::published(10).expect("registered")),
            None
        );
    }

    #[test]
    fn majority_class_rate_is_the_best_constant_predictor() {
        let examples: Vec<DenseTemporalExample> = [0u32, 1, 1, 1, 2]
            .iter()
            .map(|&label| DenseTemporalExample {
                frames: vec![0.0],
                timesteps: 1,
                n_in: 1,
                label,
            })
            .collect();
        assert_eq!(class_histogram(&examples, 3), vec![1, 3, 1]);
        assert!((majority_class_rate(&examples, 3) - 0.6).abs() < 1e-6);
    }

    #[test]
    #[should_panic(expected = "outside the registered")]
    fn a_label_outside_the_class_range_is_fatal() {
        let examples = vec![DenseTemporalExample {
            frames: vec![0.0],
            timesteps: 1,
            n_in: 1,
            label: 20,
        }];
        let _ = class_histogram(&examples, 20);
    }
}
