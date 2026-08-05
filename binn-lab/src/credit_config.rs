//! Credit-assignment repreregistration config and arm-specific hashing.
//!
//! These protocols are isolated from canonical C1 protocol v2.  They reuse its
//! gate thresholds but use fresh held-out seeds, distinct experiment names,
//! separate protocol versions, and `c1x-*` hash prefixes.
//!
//! Trial-isolation integrity (`c1x-iso-*`) is a **separate** protocol family:
//! it clears `ThreeFactor.last_spike` and applies C3-style full membrane reset
//! at trial boundaries. Frozen non-isolated `c1x-*` hashes remain byte-stable.
//!
//! Sparsity-calibrated isolation (`c1x-iso-s-*`) keeps the same trial resets and
//! G2 thresholds, but selects k-WTA over all finite membranes (winner floor) so
//! measured activity stays in-band without dendrite residue. Prior `c1x-iso-*`
//! scientific runs remain historical INVALID_HARNESS evidence.

use crate::Config;

pub const CREDIT_MATCHED_PROTOCOL_VERSION: u64 = 4;
pub const CREDIT_RPE_PROTOCOL_VERSION: u64 = 5;
pub const CREDIT_EPROP_PROTOCOL_VERSION: u64 = 6;
pub const CREDIT_DFA_PROTOCOL_VERSION: u64 = 7;

/// Added to each arm's protocol version when [`CreditConfig::trial_isolation`]
/// is enabled (matched→14, RPE→15, e-prop→16, DFA→17).
pub const CREDIT_ISOLATION_PROTOCOL_OFFSET: u64 = 10;

/// Added to each arm's protocol version for sparsity-calibrated isolation
/// (matched→24, RPE→25, e-prop→26, DFA→27).
pub const CREDIT_ISOLATION_CALIBRATED_PROTOCOL_OFFSET: u64 = 20;

/// Prefix shared by non-isolated credit-repreregistration hashes.
pub const CREDIT_HASH_PREFIX: &str = "c1x-";

/// Prefix for trial-isolation exact-forward credit hashes.
pub const CREDIT_ISOLATION_HASH_PREFIX: &str = "c1x-iso-";

/// Prefix for sparsity-calibrated trial-isolation hashes (distinct from `c1x-iso-`).
pub const CREDIT_ISOLATION_CALIBRATED_HASH_PREFIX: &str = "c1x-iso-s-";

/// Experiment-name prefix that marks a credit trial-isolation preset.
pub const CREDIT_ISOLATION_EXPERIMENT_PREFIX: &str = "c1x-iso";

/// Experiment-name prefix for sparsity-calibrated isolation presets.
pub const CREDIT_ISOLATION_CALIBRATED_EXPERIMENT_PREFIX: &str = "c1x-iso-s";

/// Learning conditions in the exact-forward C1 suite.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CreditArm {
    /// Canonical broadcast rule with one online pass (diagnostic).
    BroadcastOnePass,
    /// Canonical broadcast rule with the matched exposure count.
    BroadcastEpochMatched,
    /// Epoch-matched broadcast rule with a causal running-mean RPE.
    RpeThreeFactor,
    /// Postsynaptic, output-weight-derived e-prop-style learning signal.
    EpropExactForward,
    /// Deterministic fixed-random direct feedback alignment.
    DfaExactForward,
    /// Same-forward-graph straight-through surrogate-gradient reference.
    SurrogateGradient,
    /// Dense-topology epoch-matched broadcast control.
    DenseEpochMatched,
}

impl CreditArm {
    pub const ALL: [Self; 7] = [
        Self::BroadcastOnePass,
        Self::BroadcastEpochMatched,
        Self::RpeThreeFactor,
        Self::EpropExactForward,
        Self::DfaExactForward,
        Self::SurrogateGradient,
        Self::DenseEpochMatched,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::BroadcastOnePass => "broadcast-one-pass",
            Self::BroadcastEpochMatched => "broadcast-epoch-matched",
            Self::RpeThreeFactor => "rpe-three-factor",
            Self::EpropExactForward => "eprop-exact-forward",
            Self::DfaExactForward => "dfa-exact-forward",
            Self::SurrogateGradient => "surrogate-gradient-exact-forward",
            Self::DenseEpochMatched => "dense-epoch-matched",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|arm| arm.as_str() == value)
    }

    /// Preregistered protocol version for this mechanism (non-isolated).
    pub fn protocol_version(self) -> u64 {
        match self {
            Self::BroadcastOnePass
            | Self::BroadcastEpochMatched
            | Self::SurrogateGradient
            | Self::DenseEpochMatched => CREDIT_MATCHED_PROTOCOL_VERSION,
            Self::RpeThreeFactor => CREDIT_RPE_PROTOCOL_VERSION,
            Self::EpropExactForward => CREDIT_EPROP_PROTOCOL_VERSION,
            Self::DfaExactForward => CREDIT_DFA_PROTOCOL_VERSION,
        }
    }
}

/// Public config for the exact-forward C1 repreregistration.
#[derive(Clone, Debug, PartialEq)]
pub struct CreditConfig {
    /// C1 substrate/task knobs and unchanged G2 thresholds.
    pub base: Config,
    /// Number of identical-order training exposures for matched arms.
    pub matched_epochs: usize,
    /// Straight-through surrogate steepness.
    pub surrogate_beta: f32,
    /// Minimum scientific seed count.
    pub scientific_n_seeds: usize,
    /// Development-only schedule marker.
    pub quick: bool,
    /// When true, clear STDP pairing + full membrane reset at trial boundaries
    /// and mint isolation-family hashes (does not alter frozen `c1x-*`).
    pub trial_isolation: bool,
    /// When true, k-WTA scores all finite membranes (winner floor) instead of
    /// requiring `v > 0`. Used by sparsity-calibrated isolation only.
    pub kwta_all_finite: bool,
}

impl CreditConfig {
    /// Full held-out schedule. Seeds do not overlap canonical C1 or quick pilots.
    pub fn scientific() -> Self {
        let mut base = Config::c1_default();
        base.experiment = "c1-credit-reprereg".into();
        base.master_seed = 0xC1C4_5C1E_0001;
        base.n_seeds = 20;
        base.quick = false;
        base.matched_budget_repeat = false;
        Self {
            matched_epochs: base.bptt_epochs,
            surrogate_beta: base.surrogate_beta,
            scientific_n_seeds: 20,
            quick: false,
            trial_isolation: false,
            kwta_all_finite: false,
            base,
        }
    }

    /// Development/PILOT schedule with a disjoint seed lineage.
    pub fn quick() -> Self {
        let mut c = Self::scientific();
        c.base.experiment = "c1-credit-reprereg-quick".into();
        c.base.master_seed = 0xC1D3_5EED_0001;
        c.base.n_seeds = 5;
        c.base.n_train = 24;
        c.base.n_test = 16;
        c.base.n_hidden = 64;
        c.base.k_wta = 1;
        c.base.quick = true;
        c.matched_epochs = 4;
        c.quick = true;
        c
    }

    /// Scientific schedule with trial isolation (new `c1x-iso-*` hashes).
    ///
    /// Same G2 thresholds / substrate knobs as [`Self::scientific`], but clears
    /// `ThreeFactor.last_spike` and applies C3-style full membrane reset at
    /// every trial boundary. Fresh experiment name + seed lineage.
    pub fn scientific_isolation() -> Self {
        let mut c = Self::scientific();
        c.base.experiment = format!("{CREDIT_ISOLATION_EXPERIMENT_PREFIX}-credit-reprereg");
        c.base.master_seed = 0xC1C4_1500_0001;
        c.trial_isolation = true;
        c
    }

    /// Quick/PILOT schedule for the credit trial-isolation protocol.
    pub fn quick_isolation() -> Self {
        let mut c = Self::quick();
        c.base.experiment = format!("{CREDIT_ISOLATION_EXPERIMENT_PREFIX}-credit-reprereg-quick");
        c.base.master_seed = 0xC1D3_1500_0001;
        c.trial_isolation = true;
        c
    }

    /// Scientific trial-isolation with sparsity calibration (`c1x-iso-s-*`).
    ///
    /// Keeps pairing + membrane isolation and unchanged G2 / sparsity-band
    /// thresholds. Enables winner-floor k-WTA (`kwta_all_finite`) so activity
    /// does not depend on cross-trial dendrite residue after full reset.
    pub fn scientific_isolation_calibrated() -> Self {
        let mut c = Self::scientific_isolation();
        c.base.experiment =
            format!("{CREDIT_ISOLATION_CALIBRATED_EXPERIMENT_PREFIX}-credit-reprereg");
        c.base.master_seed = 0xC1C4_1510_0001;
        c.kwta_all_finite = true;
        c
    }

    /// Quick/PILOT schedule for sparsity-calibrated isolation.
    pub fn quick_isolation_calibrated() -> Self {
        let mut c = Self::quick_isolation();
        c.base.experiment =
            format!("{CREDIT_ISOLATION_CALIBRATED_EXPERIMENT_PREFIX}-credit-reprereg-quick");
        c.base.master_seed = 0xC1D3_1510_0001;
        c.kwta_all_finite = true;
        c
    }

    pub fn known_presets() -> Vec<Self> {
        vec![
            Self::scientific(),
            Self::quick(),
            Self::scientific_isolation(),
            Self::quick_isolation(),
            Self::scientific_isolation_calibrated(),
            Self::quick_isolation_calibrated(),
        ]
    }

    /// True when this preset applies trial-isolation resets on the exact-forward path.
    #[inline]
    pub fn is_isolation_protocol(&self) -> bool {
        self.trial_isolation
            || self
                .base
                .experiment
                .starts_with(CREDIT_ISOLATION_EXPERIMENT_PREFIX)
            || self.base.is_isolation_protocol()
    }

    /// True when this preset is the sparsity-calibrated isolation family.
    #[inline]
    pub fn is_isolation_calibrated_protocol(&self) -> bool {
        self.kwta_all_finite
            || self
                .base
                .experiment
                .starts_with(CREDIT_ISOLATION_CALIBRATED_EXPERIMENT_PREFIX)
    }

    /// Effective protocol version mixed into hashes / reports for `arm`.
    #[inline]
    pub fn protocol_version_for(&self, arm: CreditArm) -> u64 {
        let base = arm.protocol_version();
        if self.is_isolation_calibrated_protocol() {
            base + CREDIT_ISOLATION_CALIBRATED_PROTOCOL_OFFSET
        } else if self.is_isolation_protocol() {
            base + CREDIT_ISOLATION_PROTOCOL_OFFSET
        } else {
            base
        }
    }

    /// Hash prefix for this preset (`c1x-`, `c1x-iso-`, or `c1x-iso-s-`).
    #[inline]
    pub fn hash_prefix(&self) -> &'static str {
        if self.is_isolation_calibrated_protocol() {
            CREDIT_ISOLATION_CALIBRATED_HASH_PREFIX
        } else if self.is_isolation_protocol() {
            CREDIT_ISOLATION_HASH_PREFIX
        } else {
            CREDIT_HASH_PREFIX
        }
    }

    /// Arm-specific stable hash over all public scientific knobs.
    pub fn hash_for_arm(&self, arm: CreditArm) -> u64 {
        let mut h = 0xcbf2_9ce4_8422_2325_u64;
        fn mix(h: &mut u64, word: u64) {
            *h ^= word;
            *h = (*h).wrapping_mul(0x0100_0000_01b3);
        }
        mix(&mut h, self.protocol_version_for(arm));
        for b in arm.as_str().as_bytes() {
            mix(&mut h, *b as u64);
        }
        // `Config::hash` covers every public C1 knob.  The outer version and
        // arm label prevent aliasing with C1 protocol v2/v3.
        mix(&mut h, self.base.hash());
        mix(&mut h, self.matched_epochs as u64);
        mix(&mut h, self.surrogate_beta.to_bits() as u64);
        mix(&mut h, self.scientific_n_seeds as u64);
        mix(&mut h, u64::from(self.quick));
        // Calibrated knobs are already mixed via experiment name, master_seed,
        // and protocol offset (+20). Do not append extra flags here — that would
        // byte-shift frozen `c1x-*` / `c1x-iso-*` hashes.
        h
    }

    pub fn hash_string_for_arm(&self, arm: CreditArm) -> String {
        format!(
            "{}{}-{:016x}",
            self.hash_prefix(),
            arm.as_str(),
            self.hash_for_arm(arm)
        )
    }

    /// Resolve any known arm hash to its preset and arm.
    pub fn from_hash(hash: &str) -> Option<(Self, CreditArm)> {
        for preset in Self::known_presets() {
            for arm in CreditArm::ALL {
                if hash
                    .trim()
                    .eq_ignore_ascii_case(&preset.hash_string_for_arm(arm))
                {
                    return Some((preset, arm));
                }
            }
        }
        None
    }

    /// Fresh deterministic seeds for this suite.
    #[inline]
    pub fn seeds(&self) -> Vec<u64> {
        self.base.seeds()
    }

    /// Matched arms see this many epochs; one-pass remains exactly one.
    #[inline]
    pub fn epochs_for(&self, arm: CreditArm) -> usize {
        if arm == CreditArm::BroadcastOnePass {
            1
        } else {
            self.matched_epochs
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::C1_PROTOCOL_VERSION;

    #[test]
    fn canonical_hash_and_thresholds_are_unchanged() {
        let canonical = Config::c1_default();
        let credit = CreditConfig::scientific();
        assert_eq!(canonical.hash_string(), "c1-118207fbc3eaba53");
        assert_eq!(C1_PROTOCOL_VERSION, 2);
        assert!(!credit.is_isolation_protocol());
        assert_eq!(credit.base.g2_min_gap_closed, canonical.g2_min_gap_closed);
        assert_eq!(credit.base.g2_min_accuracy, canonical.g2_min_accuracy);
        assert_eq!(
            credit.base.g2_min_positive_control,
            canonical.g2_min_positive_control
        );
        assert_eq!(
            credit.base.activity_sparsity_min,
            canonical.activity_sparsity_min
        );
        assert_eq!(
            credit.base.activity_sparsity_max,
            canonical.activity_sparsity_max
        );
    }

    #[test]
    fn frozen_c1x_scientific_hashes_remain_byte_stable() {
        let c = CreditConfig::scientific();
        assert_eq!(
            c.hash_string_for_arm(CreditArm::BroadcastOnePass),
            "c1x-broadcast-one-pass-ec3c5a4d19ccd57e"
        );
        assert_eq!(
            c.hash_string_for_arm(CreditArm::BroadcastEpochMatched),
            "c1x-broadcast-epoch-matched-911a03a2a45feaf2"
        );
        assert_eq!(
            c.hash_string_for_arm(CreditArm::RpeThreeFactor),
            "c1x-rpe-three-factor-872e9eda9303f5df"
        );
        assert_eq!(
            c.hash_string_for_arm(CreditArm::EpropExactForward),
            "c1x-eprop-exact-forward-fcedc76a80ff0f0e"
        );
        assert_eq!(
            c.hash_string_for_arm(CreditArm::DfaExactForward),
            "c1x-dfa-exact-forward-4a1601e725edbc80"
        );
        assert_eq!(
            c.hash_string_for_arm(CreditArm::SurrogateGradient),
            "c1x-surrogate-gradient-exact-forward-cfe9a2c8d3e22257"
        );
        assert_eq!(
            c.hash_string_for_arm(CreditArm::DenseEpochMatched),
            "c1x-dense-epoch-matched-1387104803fe7e0a"
        );
    }

    #[test]
    fn mechanisms_have_separate_versions_and_hashes() {
        let c = CreditConfig::scientific();
        let matched = c.hash_string_for_arm(CreditArm::BroadcastEpochMatched);
        let rpe = c.hash_string_for_arm(CreditArm::RpeThreeFactor);
        let eprop = c.hash_string_for_arm(CreditArm::EpropExactForward);
        let dfa = c.hash_string_for_arm(CreditArm::DfaExactForward);
        assert_ne!(matched, rpe);
        assert_ne!(rpe, eprop);
        assert_ne!(eprop, dfa);
        assert_eq!(
            CreditArm::RpeThreeFactor.protocol_version(),
            CREDIT_RPE_PROTOCOL_VERSION
        );
        assert_eq!(
            CreditArm::EpropExactForward.protocol_version(),
            CREDIT_EPROP_PROTOCOL_VERSION
        );
        assert_eq!(
            CreditArm::DfaExactForward.protocol_version(),
            CREDIT_DFA_PROTOCOL_VERSION
        );
    }

    #[test]
    fn isolation_presets_diverge_from_frozen_c1x_and_use_offset_versions() {
        let frozen = CreditConfig::scientific();
        let iso = CreditConfig::scientific_isolation();
        let iso_q = CreditConfig::quick_isolation();
        assert!(iso.is_isolation_protocol());
        assert!(iso_q.is_isolation_protocol());
        assert!(!frozen.is_isolation_protocol());
        assert_eq!(iso.base.g2_min_gap_closed, frozen.base.g2_min_gap_closed);
        assert_eq!(iso.base.g2_min_accuracy, frozen.base.g2_min_accuracy);
        for arm in CreditArm::ALL {
            let frozen_hash = frozen.hash_string_for_arm(arm);
            let iso_hash = iso.hash_string_for_arm(arm);
            assert!(
                frozen_hash.starts_with(CREDIT_HASH_PREFIX),
                "frozen hash must keep c1x- prefix: {frozen_hash}"
            );
            assert!(
                !frozen_hash.starts_with(CREDIT_ISOLATION_HASH_PREFIX),
                "frozen hash must not use c1x-iso- prefix: {frozen_hash}"
            );
            assert!(
                iso_hash.starts_with(CREDIT_ISOLATION_HASH_PREFIX),
                "isolation hash must use c1x-iso- prefix: {iso_hash}"
            );
            assert!(
                !iso_hash.starts_with(CREDIT_ISOLATION_CALIBRATED_HASH_PREFIX),
                "legacy isolation must not use c1x-iso-s- prefix: {iso_hash}"
            );
            assert_ne!(frozen_hash, iso_hash);
            assert_eq!(
                iso.protocol_version_for(arm),
                arm.protocol_version() + CREDIT_ISOLATION_PROTOCOL_OFFSET
            );
            assert_eq!(frozen.protocol_version_for(arm), arm.protocol_version());
        }
        assert_ne!(
            iso.hash_string_for_arm(CreditArm::BroadcastEpochMatched),
            iso_q.hash_string_for_arm(CreditArm::BroadcastEpochMatched)
        );
    }

    #[test]
    fn calibrated_isolation_diverges_from_frozen_and_prior_iso() {
        let frozen = CreditConfig::scientific();
        let iso = CreditConfig::scientific_isolation();
        let cal = CreditConfig::scientific_isolation_calibrated();
        let cal_q = CreditConfig::quick_isolation_calibrated();
        assert!(cal.is_isolation_protocol());
        assert!(cal.is_isolation_calibrated_protocol());
        assert!(!iso.is_isolation_calibrated_protocol());
        assert!(cal.kwta_all_finite);
        assert!(!iso.kwta_all_finite);
        assert_eq!(cal.base.g2_min_gap_closed, frozen.base.g2_min_gap_closed);
        assert_eq!(cal.base.g2_min_accuracy, frozen.base.g2_min_accuracy);
        assert_eq!(
            cal.base.activity_sparsity_min,
            frozen.base.activity_sparsity_min
        );
        assert_eq!(
            cal.base.activity_sparsity_max,
            frozen.base.activity_sparsity_max
        );
        // Historical INVALID_HARNESS scientific iso hashes stay pinned.
        assert_eq!(
            iso.hash_string_for_arm(CreditArm::BroadcastEpochMatched),
            "c1x-iso-broadcast-epoch-matched-7becb435b63868c6"
        );
        // Calibrated scientific hashes (winner-floor isolation).
        assert_eq!(
            cal.hash_string_for_arm(CreditArm::BroadcastOnePass),
            "c1x-iso-s-broadcast-one-pass-6abe723b6700113c"
        );
        assert_eq!(
            cal.hash_string_for_arm(CreditArm::BroadcastEpochMatched),
            "c1x-iso-s-broadcast-epoch-matched-4e3236f8f60433d0"
        );
        assert_eq!(
            cal.hash_string_for_arm(CreditArm::RpeThreeFactor),
            "c1x-iso-s-rpe-three-factor-e1fd914d40873269"
        );
        assert_eq!(
            cal.hash_string_for_arm(CreditArm::EpropExactForward),
            "c1x-iso-s-eprop-exact-forward-552924e96f2dded4"
        );
        assert_eq!(
            cal.hash_string_for_arm(CreditArm::DfaExactForward),
            "c1x-iso-s-dfa-exact-forward-d2c8d3c929a68bd2"
        );
        assert_eq!(
            cal.hash_string_for_arm(CreditArm::SurrogateGradient),
            "c1x-iso-s-surrogate-gradient-exact-forward-75f280fac365d671"
        );
        assert_eq!(
            cal.hash_string_for_arm(CreditArm::DenseEpochMatched),
            "c1x-iso-s-dense-epoch-matched-1f81769d0d7623b0"
        );
        for arm in CreditArm::ALL {
            let cal_hash = cal.hash_string_for_arm(arm);
            assert!(
                cal_hash.starts_with(CREDIT_ISOLATION_CALIBRATED_HASH_PREFIX),
                "calibrated hash must use c1x-iso-s- prefix: {cal_hash}"
            );
            assert_ne!(cal_hash, frozen.hash_string_for_arm(arm));
            assert_ne!(cal_hash, iso.hash_string_for_arm(arm));
            assert_eq!(
                cal.protocol_version_for(arm),
                arm.protocol_version() + CREDIT_ISOLATION_CALIBRATED_PROTOCOL_OFFSET
            );
        }
        assert_ne!(
            cal.hash_string_for_arm(CreditArm::BroadcastEpochMatched),
            cal_q.hash_string_for_arm(CreditArm::BroadcastEpochMatched)
        );
    }

    #[test]
    fn hashes_round_trip_and_quick_seeds_are_disjoint() {
        let full = CreditConfig::scientific();
        let quick = CreditConfig::quick();
        assert!(full
            .seeds()
            .iter()
            .all(|seed| !quick.seeds().contains(seed)));
        let iso = CreditConfig::scientific_isolation();
        assert!(full.seeds().iter().all(|seed| !iso.seeds().contains(seed)));
        for preset in CreditConfig::known_presets() {
            for arm in CreditArm::ALL {
                let hash = preset.hash_string_for_arm(arm);
                let (decoded, decoded_arm) = CreditConfig::from_hash(&hash).unwrap();
                assert_eq!(decoded, preset);
                assert_eq!(decoded_arm, arm);
            }
        }
    }
}
