//! Global-norm gradient clipping, with one owner for both call sites.
//!
//! # Why this is a module and not two inline blocks
//!
//! The instrument clips in two places — the batch gradient at the optimiser
//! step, and each sample gradient before accumulation. Written inline they
//! would be two implementations of the same rule, free to drift in the way
//! `validity_problems` drifted across three analysers. They share this instead.
//!
//! # Why per-sample clipping exists at all
//!
//! `AMENDMENT_2026-08-05_SURROGATE_GAIN_FOR_RECURRENT.md` §1 lists batch
//! gradient clipping among four interventions tried against the recurrent
//! failure, and records its outcome as **"never reached — abort fires on a
//! per-sample gradient, upstream"**. The recurrent explosion compounds inside
//! a single sample's backward pass, so by the time a batch gradient exists the
//! run has already returned an error. A threshold that only ever sees the batch
//! cannot bind on the cells it was added for. That finding was recorded and the
//! code was never changed to match it.
//!
//! # Policy is the caller's
//!
//! [`clip_by_global_norm`] reports what happened and never decides what it
//! means. The two sites want different things from
//! [`ClipOutcome::NormOverflowed`]: the batch site counts it and continues,
//! because the existing non-finite accounting reports it; the per-sample site
//! refuses, because scaling by `threshold / inf` is zero and would silently
//! delete a sample from its batch while leaving a cell that looks trained.

use binn_learn::ShdArmGradient;

/// What clipping did, so a caller can count it and a cell can report it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClipOutcome {
    /// The norm was at or below the threshold. The gradient is **untouched** —
    /// not multiplied by 1.0, which is observably different: `x * 1.0` turns
    /// `-0.0` into `-0.0` but is still an operation the bit-pins would see if
    /// it ever became conditional on something else.
    Untouched,
    /// The threshold bound and every entry was scaled by `threshold / norm`.
    Bound,
    /// Every entry is finite but their sum of squares is not representable.
    /// The gradient is left untouched: a ratio cannot bring an unrepresentable
    /// norm into range, so there is nothing correct to do here except say so.
    NormOverflowed,
}

/// Scale `gradient` in place so its global L2 norm is at most `threshold`.
///
/// The scale is computed in `f64`. At h512 the norms reaching the batch site
/// are ~1e29, and `threshold / norm` in `f32` flushes to zero for norms above
/// ~1e38 — which would zero the gradient exactly when it most needed bounding.
///
/// # Panics
///
/// Panics if `threshold` is not finite and positive. Both call sites validate
/// their flag at parse time, so reaching this is a programming error rather
/// than bad input, and a silently ignored threshold is the failure mode this
/// whole module exists to avoid.
pub fn clip_by_global_norm(gradient: &mut ShdArmGradient, threshold: f64) -> ClipOutcome {
    assert!(
        threshold.is_finite() && threshold > 0.0,
        "clip threshold must be finite and positive, got {threshold}"
    );
    let norm = f64::from(gradient.l2_norm());
    if !norm.is_finite() {
        return ClipOutcome::NormOverflowed;
    }
    if norm > threshold {
        gradient.scale((threshold / norm) as f32);
        ClipOutcome::Bound
    } else {
        ClipOutcome::Untouched
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use binn_learn::{MatchedArm, ShdArmWeights, ShdMatchedWeights};

    /// A gradient-shaped value with known contents.
    ///
    /// Built through `ShdArmGradient::zeros_like` so the shape always matches
    /// whatever the arm actually carries, then filled directly — a gradient
    /// from a real backward pass would work too, but its entries would not be
    /// under the test's control and the boundary cases below need exact norms.
    fn gradient(arm: MatchedArm, fill: impl Fn(usize) -> f32) -> ShdArmGradient {
        let base = ShdMatchedWeights::deterministic(6, 5, 4, 11);
        let w_rec = if arm.recurrent {
            vec![0.0_f32; 5 * 5]
        } else {
            Vec::new()
        };
        let weights = ShdArmWeights::new(base, arm, w_rec).unwrap();
        let mut gradient = ShdArmGradient::zeros_like(&weights);
        for (index, value) in gradient.base.w_in.iter_mut().enumerate() {
            *value = fill(index);
        }
        gradient
    }

    fn bits(gradient: &ShdArmGradient) -> Vec<u32> {
        gradient
            .base
            .w_in
            .iter()
            .chain(&gradient.base.w_out)
            .chain(&gradient.base.b_out)
            .chain(&gradient.w_rec)
            .map(|v| v.to_bits())
            .collect()
    }

    /// Below the threshold, clipping must be a no-op **bitwise**, not merely
    /// numerically. A `* 1.0` pass would be invisible to a tolerance check and
    /// visible to `every_arm_forward_and_backward_is_bit_pinned`.
    #[test]
    fn a_gradient_under_the_threshold_is_left_bit_identical() {
        let mut gradient = gradient(MatchedArm::FF_FIXED, |i| ((i % 7) as f32) * 1e-3);
        let before = bits(&gradient);
        let norm = gradient.l2_norm();
        assert!(
            norm > 0.0,
            "fixture has a zero gradient; the test is vacuous"
        );
        let outcome = clip_by_global_norm(&mut gradient, f64::from(norm) * 10.0);
        assert_eq!(outcome, ClipOutcome::Untouched);
        assert_eq!(before, bits(&gradient), "an untouched gradient moved");
    }

    /// The boundary is `>`, so a norm exactly at the threshold does not bind.
    /// Pinned because an off-by-one-comparison here would silently rescale
    /// every gradient in a run whose threshold was chosen to sit at the norm.
    #[test]
    fn a_norm_exactly_at_the_threshold_does_not_bind() {
        let mut gradient = gradient(MatchedArm::FF_FIXED, |i| ((i % 5) as f32) * 2e-2);
        let before = bits(&gradient);
        let norm = f64::from(gradient.l2_norm());
        assert_eq!(
            clip_by_global_norm(&mut gradient, norm),
            ClipOutcome::Untouched
        );
        assert_eq!(before, bits(&gradient));
    }

    /// Above the threshold every entry is scaled by the same factor, and the
    /// resulting norm lands at the threshold rather than merely below it.
    #[test]
    fn clipping_scales_uniformly_and_lands_on_the_threshold() {
        let mut gradient = gradient(MatchedArm::FF_FIXED, |i| 1.0 + (i % 11) as f32);
        let before: Vec<f32> = gradient.base.w_in.clone();
        let norm = f64::from(gradient.l2_norm());
        let threshold = norm / 8.0;
        assert_eq!(
            clip_by_global_norm(&mut gradient, threshold),
            ClipOutcome::Bound
        );

        let after_norm = f64::from(gradient.l2_norm());
        let relative = (after_norm - threshold).abs() / threshold;
        assert!(
            relative < 1e-5,
            "clipped norm {after_norm:e} is not the threshold {threshold:e} \
             (relative {relative:e})"
        );

        // Direction preserved: the ratio between any two entries is unchanged,
        // so clipping rescales the step without steering it.
        let expected = (threshold / norm) as f32;
        for (index, (old, new)) in before.iter().zip(&gradient.base.w_in).enumerate() {
            if *old == 0.0 {
                continue;
            }
            let ratio = new / old;
            assert!(
                (ratio - expected).abs() / expected < 1e-5,
                "entry {index} scaled by {ratio:e}, expected {expected:e}"
            );
        }
    }

    /// Every entry finite, norm not representable. Scaling by `threshold / inf`
    /// is zero, which would delete the gradient; the contract is to report and
    /// leave it alone so the caller can refuse.
    #[test]
    fn an_overflowing_norm_is_reported_and_changes_nothing() {
        let mut gradient = gradient(MatchedArm::FF_FIXED, |_| 1e38);
        assert!(
            gradient.base.w_in.iter().all(|v| v.is_finite()),
            "fixture entries must be finite for this case to be the one under test"
        );
        assert!(
            !gradient.l2_norm().is_finite(),
            "fixture norm is finite, so this test is not exercising the overflow path"
        );
        let before = bits(&gradient);
        assert_eq!(
            clip_by_global_norm(&mut gradient, 1.0),
            ClipOutcome::NormOverflowed
        );
        assert_eq!(
            before,
            bits(&gradient),
            "an unclippable gradient was modified anyway"
        );
    }

    /// Clipping must cover every parameter block, not just `base`. An attention
    /// or recurrent arm whose extra blocks escaped the scale would report a
    /// bounded norm while carrying an unbounded step.
    #[test]
    fn every_parameter_block_is_scaled() {
        for arm in [MatchedArm::REC_FIXED, MatchedArm::REC_ALIF] {
            let mut gradient = gradient(arm, |i| 1.0 + (i % 3) as f32);
            for (index, value) in gradient.w_rec.iter_mut().enumerate() {
                *value = 2.0 + (index % 4) as f32;
            }
            let norm = f64::from(gradient.l2_norm());
            let threshold = norm / 4.0;
            assert_eq!(
                clip_by_global_norm(&mut gradient, threshold),
                ClipOutcome::Bound
            );
            let after = f64::from(gradient.l2_norm());
            assert!(
                (after - threshold).abs() / threshold < 1e-5,
                "{}: recurrent block escaped the scale; norm {after:e} vs \
                 threshold {threshold:e}",
                arm.label()
            );
        }
    }

    /// A threshold that cannot mean anything must fail loudly rather than be
    /// ignored — the same rule `reject_unknown_flags` applies to the command
    /// line.
    #[test]
    #[should_panic(expected = "clip threshold must be finite and positive")]
    fn a_non_positive_threshold_panics() {
        let mut gradient = gradient(MatchedArm::FF_FIXED, |_| 1.0);
        clip_by_global_norm(&mut gradient, 0.0);
    }
}
