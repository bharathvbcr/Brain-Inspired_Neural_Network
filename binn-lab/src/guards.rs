//! Anti-degeneracy guards for experiment readouts and report verdicts.
//!
//! # Why this module exists
//!
//! The 2026-07-24 suite shipped three classes of silent failure that no test
//! caught:
//!
//! 1. **Constant predictors.** `c1_enhanced` and `multi_area_scaling` never fed
//!    the sample into the engine, so `predicted` was the same boolean for every
//!    trial and the reported "accuracy" was just the majority-class rate.
//! 2. **Hardcoded verdicts.** Four report generators emitted the literal string
//!    `PASS` in a markdown cell, so an arm could read `FAIL | PASS` on one row.
//! 3. **Unlabelled substrate.** A benchmark reported "GPU" numbers produced on
//!    the CPU.
//!
//! The fix for (1) and (2) lives here and is *mandatory*: any experiment that
//! reports an accuracy must build a [`ReadoutAudit`], and any experiment that
//! reports a verdict must obtain it from [`Verdict::evaluate`]. `(3)` is
//! enforced in `binn_core::metal_backend`.
//!
//! The companion test `binn-lab/tests/report_verdict_guard.rs` fails the build
//! if a verdict literal reappears in an experiment's report template.

use std::collections::BTreeSet;
use std::fmt;

/// Minimum evaluation-set size below which an accuracy is reported as
/// underpowered rather than as a result.
///
/// `multi_area_scaling` reported three-point "scaling" from 20-sample test
/// splits, where the 95% CI half-width is ±0.19.
pub const MIN_EVAL_SAMPLES: usize = 50;

/// Accuracy within this margin of the constant-predictor baseline is treated as
/// indistinguishable from predicting the majority class.
pub const CONSTANT_PREDICTOR_EPS: f32 = 1e-4;

/// z for a two-sided 95% interval.
pub const Z_95: f32 = 1.96;

/// Ways a reported accuracy can be an artifact rather than a measurement.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Degeneracy {
    /// Every prediction was identical: the readout ignores its input.
    ConstantPrediction,
    /// The pre-readout network state was identical for every sample: the
    /// stimulus never reached the network.
    StimulusNotDelivered,
    /// Accuracy equals the majority-class rate to within `CONSTANT_PREDICTOR_EPS`.
    EqualsMajorityClass,
    /// Fewer than [`MIN_EVAL_SAMPLES`] evaluation samples.
    UnderpoweredEvalSet,
    /// The evaluation set contains a single class, so accuracy is unidentifiable.
    SingleClassEvalSet,
}

impl Degeneracy {
    pub const fn explain(self) -> &'static str {
        match self {
            Degeneracy::ConstantPrediction => {
                "every prediction was identical — the readout does not depend on the sample"
            }
            Degeneracy::StimulusNotDelivered => {
                "pre-readout network state was identical across samples — the stimulus \
                 never reached the network"
            }
            Degeneracy::EqualsMajorityClass => {
                "accuracy equals the majority-class rate — this is class balance, not learning"
            }
            Degeneracy::UnderpoweredEvalSet => {
                "evaluation set is smaller than MIN_EVAL_SAMPLES; the confidence interval \
                 is wider than any effect being claimed"
            }
            Degeneracy::SingleClassEvalSet => {
                "evaluation set contains only one class — accuracy is unidentifiable"
            }
        }
    }

    /// Whether this defect invalidates the number outright (as opposed to
    /// merely weakening it).
    ///
    /// `EqualsMajorityClass` is deliberately non-fatal on its own: with a
    /// hundred-sample eval set a healthy readout can land exactly on the
    /// balance by coincidence. The genuinely broken case — predictions that are
    /// all the majority class — is caught by `ConstantPrediction`, which is
    /// fatal. `EqualsMajorityClass` remains reported so a reader can see it.
    pub const fn is_fatal(self) -> bool {
        match self {
            Degeneracy::ConstantPrediction
            | Degeneracy::StimulusNotDelivered
            | Degeneracy::SingleClassEvalSet => true,
            Degeneracy::EqualsMajorityClass | Degeneracy::UnderpoweredEvalSet => false,
        }
    }
}

impl fmt::Display for Degeneracy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.explain())
    }
}

/// Fingerprints of the pre-readout network state, one per evaluated sample.
///
/// This is the guard that directly catches "the sample was never fed in".
/// Experiments accumulate a fingerprint of whatever the readout reads
/// (membrane vector, winner set, score vector) and the audit checks that more
/// than one distinct state was observed.
#[derive(Clone, Debug, Default)]
pub struct StimulusProbe {
    fingerprints: BTreeSet<u64>,
    n_observed: usize,
}

impl StimulusProbe {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record the state the readout is about to consume.
    pub fn observe_f32(&mut self, state: &[f32]) {
        // FNV-1a over the bit patterns; NaN-insensitive by canonicalising.
        let mut hash = 0xcbf2_9ce4_8422_2325u64;
        for &v in state {
            let bits = if v.is_nan() { 0 } else { v.to_bits() } as u64;
            hash ^= bits;
            hash = hash.wrapping_mul(0x100_0000_01b3);
        }
        self.fingerprints.insert(hash);
        self.n_observed += 1;
    }

    /// Record a winner set / index set.
    pub fn observe_indices(&mut self, indices: &[u32]) {
        let mut hash = 0xcbf2_9ce4_8422_2325u64;
        for &i in indices {
            hash ^= i as u64;
            hash = hash.wrapping_mul(0x100_0000_01b3);
        }
        self.fingerprints.insert(hash);
        self.n_observed += 1;
    }

    pub fn n_distinct_states(&self) -> usize {
        self.fingerprints.len()
    }

    pub fn n_observed(&self) -> usize {
        self.n_observed
    }

    /// Fraction of samples that produced a distinct pre-readout state.
    pub fn state_diversity(&self) -> f32 {
        if self.n_observed == 0 {
            return 0.0;
        }
        self.fingerprints.len() as f32 / self.n_observed as f32
    }
}

/// Full audit of a binary readout arm.
///
/// Construct one for every accuracy an experiment reports. The `Display` impl
/// is safe to paste directly into a markdown report.
#[derive(Clone, Debug)]
pub struct ReadoutAudit {
    pub n: usize,
    pub n_correct: usize,
    pub accuracy: f32,
    /// Fraction of ground-truth labels equal to the majority class.
    pub majority_class_rate: f32,
    /// Best accuracy achievable by always predicting one constant.
    pub constant_predictor_accuracy: f32,
    pub n_predicted_true: usize,
    pub n_distinct_predictions: usize,
    /// Distinct pre-readout states, when a [`StimulusProbe`] was supplied.
    pub n_distinct_states: Option<usize>,
    /// Wilson 95% lower bound on accuracy.
    pub accuracy_lcb95: f32,
    /// Wilson 95% upper bound on accuracy.
    pub accuracy_ucb95: f32,
    pub defects: Vec<Degeneracy>,
}

impl ReadoutAudit {
    /// Audit predictions against ground truth.
    ///
    /// # Panics
    ///
    /// Panics if `predictions.len() != truths.len()`.
    pub fn new(predictions: &[bool], truths: &[bool], probe: Option<&StimulusProbe>) -> Self {
        assert_eq!(
            predictions.len(),
            truths.len(),
            "predictions and truths must be the same length"
        );
        let n = predictions.len();
        let n_correct = predictions
            .iter()
            .zip(truths.iter())
            .filter(|(p, t)| p == t)
            .count();
        let accuracy = if n == 0 {
            0.0
        } else {
            n_correct as f32 / n as f32
        };

        let n_true_labels = truths.iter().filter(|t| **t).count();
        let pos_rate = if n == 0 {
            0.0
        } else {
            n_true_labels as f32 / n as f32
        };
        let majority_class_rate = pos_rate.max(1.0 - pos_rate);
        // Always-true scores `pos_rate`; always-false scores `1 - pos_rate`.
        let constant_predictor_accuracy = majority_class_rate;

        let n_predicted_true = predictions.iter().filter(|p| **p).count();
        let n_distinct_predictions = if n == 0 {
            0
        } else if n_predicted_true == 0 || n_predicted_true == n {
            1
        } else {
            2
        };

        let (accuracy_lcb95, accuracy_ucb95) = wilson_interval(n_correct, n, Z_95);

        let mut defects = Vec::new();
        if n < MIN_EVAL_SAMPLES {
            defects.push(Degeneracy::UnderpoweredEvalSet);
        }
        if n > 0 && (n_true_labels == 0 || n_true_labels == n) {
            defects.push(Degeneracy::SingleClassEvalSet);
        }
        if n > 0 && n_distinct_predictions <= 1 {
            defects.push(Degeneracy::ConstantPrediction);
        }
        if n > 0 && (accuracy - constant_predictor_accuracy).abs() <= CONSTANT_PREDICTOR_EPS {
            defects.push(Degeneracy::EqualsMajorityClass);
        }
        let n_distinct_states = probe.map(|p| p.n_distinct_states());
        if let Some(states) = n_distinct_states {
            if n > 1 && states <= 1 {
                defects.push(Degeneracy::StimulusNotDelivered);
            }
        }
        defects.sort_unstable();
        defects.dedup();

        Self {
            n,
            n_correct,
            accuracy,
            majority_class_rate,
            constant_predictor_accuracy,
            n_predicted_true,
            n_distinct_predictions,
            n_distinct_states,
            accuracy_lcb95,
            accuracy_ucb95,
            defects,
        }
    }

    /// Any defect that invalidates the number outright.
    pub fn fatal_defects(&self) -> Vec<Degeneracy> {
        self.defects
            .iter()
            .copied()
            .filter(|d| d.is_fatal())
            .collect()
    }

    pub fn is_degenerate(&self) -> bool {
        !self.fatal_defects().is_empty()
    }

    /// Whether the accuracy is statistically distinguishable from the best
    /// constant predictor.
    pub fn beats_constant_predictor(&self) -> bool {
        self.accuracy_lcb95 > self.constant_predictor_accuracy
    }

    /// Abort the run rather than write a degenerate number into a report.
    ///
    /// # Panics
    ///
    /// Panics if any fatal defect is present.
    pub fn assert_non_degenerate(&self, arm: &str) {
        let fatal = self.fatal_defects();
        if !fatal.is_empty() {
            let details = fatal
                .iter()
                .map(|d| format!("  - {d:?}: {}", d.explain()))
                .collect::<Vec<_>>()
                .join("\n");
            panic!(
                "readout arm `{arm}` is degenerate; refusing to report accuracy \
                 {:.4}\n{details}\n  (n={}, distinct predictions={}, distinct states={:?}, \
                 majority-class rate={:.4})",
                self.accuracy,
                self.n,
                self.n_distinct_predictions,
                self.n_distinct_states,
                self.majority_class_rate
            );
        }
    }

    /// Markdown fragment for the report's diagnostics section.
    pub fn markdown_row(&self, arm: &str) -> String {
        format!(
            "| {arm} | {:.4} | [{:.4}, {:.4}] | {:.4} | {} | {} | {} | {} |",
            self.accuracy,
            self.accuracy_lcb95,
            self.accuracy_ucb95,
            self.constant_predictor_accuracy,
            self.n,
            self.n_distinct_predictions,
            self.n_distinct_states
                .map(|s| s.to_string())
                .unwrap_or_else(|| "n/a".to_string()),
            if self.defects.is_empty() {
                "none".to_string()
            } else {
                self.defects
                    .iter()
                    .map(|d| format!("{d:?}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            }
        )
    }

    /// Header matching [`ReadoutAudit::markdown_row`].
    pub fn markdown_header() -> &'static str {
        "| Arm | Accuracy | 95% CI | Constant-predictor baseline | n | Distinct predictions | Distinct states | Defects |\n\
         |---|---:|---|---:|---:|---:|---:|---|"
    }
}

impl fmt::Display for ReadoutAudit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "acc={:.4} (95% CI [{:.4}, {:.4}], n={}), constant-predictor={:.4}, \
             distinct predictions={}, distinct states={:?}, defects={:?}",
            self.accuracy,
            self.accuracy_lcb95,
            self.accuracy_ucb95,
            self.n,
            self.constant_predictor_accuracy,
            self.n_distinct_predictions,
            self.n_distinct_states,
            self.defects
        )
    }
}

/// Wilson score interval for a binomial proportion.
///
/// Used instead of the normal approximation because these experiments routinely
/// evaluate on tens of samples, where the normal interval is badly wrong.
pub fn wilson_interval(successes: usize, n: usize, z: f32) -> (f32, f32) {
    if n == 0 {
        return (0.0, 1.0);
    }
    let n_f = n as f32;
    let p = successes as f32 / n_f;
    let z2 = z * z;
    let denom = 1.0 + z2 / n_f;
    let centre = p + z2 / (2.0 * n_f);
    let margin = z * ((p * (1.0 - p) / n_f) + z2 / (4.0 * n_f * n_f)).sqrt();
    (
        ((centre - margin) / denom).clamp(0.0, 1.0),
        ((centre + margin) / denom).clamp(0.0, 1.0),
    )
}

/// The only legal way for a report generator to produce a verdict cell.
///
/// Constructing a `Verdict` requires supplying the measurement and the
/// preregistered threshold, so a verdict can never disagree with the number
/// printed next to it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    Pass,
    Fail,
    /// A reference arm, not a hypothesis under test.
    Ceiling,
    /// Fatal readout defect: no PASS/FAIL claim is permitted.
    Degenerate,
    /// Below the power requirement: pilot data only.
    Underpowered,
    /// Preregistered validity gate failed (positive control / sparsity band).
    InvalidHarness,
}

impl Verdict {
    /// Evaluate an arm against its preregistered floor.
    ///
    /// Degeneracy and harness invalidity dominate the threshold comparison, so
    /// a broken run cannot report PASS.
    pub fn evaluate(audit: &ReadoutAudit, floor: f32, harness_valid: bool) -> Self {
        if audit.is_degenerate() {
            return Verdict::Degenerate;
        }
        if !harness_valid {
            return Verdict::InvalidHarness;
        }
        if audit.defects.contains(&Degeneracy::UnderpoweredEvalSet) {
            return Verdict::Underpowered;
        }
        if audit.accuracy >= floor {
            Verdict::Pass
        } else {
            Verdict::Fail
        }
    }

    /// Evaluate a plain mean against a floor, for arms aggregated over seeds.
    ///
    /// `n_seeds` must meet `required_seeds` or the verdict is `Underpowered`.
    pub fn evaluate_mean(
        mean: f32,
        floor: f32,
        n_seeds: usize,
        required_seeds: usize,
        harness_valid: bool,
    ) -> Self {
        if !harness_valid {
            return Verdict::InvalidHarness;
        }
        if n_seeds < required_seeds {
            return Verdict::Underpowered;
        }
        if mean >= floor {
            Verdict::Pass
        } else {
            Verdict::Fail
        }
    }

    /// Markdown-safe cell text.
    pub const fn label(self) -> &'static str {
        match self {
            Verdict::Pass => "PASS",
            Verdict::Fail => "FAIL",
            Verdict::Ceiling => "CEILING",
            Verdict::Degenerate => "DEGENERATE",
            Verdict::Underpowered => "UNDERPOWERED",
            Verdict::InvalidHarness => "INVALID_HARNESS",
        }
    }

    /// Whether this verdict may be cited as positive evidence downstream.
    ///
    /// `INVALID_HARNESS` and `DEGENERATE` runs were cited as positive results in
    /// the 2026-07-24 summary. They may not be.
    pub const fn is_citable_as_positive(self) -> bool {
        matches!(self, Verdict::Pass)
    }
}

impl fmt::Display for Verdict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

/// Clamped, unit-normalised gradient-gap-closed.
///
/// `runner.rs` clamps to `[0, 1]`; `track_b_rescue` and `live_transfer_rescue`
/// did not, which is how gap-closed values of 1.0244 reached a report. All
/// harnesses must use this.
///
/// Returns `None` when the reference/dense separation is below
/// `min_reference_gap`, in which case gap-closed is not identifiable.
pub fn gap_closed_clamped(
    local: f32,
    dense: f32,
    reference: f32,
    min_reference_gap: f32,
) -> Option<f32> {
    let denom = reference - dense;
    if !denom.is_finite() || denom < min_reference_gap {
        return None;
    }
    Some(((local - dense) / denom).clamp(0.0, 1.0))
}

/// True when a raw (unclamped) gap-closed exceeds 1, i.e. the arm beat the
/// reference it is supposed to be bounded by.
///
/// This is a harness warning, not a result: it means the ceiling is undertrained
/// or the task is saturated.
pub fn gap_closed_exceeds_ceiling(local: f32, dense: f32, reference: f32) -> bool {
    let denom = reference - dense;
    denom > 0.0 && (local - dense) / denom > 1.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constant_predictor_is_flagged() {
        // The `c1_enhanced` bug: prediction is always true.
        let preds = vec![true; 20];
        let truths: Vec<bool> = (0..20)
            .map(|i| i % 20 != 0 && i % 20 != 3 && i % 20 != 7)
            .collect();
        let audit = ReadoutAudit::new(&preds, &truths, None);
        assert!(audit.defects.contains(&Degeneracy::ConstantPrediction));
        assert!(audit.defects.contains(&Degeneracy::EqualsMajorityClass));
        assert!(audit.is_degenerate());
        assert_eq!(
            Verdict::evaluate(&audit, 0.65, true),
            Verdict::Degenerate,
            "a constant predictor must never report PASS"
        );
    }

    #[test]
    fn stimulus_not_delivered_is_flagged() {
        let mut probe = StimulusProbe::new();
        // Same pre-readout state every sample: engine was reset and never driven.
        for _ in 0..20 {
            probe.observe_f32(&[0.0, 0.0, 0.0, 0.0]);
        }
        let preds: Vec<bool> = (0..20).map(|i| i % 2 == 0).collect();
        let truths: Vec<bool> = (0..20).map(|i| i % 3 == 0).collect();
        let audit = ReadoutAudit::new(&preds, &truths, Some(&probe));
        assert!(audit.defects.contains(&Degeneracy::StimulusNotDelivered));
        assert_eq!(audit.n_distinct_states, Some(1));
    }

    #[test]
    fn healthy_readout_passes() {
        let truths: Vec<bool> = (0..200).map(|i| i % 2 == 0).collect();
        // 90% accurate, varies with the sample.
        let preds: Vec<bool> = truths
            .iter()
            .enumerate()
            .map(|(i, t)| if i % 10 == 0 { !t } else { *t })
            .collect();
        let mut probe = StimulusProbe::new();
        for i in 0..200 {
            probe.observe_f32(&[i as f32, (i * 7 % 13) as f32]);
        }
        let audit = ReadoutAudit::new(&preds, &truths, Some(&probe));
        assert!(!audit.is_degenerate(), "{audit}");
        assert!(audit.beats_constant_predictor());
        assert_eq!(Verdict::evaluate(&audit, 0.65, true), Verdict::Pass);
        audit.assert_non_degenerate("healthy");
    }

    #[test]
    fn underpowered_eval_set_cannot_pass() {
        // The `multi_area_scaling` bug: 20-sample test split.
        let truths: Vec<bool> = (0..20).map(|i| i % 3 != 0).collect();
        let preds: Vec<bool> = truths
            .iter()
            .enumerate()
            .map(|(i, t)| if i == 0 { !t } else { *t })
            .collect();
        let audit = ReadoutAudit::new(&preds, &truths, None);
        assert!(audit.defects.contains(&Degeneracy::UnderpoweredEvalSet));
        assert_eq!(Verdict::evaluate(&audit, 0.65, true), Verdict::Underpowered);
    }

    #[test]
    fn invalid_harness_dominates_threshold() {
        let truths: Vec<bool> = (0..200).map(|i| i % 2 == 0).collect();
        let preds = truths.clone();
        let audit = ReadoutAudit::new(&preds, &truths, None);
        let v = Verdict::evaluate(&audit, 0.65, false);
        assert_eq!(v, Verdict::InvalidHarness);
        assert!(!v.is_citable_as_positive());
    }

    #[test]
    fn only_pass_is_citable() {
        for v in [
            Verdict::Fail,
            Verdict::Ceiling,
            Verdict::Degenerate,
            Verdict::Underpowered,
            Verdict::InvalidHarness,
        ] {
            assert!(!v.is_citable_as_positive(), "{v} must not be citable");
        }
        assert!(Verdict::Pass.is_citable_as_positive());
    }

    #[test]
    fn wilson_matches_known_values() {
        // 15/20 -> approx [0.5313, 0.8881]
        let (lo, hi) = wilson_interval(15, 20, Z_95);
        assert!((lo - 0.5313).abs() < 0.005, "lo={lo}");
        assert!((hi - 0.8881).abs() < 0.005, "hi={hi}");
        // Degenerate n
        let (lo0, hi0) = wilson_interval(0, 0, Z_95);
        assert_eq!((lo0, hi0), (0.0, 1.0));
    }

    #[test]
    fn gap_closed_is_clamped_and_gated() {
        // The 1.0244 case: local beats the reference.
        assert_eq!(
            gap_closed_clamped(1.0, 0.541, 0.9895, 0.15),
            Some(1.0),
            "gap-closed must never exceed 1"
        );
        assert!(gap_closed_exceeds_ceiling(1.0, 0.541, 0.9895));
        // Reference barely separated from dense -> unidentifiable.
        assert_eq!(gap_closed_clamped(0.8, 0.70, 0.75, 0.15), None);
        // Normal case.
        let g = gap_closed_clamped(0.7, 0.5, 0.9, 0.15).unwrap();
        assert!((g - 0.5).abs() < 1e-6);
    }

    #[test]
    fn probe_counts_distinct_states() {
        let mut p = StimulusProbe::new();
        p.observe_indices(&[1, 2, 3]);
        p.observe_indices(&[1, 2, 3]);
        p.observe_indices(&[4, 5]);
        assert_eq!(p.n_observed(), 3);
        assert_eq!(p.n_distinct_states(), 2);
    }
}
